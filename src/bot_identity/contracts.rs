use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentityScheme {
    HttpMessageSignatures,
    ProviderVerifiedBot,
    ProviderSignedAgent,
    Mtls,
}

impl IdentityScheme {
    pub fn as_str(self) -> &'static str {
        match self {
            IdentityScheme::HttpMessageSignatures => "http_message_signatures",
            IdentityScheme::ProviderVerifiedBot => "provider_verified_bot",
            IdentityScheme::ProviderSignedAgent => "provider_signed_agent",
            IdentityScheme::Mtls => "mtls",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentityCategory {
    Training,
    Search,
    UserTriggeredAgent,
    Preview,
    ServiceAgent,
    Other,
}

impl IdentityCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            IdentityCategory::Training => "training",
            IdentityCategory::Search => "search",
            IdentityCategory::UserTriggeredAgent => "user_triggered_agent",
            IdentityCategory::Preview => "preview",
            IdentityCategory::ServiceAgent => "service_agent",
            IdentityCategory::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VerificationStrength {
    Cryptographic,
    ProviderAsserted,
}

impl VerificationStrength {
    pub fn as_str(self) -> &'static str {
        match self {
            VerificationStrength::Cryptographic => "cryptographic",
            VerificationStrength::ProviderAsserted => "provider_asserted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentityProvenance {
    Native,
    Provider,
}

impl IdentityProvenance {
    pub fn as_str(self) -> &'static str {
        match self {
            IdentityProvenance::Native => "native",
            IdentityProvenance::Provider => "provider",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityDirectorySource {
    pub source_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VerifiedIdentityEvidence {
    pub scheme: IdentityScheme,
    pub stable_identity: String,
    pub operator: String,
    pub category: IdentityCategory,
    pub verification_strength: VerificationStrength,
    #[serde(default)]
    pub end_user_controlled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directory_source: Option<IdentityDirectorySource>,
    pub provenance: IdentityProvenance,
}
