import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
WORKFLOW_FILES = sorted(WORKFLOWS.glob("*.yml")) + sorted(WORKFLOWS.glob("*.yaml"))

REQUIRED_ACTION_VERSIONS = {
    "actions/checkout": "v6",
    "actions/setup-node": "v6",
    "actions/upload-artifact": "v7",
}


class CiWorkflowActionVersionTests(unittest.TestCase):
    def test_no_node20_backed_action_majors_remain(self) -> None:
        offenders: list[str] = []

        for workflow in WORKFLOW_FILES:
            source = workflow.read_text(encoding="utf-8")
            for action, version in REQUIRED_ACTION_VERSIONS.items():
                pattern = re.compile(rf"{re.escape(action)}@(v\d+)")
                for match in pattern.finditer(source):
                    if match.group(1) != version:
                        offenders.append(
                            f"{workflow.relative_to(REPO_ROOT)} uses {action}@{match.group(1)}"
                        )

        self.assertEqual(
            offenders,
            [],
            msg=(
                "Workflow files must use the current Node24-backed official action majors. "
                "Offenders:\n- " + "\n- ".join(offenders)
            ),
        )

    def test_expected_versions_are_present_in_workflows_that_use_them(self) -> None:
        usage = {action: 0 for action in REQUIRED_ACTION_VERSIONS}

        for workflow in WORKFLOW_FILES:
            source = workflow.read_text(encoding="utf-8")
            for action, version in REQUIRED_ACTION_VERSIONS.items():
                usage[action] += len(re.findall(rf"{re.escape(action)}@{re.escape(version)}", source))

        self.assertGreater(usage["actions/checkout"], 0)
        self.assertGreater(usage["actions/setup-node"], 0)
        self.assertGreater(usage["actions/upload-artifact"], 0)


if __name__ == "__main__":
    unittest.main()
