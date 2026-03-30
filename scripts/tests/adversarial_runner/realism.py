"""Shared deterministic helpers for lane realism sampling and shaping."""

from __future__ import annotations

import hashlib
from typing import Any


def stable_bucket(*parts: Any) -> int:
    material = "|".join(str(part) for part in parts).encode("utf-8")
    digest = hashlib.sha256(material).digest()
    return int.from_bytes(digest[:8], "big", signed=False)


def realism_range_value(range_payload: dict[str, Any], *seed_parts: Any) -> int:
    minimum = max(0, int(range_payload.get("min") or 0))
    maximum = max(minimum, int(range_payload.get("max") or minimum))
    if maximum <= minimum:
        return minimum
    span = maximum - minimum + 1
    return minimum + (stable_bucket(*seed_parts) % span)


def partition_activity_budget(total_activities: int, burst_size: int) -> list[int]:
    normalized_total = max(0, int(total_activities))
    normalized_burst = max(1, int(burst_size))
    bursts: list[int] = []
    remaining = normalized_total
    while remaining > 0:
        current = min(normalized_burst, remaining)
        bursts.append(current)
        remaining -= current
    return bursts
