import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from typing import Optional


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "live_feedback_loop_remote.py"
SPEC = importlib.util.spec_from_file_location("live_feedback_loop_remote", SCRIPT)
LIVE_FEEDBACK_LOOP_REMOTE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(LIVE_FEEDBACK_LOOP_REMOTE)


class _FakeLiveFeedbackLoopRemote(LIVE_FEEDBACK_LOOP_REMOTE.LiveFeedbackLoopRemote):
    def __init__(self, *, temp_dir: Path, service_exec: str, service_status: Optional[str] = None) -> None:
        self.report_path = temp_dir / "report.json"
        self.transport_mode = "ssh_loopback"
        self.base_url = "https://shuma.example.com"
        self.api_key = "test-admin-key"
        self.forwarded_ip_secret = "test-forwarded-secret"
        self.local_env = {"SHUMA_ADMIN_IP_ALLOWLIST": "127.0.0.1/32"}
        self.remote_env = None
        self.receipt = {
            "identity": {
                "name": "stub-remote",
                "provider_kind": "linode",
            },
            "metadata": {
                "last_deployed_commit": "feedface",
                "last_deployed_at_utc": "2026-03-22T12:00:00Z",
            },
            "runtime": {
                "service_name": "shuma-gorath",
            },
        }
        self._service_exec = service_exec
        self._service_status = service_status or service_exec
        self._oversight_status_queue = [
            {
                "schema_version": "oversight_agent_status_v1",
                "execution_boundary": "shared_host_only",
                "periodic_trigger": {
                    "surface": "host_supervisor_wrapper",
                    "wrapper_command": "scripts/run_with_oversight_supervisor.sh",
                    "default_interval_seconds": 300,
                },
                "post_sim_trigger": {
                    "surface": "internal_adversary_sim_completion_hook",
                    "qualifying_completion": "transition_to_off_with_completed_run_id_and_generated_traffic",
                    "dedupe_key": "sim_run_id",
                },
                "latest_run": None,
                "latest_decision": None,
                "recent_runs": [],
            },
            {
                "schema_version": "oversight_agent_status_v1",
                "execution_boundary": "shared_host_only",
                "periodic_trigger": {
                    "surface": "host_supervisor_wrapper",
                    "wrapper_command": "scripts/run_with_oversight_supervisor.sh",
                    "default_interval_seconds": 300,
                },
                "post_sim_trigger": {
                    "surface": "internal_adversary_sim_completion_hook",
                    "qualifying_completion": "transition_to_off_with_completed_run_id_and_generated_traffic",
                    "dedupe_key": "sim_run_id",
                },
                "latest_run": {
                    "run_id": "ovragent-periodic-1",
                    "trigger_kind": "periodic_supervisor",
                    "execution": {
                        "decision": {"decision_id": "decision-periodic-1"},
                        "apply": {"stage": "refused"},
                        "reconcile": {
                            "outcome": "recommend_patch",
                            "latest_sim_run_id": None,
                        },
                    },
                },
                "latest_decision": {"decision_id": "decision-periodic-1"},
                "recent_runs": [
                    {
                        "run_id": "ovragent-periodic-1",
                        "trigger_kind": "periodic_supervisor",
                        "execution": {
                            "decision": {"decision_id": "decision-periodic-1"},
                            "apply": {"stage": "refused"},
                            "reconcile": {
                                "outcome": "recommend_patch",
                                "latest_sim_run_id": None,
                            },
                        },
                    }
                ],
            },
            {
                "schema_version": "oversight_agent_status_v1",
                "execution_boundary": "shared_host_only",
                "periodic_trigger": {
                    "surface": "host_supervisor_wrapper",
                    "wrapper_command": "scripts/run_with_oversight_supervisor.sh",
                    "default_interval_seconds": 300,
                },
                "post_sim_trigger": {
                    "surface": "internal_adversary_sim_completion_hook",
                    "qualifying_completion": "transition_to_off_with_completed_run_id_and_generated_traffic",
                    "dedupe_key": "sim_run_id",
                },
                "latest_run": {
                    "run_id": "ovragent-post-sim-1",
                    "trigger_kind": "post_adversary_sim",
                    "sim_run_id": "sim-run-1",
                    "execution": {
                        "decision": {"decision_id": "decision-post-sim-1"},
                        "apply": {"stage": "watch_window_open"},
                        "reconcile": {
                            "outcome": "recommend_patch",
                            "latest_sim_run_id": "sim-run-1",
                        },
                    },
                },
                "latest_decision": {"decision_id": "decision-post-sim-1"},
                "recent_runs": [
                    {
                        "run_id": "ovragent-post-sim-1",
                        "trigger_kind": "post_adversary_sim",
                        "sim_run_id": "sim-run-1",
                        "execution": {
                            "decision": {"decision_id": "decision-post-sim-1"},
                            "apply": {"stage": "watch_window_open"},
                            "reconcile": {
                                "outcome": "recommend_patch",
                                "latest_sim_run_id": "sim-run-1",
                            },
                        },
                    },
                    {
                        "run_id": "ovragent-periodic-1",
                        "trigger_kind": "periodic_supervisor",
                        "execution": {
                            "decision": {"decision_id": "decision-periodic-1"},
                            "apply": {"stage": "refused"},
                            "reconcile": {
                                "outcome": "recommend_patch",
                                "latest_sim_run_id": None,
                            },
                        },
                    },
                ],
            },
        ]
        self._history_queue = [
            {
                "schema_version": "oversight_history_v1",
                "rows": [
                    {
                        "decision_id": "decision-post-sim-1",
                        "trigger_source": "post_adversary_sim",
                        "apply": {"stage": "watch_window_open"},
                    },
                    {
                        "decision_id": "decision-periodic-1",
                        "trigger_source": "periodic_supervisor",
                        "apply": {"stage": "refused"},
                    },
                ],
            }
        ]
        self._adversary_status_queue = [
            {
                "adversary_sim_enabled": False,
                "generation_active": False,
                "phase": "off",
                "last_run_id": None,
                "generation": {"tick_count": 0, "request_count": 0, "truth_basis": "control_state"},
                "lane_diagnostics": {
                    "truth_basis": "control_state",
                    "lanes": {
                        "synthetic_traffic": {"generated_requests": 0, "beat_successes": 0}
                    },
                },
                "persisted_event_evidence": None,
            },
            {
                "adversary_sim_enabled": True,
                "generation_active": True,
                "phase": "running",
                "run_id": "sim-run-1",
                "last_run_id": None,
                "generation": {
                    "tick_count": 1,
                    "request_count": 3,
                    "truth_basis": "control_state",
                },
                "lane_diagnostics": {
                    "truth_basis": "control_state",
                    "lanes": {
                        "synthetic_traffic": {"generated_requests": 3, "beat_successes": 1}
                    },
                },
                "persisted_event_evidence": {
                    "run_id": "sim-run-1",
                    "monitoring_event_count": 3,
                },
            },
            {
                "adversary_sim_enabled": False,
                "generation_active": False,
                "phase": "off",
                "run_id": None,
                "last_run_id": "sim-run-1",
                "generation": {
                    "tick_count": 2,
                    "request_count": 6,
                    "truth_basis": "persisted_event_lower_bound",
                },
                "lane_diagnostics": {
                    "truth_basis": "persisted_event_lower_bound",
                    "lanes": {
                        "synthetic_traffic": {"generated_requests": 6, "beat_successes": 1}
                    },
                },
                "persisted_event_evidence": {
                    "run_id": "sim-run-1",
                    "monitoring_event_count": 1,
                },
            },
            {
                "adversary_sim_enabled": False,
                "generation_active": False,
                "phase": "off",
                "run_id": None,
                "last_run_id": "sim-run-1",
                "generation": {
                    "tick_count": 2,
                    "request_count": 6,
                    "truth_basis": "persisted_event_lower_bound",
                },
                "lane_diagnostics": {
                    "truth_basis": "persisted_event_lower_bound",
                    "lanes": {
                        "synthetic_traffic": {"generated_requests": 6, "beat_successes": 1}
                    },
                },
                "persisted_event_evidence": {
                    "run_id": "sim-run-1",
                    "monitoring_event_count": 1,
                },
            },
        ]
        self._control_calls = []
        self._control_headers = []
        self._disabled_calls = 0

    def _write_report(self, report):
        self.report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    def _run_ssh_command(self, command: str) -> str:
        if "systemctl show" in command:
            return self._service_exec
        if "systemctl status" in command:
            return self._service_status
        raise AssertionError(f"Unexpected SSH command: {command}")

    def _request_json(self, method: str, path: str, payload=None):
        if path == "/admin/oversight/agent/status":
            if len(self._oversight_status_queue) > 1:
                return self._oversight_status_queue.pop(0)
            return self._oversight_status_queue[0]
        if path == "/admin/operator-snapshot":
            return {
                "schema_version": "operator_snapshot_v1",
                "benchmark_results": {"overall_status": "healthy"},
            }
        if path == "/admin/events?hours=2&limit=200":
            return {
                "recent_events": [
                    {
                        "event": "Challenge",
                        "is_simulation": True,
                        "sim_run_id": "sim-run-1",
                    }
                ]
            }
        if path == "/admin/oversight/history":
            if len(self._history_queue) > 1:
                return self._history_queue.pop(0)
            return self._history_queue[0]
        if path == "/admin/adversary-sim/status":
            if len(self._adversary_status_queue) > 1:
                return self._adversary_status_queue.pop(0)
            return self._adversary_status_queue[0]
        if path == "/admin/adversary-sim/control":
            self._control_calls.append(payload)
            return {
                "operation_id": "op-enable-1" if payload["enabled"] else "op-disable-1",
                "decision": "accepted",
            }
        raise AssertionError(f"Unexpected request: {method} {path}")

    def _internal_request_json(self, method: str, path: str, payload=None):
        if path == "/internal/oversight/agent/run":
            return {
                "schema_version": "oversight_agent_execution_v1",
                "status": "executed",
                "replayed": False,
                "run": {
                    "run_id": "ovragent-periodic-1",
                    "trigger_kind": "periodic_supervisor",
                    "execution": {
                        "decision": {"decision_id": "decision-periodic-1"},
                        "apply": {"stage": "refused"},
                        "reconcile": {
                            "outcome": "recommend_patch",
                            "latest_sim_run_id": None,
                        },
                    },
                },
            }
        raise AssertionError(f"Unexpected internal request: {method} {path}")

    def _loopback_request_json(self, method: str, path: str, headers, payload=None):
        if path == "/admin/adversary-sim/control":
            self._control_calls.append(payload)
            self._control_headers.append(headers)
            return {
                "operation_id": "op-enable-1" if payload["enabled"] else "op-disable-1",
                "decision": "accepted",
            }
        if path == "/internal/oversight/agent/run":
            return self._internal_request_json(method, path, payload)
        raise AssertionError(f"Unexpected loopback request: {method} {path}")

    def _ensure_adversary_sim_disabled(self):
        self._disabled_calls += 1


class LiveFeedbackLoopRemoteTests(unittest.TestCase):
    def test_run_writes_success_report_for_periodic_and_post_sim_flow(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="live-feedback-loop-remote-"))
        runner = _FakeLiveFeedbackLoopRemote(
            temp_dir=temp_dir,
            service_exec="/bin/bash /opt/shuma-gorath/scripts/run_with_oversight_supervisor.sh spin up",
        )

        exit_code = runner.run()

        self.assertEqual(exit_code, 0)
        report = json.loads(runner.report_path.read_text(encoding="utf-8"))
        self.assertEqual(report["result"], "pass")
        self.assertTrue(report["adversary_sim"]["running_observed"])
        self.assertEqual(report["periodic_trigger"]["run_id"], "ovragent-periodic-1")
        self.assertEqual(report["periodic_trigger"]["apply_stage"], "refused")
        self.assertEqual(report["post_sim_trigger"]["sim_run_id"], "sim-run-1")
        self.assertEqual(report["post_sim_trigger"]["decision_id"], "decision-post-sim-1")
        self.assertEqual(report["post_sim_trigger"]["apply_stage"], "watch_window_open")
        self.assertEqual(report["post_sim_trigger"]["history_latest_apply_stage"], "watch_window_open")
        self.assertEqual(report["adversary_sim"]["persisted_event_count"], 1)
        self.assertEqual(
            report["adversary_sim"]["completed"]["generation_truth_basis"],
            "persisted_event_lower_bound",
        )
        self.assertEqual(
            report["adversary_sim"]["completed"]["lane_diagnostics_truth_basis"],
            "persisted_event_lower_bound",
        )
        self.assertEqual(runner._control_calls, [{"enabled": True}])
        self.assertEqual(len(runner._control_headers), 1)
        self.assertEqual(runner._control_headers[0]["Host"], "shuma.example.com")
        self.assertEqual(runner._control_headers[0]["Origin"], "https://shuma.example.com")
        self.assertEqual(runner._control_headers[0]["Referer"], "https://shuma.example.com/dashboard")
        self.assertGreaterEqual(runner._disabled_calls, 1)

    def test_run_fails_when_completed_status_still_under_reports_generation_truth(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="live-feedback-loop-remote-"))
        runner = _FakeLiveFeedbackLoopRemote(
            temp_dir=temp_dir,
            service_exec="/bin/bash /opt/shuma-gorath/scripts/run_with_oversight_supervisor.sh spin up",
        )
        runner._adversary_status_queue[2] = {
            "adversary_sim_enabled": False,
            "generation_active": False,
            "phase": "off",
            "run_id": None,
            "last_run_id": "sim-run-1",
            "generation": {"tick_count": 0, "request_count": 0, "truth_basis": "control_state"},
            "lane_diagnostics": {
                "truth_basis": "control_state",
                "lanes": {
                    "synthetic_traffic": {"generated_requests": 0, "beat_successes": 0}
                },
            },
            "persisted_event_evidence": None,
        }

        exit_code = runner.run()

        self.assertEqual(exit_code, 1)
        report = json.loads(runner.report_path.read_text(encoding="utf-8"))
        self.assertEqual(report["result"], "fail")
        self.assertIn("under-reported completion counters", report["error"])

    def test_run_fails_when_remote_service_does_not_use_oversight_wrapper(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="live-feedback-loop-remote-"))
        runner = _FakeLiveFeedbackLoopRemote(
            temp_dir=temp_dir,
            service_exec="/bin/bash /opt/shuma-gorath/scripts/run_with_adversary_sim_supervisor.sh spin up",
        )

        exit_code = runner.run()

        self.assertEqual(exit_code, 1)
        report = json.loads(runner.report_path.read_text(encoding="utf-8"))
        self.assertEqual(report["result"], "fail")
        self.assertIn("run_with_oversight_supervisor.sh", report["error"])

    def test_run_accepts_service_tree_with_wrapper_below_make_prod_start(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="live-feedback-loop-remote-"))
        runner = _FakeLiveFeedbackLoopRemote(
            temp_dir=temp_dir,
            service_exec="ExecStart={ path=/usr/bin/make ; argv[]=/usr/bin/make prod-start ; }",
            service_status=(
                "CGroup: /system.slice/shuma-gorath.service\n"
                "  /usr/bin/make prod-start\n"
                "  /bin/sh -c ./scripts/run_with_oversight_supervisor.sh spin up\n"
            ),
        )

        exit_code = runner.run()

        self.assertEqual(exit_code, 0)
        report = json.loads(runner.report_path.read_text(encoding="utf-8"))
        self.assertEqual(report["result"], "pass")


if __name__ == "__main__":
    unittest.main()
