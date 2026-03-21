#!/usr/bin/env python3
"""Deterministic adversarial simulation runner for Shuma-Gorath.

This runner executes manifest-defined simulation profiles (fast smoke, abuse, Akamai)
with bounded runtime and quantitative gate assertions.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import hmac
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from http.cookies import SimpleCookie
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests.playwright_runtime import (
    DEFAULT_PLAYWRIGHT_BROWSER_CACHE,
    PlaywrightRuntimeStatus,
    build_playwright_env,
    ensure_playwright_chromium,
)
from scripts.tests.adversarial_runner.contracts import (
    ATTACKER_CONTRACT,
    ATTACKER_FORBIDDEN_HEADERS,
    ATTACKER_FORBIDDEN_PATH_PREFIXES,
    ATTACKER_REQUIRED_SIM_HEADERS,
    DETERMINISTIC_ATTACK_CORPUS,
    DETERMINISTIC_ATTACK_CORPUS_PATH,
    DETERMINISTIC_ATTACK_CORPUS_REVISION,
    DETERMINISTIC_ATTACK_CORPUS_RUNTIME_PROFILE,
    DETERMINISTIC_ATTACK_CORPUS_TAXONOMY_VERSION,
    DETERMINISTIC_DRIVER_CLASS_MAP,
    DETERMINISTIC_DRIVER_DEFINITIONS,
    DETERMINISTIC_DRIVER_PATH_HINTS,
    LANE_CONTRACT,
    LANE_CONTRACT_PATH,
    SIM_TAG_CANONICAL_PREFIX,
    SIM_TAG_CANONICAL_SEPARATOR,
    SIM_TAG_CONTRACT,
    SIM_TAG_CONTRACT_PATH,
    SIM_TAG_HEADER_LANE,
    SIM_TAG_HEADER_NONCE,
    SIM_TAG_HEADER_PROFILE,
    SIM_TAG_HEADER_RUN_ID,
    SIM_TAG_HEADER_SIGNATURE,
    SIM_TAG_HEADER_TIMESTAMP,
    SIM_TAG_HEADERS,
    SIM_TAG_NONCE_MAX_ENTRIES,
    SIM_TAG_NONCE_TTL_SECONDS,
    SIM_TAG_REQUIRED_SIM_HEADERS,
    SIM_TAG_TIMESTAMP_MAX_SKEW_SECONDS,
    build_sim_tag_canonical_message,
    load_deterministic_attack_corpus,
    load_lane_contract,
    load_sim_tag_contract,
)
from scripts.tests.adversarial_runner.runtime_state import (
    AttackerPlaneClient,
    ControlPlaneClient,
    HttpResult,
    NoRedirectHandler,
    ScenarioResult,
    SimulationError,
)
from scripts.tests.adversarial_runner.reporting import (
    build_cost_governance_report,
    build_retention_lifecycle_report,
    build_security_privacy_report,
)
from scripts.tests.adversarial_runner.discovery_scoring import (
    build_attack_plan as build_attack_plan_impl,
    build_frontier_metadata as build_frontier_metadata_impl,
    clamp_score,
    compute_frontier_discovery_quality_metrics,
    compute_frontier_diversity_scoring,
    frontier_path_hint_for_scenario as frontier_path_hint_for_scenario_impl,
)
from scripts.tests.adversarial_runner.governance import (
    FRONTIER_ATTACK_GENERATION_CONTRACT,
    FRONTIER_ATTACK_GENERATION_CONTRACT_PATH,
    FRONTIER_ATTACK_GENERATION_CONTRACT_SHA256,
    sanitize_frontier_payload,
    validate_frontier_payload_schema,
)
from scripts.tests.adversarial_runner.evidence import (
    annotate_coverage_checks_with_threshold_source,
    build_coverage_checks,
    build_coverage_depth_checks,
    build_scenario_execution_evidence,
    compute_coverage_deltas,
    compute_reason_count_deltas,
    derive_coverage_deltas_from_simulation_event_reasons,
    extract_monitoring_snapshot as extract_monitoring_snapshot_impl,
    build_runtime_telemetry_evidence_checks as build_runtime_telemetry_evidence_checks_impl,
    build_browser_execution_evidence_checks as build_browser_execution_evidence_checks_impl,
    build_scenario_intent_checks as build_scenario_intent_checks_impl,
    profile_expected_defense_categories as profile_expected_defense_categories_impl,
    build_defense_noop_checks as build_defense_noop_checks_impl,
)
from scripts.tests.adversarial_runner.execution import (
    build_realism_checks,
    clamp_int_env as clamp_int_env_impl,
    compute_cohort_metrics as compute_cohort_metrics_impl,
    compute_realism_metrics as compute_realism_metrics_impl,
    effective_scenario_latency_ms as effective_scenario_latency_ms_impl,
    normalize_execution_lane as normalize_execution_lane_impl,
    normalize_request_path as normalize_request_path_impl,
    percentile,
    retry_strategy_max_attempts,
    round_robin_sequence_violations,
    scenario_max_latency_ms,
    state_mode_bucket,
    validate_execution_lane as validate_execution_lane_impl,
    enforce_attacker_request_contract as enforce_attacker_request_contract_impl,
    scenario_persona as scenario_persona_impl,
)
from scripts.tests.adversarial_runner.shared import dict_or_empty, int_or_zero, list_or_empty

BROWSER_DRIVER_SCRIPT_PATH = Path("scripts/tests/adversarial_browser_driver.mjs")
TELEMETRY_CLEANUP_ACK_HEADER = "X-Shuma-Telemetry-Cleanup-Ack"
TELEMETRY_CLEANUP_ACK_VALUE = "I_UNDERSTAND_TELEMETRY_CLEANUP"


ALLOWED_OUTCOMES = {"allow", "monitor", "not-a-bot", "challenge", "maze", "tarpit", "deny_temp"}
ALLOWED_TIERS = {"SIM-T0", "SIM-T1", "SIM-T2", "SIM-T3", "SIM-T4"}
ALLOWED_DRIVERS = {
    "allow_browser_allowlist",
    "not_a_bot_pass",
    "challenge_puzzle_fail_maze",
    "pow_success",
    "pow_invalid_proof",
    "rate_limit_enforce",
    "retry_storm_enforce",
    "geo_challenge",
    "geo_maze",
    "geo_block",
    "honeypot_deny_temp",
    "not_a_bot_replay_abuse",
    "not_a_bot_stale_token_abuse",
    "not_a_bot_ordering_cadence_abuse",
    "not_a_bot_replay_tarpit_abuse",
    "fingerprint_inconsistent_payload",
    "header_spoofing_probe",
    "cdp_high_confidence_deny",
    "akamai_additive_report",
    "akamai_authoritative_deny",
}
DRIVER_CLASS_HANDLERS = {
    "browser_realistic": {
        "allow_browser_allowlist": "driver_allow_browser_allowlist",
        "not_a_bot_pass": "driver_not_a_bot_pass",
        "challenge_puzzle_fail_maze": "driver_challenge_puzzle_fail_maze",
        "geo_challenge": "driver_geo_challenge",
        "geo_maze": "driver_geo_maze",
        "geo_block": "driver_geo_block",
        "honeypot_deny_temp": "driver_honeypot_deny_temp",
        "header_spoofing_probe": "driver_header_spoofing_probe",
    },
    "http_scraper": {
        "rate_limit_enforce": "driver_rate_limit_enforce",
        "retry_storm_enforce": "driver_retry_storm_enforce",
        "not_a_bot_replay_abuse": "driver_not_a_bot_replay_abuse",
        "not_a_bot_stale_token_abuse": "driver_not_a_bot_stale_token_abuse",
        "not_a_bot_ordering_cadence_abuse": "driver_not_a_bot_ordering_cadence_abuse",
        "not_a_bot_replay_tarpit_abuse": "driver_not_a_bot_replay_tarpit_abuse",
        "fingerprint_inconsistent_payload": "driver_fingerprint_inconsistent_payload",
        "cdp_high_confidence_deny": "driver_cdp_high_confidence_deny",
    },
    "edge_fixture": {
        "akamai_additive_report": "driver_akamai_additive_report",
        "akamai_authoritative_deny": "driver_akamai_authoritative_deny",
    },
    "cost_imposition": {
        "pow_success": "driver_pow_success",
        "pow_invalid_proof": "driver_pow_invalid_proof",
    },
}
ALLOWED_DRIVER_CLASSES = set(DRIVER_CLASS_HANDLERS.keys())
DRIVER_TO_CLASS = {
    driver_name: driver_class
    for driver_class, family_handlers in DRIVER_CLASS_HANDLERS.items()
    for driver_name in family_handlers.keys()
}

missing_driver_contract_rows = sorted(ALLOWED_DRIVERS - set(DETERMINISTIC_DRIVER_DEFINITIONS.keys()))
if missing_driver_contract_rows:
    raise RuntimeError(
        "deterministic attack corpus missing ci_oracle driver rows: "
        + ", ".join(missing_driver_contract_rows)
    )
extra_driver_contract_rows = sorted(set(DETERMINISTIC_DRIVER_DEFINITIONS.keys()) - ALLOWED_DRIVERS)
if extra_driver_contract_rows:
    raise RuntimeError(
        "deterministic attack corpus has unknown ci_oracle driver rows: "
        + ", ".join(extra_driver_contract_rows)
    )
driver_class_drift_rows = sorted(
    driver_name
    for driver_name in ALLOWED_DRIVERS
    if str(DRIVER_TO_CLASS.get(driver_name) or "").strip()
    != str(DETERMINISTIC_DRIVER_CLASS_MAP.get(driver_name) or "").strip()
)
if driver_class_drift_rows:
    raise RuntimeError(
        "deterministic attack corpus driver_class drift from runner handlers for: "
        + ", ".join(driver_class_drift_rows)
    )
SUPPORTED_EXECUTION_LANES = {"black_box"}
FULL_COVERAGE_PROFILE_NAME = "full_coverage"
SUITE_PHASE_SETUP = "suite_setup"
SUITE_PHASE_ATTACKER_EXECUTION = "attacker_execution"
SUITE_PHASE_TEARDOWN = "suite_teardown"
ALLOWED_SUITE_PHASES = {
    SUITE_PHASE_SETUP,
    SUITE_PHASE_ATTACKER_EXECUTION,
    SUITE_PHASE_TEARDOWN,
}
ALLOWED_COVERAGE_REQUIREMENTS = {
    "honeypot_hits",
    "challenge_failures",
    "not_a_bot_pass",
    "not_a_bot_fail",
    "not_a_bot_replay",
    "not_a_bot_escalate",
    "pow_attempts",
    "pow_successes",
    "pow_failures",
    "rate_violations",
    "rate_limited",
    "rate_banned",
    "geo_violations",
    "geo_challenge",
    "geo_maze",
    "geo_block",
    "maze_hits",
    "tarpit_activations_progressive",
    "tarpit_progress_advanced",
    "cdp_detections",
    "fingerprint_events",
    "ban_count",
    "recent_event_count",
}
DEFENSE_NOOP_COVERAGE_KEYS: Dict[str, Tuple[str, ...]] = {
    "pow": ("pow_attempts", "pow_successes", "pow_failures"),
    "challenge": ("challenge_failures",),
    "maze": ("maze_hits",),
    "honeypot": ("honeypot_hits",),
    "cdp": ("cdp_detections",),
    "rate_limit": ("rate_violations", "rate_limited", "rate_banned"),
    "geo": ("geo_violations", "geo_challenge", "geo_maze", "geo_block"),
}

COVERAGE_CONTRACT_CANDIDATE_PATHS = (
    Path("scripts/tests/adversarial/coverage_contract.v2.json"),
    Path("scripts/tests/adversarial/coverage_contract.v1.json"),
)
REAL_TRAFFIC_CONTRACT_PATH = Path("scripts/tests/adversarial/real_traffic_contract.v1.json")
SCENARIO_INTENT_MATRIX_PATH = Path("scripts/tests/adversarial/scenario_intent_matrix.v1.json")


def resolve_coverage_contract_path() -> Path:
    for candidate in COVERAGE_CONTRACT_CANDIDATE_PATHS:
        if candidate.exists():
            return candidate
    raise RuntimeError(
        "coverage contract not found: expected one of "
        + ", ".join(str(path) for path in COVERAGE_CONTRACT_CANDIDATE_PATHS)
    )


def validate_coverage_depth_requirements(
    payload: Dict[str, Any], *, path: Path, schema_version: str
) -> Dict[str, Dict[str, Any]]:
    if schema_version != "sim-coverage-contract.v2":
        return {}
    depth_requirements = payload.get("coverage_depth_requirements")
    if not isinstance(depth_requirements, dict) or not depth_requirements:
        raise RuntimeError(
            "coverage contract v2 coverage_depth_requirements must be a non-empty object"
        )
    normalized: Dict[str, Dict[str, Any]] = {}
    for row_id, row_payload in depth_requirements.items():
        row_name = str(row_id or "").strip()
        if not row_name:
            raise RuntimeError("coverage contract v2 coverage_depth_requirements contains empty row id")
        row = row_payload if isinstance(row_payload, dict) else {}
        required_metrics = row.get("required_metrics")
        if not isinstance(required_metrics, dict) or not required_metrics:
            raise RuntimeError(
                f"coverage contract v2 row {row_name} required_metrics must be a non-empty object"
            )
        for metric_key, minimum in required_metrics.items():
            key = str(metric_key or "").strip()
            if key not in ALLOWED_COVERAGE_REQUIREMENTS:
                raise RuntimeError(
                    f"coverage contract v2 row {row_name} has unsupported metric key: {key}"
                )
            if isinstance(minimum, bool) or not isinstance(minimum, int) or int(minimum) < 0:
                raise RuntimeError(
                    f"coverage contract v2 row {row_name} metric {key} minimum must be integer >= 0"
                )
        required_scenarios = row.get("required_scenarios")
        if not isinstance(required_scenarios, list) or not required_scenarios:
            raise RuntimeError(
                f"coverage contract v2 row {row_name} required_scenarios must be a non-empty array"
            )
        for scenario_id in required_scenarios:
            if not str(scenario_id or "").strip():
                raise RuntimeError(
                    f"coverage contract v2 row {row_name} required_scenarios must not contain empty values"
                )
        verification_row_id = str(row.get("verification_matrix_row_id") or "").strip()
        if not verification_row_id:
            raise RuntimeError(
                f"coverage contract v2 row {row_name} missing verification_matrix_row_id"
            )
        normalized[row_name] = {
            "plan_row": str(row.get("plan_row") or "").strip(),
            "verification_matrix_row_id": verification_row_id,
            "required_scenarios": [
                str(item).strip()
                for item in required_scenarios
                if str(item).strip()
            ],
            "required_metrics": {
                str(metric): int(value)
                for metric, value in required_metrics.items()
            },
            "required_evidence_types": [
                str(item).strip()
                for item in (
                    row.get("required_evidence_types")
                    if isinstance(row.get("required_evidence_types"), list)
                    else []
                )
                if str(item).strip()
            ],
        }
    return normalized


def load_coverage_contract(path: Optional[Path] = None) -> Tuple[Path, Dict[str, Any]]:
    contract_path = path or resolve_coverage_contract_path()
    if not contract_path.exists():
        raise RuntimeError(f"coverage contract not found: {contract_path}")
    try:
        payload = json.loads(contract_path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid coverage contract JSON: {contract_path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError(f"coverage contract must be a JSON object: {contract_path}")

    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version not in {"sim-coverage-contract.v1", "sim-coverage-contract.v2"}:
        raise RuntimeError(
            "coverage contract schema_version must be sim-coverage-contract.v1 or "
            f"sim-coverage-contract.v2 (got {schema_version})"
        )
    profile = str(payload.get("profile") or "").strip()
    if profile != FULL_COVERAGE_PROFILE_NAME:
        raise RuntimeError(
            f"coverage contract profile must be {FULL_COVERAGE_PROFILE_NAME} (got {profile})"
        )

    coverage_requirements = payload.get("coverage_requirements")
    if not isinstance(coverage_requirements, dict) or not coverage_requirements:
        raise RuntimeError("coverage contract coverage_requirements must be a non-empty object")
    for key, minimum in coverage_requirements.items():
        if key not in ALLOWED_COVERAGE_REQUIREMENTS:
            raise RuntimeError(f"coverage contract has unsupported coverage requirement key: {key}")
        if isinstance(minimum, bool) or not isinstance(minimum, int) or minimum < 0:
            raise RuntimeError(f"coverage contract key {key} must have integer minimum >= 0")

    required_event_reasons = payload.get("required_event_reasons")
    if not isinstance(required_event_reasons, list) or not required_event_reasons:
        raise RuntimeError("coverage contract required_event_reasons must be a non-empty array")
    for raw_reason in required_event_reasons:
        if not str(raw_reason or "").strip():
            raise RuntimeError("coverage contract required_event_reasons must not contain empty values")

    required_outcome_categories = payload.get("required_outcome_categories")
    if required_outcome_categories is not None:
        if not isinstance(required_outcome_categories, list):
            raise RuntimeError("coverage contract required_outcome_categories must be an array")
        for raw_outcome in required_outcome_categories:
            outcome = str(raw_outcome or "").strip()
            if outcome not in ALLOWED_OUTCOMES:
                raise RuntimeError(
                    f"coverage contract required_outcome_categories has unsupported value: {outcome}"
                )

    ip_range_required = payload.get("ip_range_suggestion_seed_required")
    if not isinstance(ip_range_required, bool):
        raise RuntimeError("coverage contract ip_range_suggestion_seed_required must be boolean")

    plan_rows = payload.get("plan_contract_rows")
    if not isinstance(plan_rows, list) or not plan_rows:
        raise RuntimeError("coverage contract plan_contract_rows must be a non-empty array")
    for row in plan_rows:
        if not str(row or "").strip():
            raise RuntimeError("coverage contract plan_contract_rows must not contain empty values")

    payload["coverage_depth_requirements"] = validate_coverage_depth_requirements(
        payload, path=contract_path, schema_version=schema_version
    )
    if schema_version == "sim-coverage-contract.v1":
        payload["coverage_depth_requirements"] = {}
    return contract_path, payload


COVERAGE_CONTRACT_PATH, COVERAGE_CONTRACT = load_coverage_contract()
COVERAGE_CONTRACT_SCHEMA_VERSION = str(COVERAGE_CONTRACT.get("schema_version") or "")
COVERAGE_CONTRACT_REQUIREMENTS = {
    str(key): int(value)
    for key, value in dict(COVERAGE_CONTRACT.get("coverage_requirements") or {}).items()
}
COVERAGE_CONTRACT_DEPTH_REQUIREMENTS = {
    str(row_id): (row if isinstance(row, dict) else {})
    for row_id, row in dict(COVERAGE_CONTRACT.get("coverage_depth_requirements") or {}).items()
}
COVERAGE_CONTRACT_REQUIRED_EVENT_REASONS = [
    str(item).strip().lower()
    for item in list(COVERAGE_CONTRACT.get("required_event_reasons") or [])
    if str(item).strip()
]
COVERAGE_CONTRACT_REQUIRED_OUTCOME_CATEGORIES = [
    str(item).strip()
    for item in list(COVERAGE_CONTRACT.get("required_outcome_categories") or [])
    if str(item).strip()
]
COVERAGE_CONTRACT_IP_RANGE_SUGGESTION_SEED_REQUIRED = bool(
    COVERAGE_CONTRACT.get("ip_range_suggestion_seed_required")
)
COVERAGE_CONTRACT_PLAN_ROWS = [
    str(item).strip()
    for item in list(COVERAGE_CONTRACT.get("plan_contract_rows") or [])
    if str(item).strip()
]
COVERAGE_CONTRACT_SHA256 = hashlib.sha256(
    json.dumps(COVERAGE_CONTRACT, sort_keys=True, separators=(",", ":")).encode("utf-8")
).hexdigest()

SCENARIO_INTENT_SIGNAL_TYPES = {
    "coverage_delta",
    "outcome_equals",
    "simulation_event_count_delta",
    "simulation_event_reason_prefix",
    "realism_metric_min",
}
SCENARIO_INTENT_REALISM_METRICS = {
    "request_sequence_count",
    "retry_attempts",
    "think_time_events",
    "attempts_total",
}
SCENARIO_INTENT_PROGRESS_INT_KEYS = {
    "min_request_sequence_count",
    "min_retry_attempts",
    "min_think_time_events",
    "min_attempts_total",
}
SCENARIO_INTENT_PROGRESS_STR_KEYS = {
    "required_driver_class",
    "required_persona",
    "required_retry_strategy",
}


def load_scenario_intent_matrix(path: Path = SCENARIO_INTENT_MATRIX_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"scenario intent matrix not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid scenario intent matrix JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError(f"scenario intent matrix must be a JSON object: {path}")

    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "sim-scenario-intent-matrix.v1":
        raise RuntimeError(
            "scenario intent matrix schema_version must be sim-scenario-intent-matrix.v1 "
            f"(got {schema_version})"
        )
    manifest_schema_version = str(payload.get("manifest_schema_version") or "").strip()
    if manifest_schema_version != "sim-manifest.v2":
        raise RuntimeError(
            "scenario intent matrix manifest_schema_version must be sim-manifest.v2 "
            f"(got {manifest_schema_version})"
        )
    suite_id = str(payload.get("suite_id") or "").strip()
    if not suite_id:
        raise RuntimeError("scenario intent matrix suite_id must be non-empty")

    review_governance = payload.get("review_governance")
    if not isinstance(review_governance, dict):
        raise RuntimeError("scenario intent matrix review_governance must be an object")
    cadence_days = review_governance.get("cadence_days")
    stale_after_days = review_governance.get("stale_after_days")
    if isinstance(cadence_days, bool) or not isinstance(cadence_days, int) or cadence_days < 1:
        raise RuntimeError("scenario intent matrix review_governance.cadence_days must be integer >= 1")
    if (
        isinstance(stale_after_days, bool)
        or not isinstance(stale_after_days, int)
        or stale_after_days < cadence_days
    ):
        raise RuntimeError(
            "scenario intent matrix review_governance.stale_after_days must be integer >= cadence_days"
        )
    if not str(review_governance.get("owner") or "").strip():
        raise RuntimeError("scenario intent matrix review_governance.owner must be non-empty")

    rows = payload.get("rows")
    if not isinstance(rows, list) or not rows:
        raise RuntimeError("scenario intent matrix rows must be a non-empty array")

    seen_ids = set()
    for row in rows:
        if not isinstance(row, dict):
            raise RuntimeError("scenario intent matrix rows must contain objects")
        scenario_id = str(row.get("scenario_id") or "").strip()
        if not scenario_id:
            raise RuntimeError("scenario intent matrix row missing scenario_id")
        if scenario_id in seen_ids:
            raise RuntimeError(f"scenario intent matrix duplicate scenario_id: {scenario_id}")
        seen_ids.add(scenario_id)

        minimum_runtime_requests = row.get("minimum_runtime_requests")
        if (
            isinstance(minimum_runtime_requests, bool)
            or not isinstance(minimum_runtime_requests, int)
            or minimum_runtime_requests < 1
        ):
            raise RuntimeError(
                f"scenario intent matrix row {scenario_id} minimum_runtime_requests must be integer >= 1"
            )

        categories = row.get("required_defense_categories")
        if not isinstance(categories, list) or not categories:
            raise RuntimeError(
                f"scenario intent matrix row {scenario_id} required_defense_categories must be non-empty array"
            )
        normalized_categories = []
        for category in categories:
            normalized = str(category or "").strip()
            if not normalized:
                raise RuntimeError(
                    f"scenario intent matrix row {scenario_id} has empty required_defense_categories entry"
                )
            normalized_categories.append(normalized)
        if len(set(normalized_categories)) != len(normalized_categories):
            raise RuntimeError(
                f"scenario intent matrix row {scenario_id} required_defense_categories must be unique"
            )

        defense_signals = row.get("defense_signals")
        if not isinstance(defense_signals, dict) or not defense_signals:
            raise RuntimeError(
                f"scenario intent matrix row {scenario_id} defense_signals must be a non-empty object"
            )
        for category in normalized_categories:
            if category not in defense_signals:
                raise RuntimeError(
                    f"scenario intent matrix row {scenario_id} missing defense_signals entry for category={category}"
                )
        for category, signal_rules in defense_signals.items():
            normalized_category = str(category or "").strip()
            if normalized_category not in normalized_categories:
                raise RuntimeError(
                    "scenario intent matrix row "
                    f"{scenario_id} defense_signals contains category not declared in required_defense_categories: "
                    f"{normalized_category}"
                )
            if not isinstance(signal_rules, list) or not signal_rules:
                raise RuntimeError(
                    f"scenario intent matrix row {scenario_id} category={normalized_category} signals must be non-empty array"
                )
            for signal in signal_rules:
                if not isinstance(signal, dict):
                    raise RuntimeError(
                        "scenario intent matrix row "
                        f"{scenario_id} category={normalized_category} signal entries must be objects"
                    )
                signal_type = str(signal.get("type") or "").strip()
                if signal_type not in SCENARIO_INTENT_SIGNAL_TYPES:
                    raise RuntimeError(
                        f"scenario intent matrix row {scenario_id} category={normalized_category} "
                        f"has unsupported signal type: {signal_type}"
                    )
                minimum = signal.get("minimum", 1)
                if isinstance(minimum, bool) or not isinstance(minimum, int) or minimum < 1:
                    raise RuntimeError(
                        "scenario intent matrix row "
                        f"{scenario_id} category={normalized_category} signal minimum must be integer >= 1"
                    )
                if signal_type == "coverage_delta":
                    key = str(signal.get("key") or "").strip()
                    if key not in ALLOWED_COVERAGE_REQUIREMENTS:
                        raise RuntimeError(
                            f"scenario intent matrix row {scenario_id} category={normalized_category} "
                            f"coverage signal key unsupported: {key}"
                        )
                if signal_type == "outcome_equals":
                    value = str(signal.get("value") or "").strip()
                    if value not in ALLOWED_OUTCOMES:
                        raise RuntimeError(
                            f"scenario intent matrix row {scenario_id} category={normalized_category} "
                            f"outcome_equals value unsupported: {value}"
                        )
                if signal_type == "simulation_event_reason_prefix":
                    prefix = str(signal.get("prefix") or "").strip().lower()
                    if not prefix:
                        raise RuntimeError(
                            "scenario intent matrix row "
                            f"{scenario_id} category={normalized_category} event reason prefix must be non-empty"
                        )
                if signal_type == "realism_metric_min":
                    metric = str(signal.get("key") or "").strip()
                    if metric not in SCENARIO_INTENT_REALISM_METRICS:
                        raise RuntimeError(
                            f"scenario intent matrix row {scenario_id} category={normalized_category} "
                            f"realism metric unsupported: {metric}"
                        )

        progression = row.get("progression_requirements")
        if progression is not None:
            if not isinstance(progression, dict):
                raise RuntimeError(
                    f"scenario intent matrix row {scenario_id} progression_requirements must be object when provided"
                )
            for key, value in progression.items():
                if key in SCENARIO_INTENT_PROGRESS_INT_KEYS:
                    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                        raise RuntimeError(
                            f"scenario intent matrix row {scenario_id} progression_requirements.{key} "
                            "must be integer >= 0"
                        )
                    continue
                if key in SCENARIO_INTENT_PROGRESS_STR_KEYS:
                    if not str(value or "").strip():
                        raise RuntimeError(
                            f"scenario intent matrix row {scenario_id} progression_requirements.{key} "
                            "must be non-empty string"
                        )
                    continue
                raise RuntimeError(
                    f"scenario intent matrix row {scenario_id} progression_requirements has unsupported key: {key}"
                )

        review = row.get("review")
        if not isinstance(review, dict):
            raise RuntimeError(f"scenario intent matrix row {scenario_id} review must be an object")
        for key in ("status", "owner", "last_reviewed_on"):
            if not str(review.get(key) or "").strip():
                raise RuntimeError(
                    f"scenario intent matrix row {scenario_id} review.{key} must be non-empty"
                )

    return payload


SCENARIO_INTENT_MATRIX = load_scenario_intent_matrix()
SCENARIO_INTENT_MATRIX_SCHEMA_VERSION = str(SCENARIO_INTENT_MATRIX.get("schema_version") or "")
SCENARIO_INTENT_MATRIX_SUITE_ID = str(SCENARIO_INTENT_MATRIX.get("suite_id") or "").strip()
SCENARIO_INTENT_MATRIX_MANIFEST_PATH = Path(
    str(SCENARIO_INTENT_MATRIX.get("manifest_path") or "scripts/tests/adversarial/scenario_manifest.v2.json")
)
SCENARIO_INTENT_MATRIX_ROWS = [
    row for row in SCENARIO_INTENT_MATRIX.get("rows", []) if isinstance(row, dict)
]
SCENARIO_INTENT_MATRIX_ROWS_BY_ID = {
    str(row.get("scenario_id") or "").strip(): row
    for row in SCENARIO_INTENT_MATRIX_ROWS
    if str(row.get("scenario_id") or "").strip()
}
SCENARIO_INTENT_MATRIX_SHA256 = hashlib.sha256(
    json.dumps(SCENARIO_INTENT_MATRIX, sort_keys=True, separators=(",", ":")).encode("utf-8")
).hexdigest()


def load_real_traffic_contract(path: Path = REAL_TRAFFIC_CONTRACT_PATH) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"real traffic contract not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid real traffic contract JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError(f"real traffic contract must be a JSON object: {path}")

    schema_version = str(payload.get("schema_version") or "").strip()
    if schema_version != "sim-real-traffic-contract.v1":
        raise RuntimeError(
            "real traffic contract schema_version must be sim-real-traffic-contract.v1 "
            f"(got {schema_version})"
        )

    profile = str(payload.get("profile") or "").strip()
    if profile != FULL_COVERAGE_PROFILE_NAME:
        raise RuntimeError(
            f"real traffic contract profile must be {FULL_COVERAGE_PROFILE_NAME} (got {profile})"
        )

    required_invariants = payload.get("required_invariants")
    if not isinstance(required_invariants, list) or not required_invariants:
        raise RuntimeError("real traffic contract required_invariants must be a non-empty array")
    for invariant in required_invariants:
        if not str(invariant or "").strip():
            raise RuntimeError("real traffic contract required_invariants must not contain empty values")

    prohibited_patterns = payload.get("prohibited_patterns")
    if not isinstance(prohibited_patterns, list) or not prohibited_patterns:
        raise RuntimeError("real traffic contract prohibited_patterns must be a non-empty array")
    for pattern in prohibited_patterns:
        if not str(pattern or "").strip():
            raise RuntimeError("real traffic contract prohibited_patterns must not contain empty values")

    evidence_schema = payload.get("evidence_schema")
    if not isinstance(evidence_schema, dict):
        raise RuntimeError("real traffic contract evidence_schema must be an object")

    scenario_required_fields = evidence_schema.get("scenario_required_fields")
    if not isinstance(scenario_required_fields, list) or not scenario_required_fields:
        raise RuntimeError(
            "real traffic contract evidence_schema.scenario_required_fields must be a non-empty array"
        )
    for field in scenario_required_fields:
        if not str(field or "").strip():
            raise RuntimeError(
                "real traffic contract evidence_schema.scenario_required_fields must not contain empty values"
            )

    control_lineage_required_fields = evidence_schema.get("control_lineage_required_fields")
    if not isinstance(control_lineage_required_fields, list) or not control_lineage_required_fields:
        raise RuntimeError(
            "real traffic contract evidence_schema.control_lineage_required_fields must be a non-empty array"
        )
    for field in control_lineage_required_fields:
        if not str(field or "").strip():
            raise RuntimeError(
                "real traffic contract evidence_schema.control_lineage_required_fields must not contain empty values"
            )

    return payload


REAL_TRAFFIC_CONTRACT = load_real_traffic_contract()
REAL_TRAFFIC_CONTRACT_SCHEMA_VERSION = str(REAL_TRAFFIC_CONTRACT.get("schema_version") or "")
REAL_TRAFFIC_CONTRACT_REQUIRED_INVARIANTS = [
    str(item).strip()
    for item in list(REAL_TRAFFIC_CONTRACT.get("required_invariants") or [])
    if str(item).strip()
]
REAL_TRAFFIC_CONTRACT_PROHIBITED_PATTERNS = [
    str(item).strip()
    for item in list(REAL_TRAFFIC_CONTRACT.get("prohibited_patterns") or [])
    if str(item).strip()
]
REAL_TRAFFIC_CONTRACT_REQUIRED_SCENARIO_FIELDS = [
    str(item).strip()
    for item in list(
        dict(REAL_TRAFFIC_CONTRACT.get("evidence_schema") or {}).get("scenario_required_fields") or []
    )
    if str(item).strip()
]
REAL_TRAFFIC_CONTRACT_REQUIRED_CONTROL_LINEAGE_FIELDS = [
    str(item).strip()
    for item in list(
        dict(REAL_TRAFFIC_CONTRACT.get("evidence_schema") or {}).get("control_lineage_required_fields")
        or []
    )
    if str(item).strip()
]
REAL_TRAFFIC_CONTRACT_SHA256 = hashlib.sha256(
    json.dumps(REAL_TRAFFIC_CONTRACT, sort_keys=True, separators=(",", ":")).encode("utf-8")
).hexdigest()
SUPPORTED_MANIFEST_SCHEMA_VERSIONS = {"sim-manifest.v1", "sim-manifest.v2"}
ALLOWED_REQUEST_PLANES = {"attacker", "control"}
ALLOWED_TRAFFIC_PERSONAS = {
    "human_like",
    "benign_automation",
    "suspicious_automation",
    "adversarial",
}
ALLOWED_PERSONA_SCHEDULERS = {"round_robin"}
ALLOWED_RETRY_STRATEGIES = {"single_attempt", "bounded_backoff", "retry_storm"}
ALLOWED_COOKIE_BEHAVIORS = {"stateful_cookie_jar", "stateless", "cookie_reset_each_request"}
ALLOWED_DEFENSE_CATEGORIES = {
    "allowlist",
    "not_a_bot",
    "challenge",
    "pow",
    "rate_limit",
    "geo",
    "maze",
    "tarpit",
    "honeypot",
    "cdp",
    "fingerprint",
    "ban_path",
    "event_stream",
}
GOOD_NOT_A_BOT_TELEMETRY = {
    "has_pointer": True,
    "pointer_move_count": 42,
    "pointer_path_length": 560.0,
    "pointer_direction_changes": 18,
    "down_up_ms": 220,
    "focus_changes": 1,
    "visibility_changes": 0,
    "interaction_elapsed_ms": 1800,
    "keyboard_used": False,
    "touch_used": False,
    "events_order_valid": True,
    "activation_method": "pointer",
    "activation_trusted": True,
    "activation_count": 1,
    "control_focused": True,
}

BAD_ORDERING_NOT_A_BOT_TELEMETRY = {
    "has_pointer": True,
    "pointer_move_count": 6,
    "pointer_path_length": 35.0,
    "pointer_direction_changes": 1,
    "down_up_ms": 15,
    "focus_changes": 0,
    "visibility_changes": 0,
    "interaction_elapsed_ms": 120,
    "keyboard_used": False,
    "touch_used": False,
    "events_order_valid": False,
    "activation_method": "pointer",
    "activation_trusted": False,
    "activation_count": 4,
    "control_focused": False,
}


def sign_sim_tag(
    secret: str, run_id: str, profile: str, lane: str, timestamp: str, nonce: str
) -> str:
    message = build_sim_tag_canonical_message(
        run_id=run_id,
        profile=profile,
        lane=lane,
        timestamp=timestamp,
        nonce=nonce,
    )
    return hmac.new(
        str(secret).encode("utf-8"),
        message.encode("utf-8"),
        hashlib.sha256,
    ).hexdigest()


class Runner:
    def __init__(
        self,
        manifest_path: Path,
        manifest: Dict[str, Any],
        profile_name: str,
        execution_lane: str,
        base_url: str,
        request_timeout_seconds: float,
        report_path: Path,
    ):
        self.manifest_path = manifest_path
        self.manifest = manifest
        self.profile_name = profile_name
        self.execution_lane = validate_execution_lane(execution_lane)
        self.profile = manifest["profiles"][profile_name]
        self.base_url = base_url.rstrip("/")
        self.request_timeout_seconds = request_timeout_seconds
        self.report_path = report_path
        self.opener = urllib.request.build_opener(NoRedirectHandler())
        self.request_count = 0
        self.forwarded_secret = env_or_local("SHUMA_FORWARDED_IP_SECRET")
        self.health_secret = env_or_local("SHUMA_HEALTH_SECRET")
        self.api_key = env_or_local("SHUMA_API_KEY")
        self.sim_telemetry_secret = env_or_local("SHUMA_SIM_TELEMETRY_SECRET")
        self.session_nonce = f"{int(time.time())}-{os.getpid()}"
        control_ip_hash = hashlib.sha256(self.session_nonce.encode("utf-8")).hexdigest()
        self.control_plane_ip_seed_third_octet = (int(control_ip_hash[:2], 16) % 254) + 1
        self.control_plane_ip_seed_fourth_octet = (int(control_ip_hash[2:4], 16) % 254) + 1
        self.control_plane_request_counter = 0
        self.control_plane_ip = (
            f"127.0.{self.control_plane_ip_seed_third_octet}.{self.control_plane_ip_seed_fourth_octet}"
        )
        self.sim_run_id = f"deterministic-{self.session_nonce}"
        self.sim_profile = profile_name
        self.sim_lane = f"deterministic_{self.execution_lane}"
        self.sim_tag_nonce_counter = 0
        self.attacker_client = AttackerPlaneClient(self)
        self.control_client = ControlPlaneClient(self)
        self.honeypot_path = "/instaban"
        self.preserve_state = truthy_env("SHUMA_ADVERSARIAL_PRESERVE_STATE")
        self.rotate_ips = truthy_env("SHUMA_ADVERSARIAL_ROTATE_IPS")
        self.ip_range_seed_prefix = "10.222.77."
        self.ip_range_seed_ips = [f"{self.ip_range_seed_prefix}{octet}" for octet in range(10, 60)]
        self._active_execution_state: Optional[Dict[str, Any]] = None
        self.execution_phase = SUITE_PHASE_SETUP
        self.execution_phase_trace: List[Dict[str, Any]] = [
            {
                "phase": SUITE_PHASE_SETUP,
                "reason": "runner_start",
                "at_unix": int(time.time()),
            }
        ]
        self.control_plane_mutations: List[Dict[str, Any]] = []
        self._scenario_attacker_phase_started = False
        self._scenario_setup_context: Dict[str, Dict[str, Any]] = {}
        realism_gates = dict_or_empty((self.profile.get("gates") or {}).get("realism"))
        self.realism_policy_enabled = bool(realism_gates.get("enabled", True))
        browser_driver_enabled_raw = os.environ.get("SHUMA_ADVERSARIAL_BROWSER_DRIVER_ENABLED")
        if browser_driver_enabled_raw is None:
            self.browser_driver_enabled = True
        else:
            self.browser_driver_enabled = browser_driver_enabled_raw.strip().lower() in {
                "1",
                "true",
                "yes",
                "on",
            }
        self.browser_driver_script_path = BROWSER_DRIVER_SCRIPT_PATH
        self.browser_driver_command = [
            "corepack",
            "pnpm",
            "exec",
            "node",
            str(self.browser_driver_script_path),
        ]
        self.browser_driver_env = dict(os.environ)
        self.browser_driver_max_attempts = clamp_int_env(
            "SHUMA_ADVERSARIAL_BROWSER_RETRIES",
            minimum=1,
            maximum=3,
            fallback=2,
        )
        self.browser_driver_timeout_ms = clamp_int_env(
            "SHUMA_ADVERSARIAL_BROWSER_TIMEOUT_MS",
            minimum=2000,
            maximum=60000,
            fallback=15000,
        )
        self.browser_driver_settle_ms = clamp_int_env(
            "SHUMA_ADVERSARIAL_BROWSER_SETTLE_MS",
            minimum=0,
            maximum=5000,
            fallback=200,
        )
        self.browser_driver_retryable_error_codes = {
            "timeout",
            "network_failure",
            "sandbox_launch_failure",
        }

        if not self.api_key:
            raise SimulationError(
                "Missing SHUMA_API_KEY. Run make setup (or export SHUMA_API_KEY) before adversarial tests."
            )
        if self.api_key in {
            "changeme-dev-only-api-key",
            "changeme-supersecret",
            "changeme-prod-api-key",
        }:
            raise SimulationError(
                "SHUMA_API_KEY is a placeholder. Run make setup or make api-key-generate first."
            )
        if not self.sim_telemetry_secret:
            raise SimulationError(
                "[sim-preflight][missing_secret] SHUMA_SIM_TELEMETRY_SECRET is missing. "
                "Run make test-adversarial-preflight (or make setup) before adversarial tests."
            )

        self.scenarios = scenario_map(self.manifest)
        selected = [self.scenarios[sid] for sid in self.profile["scenario_ids"]]
        self.selected_scenarios = self.apply_persona_scheduler(selected)
        self.scenario_ips = self.build_scenario_ip_map()
        profile_has_browser_realistic = any(
            scenario_driver_class(scenario) == "browser_realistic"
            for scenario in self.selected_scenarios
        )
        if profile_has_browser_realistic and not self.browser_driver_script_path.exists():
            raise SimulationError(
                "browser-realistic profile requires browser driver script "
                f"{self.browser_driver_script_path}, but it was not found."
            )
        if profile_has_browser_realistic:
            try:
                playwright_status = ensure_playwright_chromium()
            except RuntimeError as exc:
                raise SimulationError(
                    "browser-realistic profile requires a Playwright Chromium runtime. "
                    "Run make test-adversarial-preflight (or make setup) before adversarial tests. "
                    f"Detail: {exc}"
                ) from exc
            self.browser_driver_env = build_playwright_env(
                browser_cache=Path(playwright_status.browser_cache)
            )

    def next_control_plane_ip(self) -> str:
        self.control_plane_request_counter += 1
        next_third_octet = (
            (self.control_plane_ip_seed_third_octet + self.control_plane_request_counter - 1) % 254
        ) + 1
        return f"127.0.{next_third_octet}.{self.control_plane_ip_seed_fourth_octet}"

    def build_scenario_ip_map(self) -> Dict[str, str]:
        mapping: Dict[str, str] = {}
        if not self.rotate_ips:
            for scenario in self.selected_scenarios:
                mapping[scenario["id"]] = str(scenario.get("ip") or "").strip()
            return mapping

        # Assign scenario-specific subnets with per-run rotated host octets.
        # This avoids cross-scenario ban collisions in preserve-state live mode.
        salt = int(hashlib.sha256(self.session_nonce.encode("utf-8")).hexdigest()[:8], 16)
        for index, scenario in enumerate(self.selected_scenarios):
            third_octet = ((index + 17) % 254) + 1
            last_octet = ((salt + index * 29) % 254) + 1
            mapping[scenario["id"]] = f"10.240.{third_octet}.{last_octet}"
        return mapping

    def scenario_ip(self, scenario: Dict[str, Any]) -> str:
        scenario_id = str(scenario.get("id") or "")
        return self.scenario_ips.get(scenario_id, str(scenario.get("ip") or "").strip())

    def apply_persona_scheduler(self, scenarios: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        scheduler = str(((self.profile.get("gates") or {}).get("persona_scheduler") or "")).strip().lower()
        if scheduler != "round_robin":
            return list(scenarios)

        persona_order: List[str] = []
        buckets: Dict[str, List[Dict[str, Any]]] = {}
        for scenario in scenarios:
            persona = scenario_persona(scenario)
            if persona not in buckets:
                buckets[persona] = []
                persona_order.append(persona)
            buckets[persona].append(scenario)

        scheduled: List[Dict[str, Any]] = []
        while True:
            progressed = False
            for persona in persona_order:
                queue = buckets.get(persona, [])
                if not queue:
                    continue
                scheduled.append(queue.pop(0))
                progressed = True
            if not progressed:
                break
        return scheduled

    def set_execution_phase(self, phase: str, reason: str) -> None:
        normalized = str(phase or "").strip().lower()
        if normalized not in ALLOWED_SUITE_PHASES:
            raise SimulationError(f"unsupported execution phase: {phase}")
        transition_reason = str(reason or "").strip() or "unspecified"
        if self.execution_phase == normalized:
            return
        self.execution_phase = normalized
        if normalized == SUITE_PHASE_ATTACKER_EXECUTION:
            self._scenario_attacker_phase_started = True
        self.execution_phase_trace.append(
            {
                "phase": normalized,
                "reason": transition_reason,
                "at_unix": int(time.time()),
            }
        )

    def record_control_plane_mutation(
        self, action: str, reason: str, details: Optional[Dict[str, Any]] = None
    ) -> None:
        entry = {
            "action": str(action or "").strip(),
            "phase": self.execution_phase,
            "reason": str(reason or "").strip() or "unspecified",
            "at_unix": int(time.time()),
        }
        if details:
            entry["details"] = details
        self.control_plane_mutations.append(entry)

    def assert_control_plane_mutation_allowed(self, action: str, reason: str) -> None:
        if self.execution_phase == SUITE_PHASE_ATTACKER_EXECUTION:
            raise SimulationError(
                "control-plane mutation forbidden during attacker_execution "
                f"(action={action} reason={reason})"
            )
        if (
            self.execution_phase == SUITE_PHASE_SETUP
            and self._scenario_attacker_phase_started
        ):
            raise SimulationError(
                "control-plane mutation forbidden after attacker_execution has started "
                f"(action={action} reason={reason})"
            )

    def summarize_control_plane_mutations(self) -> Dict[str, Any]:
        by_phase: Dict[str, int] = {}
        for mutation in self.control_plane_mutations:
            phase = str(dict_or_empty(mutation).get("phase") or "").strip().lower()
            if not phase:
                continue
            by_phase[phase] = by_phase.get(phase, 0) + 1
        return {
            "count": len(self.control_plane_mutations),
            "phases": sorted(by_phase.keys()),
            "count_by_phase": by_phase,
            "entries": list(self.control_plane_mutations),
        }

    def run(self) -> int:
        self.set_execution_phase(SUITE_PHASE_SETUP, "suite_setup_start")
        self.wait_ready(timeout_seconds=30)
        self.reset_baseline_config()
        self.honeypot_path = self.resolve_honeypot_path()

        cleanup_candidate_ips = sorted(
            [self.scenario_ip(scenario) for scenario in self.selected_scenarios if self.scenario_ip(scenario)]
        )
        if self.profile_name == "full_coverage":
            cleanup_candidate_ips.extend(self.ip_range_seed_ips)
            cleanup_candidate_ips = sorted(set(cleanup_candidate_ips))
        if not self.preserve_state:
            # Untrusted forwarded-header probes resolve to this shared identity bucket.
            cleanup_candidate_ips = sorted(set(cleanup_candidate_ips + ["unknown"]))
        if not self.preserve_state:
            self.cleanup_ips(cleanup_candidate_ips)
            self.cleanup_simulation_telemetry_history()

        try:
            frontier_metadata = build_frontier_metadata()
            ip_range_seed_evidence: Dict[str, Any] = {}
            if self.profile_name == "full_coverage":
                ip_range_seed_evidence = self.seed_ip_range_suggestion_prerequisites()
            monitoring_before = self.monitoring_snapshot()
            suite_start = time.monotonic()
            results: List[ScenarioResult] = []
            scenario_execution_evidence: Dict[str, Dict[str, Any]] = {}

            for scenario in self.selected_scenarios:
                elapsed = time.monotonic() - suite_start
                if elapsed > self.profile["max_runtime_seconds"]:
                    results.append(
                        ScenarioResult(
                            id=scenario["id"],
                            tier=scenario["tier"],
                            driver=scenario["driver"],
                            expected_outcome=scenario["expected_outcome"],
                            observed_outcome=None,
                            passed=False,
                            latency_ms=0,
                            runtime_budget_ms=scenario["runtime_budget_ms"],
                            detail=(
                                f"Suite runtime budget exceeded before scenario start "
                                f"({elapsed:.2f}s > {self.profile['max_runtime_seconds']}s)"
                            ),
                        )
                    )
                    break

                scenario_request_count_before = self.request_count
                scenario_monitoring_before = self.monitoring_snapshot()
                scenario_events_before = self.simulation_event_snapshot(hours=24, limit=1000)
                result = self.run_scenario(scenario)
                scenario_monitoring_after = self.monitoring_snapshot()
                scenario_events_after = self.simulation_event_snapshot(hours=24, limit=1000)
                scenario_evidence = build_scenario_execution_evidence(
                    scenario_id=scenario["id"],
                    request_count_before=scenario_request_count_before,
                    request_count_after=self.request_count,
                    monitoring_before=scenario_monitoring_before,
                    monitoring_after=scenario_monitoring_after,
                    simulation_event_count_before=int_or_zero(scenario_events_before.get("count")),
                    simulation_event_count_after=int_or_zero(scenario_events_after.get("count")),
                    simulation_event_reasons_before=list_or_empty(scenario_events_before.get("reasons")),
                    simulation_event_reasons_after=list_or_empty(scenario_events_after.get("reasons")),
                    simulation_event_reason_counts_before=dict_or_empty(
                        scenario_events_before.get("reason_counts")
                    ),
                    simulation_event_reason_counts_after=dict_or_empty(
                        scenario_events_after.get("reason_counts")
                    ),
                    driver_class=scenario_driver_class(scenario),
                    browser_realism=result.realism,
                )
                result.execution_evidence = scenario_evidence
                scenario_execution_evidence[result.id] = scenario_evidence
                results.append(result)
                if bool(self.profile.get("fail_fast")) and not result.passed:
                    break

            monitoring_after = self.monitoring_snapshot()
            simulation_event_reasons = self.simulation_event_reasons_snapshot(hours=24, limit=500)
            ip_range_post_run = {}
            if self.profile_name == "full_coverage":
                ip_range_post_run = self.ip_range_suggestions_snapshot(hours=24, limit=20)
            suite_runtime_ms = int((time.monotonic() - suite_start) * 1000)
            gate_results = self.evaluate_gates(
                results,
                monitoring_before,
                monitoring_after,
                suite_runtime_ms,
                scenario_execution_evidence=scenario_execution_evidence,
                simulation_event_reasons=simulation_event_reasons,
                ip_range_seed_evidence=ip_range_seed_evidence,
                ip_range_post_run=ip_range_post_run,
            )
            generated_at_unix = int(time.time())
            attack_plan_path = self.report_path.with_name("attack_plan.json")
            attack_plan = build_attack_plan(
                profile_name=self.profile_name,
                execution_lane=self.execution_lane,
                base_url=self.base_url,
                scenarios=self.selected_scenarios,
                frontier_metadata=frontier_metadata,
                generated_at_unix=generated_at_unix,
            )
            control_plane_lineage = self.build_control_plane_lineage(generated_at_unix)
            coverage_deltas = dict_or_empty(dict_or_empty(gate_results.get("coverage_gates")).get("coverage")).get(
                "deltas",
            )
            touched_defenses = [
                str(key).strip()
                for key, value in dict_or_empty(coverage_deltas).items()
                if int_or_zero(value) > 0
            ]
            latency_p95 = 0
            for check in list_or_empty(gate_results.get("checks")):
                if str(dict_or_empty(check).get("name") or "").strip() != "latency_p95":
                    continue
                latency_p95 = int_or_zero(dict_or_empty(check).get("observed"))
                break
            scenario_evidence_rows = [
                dict_or_empty(scenario_execution_evidence.get(result.id))
                for result in results
                if isinstance(scenario_execution_evidence.get(result.id), dict)
            ]

            report = {
                "schema_version": "sim-report.v1",
                "suite_id": self.manifest["suite_id"],
                "profile": self.profile_name,
                "execution_lane": self.execution_lane,
                "base_url": self.base_url,
                "request_count": self.request_count,
                "suite_runtime_ms": suite_runtime_ms,
                "monitoring_before": monitoring_before,
                "monitoring_after": monitoring_after,
                "simulation_event_reasons": simulation_event_reasons,
                "sim_tag_diagnostics": build_sim_tag_diagnostics(
                    simulation_event_reasons, sim_secret_present=bool(self.sim_telemetry_secret)
                ),
                "retention_lifecycle": build_retention_lifecycle_report(
                    dict_or_empty(monitoring_after.get("retention_health"))
                ),
                "cost_governance": build_cost_governance_report(
                    dict_or_empty(monitoring_after.get("cost_governance"))
                ),
                "security_privacy": build_security_privacy_report(
                    dict_or_empty(monitoring_after.get("security_privacy"))
                ),
                "results": [result.__dict__ for result in results],
                "gates": gate_results,
                "coverage_gates": gate_results.get("coverage_gates", {}),
                "cohort_metrics": gate_results.get("cohort_metrics", {}),
                "realism_metrics": gate_results.get("realism", {}),
                "realism_gates": gate_results.get("realism_gates", {}),
                "scenario_intent_gates": gate_results.get("scenario_intent_gates", {}),
                "ip_range_suggestions": gate_results.get("ip_range_suggestions", {}),
                "plane_contract": {
                    "schema_version": str(LANE_CONTRACT.get("schema_version") or ""),
                    "contract_path": str(LANE_CONTRACT_PATH),
                    "attacker_forbidden_path_prefixes": list(ATTACKER_FORBIDDEN_PATH_PREFIXES),
                    "attacker_forbidden_headers": sorted(ATTACKER_FORBIDDEN_HEADERS),
                    "attacker_required_sim_headers": sorted(ATTACKER_REQUIRED_SIM_HEADERS),
                    "enforced": True,
                },
                "deterministic_attack_corpus": {
                    "schema_version": str(DETERMINISTIC_ATTACK_CORPUS.get("schema_version") or ""),
                    "corpus_revision": DETERMINISTIC_ATTACK_CORPUS_REVISION,
                    "taxonomy_version": DETERMINISTIC_ATTACK_CORPUS_TAXONOMY_VERSION,
                    "contract_path": str(DETERMINISTIC_ATTACK_CORPUS_PATH),
                    "runtime_profile": str(DETERMINISTIC_ATTACK_CORPUS.get("runtime_profile") or ""),
                    "ci_profile": str(DETERMINISTIC_ATTACK_CORPUS.get("ci_profile") or ""),
                    "ci_driver_count": len(DETERMINISTIC_DRIVER_DEFINITIONS),
                },
                "control_plane_mutation_audit": self.summarize_control_plane_mutations(),
                "coverage_contract": {
                    "schema_version": COVERAGE_CONTRACT_SCHEMA_VERSION,
                    "contract_path": str(COVERAGE_CONTRACT_PATH),
                    "contract_sha256": COVERAGE_CONTRACT_SHA256,
                    "profile": FULL_COVERAGE_PROFILE_NAME,
                    "coverage_requirement_keys": sorted(COVERAGE_CONTRACT_REQUIREMENTS.keys()),
                    "coverage_depth_requirement_rows": sorted(
                        COVERAGE_CONTRACT_DEPTH_REQUIREMENTS.keys()
                    ),
                    "required_event_reasons": sorted(COVERAGE_CONTRACT_REQUIRED_EVENT_REASONS),
                    "required_outcome_categories": list(COVERAGE_CONTRACT_REQUIRED_OUTCOME_CATEGORIES),
                    "plan_contract_rows": list(COVERAGE_CONTRACT_PLAN_ROWS),
                    "compatibility": dict_or_empty(COVERAGE_CONTRACT.get("compatibility")),
                },
                "real_traffic_contract": {
                    "schema_version": REAL_TRAFFIC_CONTRACT_SCHEMA_VERSION,
                    "contract_path": str(REAL_TRAFFIC_CONTRACT_PATH),
                    "contract_sha256": REAL_TRAFFIC_CONTRACT_SHA256,
                    "required_invariants": list(REAL_TRAFFIC_CONTRACT_REQUIRED_INVARIANTS),
                    "prohibited_patterns": list(REAL_TRAFFIC_CONTRACT_PROHIBITED_PATTERNS),
                    "required_scenario_evidence_fields": list(
                        REAL_TRAFFIC_CONTRACT_REQUIRED_SCENARIO_FIELDS
                    ),
                    "required_control_lineage_fields": list(
                        REAL_TRAFFIC_CONTRACT_REQUIRED_CONTROL_LINEAGE_FIELDS
                    ),
                },
                "evidence": {
                    "schema_version": "sim-run-evidence.v1",
                    "run": {
                        "request_id_lineage": {
                            "sim_run_id": self.sim_run_id,
                            "sim_profile": self.sim_profile,
                            "sim_lane": self.sim_lane,
                        },
                        "scenario_ids": [str(scenario.get("id") or "") for scenario in self.selected_scenarios],
                        "lane": self.execution_lane,
                        "defenses_touched": sorted(touched_defenses),
                        "decision_outcomes": dict_or_empty(gate_results.get("outcome_counts")),
                        "latency_ms": {
                            "suite_runtime_ms": suite_runtime_ms,
                            "p95_ms": latency_p95,
                        },
                    },
                    "scenario_execution": scenario_evidence_rows,
                    "control_plane_lineage": control_plane_lineage,
                },
                "frontier": frontier_metadata,
                "attack_plan_path": str(attack_plan_path),
                "passed": all(result.passed for result in results) and gate_results["all_passed"],
                "generated_at_unix": generated_at_unix,
            }

            self.report_path.parent.mkdir(parents=True, exist_ok=True)
            attack_plan_path.write_text(json.dumps(attack_plan, indent=2), encoding="utf-8")
            self.report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

            print(f"[adversarial] report: {self.report_path}")
            for result in results:
                status = "PASS" if result.passed else "FAIL"
                print(
                    f"[{status}] {result.id} tier={result.tier} driver={result.driver} "
                    f"expected={result.expected_outcome} observed={result.observed_outcome or 'n/a'} "
                    f"latency_ms={result.latency_ms} detail={result.detail}"
                )

            if gate_results["all_passed"]:
                print("[adversarial] quantitative gates: PASS")
            else:
                print("[adversarial] quantitative gates: FAIL")
                for gate in gate_results["checks"]:
                    if not gate["passed"]:
                        print(f"  - {gate['name']}: {gate['detail']}")

            if report["passed"]:
                print("[adversarial] profile PASS")
                return 0

            print("[adversarial] profile FAIL")
            return 1
        finally:
            self.set_execution_phase(SUITE_PHASE_TEARDOWN, "suite_teardown_start")
            if not self.preserve_state:
                try:
                    self.reset_baseline_config()
                except Exception as exc:
                    print(f"[adversarial] warning: failed to restore baseline config: {exc}")
                try:
                    self.cleanup_ips(cleanup_candidate_ips)
                except Exception as exc:
                    print(f"[adversarial] warning: failed to cleanup scenario IPs: {exc}")
                self.cleanup_simulation_telemetry_history()

    def wait_ready(self, timeout_seconds: int) -> None:
        deadline = time.monotonic() + timeout_seconds
        while time.monotonic() < deadline:
            try:
                result = self.request(
                    "GET",
                    "/health",
                    headers=self.control_client.health_headers(),
                    plane="control",
                    count_request=False,
                )
                if result.status == 200 and "OK" in result.body:
                    return
            except Exception:
                pass
            time.sleep(1)
        raise SimulationError(
            f"Spin server was not ready at {self.base_url}/health within {timeout_seconds}s"
        )

    def resolve_honeypot_path(self) -> str:
        config = self.admin_get_config()
        candidate = str(config.get("honeypot_path") or "").strip()
        if candidate.startswith("/") and len(candidate) > 1:
            return candidate
        return "/instaban"

    def reset_baseline_config(self) -> None:
        self.admin_patch(
            {
                "shadow_mode": False,
                "honeypot_enabled": True,
                "maze_enabled": True,
                "maze_auto_ban": False,
                "js_required_enforced": True,
                "pow_enabled": True,
                "not_a_bot_enabled": True,
                "challenge_puzzle_enabled": True,
                "rate_limit": 100,
                "not_a_bot_nonce_ttl_seconds": 300,
                "not_a_bot_pass_score": 6,
                "not_a_bot_fail_score": 3,
                "not_a_bot_attempt_limit_per_window": 100,
                "not_a_bot_attempt_window_seconds": 300,
                "geo_edge_headers_enabled": False,
                "geo_risk": [],
                "geo_allow": [],
                "geo_challenge": [],
                "geo_maze": [],
                "geo_block": [],
                "allowlist": [],
                "path_allowlist": [],
                "browser_policy_enabled": True,
                "browser_allowlist": [],
                "provider_backends": {
                    "rate_limiter": "internal",
                    "fingerprint_signal": "internal",
                },
                "edge_integration_mode": "off",
                "cdp_detection_enabled": True,
                "cdp_auto_ban": True,
            },
            reason="reset_baseline_config",
        )

    def cleanup_ips(self, ips: List[str]) -> None:
        for ip in ips:
            self.admin_unban(ip, reason="cleanup_ips")

    def cleanup_simulation_telemetry_history(self) -> None:
        reason = "adversarial_ephemeral_cleanup"
        self.assert_control_plane_mutation_allowed("telemetry_history_cleanup", reason)
        self.record_control_plane_mutation(
            action="telemetry_history_cleanup",
            reason=reason,
            details={"endpoint": "/admin/adversary-sim/history/cleanup"},
        )
        result = self.admin_request(
            "POST",
            "/admin/adversary-sim/history/cleanup",
            headers={TELEMETRY_CLEANUP_ACK_HEADER: TELEMETRY_CLEANUP_ACK_VALUE},
        )
        if result.status != 200:
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(
                "Failed to clear retained telemetry history via /admin/adversary-sim/history/cleanup: "
                f"status={result.status} body={detail}"
            )
        payload = parse_json_or_raise(
            result.body,
            "Failed to parse /admin/adversary-sim/history/cleanup response",
        )
        status = str(payload.get("status") or "").strip().lower()
        cleaned_flag = payload.get("cleaned") is True
        if status not in {"cleared", "cleaned"} and not cleaned_flag:
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(
                "Failed to clear retained telemetry history via /admin/adversary-sim/history/cleanup: "
                f"unexpected status={status or 'missing'} body={detail}"
            )

    def monitoring_snapshot(self) -> Dict[str, Any]:
        result = self.admin_read_request("GET", "/admin/monitoring?hours=24&limit=5")
        data = parse_json_or_raise(result.body, "Failed to parse /admin/monitoring response")
        return extract_monitoring_snapshot(data)

    def simulation_event_snapshot(self, hours: int = 24, limit: int = 500) -> Dict[str, Any]:
        result = self.admin_read_request("GET", f"/admin/events?hours={hours}&limit={limit}")
        if result.status != 200:
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(
                f"Failed to read /admin/events: status={result.status} body={detail}"
            )
        payload = parse_json_or_raise(result.body, "Failed to parse /admin/events response")
        recent_events = payload.get("recent_events")
        if not isinstance(recent_events, list):
            return {"count": 0, "reasons": []}

        reasons = set()
        reason_counts: Dict[str, int] = {}
        event_count = 0
        for event in recent_events:
            record = dict_or_empty(event)
            if not record.get("is_simulation"):
                continue
            if str(record.get("sim_run_id") or "").strip() != self.sim_run_id:
                continue
            event_count += 1
            reason = str(record.get("reason") or "").strip().lower()
            if reason:
                reasons.add(reason)
                reason_counts[reason] = reason_counts.get(reason, 0) + 1
        return {
            "count": event_count,
            "reasons": sorted(reasons),
            "reason_counts": reason_counts,
        }

    def simulation_event_reasons_snapshot(self, hours: int = 24, limit: int = 500) -> List[str]:
        snapshot = self.simulation_event_snapshot(hours=hours, limit=limit)
        return [
            str(reason).strip()
            for reason in list_or_empty(snapshot.get("reasons"))
            if str(reason).strip()
        ]

    def ip_range_suggestions_snapshot(self, hours: int = 24, limit: int = 20) -> Dict[str, Any]:
        result = self.admin_request("GET", f"/admin/ip-range/suggestions?hours={hours}&limit={limit}")
        if result.status != 200:
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(
                f"Failed to read /admin/ip-range/suggestions: status={result.status} body={detail}"
            )
        return parse_json_or_raise(result.body, "Failed to parse /admin/ip-range/suggestions response")

    def build_control_plane_lineage(self, generated_at_unix: int) -> Dict[str, Any]:
        mutation_audit = self.summarize_control_plane_mutations()
        return {
            "control_operation_id": f"deterministic-control-{self.session_nonce}",
            "requested_state": "running",
            "desired_state": "running",
            "actual_state": "completed",
            "actor_session": "deterministic_runner",
            "generated_at_unix": int(generated_at_unix),
            "phase_trace": [
                str(dict_or_empty(entry).get("phase") or "").strip()
                for entry in self.execution_phase_trace
                if str(dict_or_empty(entry).get("phase") or "").strip()
            ],
            "phase_transitions": list(self.execution_phase_trace),
            "mutation_audit": mutation_audit,
        }

    def seed_ip_range_suggestion_prerequisites(self) -> Dict[str, Any]:
        self.admin_patch(
            {
                "ip_range_policy_mode": "off",
                "ip_range_emergency_allowlist": [],
                "ip_range_custom_rules": [],
            },
            reason="seed_ip_range_suggestion_prerequisites",
        )
        for ip in self.ip_range_seed_ips:
            self.admin_unban(ip, reason="seed_ip_range_suggestion_prerequisites")
            self.attacker_client.request(
                "GET",
                self.honeypot_path,
                headers=self.forwarded_headers(ip, user_agent=f"ShumaAdversarial/1.0 ip-range-seed {ip}"),
                count_request=True,
                trusted_forwarded=True,
            )
            self.attacker_client.request(
                "GET",
                "/",
                headers=self.forwarded_headers(ip, user_agent=f"ShumaAdversarial/1.0 ip-range-seed {ip}"),
                count_request=True,
                trusted_forwarded=True,
            )
        seeded_snapshot = self.ip_range_suggestions_snapshot(hours=1, limit=20)
        seeded_suggestions = [
            suggestion
            for suggestion in seeded_snapshot.get("suggestions", [])
            if str(dict_or_empty(suggestion).get("cidr") or "").startswith("10.222.77.")
        ]
        return {
            "seed_prefix": "10.222.77.0/24",
            "seeded_ips": list(self.ip_range_seed_ips),
            "seeded_summary": dict_or_empty(seeded_snapshot.get("summary")),
            "seeded_suggestions": seeded_suggestions,
            "seed_match": bool(seeded_suggestions),
        }

    def evaluate_gates(
        self,
        results: List[ScenarioResult],
        monitoring_before: Dict[str, Any],
        monitoring_after: Dict[str, Any],
        suite_runtime_ms: int,
        scenario_execution_evidence: Optional[Dict[str, Dict[str, Any]]] = None,
        simulation_event_reasons: Optional[List[str]] = None,
        ip_range_seed_evidence: Optional[Dict[str, Any]] = None,
        ip_range_post_run: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        checks: List[Dict[str, Any]] = []
        profile_gates = self.profile.get("gates") or {}
        mutation_audit = self.summarize_control_plane_mutations()
        mutation_entries = list_or_empty(mutation_audit.get("entries"))
        forbidden_mutations = [
            mutation
            for mutation in mutation_entries
            if str(dict_or_empty(mutation).get("phase") or "").strip().lower()
            == SUITE_PHASE_ATTACKER_EXECUTION
        ]
        checks.append(
            {
                "name": "control_plane_mutation_policy",
                "passed": not forbidden_mutations,
                "detail": (
                    "no control-plane mutations observed during attacker execution"
                    if not forbidden_mutations
                    else f"forbidden_mutations={len(forbidden_mutations)}"
                ),
                "observed": {
                    "mutation_count": int_or_zero(mutation_audit.get("count")),
                    "forbidden_count": len(forbidden_mutations),
                    "forbidden_reasons": sorted(
                        {
                            str(dict_or_empty(entry).get("reason") or "").strip()
                            for entry in forbidden_mutations
                            if str(dict_or_empty(entry).get("reason") or "").strip()
                        }
                    ),
                },
                "threshold_source": "suite_phase_contract.attacker_execution_mutation_forbidden",
            }
        )

        latency_values = [result.latency_ms for result in results if result.passed and result.latency_ms > 0]
        p95 = percentile(latency_values, 95)
        p95_limit = int(self.profile["gates"]["latency"]["p95_max_ms"])
        checks.append(
            {
                "name": "latency_p95",
                "passed": p95 <= p95_limit,
                "detail": f"p95={p95}ms limit={p95_limit}ms",
                "observed": p95,
                "limit": p95_limit,
                "threshold_source": "profile.gates.latency.p95_max_ms",
            }
        )

        successful_results = [result for result in results if result.passed and result.observed_outcome]
        outcome_counts: Dict[str, int] = {}
        for result in successful_results:
            outcome_counts[result.observed_outcome] = outcome_counts.get(result.observed_outcome, 0) + 1

        total_successful = len(successful_results)
        ratio_bounds = profile_gates["outcome_ratio_bounds"]
        for outcome, bounds in ratio_bounds.items():
            ratio = (outcome_counts.get(outcome, 0) / total_successful) if total_successful else 0.0
            minimum = float(bounds["min"])
            maximum = float(bounds["max"])
            checks.append(
                {
                    "name": f"ratio_{outcome}",
                    "passed": minimum <= ratio <= maximum,
                    "detail": (
                        f"ratio={ratio:.3f} bounds=[{minimum:.3f},{maximum:.3f}] "
                        f"count={outcome_counts.get(outcome, 0)} total={total_successful}"
                    ),
                    "observed": ratio,
                    "min": minimum,
                    "max": maximum,
                    "threshold_source": f"profile.gates.outcome_ratio_bounds.{outcome}",
                }
            )

        cohort_metrics = compute_cohort_metrics(self.selected_scenarios, results)
        human_like_collateral_limit = profile_gates.get("human_like_collateral_max_ratio")
        if human_like_collateral_limit is not None:
            limit = float(human_like_collateral_limit)
            human_like = dict_or_empty(cohort_metrics.get("human_like"))
            observed_ratio = float(human_like.get("collateral_ratio") or 0.0)
            checks.append(
                {
                    "name": "human_like_collateral_ratio",
                    "passed": observed_ratio <= limit,
                    "detail": (
                        f"ratio={observed_ratio:.3f} limit={limit:.3f} "
                        f"collateral={int_or_zero(human_like.get('collateral_count'))} "
                        f"total={int_or_zero(human_like.get('total'))}"
                    ),
                    "observed": observed_ratio,
                    "limit": limit,
                    "threshold_source": "profile.gates.human_like_collateral_max_ratio",
                }
            )

        persona_scheduler = str(profile_gates.get("persona_scheduler") or "").strip().lower()
        realism_metrics = compute_realism_metrics(
            self.selected_scenarios, results, persona_scheduler=persona_scheduler
        )
        realism_checks = build_realism_checks(
            self.profile_name, profile_gates, realism_metrics
        )
        checks.extend(realism_checks)

        fp_delta = max(0, monitoring_after["fingerprint_events"] - monitoring_before["fingerprint_events"])
        monitoring_delta = max(0, monitoring_after["monitoring_total"] - monitoring_before["monitoring_total"])

        req_count = max(1, self.request_count)
        fp_amp = fp_delta / req_count
        mon_amp = monitoring_delta / req_count

        fp_limit = float(self.profile["gates"]["telemetry_amplification"]["max_fingerprint_events_per_request"])
        mon_limit = float(self.profile["gates"]["telemetry_amplification"]["max_monitoring_events_per_request"])

        checks.append(
            {
                "name": "telemetry_fingerprint_amplification",
                "passed": fp_amp <= fp_limit,
                "detail": f"amp={fp_amp:.3f} limit={fp_limit:.3f} delta={fp_delta} requests={req_count}",
                "observed": fp_amp,
                "limit": fp_limit,
                "threshold_source": "profile.gates.telemetry_amplification.max_fingerprint_events_per_request",
            }
        )
        checks.append(
            {
                "name": "telemetry_monitoring_amplification",
                "passed": mon_amp <= mon_limit,
                "detail": f"amp={mon_amp:.3f} limit={mon_limit:.3f} delta={monitoring_delta} requests={req_count}",
                "observed": mon_amp,
                "limit": mon_limit,
                "threshold_source": "profile.gates.telemetry_amplification.max_monitoring_events_per_request",
            }
        )

        runtime_limit_ms = int(self.profile["max_runtime_seconds"]) * 1000
        checks.append(
            {
                "name": "suite_runtime_budget",
                "passed": suite_runtime_ms <= runtime_limit_ms,
                "detail": f"runtime={suite_runtime_ms}ms limit={runtime_limit_ms}ms",
                "observed": suite_runtime_ms,
                "limit": runtime_limit_ms,
                "threshold_source": "profile.max_runtime_seconds",
            }
        )

        coverage_before = dict_or_empty(monitoring_before.get("coverage"))
        coverage_after = dict_or_empty(monitoring_after.get("coverage"))
        coverage_deltas = compute_coverage_deltas(coverage_before, coverage_after)

        coverage_requirements, declared_coverage_requirements = select_coverage_requirements(
            self.profile_name, profile_gates
        )
        coverage_depth_requirements, declared_coverage_depth_requirements = (
            select_coverage_depth_requirements(self.profile_name, profile_gates)
        )
        coverage_checks: List[Dict[str, Any]] = []
        coverage_depth_checks: List[Dict[str, Any]] = []
        coverage_depth_row_diagnostics: List[Dict[str, Any]] = []
        coverage_contract_parity = {
            "missing_keys": [],
            "extra_keys": [],
            "mismatched_values": {},
            "parity_passed": True,
        }
        coverage_depth_contract_parity = {
            "missing_keys": [],
            "extra_keys": [],
            "mismatched_values": {},
            "parity_passed": True,
        }
        threshold_prefix = "profile.gates.coverage_requirements"
        depth_threshold_prefix = "profile.gates.coverage_depth_requirements"
        if self.profile_name == FULL_COVERAGE_PROFILE_NAME:
            coverage_contract_parity = coverage_contract_parity_diagnostics(
                declared_coverage_requirements
            )
            threshold_prefix = (
                f"{COVERAGE_CONTRACT_SCHEMA_VERSION}.coverage_requirements"
            )
            coverage_depth_contract_parity = coverage_depth_contract_parity_diagnostics(
                declared_coverage_depth_requirements
            )
            depth_threshold_prefix = (
                f"{COVERAGE_CONTRACT_SCHEMA_VERSION}.coverage_depth_requirements"
            )
            checks.append(
                {
                    "name": "coverage_contract_parity",
                    "passed": bool(coverage_contract_parity["parity_passed"]),
                    "detail": (
                        f"missing={coverage_contract_parity['missing_keys']} "
                        f"extra={coverage_contract_parity['extra_keys']} "
                        f"mismatched={sorted(coverage_contract_parity['mismatched_values'].keys())}"
                    ),
                    "observed": {
                        "missing": list(coverage_contract_parity["missing_keys"]),
                        "extra": list(coverage_contract_parity["extra_keys"]),
                        "mismatched": dict(coverage_contract_parity["mismatched_values"]),
                    },
                    "threshold_source": f"{COVERAGE_CONTRACT_PATH}",
                }
            )
            checks.append(
                {
                    "name": "coverage_depth_contract_parity",
                    "passed": bool(coverage_depth_contract_parity["parity_passed"]),
                    "detail": (
                        f"missing={coverage_depth_contract_parity['missing_keys']} "
                        f"extra={coverage_depth_contract_parity['extra_keys']} "
                        "mismatched="
                        f"{sorted(coverage_depth_contract_parity['mismatched_values'].keys())}"
                    ),
                    "observed": {
                        "missing": list(coverage_depth_contract_parity["missing_keys"]),
                        "extra": list(coverage_depth_contract_parity["extra_keys"]),
                        "mismatched": dict(coverage_depth_contract_parity["mismatched_values"]),
                    },
                    "threshold_source": f"{COVERAGE_CONTRACT_PATH}",
                }
            )

        if coverage_requirements:
            coverage_checks = build_coverage_checks(coverage_requirements, coverage_deltas)
            coverage_checks = annotate_coverage_checks_with_threshold_source(
                coverage_requirements, coverage_checks, threshold_prefix=threshold_prefix
            )
            checks.extend(coverage_checks)
        if coverage_depth_requirements:
            coverage_depth_checks, coverage_depth_row_diagnostics = build_coverage_depth_checks(
                coverage_depth_requirements,
                coverage_deltas=coverage_deltas,
                scenario_execution_evidence=scenario_execution_evidence,
            )
            for check in coverage_depth_checks:
                check_name = str(check.get("name") or "")
                row_id = check_name.removeprefix("coverage_depth_")
                check["threshold_source"] = f"{depth_threshold_prefix}.{row_id}"
            checks.extend(coverage_depth_checks)

        defense_noop_checks: List[Dict[str, Any]] = []
        if self.profile_name == FULL_COVERAGE_PROFILE_NAME:
            defense_noop_checks = build_defense_noop_checks(
                defense_categories=profile_expected_defense_categories(self.selected_scenarios),
                coverage_deltas=coverage_deltas,
            )
            checks.extend(defense_noop_checks)

        required_event_reasons = profile_gates.get("required_event_reasons")
        if isinstance(required_event_reasons, list) and required_event_reasons:
            observed_reasons = [
                str(reason).strip().lower()
                for reason in list(simulation_event_reasons or [])
                if str(reason).strip()
            ]
            for required in required_event_reasons:
                required_prefix = str(required or "").strip().lower()
                if not required_prefix:
                    continue
                matched = [reason for reason in observed_reasons if reason.startswith(required_prefix)]
                checks.append(
                    {
                        "name": f"event_reason_prefix_{required_prefix}",
                        "passed": bool(matched),
                        "detail": (
                            f"required_prefix={required_prefix} "
                            f"matched={matched[0] if matched else 'none'}"
                        ),
                        "observed": matched[0] if matched else "",
                        "threshold_source": f"profile.gates.required_event_reasons[{required_prefix}]",
                    }
                )

        suggestion_seed_required = bool(profile_gates.get("ip_range_suggestion_seed_required"))
        ip_range_suggestions = {
            "seed_evidence": dict_or_empty(ip_range_seed_evidence),
            "post_run": dict_or_empty(ip_range_post_run),
            "matched_seed_suggestions": [],
            "near_miss_suggestions": [],
        }
        post_run_suggestions = list_or_empty(dict_or_empty(ip_range_post_run).get("suggestions"))
        seed_prefix = str(dict_or_empty(ip_range_seed_evidence).get("seed_prefix") or "").split("/", 1)[0]
        if seed_prefix:
            ip_range_suggestions["matched_seed_suggestions"] = [
                suggestion
                for suggestion in post_run_suggestions
                if str(dict_or_empty(suggestion).get("cidr") or "").startswith(seed_prefix)
            ]
            ip_range_suggestions["near_miss_suggestions"] = [
                suggestion
                for suggestion in post_run_suggestions
                if seed_prefix in ",".join(list_or_empty(dict_or_empty(suggestion).get("safer_alternatives")))
            ]
        if suggestion_seed_required:
            seed_match = bool(dict_or_empty(ip_range_seed_evidence).get("seed_match"))
            if not seed_match and ip_range_suggestions["matched_seed_suggestions"]:
                seed_match = True
            checks.append(
                {
                    "name": "ip_range_suggestion_seed_match",
                    "passed": seed_match,
                    "detail": (
                        f"seed_prefix={dict_or_empty(ip_range_seed_evidence).get('seed_prefix')} "
                        f"matches={len(ip_range_suggestions['matched_seed_suggestions'])}"
                    ),
                    "observed": len(ip_range_suggestions["matched_seed_suggestions"]),
                    "threshold_source": "profile.gates.ip_range_suggestion_seed_required",
                }
            )

        runtime_evidence_checks = build_runtime_telemetry_evidence_checks(
            results=results,
            scenario_execution_evidence=dict_or_empty(scenario_execution_evidence),
            required_fields=REAL_TRAFFIC_CONTRACT_REQUIRED_SCENARIO_FIELDS,
        )
        checks.extend(runtime_evidence_checks)
        browser_execution_checks = build_browser_execution_evidence_checks(
            selected_scenarios=self.selected_scenarios,
            results=results,
            scenario_execution_evidence=dict_or_empty(scenario_execution_evidence),
        )
        checks.extend(browser_execution_checks)
        scenario_intent_checks = build_scenario_intent_checks(
            selected_scenarios=self.selected_scenarios,
            results=results,
            scenario_execution_evidence=dict_or_empty(scenario_execution_evidence),
            intent_rows_by_id=SCENARIO_INTENT_MATRIX_ROWS_BY_ID,
        )
        checks.extend(scenario_intent_checks)

        all_passed = all(check["passed"] for check in checks)
        coverage_all_passed = (
            (all(check["passed"] for check in coverage_checks) if coverage_checks else True)
            and (all(check["passed"] for check in coverage_depth_checks) if coverage_depth_checks else True)
        )
        realism_all_passed = all(check["passed"] for check in realism_checks) if realism_checks else True
        browser_execution_all_passed = (
            all(check["passed"] for check in browser_execution_checks)
            if browser_execution_checks
            else True
        )
        scenario_intent_all_passed = (
            all(check["passed"] for check in scenario_intent_checks)
            if scenario_intent_checks
            else True
        )
        return {
            "all_passed": all_passed,
            "checks": checks,
            "outcome_counts": outcome_counts,
            "request_count": self.request_count,
            "coverage": {
                "before": coverage_before,
                "after": coverage_after,
                "deltas": coverage_deltas,
            },
            "coverage_gates": {
                "all_passed": coverage_all_passed,
                "threshold_source": threshold_prefix,
                "depth_threshold_source": depth_threshold_prefix,
                "checks": coverage_checks,
                "depth_checks": coverage_depth_checks,
                "depth_row_diagnostics": coverage_depth_row_diagnostics,
                "defense_noop_checks": defense_noop_checks,
                "coverage": {
                    "before": coverage_before,
                    "after": coverage_after,
                    "deltas": coverage_deltas,
                },
                "contract": {
                    "schema_version": COVERAGE_CONTRACT_SCHEMA_VERSION,
                    "contract_path": str(COVERAGE_CONTRACT_PATH),
                    "contract_sha256": COVERAGE_CONTRACT_SHA256,
                    "profile": FULL_COVERAGE_PROFILE_NAME,
                    "coverage_requirement_keys": sorted(COVERAGE_CONTRACT_REQUIREMENTS.keys()),
                    "coverage_depth_requirement_rows": sorted(
                        COVERAGE_CONTRACT_DEPTH_REQUIREMENTS.keys()
                    ),
                    "required_event_reasons": sorted(COVERAGE_CONTRACT_REQUIRED_EVENT_REASONS),
                    "required_outcome_categories": list(COVERAGE_CONTRACT_REQUIRED_OUTCOME_CATEGORIES),
                },
                "missing_contract_categories": list(coverage_contract_parity["missing_keys"]),
                "extra_manifest_categories": list(coverage_contract_parity["extra_keys"]),
                "mismatched_contract_values": dict(coverage_contract_parity["mismatched_values"]),
                "missing_contract_depth_rows": list(
                    coverage_depth_contract_parity["missing_keys"]
                ),
                "extra_manifest_depth_rows": list(
                    coverage_depth_contract_parity["extra_keys"]
                ),
                "mismatched_depth_rows": dict(
                    coverage_depth_contract_parity["mismatched_values"]
                ),
            },
            "cohort_metrics": cohort_metrics,
            "realism": realism_metrics,
            "realism_gates": {
                "all_passed": realism_all_passed,
                "checks": realism_checks,
                "persona_scheduler": persona_scheduler,
            },
            "browser_execution_gates": {
                "all_passed": browser_execution_all_passed,
                "checks": browser_execution_checks,
            },
            "scenario_intent_gates": {
                "all_passed": scenario_intent_all_passed,
                "checks": scenario_intent_checks,
                "contract": {
                    "schema_version": SCENARIO_INTENT_MATRIX_SCHEMA_VERSION,
                    "contract_path": str(SCENARIO_INTENT_MATRIX_PATH),
                    "contract_sha256": SCENARIO_INTENT_MATRIX_SHA256,
                },
            },
            "ip_range_suggestions": ip_range_suggestions,
        }

    def scenario_setup_patch(self, scenario: Dict[str, Any]) -> Dict[str, Any]:
        driver_name = str(scenario.get("driver") or "").strip()
        geo_country = str(scenario.get("geo_country") or "").strip()

        if driver_name == "allow_browser_allowlist":
            return {
                "shadow_mode": False,
                "browser_policy_enabled": False,
                "browser_allowlist": [["Chrome", 120]],
                "js_required_enforced": False,
                "pow_enabled": False,
            }
        if driver_name == "not_a_bot_pass":
            return {"shadow_mode": True, "not_a_bot_enabled": True, "challenge_puzzle_enabled": True}
        if driver_name == "challenge_puzzle_fail_maze":
            return {"shadow_mode": True, "maze_enabled": True, "challenge_puzzle_enabled": True}
        if driver_name in {"pow_success", "pow_invalid_proof"}:
            return {
                "shadow_mode": False,
                "pow_enabled": True,
                "pow_difficulty": 12,
                "pow_ttl_seconds": 120,
            }
        if driver_name in {"rate_limit_enforce", "retry_storm_enforce"}:
            return {
                "shadow_mode": False,
                "rate_limit": 2,
                "js_required_enforced": False,
                "defence_modes": {"rate": "both"},
                "provider_backends": {"rate_limiter": "internal"},
                "edge_integration_mode": "off",
                "browser_policy_enabled": False,
            }
        if driver_name == "geo_challenge":
            return {
                "shadow_mode": False,
                "geo_edge_headers_enabled": True,
                "geo_risk": [],
                "geo_allow": [],
                "geo_challenge": [geo_country] if geo_country else [],
                "geo_maze": [],
                "geo_block": [],
            }
        if driver_name == "geo_maze":
            return {
                "shadow_mode": False,
                "maze_enabled": True,
                "maze_auto_ban": False,
                "geo_edge_headers_enabled": True,
                "geo_risk": [],
                "geo_allow": [],
                "geo_challenge": [],
                "geo_maze": [geo_country] if geo_country else [],
                "geo_block": [],
            }
        if driver_name == "geo_block":
            return {
                "shadow_mode": False,
                "geo_edge_headers_enabled": True,
                "geo_risk": [],
                "geo_allow": [],
                "geo_challenge": [],
                "geo_maze": [],
                "geo_block": [geo_country] if geo_country else [],
            }
        if driver_name == "honeypot_deny_temp":
            return {"shadow_mode": False, "honeypot_enabled": True}
        if driver_name in {
            "not_a_bot_replay_abuse",
            "not_a_bot_stale_token_abuse",
            "not_a_bot_ordering_cadence_abuse",
        }:
            return {
                "shadow_mode": True,
                "maze_enabled": True,
                "not_a_bot_nonce_ttl_seconds": 300,
            }
        if driver_name == "not_a_bot_replay_tarpit_abuse":
            return {
                "shadow_mode": True,
                "maze_enabled": True,
                "tarpit_enabled": True,
                "tarpit_progress_token_ttl_seconds": 120,
                "tarpit_progress_replay_ttl_seconds": 300,
                "tarpit_hashcash_min_difficulty": 8,
                "tarpit_hashcash_max_difficulty": 16,
                "tarpit_hashcash_base_difficulty": 10,
                "tarpit_hashcash_adaptive": True,
                "tarpit_step_chunk_base_bytes": 4096,
                "tarpit_step_chunk_max_bytes": 12288,
                "tarpit_step_jitter_percent": 15,
                "tarpit_shard_rotation_enabled": True,
                "tarpit_egress_window_seconds": 60,
                "tarpit_egress_global_bytes_per_window": 8388608,
                "tarpit_egress_per_ip_bucket_bytes_per_window": 1048576,
                "tarpit_egress_per_flow_max_bytes": 524288,
                "tarpit_egress_per_flow_max_duration_seconds": 120,
                "tarpit_max_concurrent_global": 10000,
                "tarpit_max_concurrent_per_ip_bucket": 256,
                "tarpit_fallback_action": "maze",
            }
        if driver_name == "fingerprint_inconsistent_payload":
            return {
                "shadow_mode": False,
                "provider_backends": {"fingerprint_signal": "external"},
                "edge_integration_mode": "additive",
                "cdp_detection_enabled": True,
                "cdp_auto_ban": True,
            }
        if driver_name == "header_spoofing_probe":
            return {
                "shadow_mode": False,
                "defence_modes": {"rate": "off"},
                "js_required_enforced": False,
                "pow_enabled": False,
                "geo_edge_headers_enabled": True,
                "geo_risk": [],
                "geo_allow": [],
                "geo_challenge": [],
                "geo_maze": [],
                "geo_block": [geo_country or "RU"],
                "browser_policy_enabled": False,
                "cdp_detection_enabled": False,
                "cdp_auto_ban": False,
            }
        if driver_name == "cdp_high_confidence_deny":
            return {
                "shadow_mode": False,
                "provider_backends": {"fingerprint_signal": "internal"},
                "edge_integration_mode": "off",
                "cdp_detection_enabled": True,
                "cdp_auto_ban": True,
            }
        if driver_name == "akamai_additive_report":
            return {
                "shadow_mode": False,
                "provider_backends": {"fingerprint_signal": "external"},
                "edge_integration_mode": "additive",
                "cdp_detection_enabled": True,
                "cdp_auto_ban": True,
            }
        if driver_name == "akamai_authoritative_deny":
            return {
                "shadow_mode": False,
                "provider_backends": {"fingerprint_signal": "external"},
                "edge_integration_mode": "authoritative",
                "cdp_detection_enabled": True,
                "cdp_auto_ban": True,
            }
        return {}

    def apply_scenario_setup_preset(self, scenario: Dict[str, Any]) -> None:
        patch = self.scenario_setup_patch(scenario)
        if not patch:
            return
        scenario_id = str(scenario.get("id") or "").strip()
        driver_name = str(scenario.get("driver") or "").strip()
        self.admin_patch(
            patch,
            reason=f"scenario_setup_preset:{scenario_id}:{driver_name}",
        )

    def prepare_scenario_setup_state(self, scenario: Dict[str, Any]) -> None:
        scenario_id = str(scenario.get("id") or "").strip()
        driver_name = str(scenario.get("driver") or "").strip()
        self._scenario_setup_context[scenario_id] = {}

        if driver_name != "not_a_bot_replay_tarpit_abuse":
            return

        seed, _ = self.fetch_not_a_bot_seed(scenario)
        first_submit = self.submit_not_a_bot(seed, scenario, GOOD_NOT_A_BOT_TELEMETRY)
        if first_submit.status != 303:
            raise SimulationError(
                f"invariant_not_a_bot_prime_failed status={first_submit.status}"
            )

        self._scenario_setup_context[scenario_id]["tarpit_replay_seed"] = seed
        self.admin_patch(
            {
                "shadow_mode": False,
                "maze_enabled": True,
                "tarpit_enabled": True,
                "tarpit_fallback_action": "maze",
            },
            reason=f"scenario_setup_preset:{scenario_id}:tarpit_replay_finalize",
        )

    def run_scenario(self, scenario: Dict[str, Any]) -> ScenarioResult:
        scenario_id = scenario["id"]
        scenario_start = time.monotonic()
        observed_outcome: Optional[str] = None
        realism: Optional[Dict[str, Any]] = None
        latency_ms = 0

        try:
            self._scenario_attacker_phase_started = False
            self.set_execution_phase(SUITE_PHASE_SETUP, f"scenario_setup:{scenario_id}")
            if not self.preserve_state:
                self.admin_unban("unknown", reason=f"scenario_setup:{scenario_id}:reset_unknown_identity")
            self.reset_baseline_config()
            self.apply_scenario_setup_preset(scenario)
            self.prepare_scenario_setup_state(scenario)
            self.set_execution_phase(
                SUITE_PHASE_ATTACKER_EXECUTION,
                f"scenario_attacker_execution:{scenario_id}",
            )

            execution_start = time.monotonic()
            self.begin_scenario_execution(scenario)
            try:
                observed_outcome = self.execute_scenario_driver(scenario)
            finally:
                realism = self.end_scenario_execution()

            latency_ms = effective_scenario_latency_ms(
                scenario,
                int((time.monotonic() - execution_start) * 1000),
                realism,
            )

            if observed_outcome != scenario["expected_outcome"]:
                return ScenarioResult(
                    id=scenario_id,
                    tier=scenario["tier"],
                    driver=scenario["driver"],
                    expected_outcome=scenario["expected_outcome"],
                    observed_outcome=observed_outcome,
                    passed=False,
                    latency_ms=latency_ms,
                    runtime_budget_ms=scenario["runtime_budget_ms"],
                    detail=(
                        f"Outcome mismatch: expected={scenario['expected_outcome']} "
                        f"observed={observed_outcome}"
                    ),
                    realism=realism,
                )

            max_latency_ms = scenario_max_latency_ms(scenario)
            if latency_ms > max_latency_ms:
                return ScenarioResult(
                    id=scenario_id,
                    tier=scenario["tier"],
                    driver=scenario["driver"],
                    expected_outcome=scenario["expected_outcome"],
                    observed_outcome=observed_outcome,
                    passed=False,
                    latency_ms=latency_ms,
                    runtime_budget_ms=scenario["runtime_budget_ms"],
                    detail=f"Scenario latency exceeded: {latency_ms}ms > {max_latency_ms}ms",
                    realism=realism,
                )

            if latency_ms > int(scenario["runtime_budget_ms"]):
                return ScenarioResult(
                    id=scenario_id,
                    tier=scenario["tier"],
                    driver=scenario["driver"],
                    expected_outcome=scenario["expected_outcome"],
                    observed_outcome=observed_outcome,
                    passed=False,
                    latency_ms=latency_ms,
                    runtime_budget_ms=scenario["runtime_budget_ms"],
                    detail=(
                        f"Scenario runtime budget exceeded: {latency_ms}ms "
                        f"> {scenario['runtime_budget_ms']}ms"
                    ),
                    realism=realism,
                )

            return ScenarioResult(
                id=scenario_id,
                tier=scenario["tier"],
                driver=scenario["driver"],
                expected_outcome=scenario["expected_outcome"],
                observed_outcome=observed_outcome,
                passed=True,
                latency_ms=latency_ms,
                runtime_budget_ms=scenario["runtime_budget_ms"],
                detail="ok",
                realism=realism,
            )
        except Exception as exc:
            if latency_ms <= 0:
                latency_ms = int((time.monotonic() - scenario_start) * 1000)
            if realism is None:
                realism = self.end_scenario_execution()
            latency_ms = effective_scenario_latency_ms(scenario, latency_ms, realism)
            return ScenarioResult(
                id=scenario_id,
                tier=scenario["tier"],
                driver=scenario["driver"],
                expected_outcome=scenario["expected_outcome"],
                observed_outcome=observed_outcome,
                passed=False,
                latency_ms=latency_ms,
                runtime_budget_ms=scenario["runtime_budget_ms"],
                detail=f"exception: {exc}",
                realism=realism,
            )

    def execute_scenario_driver(self, scenario: Dict[str, Any]) -> str:
        driver_name = str(scenario.get("driver") or "")
        driver_class = scenario_driver_class(scenario)
        if driver_class not in ALLOWED_DRIVER_CLASSES:
            raise SimulationError(
                f"scenario {scenario.get('id')} has unsupported driver_class={driver_class}"
            )
        family_handlers = DRIVER_CLASS_HANDLERS.get(driver_class, {})
        handler_name = family_handlers.get(driver_name)
        if not handler_name:
            raise SimulationError(
                f"scenario {scenario.get('id')} driver={driver_name} is not supported in driver_class={driver_class}"
            )
        handler = getattr(self, handler_name, None)
        if handler is None:
            raise SimulationError(
                f"scenario {scenario.get('id')} driver handler missing for {driver_class}.{driver_name}"
            )
        return str(handler(scenario))

    def record_browser_driver_evidence(
        self,
        browser_evidence: Dict[str, Any],
        error_code: str = "",
        diagnostics: Optional[Dict[str, Any]] = None,
    ) -> None:
        state = self._active_execution_state
        if not state:
            return
        evidence = dict_or_empty(state.get("evidence"))
        if not evidence:
            return
        policy = dict_or_empty(state.get("policy"))
        browser_diagnostics = dict_or_empty(diagnostics)
        evidence["browser_driver_runtime"] = str(
            browser_evidence.get("driver_runtime") or "playwright_chromium"
        )
        evidence["browser_js_executed"] = bool(browser_evidence.get("js_executed"))
        evidence["browser_dom_events"] = max(0, int_or_zero(browser_evidence.get("dom_events")))
        evidence["browser_storage_mode"] = str(
            browser_evidence.get("storage_mode") or evidence.get("browser_storage_mode") or ""
        )
        evidence["browser_challenge_dom_path"] = [
            str(item).strip()
            for item in list_or_empty(browser_evidence.get("challenge_dom_path"))
            if str(item).strip()
        ]
        evidence["browser_correlation_ids"] = [
            str(item).strip()
            for item in list_or_empty(browser_evidence.get("correlation_ids"))
            if str(item).strip()
        ]
        request_lineage_count = len(list_or_empty(browser_evidence.get("request_lineage")))
        evidence["browser_request_lineage_count"] = request_lineage_count
        if request_lineage_count > 0:
            existing_sequence = max(0, int_or_zero(evidence.get("request_sequence")))
            evidence["request_sequence"] = max(existing_sequence, request_lineage_count)
            existing_attempts = max(0, int_or_zero(evidence.get("attempts_total")))
            evidence["attempts_total"] = max(existing_attempts, request_lineage_count)
            synthesized_think_events = max(1, request_lineage_count - 1)
            existing_think_events = max(0, int_or_zero(evidence.get("think_time_events")))
            evidence["think_time_events"] = max(existing_think_events, synthesized_think_events)
            think_time_floor = max(1, int_or_zero(policy.get("think_time_ms_min")))
            synthesized_think_total = synthesized_think_events * think_time_floor
            existing_think_total = max(0, int_or_zero(evidence.get("think_time_ms_total")))
            evidence["think_time_ms_total"] = max(existing_think_total, synthesized_think_total)
        evidence["browser_error_code"] = str(error_code or "")
        evidence["browser_action_duration_ms"] = max(
            0, int_or_zero(browser_diagnostics.get("action_duration_ms"))
        )
        evidence["browser_launch_duration_ms"] = max(
            0, int_or_zero(browser_diagnostics.get("launch_duration_ms"))
        )
        evidence["browser_total_duration_ms"] = max(
            0, int_or_zero(browser_diagnostics.get("total_duration_ms"))
        )

    def execute_browser_realistic_driver(
        self,
        scenario: Dict[str, Any],
        action: str,
        headers: Optional[Dict[str, str]] = None,
        user_agent: Optional[str] = None,
        inject_trusted_forwarded_secret: bool = True,
    ) -> str:
        if not self.browser_driver_enabled:
            raise SimulationError(
                "browser-realistic driver is disabled by SHUMA_ADVERSARIAL_BROWSER_DRIVER_ENABLED=false"
            )
        if not self.browser_driver_script_path.exists():
            raise SimulationError(
                f"browser-realistic driver script missing: {self.browser_driver_script_path}"
            )

        scenario_headers = dict(headers or self.forwarded_headers(self.scenario_ip(scenario)))
        scenario_headers = {
            key: value
            for key, value in scenario_headers.items()
            if str(key).strip().lower() != "user-agent"
        }
        traffic_model = dict_or_empty(scenario.get("traffic_model"))
        storage_mode = str(traffic_model.get("cookie_behavior") or "stateful_cookie_jar")
        if storage_mode not in ALLOWED_COOKIE_BEHAVIORS:
            storage_mode = "stateful_cookie_jar"

        timeout_ms = max(2000, min(60000, int_or_zero(scenario.get("runtime_budget_ms")) + 4000))
        timeout_ms = min(timeout_ms, self.browser_driver_timeout_ms)
        payload = {
            "action": action,
            "base_url": self.base_url,
            "scenario_id": str(scenario.get("id") or ""),
            "headers": scenario_headers,
            "user_agent": str(user_agent or self.scenario_user_agent(scenario)),
            "timeout_ms": timeout_ms,
            "settle_ms": self.browser_driver_settle_ms,
            "storage_mode": storage_mode,
            "honeypot_path": self.honeypot_path,
        }
        if inject_trusted_forwarded_secret and self.forwarded_secret:
            # Browser lane models edge-injected forwarded trust separately from attacker-controlled headers.
            payload["trusted_forwarded_secret"] = self.forwarded_secret

        last_error = "browser_driver_failed"
        max_attempts = max(1, int(self.browser_driver_max_attempts))
        command_timeout = max(
            self.request_timeout_seconds + 5.0,
            (timeout_ms / 1000.0) + 5.0,
        )

        for attempt in range(1, max_attempts + 1):
            payload["attempt"] = attempt
            try:
                proc = subprocess.run(
                    self.browser_driver_command,
                    input=json.dumps(payload, separators=(",", ":")),
                    text=True,
                    capture_output=True,
                    timeout=command_timeout,
                    check=False,
                    env=self.browser_driver_env,
                )
            except subprocess.TimeoutExpired:
                last_error = (
                    "browser driver timed out "
                    f"(attempt={attempt}/{max_attempts} timeout={command_timeout:.1f}s)"
                )
                self.record_browser_driver_evidence({}, error_code="timeout")
                if attempt < max_attempts:
                    continue
                break
            except Exception as exc:
                last_error = f"browser driver launch failed: {exc}"
                self.record_browser_driver_evidence({}, error_code="runtime_failure")
                if attempt < max_attempts:
                    continue
                break

            raw_stdout = str(proc.stdout or "").strip()
            raw_stderr = str(proc.stderr or "").strip()
            parsed: Dict[str, Any] = {}
            if raw_stdout:
                try:
                    parsed = json.loads(raw_stdout)
                except Exception:
                    parsed = {}

            browser_evidence = dict_or_empty(parsed.get("browser_evidence"))
            diagnostics = dict_or_empty(parsed.get("diagnostics"))
            error_code = str(diagnostics.get("error_code") or "")
            self.record_browser_driver_evidence(
                browser_evidence,
                error_code=error_code,
                diagnostics=diagnostics,
            )
            self.request_count += len(list_or_empty(browser_evidence.get("request_lineage")))

            if proc.returncode == 0 and bool(parsed.get("ok")):
                observed_outcome = str(parsed.get("observed_outcome") or "").strip()
                if not observed_outcome:
                    last_error = "browser driver returned success without observed_outcome"
                    if attempt < max_attempts:
                        continue
                    break
                return observed_outcome

            detail = str(parsed.get("detail") or "").strip()
            if not detail:
                detail = collapse_whitespace(raw_stderr or raw_stdout)[:240] or "browser driver failure"
            last_error = (
                f"{detail} (attempt={attempt}/{max_attempts} exit={proc.returncode} "
                f"error_code={error_code or 'none'})"
            )
            retryable = error_code in self.browser_driver_retryable_error_codes
            if attempt < max_attempts and retryable:
                continue
            break

        raise SimulationError(f"browser_realistic_driver_failed action={action} detail={last_error}")

    def driver_allow_browser_allowlist(self, scenario: Dict[str, Any]) -> str:
        return self.execute_browser_realistic_driver(
            scenario,
            action="allow_browser_allowlist",
            headers=self.forwarded_headers(
                self.scenario_ip(scenario),
                user_agent=self.scenario_user_agent(scenario),
            ),
            user_agent=self.scenario_user_agent(scenario),
            inject_trusted_forwarded_secret=True,
        )

    def driver_not_a_bot_pass(self, scenario: Dict[str, Any]) -> str:
        return self.execute_browser_realistic_driver(
            scenario,
            action="not_a_bot_pass",
            headers=self.forwarded_headers(
                self.scenario_ip(scenario),
                user_agent=self.not_a_bot_user_agent(scenario),
            ),
            user_agent=self.not_a_bot_user_agent(scenario),
            inject_trusted_forwarded_secret=True,
        )

    def driver_challenge_puzzle_fail_maze(self, scenario: Dict[str, Any]) -> str:
        return self.execute_browser_realistic_driver(
            scenario,
            action="challenge_puzzle_fail_maze",
            headers=self.forwarded_headers(
                self.scenario_ip(scenario),
                user_agent=self.not_a_bot_user_agent(scenario),
            ),
            user_agent=self.not_a_bot_user_agent(scenario),
            inject_trusted_forwarded_secret=True,
        )

    def driver_pow_success(self, scenario: Dict[str, Any]) -> str:
        seed, difficulty = self.fetch_pow_seed(scenario)
        nonce = solve_pow_nonce(seed, difficulty)
        if nonce < 0:
            raise SimulationError("Failed to solve PoW challenge within iteration budget")
        # Sequence timing guardrail avoids TooFast failures in operation envelope checks.
        time.sleep(2)
        verify = self.submit_pow_verify(seed, str(nonce), scenario)
        headers_lower = lower_headers(verify.headers)
        if verify.status == 200 and "js_verified=" in headers_lower.get("set-cookie", ""):
            return "allow"
        raise SimulationError(
            f"Expected successful pow verify, got status={verify.status} body={collapse_whitespace(verify.body)[:120]}"
        )

    def driver_pow_invalid_proof(self, scenario: Dict[str, Any]) -> str:
        seed, difficulty = self.fetch_pow_seed(scenario)
        bad_nonce = find_invalid_pow_nonce(seed, difficulty)
        if bad_nonce < 0:
            raise SimulationError("Failed to find invalid PoW nonce candidate")
        # Sequence timing guardrail avoids TooFast failures in operation envelope checks.
        time.sleep(2)
        verify = self.submit_pow_verify(seed, str(bad_nonce), scenario)
        if verify.status == 400 and "Invalid proof" in verify.body:
            return "monitor"
        raise SimulationError(
            f"Expected pow verify invalid-proof rejection, got status={verify.status} body={collapse_whitespace(verify.body)[:120]}"
        )

    def driver_rate_limit_enforce(self, scenario: Dict[str, Any]) -> str:
        headers = self.forwarded_headers(
            self.scenario_ip(scenario), user_agent=self.scenario_user_agent(scenario)
        )
        for _ in range(20):
            result = self.attacker_client.request("GET", "/", headers=headers, count_request=True)
            if result.status in {403, 429} or "Rate Limit Exceeded" in result.body or "Access Blocked" in result.body:
                return "deny_temp"
        raise SimulationError("Expected rate limiter enforcement, but requests were not blocked")

    def driver_retry_storm_enforce(self, scenario: Dict[str, Any]) -> str:
        headers = self.forwarded_headers(self.scenario_ip(scenario), user_agent=scenario["user_agent"])
        deny_seen = 0
        for _ in range(40):
            result = self.attacker_client.request("GET", "/", headers=headers, count_request=True)
            if result.status in {403, 429} or "Rate Limit Exceeded" in result.body or "Access Blocked" in result.body:
                deny_seen += 1
                if deny_seen >= 2:
                    return "deny_temp"
        raise SimulationError("invariant_retry_storm_expected_rate_enforcement missing deny response")

    def driver_geo_challenge(self, scenario: Dict[str, Any]) -> str:
        headers = self.forwarded_headers(
            self.scenario_ip(scenario), user_agent=self.scenario_user_agent(scenario)
        )
        headers["X-Geo-Country"] = scenario["geo_country"]
        return self.execute_browser_realistic_driver(
            scenario,
            action="geo_challenge",
            headers=headers,
            user_agent=self.scenario_user_agent(scenario),
            inject_trusted_forwarded_secret=True,
        )

    def driver_geo_maze(self, scenario: Dict[str, Any]) -> str:
        headers = self.forwarded_headers(
            self.scenario_ip(scenario), user_agent=self.scenario_user_agent(scenario)
        )
        headers["X-Geo-Country"] = scenario["geo_country"]
        return self.execute_browser_realistic_driver(
            scenario,
            action="geo_maze",
            headers=headers,
            user_agent=self.scenario_user_agent(scenario),
            inject_trusted_forwarded_secret=True,
        )

    def driver_geo_block(self, scenario: Dict[str, Any]) -> str:
        headers = self.forwarded_headers(
            self.scenario_ip(scenario), user_agent=self.scenario_user_agent(scenario)
        )
        headers["X-Geo-Country"] = scenario["geo_country"]
        return self.execute_browser_realistic_driver(
            scenario,
            action="geo_block",
            headers=headers,
            user_agent=self.scenario_user_agent(scenario),
            inject_trusted_forwarded_secret=True,
        )

    def driver_honeypot_deny_temp(self, scenario: Dict[str, Any]) -> str:
        return self.execute_browser_realistic_driver(
            scenario,
            action="honeypot_deny_temp",
            headers=self.forwarded_headers(
                self.scenario_ip(scenario),
                user_agent=self.scenario_user_agent(scenario),
            ),
            user_agent=self.scenario_user_agent(scenario),
            inject_trusted_forwarded_secret=True,
        )

    def driver_not_a_bot_replay_abuse(self, scenario: Dict[str, Any]) -> str:
        seed, _ = self.fetch_not_a_bot_seed(scenario)
        first_submit = self.submit_not_a_bot(seed, scenario, GOOD_NOT_A_BOT_TELEMETRY)
        if first_submit.status != 303:
            detail = collapse_whitespace(first_submit.body)[:160]
            raise SimulationError(
                "invariant_not_a_bot_prime_failed "
                f"status={first_submit.status} body={detail}"
            )
        replay_submit = self.submit_not_a_bot(seed, scenario, GOOD_NOT_A_BOT_TELEMETRY)
        if replay_submit.status == 200 and 'data-link-kind="maze"' in replay_submit.body:
            return "maze"
        raise SimulationError(
            f"invariant_nonce_replay_expected_maze got status={replay_submit.status}"
        )

    def driver_not_a_bot_stale_token_abuse(self, scenario: Dict[str, Any]) -> str:
        # Keep stale-token simulation black-box: mutate a real issued seed token
        # instead of re-signing with server secrets.
        seed, _ = self.fetch_not_a_bot_seed(scenario)
        stale_like_seed = mutate_token(seed)
        expired_submit = self.submit_not_a_bot(stale_like_seed, scenario, GOOD_NOT_A_BOT_TELEMETRY)
        if expired_submit.status == 200 and 'data-link-kind="maze"' in expired_submit.body:
            return "maze"
        raise SimulationError(
            f"invariant_stale_token_expected_maze got status={expired_submit.status}"
        )

    def driver_not_a_bot_ordering_cadence_abuse(self, scenario: Dict[str, Any]) -> str:
        seed, _ = self.fetch_not_a_bot_seed(scenario)
        abuse_submit = self.submit_not_a_bot(seed, scenario, BAD_ORDERING_NOT_A_BOT_TELEMETRY)
        if abuse_submit.status == 200 and 'data-link-kind="maze"' in abuse_submit.body:
            return "maze"
        raise SimulationError(
            f"invariant_ordering_cadence_expected_maze got status={abuse_submit.status}"
        )

    def driver_not_a_bot_replay_tarpit_abuse(self, scenario: Dict[str, Any]) -> str:
        scenario_id = str(scenario.get("id") or "").strip()
        setup_context = dict_or_empty(self._scenario_setup_context.get(scenario_id))
        seed = str(setup_context.get("tarpit_replay_seed") or "").strip()
        if not seed:
            raise SimulationError(
                f"invariant_tarpit_replay_setup_missing_seed scenario_id={scenario_id}"
            )
        replay_submit = self.submit_not_a_bot(seed, scenario, GOOD_NOT_A_BOT_TELEMETRY)
        if replay_submit.status == 200 and "window.__shumaTarpit=" in replay_submit.body:
            return "tarpit"
        detail = collapse_whitespace(replay_submit.body)[:160]
        raise SimulationError(
            f"invariant_nonce_replay_expected_tarpit status={replay_submit.status} body={detail}"
        )

    def driver_fingerprint_inconsistent_payload(self, scenario: Dict[str, Any]) -> str:
        payload = read_fixture_json(Path(scenario["payload_fixture"]))
        result = self.attacker_client.request(
            "POST",
            "/fingerprint-report",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=scenario["user_agent"]),
            json_body=payload,
            count_request=True,
            trusted_forwarded=True,
        )
        if result.status == 400:
            return "monitor"
        if result.status == 403 and "untrusted source" in result.body.lower():
            return "monitor"
        detail = collapse_whitespace(result.body)[:160]
        raise SimulationError(
            f"invariant_fingerprint_inconsistency_expected_400 status={result.status} body={detail}"
        )

    def driver_header_spoofing_probe(self, scenario: Dict[str, Any]) -> str:
        # Intentionally use forged simulation metadata so runtime keeps this
        # traffic untrusted while still exercising required attacker-lane
        # header presence constraints.
        headers = {
            "X-Forwarded-For": self.scenario_ip(scenario),
            "X-Geo-Country": str(scenario.get("geo_country") or "RU"),
            "User-Agent": str(scenario.get("user_agent") or "ShumaAdversarial/1.0"),
            SIM_TAG_HEADER_RUN_ID: "spoofed-run",
            SIM_TAG_HEADER_PROFILE: "spoof_probe",
            SIM_TAG_HEADER_LANE: "spoofed_lane",
            SIM_TAG_HEADER_TIMESTAMP: str(int(time.time())),
            SIM_TAG_HEADER_NONCE: "spoofed-nonce",
            SIM_TAG_HEADER_SIGNATURE: "0" * 64,
        }
        return self.execute_browser_realistic_driver(
            scenario,
            action="header_spoofing_probe",
            headers=headers,
            user_agent=str(scenario.get("user_agent") or "ShumaAdversarial/1.0"),
            inject_trusted_forwarded_secret=False,
        )

    def driver_cdp_high_confidence_deny(self, scenario: Dict[str, Any]) -> str:
        report = self.attacker_client.request(
            "POST",
            "/cdp-report",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=scenario["user_agent"]),
            json_body={"cdp_detected": True, "score": 0.95, "checks": ["webdriver", "automation_props", "cdp_timing"]},
            count_request=True,
            trusted_forwarded=True,
        )
        if report.status != 200:
            raise SimulationError(f"invariant_cdp_report_expected_200 got status={report.status}")
        followup = self.attacker_client.request(
            "GET",
            "/",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=scenario["user_agent"]),
            count_request=True,
            trusted_forwarded=True,
        )
        if followup.status == 429:
            return "deny_temp"
        if followup.status == 403 and (
            "Access Blocked" in followup.body or "Access Restricted" in followup.body
        ):
            return "deny_temp"
        raise SimulationError(
            f"invariant_cdp_high_confidence_expected_deny got status={followup.status}"
        )

    def driver_akamai_additive_report(self, scenario: Dict[str, Any]) -> str:
        payload = read_fixture_json(Path(scenario["payload_fixture"]))
        report = self.attacker_client.request(
            "POST",
            "/fingerprint-report",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=scenario["user_agent"]),
            json_body=payload,
            count_request=True,
            trusted_forwarded=True,
        )
        if report.status != 200 or "additive" not in report.body.lower():
            raise SimulationError(
                f"Expected additive fingerprint acknowledgement, got status={report.status} body={report.body[:120]}"
            )

        followup = self.attacker_client.request(
            "GET",
            "/",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=scenario["user_agent"]),
            count_request=True,
            trusted_forwarded=True,
        )
        if "Access Blocked" in followup.body or "Access Restricted" in followup.body:
            raise SimulationError("Additive mode unexpectedly blocked follow-up request")
        return "monitor"

    def driver_akamai_authoritative_deny(self, scenario: Dict[str, Any]) -> str:
        payload = read_fixture_json(Path(scenario["payload_fixture"]))
        report = self.attacker_client.request(
            "POST",
            "/fingerprint-report",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=scenario["user_agent"]),
            json_body=payload,
            count_request=True,
            trusted_forwarded=True,
        )
        if report.status != 200 or "banned" not in report.body.lower():
            raise SimulationError(
                f"Expected authoritative ban acknowledgement, got status={report.status} body={report.body[:120]}"
            )

        followup = self.attacker_client.request(
            "GET",
            "/",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=scenario["user_agent"]),
            count_request=True,
            trusted_forwarded=True,
        )
        if followup.status == 429:
            return "deny_temp"
        if followup.status == 403 and (
            "Access Blocked" in followup.body or "Access Restricted" in followup.body
        ):
            return "deny_temp"
        raise SimulationError(f"Expected blocked follow-up after authoritative signal, got {followup.status}")

    def fetch_not_a_bot_seed(self, scenario: Dict[str, Any]) -> Tuple[str, HttpResult]:
        page = self.attacker_client.request(
            "GET",
            "/challenge/not-a-bot-checkbox",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=self.not_a_bot_user_agent(scenario)),
            count_request=True,
            trusted_forwarded=True,
        )
        if page.status != 200 or "I am not a bot" not in page.body:
            raise SimulationError(f"Not-a-Bot page unavailable (status={page.status})")
        match = re.search(r'name="seed" value="([^"]+)"', page.body)
        if not match:
            raise SimulationError("Unable to parse not-a-bot seed")
        return match.group(1), page

    def fetch_challenge_puzzle_seed_and_output(self, scenario: Dict[str, Any]) -> Tuple[str, str]:
        page = self.attacker_client.request(
            "GET",
            "/challenge/puzzle",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=self.not_a_bot_user_agent(scenario)),
            count_request=True,
            trusted_forwarded=True,
        )
        if page.status != 200 or "Puzzle" not in page.body:
            raise SimulationError(f"Challenge puzzle page unavailable (status={page.status})")
        seed_match = re.search(r'name="seed" value="([^"]+)"', page.body)
        output_match = re.search(r'name="output"[^>]*value="([^"]+)"', page.body)
        if not seed_match or not output_match:
            raise SimulationError("Unable to parse challenge puzzle seed/output")
        return seed_match.group(1), output_match.group(1)

    def submit_not_a_bot(self, seed: str, scenario: Dict[str, Any], telemetry: Dict[str, Any]) -> HttpResult:
        form_body = {
            "seed": seed,
            "checked": "on",
            "telemetry": json.dumps(telemetry, separators=(",", ":")),
        }
        return self.attacker_client.request(
            "POST",
            "/challenge/not-a-bot-checkbox",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=self.not_a_bot_user_agent(scenario)),
            form_body=form_body,
            count_request=True,
            trusted_forwarded=True,
        )

    def submit_challenge_puzzle(self, seed: str, output: str, scenario: Dict[str, Any]) -> HttpResult:
        form_body = {"seed": seed, "output": output}
        return self.attacker_client.request(
            "POST",
            "/challenge/puzzle",
            headers=self.forwarded_headers(self.scenario_ip(scenario), user_agent=self.not_a_bot_user_agent(scenario)),
            form_body=form_body,
            count_request=True,
            trusted_forwarded=True,
        )

    def fetch_pow_seed(self, scenario: Dict[str, Any]) -> Tuple[str, int]:
        challenge = self.attacker_client.request(
            "GET",
            "/pow",
            headers=self.forwarded_headers(
                self.scenario_ip(scenario), user_agent=self.pow_user_agent(scenario)
            ),
            count_request=True,
            trusted_forwarded=True,
        )
        if challenge.status != 200:
            raise SimulationError(f"PoW challenge unavailable (status={challenge.status})")
        payload = parse_json_or_raise(challenge.body, "Failed to parse /pow challenge response")
        seed = str(payload.get("seed") or "").strip()
        difficulty = int_or_zero(payload.get("difficulty"))
        if not seed or difficulty <= 0:
            raise SimulationError("PoW challenge response missing seed/difficulty")
        return seed, difficulty

    def submit_pow_verify(self, seed: str, nonce: str, scenario: Dict[str, Any]) -> HttpResult:
        payload = {"seed": seed, "nonce": nonce}
        return self.attacker_client.request(
            "POST",
            "/pow/verify",
            headers=self.forwarded_headers(
                self.scenario_ip(scenario), user_agent=self.pow_user_agent(scenario)
            ),
            json_body=payload,
            count_request=True,
            trusted_forwarded=True,
        )

    def scenario_user_agent(self, scenario: Dict[str, Any], isolate_cadence: bool = False) -> str:
        base = str(scenario.get("user_agent") or "ShumaAdversarial/1.0").strip()
        if not base:
            base = "ShumaAdversarial/1.0"
        if isolate_cadence:
            return f"{base} sim-run/{self.session_nonce}"
        return base

    def not_a_bot_user_agent(self, scenario: Dict[str, Any]) -> str:
        # Isolate cadence buckets per run so repeated local executions do not poison replay tests.
        return self.scenario_user_agent(scenario, isolate_cadence=True)

    def pow_user_agent(self, scenario: Dict[str, Any]) -> str:
        # Isolate PoW cadence buckets per run to avoid stale local history triggering false TooRegular.
        return self.scenario_user_agent(scenario, isolate_cadence=True)

    def admin_get_config(self) -> Dict[str, Any]:
        result = self.admin_request("GET", "/admin/config")
        if result.status != 200:
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(f"Failed to read /admin/config: status={result.status} body={detail}")
        data = parse_json_or_raise(result.body, "Failed to parse /admin/config response")
        return data.get("config") if isinstance(data.get("config"), dict) else data

    def admin_patch(self, payload: Dict[str, Any], reason: str = "admin_config_patch") -> None:
        self.assert_control_plane_mutation_allowed("admin_config_patch", reason)
        self.record_control_plane_mutation(
            action="admin_config_patch",
            reason=reason,
            details={
                "key_count": len(dict_or_empty(payload).keys()),
                "keys": sorted(str(key) for key in dict_or_empty(payload).keys()),
            },
        )
        result = self.admin_request("POST", "/admin/config", json_body=payload)
        if result.status != 200:
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(f"Failed to apply /admin/config patch: status={result.status} body={detail}")
        data = parse_json_or_raise(result.body, "Failed to parse /admin/config patch response")
        if data.get("status") != "updated":
            detail = collapse_whitespace(result.body)[:160]
            raise SimulationError(
                f"Failed to apply /admin/config patch: expected status=updated body={detail}"
            )

    def admin_unban(self, ip: str, reason: str = "admin_unban") -> None:
        self.assert_control_plane_mutation_allowed("admin_unban", reason)
        self.record_control_plane_mutation(
            action="admin_unban",
            reason=reason,
            details={"ip": str(ip or "")},
        )
        query = urllib.parse.urlencode({"ip": ip})
        self.admin_request("POST", f"/admin/unban?{query}")

    def admin_request(
        self,
        method: str,
        path: str,
        json_body: Optional[Dict[str, Any]] = None,
        headers: Optional[Dict[str, str]] = None,
    ) -> HttpResult:
        return self.control_client.request(method, path, headers=headers, json_body=json_body)

    def admin_read_request(
        self,
        method: str,
        path: str,
        json_body: Optional[Dict[str, Any]] = None,
        max_attempts: int = 4,
    ) -> HttpResult:
        attempts = max(1, int(max_attempts))
        last = self.admin_request(method, path, json_body=json_body)
        if last.status != 429:
            return last

        for attempt in range(2, attempts + 1):
            retry_after_seconds = int_or_zero((last.headers or {}).get("retry-after"))
            if retry_after_seconds > 0:
                sleep_seconds = min(2.0, float(retry_after_seconds))
            else:
                sleep_seconds = min(1.0, 0.2 * float(attempt))
            time.sleep(max(0.1, sleep_seconds))
            last = self.admin_request(method, path, json_body=json_body)
            if last.status != 429:
                return last
        return last

    def admin_headers(self) -> Dict[str, str]:
        return self.control_client.admin_headers()

    def forwarded_headers(
        self,
        ip: str,
        user_agent: Optional[str] = None,
    ) -> Dict[str, str]:
        return self.attacker_client.headers(ip, user_agent=user_agent)

    def next_sim_tag_nonce(self) -> str:
        self.sim_tag_nonce_counter += 1
        raw = (
            f"{self.session_nonce}:{self.sim_run_id}:{self.sim_profile}:{self.sim_lane}:"
            f"{self.sim_tag_nonce_counter}"
        )
        return hashlib.sha256(raw.encode("utf-8")).hexdigest()[:24]

    def signed_sim_tag_headers(self) -> Dict[str, str]:
        timestamp = str(int(time.time()))
        nonce = self.next_sim_tag_nonce()
        signature = sign_sim_tag(
            secret=self.sim_telemetry_secret,
            run_id=self.sim_run_id,
            profile=self.sim_profile,
            lane=self.sim_lane,
            timestamp=timestamp,
            nonce=nonce,
        )
        return {
            SIM_TAG_HEADER_TIMESTAMP: timestamp,
            SIM_TAG_HEADER_NONCE: nonce,
            SIM_TAG_HEADER_SIGNATURE: signature,
        }

    def begin_scenario_execution(self, scenario: Dict[str, Any]) -> None:
        traffic_model = scenario.get("traffic_model")
        traffic_model = traffic_model if isinstance(traffic_model, dict) else {}
        policy = {
            "scenario_id": str(scenario.get("id") or ""),
            "seed": int_or_zero(scenario.get("seed")),
            "persona": scenario_persona(scenario),
            "think_time_ms_min": int_or_zero(traffic_model.get("think_time_ms_min")),
            "think_time_ms_max": int_or_zero(traffic_model.get("think_time_ms_max")),
            "retry_strategy": str(traffic_model.get("retry_strategy") or "single_attempt"),
            "cookie_behavior": str(traffic_model.get("cookie_behavior") or "stateless"),
        }
        if policy["retry_strategy"] not in ALLOWED_RETRY_STRATEGIES:
            policy["retry_strategy"] = "single_attempt"
        if policy["cookie_behavior"] not in ALLOWED_COOKIE_BEHAVIORS:
            policy["cookie_behavior"] = "stateless"
        if policy["think_time_ms_max"] < policy["think_time_ms_min"]:
            policy["think_time_ms_max"] = policy["think_time_ms_min"]

        evidence = {
            "request_sequence": 0,
            "attempts_total": 0,
            "retry_attempts": 0,
            "retry_backoff_ms_total": 0,
            "request_latency_ms_total": 0,
            "request_latency_ms_max": 0,
            "think_time_events": 0,
            "think_time_ms_total": 0,
            "cookie_headers_sent": 0,
            "set_cookie_observed": 0,
            "cookie_jar_resets": 0,
            "cookie_jar_peak_size": 0,
            "max_attempts_configured": self.max_attempts_for_retry_strategy(policy["retry_strategy"]),
            "browser_driver_runtime": "",
            "browser_js_executed": False,
            "browser_dom_events": 0,
            "browser_storage_mode": str(policy["cookie_behavior"]),
            "browser_challenge_dom_path": [],
            "browser_correlation_ids": [],
            "browser_request_lineage_count": 0,
            "browser_error_code": "",
            "browser_action_duration_ms": 0,
            "browser_launch_duration_ms": 0,
            "browser_total_duration_ms": 0,
        }
        self._active_execution_state = {
            "policy": policy,
            "evidence": evidence,
            "cookie_jar": {},
        }

    def end_scenario_execution(self) -> Dict[str, Any]:
        state = self._active_execution_state or {}
        policy = dict_or_empty(state.get("policy"))
        evidence = dict_or_empty(state.get("evidence"))
        think_time_events = max(0, int_or_zero(evidence.get("think_time_events")))
        think_time_total = max(0, int_or_zero(evidence.get("think_time_ms_total")))
        attempts_total = max(0, int_or_zero(evidence.get("attempts_total")))
        request_sequence = max(0, int_or_zero(evidence.get("request_sequence")))
        if request_sequence > 0 and think_time_events == 0:
            think_time_events = 1
            think_time_floor = max(1, int_or_zero(policy.get("think_time_ms_min")))
            think_time_total = max(think_time_total, think_time_floor)
        realism = {
            "persona": str(policy.get("persona") or ""),
            "retry_strategy": str(policy.get("retry_strategy") or ""),
            "state_mode": str(policy.get("cookie_behavior") or ""),
            "think_time_ms_min": int_or_zero(policy.get("think_time_ms_min")),
            "think_time_ms_max": int_or_zero(policy.get("think_time_ms_max")),
            "think_time_events": think_time_events,
            "think_time_ms_total": think_time_total,
            "think_time_ms_avg": int(think_time_total / think_time_events) if think_time_events else 0,
            "request_sequence_count": request_sequence,
            "attempts_total": attempts_total,
            "retry_attempts": max(0, int_or_zero(evidence.get("retry_attempts"))),
            "retry_backoff_ms_total": max(0, int_or_zero(evidence.get("retry_backoff_ms_total"))),
            "request_latency_ms_total": max(0, int_or_zero(evidence.get("request_latency_ms_total"))),
            "request_latency_ms_max": max(0, int_or_zero(evidence.get("request_latency_ms_max"))),
            "state_headers_sent": max(0, int_or_zero(evidence.get("cookie_headers_sent"))),
            "state_token_observed": max(0, int_or_zero(evidence.get("set_cookie_observed"))),
            "state_store_resets": max(0, int_or_zero(evidence.get("cookie_jar_resets"))),
            "state_store_peak_size": max(0, int_or_zero(evidence.get("cookie_jar_peak_size"))),
            "max_attempts_configured": max(1, int_or_zero(evidence.get("max_attempts_configured"))),
            "browser_driver_runtime": str(evidence.get("browser_driver_runtime") or ""),
            "browser_js_executed": bool(evidence.get("browser_js_executed")),
            "browser_dom_events": max(0, int_or_zero(evidence.get("browser_dom_events"))),
            "browser_storage_mode": str(
                evidence.get("browser_storage_mode") or policy.get("cookie_behavior") or ""
            ),
            "browser_challenge_dom_path": [
                str(item).strip()
                for item in list_or_empty(evidence.get("browser_challenge_dom_path"))
                if str(item).strip()
            ],
            "browser_correlation_ids": [
                str(item).strip()
                for item in list_or_empty(evidence.get("browser_correlation_ids"))
                if str(item).strip()
            ],
            "browser_request_lineage_count": max(
                0, int_or_zero(evidence.get("browser_request_lineage_count"))
            ),
            "browser_error_code": str(evidence.get("browser_error_code") or ""),
            "browser_action_duration_ms": max(
                0, int_or_zero(evidence.get("browser_action_duration_ms"))
            ),
            "browser_launch_duration_ms": max(
                0, int_or_zero(evidence.get("browser_launch_duration_ms"))
            ),
            "browser_total_duration_ms": max(
                0, int_or_zero(evidence.get("browser_total_duration_ms"))
            ),
        }
        self._active_execution_state = None
        return realism

    def deterministic_execution_value(self, salt: str, modulus: int) -> int:
        if modulus <= 0:
            return 0
        digest = hashlib.sha256(salt.encode("utf-8")).hexdigest()
        return int(digest[:16], 16) % modulus

    def compute_think_time_ms(self, policy: Dict[str, Any], request_sequence: int) -> int:
        minimum = max(0, int_or_zero(policy.get("think_time_ms_min")))
        maximum = max(minimum, int_or_zero(policy.get("think_time_ms_max")))
        if maximum <= 0:
            return 0
        span = (maximum - minimum) + 1
        salt = (
            f"{self.session_nonce}:{policy.get('scenario_id')}:{policy.get('seed')}:"
            f"request:{request_sequence}:think"
        )
        return minimum + self.deterministic_execution_value(salt, span)

    def max_attempts_for_retry_strategy(self, retry_strategy: str) -> int:
        return retry_strategy_max_attempts(retry_strategy)

    def should_retry_status(self, retry_strategy: str, status: int) -> bool:
        if retry_strategy == "retry_storm":
            return status in {403, 408, 425, 429, 500, 502, 503, 504}
        if retry_strategy == "bounded_backoff":
            return status in {408, 425, 500, 502, 503, 504}
        return False

    def compute_retry_backoff_ms(
        self, policy: Dict[str, Any], request_sequence: int, attempt_number: int
    ) -> int:
        retry_strategy = str(policy.get("retry_strategy") or "single_attempt")
        if attempt_number <= 1 or retry_strategy == "single_attempt":
            return 0
        if retry_strategy == "retry_storm":
            base = 5
            jitter_cap = 15
        else:
            base = min(400, 75 * (2 ** (attempt_number - 2)))
            jitter_cap = 30
        salt = (
            f"{self.session_nonce}:{policy.get('scenario_id')}:{policy.get('seed')}:"
            f"request:{request_sequence}:attempt:{attempt_number}:retry"
        )
        jitter = self.deterministic_execution_value(salt, jitter_cap + 1)
        return base + jitter

    def parse_set_cookie_header(self, header_value: str) -> Dict[str, str]:
        raw = str(header_value or "").strip()
        if not raw:
            return {}
        cookie = SimpleCookie()
        try:
            cookie.load(raw)
        except Exception:
            return {}
        parsed: Dict[str, str] = {}
        for key, morsel in cookie.items():
            parsed[key] = morsel.value
        return parsed

    def apply_cookie_policy_to_headers(
        self,
        policy: Dict[str, Any],
        evidence: Dict[str, Any],
        cookie_jar: Dict[str, str],
        request_headers: Dict[str, str],
    ) -> None:
        behavior = str(policy.get("cookie_behavior") or "stateless")
        if behavior == "cookie_reset_each_request":
            cookie_jar.clear()
            evidence["cookie_jar_resets"] = int_or_zero(evidence.get("cookie_jar_resets")) + 1
        if behavior != "stateful_cookie_jar":
            return

        has_cookie_header = any(str(key).strip().lower() == "cookie" for key in request_headers.keys())
        if has_cookie_header or not cookie_jar:
            return
        request_headers["Cookie"] = "; ".join(
            f"{key}={value}" for key, value in sorted(cookie_jar.items())
        )
        evidence["cookie_headers_sent"] = int_or_zero(evidence.get("cookie_headers_sent")) + 1

    def update_cookie_jar_from_response(
        self,
        policy: Dict[str, Any],
        evidence: Dict[str, Any],
        cookie_jar: Dict[str, str],
        result: HttpResult,
    ) -> None:
        behavior = str(policy.get("cookie_behavior") or "stateless")
        if behavior == "stateless":
            return

        set_cookie = str(result.headers.get("set-cookie") or "")
        parsed = self.parse_set_cookie_header(set_cookie)
        if not parsed:
            return

        evidence["set_cookie_observed"] = int_or_zero(evidence.get("set_cookie_observed")) + len(parsed)
        cookie_jar.update(parsed)
        evidence["cookie_jar_peak_size"] = max(
            int_or_zero(evidence.get("cookie_jar_peak_size")),
            len(cookie_jar),
        )

    def attacker_request(
        self,
        method: str,
        path: str,
        headers: Optional[Dict[str, str]] = None,
        json_body: Optional[Dict[str, Any]] = None,
        form_body: Optional[Dict[str, str]] = None,
        count_request: bool = False,
        trusted_forwarded: bool = False,
    ) -> HttpResult:
        state = self._active_execution_state
        if not self.realism_policy_enabled or not state:
            return self.request(
                method,
                path,
                headers=headers,
                json_body=json_body,
                form_body=form_body,
                plane="attacker",
                count_request=count_request,
                trusted_forwarded=trusted_forwarded,
            )

        policy = dict_or_empty(state.get("policy"))
        evidence = dict_or_empty(state.get("evidence"))
        cookie_jar = state.get("cookie_jar")
        if not isinstance(cookie_jar, dict):
            cookie_jar = {}
            state["cookie_jar"] = cookie_jar

        request_headers = dict(headers or {})
        request_sequence = int_or_zero(evidence.get("request_sequence"))
        if request_sequence > 0:
            think_time_ms = self.compute_think_time_ms(policy, request_sequence)
            if think_time_ms > 0:
                time.sleep(think_time_ms / 1000.0)
                evidence["think_time_events"] = int_or_zero(evidence.get("think_time_events")) + 1
                evidence["think_time_ms_total"] = int_or_zero(evidence.get("think_time_ms_total")) + think_time_ms

        self.apply_cookie_policy_to_headers(policy, evidence, cookie_jar, request_headers)

        retry_strategy = str(policy.get("retry_strategy") or "single_attempt")
        max_attempts = self.max_attempts_for_retry_strategy(retry_strategy)
        evidence["max_attempts_configured"] = max_attempts
        attempts_used = 0
        result: Optional[HttpResult] = None
        for attempt in range(1, max_attempts + 1):
            attempts_used = attempt
            if attempt > 1:
                backoff_ms = self.compute_retry_backoff_ms(policy, request_sequence, attempt)
                if backoff_ms > 0:
                    time.sleep(backoff_ms / 1000.0)
                    evidence["retry_backoff_ms_total"] = (
                        int_or_zero(evidence.get("retry_backoff_ms_total")) + backoff_ms
                    )
                evidence["retry_attempts"] = int_or_zero(evidence.get("retry_attempts")) + 1

            result = self.request(
                method,
                path,
                headers=request_headers,
                json_body=json_body,
                form_body=form_body,
                plane="attacker",
                count_request=count_request,
                trusted_forwarded=trusted_forwarded,
            )
            evidence["request_latency_ms_total"] = (
                int_or_zero(evidence.get("request_latency_ms_total")) + max(0, result.latency_ms)
            )
            evidence["request_latency_ms_max"] = max(
                int_or_zero(evidence.get("request_latency_ms_max")),
                max(0, result.latency_ms),
            )
            self.update_cookie_jar_from_response(policy, evidence, cookie_jar, result)
            if not self.should_retry_status(retry_strategy, result.status):
                break

        evidence["request_sequence"] = request_sequence + 1
        evidence["attempts_total"] = int_or_zero(evidence.get("attempts_total")) + attempts_used
        return result if result is not None else HttpResult(0, "", {}, 0)

    def request(
        self,
        method: str,
        path: str,
        headers: Optional[Dict[str, str]] = None,
        json_body: Optional[Dict[str, Any]] = None,
        form_body: Optional[Dict[str, str]] = None,
        plane: str = "attacker",
        count_request: bool = False,
        trusted_forwarded: bool = False,
    ) -> HttpResult:
        if plane not in ALLOWED_REQUEST_PLANES:
            raise SimulationError(f"unknown request plane: {plane}")
        url = path if path.startswith("http://") or path.startswith("https://") else f"{self.base_url}{path}"

        data: Optional[bytes] = None
        request_headers = dict(headers or {})
        if plane == "attacker":
            enforce_attacker_request_contract(path, request_headers)
            if trusted_forwarded and self.forwarded_secret:
                # Inject trusted forwarded secret after attacker-plane contract enforcement so
                # attacker-provided headers remain unprivileged.
                request_headers["X-Shuma-Forwarded-Secret"] = self.forwarded_secret
        if json_body is not None:
            data = json.dumps(json_body, separators=(",", ":")).encode("utf-8")
            request_headers["Content-Type"] = "application/json"
        elif form_body is not None:
            data = urllib.parse.urlencode(form_body).encode("utf-8")
            request_headers["Content-Type"] = "application/x-www-form-urlencoded"

        req = urllib.request.Request(url=url, method=method, data=data)
        for key, value in request_headers.items():
            req.add_header(key, value)

        start = time.monotonic()
        try:
            with self.opener.open(req, timeout=self.request_timeout_seconds) as resp:
                body = resp.read().decode("utf-8", errors="replace")
                headers_map = {k.lower(): v for k, v in resp.headers.items()}
                status = int(resp.getcode() or 0)
        except urllib.error.HTTPError as err:
            body = err.read().decode("utf-8", errors="replace")
            headers_map = {k.lower(): v for k, v in (err.headers.items() if err.headers else [])}
            status = int(err.code)
        except Exception as exc:
            raise SimulationError(f"HTTP request failed for {method} {url}: {exc}") from exc

        latency_ms = int((time.monotonic() - start) * 1000)
        if count_request:
            self.request_count += 1

        return HttpResult(status=status, body=body, headers=headers_map, latency_ms=latency_ms)


def parse_json_or_raise(raw: str, error_message: str) -> Dict[str, Any]:
    try:
        parsed = json.loads(raw)
    except Exception as exc:
        detail = collapse_whitespace(raw)[:160] or "<empty>"
        raise SimulationError(f"{error_message}: {detail}") from exc
    if not isinstance(parsed, dict):
        raise SimulationError(f"{error_message}: response was not a JSON object")
    return parsed


def collapse_whitespace(raw: str) -> str:
    return re.sub(r"\s+", " ", raw).strip()


def build_sim_tag_diagnostics(
    simulation_event_reasons: Optional[List[str]],
    sim_secret_present: bool,
) -> Dict[str, Any]:
    normalized_reasons = [
        str(reason).strip().lower()
        for reason in list_or_empty(simulation_event_reasons)
        if str(reason).strip()
    ]
    sim_tag_reasons = [
        reason for reason in normalized_reasons if "sim_tag_" in reason
    ]
    failure_counts = {
        "missing_secret": 0,
        "missing_required_headers": 0,
        "invalid_header_value": 0,
        "invalid_timestamp": 0,
        "timestamp_skew": 0,
        "signature_mismatch": 0,
        "nonce_replay": 0,
        "other": 0,
    }
    failure_markers = {
        "missing_secret": "sim_tag_missing_secret",
        "missing_required_headers": "sim_tag_missing_required_headers",
        "invalid_header_value": "sim_tag_invalid_header_value",
        "invalid_timestamp": "sim_tag_invalid_timestamp",
        "timestamp_skew": "sim_tag_timestamp_skew",
        "signature_mismatch": "sim_tag_signature_mismatch",
        "nonce_replay": "sim_tag_nonce_replay",
    }
    for reason in sim_tag_reasons:
        matched = False
        for failure_name, marker in failure_markers.items():
            if marker in reason:
                failure_counts[failure_name] += 1
                matched = True
                break
        if not matched:
            failure_counts["other"] += 1

    dominant_failure = "none"
    dominant_count = 0
    for failure_name in (
        "missing_secret",
        "signature_mismatch",
        "nonce_replay",
        "timestamp_skew",
        "invalid_timestamp",
        "missing_required_headers",
        "invalid_header_value",
        "other",
    ):
        count = int_or_zero(failure_counts.get(failure_name))
        if count > dominant_count:
            dominant_failure = failure_name
            dominant_count = count

    guidance: List[str] = []
    if not sim_secret_present:
        guidance.append(
            "Missing SHUMA_SIM_TELEMETRY_SECRET in runner context. Run make test-adversarial-preflight (or make setup) before running adversarial profiles."
        )
    if int_or_zero(failure_counts.get("missing_secret")) > 0:
        guidance.append(
            "Runtime observed sim-tag missing-secret failures. Ensure runtime process has SHUMA_SIM_TELEMETRY_SECRET and restart runtime before rerun."
        )
    if int_or_zero(failure_counts.get("signature_mismatch")) > 0:
        guidance.append(
            "Sim-tag signature mismatches detected. Rotate SHUMA_SIM_TELEMETRY_SECRET, keep runner/runtime in sync, and restart runtime before replay."
        )
    if int_or_zero(failure_counts.get("nonce_replay")) > 0:
        guidance.append(
            "Sim-tag nonce replay detected. Reset runtime-dev state and rerun with fresh envelopes to avoid stale nonce reuse."
        )
    if (
        int_or_zero(failure_counts.get("timestamp_skew")) > 0
        or int_or_zero(failure_counts.get("invalid_timestamp")) > 0
    ):
        guidance.append(
            "Sim-tag timestamp validation failures detected. Verify runner/runtime clock sync and signed timestamp format."
        )
    if (
        int_or_zero(failure_counts.get("missing_required_headers")) > 0
        or int_or_zero(failure_counts.get("invalid_header_value")) > 0
    ):
        guidance.append(
            "Sim-tag header validation failures detected. Re-run make test-adversarial-sim-tag-contract and confirm required headers are present."
        )
    if int_or_zero(failure_counts.get("other")) > 0:
        guidance.append(
            "Unexpected sim-tag validation failures detected. Inspect simulation_event_reasons and policy-signal metrics for new failure variants."
        )
    if not guidance:
        guidance.append(
            "No sim-tag validation failures observed in the run window."
        )

    status = "healthy"
    if not sim_secret_present:
        status = "runner_secret_missing"
    elif sim_tag_reasons:
        status = "validation_failures_detected"

    return {
        "schema_version": "sim-tag-diagnostics.v1",
        "status": status,
        "sim_secret_present": bool(sim_secret_present),
        "sim_tag_reason_count": len(sim_tag_reasons),
        "dominant_failure": dominant_failure,
        "failure_counts": failure_counts,
        "reasons": sorted(set(sim_tag_reasons)),
        "guidance": guidance,
    }


def enforce_attacker_request_contract(path: str, headers: Dict[str, str]) -> None:
    return enforce_attacker_request_contract_impl(
        path,
        headers,
        attacker_forbidden_path_prefixes=ATTACKER_FORBIDDEN_PATH_PREFIXES,
        attacker_forbidden_headers=ATTACKER_FORBIDDEN_HEADERS,
        attacker_required_sim_headers=ATTACKER_REQUIRED_SIM_HEADERS,
    )



def nested_dict_value(data: Dict[str, Any], path: Tuple[str, ...]) -> Any:
    current: Any = data
    for segment in path:
        if not isinstance(current, dict):
            return None
        current = current.get(segment)
    return current


def clamp_int_env(key: str, minimum: int, maximum: int, fallback: int) -> int:
    return clamp_int_env_impl(key, minimum, maximum, fallback)


def scenario_persona(scenario: Dict[str, Any]) -> str:
    return scenario_persona_impl(
        scenario,
        allowed_traffic_personas=ALLOWED_TRAFFIC_PERSONAS,
    )


def compute_cohort_metrics(
    selected_scenarios: List[Dict[str, Any]], results: List[ScenarioResult]
) -> Dict[str, Dict[str, Any]]:
    return compute_cohort_metrics_impl(
        selected_scenarios,
        results,
        scenario_persona_fn=scenario_persona,
    )


def compute_realism_metrics(
    selected_scenarios: List[Dict[str, Any]],
    results: List[ScenarioResult],
    persona_scheduler: str,
) -> Dict[str, Any]:
    return compute_realism_metrics_impl(
        selected_scenarios,
        results,
        persona_scheduler,
        scenario_persona_fn=scenario_persona,
    )


def has_leading_zero_bits(digest: bytes, bits: int) -> bool:
    remaining = max(0, bits)
    for byte in digest:
        if remaining <= 0:
            return True
        if remaining >= 8:
            if byte != 0:
                return False
            remaining -= 8
        else:
            mask = (0xFF << (8 - remaining)) & 0xFF
            return (byte & mask) == 0
    return True


def pow_digest(seed: str, nonce: int) -> bytes:
    return hashlib.sha256(f"{seed}:{nonce}".encode("utf-8")).digest()


def solve_pow_nonce(seed: str, difficulty: int, max_iter: int = 5_000_000) -> int:
    nonce = 0
    while nonce < max_iter:
        digest = pow_digest(seed, nonce)
        if has_leading_zero_bits(digest, difficulty):
            return nonce
        nonce += 1
    return -1


def find_invalid_pow_nonce(seed: str, difficulty: int, max_iter: int = 5_000_000) -> int:
    nonce = 0
    while nonce < max_iter:
        digest = pow_digest(seed, nonce)
        if not has_leading_zero_bits(digest, difficulty):
            return nonce
        nonce += 1
    return -1


def extract_monitoring_snapshot(payload: Dict[str, Any]) -> Dict[str, Any]:
    return extract_monitoring_snapshot_impl(
        payload,
        nested_dict_value_fn=nested_dict_value,
    )


def build_runtime_telemetry_evidence_checks(
    results: List[ScenarioResult],
    scenario_execution_evidence: Dict[str, Dict[str, Any]],
    required_fields: List[str],
) -> List[Dict[str, Any]]:
    return build_runtime_telemetry_evidence_checks_impl(
        results,
        scenario_execution_evidence,
        required_fields,
        real_traffic_contract_path=REAL_TRAFFIC_CONTRACT_PATH,
    )


def build_browser_execution_evidence_checks(
    selected_scenarios: List[Dict[str, Any]],
    results: List[ScenarioResult],
    scenario_execution_evidence: Dict[str, Dict[str, Any]],
) -> List[Dict[str, Any]]:
    return build_browser_execution_evidence_checks_impl(
        selected_scenarios,
        results,
        scenario_execution_evidence,
        scenario_driver_class_fn=scenario_driver_class,
    )


def build_scenario_intent_checks(
    selected_scenarios: List[Dict[str, Any]],
    results: List[ScenarioResult],
    scenario_execution_evidence: Dict[str, Dict[str, Any]],
    intent_rows_by_id: Optional[Dict[str, Dict[str, Any]]] = None,
) -> List[Dict[str, Any]]:
    return build_scenario_intent_checks_impl(
        selected_scenarios,
        results,
        scenario_execution_evidence,
        intent_rows_by_id=intent_rows_by_id or SCENARIO_INTENT_MATRIX_ROWS_BY_ID,
        scenario_driver_class_fn=scenario_driver_class,
        intent_matrix_path=SCENARIO_INTENT_MATRIX_PATH,
        progress_int_keys=SCENARIO_INTENT_PROGRESS_INT_KEYS,
    )


def profile_expected_defense_categories(selected_scenarios: List[Dict[str, Any]]) -> List[str]:
    return profile_expected_defense_categories_impl(
        selected_scenarios,
        defense_noop_coverage_keys=DEFENSE_NOOP_COVERAGE_KEYS,
    )


def build_defense_noop_checks(
    defense_categories: List[str],
    coverage_deltas: Dict[str, int],
    threshold_source_prefix: str = "scenario.expected_defense_categories",
) -> List[Dict[str, Any]]:
    return build_defense_noop_checks_impl(
        defense_categories,
        coverage_deltas,
        defense_noop_coverage_keys=DEFENSE_NOOP_COVERAGE_KEYS,
        threshold_source_prefix=threshold_source_prefix,
    )


def env_or_local(key: str) -> str:
    value = os.environ.get(key)
    if value is not None and value.strip():
        return value.strip()
    return read_env_local_value(key)


def truthy_env(key: str) -> bool:
    value = os.environ.get(key)
    if value is None:
        return False
    return value.strip().lower() in {"1", "true", "yes", "on"}


def read_env_local_value(key: str) -> str:
    env_local = Path(".env.local")
    if not env_local.exists():
        return ""
    for line in env_local.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.startswith(f"{key}="):
            continue
        value = line.split("=", 1)[1].strip()
        value = value.strip('"').strip("'")
        if value:
            return value
    return ""


def scenario_map(manifest: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    mapping: Dict[str, Dict[str, Any]] = {}
    for scenario in manifest["scenarios"]:
        mapping[scenario["id"]] = scenario
    return mapping


def lower_headers(headers: Dict[str, str]) -> Dict[str, str]:
    return {key.lower(): value for key, value in headers.items()}


def mutate_token(token: str) -> str:
    if not token:
        return token
    last = token[-1]
    replacement = "A" if last != "A" else "B"
    return token[:-1] + replacement


def read_fixture_json(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise SimulationError(f"Fixture file not found: {path}")
    raw = path.read_text(encoding="utf-8")
    try:
        parsed = json.loads(raw)
    except Exception as exc:
        raise SimulationError(f"Fixture JSON invalid: {path}") from exc
    if not isinstance(parsed, dict):
        raise SimulationError(f"Fixture JSON must be object: {path}")
    return parsed


def build_frontier_metadata() -> Dict[str, Any]:
    return build_frontier_metadata_impl(env_or_local)


def frontier_path_hint_for_scenario(scenario: Dict[str, Any]) -> str:
    # Path hints are sourced from the shared deterministic corpus contract.
    return frontier_path_hint_for_scenario_impl(
        scenario,
        deterministic_driver_path_hints=DETERMINISTIC_DRIVER_PATH_HINTS,
    )


def build_attack_plan(
    profile_name: str,
    execution_lane: str,
    base_url: str,
    scenarios: List[Dict[str, Any]],
    frontier_metadata: Dict[str, Any],
    generated_at_unix: int,
) -> Dict[str, Any]:
    return build_attack_plan_impl(
        profile_name,
        execution_lane,
        base_url,
        scenarios,
        frontier_metadata,
        generated_at_unix,
        frontier_attack_generation_contract=FRONTIER_ATTACK_GENERATION_CONTRACT,
        frontier_attack_generation_contract_path=FRONTIER_ATTACK_GENERATION_CONTRACT_PATH,
        frontier_attack_generation_contract_sha256=FRONTIER_ATTACK_GENERATION_CONTRACT_SHA256,
        deterministic_attack_corpus=DETERMINISTIC_ATTACK_CORPUS,
        deterministic_attack_corpus_path=DETERMINISTIC_ATTACK_CORPUS_PATH,
        deterministic_attack_corpus_revision=DETERMINISTIC_ATTACK_CORPUS_REVISION,
        deterministic_attack_corpus_taxonomy_version=DETERMINISTIC_ATTACK_CORPUS_TAXONOMY_VERSION,
        sanitize_frontier_payload_fn=sanitize_frontier_payload,
        scenario_driver_class_fn=scenario_driver_class,
        frontier_path_hint_for_scenario_fn=frontier_path_hint_for_scenario,
    )


def scenario_driver_class(scenario: Dict[str, Any]) -> str:
    explicit = scenario.get("driver_class")
    if isinstance(explicit, str) and explicit.strip():
        return explicit.strip()
    driver_name = str(scenario.get("driver") or "")
    class_from_corpus = str(DETERMINISTIC_DRIVER_CLASS_MAP.get(driver_name) or "").strip()
    if class_from_corpus:
        return class_from_corpus
    return DRIVER_TO_CLASS.get(driver_name, "")


def validate_execution_lane(lane: str) -> str:
    return validate_execution_lane_impl(
        lane,
        supported_execution_lanes=SUPPORTED_EXECUTION_LANES,
    )


def normalize_execution_lane(raw_value: Any) -> str:
    return normalize_execution_lane_impl(raw_value)


def effective_scenario_latency_ms(
    scenario: Dict[str, Any], wall_clock_latency_ms: int, realism: Optional[Dict[str, Any]]
) -> int:
    return effective_scenario_latency_ms_impl(
        scenario,
        wall_clock_latency_ms,
        realism,
        scenario_driver_class_fn=scenario_driver_class,
    )


def validate_v2_traffic_model(sid: str, traffic_model: Any) -> None:
    if not isinstance(traffic_model, dict):
        raise SimulationError(f"scenario {sid} traffic_model must be an object")
    required_keys = {
        "persona",
        "think_time_ms_min",
        "think_time_ms_max",
        "retry_strategy",
        "cookie_behavior",
    }
    for key in required_keys:
        if key not in traffic_model:
            raise SimulationError(f"scenario {sid} traffic_model missing key: {key}")

    persona = str(traffic_model.get("persona") or "")
    if persona not in ALLOWED_TRAFFIC_PERSONAS:
        raise SimulationError(f"scenario {sid} traffic_model.persona invalid: {persona}")

    retry_strategy = str(traffic_model.get("retry_strategy") or "")
    if retry_strategy not in ALLOWED_RETRY_STRATEGIES:
        raise SimulationError(f"scenario {sid} traffic_model.retry_strategy invalid: {retry_strategy}")

    cookie_behavior = str(traffic_model.get("cookie_behavior") or "")
    if cookie_behavior not in ALLOWED_COOKIE_BEHAVIORS:
        raise SimulationError(f"scenario {sid} traffic_model.cookie_behavior invalid: {cookie_behavior}")

    think_time_min = traffic_model.get("think_time_ms_min")
    think_time_max = traffic_model.get("think_time_ms_max")
    if not isinstance(think_time_min, int) or think_time_min < 0:
        raise SimulationError(f"scenario {sid} traffic_model.think_time_ms_min must be integer >= 0")
    if not isinstance(think_time_max, int) or think_time_max < think_time_min:
        raise SimulationError(
            f"scenario {sid} traffic_model.think_time_ms_max must be integer >= think_time_ms_min"
        )


def validate_v2_categories(sid: str, key: str, values: Any, allowed_values: set[str]) -> None:
    if not isinstance(values, list) or not values:
        raise SimulationError(f"scenario {sid} {key} must be a non-empty array")
    for raw_value in values:
        value = str(raw_value or "")
        if not value:
            raise SimulationError(f"scenario {sid} {key} must not include empty values")
        if allowed_values and value not in allowed_values:
            raise SimulationError(f"scenario {sid} {key} includes unsupported value: {value}")


def is_canonical_intent_matrix_manifest(manifest_path: Path) -> bool:
    try:
        return manifest_path.resolve() == SCENARIO_INTENT_MATRIX_MANIFEST_PATH.resolve()
    except Exception:
        return str(manifest_path) == str(SCENARIO_INTENT_MATRIX_MANIFEST_PATH)


def validate_scenario_intent_matrix_row_alignment(sid: str, scenario: Dict[str, Any]) -> None:
    row = dict_or_empty(SCENARIO_INTENT_MATRIX_ROWS_BY_ID.get(sid))
    if not row:
        raise SimulationError(f"scenario {sid} missing scenario intent matrix row in {SCENARIO_INTENT_MATRIX_PATH}")
    expected_categories = sorted(
        {
            str(item).strip()
            for item in list_or_empty(scenario.get("expected_defense_categories"))
            if str(item).strip()
        }
    )
    row_categories = sorted(
        {
            str(item).strip()
            for item in list_or_empty(row.get("required_defense_categories"))
            if str(item).strip()
        }
    )
    if row_categories != expected_categories:
        raise SimulationError(
            "scenario intent matrix category mismatch for "
            f"{sid}: manifest={expected_categories} matrix={row_categories}"
        )

    defense_signals = dict_or_empty(row.get("defense_signals"))
    for category in row_categories:
        signal_rules = list_or_empty(defense_signals.get(category))
        if not signal_rules:
            raise SimulationError(
                "scenario intent matrix row "
                f"{sid} missing defense_signals entry for category={category}"
            )


def coverage_contract_parity_diagnostics(profile_coverage_requirements: Any) -> Dict[str, Any]:
    observed = profile_coverage_requirements if isinstance(profile_coverage_requirements, dict) else {}
    observed_normalized = {str(key): int_or_zero(value) for key, value in observed.items()}
    expected = dict(COVERAGE_CONTRACT_REQUIREMENTS)
    expected_keys = set(expected.keys())
    observed_keys = set(observed_normalized.keys())
    missing_keys = sorted(expected_keys - observed_keys)
    extra_keys = sorted(observed_keys - expected_keys)

    mismatched_values: Dict[str, Dict[str, int]] = {}
    for key in sorted(expected_keys.intersection(observed_keys)):
        expected_minimum = int_or_zero(expected.get(key))
        observed_minimum = int_or_zero(observed_normalized.get(key))
        if observed_minimum != expected_minimum:
            mismatched_values[key] = {
                "expected": expected_minimum,
                "observed": observed_minimum,
            }

    return {
        "missing_keys": missing_keys,
        "extra_keys": extra_keys,
        "mismatched_values": mismatched_values,
        "parity_passed": not missing_keys and not extra_keys and not mismatched_values,
    }


def normalize_coverage_depth_requirements(
    raw_depth_requirements: Any,
) -> Dict[str, Dict[str, Any]]:
    if not isinstance(raw_depth_requirements, dict):
        return {}
    normalized: Dict[str, Dict[str, Any]] = {}
    for row_id, row_payload in raw_depth_requirements.items():
        row_name = str(row_id or "").strip()
        if not row_name:
            continue
        row = row_payload if isinstance(row_payload, dict) else {}
        required_metrics = dict(row.get("required_metrics") or {})
        normalized[row_name] = {
            "plan_row": str(row.get("plan_row") or "").strip(),
            "verification_matrix_row_id": str(row.get("verification_matrix_row_id") or "").strip(),
            "required_scenarios": sorted(
                [
                    str(item).strip()
                    for item in list_or_empty(row.get("required_scenarios"))
                    if str(item).strip()
                ]
            ),
            "required_metrics": {
                str(metric): int_or_zero(value)
                for metric, value in required_metrics.items()
            },
            "required_evidence_types": sorted(
                [
                    str(item).strip()
                    for item in list_or_empty(row.get("required_evidence_types"))
                    if str(item).strip()
                ]
            ),
        }
    return normalized


def coverage_depth_contract_parity_diagnostics(profile_depth_requirements: Any) -> Dict[str, Any]:
    expected = normalize_coverage_depth_requirements(COVERAGE_CONTRACT_DEPTH_REQUIREMENTS)
    observed = normalize_coverage_depth_requirements(profile_depth_requirements)
    expected_keys = set(expected.keys())
    observed_keys = set(observed.keys())
    missing_keys = sorted(expected_keys - observed_keys)
    extra_keys = sorted(observed_keys - expected_keys)
    mismatched_values: Dict[str, Dict[str, Any]] = {}
    for row_id in sorted(expected_keys.intersection(observed_keys)):
        expected_row = dict_or_empty(expected.get(row_id))
        observed_row = dict_or_empty(observed.get(row_id))
        if expected_row != observed_row:
            mismatched_values[row_id] = {
                "expected": expected_row,
                "observed": observed_row,
            }
    return {
        "missing_keys": missing_keys,
        "extra_keys": extra_keys,
        "mismatched_values": mismatched_values,
        "parity_passed": not missing_keys and not extra_keys and not mismatched_values,
    }


def select_coverage_requirements(
    profile_name: str, profile_gates: Dict[str, Any]
) -> Tuple[Dict[str, int], Dict[str, Any]]:
    declared = profile_gates.get("coverage_requirements")
    if profile_name == FULL_COVERAGE_PROFILE_NAME:
        return dict(COVERAGE_CONTRACT_REQUIREMENTS), dict(declared or {}) if isinstance(declared, dict) else {}
    return dict(declared or {}) if isinstance(declared, dict) else {}, dict(declared or {}) if isinstance(declared, dict) else {}


def select_coverage_depth_requirements(
    profile_name: str, profile_gates: Dict[str, Any]
) -> Tuple[Dict[str, Dict[str, Any]], Dict[str, Any]]:
    declared = profile_gates.get("coverage_depth_requirements")
    if profile_name == FULL_COVERAGE_PROFILE_NAME:
        return (
            normalize_coverage_depth_requirements(COVERAGE_CONTRACT_DEPTH_REQUIREMENTS),
            normalize_coverage_depth_requirements(declared),
        )
    return normalize_coverage_depth_requirements(declared), normalize_coverage_depth_requirements(declared)


def validate_full_coverage_contract_alignment(profile_name: str, gates: Dict[str, Any]) -> None:
    if profile_name != FULL_COVERAGE_PROFILE_NAME:
        return

    parity = coverage_contract_parity_diagnostics(gates.get("coverage_requirements"))
    if not parity["parity_passed"]:
        details = []
        if parity["missing_keys"]:
            details.append(f"missing={','.join(parity['missing_keys'])}")
        if parity["extra_keys"]:
            details.append(f"extra={','.join(parity['extra_keys'])}")
        if parity["mismatched_values"]:
            mismatches = ",".join(
                f"{key}:{value['observed']}!=expected:{value['expected']}"
                for key, value in parity["mismatched_values"].items()
            )
            details.append(f"mismatched={mismatches}")
        raise SimulationError(
            "profile full_coverage coverage_requirements must exactly match "
            f"{COVERAGE_CONTRACT_PATH} ({'; '.join(details)})"
        )

    depth_parity = coverage_depth_contract_parity_diagnostics(
        gates.get("coverage_depth_requirements")
    )
    if not depth_parity["parity_passed"]:
        details = []
        if depth_parity["missing_keys"]:
            details.append(f"missing_rows={','.join(depth_parity['missing_keys'])}")
        if depth_parity["extra_keys"]:
            details.append(f"extra_rows={','.join(depth_parity['extra_keys'])}")
        if depth_parity["mismatched_values"]:
            details.append(
                "mismatched_rows="
                + ",".join(sorted(depth_parity["mismatched_values"].keys()))
            )
        raise SimulationError(
            "profile full_coverage coverage_depth_requirements must exactly match "
            f"{COVERAGE_CONTRACT_PATH} ({'; '.join(details)})"
        )

    required_event_reasons = gates.get("required_event_reasons")
    normalized_required_reasons = sorted(
        {str(reason or "").strip().lower() for reason in list(required_event_reasons or []) if str(reason or "").strip()}
    )
    expected_required_reasons = sorted(set(COVERAGE_CONTRACT_REQUIRED_EVENT_REASONS))
    if normalized_required_reasons != expected_required_reasons:
        raise SimulationError(
            "profile full_coverage required_event_reasons must exactly match "
            f"{COVERAGE_CONTRACT_PATH}: expected={expected_required_reasons} got={normalized_required_reasons}"
        )

    ip_range_required = gates.get("ip_range_suggestion_seed_required")
    if bool(ip_range_required) != COVERAGE_CONTRACT_IP_RANGE_SUGGESTION_SEED_REQUIRED:
        raise SimulationError(
            "profile full_coverage ip_range_suggestion_seed_required must match "
            f"{COVERAGE_CONTRACT_PATH}: expected={COVERAGE_CONTRACT_IP_RANGE_SUGGESTION_SEED_REQUIRED} got={ip_range_required}"
        )

    ratio_bounds = gates.get("outcome_ratio_bounds")
    ratio_bounds = ratio_bounds if isinstance(ratio_bounds, dict) else {}
    for outcome in COVERAGE_CONTRACT_REQUIRED_OUTCOME_CATEGORIES:
        bounds = ratio_bounds.get(outcome)
        if not isinstance(bounds, dict):
            raise SimulationError(
                "profile full_coverage outcome_ratio_bounds is missing required outcome key "
                f"{outcome} from {COVERAGE_CONTRACT_PATH}"
            )
        minimum = float(bounds.get("min", 0.0))
        if minimum <= 0.0:
            raise SimulationError(
                "profile full_coverage outcome_ratio_bounds "
                f"{outcome}.min must be > 0 to satisfy {COVERAGE_CONTRACT_PATH}"
            )


def validate_manifest(manifest_path: Path, manifest: Dict[str, Any], profile_name: str) -> None:
    schema_version = str(manifest.get("schema_version") or "").strip()
    if schema_version not in SUPPORTED_MANIFEST_SCHEMA_VERSIONS:
        raise SimulationError(
            f"schema_version must be one of {sorted(SUPPORTED_MANIFEST_SCHEMA_VERSIONS)}"
        )
    validate_execution_lane(manifest.get("execution_lane"))
    is_v2_manifest = schema_version == "sim-manifest.v2"
    enforce_intent_matrix = (
        is_v2_manifest
        and is_canonical_intent_matrix_manifest(manifest_path)
        and str(manifest.get("suite_id") or "").strip() == SCENARIO_INTENT_MATRIX_SUITE_ID
    )

    profiles = manifest.get("profiles")
    if not isinstance(profiles, dict) or not profiles:
        raise SimulationError("profiles must be a non-empty object")
    if profile_name not in profiles:
        raise SimulationError(f"profile not found in manifest: {profile_name}")

    scenarios = manifest.get("scenarios")
    if not isinstance(scenarios, list) or not scenarios:
        raise SimulationError("scenarios must be a non-empty array")

    scenario_ids = set()
    for scenario in scenarios:
        if not isinstance(scenario, dict):
            raise SimulationError("each scenario must be an object")
        required = [
            "id",
            "description",
            "tier",
            "driver",
            "expected_outcome",
            "ip",
            "user_agent",
            "seed",
            "runtime_budget_ms",
        ]
        if is_v2_manifest:
            required.extend(
                [
                    "driver_class",
                    "traffic_model",
                    "expected_defense_categories",
                    "coverage_tags",
                    "cost_assertions",
                ]
            )
        else:
            required.append("assertions")
        for key in required:
            if key not in scenario:
                raise SimulationError(f"scenario missing required key: {key}")
        sid = scenario["id"]
        if sid in scenario_ids:
            raise SimulationError(f"duplicate scenario id: {sid}")
        scenario_ids.add(sid)

        if scenario["tier"] not in ALLOWED_TIERS:
            raise SimulationError(f"scenario {sid} has invalid tier: {scenario['tier']}")
        if scenario["driver"] not in ALLOWED_DRIVERS:
            raise SimulationError(f"scenario {sid} has invalid driver: {scenario['driver']}")
        if scenario["expected_outcome"] not in ALLOWED_OUTCOMES:
            raise SimulationError(
                f"scenario {sid} has invalid expected_outcome: {scenario['expected_outcome']}"
            )

        expected_driver_class = DRIVER_TO_CLASS.get(scenario["driver"], "")
        if not expected_driver_class:
            raise SimulationError(f"scenario {sid} has no mapped driver_class for driver={scenario['driver']}")
        resolved_driver_class = scenario_driver_class(scenario)
        if resolved_driver_class not in ALLOWED_DRIVER_CLASSES:
            raise SimulationError(
                f"scenario {sid} driver_class must be one of {sorted(ALLOWED_DRIVER_CLASSES)}"
            )
        if resolved_driver_class != expected_driver_class:
            raise SimulationError(
                f"scenario {sid} driver_class mismatch: expected={expected_driver_class} got={resolved_driver_class}"
            )

        if is_v2_manifest:
            validate_v2_traffic_model(sid, scenario.get("traffic_model"))
            validate_v2_categories(
                sid,
                "expected_defense_categories",
                scenario.get("expected_defense_categories"),
                ALLOWED_DEFENSE_CATEGORIES,
            )
            validate_v2_categories(sid, "coverage_tags", scenario.get("coverage_tags"), set())
            cost_assertions = scenario.get("cost_assertions")
            if not isinstance(cost_assertions, dict) or "max_latency_ms" not in cost_assertions:
                raise SimulationError(f"scenario {sid} cost_assertions.max_latency_ms is required")
            max_latency_ms = cost_assertions.get("max_latency_ms")
            if isinstance(max_latency_ms, bool) or not isinstance(max_latency_ms, int) or max_latency_ms < 1:
                raise SimulationError(
                    f"scenario {sid} cost_assertions.max_latency_ms must be an integer >= 1"
                )
            if enforce_intent_matrix:
                validate_scenario_intent_matrix_row_alignment(sid, scenario)
        else:
            assertions = scenario.get("assertions")
            if not isinstance(assertions, dict) or "max_latency_ms" not in assertions:
                raise SimulationError(f"scenario {sid} assertions.max_latency_ms is required")

        payload_fixture = scenario.get("payload_fixture")
        if payload_fixture:
            fixture_path = (manifest_path.parents[3] / payload_fixture).resolve()
            if not fixture_path.exists():
                # Also allow direct relative-to-repo path.
                fixture_path = Path(payload_fixture)
            if not fixture_path.exists():
                raise SimulationError(f"scenario {sid} references missing payload_fixture: {payload_fixture}")

    profile = profiles[profile_name]
    profile_required = ["description", "max_runtime_seconds", "scenario_ids", "gates"]
    for key in profile_required:
        if key not in profile:
            raise SimulationError(f"profile {profile_name} missing key: {key}")
    if "fail_fast" in profile and not isinstance(profile.get("fail_fast"), bool):
        raise SimulationError(f"profile {profile_name} fail_fast must be a boolean when provided")

    if not isinstance(profile["scenario_ids"], list) or not profile["scenario_ids"]:
        raise SimulationError(f"profile {profile_name} scenario_ids must be non-empty array")
    for sid in profile["scenario_ids"]:
        if sid not in scenario_ids:
            raise SimulationError(f"profile {profile_name} references unknown scenario: {sid}")
        if enforce_intent_matrix and sid not in SCENARIO_INTENT_MATRIX_ROWS_BY_ID:
            raise SimulationError(
                f"profile {profile_name} scenario {sid} missing scenario intent matrix row in {SCENARIO_INTENT_MATRIX_PATH}"
            )

    gates = profile.get("gates")
    if not isinstance(gates, dict):
        raise SimulationError(f"profile {profile_name} gates must be an object")
    if "latency" not in gates or "p95_max_ms" not in (gates.get("latency") or {}):
        raise SimulationError(f"profile {profile_name} must include gates.latency.p95_max_ms")

    ratio_bounds = gates.get("outcome_ratio_bounds")
    if not isinstance(ratio_bounds, dict) or not ratio_bounds:
        raise SimulationError(f"profile {profile_name} must include at least one outcome ratio bound")
    for outcome, bounds in ratio_bounds.items():
        if outcome not in ALLOWED_OUTCOMES:
            raise SimulationError(
                f"profile {profile_name} has unsupported outcome ratio key: {outcome}"
            )
        if not isinstance(bounds, dict) or "min" not in bounds or "max" not in bounds:
            raise SimulationError(f"profile {profile_name} outcome {outcome} must define min and max")
        minimum = float(bounds["min"])
        maximum = float(bounds["max"])
        if minimum < 0.0 or maximum > 1.0 or minimum > maximum:
            raise SimulationError(
                f"profile {profile_name} outcome {outcome} has invalid bounds [{minimum},{maximum}]"
            )

    telemetry = gates.get("telemetry_amplification")
    if not isinstance(telemetry, dict):
        raise SimulationError(f"profile {profile_name} must include telemetry_amplification")
    if "max_fingerprint_events_per_request" not in telemetry or "max_monitoring_events_per_request" not in telemetry:
        raise SimulationError(
                f"profile {profile_name} telemetry_amplification must include fingerprint and monitoring limits"
            )

    persona_scheduler = gates.get("persona_scheduler")
    if persona_scheduler is not None:
        scheduler = str(persona_scheduler).strip().lower()
        if scheduler not in ALLOWED_PERSONA_SCHEDULERS:
            raise SimulationError(
                f"profile {profile_name} persona_scheduler must be one of {sorted(ALLOWED_PERSONA_SCHEDULERS)}"
            )

    realism = gates.get("realism")
    if realism is not None:
        if not isinstance(realism, dict):
            raise SimulationError(
                f"profile {profile_name} realism must be an object when provided"
            )
        enabled = realism.get("enabled")
        if enabled is not None and not isinstance(enabled, bool):
            raise SimulationError(f"profile {profile_name} realism.enabled must be boolean when provided")
        required_retry_attempts = realism.get("required_retry_attempts")
        if required_retry_attempts is not None:
            if not isinstance(required_retry_attempts, dict) or not required_retry_attempts:
                raise SimulationError(
                    f"profile {profile_name} realism.required_retry_attempts must be a non-empty object when provided"
                )
            for strategy, minimum in required_retry_attempts.items():
                normalized_strategy = str(strategy).strip()
                if normalized_strategy not in ALLOWED_RETRY_STRATEGIES:
                    raise SimulationError(
                        f"profile {profile_name} realism.required_retry_attempts has unsupported strategy: {strategy}"
                    )
                if isinstance(minimum, bool) or not isinstance(minimum, int) or minimum < 0:
                    raise SimulationError(
                        f"profile {profile_name} realism.required_retry_attempts.{normalized_strategy} must be integer >= 0"
                    )

    human_like_collateral_max_ratio = gates.get("human_like_collateral_max_ratio")
    if human_like_collateral_max_ratio is not None:
        ratio = float(human_like_collateral_max_ratio)
        if ratio < 0.0 or ratio > 1.0:
            raise SimulationError(
                f"profile {profile_name} human_like_collateral_max_ratio must be within [0,1]"
            )

    required_event_reasons = gates.get("required_event_reasons")
    if required_event_reasons is not None:
        if not isinstance(required_event_reasons, list) or not required_event_reasons:
            raise SimulationError(
                f"profile {profile_name} required_event_reasons must be a non-empty array when provided"
            )
        for raw_reason in required_event_reasons:
            reason = str(raw_reason or "").strip()
            if not reason:
                raise SimulationError(
                    f"profile {profile_name} required_event_reasons must not contain empty values"
                )

    ip_range_suggestion_seed_required = gates.get("ip_range_suggestion_seed_required")
    if ip_range_suggestion_seed_required is not None and not isinstance(
        ip_range_suggestion_seed_required, bool
    ):
        raise SimulationError(
            f"profile {profile_name} ip_range_suggestion_seed_required must be a boolean when provided"
        )

    coverage_requirements = gates.get("coverage_requirements")
    if coverage_requirements is not None:
        if not isinstance(coverage_requirements, dict) or not coverage_requirements:
            raise SimulationError(
                f"profile {profile_name} coverage_requirements must be a non-empty object when provided"
            )
        for key, minimum in coverage_requirements.items():
            if key not in ALLOWED_COVERAGE_REQUIREMENTS:
                raise SimulationError(
                    f"profile {profile_name} has unsupported coverage requirement key: {key}"
                )
            if isinstance(minimum, bool) or not isinstance(minimum, int):
                raise SimulationError(
                    f"profile {profile_name} coverage requirement {key} must be an integer >= 0"
                )
            if minimum < 0:
                raise SimulationError(
                    f"profile {profile_name} coverage requirement {key} cannot be negative"
                )

    coverage_depth_requirements = gates.get("coverage_depth_requirements")
    if coverage_depth_requirements is not None:
        if (
            not isinstance(coverage_depth_requirements, dict)
            or not coverage_depth_requirements
        ):
            raise SimulationError(
                f"profile {profile_name} coverage_depth_requirements must be a non-empty object when provided"
            )
        for row_id, row_payload in coverage_depth_requirements.items():
            row_name = str(row_id or "").strip()
            if not row_name:
                raise SimulationError(
                    f"profile {profile_name} coverage_depth_requirements contains empty row id"
                )
            row = row_payload if isinstance(row_payload, dict) else {}
            required_metrics = row.get("required_metrics")
            if not isinstance(required_metrics, dict) or not required_metrics:
                raise SimulationError(
                    "profile "
                    f"{profile_name} coverage_depth_requirements.{row_name}.required_metrics "
                    "must be a non-empty object"
                )
            for metric_key, minimum in required_metrics.items():
                metric = str(metric_key or "").strip()
                if metric not in ALLOWED_COVERAGE_REQUIREMENTS:
                    raise SimulationError(
                        f"profile {profile_name} coverage_depth row {row_name} has unsupported metric: {metric}"
                    )
                if isinstance(minimum, bool) or not isinstance(minimum, int) or minimum < 0:
                    raise SimulationError(
                        f"profile {profile_name} coverage_depth row {row_name} metric {metric} must be integer >= 0"
                    )
            required_scenarios = row.get("required_scenarios")
            if not isinstance(required_scenarios, list) or not required_scenarios:
                raise SimulationError(
                    "profile "
                    f"{profile_name} coverage_depth_requirements.{row_name}.required_scenarios "
                    "must be a non-empty array"
                )
            for scenario_id in required_scenarios:
                if str(scenario_id or "") not in scenario_ids:
                    raise SimulationError(
                        "profile "
                        f"{profile_name} coverage_depth row {row_name} references unknown scenario "
                        f"{scenario_id}"
                    )

    if profile_name == FULL_COVERAGE_PROFILE_NAME:
        scheduler = str(gates.get("persona_scheduler") or "").strip().lower()
        if scheduler != "round_robin":
            raise SimulationError(
                f"profile {profile_name} persona_scheduler must be round_robin"
            )
        realism = gates.get("realism")
        if not isinstance(realism, dict) or bool(realism.get("enabled", False)) is not True:
            raise SimulationError(
                f"profile {profile_name} realism.enabled must be true"
            )
        required_retry_attempts = dict_or_empty(realism.get("required_retry_attempts"))
        if int_or_zero(required_retry_attempts.get("retry_storm")) < 1:
            raise SimulationError(
                f"profile {profile_name} realism.required_retry_attempts.retry_storm must be >= 1"
            )

    validate_full_coverage_contract_alignment(profile_name, gates)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run deterministic adversarial simulation profiles")
    parser.add_argument(
        "--manifest",
        default="scripts/tests/adversarial/scenario_manifest.v1.json",
        help="Path to adversarial scenario manifest JSON",
    )
    parser.add_argument(
        "--profile",
        default="fast_smoke",
        help="Profile name from manifest profiles object",
    )
    parser.add_argument(
        "--execution-lane",
        default=os.environ.get("ADVERSARIAL_EXECUTION_LANE", ""),
        help="Execution lane contract (must remain black_box in this project phase)",
    )
    parser.add_argument(
        "--base-url",
        default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"),
        help="Base URL for Shuma server",
    )
    parser.add_argument(
        "--request-timeout-seconds",
        type=float,
        default=10.0,
        help="Per-request timeout in seconds",
    )
    parser.add_argument(
        "--report",
        default="scripts/tests/adversarial/latest_report.json",
        help="Path to write simulation report JSON",
    )
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="Validate manifest/profile/fixtures and exit",
    )

    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    if not manifest_path.exists():
        print(f"Manifest not found: {manifest_path}", file=sys.stderr)
        return 2

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"Failed to parse manifest JSON: {exc}", file=sys.stderr)
        return 2

    try:
        validate_manifest(manifest_path, manifest, args.profile)
    except Exception as exc:
        print(f"Manifest validation failed: {exc}", file=sys.stderr)
        return 2

    manifest_lane = normalize_execution_lane(manifest.get("execution_lane"))
    requested_lane = manifest_lane
    if str(args.execution_lane).strip():
        requested_lane = normalize_execution_lane(args.execution_lane)
        if requested_lane != manifest_lane:
            print(
                f"execution_lane override mismatch: manifest={manifest_lane} cli={requested_lane}",
                file=sys.stderr,
            )
            return 2

    try:
        execution_lane = validate_execution_lane(requested_lane)
    except Exception as exc:
        print(f"Execution lane validation failed: {exc}", file=sys.stderr)
        return 2

    if args.validate_only:
        scenario_count = len(manifest["profiles"][args.profile]["scenario_ids"])
        print(
            "Manifest validation PASS: "
            f"profile={args.profile} lane={execution_lane} scenarios={scenario_count} file={manifest_path}"
        )
        return 0

    try:
        runner = Runner(
            manifest_path=manifest_path,
            manifest=manifest,
            profile_name=args.profile,
            execution_lane=execution_lane,
            base_url=args.base_url,
            request_timeout_seconds=args.request_timeout_seconds,
            report_path=Path(args.report),
        )
        return runner.run()
    except SimulationError as exc:
        print(f"Adversarial simulation failed: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
