import unittest
import os
import json
import subprocess
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
        "execution_lane": "black_box",
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
    def test_forwarded_headers_include_simulation_metadata(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )
            headers = sim_runner.forwarded_headers("10.0.0.11", user_agent="UnitTest/1.0")
            self.assertIn(runner.SIM_TAG_HEADER_RUN_ID, headers)
            self.assertEqual(headers.get(runner.SIM_TAG_HEADER_PROFILE), "test_profile")
            self.assertEqual(headers.get(runner.SIM_TAG_HEADER_LANE), "deterministic_black_box")
            self.assertIn(runner.SIM_TAG_HEADER_TIMESTAMP, headers)
            self.assertIn(runner.SIM_TAG_HEADER_NONCE, headers)
            self.assertIn(runner.SIM_TAG_HEADER_SIGNATURE, headers)
            self.assertNotIn("X-Shuma-Forwarded-Secret", headers)

    def test_control_plane_headers_split_health_loopback_and_admin_isolation(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

            admin_headers = sim_runner.control_client.admin_headers()
            admin_headers_2 = sim_runner.control_client.admin_headers()
            health_headers = sim_runner.control_client.health_headers()

            self.assertEqual(health_headers.get("X-Forwarded-For"), "127.0.0.1")
            self.assertRegex(str(admin_headers.get("X-Forwarded-For")), r"^127\.0\.\d+\.\d+$")
            self.assertRegex(str(admin_headers_2.get("X-Forwarded-For")), r"^127\.0\.\d+\.\d+$")
            self.assertNotEqual(
                admin_headers.get("X-Forwarded-For"),
                admin_headers_2.get("X-Forwarded-For"),
            )

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
            execution_lane="black_box",
            base_url="http://127.0.0.1:3000",
            scenarios=scenarios,
            frontier_metadata=frontier,
            generated_at_unix=1234,
        )

        self.assertEqual(attack_plan["schema_version"], "attack-plan.v1")
        self.assertEqual(attack_plan["execution_lane"], "black_box")
        self.assertEqual(attack_plan["frontier_mode"], "disabled")
        self.assertEqual(len(attack_plan["candidates"]), 1)
        candidate_payload = attack_plan["candidates"][0]["payload"]
        self.assertEqual(candidate_payload["schema_version"], "frontier_payload_schema.v1")
        self.assertEqual(candidate_payload["scenario"]["ip"], "[masked]")
        self.assertEqual(candidate_payload["target"]["path_hint"], "/sim/public/landing")

    def test_frontier_path_hint_for_scenario_defaults_for_unknown_driver(self):
        self.assertEqual(
            runner.frontier_path_hint_for_scenario({"driver": "allow_browser_allowlist"}),
            "/sim/public/landing",
        )
        self.assertEqual(
            runner.frontier_path_hint_for_scenario({"driver": "not_mapped"}),
            "/",
        )

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
                "tarpit": {
                    "metrics": {
                        "activations": {"progressive": 2},
                        "progress_outcomes": {"advanced": 1},
                        "budget_outcomes": {"fallback_maze": 3},
                    }
                },
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
        self.assertEqual(snapshot["tarpit"]["metrics"]["budget_outcomes"]["fallback_maze"], 3)

    def test_extract_monitoring_snapshot_collects_recent_event_reasons(self):
        payload = {
            "summary": {},
            "details": {
                "events": {
                    "recent_events": [
                        {"reason": "not_a_bot_replay"},
                        {"reason": "cdp_detected:tier=high"},
                        {"reason": "not_a_bot_replay"},
                    ]
                }
            },
        }
        snapshot = runner.extract_monitoring_snapshot(payload)
        self.assertEqual(
            snapshot["recent_event_reasons"],
            ["cdp_detected:tier=high", "not_a_bot_replay"],
        )

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

    def test_profile_expected_defense_categories_filters_to_supported_defense_signals(self):
        selected_scenarios = [
            {"expected_defense_categories": ["pow", "maze", "event_stream"]},
            {"expected_defense_categories": ["geo", "cdp", "not_a_bot"]},
            {"expected_defense_categories": ["pow", "challenge", "rate_limit"]},
        ]
        categories = runner.profile_expected_defense_categories(selected_scenarios)
        self.assertEqual(categories, ["cdp", "challenge", "geo", "maze", "pow", "rate_limit"])

    def test_build_defense_noop_checks_reports_missing_signal_as_failure(self):
        checks = runner.build_defense_noop_checks(
            defense_categories=["pow", "challenge", "maze"],
            coverage_deltas={
                "pow_attempts": 2,
                "pow_successes": 1,
                "pow_failures": 1,
                "challenge_failures": 0,
                "maze_hits": 3,
            },
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertTrue(checks_by_name["defense_noop_detector_pow"]["passed"])
        self.assertFalse(checks_by_name["defense_noop_detector_challenge"]["passed"])
        self.assertTrue(checks_by_name["defense_noop_detector_maze"]["passed"])
        self.assertIn("telemetry_delta_total=0", checks_by_name["defense_noop_detector_challenge"]["detail"])

    def test_build_scenario_execution_evidence_marks_runtime_telemetry_presence(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_a",
            request_count_before=4,
            request_count_after=7,
            monitoring_before={"monitoring_total": 3, "coverage": {"honeypot_hits": 1}},
            monitoring_after={"monitoring_total": 5, "coverage": {"honeypot_hits": 2}},
            simulation_event_count_before=2,
            simulation_event_count_after=3,
            simulation_event_reasons_before=["honeypot"],
            simulation_event_reasons_after=["honeypot", "cdp_detected:tier=high"],
        )
        self.assertEqual(evidence["scenario_id"], "scenario_a")
        self.assertEqual(evidence["runtime_request_count"], 3)
        self.assertEqual(evidence["monitoring_total_delta"], 2)
        self.assertEqual(evidence["coverage_delta_total"], 1)
        self.assertEqual(evidence["coverage_deltas"]["honeypot_hits"], 1)
        self.assertEqual(evidence["simulation_event_count_delta"], 1)
        self.assertEqual(evidence["simulation_event_reasons_delta"], ["cdp_detected:tier=high"])
        self.assertTrue(evidence["has_runtime_telemetry_evidence"])

    def test_build_scenario_execution_evidence_includes_browser_fields_for_browser_realistic_driver(self):
        evidence = runner.build_scenario_execution_evidence(
            scenario_id="scenario_browser",
            request_count_before=10,
            request_count_after=14,
            monitoring_before={"monitoring_total": 2, "coverage": {"geo_maze": 0}},
            monitoring_after={"monitoring_total": 4, "coverage": {"geo_maze": 1}},
            simulation_event_count_before=5,
            simulation_event_count_after=7,
            driver_class="browser_realistic",
            browser_realism={
                "browser_driver_runtime": "playwright_chromium",
                "browser_js_executed": True,
                "browser_dom_events": 3,
                "browser_storage_mode": "stateful_cookie_jar",
                "browser_challenge_dom_path": ["click:#not-a-bot-checkbox"],
                "browser_correlation_ids": ["nonce-a"],
                "browser_request_lineage_count": 4,
            },
        )
        self.assertEqual(evidence["driver_class"], "browser_realistic")
        self.assertEqual(evidence["browser_driver_runtime"], "playwright_chromium")
        self.assertTrue(evidence["browser_js_executed"])
        self.assertEqual(evidence["browser_dom_events"], 3)
        self.assertEqual(evidence["browser_storage_mode"], "stateful_cookie_jar")
        self.assertEqual(evidence["browser_challenge_dom_path"], ["click:#not-a-bot-checkbox"])
        self.assertEqual(evidence["browser_correlation_ids"], ["nonce-a"])
        self.assertEqual(evidence["browser_request_lineage_count"], 4)
        self.assertTrue(evidence["has_browser_execution_evidence"])

    def test_build_runtime_telemetry_evidence_checks_fail_when_passed_scenario_missing_telemetry(self):
        results = [
            runner.ScenarioResult(
                id="scenario_a",
                tier="SIM-T1",
                driver="pow_success",
                expected_outcome="allow",
                observed_outcome="allow",
                passed=True,
                latency_ms=100,
                runtime_budget_ms=1000,
                detail="ok",
            ),
            runner.ScenarioResult(
                id="scenario_b",
                tier="SIM-T3",
                driver="rate_limit_enforce",
                expected_outcome="deny_temp",
                observed_outcome="deny_temp",
                passed=True,
                latency_ms=150,
                runtime_budget_ms=1000,
                detail="ok",
            ),
        ]
        scenario_execution_evidence = {
            "scenario_a": {
                "scenario_id": "scenario_a",
                "runtime_request_count": 3,
                "monitoring_total_delta": 2,
                "coverage_delta_total": 1,
                "simulation_event_count_delta": 1,
                "has_runtime_telemetry_evidence": True,
            },
            "scenario_b": {
                "scenario_id": "scenario_b",
                "runtime_request_count": 1,
                "monitoring_total_delta": 0,
                "coverage_delta_total": 0,
                "simulation_event_count_delta": 0,
                "has_runtime_telemetry_evidence": False,
            },
        }
        checks = runner.build_runtime_telemetry_evidence_checks(
            results=results,
            scenario_execution_evidence=scenario_execution_evidence,
            required_fields=runner.REAL_TRAFFIC_CONTRACT_REQUIRED_SCENARIO_FIELDS,
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertTrue(checks_by_name["runtime_evidence_rows_for_passed_scenarios"]["passed"])
        self.assertTrue(checks_by_name["runtime_evidence_required_fields_present"]["passed"])
        self.assertFalse(checks_by_name["runtime_evidence_telemetry_for_passed_scenarios"]["passed"])
        self.assertIn(
            "scenario_b",
            checks_by_name["runtime_evidence_telemetry_for_passed_scenarios"]["detail"],
        )

    def test_build_browser_execution_evidence_checks_fail_on_missing_browser_proof_fields(self):
        selected_scenarios = [
            {"id": "scenario_browser", "driver": "allow_browser_allowlist", "driver_class": "browser_realistic"},
            {"id": "scenario_http", "driver": "rate_limit_enforce", "driver_class": "http_scraper"},
        ]
        results = [
            runner.ScenarioResult(
                id="scenario_browser",
                tier="SIM-T0",
                driver="allow_browser_allowlist",
                expected_outcome="allow",
                observed_outcome="allow",
                passed=True,
                latency_ms=100,
                runtime_budget_ms=1000,
                detail="ok",
            ),
            runner.ScenarioResult(
                id="scenario_http",
                tier="SIM-T3",
                driver="rate_limit_enforce",
                expected_outcome="deny_temp",
                observed_outcome="deny_temp",
                passed=True,
                latency_ms=100,
                runtime_budget_ms=1000,
                detail="ok",
            ),
        ]
        checks = runner.build_browser_execution_evidence_checks(
            selected_scenarios=selected_scenarios,
            results=results,
            scenario_execution_evidence={
                "scenario_browser": {
                    "scenario_id": "scenario_browser",
                    "driver_class": "browser_realistic",
                    "has_browser_execution_evidence": False,
                    "browser_js_executed": False,
                    "browser_dom_events": 0,
                    "browser_challenge_dom_path": [],
                    "browser_correlation_ids": [],
                    "browser_request_lineage_count": 0,
                }
            },
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertFalse(checks_by_name["browser_execution_required_rows_present"]["passed"])
        self.assertFalse(checks_by_name["browser_execution_js_executed"]["passed"])
        self.assertFalse(checks_by_name["browser_execution_dom_events"]["passed"])
        self.assertFalse(checks_by_name["browser_execution_dom_paths"]["passed"])
        self.assertFalse(checks_by_name["browser_execution_correlation_ids"]["passed"])

    def test_build_scenario_intent_checks_fail_when_required_defense_signal_missing(self):
        selected_scenarios = [
            {
                "id": "sim_t2_geo_challenge",
                "driver": "geo_challenge",
                "driver_class": "browser_realistic",
            }
        ]
        results = [
            runner.ScenarioResult(
                id="sim_t2_geo_challenge",
                tier="SIM-T2",
                driver="geo_challenge",
                expected_outcome="challenge",
                observed_outcome="challenge",
                passed=True,
                latency_ms=120,
                runtime_budget_ms=9000,
                detail="ok",
                realism={
                    "persona": "suspicious_automation",
                    "retry_strategy": "bounded_backoff",
                    "request_sequence_count": 1,
                    "think_time_events": 1,
                    "retry_attempts": 0,
                    "attempts_total": 1,
                },
            )
        ]
        checks = runner.build_scenario_intent_checks(
            selected_scenarios=selected_scenarios,
            results=results,
            scenario_execution_evidence={
                "sim_t2_geo_challenge": {
                    "scenario_id": "sim_t2_geo_challenge",
                    "driver_class": "browser_realistic",
                    "runtime_request_count": 1,
                    "coverage_deltas": {"geo_violations": 0, "geo_challenge": 0},
                    "simulation_event_count_delta": 0,
                    "simulation_event_reasons_delta": [],
                }
            },
        )
        checks_by_name = {check["name"]: check for check in checks}
        self.assertFalse(checks_by_name["scenario_intent_sim_t2_geo_challenge_geo"]["passed"])
        self.assertTrue(checks_by_name["scenario_intent_sim_t2_geo_challenge_challenge"]["passed"])

    def test_build_scenario_intent_checks_pass_when_signals_and_progression_match(self):
        selected_scenarios = [
            {
                "id": "sim_t2_geo_challenge",
                "driver": "geo_challenge",
                "driver_class": "browser_realistic",
            }
        ]
        results = [
            runner.ScenarioResult(
                id="sim_t2_geo_challenge",
                tier="SIM-T2",
                driver="geo_challenge",
                expected_outcome="challenge",
                observed_outcome="challenge",
                passed=True,
                latency_ms=120,
                runtime_budget_ms=9000,
                detail="ok",
                realism={
                    "persona": "suspicious_automation",
                    "retry_strategy": "bounded_backoff",
                    "request_sequence_count": 1,
                    "think_time_events": 1,
                    "retry_attempts": 0,
                    "attempts_total": 1,
                },
            )
        ]
        checks = runner.build_scenario_intent_checks(
            selected_scenarios=selected_scenarios,
            results=results,
            scenario_execution_evidence={
                "sim_t2_geo_challenge": {
                    "scenario_id": "sim_t2_geo_challenge",
                    "driver_class": "browser_realistic",
                    "runtime_request_count": 2,
                    "coverage_deltas": {"geo_violations": 1, "geo_challenge": 1, "challenge_failures": 0},
                    "simulation_event_count_delta": 1,
                    "simulation_event_reasons_delta": ["geo:challenge"],
                }
            },
        )
        failing = [check["name"] for check in checks if not check["passed"]]
        self.assertEqual(failing, [])

    def test_annotate_coverage_checks_with_threshold_sources(self):
        checks = runner.build_coverage_checks({"honeypot_hits": 1}, {"honeypot_hits": 2})
        annotated = runner.annotate_coverage_checks_with_threshold_source({"honeypot_hits": 1}, checks)
        self.assertEqual(
            annotated[0]["threshold_source"],
            "profile.gates.coverage_requirements.honeypot_hits",
        )

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

    def test_validate_manifest_accepts_collateral_and_event_reason_gates(self):
        manifest = minimal_manifest(
            {
                "human_like_collateral_max_ratio": 0.25,
                "required_event_reasons": ["not_a_bot_replay", "cdp_detected"],
                "ip_range_suggestion_seed_required": True,
            }
        )
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_invalid_human_like_collateral_ratio(self):
        manifest = minimal_manifest({"human_like_collateral_max_ratio": 1.2})
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_rejects_empty_required_event_reasons(self):
        manifest = minimal_manifest({"required_event_reasons": []})
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_accepts_new_driver_enum_values(self):
        manifest = minimal_manifest()
        manifest["scenarios"][0]["driver"] = "pow_success"
        manifest["scenarios"][0]["expected_outcome"] = "allow"
        runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_validate_manifest_accepts_tarpit_outcome(self):
        manifest = minimal_manifest()
        manifest["scenarios"][0]["driver"] = "not_a_bot_replay_tarpit_abuse"
        manifest["scenarios"][0]["expected_outcome"] = "tarpit"
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

    def test_validate_manifest_rejects_canonical_intent_matrix_category_drift(self):
        manifest_path = Path("scripts/tests/adversarial/scenario_manifest.v2.json")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["scenarios"][0]["expected_defense_categories"] = ["maze"]
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(manifest_path, manifest, "fast_smoke")

    def test_validate_manifest_rejects_unsupported_execution_lane(self):
        manifest = minimal_manifest()
        manifest["execution_lane"] = "white_box"
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(Path("scripts/tests/adversarial/scenario_manifest.v1.json"), manifest, "test_profile")

    def test_scenario_max_latency_ms_uses_v2_cost_assertions_when_present(self):
        scenario = {"id": "s1", "assertions": {"max_latency_ms": 700}, "cost_assertions": {"max_latency_ms": 333}}
        self.assertEqual(runner.scenario_max_latency_ms(scenario), 333)

    def test_scenario_driver_class_uses_mapping_when_manifest_does_not_set_class(self):
        scenario = {"driver": "pow_success"}
        self.assertEqual(runner.scenario_driver_class(scenario), "cost_imposition")

    def test_compute_cohort_metrics_reports_human_collateral_ratio(self):
        scenarios = [
            {"id": "s1", "tier": "SIM-T0"},
            {"id": "s2", "tier": "SIM-T0"},
            {"id": "s3", "tier": "SIM-T3"},
        ]
        results = [
            runner.ScenarioResult(
                id="s1",
                tier="SIM-T0",
                driver="allow_browser_allowlist",
                expected_outcome="allow",
                observed_outcome="allow",
                passed=True,
                latency_ms=100,
                runtime_budget_ms=1000,
                detail="ok",
            ),
            runner.ScenarioResult(
                id="s2",
                tier="SIM-T0",
                driver="geo_challenge",
                expected_outcome="challenge",
                observed_outcome="challenge",
                passed=True,
                latency_ms=120,
                runtime_budget_ms=1000,
                detail="ok",
            ),
            runner.ScenarioResult(
                id="s3",
                tier="SIM-T3",
                driver="rate_limit_enforce",
                expected_outcome="deny_temp",
                observed_outcome="deny_temp",
                passed=True,
                latency_ms=140,
                runtime_budget_ms=1000,
                detail="ok",
            ),
        ]

        metrics = runner.compute_cohort_metrics(scenarios, results)
        self.assertAlmostEqual(metrics["human_like"]["collateral_ratio"], 0.5, places=3)
        self.assertEqual(metrics["adversarial"]["collateral_count"], 1)

    def test_round_robin_sequence_violations_detect_consecutive_persona_when_others_pending(self):
        sequence = ["human_like", "human_like", "adversarial", "adversarial"]
        violations = runner.round_robin_sequence_violations(sequence)
        self.assertEqual(violations, [1])

    def test_realism_metrics_and_checks_cover_retry_cookie_and_persona_envelopes(self):
        scenarios = [
            {"id": "s1", "tier": "SIM-T1", "traffic_model": {"persona": "benign_automation"}},
            {"id": "s2", "tier": "SIM-T3", "traffic_model": {"persona": "adversarial"}},
        ]
        results = [
            runner.ScenarioResult(
                id="s1",
                tier="SIM-T1",
                driver="not_a_bot_pass",
                expected_outcome="not-a-bot",
                observed_outcome="not-a-bot",
                passed=True,
                latency_ms=80,
                runtime_budget_ms=1000,
                detail="ok",
                realism={
                    "persona": "benign_automation",
                    "retry_strategy": "single_attempt",
                    "state_mode": "stateful_cookie_jar",
                    "think_time_ms_min": 50,
                    "think_time_ms_max": 900,
                    "think_time_events": 1,
                    "think_time_ms_total": 200,
                    "request_sequence_count": 2,
                    "attempts_total": 2,
                    "retry_attempts": 0,
                    "retry_backoff_ms_total": 0,
                    "state_headers_sent": 1,
                    "state_token_observed": 1,
                    "state_store_resets": 0,
                    "state_store_peak_size": 1,
                    "max_attempts_configured": 1,
                },
            ),
            runner.ScenarioResult(
                id="s2",
                tier="SIM-T3",
                driver="retry_storm_enforce",
                expected_outcome="deny_temp",
                observed_outcome="deny_temp",
                passed=True,
                latency_ms=120,
                runtime_budget_ms=1000,
                detail="ok",
                realism={
                    "persona": "adversarial",
                    "retry_strategy": "retry_storm",
                    "state_mode": "stateless",
                    "think_time_ms_min": 5,
                    "think_time_ms_max": 200,
                    "think_time_events": 1,
                    "think_time_ms_total": 50,
                    "request_sequence_count": 2,
                    "attempts_total": 3,
                    "retry_attempts": 1,
                    "retry_backoff_ms_total": 10,
                    "state_headers_sent": 0,
                    "state_token_observed": 0,
                    "state_store_resets": 0,
                    "state_store_peak_size": 0,
                    "max_attempts_configured": 3,
                },
            ),
        ]

        realism_metrics = runner.compute_realism_metrics(
            scenarios, results, persona_scheduler="round_robin"
        )
        checks = runner.build_realism_checks(
            "full_coverage",
            {
                "persona_scheduler": "round_robin",
                "realism": {"enabled": True, "required_retry_attempts": {"retry_storm": 1}},
            },
            realism_metrics,
        )
        failing = [check["name"] for check in checks if not check["passed"]]
        self.assertEqual(failing, [])

    def test_attacker_request_applies_retry_and_stateful_cookie_policy(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        manifest["scenarios"][0]["traffic_model"] = {
            "persona": "adversarial",
            "think_time_ms_min": 0,
            "think_time_ms_max": 0,
            "retry_strategy": "retry_storm",
            "cookie_behavior": "stateful_cookie_jar",
        }

        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        captured_headers = []
        responses = [
            runner.HttpResult(status=500, body="retry", headers={}, latency_ms=1),
            runner.HttpResult(status=200, body="ok", headers={"set-cookie": "sid=abc; Path=/"}, latency_ms=1),
            runner.HttpResult(status=200, body="ok", headers={}, latency_ms=1),
        ]

        def fake_request(method, path, headers=None, json_body=None, form_body=None, plane="attacker", count_request=False):
            self.assertEqual(plane, "attacker")
            captured_headers.append(dict(headers or {}))
            return responses.pop(0)

        sim_runner.request = fake_request  # type: ignore[assignment]
        sim_runner.begin_scenario_execution(manifest["scenarios"][0])
        with patch("scripts.tests.adversarial_simulation_runner.time.sleep", return_value=None):
            first = sim_runner.attacker_request(
                "GET",
                "/",
                headers={"X-Forwarded-For": "10.0.0.1"},
                count_request=True,
            )
            second = sim_runner.attacker_request(
                "GET",
                "/",
                headers={"X-Forwarded-For": "10.0.0.1"},
                count_request=True,
            )
        realism = sim_runner.end_scenario_execution()

        self.assertEqual(first.status, 200)
        self.assertEqual(second.status, 200)
        self.assertEqual(len(captured_headers), 3)
        self.assertEqual(captured_headers[-1].get("Cookie"), "sid=abc")
        self.assertEqual(realism["retry_attempts"], 1)
        self.assertEqual(realism["attempts_total"], 3)
        self.assertEqual(realism["state_headers_sent"], 1)

    def test_execute_browser_realistic_driver_invokes_node_and_records_browser_evidence(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        scenario = manifest["scenarios"][0]
        scenario["driver"] = "allow_browser_allowlist"
        scenario["expected_outcome"] = "allow"
        scenario["driver_class"] = "browser_realistic"
        scenario["traffic_model"]["cookie_behavior"] = "stateful_cookie_jar"

        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.begin_scenario_execution(scenario)
        mocked_stdout = {
            "ok": True,
            "observed_outcome": "allow",
            "detail": "ok",
            "browser_evidence": {
                "driver_runtime": "playwright_chromium",
                "js_executed": True,
                "dom_events": 4,
                "storage_mode": "stateful_cookie_jar",
                "challenge_dom_path": ["read:body"],
                "correlation_ids": ["nonce-1"],
                "request_lineage": [
                    {"method": "GET", "path": "/", "sim_nonce": "nonce-1"},
                    {"method": "GET", "path": "/favicon.ico", "sim_nonce": "nonce-1"},
                ],
            },
            "diagnostics": {},
        }

        with patch(
            "scripts.tests.adversarial_simulation_runner.subprocess.run",
            return_value=subprocess.CompletedProcess(
                args=["corepack", "pnpm", "exec", "node"],
                returncode=0,
                stdout=json.dumps(mocked_stdout),
                stderr="",
            ),
        ) as run_mock:
            observed = sim_runner.execute_browser_realistic_driver(
                scenario,
                action="allow_browser_allowlist",
                headers=sim_runner.forwarded_headers("10.10.10.10", user_agent="UnitTest/1.0"),
                user_agent="UnitTest/1.0",
            )
            self.assertEqual(observed, "allow")
            self.assertEqual(sim_runner.request_count, 2)
            self.assertEqual(run_mock.call_count, 1)

        realism = sim_runner.end_scenario_execution()
        self.assertEqual(realism["browser_driver_runtime"], "playwright_chromium")
        self.assertTrue(realism["browser_js_executed"])
        self.assertEqual(realism["browser_dom_events"], 4)
        self.assertEqual(realism["browser_challenge_dom_path"], ["read:body"])
        self.assertEqual(realism["browser_correlation_ids"], ["nonce-1"])
        self.assertEqual(realism["browser_request_lineage_count"], 2)

    def test_admin_read_request_retries_throttled_reads(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        responses = [
            runner.HttpResult(status=429, body="Too Many Requests", headers={"retry-after": "1"}, latency_ms=1),
            runner.HttpResult(status=200, body="{}", headers={}, latency_ms=1),
        ]
        sim_runner.admin_request = lambda method, path, json_body=None: responses.pop(0)  # type: ignore[assignment]

        with patch("scripts.tests.adversarial_simulation_runner.time.sleep", return_value=None) as sleep_mock:
            result = sim_runner.admin_read_request("GET", "/admin/events")
        self.assertEqual(result.status, 200)
        sleep_mock.assert_called_once()

    def test_admin_read_request_returns_last_throttled_response_after_budget(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        responses = [
            runner.HttpResult(status=429, body="Too Many Requests", headers={"retry-after": "1"}, latency_ms=1),
            runner.HttpResult(status=429, body="Too Many Requests", headers={"retry-after": "1"}, latency_ms=1),
        ]
        sim_runner.admin_request = lambda method, path, json_body=None: responses.pop(0)  # type: ignore[assignment]

        with patch("scripts.tests.adversarial_simulation_runner.time.sleep", return_value=None):
            result = sim_runner.admin_read_request("GET", "/admin/events", max_attempts=2)
        self.assertEqual(result.status, 429)

    def test_run_scenario_attaches_realism_evidence(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        with patch.dict(
            os.environ,
            {
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
                "SHUMA_SIM_TELEMETRY_SECRET": "test-sim-tag-secret",
            },
            clear=False,
        ):
            sim_runner = runner.Runner(
                manifest_path=Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest=manifest,
                profile_name="test_profile",
                execution_lane="black_box",
                base_url="http://127.0.0.1:3000",
                request_timeout_seconds=5.0,
                report_path=Path("scripts/tests/adversarial/latest_report.json"),
            )

        sim_runner.preserve_state = True
        sim_runner.reset_baseline_config = lambda: None  # type: ignore[assignment]
        sim_runner.execute_scenario_driver = lambda scenario: scenario["expected_outcome"]  # type: ignore[assignment]
        result = sim_runner.run_scenario(manifest["scenarios"][0])
        self.assertTrue(result.passed)
        self.assertIsInstance(result.realism, dict)
        self.assertIn("request_sequence_count", result.realism or {})

    def test_validate_manifest_full_coverage_requires_scheduler_and_realism_retry_contract(self):
        manifest = minimal_manifest(schema_version="sim-manifest.v2")
        profile = manifest["profiles"].pop("test_profile")
        manifest["profiles"]["full_coverage"] = profile
        manifest["profiles"]["full_coverage"]["gates"]["realism"] = {"enabled": True}
        with self.assertRaises(runner.SimulationError):
            runner.validate_manifest(
                Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
                manifest,
                "full_coverage",
            )

    def test_enforce_attacker_request_contract_rejects_admin_paths(self):
        with self.assertRaises(runner.SimulationError):
            runner.enforce_attacker_request_contract("/admin/config", {})

    def test_enforce_attacker_request_contract_rejects_privileged_headers(self):
        with self.assertRaises(runner.SimulationError):
            runner.enforce_attacker_request_contract(
                "/",
                {"Authorization": "Bearer test-token"},
            )

    def test_enforce_attacker_request_contract_allows_public_path_without_privileged_headers(self):
        runner.enforce_attacker_request_contract(
            "http://127.0.0.1:3000/challenge/not-a-bot-checkbox",
            {
                "X-Forwarded-For": "10.0.0.1",
                "User-Agent": "UnitTest/1.0",
                runner.SIM_TAG_HEADER_RUN_ID: "run-1",
                runner.SIM_TAG_HEADER_PROFILE: "fast_smoke",
                runner.SIM_TAG_HEADER_LANE: "deterministic_black_box",
                runner.SIM_TAG_HEADER_TIMESTAMP: "1700000000",
                runner.SIM_TAG_HEADER_NONCE: "nonce-1",
                runner.SIM_TAG_HEADER_SIGNATURE: "a" * 64,
            },
        )


if __name__ == "__main__":
    unittest.main()
