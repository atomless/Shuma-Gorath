#!/usr/bin/env python3
"""Run a wasm32 gateway TLS failure matrix against cert-error upstreams."""

from __future__ import annotations

import argparse
import contextlib
from dataclasses import dataclass
import json
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
import uuid
from typing import Dict, Iterable, List, Optional, Sequence
from urllib import error, parse, request

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.spin_manifest import build_manifest_with_allowed_outbound_hosts, normalize_origin

FALLBACK_FAILURE_BODY = b"Gateway forwarding unavailable"
DEFAULT_CASE_MATRIX = (
    ("expired_cert", "https://expired.badssl.com"),
    ("self_signed_cert", "https://self-signed.badssl.com"),
    ("hostname_mismatch", "https://wrong.host.badssl.com"),
)


@dataclass(frozen=True)
class TlsCase:
    case_id: str
    origin: str
    expected_failure_class: str = "transport"
    expected_status_code: int = 502


@dataclass
class HttpProbe:
    status_code: Optional[int]
    body: bytes
    error_class: Optional[str]
    detail: str


@dataclass
class CaseResult:
    case_id: str
    origin: str
    passed: bool
    detail: str
    status_code: Optional[int]
    expected_status_code: int
    expected_failure_class: str
    observed_forward_failure_classes: List[str]


def canonical_origin_authority(origin: str) -> str:
    try:
        normalized_origin, _ = normalize_origin(origin)
    except ValueError as exc:
        raise ValueError(f"unsupported origin for case {origin!r}: {exc}") from exc
    return normalized_origin


def parse_prometheus_counter(metrics_text: str, klass: str) -> float:
    escaped = re.escape(klass)
    pattern = re.compile(
        rf'^bot_defence_forward_failure_total\{{class="{escaped}"\}}\s+([0-9]+(?:\.[0-9]+)?)\s*$',
        re.MULTILINE,
    )
    match = pattern.search(metrics_text)
    return float(match.group(1)) if match else 0.0


def parse_forward_failure_classes(log_text: str) -> List[str]:
    pattern = re.compile(r"\[gateway-forward\] failed status=\d+ class=([a-z_]+)")
    return [match.group(1) for match in pattern.finditer(log_text)]


def evaluate_gateway_failure(status_code: int, body: bytes) -> tuple[bool, str]:
    body_contains = FALLBACK_FAILURE_BODY in body
    status_ok = status_code in {502, 504}
    passed = status_ok and body_contains
    body_preview = body.decode("utf-8", errors="replace").strip().replace("\n", " ")
    if len(body_preview) > 120:
        body_preview = f"{body_preview[:117]}..."
    return passed, f"status={status_code} fallback_body={body_contains} body_preview={body_preview}"


def classify_transport_exception(exc: Exception) -> str:
    if isinstance(exc, socket.timeout):
        return "timeout"
    if isinstance(exc, TimeoutError):
        return "timeout"
    return "transport"


def http_probe(url: str, headers: Optional[Dict[str, str]] = None, timeout: float = 3.0) -> HttpProbe:
    req = request.Request(url=url, method="GET")
    for key, value in (headers or {}).items():
        req.add_header(key, value)
    try:
        with request.urlopen(req, timeout=timeout) as resp:  # nosec B310 (fixed harness URL inputs)
            return HttpProbe(
                status_code=int(resp.status),
                body=resp.read(),
                error_class=None,
                detail="ok",
            )
    except error.HTTPError as exc:
        return HttpProbe(
            status_code=int(exc.code),
            body=exc.read(),
            error_class=None,
            detail=f"http_error:{exc.code}",
        )
    except Exception as exc:  # noqa: BLE001
        return HttpProbe(
            status_code=None,
            body=b"",
            error_class=classify_transport_exception(exc),
            detail=f"{type(exc).__name__}: {exc}",
        )


def allocate_free_port(host: str = "127.0.0.1") -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind((host, 0))
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        return int(sock.getsockname()[1])


def read_log_tail(log_path: Path, max_lines: int = 80) -> str:
    if not log_path.exists():
        return ""
    lines = log_path.read_text(encoding="utf-8", errors="replace").splitlines()
    return "\n".join(lines[-max_lines:])


def wait_for_ready(base_url: str, headers: Dict[str, str], startup_timeout_seconds: float) -> None:
    deadline = time.time() + startup_timeout_seconds
    while time.time() < deadline:
        probe = http_probe(f"{base_url}/shuma/health", headers=headers, timeout=0.8)
        if probe.status_code is not None:
            return
        time.sleep(0.15)
    raise TimeoutError(f"spin app readiness timed out after {startup_timeout_seconds:.1f}s")


@contextlib.contextmanager
def spin_instance(
    repo_root: Path,
    manifest_path: Path,
    listen_port: int,
    env_overrides: Dict[str, str],
    startup_timeout_seconds: float,
):
    if shutil.which("spin") is None:
        raise RuntimeError("spin CLI is required for gateway wasm TLS harness")

    with tempfile.TemporaryDirectory(prefix="gw-tls-wasm-") as temp_dir:
        log_path = Path(temp_dir) / "spin.log"
        env = os.environ.copy()
        env.update(env_overrides)
        env.setdefault("SPIN_ALWAYS_BUILD", "0")

        cmd = [
            "spin",
            "up",
            "--from",
            str(manifest_path),
            "--direct-mounts",
            "--listen",
            f"127.0.0.1:{listen_port}",
        ]
        for key, value in env_overrides.items():
            cmd.extend(["-e", f"{key}={value}"])

        with log_path.open("w", encoding="utf-8") as log_handle:
            process = subprocess.Popen(
                cmd,
                cwd=str(repo_root),
                env=env,
                stdout=log_handle,
                stderr=subprocess.STDOUT,
                text=True,
            )

        base_url = f"http://127.0.0.1:{listen_port}"
        health_headers = {
            "X-Forwarded-For": "127.0.0.1",
            "X-Shuma-Forwarded-Secret": env_overrides["SHUMA_FORWARDED_IP_SECRET"],
            "X-Shuma-Health-Secret": env_overrides["SHUMA_HEALTH_SECRET"],
        }

        try:
            deadline = time.time() + startup_timeout_seconds
            while time.time() < deadline:
                if process.poll() is not None:
                    break
                probe = http_probe(f"{base_url}/shuma/health", headers=health_headers, timeout=0.5)
                if probe.status_code is not None:
                    break
                time.sleep(0.15)
            else:
                raise TimeoutError(f"spin app readiness timed out after {startup_timeout_seconds:.1f}s")

            if process.poll() is not None:
                raise RuntimeError(
                    f"spin exited before readiness (code={process.returncode})\n{read_log_tail(log_path)}"
                )

            wait_for_ready(base_url, health_headers, startup_timeout_seconds)
            yield base_url, log_path
        finally:
            if process.poll() is None:
                process.send_signal(signal.SIGTERM)
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


def build_case_env(origin: str) -> Dict[str, str]:
    return {
        "SHUMA_RUNTIME_ENV": "runtime-dev",
        "SHUMA_API_KEY": "test-admin-key",
        "SHUMA_ADMIN_READONLY_API_KEY": "test-readonly-key",
        "SHUMA_JS_SECRET": "test-js-secret",
        "SHUMA_POW_SECRET": "test-pow-secret",
        "SHUMA_CHALLENGE_SECRET": "test-challenge-secret",
        "SHUMA_MAZE_PREVIEW_SECRET": "test-maze-secret",
        "SHUMA_FORWARDED_IP_SECRET": "test-forwarded-secret",
        "SHUMA_HEALTH_SECRET": "test-health-secret",
        "SHUMA_EVENT_LOG_RETENTION_HOURS": "168",
        "SHUMA_ENFORCE_HTTPS": "false",
        "SHUMA_KV_STORE_FAIL_OPEN": "true",
        "SHUMA_DEBUG_HEADERS": "false",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED": "false",
        "SHUMA_ADVERSARY_SIM_AVAILABLE": "false",
        "SHUMA_GATEWAY_UPSTREAM_ORIGIN": origin,
        "SHUMA_GATEWAY_DEPLOYMENT_PROFILE": "shared-server",
        "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL": "false",
        "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS": "false",
        "SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST": "",
        "SHUMA_GATEWAY_PUBLIC_AUTHORITIES": "public.example.com:443",
        "SHUMA_GATEWAY_LOOP_MAX_HOPS": "3",
        "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED": "true",
        "SHUMA_GATEWAY_ORIGIN_AUTH_MODE": "network_only",
        "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME": "",
        "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE": "",
        "SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS": "90",
        "SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS": "7",
        "SHUMA_GATEWAY_TLS_STRICT": "true",
        "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED": "true",
    }


def run_case(
    repo_root: Path,
    manifest_template: str,
    case: TlsCase,
    timeout_seconds: float,
    startup_timeout_seconds: float,
) -> CaseResult:
    normalized_origin = canonical_origin_authority(case.origin)
    manifest_text = build_manifest_with_allowed_outbound_hosts(manifest_template, [normalized_origin])

    manifest_path = repo_root / f".spin.gateway_tls_case.{case.case_id}.{uuid.uuid4().hex}.toml"
    try:
        manifest_path.write_text(manifest_text, encoding="utf-8")
        port = allocate_free_port("127.0.0.1")
        with spin_instance(
            repo_root=repo_root,
            manifest_path=manifest_path,
            listen_port=port,
            env_overrides=build_case_env(normalized_origin),
            startup_timeout_seconds=startup_timeout_seconds,
        ) as (base_url, log_path):
            headers = {
                "Host": "public.example.com",
                "X-Forwarded-For": "127.0.0.1",
                "X-Shuma-Forwarded-Secret": "test-forwarded-secret",
            }
            gateway = http_probe(
                f"{base_url}/assets/__gw_tls_probe.js?case={case.case_id}",
                headers=headers,
                timeout=timeout_seconds,
            )
            time.sleep(0.1)
            forward_failure_classes = parse_forward_failure_classes(
                log_path.read_text(encoding="utf-8", errors="replace")
            )
    finally:
        manifest_path.unlink(missing_ok=True)

    if gateway.status_code is None:
        return CaseResult(
            case_id=case.case_id,
            origin=normalized_origin,
            passed=False,
            detail=f"gateway probe failed before HTTP response ({gateway.error_class} {gateway.detail})",
            status_code=None,
            expected_status_code=case.expected_status_code,
            expected_failure_class=case.expected_failure_class,
            observed_forward_failure_classes=[],
        )

    body_ok, body_detail = evaluate_gateway_failure(gateway.status_code, gateway.body)

    status_ok = gateway.status_code == case.expected_status_code
    class_ok = case.expected_failure_class in forward_failure_classes

    passed = body_ok and status_ok and class_ok
    detail = (
        f"{body_detail} expected_status={case.expected_status_code} "
        f"observed_classes={forward_failure_classes}"
    )
    return CaseResult(
        case_id=case.case_id,
        origin=normalized_origin,
        passed=passed,
        detail=detail,
        status_code=gateway.status_code,
        expected_status_code=case.expected_status_code,
        expected_failure_class=case.expected_failure_class,
        observed_forward_failure_classes=forward_failure_classes,
    )


def resolve_cases(case_ids: Optional[Iterable[str]]) -> List[TlsCase]:
    all_cases = [TlsCase(case_id=cid, origin=origin) for cid, origin in DEFAULT_CASE_MATRIX]
    if not case_ids:
        return all_cases
    requested = {value.strip() for value in case_ids if value.strip()}
    selected = [case for case in all_cases if case.case_id in requested]
    if len(selected) != len(requested):
        missing = sorted(requested - {case.case_id for case in selected})
        raise ValueError(f"unknown case id(s): {', '.join(missing)}")
    return selected


def run_harness(
    manifest: Path,
    timeout_seconds: float,
    startup_timeout_seconds: float,
    case_ids: Optional[Iterable[str]] = None,
) -> Dict[str, object]:
    repo_root = Path(__file__).resolve().parents[2]
    manifest_path = manifest if manifest.is_absolute() else (repo_root / manifest)
    if not manifest_path.exists():
        raise FileNotFoundError(f"manifest not found: {manifest_path}")

    manifest_template = manifest_path.read_text(encoding="utf-8")
    cases = resolve_cases(case_ids)

    results: List[CaseResult] = []
    for case in cases:
        results.append(
            run_case(
                repo_root=repo_root,
                manifest_template=manifest_template,
                case=case,
                timeout_seconds=timeout_seconds,
                startup_timeout_seconds=startup_timeout_seconds,
            )
        )

    return {
        "schema": "shuma.gateway_tls_wasm_harness.v1",
        "passed": all(result.passed for result in results),
        "cases": [result.__dict__ for result in results],
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run wasm32 gateway TLS failure matrix (expired/self-signed/hostname-mismatch)."
    )
    parser.add_argument("--manifest", default="spin.toml", help="Spin manifest path")
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=5.0,
        help="HTTP timeout for probe requests",
    )
    parser.add_argument(
        "--startup-timeout-seconds",
        type=float,
        default=30.0,
        help="Spin startup/readiness timeout",
    )
    parser.add_argument(
        "--case",
        action="append",
        dest="cases",
        help="Run only the given case id (can be passed multiple times)",
    )
    parser.add_argument("--json-output", default="", help="Optional output file for JSON report")
    parser.add_argument("--list-cases", action="store_true", help="List available case ids and exit")
    args = parser.parse_args()

    if args.list_cases:
        for case_id, origin in DEFAULT_CASE_MATRIX:
            print(f"{case_id}\t{origin}")
        return 0

    try:
        report = run_harness(
            manifest=Path(args.manifest),
            timeout_seconds=float(args.timeout_seconds),
            startup_timeout_seconds=float(args.startup_timeout_seconds),
            case_ids=args.cases,
        )
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"schema": "shuma.gateway_tls_wasm_harness.v1", "passed": False, "error": str(exc)}))
        return 1

    report_json = json.dumps(report, indent=2, sort_keys=True)
    print(report_json)
    if args.json_output:
        output_path = Path(args.json_output)
        if not output_path.is_absolute():
            output_path = Path.cwd() / output_path
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(report_json + "\n", encoding="utf-8")

    return 0 if bool(report.get("passed")) else 1


if __name__ == "__main__":
    raise SystemExit(main())
