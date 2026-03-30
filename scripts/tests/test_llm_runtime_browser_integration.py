#!/usr/bin/env python3

import os
import unittest
from unittest import mock

from scripts.supervisor import llm_runtime_worker
from scripts.tests.adversarial_runner.contracts import resolve_lane_realism_profile


class LlmRuntimeBrowserIntegrationTests(unittest.TestCase):
    def test_run_browser_mode_blackbox_emits_real_session_traffic_against_local_public_site(self):
        base_url = str(os.environ.get("SHUMA_BASE_URL") or "http://127.0.0.1:3000").strip()
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-browser-integration",
            "tick_id": "llm-browser-live-1",
            "tick_started_at": 1_700_000_500,
            "lane": "bot_red_team",
            "fulfillment_mode": "browser_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["browser_agent"],
            "capability_envelope": {"max_actions": 4, "max_time_budget_seconds": 90},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "browser_mode"),
        }
        generation = {
            "generation_source": "fallback_provider_unavailable",
            "provider": "",
            "model_id": "",
            "fallback_reason": "integration_browser_session",
            "actions": [
                {
                    "action_index": 1,
                    "action_type": "browser_navigate",
                    "path": "/",
                    "label": "root",
                }
            ],
        }

        with mock.patch.dict(
            os.environ,
            {
                **os.environ,
                "SHUMA_SIM_TELEMETRY_SECRET": str(
                    os.environ.get("SHUMA_SIM_TELEMETRY_SECRET")
                    or "browser-mode-live-test-secret"
                ),
            },
            clear=True,
        ):
            report = llm_runtime_worker.run_browser_mode_blackbox(
                base_url=base_url,
                fulfillment_plan=plan,
                generation_result=generation,
            )

        self.assertTrue(report["passed"], report)
        worker_payload = report["worker_payload"]
        receipts = worker_payload["traffic"]
        self.assertGreaterEqual(worker_payload["requests_sent"], 2, receipts)
        self.assertEqual(receipts[0]["path"], "/")
        self.assertTrue(any(row["path"] != "/" for row in receipts), receipts)

        realism_receipt = worker_payload["realism_receipt"]
        self.assertEqual(realism_receipt["profile_id"], "agentic.browser_mode.v1")
        self.assertEqual(realism_receipt["top_level_action_count"], len(receipts))
        self.assertEqual(
            len(realism_receipt["dwell_intervals_ms"]),
            max(0, realism_receipt["top_level_action_count"] - 1),
        )
        self.assertEqual(realism_receipt["identity_rotation_count"], 0)

        browser_evidence = worker_payload["browser_evidence"]
        self.assertEqual(browser_evidence["driver_runtime"], "playwright_chromium")
        self.assertTrue(browser_evidence["js_executed"])
        self.assertGreaterEqual(len(browser_evidence["request_lineage"]), len(receipts))


if __name__ == "__main__":
    unittest.main()
