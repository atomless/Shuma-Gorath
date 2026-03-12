import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "deploy" / "render_gateway_spin_manifest.py"


def run_render(manifest_text: str, upstream_origin: str) -> tuple[subprocess.CompletedProcess, Path]:
    temp_dir = Path(tempfile.mkdtemp(prefix="render-gateway-spin-manifest-"))
    manifest_path = temp_dir / "spin.toml"
    output_path = temp_dir / "spin.rendered.toml"
    manifest_path.write_text(manifest_text, encoding="utf-8")
    result = subprocess.run(
        [
            "python3",
            str(SCRIPT),
            "--manifest",
            str(manifest_path),
            "--output",
            str(output_path),
            "--upstream-origin",
            upstream_origin,
        ],
        cwd=str(REPO_ROOT),
        capture_output=True,
        text=True,
        check=False,
    )
    return result, output_path


class RenderGatewaySpinManifestTests(unittest.TestCase):
    def test_adds_normalized_upstream_origin_to_allowed_outbound_hosts(self) -> None:
        result, output_path = run_render(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = ["https://metrics.example.com:443"]
                """
            ),
            "https://origin.example.com",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue(output_path.exists())
        self.assertIn(
            'allowed_outbound_hosts = ["https://metrics.example.com:443", "https://origin.example.com:443"]',
            output_path.read_text(encoding="utf-8"),
        )

    def test_rejects_invalid_upstream_origin(self) -> None:
        result, _ = run_render(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                allowed_outbound_hosts = []
                """
            ),
            "origin.example.com/path",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Invalid upstream origin", result.stderr)

    def test_edge_profile_injects_bot_defence_variable_wiring(self) -> None:
        result, output_path = run_render(
            textwrap.dedent(
                """
                spin_manifest_version = 2
                [component.bot-defence]
                source = "dist/wasm/shuma_gorath.wasm"
                allowed_outbound_hosts = []
                """
            ),
            "https://origin.example.com",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        rendered = output_path.read_text(encoding="utf-8")
        self.assertNotIn('[variables]', rendered)
        self.assertNotIn('SHUMA_API_KEY = "{{ shuma_api_key }}"', rendered)

        edge_result = subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "--manifest",
                str(output_path.parent / "spin.toml"),
                "--output",
                str(output_path),
                "--upstream-origin",
                "https://origin.example.com",
                "--profile",
                "edge-fermyon",
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(edge_result.returncode, 0, msg=edge_result.stderr or edge_result.stdout)
        edge_rendered = output_path.read_text(encoding="utf-8")
        self.assertIn("[variables]", edge_rendered)
        self.assertIn('shuma_api_key = { default = "" }', edge_rendered)
        self.assertIn('shuma_debug_headers = { default = "" }', edge_rendered)
        self.assertIn("[component.bot-defence.variables]", edge_rendered)
        self.assertIn('shuma_api_key = "{{ shuma_api_key }}"', edge_rendered)
        self.assertIn('shuma_debug_headers = "{{ shuma_debug_headers }}"', edge_rendered)
        self.assertNotIn('environment = { SHUMA_API_KEY = "{{ shuma_api_key }}"', edge_rendered)


if __name__ == "__main__":
    unittest.main()
