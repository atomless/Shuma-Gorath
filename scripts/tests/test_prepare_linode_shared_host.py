import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.deploy import linode_shared_host_setup as setup


class FakeLinodeClient:
    def __init__(self, token: str, base_url: str = setup.DEFAULT_LINODE_API_URL) -> None:
        self.token = token
        self.base_url = base_url
        self.calls: list[tuple[str, object]] = []

    def validate_token(self) -> dict[str, object]:
        self.calls.append(("validate_token", None))
        return {"username": "jamestindall"}

    def create_instance(
        self,
        *,
        label: str,
        region: str,
        linode_type: str,
        image: str,
        ssh_public_key: str,
    ) -> dict[str, object]:
        self.calls.append(
            (
                "create_instance",
                {
                    "label": label,
                    "region": region,
                    "linode_type": linode_type,
                    "image": image,
                    "ssh_public_key": ssh_public_key,
                },
            )
        )
        return {"id": 123}

    def wait_for_instance_running(
        self, instance_id: int, *, attempts: int = 90, poll_interval_seconds: int = 2
    ) -> dict[str, object]:
        self.calls.append(("wait_for_instance_running", instance_id))
        return {
            "id": instance_id,
            "label": "shuma-test",
            "status": "running",
            "ipv4": ["198.51.100.24"],
            "region": "us-east",
            "type": "g6-nanode-1",
            "image": "linode/ubuntu24.04",
        }

    def get_instance(self, instance_id: int) -> dict[str, object]:
        self.calls.append(("get_instance", instance_id))
        return {
            "id": instance_id,
            "label": "existing-shuma",
            "status": "running",
            "ipv4": ["198.51.100.25"],
            "region": "us-east",
            "type": "g6-nanode-1",
            "image": "linode/ubuntu24.04",
        }


class PrepareLinodeSharedHostTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="prepare-linode-"))
        self.docroot = self.temp_dir / "dummy_static_site"
        self.docroot.mkdir()
        (self.docroot / "index.html").write_text("<h1>Hello</h1>\n", encoding="utf-8")
        (self.docroot / "about.html").write_text("<p>About</p>\n", encoding="utf-8")
        self.env_file = self.temp_dir / ".env.local"
        self.catalog_path = self.temp_dir / ".spin" / "site.surface-catalog.json"
        self.receipt_path = self.temp_dir / ".spin" / "linode-shared-host-setup.json"

    def test_prompts_for_token_persists_env_and_writes_receipt(self) -> None:
        client = FakeLinodeClient("linode-secret")

        with patch.object(setup, "LinodeApiClient", return_value=client), patch.object(
            setup, "detect_public_ip", return_value="203.0.113.8"
        ), patch.object(
            setup,
            "ensure_ssh_keypair",
            return_value=(
                Path("/Users/test/.ssh/shuma-linode"),
                Path("/Users/test/.ssh/shuma-linode.pub"),
                "ssh-ed25519 AAAATEST shuma-linode",
            ),
        ), patch.object(
            setup, "is_interactive_session", return_value=True
        ), patch.object(
            setup, "prompt_secret", return_value="linode-secret"
        ), patch.object(
            setup, "prompt_confirm", return_value=True
        ):
            rc = setup.main(
                [
                    "--docroot",
                    str(self.docroot),
                    "--env-file",
                    str(self.env_file),
                    "--catalog-output",
                    str(self.catalog_path),
                    "--receipt-output",
                    str(self.receipt_path),
                    "--label",
                    "shuma-test",
                ]
            )

        self.assertEqual(rc, 0)
        env_local = self.env_file.read_text(encoding="utf-8")
        self.assertIn("LINODE_TOKEN=linode-secret", env_local)
        self.assertIn("SHUMA_ADMIN_IP_ALLOWLIST=203.0.113.8/32", env_local)
        self.assertIn(
            f"GATEWAY_SURFACE_CATALOG_PATH={self.catalog_path.resolve()}",
            env_local,
        )

        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["mode"], "fresh-instance")
        self.assertEqual(receipt["linode"]["instance_id"], 123)
        self.assertEqual(receipt["linode"]["public_ipv4"], "198.51.100.24")
        self.assertEqual(receipt["admin_allowlist"], "203.0.113.8/32")
        self.assertEqual(receipt["catalog_path"], str(self.catalog_path.resolve()))

    def test_existing_instance_uses_saved_token_without_prompt(self) -> None:
        self.env_file.write_text("LINODE_TOKEN=stored-token\n", encoding="utf-8")
        client = FakeLinodeClient("stored-token")

        with patch.object(setup, "LinodeApiClient", return_value=client), patch.object(
            setup,
            "ensure_ssh_keypair",
            return_value=(
                Path("/Users/test/.ssh/shuma-linode"),
                Path("/Users/test/.ssh/shuma-linode.pub"),
                "ssh-ed25519 AAAATEST shuma-linode",
            ),
        ), patch.object(
            setup, "is_interactive_session", return_value=False
        ), patch.object(
            setup, "prompt_secret"
        ) as prompt_secret:
            rc = setup.main(
                [
                    "--docroot",
                    str(self.docroot),
                    "--env-file",
                    str(self.env_file),
                    "--catalog-output",
                    str(self.catalog_path),
                    "--receipt-output",
                    str(self.receipt_path),
                    "--existing-instance-id",
                    "456",
                    "--admin-ip",
                    "198.51.100.9/32",
                ]
            )

        self.assertEqual(rc, 0)
        prompt_secret.assert_not_called()
        self.assertIn(("get_instance", 456), client.calls)
        self.assertFalse(any(name == "create_instance" for name, _ in client.calls))
        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["mode"], "existing-instance")
        self.assertEqual(receipt["linode"]["instance_id"], 456)
        self.assertEqual(receipt["admin_allowlist"], "198.51.100.9/32")

    def test_make_target_passes_env_local_deploy_inputs_to_script(self) -> None:
        self.catalog_path.parent.mkdir(parents=True, exist_ok=True)
        self.catalog_path.write_text('{"inventory":[{"path":"/"}]}\n', encoding="utf-8")
        self.env_file.write_text(
            "\n".join(
                [
                    "LINODE_TOKEN=stored-token",
                    "SHUMA_API_KEY=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "SHUMA_JS_SECRET=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    "SHUMA_FORWARDED_IP_SECRET=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                    "SHUMA_HEALTH_SECRET=dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
                    "SHUMA_SIM_TELEMETRY_SECRET=eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                    "SHUMA_ADMIN_IP_ALLOWLIST=203.0.113.10/32",
                    "SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://origin.example.com",
                    "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true",
                    "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true",
                    "SHUMA_GATEWAY_TLS_STRICT=true",
                    "SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true",
                    "SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true",
                    f"GATEWAY_SURFACE_CATALOG_PATH={self.catalog_path}",
                    "",
                ]
            ),
            encoding="utf-8",
        )

        result = subprocess.run(
            [
                "make",
                "-n",
                f"ENV_LOCAL={self.env_file}",
                'DEPLOY_LINODE_ARGS=--domain shuma.example.com --preflight-only',
                "deploy-linode-one-shot",
            ],
            cwd=setup.REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn('LINODE_TOKEN="stored-token"', result.stdout)
        self.assertIn(
            'SHUMA_API_KEY="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"',
            result.stdout,
        )
        self.assertIn(
            'SHUMA_JS_SECRET="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"',
            result.stdout,
        )
        self.assertIn(
            'SHUMA_FORWARDED_IP_SECRET="cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"',
            result.stdout,
        )
        self.assertIn(
            'SHUMA_HEALTH_SECRET="dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"',
            result.stdout,
        )
        self.assertIn(
            'SHUMA_SIM_TELEMETRY_SECRET="eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"',
            result.stdout,
        )
        self.assertIn('SHUMA_ADMIN_IP_ALLOWLIST="203.0.113.10/32"', result.stdout)
        self.assertIn(f'GATEWAY_SURFACE_CATALOG_PATH="{self.catalog_path}"', result.stdout)
        self.assertIn("./scripts/deploy_linode_one_shot.sh", result.stdout)

    def test_make_target_uses_receipt_ssh_paths_when_env_is_blank(self) -> None:
        self.catalog_path.parent.mkdir(parents=True, exist_ok=True)
        self.catalog_path.write_text('{"inventory":[{"path":"/"}]}\n', encoding="utf-8")
        receipt_path = self.temp_dir / ".spin" / "linode-shared-host-setup.json"
        receipt_path.parent.mkdir(parents=True, exist_ok=True)
        receipt_path.write_text(
            json.dumps(
                {
                    "ssh": {
                        "private_key_path": "/Users/test/.ssh/shuma-linode",
                        "public_key_path": "/Users/test/.ssh/shuma-linode.pub",
                    }
                }
            )
            + "\n",
            encoding="utf-8",
        )
        self.env_file.write_text(
            "\n".join(
                [
                    "LINODE_TOKEN=stored-token",
                    "SHUMA_ADMIN_IP_ALLOWLIST=203.0.113.10/32",
                    "SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://origin.example.com",
                    "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true",
                    "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true",
                    "SHUMA_GATEWAY_TLS_STRICT=true",
                    "SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true",
                    "SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true",
                    f"GATEWAY_SURFACE_CATALOG_PATH={self.catalog_path}",
                    "",
                ]
            ),
            encoding="utf-8",
        )

        result = subprocess.run(
            [
                "make",
                "-n",
                f"ENV_LOCAL={self.env_file}",
                f"LINODE_SETUP_RECEIPT={receipt_path}",
                'DEPLOY_LINODE_ARGS=--domain shuma.example.com --preflight-only',
                "deploy-linode-one-shot",
            ],
            cwd=setup.REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn('SSH_PRIVATE_KEY_FILE="/Users/test/.ssh/shuma-linode"', result.stdout)
        self.assertIn('SSH_PUBLIC_KEY_FILE="/Users/test/.ssh/shuma-linode.pub"', result.stdout)

    def test_cli_entrypoint_help_runs(self) -> None:
        result = subprocess.run(
            ["python3", "scripts/prepare_linode_shared_host.py", "--help"],
            cwd=setup.REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("Prepare a Linode shared host", result.stdout)


if __name__ == "__main__":
    unittest.main()
