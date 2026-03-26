#!/usr/bin/env python3

import unittest

import scripts.tests.adversarial_container.worker as worker
from scripts.tests.frontier_action_contract import load_frontier_action_contract


class _FakeBrowserResponse:
    def __init__(self, status):
        self._status = status

    def status(self):
        return self._status


class _FakeLocator:
    def __init__(self, *, count=1, visible=True):
        self._count = count
        self._visible = visible
        self.clicked = False

    def count(self):
        return self._count

    def is_visible(self):
        return self._visible

    def click(self, timeout=0):
        self.clicked = True


class _FakePage:
    def __init__(self):
        self.goto_urls = []
        self._locators = {
            'a:has-text("Get Started")': _FakeLocator(),
            "a[href]": _FakeLocator(),
        }

    def goto(self, url, wait_until="", timeout=0):
        self.goto_urls.append(url)
        return _FakeBrowserResponse(200)

    def title(self):
        return "Landing"

    def content(self):
        return "<html><body><a href=\"/sim/public/landing\">Get Started</a></body></html>"

    def locator(self, selector):
        return self._locators.get(selector, _FakeLocator(count=0, visible=False))

    def wait_for_load_state(self, state="", timeout=0):
        return None

    def wait_for_timeout(self, timeout=0):
        return None


class _FakeContext:
    def __init__(self, page):
        self._page = page

    def new_page(self):
        return self._page

    def close(self):
        return None


class _FakeBrowser:
    def __init__(self, page):
        self._page = page

    def new_context(self, **_kwargs):
        return _FakeContext(self._page)

    def close(self):
        return None


class _FakePlaywright:
    def __init__(self, page):
        self.chromium = self
        self._page = page

    def launch(self, **_kwargs):
        return _FakeBrowser(self._page)


class _FakePlaywrightContextManager:
    def __init__(self, page):
        self._page = page

    def __enter__(self):
        return _FakePlaywright(self._page)

    def __exit__(self, exc_type, exc, tb):
        return False


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

    def test_resolve_worker_actions_accepts_browser_actions_only_with_explicit_allowed_tools(self):
        contract = load_frontier_action_contract()
        raw_actions = '[{"action_type":"browser_navigate","path":"/","label":"root"}]'

        with self.assertRaisesRegex(Exception, "browser_navigate"):
            worker.resolve_worker_actions(
                raw_actions,
                contract=contract,
                base_url="http://host.docker.internal:3000",
                allowed_origins=["http://host.docker.internal:3000"],
                request_budget=8,
                allowed_tools=[],
            )

        resolved = worker.resolve_worker_actions(
            raw_actions,
            contract=contract,
            base_url="http://host.docker.internal:3000",
            allowed_origins=["http://host.docker.internal:3000"],
            request_budget=8,
            allowed_tools=["browser_navigate", "browser_snapshot", "browser_click"],
        )

        self.assertEqual(len(resolved), 1)
        self.assertEqual(resolved[0]["action_type"], "browser_navigate")
        self.assertEqual(resolved[0]["path"], "/")

    def test_execute_browser_actions_records_navigation_snapshot_and_click_receipts(self):
        page = _FakePage()
        report = worker.execute_browser_actions(
            base_url="http://host.docker.internal:3000",
            sim_headers={"x-shuma-sim-run-id": "run-123"},
            resolved_actions=[
                {
                    "action_index": 1,
                    "action_type": "browser_navigate",
                    "path": "/",
                    "url": "http://host.docker.internal:3000/",
                    "label": "root",
                },
                {
                    "action_index": 2,
                    "action_type": "browser_snapshot",
                    "path": "/sim/public/landing",
                    "url": "http://host.docker.internal:3000/sim/public/landing",
                    "label": "landing",
                },
                {
                    "action_index": 3,
                    "action_type": "browser_click",
                    "path": "/sim/public/landing",
                    "url": "http://host.docker.internal:3000/sim/public/landing",
                    "label": "Get Started",
                },
            ],
            time_budget_seconds=90,
            playwright_factory=lambda: _FakePlaywrightContextManager(page),
        )

        self.assertEqual(report["requests_sent"], 3)
        self.assertEqual(len(report["traffic"]), 3)
        self.assertEqual(report["traffic"][0]["action_type"], "browser_navigate")
        self.assertEqual(report["traffic"][1]["action_type"], "browser_snapshot")
        self.assertEqual(report["traffic"][2]["action_type"], "browser_click")
        self.assertEqual(report["traffic"][2]["status"], 200)
        self.assertTrue(page._locators['a:has-text("Get Started")'].clicked)


if __name__ == "__main__":
    unittest.main()
