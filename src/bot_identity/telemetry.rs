use serde::{Deserialize, Serialize};

use super::contracts::{IdentityProvenance, IdentityScheme};
use super::verification::{
    IdentityVerificationFailure, IdentityVerificationFreshness, IdentityVerificationResultStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentityVerificationOutcomeClass {
    Skipped,
    Verified,
    Failed,
}

impl IdentityVerificationOutcomeClass {
    pub fn as_str(self) -> &'static str {
        match self {
            IdentityVerificationOutcomeClass::Skipped => "skipped",
            IdentityVerificationOutcomeClass::Verified => "verified",
            IdentityVerificationOutcomeClass::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityVerificationTelemetryRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme: Option<IdentityScheme>,
    pub provenance: IdentityProvenance,
    pub result_status: IdentityVerificationResultStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<IdentityVerificationFailure>,
    pub freshness: IdentityVerificationFreshness,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_identity: Option<String>,
}

impl IdentityVerificationTelemetryRecord {
    pub fn from_verification_result(
        default_provenance: IdentityProvenance,
        result: &super::verification::IdentityVerificationResult,
    ) -> Option<Self> {
        match result.status {
            IdentityVerificationResultStatus::Disabled
            | IdentityVerificationResultStatus::NotAttempted => None,
            IdentityVerificationResultStatus::Verified
            | IdentityVerificationResultStatus::Failed => {
                let identity = result.identity.as_ref();
                Some(Self {
                    scheme: identity.map(|value| value.scheme),
                    provenance: identity
                        .map(|value| value.provenance)
                        .unwrap_or(default_provenance),
                    result_status: result.status,
                    failure: result.failure,
                    freshness: result.freshness,
                    operator: identity.map(|value| value.operator.clone()),
                    stable_identity: identity.map(|value| value.stable_identity.clone()),
                })
            }
        }
    }

    pub fn outcome_class(&self) -> IdentityVerificationOutcomeClass {
        match self.result_status {
            IdentityVerificationResultStatus::Verified => {
                IdentityVerificationOutcomeClass::Verified
            }
            IdentityVerificationResultStatus::Failed => IdentityVerificationOutcomeClass::Failed,
            IdentityVerificationResultStatus::Disabled
            | IdentityVerificationResultStatus::NotAttempted => {
                IdentityVerificationOutcomeClass::Skipped
            }
        }
    }
}
