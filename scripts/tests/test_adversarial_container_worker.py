#!/usr/bin/env python3

import contextlib
import io
import json
import os
from unittest import mock
import unittest

from scripts.tests.adversarial_container import worker


def _sim_tag_envelopes(count: int) -> str:
    payload = []
    for index in range(count):
        payload.append(
            {
                "ts": str(1_700_000_000 + index),
                "nonce": f"nonce-{index}",
                "signature": f"sig-{index}",
            }
        )
    return json.dumps(payload)


class AdversarialContainerWorkerUnitTests(unittest.TestCase):
    def test_append_policy_audit_event_captures_action_context(self):
        events = []
        worker.append_policy_audit_event(
            events,
            stage="execution",
            decision="deny",
            code="egress_disallowed",
            detail="http://example.invalid",
            action={
                "action_index": 2,
                "action_type": "http_get",
                "path": "/admin/config",
            },
        )
        self.assertEqual(len(events), 1)
        event = events[0]
        self.assertEqual(event["stage"], "execution")
        self.assertEqual(event["decision"], "deny")
        self.assertEqual(event["code"], "egress_disallowed")
        self.assertEqual(event["action_index"], 2)
        self.assertEqual(event["action_type"], "http_get")
        self.assertEqual(event["path"], "/admin/config")
        self.assertIn("ts_unix", event)

    def test_enforce_allowlist_rejects_origin_not_in_allowlist(self):
        allowed = ["http://host.docker.internal:3000"]
        self.assertTrue(worker.enforce_allowlist("http://host.docker.internal:3000/", allowed))
        self.assertFalse(worker.enforce_allowlist("http://evil.invalid/", allowed))

    def test_parse_sim_tag_envelopes_rejects_nonce_replay(self):
        replay_payload = (
            '[{"ts":"1700000000","nonce":"nonce-1","signature":"sig-a"},'
            '{"ts":"1700000001","nonce":"nonce-1","signature":"sig-b"}]'
        )
        self.assertEqual(worker.parse_sim_tag_envelopes(replay_payload), [])

    def test_blackbox_main_emits_request_mode_realism_receipt_and_stops_after_denial_burst(self):
        actions = []
        for index in range(1, 10):
            path = "/" if index % 2 else "/robots.txt"
            label = "root" if path == "/" else "robots"
            actions.append(
                {
                    "action_index": index,
                    "action_type": "http_get",
                    "path": path,
                    "label": label,
                    "url": f"https://example.test{path}",
                }
            )

        realism_plan = {
            "schema_version": "adversary-sim-llm-request-realism-plan.v1",
            "profile_id": "agentic.request_mode.v1",
            "planned_activity_budget": 12,
            "effective_activity_budget": 9,
            "planned_burst_size": 3,
            "effective_burst_size": 3,
            "burst_sizes": [3, 3, 3],
            "inter_action_gaps_ms": [150, 220, 1400, 180, 240, 1300, 160, 210],
            "focused_page_paths": ["/", "/robots.txt"],
            "session_handles": ["agentic-request-session-1"],
        }
        env = {
            "BLACKBOX_MODE": "blackbox",
            "BLACKBOX_BASE_URL": "https://example.test/",
            "BLACKBOX_ALLOWED_ORIGINS": "https://example.test",
            "BLACKBOX_RUN_ID": "simrun-llm-runtime",
            "BLACKBOX_REQUEST_BUDGET": "9",
            "BLACKBOX_TIME_BUDGET_SECONDS": "120",
            "BLACKBOX_SIM_TAG_ENVELOPES": _sim_tag_envelopes(9),
            "BLACKBOX_ACTIONS": json.dumps(actions),
            "BLACKBOX_REQUEST_REALISM_PLAN": json.dumps(realism_plan),
            worker.CAPABILITY_ENVELOPES_ENV: "[]",
            worker.CAPABILITY_VERIFY_KEY_ENV: "verify-key",
        }
        lane_contract = {
            "schema_version": "lane-contract.v1",
            "attacker": {
                "required_sim_headers": [
                    worker.SIM_TAG_HEADER_RUN_ID,
                    worker.SIM_TAG_HEADER_PROFILE,
                    worker.SIM_TAG_HEADER_LANE,
                    worker.SIM_TAG_HEADER_TIMESTAMP,
                    worker.SIM_TAG_HEADER_NONCE,
                    worker.SIM_TAG_HEADER_SIGNATURE,
                ],
                "forbidden_headers": [],
            },
        }
        request_results = [
            {"status": 200, "latency_ms": 10, "url": "https://example.test/"},
            {"status": 200, "latency_ms": 12, "url": "https://example.test/robots.txt"},
            {"status": 200, "latency_ms": 9, "url": "https://example.test/"},
            {"status": 429, "latency_ms": 15, "url": "https://example.test/robots.txt"},
            {"status": 429, "latency_ms": 18, "url": "https://example.test/"},
            {"status": 429, "latency_ms": 20, "url": "https://example.test/robots.txt"},
        ]

        stdout = io.StringIO()
        with (
            mock.patch.dict(os.environ, env, clear=True),
            mock.patch("scripts.tests.adversarial_container.worker.os.getuid", return_value=1000),
            mock.patch("scripts.tests.adversarial_container.worker.workspace_mount_absent", return_value=True),
            mock.patch("scripts.tests.adversarial_container.worker.load_lane_contract", return_value=lane_contract),
            mock.patch(
                "scripts.tests.adversarial_container.worker.load_frontier_action_contract",
                return_value={"schema_version": "frontier_action_contract.v1"},
            ),
            mock.patch(
                "scripts.tests.adversarial_container.worker.resolve_frontier_actions",
                return_value=actions,
            ),
            mock.patch(
                "scripts.tests.adversarial_container.worker.parse_action_capability_envelopes",
                return_value=[],
            ),
            mock.patch(
                "scripts.tests.adversarial_container.worker.validate_action_capability_envelopes",
                return_value=[],
            ),
            mock.patch(
                "scripts.tests.adversarial_container.worker.make_request",
                side_effect=request_results,
            ),
            mock.patch("scripts.tests.adversarial_container.worker.time.sleep") as sleep_mock,
            contextlib.redirect_stdout(stdout),
        ):
            exit_code = worker.main()

        payload = json.loads(stdout.getvalue())
        receipt = payload["realism_receipt"]

        self.assertEqual(exit_code, 0)
        self.assertEqual(payload["requests_sent"], 6)
        self.assertEqual(len(payload["traffic"]), 6)
        self.assertEqual(receipt["profile_id"], "agentic.request_mode.v1")
        self.assertEqual(receipt["activity_count"], 6)
        self.assertEqual(receipt["burst_count"], 2)
        self.assertEqual(receipt["burst_sizes"], [3, 3])
        self.assertEqual(receipt["focused_page_set_size"], 2)
        self.assertEqual(
            receipt["session_handles"],
            ["agentic-request-session-1"],
        )
        self.assertEqual(receipt["stop_reason"], "response_pressure_stop")
        self.assertEqual(len(receipt["inter_activity_gaps_ms"]), 5)
        self.assertEqual(sleep_mock.call_count, 5)


if __name__ == "__main__":
    unittest.main()
