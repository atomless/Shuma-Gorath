#!/usr/bin/env python3
"""Host-side LLM runtime worker for bounded bot_red_team execution."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import urllib.parse
import subprocess
import sys
import tempfile
import time
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests.adversarial_runner import llm_fulfillment
from scripts.tests.adversarial_container.worker import (
    SIM_TAG_HEADER_LANE,
    SIM_TAG_HEADER_NONCE,
    SIM_TAG_HEADER_PROFILE,
    SIM_TAG_HEADER_RUN_ID,
    SIM_TAG_HEADER_SIGNATURE,
    SIM_TAG_HEADER_TIMESTAMP,
)
from scripts.tests.adversarial_container_runner import build_sim_tag_envelopes
from scripts.tests.adversarial_runner.contracts import (
    normalize_lane_realism_profile,
    resolve_lane_realism_profile,
)
from scripts.tests.adversarial_runner.identity_envelope import (
    normalize_optional_proxy_url,
    normalize_identity_pool_entries,
    summarize_identity_realism,
)
from scripts.tests.adversarial_runner.realism import (
    partition_activity_budget,
    realism_range_value,
)
from scripts.tests.adversarial_runner.transport_envelope import (
    CHROME_DESKTOP_USER_AGENT,
    resolve_browser_transport_observation,
    resolve_request_transport_observation,
)
from scripts.tests.playwright_runtime import build_playwright_env, ensure_playwright_chromium


LLM_RUNTIME_RESULT_SCHEMA_VERSION = "adversary-sim-llm-runtime-result.v1"
REQUEST_MODE_REALISM_PLAN_SCHEMA_VERSION = "adversary-sim-llm-request-realism-plan.v1"
BROWSER_MODE_REALISM_PLAN_SCHEMA_VERSION = "adversary-sim-llm-browser-realism-plan.v1"
DEFAULT_AGENTIC_BROWSER_USER_AGENT = CHROME_DESKTOP_USER_AGENT
DEFAULT_PUBLIC_HINT_PATHS = ["/robots.txt"]


def _ordered_unique_strings(values: list[Any]) -> list[str]:
    ordered: list[str] = []
    seen = set()
    for value in values:
        normalized = str(value or "").strip()
        if not normalized or normalized in seen:
            continue
        seen.add(normalized)
        ordered.append(normalized)
    return ordered


def _capability_state_for_generation(generation_result: dict[str, Any]) -> str:
    generation_source = str(generation_result.get("generation_source") or "").strip()
    if generation_source.startswith("fallback_"):
        return "degraded_fallback"
    if generation_source == "provider_response":
        return "frontier_provider"
    return "runtime_generated"


def _action_targeting_strategy(actions: list[dict[str, Any]]) -> str:
    archive_walk_paths = {"/research/", "/plans/", "/work/", "/page/2/"}
    for action in actions:
        path = str(action.get("path") or "").strip() or "/"
        if path in archive_walk_paths:
            return "archive_walk"
        query = action.get("query")
        if isinstance(query, dict):
            page_value = str(query.get("page") or "").strip()
            if page_value.isdigit() and int(page_value) >= 2:
                return "archive_walk"
    for action in actions:
        path = str(action.get("path") or "").strip() or "/"
        if path in {"/robots.txt", "/sitemap.xml", "/atom.xml"} or path.endswith(".xml"):
            return "discoverability_probe"
    return "single_entrypoint_probe"


class WorkerConfigError(ValueError):
    """Raised when required worker inputs are missing or invalid."""


def _normalized_request_mode_actions(actions: list[dict[str, Any]] | None) -> list[dict[str, Any]]:
    normalized: list[dict[str, Any]] = []
    for index, action in enumerate(list(actions or []), start=1):
        if not isinstance(action, dict):
            continue
        action_type = str(action.get("action_type") or "").strip() or "http_get"
        path = str(action.get("path") or "").strip() or "/"
        label = str(action.get("label") or "").strip()
        query = action.get("query") if isinstance(action.get("query"), dict) else {}
        method = str(action.get("method") or "").strip().upper() or (
            "HEAD" if action_type == "http_head" else "GET"
        )
        normalized.append(
            {
                "action_index": index,
                "action_type": action_type,
                "method": method,
                "path": path,
                "query": {
                    str(key).strip(): str(value).strip()
                    for key, value in dict(query).items()
                    if str(key).strip()
                },
                "label": label or None,
            }
        )
    if normalized:
        return normalized
    return [
        {
            "action_index": 1,
            "action_type": "http_get",
            "method": "GET",
            "path": "/",
            "query": {},
            "label": "root",
        }
    ]


def _normalized_browser_mode_actions(actions: list[dict[str, Any]] | None) -> list[dict[str, Any]]:
    normalized: list[dict[str, Any]] = []
    for index, action in enumerate(list(actions or []), start=1):
        if not isinstance(action, dict):
            continue
        action_type = str(action.get("action_type") or "").strip() or "browser_navigate"
        path = str(action.get("path") or "").strip() or "/"
        label = str(action.get("label") or "").strip()
        normalized.append(
            {
                "action_index": index,
                "action_type": action_type,
                "path": path,
                "label": label or None,
            }
        )
    if normalized:
        return normalized
    return [
        {
            "action_index": 1,
            "action_type": "browser_navigate",
            "path": "/",
            "label": "root",
        }
    ]


def _focused_browser_mode_paths(
    fulfillment_plan: dict[str, Any],
    actions: list[dict[str, Any]],
    *,
    top_level_action_budget: int,
) -> list[str]:
    candidate_paths: list[str] = []
    seen_paths = set()
    for action in actions:
        path = str(action.get("path") or "").strip() or "/"
        if path in seen_paths:
            continue
        seen_paths.add(path)
        candidate_paths.append(path)
    if "/" not in seen_paths:
        candidate_paths.insert(0, "/")

    unique_count = len(candidate_paths)
    min_focus_size = 1 if unique_count <= 1 else 2
    max_focus_size = max(min_focus_size, min(3, unique_count, top_level_action_budget))
    focus_size = realism_range_value(
        {"min": min_focus_size, "max": max_focus_size},
        fulfillment_plan.get("run_id"),
        fulfillment_plan.get("tick_id"),
        "browser_focused_page_set_size",
    )
    return candidate_paths[:focus_size]


def _browser_dwell_intervals_ms(
    fulfillment_plan: dict[str, Any],
    top_level_action_budget: int,
) -> list[int]:
    profile = dict(fulfillment_plan.get("realism_profile") or {})
    dwell_intervals_ms: list[int] = []
    for action_index in range(1, top_level_action_budget):
        dwell_ms = realism_range_value(
            dict(profile.get("navigation_dwell_ms") or {}),
            fulfillment_plan.get("run_id"),
            fulfillment_plan.get("tick_id"),
            fulfillment_plan.get("fulfillment_mode"),
            "navigation_dwell_ms",
            action_index,
        )
        dwell_intervals_ms.append(dwell_ms)
    return dwell_intervals_ms


def _resolve_recurrence_context(
    fulfillment_plan: dict[str, Any], profile: dict[str, Any]
) -> dict[str, int | str]:
    recurrence_context = dict(fulfillment_plan.get("recurrence_context") or {})
    if recurrence_context:
        return {
            "strategy": str(recurrence_context.get("strategy") or ""),
            "reentry_scope": str(recurrence_context.get("reentry_scope") or ""),
            "dormancy_truth_mode": str(
                recurrence_context.get("dormancy_truth_mode") or ""
            ),
            "session_index": int(recurrence_context.get("session_index") or 0),
            "reentry_count": int(recurrence_context.get("reentry_count") or 0),
            "max_reentries_per_run": int(
                recurrence_context.get("max_reentries_per_run") or 0
            ),
            "planned_dormant_gap_seconds": int(
                recurrence_context.get("planned_dormant_gap_seconds") or 0
            ),
            "representative_dormant_gap_seconds": int(
                recurrence_context.get("representative_dormant_gap_seconds") or 0
            ),
        }
    recurrence_envelope = dict(profile.get("recurrence_envelope") or {})
    dormant_gap = recurrence_envelope.get("dormant_gap_seconds")
    dormant_gap_min = 0
    if isinstance(dormant_gap, dict):
        dormant_gap_min = int(dormant_gap.get("min") or 0)
    representative_dormant_gap = recurrence_envelope.get(
        "representative_dormant_gap_seconds"
    )
    representative_dormant_gap_min = 0
    if isinstance(representative_dormant_gap, dict):
        representative_dormant_gap_min = int(representative_dormant_gap.get("min") or 0)
    return {
        "strategy": str(recurrence_envelope.get("strategy") or ""),
        "reentry_scope": str(recurrence_envelope.get("reentry_scope") or ""),
        "dormancy_truth_mode": (
            "accelerated_local_proof"
            if representative_dormant_gap_min > dormant_gap_min
            else "representative_runtime"
        ),
        "session_index": 1,
        "reentry_count": 0,
        "max_reentries_per_run": int(recurrence_envelope.get("max_reentries_per_run") or 0),
        "planned_dormant_gap_seconds": dormant_gap_min,
        "representative_dormant_gap_seconds": representative_dormant_gap_min,
    }


def build_browser_mode_realism_execution_plan(
    *,
    fulfillment_plan: dict[str, Any],
    generation_result: dict[str, Any],
) -> dict[str, Any]:
    capability_envelope = dict(fulfillment_plan.get("capability_envelope") or {})
    profile = normalize_lane_realism_profile(
        fulfillment_plan.get("realism_profile"),
        field_name="llm_fulfillment_plan.realism_profile",
    )
    planned_activity_budget = realism_range_value(
        dict(profile.get("activity_budget") or {}),
        fulfillment_plan.get("run_id"),
        fulfillment_plan.get("tick_id"),
        fulfillment_plan.get("fulfillment_mode"),
        "activity_budget",
    )
    top_level_action_budget = max(
        1,
        min(
            int(capability_envelope.get("max_actions") or 1),
            planned_activity_budget,
        ),
    )
    candidate_actions = _normalized_browser_mode_actions(
        list(generation_result.get("actions") or [])
    )
    focused_page_paths = _focused_browser_mode_paths(
        fulfillment_plan,
        candidate_actions,
        top_level_action_budget=top_level_action_budget,
    )
    dwell_intervals_ms = _browser_dwell_intervals_ms(
        fulfillment_plan,
        top_level_action_budget,
    )
    browser_proxy_url = normalize_optional_proxy_url(
        fulfillment_plan.get("browser_proxy_url"),
        field_name="llm_fulfillment_plan.browser_proxy_url",
    )
    identity_summary = summarize_identity_realism(
        profile,
        fixed_proxy_url=browser_proxy_url,
    )
    browser_transport = resolve_browser_transport_observation(profile)
    recurrence_context = _resolve_recurrence_context(fulfillment_plan, profile)
    capability_state = _capability_state_for_generation(generation_result)
    action_types_attempted = _ordered_unique_strings(
        [action.get("action_type") for action in candidate_actions]
    )
    targeting_strategy = _action_targeting_strategy(candidate_actions)
    return {
        "schema_version": BROWSER_MODE_REALISM_PLAN_SCHEMA_VERSION,
        "profile_id": str(profile.get("profile_id") or ""),
        "capability_state": capability_state,
        "action_types_attempted": action_types_attempted,
        "targeting_strategy": targeting_strategy,
        "planned_activity_budget": planned_activity_budget,
        "effective_activity_budget": top_level_action_budget,
        "top_level_action_budget": top_level_action_budget,
        "focused_page_paths": focused_page_paths,
        "focused_page_set_size": len(focused_page_paths),
        "dwell_intervals_ms": dwell_intervals_ms,
        "session_handles": ["agentic-browser-session-1"],
        "identity_realism_status": identity_summary["identity_realism_status"],
        "identity_provenance_mode": identity_summary["identity_provenance_mode"],
        "identity_envelope_classes": identity_summary["identity_envelope_classes"],
        "geo_affinity_mode": identity_summary["geo_affinity_mode"],
        "session_stickiness": identity_summary["session_stickiness"],
        "observed_country_codes": identity_summary["observed_country_codes"],
        "transport_profile": str(browser_transport.get("transport_profile") or ""),
        "transport_realism_class": str(
            browser_transport.get("transport_realism_class") or ""
        ),
        "transport_emission_basis": str(
            browser_transport.get("transport_emission_basis") or ""
        ),
        "transport_degraded_reason": str(
            browser_transport.get("transport_degraded_reason") or ""
        ),
        "user_agent_family": str(browser_transport.get("user_agent_family") or ""),
        "user_agent": str(browser_transport.get("user_agent") or ""),
        "browser_locale": str(browser_transport.get("browser_locale") or ""),
        "accept_language": str(browser_transport.get("accept_language") or ""),
        "observed_user_agent_families": [str(browser_transport.get("user_agent_family") or "")],
        "observed_accept_languages": [str(browser_transport.get("accept_language") or "")],
        "observed_browser_locales": [str(browser_transport.get("browser_locale") or "")],
        "browser_proxy_url": browser_proxy_url,
        "recurrence_strategy": str(recurrence_context["strategy"]),
        "reentry_scope": str(recurrence_context["reentry_scope"]),
        "dormancy_truth_mode": str(recurrence_context["dormancy_truth_mode"]),
        "session_index": int(recurrence_context["session_index"]),
        "reentry_count": int(recurrence_context["reentry_count"]),
        "max_reentries_per_run": int(recurrence_context["max_reentries_per_run"]),
        "planned_dormant_gap_seconds": int(recurrence_context["planned_dormant_gap_seconds"]),
        "representative_dormant_gap_seconds": int(
            recurrence_context["representative_dormant_gap_seconds"]
        ),
    }


def _focused_request_mode_actions(
    fulfillment_plan: dict[str, Any],
    actions: list[dict[str, Any]],
    *,
    effective_burst_size: int,
) -> list[dict[str, Any]]:
    unique_actions: list[dict[str, Any]] = []
    seen = set()
    for action in actions:
        query = action.get("query") if isinstance(action.get("query"), dict) else {}
        query_key = tuple(
            (str(key).strip(), str(query[key]).strip())
            for key in sorted(query.keys(), key=lambda item: str(item))
            if str(key).strip()
        )
        key = (
            str(action.get("action_type") or ""),
            str(action.get("path") or ""),
            query_key,
        )
        if key in seen:
            continue
        unique_actions.append(action)
        seen.add(key)
    if not unique_actions:
        unique_actions = _normalized_request_mode_actions(None)

    root_actions = [action for action in unique_actions if str(action.get("path") or "") == "/"]
    non_root_actions = [action for action in unique_actions if str(action.get("path") or "") != "/"]
    prioritized_actions = root_actions[:1] + non_root_actions
    if not prioritized_actions:
        prioritized_actions = unique_actions

    unique_count = len(prioritized_actions)
    min_focus_size = 1 if unique_count <= 1 else 2
    max_focus_size = max(min_focus_size, min(4, unique_count, max(2, effective_burst_size)))
    focus_size = realism_range_value(
        {"min": min_focus_size, "max": max_focus_size},
        fulfillment_plan.get("run_id"),
        fulfillment_plan.get("tick_id"),
        "focused_page_set_size",
    )
    return prioritized_actions[:focus_size]


def _inter_action_gaps_ms(
    fulfillment_plan: dict[str, Any],
    burst_sizes: list[int],
) -> list[int]:
    profile = dict(fulfillment_plan.get("realism_profile") or {})
    gaps: list[int] = []
    completed_actions = 0
    for burst_index, burst_size in enumerate(burst_sizes):
        if burst_size <= 1:
            completed_actions += max(0, burst_size)
        else:
            for _ in range(1, int(burst_size)):
                gaps.append(0)
                completed_actions += 1
            completed_actions += 1
        if burst_index >= len(burst_sizes) - 1:
            continue
        gap_ms = realism_range_value(
            dict(profile.get("between_burst_pause_ms") or {}),
            fulfillment_plan.get("run_id"),
            fulfillment_plan.get("tick_id"),
            fulfillment_plan.get("fulfillment_mode"),
            "between_burst_pause_ms",
            burst_index,
            completed_actions,
        )
        gaps.append(gap_ms)
    return gaps


def _request_mode_identity_assignments(
    fulfillment_plan: dict[str, Any],
    profile: dict[str, Any],
    burst_sizes: list[int],
    *,
    action_count: int,
) -> dict[str, Any]:
    fixed_proxy_url = normalize_optional_proxy_url(
        fulfillment_plan.get("request_proxy_url"),
        field_name="llm_fulfillment_plan.request_proxy_url",
    )
    request_identity_pool = normalize_identity_pool_entries(
        fulfillment_plan.get("request_identity_pool"),
        field_name="llm_fulfillment_plan.request_identity_pool",
    )
    action_proxy_urls: list[str | None] = []
    action_identity_rows: list[dict[str, str | None]] = []
    session_handles: list[str] = []
    observed_country_codes: list[str] = []
    if request_identity_pool:
        for burst_index, burst_size in enumerate(burst_sizes):
            entry = dict(request_identity_pool[burst_index % len(request_identity_pool)])
            session_handle = f"agentic-request-session-{entry['label']}"
            if session_handle not in session_handles:
                session_handles.append(session_handle)
            country_code = str(entry.get("country_code") or "").strip().upper()
            if country_code and country_code not in observed_country_codes:
                observed_country_codes.append(country_code)
            for _ in range(int(burst_size)):
                action_proxy_urls.append(str(entry.get("proxy_url") or "").strip() or None)
                action_identity_rows.append(
                    {
                        "country_code": country_code or None,
                        "identity_class": str(entry.get("identity_class") or "").strip() or None,
                    }
                )
        while len(action_proxy_urls) < action_count:
            action_proxy_urls.append(None)
            action_identity_rows.append({"country_code": None, "identity_class": None})
    elif fixed_proxy_url:
        session_handles = ["agentic-request-session-trusted-ingress"]
        action_proxy_urls = [fixed_proxy_url for _ in range(action_count)]
        action_identity_rows = [
            {"country_code": None, "identity_class": None} for _ in range(action_count)
        ]
    else:
        session_handles = ["agentic-request-session-1"]
        action_proxy_urls = [None for _ in range(action_count)]
        action_identity_rows = [
            {"country_code": None, "identity_class": None} for _ in range(action_count)
        ]
    return {
        **summarize_identity_realism(
            profile,
            pool_entries=request_identity_pool,
            fixed_proxy_url=fixed_proxy_url,
            observed_country_codes=observed_country_codes,
        ),
        "action_proxy_urls": action_proxy_urls[:action_count],
        "action_identity_rows": action_identity_rows[:action_count],
        "session_handles": session_handles,
        "identity_rotation_count": max(0, len(session_handles) - 1),
    }


def build_request_mode_realism_execution_plan(
    *,
    fulfillment_plan: dict[str, Any],
    generation_result: dict[str, Any],
) -> dict[str, Any]:
    capability_envelope = dict(fulfillment_plan.get("capability_envelope") or {})
    profile = normalize_lane_realism_profile(
        fulfillment_plan.get("realism_profile"),
        field_name="llm_fulfillment_plan.realism_profile",
    )
    planned_activity_budget = realism_range_value(
        dict(profile.get("activity_budget") or {}),
        fulfillment_plan.get("run_id"),
        fulfillment_plan.get("tick_id"),
        fulfillment_plan.get("fulfillment_mode"),
        "activity_budget",
    )
    effective_activity_budget = max(
        1,
        min(
            int(capability_envelope.get("max_actions") or 1),
            planned_activity_budget,
        ),
    )
    planned_burst_size = realism_range_value(
        dict(profile.get("burst_size") or {}),
        fulfillment_plan.get("run_id"),
        fulfillment_plan.get("tick_id"),
        fulfillment_plan.get("fulfillment_mode"),
        "burst_size",
    )
    effective_burst_size = max(1, min(effective_activity_budget, planned_burst_size))
    candidate_actions = _normalized_request_mode_actions(
        list(generation_result.get("actions") or [])
    )
    focused_actions = _focused_request_mode_actions(
        fulfillment_plan,
        candidate_actions,
        effective_burst_size=effective_burst_size,
    )
    expanded_actions: list[dict[str, Any]] = []
    focus_count = max(1, len(focused_actions))
    for action_index in range(1, effective_activity_budget + 1):
        template = dict(focused_actions[(action_index - 1) % focus_count])
        template["action_index"] = action_index
        expanded_actions.append(template)
    burst_sizes = partition_activity_budget(effective_activity_budget, effective_burst_size)
    inter_action_gaps_ms = _inter_action_gaps_ms(fulfillment_plan, burst_sizes)
    identity_assignments = _request_mode_identity_assignments(
        fulfillment_plan,
        profile,
        burst_sizes,
        action_count=effective_activity_budget,
    )
    action_request_headers: list[dict[str, str]] = []
    observed_user_agent_families: list[str] = []
    observed_accept_languages: list[str] = []
    action_identity_rows = list(identity_assignments.get("action_identity_rows") or [])
    for identity_row in action_identity_rows:
        request_transport = resolve_request_transport_observation(
            profile,
            country_code=str(identity_row.get("country_code") or "").strip() or None,
        )
        user_agent_family = str(request_transport.get("user_agent_family") or "").strip()
        if user_agent_family and user_agent_family not in observed_user_agent_families:
            observed_user_agent_families.append(user_agent_family)
        accept_language = str(request_transport.get("accept_language") or "").strip()
        if accept_language and accept_language not in observed_accept_languages:
            observed_accept_languages.append(accept_language)
        action_request_headers.append(
            {
                "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                "accept-language": accept_language,
                "user-agent": str(request_transport.get("user_agent") or ""),
            }
        )
    first_identity_country_code = (
        str(dict(action_identity_rows[0] if action_identity_rows else {}).get("country_code") or "").strip()
        or None
    )
    first_request_transport = resolve_request_transport_observation(
        profile,
        country_code=first_identity_country_code,
    )
    request_transport_profile = str(first_request_transport.get("transport_profile") or "")
    recurrence_context = _resolve_recurrence_context(fulfillment_plan, profile)
    capability_state = _capability_state_for_generation(generation_result)
    action_types_attempted = _ordered_unique_strings(
        [action.get("action_type") for action in candidate_actions]
    )
    targeting_strategy = _action_targeting_strategy(candidate_actions)
    return {
        "schema_version": REQUEST_MODE_REALISM_PLAN_SCHEMA_VERSION,
        "profile_id": str(profile.get("profile_id") or ""),
        "capability_state": capability_state,
        "action_types_attempted": action_types_attempted,
        "targeting_strategy": targeting_strategy,
        "planned_activity_budget": planned_activity_budget,
        "effective_activity_budget": effective_activity_budget,
        "planned_burst_size": planned_burst_size,
        "effective_burst_size": effective_burst_size,
        "burst_sizes": burst_sizes,
        "concurrency_group_sizes": burst_sizes,
        "peak_concurrent_activities": max(burst_sizes or [1]),
        "inter_action_gaps_ms": inter_action_gaps_ms,
        "focused_page_paths": [str(action.get("path") or "/") for action in focused_actions],
        "focused_page_set_size": len(focused_actions),
        "session_handles": identity_assignments["session_handles"],
        "identity_rotation_count": identity_assignments["identity_rotation_count"],
        "identity_realism_status": identity_assignments["identity_realism_status"],
        "identity_provenance_mode": identity_assignments["identity_provenance_mode"],
        "identity_envelope_classes": identity_assignments["identity_envelope_classes"],
        "geo_affinity_mode": identity_assignments["geo_affinity_mode"],
        "session_stickiness": identity_assignments["session_stickiness"],
        "observed_country_codes": identity_assignments["observed_country_codes"],
        "action_proxy_urls": identity_assignments["action_proxy_urls"],
        "action_request_headers": action_request_headers,
        "transport_profile": request_transport_profile,
        "transport_realism_class": str(
            first_request_transport.get("transport_realism_class") or ""
        ),
        "transport_emission_basis": str(
            first_request_transport.get("transport_emission_basis") or ""
        ),
        "transport_degraded_reason": str(
            first_request_transport.get("transport_degraded_reason") or ""
        ),
        "observed_user_agent_families": observed_user_agent_families,
        "observed_accept_languages": observed_accept_languages,
        "actions": expanded_actions,
        "recurrence_strategy": str(recurrence_context["strategy"]),
        "reentry_scope": str(recurrence_context["reentry_scope"]),
        "dormancy_truth_mode": str(recurrence_context["dormancy_truth_mode"]),
        "session_index": int(recurrence_context["session_index"]),
        "reentry_count": int(recurrence_context["reentry_count"]),
        "max_reentries_per_run": int(recurrence_context["max_reentries_per_run"]),
        "planned_dormant_gap_seconds": int(recurrence_context["planned_dormant_gap_seconds"]),
        "representative_dormant_gap_seconds": int(
            recurrence_context["representative_dormant_gap_seconds"]
        ),
    }


def _load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise WorkerConfigError(f"JSON payload at {path} must be an object")
    return payload


def extract_llm_fulfillment_plan(beat_response_payload: dict[str, Any]) -> dict[str, Any]:
    plan = beat_response_payload.get("llm_fulfillment_plan")
    if not isinstance(plan, dict):
        raise RuntimeError("beat response must include nested llm_fulfillment_plan object")
    normalized_plan = dict(plan)
    fulfillment_mode = str(normalized_plan.get("fulfillment_mode") or "").strip()
    realism_profile = normalize_lane_realism_profile(
        normalized_plan.get("realism_profile"),
        field_name="llm_fulfillment_plan.realism_profile",
    )
    expected_realism_profile = resolve_lane_realism_profile("bot_red_team", fulfillment_mode)
    if realism_profile != expected_realism_profile:
        raise RuntimeError(
            "llm_fulfillment_plan realism_profile must match the canonical lane realism contract"
        )
    normalized_plan["realism_profile"] = realism_profile
    normalized_plan["request_identity_pool"] = normalize_identity_pool_entries(
        normalized_plan.get("request_identity_pool"),
        field_name="llm_fulfillment_plan.request_identity_pool",
    )
    normalized_plan["request_proxy_url"] = normalize_optional_proxy_url(
        normalized_plan.get("request_proxy_url"),
        field_name="llm_fulfillment_plan.request_proxy_url",
    )
    normalized_plan["browser_proxy_url"] = normalize_optional_proxy_url(
        normalized_plan.get("browser_proxy_url"),
        field_name="llm_fulfillment_plan.browser_proxy_url",
    )
    return normalized_plan


def _normalized_host_root_entrypoint(base_url: str) -> str:
    parsed = urllib.parse.urlparse(str(base_url or "").strip())
    if not parsed.scheme or not parsed.netloc:
        raise RuntimeError("host_root_entrypoint must be an absolute URL")
    return urllib.parse.urlunparse(
        (
            parsed.scheme.lower(),
            parsed.netloc,
            "/",
            "",
            "",
            "",
        )
    )


def _action_receipts(
    generation_result: dict[str, Any],
    report_payload: dict[str, Any] | None,
) -> list[dict[str, Any]]:
    actions = [
        dict(item)
        for item in list(generation_result.get("actions") or [])
        if isinstance(item, dict)
    ]
    traffic = []
    if isinstance(report_payload, dict):
        worker_payload = dict(report_payload.get("worker_payload") or {})
        traffic = [
            dict(item)
            for item in list(worker_payload.get("traffic") or [])
            if isinstance(item, dict)
        ]
    traffic_by_index = {
        int(item.get("action_index") or index + 1): item
        for index, item in enumerate(traffic)
    }

    receipts: list[dict[str, Any]] = []
    for index, action in enumerate(actions):
        action_index = int(action.get("action_index") or index + 1)
        traffic_row = traffic_by_index.get(action_index, {})
        receipt = {
            "action_index": action_index,
            "action_type": str(action.get("action_type") or "").strip(),
            "path": str(action.get("path") or "").strip() or "/",
            "label": str(action.get("label") or "").strip() or None,
            "status": traffic_row.get("status"),
            "error": str(traffic_row.get("error") or "").strip() or None,
        }
        receipts.append(receipt)
    return receipts


def _report_terminal_failure(report_payload: dict[str, Any] | None) -> str | None:
    if not isinstance(report_payload, dict):
        return None
    terminal = report_payload.get("terminal_failure")
    if isinstance(terminal, dict):
        value = str(terminal.get("terminal_failure") or "").strip()
        return value if value and value.lower() != "none" else None
    value = str(terminal or "").strip()
    return value if value and value.lower() != "none" else None


def _report_error(report_payload: dict[str, Any] | None) -> str | None:
    if not isinstance(report_payload, dict):
        return None
    worker_failure_detail = str(report_payload.get("worker_failure_detail") or "").strip()
    if worker_failure_detail:
        return worker_failure_detail
    worker_payload = dict(report_payload.get("worker_payload") or {})
    errors = [
        str(item).strip()
        for item in list(worker_payload.get("errors") or [])
        if str(item).strip()
    ]
    if errors:
        return errors[0]
    terminal = report_payload.get("terminal_failure")
    if isinstance(terminal, dict):
        reason = str(terminal.get("reason") or "").strip()
        if reason:
            return reason
    return None


def _report_failure_class(
    report_payload: dict[str, Any] | None,
    *,
    action_receipts: list[dict[str, Any]],
) -> str | None:
    if not isinstance(report_payload, dict):
        return None
    terminal_failure = _report_terminal_failure(report_payload)
    if terminal_failure in {"deadline_exceeded", "heartbeat_loss"}:
        return "timeout"
    if terminal_failure in {"forced_kill_path", "cancelled"}:
        return "cancelled"

    worker_payload = dict(report_payload.get("worker_payload") or {})
    if any(str(item).strip() for item in list(worker_payload.get("errors") or [])):
        if any(receipt.get("status") == 0 for receipt in action_receipts):
            return "transport"
        return "transport"

    if any(receipt.get("status") not in (None, 200, 302, 303, 403, 404, 429) for receipt in action_receipts):
        return "http"
    if any(receipt.get("status") is not None and int(receipt.get("status") or 0) == 0 for receipt in action_receipts):
        return "transport"
    return None


def build_llm_runtime_result(
    *,
    fulfillment_plan: dict[str, Any],
    generation_result: dict[str, Any],
    report_payload: dict[str, Any] | None,
    tick_completed_at: int,
    worker_id: str,
    error: str | None = None,
    failure_class: str | None = None,
    terminal_failure: str | None = None,
) -> dict[str, Any]:
    action_receipts = _action_receipts(generation_result, report_payload)
    generated_action_count = len(
        [item for item in list(generation_result.get("actions") or []) if isinstance(item, dict)]
    )
    worker_payload = dict(report_payload.get("worker_payload") or {}) if isinstance(report_payload, dict) else {}
    realism_receipt = dict(worker_payload.get("realism_receipt") or {})
    traffic = [
        dict(item)
        for item in list(worker_payload.get("traffic") or [])
        if isinstance(item, dict)
    ]
    executed_action_count = int(worker_payload.get("requests_sent") or len(traffic))
    failed_action_count = sum(
        1
        for receipt in action_receipts
        if receipt.get("error")
        or int(receipt.get("status") or 0) == 0
    )
    last_response_status = None
    if traffic:
        last_response_status = traffic[-1].get("status")

    derived_error = error or _report_error(report_payload)
    derived_terminal_failure = terminal_failure or _report_terminal_failure(report_payload)
    derived_failure_class = failure_class or _report_failure_class(
        report_payload,
        action_receipts=action_receipts,
    )

    passed = bool(report_payload.get("passed")) if isinstance(report_payload, dict) else False
    if derived_error or derived_terminal_failure or derived_failure_class:
        passed = False

    return {
        "schema_version": LLM_RUNTIME_RESULT_SCHEMA_VERSION,
        "run_id": str(fulfillment_plan.get("run_id") or "").strip(),
        "tick_id": str(fulfillment_plan.get("tick_id") or "").strip(),
        "lane": str(fulfillment_plan.get("lane") or "").strip() or "bot_red_team",
        "fulfillment_mode": str(fulfillment_plan.get("fulfillment_mode") or "").strip(),
        "worker_id": str(worker_id).strip(),
        "tick_started_at": int(fulfillment_plan.get("tick_started_at") or 0),
        "tick_completed_at": int(tick_completed_at),
        "backend_kind": str(fulfillment_plan.get("backend_kind") or "").strip(),
        "backend_state": str(fulfillment_plan.get("backend_state") or "").strip(),
        "generation_source": str(generation_result.get("generation_source") or "runtime_failure").strip(),
        "provider": str(generation_result.get("provider") or "").strip(),
        "model_id": str(generation_result.get("model_id") or "").strip(),
        "fallback_reason": str(generation_result.get("fallback_reason") or "").strip() or None,
        "category_targets": [
            str(item).strip()
            for item in list(fulfillment_plan.get("category_targets") or [])
            if str(item).strip()
        ],
        "generated_action_count": generated_action_count,
        "executed_action_count": executed_action_count,
        "failed_action_count": failed_action_count,
        "last_response_status": last_response_status,
        "passed": passed,
        "failure_class": derived_failure_class,
        "error": derived_error,
        "terminal_failure": derived_terminal_failure,
        "realism_receipt": realism_receipt or None,
        "action_receipts": action_receipts,
    }


def run_request_mode_blackbox(
    *,
    base_url: str,
    fulfillment_plan: dict[str, Any],
    generation_result: dict[str, Any],
    realism_execution_plan: dict[str, Any] | None = None,
    runner: Any = subprocess.run,
    report_path: Path | None = None,
) -> dict[str, Any]:
    def request_mode_failure_report(
        *,
        detail: str,
        completed: subprocess.CompletedProcess[str],
    ) -> dict[str, Any]:
        normalized_detail = str(detail or "").strip() or "container_runner_failed"
        return {
            "passed": False,
            "terminal_failure": {
                "terminal_failure": "request_mode_execution_failed",
                "reason": normalized_detail,
            },
            "worker_failure_detail": normalized_detail,
            "worker_payload": {
                "requests_sent": 0,
                "errors": [normalized_detail],
                "traffic": [],
                "realism_receipt": None,
            },
            "_runner_exit_code": int(completed.returncode),
            "_runner_stdout": str(completed.stdout or ""),
            "_runner_stderr": str(completed.stderr or ""),
        }

    capability_envelope = dict(fulfillment_plan.get("capability_envelope") or {})
    if realism_execution_plan is None:
        realism_execution_plan = build_request_mode_realism_execution_plan(
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
        )
    execution_actions = list(realism_execution_plan.get("actions") or [])
    request_budget = max(
        1,
        min(
            int(capability_envelope.get("max_actions") or len(execution_actions) or 1),
            int(realism_execution_plan.get("effective_activity_budget") or len(execution_actions) or 1),
        ),
    )
    time_budget_seconds = max(
        10,
        int(capability_envelope.get("max_time_budget_seconds") or 120),
    )
    if report_path is None:
        report_fd, report_file = tempfile.mkstemp(
            prefix="shuma-llm-runtime-report-",
            suffix=".json",
        )
        os.close(report_fd)
        report_output_path = Path(report_file)
    else:
        report_output_path = report_path
    command = [
        sys.executable,
        str(REPO_ROOT / "scripts" / "tests" / "adversarial_container_runner.py"),
        "--mode",
        "blackbox",
        "--base-url",
        str(base_url).strip(),
        "--frontier-actions",
        json.dumps(execution_actions, separators=(",", ":")),
        "--request-realism-plan-json",
        json.dumps(realism_execution_plan, separators=(",", ":")),
        "--request-budget",
        str(request_budget),
        "--time-budget-seconds",
        str(time_budget_seconds),
        "--report",
        str(report_output_path),
    ]
    completed = runner(
        command,
        capture_output=True,
        text=True,
        check=False,
        cwd=str(REPO_ROOT),
    )
    if not report_output_path.exists():
        return request_mode_failure_report(
            detail=(
                "container_runner_report_missing:"
                f"exit_code={completed.returncode}:stderr={str(completed.stderr or '').strip()}"
            ),
            completed=completed,
        )

    report_text = ""
    try:
        report_text = report_output_path.read_text(encoding="utf-8")
    finally:
        report_output_path.unlink(missing_ok=True)

    normalized_report_text = report_text.strip()
    if not normalized_report_text:
        return request_mode_failure_report(
            detail=(
                "container_runner_report_empty:"
                f"exit_code={completed.returncode}:stderr={str(completed.stderr or '').strip()}"
            ),
            completed=completed,
        )

    try:
        payload = json.loads(normalized_report_text)
    except json.JSONDecodeError as err:
        return request_mode_failure_report(
            detail=(
                "container_runner_report_invalid_json:"
                f"{err}:exit_code={completed.returncode}:stderr={str(completed.stderr or '').strip()}"
            ),
            completed=completed,
        )
    if not isinstance(payload, dict):
        return request_mode_failure_report(
            detail="container_runner_report_invalid",
            completed=completed,
        )
    payload["_runner_exit_code"] = int(completed.returncode)
    payload["_runner_stdout"] = str(completed.stdout or "")
    payload["_runner_stderr"] = str(completed.stderr or "")
    return payload


def run_browser_mode_blackbox(
    *,
    base_url: str,
    fulfillment_plan: dict[str, Any],
    generation_result: dict[str, Any],
    public_hint_paths: list[str] | None = None,
    realism_execution_plan: dict[str, Any] | None = None,
    runner: Any = subprocess.run,
) -> dict[str, Any]:
    if realism_execution_plan is None:
        realism_execution_plan = build_browser_mode_realism_execution_plan(
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
        )

    capability_envelope = dict(fulfillment_plan.get("capability_envelope") or {})
    browser_status = ensure_playwright_chromium()
    runner_env = build_playwright_env(
        base_env=os.environ,
        browser_cache=Path(browser_status.browser_cache),
    )
    sim_tag_secret = str(os.environ.get("SHUMA_SIM_TELEMETRY_SECRET") or "").strip()
    if not sim_tag_secret:
        raise RuntimeError("browser_mode_missing_sim_telemetry_secret")
    normalized_public_hint_paths = [
        str(path).strip()
        for path in list(public_hint_paths or DEFAULT_PUBLIC_HINT_PATHS)
        if str(path).strip()
    ]

    top_level_action_budget = max(
        1,
        int(realism_execution_plan.get("top_level_action_budget") or 1),
    )
    sim_tag_envelope_count = max(24, top_level_action_budget * 8)
    sim_tag_envelopes = build_sim_tag_envelopes(
        secret=sim_tag_secret,
        run_id=str(fulfillment_plan.get("run_id") or "").strip(),
        profile=str(fulfillment_plan.get("fulfillment_mode") or "").strip() or "browser_mode",
        lane=str(fulfillment_plan.get("lane") or "").strip() or "bot_red_team",
        count=sim_tag_envelope_count,
    )
    command = [
        "corepack",
        "pnpm",
        "exec",
        "node",
        str(REPO_ROOT / "scripts" / "tests" / "adversarial_browser_driver.mjs"),
    ]
    driver_input = {
        "action": "agentic_browser_session",
        "base_url": str(base_url).strip(),
        "user_agent": str(realism_execution_plan.get("user_agent") or DEFAULT_AGENTIC_BROWSER_USER_AGENT),
        "locale": str(realism_execution_plan.get("browser_locale") or "en-US"),
        "headers": {
            "accept-language": str(realism_execution_plan.get("accept_language") or "en-US,en;q=0.9")
        },
        "proxy_url": str(realism_execution_plan.get("browser_proxy_url") or "").strip() or None,
        "timeout_ms": min(
            60_000,
            max(
                15_000,
                int(capability_envelope.get("max_time_budget_seconds") or 90) * 1_000,
            ),
        ),
        "settle_ms": 0,
        "storage_mode": "stateful_cookie_jar",
        "session_plan": realism_execution_plan,
        "public_hint_paths": normalized_public_hint_paths,
        "sim_identity": {
            "run_id": str(fulfillment_plan.get("run_id") or "").strip(),
            "profile": str(fulfillment_plan.get("fulfillment_mode") or "").strip() or "browser_mode",
            "lane": str(fulfillment_plan.get("lane") or "").strip() or "bot_red_team",
            "header_names": {
                "run_id": SIM_TAG_HEADER_RUN_ID,
                "profile": SIM_TAG_HEADER_PROFILE,
                "lane": SIM_TAG_HEADER_LANE,
                "timestamp": SIM_TAG_HEADER_TIMESTAMP,
                "nonce": SIM_TAG_HEADER_NONCE,
                "signature": SIM_TAG_HEADER_SIGNATURE,
            },
            "envelopes": sim_tag_envelopes,
        },
    }
    timeout_seconds = max(
        20.0,
        float(int(capability_envelope.get("max_time_budget_seconds") or 90) + 10),
    )
    completed = runner(
        command,
        input=json.dumps(driver_input, separators=(",", ":")),
        text=True,
        capture_output=True,
        timeout=timeout_seconds,
        check=False,
        env=runner_env,
        cwd=str(REPO_ROOT),
    )
    parsed_payload = json.loads(str(completed.stdout or "{}").strip() or "{}")
    if not isinstance(parsed_payload, dict):
        raise RuntimeError("browser_driver_report_invalid")

    top_level_actions = [
        dict(item)
        for item in list(parsed_payload.get("top_level_actions") or [])
        if isinstance(item, dict)
    ]
    detail = str(parsed_payload.get("detail") or "").strip()
    browser_evidence = dict(parsed_payload.get("browser_evidence") or {})
    realism_receipt = dict(parsed_payload.get("realism_receipt") or {})
    worker_payload = {
        "requests_sent": len(top_level_actions),
        "errors": [] if completed.returncode == 0 and bool(parsed_payload.get("ok")) else [detail or "browser_driver_failed"],
        "traffic": top_level_actions,
        "browser_evidence": browser_evidence,
        "realism_receipt": realism_receipt or None,
    }
    return {
        "passed": completed.returncode == 0 and bool(parsed_payload.get("ok")),
        "terminal_failure": {
            "terminal_failure": "" if completed.returncode == 0 and bool(parsed_payload.get("ok")) else "browser_mode_execution_failed",
            "reason": detail,
        },
        "worker_failure_detail": "" if completed.returncode == 0 and bool(parsed_payload.get("ok")) else detail,
        "worker_payload": worker_payload,
        "_runner_exit_code": int(completed.returncode),
        "_runner_stdout": str(completed.stdout or ""),
        "_runner_stderr": str(completed.stderr or ""),
        "_executed_actions": top_level_actions,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run bounded LLM runtime actions for the bot_red_team lane"
    )
    parser.add_argument("--beat-response-file", required=True)
    parser.add_argument("--result-output-file", required=True)
    parser.add_argument(
        "--base-url",
        default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"),
        help="Public host root entrypoint for black-box attacker execution.",
    )
    parser.add_argument(
        "--public-hint-path",
        action="append",
        dest="public_hint_paths",
        help="Optional additional public host-derived hint path.",
    )
    args = parser.parse_args()

    beat_response_payload = _load_json(Path(args.beat_response_file))
    fulfillment_plan = extract_llm_fulfillment_plan(beat_response_payload)
    base_url = _normalized_host_root_entrypoint(str(args.base_url or "").strip())
    public_hint_paths = list(args.public_hint_paths or DEFAULT_PUBLIC_HINT_PATHS)
    tick_completed_at = int(time.time())
    worker_id = f"llm-runtime-worker-{os.getpid()}"

    generation_result = llm_fulfillment.generate_llm_frontier_actions(
        fulfillment_plan=fulfillment_plan,
        host_root_entrypoint=base_url,
        public_hint_paths=public_hint_paths,
    )

    if str(fulfillment_plan.get("fulfillment_mode") or "").strip() == "browser_mode":
        browser_mode_execution_plan = build_browser_mode_realism_execution_plan(
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
        )
        report_payload = run_browser_mode_blackbox(
            base_url=base_url,
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
            public_hint_paths=public_hint_paths,
            realism_execution_plan=browser_mode_execution_plan,
        )
        generation_result = {
            **generation_result,
            "actions": list(report_payload.get("_executed_actions") or []),
        }
        result = build_llm_runtime_result(
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
            report_payload=report_payload,
            tick_completed_at=tick_completed_at,
            worker_id=worker_id,
        )
    else:
        request_mode_execution_plan = build_request_mode_realism_execution_plan(
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
        )
        generation_result = {
            **generation_result,
            "actions": list(request_mode_execution_plan.get("actions") or []),
        }
        report_payload = run_request_mode_blackbox(
            base_url=base_url,
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
            realism_execution_plan=request_mode_execution_plan,
        )
        result = build_llm_runtime_result(
            fulfillment_plan=fulfillment_plan,
            generation_result=generation_result,
            report_payload=report_payload,
            tick_completed_at=tick_completed_at,
            worker_id=worker_id,
        )

    Path(args.result_output_file).write_text(
        json.dumps(result, separators=(",", ":")),
        encoding="utf-8",
    )
    return 0 if bool(result.get("passed")) else 1


if __name__ == "__main__":
    raise SystemExit(main())
