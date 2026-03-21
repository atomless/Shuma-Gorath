use crate::bot_identity::policy::NonHumanTrafficStance;
use crate::config::Config;
use crate::observability::monitoring::{MonitoringSummary, RequestOutcomeBreakdownSummaryRow};

use super::benchmark_results::{BenchmarkFamilyResult, BenchmarkMetricResult};
use super::benchmark_results_families::aggregate_budget_status;

pub(super) fn beneficial_non_human_posture_family(
    summary: &MonitoringSummary,
    cfg: &Config,
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
    let allow_capable = !matches!(
        cfg.verified_identity.non_human_traffic_stance,
        NonHumanTrafficStance::DenyAllNonHuman
    );
    let coverage_ratio = ratio(
        summary.verified_identity.verified,
        summary.verified_identity.attempts,
    );
    let mismatch_ratio = if allow_capable {
        ratio(short_circuited_requests, total_requests)
    } else {
        ratio(forwarded_requests, total_requests)
    };

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
    ];

    BenchmarkFamilyResult {
        family_id: "beneficial_non_human_posture".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: "partially_supported".to_string(),
        note: note_for_stance(cfg.verified_identity.non_human_traffic_stance, policy_row),
        baseline_status: None,
        comparison_status: "not_available".to_string(),
        metrics,
    }
}

fn note_for_stance(
    stance: NonHumanTrafficStance,
    policy_row: Option<&RequestOutcomeBreakdownSummaryRow>,
) -> String {
    let observed = policy_row.map(|row| row.total_requests).unwrap_or(0);
    format!(
        "Bounded verified-identity posture currently compares {} observed verified-identity policy decisions against the local non-human stance `{}`; finer allow-vs-restrict distinctions will deepen in later tranches.",
        observed,
        stance.as_str()
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
