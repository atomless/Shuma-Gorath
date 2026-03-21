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
}
