import tempfile
import unittest
from pathlib import Path

from scripts.deploy import merge_env_overlay


class MergeEnvOverlayTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="merge-env-overlay-"))

    def test_explicit_updates_override_overlay_and_existing_values(self) -> None:
        env_path = self.temp_dir / ".env.local"
        overlay_path = self.temp_dir / "overlay.env"
        env_path.write_text(
            "SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://foreign.example.com\nSHUMA_ADMIN_IP_ALLOWLIST=203.0.113.10/32\n",
            encoding="utf-8",
        )
        overlay_path.write_text(
            "SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://still-foreign.example.com\nSHUMA_API_KEY=remote-api-key\n",
            encoding="utf-8",
        )

        merge_env_overlay.merge_env_overlay(
            overlay_path,
            env_path,
            explicit_updates={"SHUMA_GATEWAY_UPSTREAM_ORIGIN": "http://127.0.0.1:8080"},
        )

        rendered = env_path.read_text(encoding="utf-8")
        self.assertIn("SHUMA_GATEWAY_UPSTREAM_ORIGIN=http://127.0.0.1:8080\n", rendered)
        self.assertIn("SHUMA_API_KEY=remote-api-key\n", rendered)
        self.assertIn("SHUMA_ADMIN_IP_ALLOWLIST=203.0.113.10/32\n", rendered)


if __name__ == "__main__":
    unittest.main()
