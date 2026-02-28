#!/usr/bin/env python3

import unittest

import scripts.tests.check_sim2_verification_matrix as matrix_check


def sample_manifest():
    return {
        "schema_version": "sim-manifest.v2",
        "scenarios": [
            {
                "id": "scenario_allow",
                "expected_defense_categories": ["allowlist"],
            }
        ],
    }


def sample_report():
    return {
        "execution_lane": "black_box",
        "monitoring_before": {"coverage": {"ban_count": 1}},
        "monitoring_after": {"coverage": {"ban_count": 2}},
        "results": [
            {
                "id": "scenario_allow",
                "execution_evidence": {
                    "runtime_request_count": 2,
                    "monitoring_total_delta": 2,
                    "coverage_delta_total": 1,
                },
            }
        ],
        "evidence": {
            "run": {"request_id_lineage": {"sim_run_id": "run-1"}},
            "scenario_execution": [
                {
                    "scenario_id": "scenario_allow",
                    "has_runtime_telemetry_evidence": True,
                }
            ],
        },
    }


class Sim2VerificationMatrixUnitTests(unittest.TestCase):
    def test_validate_matrix_passes_with_expected_evidence(self):
        matrix = {
            "rows": [
                {
                    "row_id": "allow_row",
                    "defense_category": "allowlist",
                    "required_scenarios": ["scenario_allow"],
                    "required_lanes": ["black_box"],
                    "required_evidence_types": [
                        "runtime_telemetry",
                        "monitoring_delta",
                        "coverage_delta",
                        "lineage",
                    ],
                    "lineage_segment": "request_id_lineage",
                }
            ]
        }
        payload = matrix_check.validate_matrix(
            matrix,
            sample_manifest(),
            sample_report(),
            container_report=None,
            allow_missing_container_report=True,
        )
        self.assertTrue(payload["status"]["passed"])
        self.assertEqual(payload["status"]["failure_count"], 0)

    def test_validate_matrix_emits_missing_row_evidence_and_lineage_diagnostics(self):
        matrix = {
            "rows": [
                {
                    "row_id": "bad_row",
                    "defense_category": "not_a_bot",
                    "required_scenarios": ["scenario_allow"],
                    "required_lanes": ["black_box"],
                    "required_evidence_types": ["runtime_telemetry", "lineage"],
                    "lineage_segment": "scenario_execution",
                }
            ]
        }
        bad_report = sample_report()
        bad_report["results"][0]["execution_evidence"]["runtime_request_count"] = 0
        bad_report["evidence"]["run"]["request_id_lineage"] = {}
        bad_report["evidence"]["scenario_execution"][0]["has_runtime_telemetry_evidence"] = False
        payload = matrix_check.validate_matrix(
            matrix,
            sample_manifest(),
            bad_report,
            container_report=None,
            allow_missing_container_report=True,
        )
        self.assertFalse(payload["status"]["passed"])
        joined = " ".join(payload["status"]["failures"])
        self.assertIn("missing_matrix_row:", joined)
        self.assertIn("missing_evidence_type:", joined)
        self.assertIn("failing_telemetry_lineage_segment:", joined)


if __name__ == "__main__":
    unittest.main()
