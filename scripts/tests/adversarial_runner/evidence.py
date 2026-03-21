"""Evidence shaping helpers for adversarial runner reports and checks."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Callable, Dict, Iterable, List, Mapping, Optional, Tuple

from scripts.tests.adversarial_runner.runtime_state import ScenarioResult
from scripts.tests.adversarial_runner.shared import dict_or_empty, int_or_zero, list_or_empty


def extract_monitoring_snapshot(
    payload: Dict[str, Any],
    *,
    nested_dict_value_fn: Callable[[Dict[str, Any], Tuple[str, ...]], Any],
) -> Dict[str, Any]:
    summary = dict_or_empty(payload.get("summary"))
    details = dict_or_empty(payload.get("details"))
    retention_health = dict_or_empty(payload.get("retention_health"))
    if not retention_health:
        retention_health = dict_or_empty(nested_dict_value_fn(details, ("retention_health",)))
    cost_governance = dict_or_empty(nested_dict_value_fn(details, ("cost_governance",)))
    security_privacy = dict_or_empty(payload.get("security_privacy"))
    if not security_privacy:
        security_privacy = dict_or_empty(nested_dict_value_fn(details, ("security_privacy",)))
    tarpit_details = dict_or_empty(nested_dict_value_fn(details, ("tarpit",)))
    recent_events = nested_dict_value_fn(details, ("events", "recent_events"))
    recent_event_count = len(recent_events) if isinstance(recent_events, list) else 0
    recent_event_reasons = []
    if isinstance(recent_events, list):
        for event in recent_events:
            reason = str(dict_or_empty(event).get("reason") or "").strip().lower()
            if reason:
                recent_event_reasons.append(reason)

    coverage = {
        "honeypot_hits": int_or_zero(nested_dict_value_fn(summary, ("honeypot", "total_hits"))),
        "challenge_failures": int_or_zero(
            nested_dict_value_fn(summary, ("challenge", "total_failures"))
        ),
        "not_a_bot_pass": int_or_zero(nested_dict_value_fn(summary, ("not_a_bot", "pass"))),
        "not_a_bot_fail": int_or_zero(nested_dict_value_fn(summary, ("not_a_bot", "fail"))),
        "not_a_bot_replay": int_or_zero(nested_dict_value_fn(summary, ("not_a_bot", "replay"))),
        "not_a_bot_escalate": int_or_zero(
            nested_dict_value_fn(summary, ("not_a_bot", "escalate"))
        ),
        "pow_attempts": int_or_zero(nested_dict_value_fn(summary, ("pow", "total_attempts"))),
        "pow_successes": int_or_zero(nested_dict_value_fn(summary, ("pow", "total_successes"))),
        "pow_failures": int_or_zero(nested_dict_value_fn(summary, ("pow", "total_failures"))),
        "rate_violations": int_or_zero(nested_dict_value_fn(summary, ("rate", "total_violations"))),
        "rate_limited": int_or_zero(
            nested_dict_value_fn(summary, ("rate", "outcomes", "limited"))
        ),
        "rate_banned": int_or_zero(nested_dict_value_fn(summary, ("rate", "outcomes", "banned"))),
        "geo_violations": int_or_zero(nested_dict_value_fn(summary, ("geo", "total_violations"))),
        "geo_challenge": int_or_zero(nested_dict_value_fn(summary, ("geo", "actions", "challenge"))),
        "geo_maze": int_or_zero(nested_dict_value_fn(summary, ("geo", "actions", "maze"))),
        "geo_block": int_or_zero(nested_dict_value_fn(summary, ("geo", "actions", "block"))),
        "maze_hits": int_or_zero(nested_dict_value_fn(details, ("maze", "total_hits"))),
        "tarpit_activations_progressive": int_or_zero(
            nested_dict_value_fn(details, ("tarpit", "metrics", "activations", "progressive"))
        ),
        "tarpit_progress_advanced": int_or_zero(
            nested_dict_value_fn(details, ("tarpit", "metrics", "progress_outcomes", "advanced"))
        ),
        "cdp_detections": int_or_zero(
            nested_dict_value_fn(details, ("cdp", "stats", "total_detections"))
        ),
        "fingerprint_events": int_or_zero(
            nested_dict_value_fn(details, ("cdp", "fingerprint_stats", "events"))
        ),
        "ban_count": int_or_zero(nested_dict_value_fn(details, ("analytics", "ban_count"))),
        "recent_event_count": recent_event_count,
    }

    components = {
        "honeypot_hits": coverage["honeypot_hits"],
        "challenge_failures": coverage["challenge_failures"],
        "not_a_bot_submitted": int_or_zero(
            nested_dict_value_fn(summary, ("not_a_bot", "submitted"))
        ),
        "pow_attempts": coverage["pow_attempts"],
        "rate_violations": coverage["rate_violations"],
        "geo_violations": coverage["geo_violations"],
    }

    return {
        "fingerprint_events": coverage["fingerprint_events"],
        "monitoring_total": sum(components.values()),
        "components": components,
        "coverage": coverage,
        "tarpit": tarpit_details,
        "retention_health": retention_health,
        "cost_governance": cost_governance,
        "security_privacy": security_privacy,
        "recent_event_reasons": sorted(set(recent_event_reasons)),
    }


def compute_coverage_deltas(before: Dict[str, Any], after: Dict[str, Any]) -> Dict[str, int]:
    keys = set(before.keys()).union(after.keys())
    deltas: Dict[str, int] = {}
    for key in sorted(keys):
        before_count = int_or_zero(before.get(key))
        after_count = int_or_zero(after.get(key))
        deltas[key] = max(0, after_count - before_count)
    return deltas


def derive_coverage_deltas_from_simulation_event_reasons(
    simulation_event_reason_counts_delta: Dict[str, int],
) -> Dict[str, int]:
    deltas: Dict[str, int] = {}
    for reason, observed_count in simulation_event_reason_counts_delta.items():
        normalized = str(reason).strip().lower()
        count = max(0, int_or_zero(observed_count))
        if not normalized or count <= 0:
            continue
        if normalized.startswith("not_a_bot_pass"):
            deltas["not_a_bot_pass"] = deltas.get("not_a_bot_pass", 0) + count
        elif normalized == "not_a_bot_fail" or normalized.startswith("not_a_bot_submit_fail"):
            deltas["not_a_bot_fail"] = deltas.get("not_a_bot_fail", 0) + count
        elif normalized.startswith("not_a_bot_replay"):
            deltas["not_a_bot_replay"] = deltas.get("not_a_bot_replay", 0) + count
        elif normalized.startswith("not_a_bot_escalate"):
            deltas["not_a_bot_escalate"] = deltas.get("not_a_bot_escalate", 0) + count
        elif normalized.startswith("not_a_bot_submit_abuse") or normalized.startswith(
            "not_a_bot_abuse"
        ):
            deltas["not_a_bot_replay"] = deltas.get("not_a_bot_replay", 0) + count
    return deltas


def compute_reason_count_deltas(
    before: Optional[Dict[str, Any]],
    after: Optional[Dict[str, Any]],
) -> Dict[str, int]:
    before_counts = dict_or_empty(before)
    after_counts = dict_or_empty(after)
    deltas: Dict[str, int] = {}
    for reason in sorted(set(before_counts.keys()).union(after_counts.keys())):
        delta = max(0, int_or_zero(after_counts.get(reason)) - int_or_zero(before_counts.get(reason)))
        if delta > 0:
            deltas[str(reason)] = delta
    return deltas


def build_scenario_execution_evidence(
    scenario_id: str,
    request_count_before: int,
    request_count_after: int,
    monitoring_before: Dict[str, Any],
    monitoring_after: Dict[str, Any],
    simulation_event_count_before: int,
    simulation_event_count_after: int,
    simulation_event_reasons_before: Optional[List[str]] = None,
    simulation_event_reasons_after: Optional[List[str]] = None,
    simulation_event_reason_counts_before: Optional[Dict[str, Any]] = None,
    simulation_event_reason_counts_after: Optional[Dict[str, Any]] = None,
    driver_class: str = "",
    browser_realism: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    runtime_request_count = max(0, int_or_zero(request_count_after) - int_or_zero(request_count_before))
    monitoring_total_delta = max(
        0,
        int_or_zero(monitoring_after.get("monitoring_total"))
        - int_or_zero(monitoring_before.get("monitoring_total")),
    )
    coverage_deltas = compute_coverage_deltas(
        dict_or_empty(monitoring_before.get("coverage")),
        dict_or_empty(monitoring_after.get("coverage")),
    )
    raw_simulation_event_count_delta = max(
        0,
        int_or_zero(simulation_event_count_after) - int_or_zero(simulation_event_count_before),
    )
    reason_count_deltas = compute_reason_count_deltas(
        simulation_event_reason_counts_before,
        simulation_event_reason_counts_after,
    )
    if reason_count_deltas:
        simulation_event_reasons_delta = sorted(reason_count_deltas.keys())
    else:
        reasons_before = {
            str(reason).strip().lower()
            for reason in list_or_empty(simulation_event_reasons_before)
            if str(reason).strip()
        }
        reasons_after = {
            str(reason).strip().lower()
            for reason in list_or_empty(simulation_event_reasons_after)
            if str(reason).strip()
        }
        simulation_event_reasons_delta = sorted(reasons_after - reasons_before)
        reason_count_deltas = {reason: 1 for reason in simulation_event_reasons_delta}
    simulation_event_count_delta = max(raw_simulation_event_count_delta, len(simulation_event_reasons_delta))
    reason_coverage_deltas = derive_coverage_deltas_from_simulation_event_reasons(reason_count_deltas)
    for key, value in reason_coverage_deltas.items():
        coverage_deltas[key] = max(0, int_or_zero(coverage_deltas.get(key))) + max(
            0, int_or_zero(value)
        )
    if int_or_zero(coverage_deltas.get("recent_event_count")) <= 0 and simulation_event_count_delta > 0:
        coverage_deltas["recent_event_count"] = simulation_event_count_delta
    coverage_delta_total = sum(max(0, int_or_zero(value)) for value in coverage_deltas.values())
    browser_realism = dict_or_empty(browser_realism)
    browser_js_executed = bool(browser_realism.get("browser_js_executed"))
    browser_dom_events = max(0, int_or_zero(browser_realism.get("browser_dom_events")))
    browser_storage_mode = str(browser_realism.get("browser_storage_mode") or "")
    browser_challenge_dom_path = [
        str(item).strip()
        for item in list_or_empty(browser_realism.get("browser_challenge_dom_path"))
        if str(item).strip()
    ]
    browser_correlation_ids = [
        str(item).strip()
        for item in list_or_empty(browser_realism.get("browser_correlation_ids"))
        if str(item).strip()
    ]
    browser_request_lineage_count = max(
        0,
        int_or_zero(browser_realism.get("browser_request_lineage_count")),
    )
    browser_driver_runtime = str(browser_realism.get("browser_driver_runtime") or "")
    has_browser_execution_evidence = str(driver_class).strip() != "browser_realistic" or (
        browser_js_executed and browser_dom_events > 0 and bool(browser_challenge_dom_path)
    )
    has_runtime_telemetry_evidence = runtime_request_count > 0 and (
        monitoring_total_delta > 0 or coverage_delta_total > 0 or simulation_event_count_delta > 0
    )

    return {
        "scenario_id": str(scenario_id),
        "driver_class": str(driver_class).strip(),
        "runtime_request_count": runtime_request_count,
        "monitoring_total_delta": monitoring_total_delta,
        "coverage_delta_total": coverage_delta_total,
        "coverage_deltas": coverage_deltas,
        "simulation_event_count_delta": simulation_event_count_delta,
        "simulation_event_reasons_delta": simulation_event_reasons_delta,
        "has_runtime_telemetry_evidence": has_runtime_telemetry_evidence,
        "browser_driver_runtime": browser_driver_runtime,
        "browser_js_executed": browser_js_executed,
        "browser_dom_events": browser_dom_events,
        "browser_storage_mode": browser_storage_mode,
        "browser_challenge_dom_path": browser_challenge_dom_path,
        "browser_correlation_ids": browser_correlation_ids,
        "browser_request_lineage_count": browser_request_lineage_count,
        "has_browser_execution_evidence": has_browser_execution_evidence,
    }


def build_runtime_telemetry_evidence_checks(
    results: List[ScenarioResult],
    scenario_execution_evidence: Dict[str, Dict[str, Any]],
    required_fields: List[str],
    *,
    real_traffic_contract_path: Path,
) -> List[Dict[str, Any]]:
    checks: List[Dict[str, Any]] = []
    passed_result_ids = [result.id for result in results if result.passed]
    if not passed_result_ids:
        checks.append(
            {
                "name": "runtime_evidence_passed_scenarios_present",
                "passed": True,
                "detail": "no passed scenarios in run; runtime evidence requirement vacuously satisfied",
                "observed": 0,
                "threshold_source": str(real_traffic_contract_path),
            }
        )
        return checks

    missing_evidence_ids: List[str] = []
    missing_required_fields: Dict[str, List[str]] = {}
    missing_runtime_telemetry_ids: List[str] = []

    for scenario_id in passed_result_ids:
        evidence = dict_or_empty(scenario_execution_evidence.get(scenario_id))
        if not evidence:
            missing_evidence_ids.append(scenario_id)
            continue
        missing_fields_for_scenario = [field for field in required_fields if field not in evidence]
        if missing_fields_for_scenario:
            missing_required_fields[scenario_id] = missing_fields_for_scenario
            continue
        if not bool(evidence.get("has_runtime_telemetry_evidence")):
            missing_runtime_telemetry_ids.append(scenario_id)

    checks.append(
        {
            "name": "runtime_evidence_rows_for_passed_scenarios",
            "passed": not missing_evidence_ids,
            "detail": (
                "all passed scenarios include execution evidence"
                if not missing_evidence_ids
                else f"missing_evidence_ids={missing_evidence_ids}"
            ),
            "observed": len(passed_result_ids) - len(missing_evidence_ids),
            "minimum": len(passed_result_ids),
            "threshold_source": str(real_traffic_contract_path),
        }
    )
    checks.append(
        {
            "name": "runtime_evidence_required_fields_present",
            "passed": not missing_required_fields,
            "detail": (
                "all evidence rows include required fields"
                if not missing_required_fields
                else f"missing_required_fields={missing_required_fields}"
            ),
            "observed": len(passed_result_ids) - len(missing_required_fields),
            "minimum": len(passed_result_ids),
            "threshold_source": str(real_traffic_contract_path),
        }
    )
    checks.append(
        {
            "name": "runtime_evidence_telemetry_for_passed_scenarios",
            "passed": not missing_runtime_telemetry_ids,
            "detail": (
                "all passed scenarios have runtime telemetry evidence"
                if not missing_runtime_telemetry_ids
                else f"missing_runtime_telemetry_ids={missing_runtime_telemetry_ids}"
            ),
            "observed": len(passed_result_ids) - len(missing_runtime_telemetry_ids),
            "minimum": len(passed_result_ids),
            "threshold_source": str(real_traffic_contract_path),
        }
    )
    return checks


def build_browser_execution_evidence_checks(
    selected_scenarios: List[Dict[str, Any]],
    results: List[ScenarioResult],
    scenario_execution_evidence: Dict[str, Dict[str, Any]],
    *,
    scenario_driver_class_fn: Callable[[Dict[str, Any]], str],
) -> List[Dict[str, Any]]:
    checks: List[Dict[str, Any]] = []
    scenario_by_id = {str(scenario.get("id") or ""): scenario for scenario in selected_scenarios}
    browser_result_ids = [
        result.id
        for result in results
        if result.passed
        and scenario_driver_class_fn(dict_or_empty(scenario_by_id.get(result.id))) == "browser_realistic"
    ]
    if not browser_result_ids:
        checks.append(
            {
                "name": "browser_execution_required_rows_present",
                "passed": True,
                "detail": "no passed browser_realistic scenarios in run; browser evidence checks vacuously satisfied",
                "observed": 0,
                "threshold_source": "SIM2-GC-7 browser execution contract",
            }
        )
        return checks

    missing_rows: List[str] = []
    missing_js: List[str] = []
    missing_dom_events: List[str] = []
    missing_dom_path: List[str] = []
    missing_correlation: List[str] = []
    missing_lineage: List[str] = []

    for scenario_id in browser_result_ids:
        evidence = dict_or_empty(scenario_execution_evidence.get(scenario_id))
        if not evidence:
            missing_rows.append(scenario_id)
            continue
        if not bool(evidence.get("has_browser_execution_evidence")):
            missing_rows.append(scenario_id)
        if not bool(evidence.get("browser_js_executed")):
            missing_js.append(scenario_id)
        if int_or_zero(evidence.get("browser_dom_events")) <= 0:
            missing_dom_events.append(scenario_id)
        if not list_or_empty(evidence.get("browser_challenge_dom_path")):
            missing_dom_path.append(scenario_id)
        if not list_or_empty(evidence.get("browser_correlation_ids")):
            missing_correlation.append(scenario_id)
        if int_or_zero(evidence.get("browser_request_lineage_count")) <= 0:
            missing_lineage.append(scenario_id)

    checks.append(
        {
            "name": "browser_execution_required_rows_present",
            "passed": not missing_rows,
            "detail": (
                "all passed browser_realistic scenarios include browser evidence rows"
                if not missing_rows
                else f"missing_rows={missing_rows}"
            ),
            "observed": len(browser_result_ids) - len(missing_rows),
            "minimum": len(browser_result_ids),
            "threshold_source": "SIM2-GC-7-5 browser evidence fields",
        }
    )
    checks.append(
        {
            "name": "browser_execution_js_executed",
            "passed": not missing_js,
            "detail": "all browser scenarios executed JS" if not missing_js else f"missing_js={missing_js}",
            "observed": len(browser_result_ids) - len(missing_js),
            "minimum": len(browser_result_ids),
            "threshold_source": "SIM2-GC-7-3 browser JS/runtime checks",
        }
    )
    checks.append(
        {
            "name": "browser_execution_dom_events",
            "passed": not missing_dom_events,
            "detail": (
                "all browser scenarios produced DOM events"
                if not missing_dom_events
                else f"missing_dom_events={missing_dom_events}"
            ),
            "observed": len(browser_result_ids) - len(missing_dom_events),
            "minimum": len(browser_result_ids),
            "threshold_source": "SIM2-GC-7-2 challenge DOM interaction primitives",
        }
    )
    checks.append(
        {
            "name": "browser_execution_dom_paths",
            "passed": not missing_dom_path,
            "detail": (
                "all browser scenarios produced challenge DOM path evidence"
                if not missing_dom_path
                else f"missing_dom_path={missing_dom_path}"
            ),
            "observed": len(browser_result_ids) - len(missing_dom_path),
            "minimum": len(browser_result_ids),
            "threshold_source": "SIM2-GC-7-5 challenge_dom_path evidence",
        }
    )
    checks.append(
        {
            "name": "browser_execution_correlation_ids",
            "passed": not missing_correlation and not missing_lineage,
            "detail": (
                "all browser scenarios produced request lineage and correlation ids"
                if not missing_correlation and not missing_lineage
                else (
                    f"missing_correlation={missing_correlation} "
                    f"missing_lineage={missing_lineage}"
                )
            ),
            "observed": len(browser_result_ids) - max(
                len(set(missing_correlation)),
                len(set(missing_lineage)),
            ),
            "minimum": len(browser_result_ids),
            "threshold_source": "SIM2-GC-7-5 monitoring correlation lineage",
        }
    )
    return checks


def evaluate_scenario_intent_signal(
    signal: Dict[str, Any],
    result: ScenarioResult,
    evidence: Dict[str, Any],
) -> Dict[str, Any]:
    signal_type = str(signal.get("type") or "").strip()
    minimum = max(1, int_or_zero(signal.get("minimum")) or 1)
    observed = 0
    source = ""

    if signal_type == "coverage_delta":
        key = str(signal.get("key") or "").strip()
        observed = max(0, int_or_zero(dict_or_empty(evidence.get("coverage_deltas")).get(key)))
        source = f"coverage_deltas.{key}"
    elif signal_type == "outcome_equals":
        value = str(signal.get("value") or "").strip()
        observed = 1 if str(result.observed_outcome or "").strip() == value else 0
        source = f"result.observed_outcome=={value}"
    elif signal_type == "simulation_event_count_delta":
        observed = max(0, int_or_zero(evidence.get("simulation_event_count_delta")))
        source = "simulation_event_count_delta"
    elif signal_type == "simulation_event_reason_prefix":
        prefix = str(signal.get("prefix") or "").strip().lower()
        reasons = [
            str(reason).strip().lower()
            for reason in list_or_empty(evidence.get("simulation_event_reasons_delta"))
            if str(reason).strip()
        ]
        observed = len([reason for reason in reasons if reason.startswith(prefix)])
        source = f"simulation_event_reasons_delta prefix={prefix}"
    elif signal_type == "realism_metric_min":
        key = str(signal.get("key") or "").strip()
        observed = max(0, int_or_zero(dict_or_empty(result.realism).get(key)))
        source = f"result.realism.{key}"
    else:
        source = f"unsupported_signal_type={signal_type}"

    return {
        "signal_type": signal_type,
        "minimum": minimum,
        "observed": observed,
        "passed": observed >= minimum,
        "source": source,
    }


def build_scenario_intent_checks(
    selected_scenarios: List[Dict[str, Any]],
    results: List[ScenarioResult],
    scenario_execution_evidence: Dict[str, Dict[str, Any]],
    *,
    intent_rows_by_id: Dict[str, Dict[str, Any]],
    scenario_driver_class_fn: Callable[[Dict[str, Any]], str],
    intent_matrix_path: Path,
    progress_int_keys: Iterable[str],
) -> List[Dict[str, Any]]:
    checks: List[Dict[str, Any]] = []
    scenario_by_id = {str(scenario.get("id") or ""): scenario for scenario in selected_scenarios}

    passed_results = [result for result in results if result.passed]
    if not passed_results:
        checks.append(
            {
                "name": "scenario_intent_rows_for_passed_scenarios_present",
                "passed": True,
                "detail": "no passed scenarios in run; scenario intent checks vacuously satisfied",
                "observed": 0,
                "threshold_source": str(intent_matrix_path),
            }
        )
        return checks

    missing_rows: List[str] = []
    for result in passed_results:
        if str(result.id) not in intent_rows_by_id:
            missing_rows.append(str(result.id))
    checks.append(
        {
            "name": "scenario_intent_rows_for_passed_scenarios_present",
            "passed": not missing_rows,
            "detail": (
                "all passed scenarios have scenario intent matrix rows"
                if not missing_rows
                else f"missing_rows={sorted(missing_rows)}"
            ),
            "observed": len(passed_results) - len(missing_rows),
            "minimum": len(passed_results),
            "threshold_source": str(intent_matrix_path),
        }
    )

    progress_keys = sorted(str(key) for key in progress_int_keys)
    for result in passed_results:
        scenario_id = str(result.id)
        row = dict_or_empty(intent_rows_by_id.get(scenario_id))
        if not row:
            continue
        evidence = dict_or_empty(scenario_execution_evidence.get(scenario_id))
        minimum_runtime_requests = max(1, int_or_zero(row.get("minimum_runtime_requests")) or 1)
        runtime_request_count = max(0, int_or_zero(evidence.get("runtime_request_count")))
        checks.append(
            {
                "name": f"scenario_intent_runtime_requests_{scenario_id}",
                "passed": runtime_request_count >= minimum_runtime_requests,
                "detail": (
                    f"runtime_request_count={runtime_request_count} "
                    f"minimum={minimum_runtime_requests}"
                ),
                "observed": runtime_request_count,
                "minimum": minimum_runtime_requests,
                "threshold_source": f"{intent_matrix_path}:{scenario_id}.minimum_runtime_requests",
            }
        )

        defense_signals = dict_or_empty(row.get("defense_signals"))
        required_categories = [
            str(category).strip()
            for category in list_or_empty(row.get("required_defense_categories"))
            if str(category).strip()
        ]
        for category in required_categories:
            signal_rules = [
                dict_or_empty(signal)
                for signal in list_or_empty(defense_signals.get(category))
                if isinstance(signal, dict)
            ]
            signal_checks = [
                evaluate_scenario_intent_signal(signal_rule, result, evidence)
                for signal_rule in signal_rules
            ]
            passed_signals = [signal_check for signal_check in signal_checks if signal_check["passed"]]
            checks.append(
                {
                    "name": f"scenario_intent_{scenario_id}_{category}",
                    "passed": bool(passed_signals),
                    "detail": (
                        f"category={category} "
                        f"passed_signals={len(passed_signals)}/{len(signal_checks)} "
                        f"signals={signal_checks}"
                    ),
                    "observed": len(passed_signals),
                    "minimum": 1,
                    "threshold_source": f"{intent_matrix_path}:{scenario_id}.defense_signals.{category}",
                }
            )

        progression = dict_or_empty(row.get("progression_requirements"))
        realism = dict_or_empty(result.realism)
        selected_scenario = dict_or_empty(scenario_by_id.get(scenario_id))
        observed_driver_class = str(
            evidence.get("driver_class") or scenario_driver_class_fn(selected_scenario)
        ).strip()

        expected_driver_class = str(progression.get("required_driver_class") or "").strip()
        if expected_driver_class:
            checks.append(
                {
                    "name": f"scenario_intent_progression_driver_class_{scenario_id}",
                    "passed": observed_driver_class == expected_driver_class,
                    "detail": (
                        f"driver_class={observed_driver_class or 'none'} "
                        f"required={expected_driver_class}"
                    ),
                    "observed": observed_driver_class,
                    "threshold_source": (
                        f"{intent_matrix_path}:{scenario_id}.progression_requirements.required_driver_class"
                    ),
                }
            )

        expected_persona = str(progression.get("required_persona") or "").strip()
        if expected_persona:
            observed_persona = str(realism.get("persona") or "").strip()
            checks.append(
                {
                    "name": f"scenario_intent_progression_persona_{scenario_id}",
                    "passed": observed_persona == expected_persona,
                    "detail": f"persona={observed_persona or 'none'} required={expected_persona}",
                    "observed": observed_persona,
                    "threshold_source": (
                        f"{intent_matrix_path}:{scenario_id}.progression_requirements.required_persona"
                    ),
                }
            )

        expected_retry_strategy = str(progression.get("required_retry_strategy") or "").strip()
        if expected_retry_strategy:
            observed_retry_strategy = str(realism.get("retry_strategy") or "").strip()
            checks.append(
                {
                    "name": f"scenario_intent_progression_retry_strategy_{scenario_id}",
                    "passed": observed_retry_strategy == expected_retry_strategy,
                    "detail": (
                        f"retry_strategy={observed_retry_strategy or 'none'} "
                        f"required={expected_retry_strategy}"
                    ),
                    "observed": observed_retry_strategy,
                    "threshold_source": (
                        f"{intent_matrix_path}:{scenario_id}.progression_requirements.required_retry_strategy"
                    ),
                }
            )

        for key in progress_keys:
            if key not in progression:
                continue
            minimum = max(0, int_or_zero(progression.get(key)))
            realism_key = key.removeprefix("min_")
            observed = max(0, int_or_zero(realism.get(realism_key)))
            checks.append(
                {
                    "name": f"scenario_intent_progression_{key}_{scenario_id}",
                    "passed": observed >= minimum,
                    "detail": f"{realism_key}={observed} minimum={minimum}",
                    "observed": observed,
                    "minimum": minimum,
                    "threshold_source": f"{intent_matrix_path}:{scenario_id}.progression_requirements.{key}",
                }
            )

    return checks


def profile_expected_defense_categories(
    selected_scenarios: List[Dict[str, Any]],
    *,
    defense_noop_coverage_keys: Mapping[str, Any],
) -> List[str]:
    categories = set()
    for scenario in selected_scenarios:
        for category in list_or_empty(scenario.get("expected_defense_categories")):
            normalized = str(category).strip()
            if normalized in defense_noop_coverage_keys:
                categories.add(normalized)
    return sorted(categories)


def build_defense_noop_checks(
    defense_categories: List[str],
    coverage_deltas: Dict[str, int],
    *,
    defense_noop_coverage_keys: Mapping[str, Iterable[str]],
    threshold_source_prefix: str = "scenario.expected_defense_categories",
) -> List[Dict[str, Any]]:
    checks: List[Dict[str, Any]] = []
    for defense in sorted(set(defense_categories)):
        signal_keys = defense_noop_coverage_keys.get(defense)
        if not signal_keys:
            continue
        observed = sum(max(0, int_or_zero(coverage_deltas.get(key))) for key in signal_keys)
        checks.append(
            {
                "name": f"defense_noop_detector_{defense}",
                "passed": observed >= 1,
                "detail": (
                    f"defense={defense} telemetry_delta_total={observed} "
                    f"signal_keys={list(signal_keys)}"
                ),
                "observed": observed,
                "minimum": 1,
                "threshold_source": f"{threshold_source_prefix}.{defense}",
            }
        )
    return checks


def build_coverage_checks(
    coverage_requirements: Dict[str, Any],
    coverage_deltas: Dict[str, int],
) -> List[Dict[str, Any]]:
    checks: List[Dict[str, Any]] = []
    for key in sorted(coverage_requirements.keys()):
        minimum = int_or_zero(coverage_requirements.get(key))
        observed = int_or_zero(coverage_deltas.get(key))
        checks.append(
            {
                "name": f"coverage_{key}",
                "passed": observed >= minimum,
                "detail": f"delta={observed} minimum={minimum}",
                "observed": observed,
                "minimum": minimum,
            }
        )
    return checks


def build_coverage_depth_checks(
    coverage_depth_requirements: Dict[str, Dict[str, Any]],
    *,
    coverage_deltas: Dict[str, int],
    scenario_execution_evidence: Optional[Dict[str, Dict[str, Any]]] = None,
) -> Tuple[List[Dict[str, Any]], List[Dict[str, Any]]]:
    checks: List[Dict[str, Any]] = []
    row_diagnostics: List[Dict[str, Any]] = []
    scenario_execution_evidence = (
        scenario_execution_evidence if isinstance(scenario_execution_evidence, dict) else {}
    )
    for row_id in sorted(coverage_depth_requirements.keys()):
        row = dict_or_empty(coverage_depth_requirements.get(row_id))
        required_metrics = {
            str(metric): int_or_zero(value)
            for metric, value in dict_or_empty(row.get("required_metrics")).items()
        }
        required_scenarios = [
            str(item).strip()
            for item in list_or_empty(row.get("required_scenarios"))
            if str(item).strip()
        ]
        observed_metrics = {
            metric: max(
                int_or_zero(coverage_deltas.get(metric)),
                sum(
                    int_or_zero(
                        dict_or_empty(
                            dict_or_empty(scenario_execution_evidence.get(scenario_id)).get(
                                "coverage_deltas"
                            )
                        ).get(metric)
                    )
                    for scenario_id in required_scenarios
                ),
            )
            for metric in sorted(required_metrics.keys())
        }
        missing_metrics = [
            metric
            for metric in sorted(required_metrics.keys())
            if int_or_zero(observed_metrics.get(metric)) < int_or_zero(required_metrics.get(metric))
        ]
        missing_scenarios = [
            scenario_id
            for scenario_id in required_scenarios
            if scenario_id not in scenario_execution_evidence
        ]
        scenario_contributions: Dict[str, Dict[str, int]] = {}
        for scenario_id in required_scenarios:
            evidence_row = dict_or_empty(scenario_execution_evidence.get(scenario_id))
            coverage_map = dict_or_empty(evidence_row.get("coverage_deltas"))
            scenario_contributions[scenario_id] = {
                metric: int_or_zero(coverage_map.get(metric))
                for metric in sorted(required_metrics.keys())
            }

        passed = not missing_metrics and not missing_scenarios
        detail = (
            f"required={required_metrics} observed={observed_metrics} "
            f"missing_evidence={missing_metrics} missing_scenarios={missing_scenarios}"
        )
        checks.append(
            {
                "name": f"coverage_depth_{row_id}",
                "passed": passed,
                "detail": detail,
                "observed": {
                    "required": required_metrics,
                    "observed": observed_metrics,
                    "missing_evidence": missing_metrics,
                    "missing_scenarios": missing_scenarios,
                    "scenario_contributions": scenario_contributions,
                    "verification_matrix_row_id": str(row.get("verification_matrix_row_id") or ""),
                },
            }
        )
        row_diagnostics.append(
            {
                "row_id": row_id,
                "plan_row": str(row.get("plan_row") or ""),
                "verification_matrix_row_id": str(row.get("verification_matrix_row_id") or ""),
                "required": required_metrics,
                "observed": observed_metrics,
                "missing_evidence": missing_metrics,
                "missing_scenarios": missing_scenarios,
                "scenario_contributions": scenario_contributions,
                "passed": passed,
            }
        )
    return checks, row_diagnostics


def annotate_coverage_checks_with_threshold_source(
    coverage_requirements: Dict[str, Any],
    checks: List[Dict[str, Any]],
    threshold_prefix: str = "profile.gates.coverage_requirements",
) -> List[Dict[str, Any]]:
    annotated: List[Dict[str, Any]] = []
    for check in checks:
        check_copy = dict(check)
        name = str(check_copy.get("name") or "")
        requirement_key = name.removeprefix("coverage_")
        if requirement_key in coverage_requirements:
            check_copy["threshold_source"] = f"{threshold_prefix}.{requirement_key}"
        else:
            check_copy["threshold_source"] = threshold_prefix
        annotated.append(check_copy)
    return annotated
