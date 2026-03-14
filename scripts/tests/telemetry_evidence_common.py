#!/usr/bin/env python3
"""Shared helpers for live telemetry evidence scripts."""

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def compression_ratio_percent(plain_bytes: int, compressed_bytes: int) -> float | None:
    if plain_bytes <= 0 or compressed_bytes <= 0 or compressed_bytes >= plain_bytes:
        if plain_bytes > 0 and compressed_bytes == plain_bytes:
            return 0.0
        return None
    ratio = (1.0 - (compressed_bytes / plain_bytes)) * 100.0
    return round(ratio, 2)


def evaluate_budget_report(
    *,
    bootstrap_measurement: dict,
    delta_measurement: dict,
    bootstrap_budget_ms: float,
    delta_budget_ms: float,
) -> dict[str, float | bool]:
    bootstrap_latency_ms = float(bootstrap_measurement.get("latency_ms", 0.0) or 0.0)
    delta_latency_ms = float(delta_measurement.get("latency_ms", 0.0) or 0.0)
    return {
        "bootstrap_budget_ms": float(bootstrap_budget_ms),
        "bootstrap_within_budget": bootstrap_latency_ms <= float(bootstrap_budget_ms),
        "delta_budget_ms": float(delta_budget_ms),
        "delta_within_budget": delta_latency_ms <= float(delta_budget_ms),
    }


def summarize_recent_event_rows(rows: list[dict[str, Any]]) -> dict[str, Any]:
    reason_counts: dict[str, int] = {}
    event_counts: dict[str, int] = {}
    js_verification_rows = 0
    compact_js_verification_rows = 0
    legacy_js_verification_rows = 0

    for row in rows:
        if not isinstance(row, dict):
            continue
        reason = str(row.get("reason") or "")
        event = str(row.get("event") or "")
        if reason:
            reason_counts[reason] = reason_counts.get(reason, 0) + 1
        if event:
            event_counts[event] = event_counts.get(event, 0) + 1
        if reason != "js_verification":
            continue
        js_verification_rows += 1
        compact_shape = (
            row.get("event") == "Challenge"
            and row.get("outcome_code") == "required"
            and row.get("taxonomy") == {"level": "L4_VERIFY_JS"}
            and "outcome" not in row
        )
        if compact_shape:
            compact_js_verification_rows += 1
        else:
            legacy_js_verification_rows += 1

    challenge_rows = event_counts.get("Challenge", 0)
    return {
        "sample_count": len([row for row in rows if isinstance(row, dict)]),
        "event_counts": dict(sorted(event_counts.items())),
        "reason_counts": dict(sorted(reason_counts.items())),
        "challenge_rows": challenge_rows,
        "js_verification_rows": js_verification_rows,
        "compact_js_verification_rows": compact_js_verification_rows,
        "legacy_js_verification_rows": legacy_js_verification_rows,
        "challenge_heavy_sample": js_verification_rows >= 10,
        "low_volume_sample": len([row for row in rows if isinstance(row, dict)]) < 20,
    }
