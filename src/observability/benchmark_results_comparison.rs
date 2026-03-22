use std::collections::BTreeSet;

use crate::config::AllowedActionsSurface;

use super::benchmark_results::{
    BenchmarkBaselineReference, BenchmarkEscalationHint, BenchmarkFamilyResult,
};

#[cfg(test)]
pub(super) fn unavailable_baseline_reference() -> BenchmarkBaselineReference {
    unavailable_reference_for(
        "prior_window",
        "No prior-window benchmark subject is currently materialized for comparison.",
    )
}

pub(super) fn unavailable_reference_for(
    reference_kind: &str,
    note: &str,
) -> BenchmarkBaselineReference {
    BenchmarkBaselineReference {
        reference_kind: reference_kind.to_string(),
        status: "not_available".to_string(),
        subject_kind: None,
        generated_at: None,
        note: note.to_string(),
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
        "non_human_category_posture" => &["robots_policy", "verified_identity"],
        _ => &[],
    }
}

fn family_ids(families: &[&BenchmarkFamilyResult]) -> Vec<String> {
    families
        .iter()
        .map(|family| family.family_id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        derive_escalation_hint, overall_coverage_status, overall_status,
        unavailable_baseline_reference, unavailable_improvement_status,
    };
    use crate::config::allowed_actions_v1;
    use crate::observability::benchmark_results::{BenchmarkFamilyResult, BenchmarkMetricResult};

    fn family(family_id: &str, status: &str, capability_gate: &str) -> BenchmarkFamilyResult {
        BenchmarkFamilyResult {
            family_id: family_id.to_string(),
            status: status.to_string(),
            capability_gate: capability_gate.to_string(),
            note: "test family".to_string(),
            baseline_status: None,
            comparison_status: "not_available".to_string(),
            metrics: vec![BenchmarkMetricResult {
                metric_id: format!("{family_id}_metric"),
                status: status.to_string(),
                current: Some(0.42),
                target: Some(0.20),
                delta: Some(0.22),
                exactness: "exact".to_string(),
                basis: "observed".to_string(),
                capability_gate: capability_gate.to_string(),
                baseline_current: None,
                comparison_delta: None,
                comparison_status: "not_available".to_string(),
            }],
        }
    }

    #[test]
    fn unavailable_baseline_and_improvement_are_explicit() {
        let baseline = unavailable_baseline_reference();
        assert_eq!(baseline.reference_kind, "prior_window");
        assert_eq!(baseline.status, "not_available");
        assert_eq!(unavailable_improvement_status(), "not_available");
    }

    #[test]
    fn overall_coverage_and_status_prioritize_supported_and_worst_budget_state() {
        let families = vec![
            family("suspicious_origin_cost", "inside_budget", "supported"),
            family("likely_human_friction", "outside_budget", "partially_supported"),
            family(
                "representative_adversary_effectiveness",
                "insufficient_evidence",
                "supported",
            ),
        ];

        assert_eq!(overall_coverage_status(families.as_slice()), "partial_support");
        assert_eq!(overall_status(families.as_slice()), "outside_budget");
    }

    #[test]
    fn escalation_hint_proposes_config_tuning_for_addressable_budget_breach() {
        let hint = derive_escalation_hint(
            &allowed_actions_v1(),
            &[family(
                "likely_human_friction",
                "outside_budget",
                "partially_supported",
            )],
        );

        assert_eq!(hint.decision, "config_tuning_candidate");
        assert_eq!(hint.review_status, "manual_review_required");
        assert!(hint
            .trigger_family_ids
            .contains(&"likely_human_friction".to_string()));
        assert!(hint
            .candidate_action_families
            .contains(&"challenge".to_string()));
    }

    #[test]
    fn escalation_hint_stays_observe_longer_without_outside_budget_families() {
        let hint = derive_escalation_hint(
            &allowed_actions_v1(),
            &[family("suspicious_origin_cost", "near_limit", "supported")],
        );

        assert_eq!(hint.decision, "observe_longer");
        assert_eq!(hint.review_status, "manual_review_required");
        assert!(hint.blockers.contains(&"near_limit_only".to_string()));
    }
}
