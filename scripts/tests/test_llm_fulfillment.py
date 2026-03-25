#!/usr/bin/env python3

import unittest

from scripts.tests.adversarial_runner import llm_fulfillment


class LlmFulfillmentUnitTests(unittest.TestCase):
    def test_load_llm_fulfillment_contract_accepts_repo_contracts(self):
        contract = llm_fulfillment.load_llm_fulfillment_contract()
        self.assertEqual(
            contract["schema_version"], llm_fulfillment.LLM_FULFILLMENT_PLAN_SCHEMA_VERSION
        )
        self.assertEqual(contract["frontier_action_contract_id"], "frontier_action_contract.v1")
        self.assertEqual(
            contract["container_runtime_profile_id"], "container_runtime_profile.v1"
        )
        self.assertEqual(
            contract["backend_kinds"], ["frontier_reference", "local_candidate"]
        )
        self.assertIn("browser_mode", contract["modes"])
        self.assertIn("request_mode", contract["modes"])
        self.assertEqual(contract["black_box_boundary"]["position"], "outside_attacker")
        self.assertTrue(contract["black_box_boundary"]["host_root_only_entrypoint"])
        self.assertTrue(contract["black_box_boundary"]["shuma_blind"])
        self.assertFalse(contract["black_box_boundary"]["web_search_allowed"])
        self.assertIn(
            "robots_txt", contract["black_box_boundary"]["public_host_hint_sources"]
        )
        self.assertIn(
            "shuma_repo", contract["black_box_boundary"]["forbidden_knowledge_sources"]
        )

    def test_build_llm_fulfillment_plan_marks_single_provider_frontier_as_degraded(self):
        plan = llm_fulfillment.build_llm_fulfillment_plan(
            run_id="simrun-llm-fit",
            generated_tick_count=0,
            frontier_metadata={
                "provider_count": 1,
                "mode": "single_provider_self_play",
                "reduced_diversity_warning": True,
            },
            now=1_700_000_000,
        )

        self.assertEqual(plan["lane"], "bot_red_team")
        self.assertEqual(plan["fulfillment_mode"], "browser_mode")
        self.assertEqual(plan["backend_kind"], "frontier_reference")
        self.assertEqual(plan["backend_state"], "degraded")
        self.assertEqual(plan["backend_id"], "frontier_reference:single_provider_self_play")
        self.assertIn("local_candidate", plan["supported_backend_kinds"])
        self.assertEqual(
            plan["category_targets"],
            ["automated_browser", "browser_agent", "agent_on_behalf_of_human"],
        )
        self.assertEqual(
            plan["capability_envelope"]["allowed_tools"],
            ["browser_navigate", "browser_snapshot", "browser_click"],
        )
        self.assertTrue(plan["capability_envelope"]["browser_automation_allowed"])
        self.assertFalse(plan["capability_envelope"]["direct_request_emission_allowed"])
        self.assertEqual(plan["black_box_boundary"]["position"], "outside_attacker")
        self.assertTrue(plan["black_box_boundary"]["public_knowledge_only"])
        self.assertFalse(plan["black_box_boundary"]["repo_visibility_allowed"])
        self.assertFalse(plan["black_box_boundary"]["judge_visibility_allowed"])
        self.assertTrue(plan["black_box_boundary"]["receipt_requirements"]["attack_trace_required"])

    def test_build_llm_fulfillment_plan_uses_request_mode_when_frontier_is_unavailable(self):
        plan = llm_fulfillment.build_llm_fulfillment_plan(
            run_id="simrun-llm-fit",
            generated_tick_count=1,
            frontier_metadata={
                "provider_count_configured": 0,
                "frontier_mode": "disabled",
                "reduced_diversity_warning": False,
            },
            now=1_700_000_001,
        )

        self.assertEqual(plan["fulfillment_mode"], "request_mode")
        self.assertEqual(plan["backend_kind"], "frontier_reference")
        self.assertEqual(plan["backend_state"], "unavailable")
        self.assertEqual(plan["backend_id"], "frontier_reference:unconfigured")
        self.assertEqual(plan["category_targets"], ["http_agent", "ai_scraper_bot"])
        self.assertEqual(plan["capability_envelope"]["allowed_tools"], ["http_get"])
        self.assertFalse(plan["capability_envelope"]["browser_automation_allowed"])
        self.assertTrue(plan["capability_envelope"]["direct_request_emission_allowed"])


if __name__ == "__main__":
    unittest.main()
