#!/usr/bin/env python3

import unittest

from scripts.tests.adversarial_runner import contracts, llm_fulfillment


class AdversarialLaneRealismContractUnitTests(unittest.TestCase):
    def test_lane_realism_contract_loader_accepts_repo_contract(self):
        contract = contracts.load_lane_realism_contract()

        self.assertEqual(contract["schema_version"], "sim-lane-realism-contract.v1")
        self.assertIn("scrapling_traffic", contract["profiles"])
        self.assertIn("bot_red_team", contract["profiles"])

    def test_llm_fulfillment_plan_surfaces_realism_profile(self):
        plan = llm_fulfillment.build_llm_fulfillment_plan(
            run_id="simrun-llm-fit",
            generated_tick_count=1,
            frontier_metadata={
                "provider_count_configured": 0,
                "frontier_mode": "disabled",
                "reduced_diversity_warning": False,
            },
            now=1_700_000_011,
        )

        profile = plan.get("realism_profile")
        self.assertIsInstance(profile, dict)
        self.assertEqual(
            profile,
            contracts.resolve_lane_realism_profile("bot_red_team", "request_mode"),
        )


if __name__ == "__main__":
    unittest.main()
