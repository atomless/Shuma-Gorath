#!/usr/bin/env python3

import unittest

import scripts.tests.check_shared_host_scope_contract as scope_contract_check
import scripts.tests.shared_host_scope as shared_host_scope


class SharedHostScopeUnitTests(unittest.TestCase):
    def make_descriptor(self, **overrides):
        payload = {
            "allowed_hosts": ["Example.com", "www.example.com"],
            "denied_path_prefixes": ["/private", "/admin"],
            "require_https": True,
            "deny_ip_literals": True,
        }
        payload.update(overrides)
        return shared_host_scope.descriptor_from_payload(payload)

    def test_contract_validator_passes(self):
        errors = scope_contract_check.validate_shared_host_scope_contract()
        self.assertEqual(errors, [])

    def test_descriptor_normalization_adds_baseline_denied_prefixes(self):
        descriptor = self.make_descriptor(denied_path_prefixes=["/private/"])
        self.assertEqual(
            descriptor.allowed_hosts,
            ("example.com", "www.example.com"),
        )
        self.assertIn("/private", descriptor.denied_path_prefixes)
        self.assertIn("/admin", descriptor.denied_path_prefixes)
        self.assertIn("/internal", descriptor.denied_path_prefixes)

    def test_accepts_in_scope_https_candidate_and_strips_fragment(self):
        descriptor = self.make_descriptor()
        decision = shared_host_scope.evaluate_url_candidate(
            "https://Example.com/products?id=7#frag",
            descriptor,
        )
        self.assertTrue(decision.allowed)
        self.assertEqual(
            decision.normalized_url,
            "https://example.com/products?id=7",
        )
        self.assertIsNone(decision.rejection_reason)

    def test_rejects_relative_candidate_without_host(self):
        descriptor = self.make_descriptor()
        decision = shared_host_scope.evaluate_url_candidate("/pricing", descriptor)
        self.assertFalse(decision.allowed)
        self.assertEqual(decision.rejection_reason, "missing_host")

    def test_rejects_ip_literal_hosts_when_enabled(self):
        descriptor = self.make_descriptor()
        decision = shared_host_scope.evaluate_url_candidate(
            "https://203.0.113.10/landing",
            descriptor,
        )
        self.assertFalse(decision.allowed)
        self.assertEqual(decision.rejection_reason, "ip_literal_host")

    def test_rejects_non_http_scheme_even_when_https_requirement_is_disabled(self):
        descriptor = self.make_descriptor(require_https=False)
        decision = shared_host_scope.evaluate_url_candidate(
            "ftp://example.com/archive",
            descriptor,
        )
        self.assertFalse(decision.allowed)
        self.assertEqual(decision.rejection_reason, "malformed_url")

    def test_rejects_invalid_authority_port_as_malformed(self):
        descriptor = self.make_descriptor()
        decision = shared_host_scope.evaluate_url_candidate(
            "https://example.com:bad/catalog",
            descriptor,
        )
        self.assertFalse(decision.allowed)
        self.assertEqual(decision.rejection_reason, "malformed_url")

    def test_rejects_denied_path_prefix_on_boundary_only(self):
        descriptor = self.make_descriptor(denied_path_prefixes=["/admin"])
        blocked = shared_host_scope.evaluate_url_candidate(
            "https://example.com/admin/tools",
            descriptor,
        )
        allowed = shared_host_scope.evaluate_url_candidate(
            "https://example.com/administrator",
            descriptor,
        )
        self.assertFalse(blocked.allowed)
        self.assertEqual(blocked.rejection_reason, "denied_path_prefix")
        self.assertTrue(allowed.allowed)

    def test_redirect_revalidation_rejects_cross_host_escape(self):
        descriptor = self.make_descriptor()
        decision = shared_host_scope.evaluate_redirect_target(
            "https://example.com/start",
            "//evil.example.net/escape",
            descriptor,
        )
        self.assertFalse(decision.allowed)
        self.assertEqual(decision.rejection_reason, "redirect_target_out_of_scope")

    def test_redirect_revalidation_accepts_relative_target(self):
        descriptor = self.make_descriptor()
        decision = shared_host_scope.evaluate_redirect_target(
            "https://example.com/start",
            "/catalog/next?page=2",
            descriptor,
        )
        self.assertTrue(decision.allowed)
        self.assertEqual(
            decision.normalized_url,
            "https://example.com/catalog/next?page=2",
        )

    def test_descriptor_rejects_invalid_allowed_host_entry(self):
        with self.assertRaises(shared_host_scope.SharedHostScopeError):
            self.make_descriptor(allowed_hosts=["https://example.com"])


if __name__ == "__main__":
    unittest.main()
