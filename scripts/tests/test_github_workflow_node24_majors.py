import pathlib
import re
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
WORKFLOW_DIR = REPO_ROOT / ".github" / "workflows"
WORKFLOW_FILES = sorted(WORKFLOW_DIR.glob("*.yml"))
EXPECTED_ACTION_VERSIONS = {
    "actions/checkout": "v5",
    "actions/setup-node": "v5",
    "actions/upload-artifact": "v6",
}
DEPRECATED_ACTION_VERSIONS = {
    "actions/checkout": "v4",
    "actions/setup-node": "v4",
    "actions/upload-artifact": "v4",
}


class GithubWorkflowNode24MajorTests(unittest.TestCase):
    def test_expected_workflow_files_exist(self) -> None:
        self.assertGreater(len(WORKFLOW_FILES), 0)

    def test_no_deprecated_node20_backed_action_majors_remain(self) -> None:
        offenders: list[str] = []
        for workflow in WORKFLOW_FILES:
            source = workflow.read_text(encoding="utf-8")
            for action, version in DEPRECATED_ACTION_VERSIONS.items():
                if f"{action}@{version}" in source:
                    offenders.append(f"{workflow.name}: {action}@{version}")
        self.assertEqual(
            offenders,
            [],
            f"Deprecated Node20-backed workflow action majors remain: {offenders}",
        )

    def test_all_workflow_pins_use_expected_node24_safe_majors(self) -> None:
        found_actions = {action: False for action in EXPECTED_ACTION_VERSIONS}
        for workflow in WORKFLOW_FILES:
            source = workflow.read_text(encoding="utf-8")
            for action, expected_version in EXPECTED_ACTION_VERSIONS.items():
                matches = re.findall(rf"{re.escape(action)}@([A-Za-z0-9_.-]+)", source)
                for actual_version in matches:
                    self.assertEqual(
                        actual_version,
                        expected_version,
                        (
                            f"{workflow.name} pins {action}@{actual_version}; "
                            f"expected {action}@{expected_version}"
                        ),
                    )
                    found_actions[action] = True
        self.assertEqual(
            [action for action, seen in found_actions.items() if not seen],
            [],
            f"Expected workflow action pins were not found: {found_actions}",
        )


if __name__ == "__main__":
    unittest.main()
