#!/usr/bin/env python3

import copy
import json
import unittest
from pathlib import Path

import scripts.tests.adversarial_simulation_runner as sim_runner
import scripts.tests.check_adversarial_coverage_contract as coverage_contract_check


class AdversarialCoverageContractUnitTests(unittest.TestCase):
    def test_coverage_contract_validator_passes(self):
        errors = coverage_contract_check.validate_coverage_contract()
        self.assertEqual(errors, [])

    def test_full_coverage_manifest_requires_exact_contract_keys(self):
        manifest_path = Path("scripts/tests/adversarial/scenario_manifest.v2.json")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        modified = copy.deepcopy(manifest)
        coverage_requirements = modified["profiles"]["full_coverage"]["gates"]["coverage_requirements"]
        removed_key = next(iter(coverage_requirements.keys()))
        del coverage_requirements[removed_key]

        with self.assertRaises(sim_runner.SimulationError):
            sim_runner.validate_manifest(manifest_path, modified, "full_coverage")

    def test_full_coverage_manifest_requires_exact_depth_requirement_rows(self):
        manifest_path = Path("scripts/tests/adversarial/scenario_manifest.v2.json")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        modified = copy.deepcopy(manifest)
        depth_requirements = modified["profiles"]["full_coverage"]["gates"][
            "coverage_depth_requirements"
        ]
        removed_key = next(iter(depth_requirements.keys()))
        del depth_requirements[removed_key]

        with self.assertRaises(sim_runner.SimulationError):
            sim_runner.validate_manifest(manifest_path, modified, "full_coverage")

    def test_coverage_contract_reflects_current_proven_depth_rows(self):
        contract = json.loads(
            Path("scripts/tests/adversarial/coverage_contract.v2.json").read_text(
                encoding="utf-8"
            )
        )
        depth_rows = contract["coverage_depth_requirements"]
        self.assertNotIn("tarpit_progression_depth", depth_rows)
        self.assertIn("event_stream_health_depth", depth_rows)

    def test_coverage_contract_freezes_non_human_lane_fulfillment_matrix(self):
        contract = json.loads(
            Path("scripts/tests/adversarial/coverage_contract.v2.json").read_text(
                encoding="utf-8"
            )
        )
        section = contract["non_human_lane_fulfillment"]
        categories = section["categories"]

        self.assertEqual(section["schema_version"], "sim-non-human-lane-fulfillment.v1")
        self.assertEqual(
            sorted(categories.keys()),
            sorted(
                [
                    "indexing_bot",
                    "ai_scraper_bot",
                    "automated_browser",
                    "http_agent",
                    "browser_agent",
                    "agent_on_behalf_of_human",
                    "verified_beneficial_bot",
                    "unknown_non_human",
                ]
            ),
        )
        self.assertEqual(categories["indexing_bot"]["runtime_lane"], "scrapling_traffic")
        self.assertEqual(categories["indexing_bot"]["fulfillment_mode"], "scrapling_worker")
        self.assertEqual(categories["verified_beneficial_bot"]["assignment_status"], "gap")
        self.assertEqual(categories["unknown_non_human"]["assignment_status"], "gap")
        self.assertEqual(
            categories["agent_on_behalf_of_human"]["supporting_scenarios"],
            ["sim_t1_not_a_bot_pass"],
        )


if __name__ == "__main__":
    unittest.main()
