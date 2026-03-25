#!/usr/bin/env python3
"""Runtime-toggle Scrapling owned-surface coverage and no-impact gate.

This validates the dashboard-toggle execution lane (control endpoint + autonomous supervisor)
produces a real recent Scrapling run with covered owned-surface receipts in operator snapshot
without polluting the live-only monitoring summary paths.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from typing import Any, Dict, Optional


def parse_bool(value: str, default: bool) -> bool:
    lowered = str(value or "").strip().lower()
    if lowered in {"1", "true", "yes", "on"}:
        return True
    if lowered in {"0", "false", "no", "off"}:
        return False
    return default


def live_summary_leaks(current: Dict[str, int], baseline: Dict[str, int]) -> Dict[str, int]:
    leaked: Dict[str, int] = {}
    for name, value in current.items():
        delta = int(value) - int(baseline.get(name, 0))
        if delta > 0:
            leaked[name] = delta
    return leaked


class RuntimeToggleSurfaceGate:
    def __init__(
        self,
        base_url: str,
        api_key: str,
        forwarded_secret: str,
        health_secret: str,
        timeout_seconds: int,
    ):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.forwarded_secret = forwarded_secret.strip()
        self.health_secret = health_secret.strip()
        self.timeout_seconds = timeout_seconds
        self.opener = urllib.request.build_opener()

    def _headers(self, include_json: bool = False) -> Dict[str, str]:
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "X-Forwarded-For": "127.0.0.1",
        }
        if self.forwarded_secret:
            headers["X-Shuma-Forwarded-Secret"] = self.forwarded_secret
        if include_json:
            headers["Content-Type"] = "application/json"
        return headers

    def _health_headers(self) -> Dict[str, str]:
        headers = self._headers()
        if self.health_secret:
            headers["X-Shuma-Health-Secret"] = self.health_secret
        return headers

    def request(
        self,
        method: str,
        path: str,
        payload: Optional[Dict[str, Any]] = None,
        extra_headers: Optional[Dict[str, str]] = None,
    ) -> Dict[str, Any]:
        url = f"{self.base_url}{path}"
        body = None
        headers = self._headers(include_json=payload is not None)
        if extra_headers:
            headers.update(extra_headers)
        if payload is not None:
            body = json.dumps(payload).encode("utf-8")

        req = urllib.request.Request(url, data=body, method=method.upper(), headers=headers)
        try:
            with self.opener.open(req, timeout=5) as response:
                text = response.read().decode("utf-8", errors="replace")
                try:
                    parsed = json.loads(text) if text else {}
                except json.JSONDecodeError:
                    parsed = {"raw": text}
                return {
                    "status": int(getattr(response, "status", 0) or 0),
                    "body": parsed,
                    "raw": text,
                }
        except urllib.error.HTTPError as err:
            text = err.read().decode("utf-8", errors="replace")
            try:
                parsed = json.loads(text) if text else {}
            except json.JSONDecodeError:
                parsed = {"raw": text}
            return {"status": int(err.code), "body": parsed, "raw": text}

    @staticmethod
    def _as_obj(value: Any) -> Dict[str, Any]:
        return value if isinstance(value, dict) else {}

    @staticmethod
    def _as_list(value: Any) -> list[Any]:
        return value if isinstance(value, list) else []

    @staticmethod
    def _as_int(value: Any) -> int:
        try:
            return int(value)
        except (TypeError, ValueError):
            return 0

    def live_summary_counts(self, monitoring_body: Dict[str, Any]) -> Dict[str, int]:
        summary = self._as_obj(monitoring_body.get("summary"))
        return {
            "challenge_failures": self._as_int(
                self._as_obj(summary.get("challenge")).get("total_failures")
            ),
            "pow_attempts": self._as_int(self._as_obj(summary.get("pow")).get("total_attempts")),
            "rate_violations": self._as_int(
                self._as_obj(summary.get("rate")).get("total_violations")
            ),
            "geo_violations": self._as_int(self._as_obj(summary.get("geo")).get("total_violations")),
        }

    def read_live_summary_counts(self) -> Dict[str, int]:
        deadline = time.time() + float(self.timeout_seconds)
        counts = {
            "challenge_failures": 0,
            "pow_attempts": 0,
            "rate_violations": 0,
            "geo_violations": 0,
        }

        while time.time() < deadline:
            monitoring = self.request("GET", "/admin/monitoring?hours=24&limit=200")
            if monitoring["status"] != 200:
                time.sleep(1)
                continue
            return self.live_summary_counts(self._as_obj(monitoring["body"]))

        return counts

    def ensure_health(self) -> None:
        response = self.request("GET", "/health", extra_headers=self._health_headers())
        if response["status"] != 200:
            raise RuntimeError(f"health check failed: status={response['status']} body={response['raw'][:200]}")

    def clear_loopback_bans(self) -> None:
        for ip in ("127.0.0.1", "::1"):
            response = self.request("POST", f"/admin/unban?ip={ip}")
            if response["status"] != 200:
                raise RuntimeError(
                    f"failed to clear loopback ban for {ip}: status={response['status']} body={response['raw'][:200]}"
                )

    def configure_runtime_surface_profile(self) -> None:
        payload = {
            "defence_modes": {"rate": "both", "geo": "both", "js": "both"},
            "rate_limit": 80,
            "js_required_enforced": True,
            "pow_enabled": True,
            "challenge_puzzle_enabled": True,
            "not_a_bot_enabled": True,
            "maze_auto_ban": False,
            "geo_edge_headers_enabled": True,
            "geo_challenge": ["RU"],
            "geo_maze": [],
            "geo_block": [],
            "ban_durations": {
                "rate_limit": 1,
                "tarpit_persistence": 1,
                "not_a_bot_abuse": 1,
                "challenge_puzzle_abuse": 1,
            },
        }
        response = self.request("POST", "/admin/config", payload)
        if response["status"] != 200:
            raise RuntimeError(
                f"failed to apply runtime surface config profile: status={response['status']} body={response['raw'][:200]}"
            )

    def toggle(self, enabled: bool, suffix: str) -> None:
        max_attempts = 1 if enabled else 10
        for attempt in range(1, max_attempts + 1):
            operation_id = f"runtime-surface-{int(time.time())}-{suffix}-a{attempt}"
            response = self.request(
                "POST",
                "/admin/adversary-sim/control",
                {"enabled": bool(enabled), "reason": "runtime_surface_gate"},
                extra_headers={
                    "Idempotency-Key": operation_id,
                    "Origin": self.base_url,
                    "Sec-Fetch-Site": "same-origin",
                },
            )
            if response["status"] == 200:
                return
            if not enabled and response["status"] == 429 and attempt < max_attempts:
                time.sleep(1)
                continue
            raise RuntimeError(
                f"toggle {enabled} failed: status={response['status']} body={response['raw'][:200]}"
            )

    def recent_scrapling_run_coverage(
        self,
        operator_snapshot_body: Dict[str, Any],
    ) -> Dict[str, Any]:
        adversary_sim = self._as_obj(operator_snapshot_body.get("adversary_sim"))
        recent_runs = self._as_list(adversary_sim.get("recent_runs"))
        for row in recent_runs:
            run = self._as_obj(row)
            if str(run.get("lane") or "").strip() != "scrapling_traffic":
                continue
            coverage = self._as_obj(run.get("owned_surface_coverage"))
            if not coverage:
                continue
            return {
                "run_id": str(run.get("run_id") or "").strip(),
                "overall_status": str(coverage.get("overall_status") or "").strip(),
                "required_surface_ids": [
                    str(value).strip()
                    for value in self._as_list(coverage.get("required_surface_ids"))
                    if str(value).strip()
                ],
                "blocking_surface_ids": [
                    str(value).strip()
                    for value in self._as_list(coverage.get("blocking_surface_ids"))
                    if str(value).strip()
                ],
                "observed_fulfillment_modes": [
                    str(value).strip()
                    for value in self._as_list(run.get("observed_fulfillment_modes"))
                    if str(value).strip()
                ],
            }
        return {
            "run_id": "",
            "overall_status": "",
            "required_surface_ids": [],
            "blocking_surface_ids": [],
            "observed_fulfillment_modes": [],
        }

    def poll_recent_scrapling_run_coverage(self) -> Dict[str, Any]:
        deadline = time.time() + float(self.timeout_seconds)
        last_seen = {
            "run_id": "",
            "overall_status": "",
            "required_surface_ids": [],
            "blocking_surface_ids": [],
            "observed_fulfillment_modes": [],
        }

        while time.time() < deadline:
            operator_snapshot = self.request("GET", "/admin/operator-snapshot")
            if operator_snapshot["status"] != 200:
                time.sleep(1)
                continue
            last_seen = self.recent_scrapling_run_coverage(self._as_obj(operator_snapshot["body"]))
            if (
                last_seen["run_id"]
                and last_seen["overall_status"] == "covered"
                and bool(last_seen["required_surface_ids"])
            ):
                return last_seen
            time.sleep(1)

        return last_seen

    def poll_live_summary_matches_baseline(self, baseline: Dict[str, int]) -> Dict[str, int]:
        deadline = time.time() + float(self.timeout_seconds)
        counts = dict(baseline)

        while time.time() < deadline:
            monitoring = self.request("GET", "/admin/monitoring?hours=24&limit=200")
            if monitoring["status"] != 200:
                time.sleep(1)
                continue
            counts = self.live_summary_counts(self._as_obj(monitoring["body"]))
            if all(
                counts.get(name, 0) == baseline.get(name, 0) for name in baseline.keys()
            ):
                return counts
            time.sleep(1)

        return counts


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Runtime-toggle Scrapling owned-surface coverage and no-impact gate"
    )
    parser.add_argument("--base-url", default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"))
    parser.add_argument("--timeout-seconds", type=int, default=120)
    args = parser.parse_args()

    api_key = os.environ.get("SHUMA_API_KEY", "").strip()
    if not api_key:
        print("[runtime-surface-gate] SHUMA_API_KEY is required", file=sys.stderr)
        return 2

    forwarded_secret = os.environ.get("SHUMA_FORWARDED_IP_SECRET", "")
    health_secret = os.environ.get("SHUMA_HEALTH_SECRET", "")
    gate = RuntimeToggleSurfaceGate(
        base_url=args.base_url,
        api_key=api_key,
        forwarded_secret=forwarded_secret,
        health_secret=health_secret,
        timeout_seconds=max(10, int(args.timeout_seconds)),
    )

    try:
        gate.ensure_health()
        gate.clear_loopback_bans()
        gate.configure_runtime_surface_profile()
        live_summary_baseline = gate.read_live_summary_counts()
        gate.toggle(True, "on")
        coverage = gate.poll_recent_scrapling_run_coverage()
        live_summary_counts = gate.poll_live_summary_matches_baseline(live_summary_baseline)
    except Exception as exc:  # noqa: BLE001
        print(f"[runtime-surface-gate] error: {exc}", file=sys.stderr)
        try:
            gate.toggle(False, "off-error")
        except Exception:
            pass
        return 1

    try:
        gate.toggle(False, "off")
    except Exception as exc:  # noqa: BLE001
        print(f"[runtime-surface-gate] warning: failed to toggle off: {exc}", file=sys.stderr)

    if coverage.get("overall_status") != "covered":
        print(
            "[runtime-surface-gate] missing covered Scrapling owned-surface receipt",
            file=sys.stderr,
        )
        print(
            f"[runtime-surface-gate] coverage={json.dumps(coverage, sort_keys=True)}",
            file=sys.stderr,
        )
        return 1

    leaked = live_summary_leaks(live_summary_counts, live_summary_baseline)
    if leaked:
        print(
            "[runtime-surface-gate] live-only monitoring summary was polluted by sim traffic: "
            + json.dumps(leaked, sort_keys=True),
            file=sys.stderr,
        )
        print(
            f"[runtime-surface-gate] coverage={json.dumps(coverage, sort_keys=True)}",
            file=sys.stderr,
        )
        return 1

    print(
        "[runtime-surface-gate] PASS coverage="
        + json.dumps(coverage, sort_keys=True)
        + " live_summary="
        + json.dumps(live_summary_counts, sort_keys=True)
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
