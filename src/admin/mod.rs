pub(crate) mod adversary_sim;
pub(crate) mod adversary_sim_corpus;
pub(crate) mod adversary_sim_diagnostics;
pub(crate) mod adversary_sim_identity_pool;
pub(crate) mod adversary_sim_llm_lane;
pub(crate) mod adversary_sim_lane_runtime;
pub(crate) mod adversary_sim_realism_profile;
pub(crate) mod adversary_sim_status_truth;
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
pub(crate) mod oversight_agent;
pub(crate) mod oversight_apply;
pub(crate) mod oversight_follow_on_runs;
mod oversight_api;
pub(crate) mod oversight_decision_ledger;
pub(crate) mod oversight_observer_round_archive;
pub(crate) mod oversight_patch_policy;
pub(crate) mod oversight_reconcile;
mod operator_snapshot_api;
mod replay_promotion_api;
mod recent_changes_ledger;

pub use api::{handle_admin, handle_internal, log_event, now_ts, EventLogEntry, EventType};
pub(crate) use api::{
    log_event_with_execution_metadata, monitoring_presented_recent_event_tail,
    monitoring_recent_sim_run_summaries, monitoring_security_privacy_payload,
    monitoring_security_view_mode_label, EventExecutionMetadata,
};
pub(crate) use recent_changes_ledger::load_operator_snapshot_recent_changes;
pub(crate) use oversight_api::load_oversight_episode_archive;
#[cfg(test)]
pub(crate) use recent_changes_ledger::{
    operator_snapshot_manual_change_row, record_operator_snapshot_recent_change_rows,
};
