use base64::{engine::general_purpose, Engine as _};
#[cfg(not(test))]
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

const MONITORING_PREFIX: &str = "monitoring:v1";
const MAX_WINDOW_HOURS: u64 = 24 * 30;
const MAX_TOP_LIMIT: usize = 50;
#[cfg(not(test))]
const COUNTER_FLUSH_INTERVAL_SECONDS: u64 = 2;
#[cfg(not(test))]
const COUNTER_FLUSH_PENDING_KEYS_MAX: usize = 64;
const TELEMETRY_PATH_SEGMENT_LIMIT: usize = 3;
const TELEMETRY_PATH_SEGMENT_MAX_LEN: usize = 24;
const TELEMETRY_PATH_FALLBACK_SEGMENT: &str = ":id";
const TELEMETRY_PATH_TRUNCATED_SUFFIX: &str = "*";

const CHALLENGE_REASON_KEYS: [&str; 5] = [
    "incorrect",
    "expired_replay",
    "sequence_violation",
    "invalid_output",
    "forbidden",
];
const POW_REASON_KEYS: [&str; 5] = [
    "invalid_proof",
    "missing_seed_nonce",
    "sequence_violation",
    "expired_replay",
    "binding_timing_mismatch",
];
const POW_OUTCOME_KEYS: [&str; 2] = ["success", "failure"];
const NOT_A_BOT_OUTCOME_KEYS: [&str; 4] = ["pass", "escalate", "fail", "replay"];
const NOT_A_BOT_SOLVE_MS_BUCKET_KEYS: [&str; 4] = ["lt_1s", "1_3s", "3_10s", "10s_plus"];
const RATE_OUTCOME_KEYS: [&str; 4] = ["limited", "banned", "fallback_allow", "fallback_deny"];
const GEO_ACTION_KEYS: [&str; 3] = ["block", "challenge", "maze"];
const SHADOW_ACTION_KEYS: [&str; 8] = [
    "not_a_bot",
    "challenge",
    "js_challenge",
    "maze",
    "block",
    "tarpit",
    "redirect",
    "drop_connection",
];
const GUARDED_DIMENSION_CARDINALITY_CAP_PER_HOUR: u64 = 1000;
const GUARDED_DIMENSION_OVERFLOW_VALUE: &str = "other";
const UNSAMPLEABLE_SECURITY_EVENT_CLASSES: [&str; 8] = [
    "honeypot",
    "challenge",
    "pow",
    "rate",
    "geo",
    "not_a_bot",
    "cdp",
    "ban",
];
const GUARDED_DIMENSION_PAIRS: [(&str, &str); 6] = [
    ("honeypot", "ip"),
    ("honeypot", "path"),
    ("challenge", "ip"),
    ("pow", "ip"),
    ("rate", "ip"),
    ("rate", "path"),
];

pub(crate) fn guarded_dimension_cardinality_cap_per_hour() -> u64 {
    GUARDED_DIMENSION_CARDINALITY_CAP_PER_HOUR
}

pub(crate) fn unsampleable_security_event_classes() -> &'static [&'static str] {
    &UNSAMPLEABLE_SECURITY_EVENT_CLASSES
}

#[cfg(not(test))]
static PENDING_COUNTER_BUFFER: Lazy<Mutex<PendingCounterBuffer>> =
    Lazy::new(|| Mutex::new(PendingCounterBuffer::default()));

#[cfg(not(test))]
#[derive(Default)]
struct PendingCounterBuffer {
    last_flush_ts: u64,
    deltas: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct CountEntry {
    pub label: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct TrendPoint {
    pub ts: u64,
    pub total: u64,
    pub reasons: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct HoneypotSummary {
    pub total_hits: u64,
    pub unique_crawlers: u64,
    pub top_crawlers: Vec<CountEntry>,
    pub top_paths: Vec<CountEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FailureSummary {
    pub total_failures: u64,
    pub unique_offenders: u64,
    pub top_offenders: Vec<CountEntry>,
    pub reasons: BTreeMap<String, u64>,
    pub trend: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PowSummary {
    pub total_failures: u64,
    pub total_successes: u64,
    pub total_attempts: u64,
    pub success_ratio: f64,
    pub unique_offenders: u64,
    pub top_offenders: Vec<CountEntry>,
    pub reasons: BTreeMap<String, u64>,
    pub outcomes: BTreeMap<String, u64>,
    pub trend: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct RateSummary {
    pub total_violations: u64,
    pub unique_offenders: u64,
    pub top_offenders: Vec<CountEntry>,
    pub top_paths: Vec<CountEntry>,
    pub outcomes: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct GeoSummary {
    pub total_violations: u64,
    pub actions: BTreeMap<String, u64>,
    pub top_countries: Vec<CountEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ShadowSummary {
    pub total_actions: u64,
    pub pass_through_total: u64,
    pub actions: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct NotABotSummary {
    pub served: u64,
    pub submitted: u64,
    pub pass: u64,
    pub escalate: u64,
    pub fail: u64,
    pub replay: u64,
    pub outcomes: BTreeMap<String, u64>,
    pub solve_latency_buckets: BTreeMap<String, u64>,
    pub abandonments_estimated: u64,
    pub abandonment_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct RequestOutcomeScopeSummaryRow {
    pub traffic_origin: String,
    pub measurement_scope: String,
    pub execution_mode: String,
    pub total_requests: u64,
    pub forwarded_requests: u64,
    pub short_circuited_requests: u64,
    pub control_response_requests: u64,
    pub response_bytes: u64,
    pub forwarded_response_bytes: u64,
    pub short_circuited_response_bytes: u64,
    pub control_response_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct RequestOutcomeLaneSummaryRow {
    pub traffic_origin: String,
    pub measurement_scope: String,
    pub execution_mode: String,
    pub lane: String,
    pub exactness: String,
    pub basis: String,
    pub total_requests: u64,
    pub forwarded_requests: u64,
    pub short_circuited_requests: u64,
    pub control_response_requests: u64,
    pub response_bytes: u64,
    pub forwarded_response_bytes: u64,
    pub short_circuited_response_bytes: u64,
    pub control_response_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct RequestOutcomeBreakdownSummaryRow {
    pub traffic_origin: String,
    pub measurement_scope: String,
    pub execution_mode: String,
    pub value: String,
    pub total_requests: u64,
    pub forwarded_requests: u64,
    pub short_circuited_requests: u64,
    pub control_response_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct HumanFrictionSegmentRow {
    pub execution_mode: String,
    pub segment: String,
    pub denominator_requests: u64,
    pub not_a_bot_requests: u64,
    pub challenge_requests: u64,
    pub js_challenge_requests: u64,
    pub maze_requests: u64,
    pub friction_requests: u64,
    pub not_a_bot_rate: f64,
    pub challenge_rate: f64,
    pub js_challenge_rate: f64,
    pub maze_rate: f64,
    pub friction_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct HumanFrictionSummary {
    pub segments: Vec<HumanFrictionSegmentRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct DefenceFunnelRow {
    pub execution_mode: String,
    pub family: String,
    pub candidate_requests: Option<u64>,
    pub triggered_requests: Option<u64>,
    pub friction_requests: Option<u64>,
    pub passed_requests: Option<u64>,
    pub failed_requests: Option<u64>,
    pub escalated_requests: Option<u64>,
    pub denied_requests: Option<u64>,
    pub suspicious_forwarded_requests: Option<u64>,
    pub likely_human_affected_requests: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct DefenceFunnelSummary {
    pub rows: Vec<DefenceFunnelRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct RequestOutcomeSummary {
    pub by_scope: Vec<RequestOutcomeScopeSummaryRow>,
    pub by_lane: Vec<RequestOutcomeLaneSummaryRow>,
    pub by_response_kind: Vec<RequestOutcomeBreakdownSummaryRow>,
    pub by_policy_source: Vec<RequestOutcomeBreakdownSummaryRow>,
    pub by_route_action_family: Vec<RequestOutcomeBreakdownSummaryRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct MonitoringSummary {
    pub generated_at: u64,
    pub hours: u64,
    pub shadow: ShadowSummary,
    pub honeypot: HoneypotSummary,
    pub challenge: FailureSummary,
    pub not_a_bot: NotABotSummary,
    pub pow: PowSummary,
    pub rate: RateSummary,
    pub geo: GeoSummary,
    pub human_friction: HumanFrictionSummary,
    pub defence_funnel: DefenceFunnelSummary,
    pub request_outcomes: RequestOutcomeSummary,
}

fn now_ts() -> u64 {
    crate::admin::now_ts()
}

fn normalize_window_hours(hours: u64) -> u64 {
    hours.clamp(1, MAX_WINDOW_HOURS)
}

fn normalize_top_limit(limit: usize) -> usize {
    limit.clamp(1, MAX_TOP_LIMIT)
}

fn monitoring_retention_hours() -> u64 {
    crate::config::monitoring_retention_hours()
}

fn monitoring_prefix_for_active_context() -> &'static str {
    MONITORING_PREFIX
}

fn encode_dim(value: &str) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(value.as_bytes())
}

fn decode_dim(value: &str) -> String {
    general_purpose::URL_SAFE_NO_PAD
        .decode(value.as_bytes())
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_else(|| value.to_string())
}

fn normalize_telemetry_segment(segment: &str) -> String {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return TELEMETRY_PATH_FALLBACK_SEGMENT.to_string();
    }
    let alpha_count = trimmed.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
    let digit_count = trimmed.chars().filter(|ch| ch.is_ascii_digit()).count();
    let ascii_word_like = trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    let looks_dynamic = trimmed.len() > TELEMETRY_PATH_SEGMENT_MAX_LEN
        || trimmed.chars().all(|ch| ch.is_ascii_digit())
        || (trimmed.len() >= 8 && trimmed.chars().all(|ch| ch.is_ascii_hexdigit()))
        || (ascii_word_like
            && trimmed.len() >= 12
            && alpha_count > 0
            && digit_count > 0
            && digit_count.saturating_mul(2) >= trimmed.len());
    if looks_dynamic {
        return TELEMETRY_PATH_FALLBACK_SEGMENT.to_string();
    }
    let mut normalized = trimmed.to_ascii_lowercase();
    if normalized.len() > TELEMETRY_PATH_SEGMENT_MAX_LEN {
        normalized.truncate(TELEMETRY_PATH_SEGMENT_MAX_LEN);
    }
    normalized
}

fn normalize_telemetry_path(path: &str) -> String {
    let raw = path
        .split('?')
        .next()
        .unwrap_or(path)
        .split('#')
        .next()
        .unwrap_or(path)
        .trim();
    if raw.is_empty() {
        return "/".to_string();
    }

    let mut segments = Vec::new();
    for segment in raw.split('/').filter(|value| !value.trim().is_empty()) {
        if segments.len() >= TELEMETRY_PATH_SEGMENT_LIMIT {
            break;
        }
        segments.push(normalize_telemetry_segment(segment));
    }

    let has_extra_segments = raw
        .split('/')
        .filter(|value| !value.trim().is_empty())
        .count()
        > TELEMETRY_PATH_SEGMENT_LIMIT;
    if has_extra_segments {
        segments.push(TELEMETRY_PATH_TRUNCATED_SUFFIX.to_string());
    }

    if segments.is_empty() {
        return "/".to_string();
    }
    let mut normalized = format!("/{}", segments.join("/"));
    if normalized.len() > 120 {
        normalized.truncate(120);
    }
    normalized
}

fn normalize_challenge_reason(reason: &str) -> &'static str {
    match reason {
        "incorrect" => "incorrect",
        "expired_replay" => "expired_replay",
        "sequence_violation" => "sequence_violation",
        "invalid_output" => "invalid_output",
        "forbidden" => "forbidden",
        _ => "forbidden",
    }
}

fn normalize_pow_reason(reason: &str) -> &'static str {
    match reason {
        "invalid_proof" => "invalid_proof",
        "missing_seed_nonce" => "missing_seed_nonce",
        "sequence_violation" => "sequence_violation",
        "expired_replay" => "expired_replay",
        "binding_timing_mismatch" => "binding_timing_mismatch",
        _ => "sequence_violation",
    }
}

fn normalize_rate_outcome(outcome: &str) -> &'static str {
    match outcome {
        "limited" => "limited",
        "banned" => "banned",
        "fallback_allow" => "fallback_allow",
        "fallback_deny" => "fallback_deny",
        _ => "limited",
    }
}

fn normalize_geo_action(action: &str) -> &'static str {
    match action {
        "block" => "block",
        "challenge" => "challenge",
        "maze" => "maze",
        _ => "block",
    }
}

fn normalize_not_a_bot_outcome(outcome: &str) -> &'static str {
    match outcome {
        "pass" => "pass",
        "escalate" => "escalate",
        "replay" => "replay",
        "fail" => "fail",
        _ => "fail",
    }
}

fn normalize_shadow_action(
    action: crate::runtime::effect_intents::ShadowAction,
) -> &'static str {
    action.as_str()
}

fn normalize_execution_mode(
    mode: crate::runtime::effect_intents::ExecutionMode,
) -> &'static str {
    match mode {
        crate::runtime::effect_intents::ExecutionMode::Enforced => "enforced",
        crate::runtime::effect_intents::ExecutionMode::Shadow => "shadow",
    }
}

fn normalize_traffic_origin(origin: crate::runtime::request_outcome::TrafficOrigin) -> &'static str {
    match origin {
        crate::runtime::request_outcome::TrafficOrigin::Live => "live",
        crate::runtime::request_outcome::TrafficOrigin::AdversarySim => "adversary_sim",
    }
}

fn normalize_measurement_scope(
    scope: crate::runtime::traffic_classification::MeasurementScope,
) -> &'static str {
    match scope {
        crate::runtime::traffic_classification::MeasurementScope::IngressPrimary => {
            "ingress_primary"
        }
        crate::runtime::traffic_classification::MeasurementScope::DefenceFollowup => {
            "defence_followup"
        }
        crate::runtime::traffic_classification::MeasurementScope::BypassAndControl => {
            "bypass_and_control"
        }
        crate::runtime::traffic_classification::MeasurementScope::Excluded => "excluded",
    }
}

fn normalize_route_action_family(
    family: crate::runtime::traffic_classification::RouteActionFamily,
) -> &'static str {
    match family {
        crate::runtime::traffic_classification::RouteActionFamily::PublicContent => {
            "public_content"
        }
        crate::runtime::traffic_classification::RouteActionFamily::StaticAsset => "static_asset",
        crate::runtime::traffic_classification::RouteActionFamily::DefenceFollowup => {
            "defence_followup"
        }
        crate::runtime::traffic_classification::RouteActionFamily::AllowlistBypass => {
            "allowlist_bypass"
        }
        crate::runtime::traffic_classification::RouteActionFamily::ControlPlane => "control_plane",
        crate::runtime::traffic_classification::RouteActionFamily::SimPublic => "sim_public",
    }
}

fn normalize_traffic_lane(lane: crate::runtime::traffic_classification::TrafficLane) -> &'static str {
    match lane {
        crate::runtime::traffic_classification::TrafficLane::LikelyHuman => "likely_human",
        crate::runtime::traffic_classification::TrafficLane::UnknownInteractive => {
            "unknown_interactive"
        }
        crate::runtime::traffic_classification::TrafficLane::SuspiciousAutomation => {
            "suspicious_automation"
        }
        crate::runtime::traffic_classification::TrafficLane::DeclaredCrawler => {
            "declared_crawler"
        }
        crate::runtime::traffic_classification::TrafficLane::DeclaredUserTriggeredAgent => {
            "declared_user_triggered_agent"
        }
        crate::runtime::traffic_classification::TrafficLane::VerifiedBot => "verified_bot",
        crate::runtime::traffic_classification::TrafficLane::SignedAgent => "signed_agent",
    }
}

fn normalize_telemetry_exactness(
    exactness: crate::observability::hot_read_contract::TelemetryExactness,
) -> &'static str {
    match exactness {
        crate::observability::hot_read_contract::TelemetryExactness::Exact => "exact",
        crate::observability::hot_read_contract::TelemetryExactness::Derived => "derived",
        crate::observability::hot_read_contract::TelemetryExactness::BestEffort => "best_effort",
    }
}

fn normalize_telemetry_basis(
    basis: crate::observability::hot_read_contract::TelemetryBasis,
) -> &'static str {
    match basis {
        crate::observability::hot_read_contract::TelemetryBasis::Observed => "observed",
        crate::observability::hot_read_contract::TelemetryBasis::Policy => "policy",
        crate::observability::hot_read_contract::TelemetryBasis::Verified => "verified",
        crate::observability::hot_read_contract::TelemetryBasis::Residual => "residual",
        crate::observability::hot_read_contract::TelemetryBasis::Mixed => "mixed",
    }
}

fn normalize_request_outcome_class(
    outcome_class: crate::runtime::request_outcome::RequestOutcomeClass,
) -> &'static str {
    match outcome_class {
        crate::runtime::request_outcome::RequestOutcomeClass::Forwarded => "forwarded",
        crate::runtime::request_outcome::RequestOutcomeClass::ShortCircuited => "short_circuited",
        crate::runtime::request_outcome::RequestOutcomeClass::ControlResponse => {
            "control_response"
        }
    }
}

fn normalize_response_kind(kind: crate::runtime::request_outcome::ResponseKind) -> &'static str {
    match kind {
        crate::runtime::request_outcome::ResponseKind::ForwardAllow => "forward_allow",
        crate::runtime::request_outcome::ResponseKind::ForwardFailureFallback => {
            "forward_failure_fallback"
        }
        crate::runtime::request_outcome::ResponseKind::SyntheticShadowAllow => {
            "synthetic_shadow_allow"
        }
        crate::runtime::request_outcome::ResponseKind::SyntheticShadowAction => {
            "synthetic_shadow_action"
        }
        crate::runtime::request_outcome::ResponseKind::BlockPage => "block_page",
        crate::runtime::request_outcome::ResponseKind::PlainTextBlock => "plain_text_block",
        crate::runtime::request_outcome::ResponseKind::Redirect => "redirect",
        crate::runtime::request_outcome::ResponseKind::DropConnection => "drop_connection",
        crate::runtime::request_outcome::ResponseKind::Challenge => "challenge",
        crate::runtime::request_outcome::ResponseKind::NotABot => "not_a_bot",
        crate::runtime::request_outcome::ResponseKind::JsChallenge => "js_challenge",
        crate::runtime::request_outcome::ResponseKind::Maze => "maze",
        crate::runtime::request_outcome::ResponseKind::Tarpit => "tarpit",
        crate::runtime::request_outcome::ResponseKind::CheckpointResponse => {
            "checkpoint_response"
        }
        crate::runtime::request_outcome::ResponseKind::DefenceFollowupResponse => {
            "defence_followup_response"
        }
        crate::runtime::request_outcome::ResponseKind::SimPublicResponse => {
            "sim_public_response"
        }
        crate::runtime::request_outcome::ResponseKind::ControlPlaneResponse => {
            "control_plane_response"
        }
    }
}

fn normalize_policy_source(
    source: crate::runtime::traffic_classification::PolicySource,
) -> &'static str {
    match source {
        crate::runtime::traffic_classification::PolicySource::EarlyRoute => "early_route",
        crate::runtime::traffic_classification::PolicySource::StaticAssetBypass => {
            "static_asset_bypass"
        }
        crate::runtime::traffic_classification::PolicySource::AllowlistBypass => {
            "allowlist_bypass"
        }
        crate::runtime::traffic_classification::PolicySource::PolicyGraphFirstTranche => {
            "policy_graph_first_tranche"
        }
        crate::runtime::traffic_classification::PolicySource::PolicyGraphSecondTranche => {
            "policy_graph_second_tranche"
        }
        crate::runtime::traffic_classification::PolicySource::CleanAllow => "clean_allow",
        crate::runtime::traffic_classification::PolicySource::DefenceFollowup => {
            "defence_followup"
        }
        crate::runtime::traffic_classification::PolicySource::SimPublic => "sim_public",
        crate::runtime::traffic_classification::PolicySource::BootstrapFailure => {
            "bootstrap_failure"
        }
    }
}

fn normalize_ip_range_human_signal(signal: &str) -> &'static str {
    match signal {
        "challenge_puzzle_pass" => "challenge_puzzle_pass",
        "likely_human_sample" => "likely_human_sample",
        _ => "likely_human_sample",
    }
}

fn not_a_bot_solve_ms_bucket(solve_ms: u64) -> &'static str {
    match solve_ms {
        0..=999 => "lt_1s",
        1000..=2999 => "1_3s",
        3000..=9999 => "3_10s",
        _ => "10s_plus",
    }
}

fn should_sample_likely_human(
    ip: &str,
    sample_hint: &str,
    sample_percent: u8,
    minute_bucket: u64,
) -> bool {
    if sample_percent == 0 {
        return false;
    }
    if sample_percent >= 100 {
        return true;
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    ip.hash(&mut hasher);
    sample_hint.hash(&mut hasher);
    minute_bucket.hash(&mut hasher);
    (hasher.finish() % 100) < u64::from(sample_percent)
}

fn normalize_country(country: Option<&str>) -> String {
    country
        .map(str::trim)
        .map(str::to_ascii_uppercase)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "UNKNOWN".to_string())
}

fn parse_counter_bytes(bytes: Vec<u8>) -> u64 {
    String::from_utf8(bytes)
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0)
}

fn read_counter<S: crate::challenge::KeyValueStore>(store: &S, key: &str) -> u64 {
    store
        .get(key)
        .ok()
        .flatten()
        .map(parse_counter_bytes)
        .unwrap_or(0)
}

#[cfg(not(test))]
fn flush_counter_deltas<S: crate::challenge::KeyValueStore>(
    store: &S,
    deltas: HashMap<String, u64>,
) {
    let mut wrote_any = false;
    for (key, delta) in deltas {
        if delta == 0 {
            continue;
        }
        let current = read_counter(store, key.as_str());
        let next = current.saturating_add(delta);
        if let Err(err) = store.set(key.as_str(), next.to_string().as_bytes()) {
            eprintln!("[monitoring] failed writing {}: {:?}", key, err);
            continue;
        }
        wrote_any = true;
        if current == 0 {
            if let Some((_, _, _, hour)) =
                parse_monitoring_key_with_prefix(key.as_str(), MONITORING_PREFIX)
            {
                crate::observability::retention::register_monitoring_key(store, hour, key.as_str());
            }
        }
    }
    if wrote_any {
        crate::observability::retention::run_worker_if_due(store);
        crate::observability::hot_read_projection::refresh_after_counter_flush(store, "default");
    }
}

#[cfg(not(test))]
fn maybe_flush_pending_counter_buffer<S: crate::challenge::KeyValueStore>(store: &S, force: bool) {
    let now = now_ts();
    let pending = {
        let mut buffer = PENDING_COUNTER_BUFFER.lock().unwrap();
        if buffer.deltas.is_empty() {
            if buffer.last_flush_ts == 0 {
                buffer.last_flush_ts = now;
            }
            return;
        }
        if !force
            && buffer.deltas.len() < COUNTER_FLUSH_PENDING_KEYS_MAX
            && now.saturating_sub(buffer.last_flush_ts) < COUNTER_FLUSH_INTERVAL_SECONDS
        {
            return;
        }
        buffer.last_flush_ts = now;
        std::mem::take(&mut buffer.deltas)
    };
    flush_counter_deltas(store, pending);
}

#[cfg(test)]
fn add_counter<S: crate::challenge::KeyValueStore>(store: &S, key: &str, delta: u64) {
    if delta == 0 {
        return;
    }
    let current = read_counter(store, key);
    let next = current.saturating_add(delta);
    if let Err(err) = store.set(key, next.to_string().as_bytes()) {
        eprintln!("[monitoring] failed writing {}: {:?}", key, err);
        return;
    }
    if current == 0 {
        if let Some((_, _, _, hour)) = parse_monitoring_key_with_prefix(key, MONITORING_PREFIX) {
            crate::observability::retention::register_monitoring_key(store, hour, key);
        }
    }
    crate::observability::retention::run_worker_if_due(store);
    crate::observability::hot_read_projection::refresh_after_counter_flush(store, "default");
}

#[cfg(not(test))]
fn add_counter<S: crate::challenge::KeyValueStore>(store: &S, key: &str, delta: u64) {
    if delta == 0 {
        return;
    }
    let now = now_ts();
    {
        let mut buffer = PENDING_COUNTER_BUFFER.lock().unwrap();
        let entry = buffer.deltas.entry(key.to_string()).or_insert(0);
        *entry = entry.saturating_add(delta);
        if buffer.last_flush_ts == 0 {
            buffer.last_flush_ts = now;
        }
    }
    maybe_flush_pending_counter_buffer(store, false);
}

fn increment_counter<S: crate::challenge::KeyValueStore>(store: &S, key: &str) {
    add_counter(store, key, 1);
}

pub(crate) fn flush_pending_counters<S: crate::challenge::KeyValueStore>(_store: &S) {
    #[cfg(not(test))]
    maybe_flush_pending_counter_buffer(_store, true);
}

fn monitoring_key_with_prefix(
    prefix: &str,
    section: &str,
    metric: &str,
    dimension: Option<&str>,
    hour: u64,
) -> String {
    if let Some(value) = dimension {
        return format!(
            "{}:{}:{}:{}:{}",
            prefix,
            section,
            metric,
            encode_dim(value),
            hour
        );
    }
    format!("{}:{}:{}:{}", prefix, section, metric, hour)
}

fn monitoring_key(section: &str, metric: &str, dimension: Option<&str>, hour: u64) -> String {
    monitoring_key_with_prefix(
        monitoring_prefix_for_active_context(),
        section,
        metric,
        dimension,
        hour,
    )
}

fn parse_monitoring_key_with_prefix(
    key: &str,
    prefix: &str,
) -> Option<(String, String, Option<String>, u64)> {
    let stripped = key.strip_prefix(prefix)?.strip_prefix(':')?;
    let parts: Vec<&str> = stripped.split(':').collect();
    match parts.as_slice() {
        [section, metric, hour] => Some((
            section.to_string(),
            metric.to_string(),
            None,
            hour.parse::<u64>().ok()?,
        )),
        [section, metric, dimension, hour] => Some((
            section.to_string(),
            metric.to_string(),
            Some(decode_dim(dimension)),
            hour.parse::<u64>().ok()?,
        )),
        _ => None,
    }
}

fn matching_monitoring_prefix<'a>(key: &str, prefixes: &'a [&str]) -> Option<&'a str> {
    prefixes
        .iter()
        .copied()
        .filter(|prefix| key == *prefix || key.starts_with(format!("{}:", prefix).as_str()))
        .max_by_key(|prefix| prefix.len())
}

fn is_guarded_dimension(section: &str, metric: &str) -> bool {
    GUARDED_DIMENSION_PAIRS
        .iter()
        .any(|(guarded_section, guarded_metric)| {
            section == *guarded_section && metric == *guarded_metric
        })
}

fn cardinality_guard_marker_key(section: &str, metric: &str, value: &str, hour: u64) -> String {
    monitoring_key(
        "cardinality_guard",
        format!("{section}|{metric}").as_str(),
        Some(value),
        hour,
    )
}

fn cardinality_guard_count_key(section: &str, metric: &str, hour: u64) -> String {
    monitoring_key(
        "cardinality_guard_count",
        format!("{section}|{metric}").as_str(),
        None,
        hour,
    )
}

fn cardinality_guard_overflow_key(section: &str, metric: &str, hour: u64) -> String {
    monitoring_key(
        "cardinality_guard_overflow",
        format!("{section}|{metric}").as_str(),
        None,
        hour,
    )
}

fn apply_guarded_dimension_cardinality<S: crate::challenge::KeyValueStore>(
    store: &S,
    section: &str,
    metric: &str,
    value: &str,
    hour: u64,
) -> String {
    if !is_guarded_dimension(section, metric) {
        return value.to_string();
    }
    let marker_key = cardinality_guard_marker_key(section, metric, value, hour);
    if store
        .get(marker_key.as_str())
        .ok()
        .flatten()
        .is_some()
    {
        return value.to_string();
    }

    let count_key = cardinality_guard_count_key(section, metric, hour);
    let distinct_count = read_counter(store, count_key.as_str());
    if distinct_count < GUARDED_DIMENSION_CARDINALITY_CAP_PER_HOUR {
        if let Err(err) = store.set(marker_key.as_str(), b"1") {
            eprintln!(
                "[monitoring] failed writing cardinality marker {}: {:?}",
                marker_key, err
            );
            return value.to_string();
        }
        crate::observability::retention::register_monitoring_key(store, hour, marker_key.as_str());
        let next = distinct_count.saturating_add(1);
        if let Err(err) = store.set(count_key.as_str(), next.to_string().as_bytes()) {
            eprintln!(
                "[monitoring] failed writing cardinality count {}: {:?}",
                count_key, err
            );
            return value.to_string();
        }
        if distinct_count == 0 {
            crate::observability::retention::register_monitoring_key(store, hour, count_key.as_str());
        }
        return value.to_string();
    }

    let overflow_key = cardinality_guard_overflow_key(section, metric, hour);
    let overflow_next = read_counter(store, overflow_key.as_str()).saturating_add(1);
    if let Err(err) = store.set(overflow_key.as_str(), overflow_next.to_string().as_bytes()) {
        eprintln!(
            "[monitoring] failed writing cardinality overflow {}: {:?}",
            overflow_key, err
        );
    } else if overflow_next == 1 {
        crate::observability::retention::register_monitoring_key(store, hour, overflow_key.as_str());
    }
    GUARDED_DIMENSION_OVERFLOW_VALUE.to_string()
}

fn record_with_dimension<S: crate::challenge::KeyValueStore>(
    store: &S,
    section: &str,
    metric: &str,
    dimension: Option<&str>,
) {
    let hour = now_ts() / 3600;
    let dimension_value =
        dimension.map(|value| apply_guarded_dimension_cardinality(store, section, metric, value, hour));
    let key = monitoring_key(section, metric, dimension_value.as_deref(), hour);
    increment_counter(store, key.as_str());
}

fn record_with_dimension_delta<S: crate::challenge::KeyValueStore>(
    store: &S,
    section: &str,
    metric: &str,
    dimension: Option<&str>,
    delta: u64,
) {
    if delta == 0 {
        return;
    }
    let hour = now_ts() / 3600;
    let dimension_value =
        dimension.map(|value| apply_guarded_dimension_cardinality(store, section, metric, value, hour));
    let key = monitoring_key(section, metric, dimension_value.as_deref(), hour);
    add_counter(store, key.as_str(), delta);
}

fn request_outcome_scope_cohort(
    outcome: &crate::runtime::request_outcome::RenderedRequestOutcome,
) -> String {
    [
        normalize_traffic_origin(outcome.traffic_origin),
        normalize_measurement_scope(outcome.measurement_scope),
        normalize_execution_mode(outcome.execution_mode),
    ]
    .join("|")
}

fn request_outcome_nested_cohort(prefix: &str, suffix: &str) -> String {
    format!("{prefix}|{suffix}")
}

fn current_traffic_origin() -> crate::runtime::request_outcome::TrafficOrigin {
    if crate::runtime::sim_telemetry::current_metadata().is_some() {
        crate::runtime::request_outcome::TrafficOrigin::AdversarySim
    } else {
        crate::runtime::request_outcome::TrafficOrigin::Live
    }
}

fn origin_cohort(origin: crate::runtime::request_outcome::TrafficOrigin) -> String {
    normalize_traffic_origin(origin).to_string()
}

fn origin_nested_cohort(
    origin: crate::runtime::request_outcome::TrafficOrigin,
    suffix: &str,
) -> String {
    request_outcome_nested_cohort(origin_cohort(origin).as_str(), suffix)
}

fn request_outcome_lane_cohort(
    outcome: &crate::runtime::request_outcome::RenderedRequestOutcome,
) -> Option<String> {
    outcome.traffic_lane.map(|lane| {
        [
            request_outcome_scope_cohort(outcome),
            normalize_traffic_lane(lane.lane).to_string(),
            normalize_telemetry_exactness(lane.exactness).to_string(),
            normalize_telemetry_basis(lane.basis).to_string(),
        ]
        .join("|")
    })
}

fn split_last_cohort_segment(value: &str) -> Option<(&str, &str)> {
    value.rsplit_once('|')
}

fn parse_request_outcome_scope_cohort(
    cohort: &str,
) -> Option<(String, String, String)> {
    let mut parts = cohort.split('|');
    let traffic_origin = parts.next()?.to_string();
    let measurement_scope = parts.next()?.to_string();
    let execution_mode = parts.next()?.to_string();
    if parts.next().is_some() {
        return None;
    }
    Some((traffic_origin, measurement_scope, execution_mode))
}

fn parse_request_outcome_scope_breakdown_cohort(
    cohort: &str,
) -> Option<(String, String, String, String)> {
    let (scope_cohort, value) = split_last_cohort_segment(cohort)?;
    let (traffic_origin, measurement_scope, execution_mode) =
        parse_request_outcome_scope_cohort(scope_cohort)?;
    Some((
        traffic_origin,
        measurement_scope,
        execution_mode,
        value.to_string(),
    ))
}

fn parse_request_outcome_scope_breakdown_outcome_cohort(
    cohort: &str,
) -> Option<(String, String, String, String, String)> {
    let (breakdown_cohort, outcome_class) = split_last_cohort_segment(cohort)?;
    let (traffic_origin, measurement_scope, execution_mode, value) =
        parse_request_outcome_scope_breakdown_cohort(breakdown_cohort)?;
    Some((
        traffic_origin,
        measurement_scope,
        execution_mode,
        value,
        outcome_class.to_string(),
    ))
}

fn parse_origin_cohort(cohort: &str) -> Option<String> {
    (!cohort.contains('|')).then(|| cohort.to_string())
}

fn parse_origin_breakdown_cohort(cohort: &str) -> Option<(String, String)> {
    let (origin, value) = split_last_cohort_segment(cohort)?;
    Some((origin.to_string(), value.to_string()))
}

fn parse_request_outcome_lane_breakdown_cohort(
    cohort: &str,
) -> Option<(String, String, String, String, String, String, String)> {
    let (lane_cohort, value) = split_last_cohort_segment(cohort)?;
    let (traffic_origin, measurement_scope, execution_mode, lane, exactness, basis) =
        parse_request_outcome_lane_cohort(lane_cohort)?;
    Some((
        traffic_origin,
        measurement_scope,
        execution_mode,
        lane,
        exactness,
        basis,
        value.to_string(),
    ))
}

fn parse_request_outcome_lane_cohort(
    cohort: &str,
) -> Option<(String, String, String, String, String, String)> {
    let mut parts = cohort.split('|');
    let traffic_origin = parts.next()?.to_string();
    let measurement_scope = parts.next()?.to_string();
    let execution_mode = parts.next()?.to_string();
    let lane = parts.next()?.to_string();
    let exactness = parts.next()?.to_string();
    let basis = parts.next()?.to_string();
    if parts.next().is_some() {
        return None;
    }
    Some((
        traffic_origin,
        measurement_scope,
        execution_mode,
        lane,
        exactness,
        basis,
    ))
}

pub(crate) fn record_request_outcome<S: crate::challenge::KeyValueStore>(
    store: &S,
    outcome: &crate::runtime::request_outcome::RenderedRequestOutcome,
) {
    let scope_cohort = request_outcome_scope_cohort(outcome);
    record_with_dimension(store, "request_outcome", "total", Some(scope_cohort.as_str()));
    record_with_dimension(
        store,
        "request_outcome",
        "outcome_class",
        Some(
            request_outcome_nested_cohort(
                scope_cohort.as_str(),
                normalize_request_outcome_class(outcome.outcome_class),
            )
            .as_str(),
        ),
    );
    record_with_dimension(
        store,
        "request_outcome",
        "response_kind",
        Some(
            request_outcome_nested_cohort(
                scope_cohort.as_str(),
                normalize_response_kind(outcome.response_kind),
            )
            .as_str(),
        ),
    );
    record_with_dimension(
        store,
        "request_outcome",
        "response_kind_outcome_class",
        Some(
            request_outcome_nested_cohort(
                request_outcome_nested_cohort(
                    scope_cohort.as_str(),
                    normalize_response_kind(outcome.response_kind),
                )
                .as_str(),
                normalize_request_outcome_class(outcome.outcome_class),
            )
            .as_str(),
        ),
    );
    record_with_dimension(
        store,
        "request_outcome",
        "route_action_family",
        Some(
            request_outcome_nested_cohort(
                scope_cohort.as_str(),
                normalize_route_action_family(outcome.route_action_family),
            )
            .as_str(),
        ),
    );
    record_with_dimension(
        store,
        "request_outcome",
        "route_action_family_outcome_class",
        Some(
            request_outcome_nested_cohort(
                request_outcome_nested_cohort(
                    scope_cohort.as_str(),
                    normalize_route_action_family(outcome.route_action_family),
                )
                .as_str(),
                normalize_request_outcome_class(outcome.outcome_class),
            )
            .as_str(),
        ),
    );
    record_with_dimension(
        store,
        "request_outcome",
        "policy_source",
        Some(
            request_outcome_nested_cohort(
                scope_cohort.as_str(),
                normalize_policy_source(outcome.policy_source),
            )
            .as_str(),
        ),
    );
    record_with_dimension(
        store,
        "request_outcome",
        "policy_source_outcome_class",
        Some(
            request_outcome_nested_cohort(
                request_outcome_nested_cohort(
                    scope_cohort.as_str(),
                    normalize_policy_source(outcome.policy_source),
                )
                .as_str(),
                normalize_request_outcome_class(outcome.outcome_class),
            )
            .as_str(),
        ),
    );
    record_with_dimension_delta(
        store,
        "request_outcome",
        "response_bytes",
        Some(scope_cohort.as_str()),
        outcome.response_bytes,
    );
    record_with_dimension_delta(
        store,
        "request_outcome",
        "outcome_class_response_bytes",
        Some(
            request_outcome_nested_cohort(
                scope_cohort.as_str(),
                normalize_request_outcome_class(outcome.outcome_class),
            )
            .as_str(),
        ),
        outcome.response_bytes,
    );

    if let Some(lane_cohort) = request_outcome_lane_cohort(outcome) {
        record_with_dimension(
            store,
            "request_outcome",
            "lane_total",
            Some(lane_cohort.as_str()),
        );
        record_with_dimension(
            store,
            "request_outcome",
            "lane_outcome_class",
            Some(
                request_outcome_nested_cohort(
                    lane_cohort.as_str(),
                    normalize_request_outcome_class(outcome.outcome_class),
                )
                .as_str(),
            ),
        );
        record_with_dimension_delta(
            store,
            "request_outcome",
            "lane_response_bytes",
            Some(lane_cohort.as_str()),
            outcome.response_bytes,
        );
        record_with_dimension(
            store,
            "request_outcome",
            "lane_response_kind",
            Some(
                request_outcome_nested_cohort(
                    lane_cohort.as_str(),
                    normalize_response_kind(outcome.response_kind),
                )
                .as_str(),
            ),
        );
        record_with_dimension_delta(
            store,
            "request_outcome",
            "lane_outcome_class_response_bytes",
            Some(
                request_outcome_nested_cohort(
                    lane_cohort.as_str(),
                    normalize_request_outcome_class(outcome.outcome_class),
                )
                .as_str(),
            ),
            outcome.response_bytes,
        );
    }
}

pub(crate) fn record_honeypot_hit<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    path: &str,
) {
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let normalized_path = normalize_telemetry_path(path);
    record_with_dimension(store, "honeypot", "total", None);
    record_with_dimension(store, "honeypot", "ip", Some(ip_bucket.as_str()));
    record_with_dimension(store, "honeypot", "path", Some(normalized_path.as_str()));
}

pub(crate) fn record_challenge_failure<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    reason: &str,
) {
    let origin = current_traffic_origin();
    let normalized_reason = normalize_challenge_reason(reason);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "challenge", "total", Some(origin_cohort(origin).as_str()));
    record_with_dimension(
        store,
        "challenge",
        "reason",
        Some(origin_nested_cohort(origin, normalized_reason).as_str()),
    );
    record_with_dimension(
        store,
        "challenge",
        "ip",
        Some(origin_nested_cohort(origin, ip_bucket.as_str()).as_str()),
    );
}

pub(crate) fn record_pow_failure<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    reason: &str,
) {
    let origin = current_traffic_origin();
    let normalized_reason = normalize_pow_reason(reason);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "pow", "total", Some(origin_cohort(origin).as_str()));
    record_with_dimension(
        store,
        "pow",
        "outcome",
        Some(origin_nested_cohort(origin, "failure").as_str()),
    );
    record_with_dimension(
        store,
        "pow",
        "reason",
        Some(origin_nested_cohort(origin, normalized_reason).as_str()),
    );
    record_with_dimension(
        store,
        "pow",
        "ip",
        Some(origin_nested_cohort(origin, ip_bucket.as_str()).as_str()),
    );
}

pub(crate) fn record_pow_success<S: crate::challenge::KeyValueStore>(store: &S) {
    let origin = current_traffic_origin();
    record_with_dimension(
        store,
        "pow",
        "success",
        Some(origin_cohort(origin).as_str()),
    );
    record_with_dimension(
        store,
        "pow",
        "outcome",
        Some(origin_nested_cohort(origin, "success").as_str()),
    );
}

pub(crate) fn record_rate_violation_with_path<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    path: Option<&str>,
    outcome: &str,
) {
    let origin = current_traffic_origin();
    let normalized_outcome = normalize_rate_outcome(outcome);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "rate", "total", Some(origin_cohort(origin).as_str()));
    record_with_dimension(
        store,
        "rate",
        "outcome",
        Some(origin_nested_cohort(origin, normalized_outcome).as_str()),
    );
    record_with_dimension(
        store,
        "rate",
        "ip",
        Some(origin_nested_cohort(origin, ip_bucket.as_str()).as_str()),
    );
    if let Some(raw_path) = path {
        let normalized_path = normalize_telemetry_path(raw_path);
        record_with_dimension(
            store,
            "rate",
            "path",
            Some(origin_nested_cohort(origin, normalized_path.as_str()).as_str()),
        );
    }
}

pub(crate) fn record_rate_outcome<S: crate::challenge::KeyValueStore>(store: &S, outcome: &str) {
    let origin = current_traffic_origin();
    let normalized_outcome = normalize_rate_outcome(outcome);
    record_with_dimension(
        store,
        "rate",
        "outcome",
        Some(origin_nested_cohort(origin, normalized_outcome).as_str()),
    );
}

pub(crate) fn record_geo_violation<S: crate::challenge::KeyValueStore>(
    store: &S,
    country: Option<&str>,
    action: &str,
) {
    let origin = current_traffic_origin();
    let normalized_action = normalize_geo_action(action);
    let normalized_country = normalize_country(country);
    record_with_dimension(store, "geo", "total", Some(origin_cohort(origin).as_str()));
    record_with_dimension(
        store,
        "geo",
        "action",
        Some(origin_nested_cohort(origin, normalized_action).as_str()),
    );
    record_with_dimension(
        store,
        "geo",
        "country",
        Some(origin_nested_cohort(origin, normalized_country.as_str()).as_str()),
    );
}

pub(crate) fn record_not_a_bot_served<S: crate::challenge::KeyValueStore>(store: &S) {
    let origin = current_traffic_origin();
    record_with_dimension(
        store,
        "not_a_bot",
        "served",
        Some(origin_cohort(origin).as_str()),
    );
}

pub(crate) fn record_not_a_bot_submit<S: crate::challenge::KeyValueStore>(
    store: &S,
    outcome: &str,
    solve_ms: Option<u64>,
) {
    let origin = current_traffic_origin();
    let normalized_outcome = normalize_not_a_bot_outcome(outcome);
    record_with_dimension(
        store,
        "not_a_bot",
        "submitted",
        Some(origin_cohort(origin).as_str()),
    );
    record_with_dimension(
        store,
        "not_a_bot",
        "outcome",
        Some(origin_nested_cohort(origin, normalized_outcome).as_str()),
    );
    if let Some(ms) = solve_ms {
        let bucket = not_a_bot_solve_ms_bucket(ms);
        record_with_dimension(
            store,
            "not_a_bot",
            "solve_ms_bucket",
            Some(origin_nested_cohort(origin, bucket).as_str()),
        );
    }
}

pub(crate) fn record_shadow_action<S: crate::challenge::KeyValueStore>(
    store: &S,
    action: crate::runtime::effect_intents::ShadowAction,
) {
    let normalized_action = normalize_shadow_action(action);
    record_with_dimension(store, "shadow", "total", None);
    record_with_dimension(store, "shadow", "action", Some(normalized_action));
}

pub(crate) fn record_shadow_pass_through<S: crate::challenge::KeyValueStore>(store: &S) {
    record_with_dimension(store, "shadow", "pass_through", None);
}

fn record_ip_range_human_signal<S: crate::challenge::KeyValueStore>(store: &S, ip: &str, signal: &str) {
    let normalized_signal = normalize_ip_range_human_signal(signal);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "ip_range_suggestions", "human_total", None);
    record_with_dimension(
        store,
        "ip_range_suggestions",
        "human_signal",
        Some(normalized_signal),
    );
    record_with_dimension(
        store,
        "ip_range_suggestions",
        "human_ip",
        Some(ip_bucket.as_str()),
    );
}

pub(crate) fn record_ip_range_challenge_solved<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
) {
    record_ip_range_human_signal(store, ip, "challenge_puzzle_pass");
}

pub(crate) fn maybe_record_ip_range_likely_human_sample<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    sample_percent: u8,
    sample_hint: &str,
) {
    let minute_bucket = now_ts() / 60;
    if !should_sample_likely_human(ip, sample_hint, sample_percent, minute_bucket) {
        record_with_dimension(store, "ip_range_suggestions", "likely_human_unsampled", None);
        return;
    }
    record_with_dimension(store, "ip_range_suggestions", "likely_human_sampled", None);
    record_ip_range_human_signal(store, ip, "likely_human_sample");
}

fn build_seeded_map(keys: &[&str]) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for key in keys {
        map.insert((*key).to_string(), 0);
    }
    map
}

fn top_entries(map: &HashMap<String, u64>, limit: usize) -> Vec<CountEntry> {
    let mut rows: Vec<CountEntry> = map
        .iter()
        .map(|(label, count)| CountEntry {
            label: label.clone(),
            count: *count,
        })
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.label.cmp(&b.label)));
    rows.truncate(limit);
    rows
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TrendAccumulator {
    totals: HashMap<u64, u64>,
    reasons: HashMap<u64, HashMap<String, u64>>,
}

fn build_trend(
    start_hour: u64,
    end_hour: u64,
    base_reasons: &[&str],
    accumulator: TrendAccumulator,
) -> Vec<TrendPoint> {
    let mut trend = Vec::new();
    for hour in start_hour..=end_hour {
        let mut reasons = build_seeded_map(base_reasons);
        if let Some(row) = accumulator.reasons.get(&hour) {
            for (reason, count) in row {
                let entry = reasons.entry(reason.clone()).or_insert(0);
                *entry = entry.saturating_add(*count);
            }
        }
        let reason_total = reasons.values().copied().sum::<u64>();
        let total = accumulator
            .totals
            .get(&hour)
            .copied()
            .unwrap_or(reason_total);
        trend.push(TrendPoint {
            ts: hour.saturating_mul(3600),
            total,
            reasons,
        });
    }
    trend
}

const MONITORING_DAY_ROLLUP_SCHEMA_VERSION: &str = "monitoring-day-rollup.v1";
const MONITORING_DAY_HOURS: u64 = 24;
const MONITORING_ROLLUP_KEY_PREFIX: &str = "monitoring_rollup:v1:day";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct MonitoringAccumulator {
    shadow_total: u64,
    shadow_pass_through_total: u64,
    shadow_actions: HashMap<String, u64>,
    honeypot_total: u64,
    honeypot_ip_counts: HashMap<String, u64>,
    honeypot_path_counts: HashMap<String, u64>,
    challenge_totals_by_origin: HashMap<String, u64>,
    challenge_ip_counts_by_origin: HashMap<String, HashMap<String, u64>>,
    challenge_reason_counts_by_origin: HashMap<String, HashMap<String, u64>>,
    challenge_trends_by_origin: HashMap<String, TrendAccumulator>,
    not_a_bot_served_by_origin: HashMap<String, u64>,
    not_a_bot_submitted_by_origin: HashMap<String, u64>,
    not_a_bot_outcomes_by_origin: HashMap<String, HashMap<String, u64>>,
    not_a_bot_latency_buckets_by_origin: HashMap<String, HashMap<String, u64>>,
    pow_totals_by_origin: HashMap<String, u64>,
    pow_success_totals_by_origin: HashMap<String, u64>,
    pow_ip_counts_by_origin: HashMap<String, HashMap<String, u64>>,
    pow_reason_counts_by_origin: HashMap<String, HashMap<String, u64>>,
    pow_outcomes_by_origin: HashMap<String, HashMap<String, u64>>,
    pow_trends_by_origin: HashMap<String, TrendAccumulator>,
    rate_totals_by_origin: HashMap<String, u64>,
    rate_ip_counts_by_origin: HashMap<String, HashMap<String, u64>>,
    rate_path_counts_by_origin: HashMap<String, HashMap<String, u64>>,
    rate_outcomes_by_origin: HashMap<String, HashMap<String, u64>>,
    geo_totals_by_origin: HashMap<String, u64>,
    geo_actions_by_origin: HashMap<String, HashMap<String, u64>>,
    geo_countries_by_origin: HashMap<String, HashMap<String, u64>>,
    request_outcome_scope_totals: HashMap<String, u64>,
    request_outcome_scope_bytes: HashMap<String, u64>,
    request_outcome_scope_outcomes: HashMap<String, u64>,
    request_outcome_scope_outcome_bytes: HashMap<String, u64>,
    request_outcome_scope_response_kinds: HashMap<String, u64>,
    request_outcome_scope_response_kind_outcomes: HashMap<String, u64>,
    request_outcome_scope_policy_sources: HashMap<String, u64>,
    request_outcome_scope_policy_source_outcomes: HashMap<String, u64>,
    request_outcome_scope_route_action_families: HashMap<String, u64>,
    request_outcome_scope_route_action_family_outcomes: HashMap<String, u64>,
    request_outcome_lane_totals: HashMap<String, u64>,
    request_outcome_lane_bytes: HashMap<String, u64>,
    request_outcome_lane_response_kinds: HashMap<String, u64>,
    request_outcome_lane_outcomes: HashMap<String, u64>,
    request_outcome_lane_outcome_bytes: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitoringDayRollup {
    schema_version: String,
    day_start_hour: u64,
    day_end_hour: u64,
    accumulator: MonitoringAccumulator,
}

impl MonitoringAccumulator {
    fn add_count(map: &mut HashMap<String, u64>, key: &str, count: u64) {
        let entry = map.entry(key.to_string()).or_insert(0);
        *entry = entry.saturating_add(count);
    }

    fn merge_count_maps(target: &mut HashMap<String, u64>, source: &HashMap<String, u64>) {
        for (key, count) in source {
            let entry = target.entry(key.clone()).or_insert(0);
            *entry = entry.saturating_add(*count);
        }
    }

    fn add_nested_count(
        target: &mut HashMap<String, HashMap<String, u64>>,
        outer: &str,
        inner: &str,
        count: u64,
    ) {
        let row = target.entry(outer.to_string()).or_default();
        let entry = row.entry(inner.to_string()).or_insert(0);
        *entry = entry.saturating_add(count);
    }

    fn merge_nested_count_maps(
        target: &mut HashMap<String, HashMap<String, u64>>,
        source: &HashMap<String, HashMap<String, u64>>,
    ) {
        for (outer, source_row) in source {
            let target_row = target.entry(outer.clone()).or_default();
            for (inner, count) in source_row {
                let entry = target_row.entry(inner.clone()).or_insert(0);
                *entry = entry.saturating_add(*count);
            }
        }
    }

    fn merge_trend(target: &mut TrendAccumulator, source: &TrendAccumulator) {
        for (hour, total) in &source.totals {
            let entry = target.totals.entry(*hour).or_insert(0);
            *entry = entry.saturating_add(*total);
        }
        for (hour, reasons) in &source.reasons {
            let target_row = target.reasons.entry(*hour).or_default();
            for (reason, count) in reasons {
                let entry = target_row.entry(reason.clone()).or_insert(0);
                *entry = entry.saturating_add(*count);
            }
        }
    }

    fn consume_counter(&mut self, section: &str, metric: &str, dimension: Option<&str>, hour: u64, count: u64) {
        match section {
            "shadow" => match metric {
                "total" => self.shadow_total = self.shadow_total.saturating_add(count),
                "pass_through" => {
                    self.shadow_pass_through_total =
                        self.shadow_pass_through_total.saturating_add(count)
                }
                "action" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.shadow_actions, dim, count);
                    }
                }
                _ => {}
            },
            "honeypot" => match metric {
                "total" => self.honeypot_total = self.honeypot_total.saturating_add(count),
                "ip" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.honeypot_ip_counts, dim, count);
                    }
                }
                "path" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.honeypot_path_counts, dim, count);
                    }
                }
                _ => {}
            },
            "challenge" => match metric {
                "total" => {
                    if let Some(origin) = dimension.and_then(parse_origin_cohort) {
                        let entry = self.challenge_totals_by_origin.entry(origin.clone()).or_insert(0);
                        *entry = entry.saturating_add(count);
                        let trend = self.challenge_trends_by_origin.entry(origin).or_default();
                        let total = trend.totals.entry(hour).or_insert(0);
                        *total = total.saturating_add(count);
                    }
                }
                "ip" => {
                    if let Some((origin, ip_bucket)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.challenge_ip_counts_by_origin,
                            origin.as_str(),
                            ip_bucket.as_str(),
                            count,
                        );
                    }
                }
                "reason" => {
                    if let Some((origin, reason)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.challenge_reason_counts_by_origin,
                            origin.as_str(),
                            reason.as_str(),
                            count,
                        );
                        let trend = self.challenge_trends_by_origin.entry(origin).or_default();
                        let row = trend.reasons.entry(hour).or_default();
                        let entry = row.entry(reason).or_insert(0);
                        *entry = entry.saturating_add(count);
                    }
                }
                _ => {}
            },
            "not_a_bot" => match metric {
                "served" => {
                    if let Some(origin) = dimension.and_then(parse_origin_cohort) {
                        let entry = self.not_a_bot_served_by_origin.entry(origin).or_insert(0);
                        *entry = entry.saturating_add(count);
                    }
                }
                "submitted" => {
                    if let Some(origin) = dimension.and_then(parse_origin_cohort) {
                        let entry = self.not_a_bot_submitted_by_origin.entry(origin).or_insert(0);
                        *entry = entry.saturating_add(count);
                    }
                }
                "outcome" => {
                    if let Some((origin, outcome)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.not_a_bot_outcomes_by_origin,
                            origin.as_str(),
                            outcome.as_str(),
                            count,
                        );
                    }
                }
                "solve_ms_bucket" => {
                    if let Some((origin, bucket)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.not_a_bot_latency_buckets_by_origin,
                            origin.as_str(),
                            bucket.as_str(),
                            count,
                        );
                    }
                }
                _ => {}
            },
            "pow" => match metric {
                "total" => {
                    if let Some(origin) = dimension.and_then(parse_origin_cohort) {
                        let entry = self.pow_totals_by_origin.entry(origin.clone()).or_insert(0);
                        *entry = entry.saturating_add(count);
                        let trend = self.pow_trends_by_origin.entry(origin).or_default();
                        let total = trend.totals.entry(hour).or_insert(0);
                        *total = total.saturating_add(count);
                    }
                }
                "success" => {
                    if let Some(origin) = dimension.and_then(parse_origin_cohort) {
                        let entry = self.pow_success_totals_by_origin.entry(origin).or_insert(0);
                        *entry = entry.saturating_add(count);
                    }
                }
                "ip" => {
                    if let Some((origin, ip_bucket)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.pow_ip_counts_by_origin,
                            origin.as_str(),
                            ip_bucket.as_str(),
                            count,
                        );
                    }
                }
                "reason" => {
                    if let Some((origin, reason)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.pow_reason_counts_by_origin,
                            origin.as_str(),
                            reason.as_str(),
                            count,
                        );
                        let trend = self.pow_trends_by_origin.entry(origin).or_default();
                        let row = trend.reasons.entry(hour).or_default();
                        let entry = row.entry(reason).or_insert(0);
                        *entry = entry.saturating_add(count);
                    }
                }
                "outcome" => {
                    if let Some((origin, outcome)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.pow_outcomes_by_origin,
                            origin.as_str(),
                            outcome.as_str(),
                            count,
                        );
                    }
                }
                _ => {}
            },
            "rate" => match metric {
                "total" => {
                    if let Some(origin) = dimension.and_then(parse_origin_cohort) {
                        let entry = self.rate_totals_by_origin.entry(origin).or_insert(0);
                        *entry = entry.saturating_add(count);
                    }
                }
                "ip" => {
                    if let Some((origin, ip_bucket)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.rate_ip_counts_by_origin,
                            origin.as_str(),
                            ip_bucket.as_str(),
                            count,
                        );
                    }
                }
                "path" => {
                    if let Some((origin, path)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.rate_path_counts_by_origin,
                            origin.as_str(),
                            path.as_str(),
                            count,
                        );
                    }
                }
                "outcome" => {
                    if let Some((origin, outcome)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.rate_outcomes_by_origin,
                            origin.as_str(),
                            outcome.as_str(),
                            count,
                        );
                    }
                }
                _ => {}
            },
            "geo" => match metric {
                "total" => {
                    if let Some(origin) = dimension.and_then(parse_origin_cohort) {
                        let entry = self.geo_totals_by_origin.entry(origin).or_insert(0);
                        *entry = entry.saturating_add(count);
                    }
                }
                "action" => {
                    if let Some((origin, action)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.geo_actions_by_origin,
                            origin.as_str(),
                            action.as_str(),
                            count,
                        );
                    }
                }
                "country" => {
                    if let Some((origin, country)) =
                        dimension.and_then(parse_origin_breakdown_cohort)
                    {
                        Self::add_nested_count(
                            &mut self.geo_countries_by_origin,
                            origin.as_str(),
                            country.as_str(),
                            count,
                        );
                    }
                }
                _ => {}
            },
            "request_outcome" => match metric {
                "total" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.request_outcome_scope_totals, dim, count);
                    }
                }
                "response_bytes" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.request_outcome_scope_bytes, dim, count);
                    }
                }
                "response_kind" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.request_outcome_scope_response_kinds, dim, count);
                    }
                }
                "response_kind_outcome_class" => {
                    if let Some(dim) = dimension {
                        Self::add_count(
                            &mut self.request_outcome_scope_response_kind_outcomes,
                            dim,
                            count,
                        );
                    }
                }
                "route_action_family" => {
                    if let Some(dim) = dimension {
                        Self::add_count(
                            &mut self.request_outcome_scope_route_action_families,
                            dim,
                            count,
                        );
                    }
                }
                "route_action_family_outcome_class" => {
                    if let Some(dim) = dimension {
                        Self::add_count(
                            &mut self.request_outcome_scope_route_action_family_outcomes,
                            dim,
                            count,
                        );
                    }
                }
                "policy_source" => {
                    if let Some(dim) = dimension {
                        Self::add_count(
                            &mut self.request_outcome_scope_policy_sources,
                            dim,
                            count,
                        );
                    }
                }
                "policy_source_outcome_class" => {
                    if let Some(dim) = dimension {
                        Self::add_count(
                            &mut self.request_outcome_scope_policy_source_outcomes,
                            dim,
                            count,
                        );
                    }
                }
                "outcome_class" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.request_outcome_scope_outcomes, dim, count);
                    }
                }
                "outcome_class_response_bytes" => {
                    if let Some(dim) = dimension {
                        Self::add_count(
                            &mut self.request_outcome_scope_outcome_bytes,
                            dim,
                            count,
                        );
                    }
                }
                "lane_total" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.request_outcome_lane_totals, dim, count);
                    }
                }
                "lane_response_bytes" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.request_outcome_lane_bytes, dim, count);
                    }
                }
                "lane_response_kind" => {
                    if let Some(dim) = dimension {
                        Self::add_count(
                            &mut self.request_outcome_lane_response_kinds,
                            dim,
                            count,
                        );
                    }
                }
                "lane_outcome_class" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.request_outcome_lane_outcomes, dim, count);
                    }
                }
                "lane_outcome_class_response_bytes" => {
                    if let Some(dim) = dimension {
                        Self::add_count(
                            &mut self.request_outcome_lane_outcome_bytes,
                            dim,
                            count,
                        );
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn merge_rollup(&mut self, source: &MonitoringAccumulator) {
        self.shadow_total = self.shadow_total.saturating_add(source.shadow_total);
        self.shadow_pass_through_total = self
            .shadow_pass_through_total
            .saturating_add(source.shadow_pass_through_total);
        Self::merge_count_maps(&mut self.shadow_actions, &source.shadow_actions);
        self.honeypot_total = self.honeypot_total.saturating_add(source.honeypot_total);
        Self::merge_count_maps(&mut self.honeypot_ip_counts, &source.honeypot_ip_counts);
        Self::merge_count_maps(&mut self.honeypot_path_counts, &source.honeypot_path_counts);
        Self::merge_count_maps(
            &mut self.challenge_totals_by_origin,
            &source.challenge_totals_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.challenge_ip_counts_by_origin,
            &source.challenge_ip_counts_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.challenge_reason_counts_by_origin,
            &source.challenge_reason_counts_by_origin,
        );
        for (origin, trend) in &source.challenge_trends_by_origin {
            Self::merge_trend(self.challenge_trends_by_origin.entry(origin.clone()).or_default(), trend);
        }
        Self::merge_count_maps(
            &mut self.not_a_bot_served_by_origin,
            &source.not_a_bot_served_by_origin,
        );
        Self::merge_count_maps(
            &mut self.not_a_bot_submitted_by_origin,
            &source.not_a_bot_submitted_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.not_a_bot_outcomes_by_origin,
            &source.not_a_bot_outcomes_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.not_a_bot_latency_buckets_by_origin,
            &source.not_a_bot_latency_buckets_by_origin,
        );
        Self::merge_count_maps(
            &mut self.pow_totals_by_origin,
            &source.pow_totals_by_origin,
        );
        Self::merge_count_maps(
            &mut self.pow_success_totals_by_origin,
            &source.pow_success_totals_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.pow_ip_counts_by_origin,
            &source.pow_ip_counts_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.pow_reason_counts_by_origin,
            &source.pow_reason_counts_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.pow_outcomes_by_origin,
            &source.pow_outcomes_by_origin,
        );
        for (origin, trend) in &source.pow_trends_by_origin {
            Self::merge_trend(self.pow_trends_by_origin.entry(origin.clone()).or_default(), trend);
        }
        Self::merge_count_maps(
            &mut self.rate_totals_by_origin,
            &source.rate_totals_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.rate_ip_counts_by_origin,
            &source.rate_ip_counts_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.rate_path_counts_by_origin,
            &source.rate_path_counts_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.rate_outcomes_by_origin,
            &source.rate_outcomes_by_origin,
        );
        Self::merge_count_maps(
            &mut self.geo_totals_by_origin,
            &source.geo_totals_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.geo_actions_by_origin,
            &source.geo_actions_by_origin,
        );
        Self::merge_nested_count_maps(
            &mut self.geo_countries_by_origin,
            &source.geo_countries_by_origin,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_totals,
            &source.request_outcome_scope_totals,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_bytes,
            &source.request_outcome_scope_bytes,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_response_kinds,
            &source.request_outcome_scope_response_kinds,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_response_kind_outcomes,
            &source.request_outcome_scope_response_kind_outcomes,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_policy_sources,
            &source.request_outcome_scope_policy_sources,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_policy_source_outcomes,
            &source.request_outcome_scope_policy_source_outcomes,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_route_action_families,
            &source.request_outcome_scope_route_action_families,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_route_action_family_outcomes,
            &source.request_outcome_scope_route_action_family_outcomes,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_outcomes,
            &source.request_outcome_scope_outcomes,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_scope_outcome_bytes,
            &source.request_outcome_scope_outcome_bytes,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_lane_totals,
            &source.request_outcome_lane_totals,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_lane_bytes,
            &source.request_outcome_lane_bytes,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_lane_response_kinds,
            &source.request_outcome_lane_response_kinds,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_lane_outcomes,
            &source.request_outcome_lane_outcomes,
        );
        Self::merge_count_maps(
            &mut self.request_outcome_lane_outcome_bytes,
            &source.request_outcome_lane_outcome_bytes,
        );
    }

    fn finalize(self, now: u64, hours: u64, top_limit: usize, start_hour: u64, end_hour: u64) -> MonitoringSummary {
        let live_origin = "live";
        let challenge_total = self
            .challenge_totals_by_origin
            .get(live_origin)
            .copied()
            .unwrap_or(0);
        let challenge_ip_counts = self
            .challenge_ip_counts_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default();
        let challenge_trend = self
            .challenge_trends_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default();
        let mut challenge_reason_map = build_seeded_map(&CHALLENGE_REASON_KEYS);
        for (key, value) in self
            .challenge_reason_counts_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default()
        {
            let entry = challenge_reason_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let not_a_bot_served_total = self
            .not_a_bot_served_by_origin
            .get(live_origin)
            .copied()
            .unwrap_or(0);
        let not_a_bot_submitted_total = self
            .not_a_bot_submitted_by_origin
            .get(live_origin)
            .copied()
            .unwrap_or(0);
        let mut pow_reason_map = build_seeded_map(&POW_REASON_KEYS);
        for (key, value) in self
            .pow_reason_counts_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default()
        {
            let entry = pow_reason_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let mut not_a_bot_outcome_map = build_seeded_map(&NOT_A_BOT_OUTCOME_KEYS);
        for (key, value) in self
            .not_a_bot_outcomes_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default()
        {
            let entry = not_a_bot_outcome_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let mut not_a_bot_latency_map = build_seeded_map(&NOT_A_BOT_SOLVE_MS_BUCKET_KEYS);
        for (key, value) in self
            .not_a_bot_latency_buckets_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default()
        {
            let entry = not_a_bot_latency_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let not_a_bot_abandonments =
            not_a_bot_served_total.saturating_sub(not_a_bot_submitted_total);
        let not_a_bot_abandonment_ratio = if not_a_bot_served_total == 0 {
            0.0
        } else {
            not_a_bot_abandonments as f64 / not_a_bot_served_total as f64
        };

        let mut pow_outcome_map = build_seeded_map(&POW_OUTCOME_KEYS);
        for (key, value) in self
            .pow_outcomes_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default()
        {
            let entry = pow_outcome_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }
        let pow_outcome_failures = pow_outcome_map.get("failure").copied().unwrap_or(0);
        let pow_outcome_successes = pow_outcome_map.get("success").copied().unwrap_or(0);
        let pow_total_failures = self
            .pow_totals_by_origin
            .get(live_origin)
            .copied()
            .unwrap_or(0)
            .max(pow_outcome_failures);
        let pow_total_successes = self
            .pow_success_totals_by_origin
            .get(live_origin)
            .copied()
            .unwrap_or(0)
            .max(pow_outcome_successes);
        let pow_total_attempts = pow_total_failures.saturating_add(pow_total_successes);
        let pow_success_ratio = if pow_total_attempts == 0 {
            0.0
        } else {
            pow_total_successes as f64 / pow_total_attempts as f64
        };

        let rate_total = self
            .rate_totals_by_origin
            .get(live_origin)
            .copied()
            .unwrap_or(0);
        let rate_ip_counts = self
            .rate_ip_counts_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default();
        let rate_path_counts = self
            .rate_path_counts_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default();
        let mut rate_outcome_map = build_seeded_map(&RATE_OUTCOME_KEYS);
        for (key, value) in self
            .rate_outcomes_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default()
        {
            let entry = rate_outcome_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let mut geo_action_map = build_seeded_map(&GEO_ACTION_KEYS);
        for (key, value) in self
            .geo_actions_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default()
        {
            let entry = geo_action_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }
        let geo_total = self
            .geo_totals_by_origin
            .get(live_origin)
            .copied()
            .unwrap_or(0);
        let geo_countries = self
            .geo_countries_by_origin
            .get(live_origin)
            .cloned()
            .unwrap_or_default();

        let mut shadow_action_map = build_seeded_map(&SHADOW_ACTION_KEYS);
        for (key, value) in self.shadow_actions {
            let entry = shadow_action_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let mut request_outcome_scope_rows: BTreeMap<(String, String, String), RequestOutcomeScopeSummaryRow> =
            BTreeMap::new();
        for (cohort, count) in self.request_outcome_scope_totals {
            if let Some((traffic_origin, measurement_scope, execution_mode)) =
                parse_request_outcome_scope_cohort(cohort.as_str())
            {
                let row = request_outcome_scope_rows
                    .entry((
                        traffic_origin.clone(),
                        measurement_scope.clone(),
                        execution_mode.clone(),
                    ))
                    .or_insert_with(|| RequestOutcomeScopeSummaryRow {
                        traffic_origin,
                        measurement_scope,
                        execution_mode,
                        ..RequestOutcomeScopeSummaryRow::default()
                    });
                row.total_requests = row.total_requests.saturating_add(count);
            }
        }
        for (cohort, count) in self.request_outcome_scope_bytes {
            if let Some((traffic_origin, measurement_scope, execution_mode)) =
                parse_request_outcome_scope_cohort(cohort.as_str())
            {
                let row = request_outcome_scope_rows
                    .entry((
                        traffic_origin.clone(),
                        measurement_scope.clone(),
                        execution_mode.clone(),
                    ))
                    .or_insert_with(|| RequestOutcomeScopeSummaryRow {
                        traffic_origin,
                        measurement_scope,
                        execution_mode,
                        ..RequestOutcomeScopeSummaryRow::default()
                    });
                row.response_bytes = row.response_bytes.saturating_add(count);
            }
        }
        for (nested_cohort, count) in self.request_outcome_scope_outcomes {
            if let Some((scope_cohort, outcome_class)) =
                split_last_cohort_segment(nested_cohort.as_str())
            {
                if let Some((traffic_origin, measurement_scope, execution_mode)) =
                    parse_request_outcome_scope_cohort(scope_cohort)
                {
                    let row = request_outcome_scope_rows
                        .entry((
                            traffic_origin.clone(),
                            measurement_scope.clone(),
                            execution_mode.clone(),
                        ))
                        .or_insert_with(|| RequestOutcomeScopeSummaryRow {
                            traffic_origin,
                            measurement_scope,
                            execution_mode,
                            ..RequestOutcomeScopeSummaryRow::default()
                        });
                    match outcome_class {
                        "forwarded" => {
                            row.forwarded_requests = row.forwarded_requests.saturating_add(count)
                        }
                        "short_circuited" => {
                            row.short_circuited_requests =
                                row.short_circuited_requests.saturating_add(count)
                        }
                        "control_response" => {
                            row.control_response_requests =
                                row.control_response_requests.saturating_add(count)
                        }
                        _ => {}
                    }
                }
            }
        }
        for (nested_cohort, count) in self.request_outcome_scope_outcome_bytes {
            if let Some((scope_cohort, outcome_class)) =
                split_last_cohort_segment(nested_cohort.as_str())
            {
                if let Some((traffic_origin, measurement_scope, execution_mode)) =
                    parse_request_outcome_scope_cohort(scope_cohort)
                {
                    let row = request_outcome_scope_rows
                        .entry((
                            traffic_origin.clone(),
                            measurement_scope.clone(),
                            execution_mode.clone(),
                        ))
                        .or_insert_with(|| RequestOutcomeScopeSummaryRow {
                            traffic_origin,
                            measurement_scope,
                            execution_mode,
                            ..RequestOutcomeScopeSummaryRow::default()
                        });
                    match outcome_class {
                        "forwarded" => {
                            row.forwarded_response_bytes =
                                row.forwarded_response_bytes.saturating_add(count)
                        }
                        "short_circuited" => {
                            row.short_circuited_response_bytes =
                                row.short_circuited_response_bytes.saturating_add(count)
                        }
                        "control_response" => {
                            row.control_response_bytes =
                                row.control_response_bytes.saturating_add(count)
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut request_outcome_lane_rows: BTreeMap<
            (String, String, String, String, String, String),
            RequestOutcomeLaneSummaryRow,
        > = BTreeMap::new();
        for (cohort, count) in self.request_outcome_lane_totals {
            if let Some((traffic_origin, measurement_scope, execution_mode, lane, exactness, basis)) =
                parse_request_outcome_lane_cohort(cohort.as_str())
            {
                let row = request_outcome_lane_rows
                    .entry((
                        traffic_origin.clone(),
                        measurement_scope.clone(),
                        execution_mode.clone(),
                        lane.clone(),
                        exactness.clone(),
                        basis.clone(),
                    ))
                    .or_insert_with(|| RequestOutcomeLaneSummaryRow {
                        traffic_origin,
                        measurement_scope,
                        execution_mode,
                        lane,
                        exactness,
                        basis,
                        ..RequestOutcomeLaneSummaryRow::default()
                    });
                row.total_requests = row.total_requests.saturating_add(count);
            }
        }
        for (cohort, count) in self.request_outcome_lane_bytes {
            if let Some((traffic_origin, measurement_scope, execution_mode, lane, exactness, basis)) =
                parse_request_outcome_lane_cohort(cohort.as_str())
            {
                let row = request_outcome_lane_rows
                    .entry((
                        traffic_origin.clone(),
                        measurement_scope.clone(),
                        execution_mode.clone(),
                        lane.clone(),
                        exactness.clone(),
                        basis.clone(),
                    ))
                    .or_insert_with(|| RequestOutcomeLaneSummaryRow {
                        traffic_origin,
                        measurement_scope,
                        execution_mode,
                        lane,
                        exactness,
                        basis,
                        ..RequestOutcomeLaneSummaryRow::default()
                    });
                row.response_bytes = row.response_bytes.saturating_add(count);
            }
        }
        for (nested_cohort, count) in self.request_outcome_lane_outcomes {
            if let Some((lane_cohort, outcome_class)) =
                split_last_cohort_segment(nested_cohort.as_str())
            {
                if let Some((traffic_origin, measurement_scope, execution_mode, lane, exactness, basis)) =
                    parse_request_outcome_lane_cohort(lane_cohort)
                {
                    let row = request_outcome_lane_rows
                        .entry((
                            traffic_origin.clone(),
                            measurement_scope.clone(),
                            execution_mode.clone(),
                            lane.clone(),
                            exactness.clone(),
                            basis.clone(),
                        ))
                        .or_insert_with(|| RequestOutcomeLaneSummaryRow {
                            traffic_origin,
                            measurement_scope,
                            execution_mode,
                            lane,
                            exactness,
                            basis,
                            ..RequestOutcomeLaneSummaryRow::default()
                        });
                    match outcome_class {
                        "forwarded" => {
                            row.forwarded_requests = row.forwarded_requests.saturating_add(count)
                        }
                        "short_circuited" => {
                            row.short_circuited_requests =
                                row.short_circuited_requests.saturating_add(count)
                        }
                        "control_response" => {
                            row.control_response_requests =
                                row.control_response_requests.saturating_add(count)
                        }
                        _ => {}
                    }
                }
            }
        }
        for (nested_cohort, count) in self.request_outcome_lane_outcome_bytes {
            if let Some((lane_cohort, outcome_class)) =
                split_last_cohort_segment(nested_cohort.as_str())
            {
                if let Some((traffic_origin, measurement_scope, execution_mode, lane, exactness, basis)) =
                    parse_request_outcome_lane_cohort(lane_cohort)
                {
                    let row = request_outcome_lane_rows
                        .entry((
                            traffic_origin.clone(),
                            measurement_scope.clone(),
                            execution_mode.clone(),
                            lane.clone(),
                            exactness.clone(),
                            basis.clone(),
                        ))
                        .or_insert_with(|| RequestOutcomeLaneSummaryRow {
                            traffic_origin,
                            measurement_scope,
                            execution_mode,
                            lane,
                            exactness,
                            basis,
                            ..RequestOutcomeLaneSummaryRow::default()
                        });
                    match outcome_class {
                        "forwarded" => {
                            row.forwarded_response_bytes =
                                row.forwarded_response_bytes.saturating_add(count)
                        }
                        "short_circuited" => {
                            row.short_circuited_response_bytes =
                                row.short_circuited_response_bytes.saturating_add(count)
                        }
                        "control_response" => {
                            row.control_response_bytes =
                                row.control_response_bytes.saturating_add(count)
                        }
                        _ => {}
                    }
                }
            }
        }

        let build_request_outcome_breakdown_rows =
            |totals: HashMap<String, u64>,
             outcome_counts: HashMap<String, u64>|
             -> Vec<RequestOutcomeBreakdownSummaryRow> {
                let mut rows: BTreeMap<
                    (String, String, String, String),
                    RequestOutcomeBreakdownSummaryRow,
                > = BTreeMap::new();

                for (cohort, count) in totals {
                    if let Some((traffic_origin, measurement_scope, execution_mode, value)) =
                        parse_request_outcome_scope_breakdown_cohort(cohort.as_str())
                    {
                        let row = rows
                            .entry((
                                traffic_origin.clone(),
                                measurement_scope.clone(),
                                execution_mode.clone(),
                                value.clone(),
                            ))
                            .or_insert_with(|| RequestOutcomeBreakdownSummaryRow {
                                traffic_origin,
                                measurement_scope,
                                execution_mode,
                                value,
                                ..RequestOutcomeBreakdownSummaryRow::default()
                            });
                        row.total_requests = row.total_requests.saturating_add(count);
                    }
                }

                for (nested_cohort, count) in outcome_counts {
                    if let Some((traffic_origin, measurement_scope, execution_mode, value, outcome_class)) =
                        parse_request_outcome_scope_breakdown_outcome_cohort(
                            nested_cohort.as_str(),
                        )
                    {
                        let row = rows
                            .entry((
                                traffic_origin.clone(),
                                measurement_scope.clone(),
                                execution_mode.clone(),
                                value.clone(),
                            ))
                            .or_insert_with(|| RequestOutcomeBreakdownSummaryRow {
                                traffic_origin,
                                measurement_scope,
                                execution_mode,
                                value,
                                ..RequestOutcomeBreakdownSummaryRow::default()
                            });
                        match outcome_class.as_str() {
                            "forwarded" => {
                                row.forwarded_requests =
                                    row.forwarded_requests.saturating_add(count)
                            }
                            "short_circuited" => {
                                row.short_circuited_requests =
                                    row.short_circuited_requests.saturating_add(count)
                            }
                            "control_response" => {
                                row.control_response_requests =
                                    row.control_response_requests.saturating_add(count)
                            }
                            _ => {}
                        }
                    }
                }

                rows.into_values().collect()
            };

        let request_outcome_response_kind_rows = build_request_outcome_breakdown_rows(
            self.request_outcome_scope_response_kinds.clone(),
            self.request_outcome_scope_response_kind_outcomes.clone(),
        );
        let request_outcome_policy_source_rows = build_request_outcome_breakdown_rows(
            self.request_outcome_scope_policy_sources.clone(),
            self.request_outcome_scope_policy_source_outcomes.clone(),
        );
        let request_outcome_route_action_family_rows = build_request_outcome_breakdown_rows(
            self.request_outcome_scope_route_action_families.clone(),
            self.request_outcome_scope_route_action_family_outcomes.clone(),
        );

        let mut human_friction_rows: BTreeMap<(String, String), HumanFrictionSegmentRow> =
            BTreeMap::new();

        for row in request_outcome_lane_rows.values() {
            if row.traffic_origin != "live" || row.measurement_scope != "ingress_primary" {
                continue;
            }

            match row.lane.as_str() {
                "likely_human" | "unknown_interactive" => {
                    let segment_row = human_friction_rows
                        .entry((row.execution_mode.clone(), row.lane.clone()))
                        .or_insert_with(|| HumanFrictionSegmentRow {
                            execution_mode: row.execution_mode.clone(),
                            segment: row.lane.clone(),
                            ..HumanFrictionSegmentRow::default()
                        });
                    segment_row.denominator_requests = segment_row
                        .denominator_requests
                        .saturating_add(row.total_requests);

                    let interactive_row = human_friction_rows
                        .entry((row.execution_mode.clone(), "interactive".to_string()))
                        .or_insert_with(|| HumanFrictionSegmentRow {
                            execution_mode: row.execution_mode.clone(),
                            segment: "interactive".to_string(),
                            ..HumanFrictionSegmentRow::default()
                        });
                    interactive_row.denominator_requests = interactive_row
                        .denominator_requests
                        .saturating_add(row.total_requests);
                }
                _ => {}
            }
        }

        for (cohort, count) in &self.request_outcome_lane_response_kinds {
            let Some((traffic_origin, measurement_scope, execution_mode, lane, _exactness, _basis, value)) =
                parse_request_outcome_lane_breakdown_cohort(cohort.as_str())
            else {
                continue;
            };
            if traffic_origin != "live" || measurement_scope != "ingress_primary" {
                continue;
            }

            let apply_count = |row: &mut HumanFrictionSegmentRow, response_kind: &str, count: u64| {
                match response_kind {
                    "not_a_bot" => {
                        row.not_a_bot_requests = row.not_a_bot_requests.saturating_add(count)
                    }
                    "challenge" => {
                        row.challenge_requests = row.challenge_requests.saturating_add(count)
                    }
                    "js_challenge" => {
                        row.js_challenge_requests =
                            row.js_challenge_requests.saturating_add(count)
                    }
                    "maze" => row.maze_requests = row.maze_requests.saturating_add(count),
                    _ => return,
                }
                row.friction_requests = row
                    .not_a_bot_requests
                    .saturating_add(row.challenge_requests)
                    .saturating_add(row.js_challenge_requests)
                    .saturating_add(row.maze_requests);
            };

            match lane.as_str() {
                "likely_human" | "unknown_interactive" => {
                    let row = human_friction_rows
                        .entry((execution_mode.clone(), lane.clone()))
                        .or_insert_with(|| HumanFrictionSegmentRow {
                            execution_mode: execution_mode.clone(),
                            segment: lane.clone(),
                            ..HumanFrictionSegmentRow::default()
                        });
                    apply_count(row, value.as_str(), *count);

                    let interactive_row = human_friction_rows
                        .entry((execution_mode.clone(), "interactive".to_string()))
                        .or_insert_with(|| HumanFrictionSegmentRow {
                            execution_mode: execution_mode.clone(),
                            segment: "interactive".to_string(),
                            ..HumanFrictionSegmentRow::default()
                        });
                    apply_count(interactive_row, value.as_str(), *count);
                }
                _ => {}
            }
        }

        for row in human_friction_rows.values_mut() {
            if row.denominator_requests == 0 {
                continue;
            }
            let denominator = row.denominator_requests as f64;
            row.not_a_bot_rate = row.not_a_bot_requests as f64 / denominator;
            row.challenge_rate = row.challenge_requests as f64 / denominator;
            row.js_challenge_rate = row.js_challenge_requests as f64 / denominator;
            row.maze_rate = row.maze_requests as f64 / denominator;
            row.friction_rate = row.friction_requests as f64 / denominator;
        }

        let response_kind_total_for_mode =
            |execution_mode: &str, response_kind: &str| -> u64 {
                request_outcome_response_kind_rows
                    .iter()
                    .find(|row| {
                        row.traffic_origin == "live"
                            && row.measurement_scope == "ingress_primary"
                            && row.execution_mode == execution_mode
                            && row.value == response_kind
                    })
                    .map(|row| row.total_requests)
                    .unwrap_or(0)
            };

        let likely_human_friction_for_mode =
            |execution_mode: &str, family: &str| -> u64 {
                human_friction_rows
                    .get(&(execution_mode.to_string(), "likely_human".to_string()))
                    .map(|row| match family {
                        "not_a_bot" => row.not_a_bot_requests,
                        "challenge" => row.challenge_requests,
                        "js_challenge" => row.js_challenge_requests,
                        "maze" => row.maze_requests,
                        _ => 0,
                    })
                    .unwrap_or(0)
            };

        let mut defence_funnel_rows: BTreeMap<(String, String), DefenceFunnelRow> = BTreeMap::new();
        let not_a_bot_metrics_present = not_a_bot_served_total > 0
            || not_a_bot_submitted_total > 0
            || not_a_bot_outcome_map.values().any(|count| *count > 0);

        for execution_mode in ["enforced", "shadow"] {
            let not_a_bot_triggered = response_kind_total_for_mode(execution_mode, "not_a_bot");
            if not_a_bot_triggered > 0
                || (execution_mode == "enforced" && not_a_bot_metrics_present)
            {
                let row = defence_funnel_rows
                    .entry((execution_mode.to_string(), "not_a_bot".to_string()))
                    .or_insert_with(|| DefenceFunnelRow {
                        execution_mode: execution_mode.to_string(),
                        family: "not_a_bot".to_string(),
                        ..DefenceFunnelRow::default()
                    });
                row.candidate_requests = Some(not_a_bot_triggered);
                row.triggered_requests = Some(not_a_bot_triggered);
                row.friction_requests = Some(not_a_bot_triggered);
                row.likely_human_affected_requests =
                    Some(likely_human_friction_for_mode(execution_mode, "not_a_bot"));
                if execution_mode == "enforced" {
                    row.passed_requests = Some(*not_a_bot_outcome_map.get("pass").unwrap_or(&0));
                    row.failed_requests = Some(*not_a_bot_outcome_map.get("fail").unwrap_or(&0));
                    row.escalated_requests =
                        Some(*not_a_bot_outcome_map.get("escalate").unwrap_or(&0));
                }
            }

            let challenge_triggered = response_kind_total_for_mode(execution_mode, "challenge");
            if challenge_triggered > 0 || (execution_mode == "enforced" && challenge_total > 0) {
                let row = defence_funnel_rows
                    .entry((execution_mode.to_string(), "challenge".to_string()))
                    .or_insert_with(|| DefenceFunnelRow {
                        execution_mode: execution_mode.to_string(),
                        family: "challenge".to_string(),
                        ..DefenceFunnelRow::default()
                    });
                row.candidate_requests = Some(challenge_triggered);
                row.triggered_requests = Some(challenge_triggered);
                row.friction_requests = Some(challenge_triggered);
                row.likely_human_affected_requests =
                    Some(likely_human_friction_for_mode(execution_mode, "challenge"));
                if execution_mode == "enforced" {
                    row.failed_requests = Some(challenge_total);
                }
            }

            let js_challenge_triggered =
                response_kind_total_for_mode(execution_mode, "js_challenge");
            if js_challenge_triggered > 0 {
                let row = defence_funnel_rows
                    .entry((execution_mode.to_string(), "js_challenge".to_string()))
                    .or_insert_with(|| DefenceFunnelRow {
                        execution_mode: execution_mode.to_string(),
                        family: "js_challenge".to_string(),
                        ..DefenceFunnelRow::default()
                    });
                row.candidate_requests = Some(js_challenge_triggered);
                row.triggered_requests = Some(js_challenge_triggered);
                row.friction_requests = Some(js_challenge_triggered);
                row.likely_human_affected_requests =
                    Some(likely_human_friction_for_mode(execution_mode, "js_challenge"));
            }

            let maze_triggered = response_kind_total_for_mode(execution_mode, "maze");
            if maze_triggered > 0 {
                let row = defence_funnel_rows
                    .entry((execution_mode.to_string(), "maze".to_string()))
                    .or_insert_with(|| DefenceFunnelRow {
                        execution_mode: execution_mode.to_string(),
                        family: "maze".to_string(),
                        ..DefenceFunnelRow::default()
                    });
                row.candidate_requests = Some(maze_triggered);
                row.triggered_requests = Some(maze_triggered);
                row.friction_requests = Some(maze_triggered);
                row.likely_human_affected_requests =
                    Some(likely_human_friction_for_mode(execution_mode, "maze"));
            }
        }

        if pow_total_attempts > 0 {
            let row = defence_funnel_rows
                .entry(("enforced".to_string(), "pow".to_string()))
                .or_insert_with(|| DefenceFunnelRow {
                    execution_mode: "enforced".to_string(),
                    family: "pow".to_string(),
                    ..DefenceFunnelRow::default()
                });
            row.candidate_requests = Some(pow_total_attempts);
            row.triggered_requests = Some(pow_total_attempts);
            row.friction_requests = Some(pow_total_attempts);
            row.passed_requests = Some(pow_total_successes);
            row.failed_requests = Some(pow_total_failures);
        }

        MonitoringSummary {
            generated_at: now,
            hours,
            shadow: ShadowSummary {
                total_actions: self.shadow_total,
                pass_through_total: self.shadow_pass_through_total,
                actions: shadow_action_map,
            },
            honeypot: HoneypotSummary {
                total_hits: self.honeypot_total,
                unique_crawlers: self.honeypot_ip_counts.len() as u64,
                top_crawlers: top_entries(&self.honeypot_ip_counts, top_limit),
                top_paths: top_entries(&self.honeypot_path_counts, top_limit),
            },
            challenge: FailureSummary {
                total_failures: challenge_total,
                unique_offenders: challenge_ip_counts.len() as u64,
                top_offenders: top_entries(&challenge_ip_counts, top_limit),
                reasons: challenge_reason_map,
                trend: build_trend(start_hour, end_hour, &CHALLENGE_REASON_KEYS, challenge_trend),
            },
            not_a_bot: NotABotSummary {
                served: not_a_bot_served_total,
                submitted: not_a_bot_submitted_total,
                pass: *not_a_bot_outcome_map.get("pass").unwrap_or(&0),
                escalate: *not_a_bot_outcome_map.get("escalate").unwrap_or(&0),
                fail: *not_a_bot_outcome_map.get("fail").unwrap_or(&0),
                replay: *not_a_bot_outcome_map.get("replay").unwrap_or(&0),
                outcomes: not_a_bot_outcome_map,
                solve_latency_buckets: not_a_bot_latency_map,
                abandonments_estimated: not_a_bot_abandonments,
                abandonment_ratio: not_a_bot_abandonment_ratio,
            },
            pow: PowSummary {
                total_failures: pow_total_failures,
                total_successes: pow_total_successes,
                total_attempts: pow_total_attempts,
                success_ratio: pow_success_ratio,
                unique_offenders: self
                    .pow_ip_counts_by_origin
                    .get(live_origin)
                    .map(|row| row.len() as u64)
                    .unwrap_or(0),
                top_offenders: top_entries(
                    &self
                        .pow_ip_counts_by_origin
                        .get(live_origin)
                        .cloned()
                        .unwrap_or_default(),
                    top_limit,
                ),
                reasons: pow_reason_map,
                outcomes: pow_outcome_map,
                trend: build_trend(
                    start_hour,
                    end_hour,
                    &POW_REASON_KEYS,
                    self.pow_trends_by_origin
                        .get(live_origin)
                        .cloned()
                        .unwrap_or_default(),
                ),
            },
            rate: RateSummary {
                total_violations: rate_total,
                unique_offenders: rate_ip_counts.len() as u64,
                top_offenders: top_entries(&rate_ip_counts, top_limit),
                top_paths: top_entries(&rate_path_counts, top_limit),
                outcomes: rate_outcome_map,
            },
            geo: GeoSummary {
                total_violations: geo_total,
                actions: geo_action_map,
                top_countries: top_entries(&geo_countries, top_limit),
            },
            human_friction: HumanFrictionSummary {
                segments: human_friction_rows.into_values().collect(),
            },
            defence_funnel: DefenceFunnelSummary {
                rows: defence_funnel_rows.into_values().collect(),
            },
            request_outcomes: RequestOutcomeSummary {
                by_scope: request_outcome_scope_rows.into_values().collect(),
                by_lane: request_outcome_lane_rows.into_values().collect(),
                by_response_kind: request_outcome_response_kind_rows,
                by_policy_source: request_outcome_policy_source_rows,
                by_route_action_family: request_outcome_route_action_family_rows,
            },
        }
    }
}

fn monitoring_day_rollup_key(day_start_hour: u64) -> String {
    format!("{MONITORING_ROLLUP_KEY_PREFIX}:{day_start_hour}")
}

fn day_start_hour(hour: u64) -> u64 {
    hour.saturating_sub(hour % MONITORING_DAY_HOURS)
}

fn load_monitoring_day_rollup<S: crate::challenge::KeyValueStore>(
    store: &S,
    day_start_hour: u64,
) -> Option<MonitoringDayRollup> {
    store
        .get(monitoring_day_rollup_key(day_start_hour).as_str())
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<MonitoringDayRollup>(bytes.as_slice()).ok())
        .filter(|rollup| {
            rollup.schema_version == MONITORING_DAY_ROLLUP_SCHEMA_VERSION
                && rollup.day_start_hour == day_start_hour
        })
}

fn build_monitoring_day_rollup<S: crate::challenge::KeyValueStore>(
    store: &S,
    day_start_hour: u64,
    prefixes: &[&str],
) -> Option<MonitoringDayRollup> {
    let day_end_hour = day_start_hour.saturating_add(MONITORING_DAY_HOURS - 1);
    let mut accumulator = MonitoringAccumulator::default();
    let mut matched_any = false;
    for key in crate::observability::retention::bucket_window_keys(
        store,
        crate::observability::retention::RETENTION_DOMAIN_MONITORING,
        day_start_hour,
        day_end_hour,
    ) {
        let Some(prefix) = matching_monitoring_prefix(key.as_str(), prefixes) else {
            continue;
        };
        let Some((section, metric, dimension, hour)) =
            parse_monitoring_key_with_prefix(key.as_str(), prefix)
        else {
            continue;
        };
        if hour < day_start_hour || hour > day_end_hour {
            continue;
        }
        let count = read_counter(store, key.as_str());
        if count == 0 {
            continue;
        }
        matched_any = true;
        accumulator.consume_counter(
            section.as_str(),
            metric.as_str(),
            dimension.as_deref(),
            hour,
            count,
        );
    }
    if !matched_any {
        return None;
    }
    Some(MonitoringDayRollup {
        schema_version: MONITORING_DAY_ROLLUP_SCHEMA_VERSION.to_string(),
        day_start_hour,
        day_end_hour,
        accumulator,
    })
}

fn load_or_build_monitoring_day_rollup<S: crate::challenge::KeyValueStore>(
    store: &S,
    day_start_hour: u64,
    prefixes: &[&str],
) -> Option<MonitoringDayRollup> {
    if let Some(existing) = load_monitoring_day_rollup(store, day_start_hour) {
        return Some(existing);
    }
    let rollup = build_monitoring_day_rollup(store, day_start_hour, prefixes)?;
    let key = monitoring_day_rollup_key(day_start_hour);
    if let Ok(payload) = serde_json::to_vec(&rollup) {
        if store.set(key.as_str(), payload.as_slice()).is_ok() {
            crate::observability::retention::register_monitoring_rollup_key(
                store,
                day_start_hour,
                key.as_str(),
            );
            crate::observability::retention::run_worker_if_due(store);
        }
    }
    Some(rollup)
}

fn summarize_rollup_day_starts(start_hour: u64, end_hour: u64) -> Vec<u64> {
    if start_hour > end_hour {
        return Vec::new();
    }
    let mut days = Vec::new();
    let mut day = day_start_hour(start_hour.saturating_add(MONITORING_DAY_HOURS - 1));
    let current_day_start = day_start_hour(end_hour);
    while day.saturating_add(MONITORING_DAY_HOURS - 1) <= end_hour && day < current_day_start {
        if day >= start_hour {
            days.push(day);
        }
        day = day.saturating_add(MONITORING_DAY_HOURS);
    }
    days
}

fn summarize_with_store_prefixes<S: crate::challenge::KeyValueStore>(
    store: &S,
    hours: u64,
    limit: usize,
    prefixes: &[&str],
) -> MonitoringSummary {
    let now = now_ts();
    #[cfg(not(test))]
    maybe_flush_pending_counter_buffer(store, true);
    let hours = normalize_window_hours(hours);
    let top_limit = normalize_top_limit(limit);
    let end_hour = now / 3600;
    let start_hour = end_hour.saturating_sub(hours.saturating_sub(1));
    let mut accumulator = MonitoringAccumulator::default();
    let rollup_days = summarize_rollup_day_starts(start_hour, end_hour);
    for day_start in &rollup_days {
        if let Some(rollup) = load_or_build_monitoring_day_rollup(store, *day_start, prefixes) {
            accumulator.merge_rollup(&rollup.accumulator);
        }
    }

    let covered_day_hours: std::collections::HashSet<u64> = rollup_days
        .iter()
        .flat_map(|day_start| *day_start..=day_start.saturating_add(MONITORING_DAY_HOURS - 1))
        .collect();

    for key in crate::observability::retention::bucket_window_keys(
        store,
        crate::observability::retention::RETENTION_DOMAIN_MONITORING,
        start_hour,
        end_hour,
    ) {
        let Some(prefix) = matching_monitoring_prefix(key.as_str(), prefixes) else {
            continue;
        };
        let Some((section, metric, dimension, hour)) =
            parse_monitoring_key_with_prefix(key.as_str(), prefix)
        else {
            continue;
        };
        if hour < start_hour || hour > end_hour || covered_day_hours.contains(&hour) {
            continue;
        }
        let count = read_counter(store, key.as_str());
        if count == 0 {
            continue;
        }
        accumulator.consume_counter(
            section.as_str(),
            metric.as_str(),
            dimension.as_deref(),
            hour,
            count,
        );
    }

    accumulator.finalize(now, hours, top_limit, start_hour, end_hour)
}

pub(crate) fn summarize_with_store<S: crate::challenge::KeyValueStore>(
    store: &S,
    hours: u64,
    limit: usize,
) -> MonitoringSummary {
    summarize_with_store_prefixes(store, hours, limit, &[MONITORING_PREFIX])
}

pub(crate) fn summarize_metrics_window<S: crate::challenge::KeyValueStore>(
    store: &S,
) -> MonitoringSummary {
    let retention_hours = monitoring_retention_hours();
    // Keep Prometheus monitoring parity aligned with dashboard default 24h summaries.
    let hours = if retention_hours == 0 {
        24
    } else {
        retention_hours.min(24)
    };
    summarize_with_store(store, hours, MAX_TOP_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenge::KeyValueStore;
    use crate::observability::hot_read_contract::{TelemetryBasis, TelemetryExactness};
    use crate::runtime::effect_intents::{ExecutionMode, ShadowAction};
    use crate::runtime::request_outcome::{
        RenderedRequestOutcome, RequestOutcomeClass, RequestOutcomeLane, ResponseKind,
        TrafficOrigin,
    };
    use crate::runtime::traffic_classification::{
        MeasurementScope, PolicySource, RouteActionFamily, TrafficLane,
    };
    use std::collections::HashMap;

    #[derive(Default)]
    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
        get_keys_calls: Mutex<u64>,
    }

    impl crate::challenge::KeyValueStore for MockStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let map = self.map.lock().unwrap();
            Ok(map.get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut map = self.map.lock().unwrap();
            map.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut map = self.map.lock().unwrap();
            map.remove(key);
            Ok(())
        }

        fn get_keys(&self) -> Result<Vec<String>, ()> {
            *self.get_keys_calls.lock().unwrap() += 1;
            let map = self.map.lock().unwrap();
            Ok(map.keys().cloned().collect())
        }
    }

    impl MockStore {
        fn get_keys_calls(&self) -> u64 {
            *self.get_keys_calls.lock().unwrap()
        }
    }

    fn set_counter(store: &MockStore, key: &str, value: u64) {
        store
            .set(key, value.to_string().as_bytes())
            .expect("counter write should succeed");
        if let Some((_, _, _, hour)) = parse_monitoring_key_with_prefix(key, MONITORING_PREFIX) {
            crate::observability::retention::register_monitoring_key(store, hour, key);
        }
    }

    #[test]
    fn record_request_outcome_records_origin_scope_outcome_and_lane_counters() {
        let store = MockStore::default();
        let hour = now_ts() / 3600;
        let outcome = RenderedRequestOutcome {
            traffic_origin: TrafficOrigin::Live,
            measurement_scope: MeasurementScope::IngressPrimary,
            route_action_family: RouteActionFamily::PublicContent,
            execution_mode: ExecutionMode::Enforced,
            traffic_lane: Some(RequestOutcomeLane {
                lane: TrafficLane::LikelyHuman,
                exactness: TelemetryExactness::Exact,
                basis: TelemetryBasis::Observed,
            }),
            outcome_class: RequestOutcomeClass::Forwarded,
            response_kind: ResponseKind::ForwardAllow,
            http_status: 200,
            response_bytes: 321,
            forward_attempted: true,
            forward_failure_class: None,
            intended_action: None,
            policy_source: PolicySource::CleanAllow,
        };

        record_request_outcome(&store, &outcome);

        let scope_cohort = request_outcome_scope_cohort(&outcome);
        let lane_cohort = request_outcome_lane_cohort(&outcome).expect("lane cohort");
        let key = |metric: &str, dimension: &str| {
            monitoring_key("request_outcome", metric, Some(dimension), hour)
        };

        assert_eq!(read_counter(&store, key("total", scope_cohort.as_str()).as_str()), 1);
        assert_eq!(
            read_counter(
                &store,
                key(
                    "outcome_class",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "forwarded").as_str(),
                )
                .as_str(),
            ),
            1
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "response_kind",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "forward_allow").as_str(),
                )
                .as_str(),
            ),
            1
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "policy_source",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "clean_allow").as_str(),
                )
                .as_str(),
            ),
            1
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "route_action_family",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "public_content").as_str(),
                )
                .as_str(),
            ),
            1
        );
        assert_eq!(
            read_counter(&store, key("response_bytes", scope_cohort.as_str()).as_str()),
            321
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "outcome_class_response_bytes",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "forwarded").as_str(),
                )
                .as_str(),
            ),
            321
        );
        assert_eq!(read_counter(&store, key("lane_total", lane_cohort.as_str()).as_str()), 1);
        assert_eq!(
            read_counter(
                &store,
                key(
                    "lane_outcome_class",
                    request_outcome_nested_cohort(lane_cohort.as_str(), "forwarded").as_str(),
                )
                .as_str(),
            ),
            1
        );
        assert_eq!(
            read_counter(
                &store,
                key("lane_response_bytes", lane_cohort.as_str()).as_str(),
            ),
            321
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "lane_outcome_class_response_bytes",
                    request_outcome_nested_cohort(lane_cohort.as_str(), "forwarded").as_str(),
                )
                .as_str(),
            ),
            321
        );
    }

    #[test]
    fn record_request_outcome_keeps_adversary_sim_origin_separate_without_live_lane_inference() {
        let store = MockStore::default();
        let hour = now_ts() / 3600;
        let outcome = RenderedRequestOutcome {
            traffic_origin: TrafficOrigin::AdversarySim,
            measurement_scope: MeasurementScope::Excluded,
            route_action_family: RouteActionFamily::SimPublic,
            execution_mode: ExecutionMode::Enforced,
            traffic_lane: None,
            outcome_class: RequestOutcomeClass::ShortCircuited,
            response_kind: ResponseKind::SimPublicResponse,
            http_status: 200,
            response_bytes: 77,
            forward_attempted: false,
            forward_failure_class: None,
            intended_action: None,
            policy_source: PolicySource::SimPublic,
        };

        record_request_outcome(&store, &outcome);

        let scope_cohort = request_outcome_scope_cohort(&outcome);
        let key = |metric: &str, dimension: &str| {
            monitoring_key("request_outcome", metric, Some(dimension), hour)
        };

        assert_eq!(
            read_counter(&store, key("total", scope_cohort.as_str()).as_str()),
            1
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "outcome_class",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "short_circuited").as_str(),
                )
                .as_str(),
            ),
            1
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "response_kind",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "sim_public_response").as_str(),
                )
                .as_str(),
            ),
            1
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "policy_source",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "sim_public").as_str(),
                )
                .as_str(),
            ),
            1
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "route_action_family",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "sim_public").as_str(),
                )
                .as_str(),
            ),
            1
        );
        assert_eq!(
            read_counter(
                &store,
                key("response_bytes", scope_cohort.as_str()).as_str(),
            ),
            77
        );
        assert_eq!(
            read_counter(
                &store,
                key(
                    "outcome_class_response_bytes",
                    request_outcome_nested_cohort(scope_cohort.as_str(), "short_circuited").as_str(),
                )
                .as_str(),
            ),
            77
        );
    }

    #[test]
    fn record_request_outcome_records_lane_response_kind_counters_for_lane_backed_requests() {
        let store = MockStore::default();
        let hour = now_ts() / 3600;
        let outcome = RenderedRequestOutcome {
            traffic_origin: TrafficOrigin::Live,
            measurement_scope: MeasurementScope::IngressPrimary,
            route_action_family: RouteActionFamily::PublicContent,
            execution_mode: ExecutionMode::Enforced,
            traffic_lane: Some(RequestOutcomeLane {
                lane: TrafficLane::LikelyHuman,
                exactness: TelemetryExactness::Exact,
                basis: TelemetryBasis::Observed,
            }),
            outcome_class: RequestOutcomeClass::ShortCircuited,
            response_kind: ResponseKind::NotABot,
            http_status: 200,
            response_bytes: 111,
            forward_attempted: false,
            forward_failure_class: None,
            intended_action: None,
            policy_source: PolicySource::PolicyGraphSecondTranche,
        };

        record_request_outcome(&store, &outcome);

        let lane_cohort = request_outcome_lane_cohort(&outcome).expect("lane cohort");
        let key = |metric: &str, dimension: &str| {
            monitoring_key("request_outcome", metric, Some(dimension), hour)
        };

        assert_eq!(
            read_counter(
                &store,
                key(
                    "lane_response_kind",
                    request_outcome_nested_cohort(lane_cohort.as_str(), "not_a_bot").as_str(),
                )
                .as_str(),
            ),
            1
        );
    }

    #[test]
    fn summarize_exposes_compact_request_outcome_scope_and_lane_rows() {
        let store = MockStore::default();

        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: TelemetryExactness::Exact,
                    basis: TelemetryBasis::Observed,
                }),
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 321,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::Excluded,
                route_action_family: RouteActionFamily::SimPublic,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::SimPublicResponse,
                http_status: 200,
                response_bytes: 77,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::SimPublic,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::BypassAndControl,
                route_action_family: RouteActionFamily::ControlPlane,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: None,
                outcome_class: RequestOutcomeClass::ControlResponse,
                response_kind: ResponseKind::ControlPlaneResponse,
                http_status: 500,
                response_bytes: 9,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::BootstrapFailure,
            },
        );

        let summary = summarize_with_store(&store, 24, 10);
        assert_eq!(summary.request_outcomes.by_scope.len(), 3);
        assert_eq!(summary.request_outcomes.by_lane.len(), 1);

        let live_scope = summary
            .request_outcomes
            .by_scope
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "ingress_primary"
                    && row.execution_mode == "enforced"
            })
            .expect("live scope row");
        assert_eq!(live_scope.total_requests, 1);
        assert_eq!(live_scope.forwarded_requests, 1);
        assert_eq!(live_scope.short_circuited_requests, 0);
        assert_eq!(live_scope.control_response_requests, 0);
        assert_eq!(live_scope.response_bytes, 321);
        assert_eq!(live_scope.forwarded_response_bytes, 321);
        assert_eq!(live_scope.short_circuited_response_bytes, 0);
        assert_eq!(live_scope.control_response_bytes, 0);

        let sim_scope = summary
            .request_outcomes
            .by_scope
            .iter()
            .find(|row| {
                row.traffic_origin == "adversary_sim"
                    && row.measurement_scope == "excluded"
                    && row.execution_mode == "enforced"
            })
            .expect("sim scope row");
        assert_eq!(sim_scope.total_requests, 1);
        assert_eq!(sim_scope.forwarded_requests, 0);
        assert_eq!(sim_scope.short_circuited_requests, 1);
        assert_eq!(sim_scope.control_response_requests, 0);
        assert_eq!(sim_scope.response_bytes, 77);
        assert_eq!(sim_scope.forwarded_response_bytes, 0);
        assert_eq!(sim_scope.short_circuited_response_bytes, 77);
        assert_eq!(sim_scope.control_response_bytes, 0);

        let control_scope = summary
            .request_outcomes
            .by_scope
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "bypass_and_control"
                    && row.execution_mode == "enforced"
            })
            .expect("control scope row");
        assert_eq!(control_scope.total_requests, 1);
        assert_eq!(control_scope.forwarded_requests, 0);
        assert_eq!(control_scope.short_circuited_requests, 0);
        assert_eq!(control_scope.control_response_requests, 1);
        assert_eq!(control_scope.response_bytes, 9);
        assert_eq!(control_scope.forwarded_response_bytes, 0);
        assert_eq!(control_scope.short_circuited_response_bytes, 0);
        assert_eq!(control_scope.control_response_bytes, 9);

        let live_lane = summary
            .request_outcomes
            .by_lane
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "ingress_primary"
                    && row.execution_mode == "enforced"
                    && row.lane == "likely_human"
            })
            .expect("live lane row");
        assert_eq!(live_lane.exactness, "exact");
        assert_eq!(live_lane.basis, "observed");
        assert_eq!(live_lane.total_requests, 1);
        assert_eq!(live_lane.forwarded_requests, 1);
        assert_eq!(live_lane.short_circuited_requests, 0);
        assert_eq!(live_lane.control_response_requests, 0);
        assert_eq!(live_lane.response_bytes, 321);
        assert_eq!(live_lane.forwarded_response_bytes, 321);
        assert_eq!(live_lane.short_circuited_response_bytes, 0);
        assert_eq!(live_lane.control_response_bytes, 0);
    }

    #[test]
    fn summarize_exposes_request_outcome_breakdown_rows_for_benchmark_dimensions() {
        let store = MockStore::default();

        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: TelemetryExactness::Exact,
                    basis: TelemetryBasis::Observed,
                }),
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 321,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::Excluded,
                route_action_family: RouteActionFamily::SimPublic,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::SimPublicResponse,
                http_status: 200,
                response_bytes: 77,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::SimPublic,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::BypassAndControl,
                route_action_family: RouteActionFamily::ControlPlane,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: None,
                outcome_class: RequestOutcomeClass::ControlResponse,
                response_kind: ResponseKind::ControlPlaneResponse,
                http_status: 500,
                response_bytes: 9,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::BootstrapFailure,
            },
        );

        let summary = summarize_with_store(&store, 24, 10);
        assert_eq!(summary.request_outcomes.by_response_kind.len(), 3);
        assert_eq!(summary.request_outcomes.by_policy_source.len(), 3);
        assert_eq!(summary.request_outcomes.by_route_action_family.len(), 3);

        let forward_allow = summary
            .request_outcomes
            .by_response_kind
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "ingress_primary"
                    && row.execution_mode == "enforced"
                    && row.value == "forward_allow"
            })
            .expect("forward allow breakdown");
        assert_eq!(forward_allow.total_requests, 1);
        assert_eq!(forward_allow.forwarded_requests, 1);
        assert_eq!(forward_allow.short_circuited_requests, 0);
        assert_eq!(forward_allow.control_response_requests, 0);

        let sim_public = summary
            .request_outcomes
            .by_policy_source
            .iter()
            .find(|row| {
                row.traffic_origin == "adversary_sim"
                    && row.measurement_scope == "excluded"
                    && row.execution_mode == "enforced"
                    && row.value == "sim_public"
            })
            .expect("sim public policy source");
        assert_eq!(sim_public.total_requests, 1);
        assert_eq!(sim_public.forwarded_requests, 0);
        assert_eq!(sim_public.short_circuited_requests, 1);
        assert_eq!(sim_public.control_response_requests, 0);

        let control_plane = summary
            .request_outcomes
            .by_route_action_family
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "bypass_and_control"
                    && row.execution_mode == "enforced"
                    && row.value == "control_plane"
            })
            .expect("control-plane route family");
        assert_eq!(control_plane.total_requests, 1);
        assert_eq!(control_plane.forwarded_requests, 0);
        assert_eq!(control_plane.short_circuited_requests, 0);
        assert_eq!(control_plane.control_response_requests, 1);
    }

    #[test]
    fn summarize_derives_human_friction_segments_from_lane_denominators() {
        let store = MockStore::default();

        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: TelemetryExactness::Exact,
                    basis: TelemetryBasis::Observed,
                }),
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 321,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: TelemetryExactness::Exact,
                    basis: TelemetryBasis::Observed,
                }),
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 111,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::UnknownInteractive,
                    exactness: TelemetryExactness::Derived,
                    basis: TelemetryBasis::Residual,
                }),
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 222,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::UnknownInteractive,
                    exactness: TelemetryExactness::Derived,
                    basis: TelemetryBasis::Residual,
                }),
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::Challenge,
                http_status: 200,
                response_bytes: 95,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Shadow,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::UnknownInteractive,
                    exactness: TelemetryExactness::Derived,
                    basis: TelemetryBasis::Residual,
                }),
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::JsChallenge,
                http_status: 200,
                response_bytes: 50,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: Some(ShadowAction::JsChallenge),
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: TelemetryExactness::Exact,
                    basis: TelemetryBasis::Observed,
                }),
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::Maze,
                http_status: 200,
                response_bytes: 77,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::SimPublic,
            },
        );

        let summary = summarize_with_store(&store, 24, 10);

        let likely_human = summary
            .human_friction
            .segments
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.segment == "likely_human")
            .expect("likely human friction row");
        assert_eq!(likely_human.denominator_requests, 2);
        assert_eq!(likely_human.not_a_bot_requests, 1);
        assert_eq!(likely_human.challenge_requests, 0);
        assert_eq!(likely_human.js_challenge_requests, 0);
        assert_eq!(likely_human.maze_requests, 0);
        assert_eq!(likely_human.friction_requests, 1);
        assert!((likely_human.not_a_bot_rate - 0.5).abs() < 0.000_001);
        assert!((likely_human.friction_rate - 0.5).abs() < 0.000_001);

        let unknown_interactive = summary
            .human_friction
            .segments
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.segment == "unknown_interactive")
            .expect("unknown interactive friction row");
        assert_eq!(unknown_interactive.denominator_requests, 2);
        assert_eq!(unknown_interactive.not_a_bot_requests, 0);
        assert_eq!(unknown_interactive.challenge_requests, 1);
        assert_eq!(unknown_interactive.js_challenge_requests, 0);
        assert_eq!(unknown_interactive.maze_requests, 0);
        assert_eq!(unknown_interactive.friction_requests, 1);
        assert!((unknown_interactive.challenge_rate - 0.5).abs() < 0.000_001);
        assert!((unknown_interactive.friction_rate - 0.5).abs() < 0.000_001);

        let interactive = summary
            .human_friction
            .segments
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.segment == "interactive")
            .expect("interactive friction row");
        assert_eq!(interactive.denominator_requests, 4);
        assert_eq!(interactive.not_a_bot_requests, 1);
        assert_eq!(interactive.challenge_requests, 1);
        assert_eq!(interactive.js_challenge_requests, 0);
        assert_eq!(interactive.maze_requests, 0);
        assert_eq!(interactive.friction_requests, 2);
        assert!((interactive.not_a_bot_rate - 0.25).abs() < 0.000_001);
        assert!((interactive.challenge_rate - 0.25).abs() < 0.000_001);
        assert!((interactive.friction_rate - 0.5).abs() < 0.000_001);

        let shadow_interactive = summary
            .human_friction
            .segments
            .iter()
            .find(|row| row.execution_mode == "shadow" && row.segment == "interactive")
            .expect("shadow interactive friction row");
        assert_eq!(shadow_interactive.denominator_requests, 1);
        assert_eq!(shadow_interactive.js_challenge_requests, 1);
        assert_eq!(shadow_interactive.friction_requests, 1);
        assert!((shadow_interactive.js_challenge_rate - 1.0).abs() < 0.000_001);
    }

    #[test]
    fn summarize_exposes_normalized_defence_funnel_rows_for_supported_families() {
        let store = MockStore::default();

        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: TelemetryExactness::Exact,
                    basis: TelemetryBasis::Observed,
                }),
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 111,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::UnknownInteractive,
                    exactness: TelemetryExactness::Derived,
                    basis: TelemetryBasis::Residual,
                }),
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::Challenge,
                http_status: 200,
                response_bytes: 95,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_challenge_failure(&store, "198.51.100.30", "incorrect");

        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Shadow,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: TelemetryExactness::Exact,
                    basis: TelemetryBasis::Observed,
                }),
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::JsChallenge,
                http_status: 200,
                response_bytes: 50,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: Some(ShadowAction::JsChallenge),
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_not_a_bot_served(&store);
        record_not_a_bot_submit(&store, "pass", Some(900));
        record_not_a_bot_submit(&store, "escalate", Some(1700));

        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: TelemetryExactness::Exact,
                    basis: TelemetryBasis::Observed,
                }),
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::Maze,
                http_status: 200,
                response_bytes: 70,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );

        record_pow_failure(&store, "198.51.100.44", "invalid_proof");
        record_pow_success(&store);

        let summary = summarize_with_store(&store, 24, 10);

        let not_a_bot = summary
            .defence_funnel
            .rows
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.family == "not_a_bot")
            .expect("not_a_bot funnel row");
        assert_eq!(not_a_bot.candidate_requests, Some(1));
        assert_eq!(not_a_bot.triggered_requests, Some(1));
        assert_eq!(not_a_bot.friction_requests, Some(1));
        assert_eq!(not_a_bot.passed_requests, Some(1));
        assert_eq!(not_a_bot.failed_requests, Some(0));
        assert_eq!(not_a_bot.escalated_requests, Some(1));
        assert_eq!(not_a_bot.denied_requests, None);
        assert_eq!(not_a_bot.suspicious_forwarded_requests, None);
        assert_eq!(not_a_bot.likely_human_affected_requests, Some(1));

        let challenge = summary
            .defence_funnel
            .rows
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.family == "challenge")
            .expect("challenge funnel row");
        assert_eq!(challenge.candidate_requests, Some(1));
        assert_eq!(challenge.triggered_requests, Some(1));
        assert_eq!(challenge.friction_requests, Some(1));
        assert_eq!(challenge.passed_requests, None);
        assert_eq!(challenge.failed_requests, Some(1));
        assert_eq!(challenge.escalated_requests, None);
        assert_eq!(challenge.likely_human_affected_requests, Some(0));

        let js_challenge = summary
            .defence_funnel
            .rows
            .iter()
            .find(|row| row.execution_mode == "shadow" && row.family == "js_challenge")
            .expect("js challenge funnel row");
        assert_eq!(js_challenge.candidate_requests, Some(1));
        assert_eq!(js_challenge.triggered_requests, Some(1));
        assert_eq!(js_challenge.friction_requests, Some(1));
        assert_eq!(js_challenge.passed_requests, None);
        assert_eq!(js_challenge.failed_requests, None);
        assert_eq!(js_challenge.likely_human_affected_requests, Some(1));

        let maze = summary
            .defence_funnel
            .rows
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.family == "maze")
            .expect("maze funnel row");
        assert_eq!(maze.candidate_requests, Some(1));
        assert_eq!(maze.triggered_requests, Some(1));
        assert_eq!(maze.friction_requests, Some(1));
        assert_eq!(maze.likely_human_affected_requests, Some(1));

        let pow = summary
            .defence_funnel
            .rows
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.family == "pow")
            .expect("pow funnel row");
        assert_eq!(pow.candidate_requests, Some(2));
        assert_eq!(pow.triggered_requests, Some(2));
        assert_eq!(pow.friction_requests, Some(2));
        assert_eq!(pow.passed_requests, Some(1));
        assert_eq!(pow.failed_requests, Some(1));
        assert_eq!(pow.likely_human_affected_requests, None);
    }

    #[test]
    fn summarize_keeps_defence_funnel_live_only_even_when_legacy_followup_counters_exist() {
        let store = MockStore::default();
        let _guard = crate::runtime::sim_telemetry::enter(Some(
            crate::runtime::sim_telemetry::SimulationRequestMetadata {
                sim_run_id: "run-1".to_string(),
                sim_profile: "deterministic".to_string(),
                sim_lane: "crawler".to_string(),
            },
        ));

        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::SimPublic,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 111,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::SimPublic,
            },
        );
        record_not_a_bot_served(&store);
        record_not_a_bot_submit(&store, "pass", Some(900));
        record_challenge_failure(&store, "198.51.100.30", "incorrect");
        record_pow_failure(&store, "198.51.100.44", "invalid_proof");
        record_pow_success(&store);

        let summary = summarize_with_store(&store, 24, 10);

        assert_eq!(summary.challenge.total_failures, 0);
        assert_eq!(summary.not_a_bot.served, 0);
        assert_eq!(summary.not_a_bot.submitted, 0);
        assert_eq!(summary.pow.total_attempts, 0);
        assert!(summary.defence_funnel.rows.is_empty());
    }

    #[test]
    fn summarize_returns_seeded_maps_when_empty() {
        let store = MockStore::default();
        let summary = summarize_with_store(&store, 24, 10);
        assert_eq!(summary.shadow.total_actions, 0);
        assert_eq!(summary.shadow.pass_through_total, 0);
        assert_eq!(
            summary.shadow.actions.get("block").copied().unwrap_or(99),
            0
        );
        assert_eq!(summary.honeypot.total_hits, 0);
        assert_eq!(summary.challenge.total_failures, 0);
        assert_eq!(summary.not_a_bot.served, 0);
        assert_eq!(summary.not_a_bot.submitted, 0);
        assert_eq!(summary.not_a_bot.pass, 0);
        assert_eq!(summary.not_a_bot.replay, 0);
        assert_eq!(
            summary.challenge.reasons.get("incorrect").copied().unwrap_or(99),
            0
        );
        assert_eq!(
            summary
                .not_a_bot
                .solve_latency_buckets
                .get("lt_1s")
                .copied()
                .unwrap_or(99),
            0
        );
        assert_eq!(
            summary.pow.reasons.get("invalid_proof").copied().unwrap_or(99),
            0
        );
        assert_eq!(
            summary.rate.outcomes.get("banned").copied().unwrap_or(99),
            0
        );
        assert_eq!(summary.geo.actions.get("maze").copied().unwrap_or(99), 0);
        assert!(summary.defence_funnel.rows.is_empty());
        assert!(summary.human_friction.segments.is_empty());
        assert!(summary.request_outcomes.by_scope.is_empty());
        assert!(summary.request_outcomes.by_lane.is_empty());
    }

    #[test]
    fn summarize_aggregates_dimension_counts() {
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;

        let hp_ip = encode_dim("10.0.0.0");
        let hp_path = encode_dim("/instaban");
        set_counter(
            &store,
            format!("{}:honeypot:total:{}", MONITORING_PREFIX, now_hour).as_str(),
            3,
        );
        set_counter(
            &store,
            format!("{}:honeypot:ip:{}:{}", MONITORING_PREFIX, hp_ip, now_hour).as_str(),
            3,
        );
        set_counter(
            &store,
            format!(
                "{}:honeypot:path:{}:{}",
                MONITORING_PREFIX, hp_path, now_hour
            )
            .as_str(),
            3,
        );

        let challenge_origin = encode_dim("live");
        let challenge_reason = encode_dim("live|incorrect");
        let challenge_ip = encode_dim("live|198.51.100.0");
        set_counter(
            &store,
            format!(
                "{}:challenge:total:{}:{}",
                MONITORING_PREFIX, challenge_origin, now_hour
            )
            .as_str(),
            2,
        );
        set_counter(
            &store,
            format!(
                "{}:challenge:reason:{}:{}",
                MONITORING_PREFIX, challenge_reason, now_hour
            )
            .as_str(),
            2,
        );
        set_counter(
            &store,
            format!(
                "{}:challenge:ip:{}:{}",
                MONITORING_PREFIX, challenge_ip, now_hour
            )
            .as_str(),
            2,
        );
        let rate_origin = encode_dim("live");
        let rate_ip = encode_dim("live|203.0.113.0");
        let rate_path = encode_dim("live|/checkout");
        let rate_outcome = encode_dim("live|limited");
        set_counter(
            &store,
            format!(
                "{}:rate:total:{}:{}",
                MONITORING_PREFIX, rate_origin, now_hour
            )
            .as_str(),
            4,
        );
        set_counter(
            &store,
            format!("{}:rate:ip:{}:{}", MONITORING_PREFIX, rate_ip, now_hour).as_str(),
            4,
        );
        set_counter(
            &store,
            format!("{}:rate:path:{}:{}", MONITORING_PREFIX, rate_path, now_hour).as_str(),
            4,
        );
        set_counter(
            &store,
            format!(
                "{}:rate:outcome:{}:{}",
                MONITORING_PREFIX, rate_outcome, now_hour
            )
            .as_str(),
            4,
        );

        let summary = summarize_with_store(&store, 24, 10);
        assert_eq!(summary.honeypot.total_hits, 3);
        assert_eq!(summary.honeypot.unique_crawlers, 1);
        assert_eq!(summary.honeypot.top_paths.first().map(|v| v.count), Some(3));
        assert_eq!(summary.challenge.total_failures, 2);
        assert_eq!(
            summary.challenge.reasons.get("incorrect").copied().unwrap_or(0),
            2
        );
        assert_eq!(summary.challenge.unique_offenders, 1);
        assert_eq!(summary.challenge.trend.last().map(|v| v.total), Some(2));
        assert_eq!(summary.rate.total_violations, 4);
        assert_eq!(summary.rate.unique_offenders, 1);
        assert_eq!(
            summary.rate.top_paths.first().map(|v| (v.label.as_str(), v.count)),
            Some(("/checkout", 4))
        );
        assert_eq!(
            summary.rate.outcomes.get("limited").copied().unwrap_or(0),
            4
        );
    }

    #[test]
    fn summarize_aggregates_not_a_bot_outcomes_and_latency() {
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;
        let live_origin = encode_dim("live");
        let outcome_pass = encode_dim("live|pass");
        let outcome_escalate = encode_dim("live|escalate");
        let outcome_fail = encode_dim("live|fail");
        let latency_fast = encode_dim("live|lt_1s");
        let latency_mid = encode_dim("live|1_3s");
        let latency_slow = encode_dim("live|10s_plus");

        set_counter(
            &store,
            format!(
                "{}:not_a_bot:served:{}:{}",
                MONITORING_PREFIX, live_origin, now_hour
            )
            .as_str(),
            5,
        );
        set_counter(
            &store,
            format!(
                "{}:not_a_bot:submitted:{}:{}",
                MONITORING_PREFIX, live_origin, now_hour
            )
            .as_str(),
            4,
        );
        set_counter(
            &store,
            format!(
                "{}:not_a_bot:outcome:{}:{}",
                MONITORING_PREFIX, outcome_pass, now_hour
            )
            .as_str(),
            2,
        );
        set_counter(
            &store,
            format!(
                "{}:not_a_bot:outcome:{}:{}",
                MONITORING_PREFIX, outcome_escalate, now_hour
            )
            .as_str(),
            1,
        );
        set_counter(
            &store,
            format!(
                "{}:not_a_bot:outcome:{}:{}",
                MONITORING_PREFIX, outcome_fail, now_hour
            )
            .as_str(),
            1,
        );
        set_counter(
            &store,
            format!(
                "{}:not_a_bot:solve_ms_bucket:{}:{}",
                MONITORING_PREFIX, latency_fast, now_hour
            )
            .as_str(),
            1,
        );
        set_counter(
            &store,
            format!(
                "{}:not_a_bot:solve_ms_bucket:{}:{}",
                MONITORING_PREFIX, latency_mid, now_hour
            )
            .as_str(),
            2,
        );
        set_counter(
            &store,
            format!(
                "{}:not_a_bot:solve_ms_bucket:{}:{}",
                MONITORING_PREFIX, latency_slow, now_hour
            )
            .as_str(),
            1,
        );

        let summary = summarize_with_store(&store, 24, 10);
        assert_eq!(summary.not_a_bot.served, 5);
        assert_eq!(summary.not_a_bot.submitted, 4);
        assert_eq!(summary.not_a_bot.pass, 2);
        assert_eq!(summary.not_a_bot.escalate, 1);
        assert_eq!(summary.not_a_bot.fail, 1);
        assert_eq!(summary.not_a_bot.replay, 0);
        assert_eq!(summary.not_a_bot.abandonments_estimated, 1);
        assert!((summary.not_a_bot.abandonment_ratio - 0.2).abs() < 0.000_001);
        assert_eq!(
            summary
                .not_a_bot
                .solve_latency_buckets
                .get("1_3s")
                .copied()
                .unwrap_or(0),
            2
        );
    }

    #[test]
    fn summarize_aggregates_pow_outcomes_and_ratio() {
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;
        let pow_origin = encode_dim("live");
        let pow_ip = encode_dim("live|198.51.100.9");
        let pow_reason = encode_dim("live|invalid_proof");
        let pow_outcome_success = encode_dim("live|success");
        let pow_outcome_failure = encode_dim("live|failure");

        set_counter(
            &store,
            format!("{}:pow:total:{}:{}", MONITORING_PREFIX, pow_origin, now_hour).as_str(),
            3,
        );
        set_counter(
            &store,
            format!(
                "{}:pow:success:{}:{}",
                MONITORING_PREFIX, pow_origin, now_hour
            )
            .as_str(),
            9,
        );
        set_counter(
            &store,
            format!("{}:pow:ip:{}:{}", MONITORING_PREFIX, pow_ip, now_hour).as_str(),
            3,
        );
        set_counter(
            &store,
            format!("{}:pow:reason:{}:{}", MONITORING_PREFIX, pow_reason, now_hour).as_str(),
            3,
        );
        set_counter(
            &store,
            format!(
                "{}:pow:outcome:{}:{}",
                MONITORING_PREFIX, pow_outcome_success, now_hour
            )
            .as_str(),
            9,
        );
        set_counter(
            &store,
            format!(
                "{}:pow:outcome:{}:{}",
                MONITORING_PREFIX, pow_outcome_failure, now_hour
            )
            .as_str(),
            3,
        );

        let summary = summarize_with_store(&store, 24, 10);
        assert_eq!(summary.pow.total_failures, 3);
        assert_eq!(summary.pow.total_successes, 9);
        assert_eq!(summary.pow.total_attempts, 12);
        assert!((summary.pow.success_ratio - 0.75).abs() < 0.000_001);
        assert_eq!(summary.pow.unique_offenders, 1);
        assert_eq!(summary.pow.reasons.get("invalid_proof").copied().unwrap_or(0), 3);
        assert_eq!(summary.pow.outcomes.get("success").copied().unwrap_or(0), 9);
        assert_eq!(summary.pow.outcomes.get("failure").copied().unwrap_or(0), 3);
    }

    #[test]
    fn summarize_enforces_top_limit_and_window_bounds() {
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;
        let old_hour = now_hour.saturating_sub(72);

        set_counter(
            &store,
            format!("{}:honeypot:total:{}", MONITORING_PREFIX, old_hour).as_str(),
            900,
        );
        set_counter(
            &store,
            format!("{}:honeypot:total:{}", MONITORING_PREFIX, now_hour).as_str(),
            7,
        );

        for index in 0..70usize {
            let ip = encode_dim(format!("198.51.100.{}", index).as_str());
            set_counter(
                &store,
                format!("{}:honeypot:ip:{}:{}", MONITORING_PREFIX, ip, now_hour).as_str(),
                (index + 1) as u64,
            );
        }

        let summary = summarize_with_store(&store, 24, 500);
        assert_eq!(summary.honeypot.total_hits, 7);
        assert_eq!(summary.honeypot.top_crawlers.len(), MAX_TOP_LIMIT);
    }

    #[test]
    fn summarize_uses_bucket_indexes_without_full_keyspace_scan() {
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;
        let counter_key = format!(
            "{}:challenge:total:{}:{}",
            MONITORING_PREFIX,
            encode_dim("live"),
            now_hour
        );
        set_counter(&store, counter_key.as_str(), 4);
        crate::observability::retention::register_monitoring_key(&store, now_hour, counter_key.as_str());
        store
            .set("unrelated:telemetry:key", b"17")
            .expect("set unrelated");

        let summary = summarize_with_store(&store, 1, 10);
        assert_eq!(summary.challenge.total_failures, 4);
        assert_eq!(store.get_keys_calls(), 0);
    }

    #[test]
    fn summarize_shadow_metrics_uses_bucket_indexes_without_full_keyspace_scan() {
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;
        let total_key = format!("{}:shadow:total:{}", MONITORING_PREFIX, now_hour);
        let action_key = format!("{}:shadow:action:{}:{}", MONITORING_PREFIX, "block", now_hour);
        let pass_through_key = format!("{}:shadow:pass_through:{}", MONITORING_PREFIX, now_hour);
        set_counter(&store, total_key.as_str(), 3);
        set_counter(&store, action_key.as_str(), 2);
        set_counter(&store, pass_through_key.as_str(), 5);
        store
            .set("unrelated:shadow:key", b"17")
            .expect("set unrelated");

        let summary = summarize_with_store(&store, 1, 10);
        assert_eq!(summary.shadow.total_actions, 3);
        assert_eq!(summary.shadow.pass_through_total, 5);
        assert_eq!(summary.shadow.actions.get("block").copied(), Some(2));
        assert_eq!(store.get_keys_calls(), 0);
    }

    #[test]
    fn summarize_builds_and_reuses_day_rollups_for_complete_prior_days() {
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;
        let previous_day_start = day_start_hour(now_hour.saturating_sub(MONITORING_DAY_HOURS));
        let current_day_start = day_start_hour(now_hour);

        for hour in previous_day_start..previous_day_start.saturating_add(MONITORING_DAY_HOURS) {
            set_counter(
                &store,
                format!(
                    "{}:challenge:total:{}:{}",
                    MONITORING_PREFIX,
                    encode_dim("live"),
                    hour
                )
                .as_str(),
                1,
            );
        }
        for hour in current_day_start..=now_hour {
            set_counter(
                &store,
                format!(
                    "{}:challenge:total:{}:{}",
                    MONITORING_PREFIX,
                    encode_dim("live"),
                    hour
                )
                .as_str(),
                1,
            );
        }

        let first_summary = summarize_with_store(&store, 48, 10);
        let rollup_key = monitoring_day_rollup_key(previous_day_start);
        assert!(
            store
                .get(rollup_key.as_str())
                .expect("rollup read should succeed")
                .is_some()
        );

        for hour in previous_day_start..previous_day_start.saturating_add(MONITORING_DAY_HOURS) {
            let key = format!(
                "{}:challenge:total:{}:{}",
                MONITORING_PREFIX,
                encode_dim("live"),
                hour
            );
            store.delete(key.as_str()).expect("delete hourly key");
        }

        let second_summary = summarize_with_store(&store, 48, 10);
        assert_eq!(
            second_summary.challenge.total_failures,
            first_summary.challenge.total_failures
        );
    }

    #[test]
    fn normalize_telemetry_path_caps_dynamic_cardinality() {
        assert_eq!(normalize_telemetry_path("/"), "/");
        assert_eq!(
            normalize_telemetry_path("/api/v1/orders/12345"),
            "/api/v1/orders/*"
        );
        assert_eq!(
            normalize_telemetry_path("/checkout/af13d9c8b71e4f5a/token"),
            "/checkout/:id/token"
        );
        assert_eq!(
            normalize_telemetry_path(
                "/events/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/9876543210/details?cursor=abc"
            ),
            "/events/:id/:id/*"
        );
    }

    #[test]
    fn ip_range_human_evidence_uses_bucketed_monitoring_dimensions() {
        let store = MockStore::default();
        let hour = now_ts() / 3600;

        record_ip_range_challenge_solved(&store, "198.51.100.42");
        maybe_record_ip_range_likely_human_sample(&store, "198.51.100.43", 100, "/checkout");

        let challenge_signal_key = monitoring_key(
            "ip_range_suggestions",
            "human_signal",
            Some("challenge_puzzle_pass"),
            hour,
        );
        let sampled_signal_key = monitoring_key(
            "ip_range_suggestions",
            "human_signal",
            Some("likely_human_sample"),
            hour,
        );
        assert_eq!(read_counter(&store, challenge_signal_key.as_str()), 1);
        assert_eq!(read_counter(&store, sampled_signal_key.as_str()), 1);

        let bucket = crate::signals::ip_identity::bucket_ip("198.51.100.42");
        let bucket_key = monitoring_key(
            "ip_range_suggestions",
            "human_ip",
            Some(bucket.as_str()),
            hour,
        );
        assert_eq!(read_counter(&store, bucket_key.as_str()), 2);
    }

    #[test]
    fn likely_human_sampling_respects_disabled_percent() {
        let store = MockStore::default();
        let hour = now_ts() / 3600;
        maybe_record_ip_range_likely_human_sample(&store, "198.51.100.10", 0, "/");
        let sampled_key = monitoring_key("ip_range_suggestions", "likely_human_sampled", None, hour);
        let unsampled_key = monitoring_key("ip_range_suggestions", "likely_human_unsampled", None, hour);
        assert_eq!(read_counter(&store, sampled_key.as_str()), 0);
        assert_eq!(read_counter(&store, unsampled_key.as_str()), 1);
    }

    #[test]
    fn guarded_dimension_cardinality_caps_to_other_bucket() {
        let store = MockStore::default();
        let hour = now_ts() / 3600;

        for idx in 0..(GUARDED_DIMENSION_CARDINALITY_CAP_PER_HOUR + 2) {
            // Use path as the guarded high-cardinality dimension under test.
            record_honeypot_hit(
                &store,
                "198.51.100.42",
                format!("/overflow-check/p{}", idx).as_str(),
            );
        }

        let overflow_dimension_key = monitoring_key("honeypot", "path", Some("other"), hour);
        let guard_count_key = cardinality_guard_count_key("honeypot", "path", hour);
        let overflow_count_key = cardinality_guard_overflow_key("honeypot", "path", hour);
        assert_eq!(
            read_counter(&store, guard_count_key.as_str()),
            GUARDED_DIMENSION_CARDINALITY_CAP_PER_HOUR
        );
        assert_eq!(read_counter(&store, overflow_dimension_key.as_str()), 2);
        assert_eq!(read_counter(&store, overflow_count_key.as_str()), 2);
    }

    #[test]
    fn likely_human_sampling_hash_is_deterministic_for_same_minute_bucket() {
        let minute_bucket = 123_456u64;
        let first = should_sample_likely_human("198.51.100.10", "/products", 10, minute_bucket);
        let second = should_sample_likely_human("198.51.100.10", "/products", 10, minute_bucket);
        assert_eq!(first, second);
        assert!(!should_sample_likely_human(
            "198.51.100.10",
            "/products",
            0,
            minute_bucket
        ));
        assert!(should_sample_likely_human(
            "198.51.100.10",
            "/products",
            100,
            minute_bucket
        ));
    }

    #[test]
    fn summarize_read_path_does_not_delete_expired_monitoring_keys() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "1");
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;
        let expired_hour = now_hour.saturating_sub(6);
        let expired_key = monitoring_key("pow", "total", Some("live"), expired_hour);
        store
            .set(expired_key.as_str(), b"1")
            .expect("counter write should succeed");

        record_pow_failure(&store, "203.0.113.9", "invalid_proof");
        let _ = summarize_with_store(&store, 24, 10);

        assert!(
            store
                .get(expired_key.as_str())
                .expect("counter read should succeed")
                .is_some()
        );
        std::env::remove_var("SHUMA_EVENT_LOG_RETENTION_HOURS");
    }
}
