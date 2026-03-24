#!/usr/bin/env python3
"""Validate full-coverage contract parity across plan, manifest, and runner."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

import scripts.tests.adversarial_simulation_runner as sim_runner


COVERAGE_CONTRACT_PATHS = (
    Path("scripts/tests/adversarial/coverage_contract.v2.json"),
    Path("scripts/tests/adversarial/coverage_contract.v1.json"),
)
SIM2_PLAN_PATH = Path("docs/plans/2026-02-26-adversarial-simulation-v2-plan.md")
VERIFICATION_MATRIX_PATH = Path("scripts/tests/adversarial/verification_matrix.v1.json")
FULL_COVERAGE_PROFILE = "full_coverage"
EXPECTED_NON_HUMAN_CATEGORIES = {
    "indexing_bot",
    "ai_scraper_bot",
    "automated_browser",
    "http_agent",
    "browser_agent",
    "agent_on_behalf_of_human",
    "verified_beneficial_bot",
    "unknown_non_human",
}
EXPECTED_SCRAPLING_OWNED_DEFENSE_SURFACES = {
    "honeypot": {
        "runtime_requirement": "request_native",
        "success_contract": "must_fail_or_escalate",
    },
    "rate_limit": {
        "runtime_requirement": "request_native",
        "success_contract": "must_touch",
    },
    "geo_ip_policy": {
        "runtime_requirement": "request_native",
        "success_contract": "must_touch",
    },
    "challenge_routing": {
        "runtime_requirement": "request_native",
        "success_contract": "must_touch",
    },
    "not_a_bot": {
        "runtime_requirement": "request_native",
        "success_contract": "must_fail_or_escalate",
    },
    "challenge_puzzle": {
        "runtime_requirement": "request_native",
        "success_contract": "must_fail_or_escalate",
    },
    "proof_of_work": {
        "runtime_requirement": "request_native",
        "success_contract": "must_fail_or_escalate",
    },
}
MANIFEST_PATHS = [
    Path("scripts/tests/adversarial/scenario_manifest.v1.json"),
    Path("scripts/tests/adversarial/scenario_manifest.v2.json"),
]


class CoverageContractError(Exception):
    pass


def load_json_object(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise CoverageContractError(f"missing file: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise CoverageContractError(f"invalid JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise CoverageContractError(f"expected JSON object: {path}")
    return payload


def resolve_contract_path() -> Path:
    for path in COVERAGE_CONTRACT_PATHS:
        if path.exists():
            return path
    raise CoverageContractError(
        "coverage contract not found: expected one of "
        + ", ".join(str(path) for path in COVERAGE_CONTRACT_PATHS)
    )


def parse_plan_contract_rows(path: Path = SIM2_PLAN_PATH) -> List[str]:
    if not path.exists():
        raise CoverageContractError(f"plan file not found: {path}")
    rows: List[str] = []
    lines = path.read_text(encoding="utf-8").splitlines()
    in_table = False
    for raw_line in lines:
        line = raw_line.strip()
        if not in_table and line == "| Category | Evidence Source | Gate |":
            in_table = True
            continue
        if not in_table:
            continue
        if not line.startswith("|"):
            break
        if line.startswith("|---"):
            continue
        parts = [part.strip() for part in line.split("|")]
        if len(parts) < 4:
            continue
        category = parts[1]
        if category:
            rows.append(category)
    if not rows:
        raise CoverageContractError(
            f"unable to parse SIM2 coverage contract table rows from plan: {path}"
        )
    return rows


def compare_sets(name: str, expected: set[str], observed: set[str], errors: List[str]) -> None:
    missing = sorted(expected - observed)
    extra = sorted(observed - expected)
    if missing:
        errors.append(f"{name}: missing entries: {', '.join(missing)}")
    if extra:
        errors.append(f"{name}: unexpected entries: {', '.join(extra)}")


def compare_maps(
    name: str, expected: Dict[str, int], observed: Dict[str, Any], errors: List[str]
) -> None:
    compare_sets(name, set(expected.keys()), set(observed.keys()), errors)
    for key in sorted(set(expected.keys()).intersection(set(observed.keys()))):
        observed_value = observed.get(key)
        if isinstance(observed_value, bool) or not isinstance(observed_value, int):
            errors.append(f"{name}: key {key} must be integer (got {type(observed_value).__name__})")
            continue
        if int(observed_value) != int(expected[key]):
            errors.append(
                f"{name}: key {key} expected minimum={expected[key]} got={observed_value}"
            )


def normalize_depth_requirements(raw: Any) -> Dict[str, Dict[str, Any]]:
    if not isinstance(raw, dict):
        return {}
    normalized: Dict[str, Dict[str, Any]] = {}
    for row_id, row_payload in raw.items():
        row_name = str(row_id or "").strip()
        if not row_name:
            continue
        row = dict(row_payload or {})
        normalized[row_name] = {
            "plan_row": str(row.get("plan_row") or "").strip(),
            "verification_matrix_row_id": str(row.get("verification_matrix_row_id") or "").strip(),
            "required_scenarios": sorted(
                [
                    str(item).strip()
                    for item in list(row.get("required_scenarios") or [])
                    if str(item).strip()
                ]
            ),
            "required_metrics": {
                str(metric): int(value)
                for metric, value in dict(row.get("required_metrics") or {}).items()
            },
            "required_evidence_types": sorted(
                [
                    str(item).strip()
                    for item in list(row.get("required_evidence_types") or [])
                    if str(item).strip()
                ]
            ),
        }
    return normalized


def validate_coverage_contract() -> List[str]:
    errors: List[str] = []
    contract_path = resolve_contract_path()
    contract = load_json_object(contract_path)

    schema_version = str(contract.get("schema_version") or "").strip()
    if schema_version not in {"sim-coverage-contract.v1", "sim-coverage-contract.v2"}:
        errors.append(
            "coverage contract schema_version must be sim-coverage-contract.v1 or "
            f"sim-coverage-contract.v2 (got {schema_version})"
        )

    profile = str(contract.get("profile") or "").strip()
    if profile != FULL_COVERAGE_PROFILE:
        errors.append(
            f"coverage contract profile must be {FULL_COVERAGE_PROFILE} (got {profile})"
        )

    coverage_requirements = contract.get("coverage_requirements")
    if not isinstance(coverage_requirements, dict) or not coverage_requirements:
        errors.append("coverage contract coverage_requirements must be a non-empty object")
        return errors
    for key, minimum in coverage_requirements.items():
        if key not in sim_runner.ALLOWED_COVERAGE_REQUIREMENTS:
            errors.append(f"coverage contract has unsupported coverage requirement key: {key}")
        if isinstance(minimum, bool) or not isinstance(minimum, int) or minimum < 0:
            errors.append(f"coverage contract key {key} must be integer minimum >= 0")

    required_event_reasons = contract.get("required_event_reasons")
    if not isinstance(required_event_reasons, list) or not required_event_reasons:
        errors.append("coverage contract required_event_reasons must be a non-empty array")
        required_event_reasons = []
    normalized_required_event_reasons = sorted(
        {
            str(reason or "").strip().lower()
            for reason in required_event_reasons
            if str(reason or "").strip()
        }
    )

    required_outcome_categories = contract.get("required_outcome_categories")
    if required_outcome_categories is None:
        required_outcome_categories = []
    if not isinstance(required_outcome_categories, list):
        errors.append("coverage contract required_outcome_categories must be an array")
        required_outcome_categories = []
    for outcome in required_outcome_categories:
        if str(outcome or "").strip() not in sim_runner.ALLOWED_OUTCOMES:
            errors.append(
                f"coverage contract has unsupported required_outcome_categories value: {outcome}"
            )

    ip_range_required = contract.get("ip_range_suggestion_seed_required")
    if not isinstance(ip_range_required, bool):
        errors.append("coverage contract ip_range_suggestion_seed_required must be boolean")

    plan_contract_rows = contract.get("plan_contract_rows")
    if not isinstance(plan_contract_rows, list) or not plan_contract_rows:
        errors.append("coverage contract plan_contract_rows must be a non-empty array")
        plan_contract_rows = []
    normalized_plan_rows = [str(row or "").strip() for row in plan_contract_rows if str(row or "").strip()]
    if len(normalized_plan_rows) != len(plan_contract_rows):
        errors.append("coverage contract plan_contract_rows must not contain empty values")

    parsed_plan_rows = parse_plan_contract_rows(SIM2_PLAN_PATH)
    if normalized_plan_rows != parsed_plan_rows:
        errors.append(
            "coverage contract plan_contract_rows drift from SIM2 plan table "
            f"({SIM2_PLAN_PATH}) expected={parsed_plan_rows} got={normalized_plan_rows}"
        )

    expected_depth_requirements = normalize_depth_requirements(
        contract.get("coverage_depth_requirements")
    )
    if schema_version == "sim-coverage-contract.v2" and not expected_depth_requirements:
        errors.append("coverage contract v2 coverage_depth_requirements must be a non-empty object")
    for row_id, row in expected_depth_requirements.items():
        required_metrics = dict(row.get("required_metrics") or {})
        if not required_metrics:
            errors.append(f"coverage_depth_requirements.{row_id}.required_metrics must be non-empty")
        for metric_key, minimum in required_metrics.items():
            if metric_key not in sim_runner.ALLOWED_COVERAGE_REQUIREMENTS:
                errors.append(
                    f"coverage_depth_requirements.{row_id} has unsupported metric key: {metric_key}"
                )
            if minimum < 0:
                errors.append(
                    f"coverage_depth_requirements.{row_id}.{metric_key} minimum must be >= 0"
                )
        if not row.get("required_scenarios"):
            errors.append(
                f"coverage_depth_requirements.{row_id}.required_scenarios must be non-empty"
            )
        if not row.get("verification_matrix_row_id"):
            errors.append(
                f"coverage_depth_requirements.{row_id}.verification_matrix_row_id must be non-empty"
            )

    expected_coverage_requirements = {str(key): int(value) for key, value in coverage_requirements.items()}
    non_human_lane_fulfillment = contract.get("non_human_lane_fulfillment")
    if not isinstance(non_human_lane_fulfillment, dict):
        errors.append("coverage contract non_human_lane_fulfillment must be an object")
        non_human_lane_fulfillment = {}
    if str(non_human_lane_fulfillment.get("schema_version") or "").strip() != "sim-non-human-lane-fulfillment.v1":
        errors.append(
            "coverage contract non_human_lane_fulfillment.schema_version must be "
            "sim-non-human-lane-fulfillment.v1"
        )
    category_rows = dict(non_human_lane_fulfillment.get("categories") or {})
    compare_sets(
        "coverage contract non_human_lane_fulfillment categories",
        EXPECTED_NON_HUMAN_CATEGORIES,
        set(category_rows.keys()),
        errors,
    )
    for category_id, row_payload in category_rows.items():
        row = dict(row_payload or {})
        assignment_status = str(row.get("assignment_status") or "").strip()
        if assignment_status not in {"mapped", "gap"}:
            errors.append(
                "coverage contract non_human_lane_fulfillment.categories."
                f"{category_id}.assignment_status must be mapped or gap"
            )
        runtime_lane = str(row.get("runtime_lane") or "").strip()
        fulfillment_mode = str(row.get("fulfillment_mode") or "").strip()
        if assignment_status == "mapped" and (not runtime_lane or not fulfillment_mode):
            errors.append(
                "coverage contract non_human_lane_fulfillment.categories."
                f"{category_id} must define runtime_lane and fulfillment_mode when mapped"
            )
        supporting_scenarios = row.get("supporting_scenarios")
        if supporting_scenarios is None:
            supporting_scenarios = []
        if not isinstance(supporting_scenarios, list):
            errors.append(
                "coverage contract non_human_lane_fulfillment.categories."
                f"{category_id}.supporting_scenarios must be an array"
            )
        notes = str(row.get("notes") or "").strip()
        if not notes:
            errors.append(
                "coverage contract non_human_lane_fulfillment.categories."
                f"{category_id}.notes must be non-empty"
            )

    beneficial_bot_row = dict(category_rows.get("verified_beneficial_bot") or {})
    if str(beneficial_bot_row.get("assignment_status") or "").strip() != "gap":
        errors.append(
            "coverage contract verified_beneficial_bot must remain an explicit gap in this tranche"
        )
    unknown_non_human_row = dict(category_rows.get("unknown_non_human") or {})
    if str(unknown_non_human_row.get("assignment_status") or "").strip() != "gap":
        errors.append(
            "coverage contract unknown_non_human must remain an explicit gap in this tranche"
        )
    indexing_bot_row = dict(category_rows.get("indexing_bot") or {})
    if str(indexing_bot_row.get("runtime_lane") or "").strip() != "scrapling_traffic":
        errors.append(
            "coverage contract indexing_bot must map to scrapling_traffic in this tranche"
        )
    if str(indexing_bot_row.get("fulfillment_mode") or "").strip() != "crawler":
        errors.append(
            "coverage contract indexing_bot must map to crawler in this tranche"
        )
    ai_scraper_bot_row = dict(category_rows.get("ai_scraper_bot") or {})
    if str(ai_scraper_bot_row.get("runtime_lane") or "").strip() != "scrapling_traffic":
        errors.append(
            "coverage contract ai_scraper_bot must map to scrapling_traffic in this tranche"
        )
    if str(ai_scraper_bot_row.get("fulfillment_mode") or "").strip() != "bulk_scraper":
        errors.append(
            "coverage contract ai_scraper_bot must map to bulk_scraper in this tranche"
        )
    http_agent_row = dict(category_rows.get("http_agent") or {})
    if str(http_agent_row.get("runtime_lane") or "").strip() != "scrapling_traffic":
        errors.append(
            "coverage contract http_agent must map to scrapling_traffic in this tranche"
        )
    if str(http_agent_row.get("fulfillment_mode") or "").strip() != "http_agent":
        errors.append(
            "coverage contract http_agent must map to http_agent in this tranche"
        )

    owned_surfaces = contract.get("scrapling_owned_defense_surfaces")
    if not isinstance(owned_surfaces, dict):
        errors.append("coverage contract scrapling_owned_defense_surfaces must be an object")
        owned_surfaces = {}
    if (
        str(owned_surfaces.get("schema_version") or "").strip()
        != "sim-scrapling-owned-defense-surfaces.v1"
    ):
        errors.append(
            "coverage contract scrapling_owned_defense_surfaces.schema_version must be "
            "sim-scrapling-owned-defense-surfaces.v1"
        )
    owned_surface_rows = dict(owned_surfaces.get("surfaces") or {})
    compare_sets(
        "coverage contract scrapling_owned_defense_surfaces surfaces",
        set(EXPECTED_SCRAPLING_OWNED_DEFENSE_SURFACES.keys()),
        set(owned_surface_rows.keys()),
        errors,
    )
    for surface_id, expected in EXPECTED_SCRAPLING_OWNED_DEFENSE_SURFACES.items():
        row = dict(owned_surface_rows.get(surface_id) or {})
        runtime_requirement = str(row.get("runtime_requirement") or "").strip()
        if runtime_requirement not in {"request_native", "browser_or_stealth", "assigned_elsewhere"}:
            errors.append(
                "coverage contract scrapling_owned_defense_surfaces.surfaces."
                f"{surface_id}.runtime_requirement must be request_native, "
                "browser_or_stealth, or assigned_elsewhere"
            )
        interaction_requirement = str(row.get("interaction_requirement") or "").strip()
        if interaction_requirement not in {"must_touch", "must_avoid"}:
            errors.append(
                "coverage contract scrapling_owned_defense_surfaces.surfaces."
                f"{surface_id}.interaction_requirement must be must_touch or must_avoid"
            )
        success_contract = str(row.get("success_contract") or "").strip()
        if success_contract not in {
            "must_touch",
            "must_fail_or_escalate",
            "must_pass_when_publicly_solved",
        }:
            errors.append(
                "coverage contract scrapling_owned_defense_surfaces.surfaces."
                f"{surface_id}.success_contract must be must_touch, "
                "must_fail_or_escalate, or must_pass_when_publicly_solved"
            )
        if runtime_requirement != expected["runtime_requirement"]:
            errors.append(
                "coverage contract scrapling_owned_defense_surfaces.surfaces."
                f"{surface_id}.runtime_requirement expected={expected['runtime_requirement']} "
                f"got={runtime_requirement}"
            )
        if success_contract != expected["success_contract"]:
            errors.append(
                "coverage contract scrapling_owned_defense_surfaces.surfaces."
                f"{surface_id}.success_contract expected={expected['success_contract']} "
                f"got={success_contract}"
            )
        notes = str(row.get("notes") or "").strip()
        if not notes:
            errors.append(
                "coverage contract scrapling_owned_defense_surfaces.surfaces."
                f"{surface_id}.notes must be non-empty"
            )

    verification_matrix = load_json_object(VERIFICATION_MATRIX_PATH)
    matrix_rows = {
        str(dict(row or {}).get("row_id") or "").strip(): dict(row or {})
        for row in list(verification_matrix.get("rows") or [])
        if str(dict(row or {}).get("row_id") or "").strip()
    }
    for row_id, row in expected_depth_requirements.items():
        matrix_row_id = str(row.get("verification_matrix_row_id") or "")
        matrix_row = dict(matrix_rows.get(matrix_row_id) or {})
        if not matrix_row:
            errors.append(
                f"verification_matrix drift: missing row_id for depth requirement {row_id} -> {matrix_row_id}"
            )
            continue
        matrix_required_scenarios = sorted(
            [
                str(item).strip()
                for item in list(matrix_row.get("required_scenarios") or [])
                if str(item).strip()
            ]
        )
        if matrix_required_scenarios != list(row.get("required_scenarios") or []):
            errors.append(
                "verification_matrix drift: required_scenarios mismatch for "
                f"{row_id} expected={row.get('required_scenarios')} "
                f"got={matrix_required_scenarios}"
            )
        matrix_evidence = sorted(
            [
                str(item).strip()
                for item in list(matrix_row.get("required_evidence_types") or [])
                if str(item).strip()
            ]
        )
        if matrix_evidence != list(row.get("required_evidence_types") or []):
            errors.append(
                "verification_matrix drift: required_evidence_types mismatch for "
                f"{row_id} expected={row.get('required_evidence_types')} "
                f"got={matrix_evidence}"
            )
    for manifest_path in MANIFEST_PATHS:
        manifest = load_json_object(manifest_path)
        profiles = manifest.get("profiles")
        if not isinstance(profiles, dict) or FULL_COVERAGE_PROFILE not in profiles:
            errors.append(
                f"{manifest_path}: missing profiles.{FULL_COVERAGE_PROFILE}"
            )
            continue
        profile_config = profiles[FULL_COVERAGE_PROFILE]
        gates = profile_config.get("gates") if isinstance(profile_config, dict) else None
        if not isinstance(gates, dict):
            errors.append(f"{manifest_path}: profiles.{FULL_COVERAGE_PROFILE}.gates must be object")
            continue

        manifest_requirements = gates.get("coverage_requirements")
        if not isinstance(manifest_requirements, dict):
            errors.append(
                f"{manifest_path}: {FULL_COVERAGE_PROFILE} coverage_requirements missing"
            )
            continue
        compare_maps(
            f"{manifest_path} coverage_requirements parity",
            expected_coverage_requirements,
            manifest_requirements,
            errors,
        )

        manifest_depth_requirements = normalize_depth_requirements(
            gates.get("coverage_depth_requirements")
        )
        if expected_depth_requirements:
            if manifest_depth_requirements != expected_depth_requirements:
                errors.append(
                    f"{manifest_path}: coverage_depth_requirements drift "
                    f"expected={expected_depth_requirements} got={manifest_depth_requirements}"
                )

        manifest_required_reasons = gates.get("required_event_reasons")
        manifest_normalized_reasons = sorted(
            {
                str(reason or "").strip().lower()
                for reason in list(manifest_required_reasons or [])
                if str(reason or "").strip()
            }
        )
        if manifest_normalized_reasons != normalized_required_event_reasons:
            errors.append(
                f"{manifest_path}: required_event_reasons drift "
                f"expected={normalized_required_event_reasons} got={manifest_normalized_reasons}"
            )

        manifest_ip_range_required = gates.get("ip_range_suggestion_seed_required")
        if manifest_ip_range_required is not ip_range_required:
            errors.append(
                f"{manifest_path}: ip_range_suggestion_seed_required drift "
                f"expected={ip_range_required} got={manifest_ip_range_required}"
            )

        outcome_ratio_bounds = gates.get("outcome_ratio_bounds")
        outcome_ratio_bounds = outcome_ratio_bounds if isinstance(outcome_ratio_bounds, dict) else {}
        for outcome in required_outcome_categories:
            key = str(outcome or "").strip()
            bounds = outcome_ratio_bounds.get(key)
            if not isinstance(bounds, dict):
                errors.append(
                    f"{manifest_path}: outcome_ratio_bounds missing required key from contract: {key}"
                )
                continue
            minimum = float(bounds.get("min", 0.0))
            if minimum <= 0.0:
                errors.append(
                    f"{manifest_path}: outcome_ratio_bounds.{key}.min must be > 0 (got {minimum})"
                )

    return errors


def main() -> int:
    try:
        errors = validate_coverage_contract()
    except CoverageContractError as exc:
        print(f"coverage-contract validation failed: {exc}")
        return 1
    if errors:
        print("coverage-contract validation failed:")
        for item in errors:
            print(f"- {item}")
        return 1
    print("coverage-contract validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
