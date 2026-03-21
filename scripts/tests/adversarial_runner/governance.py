"""Frontier payload governance and safety helpers for the adversarial runner."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any, Dict

from scripts.tests.adversarial_runner.runtime_state import SimulationError
from scripts.tests.adversarial_runner.shared import dict_or_empty, int_or_zero, list_or_empty

FRONTIER_PAYLOAD_SCHEMA_PATH = Path("scripts/tests/adversarial/frontier_payload_schema.v1.json")
FRONTIER_ATTACK_GENERATION_CONTRACT_PATH = Path(
    "scripts/tests/adversarial/frontier_attack_generation_contract.v1.json"
)
FRONTIER_FORBIDDEN_FIELD_SUBSTRINGS = (
    "secret",
    "api_key",
    "apikey",
    "authorization",
    "cookie",
    "session",
    "token",
    "password",
)
FRONTIER_QUASI_IDENTIFIER_SUBSTRINGS = ("ip", "email", "phone", "user_id", "userid")
FRONTIER_IP_ADDRESS_PATTERN = re.compile(
    r"^(?:\d{1,3}\.){3}\d{1,3}$|^(?:[0-9A-Fa-f]{0,4}:){2,7}[0-9A-Fa-f]{0,4}$"
)


def canonicalize_frontier_payload(value: Any) -> Any:
    if isinstance(value, dict):
        return {
            str(key): canonicalize_frontier_payload(value[key])
            for key in sorted(value.keys(), key=lambda item: str(item))
        }
    if isinstance(value, list):
        return [canonicalize_frontier_payload(item) for item in value]
    if isinstance(value, str):
        return value.strip()
    return value


def classify_frontier_field(name: str) -> str:
    normalized = str(name or "").strip().lower()
    if not normalized:
        return "allowed"
    if any(token in normalized for token in FRONTIER_FORBIDDEN_FIELD_SUBSTRINGS):
        return "forbidden"
    if any(token in normalized for token in FRONTIER_QUASI_IDENTIFIER_SUBSTRINGS):
        return "quasi_identifier"
    return "allowed"


def drop_forbidden_frontier_fields(value: Any) -> Any:
    if isinstance(value, dict):
        filtered: Dict[str, Any] = {}
        for key in sorted(value.keys(), key=lambda item: str(item)):
            if classify_frontier_field(str(key)) == "forbidden":
                continue
            filtered[str(key)] = drop_forbidden_frontier_fields(value[key])
        return filtered
    if isinstance(value, list):
        return [drop_forbidden_frontier_fields(item) for item in value]
    return value


def mask_frontier_quasi_identifiers(value: Any, key_hint: str = "") -> Any:
    if isinstance(value, dict):
        masked: Dict[str, Any] = {}
        for key in sorted(value.keys(), key=lambda item: str(item)):
            key_name = str(key)
            classification = classify_frontier_field(key_name)
            nested = mask_frontier_quasi_identifiers(value[key], key_name)
            if classification == "quasi_identifier":
                if isinstance(nested, list):
                    masked[key_name] = ["[masked]"] * len(nested)
                else:
                    masked[key_name] = "[masked]"
            else:
                masked[key_name] = nested
        return masked
    if isinstance(value, list):
        return [mask_frontier_quasi_identifiers(item, key_hint) for item in value]
    if isinstance(value, str):
        if classify_frontier_field(key_hint) == "quasi_identifier":
            return "[masked]"
        if FRONTIER_IP_ADDRESS_PATTERN.match(value.strip()):
            return "[masked_ip]"
        return value
    return value


def load_frontier_payload_schema() -> Dict[str, Any]:
    if not FRONTIER_PAYLOAD_SCHEMA_PATH.exists():
        raise SimulationError(f"Missing frontier payload schema: {FRONTIER_PAYLOAD_SCHEMA_PATH}")
    try:
        parsed = json.loads(FRONTIER_PAYLOAD_SCHEMA_PATH.read_text(encoding="utf-8"))
    except Exception as exc:
        raise SimulationError(
            f"Invalid frontier payload schema JSON: {FRONTIER_PAYLOAD_SCHEMA_PATH}"
        ) from exc
    if not isinstance(parsed, dict):
        raise SimulationError(
            f"Frontier payload schema must be a JSON object: {FRONTIER_PAYLOAD_SCHEMA_PATH}"
        )
    return parsed


def load_frontier_attack_generation_contract(
    path: Path = FRONTIER_ATTACK_GENERATION_CONTRACT_PATH,
) -> Dict[str, Any]:
    if not path.exists():
        raise SimulationError(f"Missing frontier attack-generation contract: {path}")
    try:
        parsed = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise SimulationError(f"Invalid frontier attack-generation contract JSON: {path}") from exc
    if not isinstance(parsed, dict):
        raise SimulationError(
            f"Frontier attack-generation contract must be a JSON object: {path}"
        )

    schema_version = str(parsed.get("schema_version") or "").strip()
    if schema_version != "frontier-attack-generation-contract.v1":
        raise SimulationError(
            "frontier attack-generation contract schema_version must be "
            "frontier-attack-generation-contract.v1 "
            f"(got {schema_version})"
        )

    for key in (
        "objective",
        "constraints",
        "allowed_actions",
        "forbidden_data",
        "resource_budgets",
        "novelty_expectations",
        "diversity_scoring",
    ):
        if key not in parsed:
            raise SimulationError(
                f"frontier attack-generation contract missing required key: {key}"
            )

    constraints = dict_or_empty(parsed.get("constraints"))
    lanes = {
        str(item).strip()
        for item in list_or_empty(constraints.get("allowed_execution_lanes"))
        if str(item).strip()
    }
    if "black_box" not in lanes:
        raise SimulationError(
            "frontier attack-generation contract constraints.allowed_execution_lanes "
            "must include black_box"
        )

    budgets = dict_or_empty(parsed.get("resource_budgets"))
    max_per_seed = int_or_zero(budgets.get("max_generated_candidates_per_seed"))
    max_per_run = int_or_zero(budgets.get("max_generated_candidates_per_run"))
    if max_per_seed < 1 or max_per_run < 1:
        raise SimulationError(
            "frontier attack-generation contract resource_budgets must define "
            "max_generated_candidates_per_seed >= 1 and max_generated_candidates_per_run >= 1"
        )

    allowed_actions = dict_or_empty(parsed.get("allowed_actions"))
    mutation_catalog = list_or_empty(allowed_actions.get("mutation_catalog"))
    if not mutation_catalog:
        raise SimulationError(
            "frontier attack-generation contract allowed_actions.mutation_catalog must be non-empty"
        )
    for index, mutation in enumerate(mutation_catalog):
        entry = dict_or_empty(mutation)
        mutation_id = str(entry.get("id") or "").strip()
        behavioral_class = str(entry.get("behavioral_class") or "").strip()
        novelty_weight = float(entry.get("novelty_weight") or 0.0)
        if not mutation_id:
            raise SimulationError(
                "frontier attack-generation contract mutation_catalog item "
                f"{index} missing id"
            )
        if not behavioral_class:
            raise SimulationError(
                "frontier attack-generation contract mutation_catalog item "
                f"{index} missing behavioral_class"
            )
        if novelty_weight < 0.0 or novelty_weight > 1.0:
            raise SimulationError(
                "frontier attack-generation contract mutation_catalog item "
                f"{index} novelty_weight must be within [0,1]"
            )
    return parsed


FRONTIER_ATTACK_GENERATION_CONTRACT = load_frontier_attack_generation_contract()
FRONTIER_ATTACK_GENERATION_CONTRACT_SHA256 = hashlib.sha256(
    json.dumps(
        FRONTIER_ATTACK_GENERATION_CONTRACT,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
).hexdigest()


def has_raw_ip_string(value: Any) -> bool:
    if isinstance(value, dict):
        return any(has_raw_ip_string(item) for item in value.values())
    if isinstance(value, list):
        return any(has_raw_ip_string(item) for item in value)
    if isinstance(value, str):
        return bool(FRONTIER_IP_ADDRESS_PATTERN.match(value.strip()))
    return False


def validate_frontier_payload_schema(payload: Dict[str, Any]) -> None:
    schema = load_frontier_payload_schema()
    schema_version = str(schema.get("schema_version") or "").strip()
    allowed_top_level = schema.get("allowed_top_level_keys")
    if not isinstance(allowed_top_level, list) or not allowed_top_level:
        raise SimulationError("Frontier payload schema is missing allowed_top_level_keys.")
    allowed_keys = {str(key) for key in allowed_top_level}
    if payload.get("schema_version") != schema_version:
        raise SimulationError(
            f"Frontier payload schema_version mismatch: expected={schema_version} got={payload.get('schema_version')}"
        )
    unknown_keys = sorted([str(key) for key in payload.keys() if str(key) not in allowed_keys])
    if unknown_keys:
        raise SimulationError(
            f"Frontier payload contains unknown top-level keys: {', '.join(unknown_keys)}"
        )
    forbidden_keys = sorted(
        [str(key) for key in payload.keys() if classify_frontier_field(str(key)) == "forbidden"]
    )
    if forbidden_keys:
        raise SimulationError(
            f"Frontier payload contains forbidden top-level keys: {', '.join(forbidden_keys)}"
        )
    if has_raw_ip_string(payload):
        raise SimulationError("Frontier payload contains raw IP-like values after redaction.")


def sanitize_frontier_payload(payload: Dict[str, Any]) -> Dict[str, Any]:
    canonical = canonicalize_frontier_payload(payload)
    if not isinstance(canonical, dict):
        raise SimulationError("Frontier payload must be a JSON object before sanitization.")
    dropped = drop_forbidden_frontier_fields(canonical)
    masked = mask_frontier_quasi_identifiers(dropped)
    if not isinstance(masked, dict):
        raise SimulationError("Frontier payload sanitization produced a non-object payload.")
    validate_frontier_payload_schema(masked)
    return masked
