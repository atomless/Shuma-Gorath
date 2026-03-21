use super::{
    contracts::{IdentityCategory, IdentityScheme, VerificationStrength},
    policy::{IdentityPolicyAction, IdentityPolicyOutcome, NonHumanTrafficStance, ServiceProfile},
    verification::{
        IdentityVerificationFailure, IdentityVerificationFreshness,
        IdentityVerificationResultStatus,
    },
};

#[test]
fn canonical_verified_identity_labels_are_stable() {
    assert_eq!(
        IdentityScheme::HttpMessageSignatures.as_str(),
        "http_message_signatures"
    );
    assert_eq!(
        IdentityScheme::ProviderVerifiedBot.as_str(),
        "provider_verified_bot"
    );
    assert_eq!(
        IdentityCategory::UserTriggeredAgent.as_str(),
        "user_triggered_agent"
    );
    assert_eq!(
        VerificationStrength::ProviderAsserted.as_str(),
        "provider_asserted"
    );
}

#[test]
fn restrictive_default_policy_shapes_are_explicit() {
    assert_eq!(
        NonHumanTrafficStance::DenyAllNonHuman.as_str(),
        "deny_all_non_human"
    );
    assert_eq!(ServiceProfile::MetadataOnly.as_str(), "metadata_only");
    assert_eq!(IdentityPolicyAction::Allow.as_str(), "allow");
    assert_eq!(
        IdentityPolicyOutcome::UseServiceProfile(ServiceProfile::StructuredAgent).as_str(),
        "use_service_profile"
    );
}

#[test]
fn verification_outcomes_keep_failure_and_freshness_taxonomy_separate() {
    assert_eq!(
        IdentityVerificationResultStatus::Failed.as_str(),
        "failed"
    );
    assert_eq!(
        IdentityVerificationFailure::ReplayRejected.as_str(),
        "replay_rejected"
    );
    assert_eq!(
        IdentityVerificationFreshness::ClockSkewAccepted.as_str(),
        "clock_skew_accepted"
    );
}
