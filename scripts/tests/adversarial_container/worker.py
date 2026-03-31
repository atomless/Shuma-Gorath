#!/usr/bin/env python3
"""Container-side black-box adversary worker."""

from __future__ import annotations

import json
import os
import sys
import time
from concurrent.futures import ThreadPoolExecutor
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Dict, List

from scripts.tests.frontier_action_contract import (
    FrontierActionContractError,
    FrontierActionValidationError,
    load_frontier_action_contract,
    resolve_frontier_actions,
)
from scripts.tests.frontier_capability_envelope import (
    parse_action_capability_envelopes,
    validate_action_capability_envelopes,
)


FORBIDDEN_ENV_PREFIXES = ("SHUMA_",)
FORBIDDEN_ENV_KEYS = {
    "SHUMA_API_KEY",
    "SHUMA_ADMIN_READONLY_API_KEY",
    "SHUMA_JS_SECRET",
    "SHUMA_CHALLENGE_SECRET",
    "SHUMA_HEALTH_SECRET",
    "SHUMA_FORWARDED_IP_SECRET",
    "SHUMA_SIM_TELEMETRY_SECRET",
}
DEFAULT_CONTRACT_DIR = Path(os.environ.get("BLACKBOX_CONTRACT_DIR", "scripts/tests/adversarial"))
LANE_CONTRACT_PATH = DEFAULT_CONTRACT_DIR / "lane_contract.v1.json"
SIM_TAG_CONTRACT_PATH = DEFAULT_CONTRACT_DIR / "sim_tag_contract.v1.json"
FRONTIER_ACTION_CONTRACT_PATH = DEFAULT_CONTRACT_DIR / "frontier_action_contract.v1.json"
FRONTIER_ACTIONS_ENV = "BLACKBOX_ACTIONS"
CAPABILITY_ENVELOPES_ENV = "BLACKBOX_ACTION_ENVELOPES"
CAPABILITY_VERIFY_KEY_ENV = "BLACKBOX_CAPABILITY_VERIFY_KEY"
REQUEST_REALISM_PLAN_ENV = "BLACKBOX_REQUEST_REALISM_PLAN"
REQUEST_REALISM_PLAN_SCHEMA_VERSION = "adversary-sim-llm-request-realism-plan.v1"


def bool_env(name: str, default: bool = False) -> bool:
    raw = os.environ.get(name)
    if raw is None:
        return default
    return str(raw).strip().lower() in {"1", "true", "yes", "on"}


def parse_positive_int(name: str, default: int) -> int:
    raw = os.environ.get(name, str(default))
    try:
        parsed = int(str(raw).strip())
    except Exception:
        return default
    return max(1, parsed)


def has_forbidden_env(observed_keys: List[str]) -> bool:
    for key in observed_keys:
        if any(key.startswith(prefix) for prefix in FORBIDDEN_ENV_PREFIXES):
            return True
        if key in FORBIDDEN_ENV_KEYS:
            return True
    return False


def load_lane_contract(path: Path = LANE_CONTRACT_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"lane contract not found: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError("lane contract must be a JSON object")
    return payload


def load_sim_tag_contract(path: Path = SIM_TAG_CONTRACT_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"sim tag contract not found: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError("sim tag contract must be a JSON object")
    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "sim-tag-contract.v1":
        raise RuntimeError(
            f"sim tag contract schema_version must be sim-tag-contract.v1 (got {schema_version})"
        )
    return payload


SIM_TAG_CONTRACT = load_sim_tag_contract()
SIM_TAG_HEADERS = {
    str(key): str(value).strip().lower()
    for key, value in dict(SIM_TAG_CONTRACT.get("headers") or {}).items()
    if str(key).strip() and str(value).strip()
}
SIM_TAG_HEADER_RUN_ID = SIM_TAG_HEADERS.get("sim_run_id", "x-shuma-sim-run-id")
SIM_TAG_HEADER_PROFILE = SIM_TAG_HEADERS.get("sim_profile", "x-shuma-sim-profile")
SIM_TAG_HEADER_LANE = SIM_TAG_HEADERS.get("sim_lane", "x-shuma-sim-lane")
SIM_TAG_HEADER_TIMESTAMP = SIM_TAG_HEADERS.get("sim_timestamp", "x-shuma-sim-ts")
SIM_TAG_HEADER_NONCE = SIM_TAG_HEADERS.get("sim_nonce", "x-shuma-sim-nonce")
SIM_TAG_HEADER_SIGNATURE = SIM_TAG_HEADERS.get("sim_signature", "x-shuma-sim-signature")
SIM_TAG_CANONICAL_PREFIX = str(
    dict(SIM_TAG_CONTRACT.get("canonical") or {}).get("prefix") or "sim-tag.v1"
).strip()
SIM_TAG_CANONICAL_SEPARATOR = str(
    dict(SIM_TAG_CONTRACT.get("canonical") or {}).get("separator") or "\n"
)
SIM_TAG_ENVELOPE_ENV = "BLACKBOX_SIM_TAG_ENVELOPES"


def lane_required_sim_headers(lane_contract: Dict[str, Any]) -> List[str]:
    attacker = lane_contract.get("attacker")
    if not isinstance(attacker, dict):
        return []
    headers = attacker.get("required_sim_headers")
    if not isinstance(headers, list):
        return []
    return [str(item).strip().lower() for item in headers if str(item).strip()]


def lane_forbidden_headers(lane_contract: Dict[str, Any]) -> List[str]:
    attacker = lane_contract.get("attacker")
    if not isinstance(attacker, dict):
        return []
    headers = attacker.get("forbidden_headers")
    if not isinstance(headers, list):
        return []
    return [str(item).strip().lower() for item in headers if str(item).strip()]


def parse_sim_tag_envelopes(raw_value: str) -> List[Dict[str, str]]:
    text = str(raw_value or "").strip()
    if not text:
        return []
    try:
        payload = json.loads(text)
    except Exception:
        return []
    if not isinstance(payload, list):
        return []

    envelopes: List[Dict[str, str]] = []
    seen_nonces = set()
    for item in payload:
        if not isinstance(item, dict):
            return []
        timestamp = str(item.get("ts") or "").strip()
        nonce = str(item.get("nonce") or "").strip()
        signature = str(item.get("signature") or "").strip()
        if not timestamp or not nonce or not signature:
            return []
        if nonce in seen_nonces:
            return []
        seen_nonces.add(nonce)
        envelopes.append({"ts": timestamp, "nonce": nonce, "signature": signature})
    return envelopes


def workspace_mount_absent() -> bool:
    try:
        mounts = open("/proc/mounts", "r", encoding="utf-8", errors="replace").read().lower()
    except Exception:
        return False
    markers = ("shuma-gorath", "/users/jamestindall/projects", "/workspace")
    return not any(marker in mounts for marker in markers)


def enforce_allowlist(url: str, allowed_origins: List[str]) -> bool:
    parsed = urllib.parse.urlparse(url)
    origin = f"{parsed.scheme}://{parsed.netloc}"
    return origin in allowed_origins


def make_request(
    url: str,
    sim_headers: Dict[str, str],
    request_headers: Dict[str, str] | None = None,
    timeout_seconds: float = 10.0,
    proxy_url: str | None = None,
) -> Dict[str, Any]:
    request = urllib.request.Request(url, method="GET")
    normalized_request_headers = {
        str(key).strip().lower(): str(value).strip()
        for key, value in dict(request_headers or {}).items()
        if str(key).strip() and str(value).strip()
    }
    for key, value in normalized_request_headers.items():
        request.add_header(str(key), str(value))
    if "user-agent" not in normalized_request_headers:
        request.add_header("User-Agent", "ShumaContainerBlackBox/1.0")
    for key, value in sim_headers.items():
        request.add_header(key, value)
    start = time.monotonic()
    try:
        opener = None
        if str(proxy_url or "").strip():
            opener = urllib.request.build_opener(
                urllib.request.ProxyHandler(
                    {
                        "http": str(proxy_url).strip(),
                        "https": str(proxy_url).strip(),
                    }
                )
            )
        with (opener.open(request, timeout=timeout_seconds) if opener else urllib.request.urlopen(request, timeout=timeout_seconds)) as response:
            body = response.read().decode("utf-8", errors="replace")
            latency_ms = int((time.monotonic() - start) * 1000)
            return {
                "url": url,
                "status": response.status,
                "latency_ms": latency_ms,
                "body_sample": body[:160],
            }
    except urllib.error.HTTPError as exc:
        latency_ms = int((time.monotonic() - start) * 1000)
        return {
            "url": url,
            "status": int(exc.code),
            "latency_ms": latency_ms,
            "error": f"http_error_{exc.code}",
        }
    except Exception as exc:
        latency_ms = int((time.monotonic() - start) * 1000)
        return {
            "url": url,
            "status": 0,
            "latency_ms": latency_ms,
            "error": str(exc),
        }


def _sleep_gap_ms(delay_ms: int) -> None:
    if delay_ms > 0:
        time.sleep(delay_ms / 1000.0)


def _parse_request_realism_plan(
    raw_value: str,
    *,
    resolved_actions: List[Dict[str, Any]],
    request_budget: int,
) -> Dict[str, Any]:
    actions_count = len(resolved_actions)
    fallback_paths = []
    seen_paths = set()
    for action in resolved_actions:
        path = str(action.get("path") or "").strip() or "/"
        if path in seen_paths:
            continue
        seen_paths.add(path)
        fallback_paths.append(path)

    text = str(raw_value or "").strip()
    if not text:
        return {
            "schema_version": REQUEST_REALISM_PLAN_SCHEMA_VERSION,
            "profile_id": "",
            "planned_activity_budget": actions_count,
            "effective_activity_budget": actions_count,
            "planned_burst_size": max(1, actions_count),
            "effective_burst_size": max(1, actions_count),
            "burst_sizes": [actions_count] if actions_count else [],
            "inter_action_gaps_ms": [0 for _ in range(max(0, actions_count - 1))],
            "focused_page_paths": fallback_paths,
            "focused_page_set_size": len(fallback_paths),
            "identity_realism_status": "degraded_local",
            "identity_envelope_classes": ["residential", "mobile"],
            "geo_affinity_mode": "pool_aligned",
            "session_stickiness": "stable_per_identity",
            "observed_country_codes": [],
            "transport_profile": "urllib_direct",
            "observed_user_agent_families": [],
            "observed_accept_languages": [],
            "session_handles": ["agentic-request-session-1"],
            "action_proxy_urls": [None for _ in range(actions_count)],
            "action_request_headers": [{} for _ in range(actions_count)],
            "recurrence_strategy": "",
            "session_index": 0,
            "reentry_count": 0,
            "max_reentries_per_run": 0,
            "planned_dormant_gap_seconds": 0,
        }

    payload = json.loads(text)
    if not isinstance(payload, dict):
        raise RuntimeError("request realism plan must be a JSON object")
    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != REQUEST_REALISM_PLAN_SCHEMA_VERSION:
        raise RuntimeError(
            "request realism plan schema_version must be "
            f"{REQUEST_REALISM_PLAN_SCHEMA_VERSION}"
        )

    burst_sizes = [max(1, int(item)) for item in list(payload.get("burst_sizes") or [])]
    inter_action_gaps_ms = [
        max(0, int(item)) for item in list(payload.get("inter_action_gaps_ms") or [])
    ]
    focused_page_paths = [
        str(item).strip() or "/"
        for item in list(payload.get("focused_page_paths") or [])
        if str(item).strip()
    ]
    session_handles = [
        str(item).strip()
        for item in list(payload.get("session_handles") or [])
        if str(item).strip()
    ]
    action_proxy_urls = [
        (str(item).strip() or None) if item is not None else None
        for item in list(payload.get("action_proxy_urls") or [])
    ]
    action_request_headers = []
    for item in list(payload.get("action_request_headers") or []):
        if not isinstance(item, dict):
            raise RuntimeError("request realism plan action_request_headers entries must be objects")
        normalized_headers: Dict[str, str] = {}
        for key, value in item.items():
            normalized_key = str(key).strip().lower()
            normalized_value = str(value).strip()
            if normalized_key and normalized_value:
                normalized_headers[normalized_key] = normalized_value
        action_request_headers.append(normalized_headers)

    if sum(burst_sizes) != actions_count:
        raise RuntimeError("request realism plan burst_sizes must sum to resolved action count")
    if len(inter_action_gaps_ms) != max(0, actions_count - 1):
        raise RuntimeError(
            "request realism plan inter_action_gaps_ms must match resolved action transitions"
        )
    if actions_count > request_budget:
        raise RuntimeError("request realism plan must not exceed the worker request budget")
    if action_proxy_urls and len(action_proxy_urls) != actions_count:
        raise RuntimeError(
            "request realism plan action_proxy_urls must align to resolved action count"
        )
    if action_request_headers and len(action_request_headers) != actions_count:
        raise RuntimeError(
            "request realism plan action_request_headers must align to resolved action count"
        )

    planned_activity_budget = max(actions_count, int(payload.get("planned_activity_budget") or actions_count))
    effective_activity_budget = int(payload.get("effective_activity_budget") or actions_count)
    if effective_activity_budget != actions_count:
        raise RuntimeError(
            "request realism plan effective_activity_budget must match resolved action count"
        )
    planned_burst_size = max(1, int(payload.get("planned_burst_size") or max(1, max(burst_sizes or [1]))))
    effective_burst_size = max(1, int(payload.get("effective_burst_size") or max(burst_sizes or [1])))
    concurrency_group_sizes = [
        max(1, int(item)) for item in list(payload.get("concurrency_group_sizes") or burst_sizes)
    ]
    if len(concurrency_group_sizes) != len(burst_sizes):
        raise RuntimeError(
            "request realism plan concurrency_group_sizes must align to burst_sizes"
        )
    if any(group_size > burst_size for group_size, burst_size in zip(concurrency_group_sizes, burst_sizes)):
        raise RuntimeError(
            "request realism plan concurrency_group_sizes must not exceed burst_sizes"
        )
    peak_concurrent_activities = max(
        1,
        int(payload.get("peak_concurrent_activities") or max(concurrency_group_sizes or [1])),
    )
    identity_realism_status = str(payload.get("identity_realism_status") or "").strip() or "degraded_local"
    identity_envelope_classes = [
        str(item).strip()
        for item in list(payload.get("identity_envelope_classes") or ["residential", "mobile"])
        if str(item).strip()
    ]
    geo_affinity_mode = str(payload.get("geo_affinity_mode") or "").strip() or "pool_aligned"
    session_stickiness = (
        str(payload.get("session_stickiness") or "").strip() or "stable_per_identity"
    )
    observed_country_codes = [
        str(item).strip().upper()
        for item in list(payload.get("observed_country_codes") or [])
        if str(item).strip()
    ]
    observed_user_agent_families = [
        str(item).strip()
        for item in list(payload.get("observed_user_agent_families") or [])
        if str(item).strip()
    ]
    observed_accept_languages = [
        str(item).strip()
        for item in list(payload.get("observed_accept_languages") or [])
        if str(item).strip()
    ]
    transport_profile = str(payload.get("transport_profile") or "").strip() or "urllib_direct"
    recurrence_strategy = str(payload.get("recurrence_strategy") or "").strip()
    session_index = max(0, int(payload.get("session_index") or 0))
    reentry_count = max(0, int(payload.get("reentry_count") or 0))
    max_reentries_per_run = max(0, int(payload.get("max_reentries_per_run") or 0))
    planned_dormant_gap_seconds = max(
        0, int(payload.get("planned_dormant_gap_seconds") or 0)
    )

    return {
        "schema_version": schema_version,
        "profile_id": str(payload.get("profile_id") or "").strip(),
        "planned_activity_budget": planned_activity_budget,
        "effective_activity_budget": effective_activity_budget,
        "planned_burst_size": planned_burst_size,
        "effective_burst_size": effective_burst_size,
        "burst_sizes": burst_sizes,
        "concurrency_group_sizes": concurrency_group_sizes,
        "peak_concurrent_activities": peak_concurrent_activities,
        "inter_action_gaps_ms": inter_action_gaps_ms,
        "focused_page_paths": focused_page_paths or fallback_paths,
        "focused_page_set_size": int(payload.get("focused_page_set_size") or len(focused_page_paths or fallback_paths)),
        "identity_realism_status": identity_realism_status,
        "identity_envelope_classes": identity_envelope_classes,
        "geo_affinity_mode": geo_affinity_mode,
        "session_stickiness": session_stickiness,
        "observed_country_codes": observed_country_codes,
        "transport_profile": transport_profile,
        "observed_user_agent_families": observed_user_agent_families,
        "observed_accept_languages": observed_accept_languages,
        "session_handles": session_handles or ["agentic-request-session-1"],
        "action_proxy_urls": action_proxy_urls or [None for _ in range(actions_count)],
        "action_request_headers": action_request_headers or [{} for _ in range(actions_count)],
        "recurrence_strategy": recurrence_strategy,
        "session_index": session_index,
        "reentry_count": reentry_count,
        "max_reentries_per_run": max_reentries_per_run,
        "planned_dormant_gap_seconds": planned_dormant_gap_seconds,
    }


def execute_resolved_actions_with_realism(
    *,
    resolved_actions: List[Dict[str, Any]],
    request_realism_plan: Dict[str, Any],
    request_budget: int,
    time_budget_seconds: int,
    start: float,
    allowed_origins: List[str],
    sim_headers: Dict[str, str],
    sim_tag_envelopes: List[Dict[str, str]],
    policy_audit: List[Dict[str, Any]],
    run_id: str,
) -> Dict[str, Any]:
    traffic: List[Dict[str, Any]] = []
    errors: List[str] = []
    statuses: List[int] = []
    requests_sent = 0
    used_gaps_ms: List[int] = []
    burst_sizes_executed: List[int] = []
    burst_sizes = list(request_realism_plan.get("burst_sizes") or [])
    current_burst_statuses: List[int] = []
    stop_reason = "activity_sequence_exhausted"
    burst_sizes = list(request_realism_plan.get("burst_sizes") or [])
    concurrency_group_sizes = list(request_realism_plan.get("concurrency_group_sizes") or [])
    inter_action_gaps_ms = list(request_realism_plan.get("inter_action_gaps_ms") or [])
    action_proxy_urls = list(request_realism_plan.get("action_proxy_urls") or [])
    action_request_headers = list(request_realism_plan.get("action_request_headers") or [])

    for burst_index, planned_burst_size in enumerate(burst_sizes):
        if requests_sent >= request_budget:
            stop_reason = "request_budget_exhausted"
            break
        if requests_sent >= len(resolved_actions):
            stop_reason = "activity_sequence_exhausted"
            break
        if (time.monotonic() - start) >= time_budget_seconds:
            errors.append("time_budget_exhausted")
            append_policy_audit_event(
                policy_audit,
                stage="execution",
                decision="deny",
                code="time_budget_exhausted",
                action=resolved_actions[requests_sent],
            )
            stop_reason = "time_budget_exhausted"
            break
        if requests_sent > 0:
            gap_ms = int(inter_action_gaps_ms[requests_sent - 1])
            used_gaps_ms.append(gap_ms)
            _sleep_gap_ms(gap_ms)

        burst_end = min(len(resolved_actions), requests_sent + int(planned_burst_size))
        burst_actions = list(resolved_actions[requests_sent:burst_end])
        if requests_sent + len(burst_actions) > len(sim_tag_envelopes):
            errors.append("sim_tag_envelopes_exhausted")
            append_policy_audit_event(
                policy_audit,
                stage="execution",
                decision="deny",
                code="sim_tag_envelopes_exhausted",
                action=burst_actions[0] if burst_actions else None,
            )
            stop_reason = "sim_tag_envelopes_exhausted"
            break

        prepared_burst: List[Dict[str, Any]] = []
        burst_denied = False
        for offset, action in enumerate(burst_actions):
            envelope = sim_tag_envelopes[requests_sent + offset]
            request_headers = dict(sim_headers)
            request_headers[SIM_TAG_HEADER_TIMESTAMP] = envelope["ts"]
            request_headers[SIM_TAG_HEADER_NONCE] = envelope["nonce"]
            request_headers[SIM_TAG_HEADER_SIGNATURE] = envelope["signature"]
            url = str(action.get("url") or "")
            if not enforce_allowlist(url, allowed_origins):
                errors.append(f"egress_disallowed:{url}")
                append_policy_audit_event(
                    policy_audit,
                    stage="execution",
                    decision="deny",
                    code="egress_disallowed",
                    detail=url,
                    action=action,
                )
                stop_reason = "egress_disallowed"
                burst_denied = True
                break
            prepared_burst.append(
                {
                    "action": action,
                    "url": url,
                    "request_headers": request_headers,
                    "action_request_headers": action_request_headers[requests_sent + offset]
                    if requests_sent + offset < len(action_request_headers)
                    else {},
                    "proxy_url": action_proxy_urls[requests_sent + offset]
                    if requests_sent + offset < len(action_proxy_urls)
                    else None,
                }
            )
        if burst_denied:
            break

        for offset, _ in enumerate(prepared_burst, start=1):
            emit_heartbeat(run_id, "before_action", action_index=requests_sent + offset)

        max_workers = max(1, int(concurrency_group_sizes[burst_index] if burst_index < len(concurrency_group_sizes) else len(prepared_burst)))
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            burst_results = list(
                executor.map(
                    lambda item: make_request(
                        item["url"],
                        item["request_headers"],
                        request_headers=item.get("action_request_headers"),
                        proxy_url=item.get("proxy_url"),
                    )
                    if item.get("proxy_url")
                    else make_request(
                        item["url"],
                        item["request_headers"],
                        request_headers=item.get("action_request_headers"),
                    ),
                    prepared_burst,
                )
            )

        for offset, (prepared, result) in enumerate(zip(prepared_burst, burst_results), start=1):
            action = dict(prepared["action"])
            result["action_index"] = action.get("action_index")
            result["action_type"] = action.get("action_type")
            result["path"] = action.get("path")
            if str(action.get("label") or "").strip():
                result["label"] = str(action.get("label")).strip()
            traffic.append(result)
            requests_sent += 1
            statuses.append(int(result.get("status", 0)))
            current_burst_statuses.append(int(result.get("status", 0)))
            emit_heartbeat(run_id, "after_action", action_index=requests_sent)

            if int(result.get("status", 0)) == 0:
                errors.append(str(result.get("error") or "request_failed"))
                append_policy_audit_event(
                    policy_audit,
                    stage="execution",
                    decision="deny",
                    code="request_failed",
                    detail=str(result.get("error") or "request_failed"),
                    action=action,
                )
                stop_reason = "transport_error"
                break
        burst_sizes_executed.append(len(prepared_burst))
        if len(prepared_burst) > 1:
            used_gaps_ms.extend([0] * (len(prepared_burst) - 1))
        if stop_reason == "transport_error":
            break
        if current_burst_statuses and all(status in {403, 429} for status in current_burst_statuses):
            stop_reason = "response_pressure_stop"
            break
        current_burst_statuses = []

    realism_receipt = {
        "schema_version": "sim-lane-realism-receipt.v1",
        "profile_id": str(request_realism_plan.get("profile_id") or ""),
        "planned_activity_budget": int(request_realism_plan.get("planned_activity_budget") or requests_sent),
        "effective_activity_budget": int(request_realism_plan.get("effective_activity_budget") or requests_sent),
        "planned_burst_size": int(request_realism_plan.get("planned_burst_size") or 1),
        "effective_burst_size": int(request_realism_plan.get("effective_burst_size") or 1),
        "activity_count": requests_sent,
        "burst_count": len(burst_sizes_executed),
        "burst_sizes": burst_sizes_executed,
        "inter_activity_gaps_ms": used_gaps_ms[: max(0, requests_sent - 1)],
        "focused_page_set_size": len(set(request_realism_plan.get("focused_page_paths") or [])),
        "concurrency_group_sizes": list(burst_sizes_executed),
        "peak_concurrent_activities": max(burst_sizes_executed or [1]),
        "transport_profile": str(request_realism_plan.get("transport_profile") or "urllib_direct"),
        "observed_user_agent_families": list(
            request_realism_plan.get("observed_user_agent_families") or []
        ),
        "observed_accept_languages": list(
            request_realism_plan.get("observed_accept_languages") or []
        ),
        "identity_realism_status": str(request_realism_plan.get("identity_realism_status") or "degraded_local"),
        "identity_envelope_classes": list(request_realism_plan.get("identity_envelope_classes") or []),
        "geo_affinity_mode": str(request_realism_plan.get("geo_affinity_mode") or "pool_aligned"),
        "session_stickiness": str(request_realism_plan.get("session_stickiness") or "stable_per_identity"),
        "observed_country_codes": list(request_realism_plan.get("observed_country_codes") or []),
        "session_handles": list(request_realism_plan.get("session_handles") or []),
        "identity_rotation_count": int(request_realism_plan.get("identity_rotation_count") or 0),
        "recurrence_strategy": str(request_realism_plan.get("recurrence_strategy") or ""),
        "session_index": int(request_realism_plan.get("session_index") or 0),
        "reentry_count": int(request_realism_plan.get("reentry_count") or 0),
        "max_reentries_per_run": int(
            request_realism_plan.get("max_reentries_per_run") or 0
        ),
        "planned_dormant_gap_seconds": int(
            request_realism_plan.get("planned_dormant_gap_seconds") or 0
        ),
        "stop_reason": stop_reason,
    }

    return {
        "traffic": traffic,
        "errors": errors,
        "statuses": statuses,
        "requests_sent": requests_sent,
        "realism_receipt": realism_receipt,
    }


def append_policy_audit_event(
    events: List[Dict[str, Any]],
    *,
    stage: str,
    decision: str,
    code: str,
    detail: str = "",
    action: Dict[str, Any] | None = None,
) -> None:
    entry: Dict[str, Any] = {
        "stage": str(stage).strip(),
        "decision": str(decision).strip(),
        "code": str(code).strip(),
        "detail": str(detail).strip(),
        "ts_unix": int(time.time()),
    }
    if isinstance(action, dict):
        entry["action_index"] = int(action.get("action_index") or 0)
        entry["action_type"] = str(action.get("action_type") or "")
        entry["path"] = str(action.get("path") or "")
    events.append(entry)


def emit_heartbeat(run_id: str, stage: str, action_index: int = 0) -> None:
    print(
        "[frontier-heartbeat] "
        f"run_id={str(run_id).strip()} stage={str(stage).strip()} "
        f"action_index={int(action_index)} ts={int(time.time())}",
        file=sys.stderr,
        flush=True,
    )


def main() -> int:
    mode = str(os.environ.get("BLACKBOX_MODE", "blackbox")).strip().lower()
    base_url = str(os.environ.get("BLACKBOX_BASE_URL", "")).strip().rstrip("/")
    run_id = str(os.environ.get("BLACKBOX_RUN_ID", "")).strip() or f"container-{int(time.time())}"
    allowed_origins_raw = str(os.environ.get("BLACKBOX_ALLOWED_ORIGINS", "")).strip()
    allowed_origins = [origin.strip() for origin in allowed_origins_raw.split(",") if origin.strip()]
    request_budget = parse_positive_int("BLACKBOX_REQUEST_BUDGET", 24)
    time_budget_seconds = parse_positive_int("BLACKBOX_TIME_BUDGET_SECONDS", 120)
    start = time.monotonic()
    lane_contract_error = ""
    lane_contract: Dict[str, Any] = {}
    try:
        lane_contract = load_lane_contract()
    except Exception as exc:
        lane_contract_error = str(exc)
    frontier_action_contract_error = ""
    frontier_action_contract: Dict[str, Any] = {}
    try:
        frontier_action_contract = load_frontier_action_contract(FRONTIER_ACTION_CONTRACT_PATH)
    except FrontierActionContractError as exc:
        frontier_action_contract_error = str(exc)

    observed_env_keys = sorted(list(os.environ.keys()))
    forbidden_env_present = has_forbidden_env(observed_env_keys)
    non_root = os.getuid() != 0
    no_workspace_mount = workspace_mount_absent()
    admin_credentials_absent = not forbidden_env_present
    tooling_limited = True  # Worker is intentionally limited to urllib-based HTTP traffic.
    egress_allowlist_enforced = True
    ephemeral_run_identity = bool(run_id)

    payload: Dict[str, Any] = {
        "schema_version": "adversarial-container-worker.v1",
        "mode": mode,
        "run_id": run_id,
        "lane_contract_schema_version": str(lane_contract.get("schema_version") or ""),
        "lane_contract_error": lane_contract_error,
        "frontier_action_contract_schema_version": str(
            frontier_action_contract.get("schema_version") or ""
        ),
        "frontier_action_contract_error": frontier_action_contract_error,
        "runtime_hardening_non_root": non_root,
        "workspace_mount_absent": no_workspace_mount,
        "admin_credentials_absent": admin_credentials_absent,
        "tooling_limited": tooling_limited,
        "egress_allowlist_enforced": egress_allowlist_enforced,
        "ephemeral_run_identity": ephemeral_run_identity,
        "action_validation_passed": False,
        "resolved_action_count": 0,
        "resolved_actions": [],
        "policy_audit": [],
        "policy_violation_count": 0,
        "policy_violation_blocking": False,
        "capability_validation_passed": False,
        "capability_envelope_count": 0,
        "capability_validation_errors": [],
        "command_channel_direction": "host_to_worker_one_way",
        "evidence_channel_append_only": True,
        "control_plane_mutation_allowed": False,
        "observed_env_keys": [key for key in observed_env_keys if key.startswith("BLACKBOX_")],
        "request_budget": request_budget,
        "time_budget_seconds": time_budget_seconds,
        "traffic": [],
        "realism_receipt": None,
        "passed": False,
    }

    contract_pass = (
        non_root
        and no_workspace_mount
        and admin_credentials_absent
        and tooling_limited
        and egress_allowlist_enforced
        and ephemeral_run_identity
        and not lane_contract_error
        and not frontier_action_contract_error
    )
    emit_heartbeat(run_id, "worker_started", action_index=0)

    if mode == "isolation":
        payload["passed"] = contract_pass
        print(json.dumps(payload, separators=(",", ":")))
        return 0 if payload["passed"] else 1

    if mode != "blackbox":
        payload["error"] = f"unsupported_mode:{mode}"
        print(json.dumps(payload, separators=(",", ":")))
        return 1

    if not base_url:
        payload["error"] = "missing_base_url"
        print(json.dumps(payload, separators=(",", ":")))
        return 1
    if not allowed_origins:
        payload["error"] = "missing_allowed_origins"
        print(json.dumps(payload, separators=(",", ":")))
        return 1

    statuses: List[int] = []
    errors: List[str] = []
    policy_audit = payload["policy_audit"]
    sim_tag_envelopes = parse_sim_tag_envelopes(os.environ.get(SIM_TAG_ENVELOPE_ENV, ""))
    if not sim_tag_envelopes:
        errors.append("missing_or_invalid_sim_tag_envelopes")
        append_policy_audit_event(
            policy_audit,
            stage="validation",
            decision="deny",
            code="missing_or_invalid_sim_tag_envelopes",
        )
    sim_headers = {
        SIM_TAG_HEADER_RUN_ID: run_id,
        SIM_TAG_HEADER_PROFILE: mode,
        SIM_TAG_HEADER_LANE: "container_blackbox",
    }
    sim_header_names = {
        SIM_TAG_HEADER_RUN_ID,
        SIM_TAG_HEADER_PROFILE,
        SIM_TAG_HEADER_LANE,
        SIM_TAG_HEADER_TIMESTAMP,
        SIM_TAG_HEADER_NONCE,
        SIM_TAG_HEADER_SIGNATURE,
    }
    required_sim_headers = set(lane_required_sim_headers(lane_contract))
    missing_sim_headers = sorted([header for header in required_sim_headers if header not in sim_header_names])
    if missing_sim_headers:
        errors.append("missing_sim_headers:" + ",".join(missing_sim_headers))
        append_policy_audit_event(
            policy_audit,
            stage="validation",
            decision="deny",
            code="missing_sim_headers",
            detail=",".join(missing_sim_headers),
        )
    forbidden_sim_headers = set(lane_forbidden_headers(lane_contract))
    forbidden_present = sorted([header for header in sim_header_names if header in forbidden_sim_headers])
    if forbidden_present:
        errors.append("forbidden_sim_headers:" + ",".join(forbidden_present))
        append_policy_audit_event(
            policy_audit,
            stage="validation",
            decision="deny",
            code="forbidden_sim_headers",
            detail=",".join(forbidden_present),
        )
    resolved_actions: List[Dict[str, Any]] = []
    capability_validation_errors: List[str] = []
    if not errors:
        try:
            resolved_actions = resolve_frontier_actions(
                os.environ.get(FRONTIER_ACTIONS_ENV, ""),
                contract=frontier_action_contract,
                base_url=base_url,
                allowed_origins=allowed_origins,
                request_budget=request_budget,
            )
            payload["action_validation_passed"] = True
            payload["resolved_action_count"] = len(resolved_actions)
            payload["resolved_actions"] = [
                {
                    "action_index": action.get("action_index"),
                    "action_type": action.get("action_type"),
                    "path": action.get("path"),
                    "label": action.get("label"),
                }
                for action in resolved_actions
            ]
        except FrontierActionValidationError as exc:
            errors.append(f"action_validation_failed:{exc}")
            append_policy_audit_event(
                policy_audit,
                stage="validation",
                decision="deny",
                code="action_validation_failed",
                detail=str(exc),
            )
    if not errors:
        capability_envelopes = parse_action_capability_envelopes(
            os.environ.get(CAPABILITY_ENVELOPES_ENV, "")
        )
        capability_verify_key = str(os.environ.get(CAPABILITY_VERIFY_KEY_ENV, "")).strip()
        payload["capability_envelope_count"] = len(capability_envelopes)
        if not capability_verify_key:
            capability_validation_errors = ["missing_capability_verify_key"]
        else:
            capability_validation_errors = validate_action_capability_envelopes(
                capability_envelopes,
                verify_key=capability_verify_key,
                run_id=run_id,
                actions=resolved_actions,
            )
        payload["capability_validation_errors"] = capability_validation_errors
        if capability_validation_errors:
            errors.append("capability_validation_failed")
            append_policy_audit_event(
                policy_audit,
                stage="validation",
                decision="deny",
                code="capability_validation_failed",
                detail=";".join(capability_validation_errors),
            )
        else:
            payload["capability_validation_passed"] = True

    request_realism_plan = _parse_request_realism_plan(
        os.environ.get(REQUEST_REALISM_PLAN_ENV, ""),
        resolved_actions=resolved_actions,
        request_budget=request_budget,
    )

    execution = execute_resolved_actions_with_realism(
        resolved_actions=resolved_actions,
        request_realism_plan=request_realism_plan,
        request_budget=request_budget,
        time_budget_seconds=time_budget_seconds,
        start=start,
        allowed_origins=allowed_origins,
        sim_headers=sim_headers,
        sim_tag_envelopes=sim_tag_envelopes,
        policy_audit=policy_audit,
        run_id=run_id,
    )

    payload["traffic"] = execution["traffic"]
    payload["realism_receipt"] = execution["realism_receipt"]
    requests_sent = int(execution["requests_sent"])
    statuses = list(execution["statuses"])
    errors.extend(str(item).strip() for item in list(execution["errors"]) if str(item).strip())
    payload["requests_sent"] = requests_sent
    payload["errors"] = errors
    payload["policy_violation_count"] = len(policy_audit)
    payload["policy_violation_blocking"] = bool(policy_audit)
    payload["allowed_statuses"] = [200, 302, 303, 403, 404, 429]

    status_ok = all(status in payload["allowed_statuses"] for status in statuses if status != 0)
    payload["passed"] = contract_pass and requests_sent > 0 and status_ok and not errors
    emit_heartbeat(run_id, "worker_completed", action_index=requests_sent)
    print(json.dumps(payload, separators=(",", ":")))
    return 0 if payload["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
