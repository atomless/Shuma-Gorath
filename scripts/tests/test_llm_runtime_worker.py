#!/usr/bin/env python3

import json
from pathlib import Path
import subprocess
import tempfile
from unittest import mock
import unittest

from scripts.supervisor import llm_runtime_worker
from scripts.tests.adversarial_runner.contracts import resolve_lane_realism_profile


class LlmRuntimeWorkerUnitTests(unittest.TestCase):
    def test_build_browser_mode_realism_execution_plan_shapes_stable_session_and_dwell(self):
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-runtime",
            "tick_id": "llm-fit-tick-browser-1",
            "lane": "bot_red_team",
            "fulfillment_mode": "browser_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["browser_agent"],
            "capability_envelope": {"max_actions": 8, "max_time_budget_seconds": 90},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "browser_mode"),
        }
        generation = {
            "generation_source": "provider_response",
            "provider": "openai",
            "model_id": "gpt-5-mini",
            "actions": [
                {
                    "action_index": 1,
                    "action_type": "browser_navigate",
                    "path": "/",
                    "label": "root",
                },
                {
                    "action_index": 2,
                    "action_type": "browser_click",
                    "path": "/sim/public/research/",
                    "label": "research",
                },
                {
                    "action_index": 3,
                    "action_type": "browser_snapshot",
                    "path": "/sim/public/plans/",
                    "label": "plans",
                },
            ],
        }

        execution_plan = llm_runtime_worker.build_browser_mode_realism_execution_plan(
            fulfillment_plan=plan,
            generation_result=generation,
        )

        self.assertEqual(
            execution_plan["schema_version"],
            "adversary-sim-llm-browser-realism-plan.v1",
        )
        self.assertEqual(execution_plan["profile_id"], "agentic.browser_mode.v1")
        self.assertGreaterEqual(execution_plan["top_level_action_budget"], 1)
        self.assertIn("/", execution_plan["focused_page_paths"])
        self.assertEqual(
            execution_plan["focused_page_set_size"],
            len(execution_plan["focused_page_paths"]),
        )
        self.assertEqual(
            len(execution_plan["dwell_intervals_ms"]),
            max(0, execution_plan["top_level_action_budget"] - 1),
        )
        self.assertTrue(all(dwell >= 2000 for dwell in execution_plan["dwell_intervals_ms"]))
        self.assertEqual(
            execution_plan["session_handles"],
            ["agentic-browser-session-1"],
        )
        self.assertEqual(execution_plan["user_agent_family"], "chrome_desktop")
        self.assertEqual(execution_plan["browser_locale"], "en-US")
        self.assertEqual(execution_plan["transport_profile"], "playwright_chromium")
        self.assertIn("en-US", execution_plan["accept_language"])
        self.assertIn("Mozilla/5.0", execution_plan["user_agent"])

    def test_build_request_mode_realism_execution_plan_shapes_focused_microbursts(self):
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-runtime",
            "tick_id": "llm-fit-tick-4",
            "lane": "bot_red_team",
            "fulfillment_mode": "request_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["http_agent"],
            "capability_envelope": {"max_actions": 12, "max_time_budget_seconds": 120},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "request_mode"),
        }
        generation = {
            "generation_source": "provider_response",
            "provider": "openai",
            "model_id": "gpt-5-mini",
            "actions": [
                {"action_index": 1, "action_type": "http_get", "path": "/", "label": "root"},
                {"action_index": 2, "action_type": "http_get", "path": "/robots.txt", "label": "robots"},
                {"action_index": 3, "action_type": "http_get", "path": "/research/", "label": "research"},
                {"action_index": 4, "action_type": "http_get", "path": "/plans/", "label": "plans"},
                {"action_index": 5, "action_type": "http_get", "path": "/work/", "label": "work"},
            ],
        }

        execution_plan = llm_runtime_worker.build_request_mode_realism_execution_plan(
            fulfillment_plan=plan,
            generation_result=generation,
        )

        self.assertEqual(
            execution_plan["schema_version"],
            "adversary-sim-llm-request-realism-plan.v1",
        )
        self.assertEqual(execution_plan["profile_id"], "agentic.request_mode.v1")
        self.assertEqual(
            len(execution_plan["actions"]),
            execution_plan["effective_activity_budget"],
        )
        self.assertEqual(
            sum(execution_plan["burst_sizes"]),
            execution_plan["effective_activity_budget"],
        )
        self.assertEqual(
            len(execution_plan["inter_action_gaps_ms"]),
            max(0, execution_plan["effective_activity_budget"] - 1),
        )
        executed_paths = [action["path"] for action in execution_plan["actions"]]
        self.assertIn("/", executed_paths)
        self.assertLessEqual(len(set(executed_paths)), 4)
        self.assertEqual(
            execution_plan["focused_page_set_size"],
            len(execution_plan["focused_page_paths"]),
        )
        self.assertEqual(
            execution_plan["peak_concurrent_activities"],
            max(execution_plan["concurrency_group_sizes"]),
        )
        self.assertEqual(
            execution_plan["concurrency_group_sizes"],
            execution_plan["burst_sizes"],
        )
        self.assertTrue(any(gap == 0 for gap in execution_plan["inter_action_gaps_ms"]))
        self.assertTrue(any(gap >= 1000 for gap in execution_plan["inter_action_gaps_ms"]))

    def test_build_request_mode_realism_execution_plan_marks_degraded_identity_without_pool(self):
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-runtime",
            "tick_id": "llm-fit-tick-identity-1",
            "lane": "bot_red_team",
            "fulfillment_mode": "request_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["http_agent"],
            "capability_envelope": {"max_actions": 12, "max_time_budget_seconds": 120},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "request_mode"),
            "request_identity_pool": [],
        }
        generation = {
            "generation_source": "provider_response",
            "provider": "openai",
            "model_id": "gpt-5-mini",
            "actions": [
                {"action_index": 1, "action_type": "http_get", "path": "/", "label": "root"},
                {"action_index": 2, "action_type": "http_get", "path": "/robots.txt", "label": "robots"},
            ],
        }

        execution_plan = llm_runtime_worker.build_request_mode_realism_execution_plan(
            fulfillment_plan=plan,
            generation_result=generation,
        )

        self.assertEqual(execution_plan["identity_realism_status"], "degraded_local")
        self.assertEqual(
            execution_plan["identity_envelope_classes"],
            ["residential", "mobile"],
        )
        self.assertEqual(execution_plan["observed_country_codes"], [])
        self.assertEqual(execution_plan["transport_profile"], "urllib_direct")
        self.assertEqual(
            execution_plan["observed_user_agent_families"],
            ["chrome_android"],
        )
        self.assertEqual(
            execution_plan["observed_accept_languages"],
            ["en-US,en;q=0.9"],
        )
        self.assertEqual(
            len(execution_plan["action_request_headers"]),
            execution_plan["effective_activity_budget"],
        )
        first_headers = execution_plan["action_request_headers"][0]
        self.assertEqual(first_headers["accept-language"], "en-US,en;q=0.9")
        self.assertIn("Mozilla/5.0", first_headers["user-agent"])
        self.assertIn("text/html", first_headers["accept"])

    def test_build_request_mode_realism_execution_plan_aligns_mobile_geo_headers_with_pool_identity(self):
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-runtime",
            "tick_id": "llm-fit-tick-identity-fr",
            "lane": "bot_red_team",
            "fulfillment_mode": "request_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["http_agent"],
            "capability_envelope": {"max_actions": 6, "max_time_budget_seconds": 120},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "request_mode"),
            "request_identity_pool": [
                {
                    "label": "fr-mobile-1",
                    "proxy_url": "http://proxy.example:9001",
                    "identity_class": "mobile",
                    "country_code": "FR",
                }
            ],
        }
        generation = {
            "generation_source": "provider_response",
            "provider": "openai",
            "model_id": "gpt-5-mini",
            "actions": [
                {"action_index": 1, "action_type": "http_get", "path": "/", "label": "root"},
                {"action_index": 2, "action_type": "http_get", "path": "/robots.txt", "label": "robots"},
            ],
        }

        execution_plan = llm_runtime_worker.build_request_mode_realism_execution_plan(
            fulfillment_plan=plan,
            generation_result=generation,
        )

        self.assertEqual(execution_plan["identity_realism_status"], "fixed_proxy")
        self.assertEqual(execution_plan["transport_profile"], "urllib_direct")
        self.assertEqual(execution_plan["observed_country_codes"], ["FR"])
        self.assertEqual(execution_plan["observed_user_agent_families"], ["chrome_android"])
        self.assertEqual(
            execution_plan["observed_accept_languages"],
            ["fr-FR,fr;q=0.9,en-US;q=0.7,en;q=0.6"],
        )
        first_headers = execution_plan["action_request_headers"][0]
        self.assertEqual(
            first_headers["accept-language"],
            "fr-FR,fr;q=0.9,en-US;q=0.7,en;q=0.6",
        )

    def test_extract_llm_fulfillment_plan_requires_nested_plan(self):
        with self.assertRaises(RuntimeError):
            llm_runtime_worker.extract_llm_fulfillment_plan({})

    def test_extract_llm_fulfillment_plan_preserves_canonical_realism_profile(self):
        payload = {
            "llm_fulfillment_plan": {
                "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
                "run_id": "simrun-llm-runtime",
                "tick_id": "llm-fit-tick-0",
                "lane": "bot_red_team",
                "fulfillment_mode": "request_mode",
                "realism_profile": resolve_lane_realism_profile(
                    "bot_red_team",
                    "request_mode",
                ),
            }
        }

        plan = llm_runtime_worker.extract_llm_fulfillment_plan(payload)

        self.assertEqual(
            plan["realism_profile"],
            resolve_lane_realism_profile("bot_red_team", "request_mode"),
        )

    def test_extract_llm_fulfillment_plan_rejects_noncanonical_realism_profile(self):
        payload = {
            "llm_fulfillment_plan": {
                "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
                "run_id": "simrun-llm-runtime",
                "tick_id": "llm-fit-tick-0",
                "lane": "bot_red_team",
                "fulfillment_mode": "request_mode",
                "realism_profile": {
                    **resolve_lane_realism_profile("bot_red_team", "request_mode"),
                    "profile_id": "wrong.profile.v1",
                },
            }
        }

        with self.assertRaises(RuntimeError):
            llm_runtime_worker.extract_llm_fulfillment_plan(payload)

    def test_build_llm_runtime_result_preserves_provider_lineage_and_receipts(self):
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-runtime",
            "tick_id": "llm-fit-tick-1",
            "lane": "bot_red_team",
            "fulfillment_mode": "request_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["http_agent", "ai_scraper_bot"],
            "capability_envelope": {"max_actions": 4, "max_time_budget_seconds": 120},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "request_mode"),
        }
        generation = {
            "generation_source": "provider_response",
            "provider": "openai",
            "model_id": "gpt-5-mini",
            "actions": [
                {"action_index": 1, "action_type": "http_get", "path": "/", "label": "root"},
                {
                    "action_index": 2,
                    "action_type": "http_get",
                    "path": "/robots.txt",
                    "label": "robots",
                },
            ],
        }
        report = {
            "passed": True,
            "terminal_failure": "none",
            "worker_payload": {
                "requests_sent": 2,
                "errors": [],
                "traffic": [
                    {"action_index": 1, "action_type": "http_get", "path": "/", "status": 200},
                    {
                        "action_index": 2,
                        "action_type": "http_get",
                        "path": "/robots.txt",
                        "status": 404,
                    },
                ],
            },
        }

        result = llm_runtime_worker.build_llm_runtime_result(
            fulfillment_plan=plan,
            generation_result=generation,
            report_payload=report,
            tick_completed_at=1_700_000_200,
            worker_id="llm-runtime-worker-test",
        )

        self.assertEqual(result["schema_version"], llm_runtime_worker.LLM_RUNTIME_RESULT_SCHEMA_VERSION)
        self.assertTrue(result["passed"])
        self.assertEqual(result["generation_source"], "provider_response")
        self.assertEqual(result["provider"], "openai")
        self.assertEqual(result["model_id"], "gpt-5-mini")
        self.assertEqual(result["generated_action_count"], 2)
        self.assertEqual(result["executed_action_count"], 2)
        self.assertEqual(result["failed_action_count"], 0)
        self.assertEqual(result["last_response_status"], 404)
        self.assertEqual(result["action_receipts"][1]["status"], 404)

    def test_run_request_mode_blackbox_uses_generated_actions_and_reads_report(self):
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-runtime",
            "tick_id": "llm-fit-tick-3",
            "lane": "bot_red_team",
            "fulfillment_mode": "request_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["http_agent"],
            "capability_envelope": {"max_actions": 3, "max_time_budget_seconds": 120},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "request_mode"),
        }
        generation = {
            "generation_source": "provider_response",
            "provider": "openai",
            "model_id": "gpt-5-mini",
            "actions": [
                {"action_index": 1, "action_type": "http_get", "path": "/", "label": "root"},
                {
                    "action_index": 2,
                    "action_type": "http_get",
                    "path": "/robots.txt",
                    "label": "robots",
                },
            ],
        }

        observed = {}
        with tempfile.TemporaryDirectory() as temp_dir:
            report_path = Path(temp_dir) / "llm-runtime-report.json"

            def fake_runner(command, *, capture_output, text, check, cwd):
                observed["command"] = list(command)
                observed["cwd"] = cwd
                report_path.write_text(
                    json.dumps(
                        {
                            "passed": False,
                            "worker_payload": {
                                "requests_sent": 2,
                                "errors": [],
                                "traffic": [
                                    {
                                        "action_index": 1,
                                        "status": 200,
                                        "path": "/",
                                    },
                                    {
                                        "action_index": 2,
                                        "status": 403,
                                        "path": "/robots.txt",
                                    },
                                ],
                            },
                            "terminal_failure": {"terminal_failure": "", "reason": ""},
                            "worker_failure_detail": "",
                        }
                    ),
                    encoding="utf-8",
                )
                return subprocess.CompletedProcess(
                    args=command,
                    returncode=1,
                    stdout="runner-stdout",
                    stderr="runner-stderr",
                )

            report = llm_runtime_worker.run_request_mode_blackbox(
                base_url="http://127.0.0.1:3000/",
                fulfillment_plan=plan,
                generation_result=generation,
                runner=fake_runner,
                report_path=report_path,
            )

        self.assertEqual(observed["cwd"], str(llm_runtime_worker.REPO_ROOT))
        self.assertIn("--mode", observed["command"])
        self.assertIn("blackbox", observed["command"])
        self.assertIn("--base-url", observed["command"])
        self.assertIn("http://127.0.0.1:3000/", observed["command"])
        self.assertIn("--frontier-actions", observed["command"])
        self.assertIn("--request-realism-plan-json", observed["command"])
        self.assertIn("--request-budget", observed["command"])
        self.assertIn("3", observed["command"])
        self.assertEqual(report["_runner_exit_code"], 1)
        self.assertEqual(report["_runner_stderr"], "runner-stderr")
        self.assertEqual(
            report["worker_payload"]["traffic"][1]["status"],
            403,
        )

    def test_run_browser_mode_blackbox_uses_browser_driver_and_reads_receipt(self):
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-runtime",
            "tick_id": "llm-fit-tick-browser-2",
            "lane": "bot_red_team",
            "fulfillment_mode": "browser_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["browser_agent"],
            "capability_envelope": {"max_actions": 4, "max_time_budget_seconds": 90},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "browser_mode"),
        }
        generation = {
            "generation_source": "provider_response",
            "provider": "openai",
            "model_id": "gpt-5-mini",
            "actions": [
                {
                    "action_index": 1,
                    "action_type": "browser_navigate",
                    "path": "/",
                    "label": "root",
                }
            ],
        }
        observed = {}

        def fake_runner(command, *, input, text, capture_output, timeout, check, env, cwd):
            observed["command"] = list(command)
            observed["input"] = json.loads(input)
            observed["timeout"] = timeout
            observed["env"] = dict(env)
            observed["cwd"] = cwd
            payload = {
                "ok": True,
                "observed_outcome": "browser_session",
                "detail": "ok",
                "top_level_actions": [
                    {
                        "action_index": 1,
                        "action_type": "browser_navigate",
                        "path": "/",
                        "status": 200,
                    },
                    {
                        "action_index": 2,
                        "action_type": "browser_navigate",
                        "path": "/sim/public/research/",
                        "status": 200,
                    },
                ],
                "realism_receipt": {
                    "schema_version": "sim-lane-realism-receipt.v1",
                    "profile_id": "agentic.browser_mode.v1",
                    "planned_activity_budget": 4,
                    "effective_activity_budget": 2,
                    "activity_count": 2,
                    "top_level_action_count": 2,
                    "focused_page_set_size": 2,
                    "dwell_intervals_ms": [2400],
                    "session_handles": ["agentic-browser-session-1"],
                    "identity_rotation_count": 0,
                    "stop_reason": "top_level_budget_exhausted",
                },
                "browser_evidence": {
                    "driver_runtime": "playwright_chromium",
                    "js_executed": True,
                    "dom_events": 4,
                    "request_lineage": [
                        {"method": "GET", "path": "/", "sim_nonce": "nonce-1"},
                        {
                            "method": "GET",
                            "path": "/sim/public/research/",
                            "sim_nonce": "nonce-2",
                        },
                    ],
                    "correlation_ids": ["nonce-1", "nonce-2"],
                },
                "diagnostics": {
                    "action_duration_ms": 4200,
                    "launch_duration_ms": 300,
                    "total_duration_ms": 4500,
                },
            }
            return subprocess.CompletedProcess(
                args=command,
                returncode=0,
                stdout=json.dumps(payload),
                stderr="",
            )

        with mock.patch.dict(
            "os.environ",
            {"SHUMA_SIM_TELEMETRY_SECRET": "browser-mode-test-secret"},
            clear=False,
        ):
            with mock.patch(
                "scripts.supervisor.llm_runtime_worker.ensure_playwright_chromium",
                return_value=mock.Mock(browser_cache="/tmp/pw-cache"),
            ):
                report = llm_runtime_worker.run_browser_mode_blackbox(
                    base_url="http://127.0.0.1:3000/",
                    fulfillment_plan=plan,
                    generation_result=generation,
                    runner=fake_runner,
                )

        self.assertEqual(observed["cwd"], str(llm_runtime_worker.REPO_ROOT))
        self.assertEqual(
            observed["command"][:4],
            ["corepack", "pnpm", "exec", "node"],
        )
        self.assertEqual(
            observed["input"]["action"],
            "agentic_browser_session",
        )
        self.assertEqual(
            observed["input"]["session_plan"]["profile_id"],
            "agentic.browser_mode.v1",
        )
        self.assertEqual(
            report["worker_payload"]["realism_receipt"]["top_level_action_count"],
            2,
        )
        self.assertEqual(report["worker_payload"]["requests_sent"], 2)
        self.assertEqual(len(report["_executed_actions"]), 2)
        self.assertEqual(report["_executed_actions"][1]["path"], "/sim/public/research/")


if __name__ == "__main__":
    unittest.main()
