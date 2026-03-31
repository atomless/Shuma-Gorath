#!/usr/bin/env python3
"""Frontier action contract and reject-by-default DSL validation helpers."""

from __future__ import annotations

import copy
import json
import urllib.parse
from pathlib import Path
from typing import Any, Dict, List, Sequence


DEFAULT_FRONTIER_ACTION_CONTRACT_PATH = Path(
    "scripts/tests/adversarial/frontier_action_contract.v1.json"
)


class FrontierActionContractError(RuntimeError):
    """Raised when the contract artifact is invalid."""


class FrontierActionValidationError(RuntimeError):
    """Raised when proposed frontier actions violate the contract."""


def _is_positive_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value > 0


def _normalized_origin(url: str) -> str:
    parsed = urllib.parse.urlparse(str(url or "").strip())
    if not parsed.scheme or not parsed.netloc:
        raise FrontierActionValidationError(f"invalid URL origin input: {url}")
    return f"{parsed.scheme.lower()}://{parsed.netloc.lower()}"


def _forbidden_path(path: str, forbidden_prefixes: Sequence[str]) -> bool:
    lowered = str(path or "").strip().lower()
    return any(lowered.startswith(prefix) for prefix in forbidden_prefixes)


def _is_forbidden_hostname(hostname: str, forbidden_hostnames: Sequence[str]) -> bool:
    lowered = str(hostname or "").strip().lower()
    if not lowered:
        return True
    for forbidden in forbidden_hostnames:
        if lowered == forbidden:
            return True
    return False


def _normalize_action_path(
    path_value: Any,
    *,
    max_path_length: int,
    forbidden_prefixes: Sequence[str],
) -> str:
    path = str(path_value or "").strip()
    if not path:
        raise FrontierActionValidationError("action path must not be empty")
    if len(path) > max_path_length:
        raise FrontierActionValidationError(
            f"action path exceeds max length {max_path_length}: {path}"
        )
    if not path.startswith("/"):
        raise FrontierActionValidationError(f"action path must start with '/': {path}")
    if path.startswith("//"):
        raise FrontierActionValidationError(f"action path must not start with '//': {path}")

    parsed = urllib.parse.urlparse(path)
    if parsed.scheme or parsed.netloc:
        raise FrontierActionValidationError(
            f"action path must be relative and must not include scheme/host: {path}"
        )
    if parsed.fragment:
        raise FrontierActionValidationError(f"action path must not include fragments: {path}")
    if parsed.query:
        raise FrontierActionValidationError(
            f"action path must not include query (use action.query object): {path}"
        )
    if ".." in path:
        raise FrontierActionValidationError(
            f"action path must not include traversal segments: {path}"
        )
    if _forbidden_path(path, forbidden_prefixes):
        raise FrontierActionValidationError(
            f"action path violates forbidden prefix policy: {path}"
        )
    return path


def _normalize_action_query(
    query_value: Any,
    *,
    max_pairs: int,
    max_query_length: int,
) -> Dict[str, str]:
    if query_value is None:
        return {}
    if not isinstance(query_value, dict):
        raise FrontierActionValidationError("action query must be an object when present")
    if len(query_value) > max_pairs:
        raise FrontierActionValidationError(
            f"action query exceeds max pair budget {max_pairs}"
        )

    normalized: Dict[str, str] = {}
    for key in sorted(query_value.keys(), key=lambda item: str(item)):
        key_name = str(key).strip()
        value_text = str(query_value[key]).strip()
        if not key_name:
            raise FrontierActionValidationError("action query keys must not be empty")
        normalized[key_name] = value_text

    encoded = urllib.parse.urlencode(normalized, doseq=False)
    if len(encoded) > max_query_length:
        raise FrontierActionValidationError(
            f"action query exceeds max encoded length {max_query_length}"
        )
    return normalized


def _validate_contract_defaults(contract: Dict[str, Any]) -> None:
    defaults = contract.get("default_actions")
    if not isinstance(defaults, list) or not defaults:
        raise FrontierActionContractError("default_actions must be a non-empty array")
    for action in defaults:
        if not isinstance(action, dict):
            raise FrontierActionContractError("default_actions entries must be objects")


def _action_method_for_type(action_type: str) -> str:
    normalized = str(action_type or "").strip()
    if normalized == "http_head":
        return "HEAD"
    return "GET"


def load_frontier_action_contract(
    path: Path = DEFAULT_FRONTIER_ACTION_CONTRACT_PATH,
) -> Dict[str, Any]:
    if not path.exists():
        raise FrontierActionContractError(f"frontier action contract not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise FrontierActionContractError(
            f"invalid frontier action contract JSON: {path}"
        ) from exc
    if not isinstance(payload, dict):
        raise FrontierActionContractError("frontier action contract must be a JSON object")
    if str(payload.get("schema_version") or "").strip() != "frontier-action-contract.v1":
        raise FrontierActionContractError(
            "frontier action contract schema_version must be frontier-action-contract.v1"
        )

    allowed_tools = payload.get("allowed_tools")
    if not isinstance(allowed_tools, list) or not allowed_tools:
        raise FrontierActionContractError("allowed_tools must be a non-empty array")
    for item in allowed_tools:
        if not str(item or "").strip():
            raise FrontierActionContractError("allowed_tools entries must not be empty")

    network_constraints = payload.get("network_constraints")
    if not isinstance(network_constraints, dict):
        raise FrontierActionContractError("network_constraints must be an object")
    allowed_schemes = network_constraints.get("allowed_schemes")
    if not isinstance(allowed_schemes, list) or not allowed_schemes:
        raise FrontierActionContractError("network_constraints.allowed_schemes must be non-empty")
    for scheme in allowed_schemes:
        if str(scheme or "").strip().lower() not in {"http", "https"}:
            raise FrontierActionContractError(
                f"unsupported network scheme in contract: {scheme}"
            )
    if not isinstance(network_constraints.get("enforce_single_origin_from_allowlist"), bool):
        raise FrontierActionContractError(
            "network_constraints.enforce_single_origin_from_allowlist must be boolean"
        )
    forbidden_hostnames = network_constraints.get("forbidden_hostnames")
    if not isinstance(forbidden_hostnames, list):
        raise FrontierActionContractError("network_constraints.forbidden_hostnames must be an array")
    for hostname in forbidden_hostnames:
        if not str(hostname or "").strip():
            raise FrontierActionContractError(
                "network_constraints.forbidden_hostnames must not include empty entries"
            )

    budgets = payload.get("budgets")
    if not isinstance(budgets, dict):
        raise FrontierActionContractError("budgets must be an object")
    for key in (
        "max_actions_per_run",
        "max_time_budget_seconds",
        "max_query_pairs_per_action",
    ):
        if not _is_positive_int(budgets.get(key)):
            raise FrontierActionContractError(f"budgets.{key} must be integer > 0")

    forbidden_data_access = payload.get("forbidden_data_access")
    if not isinstance(forbidden_data_access, dict):
        raise FrontierActionContractError("forbidden_data_access must be an object")
    for key in (
        "forbidden_env_prefixes",
        "forbidden_env_keys",
        "forbidden_headers",
        "forbidden_path_prefixes",
    ):
        values = forbidden_data_access.get(key)
        if not isinstance(values, list) or not values:
            raise FrontierActionContractError(f"forbidden_data_access.{key} must be a non-empty array")
        for value in values:
            if not str(value or "").strip():
                raise FrontierActionContractError(
                    f"forbidden_data_access.{key} must not contain empty values"
                )

    dsl = payload.get("dsl")
    if not isinstance(dsl, dict):
        raise FrontierActionContractError("dsl must be an object")
    if str(dsl.get("schema_version") or "").strip() != "frontier-action-dsl.v1":
        raise FrontierActionContractError("dsl.schema_version must be frontier-action-dsl.v1")
    for key in ("allowed_action_types", "required_action_keys", "optional_action_keys"):
        values = dsl.get(key)
        if not isinstance(values, list) or not values:
            raise FrontierActionContractError(f"dsl.{key} must be a non-empty array")
        for value in values:
            if not str(value or "").strip():
                raise FrontierActionContractError(f"dsl.{key} must not contain empty values")
    for key in ("max_path_length", "max_query_length", "max_label_length"):
        if not _is_positive_int(dsl.get(key)):
            raise FrontierActionContractError(f"dsl.{key} must be integer > 0")
    _validate_contract_defaults(payload)
    return payload


def parse_frontier_actions(raw_value: str) -> List[Dict[str, Any]]:
    text = str(raw_value or "").strip()
    if not text:
        return []
    try:
        payload = json.loads(text)
    except Exception as exc:
        raise FrontierActionValidationError("frontier actions must be valid JSON") from exc
    if not isinstance(payload, list):
        raise FrontierActionValidationError("frontier actions JSON must be an array")
    parsed: List[Dict[str, Any]] = []
    for item in payload:
        if not isinstance(item, dict):
            raise FrontierActionValidationError("each frontier action must be an object")
        parsed.append(dict(item))
    return parsed


def default_frontier_actions(contract: Dict[str, Any]) -> List[Dict[str, Any]]:
    defaults = contract.get("default_actions")
    if not isinstance(defaults, list):
        raise FrontierActionValidationError("contract default_actions must be an array")
    return copy.deepcopy(defaults)


def validate_frontier_actions(
    actions: List[Dict[str, Any]],
    *,
    contract: Dict[str, Any],
    base_url: str,
    allowed_origins: List[str],
    request_budget: int,
    allowed_tools_override: Sequence[str] | None = None,
    allowed_action_types_override: Sequence[str] | None = None,
) -> List[Dict[str, Any]]:
    if not actions:
        raise FrontierActionValidationError("frontier action list must contain at least one action")

    network_constraints = dict(contract.get("network_constraints") or {})
    budgets = dict(contract.get("budgets") or {})
    dsl = dict(contract.get("dsl") or {})
    forbidden_data_access = dict(contract.get("forbidden_data_access") or {})

    allowed_tools_source = (
        allowed_tools_override
        if allowed_tools_override is not None
        else list(contract.get("allowed_tools") or [])
    )
    allowed_action_types_source = (
        allowed_action_types_override
        if allowed_action_types_override is not None
        else list(dsl.get("allowed_action_types") or [])
    )
    allowed_tools = {
        str(item).strip() for item in allowed_tools_source if str(item).strip()
    }
    allowed_action_types = {
        str(item).strip() for item in allowed_action_types_source if str(item).strip()
    }
    required_action_keys = {
        str(item).strip()
        for item in list(dsl.get("required_action_keys") or [])
        if str(item).strip()
    }
    optional_action_keys = {
        str(item).strip()
        for item in list(dsl.get("optional_action_keys") or [])
        if str(item).strip()
    }

    max_actions_by_contract = int(budgets.get("max_actions_per_run") or 1)
    max_actions = max(1, min(int(request_budget), max_actions_by_contract))
    if len(actions) > max_actions:
        raise FrontierActionValidationError(
            f"frontier action count {len(actions)} exceeds allowed max {max_actions}"
        )

    normalized_allowed_origins = {
        _normalized_origin(origin)
        for origin in allowed_origins
        if str(origin or "").strip()
    }
    if not normalized_allowed_origins:
        raise FrontierActionValidationError("allowed origins must contain at least one origin")

    base_origin = _normalized_origin(base_url)
    if base_origin not in normalized_allowed_origins:
        raise FrontierActionValidationError(
            f"base URL origin {base_origin} is not present in allowlist"
        )
    if bool(network_constraints.get("enforce_single_origin_from_allowlist")) and len(
        normalized_allowed_origins
    ) != 1:
        raise FrontierActionValidationError(
            "single-origin mode requires exactly one allowed origin"
        )

    allowed_schemes = {
        str(item).strip().lower()
        for item in list(network_constraints.get("allowed_schemes") or [])
        if str(item).strip()
    }
    forbidden_hostnames = {
        str(item).strip().lower()
        for item in list(network_constraints.get("forbidden_hostnames") or [])
        if str(item).strip()
    }
    forbidden_path_prefixes = [
        str(item).strip().lower()
        for item in list(forbidden_data_access.get("forbidden_path_prefixes") or [])
        if str(item).strip()
    ]

    max_path_length = int(dsl.get("max_path_length") or 1)
    max_query_length = int(dsl.get("max_query_length") or 1)
    max_label_length = int(dsl.get("max_label_length") or 1)
    max_query_pairs = int(budgets.get("max_query_pairs_per_action") or 1)

    validated: List[Dict[str, Any]] = []
    for index, action in enumerate(actions):
        keys = {str(key).strip() for key in action.keys()}
        missing = sorted(required_action_keys - keys)
        if missing:
            raise FrontierActionValidationError(
                f"action[{index}] missing required keys: {', '.join(missing)}"
            )
        allowed_keys = required_action_keys.union(optional_action_keys)
        unknown = sorted(keys - allowed_keys)
        if unknown:
            raise FrontierActionValidationError(
                f"action[{index}] contains unsupported keys: {', '.join(unknown)}"
            )

        action_type = str(action.get("action_type") or "").strip()
        if action_type not in allowed_action_types:
            raise FrontierActionValidationError(
                f"action[{index}] action_type is not allowed: {action_type}"
            )
        if action_type not in allowed_tools:
            raise FrontierActionValidationError(
                f"action[{index}] action_type is not in allowed_tools: {action_type}"
            )

        path = _normalize_action_path(
            action.get("path"),
            max_path_length=max_path_length,
            forbidden_prefixes=forbidden_path_prefixes,
        )
        query = _normalize_action_query(
            action.get("query"),
            max_pairs=max_query_pairs,
            max_query_length=max_query_length,
        )
        label = str(action.get("label") or "").strip()
        if label and len(label) > max_label_length:
            raise FrontierActionValidationError(
                f"action[{index}] label exceeds max length {max_label_length}"
            )

        query_text = urllib.parse.urlencode(query, doseq=False) if query else ""
        action_url = f"{base_origin}{path}"
        if query_text:
            action_url = f"{action_url}?{query_text}"
        parsed_url = urllib.parse.urlparse(action_url)
        if parsed_url.scheme.lower() not in allowed_schemes:
            raise FrontierActionValidationError(
                f"action[{index}] scheme is not allowed: {parsed_url.scheme}"
            )
        if _normalized_origin(action_url) not in normalized_allowed_origins:
            raise FrontierActionValidationError(
                f"action[{index}] resolved origin is not in allowlist: {action_url}"
            )
        if _is_forbidden_hostname(parsed_url.hostname or "", forbidden_hostnames):
            raise FrontierActionValidationError(
                f"action[{index}] target hostname is forbidden: {parsed_url.hostname}"
            )

        validated.append(
            {
                "action_index": index + 1,
                "action_type": action_type,
                "method": _action_method_for_type(action_type),
                "path": path,
                "query": query,
                "label": label,
                "url": action_url,
                "target_origin": _normalized_origin(action_url),
            }
        )
    return validated


def resolve_frontier_actions(
    raw_actions: str,
    *,
    contract: Dict[str, Any],
    base_url: str,
    allowed_origins: List[str],
    request_budget: int,
) -> List[Dict[str, Any]]:
    parsed = parse_frontier_actions(raw_actions)
    if not parsed:
        parsed = default_frontier_actions(contract)
    return validate_frontier_actions(
        parsed,
        contract=contract,
        base_url=base_url,
        allowed_origins=allowed_origins,
        request_budget=request_budget,
    )
