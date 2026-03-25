use serde::{Deserialize, Serialize};

use crate::bot_identity::contracts::IdentityCategory;
use crate::bot_identity::policy::{
    resolved_verified_identity_override_mode, IdentityPolicyAction,
};
use crate::config::Config;
use crate::runtime::non_human_taxonomy::{canonical_non_human_taxonomy, NonHumanCategoryId};

use super::operator_snapshot_objectives::{
    non_human_stance_preset_catalog, OperatorObjectivesProfile,
};

pub(crate) const EFFECTIVE_NON_HUMAN_POLICY_SCHEMA_VERSION: &str =
    "effective_non_human_policy_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EffectiveNonHumanPolicyVerifiedIdentityOverride {
    pub status: String,
    pub effective_action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_posture: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verified_identity_categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_of_authority: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EffectiveNonHumanPolicyRow {
    pub category_id: NonHumanCategoryId,
    pub category_label: String,
    pub base_posture: String,
    pub verified_identity_override: EffectiveNonHumanPolicyVerifiedIdentityOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EffectiveNonHumanPolicy {
    pub schema_version: String,
    pub resolution_mode: String,
    pub active_preset_id: String,
    pub verified_identity_mode: String,
    pub mismatched_category_count: usize,
    pub rows: Vec<EffectiveNonHumanPolicyRow>,
}

pub(crate) fn resolved_verified_identity_mode(cfg: &Config) -> String {
    resolved_verified_identity_override_mode(
        cfg.verified_identity.enabled,
        &cfg.verified_identity.named_policies,
        &cfg.verified_identity.category_defaults,
        &cfg.verified_identity.service_profiles,
    )
    .as_str()
    .to_string()
}

pub(crate) fn effective_non_human_policy(
    objectives: &OperatorObjectivesProfile,
    cfg: &Config,
) -> EffectiveNonHumanPolicy {
    let verified_identity_mode = resolved_verified_identity_mode(cfg);
    let preset_catalog = non_human_stance_preset_catalog(objectives, verified_identity_mode.as_str());
    let rows: Vec<_> = canonical_non_human_taxonomy()
        .categories
        .into_iter()
        .map(|category| {
            let base_posture = objectives
                .category_postures
                .iter()
                .find(|row| row.category_id == category.category_id)
                .map(|row| row.posture.clone())
                .unwrap_or_else(|| "blocked".to_string());
            let verified_identity_override =
                effective_verified_identity_override(cfg, category.category_id);
            EffectiveNonHumanPolicyRow {
                category_id: category.category_id,
                category_label: category.label,
                base_posture,
                verified_identity_override,
            }
        })
        .collect();
    let mismatched_category_count = rows
        .iter()
        .filter(|row| {
            row.verified_identity_override
                .effective_posture
                .as_ref()
                .is_some_and(|posture| posture != &row.base_posture)
        })
        .count();

    EffectiveNonHumanPolicy {
        schema_version: EFFECTIVE_NON_HUMAN_POLICY_SCHEMA_VERSION.to_string(),
        resolution_mode: "explicit_override_layer".to_string(),
        active_preset_id: preset_catalog.active_preset_id,
        verified_identity_mode,
        mismatched_category_count,
        rows,
    }
}

fn effective_verified_identity_override(
    cfg: &Config,
    category_id: NonHumanCategoryId,
) -> EffectiveNonHumanPolicyVerifiedIdentityOverride {
    let identity_categories = verified_identity_categories_for(category_id);
    if identity_categories.is_empty() {
        return EffectiveNonHumanPolicyVerifiedIdentityOverride {
            status: "not_supported".to_string(),
            effective_action: "not_applicable".to_string(),
            effective_posture: None,
            verified_identity_categories: Vec::new(),
            source_of_authority: vec!["operator_objectives.category_postures".to_string()],
        };
    }
    if !cfg.verified_identity.enabled {
        return EffectiveNonHumanPolicyVerifiedIdentityOverride {
            status: "disabled".to_string(),
            effective_action: "not_applicable".to_string(),
            effective_posture: None,
            verified_identity_categories: identity_categories,
            source_of_authority: vec![
                "operator_objectives.category_postures".to_string(),
                "verified_identity.enabled".to_string(),
            ],
        };
    }

    if cfg.verified_identity.category_defaults.is_empty() {
        return explicit_verified_only_override(cfg, identity_categories);
    }
    category_default_override(cfg, identity_categories)
}

fn explicit_verified_only_override(
    cfg: &Config,
    identity_categories: Vec<String>,
) -> EffectiveNonHumanPolicyVerifiedIdentityOverride {
    if cfg.verified_identity.named_policies.is_empty() {
        return EffectiveNonHumanPolicyVerifiedIdentityOverride {
            status: "named_policies_only_fallback_deny".to_string(),
            effective_action: "deny".to_string(),
            effective_posture: Some("blocked".to_string()),
            verified_identity_categories: identity_categories,
            source_of_authority: vec![
                "operator_objectives.category_postures".to_string(),
                "verified_identity.named_policies".to_string(),
            ],
        };
    }
    EffectiveNonHumanPolicyVerifiedIdentityOverride {
        status: "named_policies_only".to_string(),
        effective_action: "conditional".to_string(),
        effective_posture: None,
        verified_identity_categories: identity_categories,
        source_of_authority: vec![
            "operator_objectives.category_postures".to_string(),
            "verified_identity.named_policies".to_string(),
        ],
    }
}

fn category_default_override(
    cfg: &Config,
    identity_categories: Vec<String>,
) -> EffectiveNonHumanPolicyVerifiedIdentityOverride {
    let mut actions = Vec::new();
    for category in &identity_categories {
        let Some(category_enum) = identity_category_from_str(category.as_str()) else {
            continue;
        };
        if let Some(category_default) = cfg
            .verified_identity
            .category_defaults
            .iter()
            .find(|row| row.category == category_enum)
        {
            actions.push(&category_default.action);
        }
    }

    let mut source_of_authority = vec![
        "operator_objectives.category_postures".to_string(),
        "verified_identity.category_defaults".to_string(),
    ];
    if !cfg.verified_identity.named_policies.is_empty() {
        source_of_authority.push("verified_identity.named_policies".to_string());
    }

    if actions.is_empty() {
        return if cfg.verified_identity.named_policies.is_empty() {
            EffectiveNonHumanPolicyVerifiedIdentityOverride {
                status: "category_defaults_fallback_deny".to_string(),
                effective_action: "deny".to_string(),
                effective_posture: Some("blocked".to_string()),
                verified_identity_categories: identity_categories,
                source_of_authority,
            }
        } else {
            EffectiveNonHumanPolicyVerifiedIdentityOverride {
                status: "mixed".to_string(),
                effective_action: "conditional".to_string(),
                effective_posture: None,
                verified_identity_categories: identity_categories,
                source_of_authority,
            }
        };
    }

    if cfg.verified_identity.named_policies.is_empty() && actions_are_uniform(actions.as_slice()) {
        let action = actions[0];
        let effective_action = action_kind(action);
        return EffectiveNonHumanPolicyVerifiedIdentityOverride {
            status: "category_default".to_string(),
            effective_action: effective_action.to_string(),
            effective_posture: action_to_posture(action),
            verified_identity_categories: identity_categories,
            source_of_authority,
        };
    }

    EffectiveNonHumanPolicyVerifiedIdentityOverride {
        status: "mixed".to_string(),
        effective_action: "conditional".to_string(),
        effective_posture: None,
        verified_identity_categories: identity_categories,
        source_of_authority,
    }
}

fn verified_identity_categories_for(category_id: NonHumanCategoryId) -> Vec<String> {
    match category_id {
        NonHumanCategoryId::IndexingBot => vec![IdentityCategory::Search.as_str().to_string()],
        NonHumanCategoryId::AiScraperBot => vec![IdentityCategory::Training.as_str().to_string()],
        NonHumanCategoryId::HttpAgent => vec![
            IdentityCategory::Preview.as_str().to_string(),
            IdentityCategory::ServiceAgent.as_str().to_string(),
        ],
        NonHumanCategoryId::AgentOnBehalfOfHuman => {
            vec![IdentityCategory::UserTriggeredAgent.as_str().to_string()]
        }
        NonHumanCategoryId::VerifiedBeneficialBot => {
            vec![IdentityCategory::Other.as_str().to_string()]
        }
        NonHumanCategoryId::AutomatedBrowser
        | NonHumanCategoryId::BrowserAgent
        | NonHumanCategoryId::UnknownNonHuman => Vec::new(),
    }
}

fn identity_category_from_str(value: &str) -> Option<IdentityCategory> {
    match value {
        "training" => Some(IdentityCategory::Training),
        "search" => Some(IdentityCategory::Search),
        "user_triggered_agent" => Some(IdentityCategory::UserTriggeredAgent),
        "preview" => Some(IdentityCategory::Preview),
        "service_agent" => Some(IdentityCategory::ServiceAgent),
        "other" => Some(IdentityCategory::Other),
        _ => None,
    }
}

fn actions_are_uniform(actions: &[&IdentityPolicyAction]) -> bool {
    let Some(first) = actions.first() else {
        return false;
    };
    actions.iter().all(|action| action_kind(action) == action_kind(first))
}

fn action_kind(action: &IdentityPolicyAction) -> &'static str {
    match action {
        IdentityPolicyAction::Deny => "deny",
        IdentityPolicyAction::Restrict => "restrict",
        IdentityPolicyAction::Observe => "observe",
        IdentityPolicyAction::Allow => "allow",
        IdentityPolicyAction::UseServiceProfile(_) => "use_service_profile",
    }
}

fn action_to_posture(action: &IdentityPolicyAction) -> Option<String> {
    match action {
        IdentityPolicyAction::Deny => Some("blocked".to_string()),
        IdentityPolicyAction::Restrict => Some("restricted".to_string()),
        IdentityPolicyAction::Observe => Some("tolerated".to_string()),
        IdentityPolicyAction::Allow => Some("allowed".to_string()),
        IdentityPolicyAction::UseServiceProfile(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{effective_non_human_policy, resolved_verified_identity_mode};
    use crate::bot_identity::policy::VerifiedIdentityOverrideMode;

    #[test]
    fn effective_non_human_policy_surfaces_verified_identity_mismatch_against_default_matrix() {
        let cfg = crate::config::defaults().clone();
        let objectives =
            crate::observability::operator_snapshot_objectives::default_operator_objectives(
                1_700_000_000,
            );

        let policy = effective_non_human_policy(&objectives, &cfg);

        assert_eq!(policy.active_preset_id, "balanced_default");
        assert_eq!(policy.verified_identity_mode, "verified_identities_denied");
        let row = policy
            .rows
            .iter()
            .find(|row| row.category_id.as_str() == "verified_beneficial_bot")
            .expect("verified beneficial row");
        assert_eq!(row.base_posture, "allowed");
        assert_eq!(
            row.verified_identity_override.status,
            "named_policies_only_fallback_deny"
        );
        assert_eq!(row.verified_identity_override.effective_action, "deny");
        assert_eq!(
            row.verified_identity_override.effective_posture.as_deref(),
            Some("blocked")
        );
        assert!(policy.mismatched_category_count >= 1);
    }

    #[test]
    fn resolved_verified_identity_mode_derives_verified_only_from_explicit_allowances() {
        let mut cfg = crate::config::defaults().clone();
        cfg.verified_identity.named_policies = vec![
            crate::bot_identity::policy::IdentityPolicyEntry {
                policy_id: "allow-training".to_string(),
                description: None,
                matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                    category: Some(crate::bot_identity::contracts::IdentityCategory::Training),
                    ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
                },
                action: crate::bot_identity::policy::IdentityPolicyAction::Allow,
            },
        ];
        assert_eq!(
            resolved_verified_identity_mode(&cfg),
            VerifiedIdentityOverrideMode::VerifiedIdentitiesOnly.as_str()
        );
    }
}
