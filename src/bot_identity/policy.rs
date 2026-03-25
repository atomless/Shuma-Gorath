use serde::{Deserialize, Serialize};

use super::contracts::{IdentityCategory, IdentityScheme};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerifiedIdentityOverrideMode {
    Disabled,
    VerifiedIdentitiesDenied,
    VerifiedIdentitiesOnly,
}

impl VerifiedIdentityOverrideMode {
    pub fn as_str(self) -> &'static str {
        match self {
            VerifiedIdentityOverrideMode::Disabled => "disabled",
            VerifiedIdentityOverrideMode::VerifiedIdentitiesDenied => {
                "verified_identities_denied"
            }
            VerifiedIdentityOverrideMode::VerifiedIdentitiesOnly => "verified_identities_only",
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum IdentityPolicyResolutionSource {
    NamedPolicy(String),
    CategoryDefault(IdentityCategory),
    DefaultDeny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdentityPolicyResolutionSourceKind {
    NamedPolicy,
    CategoryDefault,
    DefaultDeny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IdentityPolicyResolution {
    pub outcome: IdentityPolicyOutcome,
    pub service_profile_id: Option<String>,
    source: IdentityPolicyResolutionSource,
}

impl IdentityPolicyResolution {
    pub fn source_label(&self) -> &'static str {
        match self.source {
            IdentityPolicyResolutionSource::NamedPolicy(_) => "named_policy",
            IdentityPolicyResolutionSource::CategoryDefault(_) => "category_default",
            IdentityPolicyResolutionSource::DefaultDeny => "default_deny",
        }
    }

    pub fn source_id(&self) -> &str {
        match &self.source {
            IdentityPolicyResolutionSource::NamedPolicy(policy_id) => policy_id.as_str(),
            IdentityPolicyResolutionSource::CategoryDefault(category) => category.as_str(),
            IdentityPolicyResolutionSource::DefaultDeny => "verified_identity_default_deny",
        }
    }

    pub(crate) fn source_kind(&self) -> IdentityPolicyResolutionSourceKind {
        match self.source {
            IdentityPolicyResolutionSource::NamedPolicy(_) => {
                IdentityPolicyResolutionSourceKind::NamedPolicy
            }
            IdentityPolicyResolutionSource::CategoryDefault(_) => {
                IdentityPolicyResolutionSourceKind::CategoryDefault
            }
            IdentityPolicyResolutionSource::DefaultDeny => {
                IdentityPolicyResolutionSourceKind::DefaultDeny
            }
        }
    }
}

pub(crate) fn resolved_verified_identity_override_mode(
    enabled: bool,
    named_policies: &[IdentityPolicyEntry],
    category_defaults: &[IdentityCategoryDefaultAction],
    service_profiles: &[IdentityServiceProfileBinding],
) -> VerifiedIdentityOverrideMode {
    if !enabled {
        return VerifiedIdentityOverrideMode::Disabled;
    }
    if has_allow_capable_override(named_policies, category_defaults, service_profiles) {
        VerifiedIdentityOverrideMode::VerifiedIdentitiesOnly
    } else {
        VerifiedIdentityOverrideMode::VerifiedIdentitiesDenied
    }
}

pub(crate) fn resolve_identity_policy(
    named_policies: &[IdentityPolicyEntry],
    category_defaults: &[IdentityCategoryDefaultAction],
    service_profiles: &[IdentityServiceProfileBinding],
    identity: &super::contracts::VerifiedIdentityEvidence,
    request_path: &str,
) -> IdentityPolicyResolution {
    for policy in named_policies {
        if !matcher_matches(&policy.matcher, identity, request_path) {
            continue;
        }
        return resolution_from_action(
            &policy.action,
            service_profiles,
            IdentityPolicyResolutionSource::NamedPolicy(policy.policy_id.clone()),
        );
    }

    for category_default in category_defaults {
        if category_default.category != identity.category {
            continue;
        }
        return resolution_from_action(
            &category_default.action,
            service_profiles,
            IdentityPolicyResolutionSource::CategoryDefault(category_default.category),
        );
    }

    IdentityPolicyResolution {
        outcome: IdentityPolicyOutcome::Deny,
        service_profile_id: None,
        source: IdentityPolicyResolutionSource::DefaultDeny,
    }
}

fn has_allow_capable_override(
    named_policies: &[IdentityPolicyEntry],
    category_defaults: &[IdentityCategoryDefaultAction],
    service_profiles: &[IdentityServiceProfileBinding],
) -> bool {
    named_policies
        .iter()
        .any(|policy| action_is_allow_capable(&policy.action, service_profiles))
        || category_defaults
            .iter()
            .any(|policy| action_is_allow_capable(&policy.action, service_profiles))
}

fn action_is_allow_capable(
    action: &IdentityPolicyAction,
    service_profiles: &[IdentityServiceProfileBinding],
) -> bool {
    match action {
        IdentityPolicyAction::Deny => false,
        IdentityPolicyAction::Restrict => true,
        IdentityPolicyAction::Observe => true,
        IdentityPolicyAction::Allow => true,
        IdentityPolicyAction::UseServiceProfile(profile_id) => service_profiles
            .iter()
            .find(|binding| binding.profile_id == *profile_id)
            .map(|binding| binding.profile != ServiceProfile::Denied)
            .unwrap_or(false),
    }
}

fn matcher_matches(
    matcher: &IdentityPolicyMatcher,
    identity: &super::contracts::VerifiedIdentityEvidence,
    request_path: &str,
) -> bool {
    if matcher.scheme.is_some_and(|scheme| scheme != identity.scheme) {
        return false;
    }
    if matcher
        .stable_identity
        .as_ref()
        .is_some_and(|stable_identity| stable_identity != &identity.stable_identity)
    {
        return false;
    }
    if matcher
        .operator
        .as_ref()
        .is_some_and(|operator| operator != &identity.operator)
    {
        return false;
    }
    if matcher
        .category
        .is_some_and(|category| category != identity.category)
    {
        return false;
    }
    if matcher.path_prefixes.is_empty() {
        return true;
    }
    matcher
        .path_prefixes
        .iter()
        .any(|path_prefix| request_path.starts_with(path_prefix))
}

fn resolution_from_action(
    action: &IdentityPolicyAction,
    service_profiles: &[IdentityServiceProfileBinding],
    source: IdentityPolicyResolutionSource,
) -> IdentityPolicyResolution {
    match action {
        IdentityPolicyAction::Deny => IdentityPolicyResolution {
            outcome: IdentityPolicyOutcome::Deny,
            service_profile_id: None,
            source,
        },
        IdentityPolicyAction::Restrict => IdentityPolicyResolution {
            outcome: IdentityPolicyOutcome::Restrict,
            service_profile_id: None,
            source,
        },
        IdentityPolicyAction::Observe => IdentityPolicyResolution {
            outcome: IdentityPolicyOutcome::Observe,
            service_profile_id: None,
            source,
        },
        IdentityPolicyAction::Allow => IdentityPolicyResolution {
            outcome: IdentityPolicyOutcome::Allow,
            service_profile_id: None,
            source,
        },
        IdentityPolicyAction::UseServiceProfile(profile_id) => {
            // Config validation guarantees referenced profiles exist. If the
            // binding is somehow missing at runtime, fail closed as denied.
            let profile = service_profiles
                .iter()
                .find(|binding| binding.profile_id == *profile_id)
                .map(|binding| binding.profile)
                .unwrap_or(ServiceProfile::Denied);
            IdentityPolicyResolution {
                outcome: IdentityPolicyOutcome::UseServiceProfile(profile),
                service_profile_id: Some(profile_id.clone()),
                source,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot_identity::contracts::{
        IdentityCategory, IdentityProvenance, IdentityScheme, VerificationStrength,
        VerifiedIdentityEvidence,
    };

    fn identity() -> VerifiedIdentityEvidence {
        VerifiedIdentityEvidence {
            scheme: IdentityScheme::ProviderSignedAgent,
            stable_identity: "chatgpt-agent".to_string(),
            operator: "openai".to_string(),
            category: IdentityCategory::UserTriggeredAgent,
            verification_strength: VerificationStrength::ProviderAsserted,
            end_user_controlled: true,
            directory_source: None,
            provenance: IdentityProvenance::Provider,
        }
    }

    fn service_profiles() -> Vec<IdentityServiceProfileBinding> {
        vec![
            IdentityServiceProfileBinding {
                profile_id: "browser_like".to_string(),
                profile: ServiceProfile::BrowserLike,
                description: None,
            },
            IdentityServiceProfileBinding {
                profile_id: "structured_agent".to_string(),
                profile: ServiceProfile::StructuredAgent,
                description: None,
            },
            IdentityServiceProfileBinding {
                profile_id: "denied".to_string(),
                profile: ServiceProfile::Denied,
                description: None,
            },
        ]
    }

    #[test]
    fn resolve_identity_policy_prefers_first_matching_named_policy() {
        let policies = vec![
            IdentityPolicyEntry {
                policy_id: "deny-openai".to_string(),
                description: None,
                matcher: IdentityPolicyMatcher {
                    operator: Some("openai".to_string()),
                    ..IdentityPolicyMatcher::default()
                },
                action: IdentityPolicyAction::Deny,
            },
            IdentityPolicyEntry {
                policy_id: "allow-openai".to_string(),
                description: None,
                matcher: IdentityPolicyMatcher {
                    operator: Some("openai".to_string()),
                    ..IdentityPolicyMatcher::default()
                },
                action: IdentityPolicyAction::Allow,
            },
        ];

        let resolution = resolve_identity_policy(
            &policies,
            &[],
            &service_profiles(),
            &identity(),
            "/",
        );

        assert_eq!(resolution.outcome, IdentityPolicyOutcome::Deny);
        assert_eq!(resolution.source_label(), "named_policy");
        assert_eq!(resolution.source_id(), "deny-openai");
    }

    #[test]
    fn resolve_identity_policy_requires_path_prefix_match() {
        let policies = vec![IdentityPolicyEntry {
            policy_id: "allow-api".to_string(),
            description: None,
            matcher: IdentityPolicyMatcher {
                operator: Some("openai".to_string()),
                path_prefixes: vec!["/api".to_string()],
                ..IdentityPolicyMatcher::default()
            },
            action: IdentityPolicyAction::Allow,
        }];

        let resolution = resolve_identity_policy(
            &policies,
            &[],
            &service_profiles(),
            &identity(),
            "/pricing",
        );

        assert_eq!(resolution.outcome, IdentityPolicyOutcome::Deny);
        assert_eq!(resolution.source_label(), "default_deny");
        assert_eq!(resolution.source_id(), "verified_identity_default_deny");
    }

    #[test]
    fn resolve_identity_policy_uses_category_defaults_as_explicit_overrides() {
        let category_defaults = vec![IdentityCategoryDefaultAction {
            category: IdentityCategory::UserTriggeredAgent,
            action: IdentityPolicyAction::UseServiceProfile("structured_agent".to_string()),
        }];

        let resolution = resolve_identity_policy(
            &[],
            &category_defaults,
            &service_profiles(),
            &identity(),
            "/",
        );

        assert_eq!(
            resolution.outcome,
            IdentityPolicyOutcome::UseServiceProfile(ServiceProfile::StructuredAgent)
        );
        assert_eq!(resolution.source_label(), "category_default");
        assert_eq!(resolution.source_id(), "user_triggered_agent");
        assert_eq!(resolution.service_profile_id.as_deref(), Some("structured_agent"));
    }

    #[test]
    fn resolve_identity_policy_without_matches_falls_back_to_default_deny() {
        let resolution = resolve_identity_policy(&[], &[], &service_profiles(), &identity(), "/");

        assert_eq!(resolution.outcome, IdentityPolicyOutcome::Deny);
        assert_eq!(resolution.source_label(), "default_deny");
        assert_eq!(resolution.source_id(), "verified_identity_default_deny");
    }

    #[test]
    fn resolve_identity_policy_preserves_observe_and_restrict_outcomes() {
        let policies = vec![
            IdentityPolicyEntry {
                policy_id: "observe-openai".to_string(),
                description: None,
                matcher: IdentityPolicyMatcher {
                    operator: Some("openai".to_string()),
                    path_prefixes: vec!["/observe".to_string()],
                    ..IdentityPolicyMatcher::default()
                },
                action: IdentityPolicyAction::Observe,
            },
            IdentityPolicyEntry {
                policy_id: "restrict-openai".to_string(),
                description: None,
                matcher: IdentityPolicyMatcher {
                    operator: Some("openai".to_string()),
                    path_prefixes: vec!["/restrict".to_string()],
                    ..IdentityPolicyMatcher::default()
                },
                action: IdentityPolicyAction::Restrict,
            },
        ];

        let observe = resolve_identity_policy(
            &policies,
            &[],
            &service_profiles(),
            &identity(),
            "/observe/path",
        );
        let restrict = resolve_identity_policy(
            &policies,
            &[],
            &service_profiles(),
            &identity(),
            "/restrict/path",
        );

        assert_eq!(observe.outcome, IdentityPolicyOutcome::Observe);
        assert_eq!(restrict.outcome, IdentityPolicyOutcome::Restrict);
    }

    #[test]
    fn resolved_verified_identity_override_mode_is_deny_without_explicit_allowances() {
        let mode = resolved_verified_identity_override_mode(true, &[], &[], &service_profiles());
        assert_eq!(
            mode,
            VerifiedIdentityOverrideMode::VerifiedIdentitiesDenied
        );
    }

    #[test]
    fn resolved_verified_identity_override_mode_detects_allow_capable_overrides() {
        let policies = vec![IdentityPolicyEntry {
            policy_id: "allow-openai".to_string(),
            description: None,
            matcher: IdentityPolicyMatcher {
                operator: Some("openai".to_string()),
                ..IdentityPolicyMatcher::default()
            },
            action: IdentityPolicyAction::UseServiceProfile("structured_agent".to_string()),
        }];
        let mode =
            resolved_verified_identity_override_mode(true, &policies, &[], &service_profiles());
        assert_eq!(mode, VerifiedIdentityOverrideMode::VerifiedIdentitiesOnly);
    }
}
