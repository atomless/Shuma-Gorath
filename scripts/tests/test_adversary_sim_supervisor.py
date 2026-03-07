import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SUPERVISOR_MANAGER_SCRIPT = REPO_ROOT / "scripts" / "run_with_adversary_sim_supervisor.sh"
SUPERVISOR_WORKER_SOURCE = REPO_ROOT / "scripts" / "supervisor" / "adversary_sim_supervisor.rs"


class AdversarySimSupervisorContractTests(unittest.TestCase):
    def test_supervisor_manager_polls_status_with_internal_supervisor_headers(self) -> None:
        script = SUPERVISOR_MANAGER_SCRIPT.read_text(encoding="utf-8")
        self.assertIn('-H "X-Forwarded-Proto: https"', script)
        self.assertIn('-H "X-Shuma-Internal-Supervisor: adversary-sim"', script)

    def test_supervisor_worker_posts_beats_with_internal_supervisor_headers(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn("X-Forwarded-Proto: https", source)
        self.assertIn("X-Shuma-Internal-Supervisor: adversary-sim", source)


if __name__ == "__main__":
    unittest.main()
