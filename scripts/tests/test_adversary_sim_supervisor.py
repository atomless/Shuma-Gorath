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

    def test_supervisor_worker_posts_external_worker_results_to_internal_endpoint(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn("/internal/adversary-sim/worker-result", source)

    def test_supervisor_worker_knows_about_scrapling_dispatch_mode(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn("scrapling_worker", source)
        self.assertIn("scrapling_worker.py", source)

    def test_supervisor_failure_result_keeps_worker_plan_fulfillment_mode(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn('json_string(beat_body, "fulfillment_mode")', source)
        self.assertIn('\\"fulfillment_mode\\":\\"{}\\"', source)

    def test_supervisor_treats_empty_scrapling_env_paths_as_unset(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn("fn non_empty_env_path", source)
        self.assertIn('.filter(|value| !value.is_empty())', source)
        self.assertIn('non_empty_env_path("ADVERSARY_SIM_SCRAPLING_PYTHON")', source)
        self.assertIn('non_empty_env_path("ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH")', source)
        self.assertIn('non_empty_env_path("ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH")', source)
        self.assertIn('non_empty_env_path("ADVERSARY_SIM_SCRAPLING_CRAWLDIR")', source)

    def test_supervisor_surfaces_bounded_worker_stdio_on_nonzero_exit(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn("const WORKER_OUTPUT_SNIPPET_BYTES", source)
        self.assertIn("fn summarize_worker_output", source)
        self.assertIn("fn enrich_worker_failure", source)
        self.assertIn(".stdout(Stdio::piped())", source)
        self.assertIn(".stderr(Stdio::piped())", source)
        self.assertIn("worker exited with status {:?}", source)
        self.assertIn("stderr:", source)

    def test_supervisor_decodes_chunked_internal_http_bodies_before_writing_worker_input(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn("fn header_value", source)
        self.assertIn("fn decode_chunked_body", source)
        self.assertIn('header_value(head, "Transfer-Encoding")', source)
        self.assertIn("value.eq_ignore_ascii_case(\"chunked\")", source)
        self.assertIn("chunked body missing terminating zero-length chunk", source)

    def test_supervisor_manager_worker_pid_is_not_trap_scoped_local(self) -> None:
        script = SUPERVISOR_MANAGER_SCRIPT.read_text(encoding="utf-8")
        self.assertNotIn('local worker_pid=""', script)


if __name__ == "__main__":
    unittest.main()
