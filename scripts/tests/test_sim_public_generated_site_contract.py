import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"
SPIN_TOML = REPO_ROOT / "spin.toml"


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


class SimPublicGeneratedSiteContractTests(unittest.TestCase):
    def test_contract_target_exists_and_is_focused(self) -> None:
        body = target_body("test-sim-public-generated-site-contract")
        self.assertIn(
            "cargo test sim_public_site_root_defaults_under_shuma_local_state_dir -- --nocapture",
            body,
        )
        self.assertIn(
            "cargo test availability_from_runtime_uses_generated_artifact_presence_not_sim_controls -- --nocapture",
            body,
        )
        self.assertIn(
            "python3 -m unittest scripts/tests/test_sim_public_generated_site_contract.py",
            body,
        )

    def test_canonical_generator_and_config_locations_exist(self) -> None:
        self.assertTrue((REPO_ROOT / "scripts" / "build_sim_public_site.py").exists())
        self.assertTrue((REPO_ROOT / "scripts" / "sim_public_site").is_dir())
        self.assertTrue((REPO_ROOT / "config" / "sim_public_site" / "corpus.toml").is_file())

    def test_repo_corpus_config_freezes_root_hosted_public_prefix(self) -> None:
        corpus = (REPO_ROOT / "config" / "sim_public_site" / "corpus.toml").read_text(
            encoding="utf-8"
        )
        self.assertIn('root_prefix = "/"', corpus)

    def test_gateway_surface_catalog_no_longer_reserves_legacy_sim_public_prefix(self) -> None:
        gateway_catalog = (
            REPO_ROOT / "scripts" / "deploy" / "gateway_surface_catalog.py"
        ).read_text(encoding="utf-8")
        self.assertNotIn('pattern="/sim/public"', gateway_catalog)

    def test_bot_defence_component_mounts_generated_public_site_artifact(self) -> None:
        spin_toml = SPIN_TOML.read_text(encoding="utf-8")
        self.assertIn("[component.bot-defence]", spin_toml)
        self.assertIn(
            'files = [{ source = ".shuma/sim-public-site", destination = ".shuma/sim-public-site" }]',
            spin_toml,
        )


if __name__ == "__main__":
    unittest.main()
