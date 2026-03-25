import json
import tempfile
import unittest
from pathlib import Path

from scripts.deploy import scrapling_deploy_prep as prep


class ScraplingDeployPrepTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="scrapling-deploy-prep-"))

    def test_default_outputs_use_root_only_seed_and_fail_closed_scope(self) -> None:
        receipt_path = self.temp_dir / "receipt.json"
        scope_path = self.temp_dir / "scope.json"
        seed_path = self.temp_dir / "seed.json"

        receipt = prep.prepare_scrapling_deploy(
            public_base_url="https://shuma.example.com/dashboard/index.html",
            runtime_mode="ssh_systemd",
            receipt_output=receipt_path,
            scope_output=scope_path,
            seed_output=seed_path,
        )

        self.assertEqual(receipt["schema"], "shuma.scrapling.deploy_prep.v1")
        self.assertEqual(receipt["runtime_mode"], "ssh_systemd")
        self.assertEqual(receipt["public_base_url"], "https://shuma.example.com/")
        self.assertEqual(receipt["scope"]["allowed_hosts"], ["shuma.example.com"])
        self.assertEqual(receipt["seed"]["primary_start_url"], "https://shuma.example.com/")
        self.assertEqual(receipt["seed"]["extra_seed_urls"], [])
        self.assertFalse(receipt["seed"]["robots_fetch_enabled"])

        scope_payload = json.loads(scope_path.read_text(encoding="utf-8"))
        self.assertEqual(scope_payload["schema_version"], "shared-host-scope-contract.v1")
        self.assertEqual(scope_payload["allowed_hosts"], ["shuma.example.com"])
        self.assertTrue(scope_payload["require_https"])
        self.assertTrue(scope_payload["deny_ip_literals"])

        seed_payload = json.loads(seed_path.read_text(encoding="utf-8"))
        self.assertEqual(seed_payload["schema_version"], "shared-host-seed-contract.v1")
        self.assertEqual(seed_payload["primary_start_url"], "https://shuma.example.com/")
        self.assertEqual(
            seed_payload["accepted_start_urls"],
            [{"sources": ["primary_start_url"], "url": "https://shuma.example.com/"}],
        )
        self.assertEqual(seed_payload["accepted_hint_documents"], [])
        self.assertEqual(seed_payload["rejected_inputs"], [])

        self.assertEqual(
            receipt["environment"]["remote"]["ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH"],
            "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-scope.json",
        )
        self.assertEqual(
            receipt["environment"]["remote"]["ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH"],
            "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-seed-inventory.json",
        )
        self.assertEqual(
            receipt["environment"]["remote"]["ADVERSARY_SIM_SCRAPLING_CRAWLDIR"],
            "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-crawldir",
        )
        self.assertIn("make test-shared-host-scope-contract", receipt["verification"]["commands"])
        self.assertIn("make test-shared-host-seed-contract", receipt["verification"]["commands"])

    def test_explicit_extra_seed_requires_same_host_scope(self) -> None:
        receipt = prep.prepare_scrapling_deploy(
            public_base_url="https://blog.example.com/",
            runtime_mode="ssh_systemd",
            receipt_output=self.temp_dir / "receipt.json",
            scope_output=self.temp_dir / "scope.json",
            seed_output=self.temp_dir / "seed.json",
            extra_seed_urls=[
                "https://blog.example.com/pricing",
                "https://www.example.com/other",
            ],
        )

        seed_payload = json.loads((self.temp_dir / "seed.json").read_text(encoding="utf-8"))
        self.assertEqual(
            seed_payload["accepted_start_urls"],
            [
                {"sources": ["primary_start_url"], "url": "https://blog.example.com/"},
                {"sources": ["manual_extra_seed"], "url": "https://blog.example.com/pricing"},
            ],
        )
        self.assertEqual(
            seed_payload["rejected_inputs"],
            [
                {
                    "raw_value": "https://www.example.com/other",
                    "reason": "host_not_allowed",
                    "source": "manual_extra_seed",
                }
            ],
        )
        self.assertEqual(receipt["scope"]["allowed_hosts"], ["blog.example.com"])

    def test_external_supervisor_mode_marks_shared_host_as_unsupported_target(self) -> None:
        receipt = prep.prepare_scrapling_deploy(
            public_base_url="https://edge.example.com/",
            runtime_mode="external_supervisor",
            receipt_output=self.temp_dir / "receipt.json",
            scope_output=self.temp_dir / "scope.json",
            seed_output=self.temp_dir / "seed.json",
        )

        self.assertEqual(receipt["runtime_mode"], "external_supervisor")
        self.assertEqual(receipt["support_tier"], "deferred_edge_runtime")
        self.assertIn("shared-host-first", receipt["notes"][0])
        self.assertEqual(
            receipt["egress"]["required_outbound_hosts"],
            ["https://edge.example.com:443"],
        )

    def test_local_http_mode_supports_loopback_safe_localhost_seed(self) -> None:
        receipt = prep.prepare_scrapling_deploy(
            public_base_url="http://localhost:3000/dashboard/index.html",
            runtime_mode="ssh_systemd",
            receipt_output=self.temp_dir / "receipt.json",
            scope_output=self.temp_dir / "scope.json",
            seed_output=self.temp_dir / "seed.json",
            require_https=False,
        )

        scope_payload = json.loads((self.temp_dir / "scope.json").read_text(encoding="utf-8"))
        seed_payload = json.loads((self.temp_dir / "seed.json").read_text(encoding="utf-8"))

        self.assertEqual(receipt["public_base_url"], "http://localhost:3000/")
        self.assertFalse(scope_payload["require_https"])
        self.assertTrue(scope_payload["deny_ip_literals"])
        self.assertEqual(scope_payload["allowed_hosts"], ["localhost"])
        self.assertEqual(seed_payload["primary_start_url"], "http://localhost:3000/")
        self.assertEqual(
            receipt["egress"]["required_outbound_hosts"],
            ["http://localhost:3000"],
        )


if __name__ == "__main__":
    unittest.main()
