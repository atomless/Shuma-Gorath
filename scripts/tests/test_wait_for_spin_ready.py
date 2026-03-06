import os
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "wait_for_spin_ready.sh"


def write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class WaitForSpinReadyTests(unittest.TestCase):
    def test_sends_forwarded_proto_https_header(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="wait-spin-ready-"))
        stub_dir = temp_dir / "bin"
        stub_dir.mkdir()

        write_executable(
            stub_dir / "curl",
            textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import sys

                args = sys.argv[1:]
                headers = []
                i = 0
                while i < len(args):
                    if args[i] == "-H":
                        headers.append(args[i + 1].lower())
                        i += 2
                        continue
                    if args[i] in {"-s", "--max-time", "-w"}:
                        i += 2
                        continue
                    i += 1

                if "x-forwarded-proto: https" in headers:
                    sys.stdout.write("OK\\n__HTTP_STATUS__:200")
                else:
                    sys.stdout.write("HTTPS required\\n__HTTP_STATUS__:403")
                """
            ),
        )

        env = os.environ.copy()
        env["PATH"] = f"{stub_dir}:{env['PATH']}"

        result = subprocess.run(
            ["bash", str(SCRIPT), "--timeout-seconds", "2", "--base-url", "http://127.0.0.1:3000"],
            cwd=temp_dir,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("Spin server is ready", result.stdout)


if __name__ == "__main__":
    unittest.main()
