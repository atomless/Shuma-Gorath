#!/usr/bin/env python3

import unittest

import scripts.tests.frontier_capability_envelope as capability


class FrontierCapabilityEnvelopeUnitTests(unittest.TestCase):
    def test_build_and_validate_action_capability_envelopes(self):
        actions = [
            {"action_type": "http_get", "path": "/sim/public/docs"},
            {"action_type": "http_get", "path": "/sim/public/contact"},
        ]
        verify_key, envelopes = capability.build_action_capability_envelopes(
            root_secret="sim-secret",
            run_id="run-123",
            actions=actions,
            ttl_seconds=120,
            now_unix=1_700_000_000,
        )
        errors = capability.validate_action_capability_envelopes(
            envelopes,
            verify_key=verify_key,
            run_id="run-123",
            actions=actions,
            now_unix=1_700_000_030,
        )
        self.assertEqual(errors, [])

    def test_validate_action_capability_envelopes_rejects_signature_mismatch(self):
        actions = [{"action_type": "http_get", "path": "/sim/public/docs"}]
        verify_key, envelopes = capability.build_action_capability_envelopes(
            root_secret="sim-secret",
            run_id="run-123",
            actions=actions,
            ttl_seconds=120,
            now_unix=1_700_000_000,
        )
        envelopes[0]["signature"] = "0" * 64
        errors = capability.validate_action_capability_envelopes(
            envelopes,
            verify_key=verify_key,
            run_id="run-123",
            actions=actions,
            now_unix=1_700_000_010,
        )
        self.assertTrue(any("invalid_signature" in error for error in errors))

    def test_validate_action_capability_envelopes_rejects_nonce_replay(self):
        actions = [
            {"action_type": "http_get", "path": "/sim/public/docs"},
            {"action_type": "http_get", "path": "/sim/public/contact"},
        ]
        verify_key, envelopes = capability.build_action_capability_envelopes(
            root_secret="sim-secret",
            run_id="run-123",
            actions=actions,
            ttl_seconds=120,
            now_unix=1_700_000_000,
        )
        envelopes[1]["nonce"] = envelopes[0]["nonce"]
        errors = capability.validate_action_capability_envelopes(
            envelopes,
            verify_key=verify_key,
            run_id="run-123",
            actions=actions,
            now_unix=1_700_000_010,
        )
        self.assertTrue(any("nonce_replay" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
