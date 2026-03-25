use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::observability::monitoring::{CountEntry, MonitoringSummary};
use crate::observability::non_human_classification::{
    summarize_verified_identity_taxonomy_alignment, NonHumanClassificationReceipt,
    VerifiedIdentityTaxonomyAlignmentSummary,
};

const TOP_VERIFIED_IDENTITY_ROWS: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct OperatorSnapshotVerifiedIdentityPolicySummary {
    pub total_requests: u64,
    pub forwarded_requests: u64,
    pub short_circuited_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotVerifiedIdentitySummary {
    pub availability: String,
    pub enabled: bool,
    pub native_web_bot_auth_enabled: bool,
    pub provider_assertions_enabled: bool,
    pub non_human_traffic_stance: String,
    pub named_policy_count: usize,
    pub service_profile_count: usize,
    pub attempts: u64,
    pub verified: u64,
    pub failed: u64,
    pub unique_verified_identities: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_failure_reasons: Vec<CountEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_schemes: Vec<CountEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_categories: Vec<CountEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_provenance: Vec<CountEntry>,
    #[serde(default, skip_serializing_if = "verified_identity_alignment_summary_is_empty")]
    pub taxonomy_alignment: VerifiedIdentityTaxonomyAlignmentSummary,
    #[serde(default, skip_serializing_if = "verified_identity_policy_summary_is_empty")]
    pub policy_tranche: OperatorSnapshotVerifiedIdentityPolicySummary,
}

pub(super) fn verified_identity_summary(
    summary: &MonitoringSummary,
    cfg: &Config,
    non_human_receipts: &[NonHumanClassificationReceipt],
) -> OperatorSnapshotVerifiedIdentitySummary {
    let policy_row = summary
        .request_outcomes
        .by_policy_source
        .iter()
        .find(|row| row.value == "policy_graph_verified_identity_tranche");

    OperatorSnapshotVerifiedIdentitySummary {
        availability: if cfg.verified_identity.enabled {
            "supported".to_string()
        } else {
            "not_configured".to_string()
        },
        enabled: cfg.verified_identity.enabled,
        native_web_bot_auth_enabled: cfg.verified_identity.native_web_bot_auth_enabled,
        provider_assertions_enabled: cfg.verified_identity.provider_assertions_enabled,
        non_human_traffic_stance: cfg.verified_identity.non_human_traffic_stance.as_str().to_string(),
        named_policy_count: cfg.verified_identity.named_policies.len(),
        service_profile_count: cfg.verified_identity.service_profiles.len(),
        attempts: summary.verified_identity.attempts,
        verified: summary.verified_identity.verified,
        failed: summary.verified_identity.failed,
        unique_verified_identities: summary.verified_identity.unique_verified_identities,
        top_failure_reasons: top_count_entries(&summary.verified_identity.failures),
        top_schemes: top_count_entries(&summary.verified_identity.schemes),
        top_categories: top_count_entries(&summary.verified_identity.categories),
        top_provenance: top_count_entries(&summary.verified_identity.provenance),
        taxonomy_alignment: summarize_verified_identity_taxonomy_alignment(
            summary,
            non_human_receipts,
        ),
        policy_tranche: OperatorSnapshotVerifiedIdentityPolicySummary {
            total_requests: policy_row.map(|row| row.total_requests).unwrap_or(0),
            forwarded_requests: policy_row.map(|row| row.forwarded_requests).unwrap_or(0),
            short_circuited_requests: policy_row
                .map(|row| row.short_circuited_requests)
                .unwrap_or(0),
        },
    }
}

fn top_count_entries(map: &std::collections::BTreeMap<String, u64>) -> Vec<CountEntry> {
    let mut rows: Vec<CountEntry> = map
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(label, count)| CountEntry {
            label: label.clone(),
            count: *count,
        })
        .collect();
    rows.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.label.cmp(&right.label))
    });
    rows.truncate(TOP_VERIFIED_IDENTITY_ROWS);
    rows
}

fn verified_identity_policy_summary_is_empty(
    summary: &OperatorSnapshotVerifiedIdentityPolicySummary,
) -> bool {
    summary.total_requests == 0
        && summary.forwarded_requests == 0
        && summary.short_circuited_requests == 0
}

fn verified_identity_alignment_summary_is_empty(
    summary: &VerifiedIdentityTaxonomyAlignmentSummary,
) -> bool {
    summary.receipts.is_empty()
}

#[cfg(test)]
mod tests {
    use super::verified_identity_summary;
    use crate::config::defaults;
    use crate::observability::non_human_classification::NonHumanClassificationReceipt;
    use crate::observability::monitoring::MonitoringSummary;

    #[test]
    fn verified_identity_summary_reports_configured_typed_snapshot_section() {
        let mut cfg = defaults().clone();
        cfg.verified_identity.enabled = true;
        cfg.verified_identity.native_web_bot_auth_enabled = true;
        cfg.verified_identity.provider_assertions_enabled = true;
        let mut summary = MonitoringSummary::default();
        summary.verified_identity.attempts = 4;
        summary.verified_identity.verified = 3;
        summary.verified_identity.failed = 1;
        summary.verified_identity.unique_verified_identities = 2;
        summary
            .verified_identity
            .failures
            .insert("directory_stale".to_string(), 1);
        summary
            .verified_identity
            .schemes
            .insert("http_message_signatures".to_string(), 3);
        summary
            .verified_identity
            .categories
            .insert("search".to_string(), 2);
        summary
            .verified_identity
            .provenance
            .insert("native".to_string(), 4);
        summary.verified_identity.top_verified_identities.push(
            crate::observability::monitoring::VerifiedIdentitySeenRow {
                operator: "search.example".to_string(),
                stable_identity: "crawler-1".to_string(),
                scheme: "http_message_signatures".to_string(),
                category: "search".to_string(),
                provenance: "native".to_string(),
                end_user_controlled: false,
                count: 3,
            },
        );
        summary.request_outcomes.by_policy_source.push(
            crate::observability::monitoring::RequestOutcomeBreakdownSummaryRow {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                value: "policy_graph_verified_identity_tranche".to_string(),
                total_requests: 4,
                forwarded_requests: 3,
                short_circuited_requests: 1,
                control_response_requests: 0,
            },
        );

        let snapshot = verified_identity_summary(
            &summary,
            &cfg,
            &[NonHumanClassificationReceipt {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                lane: "category_crosswalk".to_string(),
                category_id: "indexing_bot".to_string(),
                category_label: "Indexing Bot".to_string(),
                assignment_status: "classified".to_string(),
                exactness: "exact".to_string(),
                basis: "observed".to_string(),
                degradation_status: "current".to_string(),
                total_requests: 3,
                forwarded_requests: 3,
                short_circuited_requests: 0,
                evidence_references: vec![
                    "request_outcomes.by_non_human_category:live:ingress_primary:enforced:indexing_bot"
                        .to_string(),
                ],
            }],
        );

        assert_eq!(snapshot.availability, "supported");
        assert!(snapshot.enabled);
        assert_eq!(snapshot.named_policy_count, 0);
        assert_eq!(snapshot.service_profile_count, 4);
        assert_eq!(snapshot.attempts, 4);
        assert_eq!(snapshot.top_categories[0].label, "search");
        assert_eq!(snapshot.taxonomy_alignment.status, "aligned");
        assert_eq!(snapshot.taxonomy_alignment.aligned_count, 3);
        assert_eq!(
            snapshot.taxonomy_alignment.receipts[0].projected_category_id,
            "indexing_bot"
        );
        assert_eq!(snapshot.policy_tranche.forwarded_requests, 3);
        assert_eq!(snapshot.top_failure_reasons[0].label, "directory_stale");
    }

    #[test]
    fn verified_identity_summary_marks_disabled_sites_as_not_configured() {
        let mut cfg = defaults().clone();
        cfg.verified_identity.enabled = false;
        let snapshot = verified_identity_summary(&MonitoringSummary::default(), &cfg, &[]);

        assert_eq!(snapshot.availability, "not_configured");
        assert!(!snapshot.enabled);
        assert_eq!(snapshot.attempts, 0);
    }

    #[test]
    fn verified_identity_summary_projects_taxonomy_alignment_problem_rows() {
        let mut cfg = defaults().clone();
        cfg.verified_identity.enabled = true;
        let mut summary = MonitoringSummary::default();
        summary.verified_identity.top_verified_identities.push(
            crate::observability::monitoring::VerifiedIdentitySeenRow {
                operator: "service.example".to_string(),
                stable_identity: "svc-1".to_string(),
                scheme: "provider_verified_bot".to_string(),
                category: "service_agent".to_string(),
                provenance: "provider".to_string(),
                end_user_controlled: false,
                count: 2,
            },
        );

        let snapshot = verified_identity_summary(&summary, &cfg, &[]);

        assert_eq!(snapshot.taxonomy_alignment.status, "insufficient_evidence");
        assert_eq!(snapshot.taxonomy_alignment.insufficient_evidence_count, 2);
        assert_eq!(
            snapshot.taxonomy_alignment.receipts[0].alignment_status,
            "insufficient_evidence"
        );
        assert_eq!(
            snapshot.taxonomy_alignment.receipts[0].projected_category_id,
            "http_agent"
        );
    }
}
