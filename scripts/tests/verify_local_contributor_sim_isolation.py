#!/usr/bin/env python3
import json
import os
from pathlib import Path
import subprocess
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
MIN_TICK_DELTA = int(os.environ.get("SHUMA_LOCAL_CONTRIBUTOR_SIM_MIN_TICK_DELTA", "7"))
WAIT_TIMEOUT_SECONDS = int(
    os.environ.get("SHUMA_LOCAL_CONTRIBUTOR_SIM_WAIT_TIMEOUT_SECONDS", "20")
)
REPO_ROOT = Path(__file__).resolve().parents[2]
SUPERVISOR_LAUNCH_SCRIPT = REPO_ROOT / "scripts" / "adversary_sim_supervisor_launch.sh"


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


def launch_supervisor_process() -> subprocess.Popen[str]:
    env = dict(os.environ)
    env["SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL"] = BASE_URL
    env["SHUMA_ADVERSARY_SIM_SUPERVISOR_EXIT_WHEN_OFF"] = "1"
    return subprocess.Popen(
        [str(SUPERVISOR_LAUNCH_SCRIPT), "--exit-when-off", "--base-url", BASE_URL],
        cwd=str(REPO_ROOT),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )


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


def wait_for_tick_delta(start_tick_count: int) -> None:
    deadline = time.time() + WAIT_TIMEOUT_SECONDS
    while time.time() < deadline:
        status = fetch_status()
        tick_count = int(((status.get("generation") or {}).get("tick_count") or 0))
        if tick_count >= start_tick_count + MIN_TICK_DELTA:
            return
        time.sleep(0.5)
    current_tick_count = int(((fetch_status().get("generation") or {}).get("tick_count") or 0))
    raise SystemExit(
        f"timed out waiting for adversary sim tick delta {MIN_TICK_DELTA}; "
        f"start={start_tick_count} current={current_tick_count}"
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
    unban_loopback_triplet()
    assert_root_accessible()
    assert_trusted_ingress_configured()
    start_status = fetch_status()
    start_tick_count = int(((start_status.get("generation") or {}).get("tick_count") or 0))
    post_control(True, lane=LANE, reason="local_contributor_sim_isolation_start")
    supervisor_process = launch_supervisor_process()
    try:
        wait_for_tick_delta(start_tick_count)
    finally:
        post_control(False, reason="local_contributor_sim_isolation_stop")
        try:
            supervisor_process.wait(timeout=10.0)
        except subprocess.TimeoutExpired:
            supervisor_process.terminate()
            try:
                supervisor_process.wait(timeout=5.0)
            except subprocess.TimeoutExpired:
                supervisor_process.kill()
                supervisor_process.wait(timeout=5.0)
        stderr = (supervisor_process.stderr.read() if supervisor_process.stderr else "").strip()
        if supervisor_process.returncode not in (0, None):
            raise SystemExit(
                "local adversary sim supervisor failed during contributor isolation proof: "
                f"returncode={supervisor_process.returncode} stderr={stderr}"
            )
    assert_loopback_not_banned()
    assert_root_accessible()
    return 0


if __name__ == "__main__":
    sys.exit(main())
