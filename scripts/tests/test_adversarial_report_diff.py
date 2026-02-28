#!/usr/bin/env python3

import unittest

import scripts.tests.adversarial_report_diff as report_diff


def sample_report(
    *,
    scenario_rows,
    latency_p95=200,
    suite_runtime_ms=1200,
    request_count=10,
    collateral_ratio=0.1,
    coverage_deltas=None,
):
    return {
        "schema_version": "sim-report.v1",
        "results": scenario_rows,
        "gates": {
            "checks": [
                {
                    "name": "latency_p95",
                    "observed": latency_p95,
                }
            ]
        },
        "suite_runtime_ms": suite_runtime_ms,
        "request_count": request_count,
        "cohort_metrics": {
            "human_like": {
                "collateral_ratio": collateral_ratio,
            }
        },
        "coverage_gates": {
            "coverage": {
                "deltas": coverage_deltas or {"challenge_failures": 1, "ban_count": 1}
            }
        },
    }


class AdversarialReportDiffUnitTests(unittest.TestCase):
    def test_compare_reports_tracks_pass_regression_and_shift_metrics(self):
        baseline = sample_report(
            scenario_rows=[
                {"id": "s1", "passed": True, "observed_outcome": "allow"},
                {"id": "s2", "passed": False, "observed_outcome": "maze"},
            ],
            latency_p95=200,
            suite_runtime_ms=1200,
            request_count=10,
            collateral_ratio=0.1,
            coverage_deltas={"challenge_failures": 1, "ban_count": 1},
        )
        candidate = sample_report(
            scenario_rows=[
                {"id": "s1", "passed": False, "observed_outcome": "monitor"},
                {"id": "s2", "passed": True, "observed_outcome": "allow"},
                {"id": "s3", "passed": False, "observed_outcome": "deny_temp"},
            ],
            latency_p95=260,
            suite_runtime_ms=1500,
            request_count=14,
            collateral_ratio=0.2,
            coverage_deltas={"challenge_failures": 3, "ban_count": 0},
        )

        diff = report_diff.compare_reports(baseline, candidate)
        transitions = diff["scenario_transitions"]
        self.assertEqual(transitions["new_passes"], ["s2"])
        self.assertEqual(transitions["new_regressions"], ["s1"])
        self.assertEqual(transitions["new_scenarios"], ["s3"])
        self.assertEqual(diff["cost_shift"]["latency_p95_delta_ms"], 60)
        self.assertEqual(diff["cost_shift"]["suite_runtime_delta_ms"], 300)
        self.assertEqual(diff["collateral_shift"]["human_like_collateral_ratio_delta"], 0.1)
        self.assertIn(
            {"metric": "challenge_failures", "delta": 2},
            diff["defense_delta_shift"]["increased"],
        )

    def test_build_backlog_candidates_only_uses_new_regressions(self):
        diff = {
            "scenario_transitions": {
                "new_regressions": ["sim_t4_cdp_detection_deny"],
                "new_passes": ["sim_t2_geo_challenge"],
                "new_scenarios": [],
            }
        }
        backlog = report_diff.build_backlog_candidates(
            diff,
            owner="runtime_engineering",
            disposition_sla_hours=48,
        )
        self.assertEqual(len(backlog), 1)
        item = backlog[0]
        self.assertEqual(item["scenario_id"], "sim_t4_cdp_detection_deny")
        self.assertEqual(item["owner"], "runtime_engineering")
        self.assertEqual(item["disposition_sla_hours"], 48)


if __name__ == "__main__":
    unittest.main()
