use once_cell::sync::Lazy;
use rand::random;
use serde::{Deserialize, Serialize};

use crate::challenge::KeyValueStore;

use super::adversary_sim::{
    LaneDiagnosticsState, MAX_CONCURRENT_RUNS, STOP_TIMEOUT_SECONDS,
    AUTONOMOUS_EDGE_FERMYON_CRON_SCHEDULE, AUTONOMOUS_EDGE_FERMYON_HEARTBEAT_INTERVAL_SECONDS,
    AUTONOMOUS_EDGE_FERMYON_MAX_CATCHUP_TICKS_PER_INVOCATION,
    AUTONOMOUS_SHARED_SERVER_HEARTBEAT_INTERVAL_SECONDS,
    AUTONOMOUS_SHARED_SERVER_MAX_CATCHUP_TICKS_PER_INVOCATION,
};
use super::adversary_sim_corpus::deterministic_runtime_profile;

const STATE_KEY_PREFIX: &str = "adversary_sim:control:";
const GENERATION_DIAGNOSTIC_GRACE_SECONDS: u64 = 5;

static PROCESS_INSTANCE_ID: Lazy<String> = Lazy::new(|| {
    std::env::var("RUNTIME_INSTANCE_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime-instance-unknown".to_string())
});

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ControlPhase {
    #[default]
    Off,
    Running,
    Stopping,
}

impl ControlPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Running => "running",
            Self::Stopping => "stopping",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeLane {
    #[default]
    SyntheticTraffic,
    ScraplingTraffic,
    BotRedTeam,
    ParallelMixedTraffic,
}

impl RuntimeLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SyntheticTraffic => "synthetic_traffic",
            Self::ScraplingTraffic => "scrapling_traffic",
            Self::BotRedTeam => "bot_red_team",
            Self::ParallelMixedTraffic => "parallel_mixed_traffic",
        }
    }

    pub fn includes_worker_lane(self, worker_lane: RuntimeLane) -> bool {
        match self {
            Self::ParallelMixedTraffic => matches!(
                worker_lane,
                Self::ScraplingTraffic | Self::BotRedTeam
            ),
            _ => self == worker_lane,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlState {
    #[serde(default)]
    pub phase: ControlPhase,
    #[serde(default)]
    pub desired_enabled: bool,
    #[serde(default)]
    pub desired_lane: RuntimeLane,
    #[serde(default)]
    pub owner_instance_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub started_at: Option<u64>,
    #[serde(default)]
    pub ends_at: Option<u64>,
    #[serde(default)]
    pub stop_deadline: Option<u64>,
    #[serde(default)]
    pub active_run_count: u32,
    #[serde(default)]
    pub active_lane_count: u32,
    #[serde(default)]
    pub active_lane: Option<RuntimeLane>,
    #[serde(default)]
    pub lane_switch_seq: u64,
    #[serde(default)]
    pub last_lane_switch_at: Option<u64>,
    #[serde(default)]
    pub last_lane_switch_reason: Option<String>,
    #[serde(default)]
    pub last_transition_reason: Option<String>,
    #[serde(default)]
    pub last_terminal_failure_reason: Option<String>,
    #[serde(default)]
    pub last_run_id: Option<String>,
    #[serde(default)]
    pub generated_tick_count: u64,
    #[serde(default)]
    pub generated_request_count: u64,
    #[serde(default)]
    pub last_generated_at: Option<u64>,
    #[serde(default)]
    pub last_generation_error: Option<String>,
    #[serde(default)]
    pub pending_scrapling_tick_id: Option<String>,
    #[serde(default)]
    pub pending_scrapling_started_at: Option<u64>,
    #[serde(default)]
    pub pending_llm_tick_id: Option<String>,
    #[serde(default)]
    pub pending_llm_started_at: Option<u64>,
    #[serde(default)]
    pub recurrence_strategy: Option<String>,
    #[serde(default)]
    pub recurrence_session_index: u64,
    #[serde(default)]
    pub recurrence_reentry_count: u64,
    #[serde(default)]
    pub recurrence_max_reentries_per_run: Option<u64>,
    #[serde(default)]
    pub recurrence_last_planned_gap_seconds: Option<u64>,
    #[serde(default)]
    pub recurrence_dormant_until: Option<u64>,
    #[serde(default)]
    pub lane_diagnostics: LaneDiagnosticsState,
    #[serde(default)]
    pub updated_at: u64,
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            phase: ControlPhase::Off,
            desired_enabled: false,
            desired_lane: RuntimeLane::ScraplingTraffic,
            owner_instance_id: None,
            run_id: None,
            started_at: None,
            ends_at: None,
            stop_deadline: None,
            active_run_count: 0,
            active_lane_count: 0,
            active_lane: None,
            lane_switch_seq: 0,
            last_lane_switch_at: None,
            last_lane_switch_reason: None,
            last_transition_reason: None,
            last_terminal_failure_reason: None,
            last_run_id: None,
            generated_tick_count: 0,
            generated_request_count: 0,
            last_generated_at: None,
            last_generation_error: None,
            pending_scrapling_tick_id: None,
            pending_scrapling_started_at: None,
            pending_llm_tick_id: None,
            pending_llm_started_at: None,
            recurrence_strategy: None,
            recurrence_session_index: 0,
            recurrence_reentry_count: 0,
            recurrence_max_reentries_per_run: None,
            recurrence_last_planned_gap_seconds: None,
            recurrence_dormant_until: None,
            lane_diagnostics: LaneDiagnosticsState::default(),
            updated_at: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutonomousExecutionProfile {
    pub cadence_seconds: u64,
    pub max_catchup_ticks_per_invocation: u64,
    pub trigger_surface: &'static str,
    pub beat_endpoint: &'static str,
    pub cron_schedule: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub from: ControlPhase,
    pub to: ControlPhase,
    pub reason: String,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartError {
    QueueFull,
}

pub fn control_surface_available(
    _runtime_environment: crate::config::RuntimeEnvironment,
    env_available: bool,
) -> bool {
    env_available
}

pub fn autonomous_execution_profile() -> AutonomousExecutionProfile {
    match crate::config::gateway_deployment_profile() {
        crate::config::GatewayDeploymentProfile::SharedServer => AutonomousExecutionProfile {
            cadence_seconds: AUTONOMOUS_SHARED_SERVER_HEARTBEAT_INTERVAL_SECONDS,
            max_catchup_ticks_per_invocation:
                AUTONOMOUS_SHARED_SERVER_MAX_CATCHUP_TICKS_PER_INVOCATION,
            trigger_surface: "internal_beat_endpoint",
            beat_endpoint: crate::http_route_namespace::SHUMA_INTERNAL_ADVERSARY_SIM_BEAT_PATH,
            cron_schedule: None,
        },
        crate::config::GatewayDeploymentProfile::EdgeFermyon => AutonomousExecutionProfile {
            cadence_seconds: AUTONOMOUS_EDGE_FERMYON_HEARTBEAT_INTERVAL_SECONDS,
            max_catchup_ticks_per_invocation:
                AUTONOMOUS_EDGE_FERMYON_MAX_CATCHUP_TICKS_PER_INVOCATION,
            trigger_surface: "edge_cron",
            beat_endpoint: crate::http_route_namespace::SHUMA_INTERNAL_ADVERSARY_SIM_BEAT_PATH,
            cron_schedule: Some(AUTONOMOUS_EDGE_FERMYON_CRON_SCHEDULE),
        },
    }
}

pub(crate) fn generation_diagnostic_grace_seconds(profile: AutonomousExecutionProfile) -> u64 {
    profile
        .cadence_seconds
        .saturating_add(GENERATION_DIAGNOSTIC_GRACE_SECONDS)
}

pub fn process_instance_id() -> &'static str {
    PROCESS_INSTANCE_ID.as_str()
}

fn requires_single_process_ownership() -> bool {
    !crate::config::gateway_deployment_profile().is_edge()
}

pub fn state_key(site_id: &str) -> String {
    format!("{}{}", STATE_KEY_PREFIX, site_id)
}

pub fn clamp_duration_seconds(value: u64) -> u64 {
    value.clamp(
        crate::config::ADVERSARY_SIM_DURATION_SECONDS_MIN,
        crate::config::ADVERSARY_SIM_DURATION_SECONDS_MAX,
    )
}

pub fn load_state<S: KeyValueStore>(store: &S, site_id: &str) -> ControlState {
    let key = state_key(site_id);
    let Some(raw) = store.get(&key).ok().flatten() else {
        return ControlState::default();
    };
    serde_json::from_slice::<ControlState>(&raw).unwrap_or_default()
}

pub fn save_state<S: KeyValueStore>(store: &S, site_id: &str, state: &ControlState) -> Result<(), ()> {
    let key = state_key(site_id);
    let payload = serde_json::to_vec(state).map_err(|_| ())?;
    store.set(&key, &payload)
}

fn state_has_authoritative_desired_state(state: &ControlState) -> bool {
    state != &ControlState::default()
}

pub fn effective_desired_enabled(initial_seed_enabled: bool, state: &ControlState) -> bool {
    if state_has_authoritative_desired_state(state) {
        state.desired_enabled
    } else {
        initial_seed_enabled
    }
}

pub fn project_effective_desired_state(cfg: &mut crate::config::Config, state: &ControlState) {
    cfg.adversary_sim_enabled = effective_desired_enabled(cfg.adversary_sim_enabled, state);
}

pub(crate) fn active_lane_count_for_lane(lane: RuntimeLane) -> u32 {
    match lane {
        RuntimeLane::SyntheticTraffic => deterministic_runtime_profile().active_lane_count,
        RuntimeLane::ScraplingTraffic => 1,
        RuntimeLane::BotRedTeam => 0,
        RuntimeLane::ParallelMixedTraffic => 2,
    }
}

pub(crate) fn lane_pending_tick_id<'a>(
    state: &'a ControlState,
    lane: RuntimeLane,
) -> Option<&'a str> {
    match lane {
        RuntimeLane::ScraplingTraffic => state.pending_scrapling_tick_id.as_deref(),
        RuntimeLane::BotRedTeam => state.pending_llm_tick_id.as_deref(),
        RuntimeLane::SyntheticTraffic | RuntimeLane::ParallelMixedTraffic => None,
    }
}

pub(crate) fn lane_has_pending_worker(state: &ControlState, lane: RuntimeLane) -> bool {
    match lane {
        RuntimeLane::ScraplingTraffic => state.pending_scrapling_tick_id.is_some(),
        RuntimeLane::BotRedTeam => state.pending_llm_tick_id.is_some(),
        RuntimeLane::ParallelMixedTraffic => {
            state.pending_scrapling_tick_id.is_some() || state.pending_llm_tick_id.is_some()
        }
        RuntimeLane::SyntheticTraffic => false,
    }
}

pub(crate) fn set_lane_pending_worker(
    state: &mut ControlState,
    lane: RuntimeLane,
    tick_id: String,
    started_at: u64,
) {
    match lane {
        RuntimeLane::ScraplingTraffic => {
            state.pending_scrapling_tick_id = Some(tick_id);
            state.pending_scrapling_started_at = Some(started_at);
        }
        RuntimeLane::BotRedTeam => {
            state.pending_llm_tick_id = Some(tick_id);
            state.pending_llm_started_at = Some(started_at);
        }
        RuntimeLane::SyntheticTraffic | RuntimeLane::ParallelMixedTraffic => {}
    }
}

pub(crate) fn clear_lane_pending_worker(state: &mut ControlState, lane: RuntimeLane) {
    match lane {
        RuntimeLane::ScraplingTraffic => {
            state.pending_scrapling_tick_id = None;
            state.pending_scrapling_started_at = None;
        }
        RuntimeLane::BotRedTeam => {
            state.pending_llm_tick_id = None;
            state.pending_llm_started_at = None;
        }
        RuntimeLane::ParallelMixedTraffic => {
            state.pending_scrapling_tick_id = None;
            state.pending_scrapling_started_at = None;
            state.pending_llm_tick_id = None;
            state.pending_llm_started_at = None;
        }
        RuntimeLane::SyntheticTraffic => {}
    }
}

pub fn start_state(
    now: u64,
    duration_seconds: u64,
    current: &ControlState,
) -> Result<(ControlState, Vec<Transition>), StartError> {
    start_state_with_reason(now, duration_seconds, current, "manual_on")
}

pub fn start_state_with_reason(
    now: u64,
    duration_seconds: u64,
    current: &ControlState,
    reason: &str,
) -> Result<(ControlState, Vec<Transition>), StartError> {
    if current.phase == ControlPhase::Running && current.active_run_count >= MAX_CONCURRENT_RUNS {
        return Err(StartError::QueueFull);
    }
    let run_id = format!("simrun-{}-{:016x}", now, random::<u64>());
    let transition = Transition {
        from: current.phase,
        to: ControlPhase::Running,
        reason: reason.to_string(),
        run_id: Some(run_id.clone()),
    };
    let desired_lane = current.desired_lane;
    let next = ControlState {
        phase: ControlPhase::Running,
        desired_enabled: true,
        desired_lane,
        owner_instance_id: Some(process_instance_id().to_string()),
        run_id: Some(run_id),
        started_at: Some(now),
        ends_at: Some(now.saturating_add(clamp_duration_seconds(duration_seconds))),
        stop_deadline: None,
        active_run_count: 1,
        active_lane_count: active_lane_count_for_lane(desired_lane),
        active_lane: Some(desired_lane),
        lane_switch_seq: current.lane_switch_seq,
        last_lane_switch_at: current.last_lane_switch_at,
        last_lane_switch_reason: current.last_lane_switch_reason.clone(),
        last_transition_reason: Some(reason.to_string()),
        last_terminal_failure_reason: None,
        last_run_id: current.last_run_id.clone(),
        generated_tick_count: 0,
        generated_request_count: 0,
        last_generated_at: None,
        last_generation_error: None,
        pending_scrapling_tick_id: None,
        pending_scrapling_started_at: None,
        pending_llm_tick_id: None,
        pending_llm_started_at: None,
        recurrence_strategy: None,
        recurrence_session_index: 1,
        recurrence_reentry_count: 0,
        recurrence_max_reentries_per_run: None,
        recurrence_last_planned_gap_seconds: None,
        recurrence_dormant_until: None,
        lane_diagnostics: current.lane_diagnostics.clone(),
        updated_at: now,
    };
    Ok((next, vec![transition]))
}

pub fn stop_state(now: u64, reason: &str, current: &ControlState) -> (ControlState, Vec<Transition>) {
    if current.phase == ControlPhase::Off
        && current.active_run_count == 0
        && current.active_lane_count == 0
    {
        return (current.clone(), Vec::new());
    }

    let mut next = current.clone();
    next.desired_enabled = false;
    next.owner_instance_id = Some(process_instance_id().to_string());
    next.phase = ControlPhase::Stopping;
    next.stop_deadline = Some(now.saturating_add(STOP_TIMEOUT_SECONDS));
    next.last_transition_reason = Some(reason.to_string());
    next.active_run_count = 0;
    next.active_lane_count = 0;
    next.active_lane = None;
    next.pending_scrapling_tick_id = None;
    next.pending_scrapling_started_at = None;
    next.pending_llm_tick_id = None;
    next.pending_llm_started_at = None;
    next.recurrence_strategy = None;
    next.recurrence_session_index = 0;
    next.recurrence_reentry_count = 0;
    next.recurrence_max_reentries_per_run = None;
    next.recurrence_last_planned_gap_seconds = None;
    next.recurrence_dormant_until = None;
    next.updated_at = now;

    let transition = Transition {
        from: current.phase,
        to: ControlPhase::Stopping,
        reason: reason.to_string(),
        run_id: current.run_id.clone(),
    };
    (next, vec![transition])
}

pub fn reconcile_state(
    now: u64,
    cfg_enabled: bool,
    current: &ControlState,
) -> (ControlState, Vec<Transition>) {
    let mut next = current.clone();
    next.desired_enabled = cfg_enabled;
    let mut transitions: Vec<Transition> = Vec::new();

    if requires_single_process_ownership()
        && next.phase != ControlPhase::Off
        && next.owner_instance_id.as_deref() != Some(process_instance_id())
    {
        let (stopping, mut phase_transitions) = stop_state(now, "process_restart", &next);
        next = stopping;
        transitions.append(&mut phase_transitions);
    }

    if next.phase == ControlPhase::Running {
        let should_stop_for_disabled = !cfg_enabled;
        let should_stop_for_window = next.ends_at.map(|end| now >= end).unwrap_or(false);
        if should_stop_for_disabled || should_stop_for_window {
            let reason = if should_stop_for_disabled {
                "config_disabled"
            } else {
                "auto_window_expired"
            };
            let (stopping, mut phase_transitions) = stop_state(now, reason, &next);
            next = stopping;
            transitions.append(&mut phase_transitions);
        }
    }

    if next.phase == ControlPhase::Stopping {
        if next.active_run_count == 0 && next.active_lane_count == 0 {
            let run_id = next.run_id.clone();
            next.last_run_id = run_id.clone().or_else(|| next.last_run_id.clone());
            let reason = next
                .last_transition_reason
                .clone()
                .unwrap_or_else(|| "manual_off".to_string());
            transitions.push(Transition {
                from: ControlPhase::Stopping,
                to: ControlPhase::Off,
                reason,
                run_id,
            });
            next.phase = ControlPhase::Off;
            next.run_id = None;
            next.started_at = None;
            next.ends_at = None;
            next.stop_deadline = None;
            next.active_run_count = 0;
            next.active_lane_count = 0;
            next.active_lane = None;
            next.pending_scrapling_tick_id = None;
            next.pending_scrapling_started_at = None;
            next.pending_llm_tick_id = None;
            next.pending_llm_started_at = None;
            next.recurrence_strategy = None;
            next.recurrence_session_index = 0;
            next.recurrence_reentry_count = 0;
            next.recurrence_max_reentries_per_run = None;
            next.recurrence_last_planned_gap_seconds = None;
            next.recurrence_dormant_until = None;
            next.updated_at = now;
        } else if next.stop_deadline.map(|deadline| now >= deadline).unwrap_or(false) {
            let run_id = next.run_id.clone();
            next.last_run_id = run_id.clone().or_else(|| next.last_run_id.clone());
            transitions.push(Transition {
                from: ControlPhase::Stopping,
                to: ControlPhase::Off,
                reason: "forced_kill_timeout".to_string(),
                run_id,
            });
            next.phase = ControlPhase::Off;
            next.run_id = None;
            next.started_at = None;
            next.ends_at = None;
            next.stop_deadline = None;
            next.active_run_count = 0;
            next.active_lane_count = 0;
            next.active_lane = None;
            next.last_transition_reason = Some("forced_kill_timeout".to_string());
            next.last_terminal_failure_reason = Some("forced_kill_timeout".to_string());
            next.pending_scrapling_tick_id = None;
            next.pending_scrapling_started_at = None;
            next.pending_llm_tick_id = None;
            next.pending_llm_started_at = None;
            next.recurrence_strategy = None;
            next.recurrence_session_index = 0;
            next.recurrence_reentry_count = 0;
            next.recurrence_max_reentries_per_run = None;
            next.recurrence_last_planned_gap_seconds = None;
            next.recurrence_dormant_until = None;
            next.updated_at = now;
        }
    }

    if next.phase == ControlPhase::Off {
        next.active_run_count = 0;
        next.active_lane_count = 0;
        next.active_lane = None;
        next.pending_scrapling_tick_id = None;
        next.pending_scrapling_started_at = None;
        next.pending_llm_tick_id = None;
        next.pending_llm_started_at = None;
        next.recurrence_strategy = None;
        next.recurrence_session_index = 0;
        next.recurrence_reentry_count = 0;
        next.recurrence_max_reentries_per_run = None;
        next.recurrence_last_planned_gap_seconds = None;
        next.recurrence_dormant_until = None;
    }

    (next, transitions)
}

pub(crate) fn lane_phase(phase: ControlPhase) -> &'static str {
    match phase {
        ControlPhase::Off => "off",
        ControlPhase::Running => "running",
        ControlPhase::Stopping => "stopping",
    }
}

pub fn effective_active_lane(state: &ControlState) -> Option<RuntimeLane> {
    match state.phase {
        ControlPhase::Running => state.active_lane.or(Some(state.desired_lane)),
        ControlPhase::Off | ControlPhase::Stopping => None,
    }
}

pub fn lane_reconciliation_needed(state: &ControlState) -> bool {
    matches!(state.phase, ControlPhase::Running)
        && effective_active_lane(state) != Some(state.desired_lane)
}

pub fn select_desired_lane(now: u64, desired_lane: RuntimeLane, current: &ControlState) -> ControlState {
    if current.desired_lane == desired_lane {
        return current.clone();
    }
    let mut next = current.clone();
    next.desired_lane = desired_lane;
    next.updated_at = now;
    next
}
