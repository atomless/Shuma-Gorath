#!/usr/bin/env python3
"""Validate SIM2 ADR conformance guardrails for core architecture domains."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List


DEFAULT_OUTPUT_PATH = Path("scripts/tests/adversarial/sim2_adr_conformance_report.json")
DEFAULT_REPORT_PATH = Path("scripts/tests/adversarial/latest_report.json")
DEFAULT_REALTIME_BENCH_PATH = Path(
    "scripts/tests/adversarial/sim2_realtime_bench_report.json"
)

ADR_REQUIREMENTS = [
    {
        "id": "adr_0007",
        "path": "docs/adr/0007-adversary-sim-toggle-command-controller.md",
        "markers": [
            "Trust-Boundary",
            "adversary-sim/control",
            "SIM2-GC-11",
        ],
    },
    {
        "id": "adr_0008",
        "path": "docs/adr/0008-realtime-monitoring-cursor-sse-hybrid.md",
        "markers": [
            "cursor",
            "SSE",
            "SIM2-GC-11",
        ],
    },
    {
        "id": "adr_0009",
        "path": "docs/adr/0009-telemetry-lifecycle-retention-cost-security.md",
        "markers": [
            "retention",
            "cost",
            "security/privacy",
            "SIM2-GC-11",
        ],
    },
]

IMPLEMENTATION_REQUIREMENTS = [
    {
        "id": "api_control_endpoint",
        "path": "src/admin/api.rs",
        "markers": [
            "/admin/adversary-sim/control",
            "validate_origin_and_fetch_metadata",
        ],
    },
    {
        "id": "api_realtime_endpoints",
        "path": "src/admin/api.rs",
        "markers": [
            "/admin/monitoring/delta",
            "/admin/monitoring/stream",
            "strict_monotonic_cursor_ascending",
        ],
    },
    {
        "id": "promotion_hybrid_policy",
        "path": "scripts/tests/adversarial_promote_candidates.py",
        "markers": [
            "HYBRID_CONFIRMATION_MIN_PERCENT",
            "HYBRID_FALSE_DISCOVERY_MAX_PERCENT",
            "HYBRID_OWNER_DISPOSITION_SLA_HOURS",
        ],
    },
]

RETENTION_REQUIRED_FIELDS = {
    "bucket_cutoff_correct",
    "purge_watermark_progression",
    "purge_lag_hours",
    "purge_lag_max_hours",
    "read_path_full_keyspace_scan_count",
    "pending_expired_buckets",
}

COST_REQUIRED_FIELDS = {
    "guarded_dimension_cardinality_cap_per_hour",
    "observed_guarded_dimension_cardinality_max",
    "overflow_bucket_accounted",
    "overflow_bucket_count",
    "unsampleable_event_drop_count",
    "payload_p95_kb",
    "payload_p95_max_kb",
    "large_payload_sample_count",
    "compression_reduction_percent",
    "compression_min_percent",
    "query_budget_avg_req_per_sec_client",
    "query_budget_max_req_per_sec_client",
}

SECURITY_REQUIRED_FIELDS = {
    "field_classification_enforced",
    "secret_canary_leak_count",
    "pseudonymization_coverage_percent",
    "pseudonymization_required_percent",
    "high_risk_retention_hours",
    "high_risk_retention_max_hours",
    "incident_hook_emitted",
}


def check_markers(text: str, markers: List[str]) -> List[str]:
    missing: List[str] = []
    for marker in markers:
        if marker not in text:
            missing.append(marker)
    return missing


def read_text(path: Path) -> str:
    if not path.exists():
        raise FileNotFoundError(str(path))
    return path.read_text(encoding="utf-8")


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


def add_check(
    checks: List[Dict[str, Any]],
    failures: List[str],
    *,
    check_id: str,
    passed: bool,
    detail: str,
    failure_code: str,
    metadata: Dict[str, Any] | None = None,
) -> None:
    entry: Dict[str, Any] = {
        "id": check_id,
        "passed": passed,
        "detail": detail,
    }
    if metadata:
        entry.update(metadata)
    checks.append(entry)
    if not passed:
        failures.append(f"{failure_code}:{detail}")


def evaluate_marker_requirements(repo_root: Path) -> Dict[str, Any]:
    checks: List[Dict[str, Any]] = []
    failures: List[str] = []
    for requirement in ADR_REQUIREMENTS + IMPLEMENTATION_REQUIREMENTS:
        relative_path = Path(requirement["path"])
        absolute_path = repo_root / relative_path
        marker_list = list(requirement["markers"])
        check_id = str(requirement["id"])
        try:
            text = read_text(absolute_path)
            missing = check_markers(text, marker_list)
            passed = len(missing) == 0
            detail = (
                "all markers present"
                if passed
                else "missing markers: " + ", ".join(missing)
            )
        except Exception as exc:
            passed = False
            detail = f"read_error:{exc}"
            missing = marker_list
        add_check(
            checks,
            failures,
            check_id=check_id,
            passed=passed,
            detail=detail,
            failure_code="adr_marker_check_failed",
            metadata={
                "path": str(relative_path),
                "required_markers": marker_list,
                "missing_markers": missing,
            },
        )
    return {"checks": checks, "failures": failures}


def evaluate_realtime_evidence(
    realtime_bench_report: Dict[str, Any],
    checks: List[Dict[str, Any]],
    failures: List[str],
) -> None:
    workload = dict(realtime_bench_report.get("workload") or {})
    events_per_sec = int(workload.get("events_per_sec") or 0)
    operator_clients = int(workload.get("operator_clients") or 0)
    add_check(
        checks,
        failures,
        check_id="evidence_realtime_envelope",
        passed=events_per_sec >= 1000 and operator_clients >= 5,
        detail=f"events_per_sec={events_per_sec} operator_clients={operator_clients}",
        failure_code="adr_0008_evidence_envelope_violation",
    )

    scope = dict(realtime_bench_report.get("verification_scope") or {})
    claims_runtime_prod = bool(scope.get("claims_runtime_prod_verification"))
    add_check(
        checks,
        failures,
        check_id="evidence_realtime_runtime_prod_claim_scope",
        passed=not claims_runtime_prod,
        detail=f"claims_runtime_prod_verification={claims_runtime_prod}",
        failure_code="adr_0008_runtime_prod_claim_invalid",
    )


def evaluate_lifecycle_evidence(
    report: Dict[str, Any],
    checks: List[Dict[str, Any]],
    failures: List[str],
) -> None:
    retention = dict(report.get("retention_lifecycle") or {})
    missing_retention = sorted(RETENTION_REQUIRED_FIELDS - set(retention.keys()))
    add_check(
        checks,
        failures,
        check_id="evidence_retention_required_fields",
        passed=len(missing_retention) == 0,
        detail=(
            "all required fields present"
            if not missing_retention
            else "missing_fields=" + ",".join(missing_retention)
        ),
        failure_code="adr_0009_retention_fields_missing",
    )

    cost = dict(report.get("cost_governance") or {})
    missing_cost = sorted(COST_REQUIRED_FIELDS - set(cost.keys()))
    add_check(
        checks,
        failures,
        check_id="evidence_cost_required_fields",
        passed=len(missing_cost) == 0,
        detail=(
            "all required fields present"
            if not missing_cost
            else "missing_fields=" + ",".join(missing_cost)
        ),
        failure_code="adr_0009_cost_fields_missing",
    )

    security = dict(report.get("security_privacy") or {})
    missing_security = sorted(SECURITY_REQUIRED_FIELDS - set(security.keys()))
    add_check(
        checks,
        failures,
        check_id="evidence_security_required_fields",
        passed=len(missing_security) == 0,
        detail=(
            "all required fields present"
            if not missing_security
            else "missing_fields=" + ",".join(missing_security)
        ),
        failure_code="adr_0009_security_fields_missing",
    )


def evaluate(report: Dict[str, Any], realtime_bench_report: Dict[str, Any], repo_root: Path) -> Dict[str, Any]:
    marker_result = evaluate_marker_requirements(repo_root)
    checks = list(marker_result["checks"])
    failures = list(marker_result["failures"])

    evaluate_realtime_evidence(realtime_bench_report, checks, failures)
    evaluate_lifecycle_evidence(report, checks, failures)

    return {
        "schema_version": "sim2-adr-conformance.v2",
        "checks": checks,
        "status": {
            "passed": len(failures) == 0,
            "failures": failures,
        },
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check SIM2 ADR conformance markers and evidence diagnostics."
    )
    parser.add_argument("--report", default=str(DEFAULT_REPORT_PATH))
    parser.add_argument("--realtime-bench", default=str(DEFAULT_REALTIME_BENCH_PATH))
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT_PATH))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    output_path = Path(args.output)
    repo_root = Path(__file__).resolve().parents[2]
    report = load_json_object(Path(args.report))
    realtime_bench_report = load_json_object(Path(args.realtime_bench))
    payload = evaluate(report, realtime_bench_report, repo_root)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"[sim2-adr-conformance] report={output_path}")
    if not bool(dict(payload.get("status") or {}).get("passed")):
        print("[sim2-adr-conformance] FAIL")
        for failure in list(dict(payload.get("status") or {}).get("failures") or []):
            print(f"- {failure}")
        return 1
    print("[sim2-adr-conformance] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
