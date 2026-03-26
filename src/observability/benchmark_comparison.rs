use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::benchmark_results::{
    BenchmarkBaselineReference, BenchmarkExploitLocus, BenchmarkFamilyResult,
    BenchmarkMetricResult,
    BenchmarkResultsPayload,
};
use super::benchmark_results_comparison::{
    unavailable_improvement_status, unavailable_reference_for,
};
use super::operator_snapshot::OperatorSnapshotWindow;

const COMPARISON_EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkComparableMetric {
    pub metric_id: String,
    pub status: String,
    pub current: Option<f64>,
    pub capability_gate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkComparableFamily {
    pub family_id: String,
    pub status: String,
    pub capability_gate: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exploit_loci: Vec<BenchmarkExploitLocus>,
    pub metrics: Vec<BenchmarkComparableMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkComparableSnapshot {
    pub generated_at: u64,
    pub subject_kind: String,
    pub watch_window: OperatorSnapshotWindow,
    pub coverage_status: String,
    pub overall_status: String,
    pub families: Vec<BenchmarkComparableFamily>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkEpisodeMetricDelta {
    pub metric_id: String,
    pub status: String,
    pub current: Option<f64>,
    pub target: Option<f64>,
    pub delta: Option<f64>,
    pub comparison_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkEpisodeFamilyDelta {
    pub family_id: String,
    pub status: String,
    pub comparison_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exploit_loci: Vec<BenchmarkExploitLocus>,
    pub metric_deltas: Vec<BenchmarkEpisodeMetricDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkCompletedCycleJudgment {
    pub episode_id: String,
    pub judgment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkHomeostasisSummary {
    pub minimum_completed_cycles_for_homeostasis: u64,
    pub judged_cycle_count: usize,
    pub considered_episode_ids: Vec<String>,
    pub status: String,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricDirection {
    LowerIsBetter,
    HigherIsBetter,
}

pub(crate) fn comparable_snapshot_from_results(
    payload: &BenchmarkResultsPayload,
) -> BenchmarkComparableSnapshot {
    BenchmarkComparableSnapshot {
        generated_at: payload.generated_at,
        subject_kind: payload.subject_kind.clone(),
        watch_window: payload.watch_window.clone(),
        coverage_status: payload.coverage_status.clone(),
        overall_status: payload.overall_status.clone(),
        families: payload
            .families
            .iter()
            .map(|family| BenchmarkComparableFamily {
                family_id: family.family_id.clone(),
                status: family.status.clone(),
                capability_gate: family.capability_gate.clone(),
                exploit_loci: family.exploit_loci.clone(),
                metrics: family
                    .metrics
                    .iter()
                    .map(|metric| BenchmarkComparableMetric {
                        metric_id: metric.metric_id.clone(),
                        status: metric.status.clone(),
                        current: metric.current,
                        capability_gate: metric.capability_gate.clone(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

pub(crate) fn benchmark_episode_delta_summary(
    payload: &BenchmarkResultsPayload,
) -> Vec<BenchmarkEpisodeFamilyDelta> {
    payload
        .families
        .iter()
        .map(|family| BenchmarkEpisodeFamilyDelta {
            family_id: family.family_id.clone(),
            status: family.status.clone(),
            comparison_status: family.comparison_status.clone(),
            exploit_loci: family.exploit_loci.clone(),
            metric_deltas: family
                .metrics
                .iter()
                .map(|metric| BenchmarkEpisodeMetricDelta {
                    metric_id: metric.metric_id.clone(),
                    status: metric.status.clone(),
                    current: metric.current,
                    target: metric.target,
                    delta: metric.delta,
                    comparison_status: metric.comparison_status.clone(),
                })
                .collect(),
        })
        .collect()
}

pub(crate) fn classify_homeostasis(
    judgments: &[BenchmarkCompletedCycleJudgment],
    minimum_completed_cycles_for_homeostasis: u64,
) -> BenchmarkHomeostasisSummary {
    let considered_episode_ids = judgments
        .iter()
        .map(|judgment| judgment.episode_id.clone())
        .collect::<Vec<_>>();
    let judged_cycle_count = judgments.len();
    if judged_cycle_count < minimum_completed_cycles_for_homeostasis as usize {
        return BenchmarkHomeostasisSummary {
            minimum_completed_cycles_for_homeostasis,
            judged_cycle_count,
            considered_episode_ids,
            status: "not_enough_completed_cycles".to_string(),
            note: "Homeostasis remains unset until enough completed watch-window judgments exist."
                .to_string(),
        };
    }

    let improved_count = judgments
        .iter()
        .filter(|judgment| judgment.judgment == "improved")
        .count();
    let all_flat_or_guardrail = judgments.iter().all(|judgment| {
        matches!(judgment.judgment.as_str(), "flat" | "guardrail_blocked")
    });

    let status = if improved_count == judged_cycle_count {
        "improving"
    } else if improved_count == 0 && all_flat_or_guardrail {
        "homeostasis"
    } else {
        "mixed"
    };

    BenchmarkHomeostasisSummary {
        minimum_completed_cycles_for_homeostasis,
        judged_cycle_count,
        considered_episode_ids,
        status: status.to_string(),
        note: "Homeostasis is classified conservatively from explicit completed-cycle judgments rather than ad hoc trend prose."
            .to_string(),
    }
}

pub(crate) fn apply_prior_window_comparison(
    current_generated_at: u64,
    families: &mut [BenchmarkFamilyResult],
    prior_window_reference: Option<&BenchmarkComparableSnapshot>,
) -> (BenchmarkBaselineReference, String) {
    apply_reference_comparison(
        current_generated_at,
        families,
        ComparisonReference {
            reference_kind: "prior_window",
            unavailable_note:
                "No prior-window benchmark subject is currently materialized for comparison.",
            available_note:
                "Compared against the most recently materialized prior-window benchmark subject.",
        },
        prior_window_reference,
    )
}

#[allow(dead_code)]
pub(crate) fn apply_candidate_comparison(
    current_generated_at: u64,
    families: &mut [BenchmarkFamilyResult],
    candidate_reference: Option<&BenchmarkComparableSnapshot>,
) -> (BenchmarkBaselineReference, String) {
    apply_reference_comparison(
        current_generated_at,
        families,
        ComparisonReference {
            reference_kind: "candidate",
            unavailable_note:
                "No candidate benchmark subject is currently materialized for comparison.",
            available_note:
                "Compared against the currently materialized candidate benchmark subject.",
        },
        candidate_reference,
    )
}

#[derive(Debug, Clone, Copy)]
struct ComparisonReference<'a> {
    reference_kind: &'a str,
    unavailable_note: &'a str,
    available_note: &'a str,
}

fn apply_reference_comparison(
    current_generated_at: u64,
    families: &mut [BenchmarkFamilyResult],
    reference_metadata: ComparisonReference<'_>,
    reference: Option<&BenchmarkComparableSnapshot>,
) -> (BenchmarkBaselineReference, String) {
    let Some(reference) = reference.filter(|reference| reference.generated_at < current_generated_at)
    else {
        return (
            unavailable_reference_for(
                reference_metadata.reference_kind,
                reference_metadata.unavailable_note,
            ),
            unavailable_improvement_status(),
        );
    };

    for family in families.iter_mut() {
        apply_family_comparison(family, reference);
    }

    let improvement_status = aggregate_comparison_status(
        families
            .iter()
            .map(|family| family.comparison_status.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    );

    (
        BenchmarkBaselineReference {
            reference_kind: reference_metadata.reference_kind.to_string(),
            status: "available".to_string(),
            subject_kind: Some(reference.subject_kind.clone()),
            generated_at: Some(reference.generated_at),
            note: reference_metadata.available_note.to_string(),
        },
        improvement_status,
    )
}

fn apply_family_comparison(
    family: &mut BenchmarkFamilyResult,
    reference: &BenchmarkComparableSnapshot,
) {
    let Some(reference_family) = reference
        .families
        .iter()
        .find(|candidate| candidate.family_id == family.family_id)
    else {
        family.baseline_status = None;
        family.comparison_status = "not_available".to_string();
        for metric in family.metrics.iter_mut() {
            metric.baseline_current = None;
            metric.comparison_delta = None;
            metric.comparison_status = "not_available".to_string();
        }
        return;
    };

    family.baseline_status = Some(reference_family.status.clone());
    let mut metric_statuses = Vec::new();
    for metric in family.metrics.iter_mut() {
        let status = apply_metric_comparison(metric, reference_family);
        metric_statuses.push(status);
    }

    let family_status_comparison = compare_status_value(family.status.as_str(), reference_family.status.as_str());
    let loci_comparison =
        compare_exploit_loci(family.exploit_loci.as_slice(), reference_family.exploit_loci.as_slice());
    family.comparison_status = if family_status_comparison == "neutral" {
        let mut statuses = metric_statuses;
        statuses.push(loci_comparison);
        aggregate_comparison_status(
            statuses
                .iter()
                .map(|status| status.as_str())
                .collect::<Vec<_>>()
                .as_slice(),
        )
    } else {
        family_status_comparison.to_string()
    };
}

fn apply_metric_comparison(
    metric: &mut BenchmarkMetricResult,
    reference_family: &BenchmarkComparableFamily,
) -> String {
    let Some(reference_metric) = reference_family
        .metrics
        .iter()
        .find(|candidate| candidate.metric_id == metric.metric_id)
    else {
        metric.baseline_current = None;
        metric.comparison_delta = None;
        metric.comparison_status = "not_available".to_string();
        return "not_available".to_string();
    };

    metric.baseline_current = reference_metric.current;
    metric.comparison_delta = match (metric.current, reference_metric.current) {
        (Some(current), Some(baseline)) => Some(current - baseline),
        _ => None,
    };
    let comparison_status = compare_metric(metric, reference_metric);
    metric.comparison_status = comparison_status.clone();
    comparison_status
}

fn compare_metric(
    metric: &BenchmarkMetricResult,
    reference_metric: &BenchmarkComparableMetric,
) -> String {
    if matches!(metric.status.as_str(), "not_yet_supported" | "not_applicable") {
        return metric.status.clone();
    }
    if matches!(
        reference_metric.status.as_str(),
        "not_yet_supported" | "not_applicable"
    ) {
        return "not_available".to_string();
    }

    if let (Some(direction), Some(current), Some(reference)) = (
        metric_direction(metric.metric_id.as_str()),
        metric.current,
        reference_metric.current,
    ) {
        if (current - reference).abs() <= COMPARISON_EPSILON {
            return "neutral".to_string();
        }
        return match direction {
            MetricDirection::LowerIsBetter if current < reference => "improved".to_string(),
            MetricDirection::LowerIsBetter => "regressed".to_string(),
            MetricDirection::HigherIsBetter if current > reference => "improved".to_string(),
            MetricDirection::HigherIsBetter => "regressed".to_string(),
        };
    }

    compare_status_value(metric.status.as_str(), reference_metric.status.as_str()).to_string()
}

fn aggregate_comparison_status(statuses: &[&str]) -> String {
    let has_improved = statuses.iter().any(|status| *status == "improved");
    let has_regressed = statuses.iter().any(|status| *status == "regressed");

    if has_improved && has_regressed {
        return "mixed".to_string();
    }
    if has_regressed {
        return "regressed".to_string();
    }
    if has_improved {
        return "improved".to_string();
    }
    if statuses
        .iter()
        .any(|status| *status == "insufficient_evidence")
    {
        return "insufficient_evidence".to_string();
    }
    if statuses.iter().any(|status| *status == "neutral") {
        return "neutral".to_string();
    }
    if statuses.iter().all(|status| *status == "not_available") {
        return "not_available".to_string();
    }
    if statuses.iter().all(|status| *status == "not_applicable") {
        return "not_applicable".to_string();
    }
    if statuses
        .iter()
        .any(|status| *status == "tracking_only")
    {
        return "neutral".to_string();
    }
    "neutral".to_string()
}

fn compare_status_value(current: &str, reference: &str) -> &'static str {
    match (status_rank(current), status_rank(reference)) {
        (Some(current_rank), Some(reference_rank)) if current_rank > reference_rank => "improved",
        (Some(current_rank), Some(reference_rank)) if current_rank < reference_rank => "regressed",
        (Some(_), Some(_)) => "neutral",
        _ if current == "insufficient_evidence" || reference == "insufficient_evidence" => {
            "insufficient_evidence"
        }
        _ if current == "tracking_only" && reference == "tracking_only" => "neutral",
        _ => "not_available",
    }
}

fn compare_exploit_loci(
    current: &[BenchmarkExploitLocus],
    reference: &[BenchmarkExploitLocus],
) -> String {
    let current_ids: BTreeSet<_> = current.iter().map(|locus| locus.locus_id.as_str()).collect();
    let reference_ids: BTreeSet<_> = reference
        .iter()
        .map(|locus| locus.locus_id.as_str())
        .collect();
    let added = current_ids.difference(&reference_ids).next().is_some();
    let removed = reference_ids.difference(&current_ids).next().is_some();

    if added && removed {
        "mixed".to_string()
    } else if added {
        "regressed".to_string()
    } else if removed {
        "improved".to_string()
    } else {
        "neutral".to_string()
    }
}

fn status_rank(status: &str) -> Option<i8> {
    match status {
        "inside_budget" => Some(3),
        "near_limit" => Some(2),
        "outside_budget" => Some(1),
        "tracking_only" => Some(0),
        "insufficient_evidence" => Some(-1),
        _ => None,
    }
}

fn metric_direction(metric_id: &str) -> Option<MetricDirection> {
    if metric_id.starts_with("category_posture_alignment:") {
        return Some(MetricDirection::HigherIsBetter);
    }

    match metric_id {
        "suspicious_forwarded_request_rate"
        | "suspicious_forwarded_byte_rate"
        | "suspicious_forwarded_latency_share"
        | "suspicious_average_forward_latency_ms"
        | "likely_human_friction_rate"
        | "interactive_friction_rate"
        | "likely_human_hard_block_rate"
        | "scenario_goal_success_rate"
        | "scenario_origin_reach_rate"
        | "scenario_regression_status"
        | "scrapling_breach_surface_rate"
        | "scrapling_deepest_breach_stage_ratio"
        | "scrapling_pass_surface_success_rate"
        | "friction_mismatch_rate"
        | "deny_mismatch_rate" => Some(MetricDirection::LowerIsBetter),
        "suspicious_short_circuit_rate"
        | "suspicious_locally_served_byte_share"
        | "scenario_escalation_rate"
        | "allowed_as_intended_rate"
        | "coverage_status" => Some(MetricDirection::HigherIsBetter),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_homeostasis, BenchmarkCompletedCycleJudgment};
    use super::{
        apply_candidate_comparison, apply_prior_window_comparison,
        comparable_snapshot_from_results, BenchmarkComparableSnapshot,
    };
    use crate::observability::benchmark_results::{
        BenchmarkExploitLocus, BenchmarkFamilyResult, BenchmarkMetricResult,
        BenchmarkResultsPayload,
    };
    use crate::observability::operator_snapshot::OperatorSnapshotWindow;

    fn metric(metric_id: &str, current: Option<f64>, status: &str) -> BenchmarkMetricResult {
        BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: status.to_string(),
            current,
            target: None,
            delta: None,
            exactness: "exact".to_string(),
            basis: "observed".to_string(),
            capability_gate: "supported".to_string(),
            baseline_current: None,
            comparison_delta: None,
            comparison_status: "not_available".to_string(),
        }
    }

    fn family(metric: BenchmarkMetricResult) -> BenchmarkFamilyResult {
        BenchmarkFamilyResult {
            family_id: "beneficial_non_human_posture".to_string(),
            status: metric.status.clone(),
            capability_gate: "supported".to_string(),
            note: "test".to_string(),
            baseline_status: None,
            comparison_status: "not_available".to_string(),
            exploit_loci: Vec::new(),
            metrics: vec![metric],
        }
    }

    #[test]
    fn prior_window_comparison_marks_improved_numeric_metrics() {
        let reference = BenchmarkComparableSnapshot {
            generated_at: 100,
            subject_kind: "current_instance".to_string(),
            watch_window: OperatorSnapshotWindow {
                start_ts: 1,
                end_ts: 100,
                duration_seconds: 100,
            },
            coverage_status: "supported".to_string(),
            overall_status: "outside_budget".to_string(),
            families: vec![super::BenchmarkComparableFamily {
                family_id: "beneficial_non_human_posture".to_string(),
                status: "outside_budget".to_string(),
                capability_gate: "supported".to_string(),
                exploit_loci: Vec::new(),
                metrics: vec![super::BenchmarkComparableMetric {
                    metric_id: "deny_mismatch_rate".to_string(),
                    status: "outside_budget".to_string(),
                    current: Some(1.0),
                    capability_gate: "supported".to_string(),
                }],
            }],
        };
        let mut families = vec![family(metric("deny_mismatch_rate", Some(0.0), "inside_budget"))];

        let (baseline, improvement) =
            apply_prior_window_comparison(200, families.as_mut_slice(), Some(&reference));

        assert_eq!(baseline.status, "available");
        assert_eq!(improvement, "improved");
        assert_eq!(families[0].comparison_status, "improved");
        assert_eq!(families[0].metrics[0].baseline_current, Some(1.0));
        assert_eq!(families[0].metrics[0].comparison_status, "improved");
    }

    #[test]
    fn prior_window_comparison_marks_host_impact_metrics_as_lower_is_better() {
        let reference = BenchmarkComparableSnapshot {
            generated_at: 100,
            subject_kind: "current_instance".to_string(),
            watch_window: OperatorSnapshotWindow {
                start_ts: 1,
                end_ts: 100,
                duration_seconds: 100,
            },
            coverage_status: "supported".to_string(),
            overall_status: "outside_budget".to_string(),
            families: vec![super::BenchmarkComparableFamily {
                family_id: "suspicious_origin_cost".to_string(),
                status: "outside_budget".to_string(),
                capability_gate: "supported".to_string(),
                exploit_loci: Vec::new(),
                metrics: vec![
                    super::BenchmarkComparableMetric {
                        metric_id: "suspicious_forwarded_latency_share".to_string(),
                        status: "outside_budget".to_string(),
                        current: Some(0.7),
                        capability_gate: "supported".to_string(),
                    },
                    super::BenchmarkComparableMetric {
                        metric_id: "suspicious_average_forward_latency_ms".to_string(),
                        status: "tracking_only".to_string(),
                        current: Some(140.0),
                        capability_gate: "supported".to_string(),
                    },
                ],
            }],
        };
        let mut families = vec![super::BenchmarkFamilyResult {
            family_id: "suspicious_origin_cost".to_string(),
            status: "inside_budget".to_string(),
            capability_gate: "supported".to_string(),
            note: "test".to_string(),
            baseline_status: None,
            comparison_status: "not_available".to_string(),
            exploit_loci: Vec::new(),
            metrics: vec![
                metric(
                    "suspicious_forwarded_latency_share",
                    Some(0.2),
                    "inside_budget",
                ),
                BenchmarkMetricResult {
                    metric_id: "suspicious_average_forward_latency_ms".to_string(),
                    status: "tracking_only".to_string(),
                    current: Some(90.0),
                    target: None,
                    delta: None,
                    exactness: "exact".to_string(),
                    basis: "observed".to_string(),
                    capability_gate: "supported".to_string(),
                    baseline_current: None,
                    comparison_delta: None,
                    comparison_status: "not_available".to_string(),
                },
            ],
        }];

        let (_, improvement) =
            apply_prior_window_comparison(200, families.as_mut_slice(), Some(&reference));

        assert_eq!(improvement, "improved");
        assert_eq!(families[0].comparison_status, "improved");
        assert_eq!(families[0].metrics[0].comparison_status, "improved");
        assert_eq!(families[0].metrics[1].comparison_status, "improved");
    }

    #[test]
    fn comparable_snapshot_reduces_results_payload_to_bounded_reference() {
        let payload = BenchmarkResultsPayload {
            schema_version: "benchmark_results_v1".to_string(),
            suite_version: "benchmark_suite_v1".to_string(),
            generated_at: 200,
            input_snapshot_generated_at: 200,
            subject_kind: "current_instance".to_string(),
            watch_window: OperatorSnapshotWindow {
                start_ts: 101,
                end_ts: 200,
                duration_seconds: 100,
            },
            baseline_reference: crate::observability::benchmark_results::BenchmarkBaselineReference {
                reference_kind: "prior_window".to_string(),
                status: "not_available".to_string(),
                subject_kind: None,
                generated_at: None,
                note: "none".to_string(),
            },
            coverage_status: "supported".to_string(),
            overall_status: "inside_budget".to_string(),
            improvement_status: "not_available".to_string(),
            non_human_classification:
                crate::observability::non_human_classification::NonHumanClassificationReadiness {
                    status: "ready".to_string(),
                    blockers: Vec::new(),
                    live_receipt_count: 1,
                    adversary_sim_receipt_count: 1,
                },
            non_human_coverage: crate::observability::non_human_coverage::NonHumanCoverageSummary {
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
            tuning_eligibility: crate::observability::benchmark_results::BenchmarkTuningEligibility {
                status: "eligible".to_string(),
                blockers: Vec::new(),
            },
            families: vec![family(metric("allowed_as_intended_rate", Some(1.0), "inside_budget"))],
            escalation_hint: crate::observability::benchmark_results::BenchmarkEscalationHint {
                availability: "supported".to_string(),
                decision: "observe_longer".to_string(),
                review_status: "manual_review_required".to_string(),
                problem_class: "no_escalation_required".to_string(),
                guidance_status: "observe_longer".to_string(),
                tractability: "not_actionable_yet".to_string(),
                expected_direction: "continue_observing".to_string(),
                trigger_family_ids: Vec::new(),
                trigger_metric_ids: Vec::new(),
                candidate_action_families: Vec::new(),
                family_guidance: Vec::new(),
                blockers: Vec::new(),
                note: "test".to_string(),
            },
            replay_promotion:
                crate::observability::replay_promotion::ReplayPromotionSummary::not_materialized(),
        };

        let comparable = comparable_snapshot_from_results(&payload);

        assert_eq!(comparable.generated_at, 200);
        assert_eq!(comparable.subject_kind, "current_instance");
        assert_eq!(comparable.families.len(), 1);
        assert_eq!(comparable.families[0].metrics[0].metric_id, "allowed_as_intended_rate");
        assert!(comparable.families[0].exploit_loci.is_empty());
    }

    #[test]
    fn candidate_comparison_reuses_same_delta_semantics() {
        let candidate_reference = BenchmarkComparableSnapshot {
            generated_at: 250,
            subject_kind: "candidate_config".to_string(),
            watch_window: OperatorSnapshotWindow {
                start_ts: 151,
                end_ts: 250,
                duration_seconds: 100,
            },
            coverage_status: "supported".to_string(),
            overall_status: "inside_budget".to_string(),
            families: vec![super::BenchmarkComparableFamily {
                family_id: "beneficial_non_human_posture".to_string(),
                status: "inside_budget".to_string(),
                capability_gate: "supported".to_string(),
                exploit_loci: Vec::new(),
                metrics: vec![super::BenchmarkComparableMetric {
                    metric_id: "allowed_as_intended_rate".to_string(),
                    status: "inside_budget".to_string(),
                    current: Some(0.9),
                    capability_gate: "supported".to_string(),
                }],
            }],
        };
        let mut families = vec![family(metric(
            "allowed_as_intended_rate",
            Some(1.0),
            "inside_budget",
        ))];

        let (baseline, improvement) =
            apply_candidate_comparison(300, families.as_mut_slice(), Some(&candidate_reference));

        assert_eq!(baseline.reference_kind, "candidate");
        assert_eq!(baseline.subject_kind.as_deref(), Some("candidate_config"));
        assert_eq!(improvement, "improved");
        assert_eq!(families[0].metrics[0].baseline_current, Some(0.9));
        assert_eq!(families[0].metrics[0].comparison_status, "improved");
    }

    #[test]
    fn prior_window_comparison_marks_new_exploit_loci_as_regressed_even_when_metrics_are_flat() {
        let reference = BenchmarkComparableSnapshot {
            generated_at: 100,
            subject_kind: "current_instance".to_string(),
            watch_window: OperatorSnapshotWindow {
                start_ts: 1,
                end_ts: 100,
                duration_seconds: 100,
            },
            coverage_status: "supported".to_string(),
            overall_status: "outside_budget".to_string(),
            families: vec![super::BenchmarkComparableFamily {
                family_id: "scrapling_exploit_progress".to_string(),
                status: "outside_budget".to_string(),
                capability_gate: "supported".to_string(),
                exploit_loci: vec![BenchmarkExploitLocus {
                    locus_id: "public_path_traversal".to_string(),
                    locus_label: "Public Path Traversal".to_string(),
                    stage_id: "exposure".to_string(),
                    evidence_status: "progress_observed".to_string(),
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/landing".to_string(),
                    sample_response_status: Some(200),
                }],
                metrics: vec![super::BenchmarkComparableMetric {
                    metric_id: "scrapling_breach_surface_rate".to_string(),
                    status: "outside_budget".to_string(),
                    current: Some(0.5),
                    capability_gate: "supported".to_string(),
                }],
            }],
        };
        let mut families = vec![BenchmarkFamilyResult {
            family_id: "scrapling_exploit_progress".to_string(),
            status: "outside_budget".to_string(),
            capability_gate: "supported".to_string(),
            note: "test".to_string(),
            baseline_status: None,
            comparison_status: "not_available".to_string(),
            exploit_loci: vec![
                BenchmarkExploitLocus {
                    locus_id: "public_path_traversal".to_string(),
                    locus_label: "Public Path Traversal".to_string(),
                    stage_id: "exposure".to_string(),
                    evidence_status: "progress_observed".to_string(),
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/landing".to_string(),
                    sample_response_status: Some(200),
                },
                BenchmarkExploitLocus {
                    locus_id: "maze_navigation".to_string(),
                    locus_label: "Maze Navigation".to_string(),
                    stage_id: "interactive".to_string(),
                    evidence_status: "progress_observed".to_string(),
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/maze".to_string(),
                    sample_response_status: Some(200),
                },
            ],
            metrics: vec![metric(
                "scrapling_breach_surface_rate",
                Some(0.5),
                "outside_budget",
            )],
        }];

        let (_, improvement) =
            apply_prior_window_comparison(200, families.as_mut_slice(), Some(&reference));

        assert_eq!(improvement, "regressed");
        assert_eq!(families[0].comparison_status, "regressed");
    }

    #[test]
    fn homeostasis_requires_ten_completed_cycle_judgments_before_classifying() {
        let summary = classify_homeostasis(
            &vec![
                BenchmarkCompletedCycleJudgment {
                    episode_id: "episode-1".to_string(),
                    judgment: "improved".to_string(),
                };
                3
            ],
            10,
        );

        assert_eq!(summary.status, "not_enough_completed_cycles");
        assert_eq!(summary.judged_cycle_count, 3);
        assert_eq!(summary.minimum_completed_cycles_for_homeostasis, 10);
    }

    #[test]
    fn homeostasis_distinguishes_improving_mixed_and_flat_recent_cycles() {
        let improving = classify_homeostasis(
            &(0..10)
                .map(|idx| BenchmarkCompletedCycleJudgment {
                    episode_id: format!("improving-{idx}"),
                    judgment: "improved".to_string(),
                })
                .collect::<Vec<_>>(),
            10,
        );
        let mixed = classify_homeostasis(
            &(0..10)
                .map(|idx| BenchmarkCompletedCycleJudgment {
                    episode_id: format!("mixed-{idx}"),
                    judgment: if idx % 2 == 0 {
                        "improved".to_string()
                    } else {
                        "regressed".to_string()
                    },
                })
                .collect::<Vec<_>>(),
            10,
        );
        let flat = classify_homeostasis(
            &(0..10)
                .map(|idx| BenchmarkCompletedCycleJudgment {
                    episode_id: format!("flat-{idx}"),
                    judgment: "flat".to_string(),
                })
                .collect::<Vec<_>>(),
            10,
        );

        assert_eq!(improving.status, "improving");
        assert_eq!(mixed.status, "mixed");
        assert_eq!(flat.status, "homeostasis");
        assert_eq!(flat.considered_episode_ids.len(), 10);
    }
}
