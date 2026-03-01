use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[cfg(not(test))]
use spin_sdk::http::{Method, Request};

use crate::challenge::KeyValueStore;

pub const MAX_CONCURRENT_RUNS: u32 = 1;
pub const MAX_CPU_MILLICORES: u32 = 1000;
pub const MAX_MEMORY_MIB: u32 = 512;
pub const QUEUE_POLICY: &str = "reject_new";
pub const STOP_TIMEOUT_SECONDS: u64 = 10;
const ACTIVE_LANE_COUNT: u32 = 2;
const INTERNAL_GENERATION_BATCH_SIZE: u64 = 6;
const INTERNAL_RATE_BURST_REQUESTS: usize = 6;
const INTERNAL_RATE_BURST_EVERY_N_TICKS: u64 = 2;
const INTERNAL_RATE_BURST_IP: &str = "198.51.254.10";
const INTERNAL_CDP_REPORT_IP: &str = "198.51.254.20";
const GENERATION_DIAGNOSTIC_GRACE_SECONDS: u64 = 5;
const STATE_KEY_PREFIX: &str = "adversary_sim:control:";

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlState {
    #[serde(default)]
    pub phase: ControlPhase,
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
    pub updated_at: u64,
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            phase: ControlPhase::Off,
            run_id: None,
            started_at: None,
            ends_at: None,
            stop_deadline: None,
            active_run_count: 0,
            active_lane_count: 0,
            last_transition_reason: None,
            last_terminal_failure_reason: None,
            last_run_id: None,
            generated_tick_count: 0,
            generated_request_count: 0,
            last_generated_at: None,
            last_generation_error: None,
            updated_at: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerationDiagnostics {
    pub health: String,
    pub reason: String,
    pub recommended_action: String,
    pub generated_tick_count: u64,
    pub generated_request_count: u64,
    pub last_generated_at: Option<u64>,
    pub last_generation_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerationTickResult {
    pub generated_requests: u64,
    pub failed_requests: u64,
    pub last_response_status: Option<u16>,
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
    runtime_environment: crate::config::RuntimeEnvironment,
    env_available: bool,
) -> bool {
    runtime_environment.is_dev() && env_available
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

pub fn start_state(
    now: u64,
    duration_seconds: u64,
    current: &ControlState,
) -> Result<(ControlState, Vec<Transition>), StartError> {
    if current.phase == ControlPhase::Running && current.active_run_count >= MAX_CONCURRENT_RUNS {
        return Err(StartError::QueueFull);
    }
    let run_id = format!("simrun-{}-{:016x}", now, random::<u64>());
    let transition = Transition {
        from: current.phase,
        to: ControlPhase::Running,
        reason: "manual_on".to_string(),
        run_id: Some(run_id.clone()),
    };
    let next = ControlState {
        phase: ControlPhase::Running,
        run_id: Some(run_id),
        started_at: Some(now),
        ends_at: Some(now.saturating_add(clamp_duration_seconds(duration_seconds))),
        stop_deadline: None,
        active_run_count: 1,
        active_lane_count: ACTIVE_LANE_COUNT,
        last_transition_reason: Some("manual_on".to_string()),
        last_terminal_failure_reason: None,
        last_run_id: current.last_run_id.clone(),
        generated_tick_count: 0,
        generated_request_count: 0,
        last_generated_at: None,
        last_generation_error: None,
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
    next.phase = ControlPhase::Stopping;
    next.stop_deadline = Some(now.saturating_add(STOP_TIMEOUT_SECONDS));
    next.last_transition_reason = Some(reason.to_string());
    // Current stop path is synchronous; the forced-kill path still protects stale/stuck state.
    next.active_run_count = 0;
    next.active_lane_count = 0;
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
    let mut transitions: Vec<Transition> = Vec::new();

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
            next.last_transition_reason = Some("forced_kill_timeout".to_string());
            next.last_terminal_failure_reason = Some("forced_kill_timeout".to_string());
            next.updated_at = now;
        }
    }

    if next.phase == ControlPhase::Off {
        next.active_run_count = 0;
        next.active_lane_count = 0;
    }

    (next, transitions)
}

fn lane_phase(phase: ControlPhase) -> &'static str {
    match phase {
        ControlPhase::Off => "off",
        ControlPhase::Running => "running",
        ControlPhase::Stopping => "stopping",
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
    let duration_seconds = clamp_duration_seconds(cfg_duration_seconds);
    let remaining_seconds = match (state.phase, state.ends_at) {
        (ControlPhase::Running, Some(ends_at)) => ends_at.saturating_sub(now),
        _ => 0,
    };

    json!({
        "runtime_environment": runtime_environment.as_str(),
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
        "lanes": {
            "deterministic": lane_phase(state.phase),
            "containerized": lane_phase(state.phase)
        },
        "guardrails": {
            "max_duration_seconds": crate::config::ADVERSARY_SIM_DURATION_SECONDS_MAX,
            "max_concurrent_runs": MAX_CONCURRENT_RUNS,
            "cpu_cap_millicores": MAX_CPU_MILLICORES,
            "memory_cap_mib": MAX_MEMORY_MIB,
            "queue_policy": QUEUE_POLICY
        },
        "queue_policy": QUEUE_POLICY,
        "last_transition_reason": state.last_transition_reason.clone(),
        "last_terminal_failure_reason": state.last_terminal_failure_reason.clone(),
        "last_run_id": state.last_run_id.clone(),
        "generation": {
            "tick_count": state.generated_tick_count,
            "request_count": state.generated_request_count,
            "last_generated_at": state.last_generated_at,
            "last_generation_error": state.last_generation_error.clone()
        }
    })
}

#[cfg(not(test))]
fn simulated_request_paths(run_id: &str, tick_count: u64) -> [String; 4] {
    let run_suffix = run_id
        .chars()
        .rev()
        .take(8)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    [
        "/sim/public/landing".to_string(),
        "/sim/public/docs".to_string(),
        format!("/sim/public/search?q=run-{}-tick-{}", run_suffix, tick_count),
        "/instaban".to_string(),
    ]
}

#[cfg(not(test))]
fn simulated_request_ip(tick_count: u64, index: usize) -> String {
    let offset = tick_count
        .saturating_mul(INTERNAL_GENERATION_BATCH_SIZE)
        .saturating_add(index as u64);
    let third = ((offset / 254) % 254) + 1;
    let fourth = (offset % 254) + 1;
    format!("198.51.{}.{}", third, fourth)
}

pub fn generation_diagnostics(
    now: u64,
    cfg_enabled: bool,
    state: &ControlState,
) -> GenerationDiagnostics {
    let mut health = "inactive".to_string();
    let mut reason = "simulation_off".to_string();
    let mut recommended_action = "Enable adversary simulation to generate telemetry.".to_string();
    if state.phase == ControlPhase::Running && cfg_enabled {
        let has_error = state
            .last_generation_error
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let started_at = state.started_at.unwrap_or(now);
        let idle_window_elapsed = now
            >= started_at.saturating_add(GENERATION_DIAGNOSTIC_GRACE_SECONDS);
        if has_error {
            health = "error".to_string();
            reason = "tick_execution_failed".to_string();
            recommended_action = "Inspect generation_diagnostics.last_generation_error and restart the run if needed.".to_string();
        } else if state.generated_request_count == 0 && idle_window_elapsed {
            health = "no_traffic".to_string();
            reason = "tick_endpoint_not_invoked".to_string();
            recommended_action = "Keep the monitoring dashboard open (or call POST /admin/adversary-sim/tick) while the run is enabled.".to_string();
        } else if let Some(last_generated_at) = state.last_generated_at {
            if now >= last_generated_at.saturating_add(GENERATION_DIAGNOSTIC_GRACE_SECONDS) {
                health = "stalled".to_string();
                reason = "generation_tick_stalled".to_string();
                recommended_action = "Resume dashboard tick traffic generation or run make test-adversarial-live.".to_string();
            } else {
                health = "ok".to_string();
                reason = "traffic_observed".to_string();
                recommended_action =
                    "No action required; simulation traffic is being generated.".to_string();
            }
        } else {
            health = "warming".to_string();
            reason = "waiting_for_first_tick".to_string();
            recommended_action = "Allow one refresh interval for first generated traffic.".to_string();
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

pub fn run_internal_generation_tick(
    store: &impl KeyValueStore,
    state: &mut ControlState,
    now: u64,
) -> GenerationTickResult {
    let mut result = GenerationTickResult {
        generated_requests: 0,
        failed_requests: 0,
        last_response_status: None,
    };
    if state.phase != ControlPhase::Running {
        state.last_generation_error = Some("simulation_not_running".to_string());
        return result;
    }

    let run_id = state
        .run_id
        .clone()
        .or_else(|| state.last_run_id.clone())
        .unwrap_or_else(|| "simrun-runtime".to_string());
    let metadata = crate::runtime::sim_telemetry::SimulationRequestMetadata {
        sim_run_id: run_id.clone(),
        sim_profile: "runtime_toggle".to_string(),
        sim_lane: "deterministic_black_box".to_string(),
    };
    #[cfg(not(test))]
    {
        let mut dispatch_request = |request: Request<Vec<u8>>| {
            let _guard = crate::runtime::sim_telemetry::enter(Some(metadata.clone()));
            let response = crate::handle_bot_defence_impl(&request);
            let status = *response.status();
            result.generated_requests = result.generated_requests.saturating_add(1);
            result.last_response_status = Some(status);
            if status >= 500 {
                result.failed_requests = result.failed_requests.saturating_add(1);
            }
        };

        let paths = simulated_request_paths(run_id.as_str(), state.generated_tick_count);
        for (index, path) in paths.iter().enumerate() {
            let user_agent = format!("ShumaAdversarySim/1.0 slot={} path={}", index, path);
            let mut builder = Request::builder();
            builder
                .method(Method::Get)
                .uri(path.as_str())
                .header("x-forwarded-for", simulated_request_ip(state.generated_tick_count, index))
                .header("x-forwarded-proto", "https")
                .header("user-agent", user_agent.as_str());
            dispatch_request(builder.body(Vec::new()).build());
        }

        let cdp_probe_body = serde_json::to_vec(&json!({
            "cdp_detected": true,
            "score": 4.8,
            "checks": ["webdriver", "automation_props", "cdp_timing", "micro_timing"]
        }))
        .unwrap_or_else(|_| b"{\"cdp_detected\":true,\"score\":4.8,\"checks\":[\"webdriver\"]}".to_vec());
        let mut cdp_builder = Request::builder();
        cdp_builder
            .method(Method::Post)
            .uri("/cdp-report")
            .header("x-forwarded-for", INTERNAL_CDP_REPORT_IP)
            .header("x-forwarded-proto", "https")
            .header("content-type", "application/json")
            .header("user-agent", "ShumaAdversarySim/1.0 cdp-probe");
        dispatch_request(cdp_builder.body(cdp_probe_body).build());

        if state.generated_tick_count % INTERNAL_RATE_BURST_EVERY_N_TICKS == 0 {
            for burst_index in 0..INTERNAL_RATE_BURST_REQUESTS {
                let mut burst_builder = Request::builder();
                let burst_path = format!(
                    "/sim/public/search?q=rate-burst-{}-{}",
                    state.generated_tick_count, burst_index
                );
                let user_agent = format!("ShumaAdversarySim/1.0 rate-burst {}", burst_index);
                burst_builder
                    .method(Method::Get)
                    .uri(burst_path.as_str())
                    .header("x-forwarded-for", INTERNAL_RATE_BURST_IP)
                    .header("x-forwarded-proto", "https")
                    .header("user-agent", user_agent.as_str());
                dispatch_request(burst_builder.body(Vec::new()).build());
            }
        }
        crate::observability::monitoring::flush_pending_counters(store);
    }
    #[cfg(test)]
    {
        let _ = store;
        let _ = metadata;
        result.generated_requests = INTERNAL_GENERATION_BATCH_SIZE;
        result.last_response_status = Some(200);
    }

    state.generated_tick_count = state.generated_tick_count.saturating_add(1);
    state.generated_request_count = state
        .generated_request_count
        .saturating_add(result.generated_requests);
    state.last_generated_at = Some(now);
    if result.failed_requests > 0 {
        state.last_generation_error = Some(format!(
            "request_pipeline_errors={} of {}",
            result.failed_requests, result.generated_requests
        ));
    } else {
        state.last_generation_error = None;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_stop_transitions_reach_off_state() {
        let now = 1_000u64;
        let (started, started_transitions) =
            start_state(now, 180, &ControlState::default()).expect("start");
        assert_eq!(started.phase, ControlPhase::Running);
        assert_eq!(started.active_run_count, 1);
        assert_eq!(started.active_lane_count, 2);
        assert_eq!(started_transitions.len(), 1);
        assert_eq!(started_transitions[0].reason, "manual_on");

        let (stopping, stop_transitions) = stop_state(now + 1, "manual_off", &started);
        assert_eq!(stopping.phase, ControlPhase::Stopping);
        assert_eq!(stopping.active_run_count, 0);
        assert_eq!(stopping.active_lane_count, 0);
        assert_eq!(stop_transitions.len(), 1);

        let (off, reconcile_transitions) = reconcile_state(now + 1, false, &stopping);
        assert_eq!(off.phase, ControlPhase::Off);
        assert_eq!(off.active_run_count, 0);
        assert_eq!(off.active_lane_count, 0);
        assert_eq!(reconcile_transitions.len(), 1);
        assert_eq!(reconcile_transitions[0].to, ControlPhase::Off);
    }

    #[test]
    fn reconcile_expired_window_stops_and_turns_off() {
        let state = ControlState {
            phase: ControlPhase::Running,
            run_id: Some("run-expired".to_string()),
            started_at: Some(100),
            ends_at: Some(120),
            stop_deadline: None,
            active_run_count: 1,
            active_lane_count: 2,
            last_transition_reason: Some("manual_on".to_string()),
            updated_at: 100,
            ..ControlState::default()
        };

        let (next, transitions) = reconcile_state(121, true, &state);
        assert_eq!(next.phase, ControlPhase::Off);
        assert_eq!(next.last_transition_reason.as_deref(), Some("auto_window_expired"));
        assert_eq!(next.last_run_id.as_deref(), Some("run-expired"));
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].to, ControlPhase::Stopping);
        assert_eq!(transitions[1].to, ControlPhase::Off);
    }

    #[test]
    fn forced_kill_timeout_transitions_to_safe_off_state() {
        let state = ControlState {
            phase: ControlPhase::Stopping,
            run_id: Some("run-stuck".to_string()),
            started_at: Some(100),
            ends_at: Some(120),
            stop_deadline: Some(140),
            active_run_count: 1,
            active_lane_count: 1,
            last_transition_reason: Some("manual_off".to_string()),
            updated_at: 130,
            ..ControlState::default()
        };

        let (next, transitions) = reconcile_state(141, false, &state);
        assert_eq!(next.phase, ControlPhase::Off);
        assert_eq!(next.active_run_count, 0);
        assert_eq!(next.active_lane_count, 0);
        assert_eq!(
            next.last_terminal_failure_reason.as_deref(),
            Some("forced_kill_timeout")
        );
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].reason, "forced_kill_timeout");
    }

    #[test]
    fn start_rejects_queue_full_when_run_is_active() {
        let state = ControlState {
            phase: ControlPhase::Running,
            run_id: Some("run-active".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            stop_deadline: None,
            active_run_count: MAX_CONCURRENT_RUNS,
            active_lane_count: ACTIVE_LANE_COUNT,
            last_transition_reason: Some("manual_on".to_string()),
            updated_at: 100,
            ..ControlState::default()
        };

        let result = start_state(150, 180, &state);
        assert_eq!(result, Err(StartError::QueueFull));
    }
}
