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

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct CountEntry {
    pub label: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct TrendPoint {
    pub ts: u64,
    pub total: u64,
    pub reasons: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct HoneypotSummary {
    pub total_hits: u64,
    pub unique_crawlers: u64,
    pub top_crawlers: Vec<CountEntry>,
    pub top_paths: Vec<CountEntry>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct FailureSummary {
    pub total_failures: u64,
    pub unique_offenders: u64,
    pub top_offenders: Vec<CountEntry>,
    pub reasons: BTreeMap<String, u64>,
    pub trend: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Serialize, Default)]
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

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct RateSummary {
    pub total_violations: u64,
    pub unique_offenders: u64,
    pub top_offenders: Vec<CountEntry>,
    pub top_paths: Vec<CountEntry>,
    pub outcomes: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct GeoSummary {
    pub total_violations: u64,
    pub actions: BTreeMap<String, u64>,
    pub top_countries: Vec<CountEntry>,
}

#[derive(Debug, Clone, Serialize, Default)]
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

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct MonitoringSummary {
    pub generated_at: u64,
    pub hours: u64,
    pub honeypot: HoneypotSummary,
    pub challenge: FailureSummary,
    pub not_a_bot: NotABotSummary,
    pub pow: PowSummary,
    pub rate: RateSummary,
    pub geo: GeoSummary,
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
fn increment_counter<S: crate::challenge::KeyValueStore>(store: &S, key: &str) {
    let current = read_counter(store, key);
    let next = current.saturating_add(1);
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
}

#[cfg(not(test))]
fn increment_counter<S: crate::challenge::KeyValueStore>(store: &S, key: &str) {
    let now = now_ts();
    {
        let mut buffer = PENDING_COUNTER_BUFFER.lock().unwrap();
        let entry = buffer.deltas.entry(key.to_string()).or_insert(0);
        *entry = entry.saturating_add(1);
        if buffer.last_flush_ts == 0 {
            buffer.last_flush_ts = now;
        }
    }
    maybe_flush_pending_counter_buffer(store, false);
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
    let normalized_reason = normalize_challenge_reason(reason);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "challenge", "total", None);
    record_with_dimension(store, "challenge", "reason", Some(normalized_reason));
    record_with_dimension(store, "challenge", "ip", Some(ip_bucket.as_str()));
}

pub(crate) fn record_pow_failure<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    reason: &str,
) {
    let normalized_reason = normalize_pow_reason(reason);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "pow", "total", None);
    record_with_dimension(store, "pow", "outcome", Some("failure"));
    record_with_dimension(store, "pow", "reason", Some(normalized_reason));
    record_with_dimension(store, "pow", "ip", Some(ip_bucket.as_str()));
}

pub(crate) fn record_pow_success<S: crate::challenge::KeyValueStore>(store: &S) {
    record_with_dimension(store, "pow", "success", None);
    record_with_dimension(store, "pow", "outcome", Some("success"));
}

pub(crate) fn record_rate_violation_with_path<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    path: Option<&str>,
    outcome: &str,
) {
    let normalized_outcome = normalize_rate_outcome(outcome);
    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    record_with_dimension(store, "rate", "total", None);
    record_with_dimension(store, "rate", "outcome", Some(normalized_outcome));
    record_with_dimension(store, "rate", "ip", Some(ip_bucket.as_str()));
    if let Some(raw_path) = path {
        let normalized_path = normalize_telemetry_path(raw_path);
        record_with_dimension(store, "rate", "path", Some(normalized_path.as_str()));
    }
}

pub(crate) fn record_rate_outcome<S: crate::challenge::KeyValueStore>(store: &S, outcome: &str) {
    let normalized_outcome = normalize_rate_outcome(outcome);
    record_with_dimension(store, "rate", "outcome", Some(normalized_outcome));
}

pub(crate) fn record_geo_violation<S: crate::challenge::KeyValueStore>(
    store: &S,
    country: Option<&str>,
    action: &str,
) {
    let normalized_action = normalize_geo_action(action);
    let normalized_country = normalize_country(country);
    record_with_dimension(store, "geo", "total", None);
    record_with_dimension(store, "geo", "action", Some(normalized_action));
    record_with_dimension(store, "geo", "country", Some(normalized_country.as_str()));
}

pub(crate) fn record_not_a_bot_served<S: crate::challenge::KeyValueStore>(store: &S) {
    record_with_dimension(store, "not_a_bot", "served", None);
}

pub(crate) fn record_not_a_bot_submit<S: crate::challenge::KeyValueStore>(
    store: &S,
    outcome: &str,
    solve_ms: Option<u64>,
) {
    let normalized_outcome = normalize_not_a_bot_outcome(outcome);
    record_with_dimension(store, "not_a_bot", "submitted", None);
    record_with_dimension(store, "not_a_bot", "outcome", Some(normalized_outcome));
    if let Some(ms) = solve_ms {
        let bucket = not_a_bot_solve_ms_bucket(ms);
        record_with_dimension(store, "not_a_bot", "solve_ms_bucket", Some(bucket));
    }
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
    honeypot_total: u64,
    honeypot_ip_counts: HashMap<String, u64>,
    honeypot_path_counts: HashMap<String, u64>,
    challenge_total: u64,
    challenge_ip_counts: HashMap<String, u64>,
    challenge_reason_counts: HashMap<String, u64>,
    challenge_trend: TrendAccumulator,
    not_a_bot_served_total: u64,
    not_a_bot_submitted_total: u64,
    not_a_bot_outcomes: HashMap<String, u64>,
    not_a_bot_latency_buckets: HashMap<String, u64>,
    pow_total: u64,
    pow_success_total: u64,
    pow_ip_counts: HashMap<String, u64>,
    pow_reason_counts: HashMap<String, u64>,
    pow_outcomes: HashMap<String, u64>,
    pow_trend: TrendAccumulator,
    rate_total: u64,
    rate_ip_counts: HashMap<String, u64>,
    rate_path_counts: HashMap<String, u64>,
    rate_outcomes: HashMap<String, u64>,
    geo_total: u64,
    geo_actions: HashMap<String, u64>,
    geo_countries: HashMap<String, u64>,
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
                    self.challenge_total = self.challenge_total.saturating_add(count);
                    let entry = self.challenge_trend.totals.entry(hour).or_insert(0);
                    *entry = entry.saturating_add(count);
                }
                "ip" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.challenge_ip_counts, dim, count);
                    }
                }
                "reason" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.challenge_reason_counts, dim, count);
                        let row = self.challenge_trend.reasons.entry(hour).or_default();
                        let reason_entry = row.entry(dim.to_string()).or_insert(0);
                        *reason_entry = reason_entry.saturating_add(count);
                    }
                }
                _ => {}
            },
            "not_a_bot" => match metric {
                "served" => {
                    self.not_a_bot_served_total =
                        self.not_a_bot_served_total.saturating_add(count)
                }
                "submitted" => {
                    self.not_a_bot_submitted_total =
                        self.not_a_bot_submitted_total.saturating_add(count)
                }
                "outcome" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.not_a_bot_outcomes, dim, count);
                    }
                }
                "solve_ms_bucket" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.not_a_bot_latency_buckets, dim, count);
                    }
                }
                _ => {}
            },
            "pow" => match metric {
                "total" => {
                    self.pow_total = self.pow_total.saturating_add(count);
                    let entry = self.pow_trend.totals.entry(hour).or_insert(0);
                    *entry = entry.saturating_add(count);
                }
                "success" => {
                    self.pow_success_total = self.pow_success_total.saturating_add(count);
                }
                "ip" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.pow_ip_counts, dim, count);
                    }
                }
                "reason" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.pow_reason_counts, dim, count);
                        let row = self.pow_trend.reasons.entry(hour).or_default();
                        let reason_entry = row.entry(dim.to_string()).or_insert(0);
                        *reason_entry = reason_entry.saturating_add(count);
                    }
                }
                "outcome" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.pow_outcomes, dim, count);
                    }
                }
                _ => {}
            },
            "rate" => match metric {
                "total" => self.rate_total = self.rate_total.saturating_add(count),
                "ip" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.rate_ip_counts, dim, count);
                    }
                }
                "path" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.rate_path_counts, dim, count);
                    }
                }
                "outcome" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.rate_outcomes, dim, count);
                    }
                }
                _ => {}
            },
            "geo" => match metric {
                "total" => self.geo_total = self.geo_total.saturating_add(count),
                "action" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.geo_actions, dim, count);
                    }
                }
                "country" => {
                    if let Some(dim) = dimension {
                        Self::add_count(&mut self.geo_countries, dim, count);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn merge_rollup(&mut self, source: &MonitoringAccumulator) {
        self.honeypot_total = self.honeypot_total.saturating_add(source.honeypot_total);
        Self::merge_count_maps(&mut self.honeypot_ip_counts, &source.honeypot_ip_counts);
        Self::merge_count_maps(&mut self.honeypot_path_counts, &source.honeypot_path_counts);
        self.challenge_total = self.challenge_total.saturating_add(source.challenge_total);
        Self::merge_count_maps(&mut self.challenge_ip_counts, &source.challenge_ip_counts);
        Self::merge_count_maps(
            &mut self.challenge_reason_counts,
            &source.challenge_reason_counts,
        );
        Self::merge_trend(&mut self.challenge_trend, &source.challenge_trend);
        self.not_a_bot_served_total = self
            .not_a_bot_served_total
            .saturating_add(source.not_a_bot_served_total);
        self.not_a_bot_submitted_total = self
            .not_a_bot_submitted_total
            .saturating_add(source.not_a_bot_submitted_total);
        Self::merge_count_maps(&mut self.not_a_bot_outcomes, &source.not_a_bot_outcomes);
        Self::merge_count_maps(
            &mut self.not_a_bot_latency_buckets,
            &source.not_a_bot_latency_buckets,
        );
        self.pow_total = self.pow_total.saturating_add(source.pow_total);
        self.pow_success_total = self.pow_success_total.saturating_add(source.pow_success_total);
        Self::merge_count_maps(&mut self.pow_ip_counts, &source.pow_ip_counts);
        Self::merge_count_maps(&mut self.pow_reason_counts, &source.pow_reason_counts);
        Self::merge_count_maps(&mut self.pow_outcomes, &source.pow_outcomes);
        Self::merge_trend(&mut self.pow_trend, &source.pow_trend);
        self.rate_total = self.rate_total.saturating_add(source.rate_total);
        Self::merge_count_maps(&mut self.rate_ip_counts, &source.rate_ip_counts);
        Self::merge_count_maps(&mut self.rate_path_counts, &source.rate_path_counts);
        Self::merge_count_maps(&mut self.rate_outcomes, &source.rate_outcomes);
        self.geo_total = self.geo_total.saturating_add(source.geo_total);
        Self::merge_count_maps(&mut self.geo_actions, &source.geo_actions);
        Self::merge_count_maps(&mut self.geo_countries, &source.geo_countries);
    }

    fn finalize(self, now: u64, hours: u64, top_limit: usize, start_hour: u64, end_hour: u64) -> MonitoringSummary {
        let mut challenge_reason_map = build_seeded_map(&CHALLENGE_REASON_KEYS);
        for (key, value) in self.challenge_reason_counts {
            let entry = challenge_reason_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let mut pow_reason_map = build_seeded_map(&POW_REASON_KEYS);
        for (key, value) in self.pow_reason_counts {
            let entry = pow_reason_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let mut not_a_bot_outcome_map = build_seeded_map(&NOT_A_BOT_OUTCOME_KEYS);
        for (key, value) in self.not_a_bot_outcomes {
            let entry = not_a_bot_outcome_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let mut not_a_bot_latency_map = build_seeded_map(&NOT_A_BOT_SOLVE_MS_BUCKET_KEYS);
        for (key, value) in self.not_a_bot_latency_buckets {
            let entry = not_a_bot_latency_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let not_a_bot_abandonments =
            self.not_a_bot_served_total.saturating_sub(self.not_a_bot_submitted_total);
        let not_a_bot_abandonment_ratio = if self.not_a_bot_served_total == 0 {
            0.0
        } else {
            not_a_bot_abandonments as f64 / self.not_a_bot_served_total as f64
        };

        let mut pow_outcome_map = build_seeded_map(&POW_OUTCOME_KEYS);
        for (key, value) in self.pow_outcomes {
            let entry = pow_outcome_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }
        let pow_outcome_failures = pow_outcome_map.get("failure").copied().unwrap_or(0);
        let pow_outcome_successes = pow_outcome_map.get("success").copied().unwrap_or(0);
        let pow_total_failures = self.pow_total.max(pow_outcome_failures);
        let pow_total_successes = self.pow_success_total.max(pow_outcome_successes);
        let pow_total_attempts = pow_total_failures.saturating_add(pow_total_successes);
        let pow_success_ratio = if pow_total_attempts == 0 {
            0.0
        } else {
            pow_total_successes as f64 / pow_total_attempts as f64
        };

        let mut rate_outcome_map = build_seeded_map(&RATE_OUTCOME_KEYS);
        for (key, value) in self.rate_outcomes {
            let entry = rate_outcome_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        let mut geo_action_map = build_seeded_map(&GEO_ACTION_KEYS);
        for (key, value) in self.geo_actions {
            let entry = geo_action_map.entry(key).or_insert(0);
            *entry = entry.saturating_add(value);
        }

        MonitoringSummary {
            generated_at: now,
            hours,
            honeypot: HoneypotSummary {
                total_hits: self.honeypot_total,
                unique_crawlers: self.honeypot_ip_counts.len() as u64,
                top_crawlers: top_entries(&self.honeypot_ip_counts, top_limit),
                top_paths: top_entries(&self.honeypot_path_counts, top_limit),
            },
            challenge: FailureSummary {
                total_failures: self.challenge_total,
                unique_offenders: self.challenge_ip_counts.len() as u64,
                top_offenders: top_entries(&self.challenge_ip_counts, top_limit),
                reasons: challenge_reason_map,
                trend: build_trend(start_hour, end_hour, &CHALLENGE_REASON_KEYS, self.challenge_trend),
            },
            not_a_bot: NotABotSummary {
                served: self.not_a_bot_served_total,
                submitted: self.not_a_bot_submitted_total,
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
                unique_offenders: self.pow_ip_counts.len() as u64,
                top_offenders: top_entries(&self.pow_ip_counts, top_limit),
                reasons: pow_reason_map,
                outcomes: pow_outcome_map,
                trend: build_trend(start_hour, end_hour, &POW_REASON_KEYS, self.pow_trend),
            },
            rate: RateSummary {
                total_violations: self.rate_total,
                unique_offenders: self.rate_ip_counts.len() as u64,
                top_offenders: top_entries(&self.rate_ip_counts, top_limit),
                top_paths: top_entries(&self.rate_path_counts, top_limit),
                outcomes: rate_outcome_map,
            },
            geo: GeoSummary {
                total_violations: self.geo_total,
                actions: geo_action_map,
                top_countries: top_entries(&self.geo_countries, top_limit),
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
    fn summarize_returns_seeded_maps_when_empty() {
        let store = MockStore::default();
        let summary = summarize_with_store(&store, 24, 10);
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

        let challenge_reason = encode_dim("incorrect");
        let challenge_ip = encode_dim("198.51.100.0");
        set_counter(
            &store,
            format!("{}:challenge:total:{}", MONITORING_PREFIX, now_hour).as_str(),
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
        let rate_ip = encode_dim("203.0.113.0");
        let rate_path = encode_dim("/checkout");
        let rate_outcome = encode_dim("limited");
        set_counter(
            &store,
            format!("{}:rate:total:{}", MONITORING_PREFIX, now_hour).as_str(),
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
        let outcome_pass = encode_dim("pass");
        let outcome_escalate = encode_dim("escalate");
        let outcome_fail = encode_dim("fail");
        let latency_fast = encode_dim("lt_1s");
        let latency_mid = encode_dim("1_3s");
        let latency_slow = encode_dim("10s_plus");

        set_counter(
            &store,
            format!("{}:not_a_bot:served:{}", MONITORING_PREFIX, now_hour).as_str(),
            5,
        );
        set_counter(
            &store,
            format!("{}:not_a_bot:submitted:{}", MONITORING_PREFIX, now_hour).as_str(),
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
        let pow_ip = encode_dim("198.51.100.9");
        let pow_reason = encode_dim("invalid_proof");
        let pow_outcome_success = encode_dim("success");
        let pow_outcome_failure = encode_dim("failure");

        set_counter(
            &store,
            format!("{}:pow:total:{}", MONITORING_PREFIX, now_hour).as_str(),
            3,
        );
        set_counter(
            &store,
            format!("{}:pow:success:{}", MONITORING_PREFIX, now_hour).as_str(),
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
        let counter_key = format!("{}:challenge:total:{}", MONITORING_PREFIX, now_hour);
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
    fn summarize_builds_and_reuses_day_rollups_for_complete_prior_days() {
        let store = MockStore::default();
        let now_hour = now_ts() / 3600;
        let previous_day_start = day_start_hour(now_hour.saturating_sub(MONITORING_DAY_HOURS));
        let current_day_start = day_start_hour(now_hour);

        for hour in previous_day_start..previous_day_start.saturating_add(MONITORING_DAY_HOURS) {
            set_counter(
                &store,
                format!("{}:challenge:total:{}", MONITORING_PREFIX, hour).as_str(),
                1,
            );
        }
        for hour in current_day_start..=now_hour {
            set_counter(
                &store,
                format!("{}:challenge:total:{}", MONITORING_PREFIX, hour).as_str(),
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
            let key = format!("{}:challenge:total:{}", MONITORING_PREFIX, hour);
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
        let expired_key = monitoring_key("pow", "total", None, expired_hour);
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
