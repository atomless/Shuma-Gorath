use serde::{Deserialize, Serialize};

use super::benchmark_results::{
    BenchmarkBaselineReference, BenchmarkFamilyResult, BenchmarkMetricResult,
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
pub(crate) struct BenchmarkComparableMetricDelta {
    pub metric_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_current: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_current: Option<f64>,
    pub comparison_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkComparableFamilyDelta {
    pub family_id: String,
    pub baseline_status: String,
    pub candidate_status: String,
    pub comparison_status: String,
    pub metrics: Vec<BenchmarkComparableMetricDelta>,
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

pub(crate) fn compare_candidate_snapshot(
    candidate: &BenchmarkComparableSnapshot,
    baseline: &BenchmarkComparableSnapshot,
) -> Vec<BenchmarkComparableFamilyDelta> {
    candidate
        .families
        .iter()
        .map(|candidate_family| {
            let Some(baseline_family) = baseline
                .families
                .iter()
                .find(|family| family.family_id == candidate_family.family_id)
            else {
                return BenchmarkComparableFamilyDelta {
                    family_id: candidate_family.family_id.clone(),
                    baseline_status: "not_available".to_string(),
                    candidate_status: candidate_family.status.clone(),
                    comparison_status: "not_available".to_string(),
                    metrics: candidate_family
                        .metrics
                        .iter()
                        .map(|metric| BenchmarkComparableMetricDelta {
                            metric_id: metric.metric_id.clone(),
                            baseline_current: None,
                            candidate_current: metric.current,
                            comparison_status: "not_available".to_string(),
                        })
                        .collect(),
                };
            };

            let metrics = candidate_family
                .metrics
                .iter()
                .map(|candidate_metric| {
                    let comparison_status = baseline_family
                        .metrics
                        .iter()
                        .find(|metric| metric.metric_id == candidate_metric.metric_id)
                        .map(|baseline_metric| {
                            compare_comparable_metric(candidate_metric, baseline_metric)
                        })
                        .unwrap_or_else(|| "not_available".to_string());
                    let baseline_current = baseline_family
                        .metrics
                        .iter()
                        .find(|metric| metric.metric_id == candidate_metric.metric_id)
                        .and_then(|metric| metric.current);
                    BenchmarkComparableMetricDelta {
                        metric_id: candidate_metric.metric_id.clone(),
                        baseline_current,
                        candidate_current: candidate_metric.current,
                        comparison_status,
                    }
                })
                .collect::<Vec<_>>();
            let metric_statuses = metrics
                .iter()
                .map(|metric| metric.comparison_status.as_str())
                .collect::<Vec<_>>();
            let family_status_comparison =
                compare_status_value(candidate_family.status.as_str(), baseline_family.status.as_str());
            let comparison_status = if family_status_comparison == "neutral" {
                aggregate_comparison_status(metric_statuses.as_slice())
            } else {
                family_status_comparison.to_string()
            };

            BenchmarkComparableFamilyDelta {
                family_id: candidate_family.family_id.clone(),
                baseline_status: baseline_family.status.clone(),
                candidate_status: candidate_family.status.clone(),
                comparison_status,
                metrics,
            }
        })
        .collect()
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
    family.comparison_status = if family_status_comparison == "neutral" {
        aggregate_comparison_status(
            metric_statuses
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

fn compare_comparable_metric(
    candidate_metric: &BenchmarkComparableMetric,
    baseline_metric: &BenchmarkComparableMetric,
) -> String {
    if matches!(
        candidate_metric.status.as_str(),
        "not_yet_supported" | "not_applicable"
    ) {
        return candidate_metric.status.clone();
    }
    if matches!(
        baseline_metric.status.as_str(),
        "not_yet_supported" | "not_applicable"
    ) {
        return "not_available".to_string();
    }

    if let (Some(direction), Some(candidate), Some(baseline)) = (
        metric_direction(candidate_metric.metric_id.as_str()),
        candidate_metric.current,
        baseline_metric.current,
    ) {
        if (candidate - baseline).abs() <= COMPARISON_EPSILON {
            return "neutral".to_string();
        }
        return match direction {
            MetricDirection::LowerIsBetter if candidate < baseline => "improved".to_string(),
            MetricDirection::LowerIsBetter => "regressed".to_string(),
            MetricDirection::HigherIsBetter if candidate > baseline => "improved".to_string(),
            MetricDirection::HigherIsBetter => "regressed".to_string(),
        };
    }

    compare_status_value(candidate_metric.status.as_str(), baseline_metric.status.as_str())
        .to_string()
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
    use super::{
        apply_candidate_comparison, apply_prior_window_comparison, compare_candidate_snapshot,
        comparable_snapshot_from_results, BenchmarkComparableSnapshot,
    };
    use crate::observability::benchmark_results::{
        BenchmarkFamilyResult, BenchmarkMetricResult, BenchmarkResultsPayload,
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
                problem_class: "not_applicable".to_string(),
                guidance_status: "insufficient_evidence".to_string(),
                tractability: "observe_only".to_string(),
                trigger_family_ids: Vec::new(),
                trigger_metric_ids: Vec::new(),
                candidate_action_families: Vec::new(),
                recommended_action_family: None,
                blockers: Vec::new(),
                shortfall_guidance: Vec::new(),
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
    fn candidate_snapshot_delta_reports_family_and_metric_progress() {
        let baseline = BenchmarkComparableSnapshot {
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
                family_id: "likely_human_friction".to_string(),
                status: "outside_budget".to_string(),
                capability_gate: "supported".to_string(),
                metrics: vec![super::BenchmarkComparableMetric {
                    metric_id: "likely_human_friction_rate".to_string(),
                    status: "outside_budget".to_string(),
                    current: Some(0.20),
                    capability_gate: "supported".to_string(),
                }],
            }],
        };
        let candidate = BenchmarkComparableSnapshot {
            generated_at: 200,
            subject_kind: "current_instance".to_string(),
            watch_window: OperatorSnapshotWindow {
                start_ts: 101,
                end_ts: 200,
                duration_seconds: 100,
            },
            coverage_status: "supported".to_string(),
            overall_status: "inside_budget".to_string(),
            families: vec![super::BenchmarkComparableFamily {
                family_id: "likely_human_friction".to_string(),
                status: "inside_budget".to_string(),
                capability_gate: "supported".to_string(),
                metrics: vec![super::BenchmarkComparableMetric {
                    metric_id: "likely_human_friction_rate".to_string(),
                    status: "inside_budget".to_string(),
                    current: Some(0.02),
                    capability_gate: "supported".to_string(),
                }],
            }],
        };

        let deltas = compare_candidate_snapshot(&candidate, &baseline);

        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].family_id, "likely_human_friction");
        assert_eq!(deltas[0].comparison_status, "improved");
        assert_eq!(deltas[0].metrics[0].metric_id, "likely_human_friction_rate");
        assert_eq!(deltas[0].metrics[0].comparison_status, "improved");
        assert_eq!(deltas[0].metrics[0].baseline_current, Some(0.20));
        assert_eq!(deltas[0].metrics[0].candidate_current, Some(0.02));
    }
}
