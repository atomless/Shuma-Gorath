"""Shared identity-envelope helpers for adversarial worker realism."""

from __future__ import annotations

from typing import Any


SUPPORTED_IDENTITY_CLASSES = {"residential", "mobile", "datacenter"}


def normalize_identity_pool_entries(raw_value: Any, *, field_name: str) -> list[dict[str, str]]:
    if not isinstance(raw_value, list):
        return []
    normalized: list[dict[str, str]] = []
    for index, item in enumerate(raw_value):
        if not isinstance(item, dict):
            continue
        label = str(item.get("label") or "").strip()
        proxy_url = str(item.get("proxy_url") or "").strip()
        identity_class = str(item.get("identity_class") or "").strip()
        country_code = str(item.get("country_code") or "").strip().upper()
        if (
            not label
            or not proxy_url
            or "\r" in proxy_url
            or "\n" in proxy_url
            or identity_class not in SUPPORTED_IDENTITY_CLASSES
            or len(country_code) != 2
            or not country_code.isalpha()
        ):
            raise RuntimeError(
                f"{field_name}[{index}] must include valid label, proxy_url, "
                "identity_class, and two-letter country_code"
            )
        normalized.append(
            {
                "label": label,
                "proxy_url": proxy_url,
                "identity_class": identity_class,
                "country_code": country_code,
            }
        )
    return normalized


def summarize_identity_realism(
    profile: dict[str, Any],
    *,
    pool_entries: list[dict[str, str]] | None = None,
    fixed_proxy_url: str | None = None,
    observed_country_codes: list[str] | None = None,
) -> dict[str, Any]:
    envelope = dict(profile.get("identity_envelope") or {})
    identity_classes = [
        str(item).strip()
        for item in list(envelope.get("identity_classes") or [])
        if str(item).strip()
    ]
    geo_affinity_mode = str(envelope.get("geo_affinity_mode") or "").strip() or "pool_aligned"
    session_stickiness = (
        str(envelope.get("session_stickiness") or "").strip() or "stable_per_identity"
    )
    normalized_pool_entries = list(pool_entries or [])
    if len(normalized_pool_entries) >= 2:
        status = "pool_backed"
    elif normalized_pool_entries or str(fixed_proxy_url or "").strip():
        status = "fixed_proxy"
    else:
        status = "degraded_local"
    countries = [
        str(item).strip().upper()
        for item in list(observed_country_codes or [])
        if str(item).strip()
    ]
    if not countries:
        countries = [
            str(entry.get("country_code") or "").strip().upper()
            for entry in normalized_pool_entries
            if str(entry.get("country_code") or "").strip()
        ]
    unique_countries: list[str] = []
    for item in countries:
        if item not in unique_countries:
            unique_countries.append(item)
    return {
        "identity_realism_status": status,
        "identity_envelope_classes": identity_classes,
        "geo_affinity_mode": geo_affinity_mode,
        "session_stickiness": session_stickiness,
        "observed_country_codes": unique_countries,
    }
