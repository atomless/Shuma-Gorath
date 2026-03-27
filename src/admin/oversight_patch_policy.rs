use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::{
    controller_action_family_risk_profile, AllowedActionsSurface, Config,
};
use crate::observability::benchmark_results::BenchmarkProtectedEvidenceSummary;

pub(crate) const OVERSIGHT_VERIFICATION_WATCH_LIVE_BUDGET_WINDOW: &str =
    "watch_live_budget_window";
pub(crate) const OVERSIGHT_VERIFICATION_RERUN_ADVERSARY_SIM: &str = "rerun_adversary_sim";
pub(crate) const OVERSIGHT_VERIFICATION_REVIEW_REPLAY_PROMOTION: &str =
    "review_replay_promotion_lineage";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OversightProblemClass {
    LikelyHumanFrictionOverspend,
    SuspiciousOriginReachOverspend,
    SuspiciousOriginLatencyOverspend,
    ScraplingExploitProgressGap,
}

impl OversightProblemClass {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            OversightProblemClass::LikelyHumanFrictionOverspend => {
                "likely_human_friction_overspend"
            }
            OversightProblemClass::SuspiciousOriginReachOverspend => {
                "suspicious_forwarded_reach_overspend"
            }
            OversightProblemClass::SuspiciousOriginLatencyOverspend => {
                "suspicious_forwarded_latency_overspend"
            }
            OversightProblemClass::ScraplingExploitProgressGap => {
                "scrapling_exploit_progress_gap"
            }
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightPatchCandidate {
    pub family: String,
    pub priority_rank: usize,
    pub proposal: OversightPatchProposal,
    pub likely_human_risk: String,
    pub tolerated_non_human_risk: String,
    pub blast_radius: String,
    pub patch_size: usize,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OversightPatchPolicyError {
    NoCandidateFamily,
    UnsupportedCandidateFamily(String),
    NoBoundedPatch(String),
    InvalidPatch(String),
}

#[cfg(test)]
pub(crate) fn propose_patch(
    cfg: &Config,
    allowed_actions: &AllowedActionsSurface,
    candidate_families: &[String],
    problem_class: OversightProblemClass,
    protected_evidence: &BenchmarkProtectedEvidenceSummary,
) -> Result<OversightPatchProposal, OversightPatchPolicyError> {
    Ok(rank_patch_candidates(
        cfg,
        allowed_actions,
        candidate_families,
        problem_class,
        protected_evidence,
    )?
    .into_iter()
    .next()
    .expect("ranked candidates must contain at least one item")
    .proposal)
}

pub(crate) fn rank_patch_candidates(
    cfg: &Config,
    allowed_actions: &AllowedActionsSurface,
    candidate_families: &[String],
    problem_class: OversightProblemClass,
    protected_evidence: &BenchmarkProtectedEvidenceSummary,
) -> Result<Vec<OversightPatchCandidate>, OversightPatchPolicyError> {
    if candidate_families.is_empty() {
        return Err(OversightPatchPolicyError::NoCandidateFamily);
    }

    if let Some(family) = first_non_proposable_candidate_family(allowed_actions, candidate_families)
    {
        return Err(OversightPatchPolicyError::UnsupportedCandidateFamily(
            family.to_string(),
        ));
    }

    let ordered_families: Vec<&str> = if matches!(
        problem_class,
        OversightProblemClass::ScraplingExploitProgressGap
    ) {
        candidate_families.iter().map(String::as_str).collect()
    } else {
        candidate_priority(problem_class).to_vec()
    };

    let mut ranked = Vec::new();
    for (index, family) in ordered_families.iter().enumerate() {
        if !candidate_families.iter().any(|candidate| candidate == family) {
            continue;
        }
        if !family_is_proposable(allowed_actions, family) {
            return Err(OversightPatchPolicyError::UnsupportedCandidateFamily(
                (*family).to_string(),
            ));
        }
        if let Some(patch) = family_patch(cfg, allowed_actions, family, problem_class) {
            let proposal = build_proposal(
                allowed_actions,
                family,
                patch,
                problem_class,
                protected_evidence,
            )?;
            let (likely_human_risk, tolerated_non_human_risk, risk_note) =
                controller_action_family_risk_profile(family)
                    .map(|risk| {
                        (
                            risk.likely_human_risk,
                            risk.tolerated_non_human_risk,
                            risk.note,
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            "unknown".to_string(),
                            "unknown".to_string(),
                            "No canonical risk profile is currently materialized for this family."
                                .to_string(),
                        )
                    });
            ranked.push(OversightPatchCandidate {
                family: (*family).to_string(),
                priority_rank: index + 1,
                blast_radius: match proposal.matched_group_ids.len() {
                    0 | 1 => "single_group".to_string(),
                    2 => "family_bounded".to_string(),
                    _ => "multi_group".to_string(),
                },
                patch_size: proposal.patch.as_object().map(|patch| patch.len()).unwrap_or(0),
                likely_human_risk,
                tolerated_non_human_risk,
                note: format!("{} {}", risk_note, proposal.note).trim().to_string(),
                proposal,
            });
        }
    }

    if ranked.is_empty() {
        Err(OversightPatchPolicyError::NoBoundedPatch(
            candidate_families.join(","),
        ))
    } else {
        Ok(ranked)
    }
}

fn candidate_priority(problem_class: OversightProblemClass) -> &'static [&'static str] {
    match problem_class {
        OversightProblemClass::LikelyHumanFrictionOverspend => &[
            "proof_of_work",
            "challenge",
            "not_a_bot",
            "maze_core",
            "core_policy",
        ][..],
        OversightProblemClass::SuspiciousOriginReachOverspend => &[
            "fingerprint_signal",
            "cdp_detection",
            "proof_of_work",
            "challenge",
            "not_a_bot",
            "maze_core",
            "core_policy",
        ][..],
        OversightProblemClass::SuspiciousOriginLatencyOverspend => &[
            "fingerprint_signal",
            "cdp_detection",
            "proof_of_work",
            "challenge",
            "not_a_bot",
            "maze_core",
            "core_policy",
        ][..],
        OversightProblemClass::ScraplingExploitProgressGap => &[][..],
    }
}

fn build_proposal(
    allowed_actions: &AllowedActionsSurface,
    family: &str,
    patch: serde_json::Value,
    problem_class: OversightProblemClass,
    protected_evidence: &BenchmarkProtectedEvidenceSummary,
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
    if protected_evidence.protected_basis == "replay_promoted_lineage"
        && !protected_evidence.tuning_eligible
    {
        required_verification.push(
            OVERSIGHT_VERIFICATION_REVIEW_REPLAY_PROMOTION.to_string(),
        );
    }

    let (expected_impact, confidence) = match (family, problem_class) {
        (
            "fingerprint_signal",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (
            "Tighten low-friction automated client discrimination at the observed breach loci before broader human-visible defences move.".to_string(),
            "high".to_string(),
        ),
        (
            "cdp_detection",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (
            "Tighten browser automation detection at the observed breach loci while keeping human-facing friction low.".to_string(),
            "high".to_string(),
        ),
        ("proof_of_work", OversightProblemClass::LikelyHumanFrictionOverspend) => (
            "Ease proof-of-work burden to reduce likely-human friction while keeping other defences unchanged.".to_string(),
            "medium".to_string(),
        ),
        ("challenge", OversightProblemClass::LikelyHumanFrictionOverspend) => (
            "Raise challenge thresholds slightly to reduce likely-human challenge exposure.".to_string(),
            "medium".to_string(),
        ),
        ("not_a_bot", OversightProblemClass::LikelyHumanFrictionOverspend) => (
            "Raise not-a-bot threshold slightly so likely humans clear the low-friction path more often.".to_string(),
            "medium".to_string(),
        ),
        ("maze_core", OversightProblemClass::LikelyHumanFrictionOverspend) => (
            "Reduce maze enforcement intensity for likely-human traffic while leaving other families unchanged.".to_string(),
            "medium".to_string(),
        ),
        ("core_policy", OversightProblemClass::LikelyHumanFrictionOverspend) => (
            "Relax the JS-required gate to reduce likely-human friction when current challenge surfaces appear too expensive.".to_string(),
            "medium".to_string(),
        ),
        (
            "proof_of_work",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (
            "Increase proof-of-work cost slightly at the implicated breach loci to reduce non-human progress.".to_string(),
            "medium".to_string(),
        ),
        (
            "challenge",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (
            "Tighten challenge posture at the implicated breach loci while keeping the move bounded.".to_string(),
            "medium".to_string(),
        ),
        (
            "not_a_bot",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (
            "Tighten the not-a-bot gate at the implicated breach loci.".to_string(),
            "medium".to_string(),
        ),
        (
            "maze_core",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (
            "Increase maze enforcement intensity at the implicated breach loci to reduce non-human progress.".to_string(),
            "medium".to_string(),
        ),
        (
            "core_policy",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (
            "Re-enable the JS-required gate as a bounded way to close the observed browser-side breach path.".to_string(),
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
    problem_class: OversightProblemClass,
) -> Option<serde_json::Value> {
    match (family, problem_class) {
        ("core_policy", OversightProblemClass::LikelyHumanFrictionOverspend) => cfg
            .js_required_enforced
            .then(|| json!({ "js_required_enforced": false })),
        (
            "core_policy",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (!cfg
            .js_required_enforced)
            .then(|| json!({ "js_required_enforced": true })),
        ("proof_of_work", OversightProblemClass::LikelyHumanFrictionOverspend) => {
            step_numeric_path(
                allowed_actions,
                "pow_difficulty",
                cfg.pow_difficulty as u64,
                StepDirection::Down,
            )
            .map(|value| json!({ "pow_difficulty": value }))
        }
        (
            "proof_of_work",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => {
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
        ("challenge", OversightProblemClass::LikelyHumanFrictionOverspend) => {
            step_numeric_path(
                allowed_actions,
                "challenge_puzzle_risk_threshold",
                cfg.challenge_puzzle_risk_threshold as u64,
                StepDirection::Up,
            )
            .map(|value| json!({ "challenge_puzzle_risk_threshold": value }))
        }
        (
            "challenge",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => {
            if !cfg.challenge_puzzle_enabled {
                Some(json!({ "challenge_puzzle_enabled": true }))
            } else {
                step_numeric_path(
                    allowed_actions,
                    "challenge_puzzle_risk_threshold",
                    cfg.challenge_puzzle_risk_threshold as u64,
                    StepDirection::Down,
                )
                .filter(|value| *value > cfg.not_a_bot_risk_threshold as u64)
                .map(|value| json!({ "challenge_puzzle_risk_threshold": value }))
            }
        }
        ("not_a_bot", OversightProblemClass::LikelyHumanFrictionOverspend) => {
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
        (
            "not_a_bot",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => {
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
        ("maze_core", OversightProblemClass::LikelyHumanFrictionOverspend) => {
            if cfg.maze_rollout_phase.as_str() == "enforce" {
                Some(json!({ "maze_rollout_phase": "advisory" }))
            } else {
                None
            }
        }
        (
            "maze_core",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => {
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
        (
            "cdp_detection",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (!cfg
            .cdp_detection_enabled)
            .then(|| json!({ "cdp_detection_enabled": true })),
        (
            "fingerprint_signal",
            OversightProblemClass::SuspiciousOriginReachOverspend
            | OversightProblemClass::SuspiciousOriginLatencyOverspend
            | OversightProblemClass::ScraplingExploitProgressGap,
        ) => (!cfg
            .fingerprint_signal_enabled)
            .then(|| json!({ "fingerprint_signal_enabled": true })),
        _ => None,
    }
}

fn family_is_proposable(allowed_actions: &AllowedActionsSurface, family: &str) -> bool {
    allowed_actions.groups.iter().any(|group| {
        group.family == family && group.controller_status == "allowed"
    })
}

fn first_non_proposable_candidate_family<'a>(
    allowed_actions: &'a AllowedActionsSurface,
    candidate_families: &'a [String],
) -> Option<&'a str> {
    candidate_families
        .iter()
        .map(String::as_str)
        .find(|family| family_is_known(allowed_actions, family) && !family_is_proposable(allowed_actions, family))
}

fn family_is_known(allowed_actions: &AllowedActionsSurface, family: &str) -> bool {
    allowed_actions
        .groups
        .iter()
        .any(|group| group.family == family)
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
                    && group.patch_paths.iter().any(|path| path == key)
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
        propose_patch, rank_patch_candidates, OversightPatchPolicyError, OversightProblemClass,
        OVERSIGHT_VERIFICATION_REVIEW_REPLAY_PROMOTION,
    };
    use crate::config::{allowed_actions_v1, defaults};
    use crate::observability::benchmark_results::{
        unavailable_benchmark_protected_evidence_summary, BenchmarkProtectedEvidenceSummary,
    };

    fn replay_review_required_protected_evidence() -> BenchmarkProtectedEvidenceSummary {
        BenchmarkProtectedEvidenceSummary {
            availability: "materialized".to_string(),
            evidence_status: "advisory_only".to_string(),
            tuning_eligible: false,
            protected_basis: "replay_promoted_lineage".to_string(),
            protected_lineage_count: 1,
            eligibility_blockers: vec!["replay_promotion_owner_review_pending".to_string()],
            note: "Replay lineage is materialized but still advisory.".to_string(),
        }
    }

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
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
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
            OversightProblemClass::LikelyHumanFrictionOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect("proposal builds");

        assert_eq!(proposal.patch_family, "not_a_bot");
        assert_eq!(proposal.patch["not_a_bot_risk_threshold"], 3);
    }

    #[test]
    fn challenge_threshold_patch_matches_challenge_family_metadata() {
        let mut cfg = defaults().clone();
        cfg.challenge_puzzle_risk_threshold = 5;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["challenge".to_string()],
            OversightProblemClass::LikelyHumanFrictionOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect("proposal builds");

        assert_eq!(proposal.patch_family, "challenge");
        assert_eq!(proposal.patch["challenge_puzzle_risk_threshold"], 6);
        assert_eq!(proposal.matched_group_ids, vec!["challenge.policy".to_string()]);
        assert_eq!(proposal.controller_status, "allowed");
    }

    #[test]
    fn mixed_family_with_allowed_group_remains_proposable() {
        let mut cfg = defaults().clone();
        cfg.js_required_enforced = true;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["core_policy".to_string()],
            OversightProblemClass::LikelyHumanFrictionOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect("proposal builds");

        assert_eq!(proposal.patch_family, "core_policy");
        assert_eq!(proposal.patch["js_required_enforced"], false);
        assert_eq!(
            proposal.matched_group_ids,
            vec!["core_policy.js_required_toggle".to_string()]
        );
    }

    #[test]
    fn forbidden_verified_identity_family_is_rejected() {
        let err = propose_patch(
            &defaults(),
            &allowed_actions_v1(),
            &["verified_identity".to_string()],
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect_err("verified identity must remain controller-forbidden");

        assert_eq!(
            err,
            OversightPatchPolicyError::UnsupportedCandidateFamily(
                "verified_identity".to_string()
            )
        );
    }

    #[test]
    fn forbidden_provider_selection_family_is_rejected() {
        let err = propose_patch(
            &defaults(),
            &allowed_actions_v1(),
            &["provider_selection".to_string()],
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect_err("provider selection must remain controller-forbidden");

        assert_eq!(
            err,
            OversightPatchPolicyError::UnsupportedCandidateFamily(
                "provider_selection".to_string()
            )
        );
    }

    #[test]
    fn forbidden_robots_policy_family_is_rejected() {
        let err = propose_patch(
            &defaults(),
            &allowed_actions_v1(),
            &["robots_policy".to_string()],
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect_err("robots policy must remain controller-forbidden");

        assert_eq!(
            err,
            OversightPatchPolicyError::UnsupportedCandidateFamily("robots_policy".to_string())
        );
    }

    #[test]
    fn forbidden_allowlists_family_is_rejected() {
        let err = propose_patch(
            &defaults(),
            &allowed_actions_v1(),
            &["allowlists".to_string()],
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect_err("allowlists must remain controller-forbidden");

        assert_eq!(
            err,
            OversightPatchPolicyError::UnsupportedCandidateFamily("allowlists".to_string())
        );
    }

    #[test]
    fn forbidden_tarpit_family_is_rejected() {
        let err = propose_patch(
            &defaults(),
            &allowed_actions_v1(),
            &["tarpit".to_string()],
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect_err("tarpit must remain controller-forbidden");

        assert_eq!(
            err,
            OversightPatchPolicyError::UnsupportedCandidateFamily("tarpit".to_string())
        );
    }

    #[test]
    fn replay_promotion_blocker_adds_review_requirement() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["fingerprint_signal".to_string()],
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &replay_review_required_protected_evidence(),
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
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &replay_review_required_protected_evidence(),
        )
        .expect("proposal builds");

        assert!(proposal
            .required_verification
            .contains(&OVERSIGHT_VERIFICATION_REVIEW_REPLAY_PROMOTION.to_string()));
    }

    #[test]
    fn live_runtime_protected_evidence_does_not_require_replay_review() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &["fingerprint_signal".to_string()],
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &BenchmarkProtectedEvidenceSummary {
                availability: "materialized".to_string(),
                evidence_status: "protected".to_string(),
                tuning_eligible: true,
                protected_basis: "live_scrapling_runtime".to_string(),
                protected_lineage_count: 0,
                eligibility_blockers: Vec::new(),
                note: "Strong live Scrapling runtime evidence is protected.".to_string(),
            },
        )
        .expect("proposal builds");

        assert!(!proposal
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
            OversightProblemClass::LikelyHumanFrictionOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect_err("no lower bounded patch exists");

        assert_eq!(
            err,
            OversightPatchPolicyError::NoBoundedPatch("proof_of_work".to_string())
        );
    }

    #[test]
    fn latency_problem_class_prefers_signal_families_before_higher_friction_moves() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        cfg.js_required_enforced = false;
        cfg.pow_enabled = true;

        let proposal = propose_patch(
            &cfg,
            &allowed_actions_v1(),
            &[
                "core_policy".to_string(),
                "proof_of_work".to_string(),
                "fingerprint_signal".to_string(),
            ],
            OversightProblemClass::SuspiciousOriginLatencyOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect("proposal builds");

        assert_eq!(proposal.patch_family, "fingerprint_signal");
    }

    #[test]
    fn rank_patch_candidates_prefers_smallest_low_friction_candidate_first() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        cfg.pow_enabled = true;
        cfg.pow_difficulty = 15;

        let ranked = rank_patch_candidates(
            &cfg,
            &allowed_actions_v1(),
            &["proof_of_work".to_string(), "fingerprint_signal".to_string()],
            OversightProblemClass::SuspiciousOriginReachOverspend,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect("ranked candidates build");

        assert_eq!(ranked[0].family, "fingerprint_signal");
        assert_eq!(ranked[0].priority_rank, 1);
        assert_eq!(ranked[0].likely_human_risk, "low");
        assert_eq!(ranked[0].blast_radius, "single_group");
        assert!(ranked[0].patch_size >= 1);
        assert_eq!(ranked[1].family, "proof_of_work");
    }

    #[test]
    fn exploit_progress_policy_preserves_localized_candidate_order() {
        let mut cfg = defaults().clone();
        cfg.challenge_puzzle_enabled = false;
        cfg.fingerprint_signal_enabled = false;

        let ranked = rank_patch_candidates(
            &cfg,
            &allowed_actions_v1(),
            &["challenge".to_string(), "fingerprint_signal".to_string()],
            OversightProblemClass::ScraplingExploitProgressGap,
            &unavailable_benchmark_protected_evidence_summary(),
        )
        .expect("ranked candidates build");

        assert_eq!(ranked[0].family, "challenge");
        assert_eq!(ranked[0].priority_rank, 1);
        assert_eq!(ranked[1].family, "fingerprint_signal");
    }
}
