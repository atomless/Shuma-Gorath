use serde::{Deserialize, Serialize};

use crate::observability::monitoring::MonitoringSummary;
use crate::runtime::non_human_taxonomy::{canonical_non_human_taxonomy, NonHumanTaxonomyCatalog};

use super::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;
use super::non_human_classification::{
    non_human_decision_chain, summarize_non_human_recognition_evaluation,
    summarize_non_human_restriction_classification,
    summarize_non_human_simulator_ground_truth, NonHumanClassificationReadiness,
    NonHumanClassificationReceipt, NonHumanSimulatorGroundTruthSummary,
};
use super::non_human_coverage::{summarize_non_human_coverage, NonHumanCoverageSummary};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotNonHumanRecognitionEvaluationSummary {
    pub readiness: NonHumanClassificationReadiness,
    pub coverage: NonHumanCoverageSummary,
    pub simulator_ground_truth: NonHumanSimulatorGroundTruthSummary,
    pub comparison_status: String,
    pub current_exact_match_count: usize,
    pub degraded_match_count: usize,
    pub collapsed_to_unknown_count: usize,
    pub not_materialized_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comparison_rows: Vec<OperatorSnapshotNonHumanRecognitionComparisonRow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receipts: Vec<NonHumanClassificationReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotNonHumanRecognitionComparisonRow {
    pub category_id: String,
    pub category_label: String,
    pub inference_capability_status: String,
    pub comparison_status: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub inferred_category_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub inferred_category_label: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub exactness: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub basis: String,
    pub note: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotNonHumanTrafficSummary {
    pub availability: String,
    pub taxonomy: NonHumanTaxonomyCatalog,
    pub coverage: NonHumanCoverageSummary,
    pub restriction_readiness: NonHumanClassificationReadiness,
    pub decision_chain: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub restriction_receipts: Vec<NonHumanClassificationReceipt>,
    pub recognition_evaluation: OperatorSnapshotNonHumanRecognitionEvaluationSummary,
}

pub(super) fn non_human_traffic_summary(
    summary: &MonitoringSummary,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
) -> OperatorSnapshotNonHumanTrafficSummary {
    let (restriction_readiness, restriction_receipts) =
        summarize_non_human_restriction_classification(summary, recent_sim_runs);
    let (recognition_readiness, recognition_receipts) =
        summarize_non_human_recognition_evaluation(summary, recent_sim_runs);
    let coverage = summarize_non_human_coverage(recognition_receipts.as_slice());
    let simulator_ground_truth = summarize_non_human_simulator_ground_truth(recent_sim_runs);
    let (
        comparison_status,
        current_exact_match_count,
        degraded_match_count,
        collapsed_to_unknown_count,
        not_materialized_count,
        comparison_rows,
    ) = recognition_comparison_summary(
        recognition_receipts.as_slice(),
        &simulator_ground_truth,
    );
    OperatorSnapshotNonHumanTrafficSummary {
        availability: "taxonomy_seeded".to_string(),
        taxonomy: canonical_non_human_taxonomy(),
        coverage: coverage.clone(),
        restriction_readiness,
        decision_chain: non_human_decision_chain(),
        restriction_receipts,
        recognition_evaluation: OperatorSnapshotNonHumanRecognitionEvaluationSummary {
            readiness: recognition_readiness,
            coverage,
            simulator_ground_truth,
            comparison_status,
            current_exact_match_count,
            degraded_match_count,
            collapsed_to_unknown_count,
            not_materialized_count,
            comparison_rows,
            receipts: recognition_receipts,
        },
    }
}

fn recognition_comparison_summary(
    receipts: &[NonHumanClassificationReceipt],
    simulator_ground_truth: &NonHumanSimulatorGroundTruthSummary,
) -> (
    String,
    usize,
    usize,
    usize,
    usize,
    Vec<OperatorSnapshotNonHumanRecognitionComparisonRow>,
) {
    let adversary_sim_receipts: Vec<_> = receipts
        .iter()
        .filter(|receipt| receipt.traffic_origin == "adversary_sim")
        .collect();
    let unknown_current = adversary_sim_receipts.iter().find(|receipt| {
        receipt.category_id == "unknown_non_human"
            && receipt.degradation_status == "current"
            && receipt.basis != "projected_recent_sim_run"
    });
    let comparison_rows: Vec<_> = simulator_ground_truth
        .categories
        .iter()
        .map(|ground_truth| {
            let exact_match = adversary_sim_receipts.iter().find(|receipt| {
                receipt.category_id == ground_truth.category_id
                    && receipt.assignment_status == "classified"
                    && receipt.degradation_status == "current"
            });
            let degraded_match = adversary_sim_receipts.iter().find(|receipt| {
                receipt.category_id == ground_truth.category_id
                    && receipt.basis != "projected_recent_sim_run"
                    && (receipt.assignment_status != "classified"
                        || receipt.degradation_status != "current")
            });
            let (inference_capability_status, capability_note) =
                shared_path_inference_capability(ground_truth.category_id.as_str());
            if let Some(receipt) = exact_match {
                OperatorSnapshotNonHumanRecognitionComparisonRow {
                    category_id: ground_truth.category_id.clone(),
                    category_label: ground_truth.category_label.clone(),
                    inference_capability_status: inference_capability_status.to_string(),
                    comparison_status: "current_exact_match".to_string(),
                    inferred_category_id: receipt.category_id.clone(),
                    inferred_category_label: receipt.category_label.clone(),
                    exactness: receipt.exactness.clone(),
                    basis: receipt.basis.clone(),
                    note: capability_note.to_string(),
                    evidence_references: merge_references(
                        ground_truth.evidence_references.as_slice(),
                        receipt.evidence_references.as_slice(),
                    ),
                }
            } else if let Some(receipt) = unknown_current {
                OperatorSnapshotNonHumanRecognitionComparisonRow {
                    category_id: ground_truth.category_id.clone(),
                    category_label: ground_truth.category_label.clone(),
                    inference_capability_status: inference_capability_status.to_string(),
                    comparison_status: "collapsed_to_unknown_non_human".to_string(),
                    inferred_category_id: receipt.category_id.clone(),
                    inferred_category_label: receipt.category_label.clone(),
                    exactness: receipt.exactness.clone(),
                    basis: receipt.basis.clone(),
                    note: capability_note.to_string(),
                    evidence_references: merge_references(
                        ground_truth.evidence_references.as_slice(),
                        receipt.evidence_references.as_slice(),
                    ),
                }
            } else if let Some(receipt) = degraded_match {
                OperatorSnapshotNonHumanRecognitionComparisonRow {
                    category_id: ground_truth.category_id.clone(),
                    category_label: ground_truth.category_label.clone(),
                    inference_capability_status: inference_capability_status.to_string(),
                    comparison_status: "degraded_match_only".to_string(),
                    inferred_category_id: receipt.category_id.clone(),
                    inferred_category_label: receipt.category_label.clone(),
                    exactness: receipt.exactness.clone(),
                    basis: receipt.basis.clone(),
                    note: capability_note.to_string(),
                    evidence_references: merge_references(
                        ground_truth.evidence_references.as_slice(),
                        receipt.evidence_references.as_slice(),
                    ),
                }
            } else {
                OperatorSnapshotNonHumanRecognitionComparisonRow {
                    category_id: ground_truth.category_id.clone(),
                    category_label: ground_truth.category_label.clone(),
                    inference_capability_status: inference_capability_status.to_string(),
                    comparison_status: "not_materialized".to_string(),
                    inferred_category_id: String::new(),
                    inferred_category_label: String::new(),
                    exactness: String::new(),
                    basis: String::new(),
                    note: capability_note.to_string(),
                    evidence_references: ground_truth.evidence_references.clone(),
                }
            }
        })
        .collect();

    let current_exact_match_count = comparison_rows
        .iter()
        .filter(|row| row.comparison_status == "current_exact_match")
        .count();
    let degraded_match_count = comparison_rows
        .iter()
        .filter(|row| row.comparison_status == "degraded_match_only")
        .count();
    let collapsed_to_unknown_count = comparison_rows
        .iter()
        .filter(|row| row.comparison_status == "collapsed_to_unknown_non_human")
        .count();
    let not_materialized_count = comparison_rows
        .iter()
        .filter(|row| row.comparison_status == "not_materialized")
        .count();

    let comparison_status = if comparison_rows.is_empty() {
        "not_observed".to_string()
    } else if not_materialized_count == 0
        && degraded_match_count == 0
        && collapsed_to_unknown_count == 0
    {
        "current_exact".to_string()
    } else if current_exact_match_count > 0 || degraded_match_count > 0 {
        "partial".to_string()
    } else if collapsed_to_unknown_count > 0 {
        "unknown_only".to_string()
    } else {
        "not_materialized".to_string()
    };

    (
        comparison_status,
        current_exact_match_count,
        degraded_match_count,
        collapsed_to_unknown_count,
        not_materialized_count,
        comparison_rows,
    )
}

fn shared_path_inference_capability(category_id: &str) -> (&'static str, &'static str) {
    match category_id {
        "indexing_bot" => (
            "declared_crawler_exact_supported",
            "Current shared-path exact inference exists when Shuma classifies traffic into the declared crawler lane.",
        ),
        "agent_on_behalf_of_human" => (
            "declared_or_verified_exact_supported",
            "Current exact inference exists through declared user-triggered agent or verified signed-agent evidence.",
        ),
        "verified_beneficial_bot" => (
            "verified_identity_exact_supported",
            "Current exact inference exists through verified-identity evidence rather than undeclared hostile shared-path classification.",
        ),
        "unknown_non_human" => (
            "coarse_fallback_supported",
            "Current runtime can classify suspicious automation into the coarse unknown-non-human bucket without fine category precision.",
        ),
        "ai_scraper_bot" => (
            "undeclared_shared_path_not_currently_exact",
            "Current runtime does not infer this category exactly for undeclared hostile traffic; it is only available through verified identity or evaluation-only simulator truth.",
        ),
        "automated_browser" => (
            "undeclared_shared_path_not_currently_exact",
            "Current runtime does not infer this browser category exactly for undeclared hostile traffic; browser pressure still contributes evidence without exact category attribution.",
        ),
        "http_agent" => (
            "undeclared_shared_path_not_currently_exact",
            "Current runtime does not infer this direct-request hostile category exactly for undeclared traffic; request-native hostile traffic still collapses before exact category attribution.",
        ),
        "browser_agent" => (
            "undeclared_shared_path_not_currently_exact",
            "Current runtime does not yet infer this broader browser-agent category exactly from shared-path evidence.",
        ),
        _ => (
            "not_classified",
            "Current runtime has no explicit exact shared-path inference contract for this category.",
        ),
    }
}

fn merge_references(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for reference in right {
        if !merged.iter().any(|value| value == reference) {
            merged.push(reference.clone());
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::non_human_traffic_summary;
    use crate::observability::monitoring::{
        MonitoringSummary, RequestOutcomeCategorySummaryRow, RequestOutcomeLaneSummaryRow,
    };
    use crate::observability::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;

    #[test]
    fn non_human_snapshot_summary_exposes_seeded_taxonomy_catalog() {
        let summary = non_human_traffic_summary(&MonitoringSummary::default(), &[]);

        assert_eq!(summary.availability, "taxonomy_seeded");
        assert_eq!(summary.taxonomy.schema_version, "non_human_taxonomy_v1");
        assert_eq!(summary.taxonomy.categories.len(), 8);
        assert_eq!(summary.restriction_readiness.status, "not_observed");
        assert_eq!(summary.coverage.schema_version, "non_human_coverage_v1");
        assert_eq!(summary.coverage.overall_status, "unavailable");
        assert_eq!(summary.coverage.mapped_category_count, 6);
        assert_eq!(summary.coverage.gap_category_count, 2);
        assert_eq!(summary.recognition_evaluation.readiness.status, "not_observed");
        assert_eq!(
            summary.recognition_evaluation.simulator_ground_truth.status,
            "not_observed"
        );
        assert_eq!(
            summary.decision_chain,
            vec![
                "fingerprinting_and_evidence".to_string(),
                "categorization".to_string(),
                "cumulative_abuse_score_botness".to_string(),
                "posture_severity".to_string(),
            ]
        );
    }

    #[test]
    fn non_human_snapshot_summary_marks_recent_run_only_scrapling_categories_as_stale() {
        let mut monitoring = MonitoringSummary::default();
        monitoring.request_outcomes.by_lane = vec![RequestOutcomeLaneSummaryRow {
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

        let summary = non_human_traffic_summary(
            &monitoring,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-request-native".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec![
                    "crawler".to_string(),
                    "bulk_scraper".to_string(),
                    "browser_automation".to_string(),
                    "stealth_browser".to_string(),
                    "http_agent".to_string(),
                ],
                observed_category_ids: vec![
                    "indexing_bot".to_string(),
                    "ai_scraper_bot".to_string(),
                    "automated_browser".to_string(),
                    "http_agent".to_string(),
                ],
                first_ts: 1_700_000_000,
                last_ts: 1_700_000_100,
                monitoring_event_count: 9,
                defense_delta_count: 2,
                ban_outcome_count: 0,
                owned_surface_coverage: None,
            }],
        );

        assert_eq!(summary.restriction_readiness.status, "partial");
        assert_eq!(
            summary.restriction_readiness.adversary_sim_receipt_count,
            0
        );
        assert!(summary
            .restriction_readiness
            .blockers
            .contains(&"adversary_sim_non_human_receipts_missing".to_string()));
        assert_eq!(summary.recognition_evaluation.readiness.status, "partial");
        assert_eq!(
            summary.recognition_evaluation.readiness.adversary_sim_receipt_count,
            4
        );
        assert_eq!(summary.coverage.overall_status, "stale");
        assert_eq!(summary.coverage.stale_category_count, 4);
        assert_eq!(
            summary.recognition_evaluation.comparison_status,
            "not_materialized"
        );
        assert_eq!(summary.recognition_evaluation.degraded_match_count, 0);
        assert_eq!(summary.recognition_evaluation.not_materialized_count, 4);
        assert!(summary
            .recognition_evaluation
            .readiness
            .blockers
            .contains(&"degraded_category_receipts_present".to_string()));
        assert!(summary
            .recognition_evaluation
            .receipts
            .iter()
            .any(|receipt| receipt.category_id == "ai_scraper_bot"));
        assert!(summary
            .recognition_evaluation
            .receipts
            .iter()
            .any(|receipt| receipt.category_id == "automated_browser"));
        assert!(summary
            .recognition_evaluation
            .receipts
            .iter()
            .any(|receipt| receipt.category_id == "http_agent"));
        assert_eq!(
            summary
                .recognition_evaluation
                .simulator_ground_truth
                .recent_sim_run_count,
            1
        );
        assert!(summary
            .recognition_evaluation
            .comparison_rows
            .iter()
            .all(|row| row.comparison_status == "not_materialized"));
    }

    #[test]
    fn non_human_snapshot_summary_projects_live_verified_search_into_indexing_bot() {
        let mut monitoring = MonitoringSummary::default();
        monitoring.request_outcomes.by_non_human_category = vec![RequestOutcomeCategorySummaryRow {
            traffic_origin: "live".to_string(),
            measurement_scope: "ingress_primary".to_string(),
            execution_mode: "enforced".to_string(),
            category_id: "indexing_bot".to_string(),
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
        }];

        let summary = non_human_traffic_summary(&monitoring, &[]);

        assert_eq!(summary.restriction_readiness.live_receipt_count, 1);
        assert!(summary
            .restriction_receipts
            .iter()
            .any(|receipt| receipt.category_id == "indexing_bot"));
    }

    #[test]
    fn recognition_evaluation_marks_current_unknown_non_human_as_collapsed_truth() {
        let mut monitoring = MonitoringSummary::default();
        monitoring.request_outcomes.by_lane = vec![
            RequestOutcomeLaneSummaryRow {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                lane: "verified_bot".to_string(),
                exactness: "exact".to_string(),
                basis: "observed".to_string(),
                total_requests: 2,
                forwarded_requests: 2,
                short_circuited_requests: 0,
                control_response_requests: 0,
                response_bytes: 200,
                forwarded_upstream_latency_ms_total: 0,
                forwarded_response_bytes: 200,
                short_circuited_response_bytes: 0,
                control_response_bytes: 0,
            },
            RequestOutcomeLaneSummaryRow {
                traffic_origin: "adversary_sim".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                lane: "suspicious_automation".to_string(),
                exactness: "exact".to_string(),
                basis: "observed".to_string(),
                total_requests: 6,
                forwarded_requests: 2,
                short_circuited_requests: 4,
                control_response_requests: 0,
                response_bytes: 600,
                forwarded_upstream_latency_ms_total: 0,
                forwarded_response_bytes: 200,
                short_circuited_response_bytes: 400,
                control_response_bytes: 0,
            },
        ];

        let summary = non_human_traffic_summary(
            &monitoring,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-unknown-collapse".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec!["bulk_scraper".to_string()],
                observed_category_ids: vec!["ai_scraper_bot".to_string()],
                first_ts: 1_700_000_000,
                last_ts: 1_700_000_100,
                monitoring_event_count: 6,
                defense_delta_count: 2,
                ban_outcome_count: 0,
                owned_surface_coverage: None,
            }],
        );

        assert_eq!(summary.recognition_evaluation.comparison_status, "unknown_only");
        assert_eq!(summary.recognition_evaluation.collapsed_to_unknown_count, 1);
        let row = summary
            .recognition_evaluation
            .comparison_rows
            .iter()
            .find(|row| row.category_id == "ai_scraper_bot")
            .expect("ai scraper recognition comparison row");
        assert_eq!(row.comparison_status, "collapsed_to_unknown_non_human");
        assert_eq!(row.inferred_category_id, "unknown_non_human");
        assert_eq!(row.inferred_category_label, "Unknown non-human");
        assert_eq!(
            row.inference_capability_status,
            "undeclared_shared_path_not_currently_exact"
        );
        assert_eq!(row.basis, "observed");
    }
}
