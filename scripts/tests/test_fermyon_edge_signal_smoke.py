import http.server
import importlib.util
import json
import socketserver
import tempfile
import threading
import unittest
from pathlib import Path
from unittest.mock import patch
from urllib.parse import parse_qs, urlparse


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "fermyon_edge_signal_smoke.py"
SPEC = importlib.util.spec_from_file_location("fermyon_edge_signal_smoke", SCRIPT)
FERMYON_EDGE_SIGNAL_SMOKE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(FERMYON_EDGE_SIGNAL_SMOKE)


class _StubState:
    def __init__(self, *, guard_authoritative: bool) -> None:
        self.guard_authoritative = guard_authoritative
        self.actual_ip = "203.0.113.99"
        self.original_config = {
            "admin_config_write_enabled": True,
            "provider_backends": {
                "fingerprint_signal": "internal",
            },
            "edge_integration_mode": "off",
            "cdp_detection_enabled": True,
            "cdp_auto_ban": True,
            "geo_edge_headers_enabled": False,
            "geo_risk": [],
            "geo_allow": [],
            "geo_challenge": [],
            "geo_maze": [],
            "geo_block": [],
            "maze_enabled": True,
            "maze_auto_ban": True,
        }
        self.config = json.loads(json.dumps(self.original_config))
        self.banned_ips = set()
        self.unban_calls = []


class _ThreadedServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True


class FermyonEdgeSignalSmokeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="fermyon-edge-signal-smoke-"))
        self.env_file = self.temp_dir / ".env.local"
        self.deploy_receipt = self.temp_dir / ".shuma" / "fermyon-akamai-edge-deploy.json"
        self.report_path = self.temp_dir / "report.json"

    def _start_server(self, state: _StubState) -> str:
        handler = self._build_handler(state)
        self.server = _ThreadedServer(("127.0.0.1", 0), handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        return f"http://127.0.0.1:{self.server.server_port}"

    def tearDown(self) -> None:
        server = getattr(self, "server", None)
        if server is not None:
            server.shutdown()
            server.server_close()
        thread = getattr(self, "thread", None)
        if thread is not None:
            thread.join(timeout=2)

    def _build_handler(self, state: _StubState):
        class Handler(http.server.BaseHTTPRequestHandler):
            protocol_version = "HTTP/1.1"

            def _read_json(self):
                length = int(self.headers.get("Content-Length", "0"))
                raw = self.rfile.read(length) if length else b""
                return json.loads(raw.decode("utf-8") or "{}")

            def _send_json(self, payload, status=200):
                raw = json.dumps(payload).encode("utf-8")
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(raw)))
                self.end_headers()
                self.wfile.write(raw)

            def _send_text(self, body, status=200):
                raw = body.encode("utf-8")
                self.send_response(status)
                self.send_header("Content-Type", "text/plain; charset=utf-8")
                self.send_header("Content-Length", str(len(raw)))
                self.end_headers()
                self.wfile.write(raw)

            def _require_auth(self):
                return self.headers.get("Authorization") == "Bearer test-admin-key"

            def do_GET(self):
                parsed = urlparse(self.path)
                if parsed.path == "/admin/config":
                    if not self._require_auth():
                        self._send_text("Unauthorized", 401)
                        return
                    self._send_json(state.config)
                    return

                if parsed.path == "/admin/ban":
                    if not self._require_auth():
                        self._send_text("Unauthorized", 401)
                        return
                    bans = [
                        {
                            "ip": ip,
                            "reason": "edge_fingerprint_automation",
                            "expires": 1774000000,
                            "banned_at": 1773000000,
                            "fingerprint": {
                                "signals": ["edge_fingerprint"],
                                "summary": "provider=akamai action=deny",
                            },
                        }
                        for ip in sorted(state.banned_ips)
                    ]
                    self._send_json({"bans": bans})
                    return

                if parsed.path == "/":
                    country = self.headers.get("X-Geo-Country", "").strip().upper()
                    if state.actual_ip in state.banned_ips:
                        self._send_text("Access Blocked", 403)
                        return
                    if country and country in state.config.get("geo_block", []):
                        self._send_text("Access Restricted", 403)
                        return
                    if (
                        country
                        and country in state.config.get("geo_maze", [])
                        and state.config.get("maze_enabled") is True
                    ):
                        self._send_text('data-link-kind="maze"')
                        return
                    if country and country in state.config.get("geo_challenge", []):
                        self._send_text("Puzzle")
                        return
                    self._send_text("Origin Pass")
                    return

                self._send_text("Not Found", 404)

            def do_POST(self):
                parsed = urlparse(self.path)
                if parsed.path == "/admin/config":
                    if not self._require_auth():
                        self._send_text("Unauthorized", 401)
                        return
                    payload = self._read_json()
                    for key, value in payload.items():
                        if isinstance(value, dict) and isinstance(state.config.get(key), dict):
                            state.config[key].update(value)
                        else:
                            state.config[key] = value
                    response = dict(state.config)
                    response["status"] = "updated"
                    self._send_json(response)
                    return

                if parsed.path == "/admin/unban":
                    if not self._require_auth():
                        self._send_text("Unauthorized", 401)
                        return
                    ip = parse_qs(parsed.query).get("ip", [""])[0]
                    if ip:
                        state.banned_ips.discard(ip)
                        state.unban_calls.append(ip)
                    self._send_text("unbanned")
                    return

                if parsed.path == "/fingerprint-report":
                    payload = self._read_json()
                    backend = state.config.get("provider_backends", {}).get("fingerprint_signal")
                    mode = state.config.get("edge_integration_mode")
                    if backend != "external":
                        self._send_text("External fingerprint report ignored (internal backend)")
                        return
                    if mode == "additive":
                        self._send_text("External fingerprint report received (additive)")
                        return
                    if mode == "authoritative" and payload.get("action") == "deny":
                        if state.guard_authoritative:
                            self._send_text("Server configuration error", 503)
                            return
                        state.banned_ips.add(state.actual_ip)
                        self._send_text("External fingerprint automation detected - banned")
                        return
                    self._send_text("External fingerprint report received")
                    return

                self._send_text("Not Found", 404)

            def log_message(self, fmt, *args):
                return

        return Handler

    def _write_env_and_receipt(self, base_url: str) -> None:
        self.env_file.write_text(
            "\n".join(
                [
                    "SHUMA_API_KEY=test-admin-key",
                    "SHUMA_FORWARDED_IP_SECRET=test-forwarded-secret",
                    "",
                ]
            ),
            encoding="utf-8",
        )
        self.deploy_receipt.parent.mkdir(parents=True, exist_ok=True)
        self.deploy_receipt.write_text(
            json.dumps(
                {
                    "schema": "shuma.fermyon.akamai_edge_deploy.v1",
                    "setup_receipt_path": str(self.temp_dir / ".shuma" / "setup.json"),
                    "fermyon": {
                        "account_id": "acc_123",
                        "account_name": "",
                        "app_id": "app_123",
                        "app_name": "shuma-edge-test",
                        "primary_url": base_url,
                    },
                }
            )
            + "\n",
            encoding="utf-8",
        )

    def test_live_fermyon_edge_signal_smoke_accepts_authoritative_guardrail(self) -> None:
        state = _StubState(guard_authoritative=True)
        base_url = self._start_server(state)
        self._write_env_and_receipt(base_url)
        logs_output = (
            "2026-03-14 11:27:23 [bot-defence] "
            "[ENTERPRISE STATE ERROR] path=/fingerprint-report "
            "enterprise multi-instance rollout cannot run with local-only rate/ban state in authoritative mode\n"
        )

        def fake_run(command, capture_output=True, text=True, check=False):
            if command[:3] == ["spin", "aka", "logs"]:
                return type("Completed", (), {"returncode": 0, "stdout": logs_output, "stderr": ""})()
            raise AssertionError(f"Unexpected command: {command}")

        with patch.object(FERMYON_EDGE_SIGNAL_SMOKE.subprocess, "run", side_effect=fake_run):
            runner = FERMYON_EDGE_SIGNAL_SMOKE.FermyonEdgeSignalSmoke(
                env_file=self.env_file,
                deploy_receipt_path=self.deploy_receipt,
                report_path=self.report_path,
            )
            rc = runner.run()

        self.assertEqual(rc, 0)
        report = json.loads(self.report_path.read_text(encoding="utf-8"))
        self.assertEqual(report["fermyon"]["app_name"], "shuma-edge-test")
        self.assertEqual(report["fermyon"]["base_url"], base_url)
        self.assertTrue(all(check["ok"] for check in report["checks"]))
        auth_checks = [check for check in report["checks"] if check["name"] == "akamai_fingerprint_authoritative"]
        self.assertEqual(len(auth_checks), 1)
        self.assertIn("guardrail", auth_checks[0]["details"])
        self.assertEqual(state.banned_ips, set())
        self.assertEqual(state.unban_calls, [])
        self.assertEqual(state.config, state.original_config)

    def test_live_fermyon_edge_signal_smoke_accepts_authoritative_ban_when_state_allows_it(self) -> None:
        state = _StubState(guard_authoritative=False)
        base_url = self._start_server(state)
        self._write_env_and_receipt(base_url)

        def fake_run(command, capture_output=True, text=True, check=False):
            if command[:3] == ["spin", "aka", "logs"]:
                return type("Completed", (), {"returncode": 0, "stdout": "", "stderr": ""})()
            raise AssertionError(f"Unexpected command: {command}")

        with patch.object(FERMYON_EDGE_SIGNAL_SMOKE.subprocess, "run", side_effect=fake_run):
            runner = FERMYON_EDGE_SIGNAL_SMOKE.FermyonEdgeSignalSmoke(
                env_file=self.env_file,
                deploy_receipt_path=self.deploy_receipt,
                report_path=self.report_path,
            )
            rc = runner.run()

        self.assertEqual(rc, 0)
        report = json.loads(self.report_path.read_text(encoding="utf-8"))
        auth_checks = [check for check in report["checks"] if check["name"] == "akamai_fingerprint_authoritative"]
        self.assertEqual(len(auth_checks), 1)
        self.assertIn("immediate authoritative ban", auth_checks[0]["details"])
        self.assertEqual(state.banned_ips, set())
        self.assertEqual(state.unban_calls, [state.actual_ip])
        self.assertEqual(state.config, state.original_config)


if __name__ == "__main__":
    unittest.main()
