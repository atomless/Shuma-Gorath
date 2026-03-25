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


class TestingSurfaceArtifactPathTests(unittest.TestCase):
    def test_makefile_routes_routine_generated_artifacts_under_spin_state(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        self.assertIn("ADVERSARIAL_ARTIFACT_DIR ?= .spin/adversarial", source)
        self.assertIn("ADVERSARIAL_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/latest_report.json", source)
        self.assertIn(
            "SIM2_CI_DIAGNOSTICS_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/sim2_ci_diagnostics.json",
            source,
        )

    def test_preflight_target_writes_generated_report_to_spin_artifact_dir(self) -> None:
        body = target_body("test-adversarial-preflight")
        self.assertIn("--output $(ADVERSARIAL_PREFLIGHT_REPORT_PATH)", body)

    def test_routine_adversarial_and_sim2_targets_avoid_tracked_generated_paths(self) -> None:
        smoke_body = target_body("test-adversarial-smoke")
        self.assertIn('--report "$(ADVERSARIAL_REPORT_PATH)"', smoke_body)
        self.assertNotIn("scripts/tests/adversarial/latest_report.json", smoke_body)

        live_body = target_body("test-adversarial-live")
        self.assertIn('REPORT_PATH="$${ADVERSARIAL_REPORT_PATH:-$(ADVERSARIAL_REPORT_PATH)}"', live_body)

        repeatability_body = target_body("test-adversarial-repeatability")
        self.assertIn("--report $(ADVERSARIAL_REPEATABILITY_REPORT_PATH)", repeatability_body)
        self.assertNotIn("scripts/tests/adversarial/repeatability_report.json", repeatability_body)

        promote_body = target_body("test-adversarial-promote-candidates")
        self.assertIn('REPORT_PATH="$(ADVERSARIAL_REPORT_PATH)"', promote_body)
        self.assertIn('ATTACK_PLAN_PATH="$(ADVERSARIAL_ATTACK_PLAN_PATH)"', promote_body)
        self.assertIn("--output $(ADVERSARIAL_PROMOTION_CANDIDATES_REPORT_PATH)", promote_body)
        self.assertNotIn("scripts/tests/adversarial/promotion_candidates_report.json", promote_body)

        governance_body = target_body("test-frontier-governance")
        self.assertIn("--report $(ADVERSARIAL_REPORT_PATH)", governance_body)
        self.assertIn("--attack-plan $(ADVERSARIAL_ATTACK_PLAN_PATH)", governance_body)

        diagnostics_body = target_body("test-sim2-ci-diagnostics")
        self.assertIn("--report $(ADVERSARIAL_REPORT_PATH)", diagnostics_body)
        self.assertIn("--output $(SIM2_CI_DIAGNOSTICS_REPORT_PATH)", diagnostics_body)

        regressions_body = target_body("test-sim2-operational-regressions")
        self.assertIn("--report $(ADVERSARIAL_REPORT_PATH)", regressions_body)
        self.assertIn("--output $(SIM2_OPERATIONAL_REGRESSIONS_REPORT_PATH)", regressions_body)

        verification_e2e_body = target_body("test-sim2-verification-e2e")
        self.assertIn("--report $(ADVERSARIAL_REPORT_PATH)", verification_e2e_body)
        self.assertIn("--container-report $(ADVERSARIAL_CONTAINER_BLACKBOX_REPORT_PATH)", verification_e2e_body)
        self.assertIn("--output $(SIM2_VERIFICATION_MATRIX_REPORT_PATH)", verification_e2e_body)
        self.assertNotIn("scripts/tests/adversarial/sim2_verification_matrix_report.json", verification_e2e_body)

        diff_body = target_body("test-adversarial-report-diff")
        self.assertIn('BASELINE="$${ADVERSARIAL_DIFF_BASELINE_PATH:-$(ADVERSARIAL_DIFF_BASELINE_PATH)}"', diff_body)
        self.assertIn('CANDIDATE="$${ADVERSARIAL_DIFF_CANDIDATE_PATH:-$(ADVERSARIAL_DIFF_CANDIDATE_PATH)}"', diff_body)
        self.assertIn('OUTPUT="$${ADVERSARIAL_DIFF_OUTPUT_PATH:-$(ADVERSARIAL_DIFF_OUTPUT_PATH)}"', diff_body)
        self.assertNotIn("scripts/tests/adversarial/adversarial_report_diff.json", diff_body)

        isolation_body = target_body("test-adversarial-container-isolation")
        self.assertIn("--report $(ADVERSARIAL_CONTAINER_ISOLATION_REPORT_PATH)", isolation_body)
        self.assertNotIn("scripts/tests/adversarial/container_isolation_report.json", isolation_body)

        blackbox_body = target_body("test-adversarial-container-blackbox")
        self.assertIn("--report $(ADVERSARIAL_CONTAINER_BLACKBOX_REPORT_PATH)", blackbox_body)
        self.assertNotIn("scripts/tests/adversarial/container_blackbox_report.json", blackbox_body)


if __name__ == "__main__":
    unittest.main()
