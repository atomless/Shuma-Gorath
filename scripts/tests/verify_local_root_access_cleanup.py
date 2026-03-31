#!/usr/bin/env python3
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from typing import Optional, Tuple


BASE_URL = os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000").rstrip("/")
API_KEY = os.environ["SHUMA_API_KEY"]
FORWARDED_SECRET = os.environ["SHUMA_FORWARDED_IP_SECRET"]


def request(
    path: str,
    *,
    method: str = "GET",
    body: Optional[bytes] = None,
    auth: bool = False,
    trusted_forward: bool = False,
) -> Tuple[int, str]:
    req = urllib.request.Request(f"{BASE_URL}{path}", data=body, method=method)
    if body is not None:
        req.add_header("Content-Type", "application/json")
    if auth:
        req.add_header("Authorization", f"Bearer {API_KEY}")
    if trusted_forward:
        req.add_header("X-Forwarded-For", "127.0.0.1")
        req.add_header("X-Shuma-Forwarded-Secret", FORWARDED_SECRET)
    try:
        with urllib.request.urlopen(req) as response:
            return response.getcode(), response.read().decode("utf-8", errors="replace")
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read().decode("utf-8", errors="replace")


def honeypot_path() -> str:
    status, body = request(
        "/shuma/admin/config",
        auth=True,
        trusted_forward=True,
    )
    if status != 200:
        raise SystemExit(f"failed to load config for honeypot-path proof setup: status={status} body={body}")
    payload = json.loads(body)
    honeypots = ((payload.get("config") or {}).get("honeypots") or ["/instaban"])
    return honeypots[0]


def trigger_unknown_ban() -> None:
    status, _ = request(honeypot_path())
    if status != 403:
        raise SystemExit(f"expected honeypot request to block while setting up proof, got status={status}")


def unban_unknown() -> None:
    status, body = request(
        "/shuma/admin/unban?ip=unknown",
        method="POST",
        auth=True,
        trusted_forward=True,
    )
    if status != 200:
        raise SystemExit(f"failed to unban unknown during cleanup: status={status} body={body}")


def assert_root_blocked() -> None:
    status, body = request("/")
    if status != 403 or "Access Blocked" not in body:
        raise SystemExit(
            f"expected blocked root while unknown is banned, got status={status} body={body[:300]!r}"
        )


def assert_root_accessible() -> None:
    status, body = request("/")
    if status == 403 or "Access Blocked" in body:
        raise SystemExit(
            f"expected accessible root after cleanup, got status={status} body={body[:300]!r}"
        )


def run_cleanup_target() -> None:
    result = subprocess.run(
        ["make", "--no-print-directory", "clear-dev-loopback-bans"],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(
            "clear-dev-loopback-bans failed:\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )


def main() -> int:
    try:
        unban_unknown()
    except SystemExit:
        pass
    trigger_unknown_ban()
    assert_root_blocked()
    run_cleanup_target()
    assert_root_accessible()
    unban_unknown()
    return 0


if __name__ == "__main__":
    sys.exit(main())
