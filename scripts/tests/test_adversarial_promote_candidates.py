import unittest

import scripts.tests.adversarial_promote_candidates as promote


def sample_attack_plan(frontier_mode="single_provider_self_play", diversity_confidence="low"):
    return {
        "schema_version": "attack-plan.v1",
        "profile": "full_coverage",
        "frontier_mode": frontier_mode,
        "provider_count": 1 if frontier_mode == "single_provider_self_play" else 2,
        "providers": [
            {"provider": "openai", "model_id": "gpt-5-mini", "configured": True},
            {"provider": "anthropic", "model_id": "claude-3-5-haiku-latest", "configured": True},
        ],
        "diversity_confidence": diversity_confidence,
        "candidates": [
            {
                "scenario_id": "sim_t4_cdp_detection_deny",
                "tier": "SIM-T4",
                "driver": "cdp_high_confidence_deny",
                "payload": {
                    "target": {"path_hint": "/"},
                    "traffic_model": {
                        "user_agent": "ShumaAdversarial/1.0",
                        "retry_strategy": "bounded_backoff",
                    },
                },
            }
        ],
    }


def sample_report():
    return {
        "schema_version": "sim-report.v1",
        "profile": "full_coverage",
        "results": [
            {
                "id": "sim_t4_cdp_detection_deny",
                "driver": "cdp_high_confidence_deny",
                "expected_outcome": "deny_temp",
                "observed_outcome": "deny_temp",
                "passed": True,
                "latency_ms": 500,
                "runtime_budget_ms": 8000,
            }
        ],
    }


class PromotionPipelineUnitTests(unittest.TestCase):
    def test_stable_finding_id_is_deterministic(self):
        record = {
            "scenario_family": "cdp_high_confidence_deny",
            "path": "/",
            "headers": {"user_agent": "ShumaAdversarial/1.0"},
            "cadence_pattern": {"retry_strategy": "bounded_backoff"},
        }
        first = promote.stable_finding_id(record)
        second = promote.stable_finding_id(record)
        self.assertEqual(first, second)
        self.assertTrue(first.startswith("simf-"))

    def test_normalize_findings_include_frontier_diversity_metadata(self):
        findings = promote.normalize_findings(
            attack_plan=sample_attack_plan(
                frontier_mode="multi_provider_playoff", diversity_confidence="higher"
            ),
            report=sample_report(),
        )
        self.assertEqual(len(findings), 1)
        finding = findings[0]
        self.assertEqual(finding["frontier_mode"], "multi_provider_playoff")
        self.assertEqual(finding["diversity_confidence"], "higher")
        self.assertEqual(finding["scenario_family"], "cdp_high_confidence_deny")
        self.assertEqual(finding["observed_outcome"], "deny_temp")

    def test_classify_replay_outcome_reports_confirmed_and_not_reproducible(self):
        finding = promote.normalize_findings(sample_attack_plan(), sample_report())[0]
        confirmed = promote.classify_replay_outcome(
            finding=finding,
            replay_result={
                "status": "ok",
                "observed_outcome": "deny_temp",
                "passed": True,
                "latency_ms": 540,
            },
        )
        self.assertEqual(confirmed, "confirmed_reproducible")

        drifted = promote.classify_replay_outcome(
            finding=finding,
            replay_result={
                "status": "ok",
                "observed_outcome": "monitor",
                "passed": False,
                "latency_ms": 540,
            },
        )
        self.assertEqual(drifted, "not_reproducible")

    def test_promotion_policy_requires_owner_review_by_mode(self):
        finding = promote.normalize_findings(sample_attack_plan(), sample_report())[0]
        replay = {
            "status": "ok",
            "observed_outcome": "deny_temp",
            "passed": True,
            "latency_ms": 500,
        }
        decision = promote.build_promotion_decision(
            finding=finding,
            replay_result=replay,
            classification="confirmed_reproducible",
        )
        self.assertTrue(decision["owner_review_required"])
        self.assertFalse(decision["blocking_regression"])
        self.assertIn("single_provider_self_play", decision["review_notes"][0])

        multi_finding = promote.normalize_findings(
            sample_attack_plan(
                frontier_mode="multi_provider_playoff", diversity_confidence="higher"
            ),
            sample_report(),
        )[0]
        multi_decision = promote.build_promotion_decision(
            finding=multi_finding,
            replay_result=replay,
            classification="confirmed_reproducible",
        )
        self.assertTrue(multi_decision["owner_review_required"])
        self.assertIn("higher initial confidence", " ".join(multi_decision["review_notes"]))

    def test_hybrid_governance_thresholds_pass_when_rates_are_within_bounds(self):
        now_unix = 1_700_000_000
        lineage = []
        for index in range(20):
            classification = "confirmed_reproducible" if index < 19 else "not_reproducible"
            lineage.append(
                {
                    "classification": classification,
                    "promotion": {
                        "owner_review_required": True,
                        "owner_disposition": "pending",
                        "owner_disposition_due_at_unix": now_unix + 3600,
                    },
                }
            )
        governance = promote.evaluate_hybrid_governance(lineage, now_unix=now_unix)
        self.assertTrue(governance["thresholds_passed"])
        self.assertEqual(governance["failures"], [])
        self.assertEqual(
            governance["observed"]["deterministic_confirmation_rate_percent"],
            95.0,
        )
        self.assertEqual(governance["observed"]["false_discovery_rate_percent"], 5.0)

    def test_hybrid_governance_thresholds_fail_on_low_confirmation_high_false_discovery_and_sla(self):
        now_unix = 1_700_000_000
        lineage = []
        for index in range(10):
            classification = "confirmed_reproducible" if index < 7 else "not_reproducible"
            lineage.append(
                {
                    "classification": classification,
                    "promotion": {
                        "owner_review_required": True,
                        "owner_disposition": "pending",
                        "owner_disposition_due_at_unix": now_unix - 3600,
                    },
                }
            )
        governance = promote.evaluate_hybrid_governance(lineage, now_unix=now_unix)
        self.assertFalse(governance["thresholds_passed"])
        joined = " ".join(governance["failures"])
        self.assertIn("deterministic_confirmation_rate_below_min", joined)
        self.assertIn("false_discovery_rate_above_max", joined)
        self.assertIn("owner_disposition_sla_exceeded", joined)


if __name__ == "__main__":
    unittest.main()
