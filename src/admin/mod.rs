pub(crate) mod adversary_sim;
pub(crate) mod adversary_sim_corpus;
pub(crate) mod adversary_sim_diagnostics;
pub(crate) mod adversary_sim_lane_runtime;
pub(crate) mod adversary_sim_state;
pub(crate) mod adversary_sim_worker_plan;
mod adversary_sim_api;
pub(crate) mod adversary_sim_control;
mod benchmark_api;
mod config_api;
mod diagnostics_api;
mod monitoring_api;
mod api;
pub(crate) mod auth;
mod operator_objectives_api;
mod operator_snapshot_api;
mod recent_changes_ledger;

pub use api::{handle_admin, handle_internal, log_event, now_ts, EventLogEntry, EventType};
pub(crate) use api::{
    log_event_with_execution_metadata, monitoring_presented_recent_event_tail,
    monitoring_recent_sim_run_summaries, monitoring_security_privacy_payload,
    monitoring_security_view_mode_label, EventExecutionMetadata,
};
pub(crate) use recent_changes_ledger::load_operator_snapshot_recent_changes;
#[cfg(test)]
pub(crate) use recent_changes_ledger::{
    operator_snapshot_manual_change_row, record_operator_snapshot_recent_change_rows,
};
