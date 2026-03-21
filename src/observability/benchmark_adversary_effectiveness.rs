use super::benchmark_results::{BenchmarkFamilyResult, BenchmarkMetricResult};
use super::benchmark_results_families::aggregate_budget_status;
use super::operator_snapshot::OperatorSnapshotAdversarySim;

pub(super) fn representative_adversary_effectiveness_family(
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> BenchmarkFamilyResult {
    let total_runs = adversary_sim.recent_runs.len() as u64;
    let successful_runs = adversary_sim
        .recent_runs
        .iter()
        .filter(|run| run.defense_delta_count == 0 && run.ban_outcome_count == 0)
        .count() as u64;
    let origin_reach_proxy_runs = adversary_sim
        .recent_runs
        .iter()
        .filter(|run| {
            run.monitoring_event_count
                > run
                    .defense_delta_count
                    .saturating_add(run.ban_outcome_count)
        })
        .count() as u64;
    let escalated_runs = adversary_sim
        .recent_runs
        .iter()
        .filter(|run| run.defense_delta_count > 0 || run.ban_outcome_count > 0)
        .count() as u64;
    let goal_success_rate = ratio(successful_runs, total_runs);
    let metrics = vec![
        zero_budget_metric(
            "scenario_goal_success_rate",
            total_runs,
            goal_success_rate,
            "supported",
        ),
        tracking_metric(
            "scenario_origin_reach_rate",
            total_runs,
            ratio(origin_reach_proxy_runs, total_runs),
        ),
        tracking_metric(
            "scenario_escalation_rate",
            total_runs,
            ratio(escalated_runs, total_runs),
        ),
        zero_budget_metric(
            "scenario_regression_status",
            total_runs,
            if successful_runs > 0 { 1.0 } else { 0.0 },
            "supported",
        ),
    ];
    BenchmarkFamilyResult {
        family_id: "representative_adversary_effectiveness".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: "partially_supported".to_string(),
        note: format!(
            "Recent adversary-sim profile runs are mapped into bounded effectiveness proxies over {} recent runs; detailed scenario-family lineage will deepen with replay-promotion materialization.",
            total_runs
        ),
        baseline_status: None,
        comparison_status: "not_available".to_string(),
        metrics,
    }
}

fn zero_budget_metric(
    metric_id: &str,
    sample_size: u64,
    current: f64,
    capability_gate: &str,
) -> BenchmarkMetricResult {
    let status = if sample_size == 0 {
        "insufficient_evidence"
    } else if current <= 0.0 {
        "inside_budget"
    } else {
        "outside_budget"
    };
    BenchmarkMetricResult {
        metric_id: metric_id.to_string(),
        status: status.to_string(),
        current: if sample_size == 0 { None } else { Some(current) },
        target: if sample_size == 0 { None } else { Some(0.0) },
        delta: if sample_size == 0 {
            None
        } else {
            Some(current)
        },
        exactness: "derived".to_string(),
        basis: "observed".to_string(),
        capability_gate: capability_gate.to_string(),
        baseline_current: None,
        comparison_delta: None,
        comparison_status: "not_available".to_string(),
    }
}

fn tracking_metric(metric_id: &str, sample_size: u64, current: f64) -> BenchmarkMetricResult {
    BenchmarkMetricResult {
        metric_id: metric_id.to_string(),
        status: if sample_size == 0 {
            "insufficient_evidence".to_string()
        } else {
            "tracking_only".to_string()
        },
        current: if sample_size == 0 { None } else { Some(current) },
        target: None,
        delta: None,
        exactness: "derived".to_string(),
        basis: "observed".to_string(),
        capability_gate: "supported".to_string(),
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
