#!/usr/bin/env python3

import unittest
from pathlib import Path

import scripts.tests.frontier_action_contract as frontier_contract


class FrontierActionContractUnitTests(unittest.TestCase):
    def setUp(self):
        self.contract = frontier_contract.load_frontier_action_contract(
            Path("scripts/tests/adversarial/frontier_action_contract.v1.json")
        )

    def test_load_frontier_action_contract_accepts_repo_contract(self):
        self.assertEqual(self.contract["schema_version"], "frontier-action-contract.v1")
        self.assertIn("http_get", self.contract["allowed_tools"])

    def test_resolve_frontier_actions_uses_contract_defaults_when_empty_input(self):
        actions = frontier_contract.resolve_frontier_actions(
            "",
            contract=self.contract,
            base_url="http://host.docker.internal:3000",
            allowed_origins=["http://host.docker.internal:3000"],
            request_budget=24,
        )
        self.assertGreaterEqual(len(actions), 1)
        self.assertTrue(all(action["action_type"] == "http_get" for action in actions))

    def test_resolve_frontier_actions_rejects_forbidden_admin_prefix(self):
        raw_actions = '[{"action_type":"http_get","path":"/admin/config"}]'
        with self.assertRaises(frontier_contract.FrontierActionValidationError):
            frontier_contract.resolve_frontier_actions(
                raw_actions,
                contract=self.contract,
                base_url="http://host.docker.internal:3000",
                allowed_origins=["http://host.docker.internal:3000"],
                request_budget=24,
            )

    def test_resolve_frontier_actions_rejects_unsupported_keys(self):
        raw_actions = '[{"action_type":"http_get","path":"/","headers":{"authorization":"x"}}]'
        with self.assertRaises(frontier_contract.FrontierActionValidationError):
            frontier_contract.resolve_frontier_actions(
                raw_actions,
                contract=self.contract,
                base_url="http://host.docker.internal:3000",
                allowed_origins=["http://host.docker.internal:3000"],
                request_budget=24,
            )

    def test_resolve_frontier_actions_rejects_out_of_scope_absolute_url_attempt(self):
        raw_actions = '[{"action_type":"http_get","path":"http://evil.invalid/"}]'
        with self.assertRaises(frontier_contract.FrontierActionValidationError):
            frontier_contract.resolve_frontier_actions(
                raw_actions,
                contract=self.contract,
                base_url="http://host.docker.internal:3000",
                allowed_origins=["http://host.docker.internal:3000"],
                request_budget=24,
            )

    def test_resolve_frontier_actions_rejects_action_count_above_budget(self):
        raw_actions = (
            '[{"action_type":"http_get","path":"/a"},'
            '{"action_type":"http_get","path":"/b"},'
            '{"action_type":"http_get","path":"/c"}]'
        )
        with self.assertRaises(frontier_contract.FrontierActionValidationError):
            frontier_contract.resolve_frontier_actions(
                raw_actions,
                contract=self.contract,
                base_url="http://host.docker.internal:3000",
                allowed_origins=["http://host.docker.internal:3000"],
                request_budget=2,
            )

    def test_parse_frontier_actions_rejects_non_array_json(self):
        with self.assertRaises(frontier_contract.FrontierActionValidationError):
            frontier_contract.parse_frontier_actions('{"action_type":"http_get"}')


if __name__ == "__main__":
    unittest.main()
