import json
import os
import sqlite3
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "config_seed.sh"
DEFAULTS_FILE = REPO_ROOT / "config" / "defaults.env"


class ConfigLifecycleTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="config-lifecycle-"))
        self.db_path = self.temp_dir / "sqlite_key_value.db"
        self.defaults_file = self.temp_dir / "defaults.env"
        self.defaults_file.write_text(
            DEFAULTS_FILE.read_text(encoding="utf-8"),
            encoding="utf-8",
        )
        self.config_key = "config:default"
        self.store_name = "default"

    def run_config_seed(self, *args: str) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        env["SHUMA_CONFIG_DEFAULTS_FILE"] = str(self.defaults_file)
        env["SHUMA_CONFIG_DB_PATH"] = str(self.db_path)
        env["SHUMA_CONFIG_STORE_NAME"] = self.store_name
        env["SHUMA_CONFIG_KEY"] = self.config_key
        return subprocess.run(
            ["bash", str(SCRIPT), *args],
            cwd=REPO_ROOT,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

    def read_config(self) -> dict:
        conn = sqlite3.connect(self.db_path)
        try:
            row = conn.execute(
                "SELECT CAST(value AS TEXT) FROM spin_key_value WHERE store=? AND key=?",
                (self.store_name, self.config_key),
            ).fetchone()
        finally:
            conn.close()
        self.assertIsNotNone(row, "expected seeded config row")
        return json.loads(row[0])

    def write_raw_config(self, raw: str) -> None:
        conn = sqlite3.connect(self.db_path)
        try:
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS spin_key_value (
                  store TEXT NOT NULL,
                  key   TEXT NOT NULL,
                  value BLOB NOT NULL,
                  PRIMARY KEY (store, key)
                )
                """
            )
            conn.execute(
                """
                INSERT INTO spin_key_value(store, key, value)
                VALUES(?, ?, ?)
                ON CONFLICT(store, key) DO UPDATE SET value=excluded.value
                """,
                (self.store_name, self.config_key, raw.encode("utf-8")),
            )
            conn.commit()
        finally:
            conn.close()

    def test_verify_only_reports_missing_and_seed_then_passes(self) -> None:
        missing = self.run_config_seed("--verify-only")
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("missing KV config", missing.stderr + missing.stdout)

        seeded = self.run_config_seed()
        self.assertEqual(seeded.returncode, 0, msg=seeded.stderr or seeded.stdout)
        self.assertIn("Seeded KV config", seeded.stdout)

        ready = self.run_config_seed("--verify-only")
        self.assertEqual(ready.returncode, 0, msg=ready.stderr or ready.stdout)
        self.assertIn("schema-complete", ready.stdout)

    def test_verify_only_reports_stale_and_seed_backfills_missing_keys(self) -> None:
        seeded = self.run_config_seed()
        self.assertEqual(seeded.returncode, 0, msg=seeded.stderr or seeded.stdout)
        current = self.read_config()
        current.pop("rate_limit", None)
        current["shadow_mode"] = True
        self.write_raw_config(json.dumps(current))

        stale = self.run_config_seed("--verify-only")
        self.assertNotEqual(stale.returncode, 0)
        self.assertIn("stale KV config", stale.stderr + stale.stdout)
        self.assertIn("rate_limit", stale.stderr + stale.stdout)

        repaired = self.run_config_seed()
        self.assertEqual(repaired.returncode, 0, msg=repaired.stderr or repaired.stdout)
        self.assertIn("Backfilled missing KV config keys", repaired.stdout)
        self.assertIn("rate_limit", self.read_config())

    def test_verify_only_reports_invalid_and_seed_repairs_it(self) -> None:
        self.write_raw_config("{not valid json")

        invalid = self.run_config_seed("--verify-only")
        self.assertNotEqual(invalid.returncode, 0)
        self.assertIn("invalid KV config", invalid.stderr + invalid.stdout)

        repaired = self.run_config_seed()
        self.assertEqual(repaired.returncode, 0, msg=repaired.stderr or repaired.stdout)
        self.assertIn("Repaired invalid KV config", repaired.stdout)

        ready = self.run_config_seed("--verify-only")
        self.assertEqual(ready.returncode, 0, msg=ready.stderr or ready.stdout)
        self.assertIn("schema-complete", ready.stdout)

    def test_print_json_emits_canonical_merged_config_without_mutating_store(self) -> None:
        printed_missing = self.run_config_seed("--print-json")
        self.assertEqual(printed_missing.returncode, 0, msg=printed_missing.stderr or printed_missing.stdout)
        missing_payload = json.loads(printed_missing.stdout)
        self.assertIn("rate_limit", missing_payload)
        self.assertFalse(self.db_path.exists(), "print-json must not create the sqlite store")

        seeded = self.run_config_seed()
        self.assertEqual(seeded.returncode, 0, msg=seeded.stderr or seeded.stdout)
        current = self.read_config()
        current.pop("rate_limit", None)
        self.write_raw_config(json.dumps(current))

        printed_stale = self.run_config_seed("--print-json")
        self.assertEqual(printed_stale.returncode, 0, msg=printed_stale.stderr or printed_stale.stdout)
        stale_payload = json.loads(printed_stale.stdout)
        self.assertIn("rate_limit", stale_payload)
        self.assertFalse(stale_payload["shadow_mode"], "print-json must normalize runtime-ephemeral toggles")

        verify = self.run_config_seed("--verify-only")
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("stale KV config", verify.stderr + verify.stdout)

    def test_make_targets_use_read_only_config_verification(self) -> None:
        makefile = (REPO_ROOT / "Makefile").read_text(encoding="utf-8")
        verify_runtime_script = (
            REPO_ROOT / "scripts" / "bootstrap" / "verify-runtime.sh"
        ).read_text(encoding="utf-8")
        verify_setup_script = (
            REPO_ROOT / "scripts" / "bootstrap" / "verify-setup.sh"
        ).read_text(encoding="utf-8")
        for target in (
            "dev:",
            "dev-closed:",
            "run:",
            "run-prebuilt:",
            "prod-start:",
            "prod:",
            "deploy-profile-baseline:",
        ):
            self.assertIn(target, makefile)

        self.assertIn("config-verify", makefile)
        self.assertNotIn("@$(MAKE) --no-print-directory config-seed >/dev/null", makefile)
        self.assertNotIn("$(MAKE) --no-print-directory config-seed >/dev/null 2>&1", makefile)
        self.assertIn(
            'config-verify && $(MAKE) --no-print-directory dashboard-build >/dev/null 2>&1 && RUNTIME_INSTANCE_ID="$$(uuidgen)" && $(SUPERVISOR_HOST_ENV)',
            makefile,
        )
        self.assertIn(
            "SPIN_DEV_OVERRIDES := --env SHUMA_DEBUG_HEADERS=$(DEV_DEBUG_HEADERS) --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) --env SHUMA_ADMIN_IP_ALLOWLIST=$(DEV_ADMIN_IP_ALLOWLIST) --env SHUMA_RUNTIME_ENV=$(DEV_RUNTIME_ENV) --env SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) --env SHUMA_LOCAL_PROD_DIRECT_MODE=$(DEV_LOCAL_PROD_DIRECT_MODE) --env SHUMA_GATEWAY_ORIGIN_AUTH_MODE=network_only --env SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME= --env SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE=",
            makefile,
        )
        self.assertIn(
            '@$(MAKE) --no-print-directory dev DEV_RUNTIME_ENV=runtime-prod DEV_ADVERSARY_SIM_AVAILABLE=$(SHUMA_ADVERSARY_SIM_AVAILABLE) DEV_DEBUG_HEADERS=false DEV_ADMIN_CONFIG_WRITE_ENABLED=true DEV_LOCAL_PROD_DIRECT_MODE=true',
            makefile,
        )
        self.assertIn(
            "SHUMA_MONITORING_RETENTION_HOURS := $(if $(strip $(SHUMA_MONITORING_RETENTION_HOURS)),$(SHUMA_MONITORING_RETENTION_HOURS),$(call defaults_env_lookup,SHUMA_MONITORING_RETENTION_HOURS))",
            makefile,
        )
        self.assertIn(
            "SHUMA_MONITORING_ROLLUP_RETENTION_HOURS := $(if $(strip $(SHUMA_MONITORING_ROLLUP_RETENTION_HOURS)),$(SHUMA_MONITORING_ROLLUP_RETENTION_HOURS),$(call defaults_env_lookup,SHUMA_MONITORING_ROLLUP_RETENTION_HOURS))",
            makefile,
        )
        self.assertIn(
            "SHUMA_EVENT_LOG_IP_STORAGE_MODE := $(call strip_wrapping_quotes,$(SHUMA_EVENT_LOG_IP_STORAGE_MODE))",
            makefile,
        )
        self.assertIn("make --no-print-directory config-verify", verify_runtime_script)
        self.assertNotIn("make --no-print-directory config-seed", verify_runtime_script)
        self.assertIn("make --no-print-directory config-verify", verify_setup_script)
        self.assertNotIn("make --no-print-directory config-seed", verify_setup_script)


if __name__ == "__main__":
    unittest.main()
