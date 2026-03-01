#!/usr/bin/env python3

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import scripts.tests.check_sim2_adr_conformance as conformance


def sample_report():
    return {
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
            "overflow_bucket_count": 3,
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


def sample_realtime_bench():
    return {
        "workload": {
            "events_per_sec": 1000,
            "operator_clients": 5,
        },
        "verification_scope": {
            "claims_runtime_prod_verification": False,
        },
    }


class Sim2AdrConformanceUnitTests(unittest.TestCase):
    def test_check_markers_returns_missing_entries(self):
        text = "alpha beta gamma"
        missing = conformance.check_markers(text, ["alpha", "delta"])
        self.assertEqual(missing, ["delta"])

    def test_evaluate_marker_requirements_reports_missing_markers(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target = root / "docs/adr/test.md"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("present-marker", encoding="utf-8")
            with patch.object(
                conformance,
                "ADR_REQUIREMENTS",
                [
                    {
                        "id": "adr_test",
                        "path": "docs/adr/test.md",
                        "markers": ["present-marker", "missing"],
                    }
                ],
            ), patch.object(conformance, "IMPLEMENTATION_REQUIREMENTS", []):
                payload = conformance.evaluate_marker_requirements(root)
        self.assertEqual(len(payload["checks"]), 1)
        self.assertEqual(len(payload["failures"]), 1)
        self.assertIn("missing markers", payload["checks"][0]["detail"])

    def test_evaluate_passes_when_evidence_is_valid(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            with patch.object(conformance, "ADR_REQUIREMENTS", []), patch.object(
                conformance, "IMPLEMENTATION_REQUIREMENTS", []
            ):
                payload = conformance.evaluate(
                    sample_report(), sample_realtime_bench(), root
                )
        self.assertTrue(payload["status"]["passed"])
        self.assertEqual(payload["status"]["failures"], [])

    def test_evaluate_fails_when_runtime_prod_scope_is_claimed(self):
        report = sample_report()
        bench = sample_realtime_bench()
        bench["verification_scope"]["claims_runtime_prod_verification"] = True
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            with patch.object(conformance, "ADR_REQUIREMENTS", []), patch.object(
                conformance, "IMPLEMENTATION_REQUIREMENTS", []
            ):
                payload = conformance.evaluate(report, bench, root)
        self.assertFalse(payload["status"]["passed"])
        joined = " ".join(payload["status"]["failures"])
        self.assertIn("adr_0008_runtime_prod_claim_invalid", joined)


if __name__ == "__main__":
    unittest.main()
