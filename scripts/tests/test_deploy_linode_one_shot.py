import json
import os
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path
from typing import Dict, Optional


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "deploy_linode_one_shot.sh"
MAKEFILE = REPO_ROOT / "Makefile"


def write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class DeployLinodeOneShotTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="deploy-linode-test-"))
        self.home = self.temp_dir / "home"
        self.home.mkdir()
        ssh_dir = self.home / ".ssh"
        ssh_dir.mkdir()
        (ssh_dir / "id_ed25519").write_text("private\n", encoding="utf-8")
        (ssh_dir / "id_ed25519.pub").write_text(
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITest deploy@test\n",
            encoding="utf-8",
        )
        self.stub_dir = self.temp_dir / "bin"
        self.stub_dir.mkdir()
        self.make_log = self.temp_dir / "make.log"
        self.curl_log = self.temp_dir / "curl.log"
        self.open_log = self.temp_dir / "open.log"
        self.captured_manifest = self.temp_dir / "captured-spin.toml"
        self.catalog_path = self.temp_dir / "catalog.json"
        self.remote_receipts_dir = self.temp_dir / ".shuma" / "remotes"
        self.env_file = self.temp_dir / ".env.local"
        self.catalog_path.write_text('{"inventory":[{"path":"/"}]}\n', encoding="utf-8")

        write_executable(
            self.stub_dir / "curl",
            textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import json
                import os
                import sys
                from pathlib import Path

                output_path = None
                url = ""
                method = "GET"
                args = sys.argv[1:]
                i = 0
                while i < len(args):
                    if args[i] == "-o":
                        output_path = args[i + 1]
                        i += 2
                        continue
                    if args[i] == "-X":
                        method = args[i + 1]
                        i += 2
                        continue
                    if not args[i].startswith("-"):
                        url = args[i]
                    i += 1

                with open(os.environ["CURL_LOG"], "a", encoding="utf-8") as handle:
                    handle.write(f"{method} {url}\\n")

                if "/regions" in url:
                    payload = {"data": [{"id": "gb-lon"}]}
                elif "/linode/types" in url:
                    payload = {"data": [{"id": "g6-standard-1"}]}
                elif "/images" in url:
                    payload = {"data": [{"id": "linode/ubuntu24.04"}]}
                elif "/linode/instances/123" in url:
                    payload = {
                        "id": 123,
                        "label": "existing-shuma-host",
                        "status": "running",
                        "ipv4": ["198.51.100.24"],
                        "region": "gb-lon",
                        "type": "g6-standard-1",
                        "image": "linode/ubuntu24.04",
                    }
                else:
                    payload = {"data": []}

                if output_path:
                    Path(output_path).write_text(json.dumps(payload), encoding="utf-8")
                print("200", end="")
                """
            ),
        )
        write_executable(
            self.stub_dir / "make",
            textwrap.dedent(
                f"""\
                #!/bin/sh
                printf '%s\\n' "$@" >> "{self.make_log}"
                manifest_path="$SHUMA_SPIN_MANIFEST"
                for arg in "$@"; do
                  case "$arg" in
                    SHUMA_SPIN_MANIFEST=*)
                      manifest_path="${{arg#SHUMA_SPIN_MANIFEST=}}"
                      ;;
                  esac
                done
                for arg in "$@"; do
                  if [ "$arg" = "dashboard-build" ]; then
                    mkdir -p dist/dashboard
                    printf '<h1>Dashboard</h1>\\n' > dist/dashboard/index.html
                  fi
                done
                if [ -n "$manifest_path" ] && [ -f "$manifest_path" ]; then
                  cp "$manifest_path" "{self.captured_manifest}"
                fi
                exit 0
                """
            ),
        )
        write_executable(
            self.stub_dir / "open",
            textwrap.dedent(
                f"""\
                #!/bin/sh
                printf '%s\\n' "$@" >> "{self.open_log}"
                exit 0
                """
            ),
        )
        write_executable(
            self.stub_dir / "xdg-open",
            textwrap.dedent(
                f"""\
                #!/bin/sh
                printf '%s\\n' "$@" >> "{self.open_log}"
                exit 0
                """
            ),
        )
        write_executable(self.stub_dir / "ssh", "#!/bin/sh\nexit 0\n")
        write_executable(self.stub_dir / "scp", "#!/bin/sh\nexit 0\n")

    def env(self) -> dict[str, str]:
        env = os.environ.copy()
        env["HOME"] = str(self.home)
        env["PATH"] = f"{self.stub_dir}:{env['PATH']}"
        env["CURL_LOG"] = str(self.curl_log)
        env["LINODE_TOKEN"] = "token"
        env["SHUMA_ADMIN_IP_ALLOWLIST"] = "203.0.113.0/24"
        env["SHUMA_GATEWAY_UPSTREAM_ORIGIN"] = "https://origin.example.com"
        env["SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED"] = "true"
        env["SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED"] = "true"
        env["SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED"] = "true"
        env["SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED"] = "true"
        env["SHUMA_GATEWAY_TLS_STRICT"] = "true"
        env["GATEWAY_SURFACE_CATALOG_PATH"] = str(self.catalog_path)
        env["REMOTE_RECEIPTS_DIR"] = str(self.remote_receipts_dir)
        env["ENV_LOCAL"] = str(self.env_file)
        return env

    def run_script(
        self, *args: str, env_overrides: Optional[Dict[str, str]] = None
    ) -> subprocess.CompletedProcess:
        env = self.env()
        if env_overrides:
            env.update(env_overrides)
        return subprocess.run(
            ["bash", str(SCRIPT), *args],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_preflight_requires_domain_for_canonical_tls_path(self) -> None:
        result = self.run_script("--profile", "medium", "--region", "gb-lon", "--preflight-only")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("--domain is required", result.stderr + result.stdout)

    def test_preflight_requires_gateway_origin(self) -> None:
        result = self.run_script(
            "--domain",
            "shuma.example.com",
            "--profile",
            "medium",
            "--region",
            "gb-lon",
            "--preflight-only",
            env_overrides={"SHUMA_GATEWAY_UPSTREAM_ORIGIN": ""},
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("SHUMA_GATEWAY_UPSTREAM_ORIGIN", result.stderr + result.stdout)

    def test_preflight_runs_local_deploy_env_validate_before_provisioning(self) -> None:
        result = self.run_script(
            "--domain",
            "shuma.example.com",
            "--profile",
            "medium",
            "--region",
            "gb-lon",
            "--preflight-only",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue(self.make_log.exists())
        self.assertIn("deploy-env-validate", self.make_log.read_text(encoding="utf-8"))

    def test_preflight_passes_gateway_overrides_as_nested_make_arguments(self) -> None:
        result = self.run_script(
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
            "--preflight-only",
            env_overrides={
                "SHUMA_GATEWAY_UPSTREAM_ORIGIN": "http://127.0.0.1:8080",
                "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED": "true",
                "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED": "true",
                "SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED": "true",
                "SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED": "true",
            },
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        make_log = self.make_log.read_text(encoding="utf-8")
        self.assertIn('SHUMA_GATEWAY_UPSTREAM_ORIGIN=http://127.0.0.1:8080', make_log)
        self.assertIn('SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true', make_log)
        self.assertIn('SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true', make_log)
        self.assertIn('SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true', make_log)
        self.assertIn('SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true', make_log)

    def test_preflight_renders_gateway_manifest_for_validation(self) -> None:
        result = self.run_script(
            "--domain",
            "shuma.example.com",
            "--profile",
            "medium",
            "--region",
            "gb-lon",
            "--preflight-only",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue(self.captured_manifest.exists())
        self.assertIn(
            'allowed_outbound_hosts = ["https://origin.example.com:443"]',
            self.captured_manifest.read_text(encoding="utf-8"),
        )

    def test_existing_instance_preflight_skips_provisioning_shape_validation(self) -> None:
        result = self.run_script(
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
            "--preflight-only",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        curl_log = self.curl_log.read_text(encoding="utf-8")
        self.assertIn("/linode/instances/123", curl_log)
        self.assertNotIn("/regions", curl_log)
        self.assertNotIn("/linode/types", curl_log)

    def test_existing_instance_preflight_does_not_require_public_key_file(self) -> None:
        (self.home / ".ssh" / "id_ed25519.pub").unlink()
        result = self.run_script(
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
            "--preflight-only",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

    def test_existing_instance_uses_private_key_from_env_when_default_key_missing(self) -> None:
        private_key = self.temp_dir / "custom-shuma"
        private_key.write_text("private\n", encoding="utf-8")
        (self.home / ".ssh" / "id_ed25519").unlink()
        result = self.run_script(
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
            "--preflight-only",
            env_overrides={"SSH_PRIVATE_KEY_FILE": str(private_key)},
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

    def test_existing_instance_deploy_does_not_create_new_linode(self) -> None:
        result = self.run_script(
            "--remote-name",
            "blog-prod",
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        curl_log = self.curl_log.read_text(encoding="utf-8")
        self.assertNotIn("POST https://api.linode.com/v4/linode/instances\n", curl_log)
        self.assertIn("Host IP:   198.51.100.24", result.stdout)
        self.assertIn("Dashboard: https://shuma.example.com/dashboard", result.stdout)
        remote_receipt = json.loads(
            (self.remote_receipts_dir / "blog-prod.json").read_text(encoding="utf-8")
        )
        self.assertEqual(remote_receipt["schema"], "shuma.remote_target.v1")
        self.assertEqual(remote_receipt["identity"]["name"], "blog-prod")
        self.assertEqual(remote_receipt["runtime"]["public_base_url"], "https://shuma.example.com")
        self.assertTrue(remote_receipt["metadata"]["last_deployed_commit"])
        self.assertIn(
            "SHUMA_ACTIVE_REMOTE=blog-prod",
            self.env_file.read_text(encoding="utf-8"),
        )

    def test_existing_instance_deploy_writes_actual_instance_provider_metadata(self) -> None:
        result = self.run_script(
            "--remote-name",
            "blog-prod",
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        remote_receipt = json.loads(
            (self.remote_receipts_dir / "blog-prod.json").read_text(encoding="utf-8")
        )
        provider = remote_receipt["provider"]["linode"]
        self.assertEqual(provider["instance_id"], 123)
        self.assertEqual(provider["label"], "existing-shuma-host")
        self.assertEqual(provider["region"], "gb-lon")
        self.assertEqual(provider["type"], "g6-standard-1")
        self.assertEqual(provider["image"], "linode/ubuntu24.04")

    def test_existing_instance_deploy_persists_generated_operator_secrets_locally(self) -> None:
        result = self.run_script(
            "--remote-name",
            "blog-prod",
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        env_text = self.env_file.read_text(encoding="utf-8")
        self.assertRegex(env_text, r"SHUMA_API_KEY=[0-9a-f]{64}")
        self.assertRegex(env_text, r"SHUMA_JS_SECRET=[0-9a-f]{64}")
        self.assertRegex(env_text, r"SHUMA_FORWARDED_IP_SECRET=[0-9a-f]{64}")
        self.assertRegex(env_text, r"SHUMA_HEALTH_SECRET=[0-9a-f]{64}")
        self.assertRegex(env_text, r"SHUMA_SIM_TELEMETRY_SECRET=[0-9a-f]{64}")
        self.assertIn("SHUMA_ADMIN_IP_ALLOWLIST=203.0.113.0/24", env_text)

    def test_open_dashboard_flag_launches_local_dashboard_url(self) -> None:
        result = self.run_script(
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
            "--open-dashboard",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue(self.open_log.exists())
        self.assertIn("https://shuma.example.com/dashboard", self.open_log.read_text(encoding="utf-8"))

    def test_existing_instance_mode_rejects_profile_override(self) -> None:
        result = self.run_script(
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
            "--profile",
            "medium",
            "--preflight-only",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("--profile", result.stderr + result.stdout)

    def test_firewall_rules_do_not_use_force_for_allow(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")
        self.assertIn('sudo ufw allow OpenSSH', script)
        self.assertNotIn('sudo ufw --force allow OpenSSH', script)
        self.assertNotIn('sudo ufw --force allow 80/tcp', script)
        self.assertNotIn('sudo ufw --force allow 443/tcp', script)
        self.assertNotIn('sudo ufw --force allow 3000/tcp', script)

    def test_remote_bootstrap_merges_env_overlay_into_seeded_env_file(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")
        self.assertIn('make setup-runtime', script)
        self.assertIn(
            'python3 scripts/deploy/merge_env_overlay.py --overlay "${ENV_FILE_PATH}" --env-file ".env.local"',
            script,
        )
        self.assertNotIn('cp "${ENV_FILE_PATH}" .env.local', script)

    def test_remote_bootstrap_waits_for_spin_readiness_before_smoke(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")
        self.assertIn('REMOTE_SPIN_READY_TIMEOUT_SECONDS="${REMOTE_SPIN_READY_TIMEOUT_SECONDS:-300}"', script)
        self.assertIn(
            'SPIN_READY_TIMEOUT_SECONDS="${REMOTE_SPIN_READY_TIMEOUT_SECONDS}" make spin-wait-ready',
            script,
        )
        self.assertNotIn('curl -fsS -H "X-Shuma-Health-Secret:', script)

    def test_http_upstream_overlay_enables_insecure_local_gateway_flag(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")
        self.assertIn('if [[ "${SHUMA_GATEWAY_UPSTREAM_ORIGIN}" == http://* ]]; then', script)
        self.assertIn('SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL=${SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL_VALUE}', script)

    def test_remote_env_defaults_admin_config_writes_on_but_respects_override(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")
        self.assertIn('SHUMA_ADMIN_CONFIG_WRITE_ENABLED_VALUE="${SHUMA_ADMIN_CONFIG_WRITE_ENABLED:-true}"', script)
        self.assertIn('SHUMA_ADMIN_CONFIG_WRITE_ENABLED=${SHUMA_ADMIN_CONFIG_WRITE_ENABLED_VALUE}', script)
        self.assertNotIn('SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false', script)

    def test_remote_env_defaults_adversary_surface_on_but_respects_override(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")
        self.assertIn('SHUMA_ADVERSARY_SIM_AVAILABLE_VALUE="${SHUMA_ADVERSARY_SIM_AVAILABLE:-true}"', script)
        self.assertIn('SHUMA_ADVERSARY_SIM_AVAILABLE=${SHUMA_ADVERSARY_SIM_AVAILABLE_VALUE}', script)

    def test_remote_env_overlay_carries_monitoring_retention_overrides_when_set(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")
        self.assertIn('if [[ -n "${SHUMA_MONITORING_RETENTION_HOURS:-}" ]]; then', script)
        self.assertIn(
            'printf \'%s\\n\' "SHUMA_MONITORING_RETENTION_HOURS=${SHUMA_MONITORING_RETENTION_HOURS}" >>"${LOCAL_ENV_FILE}"',
            script,
        )
        self.assertIn('if [[ -n "${SHUMA_MONITORING_ROLLUP_RETENTION_HOURS:-}" ]]; then', script)
        self.assertIn(
            'printf \'%s\\n\' "SHUMA_MONITORING_ROLLUP_RETENTION_HOURS=${SHUMA_MONITORING_ROLLUP_RETENTION_HOURS}" >>"${LOCAL_ENV_FILE}"',
            script,
        )

    def test_makefile_defaults_prod_admin_config_writes_on_and_adversary_surface_available(self) -> None:
        makefile = MAKEFILE.read_text(encoding="utf-8")
        self.assertIn(
            "SHUMA_ADMIN_CONFIG_WRITE_ENABLED := $(if $(strip $(SHUMA_ADMIN_CONFIG_WRITE_ENABLED)),$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED),true)",
            makefile,
        )
        self.assertIn(
            "SHUMA_ADVERSARY_SIM_AVAILABLE := $(if $(strip $(SHUMA_ADVERSARY_SIM_AVAILABLE)),$(SHUMA_ADVERSARY_SIM_AVAILABLE),true)",
            makefile,
        )
        self.assertIn(
            'SHUMA_ADMIN_CONFIG_WRITE_ENABLED="$(DEPLOY_SHUMA_ADMIN_CONFIG_WRITE_ENABLED)"',
            makefile,
        )
        self.assertIn(
            'SHUMA_GATEWAY_UPSTREAM_ORIGIN="$(DEPLOY_SHUMA_GATEWAY_UPSTREAM_ORIGIN)"',
            makefile,
        )
        self.assertIn(
            "SPIN_PROD_OVERRIDES := --env SHUMA_DEBUG_HEADERS=false --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED) --env SHUMA_RUNTIME_ENV=runtime-prod --env SHUMA_ADVERSARY_SIM_AVAILABLE=$(SHUMA_ADVERSARY_SIM_AVAILABLE)",
            makefile,
        )

    def test_caddy_forwards_trusted_https_secret_header(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")
        self.assertIn('header_up X-Forwarded-Proto https', script)
        self.assertIn('header_up X-Shuma-Forwarded-Secret ${SHUMA_FORWARDED_IP_SECRET}', script)

    def test_remote_bootstrap_merges_env_overlay_without_duplicate_append(self) -> None:
        script = SCRIPT.read_text(encoding="utf-8")
        self.assertIn(
            'python3 scripts/deploy/merge_env_overlay.py --overlay "${ENV_FILE_PATH}" --env-file ".env.local"',
            script,
        )
        self.assertNotIn('cat "${ENV_FILE_PATH}" >> .env.local', script)


if __name__ == "__main__":
    unittest.main()
