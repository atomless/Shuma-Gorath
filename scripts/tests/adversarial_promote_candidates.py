#!/usr/bin/env python3
"""Frontier finding triage and deterministic promotion pipeline."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any, Dict, List


DEFAULT_REPORT_PATH = Path("scripts/tests/adversarial/latest_report.json")
DEFAULT_ATTACK_PLAN_PATH = Path("scripts/tests/adversarial/attack_plan.json")
DEFAULT_MANIFEST_PATH = Path("scripts/tests/adversarial/scenario_manifest.v2.json")
DEFAULT_OUTPUT_PATH = Path("scripts/tests/adversarial/promotion_candidates_report.json")
RUNNER_PATH = "scripts/tests/adversarial_simulation_runner.py"


def load_json(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise ValueError(f"missing JSON artifact: {path}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise ValueError(f"invalid JSON artifact: {path}") from exc
    if not isinstance(data, dict):
        raise ValueError(f"JSON artifact must be object: {path}")
    return data


def save_json(path: Path, payload: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def dict_or_empty(value: Any) -> Dict[str, Any]:
    return value if isinstance(value, dict) else {}


def list_or_empty(value: Any) -> List[Any]:
    return value if isinstance(value, list) else []


def stable_finding_id(record: Dict[str, Any]) -> str:
    canonical_basis = {
        "scenario_family": str(record.get("scenario_family") or ""),
        "path": str(record.get("path") or "/"),
        "headers": dict_or_empty(record.get("headers")),
        "cadence_pattern": dict_or_empty(record.get("cadence_pattern")),
    }
    encoded = json.dumps(canonical_basis, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return f"simf-{hashlib.sha256(encoded).hexdigest()[:16]}"


def severity_for_finding(expected: str, observed: str, finding_kind: str) -> str:
    if finding_kind != "regression_candidate":
        return "low"
    expected_high = {"deny_temp", "tarpit", "maze"}
    observed_low = {"allow", "monitor", "not-a-bot"}
    if expected in expected_high and observed in observed_low:
        return "high"
    if expected != observed:
        return "medium"
    return "low"


def risk_for_severity(severity: str) -> str:
    if severity == "high":
        return "high"
    if severity == "medium":
        return "medium"
    return "low"


def index_results(report: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    mapping: Dict[str, Dict[str, Any]] = {}
    for result in list_or_empty(report.get("results")):
        if isinstance(result, dict):
            scenario_id = str(result.get("id") or "").strip()
            if scenario_id:
                mapping[scenario_id] = result
    return mapping


def normalize_findings(attack_plan: Dict[str, Any], report: Dict[str, Any]) -> List[Dict[str, Any]]:
    findings: List[Dict[str, Any]] = []
    result_map = index_results(report)
    frontier_mode = str(attack_plan.get("frontier_mode") or "disabled")
    providers = [
        {
            "provider": str(dict_or_empty(provider).get("provider") or ""),
            "model_id": str(dict_or_empty(provider).get("model_id") or ""),
            "configured": bool(dict_or_empty(provider).get("configured")),
        }
        for provider in list_or_empty(attack_plan.get("providers"))
    ]
    diversity_confidence = str(attack_plan.get("diversity_confidence") or "none")

    for candidate in list_or_empty(attack_plan.get("candidates")):
        if not isinstance(candidate, dict):
            continue
        scenario_id = str(candidate.get("scenario_id") or "").strip()
        if not scenario_id:
            continue
        result = dict_or_empty(result_map.get(scenario_id))
        payload = dict_or_empty(candidate.get("payload"))
        traffic = dict_or_empty(payload.get("traffic_model"))
        target = dict_or_empty(payload.get("target"))
        scenario_family = str(candidate.get("driver") or result.get("driver") or "").strip()
        expected_outcome = str(result.get("expected_outcome") or "").strip()
        observed_outcome = str(result.get("observed_outcome") or "").strip()
        passed = bool(result.get("passed"))
        finding_kind = "regression_candidate" if (not passed or expected_outcome != observed_outcome) else "behavioral_candidate"

        finding: Dict[str, Any] = {
            "scenario_id": scenario_id,
            "tier": str(candidate.get("tier") or result.get("tier") or ""),
            "scenario_family": scenario_family,
            "path": str(target.get("path_hint") or "/"),
            "headers": {
                "user_agent": str(traffic.get("user_agent") or ""),
            },
            "cadence_pattern": {
                "retry_strategy": str(traffic.get("retry_strategy") or "single_attempt"),
                "think_time_ms_min": int(traffic.get("think_time_ms_min") or 0),
                "think_time_ms_max": int(traffic.get("think_time_ms_max") or 0),
            },
            "expected_outcome": expected_outcome,
            "observed_outcome": observed_outcome,
            "passed": passed,
            "runtime_budget_ms": int(result.get("runtime_budget_ms") or 0),
            "latency_ms": int(result.get("latency_ms") or 0),
            "finding_kind": finding_kind,
            "frontier_mode": frontier_mode,
            "provider_count": int(attack_plan.get("provider_count") or 0),
            "providers": providers,
            "diversity_confidence": diversity_confidence,
        }
        finding["severity"] = severity_for_finding(
            expected=finding["expected_outcome"],
            observed=finding["observed_outcome"],
            finding_kind=finding["finding_kind"],
        )
        finding["risk"] = risk_for_severity(str(finding["severity"]))
        finding["finding_id"] = stable_finding_id(finding)
        findings.append(finding)
    return findings


def classify_replay_outcome(finding: Dict[str, Any], replay_result: Dict[str, Any]) -> str:
    status = str(replay_result.get("status") or "")
    if status != "ok":
        return "needs_manual_review"

    replay_outcome = str(replay_result.get("observed_outcome") or "")
    replay_passed = bool(replay_result.get("passed"))
    expected_outcome = str(finding.get("expected_outcome") or "")
    observed_outcome = str(finding.get("observed_outcome") or "")
    finding_kind = str(finding.get("finding_kind") or "behavioral_candidate")

    if finding_kind == "regression_candidate":
        if (not replay_passed) and replay_outcome == observed_outcome:
            return "confirmed_reproducible"
        if replay_passed or replay_outcome != observed_outcome:
            return "not_reproducible"
        return "needs_manual_review"

    if replay_passed and replay_outcome == expected_outcome:
        return "confirmed_reproducible"
    if (not replay_passed) or replay_outcome != expected_outcome:
        return "not_reproducible"
    return "needs_manual_review"


def build_promotion_decision(
    finding: Dict[str, Any], replay_result: Dict[str, Any], classification: str
) -> Dict[str, Any]:
    frontier_mode = str(finding.get("frontier_mode") or "disabled")
    review_notes: List[str] = []
    owner_review_required = classification == "confirmed_reproducible"
    if classification == "confirmed_reproducible":
        if frontier_mode == "single_provider_self_play":
            review_notes.append(
                "single_provider_self_play findings must be owner-reviewed before promotion."
            )
        elif frontier_mode == "multi_provider_playoff":
            review_notes.append(
                "multi_provider_playoff provides higher initial confidence, but owner review remains required."
            )
        else:
            review_notes.append(
                "frontier mode is degraded/disabled; owner review is required before promotion."
            )
    else:
        review_notes.append("deterministic replay did not confirm promotable candidate.")

    promoted_scenario: Dict[str, Any] = {}
    if classification == "confirmed_reproducible":
        max_latency = int(replay_result.get("latency_ms") or finding.get("latency_ms") or 0)
        max_latency = max(100, int(max_latency * 1.5))
        promoted_scenario = {
            "id": f"frontier_regression_{finding.get('finding_id')}",
            "source_scenario_id": finding.get("scenario_id"),
            "expected_outcome": replay_result.get("observed_outcome")
            or finding.get("observed_outcome")
            or finding.get("expected_outcome"),
            "cost_assertions": {
                "max_latency_ms": max_latency,
            },
            "severity": finding.get("severity"),
            "risk": finding.get("risk"),
        }

    return {
        "classification": classification,
        "owner_review_required": owner_review_required,
        "blocking_regression": False,
        "review_notes": review_notes,
        "promoted_scenario": promoted_scenario,
    }


def create_replay_manifest(base_manifest: Dict[str, Any], scenario_id: str) -> Dict[str, Any]:
    scenarios = [
        scenario
        for scenario in list_or_empty(base_manifest.get("scenarios"))
        if isinstance(scenario, dict) and str(scenario.get("id") or "").strip() == scenario_id
    ]
    if not scenarios:
        raise ValueError(f"scenario_id not found in manifest: {scenario_id}")

    scenario = copy.deepcopy(scenarios[0])
    expected_outcome = str(scenario.get("expected_outcome") or "allow")
    runtime_budget_ms = int(scenario.get("runtime_budget_ms") or 1000)
    max_runtime_seconds = max(30, min(300, int((runtime_budget_ms / 1000.0) * 4)))
    profile = {
        "description": f"deterministic replay for {scenario_id}",
        "max_runtime_seconds": max_runtime_seconds,
        "scenario_ids": [scenario_id],
        "fail_fast": True,
        "gates": {
            "latency": {"p95_max_ms": max(1000, runtime_budget_ms * 4)},
            "outcome_ratio_bounds": {expected_outcome: {"min": 0.0, "max": 1.0}},
            "telemetry_amplification": {
                "max_fingerprint_events_per_request": 20.0,
                "max_monitoring_events_per_request": 20.0,
            },
        },
    }

    return {
        "schema_version": str(base_manifest.get("schema_version") or "sim-manifest.v2"),
        "suite_id": str(base_manifest.get("suite_id") or "shuma-adversarial-scenarios"),
        "description": "Generated deterministic replay manifest for frontier triage",
        "execution_lane": str(base_manifest.get("execution_lane") or "black_box"),
        "profiles": {"promotion_replay": profile},
        "scenarios": [scenario],
    }


def run_deterministic_replay(
    manifest_path: Path, scenario_id: str, timeout_seconds: float
) -> Dict[str, Any]:
    replay_result: Dict[str, Any] = {
        "status": "needs_manual_review",
        "scenario_id": scenario_id,
        "observed_outcome": "",
        "passed": False,
        "latency_ms": 0,
        "exit_code": -1,
    }
    base_manifest = load_json(manifest_path)
    replay_manifest = create_replay_manifest(base_manifest, scenario_id)

    with tempfile.TemporaryDirectory(prefix="adversarial-replay-") as tmpdir:
        temp_manifest_path = Path(tmpdir) / "promotion_replay_manifest.json"
        temp_report_path = Path(tmpdir) / "promotion_replay_report.json"
        save_json(temp_manifest_path, replay_manifest)

        cmd = [
            sys.executable,
            RUNNER_PATH,
            "--manifest",
            str(temp_manifest_path),
            "--profile",
            "promotion_replay",
            "--report",
            str(temp_report_path),
        ]

        env = os.environ.copy()
        env["SHUMA_ADVERSARIAL_PRESERVE_STATE"] = "0"
        env["SHUMA_ADVERSARIAL_ROTATE_IPS"] = "0"

        try:
            proc = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout_seconds,
                env=env,
                check=False,
            )
        except subprocess.TimeoutExpired:
            replay_result["status"] = "timeout"
            return replay_result

        replay_result["exit_code"] = int(proc.returncode)
        replay_result["stdout_tail"] = "\n".join(proc.stdout.splitlines()[-20:])
        replay_result["stderr_tail"] = "\n".join(proc.stderr.splitlines()[-20:])

        if not temp_report_path.exists():
            replay_result["status"] = "runner_failed"
            return replay_result

        replay_report = load_json(temp_report_path)
        replay_rows = [
            row
            for row in list_or_empty(replay_report.get("results"))
            if isinstance(row, dict) and str(row.get("id") or "") == scenario_id
        ]
        if not replay_rows:
            replay_result["status"] = "needs_manual_review"
            return replay_result

        row = replay_rows[0]
        replay_result["observed_outcome"] = str(row.get("observed_outcome") or "")
        replay_result["passed"] = bool(row.get("passed"))
        replay_result["latency_ms"] = int(row.get("latency_ms") or 0)
        replay_result["status"] = "ok" if proc.returncode == 0 else "runner_failed"
        return replay_result


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run frontier finding triage and deterministic promotion checks."
    )
    parser.add_argument(
        "--report",
        default=str(DEFAULT_REPORT_PATH),
        help="Path to adversarial simulation report JSON",
    )
    parser.add_argument(
        "--attack-plan",
        default=str(DEFAULT_ATTACK_PLAN_PATH),
        help="Path to attack_plan.json",
    )
    parser.add_argument(
        "--manifest",
        default=str(DEFAULT_MANIFEST_PATH),
        help="Path to canonical scenario manifest",
    )
    parser.add_argument(
        "--output",
        default=str(DEFAULT_OUTPUT_PATH),
        help="Path for promotion triage report output",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=120.0,
        help="Timeout per deterministic replay attempt",
    )
    parser.add_argument(
        "--fail-on-confirmed-regressions",
        action="store_true",
        help="Return non-zero when deterministic replay confirms regression candidates",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report_path = Path(args.report)
    attack_plan_path = Path(args.attack_plan)
    manifest_path = Path(args.manifest)
    output_path = Path(args.output)

    report = load_json(report_path)
    attack_plan = load_json(attack_plan_path)

    findings = normalize_findings(attack_plan=attack_plan, report=report)
    # Candidate set is intentionally narrowed to potential regressions; this keeps
    # protected-lane runtime bounded while still triaging blocking-risk findings.
    replay_candidates = [
        finding
        for finding in findings
        if str(finding.get("finding_kind")) == "regression_candidate"
    ]

    lineage: List[Dict[str, Any]] = []
    confirmed_regressions = 0

    for finding in replay_candidates:
        scenario_id = str(finding.get("scenario_id") or "")
        replay_result = run_deterministic_replay(
            manifest_path=manifest_path,
            scenario_id=scenario_id,
            timeout_seconds=float(args.timeout_seconds),
        )
        classification = classify_replay_outcome(
            finding=finding,
            replay_result=replay_result,
        )
        promotion = build_promotion_decision(
            finding=finding,
            replay_result=replay_result,
            classification=classification,
        )
        if classification == "confirmed_reproducible":
            confirmed_regressions += 1
            promotion["blocking_regression"] = True
            promotion["review_notes"].append(
                "deterministic replay confirmed a regression candidate; release/merge must remain blocked until owner disposition."
            )
        lineage.append(
            {
                "finding_id": finding.get("finding_id"),
                "scenario_id": scenario_id,
                "classification": classification,
                "candidate": {
                    "scenario_family": finding.get("scenario_family"),
                    "path": finding.get("path"),
                    "expected_outcome": finding.get("expected_outcome"),
                    "observed_outcome": finding.get("observed_outcome"),
                    "severity": finding.get("severity"),
                    "risk": finding.get("risk"),
                    "frontier_mode": finding.get("frontier_mode"),
                    "diversity_confidence": finding.get("diversity_confidence"),
                },
                "replay": replay_result,
                "promotion": promotion,
            }
        )

    classification_counts: Dict[str, int] = {
        "confirmed_reproducible": 0,
        "not_reproducible": 0,
        "needs_manual_review": 0,
    }
    for row in lineage:
        classification = str(row.get("classification") or "")
        if classification in classification_counts:
            classification_counts[classification] += 1

    payload = {
        "schema_version": "adversarial-promotion.v1",
        "generated_at_unix": int(time.time()),
        "source": {
            "report_path": str(report_path),
            "attack_plan_path": str(attack_plan_path),
            "manifest_path": str(manifest_path),
        },
        "frontier": {
            "frontier_mode": attack_plan.get("frontier_mode", "disabled"),
            "provider_count": int(attack_plan.get("provider_count") or 0),
            "providers": list_or_empty(attack_plan.get("providers")),
            "diversity_confidence": attack_plan.get("diversity_confidence", "none"),
        },
        "policy": {
            "deterministic_oracle_authoritative": True,
            "single_provider_self_play_requires_owner_review": True,
            "multi_provider_playoff_requires_owner_review": True,
            "blocking_requires_deterministic_confirmation": True,
        },
        "findings": findings,
        "lineage": lineage,
        "summary": {
            "total_findings": len(findings),
            "replay_candidates": len(replay_candidates),
            "classification_counts": classification_counts,
            "confirmed_regression_count": confirmed_regressions,
            "blocking_required": confirmed_regressions > 0,
        },
    }
    save_json(output_path, payload)

    print("[adversarial-promotion] report={}".format(output_path))
    print(
        "[adversarial-promotion] findings={} replay_candidates={} confirmed_regressions={}".format(
            len(findings), len(replay_candidates), confirmed_regressions
        )
    )

    if args.fail_on_confirmed_regressions and confirmed_regressions > 0:
        print(
            "[adversarial-promotion] FAIL deterministic replay confirmed reproducible regression candidates."
        )
        return 1

    print("[adversarial-promotion] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
