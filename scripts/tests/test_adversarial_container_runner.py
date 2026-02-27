#!/usr/bin/env python3

import unittest

import scripts.tests.adversarial_container_runner as container_runner


class AdversarialContainerRunnerUnitTests(unittest.TestCase):
    def test_normalize_container_base_url_rewrites_loopback(self):
        rewritten = container_runner.normalize_container_base_url("http://127.0.0.1:3000")
        self.assertEqual(rewritten, "http://host.docker.internal:3000")

    def test_target_origin_returns_scheme_and_netloc(self):
        origin = container_runner.target_origin("https://example.com:8443/path?q=1")
        self.assertEqual(origin, "https://example.com:8443")

    def test_container_command_includes_hardening_flags(self):
        command = container_runner.container_command(
            image_tag="test:image",
            mode="isolation",
            base_url="http://host.docker.internal:3000",
            allowed_origin="http://host.docker.internal:3000",
            run_id="run-123",
            request_budget=12,
            time_budget_seconds=90,
        )
        joined = " ".join(command)
        self.assertIn("--read-only", joined)
        self.assertIn("--cap-drop=ALL", joined)
        self.assertIn("--security-opt=no-new-privileges", joined)
        self.assertIn("--tmpfs=/tmp:rw,nosuid,nodev,size=64m", joined)
        self.assertIn("--add-host=host.docker.internal:host-gateway", joined)


if __name__ == "__main__":
    unittest.main()
