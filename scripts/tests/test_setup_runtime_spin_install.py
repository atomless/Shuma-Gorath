import os
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "bootstrap" / "setup-runtime.sh"


def write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class SetupRuntimeSpinInstallTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="setup-runtime-spin-"))
        self.workspace = self.temp_dir / "workspace"
        self.workspace.mkdir()
        config_dir = self.workspace / "config"
        config_dir.mkdir()
        bootstrap_dir = self.workspace / "scripts" / "bootstrap"
        bootstrap_dir.mkdir(parents=True)
        defaults_src = REPO_ROOT / "config" / "defaults.env"
        (config_dir / "defaults.env").write_text(defaults_src.read_text(encoding="utf-8"), encoding="utf-8")
        scrapling_runtime_src = REPO_ROOT / "scripts" / "bootstrap" / "scrapling_runtime.sh"
        (bootstrap_dir / "scrapling_runtime.sh").write_text(
            scrapling_runtime_src.read_text(encoding="utf-8"),
            encoding="utf-8",
        )

        self.home = self.temp_dir / "home"
        self.home.mkdir()
        self.fake_bin = self.temp_dir / "fake-bin"
        self.fake_bin.mkdir()
        write_executable(self.fake_bin / "sqlite3", "#!/bin/sh\nexit 127\n")
        self.stub_dir = self.temp_dir / "stubs"
        self.stub_dir.mkdir()
        self.make_log = self.temp_dir / "make.log"

        write_executable(
            self.stub_dir / "rustc",
            "#!/bin/sh\nprintf 'rustc 1.94.0\\n'\n",
        )
        write_executable(
            self.stub_dir / "cargo",
            "#!/bin/sh\nprintf 'cargo 1.94.0\\n'\n",
        )
        write_executable(
            self.stub_dir / "rustup",
            textwrap.dedent(
                """\
                #!/bin/sh
                if [ "$1" = "target" ] && [ "$2" = "list" ] && [ "$3" = "--installed" ]; then
                  printf 'wasm32-wasip1\\n'
                  exit 0
                fi
                if [ "$1" = "target" ] && [ "$2" = "add" ] && [ "$3" = "wasm32-wasip1" ]; then
                  exit 0
                fi
                exit 0
                """
            ),
        )
        write_executable(
            self.stub_dir / "curl",
            textwrap.dedent(
                """\
                #!/bin/sh
                cat <<'EOF'
                cat > spin <<'SPIN'
                #!/bin/sh
                printf 'spin 3.6.2\\n'
                SPIN
                chmod +x spin
                EOF
                """
            ),
        )
        write_executable(
            self.stub_dir / "python3",
            textwrap.dedent(
                """\
                #!/bin/sh
                if [ "$1" = "-m" ] && [ "$2" = "venv" ]; then
                  target="$3"
                  mkdir -p "$target/bin"
                  cat > "$target/bin/python3" <<'PYTHON'
#!/bin/sh
if [ "$1" = "-m" ] && [ "$2" = "pip" ]; then
  exit 0
fi
if [ "$1" = "-" ]; then
  exit 0
fi
exit 0
PYTHON
                  chmod +x "$target/bin/python3"
                  exit 0
                fi
                if [ "$1" = "-" ]; then
                  exit 0
                fi
                exit 0
                """
            ),
        )
        write_executable(self.stub_dir / "apt-get", "#!/bin/sh\nexit 0\n")
        write_executable(
            self.stub_dir / "sudo",
            textwrap.dedent(
                f"""\
                #!/bin/sh
                if [ "$1" = "-n" ] && [ "$2" = "true" ]; then
                  exit 0
                fi
                if [ "$1" = "-n" ]; then
                  shift
                fi
                if [ "$1" = "apt-get" ] && [ "$2" = "update" ] && [ "$3" = "-y" ]; then
                  exit 0
                fi
                if [ "$1" = "env" ] && [ "$2" = "DEBIAN_FRONTEND=noninteractive" ] && [ "$3" = "apt-get" ] && [ "$4" = "install" ] && [ "$5" = "-y" ] && [ "$6" = "sqlite3" ]; then
                  cat > "{self.fake_bin}/sqlite3" <<'SQLITE'
#!/bin/sh
printf '3.45.1\\n'
SQLITE
                  chmod +x "{self.fake_bin}/sqlite3"
                  exit 0
                fi
                if [ "$1" = "/bin/mv" ]; then
                  src="$2"
                  dest="$3"
                  if [ "$dest" = "/usr/local/bin/spin" ]; then
                    mkdir -p "{self.fake_bin}"
                    mv "$src" "{self.fake_bin}/spin"
                    exit 0
                  fi
                fi
                echo "unexpected sudo invocation: $@" >&2
                exit 1
                """
            ),
        )
        write_executable(
            self.stub_dir / "make",
            f"#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{self.make_log}\"\nexit 0\n",
        )

    def test_noninteractive_passwordless_sudo_installs_spin_and_finishes(self) -> None:
        env = os.environ.copy()
        env["HOME"] = str(self.home)
        env["PATH"] = f"{self.stub_dir}:{self.fake_bin}:/usr/bin:/bin"

        result = subprocess.run(
            ["bash", str(SCRIPT)],
            cwd=self.workspace,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue((self.fake_bin / "spin").exists())
        self.assertTrue((self.fake_bin / "sqlite3").exists())
        self.assertTrue((self.workspace / ".env.local").exists())
        self.assertTrue(self.make_log.exists())
        self.assertIn("config-seed", self.make_log.read_text(encoding="utf-8"))

    def test_passwordless_sudo_installs_matching_python_venv_package_when_ensurepip_missing(self) -> None:
        venv_ready = self.temp_dir / "venv-ready"
        write_executable(
            self.stub_dir / "python3",
            textwrap.dedent(
                f"""\
                #!/bin/sh
                if [ "$1" = "-m" ] && [ "$2" = "venv" ]; then
                  target="$3"
                  if [ ! -f "{venv_ready}" ]; then
                    mkdir -p "$target/bin"
                    cat > "$target/bin/python3" <<'PYTHON'
#!/bin/sh
if [ "$1" = "-m" ] && [ "$2" = "pip" ]; then
  printf 'No module named pip\n' >&2
  exit 1
fi
if [ "$1" = "-" ]; then
  exit 0
fi
exit 0
PYTHON
                    chmod +x "$target/bin/python3"
                    cat >&2 <<'EOF'
The virtual environment was not created successfully because ensurepip is not
available.  On Debian/Ubuntu systems, you need to install the python3.11-venv
package using the following command.
EOF
                    exit 1
                  fi
                  mkdir -p "$target/bin"
                  cat > "$target/bin/python3" <<'PYTHON'
#!/bin/sh
if [ "$1" = "-m" ] && [ "$2" = "pip" ]; then
  exit 0
fi
if [ "$1" = "-" ]; then
  exit 0
fi
exit 0
PYTHON
                  chmod +x "$target/bin/python3"
                  exit 0
                fi
                if [ "$1" = "-c" ]; then
                  printf 'python3.11-venv\n'
                  exit 0
                fi
                if [ "$1" = "-" ]; then
                  exit 0
                fi
                exit 0
                """
            ),
        )
        write_executable(
            self.stub_dir / "sudo",
            textwrap.dedent(
                f"""\
                #!/bin/sh
                if [ "$1" = "-n" ] && [ "$2" = "true" ]; then
                  exit 0
                fi
                if [ "$1" = "-n" ]; then
                  shift
                fi
                if [ "$1" = "apt-get" ] && [ "$2" = "update" ] && [ "$3" = "-y" ]; then
                  exit 0
                fi
                if [ "$1" = "env" ] && [ "$2" = "DEBIAN_FRONTEND=noninteractive" ] && [ "$3" = "apt-get" ] && [ "$4" = "install" ] && [ "$5" = "-y" ] && [ "$6" = "sqlite3" ]; then
                  cat > "{self.fake_bin}/sqlite3" <<'SQLITE'
#!/bin/sh
printf '3.45.1\\n'
SQLITE
                  chmod +x "{self.fake_bin}/sqlite3"
                  exit 0
                fi
                if [ "$1" = "env" ] && [ "$2" = "DEBIAN_FRONTEND=noninteractive" ] && [ "$3" = "apt-get" ] && [ "$4" = "install" ] && [ "$5" = "-y" ] && [ "$6" = "python3.11-venv" ]; then
                  : > "{venv_ready}"
                  exit 0
                fi
                if [ "$1" = "/bin/mv" ]; then
                  src="$2"
                  dest="$3"
                  if [ "$dest" = "/usr/local/bin/spin" ]; then
                    mkdir -p "{self.fake_bin}"
                    mv "$src" "{self.fake_bin}/spin"
                    exit 0
                  fi
                fi
                echo "unexpected sudo invocation: $@" >&2
                exit 1
                """
            ),
        )

        env = os.environ.copy()
        env["HOME"] = str(self.home)
        env["PATH"] = f"{self.stub_dir}:{self.fake_bin}:/usr/bin:/bin"

        result = subprocess.run(
            ["bash", str(SCRIPT)],
            cwd=self.workspace,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue(venv_ready.exists())
        self.assertTrue((self.workspace / ".venv-scrapling" / "bin" / "python3").exists())


if __name__ == "__main__":
    unittest.main()
