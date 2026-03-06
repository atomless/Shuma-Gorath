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
        self.captured_manifest = self.temp_dir / "captured-spin.toml"
        self.catalog_path = self.temp_dir / "catalog.json"
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
                    payload = {"id": 123, "status": "running", "ipv4": ["198.51.100.24"]}
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
                for arg in "$@"; do
                  if [ "$arg" = "dashboard-build" ]; then
                    mkdir -p dist/dashboard
                    printf '<h1>Dashboard</h1>\\n' > dist/dashboard/index.html
                  fi
                done
                if [ -n "$SHUMA_SPIN_MANIFEST" ] && [ -f "$SHUMA_SPIN_MANIFEST" ]; then
                  cp "$SHUMA_SPIN_MANIFEST" "{self.captured_manifest}"
                fi
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
            "--domain",
            "shuma.example.com",
            "--existing-instance-id",
            "123",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        curl_log = self.curl_log.read_text(encoding="utf-8")
        self.assertNotIn("POST https://api.linode.com/v4/linode/instances\n", curl_log)
        self.assertIn("Host IP:   198.51.100.24", result.stdout)

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
        self.assertIn('printf \'\\n\' >> .env.local', script)
        self.assertIn('cat "${ENV_FILE_PATH}" >> .env.local', script)
        self.assertNotIn('cp "${ENV_FILE_PATH}" .env.local', script)


if __name__ == "__main__":
    unittest.main()
