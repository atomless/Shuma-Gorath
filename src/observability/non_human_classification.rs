use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::bot_identity::contracts::IdentityCategory;
use crate::observability::monitoring::{
    MonitoringSummary, RequestOutcomeCategorySummaryRow, RequestOutcomeLaneSummaryRow,
    RequestOutcomeScopeSummaryRow, VerifiedIdentitySeenRow,
};
use crate::observability::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;
use crate::runtime::non_human_taxonomy::{
    canonical_non_human_taxonomy, NonHumanCategoryDescriptor,
};
use crate::runtime::traffic_classification::{
    non_human_category_assignment_for_lane, verified_identity_category_assignment_for_category,
    TrafficLane,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NonHumanSimulatorGroundTruthCategory {
    pub category_id: String,
    pub category_label: String,
    pub recent_run_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct NonHumanSimulatorGroundTruthSummary {
    pub status: String,
    pub recent_sim_run_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<NonHumanSimulatorGroundTruthCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct VerifiedIdentityTaxonomyAlignmentReceipt {
    pub operator: String,
    pub stable_identity: String,
    pub scheme: String,
    pub verified_identity_category: String,
    pub projected_category_id: String,
    pub projected_category_label: String,
    pub alignment_status: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub degradation_reason: String,
    pub count: u64,
    #[serde(default)]
    pub end_user_controlled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct VerifiedIdentityTaxonomyAlignmentSummary {
    pub schema_version: String,
    pub status: String,
    pub aligned_count: u64,
    pub fallback_count: u64,
    pub misaligned_count: u64,
    pub insufficient_evidence_count: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receipts: Vec<VerifiedIdentityTaxonomyAlignmentReceipt>,
}

pub(crate) fn non_human_decision_chain() -> Vec<String> {
    NON_HUMAN_DECISION_CHAIN
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(crate) fn summarize_verified_identity_taxonomy_alignment(
    summary: &MonitoringSummary,
    non_human_receipts: &[NonHumanClassificationReceipt],
) -> VerifiedIdentityTaxonomyAlignmentSummary {
    let categories = canonical_non_human_taxonomy().categories;
    let live_receipts: Vec<_> = non_human_receipts
        .iter()
        .filter(|receipt| receipt.traffic_origin == "live")
        .collect();
    let mut receipts: Vec<_> = summary
        .verified_identity
        .top_verified_identities
        .iter()
        .filter_map(|row| {
            alignment_receipt_from_seen_row(row, live_receipts.as_slice(), &categories)
        })
        .collect();
    receipts.sort_by(|left, right| {
        alignment_sort_key(left.alignment_status.as_str())
            .cmp(&alignment_sort_key(right.alignment_status.as_str()))
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.operator.cmp(&right.operator))
            .then_with(|| left.stable_identity.cmp(&right.stable_identity))
    });

    let mut summary_row = VerifiedIdentityTaxonomyAlignmentSummary {
        schema_version: "verified_identity_taxonomy_alignment_v1".to_string(),
        status: "not_observed".to_string(),
        aligned_count: 0,
        fallback_count: 0,
        misaligned_count: 0,
        insufficient_evidence_count: 0,
        receipts,
    };
    for receipt in &summary_row.receipts {
        match receipt.alignment_status.as_str() {
            "aligned" => {
                summary_row.aligned_count =
                    summary_row.aligned_count.saturating_add(receipt.count)
            }
            "fallback" => {
                summary_row.fallback_count =
                    summary_row.fallback_count.saturating_add(receipt.count)
            }
            "misaligned" => {
                summary_row.misaligned_count =
                    summary_row.misaligned_count.saturating_add(receipt.count)
            }
            "insufficient_evidence" => {
                summary_row.insufficient_evidence_count = summary_row
                    .insufficient_evidence_count
                    .saturating_add(receipt.count)
            }
            _ => {}
        }
    }
    summary_row.status = if summary_row.receipts.is_empty() {
        "not_observed".to_string()
    } else if summary_row.misaligned_count > 0 {
        "degraded".to_string()
    } else if summary_row.insufficient_evidence_count > 0 {
        "insufficient_evidence".to_string()
    } else if summary_row.fallback_count > 0 {
        "fallback_only".to_string()
    } else {
        "aligned".to_string()
    };
    summary_row
}

pub(crate) fn summarize_non_human_restriction_classification(
    summary: &MonitoringSummary,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
) -> (
    NonHumanClassificationReadiness,
    Vec<NonHumanClassificationReceipt>,
) {
    let taxonomy = canonical_non_human_taxonomy();
    let mut receipts = collect_current_receipts_for_origin(
        summary,
        recent_sim_runs,
        &taxonomy.categories,
        "live",
    );
    let sim_receipts = collect_current_receipts_for_origin(
        summary,
        recent_sim_runs,
        &taxonomy.categories,
        "adversary_sim",
    );
    receipts.extend(sim_receipts);
    sort_receipts(&mut receipts);
    (build_readiness(receipts.as_slice()), receipts)
}

pub(crate) fn summarize_non_human_recognition_evaluation(
    summary: &MonitoringSummary,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
) -> (
    NonHumanClassificationReadiness,
    Vec<NonHumanClassificationReceipt>,
) {
    let taxonomy = canonical_non_human_taxonomy();
    let sim_scope = summary.request_outcomes.by_scope.iter().find(|row| {
        row.traffic_origin == "adversary_sim"
            && row.measurement_scope == "ingress_primary"
            && row.execution_mode == "enforced"
    });
    let mut receipts = collect_current_receipts_for_origin(
        summary,
        recent_sim_runs,
        &taxonomy.categories,
        "live",
    );
    let mut sim_receipts: BTreeMap<String, NonHumanClassificationReceipt> =
        collect_current_receipts_for_origin(
            summary,
            recent_sim_runs,
            &taxonomy.categories,
            "adversary_sim",
        )
        .into_iter()
        .map(|receipt| (receipt.category_id.clone(), receipt))
        .collect();
    for receipt in sim_receipts_from_recent_runs(recent_sim_runs, &taxonomy.categories, sim_scope) {
        sim_receipts
            .entry(receipt.category_id.clone())
            .and_modify(|existing| {
                for reference in &receipt.evidence_references {
                    if !existing
                        .evidence_references
                        .iter()
                        .any(|value| value == reference)
                    {
                        existing.evidence_references.push(reference.clone());
                    }
                }
            })
            .or_insert(receipt);
    }
    receipts.extend(sim_receipts.into_values());
    sort_receipts(&mut receipts);
    (build_readiness(receipts.as_slice()), receipts)
}

pub(crate) fn summarize_non_human_simulator_ground_truth(
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
) -> NonHumanSimulatorGroundTruthSummary {
    let taxonomy = canonical_non_human_taxonomy();
    let mut categories: BTreeMap<String, NonHumanSimulatorGroundTruthCategory> = BTreeMap::new();
    for run in recent_sim_runs {
        for category_id in &run.observed_category_ids {
            let Some(category) = taxonomy
                .categories
                .iter()
                .find(|descriptor| descriptor.category_id.as_str() == category_id)
            else {
                continue;
            };
            let entry = categories
                .entry(category_id.clone())
                .or_insert_with(|| NonHumanSimulatorGroundTruthCategory {
                    category_id: category_id.clone(),
                    category_label: category.label.clone(),
                    recent_run_count: 0,
                    evidence_references: Vec::new(),
                });
            entry.recent_run_count = entry.recent_run_count.saturating_add(1);
            let reference = format!(
                "recent_sim_runs:{}:{}:{}",
                run.run_id, run.profile, category_id
            );
            if !entry.evidence_references.iter().any(|value| value == &reference) {
                entry.evidence_references.push(reference);
            }
        }
    }

    NonHumanSimulatorGroundTruthSummary {
        status: if categories.is_empty() {
            "not_observed".to_string()
        } else {
            "observed_recent_runs".to_string()
        },
        recent_sim_run_count: recent_sim_runs.len(),
        categories: categories.into_values().collect(),
    }
}

fn traffic_origin_sort_key(value: &str) -> u8 {
    match value {
        "live" => 0,
        "adversary_sim" => 1,
        _ => 2,
    }
}

fn collect_current_receipts_for_origin(
    summary: &MonitoringSummary,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
    categories: &[NonHumanCategoryDescriptor],
    traffic_origin: &str,
) -> Vec<NonHumanClassificationReceipt> {
    if summary
        .request_outcomes
        .by_non_human_category
        .iter()
        .any(|row| row.traffic_origin == traffic_origin)
    {
        summary
            .request_outcomes
            .by_non_human_category
            .iter()
            .filter(|row| row.traffic_origin == traffic_origin)
            .filter_map(|row| receipt_from_category_row(row, recent_sim_runs, categories))
            .collect()
    } else {
        summary
            .request_outcomes
            .by_lane
            .iter()
            .filter(|row| row.traffic_origin == traffic_origin)
            .filter_map(|row| receipt_from_lane_row(row, recent_sim_runs, categories))
            .collect()
    }
}

fn sort_receipts(receipts: &mut Vec<NonHumanClassificationReceipt>) {
    receipts.sort_by(|left, right| {
        traffic_origin_sort_key(left.traffic_origin.as_str())
            .cmp(&traffic_origin_sort_key(right.traffic_origin.as_str()))
            .then_with(|| left.category_id.cmp(&right.category_id))
            .then_with(|| left.lane.cmp(&right.lane))
    });
}

fn build_readiness(receipts: &[NonHumanClassificationReceipt]) -> NonHumanClassificationReadiness {
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

    NonHumanClassificationReadiness {
        status,
        blockers,
        live_receipt_count,
        adversary_sim_receipt_count,
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

fn receipt_from_category_row(
    row: &RequestOutcomeCategorySummaryRow,
    _recent_sim_runs: &[OperatorSnapshotRecentSimRun],
    categories: &[NonHumanCategoryDescriptor],
) -> Option<NonHumanClassificationReceipt> {
    let category = categories
        .iter()
        .find(|descriptor| descriptor.category_id.as_str() == row.category_id)?;
    Some(NonHumanClassificationReceipt {
        traffic_origin: row.traffic_origin.clone(),
        measurement_scope: row.measurement_scope.clone(),
        execution_mode: row.execution_mode.clone(),
        lane: "category_crosswalk".to_string(),
        category_id: row.category_id.clone(),
        category_label: category.label.clone(),
        assignment_status: row.assignment_status.clone(),
        exactness: row.exactness.clone(),
        basis: row.basis.clone(),
        degradation_status: if row.exactness == "exact"
            && matches!(row.basis.as_str(), "observed" | "verified")
        {
            "current".to_string()
        } else {
            "degraded".to_string()
        },
        total_requests: row.total_requests,
        forwarded_requests: row.forwarded_requests,
        short_circuited_requests: row.short_circuited_requests,
        evidence_references: vec![format!(
            "request_outcomes.by_non_human_category:{}:{}:{}:{}",
            row.traffic_origin, row.measurement_scope, row.execution_mode, row.category_id
        )],
    })
}

fn sim_receipts_from_recent_runs(
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
    categories: &[NonHumanCategoryDescriptor],
    _sim_scope: Option<&RequestOutcomeScopeSummaryRow>,
) -> Vec<NonHumanClassificationReceipt> {
    let mut receipts: BTreeMap<String, NonHumanClassificationReceipt> = BTreeMap::new();
    for run in recent_sim_runs {
        for category_id in &run.observed_category_ids {
            let Some(category) = categories
                .iter()
                .find(|descriptor| descriptor.category_id.as_str() == category_id)
            else {
                continue;
            };
            let entry = receipts
                .entry(category_id.clone())
                .or_insert_with(|| NonHumanClassificationReceipt {
                    traffic_origin: "adversary_sim".to_string(),
                    measurement_scope: "ingress_primary".to_string(),
                    execution_mode: "enforced".to_string(),
                    lane: run.lane.clone(),
                    category_id: category_id.clone(),
                    category_label: category.label.clone(),
                    assignment_status: "classified".to_string(),
                    exactness: "derived".to_string(),
                    basis: "projected_recent_sim_run".to_string(),
                    degradation_status: "degraded".to_string(),
                    total_requests: 0,
                    forwarded_requests: 0,
                    short_circuited_requests: 0,
                    evidence_references: Vec::new(),
                });
            let reference = format!(
                "recent_sim_runs:{}:{}:{}",
                run.run_id, run.profile, category_id
            );
            if !entry.evidence_references.iter().any(|value| value == &reference) {
                entry.evidence_references.push(reference);
            }
        }
    }
    receipts.into_values().collect()
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

fn alignment_receipt_from_seen_row(
    row: &VerifiedIdentitySeenRow,
    live_receipts: &[&NonHumanClassificationReceipt],
    categories: &[NonHumanCategoryDescriptor],
) -> Option<VerifiedIdentityTaxonomyAlignmentReceipt> {
    let identity_category = identity_category_from_str(row.category.as_str())?;
    let projected = verified_identity_category_assignment_for_category(identity_category);
    let projected_category = categories
        .iter()
        .find(|category| category.category_id == projected.category_id)?;
    let matching_live_receipt = live_receipts.iter().find(|receipt| {
        receipt.category_id == projected.category_id.as_str()
            && receipt.assignment_status == "classified"
            && receipt.degradation_status == "current"
    });
    let (alignment_status, degradation_reason) = if identity_category == IdentityCategory::Other {
        ("fallback", "verified_identity_category_other")
    } else if matching_live_receipt.is_some() {
        ("aligned", "")
    } else if live_receipts.is_empty() {
        ("insufficient_evidence", "live_non_human_receipts_missing")
    } else {
        ("misaligned", "projected_category_not_observed")
    };

    let mut evidence_references = vec![format!(
        "verified_identity.top_verified_identities:{}:{}:{}",
        row.operator, row.scheme, row.stable_identity
    )];
    if let Some(receipt) = matching_live_receipt {
        evidence_references.extend(receipt.evidence_references.iter().cloned());
    }

    Some(VerifiedIdentityTaxonomyAlignmentReceipt {
        operator: row.operator.clone(),
        stable_identity: row.stable_identity.clone(),
        scheme: row.scheme.clone(),
        verified_identity_category: row.category.clone(),
        projected_category_id: projected.category_id.as_str().to_string(),
        projected_category_label: projected_category.label.clone(),
        alignment_status: alignment_status.to_string(),
        degradation_reason: degradation_reason.to_string(),
        count: row.count,
        end_user_controlled: row.end_user_controlled,
        evidence_references,
    })
}

fn identity_category_from_str(value: &str) -> Option<IdentityCategory> {
    match value {
        "training" => Some(IdentityCategory::Training),
        "search" => Some(IdentityCategory::Search),
        "user_triggered_agent" => Some(IdentityCategory::UserTriggeredAgent),
        "preview" => Some(IdentityCategory::Preview),
        "service_agent" => Some(IdentityCategory::ServiceAgent),
        "other" => Some(IdentityCategory::Other),
        _ => None,
    }
}

fn alignment_sort_key(status: &str) -> u8 {
    match status {
        "misaligned" => 0,
        "insufficient_evidence" => 1,
        "fallback" => 2,
        "aligned" => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        summarize_non_human_recognition_evaluation,
        summarize_non_human_restriction_classification,
        summarize_verified_identity_taxonomy_alignment,
    };
    use crate::observability::monitoring::{
        MonitoringSummary, RequestOutcomeCategorySummaryRow, RequestOutcomeLaneSummaryRow,
        RequestOutcomeScopeSummaryRow, VerifiedIdentitySeenRow,
    };
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
                forwarded_upstream_latency_ms_total: 0,
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
                forwarded_upstream_latency_ms_total: 0,
                forwarded_response_bytes: 600,
                short_circuited_response_bytes: 100,
                control_response_bytes: 0,
            },
        ];

        let (readiness, receipts) = summarize_non_human_restriction_classification(
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-001".to_string(),
                lane: "scrapling_runtime".to_string(),
                profile: "crawler_baseline".to_string(),
                observed_fulfillment_modes: Vec::new(),
                observed_category_ids: Vec::new(),
                first_ts: 1_700_000_000,
                last_ts: 1_700_000_100,
                monitoring_event_count: 7,
                defense_delta_count: 1,
                ban_outcome_count: 0,
                owned_surface_coverage: None,
                latest_scrapling_realism_receipt: None,
                llm_runtime_summary: None,
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

    #[test]
    fn classification_summary_marks_recent_run_only_sim_category_receipts_as_degraded() {
        let mut summary = MonitoringSummary::default();
        summary.request_outcomes.by_lane = vec![RequestOutcomeLaneSummaryRow {
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
            forwarded_upstream_latency_ms_total: 0,
            forwarded_response_bytes: 500,
            short_circuited_response_bytes: 0,
            control_response_bytes: 0,
        }];
        summary.request_outcomes.by_scope = vec![RequestOutcomeScopeSummaryRow {
            traffic_origin: "adversary_sim".to_string(),
            measurement_scope: "ingress_primary".to_string(),
            execution_mode: "enforced".to_string(),
            total_requests: 9,
            forwarded_requests: 2,
            short_circuited_requests: 7,
            control_response_requests: 0,
            response_bytes: 900,
            forwarded_upstream_latency_ms_total: 0,
            forwarded_response_bytes: 200,
            short_circuited_response_bytes: 700,
            control_response_bytes: 0,
        }];

        let (readiness, receipts) = summarize_non_human_recognition_evaluation(
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-request-native".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec![
                    "crawler".to_string(),
                    "bulk_scraper".to_string(),
                    "http_agent".to_string(),
                ],
                observed_category_ids: vec![
                    "indexing_bot".to_string(),
                    "ai_scraper_bot".to_string(),
                    "http_agent".to_string(),
                ],
                first_ts: 1_700_000_000,
                last_ts: 1_700_000_100,
                monitoring_event_count: 9,
                defense_delta_count: 2,
                ban_outcome_count: 0,
                owned_surface_coverage: None,
                latest_scrapling_realism_receipt: None,
                llm_runtime_summary: None,
            }],
        );

        assert_eq!(readiness.status, "partial");
        assert!(readiness
            .blockers
            .contains(&"degraded_category_receipts_present".to_string()));
        assert_eq!(readiness.live_receipt_count, 1);
        assert_eq!(readiness.adversary_sim_receipt_count, 3);
        assert_eq!(receipts.len(), 4);
        assert!(receipts
            .iter()
            .any(|receipt| receipt.category_id == "indexing_bot"));
        assert!(receipts
            .iter()
            .any(|receipt| receipt.category_id == "ai_scraper_bot"));
        assert!(receipts
            .iter()
            .any(|receipt| receipt.category_id == "http_agent"));
        let ai_scraper = receipts
            .iter()
            .find(|receipt| receipt.category_id == "ai_scraper_bot")
            .expect("ai scraper receipt");
        assert_eq!(ai_scraper.exactness, "derived");
        assert_eq!(ai_scraper.basis, "projected_recent_sim_run");
        assert_eq!(ai_scraper.degradation_status, "degraded");
        assert_eq!(ai_scraper.total_requests, 0);
        assert_eq!(ai_scraper.forwarded_requests, 0);
        assert_eq!(ai_scraper.short_circuited_requests, 0);
    }

    #[test]
    fn classification_summary_projects_live_verified_category_crosswalk_receipts() {
        let mut summary = MonitoringSummary::default();
        summary.request_outcomes.by_non_human_category = vec![RequestOutcomeCategorySummaryRow {
            traffic_origin: "live".to_string(),
            measurement_scope: "ingress_primary".to_string(),
            execution_mode: "enforced".to_string(),
            category_id: "indexing_bot".to_string(),
            assignment_status: "classified".to_string(),
            exactness: "exact".to_string(),
            basis: "observed".to_string(),
            total_requests: 4,
            forwarded_requests: 4,
            short_circuited_requests: 0,
            control_response_requests: 0,
            response_bytes: 400,
            forwarded_upstream_latency_ms_total: 0,
            forwarded_response_bytes: 400,
            short_circuited_response_bytes: 0,
            control_response_bytes: 0,
        }];

        let (readiness, receipts) = summarize_non_human_restriction_classification(&summary, &[]);

        assert_eq!(readiness.live_receipt_count, 1);
        assert_eq!(receipts[0].category_id, "indexing_bot");
        assert_eq!(receipts[0].lane, "category_crosswalk");
        assert!(receipts[0]
            .evidence_references
            .iter()
            .any(|reference| reference.contains("request_outcomes.by_non_human_category")));
    }

    #[test]
    fn verified_identity_alignment_summary_distinguishes_aligned_fallback_and_misaligned_rows() {
        let mut verified_summary = MonitoringSummary::default();
        verified_summary.verified_identity.top_verified_identities = vec![
            VerifiedIdentitySeenRow {
                operator: "search.example".to_string(),
                stable_identity: "crawler-1".to_string(),
                scheme: "http_message_signatures".to_string(),
                category: "search".to_string(),
                provenance: "native".to_string(),
                end_user_controlled: false,
                count: 5,
            },
            VerifiedIdentitySeenRow {
                operator: "misc.example".to_string(),
                stable_identity: "misc-1".to_string(),
                scheme: "http_message_signatures".to_string(),
                category: "other".to_string(),
                provenance: "native".to_string(),
                end_user_controlled: false,
                count: 3,
            },
            VerifiedIdentitySeenRow {
                operator: "service.example".to_string(),
                stable_identity: "svc-1".to_string(),
                scheme: "provider_verified_bot".to_string(),
                category: "service_agent".to_string(),
                provenance: "provider".to_string(),
                end_user_controlled: false,
                count: 2,
            },
        ];
        let mut classification_summary = MonitoringSummary::default();
        classification_summary.request_outcomes.by_non_human_category = vec![
            RequestOutcomeCategorySummaryRow {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                category_id: "indexing_bot".to_string(),
                assignment_status: "classified".to_string(),
                exactness: "exact".to_string(),
                basis: "observed".to_string(),
                total_requests: 5,
                forwarded_requests: 5,
                short_circuited_requests: 0,
                control_response_requests: 0,
                response_bytes: 500,
                forwarded_upstream_latency_ms_total: 0,
                forwarded_response_bytes: 500,
                short_circuited_response_bytes: 0,
                control_response_bytes: 0,
            },
            RequestOutcomeCategorySummaryRow {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                category_id: "verified_beneficial_bot".to_string(),
                assignment_status: "classified".to_string(),
                exactness: "exact".to_string(),
                basis: "observed".to_string(),
                total_requests: 3,
                forwarded_requests: 3,
                short_circuited_requests: 0,
                control_response_requests: 0,
                response_bytes: 300,
                forwarded_upstream_latency_ms_total: 0,
                forwarded_response_bytes: 300,
                short_circuited_response_bytes: 0,
                control_response_bytes: 0,
            },
        ];

        let (_, receipts) =
            summarize_non_human_restriction_classification(&classification_summary, &[]);
        let alignment =
            summarize_verified_identity_taxonomy_alignment(&verified_summary, receipts.as_slice());

        assert_eq!(alignment.schema_version, "verified_identity_taxonomy_alignment_v1");
        assert_eq!(alignment.status, "degraded");
        assert_eq!(alignment.aligned_count, 5);
        assert_eq!(alignment.fallback_count, 3);
        assert_eq!(alignment.misaligned_count, 2);
        assert_eq!(alignment.receipts[0].alignment_status, "misaligned");
        assert_eq!(alignment.receipts[0].projected_category_id, "http_agent");
        assert_eq!(alignment.receipts[1].alignment_status, "fallback");
        assert_eq!(
            alignment.receipts[1].degradation_reason,
            "verified_identity_category_other"
        );
        assert_eq!(alignment.receipts[2].alignment_status, "aligned");
        assert!(alignment.receipts[2]
            .evidence_references
            .iter()
            .any(|reference| reference.contains("request_outcomes.by_non_human_category")));
    }
}
