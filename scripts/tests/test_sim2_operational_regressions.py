#!/usr/bin/env python3

import unittest

import scripts.tests.check_sim2_operational_regressions as regressions


def sample_report():
    return {
        "failure_injection": {
            "cases": [
                {
                    "id": "telemetry_store_delay",
                    "passed": True,
                    "expected_operator_outcome": "degraded_state_visible",
                    "operator_visible_outcome": "degraded_state_visible",
                },
                {
                    "id": "partial_write_failure",
                    "passed": True,
                    "expected_operator_outcome": "partial_write_taxonomy_visible",
                    "operator_visible_outcome": "partial_write_taxonomy_visible",
                },
                {
                    "id": "refresh_race",
                    "passed": True,
                    "expected_operator_outcome": "race_recovery_visible",
                    "operator_visible_outcome": "race_recovery_visible",
                },
            ]
        },
        "prod_mode_monitoring": {
            "p95_visibility_max_ms": 300,
            "profiles": [
                {
                    "id": "prod_like_manual",
                    "traffic_origin": "non_sim",
                    "p95_visibility_ms": 180,
                    "near_realtime_visible": True,
                    "requires_adversary_sim_toggle": False,
                }
            ],
        },
        "retention_lifecycle": {
            "bucket_cutoff_correct": True,
            "purge_watermark_progression": True,
            "purge_lag_hours": 0.5,
            "purge_lag_max_hours": 1.0,
            "read_path_full_keyspace_scan_count": 0,
            "pending_expired_buckets": 0,
        },
        "cost_governance": {
            "guarded_dimension_cardinality_cap_per_hour": 1000,
            "observed_guarded_dimension_cardinality_max": 900,
            "overflow_bucket_accounted": True,
            "overflow_bucket_count": 5,
            "unsampleable_event_drop_count": 0,
            "payload_p95_kb": 320,
            "payload_p95_max_kb": 512,
            "large_payload_sample_count": 4,
            "compression_reduction_percent": 41.5,
            "compression_min_percent": 30,
            "query_budget_avg_req_per_sec_client": 0.4,
            "query_budget_max_req_per_sec_client": 1.0,
        },
        "security_privacy": {
            "field_classification_enforced": True,
            "secret_canary_leak_count": 0,
            "pseudonymization_coverage_percent": 100,
            "pseudonymization_required_percent": 100,
            "high_risk_retention_hours": 48,
            "high_risk_retention_max_hours": 72,
            "incident_hook_emitted": True,
        },
    }


class Sim2OperationalRegressionUnitTests(unittest.TestCase):
    def test_evaluate_report_passes_for_valid_report(self):
        payload = regressions.evaluate_report(sample_report())
        self.assertTrue(payload["status"]["passed"])
        self.assertEqual(payload["status"]["failure_count"], 0)

    def test_evaluate_report_emits_failure_taxonomy_when_thresholds_regress(self):
        report = sample_report()
        report["failure_injection"]["cases"] = [
            {
                "id": "telemetry_store_delay",
                "passed": False,
                "expected_operator_outcome": "degraded_state_visible",
                "operator_visible_outcome": "",
            }
        ]
        report["prod_mode_monitoring"]["profiles"][0]["p95_visibility_ms"] = 900
        report["prod_mode_monitoring"]["profiles"][0][
            "requires_adversary_sim_toggle"
        ] = True
        report["retention_lifecycle"]["read_path_full_keyspace_scan_count"] = 4
        report["retention_lifecycle"]["purge_lag_hours"] = 4
        report["cost_governance"]["observed_guarded_dimension_cardinality_max"] = 5000
        report["cost_governance"]["unsampleable_event_drop_count"] = 3
        report["cost_governance"]["compression_reduction_percent"] = 5
        report["security_privacy"]["secret_canary_leak_count"] = 1
        report["security_privacy"]["incident_hook_emitted"] = False

        payload = regressions.evaluate_report(report)
        self.assertFalse(payload["status"]["passed"])
        joined = " ".join(payload["status"]["failures"])
        self.assertIn("failure_injection_case_failed:", joined)
        self.assertIn("prod_mode_non_sim_visibility_failed:", joined)
        self.assertIn("retention_read_path_scan_regression:", joined)
        self.assertIn("retention_purge_lag_exceeded:", joined)
        self.assertIn("cost_cardinality_cap_exceeded:", joined)
        self.assertIn("cost_unsampleable_event_dropped:", joined)
        self.assertIn("cost_compression_effectiveness_below_threshold:", joined)
        self.assertIn("security_secret_canary_leak_detected:", joined)
        self.assertIn("security_incident_hook_missing:", joined)

    def test_evaluate_report_fails_when_required_domain_sections_are_missing(self):
        payload = regressions.evaluate_report({})
        self.assertFalse(payload["status"]["passed"])
        joined = " ".join(payload["status"]["failures"])
        self.assertIn("domain_missing:failure_injection:", joined)
        self.assertIn("domain_missing:prod_mode_monitoring:", joined)
        self.assertIn("domain_missing:retention_lifecycle:", joined)
        self.assertIn("domain_missing:cost_governance:", joined)
        self.assertIn("domain_missing:security_privacy:", joined)

    def test_evaluate_report_fails_when_required_retention_cost_security_metrics_are_missing(self):
        report = sample_report()
        del report["retention_lifecycle"]["purge_lag_hours"]
        del report["cost_governance"]["payload_p95_kb"]
        del report["security_privacy"]["secret_canary_leak_count"]

        payload = regressions.evaluate_report(report)
        self.assertFalse(payload["status"]["passed"])
        joined = " ".join(payload["status"]["failures"])
        self.assertIn("domain_missing_metric:retention_lifecycle:", joined)
        self.assertIn("missing_metrics=purge_lag_hours", joined)
        self.assertIn("domain_missing_metric:cost_governance:", joined)
        self.assertIn("missing_metrics=payload_p95_kb", joined)
        self.assertIn("domain_missing_metric:security_privacy:", joined)
        self.assertIn("missing_metrics=secret_canary_leak_count", joined)


if __name__ == "__main__":
    unittest.main()
