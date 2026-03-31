use rand::random;

#[cfg(not(test))]
use serde_json::json;

#[cfg(not(test))]
use base64::{engine::general_purpose, Engine as _};
#[cfg(not(test))]
use hmac::{Hmac, Mac};
#[cfg(not(test))]
use sha2::Sha256;
#[cfg(not(test))]
use spin_sdk::http::{Method, Request};

use crate::challenge::KeyValueStore;

use super::adversary_sim::{
    next_llm_fulfillment_plan, AutonomousHeartbeatTickSummary, ControlPhase, ControlState,
    GenerationTickResult, LlmRuntimeResult, RuntimeLane, ScraplingWorkerPlan,
    ScraplingWorkerResult, WorkerFailureClass, SCRAPLING_MAX_BYTES_PER_TICK,
    SCRAPLING_MAX_DEPTH_PER_TICK, SCRAPLING_SIM_PROFILE, SCRAPLING_WORKER_PLAN_SCHEMA_VERSION,
};
use super::adversary_sim_corpus::deterministic_runtime_profile;
use super::adversary_sim_identity_pool::load_identity_pool_from_env;
use super::adversary_sim_realism_profile::scrapling_realism_profile_for_mode;
use super::adversary_sim_state::{
    active_lane_count_for_lane, autonomous_execution_profile, effective_active_lane,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SupplementalLane {
    ChallengeSubmit,
    NotABotFail,
    NotABotEscalate,
    PowVerify,
    TarpitProgress,
    FingerprintProbe,
    CdpReport,
}

pub(crate) const FULL_SUPPLEMENTAL_LANES: [SupplementalLane; 7] = [
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

pub(crate) fn simulated_request_paths(run_id: &str, tick_count: u64) -> [String; 9] {
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
        let index =
            (deterministic_lane_entropy(run_id, tick_count, slot) % public_paths.len() as u64)
                as usize;
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
    let rotation = (deterministic_lane_entropy(run_id, tick_count, 10) % paths.len() as u64)
        as usize;
    paths.rotate_left(rotation);
    paths
        .try_into()
        .unwrap_or_else(|_| unreachable!("primary request paths are fixed-size"))
}

pub(crate) fn deterministic_lane_entropy(run_id: &str, tick_count: u64, slot: u64) -> u64 {
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

pub(crate) fn primary_request_budget_for_profile(
    profile: crate::config::GatewayDeploymentProfile,
) -> usize {
    match profile {
        crate::config::GatewayDeploymentProfile::SharedServer => {
            deterministic_runtime_profile().primary_request_count as usize
        }
        crate::config::GatewayDeploymentProfile::EdgeFermyon => EDGE_FERMYON_PRIMARY_REQUESTS_PER_TICK,
    }
}

pub(crate) fn supplemental_lanes_for_profile(
    profile: crate::config::GatewayDeploymentProfile,
    tick_count: u64,
) -> Vec<SupplementalLane> {
    match profile {
        crate::config::GatewayDeploymentProfile::SharedServer => FULL_SUPPLEMENTAL_LANES.to_vec(),
        crate::config::GatewayDeploymentProfile::EdgeFermyon => {
            let lane_count =
                EDGE_FERMYON_SUPPLEMENTAL_LANES_PER_TICK.min(FULL_SUPPLEMENTAL_LANES.len());
            let start = ((tick_count as usize) * lane_count) % FULL_SUPPLEMENTAL_LANES.len();
            (0..lane_count)
                .map(|offset| FULL_SUPPLEMENTAL_LANES[(start + offset) % FULL_SUPPLEMENTAL_LANES.len()])
                .collect()
        }
    }
}

pub(crate) fn rate_burst_requests_for_profile(
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
pub(crate) fn rate_burst_requests_for_tick(tick_count: u64) -> u64 {
    rate_burst_requests_for_profile(crate::config::gateway_deployment_profile(), tick_count)
}

#[cfg(test)]
pub(crate) fn deterministic_generated_request_target_for_profile(
    profile: crate::config::GatewayDeploymentProfile,
    tick_count: u64,
) -> u64 {
    primary_request_budget_for_profile(profile) as u64
        + supplemental_lanes_for_profile(profile, tick_count).len() as u64
        + rate_burst_requests_for_profile(profile, tick_count)
}

#[cfg(test)]
pub(crate) fn deterministic_generated_request_target_for_tick(tick_count: u64) -> u64 {
    deterministic_generated_request_target_for_profile(
        crate::config::gateway_deployment_profile(),
        tick_count,
    )
}

#[cfg(not(test))]
pub(crate) fn simulated_request_ip(tick_count: u64, index: usize) -> String {
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
pub(crate) fn lane_actor_ip(
    third_octet: u8,
    tick_count: u64,
    rotate_every_ticks: u64,
    lane_salt: u64,
) -> String {
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
pub(crate) fn build_signed_not_a_bot_seed_token(
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
pub(crate) enum NotABotSubmissionProfile {
    Fail,
    EscalatePuzzle,
}

#[cfg(not(test))]
pub(crate) fn build_not_a_bot_submit_body(
    seed_token: &str,
    profile: NotABotSubmissionProfile,
) -> Vec<u8> {
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

pub(crate) fn apply_scrapling_worker_result(
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

pub(crate) fn apply_llm_runtime_result(state: &mut ControlState, result: &LlmRuntimeResult) {
    let failure_class = result.failure_class;
    let counters = state.lane_diagnostics.lane_mut(result.lane);
    if !result.passed
        || failure_class.is_some()
        || result.failed_action_count > 0
        || result.error.is_some()
    {
        counters.beat_failures = counters.beat_failures.saturating_add(1);
        counters.last_error = result.error.clone().or_else(|| {
            result.terminal_failure.clone().or_else(|| {
                Some(format!(
                    "llm_runtime_failed executed_action_count={} failed_action_count={}",
                    result.executed_action_count, result.failed_action_count
                ))
            })
        });
    } else {
        counters.beat_successes = counters.beat_successes.saturating_add(1);
        counters.last_error = None;
    }
    counters.generated_requests = counters
        .generated_requests
        .saturating_add(result.executed_action_count);
    for receipt in &result.action_receipts {
        if let Some(status) = receipt.status {
            let key = format!("status_{status}");
            let entry = counters.response_status_count.entry(key).or_insert(0);
            *entry = entry.saturating_add(1);
        }
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
        .saturating_add(result.executed_action_count);
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
    if state.pending_worker_tick_id.is_some() && state.active_lane != Some(state.desired_lane) {
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

fn next_scrapling_worker_plan(now: u64, state: &mut ControlState) -> ScraplingWorkerPlan {
    let run_id = state
        .run_id
        .clone()
        .or_else(|| state.last_run_id.clone())
        .unwrap_or_else(|| format!("simrun-runtime-{now}"));
    let tick_id = format!("scrapling-tick-{}-{:016x}", now, random::<u64>());
    let fulfillment_mode = scrapling_fulfillment_mode_for_tick(state.generated_tick_count);
    let realism_profile = scrapling_realism_profile_for_mode(fulfillment_mode);
    let request_proxy_url = optional_scrapling_proxy_env("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
    let browser_proxy_url = optional_scrapling_proxy_env("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL")
        .or_else(|| request_proxy_url.clone());
    let request_identity_pool =
        load_identity_pool_from_env("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON");
    let browser_identity_pool =
        load_identity_pool_from_env("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON");
    state.pending_worker_tick_id = Some(tick_id.clone());
    state.pending_worker_started_at = Some(now);
    state.updated_at = now;
    ScraplingWorkerPlan {
        schema_version: SCRAPLING_WORKER_PLAN_SCHEMA_VERSION.to_string(),
        run_id,
        tick_id,
        lane: RuntimeLane::ScraplingTraffic,
        sim_profile: SCRAPLING_SIM_PROFILE.to_string(),
        fulfillment_mode: fulfillment_mode.to_string(),
        category_targets:
            crate::observability::non_human_lane_fulfillment::scrapling_category_targets_for_mode(
                fulfillment_mode,
            ),
        surface_targets:
            crate::observability::scrapling_owned_surface::scrapling_owned_surface_targets_for_mode(
                fulfillment_mode,
            ),
        request_proxy_url,
        browser_proxy_url,
        request_identity_pool,
        browser_identity_pool,
        tick_started_at: now,
        max_requests: realism_profile.pressure_envelope.max_activities,
        max_depth: SCRAPLING_MAX_DEPTH_PER_TICK,
        max_bytes: SCRAPLING_MAX_BYTES_PER_TICK,
        max_ms: realism_profile.pressure_envelope.max_time_budget_ms,
        realism_profile,
    }
}

fn scrapling_fulfillment_mode_for_tick(generated_tick_count: u64) -> &'static str {
    match generated_tick_count % 5 {
        0 => "crawler",
        1 => "bulk_scraper",
        2 => "browser_automation",
        3 => "stealth_browser",
        _ => "http_agent",
    }
}

fn optional_scrapling_proxy_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{next_scrapling_worker_plan, scrapling_fulfillment_mode_for_tick};
    use crate::admin::adversary_sim::ControlState;

    #[test]
    fn scrapling_fulfillment_modes_cycle_across_full_spectrum_personas() {
        assert_eq!(scrapling_fulfillment_mode_for_tick(0), "crawler");
        assert_eq!(scrapling_fulfillment_mode_for_tick(1), "bulk_scraper");
        assert_eq!(scrapling_fulfillment_mode_for_tick(2), "browser_automation");
        assert_eq!(scrapling_fulfillment_mode_for_tick(3), "stealth_browser");
        assert_eq!(scrapling_fulfillment_mode_for_tick(4), "http_agent");
        assert_eq!(scrapling_fulfillment_mode_for_tick(5), "crawler");
    }

    #[test]
    fn scrapling_worker_plan_surfaces_realism_profile_contract() {
        let mut state = ControlState::default();
        let plan = next_scrapling_worker_plan(1_700_000_000, &mut state);
        let contract: serde_json::Value = serde_json::from_str(include_str!(
            "../../scripts/tests/adversarial/lane_realism_contract.v1.json"
        ))
        .expect("lane realism contract parses");
        let expected = &contract["profiles"]["scrapling_traffic"]["crawler"];

        assert_eq!(
            serde_json::to_value(&plan.realism_profile).expect("realism profile serializes"),
            *expected
        );
    }

    #[test]
    fn scrapling_worker_plan_uses_mode_specific_pressure_envelopes() {
        let mut crawler_state = ControlState::default();
        crawler_state.generated_tick_count = 0;
        let crawler_plan = next_scrapling_worker_plan(1_700_000_100, &mut crawler_state);

        let mut bulk_state = ControlState::default();
        bulk_state.generated_tick_count = 1;
        let bulk_plan = next_scrapling_worker_plan(1_700_000_101, &mut bulk_state);

        assert_eq!(
            crawler_plan.max_requests,
            crawler_plan.realism_profile.pressure_envelope.max_activities
        );
        assert_eq!(
            bulk_plan.max_requests,
            bulk_plan.realism_profile.pressure_envelope.max_activities
        );
        assert_eq!(
            crawler_plan.max_ms,
            crawler_plan.realism_profile.pressure_envelope.max_time_budget_ms
        );
        assert_eq!(
            bulk_plan.max_ms,
            bulk_plan.realism_profile.pressure_envelope.max_time_budget_ms
        );
        assert!(bulk_plan.max_requests > crawler_plan.max_requests);
        assert!(bulk_plan.max_requests > 8);
        assert!(bulk_plan.max_ms > 2_000);
    }
}

pub(crate) fn run_autonomous_supervisor_ticks(
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
                summary.pending_dispatch_mode = Some("scrapling_worker_pending".to_string());
                return summary;
            }
            record_lane_attempt(state, RuntimeLane::ScraplingTraffic);
            summary.worker_plan = Some(next_scrapling_worker_plan(now, state));
            return summary;
        }
        Some(RuntimeLane::BotRedTeam) => {
            if state.pending_worker_tick_id.is_some() {
                summary.worker_pending = true;
                summary.pending_dispatch_mode = Some("llm_fulfillment_plan_pending".to_string());
                return summary;
            }
            record_lane_attempt(state, RuntimeLane::BotRedTeam);
            let frontier = crate::config::frontier_summary();
            let plan = next_llm_fulfillment_plan(now, state, &frontier);
            let counters = state.lane_diagnostics.lane_mut(RuntimeLane::BotRedTeam);
            counters.last_error = if plan.backend_state == "unavailable" {
                Some("llm_backend_unavailable".to_string())
            } else if plan.backend_state == "degraded" {
                Some("llm_backend_degraded".to_string())
            } else {
                None
            };
            state.last_generation_error = counters.last_error.clone();
            state.pending_worker_tick_id = Some(plan.tick_id.clone());
            state.pending_worker_started_at = Some(plan.tick_started_at);
            state.updated_at = now;
            summary.llm_fulfillment_plan = Some(plan);
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

pub(crate) fn run_internal_generation_tick(
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
        let forwarded_secret =
            crate::config::runtime_var_trimmed_optional("SHUMA_FORWARDED_IP_SECRET");
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
            if crate::http_route_namespace::is_generated_public_site_path(path) {
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
                        format!("sim-ja3-{}-{}", state.generated_tick_count, index).as_str(),
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
                b"answer=bad&seed=invalid&return_to=%2F".to_vec();
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
                crate::http_route_namespace::PUBLIC_ABOUT_PATH,
                deterministic_lane_entropy(run_id.as_str(), state.generated_tick_count, 101),
                1 + (state.generated_tick_count % 5),
            ) {
                let fail_body =
                    build_not_a_bot_submit_body(&fail_seed, NotABotSubmissionProfile::Fail);
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
                crate::http_route_namespace::PUBLIC_RESEARCH_PATH,
                deterministic_lane_entropy(run_id.as_str(), state.generated_tick_count, 102),
                2 + (state.generated_tick_count.wrapping_mul(3) % 7),
            ) {
                let escalate_body = build_not_a_bot_submit_body(
                    &escalate_seed,
                    NotABotSubmissionProfile::EscalatePuzzle,
                );
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
            .unwrap_or_else(|_| {
                b"{\"cdp_detected\":true,\"score\":4.8,\"checks\":[\"webdriver\"]}".to_vec()
            });
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
                deterministic_lane_entropy(
                    run_id.as_str(),
                    state.generated_tick_count,
                    120 + burst_index,
                ) % 10_000
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
                    .header("x-shuma-edge-browser-family", "chrome");
            }
            dispatch_request(burst_builder.body(Vec::new()).build());
        }
        crate::observability::monitoring::flush_pending_counters(store);
    }
    #[cfg(test)]
    {
        let _ = store;
        let _ = metadata;
        result.generated_requests =
            deterministic_generated_request_target_for_tick(state.generated_tick_count);
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
