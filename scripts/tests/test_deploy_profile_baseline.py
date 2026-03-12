import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


class DeployProfileBaselineTests(unittest.TestCase):
    def test_deploy_profile_baseline_builds_dashboard_and_runtime(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        self.assertIn("deploy-profile-baseline: ## Profile wrapper baseline: verify seeded config + dashboard/runtime build", source)
        self.assertIn('@$(MAKE) --no-print-directory dashboard-build >/dev/null', source)
        self.assertIn('@$(MAKE) --no-print-directory build-runtime', source)

    def test_fermyon_rendered_manifest_lives_at_repo_root(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        self.assertIn("FERMYON_AKAMAI_RENDERED_MANIFEST ?= spin.fermyon-akamai-edge.toml", source)
        self.assertNotIn("FERMYON_AKAMAI_RENDERED_MANIFEST ?= $(SHUMA_LOCAL_STATE_DIR)/manifests/fermyon-akamai-edge.spin.toml", source)


if __name__ == "__main__":
    unittest.main()
