import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts.deploy import fermyon_akamai_edge_deploy as deploy


def result(returncode: int = 0, stdout: str = "", stderr: str = "") -> SimpleNamespace:
    return SimpleNamespace(returncode=returncode, stdout=stdout, stderr=stderr)


class DeployFermyonAkamaiEdgeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="deploy-fermyon-"))
        self.env_file = self.temp_dir / ".env.local"
        self.setup_receipt = self.temp_dir / ".shuma" / "fermyon-akamai-edge-setup.json"
        self.deploy_receipt = self.temp_dir / ".shuma" / "fermyon-akamai-edge-deploy.json"
        self.rendered_manifest = self.temp_dir / ".shuma" / "manifests" / "edge.spin.toml"
        self.surface_catalog = self.temp_dir / ".shuma" / "catalogs" / "site.surface-catalog.json"
        self.surface_catalog.parent.mkdir(parents=True, exist_ok=True)
        self.surface_catalog.write_text('{"inventory":[{"path":"/"}]}\n', encoding="utf-8")
        self.env_file.write_text(
            "\n".join(
                [
                    "SPIN_AKA_ACCESS_TOKEN=fermyon-secret",
                    "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE=origin-secret",
                    "",
                ]
            ),
            encoding="utf-8",
        )
        self.setup_receipt.parent.mkdir(parents=True, exist_ok=True)
        self.setup_receipt.write_text(
            json.dumps(
                {
                    "schema": "shuma.fermyon.akamai_edge_setup.v2",
                    "mode": "aka",
                    "status": "ready",
                    "auth_mode": "token",
                    "progress": {
                        "last_completed_step": "auth_validated",
                        "blocked_at_step": "",
                        "blocked_reason": "",
                        "next_operator_action": "Run `make deploy-fermyon-akamai-edge` to continue.",
                    },
                    "fermyon": {
                        "account_id": "acc_123",
                        "account_name": "",
                        "app_name": "shuma-edge-prod",
                    },
                    "gateway": {
                        "runtime_env": "runtime-prod",
                        "deployment_profile": "edge-fermyon",
                        "enterprise_multi_instance": True,
                        "edge_integration_mode": "additive",
                        "upstream_origin": "https://origin.example.com",
                        "tls_strict": True,
                        "origin_auth_mode": "signed_header",
                        "origin_auth_header_name": "x-shuma-origin-auth",
                        "origin_lock_confirmed": True,
                        "reserved_route_collision_check_passed": True,
                        "admin_edge_rate_limits_confirmed": True,
                        "admin_api_key_rotation_confirmed": True,
                        "surface_catalog_path": str(self.surface_catalog.resolve()),
                    },
                    "artifacts": {
                        "rendered_manifest_path": str(self.rendered_manifest.resolve()),
                    },
                }
            )
            + "\n",
            encoding="utf-8",
        )

    def test_load_receipt_rejects_blocked_setup_receipt_with_resume_context(self) -> None:
        self.setup_receipt.write_text(
            json.dumps(
                {
                    "schema": "shuma.fermyon.akamai_edge_setup.v2",
                    "mode": "aka",
                    "status": "blocked",
                    "auth_mode": "",
                    "progress": {
                        "last_completed_step": "local_state_prepared",
                        "blocked_at_step": "auth_validation",
                        "blocked_reason": "User is not allow-listed!",
                        "next_operator_action": "Wait for access approval, then rerun `make prepare-fermyon-akamai-edge`.",
                    },
                    "fermyon": {
                        "account_id": "acc_123",
                        "account_name": "",
                        "app_name": "shuma-edge-prod",
                    },
                    "gateway": {
                        "upstream_origin": "https://origin.example.com",
                        "surface_catalog_path": str(self.surface_catalog.resolve()),
                    },
                    "artifacts": {
                        "rendered_manifest_path": str(self.rendered_manifest.resolve()),
                    },
                }
            )
            + "\n",
            encoding="utf-8",
        )

        with self.assertRaises(SystemExit) as exc:
            deploy.load_receipt(self.setup_receipt)

        message = str(exc.exception)
        self.assertIn("status='blocked'", message)
        self.assertIn("blocked_at_step=auth_validation", message)
        self.assertIn("User is not allow-listed!", message)
        self.assertIn("make prepare-fermyon-akamai-edge", message)

    def test_preflight_only_shapes_make_targets_and_manifest_render(self) -> None:
        calls: list[tuple[list[str], dict[str, str] | None]] = []

        def fake_run(command, *, env=None, cwd=None, capture_output=True):
            calls.append((list(command), env))
            if command[:3] == ["python3", str(deploy.REPO_ROOT / "scripts" / "deploy" / "render_gateway_spin_manifest.py"), "--manifest"]:
                self.rendered_manifest.parent.mkdir(parents=True, exist_ok=True)
                self.rendered_manifest.write_text("allowed_outbound_hosts = []\n", encoding="utf-8")
                return result(stdout="rendered gateway Spin manifest\n")
            if command[:3] == ["make", "--no-print-directory", "deploy-enterprise-akamai"]:
                return result()
            if command[:3] == ["make", "--no-print-directory", "test-gateway-profile-edge"]:
                return result()
            if command[:3] == ["make", "--no-print-directory", "smoke-gateway-mode"]:
                return result()
            raise AssertionError(f"Unexpected command: {command}")

        with patch.object(deploy, "ensure_aka_plugin", return_value="spin-aka 0.6.0"), patch.object(
            deploy, "run_command", side_effect=fake_run
        ), patch.object(
            deploy, "fetch_aka_info", side_effect=SystemExit("Error: You are not logged in. Please run the login command")
        ), patch.object(
            deploy, "validate_aka_login", return_value=({"account": {"id": "acc_123"}}, "token")
        ):
            rc = deploy.main(
                [
                    "--env-file",
                    str(self.env_file),
                    "--setup-receipt",
                    str(self.setup_receipt),
                    "--deploy-receipt-output",
                    str(self.deploy_receipt),
                    "--preflight-only",
                ]
            )

        self.assertEqual(rc, 0)
        rendered_call = calls[0][0]
        self.assertIn("--upstream-origin", rendered_call)
        env = calls[1][1]
        self.assertIsNotNone(env)
        self.assertEqual(env["SHUMA_GATEWAY_DEPLOYMENT_PROFILE"], "edge-fermyon")
        self.assertEqual(env["SHUMA_ENTERPRISE_MULTI_INSTANCE"], "true")
        self.assertEqual(env["SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE"], "origin-secret")
        self.assertEqual(env["SHUMA_SPIN_MANIFEST"], str(self.rendered_manifest.resolve()))
        self.assertEqual(
            [call[0][2] for call in calls if call[0][:2] == ["make", "--no-print-directory"]],
            ["deploy-enterprise-akamai", "test-gateway-profile-edge", "smoke-gateway-mode"],
        )

    def test_preflight_surfaces_login_plugin_panic_clearly(self) -> None:
        def fake_run(command, *, env=None, cwd=None, capture_output=True):
            if command[:3] == ["python3", str(deploy.REPO_ROOT / "scripts" / "deploy" / "render_gateway_spin_manifest.py"), "--manifest"]:
                return result()
            raise AssertionError(f"Unexpected command: {command}")

        with patch.object(deploy, "ensure_aka_plugin", return_value="spin-aka 0.6.0"), patch.object(
            deploy, "run_command", side_effect=fake_run
        ), patch.object(
            deploy, "fetch_aka_info", side_effect=SystemExit("Error: You are not logged in. Please run the login command")
        ), patch.object(
            deploy,
            "validate_aka_login",
            side_effect=SystemExit(
                "spin aka login failed due to an upstream plugin panic. "
                "Observed failure: thread 'main' panicked at plugin/src/commands/login.rs:159:32: index out of bounds"
            ),
        ):
            with self.assertRaises(SystemExit) as exc:
                deploy.main(
                    [
                        "--env-file",
                        str(self.env_file),
                        "--setup-receipt",
                        str(self.setup_receipt),
                        "--deploy-receipt-output",
                        str(self.deploy_receipt),
                        "--preflight-only",
                    ]
                )

        self.assertIn("upstream plugin panic", str(exc.exception))

    def test_deploy_reuses_existing_app_id_from_prior_receipt(self) -> None:
        self.deploy_receipt.parent.mkdir(parents=True, exist_ok=True)
        self.deploy_receipt.write_text(
            json.dumps(
                {
                    "schema": "shuma.fermyon.akamai_edge_deploy.v1",
                    "fermyon": {
                        "app_id": "app_existing_123",
                        "app_name": "shuma-edge-prod",
                        "status": {},
                    },
                }
            )
            + "\n",
            encoding="utf-8",
        )

        calls: list[list[str]] = []

        def fake_run(command, *, env=None, cwd=None, capture_output=True):
            calls.append(list(command))
            if command[:3] == [
                "python3",
                str(deploy.REPO_ROOT / "scripts" / "deploy" / "render_gateway_spin_manifest.py"),
                "--manifest",
            ]:
                self.rendered_manifest.parent.mkdir(parents=True, exist_ok=True)
                self.rendered_manifest.write_text("allowed_outbound_hosts = []\n", encoding="utf-8")
                return result(stdout="rendered gateway Spin manifest\n")
            if command[:2] == ["make", "--no-print-directory"]:
                return result()
            if command[:4] == ["spin", "aka", "deploy", "-f"]:
                return result(stdout="deploy ok\n")
            if command[:4] == ["spin", "aka", "app", "status"]:
                return result(stdout='{"app":{"id":"app_existing_123"}}\n')
            if command[:3] == ["spin", "--version"]:
                return result(stdout="spin 3.5.1\n")
            if command[:3] == ["spin", "aka", "--version"]:
                return result(stdout="spin-aka 0.6.0\n")
            if command[:3] == ["git", "rev-parse", "HEAD"]:
                return result(stdout="deadbeef\n")
            raise AssertionError(f"Unexpected command: {command}")

        with patch.object(deploy, "ensure_aka_plugin", return_value="spin-aka 0.6.0"), patch.object(
            deploy, "run_command", side_effect=fake_run
        ), patch.object(
            deploy, "fetch_aka_info", return_value={"account": {"id": "acc_123"}}
        ):
            rc = deploy.main(
                [
                    "--env-file",
                    str(self.env_file),
                    "--setup-receipt",
                    str(self.setup_receipt),
                    "--deploy-receipt-output",
                    str(self.deploy_receipt),
                ]
            )

        self.assertEqual(rc, 0)
        deploy_call = next(call for call in calls if call[:3] == ["spin", "aka", "deploy"])
        self.assertIn("--app-id", deploy_call)
        self.assertIn("app_existing_123", deploy_call)
        self.assertNotIn("--create-name", deploy_call)

        updated_receipt = json.loads(self.deploy_receipt.read_text(encoding="utf-8"))
        self.assertEqual(updated_receipt["fermyon"]["app_id"], "app_existing_123")
