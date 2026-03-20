#!/usr/bin/env python3

import json
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path

import scripts.tests.check_shared_host_seed_contract as seed_contract_check
import scripts.tests.shared_host_scope as shared_host_scope
import scripts.tests.shared_host_seed_inventory as shared_host_seed_inventory


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "shared_host_seed_inventory.py"


class SharedHostSeedInventoryUnitTests(unittest.TestCase):
    def make_descriptor(self):
        return shared_host_scope.descriptor_from_payload(
            {
                "allowed_hosts": ["example.com"],
                "denied_path_prefixes": ["/private"],
                "require_https": True,
                "deny_ip_literals": True,
            }
        )

    def test_seed_contract_validator_passes(self):
        errors = seed_contract_check.validate_shared_host_seed_contract()
        self.assertEqual(errors, [])

    def test_build_inventory_requires_valid_primary_start_url(self):
        descriptor = self.make_descriptor()
        with self.assertRaises(shared_host_seed_inventory.SharedHostSeedError):
            shared_host_seed_inventory.build_seed_inventory(
                descriptor,
                primary_start_url="http://example.com/",
            )

    def test_build_inventory_merges_primary_and_manual_provenance(self):
        descriptor = self.make_descriptor()
        payload = shared_host_seed_inventory.build_seed_inventory(
            descriptor,
            primary_start_url="https://example.com/",
            extra_seed_urls=[
                "https://example.com/",
                "https://example.com/pricing",
            ],
        )
        self.assertEqual(
            payload["accepted_start_urls"],
            [
                {
                    "url": "https://example.com/",
                    "sources": ["primary_start_url", "manual_extra_seed"],
                },
                {
                    "url": "https://example.com/pricing",
                    "sources": ["manual_extra_seed"],
                },
            ],
        )

    def test_out_of_scope_manual_extra_seed_is_rejected_with_scope_reason(self):
        descriptor = self.make_descriptor()
        payload = shared_host_seed_inventory.build_seed_inventory(
            descriptor,
            primary_start_url="https://example.com/",
            extra_seed_urls=["https://evil.example.net/"],
        )
        self.assertEqual(
            payload["rejected_inputs"],
            [
                {
                    "source": "manual_extra_seed",
                    "raw_value": "https://evil.example.net/",
                    "reason": "host_not_allowed",
                }
            ],
        )

    def test_robots_sitemaps_become_hint_documents_only(self):
        descriptor = self.make_descriptor()
        payload = shared_host_seed_inventory.build_seed_inventory(
            descriptor,
            primary_start_url="https://example.com/",
            robots_text=textwrap.dedent(
                """\
                User-agent: *
                Disallow:
                Sitemap: https://example.com/sitemap.xml
                Sitemap: https://evil.example.net/sitemap.xml
                """
            ),
        )
        self.assertEqual(
            payload["accepted_hint_documents"],
            [
                {
                    "url": "https://example.com/sitemap.xml",
                    "sources": ["robots"],
                }
            ],
        )
        self.assertEqual(
            payload["accepted_start_urls"],
            [
                {
                    "url": "https://example.com/",
                    "sources": ["primary_start_url"],
                }
            ],
        )
        self.assertEqual(
            payload["rejected_inputs"],
            [
                {
                    "source": "robots",
                    "raw_value": "https://evil.example.net/sitemap.xml",
                    "reason": "host_not_allowed",
                }
            ],
        )

    def test_fetch_robots_failure_is_recorded(self):
        descriptor = self.make_descriptor()

        def fail_fetch(_url: str) -> str:
            raise OSError("boom")

        payload = shared_host_seed_inventory.build_seed_inventory(
            descriptor,
            primary_start_url="https://example.com/",
            robots_url="https://example.com/robots.txt",
            robots_fetcher=fail_fetch,
        )
        self.assertEqual(
            payload["rejected_inputs"],
            [
                {
                    "source": "robots",
                    "raw_value": "https://example.com/robots.txt",
                    "reason": "robots_fetch_failed",
                }
            ],
        )

    def test_cli_writes_inventory_from_local_robots_file(self):
        temp_dir = Path(tempfile.mkdtemp(prefix="shared-host-seed-inventory-"))
        descriptor_path = temp_dir / "scope.json"
        descriptor_path.write_text(
            json.dumps(
                {
                    "allowed_hosts": ["example.com"],
                    "denied_path_prefixes": ["/private"],
                    "require_https": True,
                    "deny_ip_literals": True,
                }
            ),
            encoding="utf-8",
        )
        robots_path = temp_dir / "robots.txt"
        robots_path.write_text(
            "Sitemap: https://example.com/sitemap.xml\n",
            encoding="utf-8",
        )
        output_path = temp_dir / "seed-inventory.json"

        result = subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "--scope-descriptor",
                str(descriptor_path),
                "--primary-start-url",
                "https://example.com/",
                "--extra-seed-url",
                "https://example.com/pricing",
                "--robots-file",
                str(robots_path),
                "--output",
                str(output_path),
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        payload = json.loads(output_path.read_text(encoding="utf-8"))
        self.assertEqual(payload["schema_version"], shared_host_seed_inventory.SCHEMA_VERSION)
        self.assertEqual(
            payload["accepted_start_urls"],
            [
                {
                    "sources": ["primary_start_url"],
                    "url": "https://example.com/",
                },
                {
                    "sources": ["manual_extra_seed"],
                    "url": "https://example.com/pricing",
                },
            ],
        )
        self.assertEqual(
            payload["accepted_hint_documents"],
            [
                {
                    "sources": ["robots"],
                    "url": "https://example.com/sitemap.xml",
                }
            ],
        )


if __name__ == "__main__":
    unittest.main()
