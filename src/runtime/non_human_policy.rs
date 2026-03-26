use serde::{Deserialize, Serialize};

use crate::bot_identity::contracts::VerifiedIdentityEvidence;
use crate::bot_identity::policy::IdentityPolicyContext;
use crate::observability::operator_snapshot_objectives::{
    objective_profile_is_strict_human_only, OperatorObjectivesProfile,
    HUMANS_PLUS_VERIFIED_ONLY_OBJECTIVE_PROFILE_ID,
};
use crate::runtime::non_human_taxonomy::NonHumanCategoryId;
use crate::runtime::traffic_classification::verified_identity_category_assignment;

pub(crate) const EFFECTIVE_NON_HUMAN_POLICY_SCHEMA_VERSION: &str =
    "effective_non_human_policy_v1";
pub(crate) const VERIFIED_IDENTITY_OVERRIDE_MODE_STRICT_HUMAN_ONLY: &str =
    "strict_human_only";
pub(crate) const VERIFIED_IDENTITY_OVERRIDE_MODE_EXPLICIT_OVERRIDES_ELIGIBLE: &str =
    "explicit_overrides_eligible";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EffectiveNonHumanPolicyCategoryRow {
    pub category_id: NonHumanCategoryId,
    pub base_posture: String,
    pub effective_posture: String,
    pub verified_identity_handling: String,
    pub authority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct EffectiveNonHumanPolicySummary {
    pub schema_version: String,
    pub profile_id: String,
    pub objective_revision: String,
    pub verified_identity_override_mode: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<EffectiveNonHumanPolicyCategoryRow>,
}

pub(crate) fn effective_non_human_policy_summary(
    objectives: &OperatorObjectivesProfile,
) -> EffectiveNonHumanPolicySummary {
    let override_mode = verified_identity_override_mode_for_profile(objectives);
    let strict_human_only = override_mode == VERIFIED_IDENTITY_OVERRIDE_MODE_STRICT_HUMAN_ONLY;
    EffectiveNonHumanPolicySummary {
        schema_version: EFFECTIVE_NON_HUMAN_POLICY_SCHEMA_VERSION.to_string(),
        profile_id: objectives.profile_id.clone(),
        objective_revision: objectives.revision.clone(),
        verified_identity_override_mode: override_mode.to_string(),
        rows: objectives
            .category_postures
            .iter()
            .map(|row| EffectiveNonHumanPolicyCategoryRow {
                category_id: row.category_id,
                base_posture: row.posture.clone(),
                effective_posture: row.posture.clone(),
                verified_identity_handling: if strict_human_only {
                    "verified_identity_evidence_only".to_string()
                } else {
                    "explicit_override_required".to_string()
                },
                authority: if strict_human_only {
                    "operator_objectives_v1.category_postures".to_string()
                } else {
                    "operator_objectives_v1.category_postures + verified_identity_explicit_overrides"
                        .to_string()
                },
            })
            .collect(),
    }
}

pub(crate) fn verified_identity_policy_context(
    objectives: &OperatorObjectivesProfile,
    identity: &VerifiedIdentityEvidence,
) -> IdentityPolicyContext {
    let category_assignment = verified_identity_category_assignment(identity);
    let base_posture = posture_for_category(objectives, category_assignment.category_id)
        .unwrap_or("blocked")
        .to_string();
    IdentityPolicyContext {
        profile_id: objectives.profile_id.clone(),
        verified_identity_override_mode: verified_identity_override_mode_for_profile(objectives)
            .to_string(),
        canonical_category_id: category_assignment.category_id.as_str().to_string(),
        base_posture,
    }
}

pub(crate) fn verified_identity_override_mode_for_profile(
    objectives: &OperatorObjectivesProfile,
) -> &'static str {
    if objective_profile_is_strict_human_only(objectives) {
        VERIFIED_IDENTITY_OVERRIDE_MODE_STRICT_HUMAN_ONLY
    } else if objectives.profile_id == HUMANS_PLUS_VERIFIED_ONLY_OBJECTIVE_PROFILE_ID {
        VERIFIED_IDENTITY_OVERRIDE_MODE_EXPLICIT_OVERRIDES_ELIGIBLE
    } else {
        VERIFIED_IDENTITY_OVERRIDE_MODE_EXPLICIT_OVERRIDES_ELIGIBLE
    }
}

fn posture_for_category(
    objectives: &OperatorObjectivesProfile,
    category_id: NonHumanCategoryId,
) -> Option<&str> {
    objectives
        .category_postures
        .iter()
        .find(|row| row.category_id == category_id)
        .map(|row| row.posture.as_str())
}

#[cfg(test)]
mod tests {
    use super::{
        effective_non_human_policy_summary, verified_identity_override_mode_for_profile,
        VERIFIED_IDENTITY_OVERRIDE_MODE_EXPLICIT_OVERRIDES_ELIGIBLE,
        VERIFIED_IDENTITY_OVERRIDE_MODE_STRICT_HUMAN_ONLY,
    };
    use crate::observability::operator_snapshot_objectives::{
        default_operator_objectives, humans_plus_verified_only_operator_objectives,
    };

    #[test]
    fn strict_human_only_profiles_suppress_verified_identity_overrides() {
        let objectives = default_operator_objectives(1_700_000_000);

        let summary = effective_non_human_policy_summary(&objectives);

        assert_eq!(
            verified_identity_override_mode_for_profile(&objectives),
            VERIFIED_IDENTITY_OVERRIDE_MODE_STRICT_HUMAN_ONLY
        );
        assert_eq!(summary.profile_id, "human_only_private");
        assert_eq!(summary.verified_identity_override_mode, "strict_human_only");
        assert!(summary
            .rows
            .iter()
            .all(|row| row.verified_identity_handling == "verified_identity_evidence_only"));
    }

    #[test]
    fn relaxed_verified_only_profiles_keep_explicit_override_path_machine_readable() {
        let objectives = humans_plus_verified_only_operator_objectives(1_700_000_000);

        let summary = effective_non_human_policy_summary(&objectives);

        assert_eq!(
            verified_identity_override_mode_for_profile(&objectives),
            VERIFIED_IDENTITY_OVERRIDE_MODE_EXPLICIT_OVERRIDES_ELIGIBLE
        );
        assert_eq!(summary.profile_id, "humans_plus_verified_only");
        assert_eq!(
            summary.verified_identity_override_mode,
            "explicit_overrides_eligible"
        );
        assert!(summary
            .rows
            .iter()
            .all(|row| row.verified_identity_handling == "explicit_override_required"));
    }
}
