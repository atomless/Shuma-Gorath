import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


class VerifiedIdentityMakeTargetTests(unittest.TestCase):
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
        self.assertIn("scripts/tests/test_verified_identity_make_targets.py", body)


if __name__ == "__main__":
    unittest.main()
