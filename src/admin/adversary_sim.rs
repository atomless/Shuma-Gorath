use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
#[cfg(not(test))]
use base64::{engine::general_purpose, Engine as _};
#[cfg(not(test))]
use hmac::{Hmac, Mac};
#[cfg(not(test))]
use sha2::Sha256;
#[cfg(not(test))]
use spin_sdk::http::{Method, Request};

use crate::challenge::KeyValueStore;
use super::adversary_sim_corpus::{
    deterministic_corpus_metadata_payload, deterministic_runtime_profile,
};
use super::adversary_sim_state::{
    active_lane_count_for_lane, generation_diagnostic_grace_seconds, lane_phase,
};
#[cfg(test)]
use super::adversary_sim_corpus::{
    DETERMINISTIC_ATTACK_CORPUS, DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION,
};
pub use super::adversary_sim_state::{
    autonomous_execution_profile, clamp_duration_seconds, control_surface_available,
    effective_active_lane, lane_reconciliation_needed, load_state, process_instance_id,
    project_effective_desired_state, reconcile_state, save_state, select_desired_lane,
    start_state, stop_state, ControlPhase, ControlState, RuntimeLane, StartError, Transition,
};

pub const MAX_CONCURRENT_RUNS: u32 = 1;
pub const MAX_CPU_MILLICORES: u32 = 1000;
pub const MAX_MEMORY_MIB: u32 = 512;
pub const QUEUE_POLICY: &str = "reject_new";
pub const STOP_TIMEOUT_SECONDS: u64 = 10;
pub const AUTONOMOUS_SHARED_SERVER_HEARTBEAT_INTERVAL_SECONDS: u64 = 1;
pub const AUTONOMOUS_SHARED_SERVER_MAX_CATCHUP_TICKS_PER_INVOCATION: u64 = 2;
pub const AUTONOMOUS_EDGE_FERMYON_HEARTBEAT_INTERVAL_SECONDS: u64 = 60;
pub const AUTONOMOUS_EDGE_FERMYON_MAX_CATCHUP_TICKS_PER_INVOCATION: u64 = 1;
pub const AUTONOMOUS_EDGE_FERMYON_CRON_SCHEDULE: &str =
    "staggered 5x cron set (one run per minute, each job every 5 minutes)";
const PRODUCTION_GENERATION_DEFAULT: &str = "off_until_explicit_enable";
const LANE_DIAGNOSTICS_SCHEMA_VERSION: &str = "adversary-sim-lane-diagnostics.v1";
pub const SCRAPLING_WORKER_PLAN_SCHEMA_VERSION: &str = "adversary-sim-scrapling-worker-plan.v1";
pub const SCRAPLING_WORKER_RESULT_SCHEMA_VERSION: &str = "adversary-sim-scrapling-worker-result.v1";
pub const SCRAPLING_SIM_PROFILE: &str = "scrapling_runtime_lane";
pub const SCRAPLING_MAX_REQUESTS_PER_TICK: u64 = 8;
pub const SCRAPLING_MAX_DEPTH_PER_TICK: u64 = 2;
pub const SCRAPLING_MAX_BYTES_PER_TICK: u64 = 262_144;
pub const SCRAPLING_MAX_MS_PER_TICK: u64 = 2_000;

#[cfg(test)]
pub fn state_key(site_id: &str) -> String {
    super::adversary_sim_state::state_key(site_id)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerFailureClass {
    Cancelled,
    Timeout,
    Transport,
    Http,
}

impl WorkerFailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::Timeout => "timeout",
            Self::Transport => "transport",
            Self::Http => "http",
        }
    }
}

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
    fn lane(&self, lane: RuntimeLane) -> &LaneCounterState {
        match lane {
            RuntimeLane::SyntheticTraffic => &self.synthetic_traffic,
            RuntimeLane::ScraplingTraffic => &self.scrapling_traffic,
            RuntimeLane::BotRedTeam => &self.bot_red_team,
        }
    }

    fn lane_mut(&mut self, lane: RuntimeLane) -> &mut LaneCounterState {
        match lane {
            RuntimeLane::SyntheticTraffic => &mut self.synthetic_traffic,
            RuntimeLane::ScraplingTraffic => &mut self.scrapling_traffic,
            RuntimeLane::BotRedTeam => &mut self.bot_red_team,
        }
    }

    fn failure_class_mut(&mut self, class: WorkerFailureClass) -> &mut FailureClassCounter {
        match class {
            WorkerFailureClass::Cancelled => &mut self.request_failure_classes.cancelled,
            WorkerFailureClass::Timeout => &mut self.request_failure_classes.timeout,
            WorkerFailureClass::Transport => &mut self.request_failure_classes.transport,
            WorkerFailureClass::Http => &mut self.request_failure_classes.http,
        }
    }

    fn to_payload(&self) -> serde_json::Value {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScraplingCrawlStats {
    #[serde(default)]
    pub requests_count: u64,
    #[serde(default)]
    pub offsite_requests_count: u64,
    #[serde(default)]
    pub blocked_requests_count: u64,
    #[serde(default)]
    pub response_status_count: BTreeMap<String, u64>,
    #[serde(default)]
    pub response_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScraplingWorkerPlan {
    pub schema_version: String,
    pub run_id: String,
    pub tick_id: String,
    pub lane: RuntimeLane,
    pub sim_profile: String,
    pub tick_started_at: u64,
    pub max_requests: u64,
    pub max_depth: u64,
    pub max_bytes: u64,
    pub max_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScraplingWorkerResult {
    pub schema_version: String,
    pub run_id: String,
    pub tick_id: String,
    pub lane: RuntimeLane,
    pub worker_id: String,
    pub tick_started_at: u64,
    pub tick_completed_at: u64,
    pub generated_requests: u64,
    pub failed_requests: u64,
    pub last_response_status: Option<u16>,
    #[serde(default)]
    pub failure_class: Option<WorkerFailureClass>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub crawl_stats: ScraplingCrawlStats,
    #[serde(default)]
    pub scope_rejections: BTreeMap<String, u64>,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AutonomousHeartbeatTickSummary {
    pub due_ticks: u64,
    pub executed_ticks: u64,
    pub generated_requests: u64,
    pub failed_requests: u64,
    pub last_response_status: Option<u16>,
    pub worker_pending: bool,
    pub worker_plan: Option<ScraplingWorkerPlan>,
}

fn record_failure_class(state: &mut ControlState, class: WorkerFailureClass, now: u64) {
    let counter = state.lane_diagnostics.failure_class_mut(class);
    counter.count = counter.count.saturating_add(1);
    counter.last_seen_at = Some(now);
}

fn record_lane_attempt(state: &mut ControlState, lane: RuntimeLane) {
    let counters = state.lane_diagnostics.lane_mut(lane);
    counters.beat_attempts = counters.beat_attempts.saturating_add(1);
}

fn record_lane_internal_result(
    state: &mut ControlState,
    lane: RuntimeLane,
    result: &GenerationTickResult,
    now: u64,
) {
    let had_http_failure = result.failed_requests > 0;
    let counters = state.lane_diagnostics.lane_mut(lane);
    if had_http_failure {
        counters.beat_failures = counters.beat_failures.saturating_add(1);
        counters.last_error = Some(format!(
            "request_pipeline_errors={} of {}",
            result.failed_requests, result.generated_requests
        ));
    } else {
        counters.beat_successes = counters.beat_successes.saturating_add(1);
        counters.last_error = None;
    }
    counters.generated_requests = counters
        .generated_requests
        .saturating_add(result.generated_requests);
    if let Some(status) = result.last_response_status {
        let key = format!("status_{status}");
        let entry = counters.response_status_count.entry(key).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    counters.last_generated_at = Some(now);
    let _ = counters;
    if had_http_failure {
        record_failure_class(state, WorkerFailureClass::Http, now);
    }
}

pub fn apply_scrapling_worker_result(
    state: &mut ControlState,
    result: &ScraplingWorkerResult,
) {
    let failure_class = result.failure_class;
    let counters = state.lane_diagnostics.lane_mut(result.lane);
    if failure_class.is_some() || result.failed_requests > 0 || result.error.is_some() {
        counters.beat_failures = counters.beat_failures.saturating_add(1);
        counters.last_error = result.error.clone().or_else(|| {
            Some(format!(
                "scrapling_worker_failed generated_requests={} failed_requests={}",
                result.generated_requests, result.failed_requests
            ))
        });
    } else {
        counters.beat_successes = counters.beat_successes.saturating_add(1);
        counters.last_error = None;
    }
    counters.generated_requests = counters
        .generated_requests
        .saturating_add(result.generated_requests);
    counters.blocked_requests = counters
        .blocked_requests
        .saturating_add(result.crawl_stats.blocked_requests_count);
    counters.offsite_requests = counters
        .offsite_requests
        .saturating_add(result.crawl_stats.offsite_requests_count);
    counters.response_bytes = counters
        .response_bytes
        .saturating_add(result.crawl_stats.response_bytes);
    for (status, count) in &result.crawl_stats.response_status_count {
        let entry = counters.response_status_count.entry(status.clone()).or_insert(0);
        *entry = entry.saturating_add(*count);
    }
    counters.last_generated_at = Some(result.tick_completed_at);
    let last_error = counters.last_error.clone();
    let _ = counters;
    if let Some(class) = failure_class {
        record_failure_class(state, class, result.tick_completed_at);
    }

    state.generated_tick_count = state.generated_tick_count.saturating_add(1);
    state.generated_request_count = state
        .generated_request_count
        .saturating_add(result.generated_requests);
    state.last_generated_at = Some(result.tick_completed_at);
    state.last_generation_error = last_error;
    state.pending_worker_tick_id = None;
    state.pending_worker_started_at = None;
    state.updated_at = result.tick_completed_at;
}

fn reconcile_active_lane_at_beat_boundary(now: u64, state: &mut ControlState) {
    if state.phase != ControlPhase::Running {
        return;
    }
    if effective_active_lane(state) == Some(state.desired_lane) {
        state.active_lane = Some(state.desired_lane);
        state.active_lane_count = active_lane_count_for_lane(state.desired_lane);
        return;
    }
    if state.pending_worker_tick_id.is_some() && state.desired_lane != RuntimeLane::ScraplingTraffic {
        state.pending_worker_tick_id = None;
        state.pending_worker_started_at = None;
    }
    state.active_lane = Some(state.desired_lane);
    state.active_lane_count = active_lane_count_for_lane(state.desired_lane);
    state.lane_switch_seq = state.lane_switch_seq.saturating_add(1);
    state.last_lane_switch_at = Some(now);
    state.last_lane_switch_reason = Some("beat_boundary_reconciliation".to_string());
    state.updated_at = now;
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
        }
    })
}

fn simulated_request_paths(run_id: &str, tick_count: u64) -> [String; 9] {
    let runtime_profile = deterministic_runtime_profile();
    let run_suffix = run_id
        .chars()
        .rev()
        .take(8)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let public_paths = runtime_profile.primary_public_paths.as_slice();
    let pick = |slot: u64| -> String {
        let index = (deterministic_lane_entropy(run_id, tick_count, slot) % public_paths.len() as u64) as usize;
        public_paths[index].to_string()
    };
    let mut paths = vec![
        pick(0),
        pick(1),
        pick(2),
        pick(3),
        format!(
            "{}?q=run-{}-tick-{}-probe-{}",
            runtime_profile.paths.public_search,
            run_suffix,
            tick_count,
            deterministic_lane_entropy(run_id, tick_count, 8) % 10_000
        ),
        runtime_profile.paths.pow.clone(),
        runtime_profile.paths.not_a_bot_checkbox.clone(),
        crate::maze::entry_path(format!("sim-probe-{}-{}", run_suffix, tick_count).as_str()),
        if should_emit_honeypot_probe(tick_count) {
            runtime_profile.paths.honeypot.clone()
        } else {
            format!(
                "{}?q=deep-crawl-{}-{}",
                runtime_profile.paths.public_search,
                run_suffix,
                deterministic_lane_entropy(run_id, tick_count, 9) % 10_000
            )
        },
    ];
    let rotation = (deterministic_lane_entropy(run_id, tick_count, 10) % paths.len() as u64) as usize;
    paths.rotate_left(rotation);
    paths
        .try_into()
        .unwrap_or_else(|_| unreachable!("primary request paths are fixed-size"))
}

fn deterministic_lane_entropy(run_id: &str, tick_count: u64, slot: u64) -> u64 {
    let mut hash = 0xcbf29ce484222325u64 ^ tick_count ^ slot.rotate_left(17);
    for byte in run_id.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash ^ tick_count.rotate_left((slot % 31) as u32)
}

fn should_emit_honeypot_probe(tick_count: u64) -> bool {
    deterministic_runtime_profile()
        .honeypot_probe_moduli
        .iter()
        .filter(|modulus| **modulus > 0)
        .any(|modulus| tick_count % *modulus == 0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SupplementalLane {
    ChallengeSubmit,
    NotABotFail,
    NotABotEscalate,
    PowVerify,
    TarpitProgress,
    FingerprintProbe,
    CdpReport,
}

const FULL_SUPPLEMENTAL_LANES: [SupplementalLane; 7] = [
    SupplementalLane::ChallengeSubmit,
    SupplementalLane::NotABotFail,
    SupplementalLane::NotABotEscalate,
    SupplementalLane::PowVerify,
    SupplementalLane::TarpitProgress,
    SupplementalLane::FingerprintProbe,
    SupplementalLane::CdpReport,
];
// Fermyon Wasm Functions cap request handlers at 30s, so edge beats need a smaller
// per-invocation envelope than the shared-server runtime toggle uses.
const EDGE_FERMYON_PRIMARY_REQUESTS_PER_TICK: usize = 2;
const EDGE_FERMYON_SUPPLEMENTAL_LANES_PER_TICK: usize = 1;
const EDGE_FERMYON_RATE_BURST_LOW: u64 = 1;
const EDGE_FERMYON_RATE_BURST_MEDIUM: u64 = 2;
const EDGE_FERMYON_RATE_BURST_HIGH: u64 = 3;

fn primary_request_budget_for_profile(profile: crate::config::GatewayDeploymentProfile) -> usize {
    match profile {
        crate::config::GatewayDeploymentProfile::SharedServer => {
            deterministic_runtime_profile().primary_request_count as usize
        }
        crate::config::GatewayDeploymentProfile::EdgeFermyon => {
            EDGE_FERMYON_PRIMARY_REQUESTS_PER_TICK
        }
    }
}

fn supplemental_lanes_for_profile(
    profile: crate::config::GatewayDeploymentProfile,
    tick_count: u64,
) -> Vec<SupplementalLane> {
    match profile {
        crate::config::GatewayDeploymentProfile::SharedServer => FULL_SUPPLEMENTAL_LANES.to_vec(),
        crate::config::GatewayDeploymentProfile::EdgeFermyon => {
            let lane_count = EDGE_FERMYON_SUPPLEMENTAL_LANES_PER_TICK.min(FULL_SUPPLEMENTAL_LANES.len());
            let start = ((tick_count as usize) * lane_count) % FULL_SUPPLEMENTAL_LANES.len();
            (0..lane_count)
                .map(|offset| FULL_SUPPLEMENTAL_LANES[(start + offset) % FULL_SUPPLEMENTAL_LANES.len()])
                .collect()
        }
    }
}

fn rate_burst_requests_for_profile(
    profile: crate::config::GatewayDeploymentProfile,
    tick_count: u64,
) -> u64 {
    let burst = &deterministic_runtime_profile().rate_burst;
    if burst.high_modulus > 0 && tick_count % burst.high_modulus == 0 {
        match profile {
            crate::config::GatewayDeploymentProfile::SharedServer => burst.high,
            crate::config::GatewayDeploymentProfile::EdgeFermyon => EDGE_FERMYON_RATE_BURST_HIGH,
        }
    } else if burst.medium_modulus > 0 && tick_count % burst.medium_modulus == 0 {
        match profile {
            crate::config::GatewayDeploymentProfile::SharedServer => burst.medium,
            crate::config::GatewayDeploymentProfile::EdgeFermyon => EDGE_FERMYON_RATE_BURST_MEDIUM,
        }
    } else {
        match profile {
            crate::config::GatewayDeploymentProfile::SharedServer => burst.low,
            crate::config::GatewayDeploymentProfile::EdgeFermyon => EDGE_FERMYON_RATE_BURST_LOW,
        }
    }
}

#[cfg(not(test))]
fn rate_burst_requests_for_tick(tick_count: u64) -> u64 {
    rate_burst_requests_for_profile(crate::config::gateway_deployment_profile(), tick_count)
}

#[cfg(test)]
fn deterministic_generated_request_target_for_profile(
    profile: crate::config::GatewayDeploymentProfile,
    tick_count: u64,
) -> u64 {
    primary_request_budget_for_profile(profile) as u64
        + supplemental_lanes_for_profile(profile, tick_count).len() as u64
        + rate_burst_requests_for_profile(profile, tick_count)
}

#[cfg(not(test))]
fn simulated_request_ip(tick_count: u64, index: usize) -> String {
    let runtime_profile = deterministic_runtime_profile();
    let generation_batch_size_max = runtime_profile
        .primary_request_count
        .saturating_add(runtime_profile.supplemental_request_count)
        .saturating_add(runtime_profile.rate_burst.high);
    let offset = tick_count
        .saturating_mul(generation_batch_size_max)
        .saturating_add(index as u64);
    let third = ((offset / 254) % 254) + 1;
    let fourth = (offset % 254) + 1;
    format!("198.51.{}.{}", third, fourth)
}

#[cfg(not(test))]
fn lane_actor_ip(third_octet: u8, tick_count: u64, rotate_every_ticks: u64, lane_salt: u64) -> String {
    let rotate_every_ticks = rotate_every_ticks.max(1);
    let bucket = ((tick_count / rotate_every_ticks).wrapping_add(lane_salt) % 254) + 1;
    format!("198.51.{}.{}", third_octet, bucket)
}

#[cfg(not(test))]
fn challenge_signing_secret() -> Option<String> {
    crate::config::runtime_var_trimmed_optional("SHUMA_CHALLENGE_SECRET")
        .or_else(|| crate::config::runtime_var_trimmed_optional("SHUMA_JS_SECRET"))
}

#[cfg(not(test))]
fn build_signed_not_a_bot_seed_token(
    now: u64,
    ip: &str,
    user_agent: &str,
    return_to: &str,
    entropy: u64,
    latency_seconds: u64,
) -> Option<String> {
    let signing_secret = challenge_signing_secret()?;
    let issued_at = now.saturating_sub(latency_seconds.min(30));
    let expires_at = now.saturating_add(120);
    let payload_json = json!({
        "operation_id": format!("{:016x}{:016x}", entropy, entropy.rotate_left(29)),
        "flow_id": crate::challenge::operation_envelope::FLOW_NOT_A_BOT,
        "step_id": crate::challenge::operation_envelope::STEP_NOT_A_BOT_SUBMIT,
        "step_index": crate::challenge::operation_envelope::STEP_INDEX_NOT_A_BOT_SUBMIT,
        "issued_at": issued_at,
        "expires_at": expires_at,
        "token_version": crate::challenge::operation_envelope::TOKEN_VERSION_V1,
        "ip_bucket": crate::signals::ip_identity::bucket_ip(ip),
        "ua_bucket": crate::challenge::operation_envelope::user_agent_bucket(user_agent),
        "path_class": crate::challenge::operation_envelope::PATH_CLASS_NOT_A_BOT_SUBMIT,
        "return_to": return_to
    })
    .to_string();
    let mut mac = Hmac::<Sha256>::new_from_slice(signing_secret.as_bytes()).ok()?;
    mac.update(payload_json.as_bytes());
    let signature = mac.finalize().into_bytes();
    Some(format!(
        "{}.{}",
        general_purpose::STANDARD.encode(payload_json.as_bytes()),
        general_purpose::STANDARD.encode(signature)
    ))
}

#[cfg(not(test))]
#[derive(Clone, Copy)]
enum NotABotSubmissionProfile {
    Fail,
    EscalatePuzzle,
}

#[cfg(not(test))]
fn build_not_a_bot_submit_body(seed_token: &str, profile: NotABotSubmissionProfile) -> Vec<u8> {
    let telemetry = match profile {
        NotABotSubmissionProfile::Fail => json!({
            "has_pointer": false,
            "pointer_move_count": 0,
            "pointer_path_length": 0.0,
            "pointer_direction_changes": 0,
            "down_up_ms": 50,
            "focus_changes": 5,
            "visibility_changes": 2,
            "interaction_elapsed_ms": 600,
            "keyboard_used": true,
            "touch_used": false,
            "activation_method": "unknown",
            "activation_trusted": false,
            "activation_count": 1,
            "control_focused": false
        }),
        NotABotSubmissionProfile::EscalatePuzzle => json!({
            "has_pointer": false,
            "pointer_move_count": 0,
            "pointer_path_length": 0.0,
            "pointer_direction_changes": 0,
            "down_up_ms": 90,
            "focus_changes": 5,
            "visibility_changes": 2,
            "interaction_elapsed_ms": 900,
            "keyboard_used": false,
            "touch_used": false,
            "activation_method": "unknown",
            "activation_trusted": false,
            "activation_count": 1,
            "control_focused": false
        }),
    };
    format!("seed={seed_token}&checked=1&telemetry={telemetry}").into_bytes()
}

#[cfg(test)]
fn deterministic_generated_request_target_for_tick(tick_count: u64) -> u64 {
    deterministic_generated_request_target_for_profile(
        crate::config::gateway_deployment_profile(),
        tick_count,
    )
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

fn autonomous_heartbeat_due_ticks(now: u64, state: &ControlState) -> u64 {
    if state.phase != ControlPhase::Running {
        return 0;
    }
    let profile = autonomous_execution_profile();
    let due = match state.last_generated_at {
        None => 1,
        Some(last_generated_at) => {
            let elapsed_seconds = now.saturating_sub(last_generated_at);
            if elapsed_seconds < profile.cadence_seconds {
                0
            } else {
                elapsed_seconds / profile.cadence_seconds
            }
        }
    };
    due.min(profile.max_catchup_ticks_per_invocation)
}

fn next_scrapling_worker_plan(now: u64, state: &mut ControlState) -> Option<ScraplingWorkerPlan> {
    let run_id = state
        .run_id
        .clone()
        .or_else(|| state.last_run_id.clone())
        .unwrap_or_else(|| format!("simrun-runtime-{now}"));
    let tick_id = format!("scrapling-tick-{}-{:016x}", now, random::<u64>());
    state.pending_worker_tick_id = Some(tick_id.clone());
    state.pending_worker_started_at = Some(now);
    state.updated_at = now;
    Some(ScraplingWorkerPlan {
        schema_version: SCRAPLING_WORKER_PLAN_SCHEMA_VERSION.to_string(),
        run_id,
        tick_id,
        lane: RuntimeLane::ScraplingTraffic,
        sim_profile: SCRAPLING_SIM_PROFILE.to_string(),
        tick_started_at: now,
        max_requests: SCRAPLING_MAX_REQUESTS_PER_TICK,
        max_depth: SCRAPLING_MAX_DEPTH_PER_TICK,
        max_bytes: SCRAPLING_MAX_BYTES_PER_TICK,
        max_ms: SCRAPLING_MAX_MS_PER_TICK,
    })
}

pub fn run_autonomous_supervisor_ticks(
    store: &impl KeyValueStore,
    state: &mut ControlState,
    now: u64,
) -> AutonomousHeartbeatTickSummary {
    let due_ticks = autonomous_heartbeat_due_ticks(now, state);
    let mut summary = AutonomousHeartbeatTickSummary {
        due_ticks,
        ..AutonomousHeartbeatTickSummary::default()
    };
    if due_ticks == 0 {
        return summary;
    }
    reconcile_active_lane_at_beat_boundary(now, state);
    match effective_active_lane(state) {
        Some(RuntimeLane::SyntheticTraffic) => {}
        Some(RuntimeLane::ScraplingTraffic) => {
            if state.pending_worker_tick_id.is_some() {
                summary.worker_pending = true;
                return summary;
            }
            record_lane_attempt(state, RuntimeLane::ScraplingTraffic);
            summary.worker_plan = next_scrapling_worker_plan(now, state);
            return summary;
        }
        Some(RuntimeLane::BotRedTeam) => {
            record_lane_attempt(state, RuntimeLane::BotRedTeam);
            let counters = state.lane_diagnostics.lane_mut(RuntimeLane::BotRedTeam);
            counters.beat_failures = counters.beat_failures.saturating_add(1);
            counters.last_error = Some("bot_red_team_unimplemented".to_string());
            state.last_generation_error = counters.last_error.clone();
            state.updated_at = now;
            return summary;
        }
        None => return summary,
    }
    for tick_index in 0..due_ticks {
        let tick_now = now.saturating_sub(due_ticks.saturating_sub(tick_index).saturating_sub(1));
        record_lane_attempt(state, RuntimeLane::SyntheticTraffic);
        let tick_result = run_internal_generation_tick(store, state, tick_now);
        record_lane_internal_result(state, RuntimeLane::SyntheticTraffic, &tick_result, tick_now);
        summary.executed_ticks = summary.executed_ticks.saturating_add(1);
        summary.generated_requests = summary
            .generated_requests
            .saturating_add(tick_result.generated_requests);
        summary.failed_requests = summary
            .failed_requests
            .saturating_add(tick_result.failed_requests);
        summary.last_response_status = tick_result.last_response_status;
    }
    summary
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
    let runtime_profile = deterministic_runtime_profile();
    let metadata = crate::runtime::sim_telemetry::SimulationRequestMetadata {
        sim_run_id: run_id.clone(),
        sim_profile: runtime_profile.metadata.sim_profile.clone(),
        sim_lane: runtime_profile.metadata.sim_lane.clone(),
    };
    #[cfg(not(test))]
    {
        let deployment_profile = crate::config::gateway_deployment_profile();
        let forwarded_secret = crate::config::runtime_var_trimmed_optional("SHUMA_FORWARDED_IP_SECRET");
        let selected_supplemental_lanes =
            supplemental_lanes_for_profile(deployment_profile, state.generated_tick_count);
        let includes_lane = |lane: SupplementalLane| selected_supplemental_lanes.contains(&lane);

        let mut dispatch_request = |request: Request| {
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
        for (index, path) in paths
            .iter()
            .take(primary_request_budget_for_profile(deployment_profile))
            .enumerate()
        {
            let user_agent = format!("ShumaAdversarySim/1.0 slot={} path={}", index, path);
            let mut builder = Request::builder();
            let simulated_ip = simulated_request_ip(state.generated_tick_count, index);
            builder
                .method(Method::Get)
                .uri(path.as_str())
                .header("x-forwarded-for", simulated_ip.as_str())
                .header("x-forwarded-proto", "https")
                .header("user-agent", user_agent.as_str());
            if let Some(secret) = forwarded_secret.as_deref() {
                builder.header("x-shuma-forwarded-secret", secret);
            }
            // GEO probes should target normal public-surface paths so they traverse
            // the same policy path as real traffic and are not skipped by special endpoints.
            if path.starts_with("/sim/public/") {
                builder.header("x-geo-country", "RU");
            }
            if (state.generated_tick_count + index as u64) % 4 == 0 {
                builder
                    .header(
                        "user-agent",
                        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148",
                    )
                    .header(
                        "sec-ch-ua",
                        "\"Chromium\";v=\"120\", \"Not_A Brand\";v=\"99\"",
                    )
                    .header("sec-ch-ua-platform", "\"Windows\"")
                    .header("sec-ch-ua-mobile", "?0")
                    .header(
                        "x-shuma-edge-ja3",
                        format!(
                            "sim-ja3-{}-{}",
                            state.generated_tick_count,
                            index
                        )
                        .as_str(),
                    );
            }
            dispatch_request(builder.body(Vec::new()).build());
        }

        let challenge_abuse_ip = lane_actor_ip(
            runtime_profile.lane_ip_octets.challenge_abuse,
            state.generated_tick_count,
            runtime_profile.lane_ip_rotation_ticks.challenge_abuse,
            runtime_profile.lane_ip_entropy_salts.challenge_abuse,
        );
        let pow_abuse_ip = lane_actor_ip(
            runtime_profile.lane_ip_octets.pow_abuse,
            state.generated_tick_count,
            runtime_profile.lane_ip_rotation_ticks.pow_abuse,
            runtime_profile.lane_ip_entropy_salts.pow_abuse,
        );
        let tarpit_abuse_ip = lane_actor_ip(
            runtime_profile.lane_ip_octets.tarpit_abuse,
            state.generated_tick_count,
            runtime_profile.lane_ip_rotation_ticks.tarpit_abuse,
            runtime_profile.lane_ip_entropy_salts.tarpit_abuse,
        );
        let fingerprint_probe_ip = lane_actor_ip(
            runtime_profile.lane_ip_octets.fingerprint_probe,
            state.generated_tick_count,
            runtime_profile.lane_ip_rotation_ticks.fingerprint_probe,
            runtime_profile.lane_ip_entropy_salts.fingerprint_probe,
        );
        let cdp_report_ip = lane_actor_ip(
            runtime_profile.lane_ip_octets.cdp_report,
            state.generated_tick_count,
            runtime_profile.lane_ip_rotation_ticks.cdp_report,
            runtime_profile.lane_ip_entropy_salts.cdp_report,
        );
        let rate_burst_ip = lane_actor_ip(
            runtime_profile.lane_ip_octets.rate_burst,
            state.generated_tick_count,
            runtime_profile.lane_ip_rotation_ticks.rate_burst,
            runtime_profile.lane_ip_entropy_salts.rate_burst,
        );
        let not_a_bot_fail_ip = lane_actor_ip(
            runtime_profile.lane_ip_octets.not_a_bot_fail,
            state.generated_tick_count,
            runtime_profile.lane_ip_rotation_ticks.not_a_bot_fail,
            runtime_profile.lane_ip_entropy_salts.not_a_bot_fail,
        );
        let not_a_bot_escalate_ip = lane_actor_ip(
            runtime_profile.lane_ip_octets.not_a_bot_escalate,
            state.generated_tick_count,
            runtime_profile.lane_ip_rotation_ticks.not_a_bot_escalate,
            runtime_profile.lane_ip_entropy_salts.not_a_bot_escalate,
        );

        if includes_lane(SupplementalLane::ChallengeSubmit) {
            let challenge_abuse_body =
                b"answer=bad&seed=invalid&return_to=%2Fsim%2Fpublic%2Flanding".to_vec();
            let mut challenge_submit = Request::builder();
            challenge_submit
                .method(Method::Post)
                .uri(runtime_profile.paths.challenge_submit.as_str())
                .header("x-forwarded-for", challenge_abuse_ip.as_str())
                .header("x-forwarded-proto", "https")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("user-agent", "ShumaAdversarySim/1.0 challenge-submit");
            if let Some(secret) = forwarded_secret.as_deref() {
                challenge_submit.header("x-shuma-forwarded-secret", secret);
            }
            dispatch_request(challenge_submit.body(challenge_abuse_body).build());
        }

        if includes_lane(SupplementalLane::NotABotFail) {
            if let Some(fail_seed) = build_signed_not_a_bot_seed_token(
                now,
                not_a_bot_fail_ip.as_str(),
                "ShumaAdversarySim/1.0 not-a-bot-fail",
                "/sim/public/docs",
                deterministic_lane_entropy(run_id.as_str(), state.generated_tick_count, 101),
                1 + (state.generated_tick_count % 5),
            ) {
                let fail_body = build_not_a_bot_submit_body(&fail_seed, NotABotSubmissionProfile::Fail);
                let mut not_a_bot_fail_submit = Request::builder();
                not_a_bot_fail_submit
                    .method(Method::Post)
                    .uri(runtime_profile.paths.not_a_bot_checkbox.as_str())
                    .header("x-forwarded-for", not_a_bot_fail_ip.as_str())
                    .header("x-forwarded-proto", "https")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("user-agent", "ShumaAdversarySim/1.0 not-a-bot-fail");
                if let Some(secret) = forwarded_secret.as_deref() {
                    not_a_bot_fail_submit.header("x-shuma-forwarded-secret", secret);
                }
                dispatch_request(not_a_bot_fail_submit.body(fail_body).build());
            }
        }

        if includes_lane(SupplementalLane::NotABotEscalate) {
            if let Some(escalate_seed) = build_signed_not_a_bot_seed_token(
                now,
                not_a_bot_escalate_ip.as_str(),
                "ShumaAdversarySim/1.0 not-a-bot-escalate",
                "/sim/public/pricing",
                deterministic_lane_entropy(run_id.as_str(), state.generated_tick_count, 102),
                2 + (state.generated_tick_count.wrapping_mul(3) % 7),
            ) {
                let escalate_body =
                    build_not_a_bot_submit_body(&escalate_seed, NotABotSubmissionProfile::EscalatePuzzle);
                let mut not_a_bot_escalate_submit = Request::builder();
                not_a_bot_escalate_submit
                    .method(Method::Post)
                    .uri(runtime_profile.paths.not_a_bot_checkbox.as_str())
                    .header("x-forwarded-for", not_a_bot_escalate_ip.as_str())
                    .header("x-forwarded-proto", "https")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("user-agent", "ShumaAdversarySim/1.0 not-a-bot-escalate");
                if let Some(secret) = forwarded_secret.as_deref() {
                    not_a_bot_escalate_submit.header("x-shuma-forwarded-secret", secret);
                }
                dispatch_request(not_a_bot_escalate_submit.body(escalate_body).build());
            }
        }

        if includes_lane(SupplementalLane::PowVerify) {
            let pow_verify_body = br#"{"seed":"invalid-seed","nonce":"invalid-nonce"}"#.to_vec();
            let mut pow_verify = Request::builder();
            pow_verify
                .method(Method::Post)
                .uri(runtime_profile.paths.pow_verify.as_str())
                .header("x-forwarded-for", pow_abuse_ip.as_str())
                .header("x-forwarded-proto", "https")
                .header("content-type", "application/json")
                .header("user-agent", "ShumaAdversarySim/1.0 pow-verify-submit");
            if let Some(secret) = forwarded_secret.as_deref() {
                pow_verify.header("x-shuma-forwarded-secret", secret);
            }
            dispatch_request(pow_verify.body(pow_verify_body).build());
        }

        if includes_lane(SupplementalLane::TarpitProgress) {
            let tarpit_progress_body =
                br#"{"token":"invalid","operation_id":"invalid","proof_nonce":"invalid"}"#.to_vec();
            let mut tarpit_progress = Request::builder();
            tarpit_progress
                .method(Method::Post)
                .uri(crate::tarpit::progress_path())
                .header("x-forwarded-for", tarpit_abuse_ip.as_str())
                .header("x-forwarded-proto", "https")
                .header("content-type", "application/json")
                .header("user-agent", "ShumaAdversarySim/1.0 tarpit-progress-submit");
            if let Some(secret) = forwarded_secret.as_deref() {
                tarpit_progress.header("x-shuma-forwarded-secret", secret);
            }
            dispatch_request(tarpit_progress.body(tarpit_progress_body).build());
        }

        if includes_lane(SupplementalLane::FingerprintProbe) {
            let fingerprint_probe_path =
                format!("{}?q=fingerprint-mismatch", runtime_profile.paths.public_search);
            let mut fingerprint_probe = Request::builder();
            fingerprint_probe
                .method(Method::Get)
                .uri(fingerprint_probe_path.as_str())
                .header("x-forwarded-for", fingerprint_probe_ip.as_str())
                .header("x-forwarded-proto", "https")
                .header(
                    "user-agent",
                    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148",
                )
                .header(
                    "sec-ch-ua",
                    "\"Chromium\";v=\"120\", \"Not_A Brand\";v=\"99\"",
                )
                .header("sec-ch-ua-platform", "\"Windows\"")
                .header("sec-ch-ua-mobile", "?0");
            if let Some(secret) = forwarded_secret.as_deref() {
                fingerprint_probe.header("x-shuma-forwarded-secret", secret);
            }
            dispatch_request(fingerprint_probe.body(Vec::new()).build());
        }

        if includes_lane(SupplementalLane::CdpReport) {
            let cdp_probe_body = serde_json::to_vec(&json!({
                "cdp_detected": true,
                "score": 4.8,
                "checks": ["webdriver", "automation_props", "cdp_timing", "micro_timing"]
            }))
            .unwrap_or_else(|_| b"{\"cdp_detected\":true,\"score\":4.8,\"checks\":[\"webdriver\"]}".to_vec());
            let mut cdp_builder = Request::builder();
            cdp_builder
                .method(Method::Post)
                .uri(runtime_profile.paths.cdp_report.as_str())
                .header("x-forwarded-for", cdp_report_ip.as_str())
                .header("x-forwarded-proto", "https")
                .header("content-type", "application/json")
                .header("user-agent", "ShumaAdversarySim/1.0 cdp-probe");
            if let Some(secret) = forwarded_secret.as_deref() {
                cdp_builder.header("x-shuma-forwarded-secret", secret);
            }
            dispatch_request(cdp_builder.body(cdp_probe_body).build());
        }

        let rate_burst_requests = rate_burst_requests_for_tick(state.generated_tick_count);
        for burst_index in 0..rate_burst_requests {
            let mut burst_builder = Request::builder();
            let burst_path = format!(
                "{}?q=rate-burst-{}-{}-{}",
                runtime_profile.paths.public_search,
                state.generated_tick_count,
                burst_index,
                deterministic_lane_entropy(run_id.as_str(), state.generated_tick_count, 120 + burst_index)
                    % 10_000
            );
            let user_agent = format!("ShumaAdversarySim/1.0 rate-burst {}", burst_index);
            burst_builder
                .method(Method::Get)
                .uri(burst_path.as_str())
                .header("x-forwarded-for", rate_burst_ip.as_str())
                .header("x-forwarded-proto", "https")
                .header("user-agent", user_agent.as_str());
            if let Some(secret) = forwarded_secret.as_deref() {
                burst_builder.header("x-shuma-forwarded-secret", secret);
            }
            if burst_index % 8 == 0 {
                burst_builder
                    .header("sec-ch-ua", "\"Not_A Brand\";v=\"99\", \"Chromium\";v=\"120\"")
                    .header("sec-ch-ua-platform", "\"Windows\"")
                    .header("sec-ch-ua-mobile", "?0")
                    .header(
                        "x-shuma-edge-browser-family",
                        "chrome",
                    );
            }
            dispatch_request(burst_builder.body(Vec::new()).build());
        }
        crate::observability::monitoring::flush_pending_counters(store);
    }
    #[cfg(test)]
    {
        let _ = store;
        let _ = metadata;
        result.generated_requests = deterministic_generated_request_target_for_tick(state.generated_tick_count);
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
    use crate::admin::adversary_sim_state::effective_desired_enabled;
    use crate::test_support::InMemoryStore;

    #[test]
    fn deterministic_attack_corpus_is_loaded_with_required_metadata() {
        let runtime = deterministic_runtime_profile();
        assert_eq!(
            DETERMINISTIC_ATTACK_CORPUS.schema_version,
            DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION
        );
        assert!(!DETERMINISTIC_ATTACK_CORPUS.corpus_revision.trim().is_empty());
        assert!(!DETERMINISTIC_ATTACK_CORPUS.taxonomy_version.trim().is_empty());
        assert!(runtime.active_lane_count >= 1);
        assert!(!runtime.primary_public_paths.is_empty());
        assert!(runtime.rate_burst.low > 0);
        assert!(!DETERMINISTIC_ATTACK_CORPUS.ci_oracle.drivers.is_empty());
    }

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
    fn start_and_stop_transitions_track_additive_lane_contract() {
        let now = 1_000u64;
        let (started, _) = start_state(now, 180, &ControlState::default()).expect("start");
        assert_eq!(started.desired_lane.as_str(), "synthetic_traffic");
        assert_eq!(
            started.active_lane.map(RuntimeLane::as_str),
            Some("synthetic_traffic")
        );
        assert_eq!(started.lane_switch_seq, 0);
        assert_eq!(started.last_lane_switch_at, None);
        assert_eq!(started.last_lane_switch_reason, None);

        let (stopping, _) = stop_state(now + 1, "manual_off", &started);
        assert_eq!(stopping.desired_lane.as_str(), "synthetic_traffic");
        assert_eq!(stopping.active_lane, None);
        assert_eq!(stopping.lane_switch_seq, 0);
        assert_eq!(stopping.last_lane_switch_at, None);
        assert_eq!(stopping.last_lane_switch_reason, None);
    }

    #[test]
    fn reconcile_expired_window_stops_and_turns_off() {
        let state = ControlState {
            phase: ControlPhase::Running,
            owner_instance_id: Some(process_instance_id().to_string()),
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
            owner_instance_id: Some(process_instance_id().to_string()),
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
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-active".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            stop_deadline: None,
            active_run_count: MAX_CONCURRENT_RUNS,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            last_transition_reason: Some("manual_on".to_string()),
            updated_at: 100,
            ..ControlState::default()
        };

        let result = start_state(150, 180, &state);
        assert_eq!(result, Err(StartError::QueueFull));
    }

    #[test]
    fn autonomous_supervisor_runs_initial_tick_when_running_without_history() {
        let store = InMemoryStore::default();
        let mut state = ControlState {
            phase: ControlPhase::Running,
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-supervisor".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            ..ControlState::default()
        };
        let summary = run_autonomous_supervisor_ticks(&store, &mut state, 110);
        assert_eq!(summary.due_ticks, 1);
        assert_eq!(summary.executed_ticks, 1);
        assert_eq!(state.generated_tick_count, 1);
        assert_eq!(
            state.generated_request_count,
            deterministic_generated_request_target_for_tick(0)
        );
        assert_eq!(state.last_generated_at, Some(110));
    }

    #[test]
    fn autonomous_supervisor_caps_catchup_ticks_per_invocation() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");
        let store = InMemoryStore::default();
        let mut state = ControlState {
            phase: ControlPhase::Running,
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-catchup".to_string()),
            started_at: Some(10),
            ends_at: Some(1000),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            last_generated_at: Some(10),
            ..ControlState::default()
        };
        let summary = run_autonomous_supervisor_ticks(&store, &mut state, 200);
        assert_eq!(
            summary.executed_ticks,
            AUTONOMOUS_SHARED_SERVER_MAX_CATCHUP_TICKS_PER_INVOCATION
        );
        assert_eq!(
            state.generated_tick_count,
            AUTONOMOUS_SHARED_SERVER_MAX_CATCHUP_TICKS_PER_INVOCATION
        );
        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn supervisor_status_payload_reports_off_state_inert_contract() {
        let state = ControlState::default();
        let payload = supervisor_status_payload(100, false, &state);
        assert_eq!(
            payload
                .get("heartbeat_active")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            payload
                .get("off_state_inert")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            payload
                .get("deterministic_attack_corpus")
                .and_then(|value| value.get("schema_version"))
                .and_then(|value| value.as_str()),
            Some(DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION)
        );
    }

    #[test]
    fn supervisor_status_payload_reports_edge_cron_truthfully_before_first_tick() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
        let state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            run_id: Some("run-edge".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            ..ControlState::default()
        };

        let payload = supervisor_status_payload(130, true, &state);
        assert_eq!(
            payload
                .get("heartbeat_expected")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            payload
                .get("heartbeat_active")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            payload
                .get("trigger_surface")
                .and_then(|value| value.as_str()),
            Some("edge_cron")
        );
        assert_eq!(
            payload
                .get("cadence_seconds")
                .and_then(|value| value.as_u64()),
            Some(AUTONOMOUS_EDGE_FERMYON_HEARTBEAT_INTERVAL_SECONDS)
        );
        assert_eq!(
            payload
                .get("cron_schedule")
                .and_then(|value| value.as_str()),
            Some(AUTONOMOUS_EDGE_FERMYON_CRON_SCHEDULE)
        );

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn generation_diagnostics_waits_full_edge_interval_before_no_traffic() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
        let state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            run_id: Some("run-edge".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            ..ControlState::default()
        };

        let warming = generation_diagnostics(130, true, &state);
        assert_eq!(warming.health, "warming");
        assert_eq!(warming.reason, "waiting_for_first_edge_cron_tick");

        let no_traffic = generation_diagnostics(170, true, &state);
        assert_eq!(no_traffic.health, "no_traffic");
        assert_eq!(no_traffic.reason, "edge_cron_no_traffic_yet");

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn status_payload_surfaces_explicit_production_operating_envelope() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");

        let payload = status_payload(
            100,
            crate::config::RuntimeEnvironment::RuntimeProd,
            true,
            false,
            180,
            &ControlState::default(),
        );
        assert_eq!(
            payload
                .get("gateway_deployment_profile")
                .and_then(|value| value.as_str()),
            Some("edge-fermyon")
        );
        assert_eq!(
            payload
                .get("guardrails")
                .and_then(|value| value.get("surface_available_by_default"))
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            payload
                .get("guardrails")
                .and_then(|value| value.get("generation_default"))
                .and_then(|value| value.as_str()),
            Some(PRODUCTION_GENERATION_DEFAULT)
        );
        assert_eq!(
            payload
                .get("guardrails")
                .and_then(|value| value.get("generation_requires_explicit_enable"))
                .and_then(|value| value.as_bool()),
            Some(true)
        );

        let supervisor = supervisor_status_payload(100, false, &ControlState::default());
        assert_eq!(
            supervisor
                .get("deployment_profile")
                .and_then(|value| value.as_str()),
            Some("edge-fermyon")
        );
        assert_eq!(
            supervisor
                .get("trigger_surface")
                .and_then(|value| value.as_str()),
            Some("edge_cron")
        );
        assert_eq!(
            supervisor
                .get("cadence_seconds")
                .and_then(|value| value.as_u64()),
            Some(AUTONOMOUS_EDGE_FERMYON_HEARTBEAT_INTERVAL_SECONDS)
        );
        assert_eq!(
            supervisor
                .get("cron_schedule")
                .and_then(|value| value.as_str()),
            Some(AUTONOMOUS_EDGE_FERMYON_CRON_SCHEDULE)
        );

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn status_payload_exposes_additive_lane_migration_contract() {
        let state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            run_id: Some("run-lane-contract".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            ..ControlState::default()
        };

        let payload = status_payload(
            150,
            crate::config::RuntimeEnvironment::RuntimeProd,
            true,
            true,
            180,
            &state,
        );
        assert_eq!(
            payload
                .get("desired_lane")
                .and_then(|value| value.as_str()),
            Some("synthetic_traffic")
        );
        assert_eq!(
            payload
                .get("active_lane")
                .and_then(|value| value.as_str()),
            Some("synthetic_traffic")
        );
        assert_eq!(
            payload
                .get("lane_switch_seq")
                .and_then(|value| value.as_u64()),
            Some(0)
        );
        assert_eq!(payload.get("last_lane_switch_at"), Some(&serde_json::Value::Null));
        assert_eq!(
            payload.get("last_lane_switch_reason"),
            Some(&serde_json::Value::Null)
        );
        assert_eq!(
            payload
                .get("lane_diagnostics")
                .and_then(|value| value.get("lanes"))
                .and_then(|value| value.get("scrapling_traffic"))
                .and_then(|value| value.get("beat_attempts"))
                .and_then(|value| value.as_u64()),
            Some(0)
        );
        assert_eq!(
            payload
                .get("lane_diagnostics")
                .and_then(|value| value.get("request_failure_classes"))
                .and_then(|value| value.get("cancelled"))
                .and_then(|value| value.get("count"))
                .and_then(|value| value.as_u64()),
            Some(0)
        );
    }

    #[test]
    fn effective_desired_enabled_uses_seed_before_first_control_write() {
        assert!(effective_desired_enabled(true, &ControlState::default()));
        assert!(!effective_desired_enabled(false, &ControlState::default()));
    }

    #[test]
    fn effective_desired_enabled_prefers_persisted_lifecycle_state_after_control_write() {
        let state = ControlState {
            desired_enabled: false,
            updated_at: 100,
            ..ControlState::default()
        };

        assert!(!effective_desired_enabled(true, &state));
    }

    #[test]
    fn reconcile_state_keeps_edge_runs_active_across_instance_changes() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");

        let current = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            owner_instance_id: Some("simproc-other".to_string()),
            run_id: Some("run-edge".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            last_transition_reason: Some("manual_on".to_string()),
            updated_at: 110,
            ..ControlState::default()
        };

        let (next, transitions) = reconcile_state(130, true, &current);
        assert_eq!(next.phase, ControlPhase::Running);
        assert!(transitions.is_empty());
        assert!(next.desired_enabled);
        assert_eq!(next.owner_instance_id.as_deref(), Some("simproc-other"));

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn deterministic_request_targets_cover_key_defense_surfaces() {
        let runtime_profile = deterministic_runtime_profile();
        let without_honeypot = simulated_request_paths("run-coverage", 1);
        assert!(
            without_honeypot
                .iter()
                .any(|path| path == runtime_profile.paths.pow.as_str())
        );
        assert!(without_honeypot
            .iter()
            .any(|path| path == runtime_profile.paths.not_a_bot_checkbox.as_str()));
        assert!(!without_honeypot
            .iter()
            .any(|path| path == runtime_profile.paths.honeypot.as_str()));
        assert!(without_honeypot
            .iter()
            .any(|path| path.starts_with(runtime_profile.paths.public_search.as_str())));
        assert!(without_honeypot
            .iter()
            .any(|path| path.starts_with(crate::maze::entry_path("").as_str())));

        let with_honeypot = simulated_request_paths("run-coverage", 5);
        assert!(with_honeypot
            .iter()
            .any(|path| path == runtime_profile.paths.honeypot.as_str()));
    }

    #[test]
    fn shared_server_generated_request_target_matches_batch_contract() {
        let runtime_profile = deterministic_runtime_profile();
        let burst = &runtime_profile.rate_burst;
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::SharedServer,
                0,
            ),
            runtime_profile.primary_request_count
                + runtime_profile.supplemental_request_count
                + burst.high
        );
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::SharedServer,
                1,
            ),
            runtime_profile.primary_request_count
                + runtime_profile.supplemental_request_count
                + burst.low
        );
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::SharedServer,
                3,
            ),
            runtime_profile.primary_request_count
                + runtime_profile.supplemental_request_count
                + burst.medium
        );
    }

    #[test]
    fn edge_fermyon_generated_request_target_stays_within_bounded_budget() {
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::EdgeFermyon,
                0,
            ),
            6
        );
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::EdgeFermyon,
                1,
            ),
            4
        );
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::EdgeFermyon,
                3,
            ),
            5
        );
    }

    #[test]
    fn edge_fermyon_supplemental_lane_rotation_covers_full_contract() {
        let mut observed = std::collections::BTreeSet::new();
        for tick in 0..FULL_SUPPLEMENTAL_LANES.len() as u64 {
            for lane in supplemental_lanes_for_profile(
                crate::config::GatewayDeploymentProfile::EdgeFermyon,
                tick,
            ) {
                observed.insert(lane);
            }
        }

        let expected = std::collections::BTreeSet::from(FULL_SUPPLEMENTAL_LANES);
        assert_eq!(observed, expected);
    }
}
