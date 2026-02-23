use serde::{Deserialize, Serialize};

pub(crate) const TOKEN_VERSION_V1: u8 = 1;
pub(crate) const WORK_ALG_HASHCASH_SHA256_V1: &str = "hashcash_sha256_v1";
pub(crate) const PATH_CLASS_TARPIT_PROGRESS: &str = "tarpit_progress";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct TarpitProgressToken {
    pub version: u8,
    pub operation_id: String,
    pub flow_id: String,
    pub step: u16,
    pub parent_digest: String,
    pub ip_bucket: String,
    pub ua_bucket: String,
    pub path_class: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub difficulty: u8,
    pub work_alg: String,
    pub max_chunk_bytes: u32,
    pub flow_bytes_emitted: u64,
    pub flow_started_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_epoch: Option<u32>,
}

impl TarpitProgressToken {
    pub(crate) fn operation_digest(&self) -> String {
        crate::maze::token::digest(format!("{}:{}", self.flow_id, self.operation_id).as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProgressRejectReason {
    Malformed,
    SignatureMismatch,
    InvalidVersion,
    Expired,
    InvalidWindow,
    BindingIpMismatch,
    BindingUaMismatch,
    PathMismatch,
    StepOutOfOrder,
    ParentChainMissing,
    Replay,
    InvalidProof,
    BudgetExhausted,
}

impl ProgressRejectReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ProgressRejectReason::Malformed => "tarpit_progress_malformed",
            ProgressRejectReason::SignatureMismatch => "tarpit_progress_signature_mismatch",
            ProgressRejectReason::InvalidVersion => "tarpit_progress_invalid_version",
            ProgressRejectReason::Expired => "tarpit_progress_expired",
            ProgressRejectReason::InvalidWindow => "tarpit_progress_invalid_window",
            ProgressRejectReason::BindingIpMismatch => "tarpit_progress_binding_ip_mismatch",
            ProgressRejectReason::BindingUaMismatch => "tarpit_progress_binding_ua_mismatch",
            ProgressRejectReason::PathMismatch => "tarpit_progress_path_mismatch",
            ProgressRejectReason::StepOutOfOrder => "tarpit_progress_step_out_of_order",
            ProgressRejectReason::ParentChainMissing => "tarpit_progress_parent_chain_missing",
            ProgressRejectReason::Replay => "tarpit_progress_replay",
            ProgressRejectReason::InvalidProof => "tarpit_progress_invalid_proof",
            ProgressRejectReason::BudgetExhausted => "tarpit_progress_budget_exhausted",
        }
    }

    pub(crate) fn is_budget(self) -> bool {
        matches!(self, ProgressRejectReason::BudgetExhausted)
    }
}

pub(crate) const PROGRESS_REJECT_REASON_KEYS: [&str; 13] = [
    "tarpit_progress_malformed",
    "tarpit_progress_signature_mismatch",
    "tarpit_progress_invalid_version",
    "tarpit_progress_expired",
    "tarpit_progress_invalid_window",
    "tarpit_progress_binding_ip_mismatch",
    "tarpit_progress_binding_ua_mismatch",
    "tarpit_progress_path_mismatch",
    "tarpit_progress_step_out_of_order",
    "tarpit_progress_parent_chain_missing",
    "tarpit_progress_replay",
    "tarpit_progress_invalid_proof",
    "tarpit_progress_budget_exhausted",
];
