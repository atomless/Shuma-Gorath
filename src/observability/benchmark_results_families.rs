use super::benchmark_results::{BenchmarkFamilyResult, BenchmarkMetricResult};
use super::operator_snapshot::{
    OperatorBudgetDistanceRow, OperatorBudgetDistanceSummary, OperatorSnapshotLane,
};

pub(super) fn suspicious_origin_cost_family(
    lane: Option<&OperatorSnapshotLane>,
    budget_distance: &OperatorBudgetDistanceSummary,
    strict_human_only_private: bool,
) -> BenchmarkFamilyResult {
    let request_budget = budget_row(
        budget_distance.rows.as_slice(),
        "suspicious_forwarded_request_rate",
    );
    let byte_budget = budget_row(
        budget_distance.rows.as_slice(),
        "suspicious_forwarded_byte_rate",
    );
    let latency_budget = budget_row(
        budget_distance.rows.as_slice(),
        "suspicious_forwarded_latency_share",
    );
    let metrics = vec![
        budget_metric_result(
            "suspicious_forwarded_request_rate",
            request_budget,
            "supported",
        ),
        budget_metric_result("suspicious_forwarded_byte_rate", byte_budget, "supported"),
        budget_metric_result(
            "suspicious_forwarded_latency_share",
            latency_budget,
            "supported",
        ),
        tracking_ratio_metric(
            "suspicious_short_circuit_rate",
            lane,
            lane.map(|row| ratio(row.short_circuited_requests, row.total_requests)),
        ),
        tracking_ratio_metric(
            "suspicious_locally_served_byte_share",
            lane,
            lane.map(|row| {
                let total_bytes = row
                    .forwarded_response_bytes
                    .saturating_add(row.shuma_served_response_bytes);
                ratio(row.shuma_served_response_bytes, total_bytes)
            }),
        ),
        tracking_average_metric(
            "suspicious_average_forward_latency_ms",
            lane,
            lane.map(|row| {
                if row.forwarded_requests == 0 {
                    None
                } else {
                    Some(
                        row.forwarded_upstream_latency_ms_total as f64
                            / row.forwarded_requests as f64,
                    )
                }
            })
            .flatten(),
        ),
    ];
    BenchmarkFamilyResult {
        family_id: "suspicious_origin_cost".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: "supported".to_string(),
        note: if strict_human_only_private {
            "Derived from the adversary-sim scope and strict zero-leakage budget-distance rows for the human-only reference loop."
                .to_string()
        } else {
            "Derived from the live suspicious-automation lane and current budget-distance rows."
                .to_string()
        },
        baseline_status: None,
        comparison_status: "not_available".to_string(),
        metrics,
    }
}

pub(super) fn likely_human_friction_family(
    budget_distance: &OperatorBudgetDistanceSummary,
) -> BenchmarkFamilyResult {
    let friction_budget = budget_row(
        budget_distance.rows.as_slice(),
        "likely_human_friction_rate",
    );
    let metrics = vec![
        budget_metric_result("likely_human_friction_rate", friction_budget, "supported"),
        unsupported_metric("interactive_friction_rate"),
        unsupported_metric("likely_human_hard_block_rate"),
    ];
    BenchmarkFamilyResult {
        family_id: "likely_human_friction".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: "partially_supported".to_string(),
        note: "Current results are budgeted on observed likely-human friction while interactive and hard-block breakdowns remain to be materialized.".to_string(),
        baseline_status: None,
        comparison_status: "not_available".to_string(),
        metrics,
    }
}

pub(super) fn aggregate_budget_status(metrics: &[BenchmarkMetricResult]) -> String {
    let budget_statuses: Vec<&str> = metrics
        .iter()
        .filter(|metric| {
            matches!(
                metric.status.as_str(),
                "outside_budget"
                    | "near_limit"
                    | "inside_budget"
                    | "insufficient_evidence"
                    | "not_applicable"
            )
        })
        .map(|metric| metric.status.as_str())
        .collect();
    if budget_statuses
        .iter()
        .any(|status| *status == "outside_budget")
    {
        "outside_budget".to_string()
    } else if budget_statuses.iter().any(|status| *status == "near_limit") {
        "near_limit".to_string()
    } else if budget_statuses
        .iter()
        .any(|status| *status == "inside_budget")
    {
        "inside_budget".to_string()
    } else if budget_statuses
        .iter()
        .any(|status| *status == "insufficient_evidence")
    {
        "insufficient_evidence".to_string()
    } else if budget_statuses.iter().all(|status| *status == "not_applicable") {
        "not_applicable".to_string()
    } else {
        "not_yet_supported".to_string()
    }
}

fn budget_row<'a>(
    rows: &'a [OperatorBudgetDistanceRow],
    metric: &str,
) -> Option<&'a OperatorBudgetDistanceRow> {
    rows.iter().find(|row| row.metric == metric)
}

pub(super) fn budget_metric_result(
    metric_id: &str,
    row: Option<&OperatorBudgetDistanceRow>,
    capability_gate: &str,
) -> BenchmarkMetricResult {
    match row {
        Some(value) => BenchmarkMetricResult {
            metric_id: value.metric.clone(),
            status: value.status.clone(),
            current: Some(value.current),
            target: Some(value.target),
            delta: Some(value.delta),
            exactness: value.exactness.clone(),
            basis: value.basis.clone(),
            capability_gate: capability_gate.to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
        None => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "insufficient_evidence".to_string(),
            current: None,
            target: None,
            delta: None,
            exactness: "derived".to_string(),
            basis: "mixed".to_string(),
            capability_gate: capability_gate.to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
    }
}

pub(super) fn tracking_ratio_metric(
    metric_id: &str,
    lane: Option<&OperatorSnapshotLane>,
    current: Option<f64>,
) -> BenchmarkMetricResult {
    match (lane, current) {
        (Some(value), Some(current)) if value.total_requests > 0 => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "tracking_only".to_string(),
            current: Some(current),
            target: None,
            delta: None,
            exactness: value.exactness.clone(),
            basis: value.basis.clone(),
            capability_gate: "supported".to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
        (Some(value), _) => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "insufficient_evidence".to_string(),
            current: None,
            target: None,
            delta: None,
            exactness: value.exactness.clone(),
            basis: value.basis.clone(),
            capability_gate: "supported".to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
        (None, _) => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "insufficient_evidence".to_string(),
            current: None,
            target: None,
            delta: None,
            exactness: "derived".to_string(),
            basis: "observed".to_string(),
            capability_gate: "supported".to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
    }
}

pub(super) fn tracking_average_metric(
    metric_id: &str,
    lane: Option<&OperatorSnapshotLane>,
    current: Option<f64>,
) -> BenchmarkMetricResult {
    match (lane, current) {
        (Some(value), Some(current)) if value.forwarded_requests > 0 => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "tracking_only".to_string(),
            current: Some(current),
            target: None,
            delta: None,
            exactness: value.exactness.clone(),
            basis: value.basis.clone(),
            capability_gate: "supported".to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
        (Some(value), _) => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "insufficient_evidence".to_string(),
            current: None,
            target: None,
            delta: None,
            exactness: value.exactness.clone(),
            basis: value.basis.clone(),
            capability_gate: "supported".to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
        (None, _) => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "insufficient_evidence".to_string(),
            current: None,
            target: None,
            delta: None,
            exactness: "derived".to_string(),
            basis: "observed".to_string(),
            capability_gate: "supported".to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
    }
}

pub(super) fn unsupported_metric(metric_id: &str) -> BenchmarkMetricResult {
    BenchmarkMetricResult {
        metric_id: metric_id.to_string(),
        status: "not_yet_supported".to_string(),
        current: None,
        target: None,
        delta: None,
        exactness: "derived".to_string(),
        basis: "mixed".to_string(),
        capability_gate: "not_yet_supported".to_string(),
        baseline_current: None,
        comparison_delta: None,
        comparison_status: "not_available".to_string(),
    }
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
