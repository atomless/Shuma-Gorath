#!/usr/bin/env python3

import copy
import json
import unittest
from datetime import date
from pathlib import Path

import scripts.tests.check_adversarial_scenario_intent_matrix as scenario_intent_check


class AdversarialScenarioIntentMatrixUnitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.matrix_path = Path("scripts/tests/adversarial/scenario_intent_matrix.v1.json")
        self.manifest_path = Path("scripts/tests/adversarial/scenario_manifest.v2.json")
        self.matrix = json.loads(self.matrix_path.read_text(encoding="utf-8"))
        self.manifest = json.loads(self.manifest_path.read_text(encoding="utf-8"))

    def test_scenario_intent_matrix_validator_passes(self):
        errors = scenario_intent_check.validate_scenario_intent_matrix(
            matrix=copy.deepcopy(self.matrix),
            manifest=copy.deepcopy(self.manifest),
            today=date(2026, 2, 28),
        )
        self.assertEqual(errors, [])

    def test_scenario_intent_matrix_validator_reports_stale_review_row(self):
        matrix = copy.deepcopy(self.matrix)
        matrix["review_governance"]["stale_after_days"] = 5
        matrix["rows"][0]["review"]["last_reviewed_on"] = "2026-02-01"
        errors = scenario_intent_check.validate_scenario_intent_matrix(
            matrix=matrix,
            manifest=copy.deepcopy(self.manifest),
            today=date(2026, 2, 28),
        )
        self.assertTrue(any("review is stale" in item for item in errors))

    def test_scenario_intent_matrix_validator_reports_manifest_parity_drift(self):
        matrix = copy.deepcopy(self.matrix)
        matrix["rows"][0]["required_defense_categories"] = ["maze"]
        errors = scenario_intent_check.validate_scenario_intent_matrix(
            matrix=matrix,
            manifest=copy.deepcopy(self.manifest),
            today=date(2026, 2, 28),
        )
        self.assertTrue(any("category mismatch" in item for item in errors))

    def test_scenario_intent_rows_match_current_proven_runtime_contract(self):
        rows_by_id = {row["scenario_id"]: row for row in self.matrix["rows"]}

        self.assertEqual(
            rows_by_id["sim_t3_stale_token_abuse"]["required_defense_categories"],
            ["not_a_bot", "maze"],
        )
        self.assertEqual(
            rows_by_id["sim_t4_tarpit_replay_abuse"]["required_defense_categories"],
            ["not_a_bot", "tarpit"],
        )
        self.assertNotIn(
            "min_retry_attempts",
            rows_by_id["sim_t4_tarpit_replay_abuse"]["progression_requirements"],
        )
        self.assertEqual(
            rows_by_id["sim_t4_cdp_detection_deny"]["required_defense_categories"],
            ["cdp", "ban_path", "event_stream"],
        )
        self.assertEqual(
            rows_by_id["sim_t3_header_spoofing_abuse"]["required_defense_categories"],
            ["fingerprint"],
        )


if __name__ == "__main__":
    unittest.main()
