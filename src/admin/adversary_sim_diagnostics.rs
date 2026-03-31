use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

use super::adversary_sim::{
    MAX_CONCURRENT_RUNS, MAX_CPU_MILLICORES, MAX_MEMORY_MIB, PRODUCTION_GENERATION_DEFAULT,
    QUEUE_POLICY, WorkerFailureClass,
};
use super::adversary_sim_corpus::deterministic_corpus_metadata_payload;
use super::adversary_sim_state::{
    autonomous_execution_profile, clamp_duration_seconds, effective_active_lane,
    generation_diagnostic_grace_seconds, lane_phase, ControlPhase, ControlState, RuntimeLane,
};

const LANE_DIAGNOSTICS_SCHEMA_VERSION: &str = "adversary-sim-lane-diagnostics.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FailureClassCounter {
    #[serde(default)]
    pub count: u64,
    #[serde(default)]
    pub last_seen_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RequestFailureClassCounters {
    #[serde(default)]
    pub cancelled: FailureClassCounter,
    #[serde(default)]
    pub timeout: FailureClassCounter,
    #[serde(default)]
    pub transport: FailureClassCounter,
    #[serde(default)]
    pub http: FailureClassCounter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LaneCounterState {
    #[serde(default)]
    pub beat_attempts: u64,
    #[serde(default)]
    pub beat_successes: u64,
    #[serde(default)]
    pub beat_failures: u64,
    #[serde(default)]
    pub generated_requests: u64,
    #[serde(default)]
    pub blocked_requests: u64,
    #[serde(default)]
    pub offsite_requests: u64,
    #[serde(default)]
    pub response_bytes: u64,
    #[serde(default)]
    pub response_status_count: BTreeMap<String, u64>,
    #[serde(default)]
    pub last_generated_at: Option<u64>,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LaneDiagnosticsState {
    #[serde(default)]
    pub synthetic_traffic: LaneCounterState,
    #[serde(default)]
    pub scrapling_traffic: LaneCounterState,
    #[serde(default)]
    pub bot_red_team: LaneCounterState,
    #[serde(default)]
    pub request_failure_classes: RequestFailureClassCounters,
}

impl LaneDiagnosticsState {
    pub(crate) fn lane_mut(&mut self, lane: RuntimeLane) -> &mut LaneCounterState {
        match lane {
            RuntimeLane::SyntheticTraffic => &mut self.synthetic_traffic,
            RuntimeLane::ScraplingTraffic => &mut self.scrapling_traffic,
            RuntimeLane::BotRedTeam => &mut self.bot_red_team,
        }
    }

    pub(crate) fn failure_class_mut(&mut self, class: WorkerFailureClass) -> &mut FailureClassCounter {
        match class {
            WorkerFailureClass::Cancelled => &mut self.request_failure_classes.cancelled,
            WorkerFailureClass::Timeout => &mut self.request_failure_classes.timeout,
            WorkerFailureClass::Transport => &mut self.request_failure_classes.transport,
            WorkerFailureClass::Http => &mut self.request_failure_classes.http,
        }
    }

    pub(crate) fn to_payload(&self) -> serde_json::Value {
        let lane_payload = |lane: &LaneCounterState| {
            json!({
                "beat_attempts": lane.beat_attempts,
                "beat_successes": lane.beat_successes,
                "beat_failures": lane.beat_failures,
                "generated_requests": lane.generated_requests,
                "blocked_requests": lane.blocked_requests,
                "offsite_requests": lane.offsite_requests,
                "response_bytes": lane.response_bytes,
                "response_status_count": lane.response_status_count,
                "last_generated_at": lane.last_generated_at,
                "last_error": lane.last_error
            })
        };
        let failure_payload = |counter: &FailureClassCounter| {
            json!({
                "count": counter.count,
                "last_seen_at": counter.last_seen_at
            })
        };
        json!({
            "schema_version": LANE_DIAGNOSTICS_SCHEMA_VERSION,
            "lanes": {
                "synthetic_traffic": lane_payload(&self.synthetic_traffic),
                "scrapling_traffic": lane_payload(&self.scrapling_traffic),
                "bot_red_team": lane_payload(&self.bot_red_team)
            },
            "request_failure_classes": {
                "cancelled": failure_payload(&self.request_failure_classes.cancelled),
                "timeout": failure_payload(&self.request_failure_classes.timeout),
                "transport": failure_payload(&self.request_failure_classes.transport),
                "http": failure_payload(&self.request_failure_classes.http)
            }
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct GenerationDiagnostics {
    pub health: String,
    pub reason: String,
    pub recommended_action: String,
    pub generated_tick_count: u64,
    pub generated_request_count: u64,
    pub last_generated_at: Option<u64>,
    pub last_generation_error: Option<String>,
}

fn recurrence_state(state: &ControlState, now: u64) -> &'static str {
    if state.phase != ControlPhase::Running {
        return "inactive";
    }
    match state.recurrence_dormant_until {
        Some(dormant_until) if now < dormant_until => "dormant_gap",
        _ => "ready",
    }
}

pub fn status_payload(
    now: u64,
    runtime_environment: crate::config::RuntimeEnvironment,
    env_available: bool,
    cfg_enabled: bool,
    cfg_duration_seconds: u64,
    state: &ControlState,
) -> serde_json::Value {
    let active_lane = effective_active_lane(state);
    let duration_seconds = clamp_duration_seconds(cfg_duration_seconds);
    let remaining_seconds = match (state.phase, state.ends_at) {
        (ControlPhase::Running, Some(ends_at)) => ends_at.saturating_sub(now),
        _ => 0,
    };

    json!({
        "runtime_environment": runtime_environment.as_str(),
        "gateway_deployment_profile": crate::config::gateway_deployment_profile().as_str(),
        "adversary_sim_available": env_available,
        "adversary_sim_enabled": cfg_enabled,
        "phase": state.phase.as_str(),
        "run_id": state.run_id.clone(),
        "started_at": state.started_at,
        "ends_at": state.ends_at,
        "duration_seconds": duration_seconds,
        "remaining_seconds": remaining_seconds,
        "active_run_count": state.active_run_count,
        "active_lane_count": state.active_lane_count,
        "desired_lane": state.desired_lane.as_str(),
        "active_lane": active_lane.map(RuntimeLane::as_str),
        "lane_switch_seq": state.lane_switch_seq,
        "last_lane_switch_at": state.last_lane_switch_at,
        "last_lane_switch_reason": state.last_lane_switch_reason.clone(),
        "lanes": {
            "deterministic": lane_phase(state.phase),
            "containerized": lane_phase(state.phase)
        },
        "lane_diagnostics": state.lane_diagnostics.to_payload(),
        "guardrails": {
            "surface_available_by_default": crate::config::adversary_sim_available_default(),
            "generation_default": PRODUCTION_GENERATION_DEFAULT,
            "generation_requires_explicit_enable": true,
            "max_duration_seconds": crate::config::ADVERSARY_SIM_DURATION_SECONDS_MAX,
            "max_concurrent_runs": MAX_CONCURRENT_RUNS,
            "cpu_cap_millicores": MAX_CPU_MILLICORES,
            "memory_cap_mib": MAX_MEMORY_MIB,
            "queue_policy": QUEUE_POLICY
        },
        "queue_policy": QUEUE_POLICY,
        "deterministic_attack_corpus": deterministic_corpus_metadata_payload(),
        "last_transition_reason": state.last_transition_reason.clone(),
        "last_terminal_failure_reason": state.last_terminal_failure_reason.clone(),
        "last_run_id": state.last_run_id.clone(),
        "generation": {
            "tick_count": state.generated_tick_count,
            "request_count": state.generated_request_count,
            "last_generated_at": state.last_generated_at,
            "last_generation_error": state.last_generation_error.clone()
        },
        "recurrence": {
            "state": recurrence_state(state, now),
            "strategy": state.recurrence_strategy.clone(),
            "session_index": state.recurrence_session_index,
            "reentry_count": state.recurrence_reentry_count,
            "max_reentries_per_run": state.recurrence_max_reentries_per_run,
            "last_planned_gap_seconds": state.recurrence_last_planned_gap_seconds,
            "dormant_until": state.recurrence_dormant_until
        }
    })
}

pub fn generation_diagnostics(
    now: u64,
    cfg_enabled: bool,
    state: &ControlState,
) -> GenerationDiagnostics {
    let profile = autonomous_execution_profile();
    let diagnostic_grace_seconds = generation_diagnostic_grace_seconds(profile);
    let mut health = "inactive".to_string();
    let mut reason = "simulation_off".to_string();
    let mut recommended_action = "Enable adversary simulation to generate telemetry.".to_string();
    if state.phase == ControlPhase::Running && cfg_enabled {
        if state.recurrence_dormant_until.map(|deadline| now < deadline).unwrap_or(false) {
            health = "healthy".to_string();
            reason = "recurrence_dormant_gap".to_string();
            recommended_action =
                "No action required; the active lane is intentionally dormant between bounded re-entry sessions."
                    .to_string();
            return GenerationDiagnostics {
                health,
                reason,
                recommended_action,
                generated_tick_count: state.generated_tick_count,
                generated_request_count: state.generated_request_count,
                last_generated_at: state.last_generated_at,
                last_generation_error: state.last_generation_error.clone(),
            };
        }
        let has_error = state
            .last_generation_error
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let started_at = state.started_at.unwrap_or(now);
        let idle_window_elapsed = now >= started_at.saturating_add(diagnostic_grace_seconds);
        if has_error {
            health = "error".to_string();
            reason = "tick_execution_failed".to_string();
            recommended_action = "Inspect generation_diagnostics.last_generation_error and restart the run if needed.".to_string();
        } else if state.generated_request_count == 0 && idle_window_elapsed {
            health = "no_traffic".to_string();
            reason = if profile.cron_schedule.is_some() {
                "edge_cron_no_traffic_yet".to_string()
            } else {
                "supervisor_no_traffic_yet".to_string()
            };
            recommended_action = if profile.cron_schedule.is_some() {
                "Verify the edge cron heartbeat is provisioned and allow one full cron interval for first generated traffic."
                    .to_string()
            } else {
                "Verify backend supervisor heartbeat diagnostics and confirm simulation remains running."
                    .to_string()
            };
        } else if let Some(last_generated_at) = state.last_generated_at {
            if now >= last_generated_at.saturating_add(diagnostic_grace_seconds) {
                health = "stalled".to_string();
                reason = if profile.cron_schedule.is_some() {
                    "edge_cron_tick_stalled".to_string()
                } else {
                    "supervisor_tick_stalled".to_string()
                };
                recommended_action = if profile.cron_schedule.is_some() {
                    "Check edge cron scheduling state and re-enable adversary simulation if needed."
                        .to_string()
                } else {
                    "Check backend supervisor heartbeat state and re-enable adversary simulation if needed."
                        .to_string()
                };
            } else {
                health = "ok".to_string();
                reason = "traffic_observed".to_string();
                recommended_action =
                    "No action required; simulation traffic is being generated.".to_string();
            }
        } else {
            health = "warming".to_string();
            reason = if profile.cron_schedule.is_some() {
                "waiting_for_first_edge_cron_tick".to_string()
            } else {
                "waiting_for_first_supervisor_tick".to_string()
            };
            recommended_action = if profile.cron_schedule.is_some() {
                "Allow one full cron interval for first generated traffic.".to_string()
            } else {
                "Allow one heartbeat interval for first generated traffic.".to_string()
            };
        }
    } else if cfg_enabled {
        health = "degraded".to_string();
        reason = "controller_not_running".to_string();
        recommended_action =
            "Toggle adversary simulation off then on to reconcile desired/actual state.".to_string();
    }
    GenerationDiagnostics {
        health,
        reason,
        recommended_action,
        generated_tick_count: state.generated_tick_count,
        generated_request_count: state.generated_request_count,
        last_generated_at: state.last_generated_at,
        last_generation_error: state.last_generation_error.clone(),
    }
}

pub fn supervisor_status_payload(
    now: u64,
    cfg_enabled: bool,
    state: &ControlState,
) -> serde_json::Value {
    let profile = autonomous_execution_profile();
    let heartbeat_expected = cfg_enabled && state.phase == ControlPhase::Running;
    let heartbeat_active = heartbeat_expected
        && state
            .last_generated_at
            .map(|last_generated_at| {
                now < last_generated_at.saturating_add(profile.cadence_seconds.saturating_mul(2))
            })
            .unwrap_or(false);
    let off_state_inert = !cfg_enabled
        && state.phase == ControlPhase::Off
        && state.active_run_count == 0
        && state.active_lane_count == 0;
    let idle_seconds = state
        .last_generated_at
        .map(|last_generated_at| now.saturating_sub(last_generated_at));
    json!({
        "owner": "backend_autonomous_supervisor",
        "deployment_profile": crate::config::gateway_deployment_profile().as_str(),
        "cadence_seconds": profile.cadence_seconds,
        "max_catchup_ticks_per_invocation": profile.max_catchup_ticks_per_invocation,
        "heartbeat_expected": heartbeat_expected,
        "heartbeat_active": heartbeat_active,
        "worker_active": heartbeat_active,
        "last_heartbeat_at": state.last_generated_at,
        "idle_seconds": idle_seconds,
        "off_state_inert": off_state_inert,
        "trigger_surface": profile.trigger_surface,
        "beat_endpoint": profile.beat_endpoint,
        "cron_schedule": profile.cron_schedule,
        "deterministic_attack_corpus": deterministic_corpus_metadata_payload()
    })
}
