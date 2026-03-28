use super::benchmark_results::{
    BenchmarkDiagnosisEvidenceQuality, BenchmarkFamilyResult, BenchmarkUrgencySummary,
};

pub(crate) fn benchmark_urgency_summary(
    families: &[BenchmarkFamilyResult],
    restriction_readiness_status: &str,
    exploit_evidence_quality: &BenchmarkDiagnosisEvidenceQuality,
) -> BenchmarkUrgencySummary {
    let exploit_short_window_status =
        family_status(families, "mixed_attacker_restriction_progress");
    let exploit_long_window_status =
        family_comparison_status(families, "mixed_attacker_restriction_progress");
    let suspicious_origin_short_window_status = family_status(families, "suspicious_origin_cost");
    let likely_human_short_window_status = family_status(families, "likely_human_friction");
    let likely_human_long_window_status =
        family_comparison_status(families, "likely_human_friction");
    let restriction_confidence_status = restriction_confidence_status(
        exploit_short_window_status.as_str(),
        restriction_readiness_status,
        exploit_evidence_quality,
    );
    let abuse_backstop_status =
        abuse_backstop_status(suspicious_origin_short_window_status.as_str());

    let mut homeostasis_break_reasons = Vec::new();
    if exploit_long_window_status == "regressed" {
        homeostasis_break_reasons.push("exploit_success_regressed".to_string());
    }
    if likely_human_short_window_status == "outside_budget"
        && likely_human_long_window_status == "regressed"
    {
        homeostasis_break_reasons.push("likely_human_harm_regressed".to_string());
    }

    let homeostasis_break_status = if homeostasis_break_reasons.is_empty() {
        "not_triggered"
    } else {
        "triggered"
    };
    let status = if exploit_short_window_status == "not_available"
        && abuse_backstop_status == "not_available"
        && likely_human_short_window_status == "not_available"
    {
        "not_available"
    } else if homeostasis_break_status == "triggered"
        || exploit_short_window_status == "outside_budget"
        || abuse_backstop_status == "triggered"
    {
        "critical"
    } else if matches!(
        exploit_short_window_status.as_str(),
        "near_limit" | "outside_budget"
    ) || matches!(
        exploit_long_window_status.as_str(),
        "regressed" | "mixed"
    ) || likely_human_short_window_status == "outside_budget"
        || abuse_backstop_status == "arming"
        || likely_human_long_window_status == "regressed"
    {
        "elevated"
    } else {
        "steady"
    };

    BenchmarkUrgencySummary {
        status: status.to_string(),
        exploit_short_window_status,
        exploit_long_window_status,
        restriction_confidence_status: restriction_confidence_status.to_string(),
        abuse_backstop_status: abuse_backstop_status.to_string(),
        likely_human_short_window_status,
        likely_human_long_window_status,
        homeostasis_break_status: homeostasis_break_status.to_string(),
        homeostasis_break_reasons: homeostasis_break_reasons.clone(),
        note: urgency_note(
            status,
            restriction_confidence_status,
            abuse_backstop_status,
            homeostasis_break_reasons.as_slice(),
        ),
    }
}

fn family_status(families: &[BenchmarkFamilyResult], family_id: &str) -> String {
    families
        .iter()
        .find(|family| family.family_id == family_id)
        .map(|family| family.status.clone())
        .unwrap_or_else(|| "not_available".to_string())
}

fn family_comparison_status(families: &[BenchmarkFamilyResult], family_id: &str) -> String {
    families
        .iter()
        .find(|family| family.family_id == family_id)
        .map(|family| family.comparison_status.clone())
        .unwrap_or_else(|| "not_available".to_string())
}

fn restriction_confidence_status(
    exploit_short_window_status: &str,
    restriction_readiness_status: &str,
    exploit_evidence_quality: &BenchmarkDiagnosisEvidenceQuality,
) -> &'static str {
    let surface_native_high_confidence = exploit_evidence_quality.attribution_status
        == "surface_native_shared_path"
        && exploit_evidence_quality.diagnosis_confidence == "high";
    if exploit_short_window_status == "outside_budget"
        && restriction_readiness_status == "ready"
        && surface_native_high_confidence
    {
        "high"
    } else if exploit_short_window_status == "outside_budget"
        && (restriction_readiness_status == "ready"
            || exploit_evidence_quality.status == "high_confidence")
    {
        "medium"
    } else if exploit_short_window_status == "outside_budget" {
        "low"
    } else if restriction_readiness_status == "ready" {
        "ready"
    } else if restriction_readiness_status == "partial" {
        "partial"
    } else {
        "not_available"
    }
}

fn abuse_backstop_status(suspicious_origin_short_window_status: &str) -> &'static str {
    match suspicious_origin_short_window_status {
        "outside_budget" => "triggered",
        "near_limit" => "arming",
        "inside_budget" => "quiet",
        "insufficient_evidence" => "not_ready",
        _ => "not_available",
    }
}

fn urgency_note(
    status: &str,
    restriction_confidence_status: &str,
    abuse_backstop_status: &str,
    break_reasons: &[String],
) -> String {
    if break_reasons.is_empty() {
        match status {
            "critical" => {
                format!(
                    "Current pressure is critical with restriction confidence {} and abuse backstop {} even though no explicit homeostasis-break reason is yet materialized.",
                    restriction_confidence_status,
                    abuse_backstop_status
                )
            }
            "elevated" => {
                format!(
                    "Current pressure is elevated with restriction confidence {} and abuse backstop {} and should be watched closely across the next bounded cycle.",
                    restriction_confidence_status,
                    abuse_backstop_status
                )
            }
            "steady" => {
                format!(
                    "Current exploit and likely-human harm pressure do not warrant an immediate homeostasis break; restriction confidence is {} and abuse backstop is {}.",
                    restriction_confidence_status,
                    abuse_backstop_status
                )
            }
            _ => "Urgency is not available because the required benchmark families are not materialized yet."
                .to_string(),
        }
    } else {
        format!(
            "Homeostasis break is triggered by: {}.",
            break_reasons.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::benchmark_urgency_summary;
    use crate::observability::benchmark_results::{
        unavailable_benchmark_diagnosis_evidence_quality, BenchmarkDiagnosisEvidenceQuality,
        BenchmarkFamilyResult, BenchmarkMetricResult,
    };

    fn family(family_id: &str, status: &str, comparison_status: &str) -> BenchmarkFamilyResult {
        BenchmarkFamilyResult {
            family_id: family_id.to_string(),
            status: status.to_string(),
            capability_gate: "supported".to_string(),
            note: "test family".to_string(),
            baseline_status: None,
            comparison_status: comparison_status.to_string(),
            exploit_loci: Vec::new(),
            metrics: vec![BenchmarkMetricResult {
                metric_id: "metric".to_string(),
                status: status.to_string(),
                current: None,
                target: None,
                delta: None,
                exactness: "derived".to_string(),
                basis: "observed".to_string(),
                capability_gate: "supported".to_string(),
                baseline_current: None,
                comparison_delta: None,
                comparison_status: comparison_status.to_string(),
            }],
        }
    }

    fn high_confidence_evidence() -> BenchmarkDiagnosisEvidenceQuality {
        BenchmarkDiagnosisEvidenceQuality {
            status: "high_confidence".to_string(),
            diagnosis_confidence: "high".to_string(),
            attribution_status: "surface_native_shared_path".to_string(),
            sample_status: "sufficient".to_string(),
            freshness_status: "fresh".to_string(),
            recent_window_support_status: "reproduced_recently".to_string(),
            locality_status: "localized".to_string(),
            breach_loci: Vec::new(),
            note: "test".to_string(),
        }
    }

    #[test]
    fn urgency_raises_confidence_when_restriction_is_ready_and_surface_native() {
        let summary = benchmark_urgency_summary(
            &[
                family("mixed_attacker_restriction_progress", "outside_budget", "regressed"),
                family("suspicious_origin_cost", "inside_budget", "improved"),
                family("likely_human_friction", "inside_budget", "improved"),
            ],
            "ready",
            &high_confidence_evidence(),
        );

        assert_eq!(summary.restriction_confidence_status, "high");
        assert_eq!(summary.abuse_backstop_status, "quiet");
        assert_eq!(summary.status, "critical");
    }

    #[test]
    fn urgency_uses_suspicious_origin_cost_as_abuse_backstop() {
        let summary = benchmark_urgency_summary(
            &[
                family("mixed_attacker_restriction_progress", "inside_budget", "improved"),
                family("suspicious_origin_cost", "outside_budget", "regressed"),
                family("likely_human_friction", "inside_budget", "improved"),
            ],
            "partial",
            &unavailable_benchmark_diagnosis_evidence_quality(),
        );

        assert_eq!(summary.restriction_confidence_status, "partial");
        assert_eq!(summary.abuse_backstop_status, "triggered");
        assert_eq!(summary.status, "critical");
    }
}
