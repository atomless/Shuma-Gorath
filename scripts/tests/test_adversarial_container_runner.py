#!/usr/bin/env python3

import json
import tempfile
import unittest
from pathlib import Path

import scripts.tests.adversarial_container_runner as container_runner


class AdversarialContainerRunnerUnitTests(unittest.TestCase):
    def test_normalize_container_base_url_rewrites_loopback(self):
        rewritten = container_runner.normalize_container_base_url("http://127.0.0.1:3000")
        self.assertEqual(rewritten, "http://host.docker.internal:3000")

    def test_target_origin_returns_scheme_and_netloc(self):
        origin = container_runner.target_origin("https://example.com:8443/path?q=1")
        self.assertEqual(origin, "https://example.com:8443")

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
        )
        joined = " ".join(command)
        self.assertIn("--read-only", joined)
        self.assertIn("--cap-drop=ALL", joined)
        self.assertIn("--security-opt=no-new-privileges", joined)
        self.assertIn("--tmpfs=/tmp:rw,nosuid,nodev,size=64m", joined)
        self.assertIn("--add-host=host.docker.internal:host-gateway", joined)
        self.assertIn("BLACKBOX_SIM_TAG_ENVELOPES=", joined)
        self.assertIn("BLACKBOX_ACTIONS=", joined)

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


if __name__ == "__main__":
    unittest.main()
