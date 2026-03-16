#!/usr/bin/env python3

import unittest

import scripts.tests.adversarial_live_loop as live_loop


class AdversarialLiveLoopUnitTests(unittest.TestCase):
    def test_classify_failure_transient_for_retryable_signatures(self):
        classification = live_loop.classify_failure(
            "Adversarial simulation failed: HTTP Error 503 Service Unavailable"
        )
        self.assertEqual(classification, "transient")

    def test_classify_failure_fatal_for_non_retryable_output(self):
        classification = live_loop.classify_failure("invariant_failed unexpected outcome mismatch")
        self.assertEqual(classification, "fatal")

    def test_meaningful_defense_events_rejects_admin_only_noise(self):
        meaningful, matched = live_loop.has_meaningful_defense_events(
            [
                "admin_unban",
                "config_export",
                "shadow_mode_toggle",
            ]
        )
        self.assertFalse(meaningful)
        self.assertEqual(matched, [])

    def test_meaningful_defense_events_detects_policy_activity(self):
        meaningful, matched = live_loop.has_meaningful_defense_events(
            [
                "admin_unban",
                "geo_policy_challenge",
                "rate",
                "not_a_bot_fail",
            ]
        )
        self.assertTrue(meaningful)
        self.assertIn("geo_policy_challenge", matched)
        self.assertIn("not_a_bot_fail", matched)

    def test_read_tarpit_metrics_extracts_nested_counters(self):
        report = {
            "monitoring_after": {
                "tarpit": {
                    "metrics": {
                        "activations": {"progressive": 3},
                        "progress_outcomes": {"advanced": 2},
                        "budget_outcomes": {"fallback_maze": 1, "fallback_block": 1},
                        "escalation_outcomes": {"short_ban": 4, "block": 5},
                    }
                }
            }
        }
        metrics = live_loop.read_tarpit_metrics(report)
        self.assertEqual(metrics["activations_progressive"], 3)
        self.assertEqual(metrics["progress_advanced"], 2)
        self.assertEqual(metrics["fallback_maze"], 1)
        self.assertEqual(metrics["fallback_block"], 1)
        self.assertEqual(metrics["escalation_short_ban"], 4)
        self.assertEqual(metrics["escalation_block"], 5)


if __name__ == "__main__":
    unittest.main()
