#!/usr/bin/env python3

from __future__ import annotations

import http.server
import json
import socketserver
import subprocess
import tempfile
import threading
import time
import unittest
from pathlib import Path
from typing import Any

import scripts.tests.adversarial_simulation_runner as sim_runner
import scripts.tests.shared_host_scope as shared_host_scope
import scripts.tests.shared_host_seed_inventory as shared_host_seed_inventory

try:
    import scripts.supervisor.scrapling_worker as scrapling_worker
except ModuleNotFoundError:  # TDD red phase before implementation lands.
    scrapling_worker = None


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "supervisor" / "scrapling_worker.py"
SIM_SECRET = "a" * 64


class _RecordingServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
    allow_reuse_address = True

    def __init__(self, server_address: tuple[str, int], handler_class):
        super().__init__(server_address, handler_class)
        self.requests_seen: list[dict[str, Any]] = []

    def server_bind(self) -> None:
        socketserver.TCPServer.server_bind(self)
        host, port = self.server_address[:2]
        self.server_name = str(host)
        self.server_port = int(port)


class _RecordingHandler(http.server.BaseHTTPRequestHandler):
    server: _RecordingServer

    def log_message(self, format: str, *args) -> None:  # noqa: A003
        return

    def _record(self) -> None:
        self.server.requests_seen.append(
            {
                "path": self.path,
                "headers": {key.lower(): value for key, value in self.headers.items()},
            }
        )

    def do_GET(self) -> None:  # noqa: N802
        self._record()
        if self.path == "/":
            body = (
                "<html><body>"
                '<a href="/page">page</a>'
                '<a href="/redirect-out">redirect</a>'
                '<a href="http://evil.example/outside">outside</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/page":
            body = b"<html><body>page</body></html>"
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/redirect-out":
            self.send_response(302)
            self.send_header("Location", "http://evil.example/escape")
            self.end_headers()
            return
        self.send_response(404)
        self.end_headers()


class ScraplingWorkerUnitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="scrapling-worker-test-"))
        self.httpd = _RecordingServer(("127.0.0.1", 0), _RecordingHandler)
        self.server_thread = threading.Thread(target=self.httpd.serve_forever, daemon=True)
        self.server_thread.start()
        self.base_url = f"http://127.0.0.1:{self.httpd.server_port}/"

        descriptor_payload = {
            "allowed_hosts": [f"127.0.0.1:{self.httpd.server_port}"],
            "denied_path_prefixes": ["/admin"],
            "require_https": False,
            "deny_ip_literals": False,
        }
        self.descriptor_path = self.temp_dir / "scope.json"
        self.descriptor_path.write_text(json.dumps(descriptor_payload), encoding="utf-8")
        self.descriptor = shared_host_scope.descriptor_from_payload(descriptor_payload)

        inventory = shared_host_seed_inventory.build_seed_inventory(
            self.descriptor,
            primary_start_url=self.base_url,
        )
        self.inventory_path = self.temp_dir / "seed_inventory.json"
        self.inventory_path.write_text(json.dumps(inventory), encoding="utf-8")
        self.crawldir = self.temp_dir / "crawldir"

        self.beat_payload = {
            "dispatch_mode": "scrapling_worker",
            "worker_plan": {
                "schema_version": "adversary-sim-scrapling-worker-plan.v1",
                "run_id": "simrun-scrapling-test",
                "tick_id": "tick-001",
                "lane": "scrapling_traffic",
                "sim_profile": "scrapling_runtime_lane",
                "tick_started_at": int(time.time()),
                "max_requests": 4,
                "max_depth": 2,
                "max_bytes": 65536,
                "max_ms": 4000,
            },
        }

    def tearDown(self) -> None:
        self.httpd.shutdown()
        self.httpd.server_close()
        self.server_thread.join(timeout=2)

    def test_execute_worker_plan_emits_signed_real_scrapling_requests_and_blocks_out_of_scope_targets(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        result = scrapling_worker.execute_worker_plan(
            self.beat_payload,
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )
        self.assertEqual(result["lane"], "scrapling_traffic")
        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertGreaterEqual(result["generated_requests"], 2, msg=json.dumps(result, indent=2))
        self.assertEqual(result["scope_rejections"]["host_not_allowed"], 1)
        self.assertEqual(result["scope_rejections"]["redirect_target_out_of_scope"], 1)
        self.assertIn("/", [entry["path"] for entry in self.httpd.requests_seen])
        self.assertIn("/page", [entry["path"] for entry in self.httpd.requests_seen])
        for entry in self.httpd.requests_seen:
            headers = entry["headers"]
            self.assertNotIn("authorization", headers)
            self.assertNotIn("x-forwarded-for", headers)
            self.assertNotIn("x-forwarded-proto", headers)
            self.assertNotIn("x-shuma-forwarded-secret", headers)
            self.assertNotIn("x-shuma-internal-supervisor", headers)
            self.assertEqual(
                headers.get(sim_runner.SIM_TAG_HEADER_RUN_ID),
                "simrun-scrapling-test",
            )
            self.assertEqual(
                headers.get(sim_runner.SIM_TAG_HEADER_PROFILE),
                "scrapling_runtime_lane",
            )
            self.assertEqual(
                headers.get(sim_runner.SIM_TAG_HEADER_LANE),
                "scrapling_traffic",
            )
            self.assertTrue(headers.get(sim_runner.SIM_TAG_HEADER_TIMESTAMP))
            self.assertTrue(headers.get(sim_runner.SIM_TAG_HEADER_NONCE))
            self.assertTrue(headers.get(sim_runner.SIM_TAG_HEADER_SIGNATURE))

    def test_cli_writes_result_file_for_scrapling_worker_plan(self) -> None:
        beat_path = self.temp_dir / "beat.json"
        beat_path.write_text(json.dumps(self.beat_payload), encoding="utf-8")
        result_path = self.temp_dir / "result.json"

        proc = subprocess.run(
            [
                str(REPO_ROOT / ".venv-scrapling" / "bin" / "python3"),
                str(SCRIPT),
                "--beat-response-file",
                str(beat_path),
                "--result-output-file",
                str(result_path),
                "--scope-descriptor",
                str(self.descriptor_path),
                "--seed-inventory",
                str(self.inventory_path),
                "--crawldir",
                str(self.crawldir),
            ],
            cwd=str(REPO_ROOT),
            env={
                "PATH": str(REPO_ROOT / ".venv-scrapling" / "bin"),
                "SHUMA_SIM_TELEMETRY_SECRET": SIM_SECRET,
            },
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(proc.returncode, 0, msg=proc.stderr or proc.stdout)
        payload = json.loads(result_path.read_text(encoding="utf-8"))
        self.assertEqual(payload["lane"], "scrapling_traffic")
        self.assertGreaterEqual(payload["generated_requests"], 2, msg=json.dumps(payload, indent=2))


if __name__ == "__main__":
    unittest.main()
