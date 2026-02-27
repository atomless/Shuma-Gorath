#!/usr/bin/env python3

import re
import unittest

import scripts.tests.adversarial_simulation_runner as sim_runner
import scripts.tests.check_adversarial_sim_tag_contract as sim_tag_contract_check


class AdversarialSimTagContractUnitTests(unittest.TestCase):
    def test_sim_tag_contract_validator_passes(self):
        errors = sim_tag_contract_check.validate_sim_tag_contract()
        self.assertEqual(errors, [])

    def test_sign_sim_tag_emits_hex_sha256_digest(self):
        signature = sim_runner.sign_sim_tag(
            secret="sim-secret",
            run_id="run-123",
            profile="fast_smoke",
            lane="deterministic_black_box",
            timestamp="1700000000",
            nonce="nonce-123",
        )
        self.assertRegex(signature, re.compile(r"^[0-9a-f]{64}$"))


if __name__ == "__main__":
    unittest.main()
