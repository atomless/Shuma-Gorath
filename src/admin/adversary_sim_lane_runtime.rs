#[cfg(not(test))]
use serde_json::json;

#[cfg(not(test))]
use base64::{engine::general_purpose, Engine as _};
#[cfg(not(test))]
use hmac::{Hmac, Mac};
#[cfg(not(test))]
use sha2::Sha256;

use super::adversary_sim_corpus::deterministic_runtime_profile;

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
