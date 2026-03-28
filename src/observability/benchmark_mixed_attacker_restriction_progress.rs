use std::collections::{BTreeMap, BTreeSet};

use crate::admin::adversary_sim_worker_plan::LlmRuntimeActionReceipt;
use crate::observability::operator_snapshot::{
    OperatorSnapshotAdversarySim, OperatorSnapshotRecentSimRun,
};

use super::benchmark_results::{BenchmarkExploitLocus, BenchmarkFamilyResult};
use super::benchmark_results_families::aggregate_budget_status;
use super::benchmark_scrapling_exploit_progress::{
    host_cost_channels_for_surface, latest_scrapling_recent_run, repair_families_for_surface,
    stage_id, stage_rank, zero_budget_metric,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MixedAttackerRestrictionProgressState {
    pub latest_lane_ids: Vec<String>,
    pub capability_gate: String,
    pub note: String,
    pub exploit_loci: Vec<BenchmarkExploitLocus>,
    pub relevant_locus_count: usize,
    pub relevant_lane_count: usize,
    pub breached_lane_count: usize,
    pub deepest_relevant_stage_rank: u8,
}

#[derive(Debug, Clone)]
struct LocusAggregate {
    locus_id: String,
    locus_label: String,
    stage_id: String,
    attempt_count: u64,
    sample_request_method: String,
    sample_request_path: String,
    sample_response_status: Option<u16>,
}

pub(crate) fn mixed_attacker_restriction_progress_family(
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> BenchmarkFamilyResult {
    let state = latest_mixed_attacker_restriction_state(adversary_sim);
    let breach_locus_rate = if state.relevant_locus_count == 0 {
        None
    } else {
        Some(state.exploit_loci.len() as f64 / state.relevant_locus_count as f64)
    };
    let contributing_lane_rate = if state.relevant_lane_count == 0 {
        None
    } else {
        Some(state.breached_lane_count as f64 / state.relevant_lane_count as f64)
    };
    let deepest_breach_stage_ratio = if state.relevant_locus_count == 0 {
        None
    } else {
        let deepest_breach_stage = state
            .exploit_loci
            .iter()
            .map(|locus| stage_rank(locus.locus_id.as_str()))
            .max()
            .unwrap_or(0);
        Some(deepest_breach_stage as f64 / state.deepest_relevant_stage_rank.max(1) as f64)
    };

    let metrics = vec![
        zero_budget_metric(
            "mixed_attacker_breach_locus_rate",
            state.relevant_locus_count,
            breach_locus_rate,
            state.capability_gate.as_str(),
        ),
        zero_budget_metric(
            "mixed_attacker_contributing_lane_rate",
            state.relevant_lane_count,
            contributing_lane_rate,
            state.capability_gate.as_str(),
        ),
        zero_budget_metric(
            "mixed_attacker_deepest_breach_stage_ratio",
            state.relevant_locus_count,
            deepest_breach_stage_ratio,
            state.capability_gate.as_str(),
        ),
    ];

    BenchmarkFamilyResult {
        family_id: "mixed_attacker_restriction_progress".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: state.capability_gate,
        note: state.note,
        baseline_status: None,
        comparison_status: "not_available".to_string(),
        exploit_loci: state.exploit_loci,
        metrics,
    }
}

pub(crate) fn latest_mixed_attacker_restriction_state(
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> MixedAttackerRestrictionProgressState {
    let latest_scrapling = latest_scrapling_recent_run(adversary_sim);
    let latest_llm = latest_llm_recent_run(adversary_sim);

    if latest_scrapling.is_none() && latest_llm.is_none() {
        return MixedAttackerRestrictionProgressState {
            latest_lane_ids: Vec::new(),
            capability_gate: "partially_supported".to_string(),
            note: "No recent mixed-attacker lane evidence is visible yet, so restriction scoring still has nothing truthful to judge.".to_string(),
            exploit_loci: Vec::new(),
            relevant_locus_count: 0,
            relevant_lane_count: 0,
            breached_lane_count: 0,
            deepest_relevant_stage_rank: 1,
        };
    }

    let mut relevant_locus_ids = BTreeSet::new();
    let mut relevant_lane_ids = BTreeSet::new();
    let mut breached_lane_ids = BTreeSet::new();
    let mut aggregates = BTreeMap::<String, LocusAggregate>::new();

    if let Some(run) = latest_scrapling {
        relevant_lane_ids.insert(run.lane.clone());
        if let Some(coverage) = run.owned_surface_coverage.as_ref() {
            for surface_id in &coverage.required_surface_ids {
                relevant_locus_ids.insert(surface_id.clone());
            }
            for receipt in coverage
                .receipts
                .iter()
                .filter(|receipt| receipt.coverage_status == "pass_observed")
            {
                breached_lane_ids.insert(run.lane.clone());
                merge_locus(
                    &mut aggregates,
                    receipt.surface_id.as_str(),
                    receipt.sample_request_method.as_str(),
                    receipt.sample_request_path.as_str(),
                    receipt.sample_response_status,
                    receipt.attempt_count,
                    coverage
                        .surface_labels
                        .get(receipt.surface_id.as_str())
                        .cloned()
                        .unwrap_or_else(|| receipt.surface_id.clone()),
                );
            }
        }
    }

    if let Some(run) = latest_llm {
        if let Some(summary) = run.llm_runtime_summary.as_ref() {
            let relevant_receipts: Vec<_> = summary
                .latest_action_receipts
                .iter()
                .filter_map(|receipt| llm_receipt_surface_id(receipt).map(|surface_id| (receipt, surface_id)))
                .collect();
            if !relevant_receipts.is_empty() {
                relevant_lane_ids.insert(run.lane.clone());
            }
            for (receipt, surface_id) in relevant_receipts {
                relevant_locus_ids.insert(surface_id.to_string());
                if llm_receipt_indicates_exploit_progress(receipt) {
                    breached_lane_ids.insert(run.lane.clone());
                    merge_locus(
                        &mut aggregates,
                        surface_id,
                        llm_request_method(receipt).as_str(),
                        receipt.path.as_str(),
                        receipt.status,
                        1,
                        llm_surface_label(surface_id).to_string(),
                    );
                }
            }
        }
    }

    let mut exploit_loci: Vec<BenchmarkExploitLocus> = aggregates
        .into_values()
        .map(benchmark_locus_from_aggregate)
        .collect();
    exploit_loci.sort_by(|left, right| {
        right
            .attempt_count
            .unwrap_or(0)
            .cmp(&left.attempt_count.unwrap_or(0))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.locus_label.cmp(&right.locus_label))
    });

    let latest_lane_ids: Vec<String> = relevant_lane_ids.iter().cloned().collect();
    let capability_gate = if relevant_lane_ids.contains("scrapling_traffic")
        && relevant_lane_ids.contains("bot_red_team")
    {
        "supported".to_string()
    } else {
        "partially_supported".to_string()
    };
    let note = if exploit_loci.is_empty() {
        format!(
            "Recent mixed-attacker lanes ({}) made no positive exploit progress across {} relevant board loci.",
            latest_lane_ids.join(", "),
            relevant_locus_ids.len()
        )
    } else {
        format!(
            "Recent mixed-attacker lanes ({}) made positive exploit progress at: {}.",
            latest_lane_ids.join(", "),
            exploit_loci
                .iter()
                .map(|locus| locus.locus_label.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let deepest_relevant_stage_rank = relevant_locus_ids
        .iter()
        .map(|surface_id| stage_rank(surface_id.as_str()))
        .max()
        .unwrap_or(1);

    MixedAttackerRestrictionProgressState {
        latest_lane_ids,
        capability_gate,
        note,
        exploit_loci,
        relevant_locus_count: relevant_locus_ids.len(),
        relevant_lane_count: relevant_lane_ids.len(),
        breached_lane_count: breached_lane_ids.len(),
        deepest_relevant_stage_rank,
    }
}

pub(crate) fn latest_llm_recent_run(
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> Option<&OperatorSnapshotRecentSimRun> {
    adversary_sim
        .recent_runs
        .iter()
        .filter(|run| run.lane == "bot_red_team")
        .max_by_key(|run| run.last_ts)
}

pub(crate) fn llm_receipt_surface_id(receipt: &LlmRuntimeActionReceipt) -> Option<&'static str> {
    let path = receipt.path.as_str();
    if path == "/"
        || path.starts_with("/sim/public/")
        || path.starts_with("/detail/")
        || path.starts_with("/search")
    {
        Some("public_path_traversal")
    } else if path.starts_with("/challenge") {
        Some("challenge_routing")
    } else if path.starts_with("/maze") {
        Some("maze_navigation")
    } else if path.starts_with("/pow") {
        Some("pow_verify_abuse")
    } else if path.starts_with("/tarpit") {
        Some("tarpit_progress_abuse")
    } else {
        None
    }
}

pub(crate) fn llm_receipt_indicates_exploit_progress(
    receipt: &LlmRuntimeActionReceipt,
) -> bool {
    receipt.error.is_none()
        && receipt
            .status
            .map(|status| (200..400).contains(&status))
            .unwrap_or(false)
}

fn llm_request_method(receipt: &LlmRuntimeActionReceipt) -> String {
    match receipt.action_type.as_str() {
        "http_get" | "browser_navigate" => "GET".to_string(),
        other => other.to_uppercase(),
    }
}

fn llm_surface_label(surface_id: &str) -> &'static str {
    match surface_id {
        "public_path_traversal" => "Public Path Traversal",
        "challenge_routing" => "Challenge Routing",
        "maze_navigation" => "Maze Navigation",
        "pow_verify_abuse" => "PoW Verify Abuse",
        "tarpit_progress_abuse" => "Tarpit Progress Abuse",
        _ => "Unknown Board Locus",
    }
}

fn merge_locus(
    aggregates: &mut BTreeMap<String, LocusAggregate>,
    locus_id: &str,
    sample_request_method: &str,
    sample_request_path: &str,
    sample_response_status: Option<u16>,
    attempt_count: u64,
    locus_label: String,
) {
    let entry = aggregates
        .entry(locus_id.to_string())
        .or_insert_with(|| LocusAggregate {
            locus_id: locus_id.to_string(),
            locus_label: locus_label.clone(),
            stage_id: stage_id(locus_id).to_string(),
            attempt_count: 0,
            sample_request_method: sample_request_method.to_string(),
            sample_request_path: sample_request_path.to_string(),
            sample_response_status,
        });
    entry.attempt_count = entry.attempt_count.saturating_add(attempt_count);
    if entry.sample_request_path.is_empty() {
        entry.sample_request_method = sample_request_method.to_string();
        entry.sample_request_path = sample_request_path.to_string();
        entry.sample_response_status = sample_response_status;
    }
    if entry.locus_label.is_empty() {
        entry.locus_label = locus_label;
    }
}

fn benchmark_locus_from_aggregate(aggregate: LocusAggregate) -> BenchmarkExploitLocus {
    let cost_channel_ids: Vec<String> = host_cost_channels_for_surface(aggregate.locus_id.as_str())
        .iter()
        .map(|channel| (*channel).to_string())
        .collect();
    let repair_family_candidates: Vec<String> =
        repair_families_for_surface(aggregate.locus_id.as_str())
            .iter()
            .map(|family| (*family).to_string())
            .collect();

    BenchmarkExploitLocus {
        locus_id: aggregate.locus_id,
        locus_label: aggregate.locus_label,
        stage_id: aggregate.stage_id,
        evidence_status: "progress_observed".to_string(),
        attempt_count: Some(aggregate.attempt_count),
        attempt_count_status: "measured".to_string(),
        cost_channel_ids: cost_channel_ids.clone(),
        cost_channel_status: if cost_channel_ids.is_empty() {
            "not_materialized".to_string()
        } else {
            "derived".to_string()
        },
        sample_request_method: aggregate.sample_request_method,
        sample_request_path: aggregate.sample_request_path,
        sample_response_status: aggregate.sample_response_status,
        repair_family_candidates: repair_family_candidates.clone(),
        repair_family_status: if repair_family_candidates.is_empty() {
            "not_materialized".to_string()
        } else {
            "derived".to_string()
        },
    }
}

pub(crate) fn current_mixed_breach_locus_ids(
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> BTreeSet<String> {
    latest_mixed_attacker_restriction_state(adversary_sim)
        .exploit_loci
        .into_iter()
        .map(|locus| locus.locus_id)
        .collect()
}

pub(crate) fn run_contains_breach_locus(
    run: &OperatorSnapshotRecentSimRun,
    locus_id: &str,
) -> bool {
    if run.lane == "scrapling_traffic" {
        return run
            .owned_surface_coverage
            .as_ref()
            .map(|coverage| {
                coverage
                    .receipts
                    .iter()
                    .filter(|receipt| receipt.coverage_status == "pass_observed")
                    .any(|receipt| receipt.surface_id == locus_id)
            })
            .unwrap_or(false);
    }

    run.llm_runtime_summary
        .as_ref()
        .map(|summary| {
            summary
                .latest_action_receipts
                .iter()
                .filter(|receipt| llm_receipt_indicates_exploit_progress(receipt))
                .filter_map(llm_receipt_surface_id)
                .any(|surface_id| surface_id == locus_id)
        })
        .unwrap_or(false)
}
