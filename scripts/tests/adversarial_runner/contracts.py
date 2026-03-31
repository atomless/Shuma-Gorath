"""Contract loading and schema-derived helpers for adversarial runner tooling."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict

from scripts.tests.adversarial_runner.transport_envelope import normalize_transport_envelope

LANE_CONTRACT_PATH = Path("scripts/tests/adversarial/lane_contract.v1.json")
LANE_REALISM_CONTRACT_PATH = Path("scripts/tests/adversarial/lane_realism_contract.v1.json")
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
LANE_REALISM_PROFILE_SCHEMA_VERSION = "sim-lane-realism-profile.v1"
LANE_REALISM_RECEIPT_SCHEMA_VERSION = "sim-lane-realism-receipt.v1"


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


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _normalize_realism_range(value: Any, *, field_name: str) -> Dict[str, int]:
    if not isinstance(value, dict):
        raise RuntimeError(f"{field_name} must be an object")
    min_value = value.get("min")
    max_value = value.get("max")
    if not _is_non_negative_int(min_value) or not _is_non_negative_int(max_value):
        raise RuntimeError(f"{field_name}.min and {field_name}.max must be integers >= 0")
    if int(max_value) < int(min_value):
        raise RuntimeError(f"{field_name}.max must be >= {field_name}.min")
    return {"min": int(min_value), "max": int(max_value)}


def normalize_lane_realism_profile(
    payload: Any,
    *,
    field_name: str,
) -> Dict[str, Any]:
    if not isinstance(payload, dict):
        raise RuntimeError(f"{field_name} must be an object")
    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != LANE_REALISM_PROFILE_SCHEMA_VERSION:
        raise RuntimeError(
            f"{field_name}.schema_version must be {LANE_REALISM_PROFILE_SCHEMA_VERSION}"
        )
    profile_id = str(payload.get("profile_id") or "").strip()
    if not profile_id:
        raise RuntimeError(f"{field_name}.profile_id must be non-empty")
    activity_unit = str(payload.get("activity_unit") or "").strip()
    if activity_unit not in {"request", "action"}:
        raise RuntimeError(f"{field_name}.activity_unit must be request or action")
    browser_propensity = str(payload.get("browser_propensity") or "").strip()
    if browser_propensity not in {"none", "preferred", "required"}:
        raise RuntimeError(
            f"{field_name}.browser_propensity must be none, preferred, or required"
        )
    javascript_execution = str(payload.get("javascript_execution") or "").strip()
    if javascript_execution not in {"disabled", "opportunistic", "required"}:
        raise RuntimeError(
            f"{field_name}.javascript_execution must be disabled, opportunistic, or required"
        )
    retry_ceiling = payload.get("retry_ceiling")
    if not _is_non_negative_int(retry_ceiling):
        raise RuntimeError(f"{field_name}.retry_ceiling must be integer >= 0")
    pressure_envelope = payload.get("pressure_envelope")
    if not isinstance(pressure_envelope, dict):
        raise RuntimeError(f"{field_name}.pressure_envelope must be an object")
    max_activities = pressure_envelope.get("max_activities")
    max_time_budget_ms = pressure_envelope.get("max_time_budget_ms")
    if not _is_non_negative_int(max_activities) or int(max_activities) < 1:
        raise RuntimeError(f"{field_name}.pressure_envelope.max_activities must be integer >= 1")
    if not _is_non_negative_int(max_time_budget_ms) or int(max_time_budget_ms) < 1:
        raise RuntimeError(
            f"{field_name}.pressure_envelope.max_time_budget_ms must be integer >= 1"
        )
    exploration_envelope = payload.get("exploration_envelope")
    if not isinstance(exploration_envelope, dict):
        raise RuntimeError(f"{field_name}.exploration_envelope must be an object")
    max_depth = exploration_envelope.get("max_depth")
    max_bytes = exploration_envelope.get("max_bytes")
    if not _is_non_negative_int(max_depth) or int(max_depth) < 1:
        raise RuntimeError(f"{field_name}.exploration_envelope.max_depth must be integer >= 1")
    if not _is_non_negative_int(max_bytes) or int(max_bytes) < 1:
        raise RuntimeError(f"{field_name}.exploration_envelope.max_bytes must be integer >= 1")
    recurrence_envelope = payload.get("recurrence_envelope")
    if not isinstance(recurrence_envelope, dict):
        raise RuntimeError(f"{field_name}.recurrence_envelope must be an object")
    recurrence_strategy = str(recurrence_envelope.get("strategy") or "").strip()
    if recurrence_strategy != "bounded_single_tick_reentry":
        raise RuntimeError(
            f"{field_name}.recurrence_envelope.strategy must be bounded_single_tick_reentry"
        )
    recurrence_scope = str(recurrence_envelope.get("reentry_scope") or "").strip()
    if recurrence_scope != "within_run":
        raise RuntimeError(
            f"{field_name}.recurrence_envelope.reentry_scope must be within_run"
        )
    max_reentries_per_run = recurrence_envelope.get("max_reentries_per_run")
    if not _is_non_negative_int(max_reentries_per_run) or int(max_reentries_per_run) < 1:
        raise RuntimeError(
            f"{field_name}.recurrence_envelope.max_reentries_per_run must be integer >= 1"
        )

    identity_rotation = payload.get("identity_rotation")
    if not isinstance(identity_rotation, dict):
        raise RuntimeError(f"{field_name}.identity_rotation must be an object")
    strategy = str(identity_rotation.get("strategy") or "").strip()
    if strategy not in {
        "none",
        "per_burst_when_proxy_available",
        "per_n_activities_when_proxy_available",
    }:
        raise RuntimeError(
            f"{field_name}.identity_rotation.strategy must be a supported strategy"
        )
    min_every = identity_rotation.get("min_every_n_activities")
    max_every = identity_rotation.get("max_every_n_activities")
    if not _is_non_negative_int(min_every) or not _is_non_negative_int(max_every):
        raise RuntimeError(
            f"{field_name}.identity_rotation min/max cadence values must be integers >= 0"
        )
    if int(max_every) < int(min_every):
        raise RuntimeError(
            f"{field_name}.identity_rotation.max_every_n_activities must be >= min"
        )
    if strategy == "none" and (int(min_every) != 0 or int(max_every) != 0):
        raise RuntimeError(
            f"{field_name}.identity_rotation cadence values must be 0 when strategy is none"
        )
    if strategy != "none" and (int(min_every) < 1 or int(max_every) < 1):
        raise RuntimeError(
            f"{field_name}.identity_rotation cadence values must be >= 1 for rotating strategies"
        )
    stable_session_per_tick = identity_rotation.get("stable_session_per_tick")
    proxy_required = identity_rotation.get("proxy_required")
    if not isinstance(stable_session_per_tick, bool):
        raise RuntimeError(
            f"{field_name}.identity_rotation.stable_session_per_tick must be boolean"
        )
    if not isinstance(proxy_required, bool):
        raise RuntimeError(f"{field_name}.identity_rotation.proxy_required must be boolean")

    identity_envelope = payload.get("identity_envelope")
    if not isinstance(identity_envelope, dict):
        raise RuntimeError(f"{field_name}.identity_envelope must be an object")
    identity_classes = [
        str(item).strip()
        for item in list(identity_envelope.get("identity_classes") or [])
        if str(item).strip()
    ]
    if not identity_classes:
        raise RuntimeError(
            f"{field_name}.identity_envelope.identity_classes must be a non-empty array"
        )
    if any(
        item not in {"residential", "mobile", "datacenter"} for item in identity_classes
    ):
        raise RuntimeError(
            f"{field_name}.identity_envelope.identity_classes must contain only supported classes"
        )
    geo_affinity_mode = str(identity_envelope.get("geo_affinity_mode") or "").strip()
    if geo_affinity_mode != "pool_aligned":
        raise RuntimeError(
            f"{field_name}.identity_envelope.geo_affinity_mode must be pool_aligned"
        )
    session_stickiness = str(identity_envelope.get("session_stickiness") or "").strip()
    if session_stickiness not in {"stable_per_identity", "stable_per_tick"}:
        raise RuntimeError(
            f"{field_name}.identity_envelope.session_stickiness must be stable_per_identity or stable_per_tick"
        )
    degraded_without_pool = str(identity_envelope.get("degraded_without_pool") or "").strip()
    if degraded_without_pool not in {"local_session_only", "local_browser_session_only"}:
        raise RuntimeError(
            f"{field_name}.identity_envelope.degraded_without_pool must be a supported degraded mode"
        )

    receipt_contract = payload.get("receipt_contract")
    if not isinstance(receipt_contract, dict):
        raise RuntimeError(f"{field_name}.receipt_contract must be an object")
    receipt_schema = str(receipt_contract.get("schema_version") or "").strip()
    if receipt_schema != LANE_REALISM_RECEIPT_SCHEMA_VERSION:
        raise RuntimeError(
            f"{field_name}.receipt_contract.schema_version must be {LANE_REALISM_RECEIPT_SCHEMA_VERSION}"
        )
    required_fields = [
        str(item).strip()
        for item in list(receipt_contract.get("required_fields") or [])
        if str(item).strip()
    ]
    if not required_fields:
        raise RuntimeError(
            f"{field_name}.receipt_contract.required_fields must be a non-empty array"
        )

    return {
        "schema_version": schema_version,
        "profile_id": profile_id,
        "activity_unit": activity_unit,
        "activity_budget": _normalize_realism_range(
            payload.get("activity_budget"),
            field_name=f"{field_name}.activity_budget",
        ),
        "burst_size": _normalize_realism_range(
            payload.get("burst_size"),
            field_name=f"{field_name}.burst_size",
        ),
        "intra_burst_jitter_ms": _normalize_realism_range(
            payload.get("intra_burst_jitter_ms"),
            field_name=f"{field_name}.intra_burst_jitter_ms",
        ),
        "between_burst_pause_ms": _normalize_realism_range(
            payload.get("between_burst_pause_ms"),
            field_name=f"{field_name}.between_burst_pause_ms",
        ),
        "navigation_dwell_ms": _normalize_realism_range(
            payload.get("navigation_dwell_ms"),
            field_name=f"{field_name}.navigation_dwell_ms",
        ),
        "identity_rotation": {
            "strategy": strategy,
            "min_every_n_activities": int(min_every),
            "max_every_n_activities": int(max_every),
            "stable_session_per_tick": stable_session_per_tick,
            "proxy_required": proxy_required,
        },
        "identity_envelope": {
            "identity_classes": identity_classes,
            "geo_affinity_mode": geo_affinity_mode,
            "session_stickiness": session_stickiness,
            "degraded_without_pool": degraded_without_pool,
        },
        "transport_envelope": normalize_transport_envelope(
            payload.get("transport_envelope"),
            field_name=f"{field_name}.transport_envelope",
        ),
        "browser_propensity": browser_propensity,
        "javascript_execution": javascript_execution,
        "retry_ceiling": int(retry_ceiling),
        "pressure_envelope": {
            "max_activities": int(max_activities),
            "max_time_budget_ms": int(max_time_budget_ms),
        },
        "exploration_envelope": {
            "max_depth": int(max_depth),
            "max_bytes": int(max_bytes),
        },
        "recurrence_envelope": {
            "strategy": recurrence_strategy,
            "reentry_scope": recurrence_scope,
            "dormant_gap_seconds": _normalize_realism_range(
                recurrence_envelope.get("dormant_gap_seconds"),
                field_name=f"{field_name}.recurrence_envelope.dormant_gap_seconds",
            ),
            "max_reentries_per_run": int(max_reentries_per_run),
        },
        "receipt_contract": {
            "schema_version": receipt_schema,
            "required_fields": required_fields,
        },
    }


def load_lane_realism_contract(path: Path = LANE_REALISM_CONTRACT_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"lane realism contract not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid lane realism contract JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError(f"lane realism contract must be a JSON object: {path}")
    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "sim-lane-realism-contract.v1":
        raise RuntimeError(
            "lane realism contract schema_version must be sim-lane-realism-contract.v1 "
            f"(got {schema_version})"
        )
    profiles = payload.get("profiles")
    if not isinstance(profiles, dict) or not profiles:
        raise RuntimeError("lane realism contract profiles must be a non-empty object")
    normalized_profiles: Dict[str, Dict[str, Dict[str, Any]]] = {}
    for lane_name, lane_profiles in profiles.items():
        normalized_lane = str(lane_name or "").strip()
        if not normalized_lane:
            raise RuntimeError("lane realism contract profiles contains empty lane key")
        if not isinstance(lane_profiles, dict) or not lane_profiles:
            raise RuntimeError(
                f"lane realism contract profiles.{normalized_lane} must be a non-empty object"
            )
        normalized_profiles[normalized_lane] = {}
        for mode_name, profile_payload in lane_profiles.items():
            normalized_mode = str(mode_name or "").strip()
            if not normalized_mode:
                raise RuntimeError(
                    f"lane realism contract profiles.{normalized_lane} contains empty mode key"
                )
            normalized_profiles[normalized_lane][normalized_mode] = normalize_lane_realism_profile(
                profile_payload,
                field_name=f"lane_realism_contract.profiles.{normalized_lane}.{normalized_mode}",
            )
    return {"schema_version": schema_version, "profiles": normalized_profiles}


def resolve_lane_realism_profile(
    lane: str,
    fulfillment_mode: str,
    *,
    contract: Dict[str, Any] | None = None,
) -> Dict[str, Any]:
    resolved_contract = contract or LANE_REALISM_CONTRACT
    profiles = dict(resolved_contract.get("profiles") or {})
    lane_profiles = dict(profiles.get(str(lane).strip()) or {})
    if not lane_profiles:
        raise RuntimeError(f"lane realism contract missing lane profile set: {lane}")
    profile = lane_profiles.get(str(fulfillment_mode).strip())
    if not isinstance(profile, dict) or not profile:
        raise RuntimeError(
            f"lane realism contract missing mode profile: {lane}/{fulfillment_mode}"
        )
    return dict(profile)


LANE_CONTRACT = load_lane_contract()
LANE_REALISM_CONTRACT = load_lane_realism_contract()
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
