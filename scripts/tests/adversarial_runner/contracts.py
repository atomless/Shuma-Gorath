"""Contract loading and schema-derived helpers for adversarial runner tooling."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict

LANE_CONTRACT_PATH = Path("scripts/tests/adversarial/lane_contract.v1.json")
SIM_TAG_CONTRACT_PATH = Path("scripts/tests/adversarial/sim_tag_contract.v1.json")
FRONTIER_ACTION_CONTRACT_PATH = Path(
    "scripts/tests/adversarial/frontier_action_contract.v1.json"
)
CONTAINER_RUNTIME_PROFILE_PATH = Path(
    "scripts/tests/adversarial/container_runtime_profile.v1.json"
)
DETERMINISTIC_ATTACK_CORPUS_PATH = Path(
    "scripts/tests/adversarial/deterministic_attack_corpus.v1.json"
)


def load_lane_contract(path: Path = LANE_CONTRACT_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"lane contract not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid lane contract JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError(f"lane contract must be a JSON object: {path}")
    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "sim-lane-contract.v1":
        raise RuntimeError(
            f"lane contract schema_version must be sim-lane-contract.v1 (got {schema_version})"
        )
    execution_lane = str(payload.get("execution_lane") or "").strip()
    if execution_lane != "black_box":
        raise RuntimeError(
            f"lane contract execution_lane must be black_box (got {execution_lane})"
        )
    attacker = payload.get("attacker")
    if not isinstance(attacker, dict):
        raise RuntimeError("lane contract attacker section must be an object")
    forbidden_headers = attacker.get("forbidden_headers")
    if not isinstance(forbidden_headers, list) or not forbidden_headers:
        raise RuntimeError("lane contract attacker.forbidden_headers must be a non-empty array")
    forbidden_path_prefixes = attacker.get("forbidden_path_prefixes")
    if not isinstance(forbidden_path_prefixes, list) or not forbidden_path_prefixes:
        raise RuntimeError(
            "lane contract attacker.forbidden_path_prefixes must be a non-empty array"
        )
    required_sim_headers = attacker.get("required_sim_headers")
    if not isinstance(required_sim_headers, list) or not required_sim_headers:
        raise RuntimeError(
            "lane contract attacker.required_sim_headers must be a non-empty array"
        )
    return payload


LANE_CONTRACT = load_lane_contract()
ATTACKER_CONTRACT = dict(LANE_CONTRACT.get("attacker") or {})
ATTACKER_FORBIDDEN_PATH_PREFIXES = tuple(
    str(item).strip()
    for item in ATTACKER_CONTRACT.get("forbidden_path_prefixes", [])
    if str(item).strip()
)
ATTACKER_FORBIDDEN_HEADERS = {
    str(item).strip().lower()
    for item in ATTACKER_CONTRACT.get("forbidden_headers", [])
    if str(item).strip()
}
ATTACKER_REQUIRED_SIM_HEADERS = {
    str(item).strip().lower()
    for item in ATTACKER_CONTRACT.get("required_sim_headers", [])
    if str(item).strip()
}


def load_sim_tag_contract(path: Path = SIM_TAG_CONTRACT_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"sim tag contract not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid sim tag contract JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError(f"sim tag contract must be a JSON object: {path}")
    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "sim-tag-contract.v1":
        raise RuntimeError(
            "sim tag contract schema_version must be sim-tag-contract.v1 "
            f"(got {schema_version})"
        )
    headers = payload.get("headers")
    if not isinstance(headers, dict):
        raise RuntimeError("sim tag contract headers must be an object")
    required_header_keys = {
        "sim_run_id",
        "sim_profile",
        "sim_lane",
        "sim_timestamp",
        "sim_nonce",
        "sim_signature",
    }
    missing_headers = [key for key in required_header_keys if not str(headers.get(key) or "").strip()]
    if missing_headers:
        raise RuntimeError(
            f"sim tag contract headers missing required keys: {', '.join(sorted(missing_headers))}"
        )
    required_sim_headers = payload.get("required_sim_headers")
    if not isinstance(required_sim_headers, list) or not required_sim_headers:
        raise RuntimeError("sim tag contract required_sim_headers must be a non-empty array")
    for key in ("timestamp_max_skew_seconds", "nonce_ttl_seconds", "nonce_max_entries"):
        value = payload.get(key)
        if isinstance(value, bool) or not isinstance(value, int) or int(value) < 1:
            raise RuntimeError(f"sim tag contract {key} must be integer >= 1")
    canonical = payload.get("canonical")
    if not isinstance(canonical, dict):
        raise RuntimeError("sim tag contract canonical must be an object")
    if str(canonical.get("prefix") or "").strip() != "sim-tag.v1":
        raise RuntimeError("sim tag contract canonical.prefix must be sim-tag.v1")
    separator = canonical.get("separator")
    if not isinstance(separator, str) or separator != "\n":
        raise RuntimeError("sim tag contract canonical.separator must be \\n")
    return payload


SIM_TAG_CONTRACT = load_sim_tag_contract()
SIM_TAG_HEADERS = {
    str(key): str(value).strip().lower()
    for key, value in dict(SIM_TAG_CONTRACT.get("headers") or {}).items()
    if str(key).strip() and str(value).strip()
}
SIM_TAG_REQUIRED_SIM_HEADERS = {
    str(value).strip().lower()
    for value in list(SIM_TAG_CONTRACT.get("required_sim_headers") or [])
    if str(value).strip()
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
SIM_TAG_TIMESTAMP_MAX_SKEW_SECONDS = int(
    SIM_TAG_CONTRACT.get("timestamp_max_skew_seconds") or 300
)
SIM_TAG_NONCE_TTL_SECONDS = int(SIM_TAG_CONTRACT.get("nonce_ttl_seconds") or 600)
SIM_TAG_NONCE_MAX_ENTRIES = int(SIM_TAG_CONTRACT.get("nonce_max_entries") or 4096)


def build_sim_tag_canonical_message(
    run_id: str, profile: str, lane: str, timestamp: str, nonce: str
) -> str:
    fields = [
        SIM_TAG_CANONICAL_PREFIX,
        str(run_id).strip(),
        str(profile).strip(),
        str(lane).strip(),
        str(timestamp).strip(),
        str(nonce).strip(),
    ]
    return SIM_TAG_CANONICAL_SEPARATOR.join(fields)


def load_deterministic_attack_corpus(
    path: Path = DETERMINISTIC_ATTACK_CORPUS_PATH,
) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"deterministic attack corpus not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid deterministic attack corpus JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError(f"deterministic attack corpus must be a JSON object: {path}")
    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "sim-deterministic-attack-corpus.v1":
        raise RuntimeError(
            "deterministic attack corpus schema_version must be "
            f"sim-deterministic-attack-corpus.v1 (got {schema_version})"
        )
    corpus_revision = str(payload.get("corpus_revision") or "").strip()
    if not corpus_revision:
        raise RuntimeError("deterministic attack corpus corpus_revision must be non-empty")
    taxonomy_version = str(payload.get("taxonomy_version") or "").strip()
    if not taxonomy_version:
        raise RuntimeError("deterministic attack corpus taxonomy_version must be non-empty")

    runtime_profile_name = str(payload.get("runtime_profile") or "").strip()
    if runtime_profile_name != "runtime_toggle":
        raise RuntimeError(
            "deterministic attack corpus runtime_profile must be runtime_toggle "
            f"(got {runtime_profile_name})"
        )
    runtime_profile = payload.get(runtime_profile_name)
    if not isinstance(runtime_profile, dict):
        raise RuntimeError(
            f"deterministic attack corpus runtime profile missing object: {runtime_profile_name}"
        )
    primary_public_paths = runtime_profile.get("primary_public_paths")
    if not isinstance(primary_public_paths, list) or not primary_public_paths:
        raise RuntimeError(
            "deterministic attack corpus runtime_toggle.primary_public_paths must be a non-empty array"
        )

    ci_profile_name = str(payload.get("ci_profile") or "").strip()
    if ci_profile_name != "ci_oracle":
        raise RuntimeError(
            "deterministic attack corpus ci_profile must be ci_oracle "
            f"(got {ci_profile_name})"
        )
    ci_profile = payload.get(ci_profile_name)
    if not isinstance(ci_profile, dict):
        raise RuntimeError(
            f"deterministic attack corpus CI profile missing object: {ci_profile_name}"
        )
    drivers = ci_profile.get("drivers")
    if not isinstance(drivers, dict) or not drivers:
        raise RuntimeError(
            "deterministic attack corpus ci_oracle.drivers must be a non-empty object"
        )
    for driver_name, driver_payload in drivers.items():
        normalized_driver_name = str(driver_name or "").strip()
        if not normalized_driver_name:
            raise RuntimeError(
                "deterministic attack corpus ci_oracle.drivers contains empty key"
            )
        if not isinstance(driver_payload, dict):
            raise RuntimeError(
                f"deterministic attack corpus driver row must be an object: {normalized_driver_name}"
            )
        driver_class = str(driver_payload.get("driver_class") or "").strip()
        if not driver_class:
            raise RuntimeError(
                "deterministic attack corpus driver row missing driver_class: "
                f"{normalized_driver_name}"
            )
        path_hint = str(driver_payload.get("path_hint") or "").strip()
        if not path_hint.startswith("/"):
            raise RuntimeError(
                "deterministic attack corpus driver row path_hint must be absolute-path like /...: "
                f"{normalized_driver_name}"
            )
        taxonomy_category = str(driver_payload.get("taxonomy_category") or "").strip()
        if not taxonomy_category:
            raise RuntimeError(
                "deterministic attack corpus driver row missing taxonomy_category: "
                f"{normalized_driver_name}"
            )
    return payload


DETERMINISTIC_ATTACK_CORPUS = load_deterministic_attack_corpus()
DETERMINISTIC_ATTACK_CORPUS_REVISION = str(
    DETERMINISTIC_ATTACK_CORPUS.get("corpus_revision") or ""
).strip()
DETERMINISTIC_ATTACK_CORPUS_TAXONOMY_VERSION = str(
    DETERMINISTIC_ATTACK_CORPUS.get("taxonomy_version") or ""
).strip()
DETERMINISTIC_ATTACK_CORPUS_RUNTIME_PROFILE = dict(
    DETERMINISTIC_ATTACK_CORPUS.get(
        str(DETERMINISTIC_ATTACK_CORPUS.get("runtime_profile") or "runtime_toggle")
    )
    or {}
)
DETERMINISTIC_ATTACK_CORPUS_CI_PROFILE = dict(
    DETERMINISTIC_ATTACK_CORPUS.get(
        str(DETERMINISTIC_ATTACK_CORPUS.get("ci_profile") or "ci_oracle")
    )
    or {}
)
DETERMINISTIC_DRIVER_DEFINITIONS = {
    str(driver_name).strip(): dict(driver_payload or {})
    for driver_name, driver_payload in dict(
        DETERMINISTIC_ATTACK_CORPUS_CI_PROFILE.get("drivers") or {}
    ).items()
    if str(driver_name).strip()
}
DETERMINISTIC_DRIVER_CLASS_MAP = {
    str(driver_name): str(driver_payload.get("driver_class") or "").strip()
    for driver_name, driver_payload in DETERMINISTIC_DRIVER_DEFINITIONS.items()
}
DETERMINISTIC_DRIVER_PATH_HINTS = {
    str(driver_name): str(driver_payload.get("path_hint") or "/").strip() or "/"
    for driver_name, driver_payload in DETERMINISTIC_DRIVER_DEFINITIONS.items()
}
