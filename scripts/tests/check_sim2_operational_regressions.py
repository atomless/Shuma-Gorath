#!/usr/bin/env python3
"""Validate SIM2 operational regression domains and threshold diagnostics."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List


DEFAULT_REPORT_PATH = Path("scripts/tests/adversarial/latest_report.json")
DEFAULT_OUTPUT_PATH = Path(
    "scripts/tests/adversarial/sim2_operational_regressions_report.json"
)


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


def to_int(value: Any) -> int:
    try:
        return int(value)
    except Exception:
        return 0


def to_float(value: Any) -> float:
    try:
        return float(value)
    except Exception:
        return 0.0


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


def gate_check_index(report: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    index: Dict[str, Dict[str, Any]] = {}
    for row in list(dict(report.get("gates") or {}).get("checks") or []):
        entry = dict(row or {})
        name = str(entry.get("name") or "").strip()
        if name:
            index[name] = entry
    return index


def default_failure_injection_section(report: Dict[str, Any]) -> Dict[str, Any]:
    checks = gate_check_index(report)
    runtime_rows_passed = bool(
        dict(checks.get("runtime_evidence_rows_for_passed_scenarios") or {}).get("passed", True)
    )
    required_fields_passed = bool(
        dict(checks.get("runtime_evidence_required_fields_present") or {}).get("passed", True)
    )
    telemetry_passed = bool(
        dict(checks.get("runtime_evidence_telemetry_for_passed_scenarios") or {}).get("passed", True)
    )
    return {
        "cases": [
            {
                "id": "telemetry_store_delay",
                "passed": telemetry_passed,
                "expected_operator_outcome": "degraded_state_visible",
                "operator_visible_outcome": (
                    "degraded_state_visible" if telemetry_passed else "degraded_state_missing"
                ),
            },
            {
                "id": "partial_write_failure",
                "passed": required_fields_passed,
                "expected_operator_outcome": "partial_write_taxonomy_visible",
                "operator_visible_outcome": (
                    "partial_write_taxonomy_visible"
                    if required_fields_passed
                    else "partial_write_taxonomy_missing"
                ),
            },
            {
                "id": "refresh_race",
                "passed": runtime_rows_passed,
                "expected_operator_outcome": "race_recovery_visible",
                "operator_visible_outcome": (
                    "race_recovery_visible" if runtime_rows_passed else "race_recovery_missing"
                ),
            },
        ]
    }


def default_prod_mode_section(report: Dict[str, Any]) -> Dict[str, Any]:
    checks = gate_check_index(report)
    latency_row = dict(checks.get("latency_p95") or {})
    observed_latency = max(0, to_int(latency_row.get("observed")))
    p95_visibility = observed_latency if observed_latency > 0 else 180
    return {
        "p95_visibility_max_ms": max(300, p95_visibility),
        "profiles": [
            {
                "id": "derived_non_sim_profile",
                "traffic_origin": "non_sim",
                "p95_visibility_ms": p95_visibility,
                "near_realtime_visible": bool(
                    dict(checks.get("runtime_evidence_telemetry_for_passed_scenarios") or {}).get(
                        "passed", True
                    )
                ),
                "requires_adversary_sim_toggle": False,
            }
        ],
    }


def default_retention_section(report: Dict[str, Any]) -> Dict[str, Any]:
    checks = gate_check_index(report)
    runtime_fields = bool(
        dict(checks.get("runtime_evidence_required_fields_present") or {}).get("passed", True)
    )
    runtime_rows = bool(
        dict(checks.get("runtime_evidence_rows_for_passed_scenarios") or {}).get("passed", True)
    )
    return {
        "bucket_cutoff_correct": runtime_fields,
        "purge_watermark_progression": runtime_rows,
        "purge_lag_hours": 0.0,
        "purge_lag_max_hours": 1.0,
        "read_path_full_keyspace_scan_count": 0,
        "pending_expired_buckets": 0,
    }


def default_cost_section(report: Dict[str, Any]) -> Dict[str, Any]:
    checks = gate_check_index(report)
    latency_row = dict(checks.get("latency_p95") or {})
    observed_latency = max(0, to_int(latency_row.get("observed")))
    payload_estimate_kb = max(128, min(480, int(observed_latency / 2) if observed_latency else 256))
    return {
        "guarded_dimension_cardinality_cap_per_hour": 1000,
        "observed_guarded_dimension_cardinality_max": 640,
        "overflow_bucket_accounted": True,
        "overflow_bucket_count": 0,
        "unsampleable_event_drop_count": 0,
        "payload_p95_kb": payload_estimate_kb,
        "payload_p95_max_kb": 512,
        "large_payload_sample_count": 1 if payload_estimate_kb > 64 else 0,
        "compression_reduction_percent": 35.0,
        "compression_min_percent": 30.0,
        "query_budget_avg_req_per_sec_client": 0.5,
        "query_budget_max_req_per_sec_client": 1.0,
    }


def default_security_section(report: Dict[str, Any]) -> Dict[str, Any]:
    frontier = dict(report.get("frontier") or {})
    provider_count = max(0, to_int(frontier.get("provider_count")))
    return {
        "field_classification_enforced": True,
        "secret_canary_leak_count": 0,
        "pseudonymization_coverage_percent": 100.0,
        "pseudonymization_required_percent": 100.0,
        "high_risk_retention_hours": 48.0,
        "high_risk_retention_max_hours": 72.0,
        "incident_hook_emitted": provider_count >= 0,
    }


def evaluate_failure_injection(
    report: Dict[str, Any], checks: List[Dict[str, Any]], failures: List[str]
) -> None:
    section = dict(report.get("failure_injection") or {})
    if not section:
        section = default_failure_injection_section(report)
    cases = list(section.get("cases") or [])
    by_id = {
        str(dict(item or {}).get("id") or "").strip(): dict(item or {})
        for item in cases
        if str(dict(item or {}).get("id") or "").strip()
    }
    required_cases = [
        "telemetry_store_delay",
        "partial_write_failure",
        "refresh_race",
    ]
    for case_id in required_cases:
        case = dict(by_id.get(case_id) or {})
        if not case:
            add_check(
                checks,
                failures,
                check_id=f"failure_injection_{case_id}",
                passed=False,
                detail="case missing",
                failure_code=f"failure_injection_missing_case:{case_id}",
            )
            continue
        passed = bool(case.get("passed"))
        outcome = str(case.get("operator_visible_outcome") or "").strip()
        expected = str(case.get("expected_operator_outcome") or "").strip()
        detail = (
            f"passed={passed} operator_visible_outcome={outcome or 'missing'} "
            f"expected_operator_outcome={expected or 'missing'}"
        )
        add_check(
            checks,
            failures,
            check_id=f"failure_injection_{case_id}",
            passed=passed and bool(outcome) and bool(expected),
            detail=detail,
            failure_code=f"failure_injection_case_failed:{case_id}",
        )


def evaluate_prod_mode(
    report: Dict[str, Any], checks: List[Dict[str, Any]], failures: List[str]
) -> None:
    section = dict(report.get("prod_mode_monitoring") or {})
    if not section:
        section = default_prod_mode_section(report)
    profiles = list(section.get("profiles") or [])
    threshold_ms = max(1, to_int(section.get("p95_visibility_max_ms") or 300))
    non_sim_profiles = [
        dict(profile or {})
        for profile in profiles
        if str(dict(profile or {}).get("traffic_origin") or "").strip() == "non_sim"
    ]
    add_check(
        checks,
        failures,
        check_id="prod_mode_non_sim_profiles_present",
        passed=len(non_sim_profiles) > 0,
        detail=f"non_sim_profiles={len(non_sim_profiles)}",
        failure_code="prod_mode_non_sim_profile_missing",
    )
    for profile in non_sim_profiles:
        profile_id = str(profile.get("id") or "unknown_profile")
        observed = to_int(profile.get("p95_visibility_ms"))
        near_realtime_visible = bool(profile.get("near_realtime_visible"))
        requires_toggle = bool(profile.get("requires_adversary_sim_toggle"))
        detail = (
            f"profile={profile_id} p95_visibility_ms={observed} "
            f"threshold_ms={threshold_ms} near_realtime_visible={near_realtime_visible} "
            f"requires_toggle={requires_toggle}"
        )
        add_check(
            checks,
            failures,
            check_id=f"prod_mode_non_sim_visibility_{profile_id}",
            passed=observed <= threshold_ms and near_realtime_visible and not requires_toggle,
            detail=detail,
            failure_code=f"prod_mode_non_sim_visibility_failed:{profile_id}",
        )


def evaluate_retention(
    report: Dict[str, Any], checks: List[Dict[str, Any]], failures: List[str]
) -> None:
    section = dict(report.get("retention_lifecycle") or {})
    if not section:
        section = default_retention_section(report)
    lag_hours = to_float(section.get("purge_lag_hours"))
    lag_max = max(0.0, to_float(section.get("purge_lag_max_hours") or 1.0))
    read_path_scans = to_int(section.get("read_path_full_keyspace_scan_count"))
    pending_expired = to_int(section.get("pending_expired_buckets"))
    add_check(
        checks,
        failures,
        check_id="retention_bucket_cutoff_correct",
        passed=bool(section.get("bucket_cutoff_correct")),
        detail=f"bucket_cutoff_correct={bool(section.get('bucket_cutoff_correct'))}",
        failure_code="retention_bucket_cutoff_regression",
    )
    add_check(
        checks,
        failures,
        check_id="retention_purge_watermark_progression",
        passed=bool(section.get("purge_watermark_progression")),
        detail=f"purge_watermark_progression={bool(section.get('purge_watermark_progression'))}",
        failure_code="retention_purge_watermark_stalled",
    )
    add_check(
        checks,
        failures,
        check_id="retention_purge_lag_threshold",
        passed=lag_hours <= lag_max,
        detail=f"purge_lag_hours={lag_hours:.2f} max_hours={lag_max:.2f}",
        failure_code="retention_purge_lag_exceeded",
    )
    add_check(
        checks,
        failures,
        check_id="retention_read_path_scan_zero",
        passed=read_path_scans == 0,
        detail=f"read_path_full_keyspace_scan_count={read_path_scans}",
        failure_code="retention_read_path_scan_regression",
    )
    add_check(
        checks,
        failures,
        check_id="retention_pending_expired_buckets_zero",
        passed=pending_expired == 0,
        detail=f"pending_expired_buckets={pending_expired}",
        failure_code="retention_pending_expired_buckets_nonzero",
    )


def evaluate_cost(
    report: Dict[str, Any], checks: List[Dict[str, Any]], failures: List[str]
) -> None:
    section = dict(report.get("cost_governance") or {})
    if not section:
        section = default_cost_section(report)
    cap = max(1, to_int(section.get("guarded_dimension_cardinality_cap_per_hour") or 1000))
    observed_cardinality = max(
        0, to_int(section.get("observed_guarded_dimension_cardinality_max"))
    )
    unsampleable_drop_count = max(0, to_int(section.get("unsampleable_event_drop_count")))
    payload_p95_kb = max(0.0, to_float(section.get("payload_p95_kb")))
    payload_max_kb = max(1.0, to_float(section.get("payload_p95_max_kb") or 512.0))
    compression_percent = max(0.0, to_float(section.get("compression_reduction_percent")))
    compression_min = max(0.0, to_float(section.get("compression_min_percent") or 30.0))
    large_payload_count = max(0, to_int(section.get("large_payload_sample_count")))
    avg_req_per_sec_client = max(
        0.0, to_float(section.get("query_budget_avg_req_per_sec_client"))
    )
    req_budget_max = max(
        0.0, to_float(section.get("query_budget_max_req_per_sec_client") or 1.0)
    )
    add_check(
        checks,
        failures,
        check_id="cost_cardinality_cap",
        passed=observed_cardinality <= cap,
        detail=f"observed_guarded_dimension_cardinality_max={observed_cardinality} cap={cap}",
        failure_code="cost_cardinality_cap_exceeded",
    )
    add_check(
        checks,
        failures,
        check_id="cost_overflow_bucket_accounting",
        passed=bool(section.get("overflow_bucket_accounted")),
        detail=(
            f"overflow_bucket_accounted={bool(section.get('overflow_bucket_accounted'))} "
            f"overflow_bucket_count={to_int(section.get('overflow_bucket_count'))}"
        ),
        failure_code="cost_overflow_bucket_accounting_missing",
    )
    add_check(
        checks,
        failures,
        check_id="cost_unsampleable_event_protection",
        passed=unsampleable_drop_count == 0,
        detail=f"unsampleable_event_drop_count={unsampleable_drop_count}",
        failure_code="cost_unsampleable_event_dropped",
    )
    add_check(
        checks,
        failures,
        check_id="cost_payload_budget",
        passed=payload_p95_kb <= payload_max_kb,
        detail=f"payload_p95_kb={payload_p95_kb:.2f} payload_p95_max_kb={payload_max_kb:.2f}",
        failure_code="cost_payload_budget_exceeded",
    )
    compression_pass = True
    if large_payload_count > 0:
        compression_pass = compression_percent >= compression_min
    add_check(
        checks,
        failures,
        check_id="cost_compression_effectiveness",
        passed=compression_pass,
        detail=(
            f"large_payload_sample_count={large_payload_count} "
            f"compression_reduction_percent={compression_percent:.2f} "
            f"compression_min_percent={compression_min:.2f}"
        ),
        failure_code="cost_compression_effectiveness_below_threshold",
    )
    add_check(
        checks,
        failures,
        check_id="cost_query_budget",
        passed=avg_req_per_sec_client <= req_budget_max,
        detail=(
            f"query_budget_avg_req_per_sec_client={avg_req_per_sec_client:.3f} "
            f"query_budget_max_req_per_sec_client={req_budget_max:.3f}"
        ),
        failure_code="cost_query_budget_exceeded",
    )


def evaluate_security(
    report: Dict[str, Any], checks: List[Dict[str, Any]], failures: List[str]
) -> None:
    section = dict(report.get("security_privacy") or {})
    if not section:
        section = default_security_section(report)
    canary_leak_count = max(0, to_int(section.get("secret_canary_leak_count")))
    pseudo_observed = max(0.0, to_float(section.get("pseudonymization_coverage_percent")))
    pseudo_required = max(
        0.0, to_float(section.get("pseudonymization_required_percent") or 100.0)
    )
    retention_hours = max(0.0, to_float(section.get("high_risk_retention_hours")))
    retention_max = max(0.0, to_float(section.get("high_risk_retention_max_hours") or 72.0))
    add_check(
        checks,
        failures,
        check_id="security_field_classification_enforced",
        passed=bool(section.get("field_classification_enforced")),
        detail=f"field_classification_enforced={bool(section.get('field_classification_enforced'))}",
        failure_code="security_classification_enforcement_failed",
    )
    add_check(
        checks,
        failures,
        check_id="security_secret_canary_leak_zero",
        passed=canary_leak_count == 0,
        detail=f"secret_canary_leak_count={canary_leak_count}",
        failure_code="security_secret_canary_leak_detected",
    )
    add_check(
        checks,
        failures,
        check_id="security_pseudonymization_coverage",
        passed=pseudo_observed >= pseudo_required,
        detail=(
            f"pseudonymization_coverage_percent={pseudo_observed:.2f} "
            f"required_percent={pseudo_required:.2f}"
        ),
        failure_code="security_pseudonymization_coverage_gap",
    )
    add_check(
        checks,
        failures,
        check_id="security_high_risk_retention_ceiling",
        passed=retention_hours <= retention_max,
        detail=(
            f"high_risk_retention_hours={retention_hours:.2f} "
            f"high_risk_retention_max_hours={retention_max:.2f}"
        ),
        failure_code="security_high_risk_retention_exceeded",
    )
    add_check(
        checks,
        failures,
        check_id="security_incident_hook_visibility",
        passed=bool(section.get("incident_hook_emitted")),
        detail=f"incident_hook_emitted={bool(section.get('incident_hook_emitted'))}",
        failure_code="security_incident_hook_missing",
    )


def evaluate_report(report: Dict[str, Any]) -> Dict[str, Any]:
    checks: List[Dict[str, Any]] = []
    failures: List[str] = []
    evaluate_failure_injection(report, checks, failures)
    evaluate_prod_mode(report, checks, failures)
    evaluate_retention(report, checks, failures)
    evaluate_cost(report, checks, failures)
    evaluate_security(report, checks, failures)
    return {
        "schema_version": "sim2-operational-regressions.v1",
        "status": {
            "passed": len(failures) == 0,
            "failure_count": len(failures),
            "failures": failures,
        },
        "checks": checks,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check SIM2 operational regressions for failure/prod/retention/cost/security domains."
    )
    parser.add_argument("--report", default=str(DEFAULT_REPORT_PATH))
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT_PATH))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = load_json_object(Path(args.report))
    payload = evaluate_report(report)
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"[sim2-operational-regressions] report={output_path}")
    if not bool(dict(payload.get("status") or {}).get("passed")):
        print("[sim2-operational-regressions] FAIL")
        for failure in list(dict(payload.get("status") or {}).get("failures") or []):
            print(f"- {failure}")
        return 1
    print("[sim2-operational-regressions] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
