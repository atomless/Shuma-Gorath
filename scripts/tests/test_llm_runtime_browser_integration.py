#!/usr/bin/env python3

import os
import http.server
import json
import socketserver
import threading
import unittest
from unittest import mock

from scripts.supervisor import llm_runtime_worker
from scripts.tests.adversarial_runner.contracts import resolve_lane_realism_profile


class _BrowserTrafficServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
    allow_reuse_address = True


class _BrowserTrafficHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):  # noqa: A003
        return

    def do_GET(self):  # noqa: N802
        if self.path == "/":
            body = (
                "<html><head>"
                '<link rel="stylesheet" href="/static/site.css"/>'
                '<script src="/static/app.js"></script>'
                "</head><body><h1>Browser realism</h1></body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/static/site.css":
            body = b"body { font-family: serif; }"
            self.send_response(200)
            self.send_header("Content-Type", "text/css; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/static/app.js":
            body = (
                "fetch('/api/feed', {headers: {'accept': 'application/json'}}).then(r => r.text()).catch(() => null);"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/javascript; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/api/feed":
            body = json.dumps({"ok": True}).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/robots.txt":
            body = b"User-agent: *\nAllow: /\nSitemap: /sitemap.xml\n"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/sitemap.xml":
            host = str(self.headers.get("Host") or f"127.0.0.1:{self.server.server_port}").strip()
            body = (
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
                "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"
                f"<url><loc>http://{host}/</loc></url>"
                "</urlset>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/xml; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(404)
        self.end_headers()


class LlmRuntimeBrowserIntegrationTests(unittest.TestCase):
    def test_run_browser_mode_blackbox_receipts_secondary_background_and_subresource_traffic(self):
        server = _BrowserTrafficServer(("127.0.0.1", 0), _BrowserTrafficHandler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        base_url = f"http://127.0.0.1:{server.server_port}"
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-browser-secondary",
            "tick_id": "llm-browser-secondary-1",
            "tick_started_at": 1_700_000_500,
            "lane": "bot_red_team",
            "fulfillment_mode": "browser_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["browser_agent"],
            "capability_envelope": {"max_actions": 2, "max_time_budget_seconds": 90},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "browser_mode"),
        }
        generation = {
            "generation_source": "fallback_provider_unavailable",
            "provider": "",
            "model_id": "",
            "fallback_reason": "integration_browser_session",
            "actions": [
                {
                    "action_index": 1,
                    "action_type": "browser_navigate",
                    "path": "/",
                    "label": "root",
                }
            ],
        }

        try:
            with mock.patch.dict(
                os.environ,
                {
                    **os.environ,
                    "SHUMA_SIM_TELEMETRY_SECRET": str(
                        os.environ.get("SHUMA_SIM_TELEMETRY_SECRET")
                        or "browser-mode-live-test-secret"
                    ),
                },
                clear=True,
            ):
                report = llm_runtime_worker.run_browser_mode_blackbox(
                    base_url=base_url,
                    fulfillment_plan=plan,
                    generation_result=generation,
                )
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)

        self.assertTrue(report["passed"], report)
        realism_receipt = report["worker_payload"]["realism_receipt"]
        self.assertEqual(realism_receipt["secondary_capture_mode"], "same_origin_request_events")
        self.assertGreaterEqual(realism_receipt["secondary_request_count"], 3)
        self.assertGreaterEqual(realism_receipt["background_request_count"], 1)
        self.assertGreaterEqual(realism_receipt["subresource_request_count"], 2)
        browser_evidence = report["worker_payload"]["browser_evidence"]
        self.assertEqual(
            browser_evidence["secondary_traffic"]["secondary_request_count"],
            realism_receipt["secondary_request_count"],
        )

    def test_run_browser_mode_blackbox_emits_real_session_traffic_against_local_public_site(self):
        base_url = str(os.environ.get("SHUMA_BASE_URL") or "http://127.0.0.1:3000").strip()
        plan = {
            "schema_version": "adversary-sim-llm-fulfillment-plan.v1",
            "run_id": "simrun-llm-browser-integration",
            "tick_id": "llm-browser-live-1",
            "tick_started_at": 1_700_000_500,
            "lane": "bot_red_team",
            "fulfillment_mode": "browser_mode",
            "backend_kind": "frontier_reference",
            "backend_state": "configured",
            "category_targets": ["browser_agent"],
            "capability_envelope": {"max_actions": 4, "max_time_budget_seconds": 90},
            "realism_profile": resolve_lane_realism_profile("bot_red_team", "browser_mode"),
        }
        generation = {
            "generation_source": "fallback_provider_unavailable",
            "provider": "",
            "model_id": "",
            "fallback_reason": "integration_browser_session",
            "actions": [
                {
                    "action_index": 1,
                    "action_type": "browser_navigate",
                    "path": "/",
                    "label": "root",
                }
            ],
        }

        with mock.patch.dict(
            os.environ,
            {
                **os.environ,
                "SHUMA_SIM_TELEMETRY_SECRET": str(
                    os.environ.get("SHUMA_SIM_TELEMETRY_SECRET")
                    or "browser-mode-live-test-secret"
                ),
            },
            clear=True,
        ):
            report = llm_runtime_worker.run_browser_mode_blackbox(
                base_url=base_url,
                fulfillment_plan=plan,
                generation_result=generation,
            )

        self.assertTrue(report["passed"], report)
        worker_payload = report["worker_payload"]
        receipts = worker_payload["traffic"]
        self.assertGreaterEqual(worker_payload["requests_sent"], 2, receipts)
        self.assertEqual(receipts[0]["path"], "/")
        self.assertTrue(any(row["path"] != "/" for row in receipts), receipts)

        realism_receipt = worker_payload["realism_receipt"]
        self.assertEqual(realism_receipt["profile_id"], "agentic.browser_mode.v1")
        self.assertEqual(realism_receipt["top_level_action_count"], len(receipts))
        self.assertIn("secondary_request_count", realism_receipt)
        self.assertIn("background_request_count", realism_receipt)
        self.assertIn("subresource_request_count", realism_receipt)
        self.assertEqual(
            len(realism_receipt["dwell_intervals_ms"]),
            max(0, realism_receipt["top_level_action_count"] - 1),
        )
        self.assertEqual(realism_receipt["identity_rotation_count"], 0)
        self.assertEqual(realism_receipt["recurrence_strategy"], "bounded_campaign_return")
        self.assertEqual(realism_receipt["reentry_scope"], "cross_window_campaign")
        self.assertEqual(
            realism_receipt["dormancy_truth_mode"],
            "accelerated_local_proof",
        )
        self.assertEqual(realism_receipt["session_index"], 1)
        self.assertEqual(realism_receipt["reentry_count"], 0)
        self.assertGreaterEqual(realism_receipt["max_reentries_per_run"], 1)
        self.assertGreaterEqual(realism_receipt["planned_dormant_gap_seconds"], 1)
        self.assertGreaterEqual(
            realism_receipt["representative_dormant_gap_seconds"],
            3_600,
        )
        self.assertGreater(
            realism_receipt["representative_dormant_gap_seconds"],
            realism_receipt["planned_dormant_gap_seconds"],
        )

        browser_evidence = worker_payload["browser_evidence"]
        self.assertEqual(browser_evidence["driver_runtime"], "playwright_chromium")
        self.assertTrue(browser_evidence["js_executed"])
        self.assertGreaterEqual(len(browser_evidence["request_lineage"]), len(receipts))
        self.assertIn("secondary_traffic", browser_evidence)


if __name__ == "__main__":
    unittest.main()
