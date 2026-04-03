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
    GenerationTickResult, LlmRuntimeResult, RuntimeLane, ScraplingWorkerPlan, ScraplingWorkerResult,
    WorkerFailureClass, SCRAPLING_SIM_PROFILE, SCRAPLING_WORKER_PLAN_SCHEMA_VERSION,
};
#[cfg(not(test))]
use super::adversary_sim_corpus::build_synthetic_runtime_observation;
use super::adversary_sim_corpus::deterministic_runtime_profile;
use super::adversary_sim_identity_pool::load_identity_pool_from_env;
use super::adversary_sim_realism_profile::scrapling_realism_profile_for_mode;
use super::adversary_sim_state::{
    active_lane_count_for_lane, autonomous_execution_profile, clear_lane_pending_worker,
    effective_active_lane, lane_has_pending_worker, set_lane_pending_worker,
};
use super::adversary_sim_trusted_ingress::{
    trusted_ingress_proxy_config_from_env, trusted_ingress_proxy_url_for_client_ip,
};
use super::adversary_sim_worker_plan::LaneRealismRecurrenceContext;

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

fn curated_local_contributor_request_segment(fulfillment_mode: &str) -> Option<u16> {
    match fulfillment_mode {
        "crawler" => Some(0x0010),
        "bulk_scraper" => Some(0x0012),
        "http_agent" => Some(0x0100),
        _ => None,
    }
}

fn curated_local_contributor_browser_segment(fulfillment_mode: &str) -> Option<u16> {
    match fulfillment_mode {
        "stealth_browser" => Some(0x0040),
        "browser_automation" => Some(0x0020),
        _ => None,
    }
}

fn simulated_local_contributor_client_ip(
    fulfillment_mode: &str,
    run_id: &str,
    tick_count: u64,
    index: usize,
) -> String {
    let curated_segment = match index {
        0 => curated_local_contributor_request_segment(fulfillment_mode),
        1 => curated_local_contributor_browser_segment(fulfillment_mode),
        _ => None,
    };
    if let Some(segment3) = curated_segment {
        let segment4 = ((deterministic_lane_entropy(
            run_id,
            0,
            0x4c4f4341 ^ segment3 as u64 ^ index as u64,
        )) % 65_535)
            + 1;
        let host = ((deterministic_lane_entropy(
            run_id,
            tick_count,
            0x4c434980 ^ segment3 as u64 ^ index as u64,
        )) % 65_535)
            + 1;
        return format!("2001:db8:{segment3:x}:{segment4:x}::{host:x}");
    }

    let actor_ordinal = tick_count.saturating_mul(2).saturating_add(index as u64);
    // Use synthetic documentation-only IPv6 identities locally so repeated sim runs do not
    // collapse into a tiny recycled IPv4 /24 space and accidentally inherit stale rate/fingerprint
    // state that Shuma would treat as the same actor bucket.
    let segment3 =
        ((deterministic_lane_entropy(run_id, 0, 0x4c4f4341).wrapping_add(actor_ordinal)) % 65_535)
            + 1;
    let segment4 = ((deterministic_lane_entropy(
        run_id,
        tick_count,
        0x4c434950 + index as u64,
    )) % 65_535)
        + 1;
    let host = ((deterministic_lane_entropy(
        run_id,
        tick_count,
        0x4c434980 + index as u64,
    )) % 65_535)
        + 1;
    format!("2001:db8:{segment3:x}:{segment4:x}::{host:x}")
}

#[allow(dead_code)]
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

fn reset_recurrence_state(state: &mut ControlState) {
    state.recurrence_strategy = None;
    state.recurrence_reentry_scope = None;
    state.recurrence_dormancy_truth_mode = None;
    state.recurrence_session_index = 0;
    state.recurrence_reentry_count = 0;
    state.recurrence_max_reentries_per_run = None;
    state.recurrence_last_planned_gap_seconds = None;
    state.recurrence_last_representative_gap_seconds = None;
    state.recurrence_dormant_until = None;
}

fn recurrence_gap_seconds_from_range(
    now: u64,
    state: &ControlState,
    gap_range: &super::adversary_sim_realism_profile::LaneRealismRange,
) -> u64 {
    let min_gap = gap_range.min;
    let max_gap = gap_range.max.max(min_gap);
    if min_gap == max_gap {
        return min_gap;
    }
    let salt = state.recurrence_reentry_count.saturating_add(state.generated_tick_count);
    min_gap + (deterministic_lane_entropy(state.run_id.as_deref().unwrap_or("simrun-runtime"), now, salt) % (max_gap - min_gap + 1))
}

fn recurrence_gap_seconds(
    now: u64,
    state: &ControlState,
    realism_profile: &super::adversary_sim_realism_profile::LaneRealismProfile,
) -> u64 {
    recurrence_gap_seconds_from_range(
        now,
        state,
        &realism_profile.recurrence_envelope.dormant_gap_seconds,
    )
}

fn representative_recurrence_gap_seconds(
    now: u64,
    state: &ControlState,
    realism_profile: &super::adversary_sim_realism_profile::LaneRealismProfile,
) -> u64 {
    recurrence_gap_seconds_from_range(
        now,
        state,
        &realism_profile
            .recurrence_envelope
            .representative_dormant_gap_seconds,
    )
}

fn dormancy_truth_mode(planned_gap: u64, representative_gap: u64) -> &'static str {
    if representative_gap > planned_gap {
        "accelerated_local_proof"
    } else {
        "representative_runtime"
    }
}

pub(crate) fn recurrence_context_for_profile(
    now: u64,
    state: &mut ControlState,
    realism_profile: &super::adversary_sim_realism_profile::LaneRealismProfile,
) -> Option<LaneRealismRecurrenceContext> {
    let envelope = &realism_profile.recurrence_envelope;
    if envelope.strategy.trim().is_empty() {
        return None;
    }
    if state.recurrence_strategy.as_deref() != Some(envelope.strategy.as_str()) {
        state.recurrence_strategy = Some(envelope.strategy.clone());
    }
    state.recurrence_reentry_scope = Some(envelope.reentry_scope.clone());
    if state.recurrence_session_index == 0 {
        state.recurrence_session_index = 1;
    }
    state.recurrence_max_reentries_per_run = Some(envelope.max_reentries_per_run);
    let planned_gap = recurrence_gap_seconds(now, state, realism_profile);
    let representative_gap = representative_recurrence_gap_seconds(now, state, realism_profile);
    let truth_mode = dormancy_truth_mode(planned_gap, representative_gap);
    state.recurrence_dormancy_truth_mode = Some(truth_mode.to_string());
    state.recurrence_last_planned_gap_seconds = Some(planned_gap);
    state.recurrence_last_representative_gap_seconds = Some(representative_gap);
    Some(LaneRealismRecurrenceContext {
        strategy: envelope.strategy.clone(),
        reentry_scope: envelope.reentry_scope.clone(),
        dormancy_truth_mode: truth_mode.to_string(),
        session_index: state.recurrence_session_index,
        reentry_count: state.recurrence_reentry_count,
        max_reentries_per_run: envelope.max_reentries_per_run,
        planned_dormant_gap_seconds: planned_gap,
        representative_dormant_gap_seconds: representative_gap,
    })
}

fn recurrence_dormant(state: &ControlState, now: u64) -> bool {
    state.recurrence_dormant_until
        .map(|until| now < until)
        .unwrap_or(false)
}

fn clear_recurrence_dormancy_if_ready(state: &mut ControlState, now: u64) {
    if state
        .recurrence_dormant_until
        .map(|until| now >= until)
        .unwrap_or(false)
    {
        state.recurrence_dormant_until = None;
        state.recurrence_session_index = state.recurrence_session_index.saturating_add(1);
        state.updated_at = now;
    }
}

fn schedule_recurrence_dormancy_after_tick(
    state: &mut ControlState,
    tick_completed_at: u64,
    realism_profile: &super::adversary_sim_realism_profile::LaneRealismProfile,
) {
    let envelope = &realism_profile.recurrence_envelope;
    if envelope.strategy.trim().is_empty() {
        return;
    }
    state.recurrence_strategy = Some(envelope.strategy.clone());
    state.recurrence_reentry_scope = Some(envelope.reentry_scope.clone());
    state.recurrence_max_reentries_per_run = Some(envelope.max_reentries_per_run);
    if state.recurrence_session_index == 0 {
        state.recurrence_session_index = 1;
    }
    if state.recurrence_reentry_count >= envelope.max_reentries_per_run {
        state.recurrence_dormant_until = None;
        return;
    }
    let planned_gap = state
        .recurrence_last_planned_gap_seconds
        .unwrap_or_else(|| recurrence_gap_seconds(tick_completed_at, state, realism_profile));
    let representative_gap = state
        .recurrence_last_representative_gap_seconds
        .unwrap_or_else(|| representative_recurrence_gap_seconds(tick_completed_at, state, realism_profile));
    state.recurrence_last_planned_gap_seconds = Some(planned_gap);
    state.recurrence_last_representative_gap_seconds = Some(representative_gap);
    state.recurrence_dormancy_truth_mode =
        Some(dormancy_truth_mode(planned_gap, representative_gap).to_string());
    state.recurrence_reentry_count = state.recurrence_reentry_count.saturating_add(1);
    state.recurrence_dormant_until = Some(tick_completed_at.saturating_add(planned_gap));
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
    clear_lane_pending_worker(state, RuntimeLane::ScraplingTraffic);
    let realism_profile = scrapling_realism_profile_for_mode(result.fulfillment_mode.as_str());
    if state.desired_lane == RuntimeLane::ParallelMixedTraffic {
        reset_recurrence_state(state);
    } else if scrapling_completed_within_run_mode_cycle(state) {
        schedule_recurrence_dormancy_after_tick(state, result.tick_completed_at, &realism_profile);
    } else {
        state.recurrence_dormant_until = None;
    }
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
    clear_lane_pending_worker(state, RuntimeLane::BotRedTeam);
    let realism_profile =
        super::adversary_sim_realism_profile::llm_realism_profile_for_mode(
            result.fulfillment_mode.as_str(),
        );
    if state.desired_lane == RuntimeLane::ParallelMixedTraffic {
        reset_recurrence_state(state);
    } else {
        schedule_recurrence_dormancy_after_tick(state, result.tick_completed_at, &realism_profile);
    }
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
    if lane_has_pending_worker(state, state.active_lane.unwrap_or(state.desired_lane))
        && state.active_lane != Some(state.desired_lane)
    {
        clear_lane_pending_worker(state, RuntimeLane::ParallelMixedTraffic);
    }
    state.active_lane = Some(state.desired_lane);
    state.active_lane_count = active_lane_count_for_lane(state.desired_lane);
    state.lane_switch_seq = state.lane_switch_seq.saturating_add(1);
    state.last_lane_switch_at = Some(now);
    state.last_lane_switch_reason = Some("beat_boundary_reconciliation".to_string());
    reset_recurrence_state(state);
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
    let request_identity_pool =
        load_identity_pool_from_env("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON");
    let browser_identity_pool =
        load_identity_pool_from_env("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON");
    let local_contributor_mode = local_contributor_ingress_enabled();
    let explicit_request_proxy_url =
        optional_scrapling_proxy_env("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
    let explicit_browser_proxy_url =
        optional_scrapling_proxy_env("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");
    let trusted_ingress_config = trusted_ingress_proxy_config_from_env();
    let trusted_request_proxy_url = if !local_contributor_mode && request_identity_pool.is_empty() {
        trusted_ingress_config.as_ref().and_then(|config| {
            trusted_ingress_proxy_url_for_client_ip(
                config,
                simulated_request_ip(state.generated_tick_count, 0).as_str(),
            )
        })
    } else {
        None
    };
    let trusted_browser_proxy_url = if !local_contributor_mode && browser_identity_pool.is_empty() {
        trusted_ingress_config.as_ref().and_then(|config| {
            trusted_ingress_proxy_url_for_client_ip(
                config,
                simulated_request_ip(state.generated_tick_count, 1).as_str(),
            )
        })
    } else {
        None
    };
    let request_proxy_url = explicit_request_proxy_url.or(trusted_request_proxy_url);
    let browser_proxy_url = explicit_browser_proxy_url
        .or_else(|| trusted_browser_proxy_url.clone())
        .or_else(|| request_proxy_url.clone());
    let local_request_client_ip = if local_contributor_mode
        && request_identity_pool.is_empty()
        && request_proxy_url.is_none()
    {
        Some(simulated_local_contributor_client_ip(
            fulfillment_mode,
            run_id.as_str(),
            state.generated_tick_count,
            0,
        ))
    } else {
        None
    };
    let local_browser_client_ip = if local_contributor_mode
        && browser_identity_pool.is_empty()
        && browser_proxy_url.is_none()
    {
        Some(simulated_local_contributor_client_ip(
            fulfillment_mode,
            run_id.as_str(),
            state.generated_tick_count,
            1,
        ))
    } else {
        None
    };
    let recurrence_context = recurrence_context_for_profile(now, state, &realism_profile);
    set_lane_pending_worker(state, RuntimeLane::ScraplingTraffic, tick_id.clone(), now);
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
        local_request_client_ip,
        local_browser_client_ip,
        request_identity_pool,
        browser_identity_pool,
        tick_started_at: now,
        recurrence_context,
        max_requests: realism_profile.pressure_envelope.max_activities,
        max_depth: realism_profile.exploration_envelope.max_depth,
        max_bytes: realism_profile.exploration_envelope.max_bytes,
        max_ms: realism_profile.pressure_envelope.max_time_budget_ms,
        realism_profile,
    }
}

const SCRAPLING_FULFILLMENT_MODE_CYCLE_LEN: u64 = 5;
const SCRAPLING_PENDING_WORKER_GRACE_SECONDS: u64 = 2;

fn scrapling_fulfillment_mode_for_tick(generated_tick_count: u64) -> &'static str {
    match generated_tick_count % SCRAPLING_FULFILLMENT_MODE_CYCLE_LEN {
        0 => "crawler",
        1 => "bulk_scraper",
        2 => "stealth_browser",
        3 => "http_agent",
        _ => "browser_automation",
    }
}

fn scrapling_completed_within_run_mode_cycle(state: &ControlState) -> bool {
    state.generated_tick_count > 0
        && state.generated_tick_count % SCRAPLING_FULFILLMENT_MODE_CYCLE_LEN == 0
}

fn optional_scrapling_proxy_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn local_contributor_ingress_enabled() -> bool {
    matches!(
        std::env::var("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

fn scrapling_pending_worker_timeout_seconds(state: &ControlState) -> u64 {
    let fulfillment_mode = scrapling_fulfillment_mode_for_tick(state.generated_tick_count);
    let max_time_budget_ms = scrapling_realism_profile_for_mode(fulfillment_mode)
        .pressure_envelope
        .max_time_budget_ms;
    max_time_budget_ms
        .div_ceil(1_000)
        .saturating_add(autonomous_execution_profile().cadence_seconds)
        .saturating_add(SCRAPLING_PENDING_WORKER_GRACE_SECONDS)
}

fn clear_stale_scrapling_pending_worker(state: &mut ControlState, now: u64) -> bool {
    let Some(started_at) = state.pending_scrapling_started_at else {
        return false;
    };
    let timeout_seconds = scrapling_pending_worker_timeout_seconds(state);
    if now < started_at.saturating_add(timeout_seconds) {
        return false;
    }

    let last_error = {
        let counters = state.lane_diagnostics.lane_mut(RuntimeLane::ScraplingTraffic);
        counters.beat_failures = counters.beat_failures.saturating_add(1);
        counters.last_error = Some("scrapling_worker_stale_timeout".to_string());
        counters.last_error.clone()
    };
    state.last_generation_error = last_error;
    clear_lane_pending_worker(state, RuntimeLane::ScraplingTraffic);
    state.updated_at = now;
    true
}

#[cfg(test)]
mod tests {
    use super::{
        apply_scrapling_worker_result, next_scrapling_worker_plan, run_autonomous_supervisor_ticks,
        scrapling_fulfillment_mode_for_tick, scrapling_pending_worker_timeout_seconds,
        SCRAPLING_FULFILLMENT_MODE_CYCLE_LEN,
    };
    use crate::admin::adversary_sim::{
        ControlPhase, ControlState, RuntimeLane, ScraplingWorkerResult,
        SCRAPLING_WORKER_RESULT_SCHEMA_VERSION,
    };
    use crate::admin::adversary_sim_realism_profile::scrapling_realism_profile_for_mode;
    use crate::admin::adversary_sim_state::{autonomous_execution_profile, process_instance_id};
    use crate::admin::adversary_sim_worker_plan::ScraplingCrawlStats;
    use crate::test_support::InMemoryStore;
    use std::collections::BTreeSet;

    #[test]
    fn scrapling_fulfillment_modes_cycle_across_full_spectrum_personas() {
        assert_eq!(scrapling_fulfillment_mode_for_tick(0), "crawler");
        assert_eq!(scrapling_fulfillment_mode_for_tick(1), "bulk_scraper");
        assert_eq!(scrapling_fulfillment_mode_for_tick(2), "stealth_browser");
        assert_eq!(scrapling_fulfillment_mode_for_tick(3), "http_agent");
        assert_eq!(scrapling_fulfillment_mode_for_tick(4), "browser_automation");
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

    #[test]
    fn serialized_scrapling_mode_cycle_fits_with_headroom_inside_default_runtime_window() {
        let cadence_seconds = autonomous_execution_profile().cadence_seconds;
        let fulfillment_modes = [
            "crawler",
            "bulk_scraper",
            "stealth_browser",
            "http_agent",
            "browser_automation",
        ];
        let total_budget_ms: u64 = fulfillment_modes
            .iter()
            .map(|mode| {
                scrapling_realism_profile_for_mode(mode)
                    .pressure_envelope
                    .max_time_budget_ms
            })
            .sum();
        let cycle_budget_seconds = total_budget_ms.div_ceil(1_000).saturating_add(
            cadence_seconds.saturating_mul((fulfillment_modes.len() as u64).saturating_sub(1)),
        );
        let default_window_seconds = crate::config::defaults().adversary_sim_duration_seconds;

        assert!(
            cycle_budget_seconds <= default_window_seconds.saturating_sub(8),
            "serialized five-mode cycle needs real headroom inside the default window: cycle_budget_seconds={cycle_budget_seconds} default_window_seconds={default_window_seconds}"
        );
    }

    #[test]
    fn scrapling_worker_plan_uses_mode_specific_exploration_envelopes() {
        let mut crawler_state = ControlState::default();
        crawler_state.generated_tick_count = 0;
        let crawler_plan = next_scrapling_worker_plan(1_700_000_100, &mut crawler_state);

        let mut bulk_state = ControlState::default();
        bulk_state.generated_tick_count = 1;
        let bulk_plan = next_scrapling_worker_plan(1_700_000_101, &mut bulk_state);

        assert_eq!(
            crawler_plan.max_depth,
            crawler_plan.realism_profile.exploration_envelope.max_depth
        );
        assert_eq!(
            bulk_plan.max_depth,
            bulk_plan.realism_profile.exploration_envelope.max_depth
        );
        assert_eq!(
            crawler_plan.max_bytes,
            crawler_plan.realism_profile.exploration_envelope.max_bytes
        );
        assert_eq!(
            bulk_plan.max_bytes,
            bulk_plan.realism_profile.exploration_envelope.max_bytes
        );
        assert!(bulk_plan.max_depth > crawler_plan.max_depth);
        assert!(bulk_plan.max_bytes > crawler_plan.max_bytes);
        assert!(bulk_plan.max_depth > 2);
    }

    #[test]
    fn scrapling_worker_plan_uses_trusted_ingress_proxy_when_configured_without_explicit_proxy_or_pool() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON");
        std::env::set_var(
            "ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL",
            "http://127.0.0.1:3871",
        );
        std::env::set_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN", "trusted-token");

        let mut state = ControlState::default();
        state.generated_tick_count = 0;
        let plan = next_scrapling_worker_plan(1_700_000_200, &mut state);

        assert_eq!(
            plan.request_proxy_url.as_deref(),
            Some("http://198.51.1.1:trusted-token@127.0.0.1:3871")
        );
        assert_eq!(
            plan.browser_proxy_url.as_deref(),
            Some("http://198.51.1.2:trusted-token@127.0.0.1:3871")
        );

        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN");
    }

    #[test]
    fn scrapling_worker_plan_prefers_explicit_request_proxy_over_trusted_ingress_fallback() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var(
            "ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL",
            "http://explicit-proxy.internal:9001",
        );
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON");
        std::env::set_var(
            "ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL",
            "http://127.0.0.1:3871",
        );
        std::env::set_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN", "trusted-token");

        let mut state = ControlState::default();
        state.generated_tick_count = 0;
        let plan = next_scrapling_worker_plan(1_700_000_201, &mut state);

        assert_eq!(
            plan.request_proxy_url.as_deref(),
            Some("http://explicit-proxy.internal:9001")
        );
        assert_eq!(
            plan.browser_proxy_url.as_deref(),
            Some("http://198.51.1.2:trusted-token@127.0.0.1:3871")
        );

        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN");
    }

    #[test]
    fn scrapling_worker_plan_skips_trusted_ingress_proxy_fallback_in_local_contributor_mode() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON");
        std::env::set_var("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE", "true");
        std::env::set_var(
            "ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL",
            "http://127.0.0.1:3000",
        );
        std::env::set_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN", "trusted-token");

        let mut state = ControlState::default();
        state.run_id = Some("simrun-local-proof".to_string());
        state.generated_tick_count = 0;
        let plan = next_scrapling_worker_plan(1_700_000_202, &mut state);

        assert_eq!(
            plan.request_proxy_url.as_deref(),
            None
        );
        assert_eq!(
            plan.browser_proxy_url.as_deref(),
            None
        );
        assert_eq!(
            plan.local_request_client_ip
                .as_deref()
                .map(|value| value.starts_with("2001:db8:10:")),
            Some(true)
        );
        assert_eq!(
            plan.local_browser_client_ip
                .as_deref()
                .map(|value| value.starts_with("2001:db8:")),
            Some(true)
        );
        assert_ne!(
            crate::signals::ip_identity::bucket_ip(
                plan.local_request_client_ip
                    .as_deref()
                    .expect("local request client ip"),
            ),
            crate::signals::ip_identity::bucket_ip(
                plan.local_browser_client_ip
                    .as_deref()
                    .expect("local browser client ip"),
            )
        );

        std::env::remove_var("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE");
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN");
    }

    #[test]
    fn local_contributor_scrapling_personas_do_not_share_bucketed_client_identity() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN");
        std::env::set_var("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE", "true");

        let mut observed_buckets = Vec::new();
        for tick in 0..SCRAPLING_FULFILLMENT_MODE_CYCLE_LEN {
            let mut state = ControlState::default();
            state.run_id = Some("simrun-local-proof".to_string());
            state.generated_tick_count = tick;
            let plan = next_scrapling_worker_plan(1_700_000_240 + tick, &mut state);
            let request_ip = plan
                .local_request_client_ip
                .as_deref()
                .expect("local request client ip");
            let browser_ip = plan
                .local_browser_client_ip
                .as_deref()
                .expect("local browser client ip");
            let request_bucket = crate::signals::ip_identity::bucket_ip(request_ip);
            let browser_bucket = crate::signals::ip_identity::bucket_ip(browser_ip);

            assert_ne!(
                request_bucket, browser_bucket,
                "request and browser identities for tick {tick} must not collapse into the same /64 bucket"
            );
            observed_buckets.push(request_bucket);
            observed_buckets.push(browser_bucket);
        }

        let unique_buckets: BTreeSet<String> = observed_buckets.iter().cloned().collect();
        assert_eq!(
            unique_buckets.len(),
            observed_buckets.len(),
            "each Scrapling persona identity in local contributor mode must occupy its own /64 bucket"
        );

        std::env::remove_var("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE");
    }

    #[test]
    fn local_contributor_scrapling_personas_use_mode_specific_identity_families_without_reusing_fixed_single_ips(
    ) {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_URL");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_REQUEST_PROXY_POOL_JSON");
        std::env::remove_var("ADVERSARY_SIM_SCRAPLING_BROWSER_PROXY_POOL_JSON");
        std::env::set_var("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE", "true");

        let expected = [
            (0u64, Some("2001:db8:10:"), None),
            (1u64, Some("2001:db8:12:"), None),
            (2u64, None, Some("2001:db8:40:")),
            (3u64, Some("2001:db8:100:"), None),
            (4u64, None, Some("2001:db8:20:")),
        ];

        for (tick, expected_request_prefix, expected_browser_prefix) in expected {
            let mut state = ControlState::default();
            state.run_id = Some("simrun-local-proof".to_string());
            state.generated_tick_count = tick;
            let plan = next_scrapling_worker_plan(1_700_000_250 + tick, &mut state);

            if let Some(request_prefix) = expected_request_prefix {
                assert_eq!(
                    plan.local_request_client_ip
                        .as_deref()
                        .map(|value| value.starts_with(request_prefix)),
                    Some(true)
                );
            }
            if let Some(browser_prefix) = expected_browser_prefix {
                assert_eq!(
                    plan.local_browser_client_ip
                        .as_deref()
                        .map(|value| value.starts_with(browser_prefix)),
                    Some(true)
                );
            }
        }

        let first_http_agent_ip = super::simulated_local_contributor_client_ip(
            "http_agent",
            "simrun-local-proof-a",
            3,
            0,
        );
        let second_http_agent_ip = super::simulated_local_contributor_client_ip(
            "http_agent",
            "simrun-local-proof-b",
            3,
            0,
        );
        assert_ne!(
            crate::signals::ip_identity::bucket_ip(first_http_agent_ip.as_str()),
            crate::signals::ip_identity::bucket_ip(second_http_agent_ip.as_str()),
            "local contributor hostile identities must not recycle the same /64 bucket across runs"
        );

        std::env::remove_var("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE");
    }

    #[test]
    fn scrapling_worker_plan_surfaces_long_window_recurrence_context() {
        let mut state = ControlState::default();
        let plan = next_scrapling_worker_plan(1_700_000_000, &mut state);
        let recurrence = plan
            .recurrence_context
            .as_ref()
            .expect("recurrence context");

        assert_eq!(recurrence.strategy, "bounded_campaign_return");
        assert_eq!(recurrence.reentry_scope, "cross_window_campaign");
        assert_eq!(recurrence.dormancy_truth_mode, "accelerated_local_proof");
        assert_eq!(recurrence.session_index, 1);
        assert_eq!(recurrence.reentry_count, 0);
        assert!(recurrence.max_reentries_per_run >= 1);
        assert!(recurrence.planned_dormant_gap_seconds >= 1);
        assert!(recurrence.representative_dormant_gap_seconds >= 3_600);
        assert!(
            recurrence.representative_dormant_gap_seconds
                > recurrence.planned_dormant_gap_seconds
        );
    }

    #[test]
    fn autonomous_supervisor_honors_recurrence_dormancy_before_dispatching_reentry_tick() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");
        let store = InMemoryStore::default();
        let mut state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            desired_lane: RuntimeLane::ScraplingTraffic,
            active_lane: Some(RuntimeLane::ScraplingTraffic),
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-reentry".to_string()),
            started_at: Some(100),
            ends_at: Some(500),
            active_run_count: 1,
            active_lane_count: 1,
            ..ControlState::default()
        };

        let mut now = 1_700_000_000u64;
        let mut recurrence = None;
        let mut final_result = None;
        for _ in 0..SCRAPLING_FULFILLMENT_MODE_CYCLE_LEN {
            let plan = next_scrapling_worker_plan(now, &mut state);
            recurrence = plan.recurrence_context.clone();
            let result = ScraplingWorkerResult {
                schema_version: SCRAPLING_WORKER_RESULT_SCHEMA_VERSION.to_string(),
                run_id: plan.run_id.clone(),
                tick_id: plan.tick_id.clone(),
                lane: RuntimeLane::ScraplingTraffic,
                fulfillment_mode: plan.fulfillment_mode.clone(),
                category_targets: plan.category_targets.clone(),
                worker_id: "scrapling-worker-test".to_string(),
                tick_started_at: plan.tick_started_at,
                tick_completed_at: plan.tick_started_at.saturating_add(1),
                generated_requests: 2,
                failed_requests: 0,
                last_response_status: Some(200),
                failure_class: None,
                error: None,
                crawl_stats: ScraplingCrawlStats::default(),
                scope_rejections: std::collections::BTreeMap::new(),
                realism_receipt: None,
                surface_receipts: Vec::new(),
            };
            apply_scrapling_worker_result(&mut state, &result);
            now = result.tick_completed_at.saturating_add(1);
            final_result = Some(result);
        }

        let recurrence = recurrence.expect("recurrence context");
        let final_result = final_result.expect("final cycle result");
        let dormant_at = final_result
            .tick_completed_at
            .saturating_add(recurrence.planned_dormant_gap_seconds)
            .saturating_sub(1);
        let dormant_summary = run_autonomous_supervisor_ticks(&store, &mut state, dormant_at);
        assert!(dormant_summary.worker_plan.is_none());
        assert_eq!(
            dormant_summary.pending_dispatch_mode.as_deref(),
            Some("recurrence_dormant")
        );

        let reentry_at = final_result
            .tick_completed_at
            .saturating_add(recurrence.planned_dormant_gap_seconds);
        let reentry_summary = run_autonomous_supervisor_ticks(&store, &mut state, reentry_at);
        let reentry_plan = reentry_summary.worker_plan.expect("re-entry worker plan");
        let reentry_context = reentry_plan
            .recurrence_context
            .as_ref()
            .expect("re-entry recurrence context");
        assert_eq!(reentry_plan.fulfillment_mode, "crawler");
        assert_eq!(reentry_context.session_index, 2);
        assert_eq!(reentry_context.reentry_count, 1);

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn autonomous_supervisor_keeps_dispatching_through_full_scrapling_mode_cycle_before_dormancy() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");
        let store = InMemoryStore::default();
        let mut state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            desired_lane: RuntimeLane::ScraplingTraffic,
            active_lane: Some(RuntimeLane::ScraplingTraffic),
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-bursty-cycle".to_string()),
            started_at: Some(100),
            ends_at: Some(500),
            active_run_count: 1,
            active_lane_count: 1,
            ..ControlState::default()
        };

        let mut now = 1_700_000_000u64;
        let cadence_seconds = autonomous_execution_profile().cadence_seconds;
        let expected_modes = [
            "crawler",
            "bulk_scraper",
            "stealth_browser",
            "http_agent",
            "browser_automation",
        ];

        for (index, expected_mode) in expected_modes.iter().enumerate() {
            let summary = run_autonomous_supervisor_ticks(&store, &mut state, now);
            let plan = summary.worker_plan.expect("worker plan");
            assert_eq!(plan.fulfillment_mode, *expected_mode);

            let result = ScraplingWorkerResult {
                schema_version: SCRAPLING_WORKER_RESULT_SCHEMA_VERSION.to_string(),
                run_id: plan.run_id.clone(),
                tick_id: plan.tick_id.clone(),
                lane: RuntimeLane::ScraplingTraffic,
                fulfillment_mode: plan.fulfillment_mode.clone(),
                category_targets: plan.category_targets.clone(),
                worker_id: "scrapling-worker-test".to_string(),
                tick_started_at: plan.tick_started_at,
                tick_completed_at: plan.tick_started_at.saturating_add(1),
                generated_requests: 2,
                failed_requests: 0,
                last_response_status: Some(200),
                failure_class: None,
                error: None,
                crawl_stats: ScraplingCrawlStats::default(),
                scope_rejections: std::collections::BTreeMap::new(),
                realism_receipt: None,
                surface_receipts: Vec::new(),
            };
            apply_scrapling_worker_result(&mut state, &result);
            now = result.tick_completed_at.saturating_add(cadence_seconds);

            if index < expected_modes.len() - 1 {
                assert!(
                    state.recurrence_dormant_until.is_none(),
                    "expected no recurrence dormancy before completing full cycle after {expected_mode}"
                );
            }
        }

        let dormant_summary = run_autonomous_supervisor_ticks(&store, &mut state, now);
        assert!(dormant_summary.worker_plan.is_none());
        assert_eq!(
            dormant_summary.pending_dispatch_mode.as_deref(),
            Some("recurrence_dormant")
        );

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn autonomous_supervisor_reaps_stale_pending_scrapling_worker_before_redispatching() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");

        let store = InMemoryStore::default();
        let mut state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            desired_lane: RuntimeLane::ScraplingTraffic,
            active_lane: Some(RuntimeLane::ScraplingTraffic),
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-stale-scrapling".to_string()),
            started_at: Some(100),
            ends_at: Some(500),
            active_run_count: 1,
            active_lane_count: 1,
            generated_tick_count: 3,
            pending_scrapling_tick_id: Some("scrapling-tick-stale".to_string()),
            pending_scrapling_started_at: Some(100),
            last_generated_at: Some(100),
            ..ControlState::default()
        };
        let stale_at = 100u64
            .saturating_add(scrapling_pending_worker_timeout_seconds(&state))
            .saturating_add(1);

        let summary = run_autonomous_supervisor_ticks(&store, &mut state, stale_at);
        let plan = summary.worker_plan.expect("replacement worker plan");

        assert_eq!(plan.fulfillment_mode, "http_agent");
        assert_eq!(
            state.last_generation_error.as_deref(),
            Some("scrapling_worker_stale_timeout")
        );
        assert_eq!(
            state.pending_scrapling_tick_id.as_deref(),
            Some(plan.tick_id.as_str())
        );
        assert!(state.pending_scrapling_started_at.is_some());
        assert!(!summary.worker_pending);

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn autonomous_supervisor_dispatches_parallel_scrapling_and_llm_plans_for_parallel_mixed_lane() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");
        let store = InMemoryStore::default();
        let mut state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            desired_lane: RuntimeLane::ParallelMixedTraffic,
            active_lane: Some(RuntimeLane::ParallelMixedTraffic),
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-parallel-mixed".to_string()),
            started_at: Some(100),
            ends_at: Some(500),
            active_run_count: 1,
            active_lane_count: 2,
            ..ControlState::default()
        };

        let summary = run_autonomous_supervisor_ticks(&store, &mut state, 110);

        assert_eq!(summary.due_ticks, 1);
        assert!(summary.worker_plan.is_some());
        assert!(summary.llm_fulfillment_plan.is_some());
        assert_eq!(
            summary
                .worker_plan
                .as_ref()
                .map(|plan| plan.lane.as_str()),
            Some("scrapling_traffic")
        );
        assert_eq!(
            summary
                .llm_fulfillment_plan
                .as_ref()
                .map(|plan| plan.lane.as_str()),
            Some("bot_red_team")
        );
        assert!(!summary.worker_pending);
        assert_eq!(summary.pending_dispatch_mode.as_deref(), Some("parallel_mixed_workers"));
        assert!(state.pending_scrapling_tick_id.is_some());
        assert!(state.pending_llm_tick_id.is_some());

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn autonomous_supervisor_parallel_mixed_lane_waits_for_both_worker_results_before_redispatching() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");
        let store = InMemoryStore::default();
        let mut state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            desired_lane: RuntimeLane::ParallelMixedTraffic,
            active_lane: Some(RuntimeLane::ParallelMixedTraffic),
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-parallel-pending".to_string()),
            started_at: Some(100),
            ends_at: Some(500),
            active_run_count: 1,
            active_lane_count: 2,
            pending_scrapling_tick_id: Some("scrapling-tick-pending".to_string()),
            pending_scrapling_started_at: Some(109),
            pending_llm_tick_id: Some("llm-fit-tick-pending".to_string()),
            pending_llm_started_at: Some(109),
            ..ControlState::default()
        };

        let summary = run_autonomous_supervisor_ticks(&store, &mut state, 110);

        assert!(summary.worker_plan.is_none());
        assert!(summary.llm_fulfillment_plan.is_none());
        assert!(summary.worker_pending);
        assert_eq!(
            summary.pending_dispatch_mode.as_deref(),
            Some("parallel_mixed_workers_pending")
        );

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
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
    clear_recurrence_dormancy_if_ready(state, now);
    clear_stale_scrapling_pending_worker(state, now);
    match effective_active_lane(state) {
        Some(RuntimeLane::SyntheticTraffic) => {}
        Some(RuntimeLane::ScraplingTraffic) => {
            if recurrence_dormant(state, now) {
                summary.pending_dispatch_mode = Some("recurrence_dormant".to_string());
                return summary;
            }
            if lane_has_pending_worker(state, RuntimeLane::ScraplingTraffic) {
                summary.worker_pending = true;
                summary.pending_dispatch_mode = Some("scrapling_worker_pending".to_string());
                return summary;
            }
            record_lane_attempt(state, RuntimeLane::ScraplingTraffic);
            summary.worker_plan = Some(next_scrapling_worker_plan(now, state));
            return summary;
        }
        Some(RuntimeLane::BotRedTeam) => {
            if recurrence_dormant(state, now) {
                summary.pending_dispatch_mode = Some("recurrence_dormant".to_string());
                return summary;
            }
            if lane_has_pending_worker(state, RuntimeLane::BotRedTeam) {
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
            set_lane_pending_worker(state, RuntimeLane::BotRedTeam, plan.tick_id.clone(), plan.tick_started_at);
            state.updated_at = now;
            summary.llm_fulfillment_plan = Some(plan);
            return summary;
        }
        Some(RuntimeLane::ParallelMixedTraffic) => {
            if lane_has_pending_worker(state, RuntimeLane::ParallelMixedTraffic) {
                summary.worker_pending = true;
                summary.pending_dispatch_mode = Some("parallel_mixed_workers_pending".to_string());
                return summary;
            }
            record_lane_attempt(state, RuntimeLane::ScraplingTraffic);
            record_lane_attempt(state, RuntimeLane::BotRedTeam);
            summary.worker_plan = Some(next_scrapling_worker_plan(now, state));
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
            set_lane_pending_worker(state, RuntimeLane::BotRedTeam, plan.tick_id.clone(), plan.tick_started_at);
            state.updated_at = now;
            summary.pending_dispatch_mode = Some("parallel_mixed_workers".to_string());
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
        let trusted_ingress_backed = forwarded_secret.as_deref().is_some();
        let selected_supplemental_lanes =
            supplemental_lanes_for_profile(deployment_profile, state.generated_tick_count);
        let includes_lane = |lane: SupplementalLane| selected_supplemental_lanes.contains(&lane);

        let mut dispatch_request =
            |request: Request,
             observation: crate::admin::adversary_sim_corpus::SyntheticRuntimeObservation| {
            let _guard = crate::runtime::sim_telemetry::enter(Some(metadata.clone()));
            let _synthetic_observation_guard =
                crate::runtime::sim_telemetry::enter_synthetic_runtime_observation(Some(
                    observation,
                ));
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
            let observed_country_codes: &[&str] =
                if crate::http_route_namespace::is_generated_public_site_path(path) {
                    &["RU"]
                } else {
                    &[]
                };
            dispatch_request(
                builder.body(Vec::new()).build(),
                build_synthetic_runtime_observation(
                    "public_probe",
                    trusted_ingress_backed,
                    observed_country_codes,
                ),
            );
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
            dispatch_request(
                challenge_submit.body(challenge_abuse_body).build(),
                build_synthetic_runtime_observation(
                    "challenge_submit",
                    trusted_ingress_backed,
                    &[],
                ),
            );
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
                dispatch_request(
                    not_a_bot_fail_submit.body(fail_body).build(),
                    build_synthetic_runtime_observation(
                        "not_a_bot_fail",
                        trusted_ingress_backed,
                        &[],
                    ),
                );
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
                dispatch_request(
                    not_a_bot_escalate_submit.body(escalate_body).build(),
                    build_synthetic_runtime_observation(
                        "not_a_bot_escalate",
                        trusted_ingress_backed,
                        &[],
                    ),
                );
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
            dispatch_request(
                pow_verify.body(pow_verify_body).build(),
                build_synthetic_runtime_observation("pow_verify", trusted_ingress_backed, &[]),
            );
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
            dispatch_request(
                tarpit_progress.body(tarpit_progress_body).build(),
                build_synthetic_runtime_observation(
                    "tarpit_progress",
                    trusted_ingress_backed,
                    &[],
                ),
            );
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
            dispatch_request(
                fingerprint_probe.body(Vec::new()).build(),
                build_synthetic_runtime_observation(
                    "fingerprint_probe",
                    trusted_ingress_backed,
                    &[],
                ),
            );
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
            dispatch_request(
                cdp_builder.body(cdp_probe_body).build(),
                build_synthetic_runtime_observation("cdp_report", trusted_ingress_backed, &[]),
            );
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
            dispatch_request(
                burst_builder.body(Vec::new()).build(),
                build_synthetic_runtime_observation("rate_burst", trusted_ingress_backed, &[]),
            );
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
