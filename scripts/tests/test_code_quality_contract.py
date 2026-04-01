import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"
PACKAGE_JSON = REPO_ROOT / "package.json"
CI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
CONTRIBUTING = REPO_ROOT / "CONTRIBUTING.md"
PROJECT_PRINCIPLES = REPO_ROOT / "docs" / "project-principles.md"
TESTING_GUIDE = REPO_ROOT / "docs" / "testing.md"
PR_TEMPLATE = REPO_ROOT / ".github" / "pull_request_template.md"
AGENTS = REPO_ROOT / "AGENTS.md"


def target_body(name: str) -> str:
    source = MAKEFILE.read_text(encoding="utf-8")
    match = re.search(
        rf"^{re.escape(name)}:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
        source,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        raise AssertionError(f"Makefile target '{name}' is missing")
    return match.group(0)


class CodeQualityContractTests(unittest.TestCase):
    def test_makefile_exposes_truthful_completion_gate_and_deep_audit_targets(self) -> None:
        completion_body = target_body("test-code-quality")
        self.assertIn("test-code-quality-contract", completion_body)
        self.assertIn("test-native-build-warning-hygiene", completion_body)
        self.assertIn("test-dashboard-svelte-check", completion_body)

        umbrella_body = target_body("test")
        self.assertIn("Step 1/9: Code-Quality Gate", umbrella_body)
        self.assertIn("test-code-quality", umbrella_body)

        deep_audit_body = target_body("audit-code-quality-deep")
        self.assertIn("cargo clippy --all-targets --all-features -- -D warnings", deep_audit_body)
        self.assertIn("corepack pnpm run test:dashboard:svelte-check:deep", deep_audit_body)

    def test_package_json_exposes_deep_dashboard_semantic_audit_script(self) -> None:
        source = PACKAGE_JSON.read_text(encoding="utf-8")
        self.assertIn('"test:dashboard:svelte-check:deep"', source)
        self.assertIn('\\"js,svelte\\"', source)

    def test_ci_runs_code_quality_gate_before_full_suite(self) -> None:
        source = CI_WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("name: Run code-quality completion gate", source)
        self.assertIn("run: make test-code-quality", source)

    def test_repo_policies_require_code_quality_gate_for_non_doc_completion(self) -> None:
        contributing = CONTRIBUTING.read_text(encoding="utf-8")
        self.assertIn("`make test-code-quality`", contributing)

        principles = PROJECT_PRINCIPLES.read_text(encoding="utf-8")
        self.assertIn("`make test-code-quality`", principles)

        agents = AGENTS.read_text(encoding="utf-8")
        self.assertIn("`make test-code-quality`", agents)

        testing = TESTING_GUIDE.read_text(encoding="utf-8")
        self.assertIn("make test-code-quality", testing)
        self.assertIn("make audit-code-quality-deep", testing)

        pr_template = PR_TEMPLATE.read_text(encoding="utf-8")
        self.assertIn("`make test-code-quality` executed and passing", pr_template)


if __name__ == "__main__":
    unittest.main()
