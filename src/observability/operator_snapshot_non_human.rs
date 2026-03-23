use serde::{Deserialize, Serialize};

use crate::observability::monitoring::MonitoringSummary;
use crate::runtime::non_human_taxonomy::{canonical_non_human_taxonomy, NonHumanTaxonomyCatalog};

use super::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;
use super::non_human_classification::{
    non_human_decision_chain, summarize_non_human_classification, NonHumanClassificationReadiness,
    NonHumanClassificationReceipt,
};
use super::non_human_coverage::{summarize_non_human_coverage, NonHumanCoverageSummary};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotNonHumanTrafficSummary {
    pub availability: String,
    pub taxonomy: NonHumanTaxonomyCatalog,
    pub readiness: NonHumanClassificationReadiness,
    pub coverage: NonHumanCoverageSummary,
    pub decision_chain: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receipts: Vec<NonHumanClassificationReceipt>,
}

pub(super) fn non_human_traffic_summary(
    summary: &MonitoringSummary,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
) -> OperatorSnapshotNonHumanTrafficSummary {
    let (readiness, receipts) = summarize_non_human_classification(summary, recent_sim_runs);
    let coverage = summarize_non_human_coverage(receipts.as_slice());
    OperatorSnapshotNonHumanTrafficSummary {
        availability: "taxonomy_seeded".to_string(),
        taxonomy: canonical_non_human_taxonomy(),
        readiness,
        coverage,
        decision_chain: non_human_decision_chain(),
        receipts,
    }
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
        assert_eq!(summary.readiness.status, "not_observed");
        assert_eq!(summary.coverage.schema_version, "non_human_coverage_v1");
        assert_eq!(summary.coverage.overall_status, "unavailable");
        assert_eq!(summary.coverage.mapped_category_count, 6);
        assert_eq!(summary.coverage.gap_category_count, 2);
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
    fn non_human_snapshot_summary_projects_scrapling_request_native_coverage() {
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
            }],
        );

        assert_eq!(summary.readiness.status, "ready");
        assert_eq!(summary.readiness.adversary_sim_receipt_count, 3);
        assert_eq!(summary.coverage.covered_category_count, 3);
        assert!(summary
            .receipts
            .iter()
            .any(|receipt| receipt.category_id == "ai_scraper_bot"));
        assert!(summary
            .receipts
            .iter()
            .any(|receipt| receipt.category_id == "http_agent"));
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

        assert_eq!(summary.readiness.live_receipt_count, 1);
        assert!(summary
            .receipts
            .iter()
            .any(|receipt| receipt.category_id == "indexing_bot"));
    }
}
