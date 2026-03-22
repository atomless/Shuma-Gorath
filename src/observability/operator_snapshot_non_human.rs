use serde::{Deserialize, Serialize};

use crate::observability::monitoring::MonitoringSummary;
use crate::runtime::non_human_taxonomy::{canonical_non_human_taxonomy, NonHumanTaxonomyCatalog};

use super::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotNonHumanTrafficSummary {
    pub availability: String,
    pub taxonomy: NonHumanTaxonomyCatalog,
}

pub(super) fn non_human_traffic_summary(
    _summary: &MonitoringSummary,
    _recent_sim_runs: &[OperatorSnapshotRecentSimRun],
) -> OperatorSnapshotNonHumanTrafficSummary {
    OperatorSnapshotNonHumanTrafficSummary {
        availability: "taxonomy_seeded".to_string(),
        taxonomy: canonical_non_human_taxonomy(),
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
    }
}
