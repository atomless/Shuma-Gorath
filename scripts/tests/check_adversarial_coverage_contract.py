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


COVERAGE_CONTRACT_PATH = Path("scripts/tests/adversarial/coverage_contract.v1.json")
SIM2_PLAN_PATH = Path("docs/plans/2026-02-26-adversarial-simulation-v2-plan.md")
FULL_COVERAGE_PROFILE = "full_coverage"
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


def validate_coverage_contract() -> List[str]:
    errors: List[str] = []
    contract = load_json_object(COVERAGE_CONTRACT_PATH)

    schema_version = str(contract.get("schema_version") or "").strip()
    if schema_version != "sim-coverage-contract.v1":
        errors.append(
            f"coverage contract schema_version must be sim-coverage-contract.v1 (got {schema_version})"
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

    expected_coverage_requirements = {str(key): int(value) for key, value in coverage_requirements.items()}
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
