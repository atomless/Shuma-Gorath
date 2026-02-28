#!/usr/bin/env python3
"""Validate SIM2 verification matrix rows against manifest/report evidence."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List, Tuple


DEFAULT_MATRIX_PATH = Path("scripts/tests/adversarial/verification_matrix.v1.json")
DEFAULT_MANIFEST_PATH = Path("scripts/tests/adversarial/scenario_manifest.v2.json")
DEFAULT_REPORT_PATH = Path("scripts/tests/adversarial/latest_report.json")
DEFAULT_CONTAINER_REPORT_PATH = Path(
    "scripts/tests/adversarial/container_blackbox_report.json"
)
DEFAULT_OUTPUT_PATH = Path("scripts/tests/adversarial/sim2_verification_matrix_report.json")


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


def expected_manifest_categories(manifest: Dict[str, Any]) -> List[str]:
    categories = {
        str(category).strip()
        for scenario in list(manifest.get("scenarios") or [])
        for category in list(dict(scenario or {}).get("expected_defense_categories") or [])
        if str(category).strip()
    }
    return sorted(categories)


def result_index(report: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    rows: Dict[str, Dict[str, Any]] = {}
    for row in list(report.get("results") or []):
        entry = dict(row or {})
        scenario_id = str(entry.get("id") or "").strip()
        if scenario_id:
            rows[scenario_id] = entry
    return rows


def scenario_execution_index(report: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    rows: Dict[str, Dict[str, Any]] = {}
    evidence = dict(report.get("evidence") or {})
    for row in list(evidence.get("scenario_execution") or []):
        entry = dict(row or {})
        scenario_id = str(entry.get("scenario_id") or "").strip()
        if scenario_id:
            rows[scenario_id] = entry
    return rows


def check_evidence_type(
    evidence_type: str,
    *,
    scenario_row: Dict[str, Any],
    scenario_execution_row: Dict[str, Any],
    report: Dict[str, Any],
    container_report: Dict[str, Any] | None,
) -> Tuple[bool, str]:
    execution_evidence = dict(scenario_row.get("execution_evidence") or {})
    if evidence_type == "runtime_telemetry":
        observed = to_int(execution_evidence.get("runtime_request_count"))
        return observed > 0, f"runtime_request_count={observed}"
    if evidence_type == "monitoring_delta":
        observed = to_int(execution_evidence.get("monitoring_total_delta"))
        return observed > 0, f"monitoring_total_delta={observed}"
    if evidence_type == "coverage_delta":
        observed = to_int(execution_evidence.get("coverage_delta_total"))
        return observed > 0, f"coverage_delta_total={observed}"
    if evidence_type == "lineage":
        run = dict(dict(report.get("evidence") or {}).get("run") or {})
        lineage = dict(run.get("request_id_lineage") or {})
        run_has_lineage = bool(str(lineage.get("sim_run_id") or "").strip())
        scenario_has_lineage = bool(
            scenario_execution_row.get("has_runtime_telemetry_evidence")
        )
        return (
            run_has_lineage and scenario_has_lineage,
            f"run_lineage={run_has_lineage} scenario_lineage={scenario_has_lineage}",
        )
    if evidence_type == "ip_ban_update":
        before = to_int(
            dict(dict(report.get("monitoring_before") or {}).get("coverage") or {}).get(
                "ban_count"
            )
        )
        after = to_int(
            dict(dict(report.get("monitoring_after") or {}).get("coverage") or {}).get(
                "ban_count"
            )
        )
        delta = after - before
        return delta > 0, f"ban_count_delta={delta}"
    if evidence_type == "container_passed":
        if not isinstance(container_report, dict):
            return False, "container_report_missing"
        return bool(container_report.get("passed")), f"container_passed={container_report.get('passed')}"
    if evidence_type == "frontier_lineage_complete":
        if not isinstance(container_report, dict):
            return False, "container_report_missing"
        frontier_lineage = dict(container_report.get("frontier_lineage") or {})
        complete = bool(frontier_lineage.get("lineage_complete"))
        return complete, f"frontier_lineage_complete={complete}"
    if evidence_type == "policy_violation_zero":
        if not isinstance(container_report, dict):
            return False, "container_report_missing"
        policy = dict(container_report.get("policy_audit") or {})
        violations = to_int(policy.get("violation_count"))
        return violations == 0, f"policy_violation_count={violations}"
    return False, "unknown_evidence_type"


def validate_matrix(
    matrix: Dict[str, Any],
    manifest: Dict[str, Any],
    report: Dict[str, Any],
    *,
    container_report: Dict[str, Any] | None,
    allow_missing_container_report: bool,
) -> Dict[str, Any]:
    rows = list(matrix.get("rows") or [])
    if not rows:
        raise RuntimeError("verification matrix rows must be non-empty")

    report_lane = str(report.get("execution_lane") or "").strip()
    results_by_id = result_index(report)
    execution_by_id = scenario_execution_index(report)
    manifest_scenario_ids = {
        str(dict(scenario or {}).get("id") or "").strip()
        for scenario in list(manifest.get("scenarios") or [])
        if str(dict(scenario or {}).get("id") or "").strip()
    }

    matrix_categories = {
        str(dict(row or {}).get("defense_category") or "").strip()
        for row in rows
        if str(dict(row or {}).get("defense_category") or "").strip()
    }
    missing_categories = sorted(
        set(expected_manifest_categories(manifest)) - set(matrix_categories)
    )

    failures: List[str] = []
    row_results: List[Dict[str, Any]] = []
    if missing_categories:
        failures.append(
            "missing_matrix_row:categories=" + ",".join(missing_categories)
        )

    for row in rows:
        entry = dict(row or {})
        row_id = str(entry.get("row_id") or "").strip() or "unknown_row"
        required_scenarios = [
            str(item).strip()
            for item in list(entry.get("required_scenarios") or [])
            if str(item).strip()
        ]
        required_evidence = [
            str(item).strip()
            for item in list(entry.get("required_evidence_types") or [])
            if str(item).strip()
        ]
        required_lanes = [
            str(item).strip()
            for item in list(entry.get("required_lanes") or [])
            if str(item).strip()
        ]
        lineage_segment = str(entry.get("lineage_segment") or "request_id_lineage")
        row_failures: List[str] = []
        if "black_box" in required_lanes and report_lane != "black_box":
            row_failures.append(
                f"missing_matrix_row:row={row_id}:required_lane=black_box:observed_lane={report_lane}"
            )
        for scenario_id in required_scenarios:
            if scenario_id not in manifest_scenario_ids:
                row_failures.append(
                    f"missing_matrix_row:row={row_id}:scenario_not_in_manifest:{scenario_id}"
                )
                continue
            scenario_row = dict(results_by_id.get(scenario_id) or {})
            if not scenario_row:
                row_failures.append(
                    f"missing_matrix_row:row={row_id}:scenario_not_in_report:{scenario_id}"
                )
                continue
            scenario_execution_row = dict(execution_by_id.get(scenario_id) or {})
            for evidence_type in required_evidence:
                passed, detail = check_evidence_type(
                    evidence_type,
                    scenario_row=scenario_row,
                    scenario_execution_row=scenario_execution_row,
                    report=report,
                    container_report=container_report,
                )
                if passed:
                    continue
                row_failures.append(
                    "missing_evidence_type:"
                    f"row={row_id}:scenario={scenario_id}:type={evidence_type}:detail={detail}"
                )
                if evidence_type in {
                    "lineage",
                    "frontier_lineage_complete",
                    "container_passed",
                    "policy_violation_zero",
                }:
                    row_failures.append(
                        "failing_telemetry_lineage_segment:"
                        f"row={row_id}:segment={lineage_segment}:scenario={scenario_id}"
                    )

        if "container_blackbox" in required_lanes:
            if container_report is None:
                if allow_missing_container_report:
                    row_results.append(
                        {
                            "row_id": row_id,
                            "passed": True,
                            "skipped": True,
                            "detail": "container_report_missing_but_allowed",
                            "failures": [],
                        }
                    )
                    continue
                row_failures.append(
                    f"missing_matrix_row:row={row_id}:container_report_missing"
                )
            else:
                for evidence_type in required_evidence:
                    passed, detail = check_evidence_type(
                        evidence_type,
                        scenario_row={},
                        scenario_execution_row={},
                        report=report,
                        container_report=container_report,
                    )
                    if passed:
                        continue
                    row_failures.append(
                        "missing_evidence_type:"
                        f"row={row_id}:scenario=container_blackbox:type={evidence_type}:detail={detail}"
                    )
                    row_failures.append(
                        "failing_telemetry_lineage_segment:"
                        f"row={row_id}:segment={lineage_segment}:scenario=container_blackbox"
                    )

        row_results.append(
            {
                "row_id": row_id,
                "passed": len(row_failures) == 0,
                "skipped": False,
                "detail": "ok" if len(row_failures) == 0 else "failed",
                "failures": row_failures,
            }
        )
        failures.extend(row_failures)

    return {
        "schema_version": "sim2-verification-matrix-report.v1",
        "status": {
            "passed": len(failures) == 0,
            "failure_count": len(failures),
            "failures": failures,
        },
        "rows": row_results,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check SIM2 verification matrix against manifest/report evidence."
    )
    parser.add_argument("--matrix", default=str(DEFAULT_MATRIX_PATH))
    parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST_PATH))
    parser.add_argument("--report", default=str(DEFAULT_REPORT_PATH))
    parser.add_argument("--container-report", default=str(DEFAULT_CONTAINER_REPORT_PATH))
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT_PATH))
    parser.add_argument(
        "--allow-missing-container-report",
        action="store_true",
        help="Allow missing container report for non-container validation lanes",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    matrix = load_json_object(Path(args.matrix))
    manifest = load_json_object(Path(args.manifest))
    report = load_json_object(Path(args.report))
    container_report: Dict[str, Any] | None = None
    container_report_path = Path(args.container_report)
    if container_report_path.exists():
        container_report = load_json_object(container_report_path)

    payload = validate_matrix(
        matrix,
        manifest,
        report,
        container_report=container_report,
        allow_missing_container_report=bool(args.allow_missing_container_report),
    )
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"[sim2-verification-matrix] report={output_path}")
    if not bool(dict(payload.get("status") or {}).get("passed")):
        print("[sim2-verification-matrix] FAIL")
        for failure in list(dict(payload.get("status") or {}).get("failures") or []):
            print(f"- {failure}")
        return 1
    print("[sim2-verification-matrix] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
