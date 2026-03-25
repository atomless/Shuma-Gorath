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
        if self.path.startswith("/sim/public/search"):
            body = json.dumps({"ok": True, "path": self.path, "kind": "search"}).encode("utf-8")
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
        if self.path == "/challenge/not-a-bot-checkbox":
            body = (
                "<html><body>"
                '<form action="/challenge/not-a-bot-checkbox" method="post">'
                '<input id="not-a-bot-seed" name="seed" value="seed-token"/>'
                '<input id="not-a-bot-checked" name="checked" value="0"/>'
                '<input id="not-a-bot-telemetry" name="telemetry" value=""/>'
                '<button id="not-a-bot-checkbox" type="button">I am not a bot</button>'
                "</form>"
                "<script>"
                "const form=document.querySelector('form');"
                "const checkbox=document.getElementById('not-a-bot-checkbox');"
                "const checked=document.getElementById('not-a-bot-checked');"
                "const telemetry=document.getElementById('not-a-bot-telemetry');"
                "checkbox.addEventListener('click', function(event) {"
                "checked.value='1';"
                "telemetry.value=JSON.stringify({activation_trusted:event.isTrusted,activation_count:1});"
                "form.requestSubmit();"
                "});"
                "</script>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/challenge/puzzle":
            body = (
                "<html><body>"
                "<h2>Puzzle</h2>"
                '<div id="challenge-output-grid"></div>'
                '<form action="/challenge/puzzle" method="post">'
                '<input name="seed" value="seed-token"/>'
                '<input id="challenge-output" name="output" value="0000"/>'
                '<button type="submit">Submit</button>'
                "</form>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
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
            parsed = parse_qs((self.server.requests_seen[-1]["body"] if self.server.requests_seen else ""))
            if parsed.get("checked") == ["1"] and parsed.get("telemetry"):
                self.send_response(303)
                self.send_header("Location", "/agent/final")
                self.send_header("Set-Cookie", "shuma_not_a_bot=1; Path=/")
                self.end_headers()
            else:
                body = json.dumps({"accepted": False, "outcome": "fail"}).encode("utf-8")
                self.send_response(400)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            return
        if self.path == "/challenge/puzzle":
            parsed = parse_qs((self.server.requests_seen[-1]["body"] if self.server.requests_seen else ""))
            if parsed.get("output"):
                body = b"<html><body><div data-link-kind=\"maze\">maze</div></body></html>"
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            else:
                body = json.dumps({"accepted": False, "outcome": "rejected"}).encode("utf-8")
                self.send_response(400)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            return
        if self.path == "/pow/verify":
            body = json.dumps({"verified": False}).encode("utf-8")
            self.send_response(400)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/tarpit/progress":
            body = json.dumps({"accepted": False}).encode("utf-8")
            self.send_response(400)
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
        max_requests: int = 5,
    ) -> dict[str, Any]:
        mode_surface_targets = {
            "crawler": [
                "public_path_traversal",
                "challenge_routing",
                "rate_pressure",
                "geo_ip_policy",
            ],
            "bulk_scraper": [
                "public_path_traversal",
                "challenge_routing",
                "rate_pressure",
                "geo_ip_policy",
                "not_a_bot_submit",
                "puzzle_submit_or_escalation",
            ],
            "http_agent": [
                "challenge_routing",
                "rate_pressure",
                "geo_ip_policy",
                "not_a_bot_submit",
                "puzzle_submit_or_escalation",
                "pow_verify_abuse",
                "tarpit_progress_abuse",
            ],
        }
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
                "surface_targets": mode_surface_targets[fulfillment_mode],
                "runtime_paths": {
                    "public_search": "/sim/public/search",
                    "not_a_bot_checkbox": "/challenge/not-a-bot-checkbox",
                    "challenge_submit": "/challenge/puzzle",
                    "pow_verify": "/pow/verify",
                    "tarpit_progress": "/tarpit/progress",
                },
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

    def _surface_receipts_by_id(self, result: dict[str, Any]) -> dict[str, dict[str, Any]]:
        return {
            str(entry["surface_id"]): entry
            for entry in list(result.get("surface_receipts") or [])
        }

    def _surface_receipt_statuses(
        self,
        result: dict[str, Any],
        surface_id: str,
    ) -> list[str]:
        return [
            str(entry.get("coverage_status") or "")
            for entry in list(result.get("surface_receipts") or [])
            if str(entry.get("surface_id") or "") == surface_id
        ]

    def test_import_scrapling_exposes_browser_session_classes(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        fetcher_session_cls, dynamic_session_cls, stealthy_session_cls, request_cls, spider_cls = (  # type: ignore[misc, attr-defined]
            scrapling_worker._import_scrapling()
        )

        self.assertEqual(fetcher_session_cls.__name__, "FetcherSession")
        self.assertEqual(dynamic_session_cls.__name__, "DynamicSession")
        self.assertEqual(stealthy_session_cls.__name__, "StealthySession")
        self.assertEqual(request_cls.__name__, "Request")
        self.assertEqual(spider_cls.__name__, "Spider")

    def test_dynamic_session_kwargs_lock_browser_automation_profile(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        kwargs = scrapling_worker._dynamic_session_kwargs(  # type: ignore[attr-defined]
            timeout_ms=4000,
            accept_header="text/html",
        )

        self.assertEqual(kwargs["timeout"], 4000)
        self.assertEqual(kwargs["wait"], 250)
        self.assertTrue(kwargs["network_idle"])
        self.assertTrue(kwargs["disable_resources"])
        self.assertEqual(kwargs["extra_headers"]["accept"], "text/html")

    def test_stealth_session_kwargs_lock_stealth_profile(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        kwargs = scrapling_worker._stealth_session_kwargs(  # type: ignore[attr-defined]
            timeout_ms=4000,
            accept_header="application/json",
        )

        self.assertEqual(kwargs["timeout"], 4000)
        self.assertEqual(kwargs["wait"], 250)
        self.assertTrue(kwargs["network_idle"])
        self.assertTrue(kwargs["disable_resources"])
        self.assertTrue(kwargs["solve_cloudflare"])
        self.assertTrue(kwargs["block_webrtc"])
        self.assertTrue(kwargs["hide_canvas"])
        self.assertTrue(kwargs["allow_webgl"])
        self.assertEqual(kwargs["extra_headers"]["accept"], "application/json")

    def test_browser_session_strategy_classifies_current_modes(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        self.assertEqual(
            scrapling_worker._browser_session_strategy_for_mode("crawler"),  # type: ignore[attr-defined]
            "request_native",
        )
        self.assertEqual(
            scrapling_worker._browser_session_strategy_for_mode("bulk_scraper"),  # type: ignore[attr-defined]
            "dynamic",
        )
        self.assertEqual(
            scrapling_worker._browser_session_strategy_for_mode("http_agent"),  # type: ignore[attr-defined]
            "stealth",
        )

    def test_request_native_session_kwargs_lock_explicit_chrome_impersonation_contract(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        kwargs = scrapling_worker._request_native_session_kwargs(  # type: ignore[attr-defined]
            timeout_seconds=4.0,
            accept_header="application/json",
        )

        self.assertEqual(kwargs["impersonate"], "chrome")
        self.assertTrue(kwargs["stealthy_headers"])
        self.assertFalse(kwargs["follow_redirects"])
        self.assertEqual(kwargs["retries"], 1)
        self.assertEqual(kwargs["timeout"], 4.0)
        self.assertEqual(kwargs["headers"]["accept"], "application/json")
        self.assertNotIn("user-agent", {key.lower(): value for key, value in kwargs["headers"].items()})

    def test_execute_worker_plan_no_longer_advertises_internal_worker_user_agent(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        result = scrapling_worker.execute_worker_plan(
            self.beat_payload,
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertGreaterEqual(len(self.httpd.requests_seen), 1, msg=json.dumps(result, indent=2))
        for entry in self.httpd.requests_seen:
            headers = entry["headers"]
            user_agent = headers.get("user-agent", "")
            self.assertNotIn("ShumaScraplingWorker", user_agent)
            self.assertIn("Mozilla/5.0", user_agent)
            self.assertTrue(headers.get("sec-ch-ua"))

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
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["public_path_traversal"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["challenge_routing"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["rate_pressure"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["geo_ip_policy"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
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

    def test_execute_worker_plan_http_agent_uses_method_mix_and_redirect_followup(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "http_agent",
            ["http_agent"],
            max_requests=10,
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
        methods = [entry["method"] for entry in self.httpd.requests_seen]
        self.assertIn("GET", methods)
        self.assertIn("POST", methods)
        self.assertIn("PUT", methods)
        paths = [entry["path"] for entry in self.httpd.requests_seen]
        self.assertIn("/agent/redirect", paths)
        self.assertIn("/agent/final", paths)
        submit = next(entry for entry in self.httpd.requests_seen if entry["path"] == "/agent/submit")
        self.assertIn('"mode":"http_agent"', submit["body"])
        self.assertEqual(
            submit["headers"].get("content-type"),
            "application/json",
        )
        self.assertIn(
            "shuma_agent_mode=http_agent",
            submit["headers"].get("cookie", ""),
        )
        self.assertTrue(
            all(
                entry["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE)
                == "scrapling_runtime_lane.http_agent"
                for entry in self.httpd.requests_seen
            )
        )

    def test_execute_worker_plan_bulk_scraper_attempts_owned_challenge_surfaces(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "bulk_scraper",
            ["ai_scraper_bot"],
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
        self.assertEqual(result["fulfillment_mode"], "bulk_scraper")
        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["public_path_traversal"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["challenge_routing"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["rate_pressure"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["geo_ip_policy"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["not_a_bot_submit"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["puzzle_submit_or_escalation"]["coverage_status"],
            "fail_observed",
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/catalog?page=1"), paths)
        self.assertIn(("GET", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("GET", "/challenge/puzzle"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)
        not_a_bot = next(
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/challenge/not-a-bot-checkbox"
        )
        self.assertIn("checked=1", not_a_bot["body"])
        self.assertIn("telemetry=", not_a_bot["body"])
        self.assertEqual(
            not_a_bot["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE),
            "scrapling_runtime_lane.bulk_scraper",
        )
        puzzle = next(
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/challenge/puzzle"
        )
        self.assertIn("output=", puzzle["body"])
        self.assertEqual(
            puzzle["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE),
            "scrapling_runtime_lane.bulk_scraper",
        )

    def test_public_path_traversal_receipts_keep_pass_observed_when_later_public_request_fails(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        receipts: dict[str, dict[str, Any]] = {}

        scrapling_worker._record_surface_receipt(
            receipts,
            surface_ids=["public_path_traversal"],
            coverage_status="pass_observed",
            request_method="GET",
            request_target=f"{self.base_url}catalog?page=1",
            response_status=200,
        )
        scrapling_worker._record_surface_receipt(
            receipts,
            surface_ids=["public_path_traversal"],
            coverage_status="fail_observed",
            request_method="GET",
            request_target=f"{self.base_url}detail/2",
            response_status=429,
        )

        rendered = scrapling_worker._render_surface_receipts(receipts)
        public_path_receipts = [
            entry
            for entry in rendered
            if str(entry.get("surface_id") or "") == "public_path_traversal"
        ]

        self.assertEqual(len(public_path_receipts), 2)
        self.assertCountEqual(
            [entry["coverage_status"] for entry in public_path_receipts],
            ["pass_observed", "fail_observed"],
        )

    def test_execute_worker_plan_http_agent_attempts_owned_request_native_abuse_surfaces(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "http_agent",
            ["http_agent"],
            max_requests=10,
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
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["challenge_routing"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["rate_pressure"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["geo_ip_policy"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["not_a_bot_submit"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["puzzle_submit_or_escalation"]["coverage_status"],
            "fail_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["pow_verify_abuse"]["coverage_status"],
            "fail_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["tarpit_progress_abuse"]["coverage_status"],
            "fail_observed",
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/agent/ping?mode=http_agent"), paths)
        self.assertIn(("GET", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("GET", "/challenge/puzzle"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)
        self.assertIn(("POST", "/pow/verify"), paths)
        self.assertIn(("POST", "/tarpit/progress"), paths)
        not_a_bot = next(
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/challenge/not-a-bot-checkbox"
        )
        self.assertIn("telemetry=", not_a_bot["body"])
        self.assertEqual(
            not_a_bot["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE),
            "scrapling_runtime_lane.http_agent",
        )
        puzzle = next(
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/challenge/puzzle"
        )
        self.assertIn("output=", puzzle["body"])
        self.assertEqual(
            puzzle["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE),
            "scrapling_runtime_lane.http_agent",
        )
        pow_verify = next(
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/pow/verify"
        )
        self.assertIn('"seed":"invalid-seed"', pow_verify["body"])
        self.assertIn('"nonce":"invalid-nonce"', pow_verify["body"])
        tarpit = next(
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/tarpit/progress"
        )
        self.assertIn('"token":"invalid"', tarpit["body"])
        self.assertIn('"operation_id":"invalid"', tarpit["body"])

    def test_execute_worker_plan_http_agent_reaches_pow_and_tarpit_with_live_runtime_budget(self) -> None:
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

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "pow_verify_abuse"),
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "tarpit_progress_abuse"),
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("POST", "/pow/verify"), paths)
        self.assertIn(("POST", "/tarpit/progress"), paths)

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
