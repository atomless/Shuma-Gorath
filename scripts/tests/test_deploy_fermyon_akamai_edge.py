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
        self.repo_root_patcher = patch.object(deploy, "REPO_ROOT", self.temp_dir)
        self.repo_root_patcher.start()
        self.env_file = self.temp_dir / ".env.local"
        self.setup_receipt = self.temp_dir / ".shuma" / "fermyon-akamai-edge-setup.json"
        self.deploy_receipt = self.temp_dir / ".shuma" / "fermyon-akamai-edge-deploy.json"
        self.rendered_manifest = self.temp_dir / "spin.fermyon-akamai-edge.toml"
        self.surface_catalog = self.temp_dir / ".shuma" / "catalogs" / "site.surface-catalog.json"
        self.surface_catalog.parent.mkdir(parents=True, exist_ok=True)
        self.surface_catalog.write_text('{"inventory":[{"path":"/"}]}\n', encoding="utf-8")
        self.env_file.write_text(
            "\n".join(
                [
                    "SPIN_AKA_ACCESS_TOKEN=fermyon-secret",
                    "SHUMA_API_KEY=test-admin-key",
                    "SHUMA_JS_SECRET=test-js-secret",
                    "SHUMA_FORWARDED_IP_SECRET=test-forwarded-secret",
                    "SHUMA_HEALTH_SECRET=test-health-secret",
                    "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET=test-edge-cron-secret",
                    "SHUMA_ADMIN_IP_ALLOWLIST=203.0.113.8/32",
                    "SHUMA_DEBUG_HEADERS=false",
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
                        "enterprise_unsynced_state_exception_confirmed": True,
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

    def tearDown(self) -> None:
        self.repo_root_patcher.stop()

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

    def test_run_interactive_command_returns_after_child_exit(self) -> None:
        completed = deploy.run_interactive_command(
            ["/bin/sh", "-lc", "printf 'edge deploy ok\\n'"],
            env={**deploy.os.environ},
            cwd=deploy.REPO_ROOT,
        )

        self.assertEqual(completed.returncode, 0)
        self.assertIn("edge deploy ok", completed.stdout)

    def test_resolve_rendered_manifest_path_rehomes_external_receipt_path_into_current_workspace(self) -> None:
        external_manifest = self.temp_dir.parent / "other-workspace" / "spin.fermyon-akamai-edge.toml"
        receipt = deploy.load_receipt(self.setup_receipt)
        receipt["artifacts"]["rendered_manifest_path"] = str(external_manifest)

        resolved = deploy.resolve_rendered_manifest_path(receipt)

        self.assertEqual(resolved, self.rendered_manifest.resolve())

    def test_preflight_only_shapes_make_targets_and_manifest_render(self) -> None:
        calls: list[tuple[list[str], dict[str, str] | None]] = []

        def fake_run(command, *, env=None, cwd=None, capture_output=True):
            calls.append((list(command), env))
            if command[:3] == ["python3", str(deploy.REPO_ROOT / "scripts" / "deploy" / "render_gateway_spin_manifest.py"), "--manifest"]:
                self.rendered_manifest.parent.mkdir(parents=True, exist_ok=True)
                self.rendered_manifest.write_text("allowed_outbound_hosts = []\n", encoding="utf-8")
                return result(stdout="rendered gateway Spin manifest\n")
            if command[:2] == ["make", "--no-print-directory"] and command[-1] == "deploy-enterprise-akamai":
                return result()
            if command[:2] == ["make", "--no-print-directory"] and command[-1] == "test-gateway-profile-edge":
                return result()
            if command[:2] == ["make", "--no-print-directory"] and command[-1] == "smoke-gateway-mode":
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
        make_calls = [call[0] for call in calls if call[0][:2] == ["make", "--no-print-directory"]]
        self.assertEqual(
            [call[-1] for call in make_calls],
            ["deploy-enterprise-akamai", "test-gateway-profile-edge", "smoke-gateway-mode"],
        )
        self.assertIn("SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://origin.example.com", make_calls[0])
        self.assertIn("SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon", make_calls[0])
        self.assertIn("SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true", make_calls[0])
        self.assertIn(f"SHUMA_SPIN_MANIFEST={self.rendered_manifest.resolve()}", make_calls[0])

    def test_deploy_passes_runtime_env_only_values_as_spin_variables(self) -> None:
        calls: list[list[str]] = []
        interactive_calls: list[list[str]] = []

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
            if command[:4] == ["spin", "aka", "app", "status"]:
                return result(stdout='{"app":{"id":"app_existing_123"},"urls":["https://app.example.com"]}\n')
            if command[:3] == ["spin", "--version"]:
                return result(stdout="spin 3.5.1\n")
            if command[:3] == ["spin", "aka", "--version"]:
                return result(stdout="spin-aka 0.6.0\n")
            if command[:3] == ["git", "rev-parse", "HEAD"]:
                return result(stdout="deadbeef\n")
            raise AssertionError(f"Unexpected command: {command}")

        def fake_interactive(command, *, env=None, cwd=None):
            interactive_calls.append(list(command))
            return result(stdout="deploy ok\n")

        with patch.object(deploy, "ensure_aka_plugin", return_value="spin-aka 0.6.0"), patch.object(
            deploy, "run_command", side_effect=fake_run
        ), patch.object(
            deploy, "run_interactive_command", side_effect=fake_interactive
        ), patch.object(
            deploy, "bootstrap_remote_config_if_missing"
        ) as bootstrap_mock, patch.object(
            deploy, "smoke_deployed_app"
        ) as smoke_mock, patch.object(
            deploy, "wait_for_adversary_sim_control_lease_release"
        ) as lease_release_mock, patch.object(
            deploy, "smoke_external_dashboard"
        ) as external_smoke_mock, patch.object(
            deploy,
            "ensure_adversary_sim_edge_cron",
            return_value={"job_name_prefix": "shuma-adversary-sim-beat", "job_count": 5},
        ) as cron_mock, patch.object(
            deploy, "smoke_adversary_sim_generation"
        ) as sim_smoke_mock, patch.object(
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
        self.assertEqual(len(interactive_calls), 1)
        bootstrap_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        smoke_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        lease_release_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        external_smoke_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        cron_mock.assert_called_once_with(
            env=unittest.mock.ANY,
            app_id="app_existing_123",
            account_id="acc_123",
            account_name="",
        )
        sim_smoke_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        deploy_command = interactive_calls[0]
        self.assertIn("--variable", deploy_command)
        self.assertIn("shuma_api_key=test-admin-key", deploy_command)
        self.assertIn("shuma_js_secret=test-js-secret", deploy_command)
        self.assertIn("shuma_forwarded_ip_secret=test-forwarded-secret", deploy_command)
        self.assertIn("shuma_health_secret=test-health-secret", deploy_command)
        self.assertIn("shuma_adversary_sim_edge_cron_secret=test-edge-cron-secret", deploy_command)
        self.assertIn("shuma_admin_ip_allowlist=203.0.113.8/32", deploy_command)
        self.assertIn("shuma_debug_headers=false", deploy_command)
        self.assertIn("shuma_event_log_retention_hours=168", deploy_command)
        self.assertIn("shuma_monitoring_retention_hours=168", deploy_command)
        self.assertIn("shuma_monitoring_rollup_retention_hours=720", deploy_command)

    def test_ensure_adversary_sim_edge_cron_recreates_staggered_job_set(self) -> None:
        calls: list[list[str]] = []

        def fake_run(command, *, env=None, cwd=None, capture_output=True):
            calls.append(list(command))
            if command[:4] == ["spin", "aka", "cron", "list"] and len(calls) == 1:
                return result(
                    stdout="\n".join(
                        [
                            "+----------------------------+----------------+-------------------------+",
                            "| Name                       | Schedule       | Next Run                |",
                            "+=======================================================================+",
                            "| shuma-adversary-sim-beat   | */5 * * * *    | 2026-03-12 13:05:00 UTC |",
                            "| shuma-adversary-sim-beat-0 | */5 * * * *    | 2026-03-12 13:05:00 UTC |",
                            "| shuma-adversary-sim-beat-1 | 1-59/5 * * * * | 2026-03-12 13:06:00 UTC |",
                            "+----------------------------+----------------+-------------------------+",
                        ]
                    )
                    + "\n"
                )
            if command[:4] == ["spin", "aka", "cron", "delete"]:
                return result(stdout="deleted\n")
            if command[:4] == ["spin", "aka", "cron", "create"]:
                return result(stdout="created\n")
            if command[:4] == ["spin", "aka", "cron", "list"] and len(calls) == 10:
                return result(
                    stdout="\n".join(
                        [
                            "+----------------------------+----------------+-------------------------+",
                            "| Name                       | Schedule       | Next Run                |",
                            "+=======================================================================+",
                            *[
                                f"| shuma-adversary-sim-beat-{index} | {schedule:<14} | 2026-03-12 13:05:00 UTC |"
                                for index, schedule in enumerate(deploy.EDGE_CRON_SCHEDULES)
                            ],
                            "+----------------------------+----------------+-------------------------+",
                        ]
                    )
                    + "\n"
                )
            raise AssertionError(f"Unexpected command: {command}")

        with patch.object(deploy, "run_command", side_effect=fake_run):
            cron = deploy.ensure_adversary_sim_edge_cron(
                env={"SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET": "test-edge-cron-secret"},
                app_id="app_123",
                account_id="acc_123",
                account_name="",
            )

        self.assertEqual(
            calls[0],
            ["spin", "aka", "cron", "list", "--app-id", "app_123", "--account-id", "acc_123"],
        )
        self.assertEqual(
            calls[1],
            [
                "spin",
                "aka",
                "cron",
                "delete",
                "--app-id",
                "app_123",
                "--account-id",
                "acc_123",
                "shuma-adversary-sim-beat",
            ],
        )
        self.assertEqual(
            calls[2],
            [
                "spin",
                "aka",
                "cron",
                "delete",
                "--app-id",
                "app_123",
                "--account-id",
                "acc_123",
                "shuma-adversary-sim-beat-0",
            ],
        )
        self.assertEqual(
            calls[3],
            [
                "spin",
                "aka",
                "cron",
                "delete",
                "--app-id",
                "app_123",
                "--account-id",
                "acc_123",
                "shuma-adversary-sim-beat-1",
            ],
        )
        self.assertEqual(
            calls[4][:8],
            ["spin", "aka", "cron", "create", "--app-id", "app_123", "--account-id", "acc_123"],
        )
        self.assertEqual(
            [calls[index][calls[index].index("--name") + 1] for index in range(4, 9)],
            [f"shuma-adversary-sim-beat-{index}" for index in range(5)],
        )
        self.assertEqual(
            [calls[index][calls[index].index("--schedule") + 1] for index in range(4, 9)],
            list(deploy.EDGE_CRON_SCHEDULES),
        )
        self.assertTrue(all("edge_cron_secret=test-edge-cron-secret" in calls[index][-1] for index in range(4, 9)))
        self.assertEqual(cron["job_name_prefix"], "shuma-adversary-sim-beat")
        self.assertEqual(cron["job_count"], 5)
        self.assertEqual(cron["schedules"], list(deploy.EDGE_CRON_SCHEDULES))
        self.assertEqual(cron["path_and_query"], "/internal/adversary-sim/beat?edge_cron_secret=<redacted>")

    def test_ensure_adversary_sim_edge_cron_tolerates_delete_when_job_is_already_gone(self) -> None:
        calls: list[list[str]] = []

        def fake_run(command, *, env=None, cwd=None, capture_output=True):
            calls.append(list(command))
            if command[:4] == ["spin", "aka", "cron", "list"] and len(calls) == 1:
                return result(
                    stdout="\n".join(
                        [
                            "+----------------------------+----------------+-------------------------+",
                            "| Name                       | Schedule       | Next Run                |",
                            "+=======================================================================+",
                            "| shuma-adversary-sim-beat-4 | 4-59/5 * * * * | 2026-03-12 13:09:00 UTC |",
                            "+----------------------------+----------------+-------------------------+",
                        ]
                    )
                    + "\n"
                )
            if command[:4] == ["spin", "aka", "cron", "delete"]:
                return result(returncode=1)
            if command[:4] == ["spin", "aka", "cron", "list"] and len(calls) == 3:
                return result(
                    stdout="\n".join(
                        [
                            "+----------------------------+----------------+-------------------------+",
                            "| Name                       | Schedule       | Next Run                |",
                            "+=======================================================================+",
                            "+----------------------------+----------------+-------------------------+",
                        ]
                    )
                    + "\n"
                )
            if command[:4] == ["spin", "aka", "cron", "create"]:
                return result(stdout="created\n")
            if command[:4] == ["spin", "aka", "cron", "list"] and len(calls) == 9:
                return result(
                    stdout="\n".join(
                        [
                            "+----------------------------+----------------+-------------------------+",
                            "| Name                       | Schedule       | Next Run                |",
                            "+=======================================================================+",
                            *[
                                f"| shuma-adversary-sim-beat-{index} | {schedule:<14} | 2026-03-12 13:05:00 UTC |"
                                for index, schedule in enumerate(deploy.EDGE_CRON_SCHEDULES)
                            ],
                            "+----------------------------+----------------+-------------------------+",
                        ]
                    )
                    + "\n"
                )
            raise AssertionError(f"Unexpected command: {command}")

        with patch.object(deploy, "run_command", side_effect=fake_run):
            cron = deploy.ensure_adversary_sim_edge_cron(
                env={"SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET": "test-edge-cron-secret"},
                app_id="app_123",
                account_id="acc_123",
                account_name="",
            )

        self.assertEqual(
            calls[1],
            [
                "spin",
                "aka",
                "cron",
                "delete",
                "--app-id",
                "app_123",
                "--account-id",
                "acc_123",
                "shuma-adversary-sim-beat-4",
            ],
        )
        self.assertEqual(
            calls[2],
            ["spin", "aka", "cron", "list", "--app-id", "app_123", "--account-id", "acc_123"],
        )
        self.assertEqual(cron["job_count"], 5)
        self.assertEqual(cron["schedules"], list(deploy.EDGE_CRON_SCHEDULES))

    def test_deploy_env_merges_defaults_env_before_env_file(self) -> None:
        defaults_file = self.temp_dir / "defaults.env"
        defaults_file.write_text(
            "\n".join(
                [
                    'SHUMA_MONITORING_RETENTION_HOURS="432"',
                    'SHUMA_MONITORING_ROLLUP_RETENTION_HOURS="720"',
                    "",
                ]
            ),
            encoding="utf-8",
        )
        merged = deploy.read_env_files(defaults_file, self.env_file)

        with patch.object(deploy, "DEFAULTS_ENV_FILE", defaults_file):
            env = deploy.deploy_env(
                deploy.load_receipt(self.setup_receipt),
                deploy.read_env_files(deploy.DEFAULTS_ENV_FILE, self.env_file),
                self.setup_receipt,
            )

        self.assertEqual(merged["SHUMA_MONITORING_RETENTION_HOURS"], "432")
        self.assertEqual(merged["SHUMA_MONITORING_ROLLUP_RETENTION_HOURS"], "720")
        self.assertEqual(env["SHUMA_MONITORING_RETENTION_HOURS"], "432")
        self.assertEqual(env["SHUMA_MONITORING_ROLLUP_RETENTION_HOURS"], "720")

    def test_deploy_env_requires_core_runtime_secrets(self) -> None:
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

        with self.assertRaises(SystemExit) as exc:
            deploy.deploy_env(
                deploy.load_receipt(self.setup_receipt),
                deploy.read_env_file(self.env_file),
                self.setup_receipt,
            )

        self.assertIn("Required env value is missing: SHUMA_API_KEY", str(exc.exception))

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
        interactive_calls: list[list[str]] = []

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
            if command[:4] == ["spin", "aka", "app", "status"]:
                return result(stdout='{"app":{"id":"app_existing_123"},"urls":["https://app.example.com"]}\n')
            if command[:3] == ["spin", "--version"]:
                return result(stdout="spin 3.5.1\n")
            if command[:3] == ["spin", "aka", "--version"]:
                return result(stdout="spin-aka 0.6.0\n")
            if command[:3] == ["git", "rev-parse", "HEAD"]:
                return result(stdout="deadbeef\n")
            raise AssertionError(f"Unexpected command: {command}")

        def fake_interactive(command, *, env=None, cwd=None):
            interactive_calls.append(list(command))
            return result(stdout="deploy ok\n")

        with patch.object(deploy, "ensure_aka_plugin", return_value="spin-aka 0.6.0"), patch.object(
            deploy, "run_command", side_effect=fake_run
        ), patch.object(
            deploy, "run_interactive_command", side_effect=fake_interactive
        ), patch.object(
            deploy, "bootstrap_remote_config_if_missing"
        ) as bootstrap_mock, patch.object(
            deploy, "smoke_deployed_app"
        ) as smoke_mock, patch.object(
            deploy, "wait_for_adversary_sim_control_lease_release"
        ) as lease_release_mock, patch.object(
            deploy, "smoke_external_dashboard"
        ) as external_smoke_mock, patch.object(
            deploy,
            "ensure_adversary_sim_edge_cron",
            return_value={"job_name_prefix": "shuma-adversary-sim-beat", "job_count": 5},
        ) as cron_mock, patch.object(
            deploy, "smoke_adversary_sim_generation"
        ) as sim_smoke_mock, patch.object(
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
        bootstrap_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        smoke_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        lease_release_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        cron_mock.assert_called_once_with(
            env=unittest.mock.ANY,
            app_id="app_existing_123",
            account_id="acc_123",
            account_name="",
        )
        sim_smoke_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        external_smoke_mock.assert_called_once_with("https://app.example.com", unittest.mock.ANY)
        self.assertEqual(len(interactive_calls), 1)
        deploy_call = interactive_calls[0]
        self.assertIn("--app-id", deploy_call)
        self.assertIn("app_existing_123", deploy_call)
        self.assertNotIn("--create-name", deploy_call)

        updated_receipt = json.loads(self.deploy_receipt.read_text(encoding="utf-8"))
        self.assertEqual(updated_receipt["fermyon"]["app_id"], "app_existing_123")
        self.assertEqual(updated_receipt["fermyon"]["primary_url"], "https://app.example.com")
        self.assertEqual(updated_receipt["fermyon"]["cron"]["job_name_prefix"], "shuma-adversary-sim-beat")
        self.assertEqual(updated_receipt["fermyon"]["cron"]["job_count"], 5)

    def test_smoke_adversary_sim_generation_requires_monitoring_visibility(self) -> None:
        responses = iter(
            [
                (200, {"window_end_cursor": "cursor-0"}, '{"window_end_cursor":"cursor-0"}'),
                (200, {"requested_enabled": True}, '{"requested_enabled":true}'),
                (200, {"generation": {"tick_count": 0, "request_count": 0}}, '{"generation":{"tick_count":0,"request_count":0}}'),
                (200, {"generation": {"tick_count": 1, "request_count": 40}}, '{"generation":{"tick_count":1,"request_count":40}}'),
                (200, {"events": []}, '{"events":[]}'),
                (200, {"adversary_sim_enabled": False}, '{"adversary_sim_enabled":false}'),
            ]
        )

        with patch.object(deploy, "admin_session_opener", return_value=(object(), "csrf")), patch.object(
            deploy, "admin_json_request", side_effect=lambda **_: next(responses)
        ), patch.object(
            deploy.time, "time", side_effect=[0, 0, deploy.EDGE_ADVERSARY_SIM_SMOKE_TIMEOUT_SECONDS + 1]
        ), patch.object(
            deploy.time, "sleep"
        ):
            with self.assertRaises(SystemExit) as exc:
                deploy.smoke_adversary_sim_generation("https://edge.example.com", {})

        self.assertIn("Last monitoring delta", str(exc.exception))

    def test_smoke_adversary_sim_generation_accepts_monitoring_visible_sim_events(self) -> None:
        calls: list[str] = []
        responses = iter(
            [
                (200, {"window_end_cursor": "cursor-0"}, '{"window_end_cursor":"cursor-0"}'),
                (
                    200,
                    {
                        "requested_enabled": True,
                        "status": {},
                    },
                    '{"requested_enabled":true,"status":{}}',
                ),
                (
                    200,
                    {
                        "lifecycle_diagnostics": {
                            "supervisor": {
                                "generated_tick_count": 1,
                                "generated_request_count": 40,
                                "last_successful_beat_at": 100,
                            }
                        }
                    },
                    '{"lifecycle_diagnostics":{"supervisor":{"generated_tick_count":1,"generated_request_count":40,"last_successful_beat_at":100}}}',
                ),
                (
                    200,
                    {
                        "lifecycle_diagnostics": {
                            "supervisor": {
                                "generated_tick_count": 2,
                                "generated_request_count": 64,
                                "last_successful_beat_at": 200,
                            }
                        }
                    },
                    '{"lifecycle_diagnostics":{"supervisor":{"generated_tick_count":2,"generated_request_count":64,"last_successful_beat_at":200}}}',
                ),
                (200, {"events": [{"is_simulation": True, "event": "Challenge"}]}, '{"events":[{"is_simulation":true}]}'),
                (200, {"adversary_sim_enabled": True}, '{"adversary_sim_enabled":true}'),
                (200, {"disabled": True}, '{"disabled":true}'),
            ]
        )

        def fake_admin_json_request(*, url, **kwargs):
            calls.append(url)
            return next(responses)

        with patch.object(deploy, "admin_session_opener", return_value=(object(), "csrf")), patch.object(
            deploy, "admin_json_request", side_effect=fake_admin_json_request
        ), patch.object(
            deploy.time, "time", side_effect=[0, 0, 0]
        ), patch.object(
            deploy.time, "sleep"
        ):
            deploy.smoke_adversary_sim_generation("https://edge.example.com", {})

        self.assertTrue(
            any("/admin/monitoring/delta?hours=24&limit=20&after_cursor=cursor-0" in url for url in calls)
        )

    def test_smoke_adversary_sim_generation_uses_extended_timeout_for_control_posts(self) -> None:
        calls: list[tuple[str, int]] = []
        responses = iter(
            [
                (200, {"window_end_cursor": "cursor-0"}, '{"window_end_cursor":"cursor-0"}'),
                (200, {"requested_enabled": True}, '{"requested_enabled":true}'),
                (
                    200,
                    {
                        "lifecycle_diagnostics": {
                            "supervisor": {
                                "generated_tick_count": 1,
                                "generated_request_count": 40,
                                "last_successful_beat_at": 100,
                            }
                        }
                    },
                    '{"lifecycle_diagnostics":{"supervisor":{"generated_tick_count":1,"generated_request_count":40,"last_successful_beat_at":100}}}',
                ),
                (
                    200,
                    {
                        "lifecycle_diagnostics": {
                            "supervisor": {
                                "generated_tick_count": 2,
                                "generated_request_count": 64,
                                "last_successful_beat_at": 200,
                            }
                        }
                    },
                    '{"lifecycle_diagnostics":{"supervisor":{"generated_tick_count":2,"generated_request_count":64,"last_successful_beat_at":200}}}',
                ),
                (200, {"events": [{"is_simulation": True}]}, '{"events":[{"is_simulation":true}]}'),
                (200, {"adversary_sim_enabled": True}, '{"adversary_sim_enabled":true}'),
                (200, {"disabled": True}, '{"disabled":true}'),
            ]
        )

        def fake_admin_json_request(*, url, timeout_seconds=30, **kwargs):
            calls.append((url, timeout_seconds))
            return next(responses)

        with patch.object(deploy, "admin_session_opener", return_value=(object(), "csrf")), patch.object(
            deploy, "admin_json_request", side_effect=fake_admin_json_request
        ), patch.object(
            deploy.time, "time", side_effect=[0, 0, 0]
        ), patch.object(
            deploy.time, "sleep"
        ):
            deploy.smoke_adversary_sim_generation("https://edge.example.com", {})

        control_timeouts = [
            timeout_seconds
            for url, timeout_seconds in calls
            if url == "https://edge.example.com/admin/adversary-sim/control"
        ]
        self.assertEqual(
            control_timeouts,
            [
                deploy.EDGE_ADVERSARY_SIM_CONTROL_TIMEOUT_SECONDS,
                deploy.EDGE_ADVERSARY_SIM_CONTROL_TIMEOUT_SECONDS,
            ],
        )

    def test_smoke_adversary_sim_generation_requires_follow_up_tick_beyond_prime(self) -> None:
        responses = iter(
            [
                (200, {"window_end_cursor": "cursor-0"}, '{"window_end_cursor":"cursor-0"}'),
                (
                    200,
                    {
                        "requested_enabled": True,
                        "status": {},
                    },
                    '{"requested_enabled":true,"status":{}}',
                ),
                (
                    200,
                    {
                        "lifecycle_diagnostics": {
                            "supervisor": {
                                "generated_tick_count": 1,
                                "generated_request_count": 40,
                                "last_successful_beat_at": 100,
                            }
                        }
                    },
                    '{"lifecycle_diagnostics":{"supervisor":{"generated_tick_count":1,"generated_request_count":40,"last_successful_beat_at":100}}}',
                ),
                (
                    200,
                    {
                        "lifecycle_diagnostics": {
                            "supervisor": {
                                "generated_tick_count": 1,
                                "generated_request_count": 40,
                                "last_successful_beat_at": 100,
                            }
                        }
                    },
                    '{"lifecycle_diagnostics":{"supervisor":{"generated_tick_count":1,"generated_request_count":40,"last_successful_beat_at":100}}}',
                ),
                (200, {"events": [{"is_simulation": True, "event": "Challenge"}]}, '{"events":[{"is_simulation":true}]}'),
                (200, {"adversary_sim_enabled": True}, '{"adversary_sim_enabled":true}'),
                (200, {"disabled": True}, '{"disabled":true}'),
            ]
        )

        with patch.object(deploy, "admin_session_opener", return_value=(object(), "csrf")), patch.object(
            deploy, "admin_json_request", side_effect=lambda **_: next(responses)
        ), patch.object(
            deploy.time, "time", side_effect=[0, 0, 200]
        ), patch.object(
            deploy.time, "sleep"
        ):
            with self.assertRaises(SystemExit) as exc:
                deploy.smoke_adversary_sim_generation("https://edge.example.com", {})

        self.assertIn("no generated traffic was observed", str(exc.exception))

    def test_smoke_adversary_sim_generation_accepts_first_cron_tick_without_prime(self) -> None:
        responses = iter(
            [
                (200, {"window_end_cursor": "cursor-0"}, '{"window_end_cursor":"cursor-0"}'),
                (
                    200,
                    {
                        "requested_enabled": True,
                        "status": {},
                    },
                    '{"requested_enabled":true,"status":{}}',
                ),
                (
                    200,
                    {
                        "lifecycle_diagnostics": {
                            "supervisor": {
                                "generated_tick_count": 0,
                                "generated_request_count": 0,
                                "last_successful_beat_at": 0,
                            }
                        }
                    },
                    '{"lifecycle_diagnostics":{"supervisor":{"generated_tick_count":0,"generated_request_count":0,"last_successful_beat_at":0}}}',
                ),
                (
                    200,
                    {
                        "lifecycle_diagnostics": {
                            "supervisor": {
                                "generated_tick_count": 1,
                                "generated_request_count": 40,
                                "last_successful_beat_at": 100,
                            }
                        }
                    },
                    '{"lifecycle_diagnostics":{"supervisor":{"generated_tick_count":1,"generated_request_count":40,"last_successful_beat_at":100}}}',
                ),
                (200, {"events": [{"is_simulation": True, "event": "Challenge"}]}, '{"events":[{"is_simulation":true}]}'),
                (200, {"adversary_sim_enabled": True}, '{"adversary_sim_enabled":true}'),
                (200, {"disabled": True}, '{"disabled":true}'),
            ]
        )

        with patch.object(deploy, "admin_session_opener", return_value=(object(), "csrf")), patch.object(
            deploy, "admin_json_request", side_effect=lambda **_: next(responses)
        ), patch.object(
            deploy.time, "time", side_effect=[0, 0, 0]
        ), patch.object(
            deploy.time, "sleep"
        ):
            deploy.smoke_adversary_sim_generation("https://edge.example.com", {})

    def test_bootstrap_remote_config_posts_seeded_json_when_admin_config_missing(self) -> None:
        env = {
            "SHUMA_API_KEY": "test-admin-key",
        }
        http_calls: list[tuple[str, str, dict[str, str] | None, bytes | None]] = []

        def fake_http(*, method, url, headers=None, body=None, timeout_seconds=30):
            http_calls.append((method, url, headers, body))
            if len(http_calls) == 1:
                return 500, "Configuration unavailable (missing KV config; run setup/config-seed)"
            if len(http_calls) == 2:
                return 200, '{"ok":true}'
            if len(http_calls) == 3:
                return 200, '{"config":{"rate_limit":321}}'
            raise AssertionError(f"Unexpected HTTP call {len(http_calls)}")

        with patch.object(deploy, "http_text_request", side_effect=fake_http), patch.object(
            deploy, "export_seeded_config_payload", return_value={"rate_limit": 321}
        ):
            deploy.bootstrap_remote_config_if_missing("https://app.example.com", env)

        self.assertEqual(http_calls[0][0], "GET")
        self.assertEqual(http_calls[0][1], "https://app.example.com/admin/config")
        self.assertEqual(http_calls[1][0], "POST")
        self.assertEqual(http_calls[1][1], "https://app.example.com/admin/config/bootstrap")
        self.assertEqual(
            json.loads(http_calls[1][3].decode("utf-8")),
            {"rate_limit": 321},
        )

    def test_smoke_deployed_app_rejects_configuration_unavailable_public_route(self) -> None:
        env = {
            "SHUMA_API_KEY": "test-admin-key",
        }

        responses = iter(
            [
                (200, "<!doctype html><html></html>"),
                (500, "Configuration unavailable"),
            ]
        )

        def fake_http(*, method, url, headers=None, body=None, timeout_seconds=30):
            return next(responses)

        with patch.object(deploy, "http_text_request", side_effect=fake_http):
            with self.assertRaises(SystemExit) as exc:
                deploy.smoke_deployed_app("https://app.example.com", env)

        self.assertIn("Edge public-route smoke failed", str(exc.exception))

    def test_smoke_deployed_app_accepts_defended_public_route(self) -> None:
        env = {
            "SHUMA_API_KEY": "test-admin-key",
        }

        responses = iter(
            [
                (200, "<!doctype html><html></html>"),
                (403, "<!DOCTYPE html><html><body><h1>Access Blocked</h1></body></html>"),
                (200, '{"config":{"rate_limit":120}}'),
            ]
        )

        def fake_http(*, method, url, headers=None, body=None, timeout_seconds=30):
            return next(responses)

        with patch.object(deploy, "http_text_request", side_effect=fake_http):
            deploy.smoke_deployed_app("https://app.example.com", env)
