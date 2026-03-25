import os
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


def write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class ProdStartSpinManifestTests(unittest.TestCase):
    def write_wrapper_stubs(self, stub_dir: Path) -> None:
        write_executable(stub_dir / "curl", "#!/bin/sh\nexit 0\n")
        write_executable(stub_dir / "sleep", "#!/bin/sh\nexit 0\n")

    def test_prod_start_honors_custom_spin_manifest_path(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="prod-start-manifest-"))
        stub_dir = temp_dir / "bin"
        stub_dir.mkdir()
        spin_log = temp_dir / "spin.log"
        custom_manifest = temp_dir / "spin.gateway.toml"
        config_db = temp_dir / "sqlite_key_value.db"
        custom_manifest.write_text("spin_manifest_version = 2\n", encoding="utf-8")

        write_executable(
            stub_dir / "spin",
            textwrap.dedent(
                f"""\
                #!/bin/sh
                printf '%s\\n' "$@" > "{spin_log}"
                exit 0
                """
            ),
        )
        write_executable(stub_dir / "pkill", "#!/bin/sh\nexit 0\n")
        write_executable(stub_dir / "uuidgen", "#!/bin/sh\necho test-runtime-id\n")
        self.write_wrapper_stubs(stub_dir)

        env = os.environ.copy()
        env["PATH"] = f"{stub_dir}:{env['PATH']}"
        env["SHUMA_API_KEY"] = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        env["SHUMA_FORWARDED_IP_SECRET"] = "forwarded-secret"
        env["SHUMA_SPIN_MANIFEST"] = str(custom_manifest)
        env["SHUMA_CONFIG_DB_PATH"] = str(config_db)

        seed_result = subprocess.run(
            ["bash", str(REPO_ROOT / "scripts" / "config_seed.sh")],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(seed_result.returncode, 0, msg=seed_result.stderr or seed_result.stdout)

        result = subprocess.run(
            ["make", "--no-print-directory", "prod-start"],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue(spin_log.exists())
        spin_args = spin_log.read_text(encoding="utf-8")
        self.assertIn("--from", spin_args)
        self.assertIn(str(custom_manifest), spin_args)

    def test_prod_start_exports_scrapling_supervisor_env_from_env_file(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="prod-start-scrapling-env-"))
        stub_dir = temp_dir / "bin"
        stub_dir.mkdir()
        spin_env_log = temp_dir / "spin-env.log"
        custom_manifest = temp_dir / "spin.gateway.toml"
        config_db = temp_dir / "sqlite_key_value.db"
        env_file = temp_dir / ".env.local"
        custom_manifest.write_text("spin_manifest_version = 2\n", encoding="utf-8")
        env_file.write_text(
            textwrap.dedent(
                f"""\
                SHUMA_API_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
                SHUMA_FORWARDED_IP_SECRET=forwarded-secret
                SHUMA_SIM_TELEMETRY_SECRET=feedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeed
                SHUMA_ADVERSARY_SIM_AVAILABLE=false
                ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH=/opt/shuma-gorath/.shuma/adversary-sim/scrapling-scope.json
                ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH=/opt/shuma-gorath/.shuma/adversary-sim/scrapling-seed-inventory.json
                ADVERSARY_SIM_SCRAPLING_CRAWLDIR=/opt/shuma-gorath/.shuma/adversary-sim/scrapling-crawldir
                SHUMA_SPIN_MANIFEST={custom_manifest}
                SHUMA_CONFIG_DB_PATH={config_db}
                """
            ),
            encoding="utf-8",
        )

        write_executable(
            stub_dir / "spin",
            textwrap.dedent(
                f"""\
                #!/bin/sh
                {{
                  printf 'SHUMA_SIM_TELEMETRY_SECRET=%s\\n' "$SHUMA_SIM_TELEMETRY_SECRET"
                  printf 'ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH=%s\\n' "$ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH"
                  printf 'ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH=%s\\n' "$ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH"
                  printf 'ADVERSARY_SIM_SCRAPLING_CRAWLDIR=%s\\n' "$ADVERSARY_SIM_SCRAPLING_CRAWLDIR"
                }} > "{spin_env_log}"
                exit 0
                """
            ),
        )
        write_executable(stub_dir / "pkill", "#!/bin/sh\nexit 0\n")
        write_executable(stub_dir / "uuidgen", "#!/bin/sh\necho test-runtime-id\n")
        self.write_wrapper_stubs(stub_dir)

        env = os.environ.copy()
        env["PATH"] = f"{stub_dir}:{env['PATH']}"
        env["ENV_LOCAL"] = str(env_file)
        env["SHUMA_CONFIG_DB_PATH"] = str(config_db)
        env["SHUMA_SPIN_MANIFEST"] = str(custom_manifest)

        seed_result = subprocess.run(
            ["bash", str(REPO_ROOT / "scripts" / "config_seed.sh")],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(seed_result.returncode, 0, msg=seed_result.stderr or seed_result.stdout)

        result = subprocess.run(
            ["make", "--no-print-directory", "prod-start"],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue(spin_env_log.exists())
        spin_env = spin_env_log.read_text(encoding="utf-8")
        self.assertIn(
            "SHUMA_SIM_TELEMETRY_SECRET=feedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeed",
            spin_env,
        )
        self.assertIn(
            "ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH=/opt/shuma-gorath/.shuma/adversary-sim/scrapling-scope.json",
            spin_env,
        )
        self.assertIn(
            "ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH=/opt/shuma-gorath/.shuma/adversary-sim/scrapling-seed-inventory.json",
            spin_env,
        )
        self.assertIn(
            "ADVERSARY_SIM_SCRAPLING_CRAWLDIR=/opt/shuma-gorath/.shuma/adversary-sim/scrapling-crawldir",
            spin_env,
        )

    def test_deploy_env_validate_prefers_process_manifest_over_stale_env_file_value(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="deploy-env-validate-manifest-"))
        stale_manifest = temp_dir / "stale-spin.toml"
        custom_manifest = temp_dir / "custom-spin.toml"
        env_file = temp_dir / ".env.local"
        catalog_path = temp_dir / "surface-catalog.json"

        stale_manifest.write_text(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://stale.example.com:443"]
                """
            ),
            encoding="utf-8",
        )
        custom_manifest.write_text(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://origin.example.com:443"]
                """
            ),
            encoding="utf-8",
        )
        env_file.write_text(f"SHUMA_SPIN_MANIFEST={stale_manifest}\n", encoding="utf-8")
        catalog_path.write_text('{"inventory":[{"path":"/index.html"}]}\n', encoding="utf-8")

        env = os.environ.copy()
        env["ENV_LOCAL"] = str(env_file)
        env["SHUMA_SPIN_MANIFEST"] = str(custom_manifest)
        env["GATEWAY_SURFACE_CATALOG_PATH"] = str(catalog_path)
        env["SHUMA_RUNTIME_ENV"] = "runtime-prod"
        env["SHUMA_DEBUG_HEADERS"] = "false"
        env["SHUMA_ADMIN_IP_ALLOWLIST"] = "203.0.113.1/32"
        env["SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED"] = "true"
        env["SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED"] = "true"
        env["SHUMA_ENTERPRISE_MULTI_INSTANCE"] = "false"
        env["SHUMA_GATEWAY_DEPLOYMENT_PROFILE"] = "shared-server"
        env["SHUMA_GATEWAY_UPSTREAM_ORIGIN"] = "https://origin.example.com"
        env["SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED"] = "true"
        env["SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED"] = "true"

        result = subprocess.run(
            ["make", "--no-print-directory", "deploy-env-validate"],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

    def test_prebuilt_deploy_validation_prefers_process_manifest_over_stale_env_file_value(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="deploy-prebuilt-manifest-"))
        stale_manifest = temp_dir / "stale-spin.toml"
        custom_manifest = temp_dir / "custom-spin.toml"
        env_file = temp_dir / ".env.local"
        catalog_path = temp_dir / "surface-catalog.json"
        config_db = temp_dir / "sqlite_key_value.db"
        wasm_dir = REPO_ROOT / "dist" / "wasm"
        dashboard_dir = REPO_ROOT / "dist" / "dashboard"
        wasm_dir.mkdir(parents=True, exist_ok=True)
        dashboard_dir.mkdir(parents=True, exist_ok=True)
        (wasm_dir / "shuma_gorath.wasm").write_text("wasm-binary", encoding="utf-8")
        (dashboard_dir / "index.html").write_text("<h1>Dashboard</h1>\n", encoding="utf-8")

        stale_manifest.write_text(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://stale.example.com:443"]
                """
            ),
            encoding="utf-8",
        )
        custom_manifest.write_text(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://origin.example.com:443"]
                """
            ),
            encoding="utf-8",
        )
        env_file.write_text(f"SHUMA_SPIN_MANIFEST={stale_manifest}\n", encoding="utf-8")
        catalog_path.write_text('{"inventory":[{"path":"/index.html"}]}\n', encoding="utf-8")

        env = os.environ.copy()
        env["ENV_LOCAL"] = str(env_file)
        env["SHUMA_CONFIG_DB_PATH"] = str(config_db)
        env["SHUMA_SPIN_MANIFEST"] = str(custom_manifest)
        env["GATEWAY_SURFACE_CATALOG_PATH"] = str(catalog_path)
        env["SHUMA_RUNTIME_ENV"] = "runtime-prod"
        env["SHUMA_DEBUG_HEADERS"] = "false"
        env["SHUMA_ADMIN_IP_ALLOWLIST"] = "203.0.113.1/32"
        env["SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED"] = "true"
        env["SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED"] = "true"
        env["SHUMA_ENTERPRISE_MULTI_INSTANCE"] = "false"
        env["SHUMA_GATEWAY_DEPLOYMENT_PROFILE"] = "shared-server"
        env["SHUMA_GATEWAY_UPSTREAM_ORIGIN"] = "https://origin.example.com"
        env["SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED"] = "true"
        env["SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED"] = "true"

        seed_result = subprocess.run(
            ["bash", str(REPO_ROOT / "scripts" / "config_seed.sh")],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(seed_result.returncode, 0, msg=seed_result.stderr or seed_result.stdout)

        result = subprocess.run(
            ["make", "--no-print-directory", "deploy-self-hosted-minimal-prebuilt"],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)


if __name__ == "__main__":
    unittest.main()
