#!/usr/bin/env python3
"""Validate SIM2 governance + hybrid-lane contract conformance."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests.adversarial_artifact_paths import (
    SIM2_OPERATIONAL_REGRESSIONS_REPORT_PATH,
    SIM2_REALTIME_BENCH_REPORT_PATH,
)

DEFAULT_CONTRACT_PATH = Path("scripts/tests/adversarial/hybrid_lane_contract.v1.json")
DEFAULT_PROMOTION_SCRIPT_PATH = Path("scripts/tests/adversarial_promote_candidates.py")
DEFAULT_OPERATOR_GUIDE_PATH = Path("docs/adversarial-operator-guide.md")
DEFAULT_OPERATIONAL_REPORT_PATH = SIM2_OPERATIONAL_REGRESSIONS_REPORT_PATH
DEFAULT_REALTIME_REPORT_PATH = SIM2_REALTIME_BENCH_REPORT_PATH
DEFAULT_MATRIX_REPORT_PATH = Path(
    "scripts/tests/adversarial/sim2_verification_matrix_report.json"
)
DEFAULT_OUTPUT_PATH = Path("scripts/tests/adversarial/sim2_governance_contract_report.json")


def load_json_object(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"missing JSON artifact: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid JSON artifact: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError(f"JSON artifact must be object: {path}")
    return payload


def load_text(path: Path) -> str:
    if not path.exists():
        raise RuntimeError(f"missing text artifact: {path}")
    return path.read_text(encoding="utf-8")


def to_int(value: Any) -> int:
    try:
        return int(value)
    except Exception:
        return 0


def add_check(
    checks: List[Dict[str, Any]],
    failures: List[str],
    *,
    check_id: str,
    passed: bool,
    detail: str,
    failure_code: str,
) -> None:
    checks.append({"id": check_id, "passed": passed, "detail": detail})
    if not passed:
        failures.append(f"{failure_code}:{detail}")


def parse_promotion_constant(text: str, constant_name: str) -> float | None:
    pattern = rf"^{constant_name}\s*=\s*([0-9]+(?:\.[0-9]+)?)\s*$"
    match = re.search(pattern, text, flags=re.MULTILINE)
    if not match:
        return None
    try:
        return float(match.group(1))
    except Exception:
        return None


def parse_promotion_bool(text: str, variable_name: str) -> bool | None:
    pattern = rf"^{variable_name}\s*=\s*(True|False)\s*$"
    match = re.search(pattern, text, flags=re.MULTILINE)
    if match:
        return match.group(1) == "True"
    dict_key_pattern = rf"\"{variable_name}\"\s*:\s*(True|False)"
    dict_match = re.search(dict_key_pattern, text)
    if not dict_match:
        return None
    return dict_match.group(1) == "True"


def evaluate_contract(contract: Dict[str, Any]) -> Dict[str, Any]:
    checks: List[Dict[str, Any]] = []
    failures: List[str] = []
    deterministic_lane = dict(contract.get("deterministic_conformance_lane") or {})
    emergent_lane = dict(contract.get("emergent_exploration_lane") or {})
    choreography = dict(contract.get("choreography_boundary") or {})
    objective_model = dict(contract.get("objective_model") or {})
    novelty = dict(contract.get("novelty_scoring") or {})
    promotion_pipeline = dict(contract.get("promotion_pipeline") or {})
    thresholds = dict(contract.get("promotion_thresholds") or {})
    governance = dict(contract.get("program_governance") or {})
    cadence = dict(governance.get("cadence") or {})
    ownership = dict(governance.get("ownership") or {})
    rollback = dict(governance.get("rollback_playbook") or {})
    architecture_review = dict(governance.get("architecture_review") or {})

    add_check(
        checks,
        failures,
        check_id="contract_schema_version",
        passed=str(contract.get("schema_version") or "") == "sim2-hybrid-lane-contract.v1",
        detail=f"schema_version={contract.get('schema_version')}",
        failure_code="governance_contract_schema_invalid",
    )
    add_check(
        checks,
        failures,
        check_id="deterministic_lane_blocking",
        passed=bool(deterministic_lane.get("release_blocking")),
        detail=f"release_blocking={deterministic_lane.get('release_blocking')}",
        failure_code="hybrid_lane_deterministic_blocking_not_enforced",
    )
    runtime_budget = to_int(emergent_lane.get("runtime_budget_seconds_max"))
    action_budget = to_int(emergent_lane.get("action_budget_max"))
    add_check(
        checks,
        failures,
        check_id="emergent_lane_non_blocking",
        passed=not bool(emergent_lane.get("release_blocking")),
        detail=f"release_blocking={emergent_lane.get('release_blocking')}",
        failure_code="hybrid_lane_emergent_blocking_forbidden",
    )
    add_check(
        checks,
        failures,
        check_id="emergent_lane_budget_envelope",
        passed=runtime_budget <= 180 and action_budget <= 500,
        detail=f"runtime_budget_seconds_max={runtime_budget} action_budget_max={action_budget}",
        failure_code="hybrid_lane_budget_envelope_invalid",
    )
    choreographed = {
        str(item).strip()
        for item in list(choreography.get("intentionally_choreographed") or [])
        if str(item).strip()
    }
    emergent = {
        str(item).strip()
        for item in list(choreography.get("must_be_emergent") or [])
        if str(item).strip()
    }
    add_check(
        checks,
        failures,
        check_id="choreography_boundary_defined",
        passed={
            "seed_scenarios",
            "invariant_assertions",
            "resource_guardrails",
        }.issubset(choreographed)
        and {"crawl_strategy", "attack_sequencing", "adaptation"}.issubset(emergent),
        detail=f"choreographed={sorted(choreographed)} emergent={sorted(emergent)}",
        failure_code="hybrid_lane_choreography_boundary_missing",
    )
    add_check(
        checks,
        failures,
        check_id="objective_model_present",
        passed=all(
            bool(list(objective_model.get(key) or []))
            for key in (
                "target_assets",
                "success_functions",
                "allowed_adaptation_space",
                "stop_conditions",
            )
        ),
        detail="objective model keys=target_assets/success_functions/allowed_adaptation_space/stop_conditions",
        failure_code="hybrid_lane_objective_model_incomplete",
    )
    novelty_dimensions = {
        str(item).strip()
        for item in list(novelty.get("dimensions") or [])
        if str(item).strip()
    }
    add_check(
        checks,
        failures,
        check_id="novelty_scoring_dimensions",
        passed={"novelty", "severity", "confidence", "replayability"}.issubset(
            novelty_dimensions
        ),
        detail=f"dimensions={sorted(novelty_dimensions)}",
        failure_code="hybrid_lane_novelty_dimensions_missing",
    )
    pipeline_steps = {
        str(item).strip()
        for item in list(promotion_pipeline.get("steps") or [])
        if str(item).strip()
    }
    add_check(
        checks,
        failures,
        check_id="promotion_pipeline_contract",
        passed={
            "generated_candidate",
            "deterministic_replay_confirmation",
            "owner_review_disposition",
            "promoted_blocking_scenario",
        }.issubset(pipeline_steps)
        and bool(promotion_pipeline.get("blocking_requires_deterministic_confirmation")),
        detail=(
            f"steps={sorted(pipeline_steps)} "
            "blocking_requires_deterministic_confirmation="
            f"{promotion_pipeline.get('blocking_requires_deterministic_confirmation')}"
        ),
        failure_code="hybrid_lane_promotion_pipeline_invalid",
    )
    add_check(
        checks,
        failures,
        check_id="promotion_thresholds",
        passed=to_int(thresholds.get("deterministic_confirmation_min_percent")) >= 95
        and to_int(thresholds.get("false_discovery_max_percent")) <= 20
        and to_int(thresholds.get("owner_disposition_sla_hours")) <= 48,
        detail=(
            "thresholds="
            f"confirmation_min={thresholds.get('deterministic_confirmation_min_percent')} "
            f"false_discovery_max={thresholds.get('false_discovery_max_percent')} "
            f"owner_disposition_sla_hours={thresholds.get('owner_disposition_sla_hours')}"
        ),
        failure_code="hybrid_lane_thresholds_invalid",
    )
    add_check(
        checks,
        failures,
        check_id="governance_cadence",
        passed=str(cadence.get("frequency") or "").strip().lower() == "weekly"
        and "run -> review -> tune -> replay -> promote"
        in str(cadence.get("cycle") or ""),
        detail=f"cadence={cadence}",
        failure_code="governance_cadence_missing",
    )
    add_check(
        checks,
        failures,
        check_id="governance_ownership",
        passed=all(
            bool(str(ownership.get(key) or "").strip())
            for key in (
                "adversary_owner_role",
                "defense_owner_role",
                "operations_owner_role",
            )
        ),
        detail=f"ownership={ownership}",
        failure_code="governance_ownership_incomplete",
    )
    rubric_dims = {
        str(item).strip()
        for item in list(governance.get("promotion_rubric_dimensions") or [])
        if str(item).strip()
    }
    add_check(
        checks,
        failures,
        check_id="promotion_rubric_dimensions",
        passed={"severity", "reproducibility", "collateral_risk", "mitigation_readiness"}.issubset(
            rubric_dims
        ),
        detail=f"promotion_rubric_dimensions={sorted(rubric_dims)}",
        failure_code="governance_promotion_rubric_missing",
    )
    kpis = {
        str(item).strip()
        for item in list(governance.get("kpis") or [])
        if str(item).strip()
    }
    add_check(
        checks,
        failures,
        check_id="governance_kpis",
        passed={
            "attacker_cost_shift",
            "human_friction_impact",
            "detection_latency",
            "mitigation_lead_time",
            "time_to_regression_confirmation",
            "time_to_mitigation",
            "collateral_ceiling",
            "cost_asymmetry_trend",
        }.issubset(kpis),
        detail=f"kpis={sorted(kpis)}",
        failure_code="governance_kpis_missing",
    )
    rollback_actions = {
        str(item).strip()
        for item in list(rollback.get("required_actions") or [])
        if str(item).strip()
    }
    add_check(
        checks,
        failures,
        check_id="rollback_playbook",
        passed="rollback_to_last_known_good" in rollback_actions
        and "validate_with_adversarial_fast" in rollback_actions,
        detail=f"rollback_actions={sorted(rollback_actions)}",
        failure_code="governance_rollback_playbook_missing",
    )
    add_check(
        checks,
        failures,
        check_id="architecture_review_checkpoint",
        passed=str(architecture_review.get("frequency") or "").strip().lower()
        in {"monthly", "every_month"}
        and bool(architecture_review.get("documented_outcomes_required")),
        detail=f"architecture_review={architecture_review}",
        failure_code="governance_architecture_review_missing",
    )
    return {"checks": checks, "failures": failures}


def evaluate_promotion_script_alignment(
    contract: Dict[str, Any], promotion_script: str
) -> Dict[str, Any]:
    checks: List[Dict[str, Any]] = []
    failures: List[str] = []
    thresholds = dict(contract.get("promotion_thresholds") or {})
    required_confirmation = float(
        thresholds.get("deterministic_confirmation_min_percent") or 95.0
    )
    max_false_discovery = float(thresholds.get("false_discovery_max_percent") or 20.0)
    max_owner_sla_hours = float(thresholds.get("owner_disposition_sla_hours") or 48.0)

    observed_confirmation = parse_promotion_constant(
        promotion_script, "HYBRID_CONFIRMATION_MIN_PERCENT"
    )
    observed_false_discovery = parse_promotion_constant(
        promotion_script, "HYBRID_FALSE_DISCOVERY_MAX_PERCENT"
    )
    observed_owner_sla = parse_promotion_constant(
        promotion_script, "HYBRID_OWNER_DISPOSITION_SLA_HOURS"
    )
    observed_blocking_flag = parse_promotion_bool(
        promotion_script, "blocking_requires_deterministic_confirmation"
    )

    add_check(
        checks,
        failures,
        check_id="promotion_script_confirmation_threshold",
        passed=observed_confirmation is not None
        and observed_confirmation >= required_confirmation,
        detail=(
            f"required_min={required_confirmation:.2f} "
            f"observed={observed_confirmation if observed_confirmation is not None else 'missing'}"
        ),
        failure_code="governance_promotion_threshold_mismatch",
    )
    add_check(
        checks,
        failures,
        check_id="promotion_script_false_discovery_threshold",
        passed=observed_false_discovery is not None
        and observed_false_discovery <= max_false_discovery,
        detail=(
            f"required_max={max_false_discovery:.2f} "
            f"observed={observed_false_discovery if observed_false_discovery is not None else 'missing'}"
        ),
        failure_code="governance_promotion_threshold_mismatch",
    )
    add_check(
        checks,
        failures,
        check_id="promotion_script_owner_sla_threshold",
        passed=observed_owner_sla is not None
        and observed_owner_sla <= max_owner_sla_hours,
        detail=(
            f"required_max_hours={max_owner_sla_hours:.2f} "
            f"observed={observed_owner_sla if observed_owner_sla is not None else 'missing'}"
        ),
        failure_code="governance_promotion_threshold_mismatch",
    )
    add_check(
        checks,
        failures,
        check_id="promotion_script_deterministic_blocking_flag",
        passed=observed_blocking_flag is True,
        detail=(
            "blocking_requires_deterministic_confirmation="
            f"{observed_blocking_flag if observed_blocking_flag is not None else 'missing'}"
        ),
        failure_code="governance_promotion_blocking_flag_invalid",
    )
    return {"checks": checks, "failures": failures}


def evaluate_operator_guide_markers(operator_guide: str) -> Dict[str, Any]:
    checks: List[Dict[str, Any]] = []
    failures: List[str] = []

    guide_markers = [
        "## Run-to-Run Diff + Backlog Automation (SIM2-EX8-2 / SIM2-EX8-3)",
        "## Promotion Hygiene and Scenario Corpus Maintenance (SIM2-EX8-4)",
        "## Continuous Defender-Adversary Evolution Cadence (SIM2-GC-12)",
        "## Hybrid Adversary Lane Contract (SIM2-GC-14)",
        "<=180s",
        "<=500 actions",
        "time to regression confirmation",
        "time to mitigation",
        "collateral_ceiling",
        "cost_asymmetry_trend",
    ]
    for marker in guide_markers:
        add_check(
            checks,
            failures,
            check_id=f"operator_guide_marker_{marker}",
            passed=marker in operator_guide,
            detail=f"marker={marker}",
            failure_code="governance_operator_guide_marker_missing",
        )

    return {"checks": checks, "failures": failures}


def evaluate_gate_artifacts(
    operational_report: Dict[str, Any],
    realtime_report: Dict[str, Any],
    matrix_report: Dict[str, Any],
) -> Dict[str, Any]:
    checks: List[Dict[str, Any]] = []
    failures: List[str] = []

    operational_status = dict(operational_report.get("status") or {})
    add_check(
        checks,
        failures,
        check_id="operational_report_status_passed",
        passed=bool(operational_status.get("passed")),
        detail=f"operational_status_passed={bool(operational_status.get('passed'))}",
        failure_code="governance_operational_artifact_failed",
    )

    realtime_status = dict(realtime_report.get("status") or {})
    add_check(
        checks,
        failures,
        check_id="realtime_report_status_passed",
        passed=bool(realtime_status.get("passed")),
        detail=f"realtime_status_passed={bool(realtime_status.get('passed'))}",
        failure_code="governance_realtime_artifact_failed",
    )

    matrix_status = dict(matrix_report.get("status") or {})
    add_check(
        checks,
        failures,
        check_id="verification_matrix_status_passed",
        passed=bool(matrix_status.get("passed")),
        detail=f"matrix_status_passed={bool(matrix_status.get('passed'))}",
        failure_code="governance_matrix_artifact_failed",
    )

    return {"checks": checks, "failures": failures}


def evaluate(
    contract: Dict[str, Any],
    promotion_script: str,
    operator_guide: str,
    operational_report: Dict[str, Any],
    realtime_report: Dict[str, Any],
    matrix_report: Dict[str, Any],
) -> Dict[str, Any]:
    contract_result = evaluate_contract(contract)
    promotion_result = evaluate_promotion_script_alignment(contract, promotion_script)
    guide_result = evaluate_operator_guide_markers(operator_guide)
    artifact_result = evaluate_gate_artifacts(
        operational_report, realtime_report, matrix_report
    )
    checks = (
        list(contract_result["checks"])
        + list(promotion_result["checks"])
        + list(guide_result["checks"])
        + list(artifact_result["checks"])
    )
    failures = (
        list(contract_result["failures"])
        + list(promotion_result["failures"])
        + list(guide_result["failures"])
        + list(artifact_result["failures"])
    )
    return {
        "schema_version": "sim2-governance-contract-report.v1",
        "status": {
            "passed": len(failures) == 0,
            "failure_count": len(failures),
            "failures": failures,
        },
        "checks": checks,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check SIM2 governance and hybrid-lane contract markers."
    )
    parser.add_argument("--contract", default=str(DEFAULT_CONTRACT_PATH))
    parser.add_argument("--promotion-script", default=str(DEFAULT_PROMOTION_SCRIPT_PATH))
    parser.add_argument("--operator-guide", default=str(DEFAULT_OPERATOR_GUIDE_PATH))
    parser.add_argument("--operational-report", default=str(DEFAULT_OPERATIONAL_REPORT_PATH))
    parser.add_argument("--realtime-report", default=str(DEFAULT_REALTIME_REPORT_PATH))
    parser.add_argument("--matrix-report", default=str(DEFAULT_MATRIX_REPORT_PATH))
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT_PATH))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    contract = load_json_object(Path(args.contract))
    promotion_script = load_text(Path(args.promotion_script))
    operator_guide = load_text(Path(args.operator_guide))
    operational_report = load_json_object(Path(args.operational_report))
    realtime_report = load_json_object(Path(args.realtime_report))
    matrix_report = load_json_object(Path(args.matrix_report))
    payload = evaluate(
        contract,
        promotion_script,
        operator_guide,
        operational_report,
        realtime_report,
        matrix_report,
    )
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"[sim2-governance-contract] report={output_path}")
    if not bool(dict(payload.get("status") or {}).get("passed")):
        print("[sim2-governance-contract] FAIL")
        for failure in list(dict(payload.get("status") or {}).get("failures") or []):
            print(f"- {failure}")
        return 1
    print("[sim2-governance-contract] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
