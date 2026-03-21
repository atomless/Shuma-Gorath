use serde::{Deserialize, Serialize};

use super::contracts::{IdentityCategory, IdentityScheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NonHumanTrafficStance {
    DenyAllNonHuman,
    AllowOnlyExplicitVerifiedIdentities,
    AllowVerifiedByCategory,
    AllowVerifiedWithLowCostProfilesOnly,
}

impl NonHumanTrafficStance {
    pub fn as_str(self) -> &'static str {
        match self {
            NonHumanTrafficStance::DenyAllNonHuman => "deny_all_non_human",
            NonHumanTrafficStance::AllowOnlyExplicitVerifiedIdentities => {
                "allow_only_explicit_verified_identities"
            }
            NonHumanTrafficStance::AllowVerifiedByCategory => "allow_verified_by_category",
            NonHumanTrafficStance::AllowVerifiedWithLowCostProfilesOnly => {
                "allow_verified_with_low_cost_profiles_only"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ServiceProfile {
    BrowserLike,
    StructuredAgent,
    MetadataOnly,
    Denied,
}

impl ServiceProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceProfile::BrowserLike => "browser_like",
            ServiceProfile::StructuredAgent => "structured_agent",
            ServiceProfile::MetadataOnly => "metadata_only",
            ServiceProfile::Denied => "denied",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityPolicyMatcher {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme: Option<IdentityScheme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<IdentityCategory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path_prefixes: Vec<String>,
}

impl IdentityPolicyMatcher {
    pub fn is_empty(&self) -> bool {
        self.scheme.is_none()
            && self.stable_identity.is_none()
            && self.operator.is_none()
            && self.category.is_none()
            && self.path_prefixes.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub(crate) enum IdentityPolicyAction {
    Deny,
    Restrict,
    Observe,
    Allow,
    UseServiceProfile(String),
}

impl IdentityPolicyAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            IdentityPolicyAction::Deny => "deny",
            IdentityPolicyAction::Restrict => "restrict",
            IdentityPolicyAction::Observe => "observe",
            IdentityPolicyAction::Allow => "allow",
            IdentityPolicyAction::UseServiceProfile(_) => "use_service_profile",
        }
    }

    pub fn referenced_service_profile_id(&self) -> Option<&str> {
        match self {
            IdentityPolicyAction::UseServiceProfile(profile_id) => Some(profile_id.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityPolicyEntry {
    pub policy_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub matcher: IdentityPolicyMatcher,
    pub action: IdentityPolicyAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityCategoryDefaultAction {
    pub category: IdentityCategory,
    pub action: IdentityPolicyAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityServiceProfileBinding {
    pub profile_id: String,
    pub profile: ServiceProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentityPolicyOutcome {
    NoMatch,
    Deny,
    Restrict,
    Observe,
    Allow,
    UseServiceProfile(ServiceProfile),
}

impl IdentityPolicyOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            IdentityPolicyOutcome::NoMatch => "no_match",
            IdentityPolicyOutcome::Deny => "deny",
            IdentityPolicyOutcome::Restrict => "restrict",
            IdentityPolicyOutcome::Observe => "observe",
            IdentityPolicyOutcome::Allow => "allow",
            IdentityPolicyOutcome::UseServiceProfile(_) => "use_service_profile",
        }
    }
}
