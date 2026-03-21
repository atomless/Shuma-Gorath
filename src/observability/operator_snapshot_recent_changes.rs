use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotRecentChange {
    pub changed_at_ts: u64,
    pub change_reason: String,
    pub changed_families: Vec<String>,
    pub source: String,
    pub targets: Vec<String>,
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
