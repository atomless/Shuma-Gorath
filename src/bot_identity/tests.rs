use super::{
    contracts::{IdentityCategory, IdentityProvenance, IdentityScheme, VerificationStrength},
    policy::{
        IdentityPolicyAction, IdentityPolicyOutcome, VerifiedIdentityOverrideMode, ServiceProfile,
    },
    telemetry::IdentityVerificationTelemetryRecord,
    verification::{
        IdentityVerificationFailure, IdentityVerificationFreshness, IdentityVerificationResult,
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
        VerifiedIdentityOverrideMode::VerifiedIdentitiesDenied.as_str(),
        "verified_identities_denied"
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
    assert_eq!(IdentityVerificationResultStatus::Failed.as_str(), "failed");
    assert_eq!(
        IdentityVerificationFailure::ReplayRejected.as_str(),
        "replay_rejected"
    );
    assert_eq!(
        IdentityVerificationFreshness::ClockSkewAccepted.as_str(),
        "clock_skew_accepted"
    );
}

#[test]
fn verification_telemetry_records_only_attempted_results() {
    assert!(
        IdentityVerificationTelemetryRecord::from_verification_result(
            IdentityProvenance::Provider,
            &IdentityVerificationResult::not_attempted()
        )
        .is_none()
    );
    assert!(
        IdentityVerificationTelemetryRecord::from_verification_result(
            IdentityProvenance::Native,
            &IdentityVerificationResult::disabled()
        )
        .is_none()
    );
}

#[test]
fn verification_telemetry_preserves_verified_identity_metadata() {
    let record = IdentityVerificationTelemetryRecord::from_verification_result(
        IdentityProvenance::Provider,
        &IdentityVerificationResult::verified(
            super::contracts::VerifiedIdentityEvidence {
                scheme: IdentityScheme::ProviderSignedAgent,
                stable_identity: "chatgpt-agent".to_string(),
                operator: "openai".to_string(),
                category: IdentityCategory::UserTriggeredAgent,
                verification_strength: VerificationStrength::ProviderAsserted,
                end_user_controlled: true,
                directory_source: None,
                provenance: IdentityProvenance::Provider,
            },
            IdentityVerificationFreshness::Fresh,
        ),
    )
    .expect("attempted record");

    assert_eq!(record.outcome_class().as_str(), "verified");
    assert_eq!(record.operator.as_deref(), Some("openai"));
    assert_eq!(record.stable_identity.as_deref(), Some("chatgpt-agent"));
    assert_eq!(record.scheme, Some(IdentityScheme::ProviderSignedAgent));
}
