import unittest
from unittest.mock import patch

import scripts.tests.check_frontier_payload_artifacts as frontier_governance
import scripts.tests.frontier_lane_attempt as frontier_lane


class FrontierLaneAttemptUnitTests(unittest.TestCase):
    def test_summarize_frontier_lane_detects_missing_keys(self):
        status, advisory = frontier_lane.summarize_frontier_lane(
            [
                {"provider": "openai", "configured": False, "probe_status": "not_configured"},
                {"provider": "anthropic", "configured": False, "probe_status": "not_configured"},
            ]
        )
        self.assertEqual(status, "degraded_missing_keys")
        self.assertIn("No frontier provider keys configured", advisory)

    def test_summarize_frontier_lane_detects_partial_provider_failure(self):
        status, advisory = frontier_lane.summarize_frontier_lane(
            [
                {"provider": "openai", "configured": True, "probe_status": "ok"},
                {"provider": "anthropic", "configured": True, "probe_status": "timeout"},
            ]
        )
        self.assertEqual(status, "degraded_partial_provider_failure")
        self.assertIn("partially succeeded", advisory)

    def test_build_frontier_lane_status_is_advisory_non_blocking(self):
        provider_results = [
            {
                "provider": "openai",
                "model_id": "gpt-5-mini",
                "configured": True,
                "probe_status": "ok",
                "http_status": 200,
                "probe_latency_ms": 10,
            }
        ]
        with patch(
            "scripts.tests.frontier_lane_attempt.build_provider_probe_results",
            return_value=provider_results,
        ):
            lane_status = frontier_lane.build_frontier_lane_status(timeout_seconds=1.0)

        self.assertEqual(lane_status["status"], "ok")
        self.assertFalse(lane_status["blocking"])
        self.assertTrue(lane_status["deterministic_oracle_authoritative"])
        self.assertEqual(lane_status["provider_count_configured"], 1)
        self.assertEqual(lane_status["provider_count_healthy"], 1)


class FrontierGovernanceUnitTests(unittest.TestCase):
    def sample_schema(self):
        return {
            "schema_version": "frontier_payload_schema.v1",
            "allowed_top_level_keys": [
                "schema_version",
                "request_id",
                "profile",
                "scenario",
                "traffic_model",
                "target",
                "public_crawl_content",
                "attack_metadata",
            ],
            "forbidden_field_examples": [
                "api_key",
                "authorization",
                "cookie",
                "session_token",
                "password",
                "secret",
                "raw_ip",
            ],
        }

    def sample_attack_plan(self):
        return {
            "schema_version": "attack-plan.v1",
            "candidates": [
                {
                    "scenario_id": "scenario_1",
                    "payload": {
                        "schema_version": "frontier_payload_schema.v1",
                        "request_id": "req-1",
                        "profile": "fast_smoke",
                        "scenario": {"id": "scenario_1", "ip": "[masked]"},
                        "traffic_model": {"cohort": "adversarial"},
                        "target": {"base_url": "http://127.0.0.1:3000"},
                        "public_crawl_content": {"snippet": "public text"},
                        "attack_metadata": {"note": "ok"},
                    },
                }
            ],
        }

    def sample_report(self):
        return {
            "schema_version": "sim-report.v1",
            "frontier": {
                "frontier_mode": "disabled",
                "provider_count": 0,
                "providers": [],
                "diversity_confidence": "none",
                "reduced_diversity_warning": False,
                "advisory": "No frontier provider keys are configured; run continues without frontier calls.",
            },
        }

    def test_validate_artifacts_accepts_sanitized_payloads(self):
        errors = frontier_governance.validate_artifacts(
            report=self.sample_report(),
            attack_plan=self.sample_attack_plan(),
            schema=self.sample_schema(),
            frontier_secret_values=[],
        )
        self.assertEqual(errors, [])

    def test_validate_artifacts_rejects_forbidden_keys(self):
        bad_report = self.sample_report()
        bad_report["frontier"]["api_key_hint"] = "do-not-allow"
        errors = frontier_governance.validate_artifacts(
            report=bad_report,
            attack_plan=self.sample_attack_plan(),
            schema=self.sample_schema(),
            frontier_secret_values=[],
        )
        self.assertTrue(any("forbidden key path" in error for error in errors))

    def test_validate_artifacts_rejects_secret_leak(self):
        attack_plan = self.sample_attack_plan()
        attack_plan["candidates"][0]["payload"]["attack_metadata"]["note"] = "sk-frontier-secret"
        errors = frontier_governance.validate_artifacts(
            report=self.sample_report(),
            attack_plan=attack_plan,
            schema=self.sample_schema(),
            frontier_secret_values=["sk-frontier-secret"],
        )
        self.assertTrue(any("literal frontier secret value" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
