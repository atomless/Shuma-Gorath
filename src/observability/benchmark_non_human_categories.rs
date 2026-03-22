use crate::observability::operator_snapshot::OperatorSnapshotNonHumanTrafficSummary;
use crate::observability::operator_snapshot_objectives::{
    OperatorObjectiveCategoryPosture, OperatorObjectivesProfile,
};

use super::benchmark_results::{BenchmarkFamilyResult, BenchmarkMetricResult};
use super::benchmark_results_families::aggregate_budget_status;

const CATEGORY_ALIGNMENT_NEAR_LIMIT_STEP: f64 = 0.25;

pub(super) fn non_human_category_posture_family(
    objectives: &OperatorObjectivesProfile,
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
) -> BenchmarkFamilyResult {
    let metrics: Vec<_> = objectives
        .category_postures
        .iter()
        .map(|row| category_posture_metric(row, non_human_traffic))
        .collect();

    BenchmarkFamilyResult {
        family_id: "non_human_category_posture".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: aggregate_capability_gate(metrics.as_slice()),
        note: "Per-category non-human posture alignment is derived from canonical category receipts plus the persisted operator posture scale; incomplete or degraded category evidence remains explicit and later tuning still depends on protected eligibility."
            .to_string(),
        baseline_status: None,
        comparison_status: "not_available".to_string(),
        metrics,
    }
}

fn category_posture_metric(
    row: &OperatorObjectiveCategoryPosture,
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
) -> BenchmarkMetricResult {
    let category_id = row.category_id.as_str();
    let metric_id = format!("category_posture_alignment:{category_id}");
    let receipts: Vec<_> = non_human_traffic
        .receipts
        .iter()
        .filter(|receipt| receipt.category_id == category_id)
        .collect();
    let coverage_status = non_human_traffic
        .coverage
        .receipts
        .iter()
        .find(|receipt| receipt.category_id == category_id)
        .map(|receipt| receipt.coverage_status.as_str())
        .unwrap_or("uncovered");
    let capability_gate = category_capability_gate(coverage_status, !receipts.is_empty());

    if receipts.is_empty() {
        return BenchmarkMetricResult {
            metric_id,
            status: if capability_gate == "not_yet_supported" {
                "not_yet_supported".to_string()
            } else {
                "insufficient_evidence".to_string()
            },
            current: None,
            target: posture_alignment_target(row.posture.as_str()),
            delta: None,
            exactness: "derived".to_string(),
            basis: "mixed".to_string(),
            capability_gate: capability_gate.to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        };
    }

    if receipts
        .iter()
        .any(|receipt| receipt.assignment_status != "classified" || receipt.degradation_status != "current")
    {
        return BenchmarkMetricResult {
            metric_id,
            status: "insufficient_evidence".to_string(),
            current: None,
            target: posture_alignment_target(row.posture.as_str()),
            delta: None,
            exactness: "derived".to_string(),
            basis: aggregate_basis(receipts.as_slice()),
            capability_gate: capability_gate.to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        };
    }

    let total_requests: u64 = receipts.iter().map(|receipt| receipt.total_requests).sum();
    if total_requests == 0 {
        return BenchmarkMetricResult {
            metric_id,
            status: "insufficient_evidence".to_string(),
            current: None,
            target: posture_alignment_target(row.posture.as_str()),
            delta: None,
            exactness: "derived".to_string(),
            basis: aggregate_basis(receipts.as_slice()),
            capability_gate: capability_gate.to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        };
    }

    let forwarded_requests: u64 = receipts
        .iter()
        .map(|receipt| receipt.forwarded_requests)
        .sum();
    let short_circuited_requests: u64 = receipts
        .iter()
        .map(|receipt| receipt.short_circuited_requests)
        .sum();
    let current = posture_alignment_ratio(
        row.posture.as_str(),
        forwarded_requests,
        short_circuited_requests,
        total_requests,
    );
    let target = posture_alignment_target(row.posture.as_str());

    BenchmarkMetricResult {
        metric_id,
        status: posture_alignment_status(current, target),
        current: Some(current),
        target,
        delta: target.map(|target| current - target),
        exactness: "derived".to_string(),
        basis: aggregate_basis(receipts.as_slice()),
        capability_gate: capability_gate.to_string(),
        baseline_current: None,
        comparison_delta: None,
        comparison_status: "not_available".to_string(),
    }
}

fn category_capability_gate(coverage_status: &str, has_receipts: bool) -> &'static str {
    match coverage_status {
        "covered" => "supported",
        _ if has_receipts => "partially_supported",
        "partial" | "stale" | "unavailable" => "partially_supported",
        _ => "not_yet_supported",
    }
}

fn aggregate_capability_gate(metrics: &[BenchmarkMetricResult]) -> String {
    if metrics
        .iter()
        .all(|metric| metric.capability_gate == "supported")
    {
        "supported".to_string()
    } else if metrics.iter().any(|metric| {
        matches!(
            metric.capability_gate.as_str(),
            "supported" | "partially_supported"
        )
    }) {
        "partially_supported".to_string()
    } else {
        "not_yet_supported".to_string()
    }
}

fn aggregate_basis(
    receipts: &[&crate::observability::non_human_classification::NonHumanClassificationReceipt],
) -> String {
    let Some(first) = receipts.first() else {
        return "mixed".to_string();
    };
    if receipts.iter().all(|receipt| receipt.basis == first.basis) {
        first.basis.clone()
    } else {
        "mixed".to_string()
    }
}

fn posture_alignment_ratio(
    posture: &str,
    forwarded_requests: u64,
    short_circuited_requests: u64,
    total_requests: u64,
) -> f64 {
    if total_requests == 0 {
        return 0.0;
    }
    match posture {
        "allowed" | "tolerated" => forwarded_requests as f64 / total_requests as f64,
        "cost_reduced" | "restricted" | "blocked" => {
            short_circuited_requests as f64 / total_requests as f64
        }
        _ => 0.0,
    }
}

fn posture_alignment_target(posture: &str) -> Option<f64> {
    match posture {
        "allowed" => Some(1.0),
        "tolerated" => Some(0.75),
        "cost_reduced" => Some(0.50),
        "restricted" => Some(0.75),
        "blocked" => Some(1.0),
        _ => None,
    }
}

fn posture_alignment_status(current: f64, target: Option<f64>) -> String {
    let Some(target) = target else {
        return "not_yet_supported".to_string();
    };
    if current >= target {
        "inside_budget".to_string()
    } else if current >= (target - CATEGORY_ALIGNMENT_NEAR_LIMIT_STEP).max(0.0) {
        "near_limit".to_string()
    } else {
        "outside_budget".to_string()
    }
}
