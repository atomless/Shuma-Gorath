import json
import os
import subprocess
import tempfile
import shutil
import unittest
from pathlib import Path
from typing import Dict, Optional


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "deploy" / "validate_gateway_route_collisions.py"


def run_guardrail(
    catalog_payload: Optional[object],
    overrides: Optional[Dict[str, str]] = None,
    *,
    cwd: Optional[Path] = None,
) -> subprocess.CompletedProcess:
    env = os.environ.copy()
    env.update(
        {
            "SHUMA_RUNTIME_ENV": "runtime-prod",
            "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED": "true",
        }
    )
    if overrides:
        env.update(overrides)

    temp_dir = Path(tempfile.mkdtemp(prefix="gateway-route-collision-test-"))
    command_cwd = cwd or REPO_ROOT
    report_path = temp_dir / "report.json"
    env.setdefault("GATEWAY_ROUTE_COLLISION_REPORT_PATH", str(report_path))
    catalog_path = temp_dir / "catalog.json"

    if catalog_payload is not None:
        catalog_path.write_text(json.dumps(catalog_payload), encoding="utf-8")
        env.setdefault("GATEWAY_SURFACE_CATALOG_PATH", str(catalog_path))

    try:
        return subprocess.run(
            ["python3", str(SCRIPT)],
            cwd=str(command_cwd),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)


class ValidateGatewayRouteCollisionsTests(unittest.TestCase):
    def test_passes_when_catalog_has_no_reserved_route_collisions(self) -> None:
        result = run_guardrail(
            {
                "inventory": [
                    {"path": "/"},
                    {"path": "/products"},
                    {"url": "https://origin.example.com/blog/post-1"},
                ]
            }
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("preflight passed", result.stdout.lower())

    def test_fails_when_catalog_collides_with_reserved_routes(self) -> None:
        result = run_guardrail(
            {
                "paths": [
                    "/admin/login",
                    "/products",
                ]
            }
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("collision preflight failed", result.stderr.lower())

    def test_fails_when_catalog_path_is_missing_in_runtime_prod(self) -> None:
        result = run_guardrail(catalog_payload=None)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing gateway_surface_catalog_path", result.stderr.lower())

    def test_default_report_path_writes_under_spin_deploy(self) -> None:
        command_cwd = Path(tempfile.mkdtemp(prefix="gateway-route-collision-cwd-"))
        try:
            result = run_guardrail(
                {"inventory": [{"path": "/"}]},
                overrides={"GATEWAY_ROUTE_COLLISION_REPORT_PATH": ""},
                cwd=command_cwd,
            )
            self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
            expected_report = (
                command_cwd / ".spin" / "deploy" / "gateway_reserved_route_collision_report.json"
            )
            self.assertTrue(expected_report.exists())
            payload = json.loads(expected_report.read_text(encoding="utf-8"))
            self.assertEqual(Path(payload["report_path"]).resolve(), expected_report.resolve())
            self.assertTrue(payload["passed"])
        finally:
            shutil.rmtree(command_cwd, ignore_errors=True)

    def test_skips_in_non_prod_runtime(self) -> None:
        result = run_guardrail(
            catalog_payload=None,
            overrides={"SHUMA_RUNTIME_ENV": "runtime-dev"},
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("skipped", result.stdout.lower())


if __name__ == "__main__":
    unittest.main()
