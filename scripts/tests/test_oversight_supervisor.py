import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SUPERVISOR_SCRIPT = REPO_ROOT / "scripts" / "run_with_oversight_supervisor.sh"


class OversightSupervisorContractTests(unittest.TestCase):
    def test_wrapper_posts_periodic_agent_runs_to_internal_endpoint(self) -> None:
        script = SUPERVISOR_SCRIPT.read_text(encoding="utf-8")
        self.assertIn("/shuma/internal/oversight/agent/run", script)
        self.assertIn('"trigger_kind":"periodic_supervisor"', script)

    def test_wrapper_uses_internal_oversight_supervisor_headers(self) -> None:
        script = SUPERVISOR_SCRIPT.read_text(encoding="utf-8")
        self.assertIn('-H "X-Forwarded-Proto: https"', script)
        self.assertIn('-H "X-Shuma-Internal-Supervisor: oversight-agent"', script)

    def test_wrapper_chains_existing_adversary_sim_supervisor_wrapper(self) -> None:
        script = SUPERVISOR_SCRIPT.read_text(encoding="utf-8")
        self.assertIn("run_with_adversary_sim_supervisor.sh", script)


if __name__ == "__main__":
    unittest.main()
