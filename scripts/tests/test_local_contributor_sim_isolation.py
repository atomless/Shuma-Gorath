import importlib.util
import os
import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "verify_local_contributor_sim_isolation.py"
SPEC = importlib.util.spec_from_file_location("verify_local_contributor_sim_isolation", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
os.environ.setdefault("SHUMA_API_KEY", "test-api-key")
os.environ.setdefault("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret")
LOCAL_CONTRIBUTOR_SIM_ISOLATION = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(LOCAL_CONTRIBUTOR_SIM_ISOLATION)


class LocalContributorSimIsolationTests(unittest.TestCase):
    def test_generation_progress_accepts_new_run_even_when_tick_count_resets(self) -> None:
        start_status = {
            "run_id": "simrun-old",
            "generation": {
                "request_count": 7,
                "tick_count": 7,
                "last_generated_at": 100,
            },
        }
        current_status = {
            "adversary_sim_enabled": True,
            "run_id": "simrun-new",
            "generation": {
                "request_count": 1,
                "tick_count": 1,
                "last_generated_at": 101,
            },
            "lifecycle_diagnostics": {
                "supervisor": {
                    "last_successful_beat_at": 101,
                }
            },
        }

        self.assertTrue(
            LOCAL_CONTRIBUTOR_SIM_ISOLATION.has_generation_progress(start_status, current_status)
        )

    def test_generation_progress_rejects_control_only_transition_without_worker_activity(self) -> None:
        start_status = {
            "run_id": "simrun-old",
            "generation": {
                "request_count": 7,
                "tick_count": 7,
                "last_generated_at": 100,
            },
        }
        current_status = {
            "adversary_sim_enabled": True,
            "run_id": "simrun-new",
            "generation": {
                "request_count": 0,
                "tick_count": 0,
                "last_generated_at": 100,
            },
            "lifecycle_diagnostics": {
                "supervisor": {
                    "last_successful_beat_at": 100,
                }
            },
        }

        self.assertFalse(
            LOCAL_CONTRIBUTOR_SIM_ISOLATION.has_generation_progress(start_status, current_status)
        )


if __name__ == "__main__":
    unittest.main()
