import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts.deploy import fermyon_akamai_edge_setup as setup


class PrepareFermyonAkamaiEdgeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="prepare-fermyon-"))
        self.env_file = self.temp_dir / ".env.local"
        self.receipt_path = self.temp_dir / ".shuma" / "fermyon-akamai-edge-setup.json"
        self.deploy_receipt = self.temp_dir / ".shuma" / "fermyon-akamai-edge-deploy.json"
        self.rendered_manifest = self.temp_dir / ".shuma" / "manifests" / "edge.spin.toml"
        self.docroot = self.temp_dir / "dummy_static_site"
        self.docroot.mkdir()
        (self.docroot / "index.html").write_text("<h1>Hello</h1>\n", encoding="utf-8")
        (self.docroot / "about.html").write_text("<p>About</p>\n", encoding="utf-8")

    def test_default_receipts_use_durable_local_state_dir(self) -> None:
        self.assertEqual(
            setup.DEFAULT_RECEIPT_PATH,
            setup.REPO_ROOT / ".shuma" / "fermyon-akamai-edge-setup.json",
        )
        self.assertEqual(
            setup.DEFAULT_DEPLOY_RECEIPT_PATH,
            setup.REPO_ROOT / ".shuma" / "fermyon-akamai-edge-deploy.json",
        )
        self.assertEqual(
            setup.DEFAULT_RENDERED_MANIFEST_PATH,
            setup.REPO_ROOT / "spin.fermyon-akamai-edge.toml",
        )
        self.assertNotIn("/.spin/", str(setup.DEFAULT_RECEIPT_PATH))

    def test_main_persists_token_generates_secrets_and_writes_receipt(self) -> None:
        def fake_run(command, *, env=None, cwd=None):
            self.assertEqual(command[:2], ["spin", "--version"])
            return SimpleNamespace(returncode=0, stdout="spin 3.5.1\n", stderr="")

        with patch.object(setup, "run_command", side_effect=fake_run), patch.object(
            setup, "ensure_aka_plugin", return_value="spin-aka 0.6.0"
        ), patch.object(
            setup,
            "fetch_aka_info",
            side_effect=SystemExit("Error: You are not logged in. Please run the login command"),
        ), patch.object(
            setup,
            "validate_aka_login",
            return_value=(
                {
                    "auth_info": {
                        "accounts": [
                            {"id": "acc_123", "name": "james"},
                        ]
                    }
                },
                "token",
            ),
        ):
            rc = setup.main(
                [
                    "--env-file",
                    str(self.env_file),
                    "--receipt-output",
                    str(self.receipt_path),
                    "--deploy-receipt-output",
                    str(self.deploy_receipt),
                    "--rendered-manifest-output",
                    str(self.rendered_manifest),
                    "--fermyon-token",
                    "fermyon-secret",
                    "--account-id",
                    "acc_123",
                    "--app-name",
                    "shuma-edge-prod",
                    "--upstream-origin",
                    "https://origin.example.com",
                    "--admin-ip",
                    "203.0.113.8/32",
                    "--docroot",
                    str(self.docroot),
                    "--catalog-output",
                    str(self.temp_dir / ".shuma" / "catalogs" / "dummy_static_site.surface-catalog.json"),
                    "--origin-lock-confirmed",
                    "true",
                    "--reserved-route-collision-check-passed",
                    "true",
                    "--admin-edge-rate-limits-confirmed",
                    "true",
                    "--admin-api-key-rotation-confirmed",
                    "true",
                    "--enterprise-unsynced-state-exception-confirmed",
                    "true",
                ]
            )

        self.assertEqual(rc, 0)
        env_text = self.env_file.read_text(encoding="utf-8")
        self.assertIn("SPIN_AKA_ACCESS_TOKEN=fermyon-secret", env_text)
        self.assertIn("SHUMA_ADMIN_IP_ALLOWLIST=203.0.113.8/32", env_text)
        self.assertIn("SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://origin.example.com", env_text)
        self.assertIn("SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true", env_text)
        self.assertIn("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME=x-shuma-origin-auth", env_text)
        self.assertIn("GATEWAY_SURFACE_CATALOG_PATH=", env_text)
        self.assertIn("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE=", env_text)
        self.assertIn("SHUMA_API_KEY=", env_text)
        self.assertIn("SHUMA_JS_SECRET=", env_text)
        self.assertIn("SHUMA_FORWARDED_IP_SECRET=", env_text)
        self.assertIn("SHUMA_HEALTH_SECRET=", env_text)
        self.assertIn("SHUMA_SIM_TELEMETRY_SECRET=", env_text)

        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["schema"], "shuma.fermyon.akamai_edge_setup.v2")
        self.assertEqual(receipt["mode"], "aka")
        self.assertEqual(receipt["status"], "ready")
        self.assertEqual(receipt["auth_mode"], "token")
        self.assertEqual(receipt["progress"]["last_completed_step"], "auth_validated")
        self.assertEqual(receipt["progress"]["blocked_at_step"], "")
        self.assertEqual(receipt["progress"]["blocked_reason"], "")
        self.assertEqual(receipt["fermyon"]["account_id"], "acc_123")
        self.assertEqual(receipt["fermyon"]["account_name"], "james")
        self.assertEqual(receipt["fermyon"]["app_name"], "shuma-edge-prod")
        self.assertEqual(receipt["gateway"]["upstream_origin"], "https://origin.example.com")
        self.assertTrue(receipt["gateway"]["origin_lock_confirmed"])
        self.assertTrue(receipt["gateway"]["reserved_route_collision_check_passed"])
        self.assertTrue(receipt["gateway"]["admin_edge_rate_limits_confirmed"])
        self.assertTrue(receipt["gateway"]["admin_api_key_rotation_confirmed"])
        self.assertTrue(receipt["gateway"]["enterprise_unsynced_state_exception_confirmed"])
        self.assertEqual(receipt["artifacts"]["deploy_receipt_path"], str(self.deploy_receipt.resolve()))
        self.assertEqual(
            receipt["artifacts"]["rendered_manifest_path"], str(self.rendered_manifest.resolve())
        )

    def test_validate_aka_login_surfaces_plugin_panic_clearly(self) -> None:
        with patch.object(setup, "is_interactive_session", return_value=False), patch.object(
            setup,
            "run_command",
            return_value=SimpleNamespace(
                returncode=101,
                stdout="",
                stderr="thread 'main' panicked at plugin/src/commands/login.rs:159:32: index out of bounds",
            ),
        ):
            with self.assertRaises(SystemExit) as exc:
                setup.validate_aka_login("fermyon-secret")

        self.assertIn("upstream plugin panic", str(exc.exception))

    def test_validate_aka_login_falls_back_to_device_login_when_interactive(self) -> None:
        calls: list[tuple[list[str], bool]] = []

        def fake_run(command, *, env=None, cwd=None, capture_output=True):
            calls.append((list(command), capture_output))
            if command[:3] == ["spin", "aka", "login"] and capture_output:
                return SimpleNamespace(
                    returncode=101,
                    stdout="",
                    stderr="thread 'main' panicked at plugin/src/commands/login.rs:159:32: index out of bounds",
                )
            if command[:3] == ["spin", "aka", "login"] and not capture_output:
                return SimpleNamespace(returncode=0, stdout="", stderr="")
            if command[:4] == ["spin", "aka", "info", "--format"]:
                return SimpleNamespace(returncode=0, stdout='{"account":{"id":"acc_123"}}\n', stderr="")
            raise AssertionError(f"Unexpected command: {command}")

        with patch.object(setup, "is_interactive_session", return_value=True), patch.object(
            setup, "run_command", side_effect=fake_run
        ):
            info, auth_mode = setup.validate_aka_login("fermyon-secret")

        self.assertEqual(auth_mode, "device_login")
        self.assertEqual(info["account"]["id"], "acc_123")
        self.assertEqual(calls[1], (["spin", "aka", "login"], False))

    def test_main_writes_blocked_receipt_when_authentication_cannot_complete(self) -> None:
        with patch.object(setup, "ensure_aka_plugin", return_value="spin-aka 0.6.0"), patch.object(
            setup,
            "run_command",
            return_value=SimpleNamespace(returncode=0, stdout="spin 3.5.1\n", stderr=""),
        ), patch.object(
            setup,
            "fetch_aka_info",
            side_effect=SystemExit("Error: You are not logged in. Please run the login command"),
        ), patch.object(
            setup,
            "validate_aka_login",
            side_effect=SystemExit("User is not allow-listed!"),
        ):
            with self.assertRaises(SystemExit) as exc:
                setup.main(
                    [
                        "--env-file",
                        str(self.env_file),
                        "--receipt-output",
                        str(self.receipt_path),
                        "--deploy-receipt-output",
                        str(self.deploy_receipt),
                        "--rendered-manifest-output",
                        str(self.rendered_manifest),
                        "--fermyon-token",
                        "fermyon-secret",
                        "--account-id",
                        "acc_123",
                        "--app-name",
                        "shuma-edge-prod",
                        "--upstream-origin",
                        "https://origin.example.com",
                        "--admin-ip",
                        "203.0.113.8/32",
                        "--docroot",
                        str(self.docroot),
                        "--catalog-output",
                        str(self.temp_dir / ".shuma" / "catalogs" / "dummy_static_site.surface-catalog.json"),
                    ]
                )

        self.assertIn("User is not allow-listed!", str(exc.exception))
        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["schema"], "shuma.fermyon.akamai_edge_setup.v2")
        self.assertEqual(receipt["status"], "blocked")
        self.assertEqual(receipt["auth_mode"], "")
        self.assertEqual(receipt["progress"]["last_completed_step"], "local_state_prepared")
        self.assertEqual(receipt["progress"]["blocked_at_step"], "auth_validation")
        self.assertEqual(receipt["progress"]["blocked_reason"], "User is not allow-listed!")
        self.assertIn("make prepare-fermyon-akamai-edge", receipt["progress"]["next_operator_action"])

    def test_main_blocks_when_multiple_accounts_require_explicit_selection(self) -> None:
        with patch.object(setup, "ensure_aka_plugin", return_value="spin-aka 0.6.0"), patch.object(
            setup,
            "run_command",
            return_value=SimpleNamespace(returncode=0, stdout="spin 3.5.1\n", stderr=""),
        ), patch.object(
            setup,
            "fetch_aka_info",
            side_effect=SystemExit("Error: You are not logged in. Please run the login command"),
        ), patch.object(
            setup,
            "validate_aka_login",
            return_value=(
                {
                    "auth_info": {
                        "accounts": [
                            {"id": "acc_a", "name": "team-a"},
                            {"id": "acc_b", "name": "team-b"},
                        ]
                    }
                },
                "device_login",
            ),
        ):
            with self.assertRaises(SystemExit) as exc:
                setup.main(
                    [
                        "--env-file",
                        str(self.env_file),
                        "--receipt-output",
                        str(self.receipt_path),
                        "--deploy-receipt-output",
                        str(self.deploy_receipt),
                        "--rendered-manifest-output",
                        str(self.rendered_manifest),
                        "--fermyon-token",
                        "fermyon-secret",
                        "--app-name",
                        "shuma-edge-prod",
                        "--upstream-origin",
                        "https://origin.example.com",
                        "--admin-ip",
                        "203.0.113.8/32",
                        "--docroot",
                        str(self.docroot),
                        "--catalog-output",
                        str(self.temp_dir / ".shuma" / "catalogs" / "dummy_static_site.surface-catalog.json"),
                    ]
                )

        self.assertIn("Multiple Fermyon accounts are available", str(exc.exception))
        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["status"], "blocked")
        self.assertEqual(receipt["progress"]["blocked_at_step"], "account_target_resolution")

    def test_main_reuses_existing_aka_session_without_retrying_token_login(self) -> None:
        def fake_run(command, *, env=None, cwd=None):
            self.assertEqual(command[:2], ["spin", "--version"])
            return SimpleNamespace(returncode=0, stdout="spin 3.5.1\n", stderr="")

        with patch.object(setup, "run_command", side_effect=fake_run), patch.object(
            setup, "ensure_aka_plugin", return_value="spin-aka 0.6.0"
        ), patch.object(
            setup,
            "fetch_aka_info",
            return_value={"auth_info": {"accounts": [{"id": "acc_123", "name": "james"}]}},
        ), patch.object(
            setup,
            "validate_aka_login",
        ) as validate_login:
            rc = setup.main(
                [
                    "--env-file",
                    str(self.env_file),
                    "--receipt-output",
                    str(self.receipt_path),
                    "--deploy-receipt-output",
                    str(self.deploy_receipt),
                    "--rendered-manifest-output",
                    str(self.rendered_manifest),
                    "--fermyon-token",
                    "fermyon-secret",
                    "--app-name",
                    "shuma-edge-prod",
                    "--upstream-origin",
                    "https://origin.example.com",
                    "--admin-ip",
                    "203.0.113.8/32",
                    "--docroot",
                    str(self.docroot),
                    "--catalog-output",
                    str(self.temp_dir / ".shuma" / "catalogs" / "dummy_static_site.surface-catalog.json"),
                ]
            )

        self.assertEqual(rc, 0)
        validate_login.assert_not_called()
        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["auth_mode"], "existing_session")

    def test_ensure_aka_plugin_upgrades_legacy_install_without_json_info(self) -> None:
        calls: list[list[str]] = []

        def fake_run(command, *, env=None, cwd=None, capture_output=True):
            calls.append(list(command))
            if command[:5] == ["spin", "plugins", "list", "--installed", "--format"]:
                return SimpleNamespace(
                    returncode=0,
                    stdout='[{"name":"aka","installedVersion":"0.4.4"}]\n',
                    stderr="",
                )
            if command == ["spin", "aka", "info", "--help"]:
                return SimpleNamespace(returncode=0, stdout="USAGE:\n    spin aka info\n", stderr="")
            if command == ["spin", "plugins", "upgrade", "-y", "aka"]:
                return SimpleNamespace(returncode=0, stdout="upgraded\n", stderr="")
            if command == ["spin", "aka", "--version"]:
                return SimpleNamespace(returncode=0, stdout="spin-aka 0.6.0\n", stderr="")
            raise AssertionError(f"Unexpected command: {command}")

        with patch.object(setup, "run_command", side_effect=fake_run):
            version = setup.ensure_aka_plugin()

        self.assertEqual(version, "spin-aka 0.6.0")
        self.assertIn(["spin", "plugins", "upgrade", "-y", "aka"], calls)
