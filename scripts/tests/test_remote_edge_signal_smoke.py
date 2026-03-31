import http.server
import importlib.util
import json
import socketserver
import subprocess
import tempfile
import threading
import unittest
from pathlib import Path
from urllib.parse import parse_qs, urlparse


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "tests" / "remote_edge_signal_smoke.py"
SPEC = importlib.util.spec_from_file_location("remote_edge_signal_smoke", SCRIPT)
REMOTE_EDGE_SIGNAL_SMOKE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(REMOTE_EDGE_SIGNAL_SMOKE)


class _StubState:
    def __init__(self) -> None:
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


class RemoteEdgeSignalSmokeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="remote-edge-signal-smoke-"))
        self.env_file = self.temp_dir / ".env.local"
        self.receipts_dir = self.temp_dir / ".shuma" / "remotes"
        self.receipts_dir.mkdir(parents=True)
        self.report_path = self.temp_dir / "report.json"
        self.state = _StubState()

        handler = self._build_handler(self.state)
        self.server = _ThreadedServer(("127.0.0.1", 0), handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.base_url = f"http://127.0.0.1:{self.server.server_port}"

        self.env_file.write_text(
            "\n".join(
                [
                    "SHUMA_API_KEY=test-admin-key",
                    "SHUMA_ACTIVE_REMOTE=stub-remote",
                    "",
                ]
            ),
            encoding="utf-8",
        )
        (self.receipts_dir / "stub-remote.json").write_text(
            json.dumps(
                {
                    "schema": "shuma.remote_target.v1",
                    "identity": {
                        "name": "stub-remote",
                        "backend_kind": "ssh_systemd",
                        "provider_kind": "test",
                    },
                    "ssh": {
                        "host": "127.0.0.1",
                        "port": 22,
                        "user": "shuma",
                        "private_key_path": "/tmp/test-key",
                    },
                    "runtime": {
                        "app_dir": "/opt/shuma-gorath",
                        "service_name": "shuma-gorath",
                        "public_base_url": self.base_url,
                    },
                    "deploy": {
                        "spin_manifest_path": "/opt/shuma-gorath/spin.gateway.toml",
                        "surface_catalog_path": str(self.temp_dir / "surface-catalog.json"),
                        "smoke_path": "/shuma/health",
                        "upstream_origin": "http://127.0.0.1:8080",
                    },
                    "metadata": {
                        "last_deployed_commit": "",
                        "last_deployed_at_utc": "",
                    },
                    "provider": {},
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

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
                if parsed.path == "/shuma/admin/config":
                    if not self._require_auth():
                        self._send_text("Unauthorized", 401)
                        return
                    self._send_json(state.config)
                    return

                if parsed.path == "/shuma/admin/ban":
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
                    forwarded_ip = self.headers.get("X-Forwarded-For", "").split(",", 1)[0].strip()
                    country = self.headers.get("X-Geo-Country", "").strip().upper()
                    if forwarded_ip in state.banned_ips:
                        self._send_text("Access Blocked", 403)
                        return
                    if country and country in state.config.get("geo_block", []):
                        self._send_text("Access Blocked", 403)
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
                if parsed.path == "/shuma/admin/config":
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

                if parsed.path == "/shuma/admin/unban":
                    if not self._require_auth():
                        self._send_text("Unauthorized", 401)
                        return
                    params = parse_qs(parsed.query)
                    ip = params.get("ip", [""])[0]
                    if ip:
                        state.banned_ips.discard(ip)
                        state.unban_calls.append(ip)
                    self._send_text("unbanned")
                    return

                if parsed.path == "/fingerprint-report":
                    payload = self._read_json()
                    ip = self.headers.get("X-Forwarded-For", "").split(",", 1)[0].strip()
                    backend = state.config.get("provider_backends", {}).get("fingerprint_signal")
                    mode = state.config.get("edge_integration_mode")
                    if backend != "external":
                        self._send_text("External fingerprint report ignored (internal backend)")
                        return
                    if mode == "additive":
                        self._send_text("External fingerprint report received (additive)")
                        return
                    if mode == "authoritative" and payload.get("action") == "deny":
                        state.banned_ips.add(ip)
                        self._send_text("External fingerprint automation detected - banned")
                        return
                    self._send_text("External fingerprint report received")
                    return

                self._send_text("Not Found", 404)

            def log_message(self, fmt, *args):
                return

        return Handler

    def test_live_remote_edge_signal_smoke_restores_config_after_success(self) -> None:
        result = subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "--env-file",
                str(self.env_file),
                "--receipts-dir",
                str(self.receipts_dir),
                "--report-path",
                str(self.report_path),
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, msg=result.stdout + result.stderr)

        report = json.loads(self.report_path.read_text(encoding="utf-8"))
        self.assertEqual(report["remote"]["name"], "stub-remote")
        self.assertEqual(report["remote"]["base_url"], self.base_url)
        self.assertEqual(
            [check["ok"] for check in report["checks"]],
            [True, True, True, True, True],
        )
        self.assertEqual(self.state.config, self.state.original_config)
        self.assertEqual(
            self.state.unban_calls,
            ["10.0.0.231"],
        )

    def test_auto_transport_prefers_ssh_loopback_for_real_remote_receipts(self) -> None:
        private_key = self.temp_dir / "id_ed25519"
        private_key.write_text("dummy", encoding="utf-8")
        receipt_path = self.receipts_dir / "stub-remote.json"
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        receipt["ssh"]["host"] = "198.51.100.25"
        receipt["ssh"]["private_key_path"] = str(private_key)
        receipt["runtime"]["public_base_url"] = "https://remote.example.com"
        receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")

        runner = REMOTE_EDGE_SIGNAL_SMOKE.RemoteEdgeSignalSmoke(
            env_file=self.env_file,
            receipts_dir=self.receipts_dir,
            remote_name="stub-remote",
            report_path=self.report_path,
        )

        self.assertEqual(runner.transport_mode, "ssh_loopback")


if __name__ == "__main__":
    unittest.main()
