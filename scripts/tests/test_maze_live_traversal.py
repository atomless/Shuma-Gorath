import unittest
from pathlib import Path

import scripts.tests.maze_live_traversal as maze_live_traversal


class MazeLiveTraversalTests(unittest.TestCase):
    def test_request_uses_configured_timeout_seconds(self) -> None:
        gate = maze_live_traversal.MazeLiveTraversalGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=42,
            report_path=Path("/tmp/maze-live-traversal.json"),
        )
        captured: dict[str, object] = {}

        class _Response:
            status = 200

            def read(self) -> bytes:
                return b"{}"

            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb) -> bool:
                return False

        class _Opener:
            def open(self, req, timeout=None):
                captured["timeout"] = timeout
                captured["url"] = req.full_url
                return _Response()

        gate.opener = _Opener()

        response = gate._request("GET", "/shuma/health")

        self.assertEqual(response["status"], 200)
        self.assertEqual(captured["timeout"], 42)
        self.assertEqual(captured["url"], "http://127.0.0.1:3000/shuma/health")

    def test_extract_first_maze_link_returns_first_maze_href(self) -> None:
        html = """
        <html><body>
          <a href="/ignore" class="nav-card"><h3>Ignore</h3></a>
          <a href="/_/opaque/next?mt=token-1" class="nav-card" data-link-kind="maze">
            <h3>Continue</h3>
          </a>
          <a href="/_/opaque/other?mt=token-2" class="nav-card" data-link-kind="maze">
            <h3>Later</h3>
          </a>
        </body></html>
        """

        self.assertEqual(
            maze_live_traversal.extract_first_maze_link(html),
            "/_/opaque/next?mt=token-1",
        )

    def test_extract_bootstrap_json_parses_maze_payload(self) -> None:
        html = """
        <script id="maze-bootstrap" type="application/json">
        {"flow_id":"flow-1","checkpoint_token":"cp-1","path_prefix":"/_/opaque/"}
        </script>
        """

        payload = maze_live_traversal.extract_bootstrap_json(html)

        self.assertEqual(payload["flow_id"], "flow-1")
        self.assertEqual(payload["checkpoint_token"], "cp-1")
        self.assertEqual(payload["path_prefix"], "/_/opaque/")

    def test_extract_preview_entry_path_decodes_admin_preview_link(self) -> None:
        html = """
        <a href="/shuma/admin/maze/preview?path=%2F_%2Fopaque123%2Fentry456">Continue</a>
        """

        self.assertEqual(
            maze_live_traversal.extract_preview_entry_path(html),
            "/_/opaque123/entry456",
        )

    def test_build_issue_links_payload_projects_bootstrap_fields(self) -> None:
        bootstrap = {
            "flow_id": "flow-1",
            "checkpoint_token": "cp-1",
            "entropy_nonce": "nonce-1",
            "path_prefix": "/_/opaque/",
            "client_expansion": {
                "seed": 99,
                "seed_sig": "sig-1",
                "hidden_count": 6,
                "segment_len": 16,
            },
        }

        payload = maze_live_traversal.build_issue_links_payload(bootstrap, requested_hidden_count=3)

        self.assertEqual(payload["parent_token"], "cp-1")
        self.assertEqual(payload["flow_id"], "flow-1")
        self.assertEqual(payload["entropy_nonce"], "nonce-1")
        self.assertEqual(payload["path_prefix"], "/_/opaque/")
        self.assertEqual(payload["seed"], 99)
        self.assertEqual(payload["seed_sig"], "sig-1")
        self.assertEqual(payload["hidden_count"], 6)
        self.assertEqual(payload["requested_hidden_count"], 3)
        self.assertEqual(payload["segment_len"], 16)
        self.assertEqual(payload["candidates"], [])

    def test_find_recent_fallback_event_matches_reason_action_and_type(self) -> None:
        events = [
            {
                "event": "Challenge",
                "reason": "maze_runtime_fallback",
                "outcome": "maze_checkpoint_missing action=challenge",
            },
            {
                "event": "Block",
                "reason": "maze_runtime_fallback",
                "outcome": "maze_token_replay action=block",
            },
        ]

        checkpoint = maze_live_traversal.find_recent_fallback_event(
            events,
            event_type="Challenge",
            reason_label="maze_checkpoint_missing",
            action_label="challenge",
        )
        replay = maze_live_traversal.find_recent_fallback_event(
            events,
            event_type="Block",
            reason_label="maze_token_replay",
            action_label="block",
        )

        self.assertEqual(checkpoint["event"], "Challenge")
        self.assertEqual(replay["event"], "Block")


if __name__ == "__main__":
    unittest.main()
