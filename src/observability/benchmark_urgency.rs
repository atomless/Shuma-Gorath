use super::benchmark_results::{BenchmarkFamilyResult, BenchmarkUrgencySummary};

pub(crate) fn benchmark_urgency_summary(
    families: &[BenchmarkFamilyResult],
) -> BenchmarkUrgencySummary {
    let exploit_short_window_status = family_status(families, "scrapling_exploit_progress");
    let exploit_long_window_status =
        family_comparison_status(families, "scrapling_exploit_progress");
    let likely_human_short_window_status = family_status(families, "likely_human_friction");
    let likely_human_long_window_status =
        family_comparison_status(families, "likely_human_friction");

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
        && likely_human_short_window_status == "not_available"
    {
        "not_available"
    } else if homeostasis_break_status == "triggered"
        || exploit_short_window_status == "outside_budget"
    {
        "critical"
    } else if matches!(
        exploit_short_window_status.as_str(),
        "near_limit" | "outside_budget"
    ) || matches!(
        exploit_long_window_status.as_str(),
        "regressed" | "mixed"
    ) || likely_human_short_window_status == "outside_budget"
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
        likely_human_short_window_status,
        likely_human_long_window_status,
        homeostasis_break_status: homeostasis_break_status.to_string(),
        homeostasis_break_reasons: homeostasis_break_reasons.clone(),
        note: urgency_note(status, homeostasis_break_reasons.as_slice()),
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

fn urgency_note(status: &str, break_reasons: &[String]) -> String {
    if break_reasons.is_empty() {
        match status {
            "critical" => {
                "Current exploit or likely-human harm pressure is critical even though no explicit homeostasis-break reason is yet materialized."
                    .to_string()
            }
            "elevated" => {
                "Current pressure is elevated and should be watched closely across the next bounded cycle."
                    .to_string()
            }
            "steady" => {
                "Current exploit and likely-human harm pressure do not warrant an immediate homeostasis break."
                    .to_string()
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
