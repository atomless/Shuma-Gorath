#!/usr/bin/env python3
"""Deterministic repeatability gate for adversarial profiles."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Dict, List, Tuple


DEFAULT_PROFILES = ("fast_smoke", "abuse_regression", "full_coverage")
DEFAULT_REPEATABILITY_REPORT = "scripts/tests/adversarial/repeatability_report.json"


def parse_profiles(raw_profiles: str) -> List[str]:
    profiles = [item.strip() for item in str(raw_profiles).split(",") if item.strip()]
    if not profiles:
        return list(DEFAULT_PROFILES)
    return profiles


def run_profile_once(
    manifest: str,
    profile: str,
    report_path: Path,
) -> Tuple[int, str, str]:
    command = [
        "python3",
        "scripts/tests/adversarial_simulation_runner.py",
        "--manifest",
        manifest,
        "--profile",
        profile,
        "--report",
        str(report_path),
    ]
    env = dict(os.environ)
    env["SHUMA_ADVERSARIAL_PRESERVE_STATE"] = "0"
    env["SHUMA_ADVERSARIAL_ROTATE_IPS"] = "0"
    result = subprocess.run(command, capture_output=True, text=True, env=env, check=False)
    return result.returncode, result.stdout, result.stderr


def load_report(path: Path) -> Dict[str, Any]:
    parsed = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(parsed, dict):
        raise RuntimeError(f"repeatability report must be object: {path}")
    return parsed


def scenario_vector(report: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    rows = report.get("results")
    if not isinstance(rows, list):
        return {}
    out: Dict[str, Dict[str, Any]] = {}
    for row in rows:
        if not isinstance(row, dict):
            continue
        sid = str(row.get("id") or "").strip()
        if not sid:
            continue
        out[sid] = {
            "passed": bool(row.get("passed")),
            "observed_outcome": str(row.get("observed_outcome") or ""),
            "latency_ms": int(row.get("latency_ms") or 0),
        }
    return out


def gate_vector(report: Dict[str, Any]) -> Dict[str, bool]:
    gates = report.get("gates")
    if not isinstance(gates, dict):
        return {}
    checks = gates.get("checks")
    if not isinstance(checks, list):
        return {}
    out: Dict[str, bool] = {}
    for check in checks:
        if not isinstance(check, dict):
            continue
        name = str(check.get("name") or "").strip()
        if not name:
            continue
        out[name] = bool(check.get("passed"))
    return out


def coverage_deltas(report: Dict[str, Any]) -> Dict[str, int]:
    section = report.get("coverage_gates")
    if not isinstance(section, dict):
        return {}
    coverage = section.get("coverage")
    if not isinstance(coverage, dict):
        return {}
    deltas = coverage.get("deltas")
    if not isinstance(deltas, dict):
        return {}
    out: Dict[str, int] = {}
    for key, value in deltas.items():
        out[str(key)] = int(value or 0)
    return out


def compare_reports(
    baseline: Dict[str, Any],
    candidate: Dict[str, Any],
    latency_tolerance_ms: int,
) -> List[str]:
    differences: List[str] = []
    baseline_scenarios = scenario_vector(baseline)
    candidate_scenarios = scenario_vector(candidate)
    baseline_ids = sorted(baseline_scenarios.keys())
    candidate_ids = sorted(candidate_scenarios.keys())
    if baseline_ids != candidate_ids:
        differences.append(
            f"scenario_ids drift baseline={baseline_ids} candidate={candidate_ids}"
        )
        return differences

    for scenario_id in baseline_ids:
        base = baseline_scenarios[scenario_id]
        cand = candidate_scenarios[scenario_id]
        if base["passed"] != cand["passed"]:
            differences.append(
                f"{scenario_id}: pass_flag drift baseline={base['passed']} candidate={cand['passed']}"
            )
        if base["observed_outcome"] != cand["observed_outcome"]:
            differences.append(
                f"{scenario_id}: outcome drift baseline={base['observed_outcome']} candidate={cand['observed_outcome']}"
            )
        latency_delta = abs(int(base["latency_ms"]) - int(cand["latency_ms"]))
        if latency_delta > latency_tolerance_ms:
            differences.append(
                f"{scenario_id}: latency drift baseline={base['latency_ms']} candidate={cand['latency_ms']} tolerance={latency_tolerance_ms}"
            )

    baseline_gates = gate_vector(baseline)
    candidate_gates = gate_vector(candidate)
    if baseline_gates != candidate_gates:
        gate_names = sorted(set(baseline_gates.keys()).union(candidate_gates.keys()))
        for gate_name in gate_names:
            if baseline_gates.get(gate_name) != candidate_gates.get(gate_name):
                differences.append(
                    f"gate {gate_name}: baseline={baseline_gates.get(gate_name)} candidate={candidate_gates.get(gate_name)}"
                )

    baseline_coverage = coverage_deltas(baseline)
    candidate_coverage = coverage_deltas(candidate)
    if baseline_coverage != candidate_coverage:
        coverage_names = sorted(set(baseline_coverage.keys()).union(candidate_coverage.keys()))
        for name in coverage_names:
            if baseline_coverage.get(name) != candidate_coverage.get(name):
                differences.append(
                    f"coverage_delta {name}: baseline={baseline_coverage.get(name)} candidate={candidate_coverage.get(name)}"
                )
    return differences


def main() -> int:
    parser = argparse.ArgumentParser(description="Run adversarial repeatability gate")
    parser.add_argument(
        "--manifest",
        default="scripts/tests/adversarial/scenario_manifest.v2.json",
        help="Manifest path",
    )
    parser.add_argument(
        "--profiles",
        default=",".join(DEFAULT_PROFILES),
        help="Comma-separated profile list",
    )
    parser.add_argument("--repeats", default="3", help="Repeat count")
    parser.add_argument(
        "--latency-tolerance-ms",
        default=os.environ.get("ADVERSARIAL_REPEATABILITY_LATENCY_TOLERANCE_MS", "250"),
        help="Per-scenario latency tolerance",
    )
    parser.add_argument("--report", default=DEFAULT_REPEATABILITY_REPORT, help="Summary output path")
    args = parser.parse_args()

    repeats = int(str(args.repeats).strip())
    if repeats < 2:
        print("repeats must be >= 2", file=sys.stderr)
        return 2
    latency_tolerance_ms = max(0, int(str(args.latency_tolerance_ms).strip()))
    profiles = parse_profiles(args.profiles)

    root = Path("scripts/tests/adversarial/repeatability")
    root.mkdir(parents=True, exist_ok=True)

    summary: Dict[str, Any] = {
        "schema_version": "adversarial-repeatability.v1",
        "manifest": args.manifest,
        "profiles": [],
        "repeats": repeats,
        "latency_tolerance_ms": latency_tolerance_ms,
        "generated_at_unix": int(time.time()),
        "passed": True,
    }

    for profile in profiles:
        print(f"[repeatability] profile={profile} repeats={repeats}")
        run_entries: List[Dict[str, Any]] = []
        baseline_report: Dict[str, Any] | None = None
        profile_differences: List[str] = []
        for index in range(repeats):
            run_number = index + 1
            run_report_path = root / f"{profile}.run{run_number}.json"
            code, stdout_text, stderr_text = run_profile_once(args.manifest, profile, run_report_path)
            run_entries.append(
                {
                    "run_number": run_number,
                    "report_path": str(run_report_path),
                    "exit_code": code,
                    "stdout_tail": stdout_text.splitlines()[-20:],
                    "stderr_tail": stderr_text.splitlines()[-20:],
                }
            )
            if code != 0:
                profile_differences.append(f"run_{run_number}: runner_exit_code={code}")
                continue
            report = load_report(run_report_path)
            if baseline_report is None:
                baseline_report = report
            else:
                drift = compare_reports(
                    baseline_report,
                    report,
                    latency_tolerance_ms=latency_tolerance_ms,
                )
                for item in drift:
                    profile_differences.append(f"run_{run_number}: {item}")

        profile_passed = len(profile_differences) == 0
        if not profile_passed:
            summary["passed"] = False
            print(f"[repeatability] profile={profile} status=FAIL")
            for diff in profile_differences[:20]:
                print(f"[repeatability] diff={diff}")
        else:
            print(f"[repeatability] profile={profile} status=PASS")
        summary["profiles"].append(
            {
                "profile": profile,
                "passed": profile_passed,
                "runs": run_entries,
                "differences": profile_differences,
            }
        )

    report_path = Path(args.report)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(f"[repeatability] report={report_path}")
    if not summary["passed"]:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
