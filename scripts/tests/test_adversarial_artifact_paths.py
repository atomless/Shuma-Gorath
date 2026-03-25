import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


def extract_target_body(target: str, source: str):
    match = re.search(
        rf"^{re.escape(target)}:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
        source,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        return None
    return match.group(0)


class AdversarialArtifactPathTests(unittest.TestCase):
    def test_preflight_and_sim2_outputs_use_untracked_spin_artifact_paths(self):
        tracked_receipts = [
            "scripts/tests/adversarial/preflight_report.json",
            "scripts/tests/adversarial/sim2_ci_diagnostics.json",
            "scripts/tests/adversarial/sim2_operational_regressions_report.json",
            "scripts/tests/adversarial/sim2_realtime_bench_report.json",
            "scripts/tests/adversarial/sim2_realtime_bench_summary.md",
        ]
        source = MAKEFILE.read_text(encoding="utf-8")
        self.assertIn("ADVERSARIAL_RUNTIME_ARTIFACT_DIR ?= .spin/adversarial", source)
        self.assertIn(
            "ADVERSARIAL_PREFLIGHT_REPORT ?= $(ADVERSARIAL_RUNTIME_ARTIFACT_DIR)/preflight_report.json",
            source,
        )
        self.assertIn(
            "SIM2_REALTIME_BENCH_REPORT ?= $(ADVERSARIAL_RUNTIME_ARTIFACT_DIR)/sim2_realtime_bench_report.json",
            source,
        )
        self.assertIn(
            "SIM2_REALTIME_BENCH_SUMMARY ?= $(ADVERSARIAL_RUNTIME_ARTIFACT_DIR)/sim2_realtime_bench_summary.md",
            source,
        )
        self.assertIn(
            "SIM2_CI_DIAGNOSTICS_REPORT ?= $(ADVERSARIAL_RUNTIME_ARTIFACT_DIR)/sim2_ci_diagnostics.json",
            source,
        )
        self.assertIn(
            "SIM2_OPERATIONAL_REGRESSIONS_REPORT ?= $(ADVERSARIAL_RUNTIME_ARTIFACT_DIR)/sim2_operational_regressions_report.json",
            source,
        )
        expected_targets = {
            "test-adversarial-preflight": "$(ADVERSARIAL_PREFLIGHT_REPORT)",
            "test-sim2-realtime-bench": "$(SIM2_REALTIME_BENCH_REPORT)",
            "test-sim2-ci-diagnostics": "$(SIM2_CI_DIAGNOSTICS_REPORT)",
            "test-sim2-operational-regressions": "$(SIM2_OPERATIONAL_REGRESSIONS_REPORT)",
            "test-sim2-operational-regressions-strict": "$(SIM2_OPERATIONAL_REGRESSIONS_REPORT)",
        }
        for target, output_var in expected_targets.items():
            body = extract_target_body(target, source)
            self.assertIsNotNone(body, msg=f"missing target {target}")
            self.assertIn(output_var, body)
            for tracked_receipt in tracked_receipts:
                self.assertNotIn(tracked_receipt, body)


if __name__ == "__main__":
    unittest.main()
