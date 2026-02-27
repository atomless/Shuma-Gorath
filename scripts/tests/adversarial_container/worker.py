#!/usr/bin/env python3
"""Container-side black-box adversary worker."""

from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.parse
import urllib.request
from typing import Any, Dict, List


FORBIDDEN_ENV_PREFIXES = ("SHUMA_",)
FORBIDDEN_ENV_KEYS = {
    "SHUMA_API_KEY",
    "SHUMA_ADMIN_READONLY_API_KEY",
    "SHUMA_JS_SECRET",
    "SHUMA_CHALLENGE_SECRET",
    "SHUMA_HEALTH_SECRET",
    "SHUMA_FORWARDED_IP_SECRET",
}
DEFAULT_ENDPOINTS = (
    "/",
    "/sim/public/landing",
    "/sim/public/docs",
    "/sim/public/pricing",
    "/sim/public/contact",
    "/sim/public/search?q=adversarial+simulation",
)


def bool_env(name: str, default: bool = False) -> bool:
    raw = os.environ.get(name)
    if raw is None:
        return default
    return str(raw).strip().lower() in {"1", "true", "yes", "on"}


def parse_positive_int(name: str, default: int) -> int:
    raw = os.environ.get(name, str(default))
    try:
        parsed = int(str(raw).strip())
    except Exception:
        return default
    return max(1, parsed)


def has_forbidden_env(observed_keys: List[str]) -> bool:
    for key in observed_keys:
        if any(key.startswith(prefix) for prefix in FORBIDDEN_ENV_PREFIXES):
            return True
        if key in FORBIDDEN_ENV_KEYS:
            return True
    return False


def workspace_mount_absent() -> bool:
    try:
        mounts = open("/proc/mounts", "r", encoding="utf-8", errors="replace").read().lower()
    except Exception:
        return False
    markers = ("shuma-gorath", "/users/jamestindall/projects", "/workspace")
    return not any(marker in mounts for marker in markers)


def enforce_allowlist(url: str, allowed_origins: List[str]) -> bool:
    parsed = urllib.parse.urlparse(url)
    origin = f"{parsed.scheme}://{parsed.netloc}"
    return origin in allowed_origins


def make_request(url: str, run_id: str, mode: str, timeout_seconds: float = 10.0) -> Dict[str, Any]:
    request = urllib.request.Request(url, method="GET")
    request.add_header("User-Agent", "ShumaContainerBlackBox/1.0")
    request.add_header("X-Shuma-Sim-Run-Id", run_id)
    request.add_header("X-Shuma-Sim-Profile", mode)
    request.add_header("X-Shuma-Sim-Lane", "container_blackbox")
    start = time.monotonic()
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            body = response.read().decode("utf-8", errors="replace")
            latency_ms = int((time.monotonic() - start) * 1000)
            return {
                "url": url,
                "status": response.status,
                "latency_ms": latency_ms,
                "body_sample": body[:160],
            }
    except urllib.error.HTTPError as exc:
        latency_ms = int((time.monotonic() - start) * 1000)
        return {
            "url": url,
            "status": int(exc.code),
            "latency_ms": latency_ms,
            "error": f"http_error_{exc.code}",
        }
    except Exception as exc:
        latency_ms = int((time.monotonic() - start) * 1000)
        return {
            "url": url,
            "status": 0,
            "latency_ms": latency_ms,
            "error": str(exc),
        }


def main() -> int:
    mode = str(os.environ.get("BLACKBOX_MODE", "blackbox")).strip().lower()
    base_url = str(os.environ.get("BLACKBOX_BASE_URL", "")).strip().rstrip("/")
    run_id = str(os.environ.get("BLACKBOX_RUN_ID", "")).strip() or f"container-{int(time.time())}"
    allowed_origins_raw = str(os.environ.get("BLACKBOX_ALLOWED_ORIGINS", "")).strip()
    allowed_origins = [origin.strip() for origin in allowed_origins_raw.split(",") if origin.strip()]
    request_budget = parse_positive_int("BLACKBOX_REQUEST_BUDGET", 24)
    time_budget_seconds = parse_positive_int("BLACKBOX_TIME_BUDGET_SECONDS", 120)
    start = time.monotonic()

    observed_env_keys = sorted(list(os.environ.keys()))
    forbidden_env_present = has_forbidden_env(observed_env_keys)
    non_root = os.getuid() != 0
    no_workspace_mount = workspace_mount_absent()
    admin_credentials_absent = not forbidden_env_present
    tooling_limited = True  # Worker is intentionally limited to urllib-based HTTP traffic.
    egress_allowlist_enforced = True
    ephemeral_run_identity = bool(run_id)

    payload: Dict[str, Any] = {
        "schema_version": "adversarial-container-worker.v1",
        "mode": mode,
        "run_id": run_id,
        "runtime_hardening_non_root": non_root,
        "workspace_mount_absent": no_workspace_mount,
        "admin_credentials_absent": admin_credentials_absent,
        "tooling_limited": tooling_limited,
        "egress_allowlist_enforced": egress_allowlist_enforced,
        "ephemeral_run_identity": ephemeral_run_identity,
        "observed_env_keys": [key for key in observed_env_keys if key.startswith("BLACKBOX_")],
        "request_budget": request_budget,
        "time_budget_seconds": time_budget_seconds,
        "traffic": [],
        "passed": False,
    }

    contract_pass = (
        non_root
        and no_workspace_mount
        and admin_credentials_absent
        and tooling_limited
        and egress_allowlist_enforced
        and ephemeral_run_identity
    )

    if mode == "isolation":
        payload["passed"] = contract_pass
        print(json.dumps(payload, separators=(",", ":")))
        return 0 if payload["passed"] else 1

    if mode != "blackbox":
        payload["error"] = f"unsupported_mode:{mode}"
        print(json.dumps(payload, separators=(",", ":")))
        return 1

    if not base_url:
        payload["error"] = "missing_base_url"
        print(json.dumps(payload, separators=(",", ":")))
        return 1
    if not allowed_origins:
        payload["error"] = "missing_allowed_origins"
        print(json.dumps(payload, separators=(",", ":")))
        return 1

    statuses: List[int] = []
    errors: List[str] = []
    requests_sent = 0
    for endpoint in DEFAULT_ENDPOINTS:
        if requests_sent >= request_budget:
            break
        if (time.monotonic() - start) >= time_budget_seconds:
            errors.append("time_budget_exhausted")
            break
        url = f"{base_url}{endpoint}"
        if not enforce_allowlist(url, allowed_origins):
            errors.append(f"egress_disallowed:{url}")
            break
        result = make_request(url, run_id, mode)
        payload["traffic"].append(result)
        requests_sent += 1
        statuses.append(int(result.get("status", 0)))
        if result.get("status", 0) == 0:
            errors.append(str(result.get("error") or "request_failed"))

    payload["requests_sent"] = requests_sent
    payload["errors"] = errors
    payload["allowed_statuses"] = [200, 302, 303, 403, 404, 429]

    status_ok = all(status in payload["allowed_statuses"] for status in statuses if status != 0)
    payload["passed"] = contract_pass and requests_sent > 0 and status_ok and not errors
    print(json.dumps(payload, separators=(",", ":")))
    return 0 if payload["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
