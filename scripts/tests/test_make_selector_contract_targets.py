import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


def target_body(name: str) -> str:
    source = MAKEFILE.read_text(encoding="utf-8")
    match = re.search(
        rf"^{re.escape(name)}:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
        source,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        raise AssertionError(f"target {name} not found")
    return match.group(0)


class MakeSelectorContractTargetTests(unittest.TestCase):
    def test_adversary_sim_feature_targets_do_not_hide_make_selector_tests(self) -> None:
        lifecycle_body = target_body("test-adversary-sim-lifecycle")
        self.assertNotIn("scripts/tests/test_adversary_sim_make_targets.py", lifecycle_body)

        category_fit_body = target_body("test-adversary-sim-scrapling-category-fit")
        self.assertNotIn("scripts/tests/test_adversary_sim_make_targets.py", category_fit_body)

        malicious_body = target_body("test-adversary-sim-scrapling-malicious-request-native")
        self.assertNotIn("scripts/tests/test_adversary_sim_make_targets.py", malicious_body)

        coverage_body = target_body("test-adversary-sim-scrapling-coverage-receipts")
        self.assertNotIn("scripts/tests/test_adversary_sim_make_targets.py", coverage_body)

        llm_fit_body = target_body("test-adversarial-llm-fit")
        self.assertNotIn("test_llm_fit_target_uses_bounded_lane_contract_selectors", llm_fit_body)

        contract_body = target_body("test-adversary-sim-make-target-contract")
        self.assertIn("python3 -m unittest scripts/tests/test_adversary_sim_make_targets.py", contract_body)

    def test_verified_identity_feature_targets_do_not_hide_make_selector_tests(self) -> None:
        for name in [
            "test-verified-identity-calibration-readiness",
            "test-verified-identity-alignment-receipts",
            "test-verified-identity-botness-conflicts",
            "test-verified-identity-guardrails",
        ]:
            body = target_body(name)
            self.assertNotIn("scripts/tests/test_verified_identity_make_targets.py", body)

        contract_body = target_body("test-verified-identity-make-target-contract")
        self.assertIn(
            "python3 -m unittest scripts/tests/test_verified_identity_make_targets.py",
            contract_body,
        )

    def test_host_impact_feature_targets_do_not_hide_make_selector_tests(self) -> None:
        for name in [
            "test-host-impact-telemetry",
            "test-host-impact-benchmark",
            "test-oversight-host-impact",
        ]:
            body = target_body(name)
            self.assertNotIn("scripts/tests/test_host_impact_make_targets.py", body)

        contract_body = target_body("test-host-impact-make-target-contract")
        self.assertIn("python3 -m unittest scripts/tests/test_host_impact_make_targets.py", contract_body)


if __name__ == "__main__":
    unittest.main()
