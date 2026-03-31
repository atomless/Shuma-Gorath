"""Bounded LLM fulfillment contract helpers for adversarial runner tooling."""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Callable, Dict, List, Tuple

from scripts.tests.adversarial_runner.discovery_scoring import FRONTIER_PROVIDER_SPECS
from scripts.tests.adversarial_container_runner import load_container_runtime_profile
from scripts.tests.adversarial_runner.contracts import (
    ATTACKER_FORBIDDEN_PATH_PREFIXES,
    CONTAINER_RUNTIME_PROFILE_PATH,
    FRONTIER_ACTION_CONTRACT_PATH,
    resolve_lane_realism_profile,
)
from scripts.tests.adversarial_runner.shared import dict_or_empty, int_or_zero, list_or_empty
from scripts.tests.frontier_action_contract import (
    FrontierActionValidationError,
    load_frontier_action_contract,
    validate_frontier_actions,
)


LLM_FULFILLMENT_PLAN_SCHEMA_VERSION = "adversary-sim-llm-fulfillment-plan.v1"
LLM_FULFILLMENT_CONTRACT_SCHEMA_VERSION = "adversary-sim-llm-fulfillment-contract.v1"
LLM_FULFILLMENT_RUNTIME_SCHEMA_VERSION = "adversary-sim-llm-runtime-profile.v1"
LLM_ATTACKER_BLACK_BOX_SCHEMA_VERSION = "adversary-sim-llm-attacker-black-box.v1"
LLM_ATTACKER_EPISODE_SCHEMA_VERSION = "adversary-sim-llm-attacker-episode.v1"
FRONTIER_ACTION_CONTRACT_ID = "frontier_action_contract.v1"
CONTAINER_RUNTIME_PROFILE_ID = "container_runtime_profile.v1"
SUPPORTED_BACKEND_KINDS = ["frontier_reference", "local_candidate"]
SUPPORTED_FULFILLMENT_MODES = ("browser_mode", "request_mode")
OPENAI_RESPONSES_URL = "https://api.openai.com/v1/responses"
ANTHROPIC_MESSAGES_URL = "https://api.anthropic.com/v1/messages"
GOOGLE_GENERATE_CONTENT_TEMPLATE = (
    "https://generativelanguage.googleapis.com/v1beta/models/{model_id}:generateContent"
)
XAI_CHAT_COMPLETIONS_URL = "https://api.x.ai/v1/chat/completions"


def llm_fulfillment_mode_for_tick(generated_tick_count: int) -> str:
    return "browser_mode" if int(generated_tick_count) % 2 == 0 else "request_mode"


def _normalize_string_list(values: Any, *, field_name: str) -> List[str]:
    normalized = [str(value).strip() for value in list_or_empty(values) if str(value).strip()]
    if not normalized:
        raise RuntimeError(f"{field_name} must be a non-empty array")
    return normalized


def _normalize_mode_contract(
    mode_name: str,
    mode_contract: Dict[str, Any],
    runtime_contract: Dict[str, Any],
) -> Dict[str, Any]:
    reference_backend_kind = str(mode_contract.get("reference_backend_kind") or "").strip()
    if reference_backend_kind not in SUPPORTED_BACKEND_KINDS:
        raise RuntimeError(
            f"llm_fulfillment.modes.{mode_name}.reference_backend_kind must be one of "
            f"{', '.join(SUPPORTED_BACKEND_KINDS)}"
        )
    allowed_tools = _normalize_string_list(
        mode_contract.get("allowed_tools"),
        field_name=f"llm_fulfillment.modes.{mode_name}.allowed_tools",
    )
    category_targets = _normalize_string_list(
        mode_contract.get("category_targets"),
        field_name=f"llm_fulfillment.modes.{mode_name}.category_targets",
    )

    browser_automation_allowed = runtime_contract.get("browser_automation_allowed")
    if not isinstance(browser_automation_allowed, bool):
        raise RuntimeError(
            "llm_fulfillment_runtime."
            f"{mode_name}.browser_automation_allowed must be boolean"
        )
    direct_request_emission_allowed = runtime_contract.get(
        "direct_request_emission_allowed"
    )
    if not isinstance(direct_request_emission_allowed, bool):
        raise RuntimeError(
            "llm_fulfillment_runtime."
            f"{mode_name}.direct_request_emission_allowed must be boolean"
        )

    max_actions = int_or_zero(runtime_contract.get("max_actions"))
    if max_actions < 1:
        raise RuntimeError(f"llm_fulfillment_runtime.{mode_name}.max_actions must be >= 1")
    max_time_budget_seconds = int_or_zero(runtime_contract.get("max_time_budget_seconds"))
    if max_time_budget_seconds < 1:
        raise RuntimeError(
            "llm_fulfillment_runtime."
            f"{mode_name}.max_time_budget_seconds must be >= 1"
        )

    return {
        "reference_backend_kind": reference_backend_kind,
        "allowed_tools": allowed_tools,
        "category_targets": category_targets,
        "browser_automation_allowed": browser_automation_allowed,
        "direct_request_emission_allowed": direct_request_emission_allowed,
        "max_actions": max_actions,
        "max_time_budget_seconds": max_time_budget_seconds,
    }


def _require_boolean(value: Any, *, field_name: str) -> bool:
    if not isinstance(value, bool):
        raise RuntimeError(f"{field_name} must be boolean")
    return value


def _normalize_black_box_boundary(boundary: Dict[str, Any]) -> Dict[str, Any]:
    schema_version = str(boundary.get("schema_version") or "").strip()
    if schema_version != LLM_ATTACKER_BLACK_BOX_SCHEMA_VERSION:
        raise RuntimeError(
            "llm_attacker_black_box.schema_version must be "
            f"{LLM_ATTACKER_BLACK_BOX_SCHEMA_VERSION}"
        )

    position = str(boundary.get("position") or "").strip()
    if position != "outside_attacker":
        raise RuntimeError("llm_attacker_black_box.position must be outside_attacker")

    public_host_hint_sources = _normalize_string_list(
        boundary.get("public_host_hint_sources"),
        field_name="llm_attacker_black_box.public_host_hint_sources",
    )
    allowed_observation_families = _normalize_string_list(
        boundary.get("allowed_observation_families"),
        field_name="llm_attacker_black_box.allowed_observation_families",
    )
    forbidden_knowledge_sources = _normalize_string_list(
        boundary.get("forbidden_knowledge_sources"),
        field_name="llm_attacker_black_box.forbidden_knowledge_sources",
    )
    receipt_requirements = dict_or_empty(boundary.get("receipt_requirements"))
    if not receipt_requirements:
        raise RuntimeError("llm_attacker_black_box.receipt_requirements must be an object")

    attack_trace_required = _require_boolean(
        receipt_requirements.get("attack_trace_required"),
        field_name="llm_attacker_black_box.receipt_requirements.attack_trace_required",
    )
    observation_lineage_required = _require_boolean(
        receipt_requirements.get("observation_lineage_required"),
        field_name=(
            "llm_attacker_black_box.receipt_requirements.observation_lineage_required"
        ),
    )
    category_objective_lineage_required = _require_boolean(
        receipt_requirements.get("category_objective_lineage_required"),
        field_name=(
            "llm_attacker_black_box.receipt_requirements."
            "category_objective_lineage_required"
        ),
    )

    return {
        "position": position,
        "host_root_only_entrypoint": _require_boolean(
            boundary.get("host_root_only_entrypoint"),
            field_name="llm_attacker_black_box.host_root_only_entrypoint",
        ),
        "category_objective_required": _require_boolean(
            boundary.get("category_objective_required"),
            field_name="llm_attacker_black_box.category_objective_required",
        ),
        "malicious_category_priming_required": _require_boolean(
            boundary.get("malicious_category_priming_required"),
            field_name="llm_attacker_black_box.malicious_category_priming_required",
        ),
        "public_knowledge_only": _require_boolean(
            boundary.get("public_knowledge_only"),
            field_name="llm_attacker_black_box.public_knowledge_only",
        ),
        "shuma_blind": _require_boolean(
            boundary.get("shuma_blind"),
            field_name="llm_attacker_black_box.shuma_blind",
        ),
        "web_search_allowed": _require_boolean(
            boundary.get("web_search_allowed"),
            field_name="llm_attacker_black_box.web_search_allowed",
        ),
        "repo_visibility_allowed": _require_boolean(
            boundary.get("repo_visibility_allowed"),
            field_name="llm_attacker_black_box.repo_visibility_allowed",
        ),
        "judge_visibility_allowed": _require_boolean(
            boundary.get("judge_visibility_allowed"),
            field_name="llm_attacker_black_box.judge_visibility_allowed",
        ),
        "public_host_hint_sources": public_host_hint_sources,
        "allowed_observation_families": allowed_observation_families,
        "forbidden_knowledge_sources": forbidden_knowledge_sources,
        "receipt_requirements": {
            "attack_trace_required": attack_trace_required,
            "observation_lineage_required": observation_lineage_required,
            "category_objective_lineage_required": category_objective_lineage_required,
        },
    }


def _normalize_episode_harness(harness: Dict[str, Any]) -> Dict[str, Any]:
    schema_version = str(harness.get("schema_version") or "").strip()
    if schema_version != LLM_ATTACKER_EPISODE_SCHEMA_VERSION:
        raise RuntimeError(
            "llm_attacker_episode_harness.schema_version must be "
            f"{LLM_ATTACKER_EPISODE_SCHEMA_VERSION}"
        )

    initial_context_fields = _normalize_string_list(
        harness.get("initial_context_fields"),
        field_name="llm_attacker_episode_harness.initial_context_fields",
    )
    terminal_conditions = _normalize_string_list(
        harness.get("terminal_conditions"),
        field_name="llm_attacker_episode_harness.terminal_conditions",
    )
    failure_states = _normalize_string_list(
        harness.get("failure_states"),
        field_name="llm_attacker_episode_harness.failure_states",
    )
    allowed_memory_sources = _normalize_string_list(
        harness.get("allowed_memory_sources"),
        field_name="llm_attacker_episode_harness.allowed_memory_sources",
    )
    forbidden_memory_sources = _normalize_string_list(
        harness.get("forbidden_memory_sources"),
        field_name="llm_attacker_episode_harness.forbidden_memory_sources",
    )

    environment_reset_policy = str(harness.get("environment_reset_policy") or "").strip()
    if environment_reset_policy != "fresh_episode_reset":
        raise RuntimeError(
            "llm_attacker_episode_harness.environment_reset_policy must be "
            "fresh_episode_reset"
        )

    max_retained_episode_summaries = int_or_zero(
        harness.get("max_retained_episode_summaries")
    )
    if max_retained_episode_summaries < 1:
        raise RuntimeError(
            "llm_attacker_episode_harness.max_retained_episode_summaries must be >= 1"
        )
    max_curriculum_items = int_or_zero(harness.get("max_curriculum_items"))
    if max_curriculum_items < 1:
        raise RuntimeError(
            "llm_attacker_episode_harness.max_curriculum_items must be >= 1"
        )

    return {
        "initial_context_fields": initial_context_fields,
        "environment_reset_required": _require_boolean(
            harness.get("environment_reset_required"),
            field_name="llm_attacker_episode_harness.environment_reset_required",
        ),
        "environment_reset_policy": environment_reset_policy,
        "bounded_action_horizon_required": _require_boolean(
            harness.get("bounded_action_horizon_required"),
            field_name="llm_attacker_episode_harness.bounded_action_horizon_required",
        ),
        "terminal_conditions": terminal_conditions,
        "failure_states": failure_states,
        "allowed_memory_sources": allowed_memory_sources,
        "forbidden_memory_sources": forbidden_memory_sources,
        "max_retained_episode_summaries": max_retained_episode_summaries,
        "max_curriculum_items": max_curriculum_items,
        "player_visible_protected_evidence_allowed": _require_boolean(
            harness.get("player_visible_protected_evidence_allowed"),
            field_name=(
                "llm_attacker_episode_harness.player_visible_protected_evidence_allowed"
            ),
        ),
        "held_out_evaluation_visible": _require_boolean(
            harness.get("held_out_evaluation_visible"),
            field_name="llm_attacker_episode_harness.held_out_evaluation_visible",
        ),
    }


def load_llm_fulfillment_contract(
    frontier_action_contract_path: Path = FRONTIER_ACTION_CONTRACT_PATH,
    container_runtime_profile_path: Path = CONTAINER_RUNTIME_PROFILE_PATH,
) -> Dict[str, Any]:
    frontier_contract = load_frontier_action_contract(frontier_action_contract_path)
    runtime_profile = load_container_runtime_profile(container_runtime_profile_path)

    llm_fulfillment = dict_or_empty(frontier_contract.get("llm_fulfillment"))
    schema_version = str(llm_fulfillment.get("schema_version") or "").strip()
    if schema_version != LLM_FULFILLMENT_CONTRACT_SCHEMA_VERSION:
        raise RuntimeError(
            "frontier action contract llm_fulfillment.schema_version must be "
            f"{LLM_FULFILLMENT_CONTRACT_SCHEMA_VERSION}"
        )

    backend_kinds = _normalize_string_list(
        llm_fulfillment.get("backend_kinds"),
        field_name="llm_fulfillment.backend_kinds",
    )
    if backend_kinds != SUPPORTED_BACKEND_KINDS:
        raise RuntimeError(
            "llm_fulfillment.backend_kinds must be exactly "
            f"{', '.join(SUPPORTED_BACKEND_KINDS)}"
        )

    llm_modes = dict_or_empty(llm_fulfillment.get("modes"))
    llm_runtime = dict_or_empty(runtime_profile.get("llm_fulfillment_runtime"))
    runtime_schema = str(llm_runtime.get("schema_version") or "").strip()
    if runtime_schema != LLM_FULFILLMENT_RUNTIME_SCHEMA_VERSION:
        raise RuntimeError(
            "container runtime profile llm_fulfillment_runtime.schema_version must be "
            f"{LLM_FULFILLMENT_RUNTIME_SCHEMA_VERSION}"
        )

    modes: Dict[str, Any] = {}
    for mode_name in SUPPORTED_FULFILLMENT_MODES:
        mode_contract = dict_or_empty(llm_modes.get(mode_name))
        if not mode_contract:
            raise RuntimeError(f"llm_fulfillment.modes.{mode_name} must be an object")
        runtime_contract = dict_or_empty(llm_runtime.get(mode_name))
        if not runtime_contract:
            raise RuntimeError(
                f"llm_fulfillment_runtime.{mode_name} must be an object"
            )
        modes[mode_name] = _normalize_mode_contract(
            mode_name,
            mode_contract,
            runtime_contract,
        )
        modes[mode_name]["realism_profile"] = resolve_lane_realism_profile(
            "bot_red_team",
            mode_name,
        )

    black_box_boundary = _normalize_black_box_boundary(
        dict_or_empty(frontier_contract.get("llm_attacker_black_box"))
    )
    episode_harness = _normalize_episode_harness(
        dict_or_empty(frontier_contract.get("llm_attacker_episode_harness"))
    )

    return {
        "schema_version": LLM_FULFILLMENT_PLAN_SCHEMA_VERSION,
        "frontier_action_contract_id": FRONTIER_ACTION_CONTRACT_ID,
        "container_runtime_profile_id": CONTAINER_RUNTIME_PROFILE_ID,
        "backend_kinds": backend_kinds,
        "modes": modes,
        "black_box_boundary": black_box_boundary,
        "episode_harness": episode_harness,
    }


def _frontier_backend_status(frontier_metadata: Dict[str, Any]) -> Tuple[str, str]:
    provider_count = int_or_zero(
        frontier_metadata.get("provider_count")
        or frontier_metadata.get("provider_count_configured")
    )
    if provider_count < 1:
        return ("unavailable", "frontier_reference:unconfigured")
    backend_mode = (
        str(frontier_metadata.get("mode") or frontier_metadata.get("frontier_mode") or "")
        .strip()
        or "single_provider_self_play"
    )
    if bool(frontier_metadata.get("reduced_diversity_warning")):
        return ("degraded", f"frontier_reference:{backend_mode}")
    return ("configured", f"frontier_reference:{backend_mode}")


def _build_recurrence_context(realism_profile: Dict[str, Any]) -> Dict[str, Any]:
    recurrence = dict_or_empty(realism_profile.get("recurrence_envelope"))
    dormant_gap = dict_or_empty(recurrence.get("dormant_gap_seconds"))
    min_gap = int_or_zero(dormant_gap.get("min"))
    max_reentries = int_or_zero(recurrence.get("max_reentries_per_run"))
    return {
        "strategy": str(recurrence.get("strategy") or "").strip(),
        "session_index": 1,
        "reentry_count": 0,
        "max_reentries_per_run": max_reentries,
        "planned_dormant_gap_seconds": max(1, min_gap),
    }


def build_llm_fulfillment_plan(
    *,
    run_id: str,
    generated_tick_count: int,
    frontier_metadata: Dict[str, Any],
    now: int,
    lane: str = "bot_red_team",
    contract: Dict[str, Any] | None = None,
) -> Dict[str, Any]:
    resolved_contract = contract or load_llm_fulfillment_contract()
    fulfillment_mode = llm_fulfillment_mode_for_tick(generated_tick_count)
    mode_contract = dict_or_empty(dict_or_empty(resolved_contract.get("modes")).get(fulfillment_mode))
    if not mode_contract:
        raise RuntimeError(f"unsupported llm fulfillment mode: {fulfillment_mode}")

    backend_state, backend_id = _frontier_backend_status(frontier_metadata)
    realism_profile = dict(mode_contract.get("realism_profile") or {})
    return {
        "schema_version": str(resolved_contract.get("schema_version") or "").strip(),
        "run_id": str(run_id).strip(),
        "tick_id": f"llm-fit-tick-{int(now)}-{int(generated_tick_count)}",
        "tick_started_at": int(now),
        "lane": lane,
        "fulfillment_mode": fulfillment_mode,
        "backend_kind": str(mode_contract.get("reference_backend_kind") or "").strip(),
        "backend_state": backend_state,
        "backend_id": backend_id,
        "supported_backend_kinds": list(resolved_contract.get("backend_kinds") or []),
        "category_targets": list(mode_contract.get("category_targets") or []),
        "frontier_action_contract_id": str(
            resolved_contract.get("frontier_action_contract_id") or ""
        ).strip(),
        "container_runtime_profile_id": str(
            resolved_contract.get("container_runtime_profile_id") or ""
        ).strip(),
        "black_box_boundary": dict(resolved_contract.get("black_box_boundary") or {}),
        "episode_harness": dict(resolved_contract.get("episode_harness") or {}),
        "capability_envelope": {
            "allowed_tools": list(mode_contract.get("allowed_tools") or []),
            "browser_automation_allowed": bool(
                mode_contract.get("browser_automation_allowed")
            ),
            "direct_request_emission_allowed": bool(
                mode_contract.get("direct_request_emission_allowed")
            ),
            "max_actions": int_or_zero(mode_contract.get("max_actions")),
            "max_time_budget_seconds": int_or_zero(
                mode_contract.get("max_time_budget_seconds")
            ),
        },
        "realism_profile": realism_profile,
        "recurrence_context": _build_recurrence_context(realism_profile),
    }


def _trimmed_env_value(env_reader: Callable[[str], str], key: str) -> str:
    return str(env_reader(key) or "").strip()


def _normalize_host_root_entrypoint(host_root_entrypoint: str) -> str:
    parsed = urllib.parse.urlparse(str(host_root_entrypoint or "").strip())
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


def _forbidden_hint_prefixes(contract: Dict[str, Any]) -> Tuple[str, ...]:
    forbidden_data_access = dict_or_empty(contract.get("forbidden_data_access"))
    contract_prefixes = tuple(
        str(item).strip().lower()
        for item in list_or_empty(forbidden_data_access.get("forbidden_path_prefixes"))
        if str(item).strip()
    )
    lane_prefixes = tuple(str(item).strip().lower() for item in ATTACKER_FORBIDDEN_PATH_PREFIXES)
    return tuple(sorted(set(contract_prefixes).union(lane_prefixes)))


def _sanitize_public_hint_paths(
    public_hint_paths: List[str] | None,
    *,
    contract: Dict[str, Any],
) -> List[str]:
    forbidden_prefixes = _forbidden_hint_prefixes(contract)
    sanitized: List[str] = []
    seen = set()
    for raw_value in list_or_empty(public_hint_paths):
        candidate = str(raw_value or "").strip()
        if not candidate or not candidate.startswith("/") or candidate.startswith("//"):
            continue
        parsed = urllib.parse.urlparse(candidate)
        if parsed.scheme or parsed.netloc or parsed.fragment or parsed.query:
            continue
        if ".." in candidate:
            continue
        lowered = candidate.lower()
        if any(lowered.startswith(prefix) for prefix in forbidden_prefixes):
            continue
        if candidate in seen:
            continue
        sanitized.append(candidate)
        seen.add(candidate)
    return sanitized[:8]


def _build_generation_context(
    *,
    fulfillment_plan: Dict[str, Any],
    host_root_entrypoint: str,
    public_hint_paths: List[str] | None,
    contract: Dict[str, Any],
) -> Dict[str, Any]:
    category_targets = [str(value).strip() for value in list_or_empty(fulfillment_plan.get("category_targets")) if str(value).strip()]
    normalized_root = _normalize_host_root_entrypoint(host_root_entrypoint)
    return {
        "host_root_entrypoint": normalized_root,
        "category_objective": category_targets[0] if category_targets else "",
        "category_targets": category_targets,
        "black_box_boundary": dict_or_empty(fulfillment_plan.get("black_box_boundary")),
        "capability_envelope": dict_or_empty(fulfillment_plan.get("capability_envelope")),
        "episode_harness": {
            "initial_context_fields": list_or_empty(
                dict_or_empty(fulfillment_plan.get("episode_harness")).get(
                    "initial_context_fields"
                )
            ),
            "environment_reset_policy": str(
                dict_or_empty(fulfillment_plan.get("episode_harness")).get(
                    "environment_reset_policy"
                )
                or ""
            ).strip(),
            "max_retained_episode_summaries": int_or_zero(
                dict_or_empty(fulfillment_plan.get("episode_harness")).get(
                    "max_retained_episode_summaries"
                )
            ),
            "max_curriculum_items": int_or_zero(
                dict_or_empty(fulfillment_plan.get("episode_harness")).get(
                    "max_curriculum_items"
                )
            ),
        },
        "public_hint_paths": _sanitize_public_hint_paths(
            public_hint_paths,
            contract=contract,
        ),
    }


def _select_configured_provider(
    env_reader: Callable[[str], str],
) -> Tuple[Dict[str, str] | None, str, str]:
    for provider_spec in FRONTIER_PROVIDER_SPECS:
        api_key = _trimmed_env_value(env_reader, provider_spec["api_key_env"])
        if not api_key:
            continue
        model_id = _trimmed_env_value(env_reader, provider_spec["model_env"]) or str(
            provider_spec["default_model"]
        )
        return (dict(provider_spec), model_id, api_key)
    return (None, "", "")


def _fallback_label(path: str) -> str:
    if path == "/":
        return "root"
    token = path.strip("/").split("/")[-1].strip()
    return token[:80] if token else "hint"


def _dedupe_fallback_actions(actions: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    deduped: List[Dict[str, Any]] = []
    seen = set()
    for action in actions:
        action_type = str(action.get("action_type") or "").strip()
        path = str(action.get("path") or "").strip() or "/"
        query = dict_or_empty(action.get("query"))
        query_key = tuple(
            (str(key).strip(), str(query[key]).strip())
            for key in sorted(query.keys(), key=lambda item: str(item))
            if str(key).strip()
        )
        key = (action_type, path, query_key)
        if key in seen:
            continue
        deduped.append(action)
        seen.add(key)
    return deduped


def _archive_walk_request_defaults(contract: Dict[str, Any]) -> List[Dict[str, Any]]:
    defaults = [
        dict(item)
        for item in list_or_empty(dict_or_empty(contract).get("default_actions"))
        if isinstance(item, dict)
    ]
    defaults.extend(
        [
            {
                "action_type": "http_get",
                "path": "/research/",
                "query": {"page": "2"},
                "label": "research_page_2",
            },
            {
                "action_type": "http_get",
                "path": "/plans/",
                "query": {"page": "2"},
                "label": "plans_page_2",
            },
        ]
    )
    return _dedupe_fallback_actions(defaults)


def _archive_walk_browser_defaults() -> List[Dict[str, Any]]:
    return [
        {"action_type": "browser_navigate", "path": "/", "label": "root"},
        {"action_type": "browser_navigate", "path": "/research/", "label": "research"},
        {"action_type": "browser_navigate", "path": "/plans/", "label": "plans"},
        {"action_type": "browser_navigate", "path": "/work/", "label": "work"},
        {"action_type": "browser_navigate", "path": "/page/2/", "label": "archive_page_2"},
    ]


def _fallback_actions_for_plan(
    fulfillment_plan: Dict[str, Any],
    *,
    generation_context: Dict[str, Any],
    contract: Dict[str, Any],
) -> List[Dict[str, Any]]:
    if str(fulfillment_plan.get("fulfillment_mode") or "").strip() == "browser_mode":
        actions = _archive_walk_browser_defaults()
        for hint_path in list_or_empty(generation_context.get("public_hint_paths")):
            if hint_path == "/":
                continue
            actions.append(
                {
                    "action_type": "browser_navigate",
                    "path": hint_path,
                    "label": _fallback_label(hint_path),
                }
            )
        max_actions = max(
            1,
            int_or_zero(
                dict_or_empty(fulfillment_plan.get("capability_envelope")).get("max_actions")
            ),
        )
        return _dedupe_fallback_actions(actions)[:max_actions]

    actions = _archive_walk_request_defaults(contract)
    for hint_path in list_or_empty(generation_context.get("public_hint_paths")):
        if hint_path == "/":
            continue
        action_type = "http_head" if hint_path == "/robots.txt" else "http_get"
        actions.append(
            {
                "action_type": action_type,
                "path": hint_path,
                "label": _fallback_label(hint_path),
            }
        )
    max_actions = max(
        1,
        int_or_zero(dict_or_empty(fulfillment_plan.get("capability_envelope")).get("max_actions")),
    )
    return _dedupe_fallback_actions(actions)[:max_actions]


def _prompt_text(generation_context: Dict[str, Any]) -> str:
    return (
        "You are an outside malicious non-human attacker operating against a public host. "
        "You must stay within the provided black-box boundary, treat the host as Shuma-blind, "
        "and return JSON only in the shape "
        '{"actions":[{"action_type":"...","path":"/...","label":"..."}],"rationale":"..."}.\n\n'
        "Use only the host root and public host-derived hints below.\n"
        f"{json.dumps(generation_context, sort_keys=True)}"
    )


def _read_response_json(response: Any) -> Dict[str, Any]:
    body = response.read()
    if isinstance(body, bytes):
        text = body.decode("utf-8")
    else:
        text = str(body)
    payload = json.loads(text)
    if not isinstance(payload, dict):
        raise RuntimeError("provider response payload must be a JSON object")
    return payload


def _openai_response_text(payload: Dict[str, Any]) -> str:
    direct_text = str(payload.get("output_text") or "").strip()
    if direct_text:
        return direct_text
    texts: List[str] = []
    for output_row in list_or_empty(payload.get("output")):
        for content_row in list_or_empty(dict_or_empty(output_row).get("content")):
            text = str(dict_or_empty(content_row).get("text") or "").strip()
            if text:
                texts.append(text)
    return "\n".join(texts).strip()


def _anthropic_response_text(payload: Dict[str, Any]) -> str:
    texts = []
    for content_row in list_or_empty(payload.get("content")):
        text = str(dict_or_empty(content_row).get("text") or "").strip()
        if text:
            texts.append(text)
    return "\n".join(texts).strip()


def _google_response_text(payload: Dict[str, Any]) -> str:
    candidates = list_or_empty(payload.get("candidates"))
    if not candidates:
        return ""
    parts = list_or_empty(dict_or_empty(dict_or_empty(candidates[0]).get("content")).get("parts"))
    texts = []
    for part in parts:
        text = str(dict_or_empty(part).get("text") or "").strip()
        if text:
            texts.append(text)
    return "\n".join(texts).strip()


def _xai_response_text(payload: Dict[str, Any]) -> str:
    choices = list_or_empty(payload.get("choices"))
    if not choices:
        return ""
    return str(
        dict_or_empty(dict_or_empty(choices[0]).get("message")).get("content") or ""
    ).strip()


def _provider_request_and_text(
    provider_spec: Dict[str, str],
    *,
    model_id: str,
    api_key: str,
    generation_context: Dict[str, Any],
) -> str:
    provider = str(provider_spec.get("provider") or "").strip()
    prompt = _prompt_text(generation_context)
    headers = {"Content-Type": "application/json"}
    if provider == "openai":
        headers["Authorization"] = f"Bearer {api_key}"
        payload = {
            "model": model_id,
            "instructions": (
                "Return only JSON actions for a black-box attacker. Never use Shuma-specific "
                "knowledge or internal routes."
            ),
            "input": prompt,
        }
        request = urllib.request.Request(
            OPENAI_RESPONSES_URL,
            data=json.dumps(payload).encode("utf-8"),
            headers=headers,
            method="POST",
        )
        with urllib.request.urlopen(request, timeout=20.0) as response:
            return _openai_response_text(_read_response_json(response))
    if provider == "anthropic":
        headers["x-api-key"] = api_key
        headers["anthropic-version"] = "2023-06-01"
        payload = {
            "model": model_id,
            "max_tokens": 800,
            "system": (
                "Return only JSON actions for a black-box attacker. Never use Shuma-specific "
                "knowledge or internal routes."
            ),
            "messages": [{"role": "user", "content": prompt}],
        }
        request = urllib.request.Request(
            ANTHROPIC_MESSAGES_URL,
            data=json.dumps(payload).encode("utf-8"),
            headers=headers,
            method="POST",
        )
        with urllib.request.urlopen(request, timeout=20.0) as response:
            return _anthropic_response_text(_read_response_json(response))
    if provider == "google":
        endpoint = GOOGLE_GENERATE_CONTENT_TEMPLATE.format(model_id=model_id)
        query = urllib.parse.urlencode({"key": api_key})
        payload = {
            "contents": [{"role": "user", "parts": [{"text": prompt}]}],
            "generationConfig": {
                "temperature": 0.1,
                "responseMimeType": "application/json",
            },
        }
        request = urllib.request.Request(
            f"{endpoint}?{query}",
            data=json.dumps(payload).encode("utf-8"),
            headers=headers,
            method="POST",
        )
        with urllib.request.urlopen(request, timeout=20.0) as response:
            return _google_response_text(_read_response_json(response))
    if provider == "xai":
        headers["Authorization"] = f"Bearer {api_key}"
        payload = {
            "model": model_id,
            "temperature": 0.1,
            "messages": [
                {
                    "role": "system",
                    "content": (
                        "Return only JSON actions for a black-box attacker. Never use "
                        "Shuma-specific knowledge or internal routes."
                    ),
                },
                {"role": "user", "content": prompt},
            ],
        }
        request = urllib.request.Request(
            XAI_CHAT_COMPLETIONS_URL,
            data=json.dumps(payload).encode("utf-8"),
            headers=headers,
            method="POST",
        )
        with urllib.request.urlopen(request, timeout=20.0) as response:
            return _xai_response_text(_read_response_json(response))
    raise RuntimeError(f"unsupported frontier provider for LLM generation: {provider}")


def _default_provider_executor(
    provider_spec: Dict[str, str],
    model_id: str,
    api_key: str,
    generation_context: Dict[str, Any],
) -> Dict[str, Any]:
    try:
        raw_text = _provider_request_and_text(
            provider_spec,
            model_id=model_id,
            api_key=api_key,
            generation_context=generation_context,
        )
    except urllib.error.HTTPError as exc:
        raise RuntimeError(f"provider_http_error:{exc.code}") from exc
    except urllib.error.URLError as exc:
        raise RuntimeError(f"provider_network_error:{exc.reason}") from exc
    except TimeoutError as exc:
        raise RuntimeError("provider_timeout") from exc

    parsed = json.loads(str(raw_text or "").strip() or "{}")
    if isinstance(parsed, list):
        parsed = {"actions": parsed}
    if not isinstance(parsed, dict):
        raise RuntimeError("provider response must decode to an action object")
    return parsed


def _validate_generated_actions(
    actions: List[Dict[str, Any]],
    *,
    fulfillment_plan: Dict[str, Any],
    contract: Dict[str, Any],
    host_root_entrypoint: str,
) -> List[Dict[str, Any]]:
    capability_envelope = dict_or_empty(fulfillment_plan.get("capability_envelope"))
    allowed_tools = [str(item).strip() for item in list_or_empty(capability_envelope.get("allowed_tools")) if str(item).strip()]
    fulfillment_mode = str(fulfillment_plan.get("fulfillment_mode") or "").strip()
    action_type_overrides = allowed_tools if fulfillment_mode == "browser_mode" else None
    return validate_frontier_actions(
        actions,
        contract=contract,
        base_url=host_root_entrypoint,
        allowed_origins=[host_root_entrypoint],
        request_budget=max(1, int_or_zero(capability_envelope.get("max_actions"))),
        allowed_tools_override=allowed_tools,
        allowed_action_types_override=action_type_overrides,
    )


def generate_llm_frontier_actions(
    *,
    fulfillment_plan: Dict[str, Any],
    host_root_entrypoint: str,
    public_hint_paths: List[str] | None = None,
    env_reader: Callable[[str], str] = lambda key: os.environ.get(key, ""),
    provider_executor: Callable[[Dict[str, str], str, str, Dict[str, Any]], Dict[str, Any]]
    | None = None,
    contract: Dict[str, Any] | None = None,
) -> Dict[str, Any]:
    resolved_contract = contract or load_frontier_action_contract()
    generation_context = _build_generation_context(
        fulfillment_plan=fulfillment_plan,
        host_root_entrypoint=host_root_entrypoint,
        public_hint_paths=public_hint_paths,
        contract=resolved_contract,
    )
    provider_spec, model_id, api_key = _select_configured_provider(env_reader)
    if provider_spec is None:
        fallback_actions = _fallback_actions_for_plan(
            fulfillment_plan,
            generation_context=generation_context,
            contract=resolved_contract,
        )
        return {
            "generation_source": "fallback_no_provider",
            "provider": "",
            "model_id": "",
            "fallback_reason": "no_configured_frontier_provider",
            "actions": _validate_generated_actions(
                fallback_actions,
                fulfillment_plan=fulfillment_plan,
                contract=resolved_contract,
                host_root_entrypoint=generation_context["host_root_entrypoint"],
            ),
            "generation_context": generation_context,
        }

    executor = provider_executor or _default_provider_executor
    provider_name = str(provider_spec.get("provider") or "").strip()
    try:
        provider_payload = executor(provider_spec, model_id, api_key, generation_context)
        provider_actions = [dict(item) for item in list_or_empty(dict_or_empty(provider_payload).get("actions")) if isinstance(item, dict)]
        validated_actions = _validate_generated_actions(
            provider_actions,
            fulfillment_plan=fulfillment_plan,
            contract=resolved_contract,
            host_root_entrypoint=generation_context["host_root_entrypoint"],
        )
        return {
            "generation_source": "provider_response",
            "provider": provider_name,
            "model_id": model_id,
            "actions": validated_actions,
            "rationale": str(dict_or_empty(provider_payload).get("rationale") or "").strip(),
            "generation_context": generation_context,
        }
    except (FrontierActionValidationError, RuntimeError, ValueError, json.JSONDecodeError):
        fallback_actions = _fallback_actions_for_plan(
            fulfillment_plan,
            generation_context=generation_context,
            contract=resolved_contract,
        )
        return {
            "generation_source": "fallback_validation_error",
            "provider": provider_name,
            "model_id": model_id,
            "fallback_reason": "provider_output_failed_validation",
            "actions": _validate_generated_actions(
                fallback_actions,
                fulfillment_plan=fulfillment_plan,
                contract=resolved_contract,
                host_root_entrypoint=generation_context["host_root_entrypoint"],
            ),
            "generation_context": generation_context,
        }
