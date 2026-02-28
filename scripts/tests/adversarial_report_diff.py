#!/usr/bin/env python3
"""Compare adversarial simulation reports and summarize defender/adversary deltas."""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path
from typing import Any, Dict, List


DEFAULT_BASELINE_REPORT_PATH = Path("scripts/tests/adversarial/latest_report.baseline.json")
DEFAULT_CANDIDATE_REPORT_PATH = Path("scripts/tests/adversarial/latest_report.json")
DEFAULT_OUTPUT_PATH = Path("scripts/tests/adversarial/adversarial_report_diff.json")


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


def scenario_index(report: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    rows: Dict[str, Dict[str, Any]] = {}
    for row in list(report.get("results") or []):
        entry = dict(row or {})
        scenario_id = str(entry.get("id") or "").strip()
        if scenario_id:
            rows[scenario_id] = entry
    return rows


def gate_observed_int(report: Dict[str, Any], gate_name: str) -> int:
    for row in list(dict(report.get("gates") or {}).get("checks") or []):
        entry = dict(row or {})
        if str(entry.get("name") or "").strip() == gate_name:
            try:
                return int(entry.get("observed") or 0)
            except Exception:
                return 0
    return 0


def compare_reports(baseline: Dict[str, Any], candidate: Dict[str, Any]) -> Dict[str, Any]:
    baseline_rows = scenario_index(baseline)
    candidate_rows = scenario_index(candidate)
    baseline_ids = set(baseline_rows.keys())
    candidate_ids = set(candidate_rows.keys())

    new_passes: List[str] = []
    new_regressions: List[str] = []
    resolved_scenarios: List[str] = []
    for scenario_id in sorted(baseline_ids.intersection(candidate_ids)):
        before = dict(baseline_rows.get(scenario_id) or {})
        after = dict(candidate_rows.get(scenario_id) or {})
        before_passed = bool(before.get("passed"))
        after_passed = bool(after.get("passed"))
        if not before_passed and after_passed:
            new_passes.append(scenario_id)
        if before_passed and not after_passed:
            new_regressions.append(scenario_id)
        if (
            str(before.get("observed_outcome") or "").strip()
            != str(after.get("observed_outcome") or "").strip()
            and not after_passed
            and scenario_id not in new_regressions
        ):
            new_regressions.append(scenario_id)
    for scenario_id in sorted(baseline_ids - candidate_ids):
        resolved_scenarios.append(scenario_id)
    new_scenarios = sorted(candidate_ids - baseline_ids)

    baseline_coverage = dict(dict(baseline.get("coverage_gates") or {}).get("coverage") or {})
    candidate_coverage = dict(dict(candidate.get("coverage_gates") or {}).get("coverage") or {})
    baseline_deltas = dict(baseline_coverage.get("deltas") or {})
    candidate_deltas = dict(candidate_coverage.get("deltas") or {})
    metrics = sorted(set(baseline_deltas.keys()).union(candidate_deltas.keys()))
    increased: List[Dict[str, Any]] = []
    decreased: List[Dict[str, Any]] = []
    unchanged: List[Dict[str, Any]] = []
    for metric in metrics:
        before = int(baseline_deltas.get(metric) or 0)
        after = int(candidate_deltas.get(metric) or 0)
        delta = after - before
        row = {"metric": str(metric), "delta": delta}
        if delta > 0:
            increased.append(row)
        elif delta < 0:
            decreased.append(row)
        else:
            unchanged.append(row)

    baseline_collateral = float(
        dict(dict(baseline.get("cohort_metrics") or {}).get("human_like") or {}).get(
            "collateral_ratio",
            0.0,
        )
        or 0.0
    )
    candidate_collateral = float(
        dict(dict(candidate.get("cohort_metrics") or {}).get("human_like") or {}).get(
            "collateral_ratio",
            0.0,
        )
        or 0.0
    )
    collateral_delta = round(candidate_collateral - baseline_collateral, 4)

    baseline_latency_p95 = gate_observed_int(baseline, "latency_p95")
    candidate_latency_p95 = gate_observed_int(candidate, "latency_p95")
    baseline_suite_runtime = int(baseline.get("suite_runtime_ms") or 0)
    candidate_suite_runtime = int(candidate.get("suite_runtime_ms") or 0)
    baseline_request_count = int(baseline.get("request_count") or 0)
    candidate_request_count = int(candidate.get("request_count") or 0)

    return {
        "schema_version": "adversarial-report-diff.v1",
        "scenario_transitions": {
            "new_passes": new_passes,
            "new_regressions": new_regressions,
            "new_scenarios": new_scenarios,
            "resolved_scenarios": resolved_scenarios,
        },
        "cost_shift": {
            "latency_p95_baseline_ms": baseline_latency_p95,
            "latency_p95_candidate_ms": candidate_latency_p95,
            "latency_p95_delta_ms": candidate_latency_p95 - baseline_latency_p95,
            "suite_runtime_baseline_ms": baseline_suite_runtime,
            "suite_runtime_candidate_ms": candidate_suite_runtime,
            "suite_runtime_delta_ms": candidate_suite_runtime - baseline_suite_runtime,
            "request_count_baseline": baseline_request_count,
            "request_count_candidate": candidate_request_count,
            "request_count_delta": candidate_request_count - baseline_request_count,
        },
        "collateral_shift": {
            "human_like_collateral_ratio_baseline": round(baseline_collateral, 4),
            "human_like_collateral_ratio_candidate": round(candidate_collateral, 4),
            "human_like_collateral_ratio_delta": collateral_delta,
        },
        "defense_delta_shift": {
            "increased": increased,
            "decreased": decreased,
            "unchanged": unchanged,
        },
    }


def build_backlog_candidates(
    diff: Dict[str, Any], *, owner: str, disposition_sla_hours: int
) -> List[Dict[str, Any]]:
    regressions = [
        str(item).strip()
        for item in list(
            dict(diff.get("scenario_transitions") or {}).get("new_regressions") or []
        )
        if str(item).strip()
    ]
    backlog: List[Dict[str, Any]] = []
    for scenario_id in regressions:
        backlog.append(
            {
                "scenario_id": scenario_id,
                "priority": "P0",
                "owner": str(owner or "").strip() or "runtime_engineering",
                "disposition_sla_hours": max(1, int(disposition_sla_hours)),
                "summary": (
                    f"Investigate and mitigate regression candidate from adversarial diff: {scenario_id}"
                ),
            }
        )
    return backlog


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare baseline/candidate adversarial reports and emit transition/cost/collateral deltas."
    )
    parser.add_argument("--baseline", default=str(DEFAULT_BASELINE_REPORT_PATH))
    parser.add_argument("--candidate", default=str(DEFAULT_CANDIDATE_REPORT_PATH))
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT_PATH))
    parser.add_argument("--owner", default="runtime_engineering")
    parser.add_argument("--disposition-sla-hours", type=int, default=48)
    parser.add_argument(
        "--fail-on-new-regressions",
        action="store_true",
        help="Return non-zero when candidate introduces new regressions.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    baseline = load_json_object(Path(args.baseline))
    candidate = load_json_object(Path(args.candidate))

    diff = compare_reports(baseline, candidate)
    backlog_candidates = build_backlog_candidates(
        diff,
        owner=str(args.owner),
        disposition_sla_hours=int(args.disposition_sla_hours),
    )
    payload = {
        "schema_version": "adversarial-report-diff-output.v1",
        "generated_at_unix": int(time.time()),
        "source": {
            "baseline": str(args.baseline),
            "candidate": str(args.candidate),
        },
        "diff": diff,
        "backlog_candidates": backlog_candidates,
    }
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    transitions = dict(diff.get("scenario_transitions") or {})
    print(f"[adversarial-report-diff] output={output_path}")
    print(
        "[adversarial-report-diff] new_passes={} new_regressions={} new_scenarios={}".format(
            len(list(transitions.get("new_passes") or [])),
            len(list(transitions.get("new_regressions") or [])),
            len(list(transitions.get("new_scenarios") or [])),
        )
    )
    if args.fail_on_new_regressions and list(transitions.get("new_regressions") or []):
        print("[adversarial-report-diff] FAIL new regressions detected.")
        return 1
    print("[adversarial-report-diff] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
