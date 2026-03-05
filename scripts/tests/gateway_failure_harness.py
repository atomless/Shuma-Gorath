#!/usr/bin/env python3
"""Gateway fixture failure harness for timeout/transport/http classes."""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import socket
import threading
import time
from dataclasses import dataclass
from http.client import RemoteDisconnected
from typing import Dict, List
from urllib import error, request

from gateway_upstream_fixture import create_server


@dataclass
class CheckResult:
    check_id: str
    passed: bool
    detail: str


def classify_exception(exc: Exception) -> str:
    if isinstance(exc, TimeoutError):
        return "timeout"
    if isinstance(exc, socket.timeout):
        return "timeout"
    if isinstance(exc, RemoteDisconnected):
        return "transport"
    if isinstance(exc, ConnectionError):
        return "transport"
    if isinstance(exc, OSError):
        return "transport"
    return "transport"


def run_http_request(url: str, method: str = "GET", data: bytes | None = None, timeout: float = 1.0):
    req = request.Request(url=url, method=method, data=data)
    req.add_header("x-gateway-harness", "1")
    with request.urlopen(req, timeout=timeout) as resp:  # nosec B310 (fixed local fixture URL in harness)
        body = resp.read()
        return int(resp.status), body


@contextlib.contextmanager
def fixture_server(host: str) -> Dict[str, object]:
    server = create_server(host, 0)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base_url = f"http://{host}:{server.server_address[1]}"
    try:
        deadline = time.time() + 3.0
        while time.time() < deadline:
            try:
                status, _ = run_http_request(f"{base_url}/__fixture/health", timeout=0.2)
                if status == 200:
                    break
            except Exception:
                time.sleep(0.05)
        yield {"base_url": base_url}
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=1.0)


def run_harness(host: str) -> Dict[str, object]:
    checks: List[CheckResult] = []
    with fixture_server(host) as fixture:
        base_url = str(fixture["base_url"])

        # Deterministic echo fidelity.
        body = b'{"hello":"gateway"}'
        status, payload = run_http_request(
            f"{base_url}/echo/path?alpha=1&beta=2", method="POST", data=body, timeout=1.0
        )
        decoded = json.loads(payload.decode("utf-8"))
        checks.append(
            CheckResult(
                check_id="echo_fidelity",
                passed=(
                    status == 200
                    and decoded.get("method") == "POST"
                    and decoded.get("path") == "/echo/path"
                    and str(decoded.get("query", {}).get("alpha")) == "1"
                    and decoded.get("body_sha256") == hashlib.sha256(body).hexdigest()
                ),
                detail=f"status={status} mode={decoded.get('mode')}",
            )
        )

        # Upstream non-2xx behavior path.
        try:
            run_http_request(f"{base_url}/__fixture/fail/status?status=429", timeout=1.0)
            checks.append(
                CheckResult(
                    check_id="http_status_non_2xx",
                    passed=False,
                    detail="expected HTTPError(429), request unexpectedly succeeded",
                )
            )
        except error.HTTPError as exc:
            checks.append(
                CheckResult(
                    check_id="http_status_non_2xx",
                    passed=int(exc.code) == 429,
                    detail=f"http_error_code={exc.code}",
                )
            )

        # Timeout behavior path.
        try:
            run_http_request(f"{base_url}/__fixture/fail/timeout?sleep_ms=450", timeout=0.1)
            checks.append(
                CheckResult(
                    check_id="timeout_failure_classification",
                    passed=False,
                    detail="expected timeout, request unexpectedly succeeded",
                )
            )
        except Exception as exc:  # noqa: BLE001
            klass = classify_exception(exc)
            checks.append(
                CheckResult(
                    check_id="timeout_failure_classification",
                    passed=klass == "timeout",
                    detail=f"class={klass} error={type(exc).__name__}",
                )
            )

        # Transport reset behavior path.
        try:
            run_http_request(f"{base_url}/__fixture/fail/reset", timeout=0.5)
            checks.append(
                CheckResult(
                    check_id="transport_reset_classification",
                    passed=False,
                    detail="expected transport reset, request unexpectedly succeeded",
                )
            )
        except Exception as exc:  # noqa: BLE001
            klass = classify_exception(exc)
            checks.append(
                CheckResult(
                    check_id="transport_reset_classification",
                    passed=klass == "transport",
                    detail=f"class={klass} error={type(exc).__name__}",
                )
            )

    passed = all(check.passed for check in checks)
    return {
        "version": "gateway-failure-harness.v1",
        "passed": passed,
        "checks": [check.__dict__ for check in checks],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Gateway failure harness")
    parser.add_argument("--host", default="127.0.0.1", help="fixture bind host")
    args = parser.parse_args()
    report = run_harness(args.host)
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
