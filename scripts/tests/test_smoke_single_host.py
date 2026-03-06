import os
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path
from typing import Dict, Optional


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "smoke_single_host.sh"


def write_executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class SmokeSingleHostTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="smoke-single-host-test-"))
        self.stub_dir = self.temp_dir / "bin"
        self.stub_dir.mkdir()
        self.env_local = self.temp_dir / ".env.local"
        self.env_local.write_text(
            textwrap.dedent(
                """\
                SHUMA_API_KEY=test-admin-key
                SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://origin.example.com
                """
            ),
            encoding="utf-8",
        )

        write_executable(
            self.stub_dir / "curl",
            textwrap.dedent(
                """\
                #!/usr/bin/env python3
                import os
                import sys

                args = sys.argv[1:]
                headers = []
                url = ""
                i = 0
                while i < len(args):
                    arg = args[i]
                    if arg == "-H":
                        headers.append(args[i + 1])
                        i += 2
                        continue
                    if arg == "-X":
                        i += 2
                        continue
                    if arg in {"-s", "--max-time", "-w"}:
                        i += 2
                        continue
                    if not arg.startswith("-"):
                        url = arg
                    i += 1

                auth_header = next((value for value in headers if value.lower().startswith("authorization:")), "")
                body = ""
                status = "500"

                if url.endswith("/health"):
                    body, status = "OK", "200"
                elif url.endswith("/admin/config") and auth_header:
                    body, status = '{"rate_limit":{}}', "200"
                elif url.endswith("/admin/config"):
                    body, status = "Unauthorized", "401"
                elif url.endswith("/metrics"):
                    body, status = "bot_defence_requests_total 1\\n", "200"
                elif url.endswith("/challenge/not-a-bot-checkbox"):
                    body, status = "I am not a bot", "200"
                elif url == "http://gateway.example.com/public/page":
                    body, status = os.environ.get("SHUMA_TEST_GATEWAY_FORWARD_BODY", "same-body"), "200"
                elif url == "https://origin.example.com/public/page":
                    body, status = os.environ.get("SHUMA_TEST_ORIGIN_FORWARD_BODY", "same-body"), "200"
                else:
                    body, status = f"unhandled:{url}", "404"

                sys.stdout.write(body)
                sys.stdout.write(f"\\n__HTTP_STATUS__:{status}")
                """
            ),
        )

    def run_smoke(
        self, env_overrides: Optional[Dict[str, str]] = None
    ) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        env["PATH"] = f"{self.stub_dir}:{env['PATH']}"
        env["SHUMA_SMOKE_FORWARD_PATH"] = "/public/page"
        env["SHUMA_TEST_GATEWAY_FORWARD_BODY"] = "same-body"
        env["SHUMA_TEST_ORIGIN_FORWARD_BODY"] = "same-body"
        if env_overrides:
            env.update(env_overrides)
        return subprocess.run(
            [
                "bash",
                str(SCRIPT),
                "--base-url",
                "http://gateway.example.com",
                "--challenge-path",
                "/challenge/not-a-bot-checkbox",
                "--challenge-expect",
                "I am not a bot",
            ],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_passes_when_forwarded_path_matches_origin(self) -> None:
        result = self.run_smoke()
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

    def test_fails_when_forwarded_path_differs_from_origin(self) -> None:
        result = self.run_smoke(
            {
                "SHUMA_TEST_GATEWAY_FORWARD_BODY": "gateway-body",
                "SHUMA_TEST_ORIGIN_FORWARD_BODY": "origin-body",
            }
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("/public/page", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
