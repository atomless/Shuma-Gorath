import os
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "verify_test_runtime_environment.sh"


def write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class VerifyTestRuntimeEnvironmentTests(unittest.TestCase):
    def test_passes_when_runtime_matches_expected_value(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="verify-test-runtime-"))
        stub_dir = temp_dir / "bin"
        stub_dir.mkdir()

        write_executable(
            stub_dir / "curl",
            textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import sys

                if sys.argv[-1].endswith("/admin/session"):
                    sys.stdout.write('{"runtime_environment":"runtime-dev"}')
                    sys.exit(0)
                sys.exit(1)
                """
            ),
        )

        env = os.environ.copy()
        env["PATH"] = f"{stub_dir}:{env['PATH']}"

        result = subprocess.run(
            ["bash", str(SCRIPT)],
            cwd=temp_dir,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("runtime_environment=runtime-dev", result.stdout)

    def test_fails_fast_with_clear_message_when_runtime_prod_is_running(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="verify-test-runtime-"))
        stub_dir = temp_dir / "bin"
        stub_dir.mkdir()

        write_executable(
            stub_dir / "curl",
            textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import sys

                if sys.argv[-1].endswith("/admin/session"):
                    sys.stdout.write('{"runtime_environment":"runtime-prod"}')
                    sys.exit(0)
                sys.exit(1)
                """
            ),
        )

        env = os.environ.copy()
        env["PATH"] = f"{stub_dir}:{env['PATH']}"

        result = subprocess.run(
            ["bash", str(SCRIPT)],
            cwd=temp_dir,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("make test requires a runtime-dev server from make dev", result.stderr)
        self.assertIn("Current runtime_environment=runtime-prod", result.stderr)


if __name__ == "__main__":
    unittest.main()
