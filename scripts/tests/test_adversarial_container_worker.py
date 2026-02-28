#!/usr/bin/env python3

import unittest

import scripts.tests.adversarial_container.worker as worker


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


if __name__ == "__main__":
    unittest.main()
