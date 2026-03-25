#!/usr/bin/env python3

from __future__ import annotations

import http.client
import http.server
import json
import os
import socketserver
import subprocess
import tempfile
import threading
import time
import unittest
import unittest.mock
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urljoin, urlsplit

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
        body = b""
        length = int(self.headers.get("content-length") or "0")
        if length > 0:
            body = self.rfile.read(length)
        self.server.requests_seen.append(
            {
                "method": self.command,
                "path": self.path,
                "headers": {key.lower(): value for key, value in self.headers.items()},
                "body": body.decode("utf-8", errors="replace"),
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
        if self.path.startswith("/catalog"):
            parsed = urlsplit(self.path)
            page = parse_qs(parsed.query).get("page", ["1"])[0]
            next_link = ""
            if page == "1":
                next_link = '<a href="/catalog?page=2">next</a><a href="/detail/1">detail</a>'
            elif page == "2":
                next_link = '<a href="/detail/2">detail</a>'
            body = f"<html><body>catalog-{page}{next_link}</body></html>".encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path in {"/detail/1", "/detail/2"}:
            body = json.dumps({"path": self.path, "kind": "detail"}).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/redirect-out":
            self.send_response(302)
            self.send_header("Location", "http://evil.example/escape")
            self.end_headers()
            return
        if self.path.startswith("/agent/ping"):
            body = json.dumps({"ok": True, "path": self.path}).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/agent/redirect":
            self.send_response(302)
            self.send_header("Location", "/agent/final")
            self.end_headers()
            return
        if self.path == "/agent/final":
            body = json.dumps({"ok": True, "path": self.path}).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path.startswith("/sim/public/search"):
            body = (
                "<html><body>"
                '<a href="/challenge/not-a-bot-checkbox">not-a-bot</a>'
                '<a href="/pow">pow</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/instaban":
            body = b"<html><body>honeypot tripped</body></html>"
            self.send_response(403)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/pow":
            body = b"<html><body>pow challenge</body></html>"
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(404)
        self.end_headers()

    def do_POST(self) -> None:  # noqa: N802
        self._record()
        if self.path == "/agent/submit":
            body = json.dumps({"accepted": True}).encode("utf-8")
            self.send_response(201)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/challenge/not-a-bot-checkbox":
            body = json.dumps({"accepted": False, "reason": "invalid_seed"}).encode("utf-8")
            self.send_response(403)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/challenge/puzzle":
            body = json.dumps({"accepted": False, "reason": "invalid_output"}).encode("utf-8")
            self.send_response(403)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/pow/verify":
            body = json.dumps({"accepted": False, "reason": "invalid_proof"}).encode("utf-8")
            self.send_response(403)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(404)
        self.end_headers()

    def do_PUT(self) -> None:  # noqa: N802
        self._record()
        if self.path == "/agent/update":
            body = json.dumps({"updated": True}).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(404)
        self.end_headers()


class _GeoProxyServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
    allow_reuse_address = True

    def __init__(
        self,
        server_address: tuple[str, int],
        handler_class,
        *,
        upstream_host: str,
        upstream_port: int,
        forwarded_secret: str,
        forwarded_for: str,
        geo_country: str,
    ):
        super().__init__(server_address, handler_class)
        self.requests_seen: list[dict[str, Any]] = []
        self.upstream_host = upstream_host
        self.upstream_port = upstream_port
        self.forwarded_secret = forwarded_secret
        self.forwarded_for = forwarded_for
        self.geo_country = geo_country

    def server_bind(self) -> None:
        socketserver.TCPServer.server_bind(self)
        host, port = self.server_address[:2]
        self.server_name = str(host)
        self.server_port = int(port)


class _GeoProxyHandler(http.server.BaseHTTPRequestHandler):
    server: _GeoProxyServer

    def log_message(self, format: str, *args) -> None:  # noqa: A003
        return

    def _proxy(self) -> None:
        body = b""
        length = int(self.headers.get("content-length") or "0")
        if length > 0:
            body = self.rfile.read(length)
        self.server.requests_seen.append(
            {
                "method": self.command,
                "path": self.path,
                "headers": {key.lower(): value for key, value in self.headers.items()},
                "body": body.decode("utf-8", errors="replace"),
            }
        )

        parsed = urlsplit(self.path)
        upstream_path = parsed.path or "/"
        if parsed.query:
            upstream_path = f"{upstream_path}?{parsed.query}"
        headers = {key: value for key, value in self.headers.items()}
        for hop_header in ("Host", "Proxy-Connection", "Connection"):
            headers.pop(hop_header, None)
        headers["Host"] = f"{self.server.upstream_host}:{self.server.upstream_port}"
        headers["X-Forwarded-For"] = self.server.forwarded_for
        headers["X-Shuma-Forwarded-Secret"] = self.server.forwarded_secret
        headers["X-Geo-Country"] = self.server.geo_country

        conn = http.client.HTTPConnection(
            self.server.upstream_host,
            self.server.upstream_port,
            timeout=5,
        )
        try:
            conn.request(self.command, upstream_path, body=body or None, headers=headers)
            upstream_response = conn.getresponse()
            upstream_body = upstream_response.read()
            self.send_response(upstream_response.status)
            for key, value in upstream_response.getheaders():
                if key.lower() in {"transfer-encoding", "connection"}:
                    continue
                self.send_header(key, value)
            self.end_headers()
            self.wfile.write(upstream_body)
        finally:
            conn.close()

    def do_GET(self) -> None:  # noqa: N802
        self._proxy()

    def do_POST(self) -> None:  # noqa: N802
        self._proxy()


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

        self.beat_payload = self._make_beat_payload("crawler", ["indexing_bot"])

    def _make_beat_payload(
        self,
        fulfillment_mode: str,
        category_targets: list[str],
        *,
        max_requests: int = 4,
    ) -> dict[str, Any]:
        return {
            "dispatch_mode": "scrapling_worker",
            "worker_plan": {
                "schema_version": "adversary-sim-scrapling-worker-plan.v1",
                "run_id": "simrun-scrapling-test",
                "tick_id": "tick-001",
                "lane": "scrapling_traffic",
                "sim_profile": "scrapling_runtime_lane",
                "fulfillment_mode": fulfillment_mode,
                "category_targets": category_targets,
                "tick_started_at": int(time.time()),
                "max_requests": max_requests,
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
        self.assertEqual(result["fulfillment_mode"], "crawler")
        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertGreaterEqual(result["generated_requests"], 2, msg=json.dumps(result, indent=2))
        self.assertEqual(result["scope_rejections"]["host_not_allowed"], 1)
        self.assertEqual(result["scope_rejections"]["redirect_target_out_of_scope"], 1)
        self.assertIn("/", [entry["path"] for entry in self.httpd.requests_seen])
        self.assertIn("/page", [entry["path"] for entry in self.httpd.requests_seen])
        for entry in self.httpd.requests_seen:
            self.assertEqual(entry["method"], "GET")
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
                "scrapling_runtime_lane.crawler",
            )
            self.assertEqual(
                headers.get(sim_runner.SIM_TAG_HEADER_LANE),
                "scrapling_traffic",
            )
            self.assertTrue(headers.get(sim_runner.SIM_TAG_HEADER_TIMESTAMP))
            self.assertTrue(headers.get(sim_runner.SIM_TAG_HEADER_NONCE))
            self.assertTrue(headers.get(sim_runner.SIM_TAG_HEADER_SIGNATURE))

    def test_execute_worker_plan_bulk_scraper_fetches_pagination_targets(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "bulk_scraper",
            ["ai_scraper_bot"],
            max_requests=5,
        )
        result = scrapling_worker.execute_worker_plan(
            beat_payload,
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["lane"], "scrapling_traffic")
        self.assertEqual(result["fulfillment_mode"], "bulk_scraper")
        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertGreaterEqual(result["generated_requests"], 3, msg=json.dumps(result, indent=2))
        paths = [entry["path"] for entry in self.httpd.requests_seen]
        self.assertIn("/catalog?page=1", paths)
        self.assertIn("/catalog?page=2", paths)
        self.assertTrue(any(path.startswith("/detail/") for path in paths))
        self.assertTrue(all(entry["method"] == "GET" for entry in self.httpd.requests_seen))
        self.assertTrue(
            all(
                entry["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE)
                == "scrapling_runtime_lane.bulk_scraper"
                for entry in self.httpd.requests_seen
            )
        )

    def test_execute_worker_plan_http_agent_hits_owned_surfaces_with_hostile_request_native_traffic(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "http_agent",
            ["http_agent"],
            max_requests=8,
        )
        result = scrapling_worker.execute_worker_plan(
            beat_payload,
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["lane"], "scrapling_traffic")
        self.assertEqual(result["fulfillment_mode"], "http_agent")
        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertEqual(
            result["surface_interactions"],
            {
                "challenge_puzzle": 1,
                "challenge_routing": 1,
                "honeypot": 1,
                "not_a_bot": 1,
                "proof_of_work": 1,
                "rate_limit": 1,
            },
        )
        coverage_contract = json.loads(
            (
                REPO_ROOT / "scripts" / "tests" / "adversarial" / "coverage_contract.v2.json"
            ).read_text(encoding="utf-8")
        )
        owned_surfaces = set(
            coverage_contract["scrapling_owned_defense_surfaces"]["surfaces"].keys()
        )
        observed_surfaces = set(result["surface_interactions"].keys())
        self.assertEqual(sorted(owned_surfaces - observed_surfaces), ["geo_ip_policy"])
        self.assertEqual(sorted(observed_surfaces - owned_surfaces), [])
        methods = [entry["method"] for entry in self.httpd.requests_seen]
        self.assertIn("GET", methods)
        self.assertIn("POST", methods)
        paths = [entry["path"] for entry in self.httpd.requests_seen]
        self.assertIn("/sim/public/search?q=challenge-pressure", paths)
        self.assertIn("/sim/public/search?q=rate-pressure", paths)
        self.assertIn("/instaban", paths)
        self.assertIn("/challenge/not-a-bot-checkbox", paths)
        self.assertIn("/challenge/puzzle", paths)
        self.assertIn("/pow/verify", paths)
        honeypot = next(entry for entry in self.httpd.requests_seen if entry["path"] == "/instaban")
        self.assertEqual(honeypot["method"], "GET")
        self.assertIn("shuma_agent_mode=http_agent", honeypot["headers"].get("cookie", ""))
        not_a_bot_submit = next(
            entry for entry in self.httpd.requests_seen if entry["path"] == "/challenge/not-a-bot-checkbox"
        )
        self.assertEqual(
            not_a_bot_submit["headers"].get("content-type"),
            "application/x-www-form-urlencoded",
        )
        self.assertIn("seed=invalid-seed", not_a_bot_submit["body"])
        self.assertIn("checked=1", not_a_bot_submit["body"])
        challenge_submit = next(
            entry for entry in self.httpd.requests_seen if entry["path"] == "/challenge/puzzle"
        )
        self.assertEqual(
            challenge_submit["headers"].get("content-type"),
            "application/x-www-form-urlencoded",
        )
        self.assertIn("seed=invalid-seed", challenge_submit["body"])
        self.assertIn("output=0000000000000000", challenge_submit["body"])
        pow_submit = next(entry for entry in self.httpd.requests_seen if entry["path"] == "/pow/verify")
        self.assertEqual(
            pow_submit["headers"].get("content-type"),
            "application/json",
        )
        self.assertIn('"seed":"invalid-seed"', pow_submit["body"])
        self.assertIn('"nonce":"invalid-nonce"', pow_submit["body"])
        self.assertTrue(
            all(
                entry["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE)
                == "scrapling_runtime_lane.http_agent"
                for entry in self.httpd.requests_seen
            )
        )

    def test_execute_worker_plan_http_agent_can_use_public_proxy_identity_to_touch_geo_ip_policy(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "http_agent",
            ["http_agent"],
            max_requests=9,
        )
        beat_payload["worker_plan"]["public_network_identity"] = {
            "identity_id": "proxy-br",
            "identity_class": "http_proxy",
            "expected_geo_country": "BR",
        }

        proxy = _GeoProxyServer(
            ("127.0.0.1", 0),
            _GeoProxyHandler,
            upstream_host="127.0.0.1",
            upstream_port=self.httpd.server_port,
            forwarded_secret="test-forwarded-secret",
            forwarded_for="203.0.113.10",
            geo_country="BR",
        )
        proxy_thread = threading.Thread(target=proxy.serve_forever, daemon=True)
        proxy_thread.start()
        proxy_url = f"http://127.0.0.1:{proxy.server_port}"

        try:
            with unittest.mock.patch.dict(
                os.environ,
                {
                    "ADVERSARY_SIM_SCRAPLING_PUBLIC_NETWORK_IDENTITIES": json.dumps(
                        [
                            {
                                "identity_id": "proxy-br",
                                "identity_class": "http_proxy",
                                "proxy_url": proxy_url,
                                "expected_geo_country": "BR",
                            }
                        ]
                    )
                },
                clear=False,
            ):
                result = scrapling_worker.execute_worker_plan(
                    beat_payload,
                    scope_descriptor_path=self.descriptor_path,
                    seed_inventory_path=self.inventory_path,
                    crawldir=self.crawldir,
                    sim_telemetry_secret=SIM_SECRET,
                )
        finally:
            proxy.shutdown()
            proxy.server_close()
            proxy_thread.join(timeout=2)

        self.assertEqual(result["lane"], "scrapling_traffic")
        self.assertEqual(result["fulfillment_mode"], "http_agent")
        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertEqual(result["surface_interactions"].get("geo_ip_policy"), 1)
        self.assertEqual(
            result["used_public_network_identity"],
            {
                "identity_id": "proxy-br",
                "identity_class": "http_proxy",
                "expected_geo_country": "BR",
            },
        )
        self.assertEqual(
            result["surface_identity_receipts"],
            [
                {
                    "surface_id": "geo_ip_policy",
                    "identity_id": "proxy-br",
                    "identity_class": "http_proxy",
                    "expected_geo_country": "BR",
                }
            ],
        )
        self.assertIn("/", [entry["path"] for entry in self.httpd.requests_seen])
        self.assertTrue(proxy.requests_seen)
        for entry in proxy.requests_seen:
            headers = entry["headers"]
            self.assertNotIn("x-shuma-forwarded-secret", headers)
            self.assertNotIn("x-geo-country", headers)
            self.assertNotIn("x-shuma-internal-supervisor", headers)
        origin_headers = self.httpd.requests_seen[0]["headers"]
        self.assertEqual(origin_headers.get("x-shuma-forwarded-secret"), "test-forwarded-secret")
        self.assertEqual(origin_headers.get("x-geo-country"), "BR")
        self.assertEqual(origin_headers.get("x-forwarded-for"), "203.0.113.10")

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
