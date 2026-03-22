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
    use crate::observability::monitoring::MonitoringSummary;

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
}
