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


if __name__ == "__main__":
    unittest.main()
