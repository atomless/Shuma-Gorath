#!/usr/bin/env python3
"""Validate scenario intent matrix parity, freshness, and realism governance."""

from __future__ import annotations

import json
import os
import sys
from datetime import date
from pathlib import Path
from typing import Any, Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

import scripts.tests.adversarial_simulation_runner as sim_runner


class ScenarioIntentMatrixError(Exception):
    pass


def load_json_object(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise ScenarioIntentMatrixError(f"missing file: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise ScenarioIntentMatrixError(f"invalid JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise ScenarioIntentMatrixError(f"expected JSON object: {path}")
    return payload


def parse_iso_date(value: str, label: str) -> date:
    try:
        parts = value.split("-")
        if len(parts) != 3:
            raise ValueError("not YYYY-MM-DD")
        year = int(parts[0])
        month = int(parts[1])
        day = int(parts[2])
        return date(year, month, day)
    except Exception as exc:
        raise ScenarioIntentMatrixError(f"{label} must use YYYY-MM-DD (got {value})") from exc


def resolve_today() -> date:
    override = str(os.environ.get("SCENARIO_INTENT_REVIEW_TODAY") or "").strip()
    if not override:
        return date.today()
    return parse_iso_date(override, "SCENARIO_INTENT_REVIEW_TODAY")


def validate_scenario_intent_matrix(
    matrix: Dict[str, Any] | None = None,
    manifest: Dict[str, Any] | None = None,
    today: date | None = None,
) -> List[str]:
    errors: List[str] = []
    matrix = matrix if isinstance(matrix, dict) else load_json_object(sim_runner.SCENARIO_INTENT_MATRIX_PATH)
    manifest_path = Path(
        str(matrix.get("manifest_path") or sim_runner.SCENARIO_INTENT_MATRIX_MANIFEST_PATH)
    )
    manifest = manifest if isinstance(manifest, dict) else load_json_object(manifest_path)
    today = today or resolve_today()

    matrix_suite_id = str(matrix.get("suite_id") or "").strip()
    manifest_suite_id = str(manifest.get("suite_id") or "").strip()
    if not matrix_suite_id:
        errors.append("scenario intent matrix suite_id must be non-empty")
        return errors
    if matrix_suite_id != manifest_suite_id:
        errors.append(
            f"scenario intent matrix suite_id mismatch: matrix={matrix_suite_id} manifest={manifest_suite_id}"
        )
        return errors

    rows = matrix.get("rows")
    if not isinstance(rows, list) or not rows:
        errors.append("scenario intent matrix rows must be a non-empty array")
        return errors

    governance = matrix.get("review_governance")
    if not isinstance(governance, dict):
        errors.append("scenario intent matrix review_governance must be an object")
        return errors
    stale_after_days = governance.get("stale_after_days")
    if isinstance(stale_after_days, bool) or not isinstance(stale_after_days, int) or stale_after_days < 1:
        errors.append("scenario intent matrix review_governance.stale_after_days must be integer >= 1")
        return errors

    scenario_rows: Dict[str, Dict[str, Any]] = {}
    for row in rows:
        if not isinstance(row, dict):
            errors.append("scenario intent matrix rows must contain objects")
            continue
        scenario_id = str(row.get("scenario_id") or "").strip()
        if not scenario_id:
            errors.append("scenario intent matrix row missing scenario_id")
            continue
        if scenario_id in scenario_rows:
            errors.append(f"scenario intent matrix duplicate scenario_id: {scenario_id}")
            continue
        scenario_rows[scenario_id] = row

    manifest_scenarios = manifest.get("scenarios")
    if not isinstance(manifest_scenarios, list) or not manifest_scenarios:
        errors.append(f"{manifest_path}: scenarios must be non-empty array")
        return errors
    manifest_rows = {
        str(dict(scenario).get("id") or "").strip(): dict(scenario)
        for scenario in manifest_scenarios
        if isinstance(scenario, dict) and str(dict(scenario).get("id") or "").strip()
    }

    missing_rows = sorted(set(manifest_rows.keys()) - set(scenario_rows.keys()))
    extra_rows = sorted(set(scenario_rows.keys()) - set(manifest_rows.keys()))
    if missing_rows:
        errors.append(f"scenario intent matrix missing scenario rows: {', '.join(missing_rows)}")
    if extra_rows:
        errors.append(f"scenario intent matrix has extra rows not in manifest: {', '.join(extra_rows)}")

    signatures: Dict[tuple[str, str, str, str, str], List[str]] = {}
    for scenario_id in sorted(manifest_rows.keys()):
        manifest_scenario = manifest_rows[scenario_id]
        row = scenario_rows.get(scenario_id)
        if not isinstance(row, dict):
            continue

        manifest_categories = sorted(
            {
                str(item).strip()
                for item in list(manifest_scenario.get("expected_defense_categories") or [])
                if str(item).strip()
            }
        )
        matrix_categories = sorted(
            {
                str(item).strip()
                for item in list(row.get("required_defense_categories") or [])
                if str(item).strip()
            }
        )
        if matrix_categories != manifest_categories:
            errors.append(
                "scenario intent category mismatch for "
                f"{scenario_id}: manifest={manifest_categories} matrix={matrix_categories}"
            )

        non_human_category_targets = sorted(
            {
                str(item).strip()
                for item in list(row.get("non_human_category_targets") or [])
                if str(item).strip()
            }
        )
        for category in non_human_category_targets:
            if category not in sim_runner.COVERAGE_CONTRACT_NON_HUMAN_LANE_FULFILLMENT:
                errors.append(
                    f"scenario intent row {scenario_id} has unknown non_human_category_targets value: {category}"
                )

        signals = row.get("defense_signals")
        if not isinstance(signals, dict):
            errors.append(f"scenario intent row {scenario_id} defense_signals must be object")
            continue
        for category in matrix_categories:
            signal_rules = signals.get(category)
            if not isinstance(signal_rules, list) or not signal_rules:
                errors.append(
                    f"scenario intent row {scenario_id} category {category} must define non-empty signal rules"
                )

        progression = row.get("progression_requirements")
        progression = progression if isinstance(progression, dict) else {}
        expected_driver_class = str(manifest_scenario.get("driver_class") or "").strip()
        expected_persona = str(
            dict(manifest_scenario.get("traffic_model") or {}).get("persona") or ""
        ).strip()
        expected_retry = str(
            dict(manifest_scenario.get("traffic_model") or {}).get("retry_strategy") or ""
        ).strip()

        if str(progression.get("required_driver_class") or "").strip() != expected_driver_class:
            errors.append(
                f"scenario intent row {scenario_id} required_driver_class must equal manifest driver_class "
                f"({expected_driver_class})"
            )
        if str(progression.get("required_persona") or "").strip() != expected_persona:
            errors.append(
                f"scenario intent row {scenario_id} required_persona must equal manifest traffic_model.persona "
                f"({expected_persona})"
            )
        if str(progression.get("required_retry_strategy") or "").strip() != expected_retry:
            errors.append(
                f"scenario intent row {scenario_id} required_retry_strategy must equal manifest traffic_model.retry_strategy "
                f"({expected_retry})"
            )
        if expected_retry == "retry_storm":
            minimum_retry_attempts = int(progression.get("min_retry_attempts") or 0)
            if minimum_retry_attempts < 1:
                errors.append(
                    f"scenario intent row {scenario_id} retry_storm requires min_retry_attempts >= 1"
                )

        review = row.get("review")
        review = review if isinstance(review, dict) else {}
        reviewed_on = str(review.get("last_reviewed_on") or "").strip()
        if not reviewed_on:
            errors.append(f"scenario intent row {scenario_id} review.last_reviewed_on is required")
        else:
            reviewed_date = parse_iso_date(reviewed_on, f"scenario {scenario_id} review.last_reviewed_on")
            age_days = (today - reviewed_date).days
            if age_days > stale_after_days:
                errors.append(
                    f"scenario intent row {scenario_id} review is stale by {age_days} days "
                    f"(stale_after_days={stale_after_days})"
                )

        signature = (
            str(manifest_scenario.get("driver") or "").strip(),
            str(manifest_scenario.get("expected_outcome") or "").strip(),
            ",".join(manifest_categories),
            expected_persona,
            expected_retry,
        )
        signatures.setdefault(signature, []).append(scenario_id)

    for signature, scenario_ids in sorted(signatures.items()):
        if len(scenario_ids) > 1:
            errors.append(
                "scenario intent redundancy detected for signature "
                f"{signature}: {', '.join(sorted(scenario_ids))}"
            )

    for category_id, row in sorted(sim_runner.COVERAGE_CONTRACT_NON_HUMAN_LANE_FULFILLMENT.items()):
        for scenario_id in list(dict(row).get("supporting_scenarios") or []):
            scenario_row = scenario_rows.get(str(scenario_id))
            if not isinstance(scenario_row, dict):
                errors.append(
                    f"coverage contract non-human category {category_id} references missing supporting scenario: {scenario_id}"
                )
                continue
            category_targets = {
                str(item).strip()
                for item in list(scenario_row.get("non_human_category_targets") or [])
                if str(item).strip()
            }
            if category_id not in category_targets:
                errors.append(
                    "coverage contract non-human category "
                    f"{category_id} expects supporting scenario {scenario_id} "
                    "to declare the same non_human_category_targets value"
                )

    return errors


def main() -> int:
    try:
        errors = validate_scenario_intent_matrix()
    except ScenarioIntentMatrixError as exc:
        print(f"scenario-intent-matrix validation failed: {exc}")
        return 1
    if errors:
        print("scenario-intent-matrix validation failed:")
        for item in errors:
            print(f"- {item}")
        return 1
    print("scenario-intent-matrix validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
