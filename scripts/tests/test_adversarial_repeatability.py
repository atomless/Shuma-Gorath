#!/usr/bin/env python3

import unittest

import scripts.tests.adversarial_repeatability as repeatability


def sample_report(latency_ms=10, outcome="allow", gate_pass=True):
    return {
        "results": [
            {
                "id": "sim_a",
                "passed": True,
                "observed_outcome": outcome,
                "latency_ms": latency_ms,
            }
        ],
        "gates": {
            "checks": [
                {"name": "latency_p95", "passed": gate_pass},
            ]
        },
        "coverage_gates": {
            "coverage": {
                "deltas": {"honeypot_hits": 1}
            }
        },
    }


class AdversarialRepeatabilityUnitTests(unittest.TestCase):
    def test_compare_reports_allows_latency_within_tolerance(self):
        baseline = sample_report(latency_ms=100)
        candidate = sample_report(latency_ms=120)
        diffs = repeatability.compare_reports(baseline, candidate, latency_tolerance_ms=25)
        self.assertEqual(diffs, [])

    def test_compare_reports_flags_latency_outside_tolerance(self):
        baseline = sample_report(latency_ms=100)
        candidate = sample_report(latency_ms=200)
        diffs = repeatability.compare_reports(baseline, candidate, latency_tolerance_ms=25)
        self.assertTrue(any("latency drift" in diff for diff in diffs))

    def test_compare_reports_flags_outcome_drift(self):
        baseline = sample_report(outcome="allow")
        candidate = sample_report(outcome="maze")
        diffs = repeatability.compare_reports(baseline, candidate, latency_tolerance_ms=1000)
        self.assertTrue(any("outcome drift" in diff for diff in diffs))

    def test_compare_reports_flags_gate_drift(self):
        baseline = sample_report(gate_pass=True)
        candidate = sample_report(gate_pass=False)
        diffs = repeatability.compare_reports(baseline, candidate, latency_tolerance_ms=1000)
        self.assertTrue(any("gate latency_p95" in diff for diff in diffs))


if __name__ == "__main__":
    unittest.main()
