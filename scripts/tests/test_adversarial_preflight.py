#!/usr/bin/env python3

import unittest

import scripts.tests.adversarial_preflight as preflight


class AdversarialPreflightUnitTests(unittest.TestCase):
    def test_evaluate_passes_with_valid_secrets(self):
        payload = preflight.evaluate(
            {
                "SHUMA_API_KEY": "a" * 64,
                "SHUMA_SIM_TELEMETRY_SECRET": "1" * 64,
            }
        )
        self.assertTrue(payload["status"]["passed"])
        self.assertEqual(payload["status"]["failure_count"], 0)

    def test_evaluate_fails_with_missing_and_placeholder_secrets(self):
        payload = preflight.evaluate(
            {
                "SHUMA_API_KEY": "changeme-dev-only-api-key",
                "SHUMA_SIM_TELEMETRY_SECRET": "not-hex",
            }
        )
        self.assertFalse(payload["status"]["passed"])
        joined = " ".join(payload["status"]["failures"])
        self.assertIn("preflight_placeholder_secret:SHUMA_API_KEY", joined)
        self.assertIn("preflight_invalid_secret_format:SHUMA_SIM_TELEMETRY_SECRET", joined)
        self.assertGreaterEqual(len(payload["guidance"]), 1)


if __name__ == "__main__":
    unittest.main()
