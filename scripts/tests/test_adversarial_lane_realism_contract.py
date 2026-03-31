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

    def test_profiles_surface_bounded_recurrence_envelopes(self):
        request_profile = contracts.resolve_lane_realism_profile(
            "bot_red_team",
            "request_mode",
        )
        crawler_profile = contracts.resolve_lane_realism_profile(
            "scrapling_traffic",
            "crawler",
        )

        for profile in (request_profile, crawler_profile):
            recurrence = profile["recurrence_envelope"]
            self.assertEqual(
                recurrence["strategy"],
                "bounded_single_tick_reentry",
            )
            self.assertEqual(recurrence["reentry_scope"], "within_run")
            self.assertGreaterEqual(recurrence["dormant_gap_seconds"]["min"], 1)
            self.assertGreaterEqual(
                recurrence["dormant_gap_seconds"]["max"],
                recurrence["dormant_gap_seconds"]["min"],
            )
            self.assertGreaterEqual(recurrence["max_reentries_per_run"], 1)

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
        self.assertIn(
            "secondary_capture_mode",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "secondary_request_count",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "background_request_count",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "subresource_request_count",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "recurrence_strategy",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "session_index",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "reentry_count",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "max_reentries_per_run",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "planned_dormant_gap_seconds",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "action_types_attempted",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "capability_state",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "targeting_strategy",
            profile["receipt_contract"]["required_fields"],
        )

    def test_profiles_surface_mode_specific_exploration_envelopes(self):
        crawler_profile = contracts.resolve_lane_realism_profile(
            "scrapling_traffic",
            "crawler",
        )
        bulk_profile = contracts.resolve_lane_realism_profile(
            "scrapling_traffic",
            "bulk_scraper",
        )

        self.assertIn("exploration_envelope", crawler_profile)
        self.assertIn("exploration_envelope", bulk_profile)
        self.assertGreaterEqual(
            crawler_profile["exploration_envelope"]["max_depth"],
            1,
        )
        self.assertGreater(
            bulk_profile["exploration_envelope"]["max_depth"],
            crawler_profile["exploration_envelope"]["max_depth"],
        )
        self.assertGreater(
            bulk_profile["exploration_envelope"]["max_bytes"],
            crawler_profile["exploration_envelope"]["max_bytes"],
        )
        for field in (
            "visited_url_count",
            "discovered_url_count",
            "deepest_depth_reached",
            "sitemap_documents_seen",
            "frontier_remaining_count",
            "canonical_public_pages_reached",
        ):
            self.assertIn(
                field,
                crawler_profile["receipt_contract"]["required_fields"],
            )

    def test_request_mode_profile_requires_capability_and_targeting_receipt_fields(self):
        profile = contracts.resolve_lane_realism_profile("bot_red_team", "request_mode")

        self.assertIn(
            "action_types_attempted",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "capability_state",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "targeting_strategy",
            profile["receipt_contract"]["required_fields"],
        )

    def test_scrapling_browser_profile_requires_secondary_traffic_receipt_fields(self):
        profile = contracts.resolve_lane_realism_profile("scrapling_traffic", "browser_automation")

        self.assertIn(
            "secondary_capture_mode",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "secondary_request_count",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "background_request_count",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "subresource_request_count",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "recurrence_strategy",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "session_index",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "reentry_count",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "max_reentries_per_run",
            profile["receipt_contract"]["required_fields"],
        )
        self.assertIn(
            "planned_dormant_gap_seconds",
            profile["receipt_contract"]["required_fields"],
        )


if __name__ == "__main__":
    unittest.main()
