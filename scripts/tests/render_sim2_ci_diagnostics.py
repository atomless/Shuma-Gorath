#!/usr/bin/env python3
"""Render SIM2 CI diagnostics artifact from adversarial report evidence."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, List

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tests.adversarial_artifact_paths import SIM2_CI_DIAGNOSTICS_PATH

DEFAULT_REPORT_PATH = Path("scripts/tests/adversarial/latest_report.json")
DEFAULT_OUTPUT_PATH = SIM2_CI_DIAGNOSTICS_PATH


def load_report(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"missing report: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid report JSON: {path}") from exc
    if not isinstance(payload, dict):
        raise RuntimeError(f"report must be a JSON object: {path}")
    return payload


def to_int(value: Any) -> int:
    try:
        return int(value)
    except Exception:
        return 0


def extract_timeline_snapshots(report: Dict[str, Any]) -> List[Dict[str, Any]]:
    rows = list(dict(report.get("evidence") or {}).get("scenario_execution") or [])
    snapshots: List[Dict[str, Any]] = []
    for index, row in enumerate(rows):
        data = dict(row or {})
        snapshots.append(
            {
                "sequence": index + 1,
                "scenario_id": str(data.get("scenario_id") or ""),
                "runtime_request_count": to_int(data.get("runtime_request_count")),
                "monitoring_total_delta": to_int(data.get("monitoring_total_delta")),
                "coverage_delta_total": to_int(data.get("coverage_delta_total")),
                "simulation_event_count_delta": to_int(
                    data.get("simulation_event_count_delta")
                ),
                "has_runtime_telemetry_evidence": bool(
                    data.get("has_runtime_telemetry_evidence")
                ),
            }
        )
    return snapshots


def extract_event_counts(report: Dict[str, Any]) -> Dict[str, Any]:
    run = dict(dict(report.get("evidence") or {}).get("run") or {})
    outcomes = dict(run.get("decision_outcomes") or {})
    simulation_event_reasons = list(report.get("simulation_event_reasons") or [])
    return {
        "decision_outcomes": outcomes,
        "defenses_touched": list(run.get("defenses_touched") or []),
        "simulation_event_reason_count": len(simulation_event_reasons),
        "simulation_event_reasons": simulation_event_reasons,
    }


def extract_refresh_traces(report: Dict[str, Any]) -> List[Dict[str, Any]]:
    traces: List[Dict[str, Any]] = []
    for domain, block_key in (
        ("gate_checks", "gates"),
        ("coverage_checks", "coverage_gates"),
        ("realism_checks", "realism_gates"),
    ):
        block = dict(report.get(block_key) or {})
        checks = list(block.get("checks") or [])
        for index, check in enumerate(checks):
            item = dict(check or {})
            traces.append(
                {
                    "domain": domain,
                    "sequence": index + 1,
                    "check": str(item.get("name") or item.get("id") or f"check_{index + 1}"),
                    "passed": bool(item.get("passed")),
                    "detail": str(item.get("detail") or ""),
                }
            )
    return traces


def render_diagnostics(report: Dict[str, Any]) -> Dict[str, Any]:
    timeline_snapshots = extract_timeline_snapshots(report)
    event_counts = extract_event_counts(report)
    refresh_traces = extract_refresh_traces(report)
    return {
        "schema_version": "sim2-ci-diagnostics.v1",
        "source_report": {
            "profile": str(report.get("profile") or ""),
            "execution_lane": str(report.get("execution_lane") or ""),
            "generated_at_unix": to_int(report.get("generated_at_unix")),
            "suite_runtime_ms": to_int(report.get("suite_runtime_ms")),
            "passed": bool(report.get("passed")),
        },
        "timeline_snapshots": timeline_snapshots,
        "event_counts": event_counts,
        "refresh_traces": refresh_traces,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render SIM2 CI diagnostics artifact from latest adversarial report."
    )
    parser.add_argument("--report", default=str(DEFAULT_REPORT_PATH))
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT_PATH))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report_path = Path(args.report)
    output_path = Path(args.output)
    report = load_report(report_path)
    diagnostics = render_diagnostics(report)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(diagnostics, indent=2), encoding="utf-8")
    print(f"[sim2-ci-diagnostics] source={report_path}")
    print(f"[sim2-ci-diagnostics] output={output_path}")
    print(
        "[sim2-ci-diagnostics] timeline_snapshots={} refresh_traces={}".format(
            len(diagnostics.get("timeline_snapshots") or []),
            len(diagnostics.get("refresh_traces") or []),
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
