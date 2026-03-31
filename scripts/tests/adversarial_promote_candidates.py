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
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Dict, List


DEFAULT_REPORT_PATH = Path("scripts/tests/adversarial/latest_report.json")
DEFAULT_ATTACK_PLAN_PATH = Path("scripts/tests/adversarial/attack_plan.json")
DEFAULT_MANIFEST_PATH = Path("scripts/tests/adversarial/scenario_manifest.v2.json")
DEFAULT_OUTPUT_PATH = Path("scripts/tests/adversarial/promotion_candidates_report.json")
DEFAULT_FRONTIER_STATUS_PATH = Path("scripts/tests/adversarial/frontier_lane_status.json")
DEFAULT_HYBRID_LANE_CONTRACT_PATH = Path(
    "scripts/tests/adversarial/hybrid_lane_contract.v1.json"
)
RUNNER_PATH = "scripts/tests/adversarial_simulation_runner.py"
HYBRID_CONFIRMATION_MIN_PERCENT = 95.0
HYBRID_FALSE_DISCOVERY_MAX_PERCENT = 20.0
HYBRID_OWNER_DISPOSITION_SLA_HOURS = 48
DETERMINISTIC_CONFORMANCE_LANE = "deterministic_conformance"
EMERGENT_EXPLORATION_LANE = "emergent_exploration"


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


def load_optional_json(path: Path) -> Dict[str, Any]:
    if not path.exists():
        return {}
    return load_json(path)


def save_json(path: Path, payload: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def dict_or_empty(value: Any) -> Dict[str, Any]:
    return value if isinstance(value, dict) else {}


def list_or_empty(value: Any) -> List[Any]:
    return value if isinstance(value, list) else []


def collapse_whitespace(value: str) -> str:
    return " ".join(str(value or "").split())


def stable_finding_id(record: Dict[str, Any]) -> str:
    canonical_basis = {
        "candidate_id": str(record.get("candidate_id") or ""),
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
        governance_passed = bool(candidate.get("governance_passed", False))
        if not governance_passed:
            continue
        candidate_id = str(candidate.get("candidate_id") or "").strip()
        scenario_id = str(candidate.get("scenario_id") or "").strip()
        if not scenario_id:
            continue
        source_scenario_id = str(candidate.get("source_scenario_id") or scenario_id).strip()
        generation_kind = str(candidate.get("generation_kind") or "seed").strip()
        mutation_class = str(candidate.get("mutation_class") or generation_kind).strip()
        behavioral_class = str(candidate.get("behavioral_class") or "baseline").strip()
        novelty_score = float(candidate.get("novelty_score") or 0.0)
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
            "candidate_id": candidate_id,
            "source_scenario_id": source_scenario_id,
            "generation_kind": generation_kind,
            "mutation_class": mutation_class,
            "behavioral_class": behavioral_class,
            "novelty_score": max(0.0, min(1.0, novelty_score)),
            "governance_passed": governance_passed,
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
        "owner_disposition": "pending" if owner_review_required else "not_required",
        "owner_disposition_due_at_unix": 0,
        "review_notes": review_notes,
        "promoted_scenario": promoted_scenario,
    }


def evaluate_hybrid_governance(lineage: List[Dict[str, Any]], *, now_unix: int) -> Dict[str, Any]:
    replay_candidates = len(lineage)
    confirmed_count = len(
        [
            row
            for row in lineage
            if str(dict_or_empty(row).get("classification") or "") == "confirmed_reproducible"
        ]
    )
    not_reproducible_count = len(
        [
            row
            for row in lineage
            if str(dict_or_empty(row).get("classification") or "") == "not_reproducible"
        ]
    )
    confirmation_rate = (
        100.0
        if replay_candidates == 0
        else (confirmed_count * 100.0) / float(replay_candidates)
    )
    false_discovery_rate = (
        0.0
        if replay_candidates == 0
        else (not_reproducible_count * 100.0) / float(replay_candidates)
    )

    overdue_owner_reviews = 0
    for row in lineage:
        promotion = dict_or_empty(dict_or_empty(row).get("promotion"))
        if not bool(promotion.get("owner_review_required")):
            continue
        disposition = str(promotion.get("owner_disposition") or "pending").strip().lower()
        if disposition in {"accepted", "rejected", "dismissed"}:
            continue
        due_at = int(promotion.get("owner_disposition_due_at_unix") or 0)
        if due_at > 0 and now_unix > due_at:
            overdue_owner_reviews += 1

    failures: List[str] = []
    if confirmation_rate < HYBRID_CONFIRMATION_MIN_PERCENT:
        failures.append(
            "deterministic_confirmation_rate_below_min"
            f":required={HYBRID_CONFIRMATION_MIN_PERCENT} observed={confirmation_rate:.2f}"
        )
    if false_discovery_rate > HYBRID_FALSE_DISCOVERY_MAX_PERCENT:
        failures.append(
            "false_discovery_rate_above_max"
            f":required<={HYBRID_FALSE_DISCOVERY_MAX_PERCENT} observed={false_discovery_rate:.2f}"
        )
    if overdue_owner_reviews > 0:
        failures.append(
            "owner_disposition_sla_exceeded"
            f":required<={HYBRID_OWNER_DISPOSITION_SLA_HOURS}h observed_overdue={overdue_owner_reviews}"
        )

    return {
        "thresholds": {
            "deterministic_confirmation_min_percent": HYBRID_CONFIRMATION_MIN_PERCENT,
            "false_discovery_max_percent": HYBRID_FALSE_DISCOVERY_MAX_PERCENT,
            "owner_disposition_sla_hours": HYBRID_OWNER_DISPOSITION_SLA_HOURS,
        },
        "observed": {
            "replay_candidates": replay_candidates,
            "confirmed_reproducible_count": confirmed_count,
            "not_reproducible_count": not_reproducible_count,
            "deterministic_confirmation_rate_percent": round(confirmation_rate, 2),
            "false_discovery_rate_percent": round(false_discovery_rate, 2),
            "overdue_owner_review_count": overdue_owner_reviews,
        },
        "thresholds_passed": len(failures) == 0,
        "failures": failures,
    }


def evaluate_discovery_quality_metrics(
    findings: List[Dict[str, Any]],
    lineage: List[Dict[str, Any]],
    attack_plan: Dict[str, Any],
    hybrid_governance: Dict[str, Any],
    frontier_status: Dict[str, Any],
) -> Dict[str, Any]:
    generated_findings = [
        row
        for row in findings
        if str(row.get("generation_kind") or "").strip() == "mutation"
    ]
    confirmed_rows = [
        row
        for row in lineage
        if str(row.get("classification") or "").strip() == "confirmed_reproducible"
    ]
    novel_confirmed_regressions = len(
        [
            row
            for row in confirmed_rows
            if str(dict_or_empty(row).get("generated_candidate", {}).get("generation_kind") or "")
            == "mutation"
        ]
    )
    configured = max(0, int(frontier_status.get("provider_count_configured") or 0))
    healthy = max(0, int(frontier_status.get("provider_count_healthy") or 0))
    provider_outage_impact_percent = (
        0.0
        if configured == 0
        else ((configured - healthy) * 100.0) / float(max(1, configured))
    )
    return {
        "candidate_count": len(findings),
        "generated_candidate_count": len(generated_findings),
        "novel_confirmed_regressions": novel_confirmed_regressions,
        "false_discovery_rate_percent": float(
            dict_or_empty(hybrid_governance.get("observed")).get(
                "false_discovery_rate_percent", 0.0
            )
        ),
        "provider_outage_impact_percent": round(max(0.0, provider_outage_impact_percent), 2),
        "provider_outage_status": str(frontier_status.get("status") or "unknown"),
        "deterministic_blocking_policy": "no stochastic frontier output can block release without deterministic confirmation",
        "blocking_requires_deterministic_confirmation": True,
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


def replay_promotion_admin_headers() -> Dict[str, str]:
    api_key = str(os.environ.get("SHUMA_API_KEY") or "").strip()
    if not api_key:
        raise ValueError("missing SHUMA_API_KEY required for replay-promotion materialization")

    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
        "X-Forwarded-For": "127.0.0.42",
    }
    forwarded_secret = str(os.environ.get("SHUMA_FORWARDED_IP_SECRET") or "").strip()
    if forwarded_secret:
        headers["X-Shuma-Forwarded-Secret"] = forwarded_secret
    return headers


def materialize_backend_replay_promotion(payload: Dict[str, Any]) -> Dict[str, Any]:
    base_url = str(os.environ.get("SHUMA_BASE_URL") or "").strip().rstrip("/")
    if not base_url:
        raise ValueError("missing SHUMA_BASE_URL required for replay-promotion materialization")

    request = urllib.request.Request(
        url=f"{base_url}/shuma/admin/replay-promotion",
        data=json.dumps(payload, separators=(",", ":")).encode("utf-8"),
        method="POST",
        headers=replay_promotion_admin_headers(),
    )
    try:
        with urllib.request.urlopen(request, timeout=20.0) as response:
            body = response.read().decode("utf-8", errors="replace")
            status = int(response.getcode() or 0)
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        detail = collapse_whitespace(body)[:200] or "<empty>"
        raise ValueError(
            "replay-promotion materialization failed: "
            f"status={int(exc.code)} body={detail}"
        ) from exc
    except urllib.error.URLError as exc:
        raise ValueError(
            f"replay-promotion materialization failed: {exc.reason}"
        ) from exc

    if status != 200:
        detail = collapse_whitespace(body)[:200] or "<empty>"
        raise ValueError(
            f"replay-promotion materialization failed: status={status} body={detail}"
        )

    try:
        response_payload = json.loads(body)
    except Exception as exc:
        detail = collapse_whitespace(body)[:200] or "<empty>"
        raise ValueError(
            f"replay-promotion materialization returned invalid JSON: {detail}"
        ) from exc
    if not isinstance(response_payload, dict):
        raise ValueError("replay-promotion materialization returned a non-object payload")
    if response_payload.get("updated") is not True:
        raise ValueError("replay-promotion materialization did not confirm updated=true")
    summary = dict_or_empty(response_payload.get("summary"))
    if str(summary.get("availability") or "") != "materialized":
        raise ValueError("replay-promotion materialization did not return materialized summary")
    return response_payload


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
        "--frontier-status",
        default=str(DEFAULT_FRONTIER_STATUS_PATH),
        help="Path to frontier lane status JSON for outage-impact metrics",
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
    frontier_status_path = Path(args.frontier_status)
    output_path = Path(args.output)

    report = load_json(report_path)
    attack_plan = load_json(attack_plan_path)
    frontier_status = load_optional_json(frontier_status_path)

    now_unix = int(time.time())
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
        if bool(promotion.get("owner_review_required")):
            promotion["owner_disposition_due_at_unix"] = (
                now_unix + (HYBRID_OWNER_DISPOSITION_SLA_HOURS * 3600)
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
                "candidate_id": finding.get("candidate_id"),
                "scenario_id": scenario_id,
                "classification": classification,
                "source_lane": EMERGENT_EXPLORATION_LANE,
                "deterministic_replay_lane": DETERMINISTIC_CONFORMANCE_LANE,
                "release_blocking_authority": classification == "confirmed_reproducible",
                "generated_candidate": {
                    "candidate_id": finding.get("candidate_id"),
                    "source_scenario_id": finding.get("source_scenario_id"),
                    "generation_kind": finding.get("generation_kind"),
                    "mutation_class": finding.get("mutation_class"),
                    "behavioral_class": finding.get("behavioral_class"),
                    "novelty_score": finding.get("novelty_score"),
                },
                "deterministic_confirmation": {
                    "lane": DETERMINISTIC_CONFORMANCE_LANE,
                    "classification": classification,
                    "replay_status": replay_result.get("status"),
                },
                "candidate": {
                    "scenario_family": finding.get("scenario_family"),
                    "path": finding.get("path"),
                    "expected_outcome": finding.get("expected_outcome"),
                    "observed_outcome": finding.get("observed_outcome"),
                    "severity": finding.get("severity"),
                    "risk": finding.get("risk"),
                    "frontier_mode": finding.get("frontier_mode"),
                    "diversity_confidence": finding.get("diversity_confidence"),
                    "lane": EMERGENT_EXPLORATION_LANE,
                },
                "replay": replay_result,
                "promotion": promotion,
            }
        )

    hybrid_governance = evaluate_hybrid_governance(lineage, now_unix=now_unix)
    discovery_quality_metrics = evaluate_discovery_quality_metrics(
        findings=findings,
        lineage=lineage,
        attack_plan=attack_plan,
        hybrid_governance=hybrid_governance,
        frontier_status=frontier_status,
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
            "frontier_status_path": str(frontier_status_path),
        },
        "frontier": {
            "frontier_mode": attack_plan.get("frontier_mode", "disabled"),
            "provider_count": int(attack_plan.get("provider_count") or 0),
            "providers": list_or_empty(attack_plan.get("providers")),
            "diversity_confidence": attack_plan.get("diversity_confidence", "none"),
            "attack_generation_contract": dict_or_empty(
                attack_plan.get("attack_generation_contract")
            ),
            "generation_summary": dict_or_empty(attack_plan.get("generation_summary")),
        },
        "lane_metadata": {
            "contract_path": str(DEFAULT_HYBRID_LANE_CONTRACT_PATH),
            "deterministic_conformance_lane": {
                "lane_id": DETERMINISTIC_CONFORMANCE_LANE,
                "release_blocking": True,
                "authority": "deterministic_replay_confirmation",
            },
            "emergent_exploration_lane": {
                "lane_id": EMERGENT_EXPLORATION_LANE,
                "release_blocking": False,
                "authority": "discovery_only",
            },
        },
        "promotion_pipeline": {
            "steps": [
                "generated_candidate",
                "deterministic_replay_confirmation",
                "owner_review_disposition",
                "promoted_blocking_scenario",
            ],
            "blocking_requires_deterministic_confirmation": True,
        },
        "policy": {
            "deterministic_oracle_authoritative": True,
            "single_provider_self_play_requires_owner_review": True,
            "multi_provider_playoff_requires_owner_review": True,
            "blocking_requires_deterministic_confirmation": True,
        },
        "hybrid_governance": hybrid_governance,
        "discovery_quality_metrics": discovery_quality_metrics,
        "findings": findings,
        "lineage": lineage,
        "summary": {
            "total_findings": len(findings),
            "replay_candidates": len(replay_candidates),
            "classification_counts": classification_counts,
            "confirmed_regression_count": confirmed_regressions,
            "novel_confirmed_regression_count": int(
                discovery_quality_metrics.get("novel_confirmed_regressions") or 0
            ),
            "false_discovery_rate_percent": float(
                discovery_quality_metrics.get("false_discovery_rate_percent") or 0.0
            ),
            "provider_outage_impact_percent": float(
                discovery_quality_metrics.get("provider_outage_impact_percent") or 0.0
            ),
            "blocking_required": confirmed_regressions > 0
            or not bool(hybrid_governance.get("thresholds_passed")),
        },
    }
    save_json(output_path, payload)

    try:
        backend_materialization = materialize_backend_replay_promotion(payload)
    except ValueError as exc:
        print(f"[adversarial-promotion] FAIL {exc}")
        return 1
    payload["backend_materialization"] = {
        "status": "materialized",
        "summary": dict_or_empty(backend_materialization.get("summary")),
    }
    save_json(output_path, payload)

    print("[adversarial-promotion] report={}".format(output_path))
    print(
        "[adversarial-promotion] findings={} replay_candidates={} confirmed_regressions={}".format(
            len(findings), len(replay_candidates), confirmed_regressions
        )
    )

    if args.fail_on_confirmed_regressions and bool(payload["summary"]["blocking_required"]):
        print(
            "[adversarial-promotion] FAIL deterministic replay/hybrid governance requires blocking action."
        )
        return 1

    print("[adversarial-promotion] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
