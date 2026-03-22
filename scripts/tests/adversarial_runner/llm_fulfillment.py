"""Bounded LLM fulfillment contract helpers for adversarial runner tooling."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, List, Tuple

from scripts.tests.adversarial_container_runner import load_container_runtime_profile
from scripts.tests.adversarial_runner.contracts import (
    CONTAINER_RUNTIME_PROFILE_PATH,
    FRONTIER_ACTION_CONTRACT_PATH,
)
from scripts.tests.adversarial_runner.shared import dict_or_empty, int_or_zero, list_or_empty
from scripts.tests.frontier_action_contract import load_frontier_action_contract


LLM_FULFILLMENT_PLAN_SCHEMA_VERSION = "adversary-sim-llm-fulfillment-plan.v1"
LLM_FULFILLMENT_CONTRACT_SCHEMA_VERSION = "adversary-sim-llm-fulfillment-contract.v1"
LLM_FULFILLMENT_RUNTIME_SCHEMA_VERSION = "adversary-sim-llm-runtime-profile.v1"
FRONTIER_ACTION_CONTRACT_ID = "frontier_action_contract.v1"
CONTAINER_RUNTIME_PROFILE_ID = "container_runtime_profile.v1"
SUPPORTED_BACKEND_KINDS = ["frontier_reference", "local_candidate"]
SUPPORTED_FULFILLMENT_MODES = ("browser_mode", "request_mode")


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

    return {
        "schema_version": LLM_FULFILLMENT_PLAN_SCHEMA_VERSION,
        "frontier_action_contract_id": FRONTIER_ACTION_CONTRACT_ID,
        "container_runtime_profile_id": CONTAINER_RUNTIME_PROFILE_ID,
        "backend_kinds": backend_kinds,
        "modes": modes,
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
    return {
        "schema_version": str(resolved_contract.get("schema_version") or "").strip(),
        "run_id": str(run_id).strip(),
        "tick_id": f"llm-fit-tick-{int(now)}-{int(generated_tick_count)}",
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
    }
