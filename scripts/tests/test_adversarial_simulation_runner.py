import unittest
import os
import json
import subprocess
from pathlib import Path
from unittest.mock import patch

import scripts.tests.adversarial_simulation_runner as runner


def minimal_manifest(gates_extra=None, schema_version="sim-manifest.v1"):
    gates = {
        "latency": {"p95_max_ms": 1000},
        "outcome_ratio_bounds": {"allow": {"min": 0.0, "max": 1.0}},
        "telemetry_amplification": {
            "max_fingerprint_events_per_request": 1.0,
            "max_monitoring_events_per_request": 1.0,
        },
    }
    if gates_extra:
        gates.update(gates_extra)
    scenario = {
        "id": "scenario_allow",
        "description": "allow scenario",
        "tier": "SIM-T0",
        "driver": "allow_browser_allowlist",
        "expected_outcome": "allow",
        "ip": "10.10.10.10",
        "user_agent": "UnitTest/1.0",
        "seed": 1,
        "runtime_budget_ms": 1000,
        "assertions": {"max_latency_ms": 500},
    }
    if schema_version == "sim-manifest.v2":
        scenario["driver_class"] = "browser_realistic"
        scenario["traffic_model"] = {
            "persona": "human_like",
            "think_time_ms_min": 10,
            "think_time_ms_max": 120,
            "retry_strategy": "single_attempt",
            "cookie_behavior": "stateful_cookie_jar",
        }
        scenario["expected_defense_categories"] = ["allowlist"]
        scenario["coverage_tags"] = ["SIM-T0", "allow_browser_allowlist"]
        scenario["cost_assertions"] = {"max_latency_ms": 500}

    return {
        "schema_version": schema_version,
        "suite_id": "unit-tests",
        "description": "Unit test manifest",
        "execution_lane": "black_box",
        "profiles": {
            "test_profile": {
                "description": "test profile",
                "max_runtime_seconds": 60,
                "scenario_ids": ["scenario_allow"],
                "gates": gates,
            }
        },
        "scenarios": [scenario],
    }


class AdversarialRunnerUnitTests(unittest.TestCase):
    def test_forwarded_headers_include_simulation_metadata(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )
            headers = sim_runner.forwarded_headers("10.0.0.11", user_agent="UnitTest/1.0")
            self.assertIn(runner.SIM_TAG_HEADER_RUN_ID, headers)
            self.assertEqual(headers.get(runner.SIM_TAG_HEADER_PROFILE), "test_profile")
            self.assertEqual(headers.get(runner.SIM_TAG_HEADER_LANE), "deterministic_black_box")
            self.assertIn(runner.SIM_TAG_HEADER_TIMESTAMP, headers)
            self.assertIn(runner.SIM_TAG_HEADER_NONCE, headers)
            self.assertIn(runner.SIM_TAG_HEADER_SIGNATURE, headers)
            self.assertNotIn("X-Shuma-Forwarded-Secret", headers)

    def test_attacker_request_trusted_forwarded_injects_secret_only_on_opt_in(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        captured_header_sets = []

        class _Response:
            def read(self):
                return b"ok"

            @property
            def headers(self):
                return {}

            def getcode(self):
                return 200

            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

        class _Opener:
            def open(self, req, timeout=None):
                captured_header_sets.append(
                    {str(key).lower(): str(value) for key, value in req.header_items()}
                )
                return _Response()

        sim_runner.opener = _Opener()  # type: ignore[assignment]
        headers = sim_runner.forwarded_headers("10.0.0.11", user_agent="UnitTest/1.0")

        plain = sim_runner.request(
            "GET",
            "/",
            headers=headers,
            plane="attacker",
            count_request=False,
            trusted_forwarded=False,
        )
        trusted = sim_runner.request(
            "GET",
            "/",
            headers=headers,
            plane="attacker",
            count_request=False,
            trusted_forwarded=True,
        )

        self.assertEqual(plain.status, 200)
        self.assertEqual(trusted.status, 200)
        self.assertEqual(len(captured_header_sets), 2)
        self.assertNotIn("x-shuma-forwarded-secret", captured_header_sets[0])
        self.assertEqual(
            captured_header_sets[1].get("x-shuma-forwarded-secret"),
            "forwarded-secret",
        )

    def test_deterministic_helper_requests_opt_into_trusted_forwarded_identity(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        scenario = {
            "id": "sim_t1_pow_success",
            "ip": "10.0.0.77",
            "user_agent": "UnitTest/1.0",
        }
        calls = []

        def _request(method, path, headers=None, json_body=None, form_body=None, count_request=False, trusted_forwarded=False):
            calls.append(
                {
                    "method": method,
                    "path": path,
                    "count_request": count_request,
                    "trusted_forwarded": trusted_forwarded,
                    "json_body": json_body,
                    "form_body": form_body,
                }
            )
            if path == "/pow":
                return runner.HttpResult(
                    status=200,
                    body=json.dumps({"seed": "pow-seed", "difficulty": 3}),
                    headers={},
                    latency_ms=1,
                )
            if path == "/pow/verify":
                return runner.HttpResult(
                    status=200,
                    body=json.dumps({"status": "ok"}),
                    headers={},
                    latency_ms=1,
                )
            if path == "/challenge/not-a-bot-checkbox":
                return runner.HttpResult(
                    status=200,
                    body=(
                        '<h2>I am not a bot</h2>'
                        '<input type="hidden" name="seed" value="nab-seed">'
                    ),
                    headers={},
                    latency_ms=1,
                )
            if path == "/challenge/puzzle":
                if method == "GET":
                    return runner.HttpResult(
                        status=200,
                        body=(
                            '<h2>Puzzle</h2>'
                            '<input type="hidden" name="seed" value="puzzle-seed">'
                            '<input type="hidden" name="output" value="0000">'
                        ),
                        headers={},
                        latency_ms=1,
                    )
                return runner.HttpResult(status=200, body="ok", headers={}, latency_ms=1)
            raise AssertionError(f"unexpected helper request path: {path}")

        sim_runner.attacker_client.request = _request  # type: ignore[assignment]

        seed, difficulty = sim_runner.fetch_pow_seed(scenario)
        verify_result = sim_runner.submit_pow_verify(seed, "nonce", scenario)
        not_a_bot_seed, not_a_bot_page = sim_runner.fetch_not_a_bot_seed(scenario)
        not_a_bot_submit = sim_runner.submit_not_a_bot(
            not_a_bot_seed,
            scenario,
            telemetry={"checked": True},
        )
        puzzle_seed, puzzle_output = sim_runner.fetch_challenge_puzzle_seed_and_output(scenario)
        puzzle_submit = sim_runner.submit_challenge_puzzle(puzzle_seed, puzzle_output, scenario)

        self.assertEqual(seed, "pow-seed")
        self.assertEqual(difficulty, 3)
        self.assertEqual(verify_result.status, 200)
        self.assertEqual(not_a_bot_seed, "nab-seed")
        self.assertEqual(not_a_bot_page.status, 200)
        self.assertEqual(not_a_bot_submit.status, 200)
        self.assertEqual(puzzle_seed, "puzzle-seed")
        self.assertEqual(puzzle_output, "0000")
        self.assertEqual(puzzle_submit.status, 200)

        self.assertEqual(
            [(call["method"], call["path"]) for call in calls],
            [
                ("GET", "/pow"),
                ("POST", "/pow/verify"),
                ("GET", "/challenge/not-a-bot-checkbox"),
                ("POST", "/challenge/not-a-bot-checkbox"),
                ("GET", "/challenge/puzzle"),
                ("POST", "/challenge/puzzle"),
            ],
        )
        self.assertTrue(all(call["count_request"] for call in calls))
        self.assertTrue(all(call["trusted_forwarded"] for call in calls))

    def test_control_plane_headers_split_health_loopback_and_admin_isolation(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

            admin_headers = sim_runner.control_client.admin_headers()
            admin_headers_2 = sim_runner.control_client.admin_headers()
            health_headers = sim_runner.control_client.health_headers()

            self.assertEqual(health_headers.get("X-Forwarded-For"), "127.0.0.1")
            self.assertRegex(str(admin_headers.get("X-Forwarded-For")), r"^127\.0\.\d+\.\d+$")
            self.assertRegex(str(admin_headers_2.get("X-Forwarded-For")), r"^127\.0\.\d+\.\d+$")
            self.assertNotEqual(
                admin_headers.get("X-Forwarded-For"),
                admin_headers_2.get("X-Forwarded-For"),
            )

    def test_control_plane_requests_use_separate_timeout_budget(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        captured: dict[str, object] = {}

        class _Response:
            def read(self):
                return b"{}"

            @property
            def headers(self):
                return {}

            def getcode(self):
                return 200

            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

        class _Opener:
            def open(self, req, timeout=None):
                captured["timeout"] = timeout
                captured["url"] = req.full_url
                return _Response()

        sim_runner.opener = _Opener()  # type: ignore[assignment]

        result = sim_runner.admin_request("GET", "/admin/config")

        self.assertEqual(result.status, 200)
        self.assertEqual(sim_runner.control_plane_request_timeout_seconds, 30.0)
        self.assertGreater(sim_runner.control_plane_request_timeout_seconds, sim_runner.request_timeout_seconds)
        self.assertEqual(captured["timeout"], sim_runner.control_plane_request_timeout_seconds)
        self.assertEqual(captured["url"], "http://127.0.0.1:3000/admin/config")

    def test_admin_read_request_retries_transient_timeout_failures(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        calls = {"count": 0}

        def _admin_request(method, path, json_body=None, headers=None, timeout_seconds=None):
            calls["count"] += 1
            if calls["count"] == 1:
                raise runner.SimulationError(
                    "HTTP request failed for GET http://127.0.0.1:3000/admin/monitoring/delta?hours=24&limit=40: "
                    "<urlopen error timed out>"
                )
            return runner.HttpResult(status=200, body="{}", headers={}, latency_ms=1)

        sim_runner.admin_request = _admin_request  # type: ignore[assignment]

        result = sim_runner.admin_read_request(
            "GET",
            "/admin/monitoring/delta?hours=24&limit=40",
            timeout_seconds=60.0,
            max_attempts=3,
        )

        self.assertEqual(result.status, 200)
        self.assertEqual(calls["count"], 2)

    def test_simulation_event_snapshot_uses_observation_timeout_budget(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        captured: dict[str, object] = {}

        def _admin_read_request(method, path, json_body=None, max_attempts=4, timeout_seconds=None):
            captured["method"] = method
            captured["path"] = path
            captured["max_attempts"] = max_attempts
            captured["timeout_seconds"] = timeout_seconds
            return runner.HttpResult(
                status=200,
                body=json.dumps({"events": [], "recent_sim_runs": []}),
                headers={},
                latency_ms=1,
            )

        sim_runner.admin_read_request = _admin_read_request  # type: ignore[assignment]

        snapshot = sim_runner.simulation_event_snapshot(hours=24, limit=1000)

        self.assertEqual(snapshot["count"], 0)
        self.assertEqual(captured["method"], "GET")
        self.assertEqual(
            captured["path"],
            (
                f"/admin/monitoring/delta?hours={sim_runner.monitoring_hot_read_window_hours}"
                f"&limit={sim_runner.monitoring_delta_limit}"
            ),
        )
        self.assertEqual(captured["timeout_seconds"], sim_runner.observation_request_timeout_seconds)

    def test_simulation_event_snapshot_prefers_recent_sim_run_summary_count(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.admin_read_request = lambda method, path, json_body=None, max_attempts=4, timeout_seconds=None: runner.HttpResult(  # type: ignore[assignment]
            status=200,
            body=json.dumps(
                {
                    "events": [
                        {
                            "reason": "geo_policy_block",
                            "is_simulation": True,
                            "sim_run_id": sim_runner.sim_run_id,
                        },
                        {
                            "reason": "ignore_me",
                            "is_simulation": True,
                            "sim_run_id": "other-run",
                        },
                    ],
                    "recent_sim_runs": [
                        {
                            "run_id": sim_runner.sim_run_id,
                            "monitoring_event_count": 7,
                        }
                    ],
                }
            ),
            headers={},
            latency_ms=1,
        )

        snapshot = sim_runner.simulation_event_snapshot(hours=24, limit=1000)

        self.assertEqual(snapshot["count"], 7)
        self.assertEqual(snapshot["reasons"], ["geo_policy_block"])
        self.assertEqual(snapshot["reason_counts"], {"geo_policy_block": 1})

    def test_monitoring_snapshot_uses_bootstrap_hot_read_timeout_budget(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        captured: dict[str, object] = {}

        def _admin_read_request(method, path, json_body=None, max_attempts=4, timeout_seconds=None):
            captured["method"] = method
            captured["path"] = path
            captured["timeout_seconds"] = timeout_seconds
            return runner.HttpResult(
                status=200,
                body=json.dumps({"summary": {}, "details": {"events": {"recent_events": []}}}),
                headers={},
                latency_ms=1,
            )

        sim_runner.admin_read_request = _admin_read_request  # type: ignore[assignment]

        snapshot = sim_runner.monitoring_snapshot()

        self.assertEqual(snapshot["monitoring_total"], 0)
        self.assertEqual(captured["method"], "GET")
        self.assertEqual(
            captured["path"],
            (
                f"/admin/monitoring?hours={sim_runner.monitoring_hot_read_window_hours}"
                f"&limit={sim_runner.monitoring_bootstrap_limit}&bootstrap=1"
            ),
        )
        self.assertEqual(captured["timeout_seconds"], sim_runner.observation_request_timeout_seconds)

    def test_admin_unban_retries_transient_timeout_failures(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        calls = {"count": 0}
        captured: dict[str, object] = {}

        def _admin_request(method, path, json_body=None, headers=None, timeout_seconds=None):
            calls["count"] += 1
            captured["timeout_seconds"] = timeout_seconds
            if calls["count"] == 1:
                raise runner.SimulationError(
                    "HTTP request failed for POST http://127.0.0.1:3000/admin/unban?ip=unknown: "
                    "<urlopen error timed out>"
                )
            return runner.HttpResult(status=200, body="{}", headers={}, latency_ms=1)

        sim_runner.admin_request = _admin_request  # type: ignore[assignment]

        with patch("scripts.tests.adversarial_simulation_runner.time.sleep", return_value=None) as sleep_mock:
            sim_runner.admin_unban("unknown", reason="cleanup_ips")

        self.assertEqual(calls["count"], 2)
        self.assertEqual(captured["timeout_seconds"], sim_runner.control_plane_write_timeout_seconds)
        sleep_mock.assert_called_once()

    def test_admin_patch_uses_control_plane_write_timeout_budget(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        captured: dict[str, object] = {}

        def _admin_request(method, path, json_body=None, headers=None, timeout_seconds=None):
            captured["method"] = method
            captured["path"] = path
            captured["timeout_seconds"] = timeout_seconds
            return runner.HttpResult(
                status=200,
                body=json.dumps({"status": "updated"}),
                headers={},
                latency_ms=1,
            )

        sim_runner.admin_request = _admin_request  # type: ignore[assignment]

        sim_runner.admin_patch({"shadow_mode": False})

        self.assertEqual(captured["method"], "POST")
        self.assertEqual(captured["path"], "/admin/config")
        self.assertEqual(captured["timeout_seconds"], sim_runner.control_plane_write_timeout_seconds)

    def test_execution_phase_transitions_record_suite_contract(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        self.assertEqual(sim_runner.execution_phase, runner.SUITE_PHASE_SETUP)
        sim_runner.set_execution_phase(
            runner.SUITE_PHASE_ATTACKER_EXECUTION, "unit_test_attacker_start"
        )
        sim_runner.set_execution_phase(runner.SUITE_PHASE_TEARDOWN, "unit_test_teardown")

        observed_phases = [entry.get("phase") for entry in sim_runner.execution_phase_trace]
        self.assertEqual(
            observed_phases,
            [
                runner.SUITE_PHASE_SETUP,
                runner.SUITE_PHASE_ATTACKER_EXECUTION,
                runner.SUITE_PHASE_TEARDOWN,
            ],
        )

    def test_admin_patch_records_control_plane_mutation_audit(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.admin_request = (  # type: ignore[assignment]
            lambda method, path, json_body=None, headers=None, timeout_seconds=None: runner.HttpResult(
                status=200,
                body=json.dumps({"status": "updated"}),
                headers={},
                latency_ms=1,
            )
        )
        sim_runner.set_execution_phase(runner.SUITE_PHASE_SETUP, "unit_test_setup_phase")
        sim_runner.admin_patch({"shadow_mode": True}, reason="unit_test_patch")

        self.assertEqual(len(sim_runner.control_plane_mutations), 1)
        mutation = sim_runner.control_plane_mutations[0]
        self.assertEqual(mutation.get("action"), "admin_config_patch")
        self.assertEqual(mutation.get("phase"), runner.SUITE_PHASE_SETUP)
        self.assertEqual(mutation.get("reason"), "unit_test_patch")
        self.assertEqual(
            runner.dict_or_empty(mutation.get("details")).get("keys"),
            ["shadow_mode"],
        )

    def test_admin_patch_rejects_control_plane_mutation_during_attacker_phase(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.set_execution_phase(
            runner.SUITE_PHASE_ATTACKER_EXECUTION, "unit_test_attack_phase"
        )
        with self.assertRaises(runner.SimulationError):
            sim_runner.admin_patch({"shadow_mode": True}, reason="unit_test_forbidden_patch")

    def test_admin_patch_rejects_setup_mutation_after_attacker_phase_started(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.set_execution_phase(
            runner.SUITE_PHASE_ATTACKER_EXECUTION, "unit_test_attack_phase"
        )
        sim_runner.set_execution_phase(
            runner.SUITE_PHASE_SETUP, "unit_test_illegal_setup_after_attack"
        )
        with self.assertRaises(runner.SimulationError):
            sim_runner.admin_patch({"shadow_mode": False}, reason="unit_test_late_setup_patch")

    def test_cleanup_simulation_telemetry_history_records_mutation_audit(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        captured_headers = {}

        def _admin_request(method, path, json_body=None, headers=None, timeout_seconds=None):
            captured_headers.clear()
            captured_headers.update(headers or {})
            return runner.HttpResult(
                status=200,
                body=json.dumps({"status": "cleared"}),
                headers={},
                latency_ms=1,
            )

        sim_runner.admin_request = _admin_request  # type: ignore[assignment]
        sim_runner.set_execution_phase(runner.SUITE_PHASE_SETUP, "unit_test_cleanup_phase")
        sim_runner.cleanup_simulation_telemetry_history()

        self.assertEqual(len(sim_runner.control_plane_mutations), 1)
        mutation = sim_runner.control_plane_mutations[0]
        self.assertEqual(mutation.get("action"), "telemetry_history_cleanup")
        self.assertEqual(mutation.get("phase"), runner.SUITE_PHASE_SETUP)
        self.assertEqual(mutation.get("reason"), "adversarial_ephemeral_cleanup")
        self.assertEqual(
            captured_headers.get(runner.TELEMETRY_CLEANUP_ACK_HEADER),
            runner.TELEMETRY_CLEANUP_ACK_VALUE,
        )

    def test_cleanup_simulation_telemetry_history_fails_on_non_200_response(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.admin_request = (  # type: ignore[assignment]
            lambda method, path, json_body=None, headers=None, timeout_seconds=None: runner.HttpResult(
                status=404,
                body="Not Found",
                headers={},
                latency_ms=1,
            )
        )

        with self.assertRaises(runner.SimulationError):
            sim_runner.cleanup_simulation_telemetry_history()

    def test_build_frontier_metadata_reports_disabled_single_and_multi_modes(self):
        with patch("scripts.tests.adversarial_simulation_runner.read_env_local_value", return_value=""):
            with patch.dict("os.environ", {}, clear=True):
                disabled = runner.build_frontier_metadata()
                self.assertEqual(disabled["frontier_mode"], "disabled")
                self.assertEqual(disabled["provider_count"], 0)
                self.assertEqual(disabled["diversity_confidence"], "none")
                self.assertFalse(disabled["reduced_diversity_warning"])

            with patch.dict(
                "os.environ",
                {"SHUMA_FRONTIER_OPENAI_API_KEY": "sk-openai", "SHUMA_FRONTIER_OPENAI_MODEL": "gpt-5-mini"},
                clear=True,
            ):
                single = runner.build_frontier_metadata()
                self.assertEqual(single["frontier_mode"], "single_provider_self_play")
                self.assertEqual(single["provider_count"], 1)
                self.assertEqual(single["diversity_confidence"], "low")
                self.assertTrue(single["reduced_diversity_warning"])

            with patch.dict(
                "os.environ",
                {
                    "SHUMA_FRONTIER_OPENAI_API_KEY": "sk-openai",
                    "SHUMA_FRONTIER_ANTHROPIC_API_KEY": "sk-anthropic",
                },
                clear=True,
            ):
                multi = runner.build_frontier_metadata()
                self.assertEqual(multi["frontier_mode"], "multi_provider_playoff")
                self.assertEqual(multi["provider_count"], 2)
                self.assertEqual(multi["diversity_confidence"], "higher")
                self.assertFalse(multi["reduced_diversity_warning"])

    def test_frontier_payload_sanitization_drops_forbidden_and_masks_quasi_identifiers(self):
        payload = {
            "schema_version": "frontier_payload_schema.v1",
            "request_id": "req-1",
            "profile": "fast_smoke",
            "scenario": {
                "id": "scenario_allow",
                "ip": "203.0.113.10",
                "api_key": "must-not-leak",
            },
            "traffic_model": {"cohort": "adversarial", "session_token": "must-drop"},
            "target": {"base_url": "http://127.0.0.1:3000", "authorization": "Bearer leaked"},
            "public_crawl_content": {"snippet": "public text"},
            "attack_metadata": {"cookie": "session=abc123"},
        }

        sanitized = runner.sanitize_frontier_payload(payload)
        serialized = str(sanitized)
        self.assertNotIn("must-not-leak", serialized)
        self.assertNotIn("session=abc123", serialized)
        self.assertNotIn("Bearer leaked", serialized)
        self.assertEqual(sanitized["scenario"]["ip"], "[masked]")

    def test_frontier_payload_schema_rejects_unknown_top_level_keys(self):
        invalid_payload = {
            "schema_version": "frontier_payload_schema.v1",
            "request_id": "req-1",
            "profile": "fast_smoke",
            "scenario": {"id": "scenario_allow"},
            "traffic_model": {"cohort": "adversarial"},
            "target": {"base_url": "http://127.0.0.1:3000"},
            "public_crawl_content": {"snippet": "public text"},
            "attack_metadata": {"note": "ok"},
            "forbidden_top_level": {"api_key": "secret"},
        }
        with self.assertRaises(runner.SimulationError):
            runner.validate_frontier_payload_schema(invalid_payload)

    def test_build_attack_plan_emits_seed_and_generated_candidates(self):
        frontier = {
            "frontier_mode": "disabled",
            "provider_count": 0,
            "providers": [],
            "diversity_confidence": "none",
        }
        scenarios = [
            {
                "id": "scenario_allow",
                "tier": "SIM-T0",
                "driver": "allow_browser_allowlist",
                "expected_outcome": "allow",
                "runtime_budget_ms": 1000,
                "seed": 1,
                "ip": "198.51.100.10",
                "user_agent": "UnitTest/1.0",
                "description": "allow scenario",
            }
        ]
        attack_plan = runner.build_attack_plan(
            profile_name="fast_smoke",
            execution_lane="black_box",
            base_url="http://127.0.0.1:3000",
            scenarios=scenarios,
            frontier_metadata=frontier,
            generated_at_unix=1234,
        )

        self.assertEqual(attack_plan["schema_version"], "attack-plan.v1")
        self.assertEqual(attack_plan["execution_lane"], "black_box")
        self.assertEqual(attack_plan["frontier_mode"], "disabled")
        self.assertGreaterEqual(len(attack_plan["candidates"]), 2)

        seed_candidates = [
            candidate
            for candidate in attack_plan["candidates"]
            if str(candidate.get("generation_kind") or "") == "seed"
        ]
        generated_candidates = [
            candidate
            for candidate in attack_plan["candidates"]
            if str(candidate.get("generation_kind") or "") == "mutation"
        ]
        self.assertEqual(len(seed_candidates), 1)
        self.assertGreaterEqual(len(generated_candidates), 1)

        candidate_payload = seed_candidates[0]["payload"]
        self.assertEqual(candidate_payload["schema_version"], "frontier_payload_schema.v1")
        self.assertEqual(candidate_payload["scenario"]["ip"], "[masked]")
        self.assertEqual(candidate_payload["target"]["path_hint"], "/")
        self.assertEqual(
            attack_plan["attack_generation_contract"]["schema_version"],
            "frontier-attack-generation-contract.v1",
        )
        self.assertGreaterEqual(
            int(attack_plan["generation_summary"]["generated_candidate_count"]),
            1,
        )
        self.assertIn("diversity_scoring", attack_plan)
        self.assertIn("discovery_quality_metrics", attack_plan)

    def test_build_attack_plan_records_rejected_generated_candidates(self):
        frontier = {
            "frontier_mode": "single_provider_self_play",
            "provider_count": 1,
            "providers": [{"provider": "openai", "model_id": "gpt-5-mini", "configured": True}],
            "diversity_confidence": "low",
        }
        scenarios = [
            {
                "id": "scenario_allow",
                "tier": "SIM-T0",
                "driver": "allow_browser_allowlist",
                "expected_outcome": "allow",
                "runtime_budget_ms": 1000,
                "seed": 1,
                "ip": "198.51.100.10",
                "user_agent": "UnitTest/1.0",
                "description": "allow scenario",
            }
        ]

        call_counter = {"count": 0}

        def flaky_sanitize(payload):
            call_counter["count"] += 1
            if call_counter["count"] == 2:
                raise runner.SimulationError("synthetic sanitization failure")
            return payload

        with patch(
            "scripts.tests.adversarial_simulation_runner.sanitize_frontier_payload",
            side_effect=flaky_sanitize,
        ):
            attack_plan = runner.build_attack_plan(
                profile_name="fast_smoke",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                scenarios=scenarios,
                frontier_metadata=frontier,
                generated_at_unix=1234,
            )

        self.assertGreaterEqual(
            int(attack_plan["generation_summary"]["rejected_candidate_count"]),
            1,
        )
        rejections = list(attack_plan.get("rejected_candidates") or [])
        self.assertTrue(rejections)
        self.assertIn("sanitization_error", str(rejections[0].get("reason_code") or ""))

    def test_frontier_path_hint_for_scenario_defaults_for_unknown_driver(self):
        self.assertEqual(
            runner.frontier_path_hint_for_scenario({"driver": "allow_browser_allowlist"}),
            "/",
        )
        self.assertEqual(
            runner.frontier_path_hint_for_scenario({"driver": "not_mapped"}),
            "/",
        )

    def test_has_leading_zero_bits_accepts_full_and_partial_prefixes(self):
        self.assertTrue(runner.has_leading_zero_bits(bytes.fromhex("00ff"), 8))
        self.assertTrue(runner.has_leading_zero_bits(bytes.fromhex("0fff"), 4))
        self.assertFalse(runner.has_leading_zero_bits(bytes.fromhex("10ff"), 4))

    def test_solve_pow_nonce_returns_valid_nonce_for_low_difficulty(self):
        seed = "unit-seed"
        difficulty = 8
        nonce = runner.solve_pow_nonce(seed, difficulty, max_iter=200_000)
        self.assertGreaterEqual(nonce, 0)
        digest = runner.pow_digest(seed, nonce)
        self.assertTrue(runner.has_leading_zero_bits(digest, difficulty))

    def test_find_invalid_pow_nonce_returns_nonce_that_fails_target(self):
        seed = "unit-seed"
        difficulty = 12
        nonce = runner.find_invalid_pow_nonce(seed, difficulty, max_iter=20)
        self.assertGreaterEqual(nonce, 0)
        digest = runner.pow_digest(seed, nonce)
        self.assertFalse(runner.has_leading_zero_bits(digest, difficulty))

    def test_extract_monitoring_snapshot_maps_coverage_fields(self):
        payload = {
            "summary": {
                "honeypot": {"total_hits": 3},
                "challenge": {"total_failures": 2},
                "not_a_bot": {"submitted": 4, "pass": 1, "fail": 2, "replay": 1, "escalate": 0},
                "pow": {"total_attempts": 7, "total_successes": 5, "total_failures": 2},
                "rate": {"total_violations": 6, "outcomes": {"limited": 4, "banned": 2}},
                "geo": {"total_violations": 5, "actions": {"challenge": 2, "maze": 2, "block": 1}},
            },
            "details": {
                "analytics": {"ban_count": 9},
                "events": {"recent_events": [{}, {}, {}]},
                "maze": {"total_hits": 8},
                "tarpit": {
                    "metrics": {
                        "activations": {"progressive": 2},
                        "progress_outcomes": {"advanced": 1},
                        "budget_outcomes": {"fallback_maze": 3},
                    }
                },
                "cdp": {
                    "stats": {"total_detections": 4},
                    "fingerprint_stats": {"events": 11},
                },
            },
        }

        snapshot = runner.extract_monitoring_snapshot(payload)

        self.assertEqual(snapshot["fingerprint_events"], 11)
        self.assertEqual(snapshot["coverage"]["honeypot_hits"], 3)
        self.assertEqual(snapshot["coverage"]["rate_limited"], 4)
        self.assertEqual(snapshot["coverage"]["geo_block"], 1)
        self.assertEqual(snapshot["coverage"]["tarpit_activations_progressive"], 2)
        self.assertEqual(snapshot["coverage"]["recent_event_count"], 3)
        self.assertEqual(snapshot["components"]["not_a_bot_submitted"], 4)
        self.assertEqual(snapshot["tarpit"]["metrics"]["budget_outcomes"]["fallback_maze"], 3)

    def test_extract_monitoring_snapshot_collects_recent_event_reasons(self):
        payload = {
            "summary": {},
            "details": {
                "events": {
                    "recent_events": [
                        {"reason": "not_a_bot_replay"},
                        {"reason": "cdp_detected:tier=high"},
                        {"reason": "not_a_bot_replay"},
                    ]
                }
            },
        }
        snapshot = runner.extract_monitoring_snapshot(payload)
        self.assertEqual(
            snapshot["recent_event_reasons"],
            ["cdp_detected:tier=high", "not_a_bot_replay"],
        )

    def test_extract_monitoring_snapshot_includes_retention_health(self):
        payload = {
            "summary": {},
            "security_privacy": {
                "classification": {"field_classification_enforced": True},
                "sanitization": {"secret_canary_leak_count": 0},
                "access_control": {
                    "view_mode": "pseudonymized_default",
                    "pseudonymization_coverage_percent": 100.0,
                    "pseudonymization_required_percent": 100.0,
                },
                "retention_tiers": {
                    "high_risk_raw_artifacts_hours": 72,
                    "high_risk_raw_artifacts_max_hours": 72,
                },
                "incident_response": {
                    "incident_hook_emitted": True,
                    "incident_hook_emitted_total": 0,
                },
            },
            "details": {
                "retention_health": {
                    "retention_hours": 168,
                    "pending_expired_buckets": 0,
                    "purge_lag_hours": 0,
                },
                "cost_governance": {
                    "guarded_dimension_cardinality_cap_per_hour": 1000,
                    "observed_guarded_dimension_cardinality_max": 42,
                },
            },
        }
        snapshot = runner.extract_monitoring_snapshot(payload)
        self.assertEqual(snapshot["retention_health"]["retention_hours"], 168)
        self.assertEqual(snapshot["retention_health"]["pending_expired_buckets"], 0)
        self.assertEqual(snapshot["cost_governance"]["observed_guarded_dimension_cardinality_max"], 42)
        self.assertEqual(
            snapshot["security_privacy"]["retention_tiers"]["high_risk_raw_artifacts_hours"],
            72,
        )

    def test_compute_coverage_deltas_clamps_negative_values(self):
        before = {"honeypot_hits": 5, "geo_maze": 3}
        after = {"honeypot_hits": 3, "geo_maze": 7}
        deltas = runner.compute_coverage_deltas(before, after)
        self.assertEqual(deltas["honeypot_hits"], 0)
        self.assertEqual(deltas["geo_maze"], 4)

    def test_build_coverage_depth_checks_reports_row_level_diagnostics(self):
        depth_requirements = {
            "tarpit_progression_depth": {
                "required_metrics": {
                    "tarpit_activations_progressive": 1,
                    "tarpit_progress_advanced": 1,
                },
                "required_scenarios": ["sim_t4_tarpit_replay_abuse"],
                "verification_matrix_row_id": "tarpit_progression",
            }
        }
        coverage_deltas = {
            "tarpit_activations_progressive": 2,
            "tarpit_progress_advanced": 1,
        }
        scenario_execution = {
            "sim_t4_tarpit_replay_abuse": {
                "coverage_deltas": {
                    "tarpit_activations_progressive": 2,
                    "tarpit_progress_advanced": 1,
                }
            }
        }
        checks, diagnostics = runner.build_coverage_depth_checks(
            depth_requirements,
            coverage_deltas=coverage_deltas,
            scenario_execution_evidence=scenario_execution,
        )
        self.assertEqual(len(checks), 1)
        self.assertTrue(checks[0]["passed"])
        self.assertEqual(len(diagnostics), 1)
        self.assertEqual(diagnostics[0]["missing_evidence"], [])
        self.assertEqual(diagnostics[0]["missing_scenarios"], [])

    def test_build_coverage_depth_checks_fails_when_metrics_or_scenarios_missing(self):
        depth_requirements = {
            "event_stream_health_depth": {
                "required_metrics": {"recent_event_count": 2},
                "required_scenarios": [
                    "sim_t3_replay_abuse",
                    "sim_t4_ordering_cadence_abuse",
                ],
                "verification_matrix_row_id": "event_stream_integrity",
            }
        }
        checks, diagnostics = runner.build_coverage_depth_checks(
            depth_requirements,
            coverage_deltas={"recent_event_count": 1},
            scenario_execution_evidence={"sim_t3_replay_abuse": {"coverage_deltas": {}}},
        )
        self.assertEqual(len(checks), 1)
        self.assertFalse(checks[0]["passed"])
        self.assertIn("recent_event_count", checks[0]["observed"]["missing_evidence"])
        self.assertEqual(
            diagnostics[0]["missing_scenarios"],
            ["sim_t4_ordering_cadence_abuse"],
        )

    def test_build_coverage_depth_checks_uses_scenario_contributions_when_recent_event_window_is_capped(self):
        depth_requirements = {
            "event_stream_health_depth": {
                "required_metrics": {"recent_event_count": 2},
                "required_scenarios": [
                    "sim_t3_replay_abuse",
                    "sim_t4_ordering_cadence_abuse",
                ],
                "verification_matrix_row_id": "event_stream_integrity",
            }
        }
        checks, diagnostics = runner.build_coverage_depth_checks(
            depth_requirements,
            coverage_deltas={"recent_event_count": 0},
            scenario_execution_evidence={
                "sim_t3_replay_abuse": {"coverage_deltas": {"recent_event_count": 1}},
                "sim_t4_ordering_cadence_abuse": {"coverage_deltas": {"recent_event_count": 1}},
            },
        )
        self.assertEqual(len(checks), 1)
        self.assertTrue(checks[0]["passed"])
        self.assertEqual(
            checks[0]["observed"]["observed"]["recent_event_count"],
            2,
        )
        self.assertEqual(diagnostics[0]["missing_evidence"], [])

    def test_build_retention_lifecycle_report_maps_health_fields(self):
        section = runner.build_retention_lifecycle_report(
            {
                "retention_hours": 168,
                "oldest_retained_ts": 1_700_000_000,
                "pending_expired_buckets": 2,
                "purge_lag_hours": 2.5,
                "last_purged_bucket": "eventlog:100",
                "last_error": "",
                "state": "degraded",
                "guidance": "investigate",
                "bucket_schema": [
                    "bucket_id",
                    "window_start",
                    "window_end",
                    "record_count",
                    "state",
                ],
            }
        )
        self.assertTrue(section["bucket_cutoff_correct"])
        self.assertTrue(section["purge_watermark_progression"])
        self.assertEqual(section["retention_hours"], 168)
        self.assertEqual(section["pending_expired_buckets"], 2)
        self.assertAlmostEqual(section["purge_lag_hours"], 2.5, places=2)

    def test_build_cost_governance_report_maps_runtime_fields(self):
        section = runner.build_cost_governance_report(
            {
                "guarded_dimension_cardinality_cap_per_hour": 1000,
                "observed_guarded_dimension_cardinality_max": 640,
                "overflow_bucket_accounted": True,
                "overflow_bucket_count": 1,
                "unsampleable_event_drop_count": 0,
                "payload_budget": {"p95_max_kb": 512, "estimated_current_payload_kb": 128},
                "compression": {
                    "reduction_percent": 35.0,
                    "min_percent": 30.0,
                    "negotiated": True,
                },
                "query_budget": {
                    "avg_req_per_sec_client_target": 0.5,
                    "max_req_per_sec_client": 1.0,
                },
                "cardinality_pressure": "normal",
                "payload_budget_status": "within_budget",
                "sampling_status": "compliant",
                "query_budget_status": "within_budget",
                "degraded_state": "normal",
            }
        )
        self.assertEqual(section["guarded_dimension_cardinality_cap_per_hour"], 1000)
        self.assertEqual(section["observed_guarded_dimension_cardinality_max"], 640)
        self.assertEqual(section["overflow_bucket_count"], 1)
        self.assertAlmostEqual(section["payload_p95_kb"], 128.0, places=2)
        self.assertAlmostEqual(section["compression_reduction_percent"], 35.0, places=2)
        self.assertEqual(section["large_payload_sample_count"], 1)
        self.assertAlmostEqual(section["query_budget_max_req_per_sec_client"], 1.0, places=2)
        self.assertEqual(section["cardinality_pressure"], "normal")
        self.assertEqual(section["degraded_state"], "normal")

    def test_build_cost_governance_report_ignores_large_payload_when_compression_not_negotiated(self):
        section = runner.build_cost_governance_report(
            {
                "payload_budget": {"p95_max_kb": 512, "estimated_current_payload_kb": 128},
                "compression": {"reduction_percent": 0.0, "min_percent": 30.0, "negotiated": False},
            }
        )
        self.assertEqual(section["large_payload_sample_count"], 0)

    def test_build_security_privacy_report_maps_runtime_fields(self):
        section = runner.build_security_privacy_report(
            {
                "classification": {"field_classification_enforced": True},
                "sanitization": {"secret_canary_leak_count": 0},
                "access_control": {
                    "view_mode": "pseudonymized_default",
                    "pseudonymization_coverage_percent": 100.0,
                    "pseudonymization_required_percent": 100.0,
                },
                "retention_tiers": {
                    "high_risk_raw_artifacts_hours": 72,
                    "high_risk_raw_artifacts_max_hours": 72,
                },
                "incident_response": {
                    "incident_hook_emitted": True,
                    "incident_hook_emitted_total": 2,
                },
            }
        )
        self.assertTrue(section["field_classification_enforced"])
        self.assertEqual(section["secret_canary_leak_count"], 0)
        self.assertAlmostEqual(section["pseudonymization_coverage_percent"], 100.0, places=2)
        self.assertAlmostEqual(section["high_risk_retention_hours"], 72.0, places=2)
        self.assertTrue(section["incident_hook_emitted"])
        self.assertEqual(section["incident_hook_emitted_total"], 2)
        self.assertEqual(section["security_mode"], "pseudonymized_default")

    def test_build_sim_tag_diagnostics_reports_healthy_when_no_sim_tag_failures(self):
        diagnostics = runner.build_sim_tag_diagnostics(
            simulation_event_reasons=["geo:challenge", "not_a_bot_replay"],
            sim_secret_present=True,
        )
        self.assertEqual(diagnostics["schema_version"], "sim-tag-diagnostics.v1")
        self.assertEqual(diagnostics["status"], "healthy")
        self.assertEqual(diagnostics["sim_tag_reason_count"], 0)
        self.assertEqual(diagnostics["dominant_failure"], "none")
        self.assertIn("No sim-tag validation failures observed", diagnostics["guidance"][0])

    def test_build_sim_tag_diagnostics_classifies_signature_and_replay_failures(self):
        diagnostics = runner.build_sim_tag_diagnostics(
            simulation_event_reasons=[
                "S_SIM_TAG_SIGNATURE_MISMATCH",
                "sim_tag_nonce_replay",
                "sim_tag_signature_mismatch",
                "geo:block",
            ],
            sim_secret_present=True,
        )
        self.assertEqual(diagnostics["status"], "validation_failures_detected")
        self.assertEqual(diagnostics["sim_tag_reason_count"], 3)
        self.assertEqual(diagnostics["failure_counts"]["signature_mismatch"], 2)
        self.assertEqual(diagnostics["failure_counts"]["nonce_replay"], 1)
        self.assertEqual(diagnostics["dominant_failure"], "signature_mismatch")
        joined_guidance = " ".join(diagnostics["guidance"])
        self.assertIn("Rotate SHUMA_SIM_TELEMETRY_SECRET", joined_guidance)
        self.assertIn("nonce replay", joined_guidance.lower())

    def test_build_coverage_checks_reports_pass_and_fail(self):
        checks = runner.build_coverage_checks(
            {"honeypot_hits": 1, "geo_block": 2},
            {"honeypot_hits": 3, "geo_block": 1},
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertTrue(checks_by_name["coverage_honeypot_hits"]["passed"])
        self.assertFalse(checks_by_name["coverage_geo_block"]["passed"])

    def test_profile_expected_defense_categories_filters_to_supported_defense_signals(self):
        selected_scenarios = [
            {"expected_defense_categories": ["pow", "maze", "event_stream"]},
            {"expected_defense_categories": ["geo", "cdp", "not_a_bot"]},
            {"expected_defense_categories": ["pow", "challenge", "rate_limit"]},
        ]
        categories = runner.profile_expected_defense_categories(selected_scenarios)
        self.assertEqual(categories, ["cdp", "challenge", "geo", "maze", "pow", "rate_limit"])

    def test_build_defense_noop_checks_reports_missing_signal_as_failure(self):
        checks = runner.build_defense_noop_checks(
            defense_categories=["pow", "challenge", "maze"],
            coverage_deltas={
                "pow_attempts": 2,
                "pow_successes": 1,
                "pow_failures": 1,
                "challenge_failures": 0,
                "maze_hits": 3,
            },
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertTrue(checks_by_name["defense_noop_detector_pow"]["passed"])
        self.assertFalse(checks_by_name["defense_noop_detector_challenge"]["passed"])
        self.assertTrue(checks_by_name["defense_noop_detector_maze"]["passed"])
        self.assertIn("telemetry_delta_total=0", checks_by_name["defense_noop_detector_challenge"]["detail"])

    def test_build_scenario_execution_evidence_marks_runtime_telemetry_presence(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_a",
            request_count_before=4,
            request_count_after=7,
            monitoring_before={"monitoring_total": 3, "coverage": {"honeypot_hits": 1}},
            monitoring_after={"monitoring_total": 5, "coverage": {"honeypot_hits": 2}},
            simulation_event_count_before=2,
            simulation_event_count_after=3,
            simulation_event_reasons_before=["honeypot"],
            simulation_event_reasons_after=["honeypot", "cdp_detected:tier=high"],
        )
        self.assertEqual(evidence["scenario_id"], "scenario_a")
        self.assertEqual(evidence["runtime_request_count"], 3)
        self.assertEqual(evidence["monitoring_total_delta"], 2)
        self.assertEqual(evidence["coverage_delta_total"], 2)
        self.assertEqual(evidence["coverage_deltas"]["honeypot_hits"], 1)
        self.assertEqual(evidence["coverage_deltas"]["recent_event_count"], 1)
        self.assertEqual(evidence["simulation_event_count_delta"], 1)
        self.assertEqual(evidence["simulation_event_reasons_delta"], ["cdp_detected:tier=high"])
        self.assertTrue(evidence["has_runtime_telemetry_evidence"])

    def test_build_scenario_execution_evidence_uses_reason_delta_when_event_count_is_window_capped(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_event_stream",
            request_count_before=4,
            request_count_after=5,
            monitoring_before={"monitoring_total": 0, "coverage": {"recent_event_count": 50}},
            monitoring_after={"monitoring_total": 0, "coverage": {"recent_event_count": 50}},
            simulation_event_count_before=50,
            simulation_event_count_after=50,
            simulation_event_reasons_before=["not_a_bot_fail"],
            simulation_event_reasons_after=["not_a_bot_fail", "not_a_bot_submit_fail_maze"],
        )
        self.assertEqual(evidence["simulation_event_count_delta"], 1)
        self.assertEqual(
            evidence["simulation_event_reasons_delta"],
            ["not_a_bot_submit_fail_maze"],
        )
        self.assertEqual(evidence["coverage_deltas"]["recent_event_count"], 1)
        self.assertTrue(evidence["has_runtime_telemetry_evidence"])

    def test_build_scenario_execution_evidence_supplements_not_a_bot_pass_from_sim_reason_delta(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_not_a_bot_pass",
            request_count_before=4,
            request_count_after=7,
            monitoring_before={"monitoring_total": 0, "coverage": {"not_a_bot_pass": 0}},
            monitoring_after={"monitoring_total": 0, "coverage": {"not_a_bot_pass": 0}},
            simulation_event_count_before=10,
            simulation_event_count_after=12,
            simulation_event_reasons_before=["js_verification"],
            simulation_event_reasons_after=["js_verification", "not_a_bot_pass"],
        )
        self.assertEqual(evidence["coverage_deltas"]["not_a_bot_pass"], 1)
        self.assertEqual(evidence["simulation_event_reasons_delta"], ["not_a_bot_pass"])
        self.assertTrue(evidence["has_runtime_telemetry_evidence"])

    def test_derive_coverage_deltas_from_simulation_event_reasons_maps_geo_policy_actions(self):
        deltas = runner.derive_coverage_deltas_from_simulation_event_reasons(
            {
                "geo_policy_challenge": 2,
                "geo_policy_maze": 3,
                "geo_policy_block": 4,
            }
        )
        self.assertEqual(deltas["geo_violations"], 9)
        self.assertEqual(deltas["geo_challenge"], 2)
        self.assertEqual(deltas["geo_maze"], 3)
        self.assertEqual(deltas["geo_block"], 4)

    def test_derive_coverage_deltas_from_simulation_event_reasons_maps_rate_enforcement(self):
        deltas = runner.derive_coverage_deltas_from_simulation_event_reasons(
            {
                "rate": 2,
                "ip_range_policy_rate_limit": 1,
            }
        )
        self.assertEqual(deltas["rate_violations"], 3)
        self.assertEqual(deltas["rate_banned"], 2)
        self.assertEqual(deltas["rate_limited"], 1)

    def test_build_scenario_execution_evidence_supplements_geo_coverage_from_sim_reason_delta(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_geo_challenge",
            request_count_before=2,
            request_count_after=3,
            monitoring_before={"monitoring_total": 0, "coverage": {"geo_violations": 0, "geo_challenge": 0}},
            monitoring_after={"monitoring_total": 0, "coverage": {"geo_violations": 0, "geo_challenge": 0}},
            simulation_event_count_before=5,
            simulation_event_count_after=6,
            simulation_event_reasons_before=["js_verification"],
            simulation_event_reasons_after=["js_verification", "geo_policy_challenge"],
        )
        self.assertEqual(evidence["coverage_deltas"]["geo_violations"], 1)
        self.assertEqual(evidence["coverage_deltas"]["geo_challenge"], 1)
        self.assertEqual(evidence["simulation_event_reasons_delta"], ["geo_policy_challenge"])
        self.assertTrue(evidence["has_runtime_telemetry_evidence"])

    def test_build_scenario_execution_evidence_supplements_rate_coverage_from_sim_reason_delta(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_rate",
            request_count_before=5,
            request_count_after=8,
            monitoring_before={"monitoring_total": 0, "coverage": {"rate_violations": 0, "rate_banned": 0}},
            monitoring_after={"monitoring_total": 0, "coverage": {"rate_violations": 0, "rate_banned": 0}},
            simulation_event_count_before=10,
            simulation_event_count_after=12,
            simulation_event_reasons_before=["js_verification"],
            simulation_event_reasons_after=["js_verification", "rate"],
        )
        self.assertEqual(evidence["coverage_deltas"]["rate_violations"], 1)
        self.assertEqual(evidence["coverage_deltas"]["rate_banned"], 1)
        self.assertEqual(evidence["simulation_event_reasons_delta"], ["rate"])
        self.assertTrue(evidence["has_runtime_telemetry_evidence"])

    def test_build_scenario_execution_evidence_maps_not_a_bot_abuse_reason_to_replay_delta(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_not_a_bot_abuse",
            request_count_before=4,
            request_count_after=7,
            monitoring_before={"monitoring_total": 0, "coverage": {"not_a_bot_replay": 0}},
            monitoring_after={"monitoring_total": 0, "coverage": {"not_a_bot_replay": 0}},
            simulation_event_count_before=10,
            simulation_event_count_after=12,
            simulation_event_reasons_before=["not_a_bot_fail"],
            simulation_event_reasons_after=[
                "not_a_bot_fail",
                "not_a_bot_submit_abuse_shadow_mode_maze",
            ],
        )
        self.assertEqual(evidence["coverage_deltas"]["not_a_bot_replay"], 1)
        self.assertEqual(
            evidence["simulation_event_reasons_delta"],
            ["not_a_bot_submit_abuse_shadow_mode_maze"],
        )
        self.assertTrue(evidence["has_runtime_telemetry_evidence"])

    def test_build_scenario_execution_evidence_uses_reason_count_delta_for_repeated_sim_reasons(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_not_a_bot_stale_abuse",
            request_count_before=4,
            request_count_after=6,
            monitoring_before={"monitoring_total": 0, "coverage": {"not_a_bot_fail": 0}},
            monitoring_after={"monitoring_total": 0, "coverage": {"not_a_bot_fail": 0}},
            simulation_event_count_before=3,
            simulation_event_count_after=5,
            simulation_event_reasons_before=["not_a_bot_fail"],
            simulation_event_reasons_after=["not_a_bot_fail"],
            simulation_event_reason_counts_before={"not_a_bot_fail": 1},
            simulation_event_reason_counts_after={"not_a_bot_fail": 3},
        )
        self.assertEqual(evidence["coverage_deltas"]["not_a_bot_fail"], 2)
        self.assertEqual(evidence["simulation_event_reasons_delta"], ["not_a_bot_fail"])
        self.assertEqual(evidence["simulation_event_count_delta"], 2)
        self.assertTrue(evidence["has_runtime_telemetry_evidence"])

    def test_build_scenario_execution_evidence_includes_browser_fields_for_browser_realistic_driver(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_browser",
            request_count_before=10,
            request_count_after=14,
            monitoring_before={"monitoring_total": 2, "coverage": {"geo_maze": 0}},
            monitoring_after={"monitoring_total": 4, "coverage": {"geo_maze": 1}},
            simulation_event_count_before=5,
            simulation_event_count_after=7,
            driver_class="browser_realistic",
            browser_realism={
                "browser_driver_runtime": "playwright_chromium",
                "browser_js_executed": True,
                "browser_dom_events": 3,
                "browser_storage_mode": "stateful_cookie_jar",
                "browser_challenge_dom_path": ["click:#not-a-bot-checkbox"],
                "browser_correlation_ids": ["nonce-a"],
                "browser_request_lineage_count": 4,
            },
        )
        self.assertEqual(evidence["driver_class"], "browser_realistic")
        self.assertEqual(evidence["browser_driver_runtime"], "playwright_chromium")
        self.assertTrue(evidence["browser_js_executed"])
        self.assertEqual(evidence["browser_dom_events"], 3)
        self.assertEqual(evidence["browser_storage_mode"], "stateful_cookie_jar")
        self.assertEqual(evidence["browser_challenge_dom_path"], ["click:#not-a-bot-checkbox"])
        self.assertEqual(evidence["browser_correlation_ids"], ["nonce-a"])
        self.assertEqual(evidence["browser_request_lineage_count"], 4)
        self.assertTrue(evidence["has_browser_execution_evidence"])

    def test_build_scenario_execution_evidence_treats_browser_request_lineage_as_runtime_evidence(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_browser_allow",
            request_count_before=2,
            request_count_after=3,
            monitoring_before={"monitoring_total": 0, "coverage": {}},
            monitoring_after={"monitoring_total": 0, "coverage": {}},
            simulation_event_count_before=0,
            simulation_event_count_after=0,
            driver_class="browser_realistic",
            browser_realism={
                "browser_js_executed": True,
                "browser_dom_events": 2,
                "browser_challenge_dom_path": ["read:body"],
                "browser_request_lineage_count": 1,
            },
        )

        self.assertEqual(evidence["runtime_request_count"], 1)
        self.assertEqual(evidence["browser_request_lineage_count"], 1)
        self.assertTrue(evidence["has_runtime_telemetry_evidence"])

    def test_seed_ip_range_suggestion_prerequisites_uses_trusted_forwarded_requests(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        trusted_forwarded_flags = []
        sim_runner.admin_patch = lambda payload, reason="": None  # type: ignore[assignment]
        sim_runner.admin_unban = lambda ip, reason="": None  # type: ignore[assignment]
        sim_runner.ip_range_suggestions_snapshot = lambda hours=24, limit=20: {  # type: ignore[assignment]
            "summary": {"suggestions_total": 0},
            "suggestions": [],
        }

        def _request(method, path, headers=None, json_body=None, form_body=None, count_request=False, trusted_forwarded=False):
            trusted_forwarded_flags.append(trusted_forwarded)
            return runner.HttpResult(status=200, body="ok", headers={}, latency_ms=1)

        sim_runner.attacker_client.request = _request  # type: ignore[assignment]

        evidence = sim_runner.seed_ip_range_suggestion_prerequisites()

        self.assertEqual(evidence["seed_prefix"], "10.222.77.0/24")
        self.assertGreater(len(trusted_forwarded_flags), 0)
        self.assertTrue(all(trusted_forwarded_flags))

    def test_build_runtime_telemetry_evidence_checks_fail_when_passed_scenario_missing_telemetry(self):
        results = [
            runner.ScenarioResult(
                id="scenario_a",
                tier="SIM-T1",
                driver="pow_success",
                expected_outcome="allow",
                observed_outcome="allow",
                passed=True,
                latency_ms=100,
                runtime_budget_ms=1000,
                detail="ok",
            ),
            runner.ScenarioResult(
                id="scenario_b",
                tier="SIM-T3",
                driver="rate_limit_enforce",
                expected_outcome="deny_temp",
                observed_outcome="deny_temp",
                passed=True,
                latency_ms=150,
                runtime_budget_ms=1000,
                detail="ok",
            ),
        ]
        scenario_execution_evidence = {
            "scenario_a": {
                "scenario_id": "scenario_a",
                "runtime_request_count": 3,
                "monitoring_total_delta": 2,
                "coverage_delta_total": 1,
                "simulation_event_count_delta": 1,
                "has_runtime_telemetry_evidence": True,
            },
            "scenario_b": {
                "scenario_id": "scenario_b",
                "runtime_request_count": 1,
                "monitoring_total_delta": 0,
                "coverage_delta_total": 0,
                "simulation_event_count_delta": 0,
                "has_runtime_telemetry_evidence": False,
            },
        }
        checks = runner.build_runtime_telemetry_evidence_checks(
            results=results,
            scenario_execution_evidence=scenario_execution_evidence,
            required_fields=runner.REAL_TRAFFIC_CONTRACT_REQUIRED_SCENARIO_FIELDS,
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertTrue(checks_by_name["runtime_evidence_rows_for_passed_scenarios"]["passed"])
        self.assertTrue(checks_by_name["runtime_evidence_required_fields_present"]["passed"])
        self.assertFalse(checks_by_name["runtime_evidence_telemetry_for_passed_scenarios"]["passed"])
        self.assertIn(
            "scenario_b",
            checks_by_name["runtime_evidence_telemetry_for_passed_scenarios"]["detail"],
        )

    def test_build_browser_execution_evidence_checks_fail_on_missing_browser_proof_fields(self):
        selected_scenarios = [
            {"id": "scenario_browser", "driver": "allow_browser_allowlist", "driver_class": "browser_realistic"},
            {"id": "scenario_http", "driver": "rate_limit_enforce", "driver_class": "http_scraper"},
        ]
        results = [
            runner.ScenarioResult(
                id="scenario_browser",
                tier="SIM-T0",
                driver="allow_browser_allowlist",
                expected_outcome="allow",
                observed_outcome="allow",
                passed=True,
                latency_ms=100,
                runtime_budget_ms=1000,
                detail="ok",
            ),
            runner.ScenarioResult(
                id="scenario_http",
                tier="SIM-T3",
                driver="rate_limit_enforce",
                expected_outcome="deny_temp",
                observed_outcome="deny_temp",
                passed=True,
                latency_ms=100,
                runtime_budget_ms=1000,
                detail="ok",
            ),
        ]
        checks = runner.build_browser_execution_evidence_checks(
            selected_scenarios=selected_scenarios,
            results=results,
            scenario_execution_evidence={
                "scenario_browser": {
                    "scenario_id": "scenario_browser",
                    "driver_class": "browser_realistic",
                    "has_browser_execution_evidence": False,
                    "browser_js_executed": False,
                    "browser_dom_events": 0,
                    "browser_challenge_dom_path": [],
                    "browser_correlation_ids": [],
                    "browser_request_lineage_count": 0,
                }
            },
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertFalse(checks_by_name["browser_execution_required_rows_present"]["passed"])
        self.assertFalse(checks_by_name["browser_execution_js_executed"]["passed"])
        self.assertFalse(checks_by_name["browser_execution_dom_events"]["passed"])
        self.assertFalse(checks_by_name["browser_execution_dom_paths"]["passed"])
        self.assertFalse(checks_by_name["browser_execution_correlation_ids"]["passed"])

    def test_build_scenario_intent_checks_fail_when_required_defense_signal_missing(self):
        selected_scenarios = [
            {
                "id": "sim_t2_geo_challenge",
                "driver": "geo_challenge",
                "driver_class": "browser_realistic",
            }
        ]
        results = [
            runner.ScenarioResult(
                id="sim_t2_geo_challenge",
                tier="SIM-T2",
                driver="geo_challenge",
                expected_outcome="challenge",
                observed_outcome="challenge",
                passed=True,
                latency_ms=120,
                runtime_budget_ms=9000,
                detail="ok",
                realism={
                    "persona": "suspicious_automation",
                    "retry_strategy": "bounded_backoff",
                    "request_sequence_count": 1,
                    "think_time_events": 1,
                    "retry_attempts": 0,
                    "attempts_total": 1,
                },
            )
        ]
        checks = runner.build_scenario_intent_checks(
            selected_scenarios=selected_scenarios,
            results=results,
            scenario_execution_evidence={
                "sim_t2_geo_challenge": {
                    "scenario_id": "sim_t2_geo_challenge",
                    "driver_class": "browser_realistic",
                    "runtime_request_count": 1,
                    "coverage_deltas": {"geo_violations": 0, "geo_challenge": 0},
                    "simulation_event_count_delta": 0,
                    "simulation_event_reasons_delta": [],
                }
            },
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertFalse(checks_by_name["scenario_intent_sim_t2_geo_challenge_geo"]["passed"])
        self.assertTrue(checks_by_name["scenario_intent_sim_t2_geo_challenge_challenge"]["passed"])

    def test_build_scenario_intent_checks_pass_when_signals_and_progression_match(self):
        selected_scenarios = [
            {
                "id": "sim_t2_geo_challenge",
                "driver": "geo_challenge",
                "driver_class": "browser_realistic",
            }
        ]
        results = [
            runner.ScenarioResult(
                id="sim_t2_geo_challenge",
                tier="SIM-T2",
                driver="geo_challenge",
                expected_outcome="challenge",
                observed_outcome="challenge",
                passed=True,
                latency_ms=120,
                runtime_budget_ms=9000,
                detail="ok",
                realism={
                    "persona": "suspicious_automation",
                    "retry_strategy": "bounded_backoff",
                    "request_sequence_count": 1,
                    "think_time_events": 1,
                    "retry_attempts": 0,
                    "attempts_total": 1,
                },
            )
        ]
        checks = runner.build_scenario_intent_checks(
            selected_scenarios=selected_scenarios,
            results=results,
            scenario_execution_evidence={
                "sim_t2_geo_challenge": {
                    "scenario_id": "sim_t2_geo_challenge",
                    "driver_class": "browser_realistic",
                    "runtime_request_count": 2,
                    "coverage_deltas": {"geo_violations": 1, "geo_challenge": 1, "challenge_failures": 0},
                    "simulation_event_count_delta": 1,
                    "simulation_event_reasons_delta": ["geo:challenge"],
                }
            },
        )
        failing = [check["name"] for check in checks if not check["passed"]]
        self.assertEqual(failing, [])

    def test_annotate_coverage_checks_with_threshold_sources(self):
        checks = runner.build_coverage_checks({"honeypot_hits": 1}, {"honeypot_hits": 2})
        annotated = runner.annotate_coverage_checks_with_threshold_source({"honeypot_hits": 1}, checks)
        self.assertEqual(
            annotated[0]["threshold_source"],
            "profile.gates.coverage_requirements.honeypot_hits",
        )

    def test_validate_manifest_accepts_supported_coverage_requirements(self):
        manifest = minimal_manifest(
            {"coverage_requirements": {"honeypot_hits": 1, "geo_maze": 1}}
        )
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_unknown_coverage_requirement_key(self):
        manifest = minimal_manifest({"coverage_requirements": {"unknown_counter": 1}})
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(
                Path("scripts/tests/adversarial/scenario_manifest.v1.json"),
                manifest,
                "test_profile",
            )

    def test_validate_manifest_accepts_collateral_and_event_reason_gates(self):
        manifest = minimal_manifest(
            {
                "human_like_collateral_max_ratio": 0.25,
                "required_event_reasons": ["not_a_bot_replay", "cdp_detected"],
                "ip_range_suggestion_seed_required": True,
            }
        )
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_invalid_human_like_collateral_ratio(self):
        manifest = minimal_manifest({"human_like_collateral_max_ratio": 1.2})
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_empty_required_event_reasons(self):
        manifest = minimal_manifest({"required_event_reasons": []})
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_accepts_new_driver_enum_values(self):
        manifest = minimal_manifest()
        manifest["scenarios"][0]["driver"] = "pow_success"
        manifest["scenarios"][0]["expected_outcome"] = "allow"
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_accepts_tarpit_outcome(self):
        manifest = minimal_manifest()
        manifest["scenarios"][0]["driver"] = "not_a_bot_replay_tarpit_abuse"
        manifest["scenarios"][0]["expected_outcome"] = "tarpit"
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_accepts_v2_contract_fields(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v2.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_v2_missing_traffic_model(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        del manifest["scenarios"][0]["traffic_model"]
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v2.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_v2_driver_class_mismatch(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        manifest["scenarios"][0]["driver_class"] = "http_scraper"
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v2.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_canonical_intent_matrix_category_drift(self):
        manifest_path = Path("scripts/tests/adversarial/scenario_manifest.v2.json")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["scenarios"][0]["expected_defense_categories"] = ["maze"]
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(manifest_path, manifest, "fast_smoke")

    def test_canonical_manifest_telemetry_amplification_bounds_cover_cross_env_ci_baseline(self):
        manifest_v2 = json.loads(
            Path("scripts/tests/adversarial/scenario_manifest.v2.json").read_text(encoding="utf-8")
        )
        manifest_v1 = json.loads(
            Path("scripts/tests/adversarial/scenario_manifest.v1.json").read_text(encoding="utf-8")
        )

        fast_smoke_v2 = manifest_v2["profiles"]["fast_smoke"]["gates"]["telemetry_amplification"]
        fast_smoke_v1 = manifest_v1["profiles"]["fast_smoke"]["gates"]["telemetry_amplification"]
        full_coverage_v2 = manifest_v2["profiles"]["full_coverage"]["gates"]["telemetry_amplification"]

        self.assertEqual(fast_smoke_v2, fast_smoke_v1)
        self.assertEqual(fast_smoke_v2["max_fingerprint_events_per_request"], 4.0)
        self.assertEqual(fast_smoke_v2["max_monitoring_events_per_request"], 9.0)
        self.assertGreaterEqual(
            full_coverage_v2["max_fingerprint_events_per_request"],
            fast_smoke_v2["max_fingerprint_events_per_request"],
        )
        self.assertGreaterEqual(
            full_coverage_v2["max_monitoring_events_per_request"],
            fast_smoke_v2["max_monitoring_events_per_request"],
        )

    def test_validate_manifest_rejects_unsupported_execution_lane(self):
        manifest = minimal_manifest()
        manifest["execution_lane"] = "white_box"
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_scenario_max_latency_ms_uses_v2_cost_assertions_when_present(self):
        scenario = {"id": "s1", "assertions": {"max_latency_ms": 700}, "cost_assertions": {"max_latency_ms": 333}}
        self.assertEqual(runner.scenario_max_latency_ms(scenario), 333)

    def test_scenario_driver_class_uses_mapping_when_manifest_does_not_set_class(self):
        scenario = {"driver": "pow_success"}
        self.assertEqual(runner.scenario_driver_class(scenario), "cost_imposition")

    def test_compute_cohort_metrics_reports_human_collateral_ratio(self):
        scenarios = [
            {"id": "s1", "tier": "SIM-T0"},
            {"id": "s2", "tier": "SIM-T0"},
            {"id": "s3", "tier": "SIM-T3"},
        ]
        results = [
            runner.ScenarioResult(
                id="s1",
                tier="SIM-T0",
                driver="allow_browser_allowlist",
                expected_outcome="allow",
                observed_outcome="allow",
                passed=True,
                latency_ms=100,
                runtime_budget_ms=1000,
                detail="ok",
            ),
            runner.ScenarioResult(
                id="s2",
                tier="SIM-T0",
                driver="geo_challenge",
                expected_outcome="challenge",
                observed_outcome="challenge",
                passed=True,
                latency_ms=120,
                runtime_budget_ms=1000,
                detail="ok",
            ),
            runner.ScenarioResult(
                id="s3",
                tier="SIM-T3",
                driver="rate_limit_enforce",
                expected_outcome="deny_temp",
                observed_outcome="deny_temp",
                passed=True,
                latency_ms=140,
                runtime_budget_ms=1000,
                detail="ok",
            ),
        ]

        metrics = runner.compute_cohort_metrics(scenarios, results)
        self.assertAlmostEqual(metrics["human_like"]["collateral_ratio"], 0.5, places=3)
        self.assertEqual(metrics["adversarial"]["collateral_count"], 1)

    def test_round_robin_sequence_violations_detect_consecutive_persona_when_others_pending(self):
        sequence = ["human_like", "human_like", "adversarial", "adversarial"]
        violations = runner.round_robin_sequence_violations(sequence)
        self.assertEqual(violations, [1])

    def test_realism_metrics_and_checks_cover_retry_cookie_and_persona_envelopes(self):
        scenarios = [
            {"id": "s1", "tier": "SIM-T1", "traffic_model": {"persona": "benign_automation"}},
            {"id": "s2", "tier": "SIM-T3", "traffic_model": {"persona": "adversarial"}},
        ]
        results = [
            runner.ScenarioResult(
                id="s1",
                tier="SIM-T1",
                driver="not_a_bot_pass",
                expected_outcome="not-a-bot",
                observed_outcome="not-a-bot",
                passed=True,
                latency_ms=80,
                runtime_budget_ms=1000,
                detail="ok",
                realism={
                    "persona": "benign_automation",
                    "retry_strategy": "single_attempt",
                    "state_mode": "stateful_cookie_jar",
                    "think_time_ms_min": 50,
                    "think_time_ms_max": 900,
                    "think_time_events": 1,
                    "think_time_ms_total": 200,
                    "request_sequence_count": 2,
                    "attempts_total": 2,
                    "retry_attempts": 0,
                    "retry_backoff_ms_total": 0,
                    "state_headers_sent": 1,
                    "state_token_observed": 1,
                    "state_store_resets": 0,
                    "state_store_peak_size": 1,
                    "max_attempts_configured": 1,
                },
            ),
            runner.ScenarioResult(
                id="s2",
                tier="SIM-T3",
                driver="retry_storm_enforce",
                expected_outcome="deny_temp",
                observed_outcome="deny_temp",
                passed=True,
                latency_ms=120,
                runtime_budget_ms=1000,
                detail="ok",
                realism={
                    "persona": "adversarial",
                    "retry_strategy": "retry_storm",
                    "state_mode": "stateless",
                    "think_time_ms_min": 5,
                    "think_time_ms_max": 200,
                    "think_time_events": 1,
                    "think_time_ms_total": 50,
                    "request_sequence_count": 2,
                    "attempts_total": 3,
                    "retry_attempts": 1,
                    "retry_backoff_ms_total": 10,
                    "state_headers_sent": 0,
                    "state_token_observed": 0,
                    "state_store_resets": 0,
                    "state_store_peak_size": 0,
                    "max_attempts_configured": 3,
                },
            ),
        ]

        realism_metrics = runner.compute_realism_metrics(
            scenarios, results, persona_scheduler="round_robin"
        )
        checks = runner.build_realism_checks(
            "full_coverage",
            {
                "persona_scheduler": "round_robin",
                "realism": {"enabled": True, "required_retry_attempts": {"retry_storm": 1}},
            },
            realism_metrics,
        )
        failing = [check["name"] for check in checks if not check["passed"]]
        self.assertEqual(failing, [])

    def test_attacker_request_applies_retry_and_stateful_cookie_policy(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        manifest["scenarios"][0]["traffic_model"] = {
            "persona": "adversarial",
            "think_time_ms_min": 0,
            "think_time_ms_max": 0,
            "retry_strategy": "retry_storm",
            "cookie_behavior": "stateful_cookie_jar",
        }

        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        captured_headers = []
        responses = [
            runner.HttpResult(status=500, body="retry", headers={}, latency_ms=1),
            runner.HttpResult(status=200, body="ok", headers={"set-cookie": "sid=abc; Path=/"}, latency_ms=1),
            runner.HttpResult(status=200, body="ok", headers={}, latency_ms=1),
        ]

        def fake_request(
            method,
            path,
            headers=None,
            json_body=None,
            form_body=None,
            plane="attacker",
            count_request=False,
            trusted_forwarded=False,
        ):
            self.assertEqual(plane, "attacker")
            captured_headers.append(dict(headers or {}))
            return responses.pop(0)

        sim_runner.request = fake_request  # type: ignore[assignment]
        sim_runner.begin_scenario_execution(manifest["scenarios"][0])
        with patch("scripts.tests.adversarial_simulation_runner.time.sleep", return_value=None):
            first = sim_runner.attacker_request(
                "GET",
                "/",
                headers={"X-Forwarded-For": "10.0.0.1"},
                count_request=True,
            )
            second = sim_runner.attacker_request(
                "GET",
                "/",
                headers={"X-Forwarded-For": "10.0.0.1"},
                count_request=True,
            )
        realism = sim_runner.end_scenario_execution()

        self.assertEqual(first.status, 200)
        self.assertEqual(second.status, 200)
        self.assertEqual(len(captured_headers), 3)
        self.assertEqual(captured_headers[-1].get("Cookie"), "sid=abc")
        self.assertEqual(realism["retry_attempts"], 1)
        self.assertEqual(realism["attempts_total"], 3)
        self.assertEqual(realism["state_headers_sent"], 1)
        self.assertEqual(realism["request_latency_ms_total"], 3)
        self.assertEqual(realism["request_latency_ms_max"], 1)

    def test_execute_browser_realistic_driver_invokes_node_and_records_browser_evidence(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        scenario = manifest["scenarios"][0]
        scenario["driver"] = "allow_browser_allowlist"
        scenario["expected_outcome"] = "allow"
        scenario["driver_class"] = "browser_realistic"
        scenario["traffic_model"]["cookie_behavior"] = "stateful_cookie_jar"

        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.begin_scenario_execution(scenario)
        mocked_stdout = {
            "ok": True,
            "observed_outcome": "allow",
            "detail": "ok",
            "browser_evidence": {
                "driver_runtime": "playwright_chromium",
                "js_executed": True,
                "dom_events": 4,
                "storage_mode": "stateful_cookie_jar",
                "challenge_dom_path": ["read:body"],
                "correlation_ids": ["nonce-1"],
                "request_lineage": [
                    {"method": "GET", "path": "/", "sim_nonce": "nonce-1"},
                    {"method": "GET", "path": "/favicon.ico", "sim_nonce": "nonce-1"},
                ],
            },
            "diagnostics": {},
        }

        with patch(
            "scripts.tests.adversarial_simulation_runner.subprocess.run",
            return_value=subprocess.CompletedProcess(
                args=["corepack", "pnpm", "exec", "node"],
                returncode=0,
                stdout=json.dumps(mocked_stdout),
                stderr="",
            ),
        ) as run_mock:
            observed = sim_runner.execute_browser_realistic_driver(
                scenario,
                action="allow_browser_allowlist",
                headers=sim_runner.forwarded_headers("10.10.10.10", user_agent="UnitTest/1.0"),
                user_agent="UnitTest/1.0",
            )
            self.assertEqual(observed, "allow")
            self.assertEqual(sim_runner.request_count, 2)
            self.assertEqual(run_mock.call_count, 1)
            payload = json.loads(str(run_mock.call_args.kwargs.get("input") or "{}"))
            self.assertEqual(payload.get("trusted_forwarded_secret"), "forwarded-secret")
            self.assertNotIn("X-Shuma-Forwarded-Secret", dict(payload.get("headers") or {}))

        realism = sim_runner.end_scenario_execution()
        self.assertEqual(realism["browser_driver_runtime"], "playwright_chromium")
        self.assertTrue(realism["browser_js_executed"])
        self.assertEqual(realism["browser_dom_events"], 4)
        self.assertEqual(realism["browser_challenge_dom_path"], ["read:body"])
        self.assertEqual(realism["browser_correlation_ids"], ["nonce-1"])
        self.assertEqual(realism["browser_request_lineage_count"], 2)
        self.assertEqual(realism["browser_action_duration_ms"], 0)
        self.assertEqual(realism["browser_launch_duration_ms"], 0)
        self.assertEqual(realism["browser_total_duration_ms"], 0)

    def test_runner_initialization_ensures_playwright_runtime_for_browser_realistic_profiles(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        browser_status = runner.PlaywrightRuntimeStatus(
            browser_cache=str(runner.DEFAULT_PLAYWRIGHT_BROWSER_CACHE),
            chromium_executable="/tmp/chromium/chrome",
            installed_now=False,
        )

        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ), patch(
            "scripts.tests.adversarial_simulation_runner.ensure_playwright_chromium",
            return_value=browser_status,
        ) as ensure_mock:
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        ensure_mock.assert_called_once()
        self.assertEqual(
            sim_runner.browser_driver_env.get("PLAYWRIGHT_BROWSERS_PATH"),
            str(runner.DEFAULT_PLAYWRIGHT_BROWSER_CACHE),
        )

    def test_admin_read_request_retries_throttled_reads(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        responses = [
            runner.HttpResult(status=429, body="Too Many Requests", headers={"retry-after": "1"}, latency_ms=1),
            runner.HttpResult(status=200, body="{}", headers={}, latency_ms=1),
        ]
        sim_runner.admin_request = (  # type: ignore[assignment]
            lambda method, path, json_body=None, headers=None, timeout_seconds=None: responses.pop(0)
        )

        with patch("scripts.tests.adversarial_simulation_runner.time.sleep", return_value=None) as sleep_mock:
            result = sim_runner.admin_read_request("GET", "/admin/events")
        self.assertEqual(result.status, 200)
        sleep_mock.assert_called_once()

    def test_admin_read_request_returns_last_throttled_response_after_budget(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        responses = [
            runner.HttpResult(status=429, body="Too Many Requests", headers={"retry-after": "1"}, latency_ms=1),
            runner.HttpResult(status=429, body="Too Many Requests", headers={"retry-after": "1"}, latency_ms=1),
        ]
        sim_runner.admin_request = (  # type: ignore[assignment]
            lambda method, path, json_body=None, headers=None, timeout_seconds=None: responses.pop(0)
        )

        with patch("scripts.tests.adversarial_simulation_runner.time.sleep", return_value=None):
            result = sim_runner.admin_read_request("GET", "/admin/events", max_attempts=2)
        self.assertEqual(result.status, 429)

    def test_replay_promotion_snapshot_reads_machine_first_contract(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.admin_read_request = lambda method, path, json_body=None, max_attempts=4, timeout_seconds=None: runner.HttpResult(  # type: ignore[assignment]
            status=200,
            body=json.dumps(
                {
                    "schema_version": "replay_promotion_v1",
                    "generated_at_unix": 1_700_000_200,
                    "summary": {"blocking_required": True},
                }
            ),
            headers={},
            latency_ms=1,
        )

        payload = sim_runner.replay_promotion_snapshot()

        self.assertEqual(payload["schema_version"], "replay_promotion_v1")
        self.assertTrue(payload["summary"]["blocking_required"])

    def test_run_scenario_applies_setup_phase_before_attacker_execution(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.preserve_state = True
        observed_reset_phase = {}

        def fake_reset():
            observed_reset_phase["phase"] = sim_runner.execution_phase

        sim_runner.reset_baseline_config = fake_reset  # type: ignore[assignment]
        sim_runner.apply_scenario_setup_preset = lambda scenario: None  # type: ignore[assignment]
        sim_runner.execute_scenario_driver = lambda scenario: scenario["expected_outcome"]  # type: ignore[assignment]

        result = sim_runner.run_scenario(manifest["scenarios"][0])
        self.assertTrue(result.passed)
        self.assertEqual(
            observed_reset_phase.get("phase"),
            runner.SUITE_PHASE_SETUP,
        )
        self.assertEqual(sim_runner.execution_phase, runner.SUITE_PHASE_ATTACKER_EXECUTION)
        observed_phases = [entry.get("phase") for entry in sim_runner.execution_phase_trace]
        self.assertIn(runner.SUITE_PHASE_SETUP, observed_phases)
        self.assertIn(runner.SUITE_PHASE_ATTACKER_EXECUTION, observed_phases)

    def test_run_scenario_attaches_realism_evidence(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.preserve_state = True
        sim_runner.reset_baseline_config = lambda: None  # type: ignore[assignment]
        sim_runner.apply_scenario_setup_preset = lambda scenario: None  # type: ignore[assignment]
        sim_runner.execute_scenario_driver = lambda scenario: scenario["expected_outcome"]  # type: ignore[assignment]
        result = sim_runner.run_scenario(manifest["scenarios"][0])
        self.assertTrue(result.passed)
        self.assertIsInstance(result.realism, dict)
        self.assertIn("request_sequence_count", result.realism or {})

    def test_run_scenario_uses_explicit_request_latency_for_edge_fixture_scenarios(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        scenario = manifest["scenarios"][0]
        scenario["driver_class"] = "edge_fixture"
        scenario["traffic_model"] = {
            "persona": "suspicious_automation",
            "think_time_ms_min": 0,
            "think_time_ms_max": 0,
            "retry_strategy": "single_attempt",
            "cookie_behavior": "stateless",
        }
        scenario["cost_assertions"] = {"max_latency_ms": 500}

        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.preserve_state = True
        sim_runner.reset_baseline_config = lambda: None  # type: ignore[assignment]
        sim_runner.apply_scenario_setup_preset = lambda scenario: None  # type: ignore[assignment]

        def fake_execute(active_scenario):
            state = sim_runner._active_execution_state or {}
            evidence = state.get("evidence")
            if isinstance(evidence, dict):
                evidence["request_latency_ms_total"] = 120
                evidence["request_latency_ms_max"] = 70
            return active_scenario["expected_outcome"]

        sim_runner.execute_scenario_driver = fake_execute  # type: ignore[assignment]

        with patch(
            "scripts.tests.adversarial_simulation_runner.time.monotonic",
            side_effect=[100.0, 101.0, 103.5],
        ):
            result = sim_runner.run_scenario(scenario)

        self.assertTrue(result.passed)
        self.assertEqual(result.latency_ms, 120)

    def test_reset_baseline_config_restores_js_required_default(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        captured = {}

        def fake_admin_patch(payload, reason="admin_config_patch"):
            captured["payload"] = dict(payload)
            captured["reason"] = reason

        sim_runner.admin_patch = fake_admin_patch  # type: ignore[assignment]
        sim_runner.reset_baseline_config()

        self.assertEqual(captured.get("reason"), "reset_baseline_config")
        self.assertIs(captured.get("payload", {}).get("js_required_enforced"), True)
        self.assertIs(captured.get("payload", {}).get("pow_enabled"), True)

    def test_header_spoofing_setup_patch_disables_rate_defence_to_avoid_profile_leak_collateral(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        patch_payload = sim_runner.scenario_setup_patch(
            {
                "id": "sim_t3_header_spoofing_abuse",
                "driver": "header_spoofing_probe",
                "geo_country": "RU",
            }
        )

        self.assertEqual(
            patch_payload.get("defence_modes", {}).get("rate"),
            "off",
        )
        self.assertEqual(patch_payload.get("geo_block"), ["RU"])
        self.assertIs(patch_payload.get("browser_policy_enabled"), False)
        self.assertIs(patch_payload.get("js_required_enforced"), False)
        self.assertIs(patch_payload.get("pow_enabled"), False)
        self.assertIs(patch_payload.get("cdp_detection_enabled"), False)
        self.assertIs(patch_payload.get("cdp_auto_ban"), False)

    def test_allow_browser_allowlist_setup_patch_disables_first_touch_friction(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        patch_payload = sim_runner.scenario_setup_patch(
            {
                "id": "sim_t0_allow_browser_allowlist",
                "driver": "allow_browser_allowlist",
            }
        )

        self.assertIs(patch_payload.get("browser_policy_enabled"), False)
        self.assertEqual(patch_payload.get("browser_allowlist"), [["Chrome", 120]])
        self.assertIs(patch_payload.get("js_required_enforced"), False)
        self.assertIs(patch_payload.get("pow_enabled"), False)

    def test_validate_manifest_full_coverage_requires_scheduler_and_realism_retry_contract(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        profile = manifest["profiles"].pop("test_profile")
        manifest["profiles"]["full_coverage"] = profile
        manifest["profiles"]["full_coverage"]["gates"]["realism"] = {"enabled": True}
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(
                Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest,
                "full_coverage",
            )

    def test_enforce_attacker_request_contract_rejects_admin_paths(self):
        with self.assertRaises(runner.SimulationError):
            runner.enforce_attacker_request_contract("/admin/config", {})

    def test_enforce_attacker_request_contract_rejects_privileged_headers(self):
        with self.assertRaises(runner.SimulationError):
            runner.enforce_attacker_request_contract(
                "/",
                {"Authorization": "Bearer test-token"},
            )

    def test_enforce_attacker_request_contract_allows_public_path_without_privileged_headers(self):
        runner.enforce_attacker_request_contract(
            "http://127.0.0.1:3000/challenge/not-a-bot-checkbox",
            {
                "X-Forwarded-For": "10.0.0.1",
                "User-Agent": "UnitTest/1.0",
                runner.SIM_TAG_HEADER_RUN_ID: "run-1",
                runner.SIM_TAG_HEADER_PROFILE: "fast_smoke",
                runner.SIM_TAG_HEADER_LANE: "deterministic_black_box",
                runner.SIM_TAG_HEADER_TIMESTAMP: "1700000000",
                runner.SIM_TAG_HEADER_NONCE: "nonce-1",
                runner.SIM_TAG_HEADER_SIGNATURE: "a" * 64,
            },
        )


if __name__ == "__main__":
    unittest.main()
