import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


def target_body(name: str) -> str:
    source = MAKEFILE.read_text(encoding="utf-8")
    match = re.search(
        rf"^{re.escape(name)}:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
        source,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        raise AssertionError(f"target {name} not found")
    return match.group(0)


class ShumaRouteNamespaceContractTests(unittest.TestCase):
    def test_contract_target_exists_and_is_focused(self) -> None:
        body = target_body("test-shuma-route-namespace-contract")
        self.assertIn(
            "cargo test canonical_shuma_owned_routes_live_under_shuma_prefix -- --nocapture",
            body,
        )
        self.assertIn(
            "cargo test shuma_route_matchers_are_boundary_aware -- --nocapture",
            body,
        )
        self.assertIn(
            "cargo test static_bypass_preserves_route_namespace_public_and_shuma_control_paths -- --nocapture",
            body,
        )
        self.assertIn(
            "python3 -m unittest scripts/tests/test_shuma_route_namespace_contract.py",
            body,
        )

    def test_canonical_namespace_module_exists(self) -> None:
        self.assertTrue((REPO_ROOT / "src" / "http_route_namespace.rs").is_file())

    def test_runtime_routing_imports_route_namespace_module(self) -> None:
        request_router = (REPO_ROOT / "src" / "runtime" / "request_router.rs").read_text(
            encoding="utf-8"
        )
        lib_rs = (REPO_ROOT / "src" / "lib.rs").read_text(encoding="utf-8")
        self.assertIn("http_route_namespace", request_router)
        self.assertIn("http_route_namespace", lib_rs)


if __name__ == "__main__":
    unittest.main()
