#!/usr/bin/env python3
import os
import sys
import urllib.error
import urllib.request


BASE_URL = os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000").rstrip("/")


def fetch(path: str) -> tuple[int, str]:
    req = urllib.request.Request(f"{BASE_URL}{path}")
    try:
        with urllib.request.urlopen(req, timeout=15.0) as response:
            return response.getcode(), response.read().decode("utf-8", errors="replace")
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read().decode("utf-8", errors="replace")


def assert_status(path: str, expected_status: int) -> None:
    status, body = fetch(path)
    if status != expected_status:
        raise SystemExit(
            f"expected {path} to return {expected_status}, got status={status} body={body[:300]!r}"
        )
    if "Gateway forwarding unavailable" in body:
        raise SystemExit(
            f"expected {path} to stay self-contained locally, but gateway fallback body was returned"
        )


def assert_stays_local(path: str) -> None:
    status, body = fetch(path)
    if status == 502:
        raise SystemExit(
            f"expected {path} to stay self-contained locally, got gateway-like status=502 body={body[:300]!r}"
        )
    if "Gateway forwarding unavailable" in body:
        raise SystemExit(
            f"expected {path} to stay self-contained locally, but gateway fallback body was returned"
        )


def main() -> int:
    assert_status("/", 200)
    assert_status("/favicon.ico", 404)
    assert_stays_local("/totally-unlisted/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
