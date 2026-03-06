import os
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


def write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class ProdStartSpinManifestTests(unittest.TestCase):
    def test_prod_start_honors_custom_spin_manifest_path(self) -> None:
        temp_dir = Path(tempfile.mkdtemp(prefix="prod-start-manifest-"))
        stub_dir = temp_dir / "bin"
        stub_dir.mkdir()
        spin_log = temp_dir / "spin.log"
        custom_manifest = temp_dir / "spin.gateway.toml"
        custom_manifest.write_text("spin_manifest_version = 2\n", encoding="utf-8")

        write_executable(
            stub_dir / "spin",
            textwrap.dedent(
                f"""\
                #!/bin/sh
                printf '%s\\n' "$@" > "{spin_log}"
                exit 0
                """
            ),
        )
        write_executable(stub_dir / "pkill", "#!/bin/sh\nexit 0\n")
        write_executable(stub_dir / "uuidgen", "#!/bin/sh\necho test-runtime-id\n")

        env = os.environ.copy()
        env["PATH"] = f"{stub_dir}:{env['PATH']}"
        env["SHUMA_API_KEY"] = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        env["SHUMA_FORWARDED_IP_SECRET"] = "forwarded-secret"
        env["SHUMA_SPIN_MANIFEST"] = str(custom_manifest)

        result = subprocess.run(
            ["make", "--no-print-directory", "prod-start"],
            cwd=str(REPO_ROOT),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue(spin_log.exists())
        spin_args = spin_log.read_text(encoding="utf-8")
        self.assertIn("--from", spin_args)
        self.assertIn(str(custom_manifest), spin_args)


if __name__ == "__main__":
    unittest.main()
