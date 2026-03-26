use crate::config::Config;
use crate::observability::monitoring::{MonitoringSummary, RequestOutcomeBreakdownSummaryRow};
use crate::observability::operator_snapshot::OperatorSnapshotNonHumanTrafficSummary;
use crate::observability::operator_snapshot_objectives::OperatorObjectivesProfile;
use crate::observability::operator_snapshot_verified_identity::OperatorSnapshotVerifiedIdentitySummary;

use super::benchmark_results::{BenchmarkFamilyResult, BenchmarkMetricResult};
use super::benchmark_results_families::aggregate_budget_status;

const MIN_VERIFIED_CONFLICT_SAMPLE_SIZE: u64 = 3;

pub(super) fn beneficial_non_human_posture_family(
    summary: &MonitoringSummary,
    cfg: &Config,
    objectives: &OperatorObjectivesProfile,
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
    verified_identity: &OperatorSnapshotVerifiedIdentitySummary,
) -> BenchmarkFamilyResult {
    if !cfg.verified_identity.enabled {
        return BenchmarkFamilyResult {
            family_id: "beneficial_non_human_posture".to_string(),
            status: "not_applicable".to_string(),
            capability_gate: "partially_supported".to_string(),
            note: "Verified-identity benchmarking is configured off for this site, so the beneficial non-human family is currently not applicable.".to_string(),
            baseline_status: None,
            comparison_status: "not_available".to_string(),
            metrics: vec![
                not_applicable_metric("allowed_as_intended_rate"),
                not_applicable_metric("friction_mismatch_rate"),
                not_applicable_metric("deny_mismatch_rate"),
                not_applicable_metric("coverage_status"),
                not_applicable_metric("taxonomy_alignment_mismatch_rate"),
                not_applicable_metric("verified_botness_conflict_rate"),
                not_applicable_metric("user_triggered_agent_friction_mismatch_rate"),
            ],
        };
    }

    let policy_row = summary
        .request_outcomes
        .by_policy_source
        .iter()
        .find(|row| row.value == "policy_graph_verified_identity_tranche");
    let total_requests = policy_row.map(|row| row.total_requests).unwrap_or(0);
    let forwarded_requests = policy_row.map(|row| row.forwarded_requests).unwrap_or(0);
    let short_circuited_requests = policy_row
        .map(|row| row.short_circuited_requests)
        .unwrap_or(0);
    let allow_capable = verified_identity
        .effective_non_human_policy
        .verified_identity_override_mode
        == "explicit_overrides_eligible";
    let coverage_ratio = ratio(
        summary.verified_identity.verified,
        summary.verified_identity.attempts,
    );
    let mismatch_ratio = if allow_capable {
        ratio(short_circuited_requests, total_requests)
    } else {
        ratio(forwarded_requests, total_requests)
    };
    let alignment_sample_size = verified_identity
        .taxonomy_alignment
        .aligned_count
        .saturating_add(verified_identity.taxonomy_alignment.fallback_count)
        .saturating_add(verified_identity.taxonomy_alignment.misaligned_count);
    let alignment_mismatch_ratio = ratio(
        verified_identity.taxonomy_alignment.misaligned_count,
        alignment_sample_size,
    );
    let (protected_total_requests, protected_short_circuited_requests) =
        protected_verified_conflict_sample(objectives, non_human_traffic, verified_identity);
    let (user_triggered_total_requests, user_triggered_short_circuited_requests) =
        user_triggered_agent_conflict_sample(objectives, non_human_traffic, verified_identity);

    let metrics = vec![
        if allow_capable {
            unit_target_metric(
                "allowed_as_intended_rate",
                total_requests,
                ratio(forwarded_requests, total_requests),
            )
        } else {
            not_applicable_metric("allowed_as_intended_rate")
        },
        if allow_capable {
            zero_budget_metric(
                "friction_mismatch_rate",
                total_requests,
                ratio(short_circuited_requests, total_requests),
            )
        } else {
            not_applicable_metric("friction_mismatch_rate")
        },
        zero_budget_metric("deny_mismatch_rate", total_requests, mismatch_ratio),
        coverage_metric(summary.verified_identity.attempts, coverage_ratio),
        zero_budget_metric_with_min_sample(
            "taxonomy_alignment_mismatch_rate",
            alignment_sample_size,
            alignment_mismatch_ratio,
            MIN_VERIFIED_CONFLICT_SAMPLE_SIZE,
        ),
        zero_budget_metric_with_min_sample(
            "verified_botness_conflict_rate",
            protected_total_requests,
            ratio(protected_short_circuited_requests, protected_total_requests),
            MIN_VERIFIED_CONFLICT_SAMPLE_SIZE,
        ),
        zero_budget_metric_with_min_sample(
            "user_triggered_agent_friction_mismatch_rate",
            user_triggered_total_requests,
            ratio(
                user_triggered_short_circuited_requests,
                user_triggered_total_requests,
            ),
            MIN_VERIFIED_CONFLICT_SAMPLE_SIZE,
        ),
    ];

    BenchmarkFamilyResult {
        family_id: "beneficial_non_human_posture".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: "partially_supported".to_string(),
        note: note_for_stance(
            policy_row,
            verified_identity,
        ),
        baseline_status: None,
        comparison_status: "not_available".to_string(),
        metrics,
    }
}

fn note_for_stance(
    policy_row: Option<&RequestOutcomeBreakdownSummaryRow>,
    verified_identity: &OperatorSnapshotVerifiedIdentitySummary,
) -> String {
    let observed = policy_row.map(|row| row.total_requests).unwrap_or(0);
    format!(
        "Bounded verified-identity posture currently compares {} observed verified-identity policy decisions against the resolved effective non-human policy `{}` with verified override mode `{}` while {} alignment receipts calibrate verified categories against the canonical taxonomy.",
        observed,
        verified_identity.effective_non_human_policy.profile_id,
        verified_identity
            .effective_non_human_policy
            .verified_identity_override_mode,
        verified_identity.taxonomy_alignment.receipts.len()
    )
}

fn unit_target_metric(metric_id: &str, sample_size: u64, current: f64) -> BenchmarkMetricResult {
    let status = if sample_size == 0 {
        "insufficient_evidence"
    } else if (1.0 - current).abs() <= f64::EPSILON {
        "inside_budget"
    } else if current >= 0.75 {
        "near_limit"
    } else {
        "outside_budget"
    };
    BenchmarkMetricResult {
        metric_id: metric_id.to_string(),
        status: status.to_string(),
        current: if sample_size == 0 { None } else { Some(current) },
        target: if sample_size == 0 { None } else { Some(1.0) },
        delta: if sample_size == 0 {
            None
        } else {
            Some(current - 1.0)
        },
        exactness: "derived".to_string(),
        basis: "mixed".to_string(),
        capability_gate: "supported".to_string(),
        baseline_current: None,
        comparison_delta: None,
        comparison_status: "not_available".to_string(),
    }
}

fn zero_budget_metric(metric_id: &str, sample_size: u64, current: f64) -> BenchmarkMetricResult {
    let status = if sample_size == 0 {
        "insufficient_evidence"
    } else if current <= 0.0 {
        "inside_budget"
    } else if current <= 0.25 {
        "near_limit"
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
        basis: "mixed".to_string(),
        capability_gate: "supported".to_string(),
        baseline_current: None,
        comparison_delta: None,
        comparison_status: "not_available".to_string(),
    }
}

fn zero_budget_metric_with_min_sample(
    metric_id: &str,
    sample_size: u64,
    current: f64,
    min_sample_size: u64,
) -> BenchmarkMetricResult {
    if sample_size < min_sample_size {
        return BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "insufficient_evidence".to_string(),
            current: None,
            target: None,
            delta: None,
            exactness: "derived".to_string(),
            basis: "mixed".to_string(),
            capability_gate: "supported".to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        };
    }
    zero_budget_metric(metric_id, sample_size, current)
}

fn coverage_metric(sample_size: u64, current: f64) -> BenchmarkMetricResult {
    BenchmarkMetricResult {
        metric_id: "coverage_status".to_string(),
        status: if sample_size == 0 {
            "insufficient_evidence".to_string()
        } else {
            "tracking_only".to_string()
        },
        current: if sample_size == 0 { None } else { Some(current) },
        target: None,
        delta: None,
        exactness: "derived".to_string(),
        basis: "verified".to_string(),
        capability_gate: "supported".to_string(),
        baseline_current: None,
        comparison_delta: None,
        comparison_status: "not_available".to_string(),
    }
}

fn not_applicable_metric(metric_id: &str) -> BenchmarkMetricResult {
    BenchmarkMetricResult {
        metric_id: metric_id.to_string(),
        status: "not_applicable".to_string(),
        current: None,
        target: None,
        delta: None,
        exactness: "derived".to_string(),
        basis: "mixed".to_string(),
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

fn protected_verified_conflict_sample(
    objectives: &OperatorObjectivesProfile,
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
    verified_identity: &OperatorSnapshotVerifiedIdentitySummary,
) -> (u64, u64) {
    let mut total_requests = 0u64;
    let mut short_circuited_requests = 0u64;
    let protected_categories = verified_identity
        .taxonomy_alignment
        .receipts
        .iter()
        .filter(|receipt| {
            matches!(receipt.alignment_status.as_str(), "aligned" | "fallback")
                && matches!(
                    category_posture(objectives, receipt.projected_category_id.as_str()),
                    Some("allowed" | "tolerated")
                )
        })
        .map(|receipt| receipt.projected_category_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    for category_id in protected_categories {
        for receipt in non_human_traffic.receipts.iter().filter(|receipt| {
            receipt.traffic_origin == "live"
                && receipt.category_id == category_id
                && receipt.assignment_status == "classified"
                && receipt.degradation_status == "current"
        }) {
            total_requests = total_requests.saturating_add(receipt.total_requests);
            short_circuited_requests = short_circuited_requests
                .saturating_add(receipt.short_circuited_requests);
        }
    }

    (total_requests, short_circuited_requests)
}

fn user_triggered_agent_conflict_sample(
    objectives: &OperatorObjectivesProfile,
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
    verified_identity: &OperatorSnapshotVerifiedIdentitySummary,
) -> (u64, u64) {
    if !verified_identity.taxonomy_alignment.receipts.iter().any(|receipt| {
        receipt.verified_identity_category == "user_triggered_agent"
            && matches!(receipt.alignment_status.as_str(), "aligned" | "fallback")
    }) {
        return (0, 0);
    }
    if !matches!(
        category_posture(objectives, "agent_on_behalf_of_human"),
        Some("allowed" | "tolerated")
    ) {
        return (0, 0);
    }

    let mut total_requests = 0u64;
    let mut short_circuited_requests = 0u64;
    for receipt in non_human_traffic.receipts.iter().filter(|receipt| {
        receipt.traffic_origin == "live"
            && receipt.category_id == "agent_on_behalf_of_human"
            && receipt.assignment_status == "classified"
            && receipt.degradation_status == "current"
    }) {
        total_requests = total_requests.saturating_add(receipt.total_requests);
        short_circuited_requests =
            short_circuited_requests.saturating_add(receipt.short_circuited_requests);
    }
    (total_requests, short_circuited_requests)
}

fn category_posture<'a>(
    objectives: &'a OperatorObjectivesProfile,
    category_id: &str,
) -> Option<&'a str> {
    objectives
        .category_postures
        .iter()
        .find(|row| row.category_id.as_str() == category_id)
        .map(|row| row.posture.as_str())
}

#[cfg(test)]
mod tests {
    use super::beneficial_non_human_posture_family;
    use crate::config::defaults;
    use crate::observability::monitoring::{
        MonitoringSummary, RequestOutcomeBreakdownSummaryRow,
    };
    use crate::observability::non_human_classification::{
        NonHumanClassificationReadiness, NonHumanClassificationReceipt,
        VerifiedIdentityTaxonomyAlignmentReceipt, VerifiedIdentityTaxonomyAlignmentSummary,
    };
    use crate::observability::non_human_coverage::NonHumanCoverageSummary;
    use crate::observability::operator_snapshot::{
        OperatorSnapshotNonHumanTrafficSummary, OperatorSnapshotVerifiedIdentitySummary,
    };
    use crate::observability::operator_snapshot_objectives::default_operator_objectives;

    fn sample_non_human_summary() -> OperatorSnapshotNonHumanTrafficSummary {
        OperatorSnapshotNonHumanTrafficSummary {
            availability: "taxonomy_seeded".to_string(),
            taxonomy: crate::runtime::non_human_taxonomy::canonical_non_human_taxonomy(),
            readiness: NonHumanClassificationReadiness {
                status: "ready".to_string(),
                blockers: Vec::new(),
                live_receipt_count: 1,
                adversary_sim_receipt_count: 1,
            },
            coverage: NonHumanCoverageSummary {
                schema_version: "non_human_coverage_v1".to_string(),
                overall_status: "covered".to_string(),
                blocking_reasons: Vec::new(),
                blocking_category_ids: Vec::new(),
                mapped_category_count: 6,
                gap_category_count: 2,
                covered_category_count: 6,
                partial_category_count: 0,
                stale_category_count: 0,
                unavailable_category_count: 0,
                uncovered_category_count: 2,
                receipts: Vec::new(),
            },
            decision_chain: vec![],
            receipts: vec![NonHumanClassificationReceipt {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                lane: "category_crosswalk".to_string(),
                category_id: "agent_on_behalf_of_human".to_string(),
                category_label: "Agent On Behalf Of Human".to_string(),
                assignment_status: "classified".to_string(),
                exactness: "exact".to_string(),
                basis: "observed".to_string(),
                degradation_status: "current".to_string(),
                total_requests: 6,
                forwarded_requests: 2,
                short_circuited_requests: 4,
                evidence_references: vec![],
            }],
        }
    }

    fn sample_verified_identity_summary() -> OperatorSnapshotVerifiedIdentitySummary {
        OperatorSnapshotVerifiedIdentitySummary {
            availability: "supported".to_string(),
            enabled: true,
            native_web_bot_auth_enabled: true,
            provider_assertions_enabled: true,
            effective_non_human_policy:
                crate::runtime::non_human_policy::effective_non_human_policy_summary(
                    &crate::observability::operator_snapshot_objectives::humans_plus_verified_only_operator_objectives(
                        1_700_000_000,
                    ),
                ),
            named_policy_count: 0,
            service_profile_count: 0,
            attempts: 6,
            verified: 6,
            failed: 0,
            unique_verified_identities: 1,
            top_failure_reasons: Vec::new(),
            top_schemes: Vec::new(),
            top_categories: Vec::new(),
            top_provenance: Vec::new(),
            taxonomy_alignment: VerifiedIdentityTaxonomyAlignmentSummary {
                schema_version: "verified_identity_taxonomy_alignment_v1".to_string(),
                status: "aligned".to_string(),
                aligned_count: 6,
                fallback_count: 0,
                misaligned_count: 0,
                insufficient_evidence_count: 0,
                receipts: vec![VerifiedIdentityTaxonomyAlignmentReceipt {
                    operator: "openai".to_string(),
                    stable_identity: "chatgpt-agent".to_string(),
                    scheme: "provider_signed_agent".to_string(),
                    verified_identity_category: "user_triggered_agent".to_string(),
                    projected_category_id: "agent_on_behalf_of_human".to_string(),
                    projected_category_label: "Agent On Behalf Of Human".to_string(),
                    alignment_status: "aligned".to_string(),
                    degradation_reason: "".to_string(),
                    count: 6,
                    end_user_controlled: true,
                    evidence_references: vec![],
                }],
            },
            policy_tranche: crate::observability::operator_snapshot_verified_identity::OperatorSnapshotVerifiedIdentityPolicySummary {
                total_requests: 6,
                forwarded_requests: 2,
                short_circuited_requests: 4,
            },
        }
    }

    #[test]
    fn beneficial_family_surfaces_verified_conflict_metrics_for_tolerated_agents() {
        let mut cfg = defaults().clone();
        cfg.verified_identity.enabled = true;
        let mut objectives = default_operator_objectives(1_700_000_000);
        objectives
            .category_postures
            .iter_mut()
            .find(|row| row.category_id.as_str() == "agent_on_behalf_of_human")
            .expect("agent-on-behalf-of-human posture")
            .posture = "tolerated".to_string();
        let mut monitoring = MonitoringSummary::default();
        monitoring.request_outcomes.by_policy_source.push(RequestOutcomeBreakdownSummaryRow {
            traffic_origin: "live".to_string(),
            measurement_scope: "ingress_primary".to_string(),
            execution_mode: "enforced".to_string(),
            value: "policy_graph_verified_identity_tranche".to_string(),
            total_requests: 6,
            forwarded_requests: 2,
            short_circuited_requests: 4,
            control_response_requests: 0,
        });
        monitoring.verified_identity.attempts = 6;
        monitoring.verified_identity.verified = 6;

        let family = beneficial_non_human_posture_family(
            &monitoring,
            &cfg,
            &objectives,
            &sample_non_human_summary(),
            &sample_verified_identity_summary(),
        );

        assert_eq!(family.family_id, "beneficial_non_human_posture");
        assert_eq!(
            family
                .metrics
                .iter()
                .find(|metric| metric.metric_id == "verified_botness_conflict_rate")
                .expect("verified botness conflict metric")
                .status,
            "outside_budget"
        );
        assert_eq!(
            family
                .metrics
                .iter()
                .find(|metric| metric.metric_id == "user_triggered_agent_friction_mismatch_rate")
                .expect("user-triggered mismatch metric")
                .status,
            "outside_budget"
        );
    }
}
