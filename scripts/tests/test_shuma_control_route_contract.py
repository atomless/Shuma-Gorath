#!/usr/bin/env python3

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

ACTIVE_SHUMA_ROUTE_SURFACES = {
    Path("src/runtime/request_router.rs"): [
        "is_shuma_dashboard_root_path(path)",
        "SHUMA_DASHBOARD_INDEX_PATH",
        "SHUMA_HEALTH_PATH",
        "SHUMA_METRICS_PATH",
        "is_shuma_admin_path(path)",
    ],
    Path("src/runtime/request_router/tests.rs"): [
        "/shuma/dashboard",
        "/shuma/dashboard/index.html",
        "/shuma/health",
        "/shuma/admin/config",
    ],
    Path("dashboard/src/lib/runtime/dashboard-paths.js"): [
        "/shuma/dashboard",
    ],
    Path("dashboard/src/routes/login.html/+page.svelte"): [
        "/shuma/admin/session",
        "/shuma/admin/login",
        "dashboardBasePath",
    ],
    Path("dashboard/src/lib/runtime/dashboard-native-runtime.js"): [
        "/shuma/admin/session",
        "/shuma/admin/logout",
    ],
    Path("dashboard/src/lib/domain/api-client.js"): [
        "/shuma/admin/config",
        "/shuma/admin/adversary-sim/status",
        "/shuma/dashboard/login.html",
    ],
    Path("dashboard/svelte.config.js"): [
        "base: '/shuma/dashboard'",
    ],
    Path("scripts/deploy/gateway_surface_catalog.py"): [
        'pattern="/shuma/dashboard"',
        'pattern="/shuma/health"',
        'pattern="/shuma/metrics"',
        'pattern="/shuma/admin"',
    ],
    Path("scripts/run_with_adversary_sim_supervisor.sh"): [
        "/shuma/admin/adversary-sim/status",
    ],
    Path("scripts/deploy/remote_target.py"): [
        "http://127.0.0.1:3000/shuma/health",
    ],
    Path("scripts/deploy_linode_one_shot.sh"): [
        '${BASE_URL}/shuma/dashboard',
        '${BASE_URL}/shuma/health',
    ],
    Path("scripts/tests/wait_for_spin_ready.sh"): [
        "/shuma/health",
    ],
    Path("scripts/tests/dashboard_external_live_smoke.mjs"): [
        "/shuma/dashboard/login.html?next=%2Fshuma%2Fdashboard%2Findex.html",
    ],
    Path("e2e/dashboard.smoke.spec.js"): [
        "${BASE_URL}/shuma/dashboard",
        "/shuma/admin/config",
        "/shuma/metrics",
    ],
    Path("src/admin/api.rs"): [
        "route_namespace::SHUMA_METRICS_PATH",
        "%2Fshuma%2Fdashboard%2Findex.html",
    ],
    Path("src/request_validation.rs"): [
        "%2Fshuma%2Fdashboard%2Findex.html%23status",
    ],
    Path("src/challenge/not_a_bot/render.rs"): [
        "next=/shuma/dashboard",
    ],
}

LEGACY_ROUTE_DRIFT_SURFACES = {
    Path("scripts/run_with_adversary_sim_supervisor.sh"): [
        r"(?<!/shuma)/admin/adversary-sim/status",
    ],
    Path("scripts/deploy/remote_target.py"): [
        r"http://127\.0\.0\.1:3000/health(?![A-Za-z0-9_-])",
    ],
    Path("scripts/deploy_linode_one_shot.sh"): [
        r"\$\{BASE_URL\}/dashboard(?![A-Za-z0-9_-])",
        r"\$\{BASE_URL\}/health(?![A-Za-z0-9_-])",
    ],
    Path("scripts/tests/wait_for_spin_ready.sh"): [
        r"\$\{BASE_URL\}/health(?![A-Za-z0-9_-])",
    ],
    Path("scripts/tests/dashboard_external_live_smoke.mjs"): [
        r"(?<!%2Fshuma)%2Fdashboard%2Findex\.html",
    ],
    Path("e2e/dashboard.smoke.spec.js"): [
        r"\$\{BASE_URL\}/dashboard(?![A-Za-z0-9_-])",
        r"(?<!%2Fshuma)%2Fdashboard%2Findex\.html",
        r"\$\{BASE_URL\}/metrics(?![A-Za-z0-9_-])",
    ],
    Path("src/admin/api.rs"): [
        r'"/metrics"',
        r"fetch\('/metrics'\)",
        r"(?<!%2Fshuma)%2Fdashboard%2Findex\.html",
    ],
    Path("src/request_validation.rs"): [
        r"(?<!%2Fshuma)%2Fdashboard%2Findex\.html",
    ],
    Path("src/challenge/not_a_bot/render.rs"): [
        r"next=/dashboard(?![A-Za-z0-9_-])",
    ],
}


class ShumaControlRouteContractTests(unittest.TestCase):
    def test_active_surfaces_advertise_shuma_control_namespace(self) -> None:
        missing_markers = {}
        for relative_path, expected_markers in ACTIVE_SHUMA_ROUTE_SURFACES.items():
            content = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            missing = [marker for marker in expected_markers if marker not in content]
            if missing:
                missing_markers[relative_path.as_posix()] = missing
        self.assertEqual(missing_markers, {})

    def test_active_surfaces_do_not_keep_legacy_top_level_control_routes(self) -> None:
        stale_markers = {}
        for relative_path, forbidden_patterns in LEGACY_ROUTE_DRIFT_SURFACES.items():
            content = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            present = [
                pattern for pattern in forbidden_patterns if re.search(pattern, content) is not None
            ]
            if present:
                stale_markers[relative_path.as_posix()] = present
        self.assertEqual(stale_markers, {})


if __name__ == "__main__":
    unittest.main()
