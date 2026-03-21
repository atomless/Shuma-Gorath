use std::collections::BTreeSet;

use crate::config::AllowedActionsSurface;

use super::benchmark_results::{
    BenchmarkBaselineReference, BenchmarkEscalationHint, BenchmarkFamilyResult,
};

pub(super) fn unavailable_baseline_reference() -> BenchmarkBaselineReference {
    BenchmarkBaselineReference {
        reference_kind: "prior_window".to_string(),
        status: "not_available".to_string(),
        note: "Baseline comparison materializes with benchmark-result history.".to_string(),
    }
}

pub(super) fn unavailable_improvement_status() -> String {
    "not_available".to_string()
}

pub(super) fn overall_coverage_status(families: &[BenchmarkFamilyResult]) -> String {
    if families
        .iter()
        .all(|family| family.capability_gate == "supported")
    {
        "supported".to_string()
    } else if families
        .iter()
        .any(|family| family.capability_gate == "supported")
    {
        "partial_support".to_string()
    } else {
        "not_yet_supported".to_string()
    }
}

pub(super) fn overall_status(families: &[BenchmarkFamilyResult]) -> String {
    if families
        .iter()
        .any(|family| family.status == "outside_budget")
    {
        "outside_budget".to_string()
    } else if families.iter().any(|family| family.status == "near_limit") {
        "near_limit".to_string()
    } else if families
        .iter()
        .any(|family| family.status == "inside_budget")
    {
        "inside_budget".to_string()
    } else if families
        .iter()
        .any(|family| family.status == "insufficient_evidence")
    {
        "insufficient_evidence".to_string()
    } else {
        "not_yet_supported".to_string()
    }
}

pub(super) fn derive_escalation_hint(
    allowed_actions: &AllowedActionsSurface,
    families: &[BenchmarkFamilyResult],
) -> BenchmarkEscalationHint {
    let outside_budget_families: Vec<&BenchmarkFamilyResult> = families
        .iter()
        .filter(|family| family.status == "outside_budget")
        .collect();
    let near_limit_families: Vec<&BenchmarkFamilyResult> = families
        .iter()
        .filter(|family| family.status == "near_limit")
        .collect();
    let insufficient_families: Vec<&BenchmarkFamilyResult> = families
        .iter()
        .filter(|family| family.status == "insufficient_evidence")
        .collect();

    let availability = "partial_support".to_string();
    let review_status = "manual_review_required".to_string();

    if outside_budget_families.is_empty() {
        let mut blockers = Vec::new();
        let trigger_family_ids = if !near_limit_families.is_empty() {
            blockers.push("near_limit_only".to_string());
            family_ids(&near_limit_families)
        } else if !insufficient_families.is_empty() {
            blockers.push("insufficient_evidence".to_string());
            family_ids(&insufficient_families)
        } else {
            blockers.push("outside_budget_not_observed".to_string());
            Vec::new()
        };
        return BenchmarkEscalationHint {
            availability,
            decision: "observe_longer".to_string(),
            review_status,
            trigger_family_ids,
            candidate_action_families: Vec::new(),
            blockers,
            note:
                "Current benchmark evidence does not yet justify config or code escalation; keep observing additional windows."
                    .to_string(),
        };
    }

    let trigger_family_ids = family_ids(&outside_budget_families);
    let mut candidate_action_families = BTreeSet::new();
    let mut blockers = BTreeSet::new();

    for family in outside_budget_families {
        if family.capability_gate == "not_yet_supported" {
            blockers.insert("family_capability_gap".to_string());
        }

        let mapped_families = benchmark_action_families(family.family_id.as_str());
        if mapped_families.is_empty() {
            blockers.insert("no_matching_config_surface".to_string());
            continue;
        }

        let matching_surface_families: Vec<_> = allowed_actions
            .families
            .iter()
            .filter(|allowed_family| mapped_families.contains(&allowed_family.family.as_str()))
            .collect();

        if matching_surface_families.is_empty() {
            blockers.insert("no_matching_config_surface".to_string());
            continue;
        }

        let has_addressable_surface = matching_surface_families.iter().any(|allowed_family| {
            matches!(
                allowed_family.controller_status.as_str(),
                "allowed" | "manual_only"
            )
        });

        if has_addressable_surface {
            for allowed_family in matching_surface_families {
                if matches!(
                    allowed_family.controller_status.as_str(),
                    "allowed" | "manual_only"
                ) {
                    candidate_action_families.insert(allowed_family.family.clone());
                }
            }
        } else {
            blockers.insert("no_matching_config_surface".to_string());
        }
    }

    if blockers.is_empty() && !candidate_action_families.is_empty() {
        return BenchmarkEscalationHint {
            availability,
            decision: "config_tuning_candidate".to_string(),
            review_status,
            trigger_family_ids,
            candidate_action_families: candidate_action_families.into_iter().collect(),
            blockers: Vec::new(),
            note: "Current-window benchmark misses align with existing config surfaces; manual review remains required before proposing a tuning change."
                .to_string(),
        };
    }

    BenchmarkEscalationHint {
        availability,
        decision: "code_evolution_candidate".to_string(),
        review_status,
        trigger_family_ids,
        candidate_action_families: candidate_action_families.into_iter().collect(),
        blockers: blockers.into_iter().collect(),
        note: "At least one outside-budget benchmark family is not addressable through the current config surface or requires missing capability; manual review should consider code evolution."
            .to_string(),
    }
}

fn benchmark_action_families(family_id: &str) -> &'static [&'static str] {
    match family_id {
        "suspicious_origin_cost" => &[
            "geo_policy",
            "ip_range_policy",
            "honeypot",
            "maze_core",
            "tarpit",
            "proof_of_work",
            "challenge",
            "not_a_bot",
            "botness",
            "cdp_detection",
            "fingerprint_signal",
        ],
        "likely_human_friction" => &[
            "core_policy",
            "browser_policy",
            "proof_of_work",
            "challenge",
            "not_a_bot",
            "botness",
            "maze_core",
        ],
        _ => &[],
    }
}

fn family_ids(families: &[&BenchmarkFamilyResult]) -> Vec<String> {
    families
        .iter()
        .map(|family| family.family_id.clone())
        .collect()
}
