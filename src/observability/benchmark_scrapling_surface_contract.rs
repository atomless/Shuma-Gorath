use crate::observability::operator_snapshot::OperatorSnapshotAdversarySim;
use crate::observability::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;
use crate::observability::scrapling_owned_surface::coverage_receipt_state_label;

use super::benchmark_results::{BenchmarkFamilyResult, BenchmarkMetricResult};
use super::benchmark_results_families::aggregate_budget_status;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScraplingSurfaceContractState {
    pub latest_run_id: Option<String>,
    pub capability_gate: String,
    pub overall_status: String,
    pub required_surface_ids: Vec<String>,
    pub satisfied_surface_ids: Vec<String>,
    pub blocking_surface_ids: Vec<String>,
    pub note: String,
}

pub(crate) fn scrapling_surface_contract_family(
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> BenchmarkFamilyResult {
    let state = latest_scrapling_surface_contract_state(adversary_sim);
    let required_count = state.required_surface_ids.len();
    let satisfied_count = state.satisfied_surface_ids.len();
    let blocking_count = state.blocking_surface_ids.len();
    let satisfaction_rate = if required_count == 0 {
        None
    } else {
        Some(satisfied_count as f64 / required_count as f64)
    };
    let basis = match state.overall_status.as_str() {
        "inside_budget" | "outside_budget" => "observed_recent_run_surface_receipts",
        "insufficient_evidence" => "missing_recent_run_surface_receipts",
        _ => "mixed",
    };
    let metrics = vec![
        BenchmarkMetricResult {
            metric_id: "scrapling_required_surface_satisfaction_rate".to_string(),
            status: match satisfaction_rate {
                Some(rate) if rate >= 1.0 => "inside_budget".to_string(),
                Some(_) => "outside_budget".to_string(),
                None => "insufficient_evidence".to_string(),
            },
            current: satisfaction_rate,
            target: if required_count == 0 { None } else { Some(1.0) },
            delta: satisfaction_rate.map(|rate| rate - 1.0),
            exactness: "derived".to_string(),
            basis: basis.to_string(),
            capability_gate: state.capability_gate.clone(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
        BenchmarkMetricResult {
            metric_id: "scrapling_blocking_required_surface_count".to_string(),
            status: if required_count == 0 {
                "insufficient_evidence".to_string()
            } else if blocking_count == 0 {
                "inside_budget".to_string()
            } else {
                "outside_budget".to_string()
            },
            current: if required_count == 0 {
                None
            } else {
                Some(blocking_count as f64)
            },
            target: if required_count == 0 { None } else { Some(0.0) },
            delta: if required_count == 0 {
                None
            } else {
                Some(blocking_count as f64)
            },
            exactness: "exact".to_string(),
            basis: basis.to_string(),
            capability_gate: state.capability_gate.clone(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        },
    ];

    BenchmarkFamilyResult {
        family_id: "scrapling_surface_contract".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: state.capability_gate,
        note: state.note,
        baseline_status: None,
        comparison_status: "not_available".to_string(),
        exploit_loci: Vec::new(),
        metrics,
    }
}

pub(crate) fn scrapling_surface_contract_tuning_blockers(
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> Vec<String> {
    let state = latest_scrapling_surface_contract_state(adversary_sim);
    match state.overall_status.as_str() {
        "inside_budget" => Vec::new(),
        "outside_budget" => {
            let mut blockers = vec!["scrapling_surface_contract_not_ready".to_string()];
            blockers.extend(
                state
                    .blocking_surface_ids
                    .into_iter()
                    .map(|surface_id| format!("scrapling_surface_blocking:{surface_id}")),
            );
            blockers
        }
        "insufficient_evidence" if state.latest_run_id.is_some() => {
            vec!["scrapling_surface_contract_not_ready".to_string()]
        }
        _ => Vec::new(),
    }
}

pub(crate) fn latest_scrapling_surface_contract_state(
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> ScraplingSurfaceContractState {
    let Some(run) = latest_scrapling_recent_run(adversary_sim) else {
        return ScraplingSurfaceContractState {
            latest_run_id: None,
            capability_gate: "partially_supported".to_string(),
            overall_status: "insufficient_evidence".to_string(),
            required_surface_ids: Vec::new(),
            satisfied_surface_ids: Vec::new(),
            blocking_surface_ids: Vec::new(),
            note: "No recent Scrapling run is currently visible, so the controller cannot yet score required defense-surface satisfaction.".to_string(),
        };
    };
    let Some(coverage) = run.owned_surface_coverage.as_ref() else {
        return ScraplingSurfaceContractState {
            latest_run_id: Some(run.run_id.clone()),
            capability_gate: "partially_supported".to_string(),
            overall_status: "insufficient_evidence".to_string(),
            required_surface_ids: Vec::new(),
            satisfied_surface_ids: Vec::new(),
            blocking_surface_ids: Vec::new(),
            note: format!(
                "Latest Scrapling run {} has no owned-surface coverage summary, so aggregate leakage pressure cannot stand in for defense-surface truth.",
                run.run_id
            ),
        };
    };

    let blocking_surface_ids = coverage.blocking_surface_ids.clone();
    let overall_status = if !coverage.required_surface_ids.is_empty()
        && blocking_surface_ids.is_empty()
        && coverage.overall_status == "covered"
    {
        "inside_budget"
    } else {
        "outside_budget"
    };
    let blocking_surface_labels: Vec<String> = blocking_surface_ids
        .iter()
        .map(|surface_id| {
            coverage
                .receipts
                .iter()
                .find(|receipt| receipt.surface_id == *surface_id)
                .map(|receipt| {
                    let label = coverage
                        .surface_labels
                        .get(surface_id)
                        .cloned()
                        .unwrap_or_else(|| surface_id.clone());
                    format!("{label} ({})", coverage_receipt_state_label(receipt))
                })
                .unwrap_or_else(|| {
                    coverage
                        .surface_labels
                        .get(surface_id)
                        .cloned()
                        .unwrap_or_else(|| surface_id.clone())
                })
        })
        .collect();
    let note = if blocking_surface_ids.is_empty() && coverage.overall_status == "covered" {
        format!(
            "Latest Scrapling run {} satisfied all {} required defense-surface contracts.",
            run.run_id,
            coverage.required_surface_ids.len()
        )
    } else {
        format!(
            "Latest Scrapling run {} still has blocking required defense surfaces: {}.",
            run.run_id,
            blocking_surface_labels.join(", ")
        )
    };

    ScraplingSurfaceContractState {
        latest_run_id: Some(run.run_id.clone()),
        capability_gate: "supported".to_string(),
        overall_status: overall_status.to_string(),
        required_surface_ids: coverage.required_surface_ids.clone(),
        satisfied_surface_ids: coverage.satisfied_surface_ids.clone(),
        blocking_surface_ids,
        note,
    }
}

fn latest_scrapling_recent_run(
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> Option<&OperatorSnapshotRecentSimRun> {
    adversary_sim
        .recent_runs
        .iter()
        .filter(|run| run.lane == "scrapling_traffic")
        .max_by_key(|run| run.last_ts)
}
