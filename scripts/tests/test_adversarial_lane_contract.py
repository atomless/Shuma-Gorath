#!/usr/bin/env python3

import unittest

import scripts.tests.adversarial_simulation_runner as sim_runner
import scripts.tests.check_adversarial_lane_contract as lane_contract_check


class AdversarialLaneContractUnitTests(unittest.TestCase):
    def test_lane_contract_validator_passes(self):
        errors = lane_contract_check.validate_lane_contract()
        self.assertEqual(errors, [])

    def test_attacker_contract_rejects_forwarded_secret(self):
        with self.assertRaises(sim_runner.SimulationError):
            sim_runner.enforce_attacker_request_contract(
                "/",
                {
                    "X-Forwarded-For": "10.0.0.1",
                    "X-Shuma-Sim-Run-Id": "run-1",
                    "X-Shuma-Sim-Profile": "fast_smoke",
                    "X-Shuma-Sim-Lane": "deterministic_black_box",
                    "X-Shuma-Forwarded-Secret": "forbidden",
                },
            )

    def test_attacker_contract_allows_request_native_headers_used_by_scrapling_personas(self):
        sim_runner.enforce_attacker_request_contract(
            "/agent/submit",
            {
                "Accept": "application/json",
                "Content-Type": "application/json",
                "Cookie": "shuma_agent_mode=http_agent",
                "X-Shuma-Sim-Run-Id": "run-1",
                "X-Shuma-Sim-Profile": "scrapling_runtime_lane",
                "X-Shuma-Sim-Lane": "scrapling_traffic",
                "X-Shuma-Sim-Nonce": "nonce",
                "X-Shuma-Sim-Ts": "1234567890",
                "X-Shuma-Sim-Signature": "sig",
            },
        )


if __name__ == "__main__":
    unittest.main()
