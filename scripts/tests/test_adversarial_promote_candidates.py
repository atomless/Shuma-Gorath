import json
import unittest
from unittest import mock

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
        "generation_summary": {
            "seed_candidate_count": 1,
            "generated_candidate_count": 1,
            "accepted_candidate_count": 2,
            "rejected_candidate_count": 0,
        },
        "discovery_quality_metrics": {
            "candidate_count": 2,
            "generated_candidate_count": 1,
            "novel_candidate_count": 1,
            "provider_outage_impact_percent": 0.0,
        },
        "candidates": [
            {
                "candidate_id": "cand-seed-sim_t4_cdp_detection_deny",
                "source_scenario_id": "sim_t4_cdp_detection_deny",
                "generation_kind": "seed",
                "mutation_class": "seed",
                "behavioral_class": "baseline",
                "novelty_score": 0.0,
                "governance_passed": True,
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
            },
            {
                "candidate_id": "cand-mut-sim_t4_cdp_detection_deny-retry",
                "source_scenario_id": "sim_t4_cdp_detection_deny",
                "generation_kind": "mutation",
                "mutation_class": "retry_strategy",
                "behavioral_class": "timing_variation",
                "novelty_score": 0.72,
                "governance_passed": True,
                "scenario_id": "sim_t4_cdp_detection_deny",
                "tier": "SIM-T4",
                "driver": "cdp_high_confidence_deny",
                "payload": {
                    "target": {"path_hint": "/"},
                    "traffic_model": {
                        "user_agent": "ShumaAdversarial/1.0 mutated",
                        "retry_strategy": "retry_storm",
                    },
                },
            },
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


def sample_promotion_payload():
    return {
        "schema_version": "adversarial-promotion.v1",
        "generated_at_unix": 1_700_000_123,
        "frontier": {
            "frontier_mode": "multi_provider_playoff",
            "provider_count": 2,
            "diversity_confidence": "higher",
        },
        "hybrid_governance": {
            "thresholds_passed": True,
            "failures": [],
            "observed": {
                "deterministic_confirmation_rate_percent": 100.0,
                "false_discovery_rate_percent": 0.0,
                "overdue_owner_review_count": 0,
            },
        },
        "discovery_quality_metrics": {
            "candidate_count": 2,
            "generated_candidate_count": 1,
            "novel_confirmed_regressions": 1,
            "false_discovery_rate_percent": 0.0,
            "provider_outage_impact_percent": 0.0,
            "provider_outage_status": "healthy",
            "blocking_requires_deterministic_confirmation": True,
        },
        "summary": {
            "total_findings": 2,
            "replay_candidates": 1,
            "classification_counts": {
                "confirmed_reproducible": 1,
                "not_reproducible": 0,
                "needs_manual_review": 0,
            },
            "confirmed_regression_count": 1,
            "novel_confirmed_regression_count": 1,
            "false_discovery_rate_percent": 0.0,
            "provider_outage_impact_percent": 0.0,
            "blocking_required": True,
        },
        "lineage": [],
    }


class PromotionPipelineUnitTests(unittest.TestCase):
    def test_hybrid_lane_constants_are_stable(self):
        self.assertEqual(promote.DETERMINISTIC_CONFORMANCE_LANE, "deterministic_conformance")
        self.assertEqual(promote.EMERGENT_EXPLORATION_LANE, "emergent_exploration")
        self.assertEqual(
            str(promote.DEFAULT_HYBRID_LANE_CONTRACT_PATH),
            "scripts/tests/adversarial/hybrid_lane_contract.v1.json",
        )

    def test_materialize_backend_replay_promotion_requires_runtime_endpoint_and_api_key(self):
        with mock.patch.dict(promote.os.environ, {}, clear=True):
            with self.assertRaisesRegex(ValueError, "missing SHUMA_BASE_URL"):
                promote.materialize_backend_replay_promotion(sample_promotion_payload())

        with mock.patch.dict(
            promote.os.environ,
            {"SHUMA_BASE_URL": "http://127.0.0.1:3000"},
            clear=True,
        ):
            with self.assertRaisesRegex(ValueError, "missing SHUMA_API_KEY"):
                promote.materialize_backend_replay_promotion(sample_promotion_payload())

    def test_materialize_backend_replay_promotion_posts_payload_and_requires_materialized_summary(self):
        captured = {}

        class FakeResponse:
            def __init__(self, body):
                self._body = body

            def read(self):
                return self._body

            def getcode(self):
                return 200

            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

        def fake_urlopen(request, timeout):
            captured["url"] = request.full_url
            captured["method"] = request.get_method()
            captured["timeout"] = timeout
            captured["authorization"] = request.get_header("Authorization")
            captured["forwarded"] = request.get_header("X-shuma-forwarded-secret")
            captured["payload"] = json.loads(request.data.decode("utf-8"))
            return FakeResponse(
                json.dumps(
                    {
                        "updated": True,
                        "summary": {"availability": "materialized"},
                    }
                ).encode("utf-8")
            )

        with mock.patch.dict(
            promote.os.environ,
            {
                "SHUMA_BASE_URL": "http://127.0.0.1:3000",
                "SHUMA_API_KEY": "test-api-key",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
            },
            clear=True,
        ):
            with mock.patch(
                "scripts.tests.adversarial_promote_candidates.urllib.request.urlopen",
                side_effect=fake_urlopen,
            ):
                response = promote.materialize_backend_replay_promotion(
                    sample_promotion_payload()
                )

        self.assertEqual(captured["url"], "http://127.0.0.1:3000/admin/replay-promotion")
        self.assertEqual(captured["method"], "POST")
        self.assertEqual(captured["timeout"], 20.0)
        self.assertEqual(captured["authorization"], "Bearer test-api-key")
        self.assertEqual(captured["forwarded"], "forwarded-secret")
        self.assertEqual(captured["payload"]["schema_version"], "adversarial-promotion.v1")
        self.assertEqual(response["summary"]["availability"], "materialized")

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
        self.assertEqual(len(findings), 2)
        finding = findings[0]
        self.assertEqual(finding["frontier_mode"], "multi_provider_playoff")
        self.assertEqual(finding["diversity_confidence"], "higher")
        self.assertEqual(finding["scenario_family"], "cdp_high_confidence_deny")
        self.assertEqual(finding["observed_outcome"], "deny_temp")
        self.assertTrue(str(finding.get("candidate_id") or "").startswith("cand-"))
        self.assertIn(str(finding.get("generation_kind") or ""), {"seed", "mutation"})

    def test_normalize_findings_carries_generated_candidate_lineage_fields(self):
        findings = promote.normalize_findings(
            attack_plan=sample_attack_plan(),
            report=sample_report(),
        )
        mutation = [row for row in findings if str(row.get("generation_kind")) == "mutation"][0]
        self.assertEqual(mutation["candidate_id"], "cand-mut-sim_t4_cdp_detection_deny-retry")
        self.assertEqual(mutation["source_scenario_id"], "sim_t4_cdp_detection_deny")
        self.assertEqual(mutation["mutation_class"], "retry_strategy")
        self.assertEqual(mutation["behavioral_class"], "timing_variation")
        self.assertGreater(float(mutation["novelty_score"]), 0.0)

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

    def test_discovery_quality_metrics_track_novel_confirmed_and_provider_outage(self):
        findings = promote.normalize_findings(sample_attack_plan(), sample_report())
        lineage = [
            {
                "classification": "confirmed_reproducible",
                "generated_candidate": {"generation_kind": "mutation"},
            },
            {
                "classification": "not_reproducible",
                "generated_candidate": {"generation_kind": "seed"},
            },
        ]
        hybrid = {"observed": {"false_discovery_rate_percent": 50.0}}
        frontier_status = {
            "status": "degraded_partial_provider_failure",
            "provider_count_configured": 2,
            "provider_count_healthy": 1,
        }
        metrics = promote.evaluate_discovery_quality_metrics(
            findings=findings,
            lineage=lineage,
            attack_plan=sample_attack_plan(),
            hybrid_governance=hybrid,
            frontier_status=frontier_status,
        )
        self.assertEqual(metrics["candidate_count"], 2)
        self.assertEqual(metrics["generated_candidate_count"], 1)
        self.assertEqual(metrics["novel_confirmed_regressions"], 1)
        self.assertEqual(metrics["provider_outage_impact_percent"], 50.0)
        self.assertEqual(metrics["false_discovery_rate_percent"], 50.0)


if __name__ == "__main__":
    unittest.main()
