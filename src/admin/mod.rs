pub(crate) mod adversary_sim;
pub(crate) mod adversary_sim_control;
mod api;
pub(crate) mod auth;

pub use api::{handle_admin, handle_internal, log_event, now_ts, EventLogEntry, EventType};
pub(crate) use api::{
    load_operator_snapshot_recent_changes, log_event_with_execution_metadata,
    monitoring_presented_recent_event_tail, monitoring_recent_sim_run_summaries,
    monitoring_security_privacy_payload, monitoring_security_view_mode_label,
    EventExecutionMetadata,
};
#[cfg(test)]
pub(crate) use api::{
    operator_snapshot_manual_change_row, record_operator_snapshot_recent_change_rows,
};
