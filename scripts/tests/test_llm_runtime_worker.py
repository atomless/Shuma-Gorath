#!/usr/bin/env python3

import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from scripts.supervisor import llm_runtime_worker
from scripts.tests.adversarial_runner.contracts import resolve_lane_realism_profile


class LlmRuntimeWorkerUnitTests(unittest.TestCase):
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
        self.assertTrue(any(0 < gap <= 350 for gap in execution_plan["inter_action_gaps_ms"]))
        self.assertTrue(any(gap >= 1000 for gap in execution_plan["inter_action_gaps_ms"]))

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

    def test_build_llm_runtime_result_can_fail_closed_for_unsupported_browser_mode(self):
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-runtime",
            "tick_id": "llm-fit-tick-2",
            "lane": "bot_red_team",
            "fulfillment_mode": "browser_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["browser_agent"],
            "capability_envelope": {"max_actions": 4, "max_time_budget_seconds": 90},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "browser_mode"),
        }
        generation = {
            "generation_source": "fallback_validation_error",
            "provider": "openai",
            "model_id": "gpt-5-mini",
            "fallback_reason": "provider_output_failed_validation",
            "actions": [
                {
                    "action_index": 1,
                    "action_type": "browser_navigate",
                    "path": "/",
                    "label": "root",
                }
            ],
        }

        result = llm_runtime_worker.build_llm_runtime_result(
            fulfillment_plan=plan,
            generation_result=generation,
            report_payload=None,
            tick_completed_at=1_700_000_201,
            worker_id="llm-runtime-worker-test",
            error="browser_mode_dispatch_not_yet_supported_by_blackbox_worker",
            failure_class="transport",
            terminal_failure="browser_mode_not_supported",
        )

        self.assertFalse(result["passed"])
        self.assertEqual(result["failure_class"], "transport")
        self.assertEqual(result["terminal_failure"], "browser_mode_not_supported")
        self.assertEqual(
            result["error"],
            "browser_mode_dispatch_not_yet_supported_by_blackbox_worker",
        )
        self.assertEqual(result["action_receipts"][0]["action_type"], "browser_navigate")
        self.assertEqual(result["action_receipts"][0]["path"], "/")

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


if __name__ == "__main__":
    unittest.main()
