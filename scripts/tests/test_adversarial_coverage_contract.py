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


if __name__ == "__main__":
    unittest.main()
