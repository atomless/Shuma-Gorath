#!/usr/bin/env python3

from __future__ import annotations

import http.server
import json
import os
import socketserver
import subprocess
import tempfile
import threading
import time
from types import SimpleNamespace
import unittest
from unittest import mock
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urljoin, urlsplit

import scripts.tests.adversarial_simulation_runner as sim_runner
import scripts.tests.shared_host_scope as shared_host_scope
import scripts.tests.shared_host_seed_inventory as shared_host_seed_inventory
from scripts.tests.adversarial_runner.contracts import resolve_lane_realism_profile

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
        self.root_pressure_counts: dict[str, int] = {}
        self.challenge_pressure_counts: dict[str, int] = {}

    def server_bind(self) -> None:
        socketserver.TCPServer.server_bind(self)
        host, port = self.server_address[:2]
        self.server_name = str(host)
        self.server_port = int(port)


class _RecordingHandler(http.server.BaseHTTPRequestHandler):
    server: _RecordingServer

    def log_message(self, format: str, *args) -> None:  # noqa: A003
        return

    @staticmethod
    def _not_a_bot_score_from_body(body_text: str) -> int | None:
        parsed = parse_qs(body_text, keep_blank_values=True)
        telemetry_values = parsed.get("telemetry") or []
        if not telemetry_values:
            return None
        try:
            telemetry = json.loads(telemetry_values[-1])
        except json.JSONDecodeError:
            return None
        checked = str((parsed.get("checked") or [""])[-1]).strip().lower() in {
            "1",
            "true",
            "yes",
            "on",
        }
        if not checked:
            return None
        interaction_elapsed_ms = int(telemetry.get("interaction_elapsed_ms") or 0)
        if interaction_elapsed_ms < 250 or interaction_elapsed_ms > 180_000:
            return None
        activation_count = int(telemetry.get("activation_count") or 0)
        if activation_count == 0 or activation_count > 2:
            return None
        activation_method = str(telemetry.get("activation_method") or "").strip().lower()
        if activation_method not in {"pointer", "touch", "keyboard", "unknown", ""}:
            return None
        down_up_ms = int(telemetry.get("down_up_ms") or 0)
        if down_up_ms > 0 and (down_up_ms < 25 or down_up_ms > 12_000):
            return None

        score = 1
        if interaction_elapsed_ms >= 900:
            score += 2
        elif interaction_elapsed_ms >= 500:
            score += 1

        if 80 <= down_up_ms <= 5000:
            score += 1

        has_pointer = bool(telemetry.get("has_pointer"))
        keyboard_used = bool(telemetry.get("keyboard_used"))
        touch_used = bool(telemetry.get("touch_used"))
        control_focused = bool(telemetry.get("control_focused"))
        activation_trusted = bool(telemetry.get("activation_trusted"))
        focus_changes = int(telemetry.get("focus_changes") or 0)
        visibility_changes = int(telemetry.get("visibility_changes") or 0)
        pointer_move_count = int(telemetry.get("pointer_move_count") or 0)
        pointer_path_length = float(telemetry.get("pointer_path_length") or 0.0)
        pointer_direction_changes = int(telemetry.get("pointer_direction_changes") or 0)
        plausible_pointer_motion = (
            2 <= pointer_move_count <= 3000
            and 8.0 <= pointer_path_length <= 80_000.0
            and 1 <= pointer_direction_changes <= 3000
        )

        if activation_method == "pointer":
            if not has_pointer:
                return None
            if plausible_pointer_motion:
                score += 3
            elif interaction_elapsed_ms >= 1200:
                score += 1
        elif activation_method == "touch":
            if not touch_used:
                return None
            if plausible_pointer_motion or interaction_elapsed_ms >= 800:
                score += 2
        elif activation_method == "keyboard":
            if not keyboard_used:
                return None
            score += 3 if control_focused else 2
        elif activation_method in {"unknown", ""}:
            if control_focused and interaction_elapsed_ms >= 900:
                score += 1

        if keyboard_used or touch_used or has_pointer:
            score += 1
        if control_focused:
            score += 1
        if focus_changes <= 3 and visibility_changes <= 1:
            score += 1
        if activation_trusted:
            score += 1
        return min(score, 10)

    @staticmethod
    def _challenge_output_kind(body_text: str) -> str | None:
        parsed = parse_qs(body_text, keep_blank_values=True)
        output_values = parsed.get("output") or []
        if not output_values:
            return None
        output = str(output_values[-1] or "")
        if output == "bad":
            return "abuse_invalid"
        if len(output) == 16 and set(output) <= {"0", "1"}:
            return "user_incorrect"
        return "other"

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
        if self.path == "/feed-root":
            body = (
                "<html><body>"
                '<nav><a href="/about/">about</a><a href="/research/">research</a><a href="/plans/">plans</a><a href="/work/">work</a></nav>'
                '<main><a href="/page/2/">older</a><a href="/research/2026-03-30-gap-review/">gap-review</a></main>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/page/2/":
            body = (
                "<html><body>"
                '<a href="/page/3/">older</a>'
                '<a href="/plans/2026-03-30-gap-plan/">gap-plan</a>'
                '<a href="/work/2026-03-31-realism-fix/">completed-work</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/page/3/":
            body = (
                "<html><body>"
                '<a href="/research/2026-03-31-defence-proof/">defence-proof</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path in {
            "/about/",
            "/research/",
            "/plans/",
            "/work/",
            "/research/2026-03-30-gap-review/",
            "/research/2026-03-31-defence-proof/",
            "/plans/2026-03-30-gap-plan/",
            "/work/2026-03-31-realism-fix/",
        }:
            body = (
                f"<html><body>{self.path}"
                '<a href="/page/2/">archive</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/challenged-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                '<input type="hidden" name="seed" value="seed" />'
                '<input type="hidden" name="output" value="0000000000000000" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/rate-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="rate-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/geo-block-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                "<h1>Blocked by regional access policy</h1>"
                '<a href="/geo-policy/help">help</a>'
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(403)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/geo-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="geo-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/opaque-maze-root":
            body = (
                "<html><body>"
                '<script id="maze-bootstrap" type="application/json">'
                '{"path_prefix":"/_/geo-policy/"}'
                "</script>"
                '<a data-link-kind="maze" href="/_/geo-policy/opaque-next">continue</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/_/geo-policy/opaque-next":
            body = (
                "<html><body>"
                '<script id="maze-bootstrap" type="application/json">'
                '{"path_prefix":"/_/geo-policy/","node":"opaque-next"}'
                "</script>"
                '<a data-link-kind="maze" href="/_/geo-policy/opaque-next-2">continue</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/_/geo-policy/opaque-next-2":
            body = (
                "<html><body>"
                '<script id="maze-bootstrap" type="application/json">'
                '{"path_prefix":"/_/geo-policy/","node":"opaque-next-2"}'
                "</script>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/runtime-tarpit-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="runtime-tarpit-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                '<input type="hidden" name="seed" value="runtime-tarpit-seed" />'
                '<input type="hidden" name="output" value="0000000000000000" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/html-fail-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="html-fail-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                '<input type="hidden" name="seed" value="html-fail-seed" />'
                '<input type="hidden" name="output" value="0000000000000000" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/browser-challenge-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="browser-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/browser-low-score-challenge-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="browser-low-score-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/browser-escalation-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="browser-escalation-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/pressure-root":
            current_count = int(self.server.root_pressure_counts.get(self.path, 0)) + 1
            self.server.root_pressure_counts[self.path] = current_count
            if current_count >= 6:
                body = (
                    "<html><body>"
                    "<h1>Rate Limit Exceeded</h1>"
                    "<p>Too many requests have been received from your IP address. Please try again later.</p>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(429)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Retry-After", "60")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if current_count >= 3:
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    '<h1>Please confrim you are not a bot</h1>'
                    '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                    '<input type="hidden" name="seed" value="pressure-seed" />'
                    '<input type="hidden" name="checked" value="1" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            body = (
                "<html><body>"
                '<main class="panel">'
                "<h1>Public entry</h1>"
                "<p>Nothing suspicious here yet.</p>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/high-rate-root":
            current_count = int(self.server.root_pressure_counts.get(self.path, 0)) + 1
            self.server.root_pressure_counts[self.path] = current_count
            if current_count >= 24:
                body = (
                    "<html><body>"
                    "<h1>Rate Limit Exceeded</h1>"
                    "<p>Too many requests have been received from your IP address. Please try again later.</p>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(429)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Retry-After", "60")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="high-rate-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                '<input type="hidden" name="seed" value="high-rate-seed" />'
                '<input type="hidden" name="output" value="0000000000000000" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/bulk-hostile-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="bulk-hostile-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/bulk-frontloaded-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="bulk-frontloaded-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                '<input type="hidden" name="seed" value="bulk-frontloaded-seed" />'
                '<input type="hidden" name="output" value="0000000000000000" />'
                "</form>"
                '<nav><a href="/about/">about</a><a href="/page/2/">archive</a></nav>'
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/scored-challenge-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                '<h1>Please confrim you are not a bot</h1>'
                '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                '<input type="hidden" name="seed" value="scored-seed" />'
                '<input type="hidden" name="checked" value="1" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/browser-js-then-challenge-root":
            cookie = str(self.headers.get("Cookie") or "")
            if "js_verified=1" not in cookie:
                body = (
                    "<html><head></head><body>"
                    "<script>"
                    "window._checkCDPAutomation = async function () { return { detected: false, score: 0, checks: [] }; };"
                    "document.cookie='js_verified=1; path=/';"
                    "window.location.reload();"
                    "</script>"
                    "<noscript>Please enable JS to continue.</noscript>"
                    "</body></html>"
                ).encode("utf-8")
            else:
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    '<h1>Please confrim you are not a bot</h1>'
                    '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                    '<input type="hidden" name="seed" value="browser-js-seed" />'
                    '<input type="hidden" name="checked" value="1" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/browser-js-detected-then-challenge-root":
            cookie = str(self.headers.get("Cookie") or "")
            if "js_verified=1" not in cookie:
                body = (
                    "<html><head></head><body data-detected=\"1\">"
                    "<script>"
                    "window._checkCDPAutomation = async function () { return { detected: true, score: 1.3, checks: ['webdriver', 'chrome_obj'] }; };"
                    "document.cookie='js_verified=1; path=/';"
                    "window.location.reload();"
                    "</script>"
                    "<noscript>Please enable JS to continue.</noscript>"
                    "</body></html>"
                ).encode("utf-8")
            else:
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    '<h1>Please confrim you are not a bot</h1>'
                    '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                    '<input type="hidden" name="seed" value="browser-js-detected-seed" />'
                    '<input type="hidden" name="checked" value="1" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/browser-public-explore-root":
            body = (
                "<html><body>"
                '<a href="/page/browser-public-explore/">older</a>'
                '<a href="/about/">about</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/page/browser-public-explore/":
            body = (
                "<html><head></head><body>"
                "<script>"
                "window._checkCDPAutomation = async function () { return { detected: false, score: 0, checks: [] }; };"
                "const POW_SEED = 'seed';"
                "async function solvePow(seed, difficulty) { return 'nonce'; }"
                "function showVerifying() { document.body.innerText = 'Verifying...'; }"
                "async function runPow() {"
                "  if (!window.crypto || !crypto.subtle) { return; }"
                "  const nonce = await solvePow(POW_SEED, 16);"
                "  return fetch('/pow/verify', {"
                "    method: 'POST',"
                "    headers: { 'Content-Type': 'application/json' },"
                "    body: JSON.stringify({ seed: POW_SEED, nonce: nonce })"
                "  });"
                "}"
                "</script>"
                "<noscript>Please enable JS to continue.</noscript>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/puzzle-root":
            body = (
                "<html><body>"
                '<main class="panel">'
                "<h1>Additional verification required</h1>"
                '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                '<input type="hidden" name="seed" value="seed" />'
                '<input type="hidden" name="output" value="0000000000000000" />'
                "</form>"
                "</main>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/root-js-challenge":
            body = (
                "<html><head></head><body>"
                "<script>"
                "window._checkCDPAutomation = async function () { return { detected: false, score: 0, checks: [] }; };"
                "const POW_SEED = 'seed';"
                "async function solvePow(seed, difficulty) { return 'nonce'; }"
                "function showVerifying() { document.body.innerText = 'Verifying...'; }"
                "async function runPow() {"
                "  if (!window.crypto || !crypto.subtle) { return; }"
                "  const nonce = await solvePow(POW_SEED, 16);"
                "  return fetch('/pow/verify', {"
                "    method: 'POST',"
                "    headers: { 'Content-Type': 'application/json' },"
                "    body: JSON.stringify({ seed: POW_SEED, nonce: nonce })"
                "  });"
                "}"
                "</script>"
                "<noscript>Please enable JS to continue.</noscript>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/":
            accept = str(self.headers.get("Accept") or "")
            if (
                getattr(self.server, "root_accept_variant", "") == "browserish_js_then_rate_challenge"
                and "text/html" in accept
            ):
                current_count = int(self.server.root_pressure_counts.get(self.path, 0)) + 1
                self.server.root_pressure_counts[self.path] = current_count
                if current_count >= 6:
                    body = (
                        "<html><body>"
                        "<h1>Rate Limit Exceeded</h1>"
                        "<p>Too many requests have been received from your IP address. Please try again later.</p>"
                        "</body></html>"
                    ).encode("utf-8")
                    self.send_response(429)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Retry-After", "60")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    return
                if current_count >= 3:
                    body = (
                        "<html><body>"
                        '<main class="panel">'
                        '<h1>Please confrim you are not a bot</h1>'
                        '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                        '<input type="hidden" name="seed" value="browserish-pressure-seed" />'
                        '<input type="hidden" name="checked" value="1" />'
                        "</form>"
                        "</main>"
                        "</body></html>"
                    ).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    return
                body = (
                    "<html><head></head><body>"
                    "<script>"
                    "window._checkCDPAutomation = async function () { return { detected: false, score: 0, checks: [] }; };"
                    "const POW_SEED = 'seed';"
                    "async function solvePow(seed, difficulty) { return 'nonce'; }"
                    "function showVerifying() { document.body.innerText = 'Verifying...'; }"
                    "async function runPow() {"
                    "  if (!window.crypto || !crypto.subtle) { return; }"
                    "  const nonce = await solvePow(POW_SEED, 16);"
                    "  return fetch('/pow/verify', {"
                    "    method: 'POST',"
                    "    headers: { 'Content-Type': 'application/json' },"
                    "    body: JSON.stringify({ seed: POW_SEED, nonce: nonce })"
                    "  });"
                    "}"
                    "</script>"
                    "<noscript>Please enable JS to continue.</noscript>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if (
                getattr(self.server, "root_accept_variant", "") == "browserish_js_interstitial"
                and "text/html" in accept
            ):
                body = (
                    "<html><head></head><body>"
                    "<script>"
                    "window._checkCDPAutomation = async function () { return { detected: false, score: 0, checks: [] }; };"
                    "const POW_SEED = 'seed';"
                    "async function solvePow(seed, difficulty) { return 'nonce'; }"
                    "function showVerifying() { document.body.innerText = 'Verifying...'; }"
                    "async function runPow() {"
                    "  if (!window.crypto || !crypto.subtle) { return; }"
                    "  const nonce = await solvePow(POW_SEED, 16);"
                    "  return fetch('/pow/verify', {"
                    "    method: 'POST',"
                    "    headers: { 'Content-Type': 'application/json' },"
                    "    body: JSON.stringify({ seed: POW_SEED, nonce: nonce })"
                    "  });"
                    "}"
                    "</script>"
                    "<noscript>Please enable JS to continue.</noscript>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            body = (
                "<html><body>"
                '<link rel="stylesheet" href="/static/site.css"/>'
                '<script src="/static/app.js"></script>'
                '<img src="/static/pixel.png" alt="pixel"/>'
                '<a href="/page">page</a>'
                '<a href="/timeline">timeline</a>'
                '<a href="/catalog?page=1">catalog</a>'
                '<a href="/challenge/not-a-bot-checkbox">checkpoint</a>'
                '<a href="/pow">pow</a>'
                '<a href="/maze/start">maze</a>'
                '<a href="/redirect-chain">redirect chain</a>'
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
        if self.path == "/static/site.css":
            body = b"body { color: black; }"
            self.send_response(200)
            self.send_header("Content-Type", "text/css; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/static/app.js":
            body = (
                "fetch('/browser-beacon', {method: 'POST', headers: {'content-type': 'application/json'}, "
                "body: JSON.stringify({kind: 'browser-beacon'})}).catch(() => null);"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/javascript; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/static/pixel.png":
            body = (
                b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR"
                b"\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00"
                b"\x1f\x15\xc4\x89\x00\x00\x00\rIDATx\x9cc````\x00\x00\x00\x05\x00\x01"
                b"\x0d\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82"
            )
            self.send_response(200)
            self.send_header("Content-Type", "image/png")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/page":
            body = b"<html><body>page<a href=\"/detail/1\">detail</a></body></html>"
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/timeline":
            body = (
                "<html><body>"
                '<a href="/timeline/2026/03">march-2026</a>'
                '<a href="/catalog?page=1">catalog</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/timeline/2026/03":
            body = (
                "<html><body>"
                '<a href="/timeline/2026/03/entry-1">entry-1</a>'
                '<a href="/timeline/2026/03/entry-2">entry-2</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path in {"/timeline/2026/03/entry-1", "/timeline/2026/03/entry-2"}:
            body = (
                f"<html><body>{self.path}"
                '<a href="/detail/2">detail-2</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/sitemaps/pages.xml":
            base_url = f"http://127.0.0.1:{self.server.server_port}"
            body = (
                '<?xml version="1.0" encoding="UTF-8"?>'
                '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'
                f"<url><loc>{base_url}/timeline</loc></url>"
                f"<url><loc>{base_url}/timeline/2026/03</loc></url>"
                "</urlset>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/xml; charset=utf-8")
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
        if self.path == "/" or self.path.startswith("/?"):
            body = json.dumps({"ok": True, "path": self.path, "kind": "search"}).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/redirect-chain":
            self.send_response(302)
            self.send_header("Location", "/landing-final")
            self.end_headers()
            return
        if self.path == "/landing-final":
            body = json.dumps({"ok": True, "path": self.path, "kind": "landing"}).encode("utf-8")
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
                '<input name="seed" value="seed"/>'
                "</form>"
                '<form action="/challenge/puzzle" method="post">'
                '<input name="answer" value=""/>'
                '<input name="seed" value="seed"/>'
                "</form>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/pow":
            body = (
                "<html><body>"
                '<script>'
                "window._checkCDPAutomation=function(){return document.body.dataset.detected==='1';};"
                "document.cookie='js_verified=1; path=/';"
                "</script>"
                '<div id="pow-bootstrap" data-js-verified="1">pow</div>'
                '<form action="/pow/verify" method="post">'
                '<input name="seed" value="seed"/>'
                '<input name="nonce" value="nonce"/>'
                "</form>"
                '<form action="/tarpit/progress" method="post">'
                '<input name="token" value="token"/>'
                '<input name="nonce" value="proof"/>'
                "</form>"
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/maze/start":
            body = (
                "<html><body>"
                '<div id="maze-bootstrap">start</div>'
                '<a data-link-kind="maze" href="/maze/next">next</a>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/maze/next":
            body = (
                "<html><body>"
                '<div id="maze-bootstrap">next</div>'
                "</body></html>"
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/maze/hostile-next":
            body = (
                "<html><body>"
                '<div id="maze-bootstrap">hostile-next</div>'
                '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                '<input type="hidden" name="seed" value="bulk-hostile-seed" />'
                '<input type="hidden" name="output" value="0000000000000000" />'
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
        body_text = str(self.server.requests_seen[-1]["body"] if self.server.requests_seen else "")
        if self.path == "/agent/submit":
            body = json.dumps({"accepted": True}).encode("utf-8")
            self.send_response(201)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/challenge/not-a-bot-checkbox":
            if "seed=high-rate-seed" in body_text:
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    "<h1>Additional verification required</h1>"
                    '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                    '<input type="hidden" name="seed" value="high-rate-seed" />'
                    '<input type="hidden" name="output" value="0000000000000000" />'
                    "</form>"
                    '<form id="not-a-bot-form" method="POST" action="/challenge/not-a-bot-checkbox">'
                    '<input type="hidden" name="seed" value="high-rate-seed" />'
                    '<input type="hidden" name="checked" value="1" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=bulk-hostile-seed" in body_text:
                score = self._not_a_bot_score_from_body(body_text)
                if score is not None and score <= 2:
                    body = (
                        "<html><body>"
                        '<div id="maze-bootstrap">bulk-hostile-maze</div>'
                        '<a data-link-kind="maze" href="/maze/hostile-next">next</a>'
                        '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                        '<input type="hidden" name="seed" value="bulk-hostile-seed" />'
                        '<input type="hidden" name="output" value="0000000000000000" />'
                        "</form>"
                        "</body></html>"
                    ).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    return
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    "<h1>Additional verification required</h1>"
                    '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                    '<input type="hidden" name="seed" value="bulk-hostile-seed" />'
                    '<input type="hidden" name="output" value="0000000000000000" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=scored-seed" in body_text:
                score = self._not_a_bot_score_from_body(body_text)
                if score is not None and 5 <= score < 8:
                    body = (
                        "<html><body>"
                        '<main class="panel">'
                        "<h1>Additional verification required</h1>"
                        '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                        '<input type="hidden" name="seed" value="scored-seed" />'
                        '<input type="hidden" name="output" value="0000000000000000" />'
                        "</form>"
                        "</main>"
                        "</body></html>"
                    ).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    return
                body = (
                    "<html><body>"
                    '<div id="maze-bootstrap">challenge-fail</div>'
                    '<a data-link-kind="maze" href="/maze/next">next</a>'
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=browser-escalation-seed" in body_text:
                score = self._not_a_bot_score_from_body(body_text)
                if score is not None and 5 <= score < 8:
                    body = (
                        "<html><body>"
                        '<main class="panel">'
                        "<h1>Additional verification required</h1>"
                        '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                        '<input type="hidden" name="seed" value="browser-escalation-seed" />'
                        '<input type="hidden" name="output" value="0000000000000000" />'
                        "</form>"
                        "</main>"
                        "</body></html>"
                    ).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    return
                body = (
                    "<html><body>"
                    '<div id="maze-bootstrap">challenge-fail</div>'
                    '<a data-link-kind="maze" href="/maze/next">next</a>'
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=rate-seed" in body_text:
                body = (
                    "<html><body>"
                    "<h1>Rate Limit Exceeded</h1>"
                    "<p>Too many requests have been received from your IP address. Please try again later.</p>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(429)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Retry-After", "60")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=geo-seed" in body_text:
                body = (
                    "<html><body>"
                    "<h1>Access Restricted</h1>"
                    "<p>Your request was blocked by regional access policy.</p>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(403)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=browser-seed" in body_text:
                body = (
                    "<html><body>"
                    '<script>'
                    "window._checkCDPAutomation=function(){return document.body.dataset.detected==='1';};"
                    "document.cookie='js_verified=1; path=/';"
                    "</script>"
                    '<div id="pow-bootstrap" data-js-verified="1">pow</div>'
                    '<div id="maze-bootstrap">start</div>'
                    '<a data-link-kind="maze" href="/maze/next">next</a>'
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=browser-low-score-seed" in body_text:
                score = self._not_a_bot_score_from_body(body_text)
                if score is not None and score <= 2:
                    body = (
                        "<html><body>"
                        '<div id="maze-bootstrap">low-score-maze</div>'
                        '<a data-link-kind="maze" href="/maze/next">next</a>'
                        "</body></html>"
                    ).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    return
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    "<h1>Additional verification required</h1>"
                    '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                    '<input type="hidden" name="seed" value="browser-low-score-seed" />'
                    '<input type="hidden" name="output" value="0000000000000000" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=browser-js-detected-seed" in body_text:
                body = (
                    "<html><body>"
                    '<div id="maze-bootstrap">start</div>'
                    '<a data-link-kind="maze" href="/maze/next">next</a>'
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=browser-js-seed" in body_text:
                body = (
                    "<html><body>"
                    '<div id="maze-bootstrap">start</div>'
                    '<a data-link-kind="maze" href="/maze/next">next</a>'
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=runtime-tarpit-seed" in body_text:
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    "<h1>Additional verification required</h1>"
                    '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                    '<input type="hidden" name="seed" value="runtime-tarpit-seed" />'
                    '<input type="hidden" name="output" value="0000000000000000" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=pressure-seed" in body_text:
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    "<h1>Additional verification required</h1>"
                    '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                    '<input type="hidden" name="seed" value="pressure-seed" />'
                    '<input type="hidden" name="output" value="0000000000000000" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=browserish-pressure-seed" in body_text:
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    "<h1>Additional verification required</h1>"
                    '<form id="challenge-puzzle-form" method="POST" action="/challenge/puzzle">'
                    '<input type="hidden" name="seed" value="browserish-pressure-seed" />'
                    '<input type="hidden" name="output" value="0000000000000000" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=html-fail-seed" in body_text:
                body = (
                    "<html><body>"
                    '<div id="maze-bootstrap">challenge-fail</div>'
                    '<a data-link-kind="maze" href="/maze/next">next</a>'
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            body = json.dumps({"accepted": False, "outcome": "fail"}).encode("utf-8")
            self.send_response(400)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/challenge/puzzle":
            output_kind = self._challenge_output_kind(body_text)
            if output_kind == "abuse_invalid":
                if "seed=bulk-hostile-seed" in body_text:
                    body = (
                        "<html><body>"
                        '<main class="panel">'
                        "<h1>Further verification required</h1>"
                        '<form id="tarpit-progress-form" method="POST" action="/tarpit/progress">'
                        '<input type="hidden" name="token" value="bulk-hostile-issued-token" />'
                        '<input type="hidden" name="nonce" value="bulk-hostile-issued-proof" />'
                        "</form>"
                        "</main>"
                        "</body></html>"
                    ).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    return
                if "seed=pressure-seed" in body_text:
                    body = (
                        "<html><body>"
                        '<main class="panel">'
                        "<h1>Further verification required</h1>"
                        '<form id="tarpit-progress-form" method="POST" action="/tarpit/progress">'
                        '<input type="hidden" name="token" value="pressure-issued-token" />'
                        '<input type="hidden" name="nonce" value="pressure-issued-proof" />'
                        "</form>"
                        "</main>"
                        "</body></html>"
                    ).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    return
                if "seed=runtime-tarpit-seed" in body_text:
                    body = (
                        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"></head><body>"
                        "<main><h1>Verification pending</h1>"
                        "<p data-tarpit-source=\"/challenge/puzzle\">Progress endpoint: <code>/tarpit/progress</code></p>"
                        "</main>"
                        "<script>window.__shumaTarpit={token:\"issued-token\",endpoint:\"/tarpit/progress\",difficulty:4};</script>"
                        "</body></html>"
                    ).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    return
                body = (
                    "<html><body>"
                    '<main class="panel">'
                    "<h1>Further verification required</h1>"
                    '<form id="tarpit-progress-form" method="POST" action="/tarpit/progress">'
                    '<input type="hidden" name="token" value="issued-token" />'
                    '<input type="hidden" name="nonce" value="issued-proof" />'
                    "</form>"
                    "</main>"
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if output_kind == "user_incorrect" and "seed=browser-escalation-seed" in body_text:
                body = (
                    "<html><body>"
                    '<div id="maze-bootstrap">puzzle-fail</div>'
                    '<a data-link-kind="maze" href="/maze/next">next</a>'
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if output_kind == "user_incorrect" and "seed=bulk-hostile-seed" in body_text:
                body = (
                    "<html><body>"
                    '<div id="maze-bootstrap">bulk-hostile-puzzle-fail</div>'
                    '<a data-link-kind="maze" href="/maze/hostile-next">next</a>'
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if "seed=html-fail-seed" in body_text:
                body = (
                    "<html><body>"
                    '<div id="maze-bootstrap">puzzle-fail</div>'
                    '<a data-link-kind="maze" href="/maze/next">next</a>'
                    "</body></html>"
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            body = json.dumps({"accepted": False, "outcome": "rejected"}).encode("utf-8")
            self.send_response(400)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == "/fingerprint-report":
            self.send_response(204)
            self.end_headers()
            return
        if self.path == "/browser-beacon":
            body = json.dumps({"accepted": True}).encode("utf-8")
            self.send_response(202)
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
            "denied_path_prefixes": ["/shuma/admin"],
            "require_https": False,
            "deny_ip_literals": False,
        }
        self.descriptor_path = self.temp_dir / "scope.json"
        self.descriptor_path.write_text(json.dumps(descriptor_payload), encoding="utf-8")
        self.descriptor = shared_host_scope.descriptor_from_payload(descriptor_payload)

        self.inventory_path = self.temp_dir / "seed_inventory.json"
        self._write_inventory(self.base_url)
        self.crawldir = self.temp_dir / "crawldir"

        self.beat_payload = self._make_beat_payload("crawler", ["indexing_bot"])

    def test_local_contributor_forwarding_headers_include_geo_country_when_available(self) -> None:
        if scrapling_worker is None:
            self.fail("scrapling_worker module is required")

        with mock.patch.dict(
            os.environ,
            {
                "SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE": "1",
                "SHUMA_LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING": "1",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
            },
            clear=False,
        ):
            headers = scrapling_worker._local_contributor_forwarding_headers(
                "http://198.51.100.44:token@127.0.0.1:3871",
                "gb",
            )

        self.assertEqual(headers["X-Forwarded-For"], "198.51.100.44")
        self.assertEqual(headers["X-Forwarded-Proto"], "https")
        self.assertEqual(headers["X-Shuma-Forwarded-Secret"], "forwarded-secret")
        self.assertEqual(headers["X-Geo-Country"], "GB")

    def test_local_contributor_forwarding_headers_accept_explicit_local_client_ip_without_proxy_url(
        self,
    ) -> None:
        if scrapling_worker is None:
            self.fail("scrapling_worker module is required")

        with mock.patch.dict(
            os.environ,
            {
                "SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE": "1",
                "SHUMA_LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING": "1",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
            },
            clear=False,
        ):
            headers = scrapling_worker._local_contributor_forwarding_headers(
                None,
                "de",
                "198.51.24.8",
            )

        self.assertEqual(headers["X-Forwarded-For"], "198.51.24.8")
        self.assertEqual(headers["X-Forwarded-Proto"], "https")
        self.assertEqual(headers["X-Shuma-Forwarded-Secret"], "forwarded-secret")
        self.assertEqual(headers["X-Geo-Country"], "DE")

    def test_local_fallback_country_code_spreads_request_native_hostile_personas_across_local_geo_pool(
        self,
    ) -> None:
        if scrapling_worker is None:
            self.fail("scrapling_worker module is required")

        self.assertEqual(scrapling_worker._local_fallback_country_code("crawler"), "RU")  # type: ignore[attr-defined]
        self.assertEqual(scrapling_worker._local_fallback_country_code("bulk_scraper"), "BR")  # type: ignore[attr-defined]
        self.assertEqual(scrapling_worker._local_fallback_country_code("http_agent"), "DE")  # type: ignore[attr-defined]
        self.assertEqual(scrapling_worker._local_fallback_country_code("browser_automation"), "DE")  # type: ignore[attr-defined]
        self.assertEqual(scrapling_worker._local_fallback_country_code("stealth_browser"), "DE")  # type: ignore[attr-defined]

    def test_request_native_followup_pause_windows_respect_server_minimum_step_latency(
        self,
    ) -> None:
        if scrapling_worker is None:
            self.fail("scrapling_worker module is required")

        challenge_submit_min, challenge_submit_max = (
            scrapling_worker._request_native_followup_pause_window_ms(  # type: ignore[attr-defined]
                "challenge_puzzle_submit"
            )
        )
        challenge_abuse_min, challenge_abuse_max = (
            scrapling_worker._request_native_followup_pause_window_ms(  # type: ignore[attr-defined]
                "challenge_puzzle_abuse"
            )
        )
        pow_abuse_min, pow_abuse_max = scrapling_worker._request_native_followup_pause_window_ms(  # type: ignore[attr-defined]
            "pow_verify_abuse"
        )

        self.assertGreaterEqual(challenge_submit_min, 1_000)
        self.assertGreater(challenge_submit_max, challenge_submit_min)
        self.assertGreaterEqual(challenge_abuse_min, 1_000)
        self.assertGreater(challenge_abuse_max, challenge_abuse_min)
        self.assertGreaterEqual(pow_abuse_min, 1_000)
        self.assertGreater(pow_abuse_max, pow_abuse_min)

    def test_http_agent_first_request_in_local_contributor_mode_carries_forwarded_identity_headers(
        self,
    ) -> None:
        if scrapling_worker is None:
            self.fail("scrapling_worker module is required")

        self.httpd.requests_seen.clear()
        beat_payload = self._make_beat_payload(
            "http_agent",
            ["http_agent"],
            max_requests=1,
            max_depth=1,
            max_ms=1_000,
        )
        beat_payload["worker_plan"]["local_request_client_ip"] = "198.51.24.8"

        with mock.patch.dict(
            os.environ,
            {
                "SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE": "1",
                "SHUMA_LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING": "1",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
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

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        first_request = self.httpd.requests_seen[0]
        self.assertEqual(first_request["method"], "GET")
        self.assertEqual(first_request["path"], "/")
        self.assertEqual(first_request["headers"].get("x-forwarded-for"), "198.51.24.8")
        self.assertEqual(first_request["headers"].get("x-forwarded-proto"), "https")
        self.assertEqual(
            first_request["headers"].get("x-shuma-forwarded-secret"),
            "forwarded-secret",
        )
        self.assertEqual(first_request["headers"].get("x-geo-country"), "DE")

    def test_realism_tracker_respects_bulk_scraper_pressure_envelope_above_legacy_flat_cap(
        self,
    ) -> None:
        if scrapling_worker is None:
            self.fail("scrapling_worker module is required")

        plan = {
            "schema_version": "adversary-sim-scrapling-worker-plan.v1",
            "run_id": "simrun-pressure-envelope",
            "tick_id": "scrapling-tick-pressure-envelope",
            "lane": "scrapling_traffic",
            "sim_profile": "scrapling_runtime_lane",
            "fulfillment_mode": "bulk_scraper",
            "category_targets": ["ai_scraper_bot", "indexing_bot"],
            "surface_targets": ["public_path_traversal"],
            "tick_started_at": 1_700_000_000,
            "realism_profile": resolve_lane_realism_profile("scrapling_traffic", "bulk_scraper"),
            "max_requests": 45,
            "max_depth": 2,
            "max_bytes": 262_144,
            "max_ms": 30_000,
        }

        tracker = scrapling_worker._ScraplingRealismTracker(  # noqa: SLF001
            plan=plan,
            browser_session=False,
            proxy_configured=False,
        )

        self.assertGreater(tracker.effective_activity_budget, 8)
        self.assertEqual(
            tracker.effective_activity_budget,
            min(tracker.max_requests, tracker.planned_activity_budget),
        )

    def test_realism_tracker_marks_degraded_identity_realism_without_pool(self) -> None:
        if scrapling_worker is None:
            self.fail("scrapling_worker module is required")

        plan = {
            "schema_version": "adversary-sim-scrapling-worker-plan.v1",
            "run_id": "simrun-identity-envelope",
            "tick_id": "scrapling-tick-identity-envelope",
            "lane": "scrapling_traffic",
            "sim_profile": "scrapling_runtime_lane",
            "fulfillment_mode": "bulk_scraper",
            "category_targets": ["ai_scraper_bot"],
            "surface_targets": ["public_path_traversal"],
            "tick_started_at": 1_700_000_000,
            "realism_profile": resolve_lane_realism_profile("scrapling_traffic", "bulk_scraper"),
            "request_identity_pool": [],
            "browser_identity_pool": [],
            "max_requests": 24,
            "max_depth": 2,
            "max_bytes": 262_144,
            "max_ms": 30_000,
        }

        tracker = scrapling_worker._ScraplingRealismTracker(  # noqa: SLF001
            plan=plan,
            browser_session=False,
            proxy_configured=False,
        )
        tracker.mark_request_attempt("request-session-1")

        receipt = tracker.render_receipt(
            bytes_observed=512,
            deadline_reached=False,
            activity_sequence_exhausted=True,
            transport_failure=False,
        )

        self.assertEqual(receipt["identity_realism_status"], "degraded_local")
        self.assertEqual(
            receipt["identity_envelope_classes"],
            ["residential", "mobile"],
        )
        self.assertEqual(receipt["observed_country_codes"], [])

    def test_realism_tracker_marks_trusted_ingress_identity_when_forwarded_headers_are_present(self) -> None:
        if scrapling_worker is None:
            self.fail("scrapling_worker module is required")

        plan = {
            "schema_version": "adversary-sim-scrapling-worker-plan.v1",
            "run_id": "simrun-identity-envelope",
            "tick_id": "scrapling-tick-identity-envelope",
            "lane": "scrapling_traffic",
            "sim_profile": "scrapling_runtime_lane",
            "fulfillment_mode": "http_agent",
            "category_targets": ["http_agent"],
            "surface_targets": ["challenge_routing"],
            "tick_started_at": 1_700_000_000,
            "realism_profile": resolve_lane_realism_profile("scrapling_traffic", "http_agent"),
            "request_identity_pool": [],
            "browser_identity_pool": [],
            "local_request_client_ip": "198.51.100.24",
            "max_requests": 24,
            "max_depth": 1,
            "max_bytes": 262_144,
            "max_ms": 30_000,
        }

        tracker = scrapling_worker._ScraplingRealismTracker(  # noqa: SLF001
            plan=plan,
            browser_session=False,
            proxy_configured=False,
        )
        tracker.observe_trusted_ingress_headers(
            {
                "X-Forwarded-For": "198.51.100.24",
                "X-Shuma-Forwarded-Secret": "forwarded-secret",
            }
        )
        tracker.mark_request_attempt("request-session-1", country_code="DE")

        receipt = tracker.render_receipt(
            bytes_observed=512,
            deadline_reached=False,
            activity_sequence_exhausted=True,
            transport_failure=False,
        )

        self.assertEqual(receipt["identity_realism_status"], "fixed_proxy")
        self.assertEqual(receipt["identity_provenance_mode"], "trusted_ingress_backed")
        self.assertEqual(receipt["observed_country_codes"], ["DE"])

    def _make_beat_payload(
        self,
        fulfillment_mode: str,
        category_targets: list[str],
        *,
        max_requests: int = 5,
        max_depth: int = 2,
        max_bytes: int = 65536,
        max_ms: int = 4000,
    ) -> dict[str, Any]:
        realism_profile = resolve_lane_realism_profile(
            "scrapling_traffic",
            fulfillment_mode,
        )
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
                "tarpit_progress_abuse",
                "maze_navigation",
            ],
            "browser_automation": [
                "challenge_routing",
                "maze_navigation",
                "js_verification_execution",
                "browser_automation_detection",
            ],
            "stealth_browser": [
                "challenge_routing",
                "maze_navigation",
                "js_verification_execution",
                "browser_automation_detection",
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
                "tick_started_at": int(time.time()),
                "realism_profile": realism_profile,
                "recurrence_context": {
                    "strategy": realism_profile["recurrence_envelope"]["strategy"],
                    "reentry_scope": realism_profile["recurrence_envelope"]["reentry_scope"],
                    "dormancy_truth_mode": "accelerated_local_proof",
                    "session_index": 1,
                    "reentry_count": 0,
                    "max_reentries_per_run": realism_profile["recurrence_envelope"][
                        "max_reentries_per_run"
                    ],
                    "planned_dormant_gap_seconds": realism_profile["recurrence_envelope"][
                        "dormant_gap_seconds"
                    ]["min"],
                    "representative_dormant_gap_seconds": realism_profile[
                        "recurrence_envelope"
                    ]["representative_dormant_gap_seconds"]["min"],
                },
                "request_identity_pool": [],
                "browser_identity_pool": [],
                "max_requests": max_requests,
                "max_depth": max_depth,
                "max_bytes": max_bytes,
                "max_ms": max_ms,
            },
        }

    def tearDown(self) -> None:
        self.httpd.shutdown()
        self.httpd.server_close()
        self.server_thread.join(timeout=2)

    def _write_inventory(self, start_url: str) -> None:
        inventory = shared_host_seed_inventory.build_seed_inventory(
            self.descriptor,
            primary_start_url=start_url,
        )
        self.inventory_path.write_text(json.dumps(inventory), encoding="utf-8")

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

    def _realism_receipt(self, result: dict[str, Any]) -> dict[str, Any]:
        receipt = result.get("realism_receipt")
        self.assertIsInstance(receipt, dict, msg=json.dumps(result, indent=2))
        return dict(receipt or {})

    def test_execute_worker_plan_preserves_category_targets_in_result_contract(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        result = scrapling_worker.execute_worker_plan(  # type: ignore[attr-defined]
            self._make_beat_payload("bulk_scraper", ["ai_scraper_bot"], max_requests=2),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["schema_version"], "adversary-sim-scrapling-worker-result.v1")
        self.assertEqual(result.get("category_targets"), ["ai_scraper_bot"])

    def test_execute_worker_plan_rejects_noncanonical_realism_profile(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        beat_payload = self._make_beat_payload("crawler", ["indexing_bot"])
        beat_payload["worker_plan"]["realism_profile"]["profile_id"] = "wrong.profile.v1"

        result = scrapling_worker.execute_worker_plan(  # type: ignore[attr-defined]
            beat_payload,
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], "transport")
        self.assertIn("realism_profile", str(result.get("error") or ""))

    def test_request_native_session_kwargs_support_mobile_posture_and_geo_aligned_language(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        kwargs = scrapling_worker._request_native_session_kwargs(  # type: ignore[attr-defined]
            timeout_seconds=4.0,
            accept_header="application/json",
            request_impersonate="chrome131_android",
            accept_language="fr-FR,fr;q=0.9,en-US;q=0.7,en;q=0.6",
        )

        self.assertEqual(kwargs["impersonate"], "chrome131_android")
        self.assertTrue(kwargs["stealthy_headers"])
        self.assertFalse(kwargs["follow_redirects"])
        self.assertEqual(kwargs["retries"], 1)
        self.assertEqual(kwargs["timeout"], 4.0)
        self.assertEqual(kwargs["headers"]["accept"], "application/json")
        self.assertEqual(
            kwargs["headers"]["accept-language"],
            "fr-FR,fr;q=0.9,en-US;q=0.7,en;q=0.6",
        )
        self.assertNotIn("user-agent", {key.lower(): value for key, value in kwargs["headers"].items()})

    def test_request_native_session_kwargs_accept_optional_proxy_contract(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        kwargs = scrapling_worker._request_native_session_kwargs(  # type: ignore[attr-defined]
            timeout_seconds=4.0,
            accept_header="application/json",
            request_impersonate="chrome",
            accept_language="en-US,en;q=0.9",
            proxy_url="http://127.0.0.1:8899",
        )

        self.assertEqual(kwargs["proxy"], "http://127.0.0.1:8899")

    def test_browser_session_kwargs_accept_optional_proxy_contract(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")

        kwargs = scrapling_worker._browser_session_kwargs(  # type: ignore[attr-defined]
            fulfillment_mode="stealth_browser",
            timeout_ms=4000,
            proxy_url="http://127.0.0.1:9900",
            locale="fr-FR",
            useragent="Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36",
        )

        self.assertEqual(kwargs["proxy"], "http://127.0.0.1:9900")
        self.assertEqual(kwargs["locale"], "fr-FR")
        self.assertIn("Android 14", kwargs["useragent"])
        self.assertTrue(kwargs["hide_canvas"])
        self.assertTrue(kwargs["block_webrtc"])

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
        self.assertNotIn("challenge_routing", receipts, msg=json.dumps(result, indent=2))
        self.assertNotIn("rate_pressure", receipts, msg=json.dumps(result, indent=2))
        self.assertNotIn("geo_ip_policy", receipts, msg=json.dumps(result, indent=2))

    def test_plain_public_feed_root_does_not_count_as_challenge_routing(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "feed-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("crawler", ["indexing_bot"], max_requests=6, max_depth=3),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        challenge_statuses = self._surface_receipt_statuses(result, "challenge_routing")
        self.assertEqual(challenge_statuses, [], msg=json.dumps(result, indent=2))
        self.assertEqual(
            self._surface_receipt_statuses(result, "rate_pressure"),
            [],
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            self._surface_receipt_statuses(result, "geo_ip_policy"),
            [],
            msg=json.dumps(result, indent=2),
        )

    def test_crawler_materializes_geo_policy_from_root_served_geo_block(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "geo-block-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("crawler", ["indexing_bot"], max_requests=4, max_depth=1),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "geo_ip_policy"),
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(result["crawl_stats"]["blocked_requests_count"], 1, msg=json.dumps(result, indent=2))
        self.assertEqual(result["realism_receipt"]["activity_count"], 1, msg=json.dumps(result, indent=2))
        self.assertEqual(result["realism_receipt"]["visited_url_count"], 1, msg=json.dumps(result, indent=2))
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/geo-block-root"), paths)

    def test_bulk_scraper_traverses_feed_and_archive_pages_on_generated_site_shape(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "feed-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("bulk_scraper", ["ai_scraper_bot"], max_requests=8, max_depth=3),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        paths = {entry["path"] for entry in self.httpd.requests_seen}
        self.assertIn("/feed-root", paths)
        self.assertIn("/page/2/", paths)
        self.assertTrue(
            paths.intersection(
                {
                    "/research/",
                    "/plans/",
                    "/work/",
                    "/research/2026-03-30-gap-review/",
                    "/plans/2026-03-30-gap-plan/",
                }
            ),
            msg=json.dumps(result, indent=2),
        )

    def test_http_agent_follows_root_served_not_a_bot_without_public_defence_link(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "challenged-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("http_agent", ["http_agent"], max_requests=5, max_depth=1),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        challenge_statuses = self._surface_receipt_statuses(result, "challenge_routing")
        self.assertIn("pass_observed", challenge_statuses, msg=json.dumps(result, indent=2))
        not_a_bot_statuses = self._surface_receipt_statuses(result, "not_a_bot_submit")
        self.assertIn("fail_observed", not_a_bot_statuses, msg=json.dumps(result, indent=2))
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/challenged-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
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
                "scrapling_runtime_lane.http_agent",
            )
            self.assertEqual(
                headers.get(sim_runner.SIM_TAG_HEADER_LANE),
                "scrapling_traffic",
            )
            self.assertTrue(headers.get(sim_runner.SIM_TAG_HEADER_TIMESTAMP))
            self.assertTrue(headers.get(sim_runner.SIM_TAG_HEADER_NONCE))
            self.assertTrue(headers.get(sim_runner.SIM_TAG_HEADER_SIGNATURE))
            self.assertLessEqual(
                len(headers.get(sim_runner.SIM_TAG_HEADER_NONCE) or ""),
                96,
            )

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
        self.assertFalse(any("scrapling-" in path for path in paths))
        self.assertTrue(
            all(
                entry["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE)
                == "scrapling_runtime_lane.bulk_scraper"
                for entry in self.httpd.requests_seen
            )
        )

    def test_execute_worker_plan_http_agent_discovers_public_redirects_and_observed_forms(self) -> None:
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
        paths = [entry["path"] for entry in self.httpd.requests_seen]
        self.assertIn("/redirect-chain", paths)
        self.assertIn("/landing-final", paths)
        self.assertFalse(any(path.startswith("/agent/") for path in paths))
        self.assertFalse(any("scrapling-" in path for path in paths))
        self.assertTrue(
            all(
                entry["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE)
                == "scrapling_runtime_lane.http_agent"
                for entry in self.httpd.requests_seen
            )
        )

    def test_execute_worker_plan_bulk_scraper_attempts_owned_challenge_surfaces(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "challenged-root"))
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
        self.assertNotIn("rate_pressure", receipts, msg=json.dumps(result, indent=2))
        self.assertNotIn("geo_ip_policy", receipts, msg=json.dumps(result, indent=2))
        self.assertEqual(
            receipts["not_a_bot_submit"]["coverage_status"],
            "fail_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["puzzle_submit_or_escalation"]["coverage_status"],
            "fail_observed",
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/challenged-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)
        not_a_bot = next(
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/challenge/not-a-bot-checkbox"
        )
        self.assertIn("seed=seed", not_a_bot["body"])
        self.assertIn("checked=1", not_a_bot["body"])
        puzzle = next(
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/challenge/puzzle"
        )
        self.assertIn("seed=seed", puzzle["body"])
        self.assertIn("output=", puzzle["body"])

    def test_bulk_scraper_follows_root_served_puzzle_without_public_defence_link(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "puzzle-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("bulk_scraper", ["ai_scraper_bot"], max_requests=6, max_depth=1),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        challenge_statuses = self._surface_receipt_statuses(result, "challenge_routing")
        self.assertIn("pass_observed", challenge_statuses, msg=json.dumps(result, indent=2))
        puzzle_statuses = self._surface_receipt_statuses(result, "puzzle_submit_or_escalation")
        self.assertIn("fail_observed", puzzle_statuses, msg=json.dumps(result, indent=2))
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/puzzle-root"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)
        self.assertNotIn(("POST", "/challenge/not-a-bot-checkbox"), paths)

    def test_bulk_scraper_keeps_spending_budget_on_root_served_challenge_cycles(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "challenged-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("bulk_scraper", ["ai_scraper_bot"], max_requests=10, max_depth=1),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        receipt = self._realism_receipt(result)
        self.assertGreaterEqual(receipt["activity_count"], 8, msg=json.dumps(result, indent=2))
        challenged_root_gets = sum(
            1
            for entry in self.httpd.requests_seen
            if entry["method"] == "GET" and entry["path"] == "/challenged-root"
        )
        self.assertGreaterEqual(challenged_root_gets, 2, msg=json.dumps(result, indent=2))
        self.assertGreaterEqual(
            sum(
                1
                for entry in self.httpd.requests_seen
                if entry["method"] == "POST" and entry["path"] == "/challenge/not-a-bot-checkbox"
            ),
            2,
            msg=json.dumps(result, indent=2),
        )
        self.assertGreaterEqual(
            sum(
                1
                for entry in self.httpd.requests_seen
                if entry["method"] == "POST"
                and entry["path"] in {"/challenge/puzzle", "/tarpit/progress"}
            ),
            2,
            msg=json.dumps(result, indent=2),
        )

    def test_bulk_scraper_frontloads_root_served_challenge_before_public_crawl_exhaustion(
        self,
    ) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "bulk-frontloaded-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "bulk_scraper",
                ["ai_scraper_bot"],
                max_requests=6,
                max_ms=2_500,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        request_sequence = [
            (entry["method"], urlsplit(entry["path"]).path)
            for entry in self.httpd.requests_seen
        ]
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), request_sequence)
        public_followup_index = next(
            (
                index
                for index, request in enumerate(request_sequence)
                if request[0] == "GET" and request[1] not in {"/bulk-frontloaded-root"}
            ),
            None,
        )
        self.assertIsNotNone(public_followup_index, msg=json.dumps(result, indent=2))
        self.assertLess(
            request_sequence.index(("POST", "/challenge/not-a-bot-checkbox")),
            int(public_followup_index),
            msg=json.dumps(result, indent=2),
        )

    def test_bulk_scraper_confronts_maze_and_tarpit_from_root_served_challenge_failures(
        self,
    ) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "bulk-hostile-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "bulk_scraper",
                ["ai_scraper_bot"],
                max_requests=18,
                max_ms=6_000,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "pass_observed",
            self._surface_receipt_statuses(result, "maze_navigation"),
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "tarpit_progress_abuse"),
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/bulk-hostile-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("GET", "/maze/hostile-next"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)
        self.assertIn(("POST", "/tarpit/progress"), paths)

    def test_semantic_surface_classification_keeps_html_challenge_submits_as_fail_observed(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        response = SimpleNamespace(
            status=200,
            body=(
                "<html><body>"
                '<div id="maze-bootstrap">challenge-fail</div>'
                '<a data-link-kind="maze" href="/maze/next">next</a>'
                "</body></html>"
            ).encode("utf-8"),
            headers={},
            url=urljoin(self.base_url, "challenge/not-a-bot-checkbox"),
            request=SimpleNamespace(method="POST"),
        )

        self.assertEqual(
            scrapling_worker._surface_coverage_status_for_response(  # type: ignore[attr-defined]
                "not_a_bot_submit",
                response,
            ),
            "fail_observed",
        )
        self.assertEqual(
            scrapling_worker._surface_coverage_status_for_response(  # type: ignore[attr-defined]
                "puzzle_submit_or_escalation",
                response,
            ),
            "fail_observed",
        )

    def test_geo_policy_surface_detects_live_geo_maze_namespace_without_block_page_copy(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        response = SimpleNamespace(
            status=200,
            body=(
                "<html><body>"
                '<script id="maze-bootstrap" type="application/json">'
                '{"path_prefix":"/_/geo-policy/"}'
                "</script>"
                '<a data-link-kind="maze" href="/_/geo-policy/opaque-next">continue</a>'
                "</body></html>"
            ).encode("utf-8"),
            headers={},
            url=urljoin(self.base_url, "/"),
            request=SimpleNamespace(method="GET"),
        )

        self.assertTrue(
            scrapling_worker._response_indicates_geo_ip_policy(response),  # type: ignore[attr-defined]
        )
        self.assertEqual(
            scrapling_worker._surface_coverage_status_for_response(  # type: ignore[attr-defined]
                "geo_ip_policy",
                response,
            ),
            "fail_observed",
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

    def test_request_native_puzzle_body_prefers_real_served_output_field(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        body = scrapling_worker._request_native_puzzle_body(  # type: ignore[attr-defined]
            {
                "seed": "served-seed",
                "output": "0000000000000000",
            }
        )

        self.assertIn("seed=served-seed", body)
        self.assertIn("output=1000000000000000", body)
        self.assertNotIn("answer=bad", body)

    def test_request_native_not_a_bot_body_uses_served_seed(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        body = scrapling_worker._request_native_not_a_bot_body("served-seed")  # type: ignore[attr-defined]

        self.assertIn("seed=served-seed", body)
        self.assertIn("checked=1", body)
        self.assertIn("telemetry=", body)

    def test_request_native_not_a_bot_body_scores_into_puzzle_escalation_band(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        body = scrapling_worker._request_native_not_a_bot_body("served-seed")  # type: ignore[attr-defined]
        score = _RecordingHandler._not_a_bot_score_from_body(body)
        self.assertIsNotNone(score)
        self.assertGreaterEqual(score, 5)
        self.assertLess(score, 8)

    def test_browser_surface_detection_accepts_html_marker_fallbacks(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.assertTrue(
            scrapling_worker._browser_state_indicates_pow_surface(  # type: ignore[attr-defined]
                {"has_js_verification_script": True},
                background_paths=[],
            )
        )
        self.assertTrue(
            scrapling_worker._browser_state_indicates_maze_surface(  # type: ignore[attr-defined]
                {"has_maze_script": True}
            )
        )

    def test_http_agent_materializes_rate_pressure_from_root_served_not_a_bot_response(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "rate-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "http_agent",
                ["http_agent"],
                max_requests=8,
                max_depth=1,
                max_ms=3_000,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "pass_observed",
            self._surface_receipt_statuses(result, "challenge_routing"),
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "rate_pressure"),
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/rate-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)

    def test_http_agent_materializes_geo_policy_from_root_served_not_a_bot_response(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "geo-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("http_agent", ["http_agent"], max_requests=8, max_depth=1),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "pass_observed",
            self._surface_receipt_statuses(result, "challenge_routing"),
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "geo_ip_policy"),
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/geo-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)

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
        self.assertNotIn("rate_pressure", receipts, msg=json.dumps(result, indent=2))
        self.assertNotIn("geo_ip_policy", receipts, msg=json.dumps(result, indent=2))
        self.assertEqual(
            receipts["not_a_bot_submit"]["coverage_status"],
            "fail_observed",
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
        self.assertIn(("GET", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("GET", "/pow"), paths)
        self.assertIn(("GET", "/redirect-chain"), paths)
        self.assertIn(("GET", "/landing-final"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)
        self.assertIn(("POST", "/pow/verify"), paths)
        self.assertIn(("POST", "/tarpit/progress"), paths)
        self.assertFalse(any(path.startswith("/agent/") for _, path in paths))
        self.assertFalse(any("scrapling-" in path for _, path in paths))
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
        self.assertIn('"nonce":"invalid"', tarpit["body"])

    def test_execute_worker_plan_http_agent_reaches_pow_and_tarpit_with_live_runtime_budget(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "http_agent",
            ["http_agent"],
            max_requests=8,
            max_ms=3_000,
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
        self.assertFalse(any(path.startswith("/agent/") for _, path in paths))

    def test_http_agent_materializes_tarpit_from_abusive_challenge_submit_without_public_tarpit_link(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "challenged-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "http_agent",
                ["http_agent"],
                max_requests=8,
                max_ms=3_000,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "tarpit_progress_abuse"),
            msg=json.dumps(result, indent=2),
        )
        puzzle_posts = [
            entry["body"]
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/challenge/puzzle"
        ]
        self.assertTrue(any("output=bad" in body for body in puzzle_posts), msg=puzzle_posts)
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("POST", "/tarpit/progress"), paths)

    def test_http_agent_earns_puzzle_escalation_from_scored_not_a_bot_submission(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "scored-challenge-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "http_agent",
                ["http_agent"],
                max_requests=8,
                max_ms=3_000,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/scored-challenge-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "puzzle_submit_or_escalation"),
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "tarpit_progress_abuse"),
            msg=json.dumps(result, indent=2),
        )

    def test_http_agent_materializes_tarpit_from_runtime_entry_response_without_form_link(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "runtime-tarpit-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("http_agent", ["http_agent"], max_requests=8),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "tarpit_progress_abuse"),
            msg=json.dumps(result, indent=2),
        )
        puzzle_posts = [
            entry["body"]
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/challenge/puzzle"
        ]
        self.assertTrue(any("seed=runtime-tarpit-seed" in body for body in puzzle_posts), msg=puzzle_posts)
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("POST", "/tarpit/progress"), paths)

    def test_http_agent_carries_root_pressure_into_tarpit_and_rate_limit_without_public_defence_links(
        self,
    ) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self.httpd.root_pressure_counts.clear()
        self._write_inventory(urljoin(self.base_url, "pressure-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "http_agent",
                ["http_agent"],
                max_requests=12,
                max_ms=6_000,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "tarpit_progress_abuse"),
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "rate_pressure"),
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/pressure-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)
        self.assertIn(("POST", "/tarpit/progress"), paths)
        self.assertNotIn(("GET", "/pow"), paths)

    def test_http_agent_pivots_into_root_served_challenge_before_burst_budget_is_exhausted(
        self,
    ) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self.httpd.root_pressure_counts.clear()
        self._write_inventory(urljoin(self.base_url, "pressure-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "http_agent",
                ["http_agent"],
                max_requests=6,
                max_ms=6_000,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "not_a_bot_submit"),
            msg=json.dumps(result, indent=2),
        )
        request_sequence = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), request_sequence)
        first_not_a_bot_submit_index = request_sequence.index(("POST", "/challenge/not-a-bot-checkbox"))
        self.assertEqual(
            request_sequence[:first_not_a_bot_submit_index].count(("GET", "/pressure-root")),
            3,
            msg=json.dumps(request_sequence, indent=2),
        )

    def test_http_agent_hits_live_like_rate_threshold_from_root_served_challenge(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self.httpd.root_pressure_counts.clear()
        self._write_inventory(urljoin(self.base_url, "high-rate-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "http_agent",
                ["http_agent"],
                max_requests=48,
                max_ms=6_000,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "rate_pressure"),
            msg=json.dumps(result, indent=2),
        )
        repeated_not_a_bot_posts = [
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST"
            and entry["path"] == "/challenge/not-a-bot-checkbox"
            and "seed=high-rate-seed" in entry["body"]
        ]
        repeated_root_gets = [
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "GET" and entry["path"] == "/high-rate-root"
        ]
        self.assertEqual(
            self.httpd.root_pressure_counts.get("/high-rate-root"),
            24,
            msg=json.dumps(self.httpd.root_pressure_counts, indent=2),
        )
        self.assertGreaterEqual(len(repeated_root_gets), 24, msg=json.dumps(result, indent=2))
        self.assertGreaterEqual(len(repeated_not_a_bot_posts), 1, msg=json.dumps(result, indent=2))
        self.assertLess(
            len(repeated_not_a_bot_posts),
            len(repeated_root_gets),
            msg=json.dumps(self.httpd.requests_seen, indent=2),
        )

    def test_http_agent_keeps_spending_budget_across_hostile_request_cycles(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("http_agent", ["http_agent"], max_requests=12),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        receipt = self._realism_receipt(result)
        self.assertGreaterEqual(receipt["activity_count"], 10, msg=json.dumps(result, indent=2))
        self.assertGreaterEqual(
            sum(
                1
                for entry in self.httpd.requests_seen
                if entry["method"] == "POST"
                and entry["path"]
                in {
                    "/challenge/not-a-bot-checkbox",
                    "/challenge/puzzle",
                    "/pow/verify",
                    "/tarpit/progress",
                }
            ),
            4,
            msg=json.dumps(result, indent=2),
        )
        self.assertGreaterEqual(
            sum(
                1
                for entry in self.httpd.requests_seen
                if entry["method"] == "POST" and entry["path"] == "/pow/verify"
            ),
            1,
            msg=json.dumps(result, indent=2),
        )
        self.assertGreaterEqual(
            sum(
                1
                for entry in self.httpd.requests_seen
                if entry["method"] == "POST" and entry["path"] == "/challenge/not-a-bot-checkbox"
            ),
            2,
            msg=json.dumps(result, indent=2),
        )

    def test_http_agent_derives_pow_verify_from_root_served_js_interstitial_without_public_link(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "root-js-challenge"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("http_agent", ["http_agent"], max_requests=6, max_depth=1),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        challenge_statuses = self._surface_receipt_statuses(result, "challenge_routing")
        self.assertIn("pass_observed", challenge_statuses, msg=json.dumps(result, indent=2))
        pow_statuses = self._surface_receipt_statuses(result, "pow_verify_abuse")
        self.assertIn("fail_observed", pow_statuses, msg=json.dumps(result, indent=2))
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/root-js-challenge"), paths)
        self.assertIn(("POST", "/pow/verify"), paths)
        self.assertNotIn(("GET", "/pow"), paths)

    def test_http_agent_derives_pow_verify_from_browserish_root_interstitial_without_public_link(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self.httpd.root_accept_variant = "browserish_js_interstitial"
        self._write_inventory(self.base_url)

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload("http_agent", ["http_agent"], max_requests=6, max_depth=1),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        pow_statuses = self._surface_receipt_statuses(result, "pow_verify_abuse")
        self.assertIn("fail_observed", pow_statuses, msg=json.dumps(result, indent=2))
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/"), paths)
        self.assertIn(("POST", "/pow/verify"), paths)
        self.assertNotIn(("GET", "/pow"), paths)

    def test_http_agent_pivots_from_root_served_js_interstitial_into_later_root_challenge_followthrough(
        self,
    ) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self.httpd.root_pressure_counts.clear()
        self.httpd.root_accept_variant = "browserish_js_then_rate_challenge"
        self._write_inventory(self.base_url)

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "http_agent",
                ["http_agent"],
                max_requests=16,
                max_depth=1,
                max_ms=6_000,
            ),
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
            self._surface_receipt_statuses(result, "not_a_bot_submit"),
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "puzzle_submit_or_escalation"),
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "tarpit_progress_abuse"),
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            "fail_observed",
            self._surface_receipt_statuses(result, "rate_pressure"),
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/"), paths)
        self.assertIn(("POST", "/pow/verify"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)
        self.assertIn(("POST", "/tarpit/progress"), paths)
        self.assertNotIn(("GET", "/pow"), paths)
        repeated_not_a_bot_posts = [
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST"
            and entry["path"] == "/challenge/not-a-bot-checkbox"
            and "seed=browserish-pressure-seed" in entry["body"]
        ]
        self.assertGreaterEqual(
            len(repeated_not_a_bot_posts),
            1,
            msg=json.dumps(self.httpd.requests_seen, indent=2),
        )

    def test_execute_worker_plan_bulk_scraper_emits_request_realism_receipt(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "bulk_scraper",
            ["ai_scraper_bot"],
            max_requests=50,
        )
        with mock.patch("scripts.supervisor.scrapling_worker.time.sleep") as sleep_mock:
            result = scrapling_worker.execute_worker_plan(
                beat_payload,
                scope_descriptor_path=self.descriptor_path,
                seed_inventory_path=self.inventory_path,
                crawldir=self.crawldir,
                sim_telemetry_secret=SIM_SECRET,
            )

        receipt = self._realism_receipt(result)
        self.assertEqual(receipt["schema_version"], "sim-lane-realism-receipt.v1")
        self.assertEqual(receipt["profile_id"], "scrapling.bulk_scraper.v1")
        self.assertEqual(receipt["activity_unit"], "request")
        self.assertGreaterEqual(receipt["planned_activity_budget"], 18)
        self.assertGreaterEqual(receipt["planned_burst_size"], 2)
        self.assertEqual(receipt["activity_count"], sum(receipt["burst_sizes"]))
        self.assertEqual(receipt["burst_count"], len(receipt["burst_sizes"]))
        self.assertLessEqual(
            len(receipt["inter_activity_gaps_ms"]),
            max(0, receipt["activity_count"] - 1),
        )
        self.assertGreaterEqual(len(receipt["inter_activity_gaps_ms"]), 1)
        self.assertGreaterEqual(len(receipt["identity_handles"]), 1)
        self.assertEqual(receipt["transport_profile"], "curl_impersonate")
        self.assertEqual(receipt["transport_realism_class"], "impersonated_request_stack")
        self.assertEqual(receipt["transport_emission_basis"], "curl_cffi_impersonate")
        self.assertEqual(receipt["transport_degraded_reason"], "")
        self.assertIn("chrome_android", receipt["observed_user_agent_families"])
        self.assertIn("en-US,en;q=0.9", receipt["observed_accept_languages"])
        self.assertEqual(receipt["recurrence_strategy"], "bounded_campaign_return")
        self.assertEqual(receipt["reentry_scope"], "cross_window_campaign")
        self.assertEqual(receipt["dormancy_truth_mode"], "accelerated_local_proof")
        self.assertEqual(receipt["session_index"], 1)
        self.assertEqual(receipt["reentry_count"], 0)
        self.assertGreaterEqual(receipt["max_reentries_per_run"], 1)
        self.assertGreaterEqual(receipt["planned_dormant_gap_seconds"], 1)
        self.assertGreaterEqual(receipt["representative_dormant_gap_seconds"], 3_600)
        self.assertGreater(
            receipt["representative_dormant_gap_seconds"],
            receipt["planned_dormant_gap_seconds"],
        )
        self.assertGreaterEqual(sleep_mock.call_count, len(receipt["inter_activity_gaps_ms"]))
        self.assertIn(
            receipt["stop_reason"],
            {
                "activity_sequence_exhausted",
                "activity_budget_reached",
                "max_requests_exhausted",
                "time_budget_exhausted",
                "byte_budget_exhausted",
            },
        )

    def test_execute_worker_plan_crawler_emits_exploration_receipt_fields(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        sitemap_url = urljoin(self.base_url, "/sitemaps/pages.xml")
        inventory = shared_host_seed_inventory.build_seed_inventory(
            self.descriptor,
            primary_start_url=urljoin(self.base_url, "/timeline"),
            robots_text=f"User-agent: *\nAllow: /\nSitemap: {sitemap_url}\n",
        )
        inventory_path = self.temp_dir / "seed_inventory_with_sitemap.json"
        inventory_path.write_text(json.dumps(inventory), encoding="utf-8")
        beat_payload = self._make_beat_payload(
            "crawler",
            ["indexing_bot"],
            max_requests=20,
            max_depth=4,
            max_bytes=262_144,
            max_ms=12_000,
        )

        result = scrapling_worker.execute_worker_plan(
            beat_payload,
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        receipt = self._realism_receipt(result)
        self.assertGreaterEqual(receipt["visited_url_count"], 4)
        self.assertGreaterEqual(
            receipt["discovered_url_count"],
            receipt["visited_url_count"],
        )
        self.assertGreaterEqual(receipt["deepest_depth_reached"], 2)
        self.assertGreaterEqual(receipt["sitemap_documents_seen"], 1)
        self.assertGreaterEqual(receipt["canonical_public_pages_reached"], 3)
        self.assertGreaterEqual(receipt["frontier_remaining_count"], 0)
        paths = [entry["path"] for entry in self.httpd.requests_seen]
        self.assertIn("/sitemaps/pages.xml", paths)
        self.assertTrue(
            any(path.startswith("/timeline") for path in paths),
            msg=json.dumps(result, indent=2),
        )

    def test_execute_worker_plan_browser_automation_emits_browser_realism_receipt(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "browser_automation",
            ["automated_browser"],
            max_requests=6,
        )
        with mock.patch("scripts.supervisor.scrapling_worker.time.sleep") as sleep_mock:
            result = scrapling_worker.execute_worker_plan(
                beat_payload,
                scope_descriptor_path=self.descriptor_path,
                seed_inventory_path=self.inventory_path,
                crawldir=self.crawldir,
                sim_telemetry_secret=SIM_SECRET,
            )

        receipt = self._realism_receipt(result)
        self.assertEqual(receipt["schema_version"], "sim-lane-realism-receipt.v1")
        self.assertEqual(receipt["profile_id"], "scrapling.browser_automation.v1")
        self.assertEqual(receipt["activity_unit"], "action")
        self.assertGreaterEqual(receipt["planned_activity_budget"], 4)
        self.assertEqual(receipt["top_level_action_count"], receipt["activity_count"])
        self.assertGreaterEqual(len(receipt["session_handles"]), 1)
        self.assertEqual(receipt["transport_profile"], "playwright_chromium")
        self.assertEqual(receipt["transport_realism_class"], "browser_runtime_stack")
        self.assertEqual(receipt["transport_emission_basis"], "playwright_chromium_runtime")
        self.assertEqual(receipt["transport_degraded_reason"], "")
        self.assertIn("chrome_desktop", receipt["observed_user_agent_families"])
        self.assertIn("en-US,en;q=0.9", receipt["observed_accept_languages"])
        self.assertIn("en-US", receipt["observed_browser_locales"])
        self.assertEqual(receipt["secondary_capture_mode"], "xhr_capture")
        self.assertGreaterEqual(receipt["secondary_request_count"], 1)
        self.assertGreaterEqual(receipt["background_request_count"], 1)
        self.assertEqual(receipt["subresource_request_count"], 0)
        self.assertEqual(receipt["recurrence_strategy"], "bounded_campaign_return")
        self.assertEqual(receipt["reentry_scope"], "cross_window_campaign")
        self.assertEqual(receipt["dormancy_truth_mode"], "accelerated_local_proof")
        self.assertEqual(receipt["session_index"], 1)
        self.assertEqual(receipt["reentry_count"], 0)
        self.assertGreaterEqual(receipt["max_reentries_per_run"], 1)
        self.assertGreaterEqual(receipt["planned_dormant_gap_seconds"], 1)
        self.assertGreaterEqual(receipt["representative_dormant_gap_seconds"], 3_600)
        self.assertGreater(
            receipt["representative_dormant_gap_seconds"],
            receipt["planned_dormant_gap_seconds"],
        )
        self.assertEqual(
            len(receipt["dwell_intervals_ms"]),
            max(0, receipt["top_level_action_count"] - 1),
        )
        self.assertGreaterEqual(sleep_mock.call_count, len(receipt["dwell_intervals_ms"]))
        self.assertIn(
            receipt["stop_reason"],
            {
                "activity_sequence_exhausted",
                "activity_budget_reached",
                "max_requests_exhausted",
                "time_budget_exhausted",
                "byte_budget_exhausted",
            },
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("POST", "/browser-beacon"), paths)

    def test_execute_worker_plan_browser_automation_attempts_browser_owned_surfaces(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "browser_automation",
            ["automated_browser"],
            max_requests=6,
        )
        result = scrapling_worker.execute_worker_plan(
            beat_payload,
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["lane"], "scrapling_traffic")
        self.assertEqual(result["fulfillment_mode"], "browser_automation")
        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["js_verification_execution"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["maze_navigation"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            receipts["browser_automation_detection"]["coverage_status"],
            {"pass_observed", "fail_observed"},
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/pow"), paths)
        self.assertIn(("GET", "/maze/start"), paths)

    def test_execute_worker_plan_stealth_browser_attempts_browser_owned_surfaces(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        beat_payload = self._make_beat_payload(
            "stealth_browser",
            ["automated_browser"],
            max_requests=6,
        )
        result = scrapling_worker.execute_worker_plan(
            beat_payload,
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["lane"], "scrapling_traffic")
        self.assertEqual(result["fulfillment_mode"], "stealth_browser")
        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["js_verification_execution"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["maze_navigation"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            receipts["browser_automation_detection"]["coverage_status"],
            {"pass_observed", "fail_observed"},
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/pow"), paths)
        self.assertIn(("GET", "/maze/start"), paths)

    def test_browser_automation_executes_root_served_js_interstitial_without_public_pow_link(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "root-js-challenge"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "browser_automation",
                ["automated_browser"],
                max_requests=6,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["js_verification_execution"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            receipts["browser_automation_detection"]["coverage_status"],
            {"pass_observed", "fail_observed"},
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/root-js-challenge"), paths)
        self.assertNotIn(("GET", "/pow"), paths)

    def test_browser_automation_navigates_live_opaque_maze_namespace(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "opaque-maze-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "browser_automation",
                ["automated_browser"],
                max_requests=6,
                max_ms=2_500,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["maze_navigation"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/opaque-maze-root"), paths)
        self.assertIn(("GET", "/_/geo-policy/opaque-next"), paths)

    def test_browser_automation_escalates_root_served_challenge_without_public_defence_links(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "browser-challenge-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "browser_automation",
                ["automated_browser"],
                max_requests=8,
                max_ms=2_500,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["js_verification_execution"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["maze_navigation"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertIn(
            receipts["browser_automation_detection"]["coverage_status"],
            {"pass_observed", "fail_observed"},
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/browser-challenge-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertNotIn(("GET", "/pow"), paths)
        not_a_bot_submit = next(
            entry
            for entry in self.httpd.requests_seen
            if entry["method"] == "POST" and entry["path"] == "/challenge/not-a-bot-checkbox"
        )
        self.assertEqual(
            not_a_bot_submit["headers"].get(sim_runner.SIM_TAG_HEADER_PROFILE),
            "scrapling_runtime_lane.browser_automation",
        )

    def test_browser_automation_reaches_maze_via_low_score_not_a_bot_without_public_defence_links(
        self,
    ) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "browser-low-score-challenge-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "browser_automation",
                ["automated_browser"],
                max_requests=8,
                max_ms=2_500,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["maze_navigation"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/browser-low-score-challenge-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertNotIn(("POST", "/challenge/puzzle"), paths)
        self.assertNotIn(("GET", "/pow"), paths)

    def test_browser_automation_follows_root_served_not_a_bot_after_js_interstitial(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "browser-js-then-challenge-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "browser_automation",
                ["automated_browser"],
                max_requests=8,
                max_ms=2_500,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["js_verification_execution"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["maze_navigation"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/browser-js-then-challenge-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertNotIn(("GET", "/pow"), paths)

    def test_stealth_browser_reaches_maze_via_scored_puzzle_escalation(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "browser-escalation-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "stealth_browser",
                ["automated_browser"],
                max_requests=8,
                max_ms=2_500,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["maze_navigation"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/browser-escalation-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)
        self.assertIn(("POST", "/challenge/puzzle"), paths)

    def test_browser_automation_preserves_root_detection_state_after_js_interstitial_redirects_into_challenge(
        self,
    ) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "browser-js-detected-then-challenge-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "browser_automation",
                ["automated_browser"],
                max_requests=8,
                max_ms=2_500,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["js_verification_execution"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["browser_automation_detection"]["coverage_status"],
            "fail_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            receipts["maze_navigation"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/browser-js-detected-then-challenge-root"), paths)
        self.assertIn(("POST", "/challenge/not-a-bot-checkbox"), paths)

    def test_browser_automation_traverses_public_pages_before_root_served_js_confrontation(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        self._write_inventory(urljoin(self.base_url, "browser-public-explore-root"))

        result = scrapling_worker.execute_worker_plan(
            self._make_beat_payload(
                "browser_automation",
                ["automated_browser"],
                max_requests=8,
            ),
            scope_descriptor_path=self.descriptor_path,
            seed_inventory_path=self.inventory_path,
            crawldir=self.crawldir,
            sim_telemetry_secret=SIM_SECRET,
        )

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        receipts = self._surface_receipts_by_id(result)
        self.assertEqual(
            receipts["js_verification_execution"]["coverage_status"],
            "pass_observed",
            msg=json.dumps(result, indent=2),
        )
        self.assertGreaterEqual(
            int(result["realism_receipt"]["top_level_action_count"]),
            2,
            msg=json.dumps(result, indent=2),
        )
        paths = [(entry["method"], entry["path"]) for entry in self.httpd.requests_seen]
        self.assertIn(("GET", "/browser-public-explore-root"), paths)
        self.assertIn(("GET", "/page/browser-public-explore/"), paths)
        self.assertNotIn(("GET", "/pow"), paths)

    def test_browser_automation_uses_local_trusted_forwarding_headers_in_contributor_mode(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        beat_payload = self._make_beat_payload(
            "browser_automation",
            ["automated_browser"],
            max_requests=6,
        )
        beat_payload["worker_plan"]["local_browser_client_ip"] = "198.51.24.8"

        with mock.patch.dict(
            os.environ,
            {
                "SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE": "1",
                "SHUMA_LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING": "1",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
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

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertTrue(self.httpd.requests_seen, msg=json.dumps(result, indent=2))
        first_request = self.httpd.requests_seen[0]
        self.assertEqual(first_request["headers"].get("x-forwarded-for"), "198.51.24.8")
        self.assertEqual(
            first_request["headers"].get("x-shuma-forwarded-secret"),
            "forwarded-secret",
        )
        self.assertEqual(
            result["realism_receipt"]["identity_provenance_mode"],
            "trusted_ingress_backed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            result["realism_receipt"]["identity_realism_status"],
            "fixed_proxy",
            msg=json.dumps(result, indent=2),
        )

    def test_http_agent_uses_local_trusted_forwarding_headers_in_contributor_mode(self) -> None:
        self.assertIsNotNone(scrapling_worker, "worker module missing")
        self.httpd.requests_seen.clear()
        beat_payload = self._make_beat_payload(
            "http_agent",
            ["http_agent"],
            max_requests=6,
        )
        beat_payload["worker_plan"]["local_request_client_ip"] = "198.51.24.9"

        with mock.patch.dict(
            os.environ,
            {
                "SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE": "1",
                "SHUMA_LOCAL_CONTRIBUTOR_ALLOW_TRUSTED_FORWARDING": "1",
                "SHUMA_FORWARDED_IP_SECRET": "forwarded-secret",
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

        self.assertEqual(result["failure_class"], None, msg=json.dumps(result, indent=2))
        self.assertTrue(self.httpd.requests_seen, msg=json.dumps(result, indent=2))
        first_request = self.httpd.requests_seen[0]
        self.assertEqual(first_request["headers"].get("x-forwarded-for"), "198.51.24.9")
        self.assertEqual(
            first_request["headers"].get("x-shuma-forwarded-secret"),
            "forwarded-secret",
        )
        self.assertEqual(
            result["realism_receipt"]["identity_provenance_mode"],
            "trusted_ingress_backed",
            msg=json.dumps(result, indent=2),
        )
        self.assertEqual(
            result["realism_receipt"]["identity_realism_status"],
            "fixed_proxy",
            msg=json.dumps(result, indent=2),
        )

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
