#!/usr/bin/env python3
import json
import os
from pathlib import Path
import sys
import time
import urllib.error
import urllib.request
import uuid
from typing import Mapping, Optional, Tuple


BASE_URL = os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000").rstrip("/")
API_KEY = os.environ["SHUMA_API_KEY"]
FORWARDED_SECRET = os.environ["SHUMA_FORWARDED_IP_SECRET"]
LANE = os.environ.get("SHUMA_LOCAL_CONTRIBUTOR_SIM_LANE", "scrapling_traffic").strip() or "scrapling_traffic"
WAIT_TIMEOUT_SECONDS = int(
    os.environ.get("SHUMA_LOCAL_CONTRIBUTOR_SIM_WAIT_TIMEOUT_SECONDS", "120")
)
REPO_ROOT = Path(__file__).resolve().parents[2]

if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests.adversary_runtime_toggle_surface_gate import (  # noqa: E402
    RuntimeToggleSurfaceGate,
    runtime_surface_coverage_meets_gate,
)


def request(
    path: str,
    *,
    method: str = "GET",
    body: Optional[bytes] = None,
    auth: bool = False,
    trusted_forward: bool = False,
    control: bool = False,
    extra_headers: Optional[Mapping[str, str]] = None,
) -> Tuple[int, str]:
    req = urllib.request.Request(f"{BASE_URL}{path}", data=body, method=method)
    if body is not None:
        req.add_header("Content-Type", "application/json")
    if auth:
        req.add_header("Authorization", f"Bearer {API_KEY}")
    if trusted_forward:
        req.add_header("X-Forwarded-For", "127.0.0.1")
        req.add_header("X-Forwarded-Proto", "https")
        req.add_header("X-Shuma-Forwarded-Secret", FORWARDED_SECRET)
    if control:
        req.add_header("Idempotency-Key", f"local-contributor-sim-isolation-{uuid.uuid4()}")
        req.add_header("Origin", BASE_URL)
        req.add_header("Referer", f"{BASE_URL}/shuma/dashboard/index.html")
        req.add_header("Sec-Fetch-Site", "same-origin")
    for key, value in (extra_headers or {}).items():
        req.add_header(str(key), str(value))
    try:
        with urllib.request.urlopen(req, timeout=15.0) as response:
            return response.getcode(), response.read().decode("utf-8", errors="replace")
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read().decode("utf-8", errors="replace")


def fetch_status() -> dict:
    status_code, body = request(
        "/shuma/admin/adversary-sim/status",
        auth=True,
        trusted_forward=True,
    )
    if status_code != 200:
        raise SystemExit(f"failed to load adversary sim status: status={status_code} body={body}")
    payload = json.loads(body)
    if not isinstance(payload, dict):
        raise SystemExit("adversary sim status response was not a JSON object")
    return payload


def fetch_bans() -> dict:
    status_code, body = request(
        "/shuma/admin/ban?active=true",
        auth=True,
        trusted_forward=True,
    )
    if status_code != 200:
        raise SystemExit(f"failed to load active bans: status={status_code} body={body}")
    payload = json.loads(body)
    if not isinstance(payload, dict):
        raise SystemExit("active bans response was not a JSON object")
    return payload


def post_control(enabled: bool, *, lane: Optional[str] = None, reason: str) -> dict:
    payload = {"enabled": enabled, "reason": reason}
    if lane is not None:
        payload["lane"] = lane
    status_code, body = request(
        "/shuma/admin/adversary-sim/control",
        method="POST",
        body=json.dumps(payload).encode("utf-8"),
        auth=True,
        trusted_forward=True,
        control=True,
    )
    if status_code != 200:
        raise SystemExit(f"failed to post adversary sim control: status={status_code} body={body}")
    parsed = json.loads(body)
    if not isinstance(parsed, dict):
        raise SystemExit("adversary sim control response was not a JSON object")
    return parsed


def unban_loopback_triplet() -> None:
    for ip in ("127.0.0.1", "::1", "unknown"):
        request(
            f"/shuma/admin/unban?ip={ip}",
            method="POST",
            auth=True,
            trusted_forward=True,
        )


def assert_root_accessible() -> None:
    status_code, body = request("/")
    if status_code == 403 or "Access Blocked" in body:
        raise SystemExit(
            f"expected accessible public root, got status={status_code} body={body[:300]!r}"
        )


def assert_trusted_ingress_configured() -> None:
    status = fetch_status()
    configured = (
        ((status.get("representativeness_readiness") or {}).get("prerequisites") or {}).get(
            "trusted_ingress_configured"
        )
    )
    if configured is not True:
        raise SystemExit(
            "expected local runtime to report trusted_ingress_configured=true, "
            f"got {configured!r} in adversary sim status"
        )


def has_generation_progress(start_status: dict, current_status: dict) -> bool:
    current_generation = current_status.get("generation") or {}
    current_request_count = int(current_generation.get("request_count") or 0)
    current_tick_count = int(current_generation.get("tick_count") or 0)
    current_last_generated_at = int(current_generation.get("last_generated_at") or 0)
    current_run_id = str(current_status.get("run_id") or "").strip()
    current_last_successful_beat_at = int(
        (((current_status.get("lifecycle_diagnostics") or {}).get("supervisor") or {}).get("last_successful_beat_at") or 0)
    )

    if current_status.get("adversary_sim_enabled") is not True:
        return False
    if current_request_count <= 0 and current_tick_count <= 0:
        return False

    start_generation = start_status.get("generation") or {}
    start_last_generated_at = int(start_generation.get("last_generated_at") or 0)
    start_run_id = str(start_status.get("run_id") or "").strip()
    start_last_successful_beat_at = int(
        (((start_status.get("lifecycle_diagnostics") or {}).get("supervisor") or {}).get("last_successful_beat_at") or 0)
    )

    if current_run_id and current_run_id != start_run_id:
        return True
    if current_last_generated_at > start_last_generated_at:
        return True
    if current_last_successful_beat_at > start_last_successful_beat_at:
        return True
    return False


def wait_for_generation_progress(start_status: dict) -> None:
    deadline = time.time() + WAIT_TIMEOUT_SECONDS
    while time.time() < deadline:
        status = fetch_status()
        if has_generation_progress(start_status, status):
            return
        time.sleep(0.5)
    current_status = fetch_status()
    raise SystemExit(
        "timed out waiting for local adversary sim worker activity; "
        f"start_run_id={start_status.get('run_id')!r} "
        f"current_run_id={current_status.get('run_id')!r} "
        f"current_generation={json.dumps(current_status.get('generation') or {}, sort_keys=True)}"
    )


def build_runtime_surface_gate() -> RuntimeToggleSurfaceGate:
    return RuntimeToggleSurfaceGate(
        base_url=BASE_URL,
        api_key=API_KEY,
        forwarded_secret=FORWARDED_SECRET,
        health_secret=os.environ.get("SHUMA_HEALTH_SECRET", ""),
        timeout_seconds=max(10, WAIT_TIMEOUT_SECONDS),
    )


def wait_for_meaningful_recent_run(
    gate: RuntimeToggleSurfaceGate,
    *,
    existing_run_ids: set[str],
    minimum_started_at: int,
) -> dict:
    coverage = gate.poll_recent_scrapling_run_coverage(
        existing_run_ids=existing_run_ids,
        minimum_started_at=minimum_started_at,
    )
    if runtime_surface_coverage_meets_gate(coverage):
        return coverage
    raise SystemExit(
        "local contributor sim isolation observed only truncated Scrapling coverage; "
        f"coverage={json.dumps(coverage, sort_keys=True)}"
    )


def assert_loopback_not_banned() -> None:
    bans = fetch_bans()
    entries = bans.get("bans") or []
    active_ips = {str(entry.get("ip") or "").strip() for entry in entries if isinstance(entry, dict)}
    for blocked_identity in ("127.0.0.1", "::1", "unknown"):
        if blocked_identity in active_ips:
            raise SystemExit(
                f"expected contributor identities to stay unbanned after local sim run; "
                f"found {blocked_identity} in active bans: {json.dumps(entries)}"
            )


def main() -> int:
    gate = build_runtime_surface_gate()
    unban_loopback_triplet()
    assert_root_accessible()
    assert_trusted_ingress_configured()
    gate.clear_loopback_bans()
    gate.clear_runtime_surface_bans()
    gate.configure_runtime_surface_profile()
    live_summary_baseline = gate.read_live_summary_counts()
    existing_run_ids = gate.current_recent_scrapling_run_ids()
    minimum_started_at = max(0, int(time.time()) - 1)
    try:
        post_control(True, lane=LANE, reason="local_contributor_sim_isolation_start")
        wait_for_meaningful_recent_run(
            gate,
            existing_run_ids=existing_run_ids,
            minimum_started_at=minimum_started_at,
        )
        live_summary_counts = gate.poll_live_summary_matches_baseline(live_summary_baseline)
        if live_summary_counts != live_summary_baseline:
            raise SystemExit(
                "local contributor sim isolation did not restore the live summary baseline; "
                f"baseline={json.dumps(live_summary_baseline, sort_keys=True)} "
                f"current={json.dumps(live_summary_counts, sort_keys=True)}"
            )
    finally:
        post_control(False, reason="local_contributor_sim_isolation_stop")
        gate.clear_loopback_bans()
    assert_loopback_not_banned()
    assert_root_accessible()
    return 0


if __name__ == "__main__":
    sys.exit(main())
