#!/usr/bin/env python3
"""Shared helpers for live telemetry evidence scripts."""

from __future__ import annotations

from datetime import datetime, timezone


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

