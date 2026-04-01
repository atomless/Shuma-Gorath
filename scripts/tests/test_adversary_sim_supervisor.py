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
        self.assertIn("/shuma/internal/adversary-sim/worker-result", source)

    def test_supervisor_worker_knows_about_scrapling_dispatch_mode(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn("scrapling_worker", source)
        self.assertIn("scrapling_worker.py", source)

    def test_supervisor_worker_knows_about_llm_runtime_dispatch_mode(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn("llm_fulfillment_plan", source)
        self.assertIn("llm_runtime_worker.py", source)

    def test_supervisor_worker_knows_about_parallel_mixed_dispatch_mode(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn('\\"worker_plan\\":{', source)
        self.assertIn('\\"llm_fulfillment_plan\\":{', source)
        self.assertIn("worker_dispatches", source)
        self.assertIn("thread::spawn", source)

    def test_supervisor_worker_failure_fallback_keeps_fulfillment_mode_field(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn('json_string(beat_body, "fulfillment_mode")', source)
        self.assertIn('\\"fulfillment_mode\\":\\"{}\\"', source)

    def test_supervisor_worker_failure_result_includes_python_stderr_context(self) -> None:
        source = SUPERVISOR_WORKER_SOURCE.read_text(encoding="utf-8")
        self.assertIn(".stderr(Stdio::piped())", source)
        self.assertIn("worker exited with status {:?}; stderr={}", source)

    def test_supervisor_manager_worker_pid_is_not_trap_scoped_local(self) -> None:
        script = SUPERVISOR_MANAGER_SCRIPT.read_text(encoding="utf-8")
        self.assertNotIn('local worker_pid=""', script)

    def test_supervisor_manager_boots_local_trusted_ingress_proxy_for_worker_realism(self) -> None:
        script = SUPERVISOR_MANAGER_SCRIPT.read_text(encoding="utf-8")
        self.assertIn("trusted_ingress_proxy.py", script)
        self.assertIn("ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL", script)
        self.assertIn("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN", script)

    def test_supervisor_manager_supports_local_contributor_ingress_with_public_to_origin_rewrite(self) -> None:
        script = SUPERVISOR_MANAGER_SCRIPT.read_text(encoding="utf-8")
        self.assertIn("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE", script)
        self.assertIn("SHUMA_LOCAL_CONTRIBUTOR_ORIGIN_BASE_URL", script)
        self.assertIn("--public-base-url", script)
        self.assertIn("--allow-direct-browser-requests", script)
        self.assertIn("--allow-local-trusted-forwarded-passthrough", script)


if __name__ == "__main__":
    unittest.main()
