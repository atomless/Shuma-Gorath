#!/usr/bin/env python3

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

ACTIVE_SURFACES = [
    Path("config/sim_public_site/corpus.toml"),
    Path("scripts/README.md"),
    Path("scripts/build_sim_public_site.py"),
    Path("scripts/deploy/gateway_surface_catalog.py"),
    Path("scripts/sim_public_site/__init__.py"),
    Path("scripts/sim_public_site/build.py"),
    Path("scripts/tests/adversarial/deterministic_attack_corpus.v1.json"),
    Path("scripts/tests/adversarial/frontier_action_contract.v1.json"),
    Path("scripts/tests/adversarial/frontier_attack_generation_contract.v1.json"),
    Path("scripts/tests/test_build_sim_public_site.py"),
    Path("src/admin/adversary_sim_corpus.rs"),
    Path("src/admin/adversary_sim_lane_runtime.rs"),
    Path("src/crawler_policy/robots.rs"),
    Path("src/http_route_namespace.rs"),
    Path("src/lib.rs"),
    Path("src/observability/llm_surface_observation.rs"),
    Path("src/runtime/request_flow.rs"),
    Path("src/runtime/sim_public.rs"),
    Path("dashboard/src/lib/components/dashboard/monitoring-view-model.js"),
    Path("e2e/dashboard.modules.unit.test.js"),
    Path("docs/adversarial-operator-guide.md"),
    Path("docs/configuration.md"),
    Path("docs/testing.md"),
]


class RootedPublicPathContractTests(unittest.TestCase):
    def test_active_surfaces_do_not_advertise_legacy_sim_public_prefix(self) -> None:
        offenders = []
        for relative_path in ACTIVE_SURFACES:
            content = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            if "/sim/public" in content:
                offenders.append(relative_path.as_posix())
        self.assertEqual(offenders, [])

    def test_root_hosted_contract_markers_are_present(self) -> None:
        corpus = (REPO_ROOT / "config" / "sim_public_site" / "corpus.toml").read_text(
            encoding="utf-8"
        )
        runtime = (REPO_ROOT / "src" / "runtime" / "sim_public.rs").read_text(encoding="utf-8")
        route_namespace = (
            REPO_ROOT / "src" / "http_route_namespace.rs"
        ).read_text(encoding="utf-8")

        self.assertIn('root_prefix = "/"', corpus)
        self.assertIn('PUBLIC_ROOT_PATH', route_namespace)
        self.assertIn('PUBLIC_SITEMAP_XML_PATH', route_namespace)
        self.assertNotIn('const SIM_PUBLIC_PREFIX', runtime)


if __name__ == "__main__":
    unittest.main()
