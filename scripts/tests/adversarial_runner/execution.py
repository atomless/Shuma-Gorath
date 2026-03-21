"""Execution and profile-coordination helpers for the adversarial runner."""

from __future__ import annotations

import os
import re
import urllib.parse
from typing import Any, Callable, Dict, Iterable, List, Optional

from scripts.tests.adversarial_runner.runtime_state import ScenarioResult, SimulationError
from scripts.tests.adversarial_runner.shared import dict_or_empty, int_or_zero, list_or_empty


def retry_strategy_max_attempts(retry_strategy: str) -> int:
    if retry_strategy == "bounded_backoff":
        return 2
    if retry_strategy == "retry_storm":
        return 3
    return 1


def state_mode_bucket(state_mode: str) -> str:
    if state_mode == "stateful_cookie_jar":
        return "stateful"
    if state_mode == "cookie_reset_each_request":
        return "reset_each_request"
    if state_mode == "stateless":
        return "stateless"
    normalized = re.sub(r"[^a-z0-9_]+", "_", str(state_mode or "").strip().lower())
    return normalized or "unknown"


def normalize_request_path(raw_path: str) -> str:
    parsed = urllib.parse.urlparse(raw_path)
    if parsed.scheme and parsed.netloc:
        return parsed.path or "/"
    if raw_path.startswith("/"):
        return raw_path
    return f"/{raw_path}"


def enforce_attacker_request_contract(
    path: str,
    headers: Dict[str, str],
    *,
    attacker_forbidden_path_prefixes: Iterable[str],
    attacker_forbidden_headers: Iterable[str],
    attacker_required_sim_headers: Iterable[str],
) -> None:
    normalized_path = normalize_request_path(str(path or ""))
    lowered_headers = {str(key).strip().lower() for key in headers.keys()}

    for prefix in attacker_forbidden_path_prefixes:
        if normalized_path.startswith(prefix):
            raise SimulationError(f"attacker_plane_forbidden_path path={normalized_path} prefix={prefix}")

    for forbidden_header in attacker_forbidden_headers:
        if forbidden_header in lowered_headers:
            raise SimulationError(
                f"attacker_plane_forbidden_header header={forbidden_header} path={normalized_path}"
            )

    missing_required_headers = sorted(
        header for header in attacker_required_sim_headers if header not in lowered_headers
    )
    if missing_required_headers:
        raise SimulationError(
            "attacker_plane_missing_required_sim_headers "
            f"path={normalized_path} missing={','.join(missing_required_headers)}"
        )


def clamp_int_env(key: str, minimum: int, maximum: int, fallback: int) -> int:
    raw = os.environ.get(key)
    if raw is None:
        return fallback
    try:
        parsed = int(str(raw).strip())
    except Exception:
        return fallback
    if parsed < minimum:
        return minimum
    if parsed > maximum:
        return maximum
    return parsed


def scenario_persona(
    scenario: Dict[str, Any],
    *,
    allowed_traffic_personas: Iterable[str],
) -> str:
    allowed = set(allowed_traffic_personas)
    traffic_model = scenario.get("traffic_model")
    if isinstance(traffic_model, dict):
        persona = str(traffic_model.get("persona") or "").strip()
        if persona in allowed:
            return persona
    tier = str(scenario.get("tier") or "").strip()
    if tier == "SIM-T0":
        return "human_like"
    if tier == "SIM-T1":
        return "benign_automation"
    if tier == "SIM-T2":
        return "suspicious_automation"
    return "adversarial"


def percentile(values: List[int], pct: int) -> int:
    if not values:
        return 0
    sorted_values = sorted(values)
    index = int(round((pct / 100.0) * (len(sorted_values) - 1)))
    index = max(0, min(len(sorted_values) - 1, index))
    return sorted_values[index]


def compute_cohort_metrics(
    selected_scenarios: List[Dict[str, Any]],
    results: List[ScenarioResult],
    *,
    scenario_persona_fn: Callable[[Dict[str, Any]], str],
) -> Dict[str, Dict[str, Any]]:
    scenario_by_id = {str(scenario.get("id") or ""): scenario for scenario in selected_scenarios}
    collateral_outcomes = {"challenge", "maze", "tarpit", "deny_temp"}
    raw: Dict[str, Dict[str, Any]] = {}
    for result in results:
        scenario = scenario_by_id.get(result.id, {})
        persona = scenario_persona_fn(scenario)
        cohort = raw.setdefault(
            persona,
            {
                "total": 0,
                "passed": 0,
                "failed": 0,
                "latency_values": [],
                "outcome_counts": {},
                "collateral_count": 0,
            },
        )
        cohort["total"] += 1
        if result.passed:
            cohort["passed"] += 1
            if result.latency_ms > 0:
                cohort["latency_values"].append(result.latency_ms)
            observed = str(result.observed_outcome or "")
            if observed:
                outcome_counts = cohort["outcome_counts"]
                outcome_counts[observed] = int_or_zero(outcome_counts.get(observed)) + 1
                if observed in collateral_outcomes:
                    cohort["collateral_count"] += 1
        else:
            cohort["failed"] += 1

    metrics: Dict[str, Dict[str, Any]] = {}
    for persona, cohort in raw.items():
        latency_values = list_or_empty(cohort.get("latency_values"))
        total = int_or_zero(cohort.get("total"))
        collateral_count = int_or_zero(cohort.get("collateral_count"))
        metrics[persona] = {
            "total": total,
            "passed": int_or_zero(cohort.get("passed")),
            "failed": int_or_zero(cohort.get("failed")),
            "latency_p95_ms": percentile([int_or_zero(value) for value in latency_values], 95),
            "outcome_counts": dict_or_empty(cohort.get("outcome_counts")),
            "collateral_count": collateral_count,
            "collateral_ratio": (collateral_count / total) if total else 0.0,
        }
    return metrics


def round_robin_sequence_violations(sequence: List[str]) -> List[int]:
    remaining: Dict[str, int] = {}
    for persona in sequence:
        remaining[persona] = int_or_zero(remaining.get(persona)) + 1

    violations: List[int] = []
    for index in range(len(sequence) - 1):
        current = sequence[index]
        following = sequence[index + 1]
        remaining[current] = max(0, int_or_zero(remaining.get(current)) - 1)
        if current != following:
            continue
        other_persona_pending = any(
            int_or_zero(count) > 0 for persona, count in remaining.items() if persona != current
        )
        if other_persona_pending:
            violations.append(index + 1)
    return violations


def compute_realism_metrics(
    selected_scenarios: List[Dict[str, Any]],
    results: List[ScenarioResult],
    persona_scheduler: str,
    *,
    scenario_persona_fn: Callable[[Dict[str, Any]], str],
) -> Dict[str, Any]:
    scenario_by_id = {str(scenario.get("id") or ""): scenario for scenario in selected_scenarios}
    persona_metrics: Dict[str, Dict[str, Any]] = {}
    retry_strategy_metrics: Dict[str, Dict[str, Any]] = {}
    state_mode_metrics: Dict[str, Dict[str, Any]] = {}
    persona_sequence: List[str] = []
    missing_result_ids: List[str] = []

    for result in results:
        scenario = scenario_by_id.get(result.id, {})
        realism = dict_or_empty(result.realism)
        if not realism:
            missing_result_ids.append(result.id)
            continue

        persona = str(realism.get("persona") or scenario_persona_fn(scenario) or "adversarial")
        retry_strategy = str(realism.get("retry_strategy") or "single_attempt")
        state_mode = str(realism.get("state_mode") or "stateless")
        think_time_min = max(0, int_or_zero(realism.get("think_time_ms_min")))
        think_time_max = max(think_time_min, int_or_zero(realism.get("think_time_ms_max")))
        think_time_events = max(0, int_or_zero(realism.get("think_time_events")))
        think_time_ms_total = max(0, int_or_zero(realism.get("think_time_ms_total")))
        request_sequence_count = max(0, int_or_zero(realism.get("request_sequence_count")))
        attempts_total = max(0, int_or_zero(realism.get("attempts_total")))
        retry_attempts = max(0, int_or_zero(realism.get("retry_attempts")))
        retry_backoff_ms_total = max(0, int_or_zero(realism.get("retry_backoff_ms_total")))
        state_headers_sent = max(0, int_or_zero(realism.get("state_headers_sent")))
        state_token_observed = max(0, int_or_zero(realism.get("state_token_observed")))
        state_store_resets = max(0, int_or_zero(realism.get("state_store_resets")))
        state_store_peak_size = max(0, int_or_zero(realism.get("state_store_peak_size")))
        max_attempts_configured = max(1, int_or_zero(realism.get("max_attempts_configured")))

        persona_sequence.append(persona)

        persona_metric = persona_metrics.setdefault(
            persona,
            {
                "scenario_count": 0,
                "request_sequence_total": 0,
                "attempts_total": 0,
                "retry_attempts": 0,
                "retry_backoff_ms_total": 0,
                "think_time_events": 0,
                "think_time_ms_total": 0,
                "expected_think_time_min_total": 0,
                "expected_think_time_max_total": 0,
                "state_headers_sent": 0,
                "state_token_observed": 0,
                "state_store_resets": 0,
                "state_store_peak_size_max": 0,
            },
        )
        persona_metric["scenario_count"] = int_or_zero(persona_metric.get("scenario_count")) + 1
        persona_metric["request_sequence_total"] = (
            int_or_zero(persona_metric.get("request_sequence_total")) + request_sequence_count
        )
        persona_metric["attempts_total"] = int_or_zero(persona_metric.get("attempts_total")) + attempts_total
        persona_metric["retry_attempts"] = int_or_zero(persona_metric.get("retry_attempts")) + retry_attempts
        persona_metric["retry_backoff_ms_total"] = (
            int_or_zero(persona_metric.get("retry_backoff_ms_total")) + retry_backoff_ms_total
        )
        persona_metric["think_time_events"] = int_or_zero(persona_metric.get("think_time_events")) + think_time_events
        persona_metric["think_time_ms_total"] = (
            int_or_zero(persona_metric.get("think_time_ms_total")) + think_time_ms_total
        )
        persona_metric["expected_think_time_min_total"] = (
            int_or_zero(persona_metric.get("expected_think_time_min_total"))
            + (think_time_events * think_time_min)
        )
        persona_metric["expected_think_time_max_total"] = (
            int_or_zero(persona_metric.get("expected_think_time_max_total"))
            + (think_time_events * think_time_max)
        )
        persona_metric["state_headers_sent"] = (
            int_or_zero(persona_metric.get("state_headers_sent")) + state_headers_sent
        )
        persona_metric["state_token_observed"] = (
            int_or_zero(persona_metric.get("state_token_observed")) + state_token_observed
        )
        persona_metric["state_store_resets"] = (
            int_or_zero(persona_metric.get("state_store_resets")) + state_store_resets
        )
        persona_metric["state_store_peak_size_max"] = max(
            int_or_zero(persona_metric.get("state_store_peak_size_max")),
            state_store_peak_size,
        )

        retry_metric = retry_strategy_metrics.setdefault(
            retry_strategy,
            {
                "scenario_count": 0,
                "request_sequence_total": 0,
                "attempts_total": 0,
                "retry_attempts": 0,
                "retry_backoff_ms_total": 0,
                "max_attempts_configured_max": 0,
            },
        )
        retry_metric["scenario_count"] = int_or_zero(retry_metric.get("scenario_count")) + 1
        retry_metric["request_sequence_total"] = (
            int_or_zero(retry_metric.get("request_sequence_total")) + request_sequence_count
        )
        retry_metric["attempts_total"] = int_or_zero(retry_metric.get("attempts_total")) + attempts_total
        retry_metric["retry_attempts"] = int_or_zero(retry_metric.get("retry_attempts")) + retry_attempts
        retry_metric["retry_backoff_ms_total"] = (
            int_or_zero(retry_metric.get("retry_backoff_ms_total")) + retry_backoff_ms_total
        )
        retry_metric["max_attempts_configured_max"] = max(
            int_or_zero(retry_metric.get("max_attempts_configured_max")),
            max_attempts_configured,
        )

        state_bucket = state_mode_bucket(state_mode)
        state_metric = state_mode_metrics.setdefault(
            state_bucket,
            {
                "state_mode": state_mode,
                "scenario_count": 0,
                "request_sequence_total": 0,
                "state_headers_sent": 0,
                "state_token_observed": 0,
                "state_store_resets": 0,
                "state_store_peak_size_max": 0,
            },
        )
        state_metric["scenario_count"] = int_or_zero(state_metric.get("scenario_count")) + 1
        state_metric["request_sequence_total"] = (
            int_or_zero(state_metric.get("request_sequence_total")) + request_sequence_count
        )
        state_metric["state_headers_sent"] = (
            int_or_zero(state_metric.get("state_headers_sent")) + state_headers_sent
        )
        state_metric["state_token_observed"] = (
            int_or_zero(state_metric.get("state_token_observed")) + state_token_observed
        )
        state_metric["state_store_resets"] = (
            int_or_zero(state_metric.get("state_store_resets")) + state_store_resets
        )
        state_metric["state_store_peak_size_max"] = max(
            int_or_zero(state_metric.get("state_store_peak_size_max")),
            state_store_peak_size,
        )

    for persona_metric in persona_metrics.values():
        events = int_or_zero(persona_metric.get("think_time_events"))
        total = int_or_zero(persona_metric.get("think_time_ms_total"))
        persona_metric["think_time_ms_avg"] = int(total / events) if events else 0

    return {
        "persona_scheduler": persona_scheduler,
        "persona_sequence": persona_sequence,
        "missing_result_ids": missing_result_ids,
        "persona_metrics": persona_metrics,
        "retry_strategy_metrics": retry_strategy_metrics,
        "state_mode_metrics": state_mode_metrics,
        "totals": {
            "scenario_results": len(results),
            "missing_result_count": len(missing_result_ids),
            "think_time_events_total": sum(
                int_or_zero(metric.get("think_time_events")) for metric in persona_metrics.values()
            ),
            "retry_attempts_total": sum(
                int_or_zero(metric.get("retry_attempts")) for metric in retry_strategy_metrics.values()
            ),
        },
    }


def build_realism_checks(
    profile_name: str,
    profile_gates: Dict[str, Any],
    realism_metrics: Dict[str, Any],
) -> List[Dict[str, Any]]:
    del profile_name
    checks: List[Dict[str, Any]] = []
    realism_gate = dict_or_empty(profile_gates.get("realism"))
    realism_enabled = bool(realism_gate.get("enabled", True))
    if not realism_enabled:
        return checks

    missing_result_ids = list_or_empty(realism_metrics.get("missing_result_ids"))
    checks.append(
        {
            "name": "realism_evidence_attached",
            "passed": len(missing_result_ids) == 0,
            "detail": (
                f"missing_result_ids={missing_result_ids}"
                if missing_result_ids
                else "all scenario results include realism evidence"
            ),
            "observed": len(missing_result_ids),
            "threshold_source": "profile.gates.realism.enabled",
        }
    )

    persona_scheduler = str(realism_metrics.get("persona_scheduler") or "").strip().lower()
    persona_sequence = [
        str(persona).strip()
        for persona in list_or_empty(realism_metrics.get("persona_sequence"))
        if str(persona).strip()
    ]
    if persona_scheduler == "round_robin":
        violations = round_robin_sequence_violations(persona_sequence)
        checks.append(
            {
                "name": "realism_persona_scheduler_round_robin",
                "passed": len(violations) == 0,
                "detail": (
                    f"sequence={persona_sequence}"
                    if not violations
                    else f"violations={violations} sequence={persona_sequence}"
                ),
                "observed": len(violations),
                "threshold_source": "profile.gates.persona_scheduler",
            }
        )

    totals = dict_or_empty(realism_metrics.get("totals"))
    think_time_events_total = int_or_zero(totals.get("think_time_events_total"))
    checks.append(
        {
            "name": "realism_think_time_events_total",
            "passed": think_time_events_total > 0,
            "detail": f"think_time_events_total={think_time_events_total}",
            "observed": think_time_events_total,
            "threshold_source": "profile.gates.realism.enabled",
        }
    )

    persona_metrics = dict_or_empty(realism_metrics.get("persona_metrics"))
    for persona in sorted(persona_metrics.keys()):
        metric = dict_or_empty(persona_metrics.get(persona))
        events = int_or_zero(metric.get("think_time_events"))
        if events <= 0:
            continue
        observed_total = int_or_zero(metric.get("think_time_ms_total"))
        minimum_total = int_or_zero(metric.get("expected_think_time_min_total"))
        maximum_total = int_or_zero(metric.get("expected_think_time_max_total"))
        checks.append(
            {
                "name": f"realism_persona_think_time_envelope_{persona}",
                "passed": minimum_total <= observed_total <= maximum_total,
                "detail": (
                    f"observed_total={observed_total}ms "
                    f"expected=[{minimum_total},{maximum_total}]ms events={events}"
                ),
                "observed": observed_total,
                "min": minimum_total,
                "max": maximum_total,
                "threshold_source": "scenario.traffic_model.think_time_ms_*",
            }
        )

    retry_strategy_metrics = dict_or_empty(realism_metrics.get("retry_strategy_metrics"))
    for strategy in sorted(retry_strategy_metrics.keys()):
        metric = dict_or_empty(retry_strategy_metrics.get(strategy))
        request_sequence_total = int_or_zero(metric.get("request_sequence_total"))
        attempts_total = int_or_zero(metric.get("attempts_total"))
        retry_attempts = int_or_zero(metric.get("retry_attempts"))
        max_attempts = retry_strategy_max_attempts(strategy)
        minimum_attempts = request_sequence_total
        maximum_attempts = request_sequence_total * max_attempts
        checks.append(
            {
                "name": f"realism_retry_envelope_{strategy}",
                "passed": minimum_attempts <= attempts_total <= maximum_attempts,
                "detail": (
                    f"attempts_total={attempts_total} expected=[{minimum_attempts},{maximum_attempts}] "
                    f"request_sequence_total={request_sequence_total}"
                ),
                "observed": attempts_total,
                "min": minimum_attempts,
                "max": maximum_attempts,
                "threshold_source": "scenario.traffic_model.retry_strategy",
            }
        )
        if strategy == "single_attempt":
            checks.append(
                {
                    "name": "realism_retry_single_attempt_no_retries",
                    "passed": retry_attempts == 0,
                    "detail": f"retry_attempts={retry_attempts}",
                    "observed": retry_attempts,
                    "threshold_source": "scenario.traffic_model.retry_strategy",
                }
            )

    required_retry_attempts = dict_or_empty(realism_gate.get("required_retry_attempts"))
    for strategy in sorted(required_retry_attempts.keys()):
        minimum = int_or_zero(required_retry_attempts.get(strategy))
        observed = int_or_zero(dict_or_empty(retry_strategy_metrics.get(strategy)).get("retry_attempts"))
        checks.append(
            {
                "name": f"realism_required_retry_attempts_{strategy}",
                "passed": observed >= minimum,
                "detail": f"retry_attempts={observed} minimum={minimum}",
                "observed": observed,
                "minimum": minimum,
                "threshold_source": f"profile.gates.realism.required_retry_attempts.{strategy}",
            }
        )

    state_mode_metrics = dict_or_empty(realism_metrics.get("state_mode_metrics"))
    for behavior_bucket in sorted(state_mode_metrics.keys()):
        metric = dict_or_empty(state_mode_metrics.get(behavior_bucket))
        behavior = str(metric.get("state_mode") or behavior_bucket)
        request_sequence_total = int_or_zero(metric.get("request_sequence_total"))
        state_headers_sent = int_or_zero(metric.get("state_headers_sent"))
        state_token_observed = int_or_zero(metric.get("state_token_observed"))
        state_store_resets = int_or_zero(metric.get("state_store_resets"))
        state_store_peak_size_max = int_or_zero(metric.get("state_store_peak_size_max"))

        if behavior == "stateless":
            passed = (
                state_headers_sent == 0
                and state_store_resets == 0
                and state_store_peak_size_max == 0
            )
            checks.append(
                {
                    "name": f"realism_state_mode_{behavior_bucket}_envelope",
                    "passed": passed,
                    "detail": (
                        f"state_headers_sent={state_headers_sent} "
                        f"state_store_resets={state_store_resets} "
                        f"state_store_peak_size_max={state_store_peak_size_max}"
                    ),
                    "observed": {
                        "state_headers_sent": state_headers_sent,
                        "state_store_resets": state_store_resets,
                        "state_store_peak_size_max": state_store_peak_size_max,
                    },
                    "threshold_source": "scenario.traffic_model.cookie_behavior",
                }
            )
            continue

        if behavior == "cookie_reset_each_request":
            checks.append(
                {
                    "name": f"realism_state_mode_{behavior_bucket}_envelope",
                    "passed": state_store_resets >= request_sequence_total,
                    "detail": (
                        f"state_store_resets={state_store_resets} "
                        f"request_sequence_total={request_sequence_total}"
                    ),
                    "observed": state_store_resets,
                    "minimum": request_sequence_total,
                    "threshold_source": "scenario.traffic_model.cookie_behavior",
                }
            )
            continue

        if behavior == "stateful_cookie_jar":
            checks.append(
                {
                    "name": f"realism_state_mode_{behavior_bucket}_envelope",
                    "passed": state_store_resets == 0 and state_headers_sent <= request_sequence_total,
                    "detail": (
                        f"state_headers_sent={state_headers_sent} "
                        f"request_sequence_total={request_sequence_total} "
                        f"state_store_resets={state_store_resets} "
                        f"state_token_observed={state_token_observed}"
                    ),
                    "observed": {
                        "state_headers_sent": state_headers_sent,
                        "request_sequence_total": request_sequence_total,
                        "state_store_resets": state_store_resets,
                        "state_token_observed": state_token_observed,
                    },
                    "threshold_source": "scenario.traffic_model.cookie_behavior",
                }
            )

    return checks


def normalize_execution_lane(raw_value: Any) -> str:
    lane = str(raw_value or "").strip().lower()
    if not lane:
        return "black_box"
    return lane


def validate_execution_lane(
    lane: str,
    *,
    supported_execution_lanes: Iterable[str],
) -> str:
    normalized = normalize_execution_lane(lane)
    supported = set(supported_execution_lanes)
    if normalized not in supported:
        raise SimulationError(
            f"execution_lane must be one of {sorted(supported)} (got {normalized})"
        )
    return normalized


def effective_scenario_latency_ms(
    scenario: Dict[str, Any],
    wall_clock_latency_ms: int,
    realism: Optional[Dict[str, Any]],
    *,
    scenario_driver_class_fn: Callable[[Dict[str, Any]], str],
) -> int:
    latency_ms = max(0, int_or_zero(wall_clock_latency_ms))
    realism_data = dict_or_empty(realism)
    browser_action_latency_ms = max(0, int_or_zero(realism_data.get("browser_action_duration_ms")))
    if scenario_driver_class_fn(scenario) == "browser_realistic" and browser_action_latency_ms > 0:
        return browser_action_latency_ms
    if scenario_driver_class_fn(scenario) == "edge_fixture":
        explicit_latency_ms = (
            max(0, int_or_zero(realism_data.get("request_latency_ms_total")))
            + max(0, int_or_zero(realism_data.get("think_time_ms_total")))
            + max(0, int_or_zero(realism_data.get("retry_backoff_ms_total")))
        )
        if explicit_latency_ms > 0:
            return explicit_latency_ms
    return latency_ms


def scenario_max_latency_ms(scenario: Dict[str, Any]) -> int:
    cost_assertions = scenario.get("cost_assertions")
    if isinstance(cost_assertions, dict) and "max_latency_ms" in cost_assertions:
        return int(cost_assertions["max_latency_ms"])
    assertions = scenario.get("assertions")
    if isinstance(assertions, dict) and "max_latency_ms" in assertions:
        return int(assertions["max_latency_ms"])
    raise SimulationError(
        f"scenario {scenario.get('id')} must define cost_assertions.max_latency_ms (v2) or assertions.max_latency_ms (v1)"
    )
