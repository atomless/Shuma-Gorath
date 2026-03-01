#!/usr/bin/env python3

import copy
import unittest

import scripts.tests.check_sim2_governance_contract as governance_check


def sample_contract():
    return {
        "schema_version": "sim2-hybrid-lane-contract.v1",
        "deterministic_conformance_lane": {"release_blocking": True},
        "emergent_exploration_lane": {
            "release_blocking": False,
            "runtime_budget_seconds_max": 180,
            "action_budget_max": 500,
        },
        "choreography_boundary": {
            "intentionally_choreographed": [
                "seed_scenarios",
                "invariant_assertions",
                "resource_guardrails",
            ],
            "must_be_emergent": [
                "crawl_strategy",
                "attack_sequencing",
                "adaptation",
            ],
        },
        "objective_model": {
            "target_assets": ["public_http_surface"],
            "success_functions": ["unexpected_allow_outcome"],
            "allowed_adaptation_space": ["path_selection"],
            "stop_conditions": ["runtime_budget_exhausted"],
        },
        "novelty_scoring": {
            "dimensions": ["novelty", "severity", "confidence", "replayability"]
        },
        "promotion_pipeline": {
            "steps": [
                "generated_candidate",
                "deterministic_replay_confirmation",
                "owner_review_disposition",
                "promoted_blocking_scenario",
            ],
            "blocking_requires_deterministic_confirmation": True,
        },
        "promotion_thresholds": {
            "deterministic_confirmation_min_percent": 95,
            "false_discovery_max_percent": 20,
            "owner_disposition_sla_hours": 48,
        },
        "program_governance": {
            "cadence": {
                "cycle": "run -> review -> tune -> replay -> promote",
                "frequency": "weekly",
            },
            "ownership": {
                "adversary_owner_role": "security_engineering",
                "defense_owner_role": "runtime_engineering",
                "operations_owner_role": "platform_operations",
            },
            "promotion_rubric_dimensions": [
                "severity",
                "reproducibility",
                "collateral_risk",
                "mitigation_readiness",
            ],
            "kpis": [
                "attacker_cost_shift",
                "human_friction_impact",
                "detection_latency",
                "mitigation_lead_time",
                "time_to_regression_confirmation",
                "time_to_mitigation",
                "collateral_ceiling",
                "cost_asymmetry_trend",
            ],
            "rollback_playbook": {
                "required_actions": [
                    "rollback_to_last_known_good",
                    "validate_with_adversarial_fast",
                ]
            },
            "architecture_review": {
                "frequency": "monthly",
                "documented_outcomes_required": True,
            },
        },
    }


def sample_promotion_script():
    return "\n".join(
        [
            "HYBRID_CONFIRMATION_MIN_PERCENT = 95.0",
            "HYBRID_FALSE_DISCOVERY_MAX_PERCENT = 20.0",
            "HYBRID_OWNER_DISPOSITION_SLA_HOURS = 48",
            "blocking_requires_deterministic_confirmation = True",
        ]
    )


def sample_operator_guide():
    return "\n".join(
        [
            "## Continuous Defender-Adversary Evolution Cadence (SIM2-GC-12)",
            "## Run-to-Run Diff + Backlog Automation (SIM2-EX8-2 / SIM2-EX8-3)",
            "## Promotion Hygiene and Scenario Corpus Maintenance (SIM2-EX8-4)",
            "## Hybrid Adversary Lane Contract (SIM2-GC-14)",
            "<=180s and <=500 actions",
            "time to regression confirmation",
            "time to mitigation",
            "collateral_ceiling",
            "cost_asymmetry_trend",
        ]
    )


def sample_operational_report():
    return {"status": {"passed": True}}


def sample_realtime_report():
    return {"status": {"passed": True}}


def sample_matrix_report():
    return {"status": {"passed": True}}


class Sim2GovernanceContractTests(unittest.TestCase):
    def test_evaluate_passes_for_valid_contract_and_markers(self):
        payload = governance_check.evaluate(
            sample_contract(),
            sample_promotion_script(),
            sample_operator_guide(),
            sample_operational_report(),
            sample_realtime_report(),
            sample_matrix_report(),
        )
        self.assertTrue(payload["status"]["passed"])
        self.assertEqual(payload["status"]["failure_count"], 0)

    def test_evaluate_fails_for_invalid_thresholds_and_missing_markers(self):
        broken = copy.deepcopy(sample_contract())
        broken["emergent_exploration_lane"]["runtime_budget_seconds_max"] = 600
        broken["promotion_thresholds"]["deterministic_confirmation_min_percent"] = 60
        bad_operational_report = {"status": {"passed": False}}
        payload = governance_check.evaluate(
            broken,
            "no markers",
            "no headings",
            bad_operational_report,
            sample_realtime_report(),
            sample_matrix_report(),
        )
        self.assertFalse(payload["status"]["passed"])
        joined = " ".join(payload["status"]["failures"])
        self.assertIn("hybrid_lane_budget_envelope_invalid:", joined)
        self.assertIn("hybrid_lane_thresholds_invalid:", joined)
        self.assertIn("governance_promotion_threshold_mismatch:", joined)
        self.assertIn("governance_operator_guide_marker_missing:", joined)
        self.assertIn("governance_operational_artifact_failed:", joined)


if __name__ == "__main__":
    unittest.main()
