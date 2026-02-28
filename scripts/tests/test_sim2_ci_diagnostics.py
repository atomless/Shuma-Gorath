#!/usr/bin/env python3

import unittest

import scripts.tests.render_sim2_ci_diagnostics as diagnostics


def sample_report():
    return {
        "profile": "full_coverage",
        "execution_lane": "black_box",
        "generated_at_unix": 1_700_000_000,
        "suite_runtime_ms": 12345,
        "passed": True,
        "simulation_event_reasons": ["not_a_bot_pass", "geo_block"],
        "evidence": {
            "run": {
                "decision_outcomes": {"allow": 2, "deny_temp": 1},
                "defenses_touched": ["not_a_bot", "geo"],
            },
            "scenario_execution": [
                {
                    "scenario_id": "sim_a",
                    "runtime_request_count": 3,
                    "monitoring_total_delta": 4,
                    "coverage_delta_total": 2,
                    "simulation_event_count_delta": 1,
                    "has_runtime_telemetry_evidence": True,
                }
            ],
        },
        "gates": {
            "checks": [{"name": "latency", "passed": True, "detail": "ok"}]
        },
        "coverage_gates": {
            "checks": [{"name": "coverage", "passed": True, "detail": "ok"}]
        },
        "realism_gates": {
            "checks": [{"name": "persona", "passed": True, "detail": "ok"}]
        },
    }


class Sim2CiDiagnosticsUnitTests(unittest.TestCase):
    def test_render_diagnostics_extracts_timeline_event_counts_and_refresh_traces(self):
        payload = diagnostics.render_diagnostics(sample_report())
        self.assertEqual(payload["schema_version"], "sim2-ci-diagnostics.v1")
        self.assertEqual(len(payload["timeline_snapshots"]), 1)
        self.assertEqual(payload["timeline_snapshots"][0]["scenario_id"], "sim_a")
        self.assertEqual(
            payload["event_counts"]["simulation_event_reason_count"],
            2,
        )
        self.assertEqual(len(payload["refresh_traces"]), 3)


if __name__ == "__main__":
    unittest.main()
