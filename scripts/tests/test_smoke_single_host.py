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
                SHUMA_ADMIN_IP_ALLOWLIST=198.51.100.8/32
                SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://origin.example.com
                SHUMA_JS_SECRET=test-js-secret
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
                insecure_tls = False
                i = 0
                while i < len(args):
                    arg = args[i]
                    if arg == "-k":
                        insecure_tls = True
                        i += 1
                        continue
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
                forwarded_proto_header = next((value for value in headers if value.lower().startswith("x-forwarded-proto:")), "")
                forwarded_for_header = next((value for value in headers if value.lower().startswith("x-forwarded-for:")), "")
                cookie_header = next((value for value in headers if value.lower().startswith("cookie:")), "")
                body = ""
                status = "500"

                require_https_forward_proto = os.environ.get("SHUMA_REQUIRE_HTTPS_FORWARD_PROTO") == "1"
                require_insecure_tls_flag = os.environ.get("SHUMA_REQUIRE_INSECURE_TLS_FLAG") == "1"
                require_js_verified_for_forward = os.environ.get("SHUMA_TEST_REQUIRE_JS_VERIFIED_FOR_FORWARD") == "1"
                required_admin_ip = os.environ.get("SHUMA_TEST_ADMIN_ALLOWLIST_IP", "").strip()
                forwarded_ip = forwarded_for_header.split(":", 1)[1].strip() if ":" in forwarded_for_header else ""
                js_secret = os.environ.get("SHUMA_JS_SECRET", "test-js-secret")

                def expected_js_cookie(ip):
                    import base64
                    import hashlib
                    import hmac
                    token = base64.b64encode(hmac.new(js_secret.encode("utf-8"), ip.encode("utf-8"), hashlib.sha256).digest()).decode("ascii")
                    return f"js_verified={token}"

                gateway_request = (
                    url.startswith("http://gateway.example.com")
                    or url.startswith("https://172.239.98.201.sslip.io")
                )
                if require_insecure_tls_flag and not insecure_tls:
                    body, status = "TLS validation failed", "000"
                elif require_https_forward_proto and gateway_request and forwarded_proto_header.lower() != "x-forwarded-proto: https":
                    body, status = "HTTPS required", "403"
                elif url.endswith("/admin/config") and required_admin_ip and forwarded_ip != required_admin_ip:
                    body, status = "Forbidden", "403"
                elif url.endswith("/health"):
                    health_status = os.environ.get("SHUMA_TEST_HEALTH_STATUS", "200")
                    body, status = ("OK", "200") if health_status == "200" else ("Forbidden", health_status)
                elif url.endswith("/admin/config") and auth_header:
                    body, status = '{"rate_limit":{}}', "200"
                elif url.endswith("/admin/config") and os.environ.get("SHUMA_TEST_ADMIN_REDIRECT_UNAUTH") == "1":
                    body, status = "", "302"
                elif url.endswith("/admin/config"):
                    body, status = "Unauthorized", "401"
                elif url.endswith("/metrics"):
                    body, status = "bot_defence_requests_total 1\\n", "200"
                elif url.endswith("/challenge/not-a-bot-checkbox"):
                    body, status = "I am not a bot", "200"
                elif url in {"http://gateway.example.com/public/page", "https://172.239.98.201.sslip.io/public/page"}:
                    cookie_value = cookie_header.split(":", 1)[1].strip() if ":" in cookie_header else ""
                    if require_js_verified_for_forward and cookie_value != expected_js_cookie(forwarded_ip):
                        body, status = "Verifying...", "200"
                    else:
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
        self,
        env_overrides: Optional[Dict[str, str]] = None,
        *,
        base_url: str = "http://gateway.example.com",
        auto_challenge: bool = False,
    ) -> subprocess.CompletedProcess:
        env = os.environ.copy()
        env["PATH"] = f"{self.stub_dir}:{env['PATH']}"
        env["SHUMA_SMOKE_FORWARD_PATH"] = "/public/page"
        env["SHUMA_TEST_GATEWAY_FORWARD_BODY"] = "same-body"
        env["SHUMA_TEST_ORIGIN_FORWARD_BODY"] = "same-body"
        if env_overrides:
            env.update(env_overrides)
        command = [
            "bash",
            str(SCRIPT),
            "--base-url",
            base_url,
        ]
        if not auto_challenge:
            command.extend(
                [
                    "--challenge-path",
                    "/challenge/not-a-bot-checkbox",
                    "--challenge-expect",
                    "I am not a bot",
                ]
            )
        return subprocess.run(
            command,
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

    def test_forwarded_path_parity_uses_js_verified_cookie_when_secret_is_available(self) -> None:
        result = self.run_smoke({"SHUMA_TEST_REQUIRE_JS_VERIFIED_FOR_FORWARD": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

    def test_includes_forwarded_proto_for_https_enforced_loopback_checks(self) -> None:
        result = self.run_smoke({"SHUMA_REQUIRE_HTTPS_FORWARD_PROTO": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

    def test_uses_allowlisted_ip_for_admin_checks_by_default(self) -> None:
        result = self.run_smoke({"SHUMA_TEST_ADMIN_ALLOWLIST_IP": "198.51.100.8"})
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

    def test_accepts_redirect_to_login_for_unauthenticated_admin_config(self) -> None:
        result = self.run_smoke({"SHUMA_TEST_ADMIN_REDIRECT_UNAUTH": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

    def test_skip_admin_auth_allows_proof_domain_smoke_without_admin_api_probe(self) -> None:
        result = self.run_smoke({"SHUMA_SMOKE_SKIP_ADMIN_AUTH": "true"})
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("Skipping /admin/config auth checks", result.stdout)

    def test_skip_reserved_routes_allows_public_smoke_without_admin_or_metrics_probes(self) -> None:
        result = self.run_smoke({"SHUMA_SMOKE_SKIP_RESERVED_ROUTES": "true"})
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("Skipping reserved-route smoke probes", result.stdout)

    def test_skip_reserved_routes_can_auto_select_default_challenge_without_admin_config(self) -> None:
        result = self.run_smoke({"SHUMA_SMOKE_SKIP_RESERVED_ROUTES": "true"}, auto_challenge=True)
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("/challenge/not-a-bot-checkbox challenge route responds with expected content", result.stdout)

    def test_skip_health_allows_public_route_smoke_without_public_health_probe(self) -> None:
        result = self.run_smoke(
            {
                "SHUMA_TEST_HEALTH_STATUS": "403",
                "SHUMA_SMOKE_SKIP_HEALTH": "1",
            }
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn("Skipping /health check", result.stdout)

    def test_insecure_tls_flag_adds_k_for_sslip_proof_domains(self) -> None:
        result = self.run_smoke(
            {
                "SHUMA_SMOKE_INSECURE_TLS": "true",
                "SHUMA_REQUIRE_INSECURE_TLS_FLAG": "1",
            },
            base_url="https://172.239.98.201.sslip.io",
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)



if __name__ == "__main__":
    unittest.main()
