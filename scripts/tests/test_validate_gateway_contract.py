import os
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "deploy" / "validate_gateway_contract.py"


def run_guardrail(manifest_text: str, overrides: dict[str, str]) -> subprocess.CompletedProcess:
    with tempfile.NamedTemporaryFile("w", delete=False, suffix=".toml") as handle:
        handle.write(manifest_text)
        manifest_path = Path(handle.name)
    env = os.environ.copy()
    env.update(
        {
            "SHUMA_SPIN_MANIFEST": str(manifest_path),
            "SHUMA_RUNTIME_ENV": "runtime-prod",
            "SHUMA_GATEWAY_DEPLOYMENT_PROFILE": "shared-server",
            "SHUMA_GATEWAY_UPSTREAM_ORIGIN": "https://origin.example.com",
        }
    )
    env.update(overrides)
    try:
        return subprocess.run(
            ["python3", str(SCRIPT)],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
    finally:
        manifest_path.unlink(missing_ok=True)


class ValidateGatewayContractTests(unittest.TestCase):
    def test_passes_when_upstream_matches_outbound_allowlist(self) -> None:
        result = run_guardrail(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://origin.example.com:443"]
                """
            ),
            {},
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

    def test_rejects_missing_upstream_origin_in_runtime_prod(self) -> None:
        result = run_guardrail(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://origin.example.com:443"]
                """
            ),
            {"SHUMA_GATEWAY_UPSTREAM_ORIGIN": ""},
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Invalid SHUMA_GATEWAY_UPSTREAM_ORIGIN", result.stderr)

    def test_rejects_wildcard_allowed_outbound_hosts_in_runtime_prod(self) -> None:
        result = run_guardrail(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://origin.example.com:443", "https://*"]
                """
            ),
            {},
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Wildcard entries", result.stderr)

    def test_rejects_variable_templated_outbound_hosts_for_edge_profile(self) -> None:
        result = run_guardrail(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://${UPSTREAM_HOST}:443"]
                """
            ),
            {
                "SHUMA_GATEWAY_DEPLOYMENT_PROFILE": "edge-fermyon",
                "SHUMA_GATEWAY_UPSTREAM_ORIGIN": "https://edge-origin.example.com",
            },
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Variable-templated", result.stderr)

    def test_edge_profile_accepts_explicit_https_origin_and_allowlist_entry(self) -> None:
        result = run_guardrail(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://edge-origin.example.com:443"]
                """
            ),
            {
                "SHUMA_GATEWAY_DEPLOYMENT_PROFILE": "edge-fermyon",
                "SHUMA_GATEWAY_UPSTREAM_ORIGIN": "https://edge-origin.example.com",
            },
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)


if __name__ == "__main__":
    unittest.main()
