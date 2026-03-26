#!/usr/bin/env python3

import json
import os
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

import scripts.tests.adversarial_container_runner as container_runner
from scripts.tests.frontier_action_contract import (
    load_frontier_action_contract,
    resolve_frontier_actions,
)


class AdversarialContainerRunnerUnitTests(unittest.TestCase):
    def test_normalize_container_base_url_rewrites_loopback(self):
        rewritten = container_runner.normalize_container_base_url("http://127.0.0.1:3000")
        self.assertEqual(rewritten, "http://host.docker.internal:3000")

    def test_build_container_transport_plan_uses_bridge_gateway_for_non_linux_loopback(self):
        plan = container_runner.build_container_transport_plan(
            "http://127.0.0.1:3000",
            platform_system="Darwin",
        )
        self.assertEqual(plan["container_base_url"], "http://host.docker.internal:3000")
        self.assertEqual(plan["allowed_origin"], "http://host.docker.internal:3000")
        self.assertEqual(
            plan["docker_flags"],
            ["--add-host=host.docker.internal:host-gateway", "--network=bridge"],
        )
        self.assertEqual(plan["network_mode"], "bridge")

    def test_build_container_transport_plan_uses_host_network_for_linux_loopback(self):
        plan = container_runner.build_container_transport_plan(
            "http://127.0.0.1:3000",
            platform_system="Linux",
        )
        self.assertEqual(plan["container_base_url"], "http://127.0.0.1:3000")
        self.assertEqual(plan["allowed_origin"], "http://127.0.0.1:3000")
        self.assertEqual(plan["docker_flags"], ["--network=host"])
        self.assertEqual(plan["network_mode"], "host")

    def test_target_origin_returns_scheme_and_netloc(self):
        origin = container_runner.target_origin("https://example.com:8443/path?q=1")
        self.assertEqual(origin, "https://example.com:8443")

    def test_orchestrator_reset_hook_uses_control_endpoint_for_sim_disable(self):
        requests = []

        class FakeResponse:
            def __init__(self, status: int):
                self.status = status

            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

            def read(self) -> bytes:
                return b"{}"

        def fake_urlopen(request, timeout=0):
            requests.append(
                {
                    "url": request.full_url,
                    "method": request.get_method(),
                    "headers": {key.lower(): value for key, value in request.header_items()},
                    "body": request.data.decode("utf-8") if request.data else "",
                }
            )
            return FakeResponse(200)

        with patch(
            "scripts.tests.adversarial_container_runner.urllib.request.urlopen",
            side_effect=fake_urlopen,
        ):
            result = container_runner.orchestrator_reset_hook(
                "http://127.0.0.1:3000",
                "test-api-key",
                "forwarded-secret",
            )

        self.assertTrue(result["performed"])
        self.assertEqual(len(requests), 2)

        config_request, control_request = requests
        self.assertEqual(config_request["url"], "http://127.0.0.1:3000/admin/config")
        self.assertEqual(config_request["method"], "POST")
        self.assertEqual(json.loads(config_request["body"]), {"shadow_mode": False})
        self.assertNotIn("adversary_sim_enabled", config_request["body"])

        self.assertEqual(
            control_request["url"],
            "http://127.0.0.1:3000/admin/adversary-sim/control",
        )
        self.assertEqual(control_request["method"], "POST")
        self.assertEqual(
            json.loads(control_request["body"]),
            {"enabled": False, "reason": "container_blackbox_reset"},
        )
        self.assertEqual(control_request["headers"]["authorization"], "Bearer test-api-key")
        self.assertEqual(
            control_request["headers"]["x-shuma-forwarded-secret"],
            "forwarded-secret",
        )
        self.assertEqual(control_request["headers"]["origin"], "http://127.0.0.1:3000")
        self.assertEqual(control_request["headers"]["sec-fetch-site"], "same-origin")
        self.assertTrue(control_request["headers"]["idempotency-key"])

    def test_build_sim_tag_envelopes_uses_unique_nonces(self):
        envelopes = container_runner.build_sim_tag_envelopes(
            secret="sim-secret",
            run_id="run-123",
            profile="blackbox",
            lane="container_blackbox",
            count=3,
        )
        self.assertEqual(len(envelopes), 3)
        nonces = {entry["nonce"] for entry in envelopes}
        self.assertEqual(len(nonces), 3)
        for entry in envelopes:
            self.assertTrue(entry["ts"])
            self.assertTrue(entry["signature"])

    def test_container_command_includes_hardening_flags(self):
        command = container_runner.container_command(
            image_tag="test:image",
            mode="isolation",
            base_url="http://host.docker.internal:3000",
            allowed_origin="http://host.docker.internal:3000",
            run_id="run-123",
            request_budget=12,
            time_budget_seconds=90,
            sim_tag_envelopes_json='[{"ts":"1","nonce":"n","signature":"s"}]',
            frontier_actions_json='[{"action_type":"http_get","path":"/"}]',
            allowed_tools_json='["http_get"]',
            capability_envelopes_json='[{"run_id":"r","step_id":1,"action_type":"http_get","path":"/","nonce":"n","issued_at":1,"expires_at":2,"key_id":"k","signature":"s"}]',
            capability_verify_key="verify-key",
            docker_flags=["--add-host=host.docker.internal:host-gateway", "--network=bridge"],
        )
        joined = " ".join(command)
        self.assertIn("--read-only", joined)
        self.assertIn("--cap-drop=ALL", joined)
        self.assertIn("--security-opt=no-new-privileges", joined)
        self.assertIn("--tmpfs=/tmp:rw,nosuid,nodev,size=64m", joined)
        self.assertIn("--add-host=host.docker.internal:host-gateway", joined)
        self.assertIn("BLACKBOX_SIM_TAG_ENVELOPES=", joined)
        self.assertIn("BLACKBOX_ACTIONS=", joined)
        self.assertIn("BLACKBOX_ALLOWED_TOOLS=", joined)
        self.assertIn("BLACKBOX_ACTION_ENVELOPES=", joined)
        self.assertIn("BLACKBOX_CAPABILITY_VERIFY_KEY=", joined)

    def test_worker_frontier_actions_payload_strips_resolved_only_fields(self):
        payload = container_runner.worker_frontier_actions_payload(
            [
                {
                    "action_index": 1,
                    "action_type": "http_get",
                    "method": "GET",
                    "path": "/sim/public/docs",
                    "query": {"q": "adaptive frontier probe"},
                    "label": "docs",
                    "url": "http://host.docker.internal:3000/sim/public/docs?q=adaptive+frontier+probe",
                    "target_origin": "http://host.docker.internal:3000",
                }
            ]
        )
        self.assertEqual(
            payload,
            [
                {
                    "action_type": "http_get",
                    "path": "/sim/public/docs",
                    "query": {"q": "adaptive frontier probe"},
                    "label": "docs",
                }
            ],
        )

    def test_worker_frontier_actions_payload_round_trips_through_contract(self):
        contract = load_frontier_action_contract(
            Path("scripts/tests/adversarial/frontier_action_contract.v1.json")
        )
        resolved_actions = resolve_frontier_actions(
            "",
            contract=contract,
            base_url="http://host.docker.internal:3000",
            allowed_origins=["http://host.docker.internal:3000"],
            request_budget=24,
        )
        worker_payload = container_runner.worker_frontier_actions_payload(resolved_actions)
        rerendered = resolve_frontier_actions(
            json.dumps(worker_payload),
            contract=contract,
            base_url="http://host.docker.internal:3000",
            allowed_origins=["http://host.docker.internal:3000"],
            request_budget=24,
        )
        self.assertEqual(len(rerendered), len(resolved_actions))
        self.assertEqual(rerendered[0]["path"], resolved_actions[0]["path"])
        self.assertEqual(rerendered[0]["action_type"], resolved_actions[0]["action_type"])

    def test_extract_frontier_actions_from_attack_plan_uses_candidate_path_hints(self):
        attack_plan = {
            "schema_version": "attack-plan.v1",
            "candidates": [
                {
                    "scenario_id": "scenario_a",
                    "payload": {
                        "schema_version": "frontier_payload_schema.v1",
                        "request_id": "req-a",
                        "target": {"path_hint": "/sim/public/docs"},
                    },
                },
                {
                    "scenario_id": "scenario_b",
                    "payload": {
                        "schema_version": "frontier_payload_schema.v1",
                        "request_id": "req-b",
                        "target": {"path_hint": "/challenge/not-a-bot-checkbox"},
                    },
                },
            ],
        }
        actions, lineage, rejected = container_runner.extract_frontier_actions_from_attack_plan(
            attack_plan,
            request_budget=2,
            forbidden_secret_values=[],
        )
        self.assertEqual(len(actions), 2)
        self.assertEqual(actions[0]["path"], "/sim/public/docs")
        self.assertEqual(actions[1]["path"], "/challenge/not-a-bot-checkbox")
        self.assertEqual(lineage[0]["scenario_id"], "scenario_a")
        self.assertEqual(lineage[1]["request_id"], "req-b")
        self.assertEqual(rejected, [])

    def test_extract_frontier_actions_from_attack_plan_rejects_unsafe_candidate_payload(self):
        attack_plan = {
            "schema_version": "attack-plan.v1",
            "candidates": [
                {
                    "scenario_id": "scenario_bad",
                    "payload": {
                        "schema_version": "frontier_payload_schema.v1",
                        "request_id": "req-bad",
                        "api_key_hint": "must-not-pass",
                        "target": {"path_hint": "/sim/public/docs"},
                    },
                }
            ],
        }
        with self.assertRaises(RuntimeError):
            container_runner.extract_frontier_actions_from_attack_plan(
                attack_plan,
                request_budget=1,
                forbidden_secret_values=[],
            )

    def test_load_attack_plan_requires_schema_and_candidates(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            attack_plan_path = Path(temp_dir) / "attack_plan.json"
            attack_plan_path.write_text(
                json.dumps({"schema_version": "attack-plan.v1", "candidates": []}),
                encoding="utf-8",
            )
            with self.assertRaises(RuntimeError):
                container_runner.load_attack_plan(attack_plan_path)

    def test_load_container_runtime_profile_accepts_repo_profile(self):
        profile = container_runner.load_container_runtime_profile(
            Path("scripts/tests/adversarial/container_runtime_profile.v1.json")
        )
        self.assertEqual(profile["schema_version"], "container-runtime-profile.v1")
        self.assertIn("--read-only", profile["required_flags"])
        self.assertEqual(
            profile["required_flag_groups_any_of"],
            [
                ["--add-host=host.docker.internal:host-gateway", "--network=bridge"],
                ["--network=host"],
            ],
        )

    def test_validate_attack_plan_candidate_payload_detects_secret_literal(self):
        payload = {
            "schema_version": "frontier_payload_schema.v1",
            "request_id": "req-1",
            "target": {"path_hint": "/sim/public/docs"},
            "attack_metadata": {"note": "sk-secret-value"},
        }
        reasons = container_runner.validate_attack_plan_candidate_payload(
            payload,
            forbidden_secret_values=["sk-secret-value"],
        )
        self.assertIn("literal_secret_value_detected", reasons)

    def test_evaluate_container_command_against_profile_detects_privileged_flags(self):
        profile = container_runner.load_container_runtime_profile(
            Path("scripts/tests/adversarial/container_runtime_profile.v1.json")
        )
        command = [
            "docker",
            "run",
            "--rm",
            "--privileged",
            "--network=bridge",
            "--add-host=host.docker.internal:host-gateway",
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--pids-limit=128",
            "--memory=256m",
            "--cpus=1.0",
            "--tmpfs=/tmp:rw,nosuid,nodev,size=64m",
            "test:image",
        ]
        violations = container_runner.evaluate_container_command_against_profile(
            command,
            profile,
        )
        self.assertTrue(any("forbidden_flag:--privileged" == item for item in violations))

    def test_evaluate_container_command_against_profile_accepts_supported_network_groups(self):
        profile = container_runner.load_container_runtime_profile(
            Path("scripts/tests/adversarial/container_runtime_profile.v1.json")
        )
        bridge_command = [
            "docker",
            "run",
            "--rm",
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--pids-limit=128",
            "--memory=256m",
            "--cpus=1.0",
            "--tmpfs=/tmp:rw,nosuid,nodev,size=64m",
            "--add-host=host.docker.internal:host-gateway",
            "--network=bridge",
            "test:image",
        ]
        host_command = [
            "docker",
            "run",
            "--rm",
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--pids-limit=128",
            "--memory=256m",
            "--cpus=1.0",
            "--tmpfs=/tmp:rw,nosuid,nodev,size=64m",
            "--network=host",
            "test:image",
        ]
        self.assertEqual(
            container_runner.evaluate_container_command_against_profile(bridge_command, profile),
            [],
        )
        self.assertEqual(
            container_runner.evaluate_container_command_against_profile(host_command, profile),
            [],
        )

    def test_evaluate_container_command_against_profile_detects_missing_hardening_flags(self):
        profile = container_runner.load_container_runtime_profile(
            Path("scripts/tests/adversarial/container_runtime_profile.v1.json")
        )
        command = ["docker", "run", "--rm", "test:image"]
        violations = container_runner.evaluate_container_command_against_profile(
            command,
            profile,
        )
        self.assertTrue(any(item == "missing_required_flag:--read-only" for item in violations))
        self.assertTrue(any(item == "missing_required_flag:--cap-drop=ALL" for item in violations))
        self.assertTrue(
            any(item == "missing_required_flag:--security-opt=no-new-privileges" for item in violations)
        )
        self.assertTrue(any(item.startswith("missing_required_flag_group_any_of:") for item in violations))

    def test_evaluate_container_command_against_profile_detects_forbidden_host_mounts(self):
        profile = container_runner.load_container_runtime_profile(
            Path("scripts/tests/adversarial/container_runtime_profile.v1.json")
        )
        command = [
            "docker",
            "run",
            "--rm",
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--network=bridge",
            "-v",
            "/var/run/docker.sock:/var/run/docker.sock",
            "-v",
            "/:/host-root:ro",
            "test:image",
        ]
        violations = container_runner.evaluate_container_command_against_profile(
            command,
            profile,
        )
        self.assertTrue(
            any(
                item == "forbidden_mount_fragment:/var/run/docker.sock:/var/run/docker.sock"
                for item in violations
            )
        )
        self.assertTrue(any(item == "forbidden_mount_fragment:/:/host-root:ro" for item in violations))

    def test_prepare_command_channel_reports_backpressure_overflow(self):
        channel = container_runner.prepare_command_channel(
            actions=[
                {"action_type": "http_get", "path": "/a"},
                {"action_type": "http_get", "path": "/b"},
                {"action_type": "http_get", "path": "/c"},
            ],
            queue_capacity=2,
        )
        self.assertEqual(channel["queued_action_count"], 2)
        self.assertEqual(channel["overflow_count"], 1)
        self.assertTrue(channel["backpressure_applied"])

    def test_build_frontier_lineage_summary_links_model_execution_and_runtime_events(self):
        summary = container_runner.build_frontier_lineage_summary(
            frontier_action_lineage=[
                {
                    "candidate_index": 1,
                    "scenario_id": "scenario_a",
                    "request_id": "req-a",
                    "proposed_action": {
                        "action_index": 1,
                        "action_type": "http_get",
                        "path": "/sim/public/docs",
                    },
                }
            ],
            worker_payload={
                "traffic": [
                    {
                        "action_index": 1,
                        "status": 200,
                        "path": "/sim/public/docs",
                    }
                ]
            },
            runtime_events=[{"reason": "not_a_bot_pass"}],
            monitoring_events=[{"reason": "not_a_bot_pass"}],
        )
        self.assertTrue(summary["lineage_complete"])
        self.assertEqual(summary["model_suggestion_count"], 1)
        self.assertEqual(summary["executed_action_count"], 1)
        self.assertEqual(summary["runtime_event_count"], 1)
        self.assertEqual(summary["monitoring_event_count"], 1)

    def test_build_frontier_runtime_state_marks_degraded_fallback_paths(self):
        degraded = container_runner.build_frontier_runtime_state(
            mode="blackbox",
            frontier_actions_source="contract_default_fallback",
            frontier_action_source_error="attack plan not found",
            frontier_lineage={"detail": "lineage_collection_error:missing_api_key"},
        )
        self.assertTrue(degraded["degraded"])
        self.assertEqual(degraded["status"], "degraded")
        self.assertIn("attack_plan_unavailable_or_invalid", degraded["reasons"])

        healthy = container_runner.build_frontier_runtime_state(
            mode="blackbox",
            frontier_actions_source="attack_plan_candidates",
            frontier_action_source_error="",
            frontier_lineage={"detail": "ok"},
        )
        self.assertFalse(healthy["degraded"])
        self.assertEqual(healthy["status"], "ok")

    def test_cleanup_frontier_artifacts_deletes_only_stale_reports(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            artifacts_dir = Path(temp_dir)
            stale = artifacts_dir / "container_blackbox_report.old.json"
            fresh = artifacts_dir / "container_blackbox_report.new.json"
            stale.write_text("{}", encoding="utf-8")
            fresh.write_text("{}", encoding="utf-8")

            old_unix = time.time() - (8 * 3600)
            os.utime(stale, (old_unix, old_unix))

            result = container_runner.cleanup_frontier_artifacts(
                artifacts_dir,
                ttl_hours=1,
                max_delete=10,
            )
            self.assertEqual(result["deleted_count"], 0)
            # Only files matching container_*_report.json are subject to cleanup.
            stale_eligible = artifacts_dir / "container_stale_report.json"
            fresh_eligible = artifacts_dir / "container_fresh_report.json"
            stale_eligible.write_text("{}", encoding="utf-8")
            fresh_eligible.write_text("{}", encoding="utf-8")
            os.utime(stale_eligible, (old_unix, old_unix))

            result = container_runner.cleanup_frontier_artifacts(
                artifacts_dir,
                ttl_hours=1,
                max_delete=10,
            )
            self.assertEqual(result["deleted_count"], 1)
            self.assertFalse(stale_eligible.exists())
            self.assertTrue(fresh_eligible.exists())
            self.assertEqual(result["failed_count"], 0)

    def test_parse_worker_failure_taxonomy_maps_deadline_and_heartbeat_failures(self):
        deadline = container_runner.parse_worker_failure_taxonomy(
            "hard_deadline_exceeded:stop_latency_ms=120:hard_deadline_seconds=45:forced_kill=false"
        )
        self.assertEqual(deadline["terminal_failure"], "deadline_exceeded")
        self.assertEqual(deadline["reason"], "hard_deadline_exceeded")
        self.assertEqual(deadline["stop_latency_ms"], 120)
        self.assertEqual(deadline["hard_deadline_seconds"], 45)
        self.assertFalse(deadline["forced_kill"])

        heartbeat = container_runner.parse_worker_failure_taxonomy(
            "heartbeat_timeout:stop_latency_ms=22:hard_deadline_seconds=120:forced_kill=false"
        )
        self.assertEqual(heartbeat["terminal_failure"], "heartbeat_loss")
        self.assertEqual(heartbeat["reason"], "heartbeat_timeout")
        self.assertFalse(heartbeat["forced_kill"])

    def test_parse_worker_failure_taxonomy_marks_forced_kill_path(self):
        forced = container_runner.parse_worker_failure_taxonomy(
            "kill_switch_triggered:stop_latency_ms=9999:hard_deadline_seconds=90:forced_kill=true"
        )
        self.assertEqual(forced["terminal_failure"], "forced_kill_path")
        self.assertEqual(forced["reason"], "kill_switch_triggered")
        self.assertTrue(forced["forced_kill"])


if __name__ == "__main__":
    unittest.main()
