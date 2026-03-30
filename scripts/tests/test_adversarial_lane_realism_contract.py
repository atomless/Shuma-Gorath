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

    def test_request_mode_profile_surfaces_identity_envelope_contract(self):
        profile = contracts.resolve_lane_realism_profile("bot_red_team", "request_mode")

        self.assertEqual(
            profile["identity_envelope"]["geo_affinity_mode"],
            "pool_aligned",
        )
        self.assertEqual(
            profile["identity_envelope"]["session_stickiness"],
            "stable_per_identity",
        )
        self.assertEqual(
            profile["identity_envelope"]["degraded_without_pool"],
            "local_session_only",
        )
        self.assertIn(
            "residential",
            profile["identity_envelope"]["identity_classes"],
        )

    def test_bulk_scraper_profile_surfaces_transport_envelope_contract(self):
        profile = contracts.resolve_lane_realism_profile("scrapling_traffic", "bulk_scraper")

        self.assertEqual(
            profile["transport_envelope"]["request_client_posture"],
            "mobile_browser_like",
        )
        self.assertEqual(
            profile["transport_envelope"]["accept_language_strategy"],
            "identity_geo_aligned",
        )
        self.assertEqual(
            profile["transport_envelope"]["request_transport_profile"],
            "curl_impersonate",
        )

    def test_browser_mode_profile_surfaces_browser_locale_transport_contract(self):
        profile = contracts.resolve_lane_realism_profile("bot_red_team", "browser_mode")

        self.assertEqual(
            profile["transport_envelope"]["browser_client_posture"],
            "desktop_browser_like",
        )
        self.assertEqual(
            profile["transport_envelope"]["browser_locale_strategy"],
            "identity_geo_aligned",
        )
        self.assertEqual(
            profile["transport_envelope"]["browser_transport_profile"],
            "playwright_chromium",
        )


if __name__ == "__main__":
    unittest.main()
