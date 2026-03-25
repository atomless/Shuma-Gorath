import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


class VerifiedIdentityMakeTargetTests(unittest.TestCase):
    def test_explicit_verified_identity_target_contract_target_owns_selector_microtests(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-verified-identity-target-contracts:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("scripts/tests/test_verified_identity_make_targets.py", body)

    def test_calibration_readiness_target_uses_current_seam_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-verified-identity-calibration-readiness:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("runtime::traffic_classification::tests::verified_identity_", body)
        self.assertIn(
            "observability::operator_snapshot_verified_identity::tests::",
            body,
        )
        self.assertIn(
            "benchmark_results_materialize_supported_adversary_and_beneficial_non_human_families",
            body,
        )
        self.assertIn(
            "manual_reconcile_route_records_observe_longer_when_classification_is_not_ready",
            body,
        )
        self.assertNotIn("scripts/tests/test_verified_identity_make_targets.py", body)

    def test_alignment_receipts_target_uses_alignment_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-verified-identity-alignment-receipts:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "observability::non_human_classification::tests::verified_identity_alignment_",
            body,
        )
        self.assertIn(
            "observability::operator_snapshot_verified_identity::tests::verified_identity_summary_projects_taxonomy_alignment_",
            body,
        )
        self.assertNotIn("scripts/tests/test_verified_identity_make_targets.py", body)

    def test_botness_conflicts_target_uses_conflict_metric_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-verified-identity-botness-conflicts:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "observability::benchmark_beneficial_non_human::tests::",
            body,
        )
        self.assertNotIn("scripts/tests/test_verified_identity_make_targets.py", body)

    def test_guardrails_target_uses_reconcile_and_benchmark_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-verified-identity-guardrails:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "verified_identity_guardrails_block_tuning_when_conflicts_are_outside_budget",
            body,
        )
        self.assertIn(
            "observe_longer_when_verified_identity_guardrail_blocks_candidate",
            body,
        )
        self.assertNotIn("scripts/tests/test_verified_identity_make_targets.py", body)


if __name__ == "__main__":
    unittest.main()
