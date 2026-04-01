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


class SimPublicBuildFlowContractTests(unittest.TestCase):
    def test_refresh_targets_exist_and_use_generator_entrypoint(self) -> None:
        refresh_body = target_body("sim-public-refresh")
        stale_body = target_body("sim-public-refresh-if-stale")
        warn_body = target_body("sim-public-warn-if-stale")
        self.assertIn("$(SIM_PUBLIC_SITE_GENERATOR)", refresh_body)
        self.assertIn("$(SIM_PUBLIC_SITE_GENERATOR)", stale_body)
        self.assertIn("$(SIM_PUBLIC_SITE_GENERATOR)", warn_body)
        self.assertIn("--artifact-root", refresh_body)
        self.assertIn("--if-stale-hours", stale_body)
        self.assertIn("--check-stale-hours", warn_body)

    def test_local_runtime_flows_warn_when_sim_public_is_stale_without_regenerating(self) -> None:
        dev_body = target_body("dev")
        dev_closed_body = target_body("dev-closed")
        run_body = target_body("run")
        run_prebuilt_body = target_body("run-prebuilt")
        for body in (dev_body, dev_closed_body, run_body, run_prebuilt_body):
            self.assertIn("sim-public-warn-if-stale", body)
            self.assertNotIn("sim-public-refresh-if-stale", body)

    def test_build_and_deploy_flows_do_not_generate_contributor_site(self) -> None:
        setup_runtime_body = target_body("setup-runtime")
        prod_start_body = target_body("prod-start")
        run_prebuilt_body = target_body("run-prebuilt")
        build_body = target_body("build")
        deploy_body = target_body("deploy")
        self.assertNotIn("sim-public-refresh", setup_runtime_body)
        self.assertNotIn("sim-public-refresh", run_prebuilt_body)
        self.assertNotIn("sim-public-refresh", build_body)
        self.assertNotIn("sim-public-refresh", prod_start_body)
        self.assertNotIn("sim-public-refresh", deploy_body)

    def test_contract_target_exists_and_is_focused(self) -> None:
        body = target_body("test-sim-public-build-flow-contract")
        self.assertIn(
            "python3 -m unittest scripts/tests/test_sim_public_build_flow_contract.py",
            body,
        )


if __name__ == "__main__":
    unittest.main()
