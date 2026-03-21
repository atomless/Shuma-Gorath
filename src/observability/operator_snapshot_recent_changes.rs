use serde::{Deserialize, Serialize};

use super::decision_ledger::OperatorDecisionEvidenceReference;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotRecentChange {
    pub changed_at_ts: u64,
    pub change_reason: String,
    pub changed_families: Vec<String>,
    pub source: String,
    pub targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_impact_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_references: Vec<OperatorDecisionEvidenceReference>,
    pub watch_window_status: String,
    pub watch_window_elapsed_seconds: u64,
    pub watch_window_remaining_seconds: u64,
    pub change_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct OperatorSnapshotRecentChanges {
    pub lookback_seconds: u64,
    pub watch_window_seconds: u64,
    pub rows: Vec<OperatorSnapshotRecentChange>,
}
