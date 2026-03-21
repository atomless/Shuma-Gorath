use serde::{Deserialize, Serialize};

use super::contracts::VerifiedIdentityEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentityVerificationResultStatus {
    Disabled,
    NotAttempted,
    Verified,
    Failed,
}

impl IdentityVerificationResultStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            IdentityVerificationResultStatus::Disabled => "disabled",
            IdentityVerificationResultStatus::NotAttempted => "not_attempted",
            IdentityVerificationResultStatus::Verified => "verified",
            IdentityVerificationResultStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentityVerificationFailure {
    MissingAssertion,
    MissingSignature,
    SignatureInvalid,
    ReplayRejected,
    ClockSkewRejected,
    DirectoryUnavailable,
    DirectoryStale,
    ProviderRejected,
    ProviderUnavailable,
    UnsupportedScheme,
}

impl IdentityVerificationFailure {
    pub fn as_str(self) -> &'static str {
        match self {
            IdentityVerificationFailure::MissingAssertion => "missing_assertion",
            IdentityVerificationFailure::MissingSignature => "missing_signature",
            IdentityVerificationFailure::SignatureInvalid => "signature_invalid",
            IdentityVerificationFailure::ReplayRejected => "replay_rejected",
            IdentityVerificationFailure::ClockSkewRejected => "clock_skew_rejected",
            IdentityVerificationFailure::DirectoryUnavailable => "directory_unavailable",
            IdentityVerificationFailure::DirectoryStale => "directory_stale",
            IdentityVerificationFailure::ProviderRejected => "provider_rejected",
            IdentityVerificationFailure::ProviderUnavailable => "provider_unavailable",
            IdentityVerificationFailure::UnsupportedScheme => "unsupported_scheme",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentityVerificationFreshness {
    NotApplicable,
    Fresh,
    ClockSkewAccepted,
    Stale,
    ReplayRejected,
}

impl IdentityVerificationFreshness {
    pub fn as_str(self) -> &'static str {
        match self {
            IdentityVerificationFreshness::NotApplicable => "not_applicable",
            IdentityVerificationFreshness::Fresh => "fresh",
            IdentityVerificationFreshness::ClockSkewAccepted => "clock_skew_accepted",
            IdentityVerificationFreshness::Stale => "stale",
            IdentityVerificationFreshness::ReplayRejected => "replay_rejected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityVerificationResult {
    pub status: IdentityVerificationResultStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<VerifiedIdentityEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<IdentityVerificationFailure>,
    pub freshness: IdentityVerificationFreshness,
}

impl IdentityVerificationResult {
    pub fn disabled() -> Self {
        Self {
            status: IdentityVerificationResultStatus::Disabled,
            identity: None,
            failure: None,
            freshness: IdentityVerificationFreshness::NotApplicable,
        }
    }

    pub fn not_attempted() -> Self {
        Self {
            status: IdentityVerificationResultStatus::NotAttempted,
            identity: None,
            failure: None,
            freshness: IdentityVerificationFreshness::NotApplicable,
        }
    }

    pub fn verified(identity: VerifiedIdentityEvidence, freshness: IdentityVerificationFreshness) -> Self {
        Self {
            status: IdentityVerificationResultStatus::Verified,
            identity: Some(identity),
            failure: None,
            freshness,
        }
    }

    pub fn failed(
        failure: IdentityVerificationFailure,
        freshness: IdentityVerificationFreshness,
    ) -> Self {
        Self {
            status: IdentityVerificationResultStatus::Failed,
            identity: None,
            failure: Some(failure),
            freshness,
        }
    }
}
