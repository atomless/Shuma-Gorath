use std::collections::BTreeSet;

use crate::config::{
    next_numeric_constraint_value, AllowedActionStepDirection, AllowedActionsSurface, Config,
};

use super::benchmark_results::{
    BenchmarkBaselineReference, BenchmarkEscalationHint, BenchmarkFamilyResult,
    BenchmarkShortfallGuidance,
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
    cfg: &Config,
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
        let shortfall_guidance = if !near_limit_families.is_empty() {
            blockers.push("near_limit_only".to_string());
            near_limit_families
                .iter()
                .map(|family| observe_only_shortfall_guidance(family, "near_limit_only"))
                .collect::<Vec<_>>()
        } else if !insufficient_families.is_empty() {
            blockers.push("insufficient_evidence".to_string());
            insufficient_families
                .iter()
                .map(|family| observe_only_shortfall_guidance(family, "insufficient_evidence"))
                .collect::<Vec<_>>()
        } else {
            blockers.push("outside_budget_not_observed".to_string());
            Vec::new()
        };
        let trigger_family_ids = shortfall_guidance
            .iter()
            .map(|guidance| guidance.family_id.clone())
            .collect::<Vec<_>>();
        let trigger_metric_ids = shortfall_guidance
            .iter()
            .flat_map(|guidance| guidance.trigger_metric_ids.iter().cloned())
            .collect::<Vec<_>>();
        let (problem_class, guidance_status, tractability) =
            primary_guidance_fields(shortfall_guidance.as_slice());
        return BenchmarkEscalationHint {
            availability,
            decision: "observe_longer".to_string(),
            review_status,
            problem_class,
            guidance_status,
            tractability,
            trigger_family_ids,
            trigger_metric_ids,
            candidate_action_families: Vec::new(),
            recommended_action_family: None,
            blockers,
            shortfall_guidance,
            note:
                "Current benchmark evidence does not yet justify config or code escalation; keep observing additional windows."
                    .to_string(),
        };
    }

    let trigger_family_ids = family_ids(&outside_budget_families);
    let shortfall_guidance = outside_budget_families
        .iter()
        .map(|family| outside_budget_shortfall_guidance(cfg, allowed_actions, family))
        .collect::<Vec<_>>();
    let primary_guidance = primary_shortfall_guidance(shortfall_guidance.as_slice());
    let trigger_metric_ids = shortfall_guidance
        .iter()
        .flat_map(|guidance| guidance.trigger_metric_ids.iter().cloned())
        .collect::<Vec<_>>();
    let candidate_action_families = ordered_candidate_action_families(
        shortfall_guidance.as_slice(),
        primary_guidance,
    );
    let mut blockers = aggregated_guidance_blockers(shortfall_guidance.as_slice());
    let any_actionable = shortfall_guidance
        .iter()
        .any(|guidance| guidance_is_actionable(guidance.guidance_status.as_str()));
    let any_code_gap = shortfall_guidance
        .iter()
        .any(|guidance| guidance.guidance_status == "code_evolution_only");
    let (problem_class, guidance_status, tractability, recommended_action_family, note) =
        if let Some(primary_guidance) = primary_guidance {
            (
                primary_guidance.problem_class.clone(),
                primary_guidance.guidance_status.clone(),
                primary_guidance.tractability.clone(),
                primary_guidance.recommended_action_family.clone(),
                primary_guidance.note.clone(),
            )
        } else {
            (
                "not_applicable".to_string(),
                "insufficient_evidence".to_string(),
                "observe_only".to_string(),
                None,
                "Current benchmark pressure did not yield explicit shortfall guidance."
                    .to_string(),
            )
        };

    if any_actionable && !candidate_action_families.is_empty() {
        if any_code_gap {
            blockers.insert("additional_code_or_capability_gap_present".to_string());
        }
        return BenchmarkEscalationHint {
            availability,
            decision: "config_tuning_candidate".to_string(),
            review_status,
            problem_class,
            guidance_status,
            tractability,
            trigger_family_ids,
            trigger_metric_ids,
            candidate_action_families,
            recommended_action_family,
            blockers: blockers.into_iter().collect(),
            shortfall_guidance,
            note,
        };
    }

    BenchmarkEscalationHint {
        availability,
        decision: "code_evolution_candidate".to_string(),
        review_status,
        problem_class,
        guidance_status: "code_evolution_only".to_string(),
        tractability: "code_or_capability_gap".to_string(),
        trigger_family_ids,
        trigger_metric_ids,
        candidate_action_families,
        recommended_action_family: None,
        blockers: blockers.into_iter().collect(),
        shortfall_guidance,
        note: "At least one outside-budget benchmark family is not addressable through the current bounded config surface; manual review should consider code evolution or capability work."
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

fn outside_budget_shortfall_guidance(
    cfg: &Config,
    allowed_actions: &AllowedActionsSurface,
    family: &BenchmarkFamilyResult,
) -> BenchmarkShortfallGuidance {
    let trigger_metric_ids = metric_ids_with_status(family, "outside_budget");
    match family.family_id.as_str() {
        "likely_human_friction" => likely_human_friction_shortfall_guidance(
            cfg,
            allowed_actions,
            family,
            trigger_metric_ids,
        ),
        "suspicious_origin_cost" => suspicious_origin_cost_shortfall_guidance(
            cfg,
            allowed_actions,
            family,
            trigger_metric_ids,
        ),
        "non_human_category_posture" => BenchmarkShortfallGuidance {
            family_id: family.family_id.clone(),
            problem_class: "category_target_shortfall".to_string(),
            guidance_status: "code_evolution_only".to_string(),
            tractability: "code_or_capability_gap".to_string(),
            trigger_metric_ids,
            eligible_action_families: Vec::new(),
            recommended_action_family: None,
            expected_change_direction: "n/a".to_string(),
            human_friction_risk: "n/a".to_string(),
            tolerated_traffic_risk: "high".to_string(),
            blockers: vec!["operator_owned_rule_surface".to_string()],
            note: "Category posture misses point at operator-owned policy targets, not controller-tunable config, so the bounded config loop must not retarget them."
                .to_string(),
        },
        _ => generic_code_gap_shortfall_guidance(family, trigger_metric_ids),
    }
}

fn likely_human_friction_shortfall_guidance(
    cfg: &Config,
    allowed_actions: &AllowedActionsSurface,
    family: &BenchmarkFamilyResult,
    trigger_metric_ids: Vec<String>,
) -> BenchmarkShortfallGuidance {
    let eligible_action_families = ordered_auto_proposable_action_families(
        allowed_actions,
        benchmark_action_families(family.family_id.as_str()),
    );

    if cfg.pow_enabled
        && next_numeric_constraint_value(
            allowed_actions,
            "pow_difficulty",
            cfg.pow_difficulty as u64,
            AllowedActionStepDirection::Down,
        )
        .is_some()
    {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "likely_human_friction_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("proof_of_work".to_string()),
            "loosen".to_string(),
            "medium".to_string(),
            "low".to_string(),
            "Likely-human friction is outside budget and proof-of-work difficulty can step down one bounded notch immediately."
                .to_string(),
        );
    }
    if cfg.challenge_puzzle_enabled {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "likely_human_friction_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("challenge".to_string()),
            "loosen".to_string(),
            "medium".to_string(),
            "low".to_string(),
            "Likely-human friction is outside budget and the puzzle challenge can be disabled as a bounded reversible move."
                .to_string(),
        );
    }
    let max_not_a_bot_threshold = cfg
        .challenge_puzzle_risk_threshold
        .saturating_sub(1)
        .max(cfg.not_a_bot_risk_threshold);
    if cfg.not_a_bot_enabled
        && next_numeric_constraint_value(
            allowed_actions,
            "not_a_bot_risk_threshold",
            cfg.not_a_bot_risk_threshold as u64,
            AllowedActionStepDirection::Up,
        )
        .filter(|value| *value <= max_not_a_bot_threshold as u64)
        .is_some()
    {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "likely_human_friction_overspend",
            "bounded_heuristic_guidance",
            "family_policy_choice",
            trigger_metric_ids,
            eligible_action_families,
            Some("not_a_bot".to_string()),
            "loosen".to_string(),
            "medium".to_string(),
            "medium".to_string(),
            "Likely-human friction is outside budget and the lowest-risk bounded next step is to raise the not-a-bot threshold slightly."
                .to_string(),
        );
    }
    if next_numeric_constraint_value(
        allowed_actions,
        "challenge_puzzle_risk_threshold",
        cfg.challenge_puzzle_risk_threshold as u64,
        AllowedActionStepDirection::Up,
    )
    .is_some()
    {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "likely_human_friction_overspend",
            "bounded_heuristic_guidance",
            "family_policy_choice",
            trigger_metric_ids,
            eligible_action_families,
            Some("botness".to_string()),
            "loosen".to_string(),
            "medium".to_string(),
            "medium".to_string(),
            "Likely-human friction is outside budget and the bounded next heuristic is to delay challenge escalation by raising the puzzle threshold slightly."
                .to_string(),
        );
    }
    if cfg.maze_rollout_phase.as_str() == "enforce" {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "likely_human_friction_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("maze_core".to_string()),
            "loosen".to_string(),
            "high".to_string(),
            "medium".to_string(),
            "Likely-human friction is outside budget and maze rollout can step down from enforce to advisory immediately."
                .to_string(),
        );
    }
    if cfg.js_required_enforced {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "likely_human_friction_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("core_policy".to_string()),
            "loosen".to_string(),
            "high".to_string(),
            "medium".to_string(),
            "Likely-human friction is outside budget and the JS-required gate can be relaxed as a bounded reversible move."
                .to_string(),
        );
    }

    code_gap_shortfall_guidance(
        family.family_id.as_str(),
        "likely_human_friction_overspend",
        trigger_metric_ids,
        eligible_action_families,
        "Shuma is already near the bounded lower-friction edge of the current legal move ring, so further improvement now points to code or capability work rather than another safe config move."
            .to_string(),
    )
}

fn suspicious_origin_cost_shortfall_guidance(
    cfg: &Config,
    allowed_actions: &AllowedActionsSurface,
    family: &BenchmarkFamilyResult,
    trigger_metric_ids: Vec<String>,
) -> BenchmarkShortfallGuidance {
    let eligible_action_families = ordered_auto_proposable_action_families(
        allowed_actions,
        benchmark_action_families(family.family_id.as_str()),
    );

    if !cfg.fingerprint_signal_enabled {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("fingerprint_signal".to_string()),
            "tighten".to_string(),
            "low".to_string(),
            "medium".to_string(),
            "Suspicious-origin cost is outside budget and the lowest-friction exact move is to enable fingerprint-signal discrimination."
                .to_string(),
        );
    }
    if !cfg.cdp_detection_enabled {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("cdp_detection".to_string()),
            "tighten".to_string(),
            "low".to_string(),
            "medium".to_string(),
            "Suspicious-origin cost is outside budget and the lowest-friction exact move is to enable CDP detection."
                .to_string(),
        );
    }
    if !cfg.pow_enabled {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("proof_of_work".to_string()),
            "tighten".to_string(),
            "medium".to_string(),
            "low".to_string(),
            "Suspicious-origin cost is outside budget and proof-of-work can be enabled immediately as a bounded cost-shift."
                .to_string(),
        );
    }
    if next_numeric_constraint_value(
        allowed_actions,
        "pow_difficulty",
        cfg.pow_difficulty as u64,
        AllowedActionStepDirection::Up,
    )
    .is_some()
    {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "bounded_heuristic_guidance",
            "family_policy_choice",
            trigger_metric_ids,
            eligible_action_families,
            Some("proof_of_work".to_string()),
            "tighten".to_string(),
            "medium".to_string(),
            "low".to_string(),
            "Suspicious-origin cost is outside budget and the bounded next heuristic is to increase proof-of-work difficulty one notch."
                .to_string(),
        );
    }
    if !cfg.not_a_bot_enabled {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("not_a_bot".to_string()),
            "tighten".to_string(),
            "medium".to_string(),
            "low".to_string(),
            "Suspicious-origin cost is outside budget and not-a-bot can be enabled immediately as the next bounded gate."
                .to_string(),
        );
    }
    if next_numeric_constraint_value(
        allowed_actions,
        "not_a_bot_risk_threshold",
        cfg.not_a_bot_risk_threshold as u64,
        AllowedActionStepDirection::Down,
    )
    .is_some()
    {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "bounded_heuristic_guidance",
            "family_policy_choice",
            trigger_metric_ids,
            eligible_action_families,
            Some("not_a_bot".to_string()),
            "tighten".to_string(),
            "medium".to_string(),
            "low".to_string(),
            "Suspicious-origin cost is outside budget and the bounded next heuristic is to tighten the not-a-bot threshold slightly."
                .to_string(),
        );
    }
    if !cfg.challenge_puzzle_enabled {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("challenge".to_string()),
            "tighten".to_string(),
            "medium".to_string(),
            "low".to_string(),
            "Suspicious-origin cost is outside budget and the next exact bounded move is to enable the puzzle challenge."
                .to_string(),
        );
    }
    if next_numeric_constraint_value(
        allowed_actions,
        "challenge_puzzle_risk_threshold",
        cfg.challenge_puzzle_risk_threshold as u64,
        AllowedActionStepDirection::Down,
    )
    .filter(|value| *value > cfg.not_a_bot_risk_threshold as u64)
    .is_some()
    {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "bounded_heuristic_guidance",
            "family_policy_choice",
            trigger_metric_ids,
            eligible_action_families,
            Some("botness".to_string()),
            "tighten".to_string(),
            "medium".to_string(),
            "medium".to_string(),
            "Suspicious-origin cost is outside budget and the bounded next heuristic is to lower the challenge-escalation threshold slightly."
                .to_string(),
        );
    }
    if !cfg.maze_enabled || cfg.maze_rollout_phase.as_str() != "enforce" || !cfg.maze_auto_ban {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "bounded_heuristic_guidance",
            "family_policy_choice",
            trigger_metric_ids,
            eligible_action_families,
            Some("maze_core".to_string()),
            "tighten".to_string(),
            "high".to_string(),
            "medium".to_string(),
            "Suspicious-origin cost is outside budget and the bounded next heuristic is to move the maze rollout toward stronger enforcement."
                .to_string(),
        );
    }
    if !cfg.js_required_enforced {
        return actionable_shortfall_guidance(
            family.family_id.as_str(),
            "suspicious_origin_cost_overspend",
            "exact_move_guidance",
            "exact_config_move",
            trigger_metric_ids,
            eligible_action_families,
            Some("core_policy".to_string()),
            "tighten".to_string(),
            "high".to_string(),
            "medium".to_string(),
            "Suspicious-origin cost is outside budget and the JS-required gate can be re-enabled as a bounded reversible move."
                .to_string(),
        );
    }

    code_gap_shortfall_guidance(
        family.family_id.as_str(),
        "suspicious_origin_cost_overspend",
        trigger_metric_ids,
        eligible_action_families,
        "Shuma is already near the bounded stronger-enforcement edge of the current legal move ring, so further improvement now points to code or capability work rather than another safe config move."
            .to_string(),
    )
}

fn generic_code_gap_shortfall_guidance(
    family: &BenchmarkFamilyResult,
    trigger_metric_ids: Vec<String>,
) -> BenchmarkShortfallGuidance {
    let mut blockers = Vec::new();
    if family.capability_gate == "not_yet_supported" {
        blockers.push("family_capability_gap".to_string());
    }
    blockers.push("no_matching_controller_tunable_surface".to_string());
    BenchmarkShortfallGuidance {
        family_id: family.family_id.clone(),
        problem_class: "benchmark_family_gap".to_string(),
        guidance_status: "code_evolution_only".to_string(),
        tractability: "code_or_capability_gap".to_string(),
        trigger_metric_ids,
        eligible_action_families: Vec::new(),
        recommended_action_family: None,
        expected_change_direction: "n/a".to_string(),
        human_friction_risk: "n/a".to_string(),
        tolerated_traffic_risk: "n/a".to_string(),
        blockers,
        note: "This outside-budget benchmark family is not yet mapped to a controller-tunable config surface."
            .to_string(),
    }
}

fn observe_only_shortfall_guidance(
    family: &BenchmarkFamilyResult,
    blocker: &str,
) -> BenchmarkShortfallGuidance {
    let problem_class = match family.family_id.as_str() {
        "likely_human_friction" => "likely_human_friction_watch",
        "suspicious_origin_cost" => "suspicious_origin_cost_watch",
        "non_human_category_posture" => "category_target_watch",
        _ => "benchmark_family_watch",
    };
    BenchmarkShortfallGuidance {
        family_id: family.family_id.clone(),
        problem_class: problem_class.to_string(),
        guidance_status: "insufficient_evidence".to_string(),
        tractability: "observe_only".to_string(),
        trigger_metric_ids: metric_ids_with_status(family, family.status.as_str()),
        eligible_action_families: Vec::new(),
        recommended_action_family: None,
        expected_change_direction: "n/a".to_string(),
        human_friction_risk: "n/a".to_string(),
        tolerated_traffic_risk: "n/a".to_string(),
        blockers: vec![blocker.to_string()],
        note: "Current benchmark evidence is not yet strong enough to justify a bounded move."
            .to_string(),
    }
}

fn actionable_shortfall_guidance(
    family_id: &str,
    problem_class: &str,
    guidance_status: &str,
    tractability: &str,
    trigger_metric_ids: Vec<String>,
    eligible_action_families: Vec<String>,
    recommended_action_family: Option<String>,
    expected_change_direction: String,
    human_friction_risk: String,
    tolerated_traffic_risk: String,
    note: String,
) -> BenchmarkShortfallGuidance {
    BenchmarkShortfallGuidance {
        family_id: family_id.to_string(),
        problem_class: problem_class.to_string(),
        guidance_status: guidance_status.to_string(),
        tractability: tractability.to_string(),
        trigger_metric_ids,
        eligible_action_families,
        recommended_action_family,
        expected_change_direction,
        human_friction_risk,
        tolerated_traffic_risk,
        blockers: Vec::new(),
        note,
    }
}

fn code_gap_shortfall_guidance(
    family_id: &str,
    problem_class: &str,
    trigger_metric_ids: Vec<String>,
    eligible_action_families: Vec<String>,
    note: String,
) -> BenchmarkShortfallGuidance {
    BenchmarkShortfallGuidance {
        family_id: family_id.to_string(),
        problem_class: problem_class.to_string(),
        guidance_status: "code_evolution_only".to_string(),
        tractability: "code_or_capability_gap".to_string(),
        trigger_metric_ids,
        eligible_action_families,
        recommended_action_family: None,
        expected_change_direction: "n/a".to_string(),
        human_friction_risk: "n/a".to_string(),
        tolerated_traffic_risk: "n/a".to_string(),
        blockers: vec!["no_bounded_config_move_remaining".to_string()],
        note,
    }
}

fn ordered_auto_proposable_action_families(
    allowed_actions: &AllowedActionsSurface,
    family_ids: &[&str],
) -> Vec<String> {
    family_ids
        .iter()
        .filter_map(|family_id| {
            allowed_actions
                .families
                .iter()
                .find(|family| {
                    family.family == *family_id
                        && matches!(
                            family.auto_proposal_status.as_str(),
                            "supported" | "partial_support"
                        )
                })
                .map(|_| (*family_id).to_string())
        })
        .collect()
}

fn ordered_candidate_action_families(
    shortfall_guidance: &[BenchmarkShortfallGuidance],
    primary_guidance: Option<&BenchmarkShortfallGuidance>,
) -> Vec<String> {
    let mut ordered = Vec::new();
    if let Some(primary_guidance) = primary_guidance {
        extend_candidate_action_families(&mut ordered, primary_guidance);
    }
    for guidance in shortfall_guidance {
        if primary_guidance
            .map(|primary| primary.family_id == guidance.family_id)
            .unwrap_or(false)
        {
            continue;
        }
        extend_candidate_action_families(&mut ordered, guidance);
    }
    ordered
}

fn extend_candidate_action_families(
    ordered: &mut Vec<String>,
    guidance: &BenchmarkShortfallGuidance,
) {
    if !guidance_is_actionable(guidance.guidance_status.as_str()) {
        return;
    }
    if let Some(recommended) = &guidance.recommended_action_family {
        if !ordered.iter().any(|family| family == recommended) {
            ordered.push(recommended.clone());
        }
    }
    for family in &guidance.eligible_action_families {
        if !ordered.iter().any(|existing| existing == family) {
            ordered.push(family.clone());
        }
    }
}

fn guidance_is_actionable(guidance_status: &str) -> bool {
    matches!(
        guidance_status,
        "exact_move_guidance" | "bounded_heuristic_guidance"
    )
}

fn aggregated_guidance_blockers(
    shortfall_guidance: &[BenchmarkShortfallGuidance],
) -> BTreeSet<String> {
    let mut blockers = BTreeSet::new();
    for guidance in shortfall_guidance {
        for blocker in &guidance.blockers {
            blockers.insert(blocker.clone());
        }
    }
    blockers
}

fn primary_shortfall_guidance<'a>(
    shortfall_guidance: &'a [BenchmarkShortfallGuidance],
) -> Option<&'a BenchmarkShortfallGuidance> {
    shortfall_guidance
        .iter()
        .find(|guidance| {
            guidance.problem_class == "likely_human_friction_overspend"
                && guidance_is_actionable(guidance.guidance_status.as_str())
        })
        .or_else(|| {
            shortfall_guidance.iter().find(|guidance| {
                guidance.problem_class == "suspicious_origin_cost_overspend"
                    && guidance_is_actionable(guidance.guidance_status.as_str())
            })
        })
        .or_else(|| {
            shortfall_guidance
                .iter()
                .find(|guidance| guidance_is_actionable(guidance.guidance_status.as_str()))
        })
        .or_else(|| shortfall_guidance.first())
}

fn primary_guidance_fields(
    shortfall_guidance: &[BenchmarkShortfallGuidance],
) -> (String, String, String) {
    primary_shortfall_guidance(shortfall_guidance)
        .map(|guidance| {
            (
                guidance.problem_class.clone(),
                guidance.guidance_status.clone(),
                guidance.tractability.clone(),
            )
        })
        .unwrap_or_else(|| {
            (
                "not_applicable".to_string(),
                "insufficient_evidence".to_string(),
                "observe_only".to_string(),
            )
        })
}

fn metric_ids_with_status(
    family: &BenchmarkFamilyResult,
    status: &str,
) -> Vec<String> {
    family
        .metrics
        .iter()
        .filter(|metric| metric.status == status)
        .map(|metric| metric.metric_id.clone())
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
    use crate::config::{allowed_actions_v1, defaults};
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
        let mut cfg = defaults().clone();
        cfg.pow_enabled = true;
        cfg.pow_difficulty = 15;

        let hint = derive_escalation_hint(
            &cfg,
            &allowed_actions_v1(),
            &[family(
                "likely_human_friction",
                "outside_budget",
                "partially_supported",
            )],
        );

        assert_eq!(hint.decision, "config_tuning_candidate");
        assert_eq!(hint.review_status, "manual_review_required");
        assert_eq!(hint.problem_class, "likely_human_friction_overspend");
        assert_eq!(hint.guidance_status, "exact_move_guidance");
        assert_eq!(hint.tractability, "exact_config_move");
        assert_eq!(
            hint.recommended_action_family.as_deref(),
            Some("proof_of_work")
        );
        assert!(hint
            .trigger_family_ids
            .contains(&"likely_human_friction".to_string()));
        assert!(hint
            .trigger_metric_ids
            .contains(&"likely_human_friction_metric".to_string()));
        assert_eq!(
            hint.shortfall_guidance[0].recommended_action_family.as_deref(),
            Some("proof_of_work")
        );
    }

    #[test]
    fn escalation_hint_stays_observe_longer_without_outside_budget_families() {
        let cfg = defaults().clone();
        let hint = derive_escalation_hint(
            &cfg,
            &allowed_actions_v1(),
            &[family("suspicious_origin_cost", "near_limit", "supported")],
        );

        assert_eq!(hint.decision, "observe_longer");
        assert_eq!(hint.review_status, "manual_review_required");
        assert_eq!(hint.guidance_status, "insufficient_evidence");
        assert_eq!(hint.tractability, "observe_only");
        assert!(hint.blockers.contains(&"near_limit_only".to_string()));
    }

    #[test]
    fn escalation_hint_does_not_surface_operator_owned_policy_families_as_tuning_candidates() {
        let cfg = defaults().clone();
        let hint = derive_escalation_hint(
            &cfg,
            &allowed_actions_v1(),
            &[family(
                "non_human_category_posture",
                "outside_budget",
                "supported",
            )],
        );

        assert_eq!(hint.decision, "code_evolution_candidate");
        assert_eq!(hint.problem_class, "category_target_shortfall");
        assert!(hint.candidate_action_families.is_empty());
        assert!(hint
            .blockers
            .contains(&"operator_owned_rule_surface".to_string()));
    }

    #[test]
    fn suspicious_origin_hint_marks_threshold_tightening_as_bounded_heuristic() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        cfg.cdp_detection_enabled = true;
        cfg.pow_enabled = true;
        cfg.pow_difficulty = 15;

        let hint = derive_escalation_hint(
            &cfg,
            &allowed_actions_v1(),
            &[family(
                "suspicious_origin_cost",
                "outside_budget",
                "supported",
            )],
        );

        assert_eq!(hint.decision, "config_tuning_candidate");
        assert_eq!(hint.problem_class, "suspicious_origin_cost_overspend");
        assert_eq!(hint.guidance_status, "bounded_heuristic_guidance");
        assert_eq!(hint.tractability, "family_policy_choice");
        assert_eq!(
            hint.recommended_action_family.as_deref(),
            Some("proof_of_work")
        );
    }
}
