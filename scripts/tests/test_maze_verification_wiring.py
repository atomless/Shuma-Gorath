import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"
CI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
RELEASE_GATE_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release-gate.yml"


def target_body(source: str, target: str) -> str:
    match = re.search(
        rf"^{re.escape(target)}:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
        source,
        re.MULTILINE | re.DOTALL,
    )
    if match is None:
        raise AssertionError(f"Makefile target '{target}' is missing")
    return match.group(0)


class MazeVerificationWiringTests(unittest.TestCase):
    def test_canonical_maze_verification_gate_groups_all_required_proofs(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        body = target_body(source, "test-maze-verification-gate")
        self.assertIn("test-maze-benchmark", body)
        self.assertIn("test-maze-live-traversal-contract", body)
        self.assertIn("test-maze-live-browser-contract", body)
        self.assertIn("test-maze-state-concurrency-contract", body)

    def test_full_suite_routes_maze_step_through_canonical_gate(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        body = target_body(source, "test")
        self.assertIn("test-maze-verification-gate", body)
        self.assertNotIn("Step 2/8: Maze Asymmetry Benchmark Gate", body)

    def test_pre_merge_ci_still_runs_canonical_full_suite(self) -> None:
        source = CI_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("run: make test", source)

    def test_release_gate_reuses_canonical_maze_verification_gate(self) -> None:
        source = RELEASE_GATE_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("make test-maze-verification-gate", source)


if __name__ == "__main__":
    unittest.main()
