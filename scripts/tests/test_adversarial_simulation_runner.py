import unittest
from pathlib import Path

import scripts.tests.adversarial_simulation_runner as runner


def minimal_manifest(gates_extra=None):
    gates = {
        "latency": {"p95_max_ms": 1000},
        "outcome_ratio_bounds": {"allow": {"min": 0.0, "max": 1.0}},
        "telemetry_amplification": {
            "max_fingerprint_events_per_request": 1.0,
            "max_monitoring_events_per_request": 1.0,
        },
    }
    if gates_extra:
        gates.update(gates_extra)
    return {
        "schema_version": "sim-manifest.v1",
        "suite_id": "unit-tests",
        "description": "Unit test manifest",
        "profiles": {
            "test_profile": {
                "description": "test profile",
                "max_runtime_seconds": 60,
                "scenario_ids": ["scenario_allow"],
                "gates": gates,
            }
        },
        "scenarios": [
            {
                "id": "scenario_allow",
                "description": "allow scenario",
                "tier": "SIM-T0",
                "driver": "allow_browser_allowlist",
                "expected_outcome": "allow",
                "ip": "10.10.10.10",
                "user_agent": "UnitTest/1.0",
                "seed": 1,
                "runtime_budget_ms": 1000,
                "assertions": {"max_latency_ms": 500},
            }
        ],
    }


class AdversarialRunnerUnitTests(unittest.TestCase):
    def test_extract_monitoring_snapshot_maps_coverage_fields(self):
        payload = {
            "summary": {
                "honeypot": {"total_hits": 3},
                "challenge": {"total_failures": 2},
                "not_a_bot": {"submitted": 4, "pass": 1, "fail": 2, "replay": 1, "escalate": 0},
                "pow": {"total_attempts": 7, "total_successes": 5, "total_failures": 2},
                "rate": {"total_violations": 6, "outcomes": {"limited": 4, "banned": 2}},
                "geo": {"total_violations": 5, "actions": {"challenge": 2, "maze": 2, "block": 1}},
            },
            "details": {
                "analytics": {"ban_count": 9},
                "events": {"recent_events": [{}, {}, {}]},
                "maze": {"total_hits": 8},
                "tarpit": {"metrics": {"activations": {"progressive": 2}, "progress_outcomes": {"advanced": 1}}},
                "cdp": {
                    "stats": {"total_detections": 4},
                    "fingerprint_stats": {"events": 11},
                },
            },
        }

        snapshot = runner.extract_monitoring_snapshot(payload)

        self.assertEqual(snapshot["fingerprint_events"], 11)
        self.assertEqual(snapshot["coverage"]["honeypot_hits"], 3)
        self.assertEqual(snapshot["coverage"]["rate_limited"], 4)
        self.assertEqual(snapshot["coverage"]["geo_block"], 1)
        self.assertEqual(snapshot["coverage"]["tarpit_activations_progressive"], 2)
        self.assertEqual(snapshot["coverage"]["recent_event_count"], 3)
        self.assertEqual(snapshot["components"]["not_a_bot_submitted"], 4)

    def test_compute_coverage_deltas_clamps_negative_values(self):
        before = {"honeypot_hits": 5, "geo_maze": 3}
        after = {"honeypot_hits": 3, "geo_maze": 7}
        deltas = runner.compute_coverage_deltas(before, after)
        self.assertEqual(deltas["honeypot_hits"], 0)
        self.assertEqual(deltas["geo_maze"], 4)

    def test_build_coverage_checks_reports_pass_and_fail(self):
        checks = runner.build_coverage_checks(
            {"honeypot_hits": 1, "geo_block": 2},
            {"honeypot_hits": 3, "geo_block": 1},
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertTrue(checks_by_name["coverage_honeypot_hits"]["passed"])
        self.assertFalse(checks_by_name["coverage_geo_block"]["passed"])

    def test_validate_manifest_accepts_supported_coverage_requirements(self):
        manifest = minimal_manifest(
            {"coverage_requirements": {"honeypot_hits": 1, "geo_maze": 1}}
        )
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_unknown_coverage_requirement_key(self):
        manifest = minimal_manifest({"coverage_requirements": {"unknown_counter": 1}})
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(
                Path("scripts/tests/adversarial/scenario_manifest.v1.json"),
                manifest,
                "test_profile",
            )


if __name__ == "__main__":
    unittest.main()
