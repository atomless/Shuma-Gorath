#!/usr/bin/env python3
"""Deterministic upstream fixture for gateway/proxy integration tests."""

from __future__ import annotations

import argparse
import hashlib
import json
import socket
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Dict, Tuple
from urllib.parse import parse_qs, urlsplit


def _json_headers(handler: BaseHTTPRequestHandler, status: int = 200) -> None:
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Cache-Control", "no-store")


def _normalize_query_values(query: Dict[str, list[str]]) -> Dict[str, object]:
    normalized: Dict[str, object] = {}
    for key in sorted(query.keys()):
        values = query[key]
        normalized[key] = values[0] if len(values) == 1 else values
    return normalized


class GatewayUpstreamFixtureHandler(BaseHTTPRequestHandler):
    server_version = "GatewayUpstreamFixture/1.0"
    protocol_version = "HTTP/1.1"

    def log_message(self, format: str, *args: object) -> None:  # noqa: A003
        return

    def _read_body(self) -> bytes:
        length_raw = self.headers.get("Content-Length", "0").strip()
        try:
            length = int(length_raw)
        except ValueError:
            length = 0
        if length <= 0:
            return b""
        return self.rfile.read(length)

    def _dispatch(self) -> None:
        split = urlsplit(self.path)
        query = parse_qs(split.query, keep_blank_values=True)

        if split.path == "/__fixture/health":
            payload = {"ok": True, "mode": "health"}
            data = json.dumps(payload).encode("utf-8")
            _json_headers(self, 200)
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
            return

        mode = query.get("mode", ["echo"])[0]
        if split.path.startswith("/__fixture/fail/"):
            mode = split.path.rsplit("/", 1)[-1].strip().lower()

        if mode == "timeout":
            sleep_ms_raw = query.get("sleep_ms", ["500"])[0]
            try:
                sleep_ms = max(0, int(sleep_ms_raw))
            except ValueError:
                sleep_ms = 500
            time.sleep(sleep_ms / 1000.0)
            payload = {"ok": False, "mode": "timeout", "sleep_ms": sleep_ms}
            data = json.dumps(payload).encode("utf-8")
            _json_headers(self, 504)
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
            return

        if mode == "reset":
            try:
                self.connection.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass
            self.connection.close()
            return

        if mode == "status":
            status_raw = query.get("status", ["503"])[0]
            try:
                status = int(status_raw)
            except ValueError:
                status = 503
            status = min(max(status, 100), 599)
            payload = {"ok": False, "mode": "status", "status": status}
            data = json.dumps(payload).encode("utf-8")
            _json_headers(self, status)
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
            return

        body = self._read_body()
        headers = {
            key.lower(): value.strip()
            for key, value in sorted(self.headers.items(), key=lambda row: row[0].lower())
        }
        payload = {
            "ok": True,
            "mode": "echo",
            "method": self.command,
            "path": split.path,
            "query": _normalize_query_values(query),
            "headers": headers,
            "body_len": len(body),
            "body_sha256": hashlib.sha256(body).hexdigest(),
            "body_preview_utf8": body[:120].decode("utf-8", errors="replace"),
        }
        data = json.dumps(payload, sort_keys=True).encode("utf-8")
        _json_headers(self, 200)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self) -> None:  # noqa: N802
        self._dispatch()

    def do_POST(self) -> None:  # noqa: N802
        self._dispatch()

    def do_PUT(self) -> None:  # noqa: N802
        self._dispatch()

    def do_PATCH(self) -> None:  # noqa: N802
        self._dispatch()

    def do_DELETE(self) -> None:  # noqa: N802
        self._dispatch()

    def do_OPTIONS(self) -> None:  # noqa: N802
        self._dispatch()

    def do_HEAD(self) -> None:  # noqa: N802
        self._dispatch()


def create_server(host: str, port: int) -> ThreadingHTTPServer:
    return ThreadingHTTPServer((host, port), GatewayUpstreamFixtureHandler)


def run_server(host: str, port: int) -> None:
    server = create_server(host, port)
    print(f"[gateway-upstream-fixture] listening on http://{host}:{port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


def parse_args() -> Tuple[str, int]:
    parser = argparse.ArgumentParser(description="Deterministic gateway upstream fixture server")
    parser.add_argument("--host", default="127.0.0.1", help="bind host")
    parser.add_argument("--port", type=int, default=19081, help="bind port")
    args = parser.parse_args()
    return args.host, args.port


if __name__ == "__main__":
    host_arg, port_arg = parse_args()
    run_server(host_arg, port_arg)
