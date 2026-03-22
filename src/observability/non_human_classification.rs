use serde::{Deserialize, Serialize};

use crate::observability::monitoring::{MonitoringSummary, RequestOutcomeLaneSummaryRow};
use crate::observability::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;
use crate::runtime::non_human_taxonomy::{
    canonical_non_human_taxonomy, NonHumanCategoryDescriptor,
};
use crate::runtime::traffic_classification::{
    non_human_category_assignment_for_lane, TrafficLane,
};

pub(crate) const NON_HUMAN_DECISION_CHAIN: [&str; 4] = [
    "fingerprinting_and_evidence",
    "categorization",
    "cumulative_abuse_score_botness",
    "posture_severity",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct NonHumanClassificationReadiness {
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<String>,
    pub live_receipt_count: usize,
    pub adversary_sim_receipt_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NonHumanClassificationReceipt {
    pub traffic_origin: String,
    pub measurement_scope: String,
    pub execution_mode: String,
    pub lane: String,
    pub category_id: String,
    pub category_label: String,
    pub assignment_status: String,
    pub exactness: String,
    pub basis: String,
    pub degradation_status: String,
    pub total_requests: u64,
    pub forwarded_requests: u64,
    pub short_circuited_requests: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_references: Vec<String>,
}

pub(crate) fn non_human_decision_chain() -> Vec<String> {
    NON_HUMAN_DECISION_CHAIN
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(crate) fn summarize_non_human_classification(
    summary: &MonitoringSummary,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
) -> (
    NonHumanClassificationReadiness,
    Vec<NonHumanClassificationReceipt>,
) {
    let taxonomy = canonical_non_human_taxonomy();
    let mut receipts: Vec<NonHumanClassificationReceipt> = summary
        .request_outcomes
        .by_lane
        .iter()
        .filter_map(|row| receipt_from_lane_row(row, recent_sim_runs, &taxonomy.categories))
        .collect();
    receipts.sort_by(|left, right| {
        traffic_origin_sort_key(left.traffic_origin.as_str())
            .cmp(&traffic_origin_sort_key(right.traffic_origin.as_str()))
            .then_with(|| left.lane.cmp(&right.lane))
    });

    let live_receipt_count = receipts
        .iter()
        .filter(|receipt| receipt.traffic_origin == "live")
        .count();
    let adversary_sim_receipt_count = receipts
        .iter()
        .filter(|receipt| receipt.traffic_origin == "adversary_sim")
        .count();

    let mut blockers = Vec::new();
    if live_receipt_count == 0 {
        blockers.push("live_non_human_receipts_missing".to_string());
    }
    if adversary_sim_receipt_count == 0 {
        blockers.push("adversary_sim_non_human_receipts_missing".to_string());
    }
    if receipts
        .iter()
        .any(|receipt| receipt.assignment_status != "classified")
    {
        blockers.push("insufficient_category_evidence".to_string());
    }
    if receipts
        .iter()
        .any(|receipt| receipt.degradation_status != "current")
    {
        blockers.push("degraded_category_receipts_present".to_string());
    }

    let status = if receipts.is_empty() {
        "not_observed".to_string()
    } else if blockers.is_empty() {
        "ready".to_string()
    } else {
        "partial".to_string()
    };

    (
        NonHumanClassificationReadiness {
            status,
            blockers,
            live_receipt_count,
            adversary_sim_receipt_count,
        },
        receipts,
    )
}

fn traffic_origin_sort_key(value: &str) -> u8 {
    match value {
        "live" => 0,
        "adversary_sim" => 1,
        _ => 2,
    }
}

fn receipt_from_lane_row(
    row: &RequestOutcomeLaneSummaryRow,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
    categories: &[NonHumanCategoryDescriptor],
) -> Option<NonHumanClassificationReceipt> {
    let lane = lane_from_summary_value(row.lane.as_str())?;
    let assignment = non_human_category_assignment_for_lane(lane)?;
    let category = categories
        .iter()
        .find(|category| category.category_id == assignment.category_id)?;
    Some(NonHumanClassificationReceipt {
        traffic_origin: row.traffic_origin.clone(),
        measurement_scope: row.measurement_scope.clone(),
        execution_mode: row.execution_mode.clone(),
        lane: row.lane.clone(),
        category_id: assignment.category_id.as_str().to_string(),
        category_label: category.label.clone(),
        assignment_status: assignment.assignment_status.to_string(),
        exactness: row.exactness.clone(),
        basis: row.basis.clone(),
        degradation_status: degradation_status(row),
        total_requests: row.total_requests,
        forwarded_requests: row.forwarded_requests,
        short_circuited_requests: row.short_circuited_requests,
        evidence_references: evidence_references(row, recent_sim_runs),
    })
}

fn lane_from_summary_value(value: &str) -> Option<TrafficLane> {
    match value {
        "likely_human" => Some(TrafficLane::LikelyHuman),
        "unknown_interactive" => Some(TrafficLane::UnknownInteractive),
        "suspicious_automation" => Some(TrafficLane::SuspiciousAutomation),
        "declared_crawler" => Some(TrafficLane::DeclaredCrawler),
        "declared_user_triggered_agent" => Some(TrafficLane::DeclaredUserTriggeredAgent),
        "verified_bot" => Some(TrafficLane::VerifiedBot),
        "signed_agent" => Some(TrafficLane::SignedAgent),
        _ => None,
    }
}

fn degradation_status(row: &RequestOutcomeLaneSummaryRow) -> String {
    if row.exactness == "exact" && matches!(row.basis.as_str(), "observed" | "verified") {
        "current".to_string()
    } else {
        "degraded".to_string()
    }
}

fn evidence_references(
    row: &RequestOutcomeLaneSummaryRow,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
) -> Vec<String> {
    let mut references = vec![format!(
        "request_outcomes.by_lane:{}:{}:{}:{}",
        row.traffic_origin, row.measurement_scope, row.execution_mode, row.lane
    )];
    if row.traffic_origin == "adversary_sim" {
        for run in recent_sim_runs {
            references.push(format!("recent_sim_runs:{}", run.run_id));
        }
    }
    references
}

#[cfg(test)]
mod tests {
    use super::summarize_non_human_classification;
    use crate::observability::monitoring::{MonitoringSummary, RequestOutcomeLaneSummaryRow};
    use crate::observability::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;

    #[test]
    fn classification_summary_requires_current_live_and_sim_receipts_for_ready_status() {
        let mut summary = MonitoringSummary::default();
        summary.request_outcomes.by_lane = vec![
            RequestOutcomeLaneSummaryRow {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                lane: "verified_bot".to_string(),
                exactness: "exact".to_string(),
                basis: "observed".to_string(),
                total_requests: 5,
                forwarded_requests: 5,
                short_circuited_requests: 0,
                control_response_requests: 0,
                response_bytes: 500,
                forwarded_response_bytes: 500,
                short_circuited_response_bytes: 0,
                control_response_bytes: 0,
            },
            RequestOutcomeLaneSummaryRow {
                traffic_origin: "adversary_sim".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                lane: "declared_crawler".to_string(),
                exactness: "exact".to_string(),
                basis: "observed".to_string(),
                total_requests: 7,
                forwarded_requests: 6,
                short_circuited_requests: 1,
                control_response_requests: 0,
                response_bytes: 700,
                forwarded_response_bytes: 600,
                short_circuited_response_bytes: 100,
                control_response_bytes: 0,
            },
        ];

        let (readiness, receipts) = summarize_non_human_classification(
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-001".to_string(),
                lane: "scrapling_runtime".to_string(),
                profile: "crawler_baseline".to_string(),
                first_ts: 1_700_000_000,
                last_ts: 1_700_000_100,
                monitoring_event_count: 7,
                defense_delta_count: 1,
                ban_outcome_count: 0,
            }],
        );

        assert_eq!(readiness.status, "ready");
        assert!(readiness.blockers.is_empty());
        assert_eq!(readiness.live_receipt_count, 1);
        assert_eq!(readiness.adversary_sim_receipt_count, 1);
        assert_eq!(receipts.len(), 2);
        assert_eq!(receipts[0].category_id, "verified_beneficial_bot");
        assert_eq!(receipts[1].category_id, "indexing_bot");
        assert_eq!(receipts[1].assignment_status, "classified");
        assert!(receipts[1]
            .evidence_references
            .iter()
            .any(|reference| reference.contains("recent_sim_runs:simrun-001")));
    }
}
