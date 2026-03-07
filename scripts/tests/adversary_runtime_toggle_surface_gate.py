#!/usr/bin/env python3
"""Runtime-toggle deterministic surface coverage gate.

This validates the dashboard-toggle execution lane (control endpoint + autonomous supervisor)
emits required defense-surface telemetry categories in live runtime mode.
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

    def ensure_health(self) -> None:
        response = self.request("GET", "/health", extra_headers=self._health_headers())
        if response["status"] != 200:
            raise RuntimeError(f"health check failed: status={response['status']} body={response['raw'][:200]}")

    def configure_runtime_surface_profile(self) -> None:
        payload = {
            "defence_modes": {"rate": "both", "geo": "both", "js": "both"},
            "js_required_enforced": True,
            "geo_edge_headers_enabled": True,
            "geo_challenge": ["RU"],
            "geo_maze": [],
            "geo_block": [],
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

    def poll_categories(self) -> Dict[str, bool]:
        deadline = time.time() + float(self.timeout_seconds)
        seen = {
            "challenge": False,
            "js_required": False,
            "pow": False,
            "rate": False,
            "geo": False,
            "maze_or_tarpit": False,
            "fingerprint_or_cdp": False,
            "ban": False,
        }

        while time.time() < deadline:
            monitoring = self.request("GET", "/admin/monitoring?hours=24&limit=200")
            if monitoring["status"] != 200:
                time.sleep(1)
                continue
            body = self._as_obj(monitoring["body"])
            summary = self._as_obj(body.get("summary"))
            details = self._as_obj(body.get("details"))
            events = self._as_list(self._as_obj(details.get("events")).get("recent_events"))

            pow_attempts = self._as_int(self._as_obj(summary.get("pow")).get("total_attempts"))
            rate_violations = self._as_int(
                self._as_obj(summary.get("rate")).get("total_violations")
            )
            geo_violations = self._as_int(self._as_obj(summary.get("geo")).get("total_violations"))
            ban_count = self._as_int(self._as_obj(details.get("analytics")).get("ban_count"))
            cdp_total = self._as_int(
                self._as_obj(self._as_obj(details.get("cdp")).get("stats")).get("total_detections")
            )
            fingerprint_events = self._as_int(
                self._as_obj(self._as_obj(details.get("cdp")).get("fingerprint_stats")).get("events")
            )
            tarpit_progressive = self._as_int(
                self._as_obj(
                    self._as_obj(self._as_obj(details.get("tarpit")).get("metrics")).get("activations")
                ).get("progressive")
            )

            seen["pow"] = seen["pow"] or pow_attempts > 0
            seen["rate"] = seen["rate"] or rate_violations > 0
            seen["geo"] = seen["geo"] or geo_violations > 0
            seen["ban"] = seen["ban"] or ban_count > 0
            seen["fingerprint_or_cdp"] = seen["fingerprint_or_cdp"] or cdp_total > 0 or fingerprint_events > 0
            seen["maze_or_tarpit"] = seen["maze_or_tarpit"] or tarpit_progressive > 0

            for row in events:
                event = self._as_obj(row)
                if not bool(event.get("is_simulation", False)):
                    continue
                name = str(event.get("event") or "").strip().lower()
                reason = str(event.get("reason") or "").strip().lower()
                outcome = str(event.get("outcome") or "").strip().lower()

                if name == "challenge" or "challenge" in reason:
                    seen["challenge"] = True
                if reason == "js_verification":
                    seen["js_required"] = True
                    seen["challenge"] = True
                if "s_js_required_missing" in outcome or "js_verification_required:active" in outcome:
                    seen["js_required"] = True
                if "pow" in reason or "pow_" in outcome:
                    seen["pow"] = True
                if "rate" in reason or "rate_" in outcome:
                    seen["rate"] = True
                if reason.startswith("geo_policy_") or "d_geo_route" in outcome:
                    seen["geo"] = True
                if "maze" in reason or "maze" in outcome or "tarpit" in reason or "tarpit" in outcome:
                    seen["maze_or_tarpit"] = True
                if "fingerprint" in reason or "fingerprint" in outcome or "cdp" in reason or "cdp" in outcome:
                    seen["fingerprint_or_cdp"] = True
                if name == "ban":
                    seen["ban"] = True

            if all(seen.values()):
                return seen
            time.sleep(1)

        return seen


def main() -> int:
    parser = argparse.ArgumentParser(description="Runtime-toggle deterministic telemetry surface gate")
    parser.add_argument("--base-url", default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"))
    parser.add_argument("--timeout-seconds", type=int, default=45)
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
        gate.configure_runtime_surface_profile()
        gate.toggle(True, "on")
        seen = gate.poll_categories()
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

    missing = [name for name, present in seen.items() if not present]
    if missing:
        print(
            "[runtime-surface-gate] missing required categories: "
            + ", ".join(sorted(missing)),
            file=sys.stderr,
        )
        print(f"[runtime-surface-gate] observed={json.dumps(seen, sort_keys=True)}", file=sys.stderr)
        return 1

    print(f"[runtime-surface-gate] PASS observed={json.dumps(seen, sort_keys=True)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
