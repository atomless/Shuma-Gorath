use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::json;
use once_cell::sync::Lazy;
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

pub const MAX_CONCURRENT_RUNS: u32 = 1;
pub const MAX_CPU_MILLICORES: u32 = 1000;
pub const MAX_MEMORY_MIB: u32 = 512;
pub const QUEUE_POLICY: &str = "reject_new";
pub const STOP_TIMEOUT_SECONDS: u64 = 10;
pub const AUTONOMOUS_HEARTBEAT_INTERVAL_SECONDS: u64 = 1;
pub const AUTONOMOUS_HEARTBEAT_MAX_CATCHUP_TICKS_PER_INVOCATION: u64 = 2;
const DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION: &str = "sim-deterministic-attack-corpus.v1";
const DETERMINISTIC_ATTACK_CORPUS_PATH: &str =
    "scripts/tests/adversarial/deterministic_attack_corpus.v1.json";
const GENERATION_DIAGNOSTIC_GRACE_SECONDS: u64 = 5;
const STATE_KEY_PREFIX: &str = "adversary_sim:control:";
static PROCESS_INSTANCE_ID: Lazy<String> = Lazy::new(|| {
    std::env::var("RUNTIME_INSTANCE_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime-instance-unknown".to_string())
});
static DETERMINISTIC_ATTACK_CORPUS: Lazy<DeterministicAttackCorpus> =
    Lazy::new(load_deterministic_attack_corpus);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct DeterministicAttackCorpus {
    schema_version: String,
    corpus_revision: String,
    taxonomy_version: String,
    runtime_profile: String,
    ci_profile: String,
    runtime_toggle: RuntimeDeterministicProfile,
    ci_oracle: CiOracleDeterministicProfile,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RuntimeDeterministicProfile {
    active_lane_count: u32,
    primary_request_count: u64,
    supplemental_request_count: u64,
    primary_public_paths: Vec<String>,
    honeypot_probe_moduli: Vec<u64>,
    rate_burst: RateBurstProfile,
    lane_ip_octets: LaneIpOctets,
    lane_ip_rotation_ticks: LaneIpRotationTicks,
    lane_ip_entropy_salts: LaneIpEntropySalts,
    metadata: RuntimeMetadataProfile,
    paths: RuntimePathProfile,
    taxonomy: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RateBurstProfile {
    low: u64,
    medium: u64,
    high: u64,
    high_modulus: u64,
    medium_modulus: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct LaneIpOctets {
    rate_burst: u8,
    fingerprint_probe: u8,
    challenge_abuse: u8,
    pow_abuse: u8,
    tarpit_abuse: u8,
    cdp_report: u8,
    not_a_bot_fail: u8,
    not_a_bot_escalate: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct LaneIpRotationTicks {
    rate_burst: u64,
    fingerprint_probe: u64,
    challenge_abuse: u64,
    pow_abuse: u64,
    tarpit_abuse: u64,
    cdp_report: u64,
    not_a_bot_fail: u64,
    not_a_bot_escalate: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct LaneIpEntropySalts {
    rate_burst: u64,
    fingerprint_probe: u64,
    challenge_abuse: u64,
    pow_abuse: u64,
    tarpit_abuse: u64,
    cdp_report: u64,
    not_a_bot_fail: u64,
    not_a_bot_escalate: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RuntimeMetadataProfile {
    sim_profile: String,
    sim_lane: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RuntimePathProfile {
    public_search: String,
    pow: String,
    not_a_bot_checkbox: String,
    honeypot: String,
    challenge_submit: String,
    pow_verify: String,
    cdp_report: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct CiOracleDeterministicProfile {
    drivers: BTreeMap<String, CiDriverDefinition>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct CiDriverDefinition {
    driver_class: String,
    path_hint: String,
    taxonomy_category: String,
}

fn default_deterministic_attack_corpus() -> DeterministicAttackCorpus {
    let mut ci_drivers = BTreeMap::new();
    for (driver, driver_class, path_hint, taxonomy_category) in [
        (
            "allow_browser_allowlist",
            "browser_realistic",
            "/sim/public/landing",
            "allowlist",
        ),
        (
            "not_a_bot_pass",
            "browser_realistic",
            "/challenge/not-a-bot-checkbox",
            "not_a_bot",
        ),
        (
            "challenge_puzzle_fail_maze",
            "browser_realistic",
            "/challenge/puzzle",
            "challenge",
        ),
        ("pow_success", "cost_imposition", "/pow", "pow"),
        ("pow_invalid_proof", "cost_imposition", "/pow/verify", "pow"),
        (
            "rate_limit_enforce",
            "http_scraper",
            "/sim/public/search",
            "rate",
        ),
        (
            "retry_storm_enforce",
            "http_scraper",
            "/sim/public/search",
            "rate",
        ),
        ("geo_challenge", "browser_realistic", "/sim/public/docs", "geo"),
        ("geo_maze", "browser_realistic", "/sim/public/pricing", "geo"),
        ("geo_block", "browser_realistic", "/sim/public/contact", "geo"),
        ("honeypot_deny_temp", "browser_realistic", "/instaban", "honeypot"),
        (
            "not_a_bot_replay_abuse",
            "http_scraper",
            "/challenge/not-a-bot-checkbox",
            "not_a_bot",
        ),
        (
            "not_a_bot_stale_token_abuse",
            "http_scraper",
            "/challenge/not-a-bot-checkbox",
            "not_a_bot",
        ),
        (
            "not_a_bot_ordering_cadence_abuse",
            "http_scraper",
            "/challenge/not-a-bot-checkbox",
            "not_a_bot",
        ),
        (
            "not_a_bot_replay_tarpit_abuse",
            "http_scraper",
            "/challenge/not-a-bot-checkbox",
            "tarpit",
        ),
        (
            "fingerprint_inconsistent_payload",
            "http_scraper",
            "/fingerprint-report",
            "fingerprint",
        ),
        (
            "header_spoofing_probe",
            "browser_realistic",
            "/sim/public/search",
            "headers",
        ),
        ("cdp_high_confidence_deny", "http_scraper", "/cdp-report", "cdp"),
        (
            "akamai_additive_report",
            "edge_fixture",
            "/fingerprint-report",
            "akamai",
        ),
        (
            "akamai_authoritative_deny",
            "edge_fixture",
            "/fingerprint-report",
            "akamai",
        ),
    ] {
        ci_drivers.insert(
            driver.to_string(),
            CiDriverDefinition {
                driver_class: driver_class.to_string(),
                path_hint: path_hint.to_string(),
                taxonomy_category: taxonomy_category.to_string(),
            },
        );
    }

    let taxonomy = BTreeMap::from([
        ("public_probe".to_string(), "crawl_probe".to_string()),
        ("challenge_submit".to_string(), "challenge_abuse".to_string()),
        ("not_a_bot_fail".to_string(), "not_a_bot_fail".to_string()),
        (
            "not_a_bot_escalate".to_string(),
            "not_a_bot_escalate".to_string(),
        ),
        ("pow_verify".to_string(), "pow_abuse".to_string()),
        ("tarpit_progress".to_string(), "tarpit_abuse".to_string()),
        ("fingerprint_probe".to_string(), "fingerprint_probe".to_string()),
        ("cdp_report".to_string(), "cdp_probe".to_string()),
        ("rate_burst".to_string(), "rate_burst".to_string()),
    ]);

    DeterministicAttackCorpus {
        schema_version: DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION.to_string(),
        corpus_revision: "default-fallback".to_string(),
        taxonomy_version: "sim-policy-taxonomy.v1".to_string(),
        runtime_profile: "runtime_toggle".to_string(),
        ci_profile: "ci_oracle".to_string(),
        runtime_toggle: RuntimeDeterministicProfile {
            active_lane_count: 2,
            primary_request_count: 9,
            supplemental_request_count: 7,
            primary_public_paths: vec![
                "/sim/public/landing".to_string(),
                "/sim/public/docs".to_string(),
                "/sim/public/pricing".to_string(),
                "/sim/public/contact".to_string(),
                "/sim/public/changelog".to_string(),
                "/sim/public/faq".to_string(),
            ],
            honeypot_probe_moduli: vec![5, 7],
            rate_burst: RateBurstProfile {
                low: 8,
                medium: 16,
                high: 24,
                high_modulus: 9,
                medium_modulus: 3,
            },
            lane_ip_octets: LaneIpOctets {
                rate_burst: 248,
                fingerprint_probe: 249,
                challenge_abuse: 250,
                pow_abuse: 251,
                tarpit_abuse: 252,
                cdp_report: 253,
                not_a_bot_fail: 246,
                not_a_bot_escalate: 247,
            },
            lane_ip_rotation_ticks: LaneIpRotationTicks {
                rate_burst: 24,
                fingerprint_probe: 2,
                challenge_abuse: 1,
                pow_abuse: 1,
                tarpit_abuse: 1,
                cdp_report: 2,
                not_a_bot_fail: 2,
                not_a_bot_escalate: 2,
            },
            lane_ip_entropy_salts: LaneIpEntropySalts {
                rate_burst: 79,
                fingerprint_probe: 53,
                challenge_abuse: 17,
                pow_abuse: 29,
                tarpit_abuse: 41,
                cdp_report: 67,
                not_a_bot_fail: 97,
                not_a_bot_escalate: 113,
            },
            metadata: RuntimeMetadataProfile {
                sim_profile: "runtime_toggle".to_string(),
                sim_lane: "deterministic_black_box".to_string(),
            },
            paths: RuntimePathProfile {
                public_search: "/sim/public/search".to_string(),
                pow: "/pow".to_string(),
                not_a_bot_checkbox: "/challenge/not-a-bot-checkbox".to_string(),
                honeypot: "/instaban".to_string(),
                challenge_submit: "/challenge/puzzle".to_string(),
                pow_verify: "/pow/verify".to_string(),
                cdp_report: "/cdp-report".to_string(),
            },
            taxonomy,
        },
        ci_oracle: CiOracleDeterministicProfile { drivers: ci_drivers },
    }
}

fn load_deterministic_attack_corpus() -> DeterministicAttackCorpus {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/scripts/tests/adversarial/deterministic_attack_corpus.v1.json"
    ));
    let parsed = serde_json::from_str::<DeterministicAttackCorpus>(raw)
        .ok()
        .filter(|corpus| corpus.schema_version == DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION)
        .filter(|corpus| !corpus.corpus_revision.trim().is_empty())
        .filter(|corpus| !corpus.taxonomy_version.trim().is_empty())
        .filter(|corpus| !corpus.runtime_toggle.primary_public_paths.is_empty())
        .filter(|corpus| {
            corpus.runtime_toggle.primary_request_count
                == corpus.runtime_toggle.primary_public_paths.len() as u64 + 3
        })
        .filter(|corpus| !corpus.runtime_toggle.honeypot_probe_moduli.is_empty())
        .filter(|corpus| !corpus.ci_oracle.drivers.is_empty());
    parsed.unwrap_or_else(default_deterministic_attack_corpus)
}

fn deterministic_runtime_profile() -> &'static RuntimeDeterministicProfile {
    &DETERMINISTIC_ATTACK_CORPUS.runtime_toggle
}

fn deterministic_corpus_metadata_payload() -> serde_json::Value {
    json!({
        "schema_version": DETERMINISTIC_ATTACK_CORPUS.schema_version.clone(),
        "corpus_revision": DETERMINISTIC_ATTACK_CORPUS.corpus_revision.clone(),
        "taxonomy_version": DETERMINISTIC_ATTACK_CORPUS.taxonomy_version.clone(),
        "contract_path": DETERMINISTIC_ATTACK_CORPUS_PATH,
        "runtime_profile": DETERMINISTIC_ATTACK_CORPUS.runtime_profile.clone(),
        "ci_profile": DETERMINISTIC_ATTACK_CORPUS.ci_profile.clone(),
        "ci_driver_count": DETERMINISTIC_ATTACK_CORPUS.ci_oracle.drivers.len()
    })
}

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
    pub desired_enabled: bool,
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
            desired_enabled: false,
            owner_instance_id: None,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AutonomousHeartbeatTickSummary {
    pub due_ticks: u64,
    pub executed_ticks: u64,
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
    _runtime_environment: crate::config::RuntimeEnvironment,
    env_available: bool,
) -> bool {
    env_available
}

pub fn process_instance_id() -> &'static str {
    PROCESS_INSTANCE_ID.as_str()
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
        desired_enabled: true,
        owner_instance_id: Some(process_instance_id().to_string()),
        run_id: Some(run_id),
        started_at: Some(now),
        ends_at: Some(now.saturating_add(clamp_duration_seconds(duration_seconds))),
        stop_deadline: None,
        active_run_count: 1,
        active_lane_count: deterministic_runtime_profile().active_lane_count,
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
    next.desired_enabled = false;
    next.owner_instance_id = Some(process_instance_id().to_string());
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
    next.desired_enabled = cfg_enabled;
    let mut transitions: Vec<Transition> = Vec::new();

    if next.phase != ControlPhase::Off
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

fn rate_burst_requests_for_tick(tick_count: u64) -> u64 {
    let burst = &deterministic_runtime_profile().rate_burst;
    if burst.high_modulus > 0 && tick_count % burst.high_modulus == 0 {
        burst.high
    } else if burst.medium_modulus > 0 && tick_count % burst.medium_modulus == 0 {
        burst.medium
    } else {
        burst.low
    }
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
    std::env::var("SHUMA_CHALLENGE_SECRET")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("SHUMA_JS_SECRET")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
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
    deterministic_runtime_profile().primary_request_count
        + deterministic_runtime_profile().supplemental_request_count
        + rate_burst_requests_for_tick(tick_count)
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
            reason = "supervisor_no_traffic_yet".to_string();
            recommended_action =
                "Verify backend supervisor heartbeat diagnostics and confirm simulation remains running.".to_string();
        } else if let Some(last_generated_at) = state.last_generated_at {
            if now >= last_generated_at.saturating_add(GENERATION_DIAGNOSTIC_GRACE_SECONDS) {
                health = "stalled".to_string();
                reason = "supervisor_tick_stalled".to_string();
                recommended_action =
                    "Check backend supervisor heartbeat state and re-enable adversary simulation if needed.".to_string();
            } else {
                health = "ok".to_string();
                reason = "traffic_observed".to_string();
                recommended_action =
                    "No action required; simulation traffic is being generated.".to_string();
            }
        } else {
            health = "warming".to_string();
            reason = "waiting_for_first_supervisor_tick".to_string();
            recommended_action = "Allow one heartbeat interval for first generated traffic.".to_string();
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
    let heartbeat_active = cfg_enabled && state.phase == ControlPhase::Running;
    let off_state_inert = !cfg_enabled
        && state.phase == ControlPhase::Off
        && state.active_run_count == 0
        && state.active_lane_count == 0;
    let idle_seconds = state
        .last_generated_at
        .map(|last_generated_at| now.saturating_sub(last_generated_at));
    json!({
        "owner": "backend_autonomous_supervisor",
        "cadence_seconds": AUTONOMOUS_HEARTBEAT_INTERVAL_SECONDS,
        "max_catchup_ticks_per_invocation": AUTONOMOUS_HEARTBEAT_MAX_CATCHUP_TICKS_PER_INVOCATION,
        "heartbeat_active": heartbeat_active,
        "worker_active": heartbeat_active,
        "last_heartbeat_at": state.last_generated_at,
        "idle_seconds": idle_seconds,
        "off_state_inert": off_state_inert,
        "trigger_surface": "internal_beat_endpoint",
        "beat_endpoint": "/internal/adversary-sim/beat",
        "deterministic_attack_corpus": deterministic_corpus_metadata_payload()
    })
}

fn autonomous_heartbeat_due_ticks(now: u64, state: &ControlState) -> u64 {
    if state.phase != ControlPhase::Running {
        return 0;
    }
    let due = match state.last_generated_at {
        None => 1,
        Some(last_generated_at) => {
            let elapsed_seconds = now.saturating_sub(last_generated_at);
            if elapsed_seconds < AUTONOMOUS_HEARTBEAT_INTERVAL_SECONDS {
                0
            } else {
                elapsed_seconds / AUTONOMOUS_HEARTBEAT_INTERVAL_SECONDS
            }
        }
    };
    due.min(AUTONOMOUS_HEARTBEAT_MAX_CATCHUP_TICKS_PER_INVOCATION)
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
    for tick_index in 0..due_ticks {
        let tick_now = now.saturating_sub(due_ticks.saturating_sub(tick_index).saturating_sub(1));
        let tick_result = run_internal_generation_tick(store, state, tick_now);
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
        let forwarded_secret = std::env::var("SHUMA_FORWARDED_IP_SECRET")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

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
        for (index, path) in paths.iter().enumerate() {
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

        let challenge_abuse_body = b"answer=bad&seed=invalid&return_to=%2Fsim%2Fpublic%2Flanding".to_vec();
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

        let tarpit_progress_body = br#"{"token":"invalid","operation_id":"invalid","proof_nonce":"invalid"}"#.to_vec();
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
            AUTONOMOUS_HEARTBEAT_MAX_CATCHUP_TICKS_PER_INVOCATION
        );
        assert_eq!(
            state.generated_tick_count,
            AUTONOMOUS_HEARTBEAT_MAX_CATCHUP_TICKS_PER_INVOCATION
        );
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
    fn deterministic_generated_request_target_matches_batch_contract() {
        let runtime_profile = deterministic_runtime_profile();
        let burst = &runtime_profile.rate_burst;
        assert_eq!(
            deterministic_generated_request_target_for_tick(0),
            runtime_profile.primary_request_count
                + runtime_profile.supplemental_request_count
                + burst.high
        );
        assert_eq!(
            deterministic_generated_request_target_for_tick(1),
            runtime_profile.primary_request_count
                + runtime_profile.supplemental_request_count
                + burst.low
        );
        assert_eq!(
            deterministic_generated_request_target_for_tick(3),
            runtime_profile.primary_request_count
                + runtime_profile.supplemental_request_count
                + burst.medium
        );
    }
}
