use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::{AllowedActionsSurface, Config};
use crate::observability::replay_promotion::ReplayPromotionSummary;

pub(crate) const OVERSIGHT_VERIFICATION_WATCH_LIVE_BUDGET_WINDOW: &str =
    "watch_live_budget_window";
pub(crate) const OVERSIGHT_VERIFICATION_RERUN_ADVERSARY_SIM: &str = "rerun_adversary_sim";
pub(crate) const OVERSIGHT_VERIFICATION_REVIEW_REPLAY_PROMOTION: &str =
    "review_replay_promotion_lineage";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OversightPressure {
    ReduceLikelyHumanFriction,
    ReduceSuspiciousOriginCost,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightPatchProposal {
    pub patch_family: String,
    pub patch: serde_json::Value,
    pub expected_impact: String,
    pub confidence: String,
    pub required_verification: Vec<String>,
    pub controller_status: String,
    pub canary_requirement: String,
    pub matched_group_ids: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OversightPatchPolicyError {
    NoCandidateFamily,
    UnsupportedCandidateFamily(String),
    NoBoundedPatch(String),
    InvalidPatch(String),
}

pub(crate) fn propose_patch(
    cfg: &Config,
    allowed_actions: &AllowedActionsSurface,
    candidate_families: &[String],
    pressure: OversightPressure,
    replay_promotion: &ReplayPromotionSummary,
) -> Result<OversightPatchProposal, OversightPatchPolicyError> {
    if candidate_families.is_empty() {
        return Err(OversightPatchPolicyError::NoCandidateFamily);
    }

    let priority = match pressure {
        OversightPressure::ReduceLikelyHumanFriction => &[
            "proof_of_work",
            "botness",
            "challenge",
            "not_a_bot",
            "maze_core",
            "core_policy",
        ][..],
        OversightPressure::ReduceSuspiciousOriginCost => &[
            "fingerprint_signal",
            "cdp_detection",
            "proof_of_work",
            "botness",
            "challenge",
            "not_a_bot",
            "maze_core",
            "core_policy",
        ][..],
    };

    for family in priority {
        if !candidate_families.iter().any(|candidate| candidate == family) {
            continue;
        }
        if !family_is_proposable(allowed_actions, family) {
            return Err(OversightPatchPolicyError::UnsupportedCandidateFamily(
                (*family).to_string(),
            ));
        }
        if let Some(patch) = family_patch(cfg, allowed_actions, family, pressure) {
            return build_proposal(
                allowed_actions,
                family,
                patch,
                pressure,
                replay_promotion,
            );
        }
    }

    Err(OversightPatchPolicyError::NoBoundedPatch(
        candidate_families.join(","),
    ))
}

fn build_proposal(
    allowed_actions: &AllowedActionsSurface,
    family: &str,
    patch: serde_json::Value,
    pressure: OversightPressure,
    replay_promotion: &ReplayPromotionSummary,
) -> Result<OversightPatchProposal, OversightPatchPolicyError> {
    let patch_object = patch
        .as_object()
        .ok_or_else(|| OversightPatchPolicyError::InvalidPatch(family.to_string()))?;
    let groups = matched_groups_for_patch(allowed_actions, family, patch_object.keys())?;
    let controller_status = dominant_group_value(groups.as_slice(), |group| &group.controller_status);
    let canary_requirement =
        dominant_group_value(groups.as_slice(), |group| &group.canary_requirement);
    let mut required_verification = vec![
        OVERSIGHT_VERIFICATION_WATCH_LIVE_BUDGET_WINDOW.to_string(),
        OVERSIGHT_VERIFICATION_RERUN_ADVERSARY_SIM.to_string(),
    ];
    if !replay_promotion.tuning_eligible
        || replay_promotion.blocking_required
        || replay_promotion.pending_owner_review_count > 0
    {
        required_verification.push(
            OVERSIGHT_VERIFICATION_REVIEW_REPLAY_PROMOTION.to_string(),
        );
    }

    let (expected_impact, confidence) = match (family, pressure) {
        ("fingerprint_signal", OversightPressure::ReduceSuspiciousOriginCost) => (
            "Tighten low-friction automated client discrimination before more human-visible defences move.".to_string(),
            "high".to_string(),
        ),
        ("cdp_detection", OversightPressure::ReduceSuspiciousOriginCost) => (
            "Tighten automation-detection coverage to reduce suspicious forwarded cost while keeping human-facing friction low.".to_string(),
            "high".to_string(),
        ),
        ("proof_of_work", OversightPressure::ReduceLikelyHumanFriction) => (
            "Ease proof-of-work burden to reduce likely-human friction while keeping other defences unchanged.".to_string(),
            "medium".to_string(),
        ),
        ("challenge", OversightPressure::ReduceLikelyHumanFriction) => (
            "Relax challenge enablement slightly to reduce likely-human challenge exposure.".to_string(),
            "medium".to_string(),
        ),
        ("botness", OversightPressure::ReduceLikelyHumanFriction) => (
            "Raise the challenge-escalation threshold slightly so likely humans stay on the lower-friction path more often.".to_string(),
            "medium".to_string(),
        ),
        ("not_a_bot", OversightPressure::ReduceLikelyHumanFriction) => (
            "Raise not-a-bot threshold slightly so likely humans clear the low-friction path more often.".to_string(),
            "medium".to_string(),
        ),
        ("maze_core", OversightPressure::ReduceLikelyHumanFriction) => (
            "Reduce maze enforcement intensity for likely-human traffic while leaving other families unchanged.".to_string(),
            "medium".to_string(),
        ),
        ("core_policy", OversightPressure::ReduceLikelyHumanFriction) => (
            "Relax the JS-required gate to reduce likely-human friction when current challenge surfaces appear too expensive.".to_string(),
            "medium".to_string(),
        ),
        ("proof_of_work", OversightPressure::ReduceSuspiciousOriginCost) => (
            "Increase proof-of-work cost slightly to reduce suspicious forwarded request and byte share.".to_string(),
            "medium".to_string(),
        ),
        ("challenge", OversightPressure::ReduceSuspiciousOriginCost) => (
            "Enable the puzzle challenge as a bounded next step to reduce suspicious forwarded cost while keeping the change reviewable.".to_string(),
            "medium".to_string(),
        ),
        ("botness", OversightPressure::ReduceSuspiciousOriginCost) => (
            "Lower the challenge-escalation threshold slightly so suspicious traffic hits the puzzle lane earlier.".to_string(),
            "medium".to_string(),
        ),
        ("not_a_bot", OversightPressure::ReduceSuspiciousOriginCost) => (
            "Tighten the not-a-bot gate slightly to reduce suspicious origin reach.".to_string(),
            "medium".to_string(),
        ),
        ("maze_core", OversightPressure::ReduceSuspiciousOriginCost) => (
            "Increase maze enforcement intensity to reduce suspicious forwarded work and byte cost.".to_string(),
            "medium".to_string(),
        ),
        ("core_policy", OversightPressure::ReduceSuspiciousOriginCost) => (
            "Re-enable the JS-required gate as a bounded way to shift suspicious traffic cost away from origin.".to_string(),
            "medium".to_string(),
        ),
        _ => (
            "Adjust the selected bounded config family in the direction implied by the benchmark pressure.".to_string(),
            "medium".to_string(),
        ),
    };

    Ok(OversightPatchProposal {
        patch_family: family.to_string(),
        patch,
        expected_impact,
        confidence,
        required_verification,
        controller_status,
        canary_requirement,
        matched_group_ids: groups.iter().map(|group| group.group_id.clone()).collect(),
        note: groups
            .iter()
            .map(|group| group.note.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    })
}

fn family_patch(
    cfg: &Config,
    allowed_actions: &AllowedActionsSurface,
    family: &str,
    pressure: OversightPressure,
) -> Option<serde_json::Value> {
    match (family, pressure) {
        ("core_policy", OversightPressure::ReduceLikelyHumanFriction) => cfg
            .js_required_enforced
            .then(|| json!({ "js_required_enforced": false })),
        ("core_policy", OversightPressure::ReduceSuspiciousOriginCost) => (!cfg
            .js_required_enforced)
            .then(|| json!({ "js_required_enforced": true })),
        ("proof_of_work", OversightPressure::ReduceLikelyHumanFriction) => {
            step_numeric_path(
                allowed_actions,
                "pow_difficulty",
                cfg.pow_difficulty as u64,
                StepDirection::Down,
            )
            .map(|value| json!({ "pow_difficulty": value }))
        }
        ("proof_of_work", OversightPressure::ReduceSuspiciousOriginCost) => {
            if !cfg.pow_enabled {
                Some(json!({ "pow_enabled": true }))
            } else {
                step_numeric_path(
                    allowed_actions,
                    "pow_difficulty",
                    cfg.pow_difficulty as u64,
                    StepDirection::Up,
                )
                .map(|value| json!({ "pow_difficulty": value }))
            }
        }
        ("challenge", OversightPressure::ReduceLikelyHumanFriction) => {
            cfg.challenge_puzzle_enabled
                .then(|| json!({ "challenge_puzzle_enabled": false }))
        }
        ("challenge", OversightPressure::ReduceSuspiciousOriginCost) => {
            if !cfg.challenge_puzzle_enabled {
                Some(json!({ "challenge_puzzle_enabled": true }))
            } else {
                None
            }
        }
        ("botness", OversightPressure::ReduceLikelyHumanFriction) => {
            step_numeric_path(
                allowed_actions,
                "challenge_puzzle_risk_threshold",
                cfg.challenge_puzzle_risk_threshold as u64,
                StepDirection::Up,
            )
            .map(|value| json!({ "challenge_puzzle_risk_threshold": value }))
        }
        ("botness", OversightPressure::ReduceSuspiciousOriginCost) => {
            step_numeric_path(
                allowed_actions,
                "challenge_puzzle_risk_threshold",
                cfg.challenge_puzzle_risk_threshold as u64,
                StepDirection::Down,
            )
            .filter(|value| *value > cfg.not_a_bot_risk_threshold as u64)
            .map(|value| json!({ "challenge_puzzle_risk_threshold": value }))
        }
        ("not_a_bot", OversightPressure::ReduceLikelyHumanFriction) => {
            let upper_bound = cfg
                .challenge_puzzle_risk_threshold
                .saturating_sub(1)
                .max(cfg.not_a_bot_risk_threshold);
            step_numeric_path(
                allowed_actions,
                "not_a_bot_risk_threshold",
                cfg.not_a_bot_risk_threshold as u64,
                StepDirection::Up,
            )
            .filter(|value| *value <= upper_bound as u64)
            .map(|value| json!({ "not_a_bot_risk_threshold": value }))
        }
        ("not_a_bot", OversightPressure::ReduceSuspiciousOriginCost) => {
            if !cfg.not_a_bot_enabled {
                Some(json!({ "not_a_bot_enabled": true }))
            } else {
                step_numeric_path(
                    allowed_actions,
                    "not_a_bot_risk_threshold",
                    cfg.not_a_bot_risk_threshold as u64,
                    StepDirection::Down,
                )
                .map(|value| json!({ "not_a_bot_risk_threshold": value }))
            }
        }
        ("maze_core", OversightPressure::ReduceLikelyHumanFriction) => {
            if cfg.maze_rollout_phase.as_str() == "enforce" {
                Some(json!({ "maze_rollout_phase": "advisory" }))
            } else {
                None
            }
        }
        ("maze_core", OversightPressure::ReduceSuspiciousOriginCost) => {
            if !cfg.maze_enabled {
                Some(json!({ "maze_enabled": true }))
            } else if cfg.maze_rollout_phase.as_str() != "enforce" {
                Some(json!({ "maze_rollout_phase": "enforce" }))
            } else if !cfg.maze_auto_ban {
                Some(json!({ "maze_auto_ban": true }))
            } else {
                None
            }
        }
        ("cdp_detection", OversightPressure::ReduceSuspiciousOriginCost) => (!cfg
            .cdp_detection_enabled)
            .then(|| json!({ "cdp_detection_enabled": true })),
        ("fingerprint_signal", OversightPressure::ReduceSuspiciousOriginCost) => (!cfg
            .fingerprint_signal_enabled)
            .then(|| json!({ "fingerprint_signal_enabled": true })),
        _ => None,
    }
}

fn family_is_proposable(allowed_actions: &AllowedActionsSurface, family: &str) -> bool {
    allowed_actions.groups.iter().any(|group| {
        group.family == family
            && group.controller_status == "allowed"
            && !group.proposable_patch_paths.is_empty()
    })
}

fn matched_groups_for_patch<'a>(
    allowed_actions: &'a AllowedActionsSurface,
    family: &str,
    keys: impl Iterator<Item = &'a String>,
) -> Result<Vec<GroupSummary>, OversightPatchPolicyError> {
    let mut groups = Vec::new();
    for key in keys {
        let group = allowed_actions
            .groups
            .iter()
            .find(|group| {
                group.family == family
                    && group.proposable_patch_paths.iter().any(|path| path == key)
                    && group.controller_status == "allowed"
            })
            .ok_or_else(|| OversightPatchPolicyError::InvalidPatch(key.clone()))?;
        if !groups
            .iter()
            .any(|existing: &GroupSummary| existing.group_id == group.group_id)
        {
            groups.push(GroupSummary {
                group_id: group.group_id.clone(),
                controller_status: group.controller_status.clone(),
                canary_requirement: group.canary_requirement.clone(),
                note: group.note.clone(),
            });
        }
    }
    Ok(groups)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupSummary {
    group_id: String,
    controller_status: String,
    canary_requirement: String,
    note: String,
}

fn dominant_group_value(
    groups: &[GroupSummary],
    selector: impl Fn(&GroupSummary) -> &String,
) -> String {
    if groups
        .iter()
        .any(|group| selector(group).as_str() == "manual_only")
    {
        "manual_only".to_string()
    } else if groups
        .iter()
        .any(|group| selector(group).as_str() == "required")
    {
        "required".to_string()
    } else {
        groups
            .first()
            .map(|group| selector(group).clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepDirection {
    Down,
    Up,
}

fn step_numeric_path(
    allowed_actions: &AllowedActionsSurface,
    path: &str,
    current: u64,
    direction: StepDirection,
) -> Option<u64> {
    let constraint = allowed_actions
        .groups
        .iter()
        .flat_map(|group| group.value_constraints.iter())
        .find(|constraint| constraint.path == path)?;
    let min = constraint
        .min_inclusive
        .map(|value| value.max(0.0) as u64)
        .unwrap_or(0);
    let max = constraint
        .max_inclusive
        .map(|value| value.max(0.0) as u64)
        .unwrap_or(current);
    match direction {
        StepDirection::Down if current > min => Some(current.saturating_sub(1).max(min)),
        StepDirection::Up if current < max => Some(current.saturating_add(1).min(max)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        propose_patch, OversightPatchPolicyError, OversightPressure,
        OVERSIGHT_VERIFICATION_REVIEW_REPLAY_PROMOTION,
    };
    use crate::config::{allowed_actions_v1, defaults};
    use crate::observability::replay_promotion::ReplayPromotionSummary;

    #[test]
    fn suspicious_cost_policy_prefers_low_friction_signal_families_first() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        cfg.pow_enabled = true;
        cfg.pow_difficulty = 15;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["proof_of_work".to_string(), "fingerprint_signal".to_string()],
            OversightPressure::ReduceSuspiciousOriginCost,
            &ReplayPromotionSummary::not_materialized(),
        )
        .expect("proposal builds");

        assert_eq!(proposal.patch_family, "fingerprint_signal");
        assert_eq!(proposal.patch["fingerprint_signal_enabled"], true);
    }

    #[test]
    fn friction_policy_respects_not_a_bot_challenge_guardrail() {
        let mut cfg = defaults().clone();
        cfg.not_a_bot_risk_threshold = 2;
        cfg.challenge_puzzle_risk_threshold = 4;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["not_a_bot".to_string()],
            OversightPressure::ReduceLikelyHumanFriction,
            &ReplayPromotionSummary::not_materialized(),
        )
        .expect("proposal builds");

        assert_eq!(proposal.patch_family, "not_a_bot");
        assert_eq!(proposal.patch["not_a_bot_risk_threshold"], 3);
    }

    #[test]
    fn challenge_threshold_tuning_routes_through_botness_family() {
        let mut cfg = defaults().clone();
        cfg.not_a_bot_risk_threshold = 2;
        cfg.challenge_puzzle_risk_threshold = 4;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["botness".to_string()],
            OversightPressure::ReduceLikelyHumanFriction,
            &ReplayPromotionSummary::not_materialized(),
        )
        .expect("proposal builds");

        assert_eq!(proposal.patch_family, "botness");
        assert_eq!(proposal.patch["challenge_puzzle_risk_threshold"], 5);
    }

    #[test]
    fn replay_promotion_blocker_adds_review_requirement() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        let mut replay = ReplayPromotionSummary::not_materialized();
        replay.blocking_required = true;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["fingerprint_signal".to_string()],
            OversightPressure::ReduceSuspiciousOriginCost,
            &replay,
        )
        .expect("proposal builds");

        assert!(proposal
            .required_verification
            .contains(&OVERSIGHT_VERIFICATION_REVIEW_REPLAY_PROMOTION.to_string()));
    }

    #[test]
    fn advisory_only_replay_promotion_requires_review_verification() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["fingerprint_signal".to_string()],
            OversightPressure::ReduceSuspiciousOriginCost,
            &ReplayPromotionSummary::not_materialized(),
        )
        .expect("proposal builds");

        assert!(proposal
            .required_verification
            .contains(&OVERSIGHT_VERIFICATION_REVIEW_REPLAY_PROMOTION.to_string()));
    }

    #[test]
    fn policy_returns_no_bounded_patch_when_selected_family_cannot_move_safely() {
        let mut cfg = defaults().clone();
        cfg.pow_difficulty = 12;

        let err = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["proof_of_work".to_string()],
            OversightPressure::ReduceLikelyHumanFriction,
            &ReplayPromotionSummary::not_materialized(),
        )
        .expect_err("no lower bounded patch exists");

        assert_eq!(
            err,
            OversightPatchPolicyError::NoBoundedPatch("proof_of_work".to_string())
        );
    }
}
