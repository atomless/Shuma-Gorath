#!/usr/bin/env python3
"""Optional active probe for direct-origin bypass exposure."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import json
from pathlib import Path
import socket
import time
from typing import Dict, Optional, Sequence, Tuple
from urllib import error, request


@dataclass
class ProbeResponse:
    status_code: Optional[int]
    error_class: Optional[str]
    detail: str


def classify_request_exception(exc: Exception) -> str:
    if isinstance(exc, TimeoutError):
        return "timeout"
    if isinstance(exc, socket.timeout):
        return "timeout"
    return "transport"


def run_probe(url: str, timeout_seconds: float, headers: Optional[Dict[str, str]] = None) -> ProbeResponse:
    req = request.Request(url=url, method="GET")
    for key, value in (headers or {}).items():
        req.add_header(key, value)

    try:
        with request.urlopen(req, timeout=timeout_seconds) as resp:  # nosec B310 (probe target is explicit operator input)
            _ = resp.read(2048)
            return ProbeResponse(status_code=int(resp.status), error_class=None, detail="ok")
    except error.HTTPError as exc:
        return ProbeResponse(status_code=int(exc.code), error_class=None, detail=f"http_error:{exc.code}")
    except Exception as exc:  # noqa: BLE001
        return ProbeResponse(
            status_code=None,
            error_class=classify_request_exception(exc),
            detail=f"{type(exc).__name__}: {exc}",
        )


def classify_origin_bypass(
    gateway_result: ProbeResponse,
    direct_result: ProbeResponse,
    protected_statuses: Sequence[int] = (401, 403, 404, 429),
) -> Tuple[str, str]:
    if gateway_result.status_code is None:
        return (
            "inconclusive",
            "gateway probe did not return an HTTP response; verify gateway endpoint health first",
        )

    if direct_result.status_code is None:
        return (
            "protected",
            "direct origin appears unreachable from probe vantage (transport/timeout)",
        )

    if direct_result.status_code in set(protected_statuses):
        return (
            "protected",
            f"direct origin returned deny status {direct_result.status_code}",
        )

    return (
        "exposed",
        f"direct origin is reachable with status {direct_result.status_code}",
    )


def build_probe_url(base_url: str, probe_path: str) -> str:
    normalized_base = base_url.strip().rstrip("/")
    normalized_path = probe_path.strip()
    if not normalized_path.startswith("/"):
        normalized_path = f"/{normalized_path}"
    nonce = f"shuma_probe_ts={int(time.time() * 1000)}"
    separator = "&" if "?" in normalized_path else "?"
    return f"{normalized_base}{normalized_path}{separator}{nonce}"


def report_payload(
    gateway_url: str,
    origin_url: str,
    probe_path: str,
    gateway_result: ProbeResponse,
    direct_result: ProbeResponse,
    decision: str,
    reason: str,
) -> Dict[str, object]:
    return {
        "schema": "shuma.gateway_origin_bypass_probe.v1",
        "checked_at_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "gateway_url": gateway_url,
        "origin_url": origin_url,
        "probe_path": probe_path,
        "gateway": gateway_result.__dict__,
        "direct_origin": direct_result.__dict__,
        "decision": decision,
        "reason": reason,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Optional active probe to detect direct-origin bypass exposure."
    )
    parser.add_argument("--gateway-url", required=True, help="Gateway base URL (for example https://shield.example.com)")
    parser.add_argument("--origin-url", required=True, help="Origin base URL (for example https://origin.internal.example)")
    parser.add_argument("--probe-path", default="/", help="Path to probe on both gateway and origin")
    parser.add_argument("--timeout-seconds", type=float, default=5.0, help="Per-request timeout")
    parser.add_argument(
        "--protected-statuses",
        default="401,403,404,429",
        help="Comma-separated direct-origin HTTP statuses considered protected",
    )
    parser.add_argument(
        "--fail-on-inconclusive",
        action="store_true",
        help="Treat inconclusive result as non-zero exit status",
    )
    parser.add_argument("--json-output", default="", help="Optional JSON report output path")
    args = parser.parse_args()

    protected_statuses = []
    for raw in args.protected_statuses.split(","):
        token = raw.strip()
        if not token:
            continue
        try:
            protected_statuses.append(int(token))
        except ValueError:
            print(f"❌ invalid protected status code: {token}")
            return 1

    gateway_probe_url = build_probe_url(args.gateway_url, args.probe_path)
    origin_probe_url = build_probe_url(args.origin_url, args.probe_path)

    gateway_result = run_probe(gateway_probe_url, timeout_seconds=float(args.timeout_seconds))
    direct_result = run_probe(origin_probe_url, timeout_seconds=float(args.timeout_seconds))
    decision, reason = classify_origin_bypass(
        gateway_result,
        direct_result,
        protected_statuses=protected_statuses,
    )

    payload = report_payload(
        gateway_url=gateway_probe_url,
        origin_url=origin_probe_url,
        probe_path=args.probe_path,
        gateway_result=gateway_result,
        direct_result=direct_result,
        decision=decision,
        reason=reason,
    )

    encoded = json.dumps(payload, indent=2, sort_keys=True)
    print(encoded)

    if args.json_output:
        output_path = Path(args.json_output)
        if not output_path.is_absolute():
            output_path = Path.cwd() / output_path
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(encoded + "\n", encoding="utf-8")

    if decision == "exposed":
        return 1
    if decision == "inconclusive" and args.fail_on_inconclusive:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
