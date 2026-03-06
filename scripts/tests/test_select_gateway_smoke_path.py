import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "deploy" / "select_gateway_smoke_path.py"


def run_selector(catalog_text: str) -> subprocess.CompletedProcess:
    temp_dir = Path(tempfile.mkdtemp(prefix="select-gateway-smoke-path-"))
    catalog_path = temp_dir / "catalog.json"
    catalog_path.write_text(catalog_text, encoding="utf-8")
    return subprocess.run(
        ["python3", str(SCRIPT), "--catalog", str(catalog_path)],
        cwd=str(REPO_ROOT),
        capture_output=True,
        text=True,
        check=False,
    )


class SelectGatewaySmokePathTests(unittest.TestCase):
    def test_prefers_non_reserved_text_like_path(self) -> None:
        result = run_selector(
            '{"inventory":[{"path":"/health"},{"path":"/"},{"path":"/assets/app.js"},{"path":"/public/page"}]}\n'
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertEqual(result.stdout.strip(), "/assets/app.js")

    def test_prefers_static_asset_over_html_page(self) -> None:
        result = run_selector(
            '{"inventory":[{"path":"/cv.html"},{"path":"/css/style.css"},{"path":"/about.html"}]}\n'
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertEqual(result.stdout.strip(), "/css/style.css")

    def test_fails_when_catalog_only_contains_reserved_paths(self) -> None:
        result = run_selector('{"inventory":[{"path":"/health"},{"path":"/metrics"}]}\n')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("non-reserved public path", result.stderr)


if __name__ == "__main__":
    unittest.main()
