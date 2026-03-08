import os
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


def write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class LocalStateContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="clean-contract-"))
        self.bin_dir = self.temp_dir / "bin"
        self.bin_dir.mkdir()
        write_executable(
            self.bin_dir / "cargo",
            textwrap.dedent(
                """\
                #!/bin/sh
                if [ "$1" = "clean" ]; then
                  rm -rf target
                fi
                exit 0
                """
            ),
        )

        (self.temp_dir / "target").mkdir()
        (self.temp_dir / "dist" / "wasm").mkdir(parents=True)
        (self.temp_dir / "playwright-report").mkdir()
        (self.temp_dir / "test-results").mkdir()
        (self.temp_dir / ".spin" / "logs").mkdir(parents=True)
        (self.temp_dir / ".spin" / "logs" / "bot-defence_stdout.txt").write_text("log\n", encoding="utf-8")
        (self.temp_dir / ".shuma" / "remotes").mkdir(parents=True)
        (self.temp_dir / ".shuma" / "remotes" / "blog-prod.json").write_text("{}\n", encoding="utf-8")

    def run_make(self, target: str) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        env["PATH"] = f"{self.bin_dir}:{env['PATH']}"
        return subprocess.run(
            ["make", "-f", str(MAKEFILE), target],
            cwd=self.temp_dir,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_clean_preserves_durable_operator_state(self) -> None:
        result = self.run_make("clean")
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertFalse((self.temp_dir / "target").exists())
        self.assertFalse((self.temp_dir / "dist" / "wasm").exists())
        self.assertFalse((self.temp_dir / "playwright-report").exists())
        self.assertFalse((self.temp_dir / "test-results").exists())
        self.assertTrue((self.temp_dir / ".spin").exists())
        self.assertTrue((self.temp_dir / ".shuma" / "remotes" / "blog-prod.json").exists())

    def test_reset_local_state_removes_spin_but_preserves_durable_operator_state(self) -> None:
        result = self.run_make("reset-local-state")
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertFalse((self.temp_dir / ".spin").exists())
        self.assertTrue((self.temp_dir / ".shuma" / "remotes" / "blog-prod.json").exists())


if __name__ == "__main__":
    unittest.main()
