import unittest
from pathlib import Path
from unittest.mock import patch

import scripts.tests.adversarial_simulation_runner as runner


def minimal_manifest(gates_extra=None, schema_version="sim-manifest.v1"):
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
    scenario = {
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
    if schema_version == "sim-manifest.v2":
        scenario["driver_class"] = "browser_realistic"
        scenario["traffic_model"] = {
            "persona": "human_like",
            "think_time_ms_min": 10,
            "think_time_ms_max": 120,
            "retry_strategy": "single_attempt",
            "cookie_behavior": "stateful_cookie_jar",
        }
        scenario["expected_defense_categories"] = ["allowlist"]
        scenario["coverage_tags"] = ["SIM-T0", "allow_browser_allowlist"]
        scenario["cost_assertions"] = {"max_latency_ms": 500}

    return {
        "schema_version": schema_version,
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
        "scenarios": [scenario],
    }


class AdversarialRunnerUnitTests(unittest.TestCase):
    def test_build_frontier_metadata_reports_disabled_single_and_multi_modes(self):
        with patch("scripts.tests.adversarial_simulation_runner.read_env_local_value", return_value=""):
            with patch.dict("os.environ", {}, clear=True):
                disabled = runner.build_frontier_metadata()
                self.assertEqual(disabled["frontier_mode"], "disabled")
                self.assertEqual(disabled["provider_count"], 0)
                self.assertEqual(disabled["diversity_confidence"], "none")
                self.assertFalse(disabled["reduced_diversity_warning"])

            with patch.dict(
                "os.environ",
                {"SHUMA_FRONTIER_OPENAI_API_KEY": "sk-openai", "SHUMA_FRONTIER_OPENAI_MODEL": "gpt-5-mini"},
                clear=True,
            ):
                single = runner.build_frontier_metadata()
                self.assertEqual(single["frontier_mode"], "single_provider_self_play")
                self.assertEqual(single["provider_count"], 1)
                self.assertEqual(single["diversity_confidence"], "low")
                self.assertTrue(single["reduced_diversity_warning"])

            with patch.dict(
                "os.environ",
                {
                    "SHUMA_FRONTIER_OPENAI_API_KEY": "sk-openai",
                    "SHUMA_FRONTIER_ANTHROPIC_API_KEY": "sk-anthropic",
                },
                clear=True,
            ):
                multi = runner.build_frontier_metadata()
                self.assertEqual(multi["frontier_mode"], "multi_provider_playoff")
                self.assertEqual(multi["provider_count"], 2)
                self.assertEqual(multi["diversity_confidence"], "higher")
                self.assertFalse(multi["reduced_diversity_warning"])

    def test_frontier_payload_sanitization_drops_forbidden_and_masks_quasi_identifiers(self):
        payload = {
            "schema_version": "frontier_payload_schema.v1",
            "request_id": "req-1",
            "profile": "fast_smoke",
            "scenario": {
                "id": "scenario_allow",
                "ip": "203.0.113.10",
                "api_key": "must-not-leak",
            },
            "traffic_model": {"cohort": "adversarial", "session_token": "must-drop"},
            "target": {"base_url": "http://127.0.0.1:3000", "authorization": "Bearer leaked"},
            "public_crawl_content": {"snippet": "public text"},
            "attack_metadata": {"cookie": "session=abc123"},
        }

        sanitized = runner.sanitize_frontier_payload(payload)
        serialized = str(sanitized)
        self.assertNotIn("must-not-leak", serialized)
        self.assertNotIn("session=abc123", serialized)
        self.assertNotIn("Bearer leaked", serialized)
        self.assertEqual(sanitized["scenario"]["ip"], "[masked]")

    def test_frontier_payload_schema_rejects_unknown_top_level_keys(self):
        invalid_payload = {
            "schema_version": "frontier_payload_schema.v1",
            "request_id": "req-1",
            "profile": "fast_smoke",
            "scenario": {"id": "scenario_allow"},
            "traffic_model": {"cohort": "adversarial"},
            "target": {"base_url": "http://127.0.0.1:3000"},
            "public_crawl_content": {"snippet": "public text"},
            "attack_metadata": {"note": "ok"},
            "forbidden_top_level": {"api_key": "secret"},
        }
        with self.assertRaises(runner.SimulationError):
            runner.validate_frontier_payload_schema(invalid_payload)

    def test_build_attack_plan_emits_sanitized_candidates(self):
        frontier = {
            "frontier_mode": "disabled",
            "provider_count": 0,
            "providers": [],
            "diversity_confidence": "none",
        }
        scenarios = [
            {
                "id": "scenario_allow",
                "tier": "SIM-T0",
                "driver": "allow_browser_allowlist",
                "expected_outcome": "allow",
                "runtime_budget_ms": 1000,
                "seed": 1,
                "ip": "198.51.100.10",
                "user_agent": "UnitTest/1.0",
                "description": "allow scenario",
            }
        ]
        attack_plan = runner.build_attack_plan(
            profile_name="fast_smoke",
            base_url="http://127.0.0.1:3000",
            scenarios=scenarios,
            frontier_metadata=frontier,
            generated_at_unix=1234,
        )

        self.assertEqual(attack_plan["schema_version"], "attack-plan.v1")
        self.assertEqual(attack_plan["frontier_mode"], "disabled")
        self.assertEqual(len(attack_plan["candidates"]), 1)
        candidate_payload = attack_plan["candidates"][0]["payload"]
        self.assertEqual(candidate_payload["schema_version"], "frontier_payload_schema.v1")
        self.assertEqual(candidate_payload["scenario"]["ip"], "[masked]")

    def test_has_leading_zero_bits_accepts_full_and_partial_prefixes(self):
        self.assertTrue(runner.has_leading_zero_bits(bytes.fromhex("00ff"), 8))
        self.assertTrue(runner.has_leading_zero_bits(bytes.fromhex("0fff"), 4))
        self.assertFalse(runner.has_leading_zero_bits(bytes.fromhex("10ff"), 4))

    def test_solve_pow_nonce_returns_valid_nonce_for_low_difficulty(self):
        seed = "unit-seed"
        difficulty = 8
        nonce = runner.solve_pow_nonce(seed, difficulty, max_iter=200_000)
        self.assertGreaterEqual(nonce, 0)
        digest = runner.pow_digest(seed, nonce)
        self.assertTrue(runner.has_leading_zero_bits(digest, difficulty))

    def test_find_invalid_pow_nonce_returns_nonce_that_fails_target(self):
        seed = "unit-seed"
        difficulty = 12
        nonce = runner.find_invalid_pow_nonce(seed, difficulty, max_iter=20)
        self.assertGreaterEqual(nonce, 0)
        digest = runner.pow_digest(seed, nonce)
        self.assertFalse(runner.has_leading_zero_bits(digest, difficulty))

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

    def test_validate_manifest_accepts_new_driver_enum_values(self):
        manifest = minimal_manifest()
        manifest["scenarios"][0]["driver"] = "pow_success"
        manifest["scenarios"][0]["expected_outcome"] = "allow"
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_accepts_v2_contract_fields(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v2.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_v2_missing_traffic_model(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        del manifest["scenarios"][0]["traffic_model"]
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v2.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_v2_driver_class_mismatch(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        manifest["scenarios"][0]["driver_class"] = "http_scraper"
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v2.json"), manifest, "test_profile")

    def test_scenario_max_latency_ms_uses_v2_cost_assertions_when_present(self):
        scenario = {"id": "s1", "assertions": {"max_latency_ms": 700}, "cost_assertions": {"max_latency_ms": 333}}
        self.assertEqual(runner.scenario_max_latency_ms(scenario), 333)

    def test_scenario_driver_class_uses_mapping_when_manifest_does_not_set_class(self):
        scenario = {"driver": "pow_success"}
        self.assertEqual(runner.scenario_driver_class(scenario), "cost_imposition")


if __name__ == "__main__":
    unittest.main()
