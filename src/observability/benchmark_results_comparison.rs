use std::collections::BTreeSet;

use crate::config::{controller_action_family_risk_profile, AllowedActionsSurface};

use super::benchmark_results::{
    BenchmarkBaselineReference, BenchmarkEscalationFamilyGuidance, BenchmarkEscalationHint,
    BenchmarkFamilyResult,
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
            availability: "partial_support".to_string(),
            decision: "observe_longer".to_string(),
            review_status,
            problem_class: "no_escalation_required".to_string(),
            guidance_status: "observe_longer".to_string(),
            tractability: "not_actionable_yet".to_string(),
            expected_direction: "continue_observing".to_string(),
            trigger_family_ids,
            trigger_metric_ids: Vec::new(),
            candidate_action_families: Vec::new(),
            family_guidance: Vec::new(),
            blockers,
            note:
                "Current benchmark evidence does not yet justify config or code escalation; keep observing additional windows."
                    .to_string(),
        };
    }

    let primary_family = primary_outside_budget_family(outside_budget_families.as_slice());
    let trigger_family_ids = family_ids(&outside_budget_families);
    let trigger_metric_ids = outside_budget_metric_ids(primary_family);
    let classification = classify_problem(primary_family);
    let candidate_action_families =
        allowed_candidate_action_families(allowed_actions, classification.action_families);
    let family_guidance = family_guidance_rows(candidate_action_families.as_slice());
    let mut blockers = BTreeSet::new();

    if primary_family.capability_gate == "not_yet_supported" {
        blockers.insert("family_capability_gap".to_string());
    }
    if classification.decision == "config_tuning_candidate" && candidate_action_families.is_empty() {
        blockers.insert("no_matching_config_surface".to_string());
    }

    if classification.decision == "config_tuning_candidate"
        && blockers.is_empty()
        && !candidate_action_families.is_empty()
    {
        return BenchmarkEscalationHint {
            availability: family_availability(primary_family).to_string(),
            decision: classification.decision.to_string(),
            review_status,
            problem_class: classification.problem_class.to_string(),
            guidance_status: classification.guidance_status.to_string(),
            tractability: classification.tractability.to_string(),
            expected_direction: classification.expected_direction.to_string(),
            trigger_family_ids,
            trigger_metric_ids,
            candidate_action_families,
            family_guidance,
            blockers: Vec::new(),
            note: classification.note.to_string(),
        };
    }

    BenchmarkEscalationHint {
        availability: family_availability(primary_family).to_string(),
        decision: "code_evolution_candidate".to_string(),
        review_status,
        problem_class: classification.problem_class.to_string(),
        guidance_status: "code_evolution_only".to_string(),
        tractability: "code_or_capability_gap".to_string(),
        expected_direction: classification.expected_direction.to_string(),
        trigger_family_ids,
        trigger_metric_ids,
        candidate_action_families,
        family_guidance,
        blockers: blockers.into_iter().collect(),
        note: if classification.decision == "config_tuning_candidate" {
            "Observed benchmark misses do not map cleanly to a still-legal bounded config move from the current surface, so code or capability evolution remains the next review path."
                .to_string()
        } else {
            classification.note.to_string()
        },
    }
}

struct ProblemClassification {
    problem_class: &'static str,
    decision: &'static str,
    guidance_status: &'static str,
    tractability: &'static str,
    expected_direction: &'static str,
    note: &'static str,
    action_families: &'static [&'static str],
}

fn primary_outside_budget_family<'a>(
    families: &'a [&BenchmarkFamilyResult],
) -> &'a BenchmarkFamilyResult {
    families
        .iter()
        .copied()
        .min_by_key(|family| family_priority(family.family_id.as_str()))
        .expect("outside-budget family set must be non-empty")
}

fn family_priority(family_id: &str) -> u8 {
    match family_id {
        "likely_human_friction" => 0,
        "scrapling_surface_contract" => 1,
        "suspicious_origin_cost" => 2,
        "beneficial_non_human_posture" => 3,
        "non_human_category_posture" => 4,
        "representative_adversary_effectiveness" => 5,
        _ => 10,
    }
}

fn family_availability(family: &BenchmarkFamilyResult) -> &'static str {
    match family.capability_gate.as_str() {
        "supported" => "supported",
        "partially_supported" => "partial_support",
        _ => "partial_support",
    }
}

fn classify_problem(family: &BenchmarkFamilyResult) -> ProblemClassification {
    match family.family_id.as_str() {
        "likely_human_friction" => ProblemClassification {
            problem_class: "likely_human_friction_overspend",
            decision: "config_tuning_candidate",
            guidance_status: "bounded_family_guidance",
            tractability: "family_level_policy_choice",
            expected_direction: "reduce_likely_human_friction",
            note: "Likely-human friction is above target and currently maps to bounded controller-tunable families that can ease human-visible burden first.",
            action_families: benchmark_action_families("likely_human_friction"),
        },
        "suspicious_origin_cost" => {
            let latency_only = family
                .metrics
                .iter()
                .any(|metric| {
                    metric.metric_id == "suspicious_forwarded_latency_share"
                        && metric.status == "outside_budget"
                });
            ProblemClassification {
                problem_class: if latency_only {
                    "suspicious_forwarded_latency_overspend"
                } else {
                    "suspicious_forwarded_reach_overspend"
                },
                decision: "config_tuning_candidate",
                guidance_status: "bounded_family_guidance",
                tractability: "family_level_policy_choice",
                expected_direction: "tighten_suspicious_origin_controls",
                note: "Suspicious-origin reach or latency is above target and the controller should prefer lower-friction signal families before broader human-visible gates.",
                action_families: benchmark_action_families("suspicious_origin_cost"),
            }
        }
        "scrapling_surface_contract" => ProblemClassification {
            problem_class: "scrapling_surface_contract_gap",
            decision: "code_evolution_candidate",
            guidance_status: "code_evolution_only",
            tractability: "code_or_capability_gap",
            expected_direction: "close_required_scrapling_surface_gaps",
            note: "Latest Scrapling defense-surface contract misses mean the loop cannot yet treat aggregate suspicious-origin suppression as operationally healthy or tuning-ready.",
            action_families: &[],
        },
        "beneficial_non_human_posture" => ProblemClassification {
            problem_class: "beneficial_non_human_harm",
            decision: "code_evolution_candidate",
            guidance_status: "code_evolution_only",
            tractability: "code_or_capability_gap",
            expected_direction: "protect_beneficial_non_human_traffic",
            note: "Beneficial non-human harm is policy-shaped and remains outside the bounded autonomous config move ring.",
            action_families: &[],
        },
        "non_human_category_posture" => ProblemClassification {
            problem_class: "category_posture_gap",
            decision: "code_evolution_candidate",
            guidance_status: "code_evolution_only",
            tractability: "code_or_capability_gap",
            expected_direction: "improve_category_target_achievement",
            note: "Category posture misses indicate classification, evidence, or defense-capability gaps rather than a clean bounded config move.",
            action_families: &[],
        },
        "representative_adversary_effectiveness" => ProblemClassification {
            problem_class: "representative_adversary_gap",
            decision: "code_evolution_candidate",
            guidance_status: "code_evolution_only",
            tractability: "code_or_capability_gap",
            expected_direction: "reduce_adversary_goal_success",
            note: "Representative adversary effectiveness misses require scenario or capability evolution before autonomous config tuning is trustworthy.",
            action_families: &[],
        },
        _ => ProblemClassification {
            problem_class: "unclassified_outside_budget_gap",
            decision: "code_evolution_candidate",
            guidance_status: "code_evolution_only",
            tractability: "code_or_capability_gap",
            expected_direction: "manual_investigation_required",
            note: "The current outside-budget family does not yet map to an approved bounded move class.",
            action_families: &[],
        },
    }
}

fn benchmark_action_families(family_id: &str) -> &'static [&'static str] {
    match family_id {
        "suspicious_origin_cost" => &[
            "maze_core",
            "core_policy",
            "proof_of_work",
            "challenge",
            "not_a_bot",
            "botness",
            "cdp_detection",
            "fingerprint_signal",
        ],
        "likely_human_friction" => &[
            "core_policy",
            "proof_of_work",
            "challenge",
            "not_a_bot",
            "botness",
            "maze_core",
        ],
        "non_human_category_posture" => &[],
        _ => &[],
    }
}

fn outside_budget_metric_ids(family: &BenchmarkFamilyResult) -> Vec<String> {
    family
        .metrics
        .iter()
        .filter(|metric| metric.status == "outside_budget")
        .map(|metric| metric.metric_id.clone())
        .collect()
}

fn allowed_candidate_action_families(
    allowed_actions: &AllowedActionsSurface,
    mapped_families: &[&str],
) -> Vec<String> {
    let mapped_families = mapped_families.iter().copied().collect::<BTreeSet<_>>();
    let mut candidate_action_families = BTreeSet::new();
    for allowed_group in &allowed_actions.groups {
        if allowed_group.controller_status == "allowed"
            && mapped_families.contains(allowed_group.family.as_str())
        {
            candidate_action_families.insert(allowed_group.family.clone());
        }
    }
    candidate_action_families.into_iter().collect()
}

fn family_guidance_rows(
    candidate_action_families: &[String],
) -> Vec<BenchmarkEscalationFamilyGuidance> {
    candidate_action_families
        .iter()
        .filter_map(|family| {
            controller_action_family_risk_profile(family.as_str()).map(|profile| {
                BenchmarkEscalationFamilyGuidance {
                    family: profile.family,
                    likely_human_risk: profile.likely_human_risk,
                    tolerated_non_human_risk: profile.tolerated_non_human_risk,
                    note: profile.note,
                }
            })
        })
        .collect()
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
    use crate::observability::benchmark_results::{
        BenchmarkFamilyResult, BenchmarkMetricResult,
    };

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

    fn metric(metric_id: &str, status: &str, capability_gate: &str) -> BenchmarkMetricResult {
        BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
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
        }
    }

    fn family_with_metrics(
        family_id: &str,
        status: &str,
        capability_gate: &str,
        metrics: Vec<BenchmarkMetricResult>,
    ) -> BenchmarkFamilyResult {
        BenchmarkFamilyResult {
            family_id: family_id.to_string(),
            status: status.to_string(),
            capability_gate: capability_gate.to_string(),
            note: "test family".to_string(),
            baseline_status: None,
            comparison_status: "not_available".to_string(),
            metrics,
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
        assert!(hint
            .candidate_action_families
            .contains(&"core_policy".to_string()));
        assert!(!hint
            .candidate_action_families
            .contains(&"browser_policy".to_string()));
    }

    #[test]
    fn escalation_hint_filters_out_controller_forbidden_families() {
        let hint = derive_escalation_hint(
            &allowed_actions_v1(),
            &[family(
                "suspicious_origin_cost",
                "outside_budget",
                "partially_supported",
            )],
        );

        assert_eq!(hint.decision, "config_tuning_candidate");
        assert!(hint
            .candidate_action_families
            .contains(&"fingerprint_signal".to_string()));
        assert!(hint
            .candidate_action_families
            .contains(&"cdp_detection".to_string()));
        assert!(!hint
            .candidate_action_families
            .contains(&"geo_policy".to_string()));
        assert!(!hint
            .candidate_action_families
            .contains(&"ip_range_policy".to_string()));
    }

    #[test]
    fn escalation_hint_names_problem_class_trigger_metrics_and_guidance() {
        let hint = derive_escalation_hint(
            &allowed_actions_v1(),
            &[family_with_metrics(
                "suspicious_origin_cost",
                "outside_budget",
                "supported",
                vec![
                    metric(
                        "suspicious_forwarded_latency_share",
                        "outside_budget",
                        "supported",
                    ),
                    metric(
                        "suspicious_forwarded_request_rate",
                        "inside_budget",
                        "supported",
                    ),
                ],
            )],
        );

        assert_eq!(hint.problem_class, "suspicious_forwarded_latency_overspend");
        assert_eq!(hint.guidance_status, "bounded_family_guidance");
        assert_eq!(hint.tractability, "family_level_policy_choice");
        assert_eq!(hint.expected_direction, "tighten_suspicious_origin_controls");
        assert_eq!(
            hint.trigger_metric_ids,
            vec!["suspicious_forwarded_latency_share".to_string()]
        );
        assert!(hint
            .family_guidance
            .iter()
            .any(|row| row.family == "fingerprint_signal"));
    }

    #[test]
    fn escalation_hint_marks_category_posture_gap_as_code_evolution_only() {
        let hint = derive_escalation_hint(
            &allowed_actions_v1(),
            &[family_with_metrics(
                "non_human_category_posture",
                "outside_budget",
                "partially_supported",
                vec![metric(
                    "category_posture_alignment:indexing_bot",
                    "outside_budget",
                    "partially_supported",
                )],
            )],
        );

        assert_eq!(hint.problem_class, "category_posture_gap");
        assert_eq!(hint.guidance_status, "code_evolution_only");
        assert_eq!(hint.tractability, "code_or_capability_gap");
        assert_eq!(hint.decision, "code_evolution_candidate");
        assert!(hint.candidate_action_families.is_empty());
    }

    #[test]
    fn escalation_hint_marks_scrapling_surface_contract_gap_as_code_evolution_only() {
        let hint = derive_escalation_hint(
            &allowed_actions_v1(),
            &[family_with_metrics(
                "scrapling_surface_contract",
                "outside_budget",
                "supported",
                vec![metric(
                    "scrapling_required_surface_satisfaction_rate",
                    "outside_budget",
                    "supported",
                )],
            )],
        );

        assert_eq!(hint.problem_class, "scrapling_surface_contract_gap");
        assert_eq!(hint.guidance_status, "code_evolution_only");
        assert_eq!(hint.tractability, "code_or_capability_gap");
        assert_eq!(hint.decision, "code_evolution_candidate");
        assert!(hint.candidate_action_families.is_empty());
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
