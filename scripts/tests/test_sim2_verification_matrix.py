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


def sample_container_report():
    return {
        "passed": True,
        "frontier_lineage": {"lineage_complete": True},
        "policy_audit": {"violation_count": 0},
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

    def test_validate_matrix_fails_when_container_lane_evidence_is_missing_in_strict_mode(self):
        matrix = {
            "rows": [
                {
                    "row_id": "allow_row",
                    "defense_category": "allowlist",
                    "required_scenarios": ["scenario_allow"],
                    "required_lanes": ["black_box"],
                    "required_evidence_types": ["runtime_telemetry"],
                    "lineage_segment": "request_id_lineage",
                },
                {
                    "row_id": "container_lane_row",
                    "defense_category": "frontier_container",
                    "required_scenarios": [],
                    "required_lanes": ["container_blackbox"],
                    "required_evidence_types": [
                        "container_passed",
                        "frontier_lineage_complete",
                        "policy_violation_zero",
                    ],
                    "lineage_segment": "frontier_lineage",
                }
            ]
        }
        payload = matrix_check.validate_matrix(
            matrix,
            sample_manifest(),
            sample_report(),
            container_report=None,
            allow_missing_container_report=False,
        )
        self.assertFalse(payload["status"]["passed"])
        joined = " ".join(payload["status"]["failures"])
        self.assertIn(
            "missing_matrix_row:row=container_lane_row:container_report_missing",
            joined,
        )

    def test_validate_matrix_passes_when_container_lane_evidence_is_present(self):
        matrix = {
            "rows": [
                {
                    "row_id": "allow_row",
                    "defense_category": "allowlist",
                    "required_scenarios": ["scenario_allow"],
                    "required_lanes": ["black_box"],
                    "required_evidence_types": ["runtime_telemetry"],
                    "lineage_segment": "request_id_lineage",
                },
                {
                    "row_id": "container_lane_row",
                    "defense_category": "frontier_container",
                    "required_scenarios": [],
                    "required_lanes": ["container_blackbox"],
                    "required_evidence_types": [
                        "container_passed",
                        "frontier_lineage_complete",
                        "policy_violation_zero",
                    ],
                    "lineage_segment": "frontier_lineage",
                }
            ]
        }
        payload = matrix_check.validate_matrix(
            matrix,
            sample_manifest(),
            sample_report(),
            container_report=sample_container_report(),
            allow_missing_container_report=False,
        )
        self.assertTrue(payload["status"]["passed"])
        self.assertEqual(payload["status"]["failure_count"], 0)

    def test_validate_matrix_advisory_mode_allows_missing_scenarios_in_report(self):
        matrix = {
            "rows": [
                {
                    "row_id": "allow_row",
                    "defense_category": "allowlist",
                    "required_scenarios": ["scenario_allow", "scenario_missing"],
                    "required_lanes": ["black_box"],
                    "required_evidence_types": ["runtime_telemetry"],
                    "lineage_segment": "request_id_lineage",
                }
            ]
        }
        manifest = sample_manifest()
        manifest["scenarios"].append(
            {
                "id": "scenario_missing",
                "expected_defense_categories": ["allowlist"],
            }
        )
        payload = matrix_check.validate_matrix(
            matrix,
            manifest,
            sample_report(),
            container_report=None,
            allow_missing_container_report=True,
        )
        self.assertTrue(payload["status"]["passed"])
        self.assertEqual(payload["status"]["failure_count"], 0)


if __name__ == "__main__":
    unittest.main()
