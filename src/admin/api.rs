use flate2::write::GzEncoder;
use flate2::Compression;
use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
/// Event types for activity logging
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventType {
    Ban,
    Unban,
    Challenge,
    Block,
    AdminAction,
}

/// Event log entry
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventLogEntry {
    pub ts: u64, // unix timestamp
    pub event: EventType,
    pub ip: Option<String>,
    pub reason: Option<String>,
    pub outcome: Option<String>,
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EventLogRecord {
    #[serde(flatten)]
    pub entry: EventLogEntry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sim_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sim_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sim_lane: Option<String>,
    #[serde(default)]
    pub is_simulation: bool,
}

impl EventLogRecord {
    fn from_entry(entry: EventLogEntry) -> Self {
        EventLogRecord {
            entry,
            sim_run_id: None,
            sim_profile: None,
            sim_lane: None,
            is_simulation: false,
        }
    }
}

/// Event log storage notes:
/// - v2 format stores immutable records per event: eventlog:v2:<hour>:<ts>-<nonce>
const EVENTLOG_V2_PREFIX: &str = "eventlog:v2";
const POW_DIFFICULTY_MIN: u8 = crate::config::POW_DIFFICULTY_MIN;
const POW_DIFFICULTY_MAX: u8 = crate::config::POW_DIFFICULTY_MAX;
const POW_TTL_MIN: u64 = crate::config::POW_TTL_MIN;
const POW_TTL_MAX: u64 = crate::config::POW_TTL_MAX;
const CHALLENGE_TRANSFORM_COUNT_MIN: u64 = 4;
const CHALLENGE_TRANSFORM_COUNT_MAX: u64 = 8;
const CHALLENGE_PUZZLE_SEED_TTL_MIN: u64 = 30;
const CHALLENGE_PUZZLE_SEED_TTL_MAX: u64 = 300;
const CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MIN: u64 = 1;
const CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MAX: u64 = 100;
const CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MIN: u64 = 30;
const CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MAX: u64 = 3600;
const NOT_A_BOT_THRESHOLD_MIN: u64 = 1;
const NOT_A_BOT_THRESHOLD_MAX: u64 = 10;
const NOT_A_BOT_SCORE_MIN: u64 = 1;
const NOT_A_BOT_SCORE_MAX: u64 = 10;
const NOT_A_BOT_NONCE_TTL_MIN: u64 = 30;
const NOT_A_BOT_NONCE_TTL_MAX: u64 = 300;
const NOT_A_BOT_MARKER_TTL_MIN: u64 = 60;
const NOT_A_BOT_MARKER_TTL_MAX: u64 = 3600;
const NOT_A_BOT_ATTEMPT_LIMIT_MIN: u64 = 1;
const NOT_A_BOT_ATTEMPT_LIMIT_MAX: u64 = 100;
const NOT_A_BOT_ATTEMPT_WINDOW_MIN: u64 = 30;
const NOT_A_BOT_ATTEMPT_WINDOW_MAX: u64 = 3600;
const TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MIN: u64 = 20;
const TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MAX: u64 = 300;
const TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MIN: u64 = 30;
const TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MAX: u64 = 3_600;
const TARPIT_HASHCASH_DIFFICULTY_MIN: u64 = 4;
const TARPIT_HASHCASH_DIFFICULTY_MAX: u64 = 28;
const TARPIT_STEP_CHUNK_BASE_BYTES_MIN: u64 = 256;
const TARPIT_STEP_CHUNK_BASE_BYTES_MAX: u64 = 65_536;
const TARPIT_STEP_CHUNK_MAX_BYTES_MIN: u64 = 512;
const TARPIT_STEP_CHUNK_MAX_BYTES_MAX: u64 = 131_072;
const TARPIT_STEP_JITTER_PERCENT_MIN: u64 = 0;
const TARPIT_STEP_JITTER_PERCENT_MAX: u64 = 40;
const TARPIT_EGRESS_WINDOW_SECONDS_MIN: u64 = 10;
const TARPIT_EGRESS_WINDOW_SECONDS_MAX: u64 = 3_600;
const TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW_MIN: u64 = 1_024;
const TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW_MAX: u64 = 1_073_741_824;
const TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW_MIN: u64 = 512;
const TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW_MAX: u64 = 268_435_456;
const TARPIT_EGRESS_PER_FLOW_MAX_BYTES_MIN: u64 = 1_024;
const TARPIT_EGRESS_PER_FLOW_MAX_BYTES_MAX: u64 = 268_435_456;
const TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS_MIN: u64 = 5;
const TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS_MAX: u64 = 3_600;
const TARPIT_MAX_CONCURRENT_GLOBAL_MIN: u64 = 1;
const TARPIT_MAX_CONCURRENT_GLOBAL_MAX: u64 = 10_000;
const TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MIN: u64 = 1;
const TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MAX: u64 = 256;
const IP_RANGE_MAX_RULES: usize = 64;
const IP_RANGE_MAX_CIDRS_PER_RULE: usize = 512;
const IP_RANGE_MAX_EMERGENCY_ALLOWLIST: usize = 1024;
const IP_RANGE_CUSTOM_MESSAGE_MAX_CHARS: usize = 280;
const IP_RANGE_REDIRECT_URL_MAX_CHARS: usize = 512;
const CONFIG_EXPORT_SECRET_KEYS: [&str; 15] = [
    "SHUMA_API_KEY",
    "SHUMA_ADMIN_READONLY_API_KEY",
    "SHUMA_JS_SECRET",
    "SHUMA_POW_SECRET",
    "SHUMA_CHALLENGE_SECRET",
    "SHUMA_MAZE_PREVIEW_SECRET",
    "SHUMA_FORWARDED_IP_SECRET",
    "SHUMA_HEALTH_SECRET",
    "SHUMA_SIM_TELEMETRY_SECRET",
    "SHUMA_FRONTIER_OPENAI_API_KEY",
    "SHUMA_FRONTIER_ANTHROPIC_API_KEY",
    "SHUMA_FRONTIER_GOOGLE_API_KEY",
    "SHUMA_FRONTIER_XAI_API_KEY",
    "SHUMA_RATE_LIMITER_REDIS_URL",
    "SHUMA_BAN_STORE_REDIS_URL",
];
const SECURITY_PRIVACY_PREFIX: &str = "security_privacy:v1";
const SECURITY_PRIVACY_CLASSIFICATION_VERSION: &str = "telemetry-security-classification.v1";
const SECURITY_FORENSIC_ACK_VALUE: &str = "I_UNDERSTAND_FORENSIC";
const SECURITY_HIGH_RISK_RETENTION_MAX_HOURS: u64 = 72;
const SECRET_LIKE_SUBSTRINGS: [&str; 8] = [
    "sk-",
    "api_key",
    "authorization: bearer",
    "bearer ",
    "x-api-key",
    "x-shuma-sim-telemetry-secret",
    "private key",
    "-----begin",
];
const SECRET_CANARY_MARKERS: [&str; 3] = [
    "shuma_canary_secret",
    "frontier_secret_canary",
    "sim_secret_canary",
];

fn event_log_retention_hours() -> u64 {
    crate::observability::retention::event_log_high_risk_retention_hours()
}

fn configured_event_log_retention_hours() -> u64 {
    crate::config::event_log_retention_hours()
}

fn event_log_retention_override_requested() -> bool {
    configured_event_log_retention_hours() > SECURITY_HIGH_RISK_RETENTION_MAX_HOURS
}

fn effective_event_log_query_hours(requested_hours: u64) -> u64 {
    let retention = event_log_retention_hours();
    if retention == 0 {
        return requested_hours.clamp(1, 720);
    }
    requested_hours.clamp(1, retention.clamp(1, 720))
}

fn make_v2_event_key(hour: u64, ts: u64) -> String {
    format!("{}:{}:{}-{:016x}", EVENTLOG_V2_PREFIX, hour, ts, random::<u64>())
}

fn parse_v2_event_key(key: &str) -> Option<u64> {
    let mut parts = key.splitn(5, ':');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("eventlog"), Some("v2"), Some(hour), Some(_tail)) => Some(hour.parse::<u64>().ok()?),
        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
struct EventSecuritySanitizationResult {
    scrubbed_fields: u64,
    canary_detected: bool,
}

fn security_privacy_counter_key(metric: &str, hour: u64) -> String {
    format!("{SECURITY_PRIVACY_PREFIX}:{metric}:{hour}")
}

fn increment_security_privacy_counter<S: crate::challenge::KeyValueStore>(
    store: &S,
    metric: &str,
    ts: u64,
) {
    let hour = ts / 3600;
    let key = security_privacy_counter_key(metric, hour);
    let next = read_u64_counter(store, key.as_str()).saturating_add(1);
    if store.set(key.as_str(), next.to_string().as_bytes()).is_err() {
        return;
    }
    if next == 1 {
        crate::observability::retention::register_monitoring_key(store, hour, key.as_str());
    }
}

fn read_security_privacy_counter_window<S: crate::challenge::KeyValueStore>(
    store: &S,
    metric: &str,
    now: u64,
    hours: u64,
) -> u64 {
    let now_hour = now / 3600;
    let window_hours = hours.clamp(1, 720);
    let start_hour = now_hour.saturating_sub(window_hours.saturating_sub(1));
    let mut total = 0u64;
    for hour in start_hour..=now_hour {
        let key = security_privacy_counter_key(metric, hour);
        total = total.saturating_add(read_u64_counter(store, key.as_str()));
    }
    total
}

fn set_security_privacy_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    key_suffix: &str,
    payload: serde_json::Value,
) {
    let storage_key = format!("{SECURITY_PRIVACY_PREFIX}:{key_suffix}");
    if let Ok(bytes) = serde_json::to_vec(&payload) {
        let _ = store.set(storage_key.as_str(), bytes.as_slice());
    }
}

fn load_security_privacy_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    key_suffix: &str,
) -> serde_json::Value {
    let storage_key = format!("{SECURITY_PRIVACY_PREFIX}:{key_suffix}");
    store
        .get(storage_key.as_str())
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(bytes.as_slice()).ok())
        .unwrap_or_else(|| json!({}))
}

fn contains_secret_canary(raw: &str) -> bool {
    let lowered = raw.to_ascii_lowercase();
    SECRET_CANARY_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
}

fn scrub_secret_like_text(raw: &str) -> (String, bool, bool) {
    let lowered = raw.to_ascii_lowercase();
    if contains_secret_canary(raw) {
        return ("[redacted:secret_canary]".to_string(), true, true);
    }
    if SECRET_LIKE_SUBSTRINGS
        .iter()
        .any(|token| lowered.contains(token))
    {
        return ("[redacted:secret]".to_string(), true, false);
    }
    (raw.to_string(), false, false)
}

fn sanitize_event_record_for_persistence(record: &mut EventLogRecord) -> EventSecuritySanitizationResult {
    let mut result = EventSecuritySanitizationResult::default();

    if let Some(reason) = record.entry.reason.clone() {
        let (next, scrubbed, canary) = scrub_secret_like_text(reason.as_str());
        record.entry.reason = Some(next);
        if scrubbed {
            result.scrubbed_fields = result.scrubbed_fields.saturating_add(1);
        }
        if canary {
            result.canary_detected = true;
        }
    }
    if let Some(outcome) = record.entry.outcome.clone() {
        let (next, scrubbed, canary) = scrub_secret_like_text(outcome.as_str());
        record.entry.outcome = Some(next);
        if scrubbed {
            result.scrubbed_fields = result.scrubbed_fields.saturating_add(1);
        }
        if canary {
            result.canary_detected = true;
        }
    }
    if let Some(admin) = record.entry.admin.clone() {
        let (next, scrubbed, canary) = scrub_secret_like_text(admin.as_str());
        record.entry.admin = Some(next);
        if scrubbed {
            result.scrubbed_fields = result.scrubbed_fields.saturating_add(1);
        }
        if canary {
            result.canary_detected = true;
        }
    }
    if let Some(ip) = record.entry.ip.clone() {
        if contains_secret_canary(ip.as_str()) {
            record.entry.ip = Some("[redacted:secret_canary]".to_string());
            result.scrubbed_fields = result.scrubbed_fields.saturating_add(1);
            result.canary_detected = true;
        }
    }

    result
}

fn forensic_access_mode(query: &str) -> bool {
    let forensic_requested = crate::request_validation::query_param(query, "forensic")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let forensic_ack = crate::request_validation::query_param(query, "forensic_ack")
        .unwrap_or_default();
    forensic_requested && forensic_ack == SECURITY_FORENSIC_ACK_VALUE
}

fn pseudonymize_ip_identifier(ip: &str) -> String {
    crate::signals::ip_identity::bucket_ip(ip)
}

fn pseudonymize_event_record(record: &EventLogRecord) -> EventLogRecord {
    let mut next = record.clone();
    if let Some(ip) = record.entry.ip.as_ref() {
        next.entry.ip = Some(pseudonymize_ip_identifier(ip.as_str()));
    }
    if next.entry.admin.is_some() {
        next.entry.admin = Some("[masked]".to_string());
    }
    next
}

fn telemetry_field_classification_schema() -> serde_json::Value {
    json!([
        {
            "field": "event.ts",
            "class": "public",
            "persistence": "allow"
        },
        {
            "field": "event.event",
            "class": "public",
            "persistence": "allow"
        },
        {
            "field": "event.ip",
            "class": "sensitive",
            "persistence": "allow_pseudonymized_default"
        },
        {
            "field": "event.reason",
            "class": "internal",
            "persistence": "allow_with_secret_scrub"
        },
        {
            "field": "event.outcome",
            "class": "internal",
            "persistence": "allow_with_secret_scrub"
        },
        {
            "field": "event.admin",
            "class": "sensitive",
            "persistence": "allow_masked_default"
        },
        {
            "field": "artifact.raw_secret_like_value",
            "class": "secret-prohibited",
            "persistence": "deny_fail_closed"
        }
    ])
}

fn security_view_mode_label(forensic_mode: bool) -> &'static str {
    if forensic_mode {
        "forensic_raw"
    } else {
        "pseudonymized_default"
    }
}

fn present_event_record(record: &EventLogRecord, forensic_mode: bool) -> EventLogRecord {
    if forensic_mode {
        return record.clone();
    }
    pseudonymize_event_record(record)
}

fn present_event_records(records: &[EventLogRecord], forensic_mode: bool) -> Vec<EventLogRecord> {
    records
        .iter()
        .map(|record| present_event_record(record, forensic_mode))
        .collect()
}

fn security_privacy_payload<S: crate::challenge::KeyValueStore>(
    store: &S,
    now: u64,
    hours: u64,
    forensic_mode: bool,
) -> serde_json::Value {
    let effective_hours = hours.clamp(1, 720);
    let classification_enforced_total = read_security_privacy_counter_window(
        store,
        "field_classification_enforced_total",
        now,
        effective_hours,
    );
    let secret_scrub_actions_total =
        read_security_privacy_counter_window(store, "secret_scrub_actions_total", now, effective_hours);
    let secret_canary_detected_total = read_security_privacy_counter_window(
        store,
        "secret_canary_detected_total",
        now,
        effective_hours,
    );
    let incident_hook_emitted_total = read_security_privacy_counter_window(
        store,
        "incident_hook_emitted_total",
        now,
        effective_hours,
    );
    let retention_override_requested = event_log_retention_override_requested();
    let last_violation = load_security_privacy_state(store, "last_violation");
    let last_incident = load_security_privacy_state(store, "last_incident");
    let retention_override_audit = load_security_privacy_state(store, "retention_override_audit");
    let operator_action_required = secret_canary_detected_total > 0
        || last_incident
            .get("incident_id")
            .and_then(|value| value.as_str())
            .map(|value| !value.is_empty())
            .unwrap_or(false);

    json!({
        "classification": {
            "version": SECURITY_PRIVACY_CLASSIFICATION_VERSION,
            "field_classes": ["public", "internal", "sensitive", "secret-prohibited"],
            "schema": telemetry_field_classification_schema(),
            "field_classification_enforced": true,
            "field_classification_enforced_total": classification_enforced_total
        },
        "sanitization": {
            "secret_scrub_actions_total": secret_scrub_actions_total,
            "secret_canary_leak_count": 0,
            "secret_canary_detected_total": secret_canary_detected_total
        },
        "access_control": {
            "view_mode": security_view_mode_label(forensic_mode),
            "pseudonymization_required_percent": 100.0,
            "pseudonymization_coverage_percent": if forensic_mode { 0.0 } else { 100.0 },
            "forensic_break_glass": {
                "active": forensic_mode,
                "acknowledgement_required_query_param": "forensic_ack",
                "acknowledgement_value_hint": SECURITY_FORENSIC_ACK_VALUE,
                "audit_state": if forensic_mode { "acknowledged" } else { "inactive" }
            }
        },
        "retention_tiers": {
            "high_risk_raw_artifacts_hours": event_log_retention_hours(),
            "high_risk_raw_artifacts_max_hours": SECURITY_HIGH_RISK_RETENTION_MAX_HOURS,
            "redacted_summary_hours": configured_event_log_retention_hours(),
            "override_requested": retention_override_requested,
            "override_policy": "requires_explicit_audit_entry",
            "override_audit_entry": retention_override_audit
        },
        "incident_response": {
            "incident_hook_emitted": true,
            "incident_hook_emitted_total": incident_hook_emitted_total,
            "state": if operator_action_required { "operator_action_required" } else { "healthy" },
            "workflow": ["detect", "contain", "quarantine", "operator_action_required"],
            "last_violation": last_violation,
            "last_incident": last_incident
        }
    })
}

pub fn log_event<S: crate::challenge::KeyValueStore>(store: &S, entry: &EventLogEntry) {
    // Write each event to a distinct immutable key to avoid read-modify-write races.
    let mut record = EventLogRecord::from_entry(entry.clone());
    if let Some(sim_metadata) = crate::runtime::sim_telemetry::current_metadata() {
        record.sim_run_id = Some(sim_metadata.sim_run_id);
        record.sim_profile = Some(sim_metadata.sim_profile);
        record.sim_lane = Some(sim_metadata.sim_lane);
        record.is_simulation = true;
    }

    let hour = record.entry.ts / 3600;
    let key = make_v2_event_key(hour, record.entry.ts);
    increment_security_privacy_counter(store, "field_classification_enforced_total", record.entry.ts);
    if event_log_retention_override_requested() {
        set_security_privacy_state(
            store,
            "retention_override_audit",
            json!({
                "ts": record.entry.ts,
                "policy": "high_risk_retention_cap_enforced",
                "override_requested_hours": configured_event_log_retention_hours(),
                "enforced_hours": event_log_retention_hours(),
                "operation_id": key,
                "requires_operator_action": true
            }),
        );
    }
    let sanitization = sanitize_event_record_for_persistence(&mut record);
    if sanitization.scrubbed_fields > 0 {
        increment_security_privacy_counter(store, "secret_scrub_actions_total", record.entry.ts);
        set_security_privacy_state(
            store,
            "last_violation",
            json!({
                "ts": record.entry.ts,
                "type": "secret_scrub_applied",
                "classification": "internal",
                "action": "scrubbed_allow",
                "scrubbed_fields": sanitization.scrubbed_fields,
                "operation_id": key,
                "sim_run_id": record.sim_run_id
            }),
        );
    }
    if sanitization.canary_detected {
        increment_security_privacy_counter(store, "secret_canary_detected_total", record.entry.ts);
        increment_security_privacy_counter(store, "policy_violation_total", record.entry.ts);
        increment_security_privacy_counter(store, "incident_hook_emitted_total", record.entry.ts);
        let incident_id = format!("secinc-{}-{:08x}", record.entry.ts, random::<u32>());
        set_security_privacy_state(
            store,
            "last_violation",
            json!({
                "ts": record.entry.ts,
                "type": "secret_canary_detected",
                "classification": "secret-prohibited",
                "action": "quarantine_drop",
                "operation_id": key,
                "sim_run_id": record.sim_run_id
            }),
        );
        set_security_privacy_state(
            store,
            "last_incident",
            json!({
                "incident_id": incident_id,
                "ts": record.entry.ts,
                "type": "secret_canary_detected",
                "action": "quarantine_drop",
                "workflow": ["detect", "contain", "quarantine", "operator_action_required"],
                "state": "operator_action_required",
                "operation_id": key,
                "sim_run_id": record.sim_run_id
            }),
        );
        eprintln!(
            "[log_event] dropped event due to secret canary detection operation_id={}",
            key
        );
        return;
    }

    match serde_json::to_vec(&record) {
        Ok(payload) => {
            if store.set(&key, &payload).is_err() {
                eprintln!("[log_event] KV error writing {}", key);
                return;
            }
            crate::observability::retention::register_event_log_key(store, hour, key.as_str());
            crate::observability::retention::run_worker_if_due(store);
        }
        Err(_) => eprintln!(
            "[log_event] serialization error; dropping event for key {}",
            key
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenge::KeyValueStore;
    use spin_sdk::http::Method;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl MockStore {
        fn new() -> Self {
            MockStore {
                map: Mutex::new(HashMap::new()),
            }
        }
    }

    impl crate::challenge::KeyValueStore for MockStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let m = self.map.lock().unwrap();
            Ok(m.get(key).cloned())
        }
        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.insert(key.to_string(), value.to_vec());
            Ok(())
        }
        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.remove(key);
            Ok(())
        }
        fn get_keys(&self) -> Result<Vec<String>, ()> {
            let m = self.map.lock().unwrap();
            Ok(m.keys().cloned().collect())
        }
    }

    #[test]
    fn log_event_writes_distinct_v2_records() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("1.2.3.4".to_string()),
            reason: Some("test".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("me".to_string()),
        };
        for _ in 0..5 {
            log_event(&store, &entry);
        }
        let hour = now / 3600;
        let prefix = format!("eventlog:v2:{}:", hour);
        let keys: Vec<String> = store
            .map
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .filter(|k| k.starts_with(&prefix))
            .collect();
        assert_eq!(keys.len(), 5);
    }

    #[test]
    fn log_event_writes_sim_metadata_when_context_active() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("1.2.3.4".to_string()),
            reason: Some("test".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("me".to_string()),
        };
        let _guard = crate::runtime::sim_telemetry::enter(Some(
            crate::runtime::sim_telemetry::SimulationRequestMetadata {
                sim_run_id: "run_001".to_string(),
                sim_profile: "fast_smoke".to_string(),
                sim_lane: "deterministic_black_box".to_string(),
            },
        ));
        log_event(&store, &entry);

        let hour = now / 3600;
        let prefix = format!("eventlog:v2:{}:", hour);
        let records: Vec<(String, Vec<u8>)> = store
            .map
            .lock()
            .unwrap()
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_eq!(records.len(), 1);
        let payload: serde_json::Value = serde_json::from_slice(&records[0].1).unwrap();
        assert_eq!(payload.get("sim_run_id").and_then(|v| v.as_str()), Some("run_001"));
        assert_eq!(payload.get("sim_profile").and_then(|v| v.as_str()), Some("fast_smoke"));
        assert_eq!(
            payload.get("sim_lane").and_then(|v| v.as_str()),
            Some("deterministic_black_box")
        );
        assert_eq!(
            payload.get("is_simulation").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn log_event_scrubs_secret_like_fields_before_persistence() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("198.51.100.10".to_string()),
            reason: Some("authorization: bearer abc123".to_string()),
            outcome: Some("api_key=secret".to_string()),
            admin: Some("x-api-key: abc".to_string()),
        };
        log_event(&store, &entry);

        let hour = now / 3600;
        let prefix = format!("eventlog:v2:{}:", hour);
        let records: Vec<Vec<u8>> = store
            .map
            .lock()
            .unwrap()
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(_, value)| value.clone())
            .collect();
        assert_eq!(records.len(), 1);

        let payload: EventLogRecord = serde_json::from_slice(records[0].as_slice()).unwrap();
        assert_eq!(payload.entry.reason.as_deref(), Some("[redacted:secret]"));
        assert_eq!(payload.entry.outcome.as_deref(), Some("[redacted:secret]"));
        assert_eq!(payload.entry.admin.as_deref(), Some("[redacted:secret]"));

        let scrub_counter_key = security_privacy_counter_key("secret_scrub_actions_total", hour);
        assert_eq!(read_u64_counter(&store, scrub_counter_key.as_str()), 1);
    }

    #[test]
    fn log_event_drops_secret_canary_and_emits_incident_state() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("198.51.100.20".to_string()),
            reason: Some("frontier_secret_canary".to_string()),
            outcome: Some("should_drop".to_string()),
            admin: Some("ops".to_string()),
        };
        log_event(&store, &entry);

        let hour = now / 3600;
        let prefix = format!("eventlog:v2:{}:", hour);
        let persisted = store
            .map
            .lock()
            .unwrap()
            .keys()
            .any(|key| key.starts_with(&prefix));
        assert!(!persisted);

        let canary_counter_key = security_privacy_counter_key("secret_canary_detected_total", hour);
        let incident_counter_key = security_privacy_counter_key("incident_hook_emitted_total", hour);
        assert_eq!(read_u64_counter(&store, canary_counter_key.as_str()), 1);
        assert_eq!(read_u64_counter(&store, incident_counter_key.as_str()), 1);

        let incident = load_security_privacy_state(&store, "last_incident");
        assert_eq!(
            incident.get("action").and_then(|value| value.as_str()),
            Some("quarantine_drop")
        );
        assert_eq!(
            incident.get("state").and_then(|value| value.as_str()),
            Some("operator_action_required")
        );
    }

    #[test]
    fn security_privacy_payload_enforces_high_risk_retention_cap() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "240");
        let store = MockStore::new();
        let now = now_ts();
        log_event(
            &store,
            &EventLogEntry {
                ts: now,
                event: EventType::AdminAction,
                ip: Some("198.51.100.30".to_string()),
                reason: Some("ok".to_string()),
                outcome: Some("ok".to_string()),
                admin: Some("ops".to_string()),
            },
        );
        let payload = security_privacy_payload(&store, now, 24, false);
        assert_eq!(
            payload
                .get("retention_tiers")
                .and_then(|value| value.get("high_risk_raw_artifacts_hours"))
                .and_then(|value| value.as_u64()),
            Some(72)
        );
        assert_eq!(
            payload
                .get("retention_tiers")
                .and_then(|value| value.get("redacted_summary_hours"))
                .and_then(|value| value.as_u64()),
            Some(240)
        );
        assert_eq!(
            payload
                .get("retention_tiers")
                .and_then(|value| value.get("override_requested"))
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        std::env::remove_var("SHUMA_EVENT_LOG_RETENTION_HOURS");
    }

    #[test]
    fn load_recent_events_includes_v2_records() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("1.2.3.4".to_string()),
            reason: Some("test".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("me".to_string()),
        };
        let hour = now / 3600;
        let key = format!("eventlog:v2:{}:{}-deadbeef", hour, now);
        store
            .set(&key, serde_json::to_vec(&entry).unwrap().as_slice())
            .unwrap();

        let events = load_recent_events(&store, now, 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason.as_deref(), Some("test"));
    }

    #[test]
    fn load_recent_event_records_keeps_simulation_metadata() {
        let store = MockStore::new();
        let now = now_ts();
        let hour = now / 3600;

        let non_sim = EventLogRecord::from_entry(EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("1.2.3.4".to_string()),
            reason: Some("non_sim".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("me".to_string()),
        });
        let sim = EventLogRecord {
            entry: EventLogEntry {
                ts: now,
                event: EventType::AdminAction,
                ip: Some("5.6.7.8".to_string()),
                reason: Some("sim".to_string()),
                outcome: Some("ok".to_string()),
                admin: Some("me".to_string()),
            },
            sim_run_id: Some("run_001".to_string()),
            sim_profile: Some("fast_smoke".to_string()),
            sim_lane: Some("deterministic_black_box".to_string()),
            is_simulation: true,
        };

        let non_sim_key = format!("eventlog:v2:{}:{}-non-sim", hour, now);
        let sim_key = format!("eventlog:v2:{}:{}-sim", hour, now);
        store
            .set(&non_sim_key, serde_json::to_vec(&non_sim).unwrap().as_slice())
            .unwrap();
        store
            .set(&sim_key, serde_json::to_vec(&sim).unwrap().as_slice())
            .unwrap();

        let records = load_recent_event_records(&store, now, 1);
        assert_eq!(records.len(), 2);
        assert!(records.iter().any(|record| record.is_simulation));
        assert!(records.iter().any(|record| !record.is_simulation));
    }

    #[test]
    fn paginate_cursor_rows_supports_monotonic_resume_contract() {
        let base_ts = now_ts();
        let make_row = |offset: u64, key: &str, reason: &str| CursorEventRecord {
            cursor: build_event_cursor(base_ts + offset, key),
            record: EventLogRecord::from_entry(EventLogEntry {
                ts: base_ts + offset,
                event: EventType::AdminAction,
                ip: Some("198.51.100.1".to_string()),
                reason: Some(reason.to_string()),
                outcome: Some("ok".to_string()),
                admin: Some("tester".to_string()),
            }),
        };
        let all_rows = vec![
            make_row(2, "eventlog:v2:1:2-b", "c"),
            make_row(0, "eventlog:v2:1:0-a", "a"),
            make_row(1, "eventlog:v2:1:1-a", "b"),
        ];

        let (page_one, cursor_one, has_more_one, overflow_one) =
            paginate_cursor_rows(all_rows.clone(), "", 2);
        assert_eq!(page_one.len(), 2);
        assert!(has_more_one);
        assert_eq!(overflow_one, "limit_exceeded");

        let (page_two, _cursor_two, has_more_two, overflow_two) =
            paginate_cursor_rows(all_rows, cursor_one.as_str(), 2);
        assert_eq!(page_two.len(), 1);
        assert!(!has_more_two);
        assert_eq!(overflow_two, "none");
    }

    #[test]
    fn handle_admin_monitoring_delta_rejects_oversized_after_cursor() {
        let store = MockStore::new();
        let oversized = "a".repeat(513);
        let mut builder = Request::builder();
        builder.method(Method::Get).uri(
            format!("/admin/monitoring/delta?after_cursor={}", oversized).as_str(),
        );
        let req = builder.build();
        let resp = handle_admin_monitoring_delta(&req, &store);
        assert_eq!(*resp.status(), 400u16);
        assert!(String::from_utf8_lossy(resp.body()).contains("after_cursor must be <= 512 chars"));
    }

    #[test]
    fn handle_admin_monitoring_delta_includes_freshness_and_load_contracts() {
        let store = MockStore::new();
        let now = now_ts();
        let hour = now / 3600;
        let key = format!("eventlog:v2:{}:{}-freshness", hour, now);
        let event = EventLogEntry {
            ts: now,
            event: EventType::Challenge,
            ip: Some("198.51.100.44".to_string()),
            reason: Some("challenge_served".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("ops".to_string()),
        };
        store
            .set(&key, serde_json::to_vec(&event).unwrap().as_slice())
            .unwrap();

        let mut builder = Request::builder();
        builder
            .method(Method::Get)
            .uri("/admin/monitoring/delta?hours=1&limit=10");
        let req = builder.build();
        let resp = handle_admin_monitoring_delta(&req, &store);
        assert_eq!(*resp.status(), 200u16);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();

        assert!(payload.get("freshness_slo").is_some());
        assert!(payload.get("load_envelope").is_some());
        assert!(payload.get("freshness").is_some());
        assert_eq!(
            payload
                .get("stream_endpoint")
                .and_then(|value| value.as_str()),
            Some("/admin/monitoring/stream")
        );
        assert_eq!(
            payload
                .get("freshness")
                .and_then(|value| value.get("state"))
                .and_then(|value| value.as_str()),
            Some("fresh")
        );
    }

    #[test]
    fn handle_admin_monitoring_snapshot_includes_freshness_and_load_contracts() {
        let store = MockStore::new();
        let now = now_ts();
        let hour = now / 3600;
        let key = format!("eventlog:v2:{}:{}-snapshot-freshness", hour, now);
        let event = EventLogEntry {
            ts: now,
            event: EventType::Challenge,
            ip: Some("198.51.100.61".to_string()),
            reason: Some("challenge_served".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("ops".to_string()),
        };
        store
            .set(&key, serde_json::to_vec(&event).unwrap().as_slice())
            .unwrap();

        let mut builder = Request::builder();
        builder
            .method(Method::Get)
            .uri("/admin/monitoring?hours=1&limit=10");
        let req = builder.build();
        let resp = handle_admin_monitoring(&req, &store);
        assert_eq!(*resp.status(), 200u16);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();

        assert!(payload.get("freshness_slo").is_some());
        assert!(payload.get("load_envelope").is_some());
        assert!(payload.get("freshness").is_some());
        assert_eq!(
            payload
                .get("freshness")
                .and_then(|value| value.get("state"))
                .and_then(|value| value.as_str()),
            Some("fresh")
        );
        assert_eq!(
            payload
                .get("freshness")
                .and_then(|value| value.get("transport"))
                .and_then(|value| value.as_str()),
            Some("snapshot_poll")
        );
    }

    #[test]
    fn handle_admin_monitoring_delta_keeps_freshness_anchor_when_page_is_empty() {
        let store = MockStore::new();
        let now = now_ts();
        let hour = now / 3600;
        let key = format!("eventlog:v2:{}:{}-cursor-anchor", hour, now);
        let event = EventLogEntry {
            ts: now,
            event: EventType::Challenge,
            ip: Some("198.51.100.50".to_string()),
            reason: Some("challenge_served".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("ops".to_string()),
        };
        store
            .set(&key, serde_json::to_vec(&event).unwrap().as_slice())
            .unwrap();

        let mut baseline_builder = Request::builder();
        baseline_builder
            .method(Method::Get)
            .uri("/admin/monitoring/delta?hours=1&limit=10");
        let baseline_req = baseline_builder.build();
        let baseline_resp = handle_admin_monitoring_delta(&baseline_req, &store);
        assert_eq!(*baseline_resp.status(), 200u16);
        let baseline_payload: serde_json::Value =
            serde_json::from_slice(baseline_resp.body()).unwrap();
        let anchor_cursor = baseline_payload
            .get("window_end_cursor")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        assert!(!anchor_cursor.is_empty());

        let mut delta_builder = Request::builder();
        delta_builder
            .method(Method::Get)
            .uri(format!("/admin/monitoring/delta?hours=1&limit=10&after_cursor={}", anchor_cursor).as_str());
        let delta_req = delta_builder.build();
        let delta_resp = handle_admin_monitoring_delta(&delta_req, &store);
        assert_eq!(*delta_resp.status(), 200u16);
        let delta_payload: serde_json::Value = serde_json::from_slice(delta_resp.body()).unwrap();
        let events = delta_payload
            .get("events")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(events.is_empty());
        assert_eq!(
            delta_payload
                .get("freshness")
                .and_then(|value| value.get("last_event_ts"))
                .and_then(|value| value.as_u64()),
            Some(now)
        );
        assert_eq!(
            delta_payload
                .get("freshness")
                .and_then(|value| value.get("state"))
                .and_then(|value| value.as_str()),
            Some("fresh")
        );
    }

    #[test]
    fn handle_admin_monitoring_stream_resumes_from_last_event_id() {
        let store = MockStore::new();
        let now = now_ts();
        let hour = now / 3600;
        let first_key = format!("eventlog:v2:{}:{}-first", hour, now);
        let second_ts = now.saturating_add(1);
        let second_key = format!("eventlog:v2:{}:{}-second", hour, second_ts);

        let first_event = EventLogEntry {
            ts: now,
            event: EventType::Challenge,
            ip: Some("203.0.113.1".to_string()),
            reason: Some("challenge_served".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("ops".to_string()),
        };
        let second_event = EventLogEntry {
            ts: second_ts,
            event: EventType::Block,
            ip: Some("203.0.113.2".to_string()),
            reason: Some("blocked".to_string()),
            outcome: Some("blocked".to_string()),
            admin: Some("ops".to_string()),
        };
        store
            .set(&first_key, serde_json::to_vec(&first_event).unwrap().as_slice())
            .unwrap();
        store
            .set(&second_key, serde_json::to_vec(&second_event).unwrap().as_slice())
            .unwrap();

        let first_cursor = build_event_cursor(now, first_key.as_str());
        let second_cursor = build_event_cursor(second_ts, second_key.as_str());
        let mut builder = Request::builder();
        builder
            .method(Method::Get)
            .uri("/admin/monitoring/stream?hours=1&limit=10")
            .header("Last-Event-ID", first_cursor.as_str());
        let req = builder.build();
        let resp = handle_admin_monitoring_stream(&req, &store);
        assert_eq!(*resp.status(), 200u16);
        assert!(
            resp.header("connection")
                .and_then(|value| value.as_str())
                .is_none()
        );

        let body = String::from_utf8_lossy(resp.body()).to_string();
        assert!(body.contains("event: monitoring_delta"));
        assert!(body.contains(format!("id: {}", second_cursor).as_str()));
        let data_line = body
            .lines()
            .find(|line| line.starts_with("data: "))
            .expect("expected data line in SSE payload");
        let payload: serde_json::Value =
            serde_json::from_str(data_line.trim_start_matches("data: ")).unwrap();
        let events = payload
            .get("events")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert_eq!(events.len(), 1);
        assert_eq!(
            payload
                .get("after_cursor")
                .and_then(|value| value.as_str()),
            Some(first_cursor.as_str())
        );
    }

    #[test]
    fn handle_admin_monitoring_stream_orders_event_ids_across_reconnects() {
        let store = MockStore::new();
        let now = now_ts();
        let hour = now / 3600;
        let first_key = format!("eventlog:v2:{}:{}-first-order", hour, now);
        let second_ts = now.saturating_add(1);
        let second_key = format!("eventlog:v2:{}:{}-second-order", hour, second_ts);

        let first_event = EventLogEntry {
            ts: now,
            event: EventType::Challenge,
            ip: Some("198.51.100.2".to_string()),
            reason: Some("challenge_served".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("ops".to_string()),
        };
        let second_event = EventLogEntry {
            ts: second_ts,
            event: EventType::Block,
            ip: Some("198.51.100.3".to_string()),
            reason: Some("blocked".to_string()),
            outcome: Some("blocked".to_string()),
            admin: Some("ops".to_string()),
        };
        store
            .set(&first_key, serde_json::to_vec(&first_event).unwrap().as_slice())
            .unwrap();
        store
            .set(&second_key, serde_json::to_vec(&second_event).unwrap().as_slice())
            .unwrap();

        let first_cursor = build_event_cursor(now, first_key.as_str());
        let second_cursor = build_event_cursor(second_ts, second_key.as_str());

        let mut first_builder = Request::builder();
        first_builder
            .method(Method::Get)
            .uri("/admin/monitoring/stream?hours=1&limit=1");
        let first_req = first_builder.build();
        let first_resp = handle_admin_monitoring_stream(&first_req, &store);
        assert_eq!(*first_resp.status(), 200u16);
        let first_body = String::from_utf8_lossy(first_resp.body()).to_string();
        assert!(first_body.contains(format!("id: {}", first_cursor).as_str()));

        let mut second_builder = Request::builder();
        second_builder
            .method(Method::Get)
            .uri("/admin/monitoring/stream?hours=1&limit=1")
            .header("Last-Event-ID", first_cursor.as_str());
        let second_req = second_builder.build();
        let second_resp = handle_admin_monitoring_stream(&second_req, &store);
        assert_eq!(*second_resp.status(), 200u16);
        let second_body = String::from_utf8_lossy(second_resp.body()).to_string();
        assert!(second_body.contains(format!("id: {}", second_cursor).as_str()));
    }

    #[test]
    fn handle_admin_ip_bans_delta_filters_to_ban_and_unban_events() {
        let store = MockStore::new();
        let now = now_ts();
        let hour = now / 3600;
        let ban_key = format!("eventlog:v2:{}:{}-ban", hour, now);
        let unban_key = format!("eventlog:v2:{}:{}-unban", hour, now.saturating_add(1));
        let challenge_key = format!("eventlog:v2:{}:{}-challenge", hour, now.saturating_add(2));

        let ban_event = EventLogEntry {
            ts: now,
            event: EventType::Ban,
            ip: Some("203.0.113.10".to_string()),
            reason: Some("manual_ban".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("ops".to_string()),
        };
        let unban_event = EventLogEntry {
            ts: now.saturating_add(1),
            event: EventType::Unban,
            ip: Some("203.0.113.10".to_string()),
            reason: Some("manual_unban".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("ops".to_string()),
        };
        let challenge_event = EventLogEntry {
            ts: now.saturating_add(2),
            event: EventType::Challenge,
            ip: Some("203.0.113.20".to_string()),
            reason: Some("challenge_served".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("ops".to_string()),
        };

        store
            .set(&ban_key, serde_json::to_vec(&ban_event).unwrap().as_slice())
            .unwrap();
        store
            .set(&unban_key, serde_json::to_vec(&unban_event).unwrap().as_slice())
            .unwrap();
        store
            .set(
                &challenge_key,
                serde_json::to_vec(&challenge_event).unwrap().as_slice(),
            )
            .unwrap();

        let mut builder = Request::builder();
        builder.method(Method::Get).uri("/admin/ip-bans/delta?hours=1&limit=10");
        let req = builder.build();
        let resp = handle_admin_ip_bans_delta(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let events = payload
            .get("events")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|row| {
            row.get("event")
                .and_then(|value| value.as_str())
                .map(|event| event == "Ban" || event == "Unban")
                .unwrap_or(false)
        }));
    }

    #[test]
    fn handle_admin_ip_bans_delta_preserves_simulation_metadata() {
        let store = MockStore::new();
        let now = now_ts();

        {
            let _guard = crate::runtime::sim_telemetry::enter(Some(
                crate::runtime::sim_telemetry::SimulationRequestMetadata {
                    sim_run_id: "run-ipbans-sim".to_string(),
                    sim_profile: "fast_smoke".to_string(),
                    sim_lane: "deterministic_black_box".to_string(),
                },
            ));
            log_event(
                &store,
                &EventLogEntry {
                    ts: now,
                    event: EventType::Ban,
                    ip: Some("203.0.113.44".to_string()),
                    reason: Some("sim_ban".to_string()),
                    outcome: Some("ok".to_string()),
                    admin: Some("ops".to_string()),
                },
            );
        }

        log_event(
            &store,
            &EventLogEntry {
                ts: now.saturating_add(1),
                event: EventType::Ban,
                ip: Some("203.0.113.45".to_string()),
                reason: Some("baseline_ban".to_string()),
                outcome: Some("ok".to_string()),
                admin: Some("ops".to_string()),
            },
        );

        let mut builder = Request::builder();
        builder.method(Method::Get).uri("/admin/ip-bans/delta?hours=1&limit=10");
        let req = builder.build();
        let resp = handle_admin_ip_bans_delta(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let events = payload
            .get("events")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(events.len() >= 2);
        assert!(events.iter().any(|row| {
            row.get("is_simulation").and_then(|value| value.as_bool()) == Some(true)
                && row.get("sim_run_id").and_then(|value| value.as_str()) == Some("run-ipbans-sim")
        }));
        assert!(events.iter().any(|row| {
            row.get("is_simulation").and_then(|value| value.as_bool()) == Some(false)
                && row.get("reason").and_then(|value| value.as_str()) == Some("baseline_ban")
        }));
    }

    #[test]
    fn load_recent_events_ignores_legacy_v1_pages() {
        let store = MockStore::new();
        let now = now_ts();
        let entry = EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: Some("1.2.3.4".to_string()),
            reason: Some("legacy".to_string()),
            outcome: Some("ok".to_string()),
            admin: Some("me".to_string()),
        };
        let hour = now / 3600;
        let key = format!("eventlog:{}:1", hour);
        let page = vec![entry];
        store
            .set(&key, serde_json::to_vec(&page).unwrap().as_slice())
            .unwrap();

        let events = load_recent_events(&store, now, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn retention_worker_deletes_eventlog_buckets_older_than_retention() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "2");

        let store = MockStore::new();
        let current_hour = now_ts() / 3600;
        let stale_hours = [
            current_hour.saturating_sub(6),
            current_hour.saturating_sub(4),
            current_hour.saturating_sub(3),
        ];
        let retained_hour = current_hour.saturating_sub(2);

        for hour in stale_hours {
            let key = format!("eventlog:v2:{}:{}-stale", hour, hour.saturating_mul(3600));
            store.set(&key, br#"{"stale":true}"#).unwrap();
            crate::observability::retention::register_event_log_key(&store, hour, key.as_str());
        }
        let retained_key = format!(
            "eventlog:v2:{}:{}-retained",
            retained_hour,
            retained_hour.saturating_mul(3600)
        );
        store.set(&retained_key, br#"{"retained":true}"#).unwrap();
        crate::observability::retention::register_event_log_key(
            &store,
            retained_hour,
            retained_key.as_str(),
        );
        crate::observability::retention::run_worker_if_due(&store);

        for hour in stale_hours {
            let key = format!("eventlog:v2:{}:{}-stale", hour, hour.saturating_mul(3600));
            assert!(
                store.get(&key).unwrap().is_none(),
                "expected stale key {} to be deleted",
                key
            );
        }
        assert!(store.get(&retained_key).unwrap().is_some());
        std::env::remove_var("SHUMA_EVENT_LOG_RETENTION_HOURS");
    }

    #[test]
    fn expensive_admin_read_limiter_blocks_at_limit() {
        let store = MockStore::new();
        let mut builder = spin_sdk::http::Request::builder();
        builder.method(Method::Get).uri("/admin/events");
        let req = builder.build();

        let ip = crate::extract_client_ip(&req);
        let bucket = crate::signals::ip_identity::bucket_ip(&ip);
        let now_window = now_ts() / 60;
        for window in [now_window, now_window + 1] {
            let key = format!(
                "rate:{}:{}:{}",
                ADMIN_EXPENSIVE_READ_SITE_ID, bucket, window
            );
            store
                .set(
                    &key,
                    ADMIN_EXPENSIVE_READ_LIMIT_PER_MINUTE
                        .to_string()
                        .as_bytes(),
                )
                .unwrap();
        }

        assert!(expensive_admin_read_limit_check_internal_with_identity(
            &store,
            &ip,
            ADMIN_EXPENSIVE_READ_SITE_ID,
            ADMIN_EXPENSIVE_READ_LIMIT_PER_MINUTE
        ));
    }

    #[test]
    fn dashboard_refresh_limiter_blocks_session_burst_at_limit() {
        let store = MockStore::new();
        let auth = crate::admin::auth::AdminAuthResult {
            method: Some(crate::admin::auth::AdminAuthMethod::SessionCookie),
            access: Some(crate::admin::auth::AdminAccessLevel::ReadWrite),
            csrf_token: Some("csrf-token".to_string()),
            session_id: Some("session-abc".to_string()),
            session_expires_at: Some(now_ts().saturating_add(3600)),
        };

        let session_scope = dashboard_refresh_session_scope(&auth).expect("session scope");
        let bucket = crate::signals::ip_identity::bucket_ip(&session_scope);
        let now_window = now_ts() / 60;
        for window in [now_window, now_window + 1] {
            let key = format!(
                "rate:{}:{}:{}",
                ADMIN_DASHBOARD_REFRESH_SESSION_SITE_ID, bucket, window
            );
            store
                .set(
                    &key,
                    ADMIN_DASHBOARD_REFRESH_SESSION_LIMIT_PER_MINUTE
                        .to_string()
                        .as_bytes(),
                )
                .unwrap();
        }

        assert!(expensive_admin_read_limit_check_internal_with_identity(
            &store,
            &session_scope,
            ADMIN_DASHBOARD_REFRESH_SESSION_SITE_ID,
            ADMIN_DASHBOARD_REFRESH_SESSION_LIMIT_PER_MINUTE
        ));
    }

    #[test]
    fn dashboard_refresh_limiter_ignores_non_session_auth() {
        let auth = crate::admin::auth::AdminAuthResult {
            method: Some(crate::admin::auth::AdminAuthMethod::BearerToken),
            access: Some(crate::admin::auth::AdminAccessLevel::ReadOnly),
            csrf_token: None,
            session_id: None,
            session_expires_at: None,
        };
        assert!(dashboard_refresh_session_scope(&auth).is_none());
    }

    #[test]
    fn monitoring_refresh_limits_support_one_hz_dashboard_polling_contract() {
        assert!(
            ADMIN_DASHBOARD_REFRESH_SESSION_LIMIT_PER_MINUTE >= 60,
            "dashboard refresh session limiter must allow at least 1Hz polling"
        );
        assert!(
            ADMIN_EXPENSIVE_READ_SESSION_LIMIT_PER_MINUTE >= 60,
            "expensive read session limiter must allow at least 1Hz polling"
        );
        assert!(
            ADMIN_EXPENSIVE_READ_LIMIT_PER_MINUTE >= 60,
            "expensive read IP limiter must allow at least 1Hz polling"
        );
    }

    #[test]
    fn query_u64_param_parses_multi_param_query() {
        let query = "hours=24&limit=500";
        assert_eq!(query_u64_param(query, "hours", 1), 24);
        assert_eq!(query_u64_param(query, "limit", 10), 500);
        assert_eq!(query_u64_param(query, "missing", 42), 42);
    }

    #[test]
    fn is_cdp_event_reason_matches_detection_and_auto_ban() {
        assert!(is_cdp_event_reason("cdp_detected:tier=medium score=0.7"));
        assert!(is_cdp_event_reason("cdp_automation"));
        assert!(!is_cdp_event_reason("maze_crawler"));
    }

    #[test]
    fn parse_unban_identity_allows_unknown_bucket() {
        assert_eq!(
            parse_unban_identity("unknown"),
            Some("unknown".to_string())
        );
        assert_eq!(
            parse_unban_identity(" UnKnOwN "),
            Some("unknown".to_string())
        );
        assert_eq!(
            parse_unban_identity("198.51.100.7"),
            Some("198.51.100.7".to_string())
        );
    }

    #[test]
    fn parse_unban_identity_rejects_invalid_values() {
        assert_eq!(parse_unban_identity(""), None);
        assert_eq!(parse_unban_identity("not-an-ip"), None);
    }
}

#[cfg(test)]
mod admin_config_tests {
    use super::*;
    use crate::challenge::KeyValueStore;
    use flate2::read::GzDecoder;
    use spin_sdk::http::{Method, Request};
    use std::collections::HashMap;
    use std::io::Read;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;

    static IDEMPOTENCY_COUNTER: AtomicU64 = AtomicU64::new(1);

    fn make_request(method: Method, path: &str, body: Vec<u8>) -> Request {
        let idempotency = format!(
            "test-idempotency-key-{}",
            IDEMPOTENCY_COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let mut builder = Request::builder();
        builder
            .method(method)
            .uri(path)
            .header("host", "localhost:3000")
            .header("authorization", "Bearer changeme-dev-only-api-key")
            .header("origin", "http://localhost:3000")
            .header("sec-fetch-site", "same-origin")
            .header("idempotency-key", idempotency.as_str())
            .body(body);
        builder.build()
    }

    fn bearer_rw_auth() -> crate::admin::auth::AdminAuthResult {
        crate::admin::auth::AdminAuthResult {
            method: Some(crate::admin::auth::AdminAuthMethod::BearerToken),
            access: Some(crate::admin::auth::AdminAccessLevel::ReadWrite),
            csrf_token: None,
            session_id: None,
            session_expires_at: None,
        }
    }

    fn session_rw_auth(
        session_id: &str,
        csrf_token: &str,
        session_expires_at: u64,
    ) -> crate::admin::auth::AdminAuthResult {
        crate::admin::auth::AdminAuthResult {
            method: Some(crate::admin::auth::AdminAuthMethod::SessionCookie),
            access: Some(crate::admin::auth::AdminAccessLevel::ReadWrite),
            csrf_token: Some(csrf_token.to_string()),
            session_id: Some(session_id.to_string()),
            session_expires_at: Some(session_expires_at),
        }
    }

    fn make_control_request_with_trust_headers(
        enabled: bool,
        idempotency_key: &str,
        origin: Option<&str>,
        fetch_site: Option<&str>,
        csrf_token: Option<&str>,
    ) -> Request {
        let body = if enabled {
            br#"{"enabled":true}"#.to_vec()
        } else {
            br#"{"enabled":false}"#.to_vec()
        };
        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/admin/adversary-sim/control")
            .header("host", "localhost:3000")
            .header("authorization", "Bearer changeme-dev-only-api-key")
            .header("idempotency-key", idempotency_key);
        if let Some(value) = origin {
            builder.header("origin", value);
        }
        if let Some(value) = fetch_site {
            builder.header("sec-fetch-site", value);
        }
        if let Some(value) = csrf_token {
            builder.header("x-shuma-csrf", value);
        }
        builder.body(body);
        builder.build()
    }

    fn make_control_request(enabled: bool, idempotency_key: &str) -> Request {
        make_control_request_with_trust_headers(
            enabled,
            idempotency_key,
            Some("http://localhost:3000"),
            Some("same-origin"),
            None,
        )
    }

    fn make_internal_beat_request(api_key: &str) -> Request {
        let authorization = format!("Bearer {}", api_key);
        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/internal/adversary-sim/beat")
            .header("host", "localhost:3000")
            .header("authorization", authorization.as_str())
            .body(Vec::new());
        builder.build()
    }

    fn collect_control_audit_decisions(store: &TestStore) -> Vec<String> {
        let map = store.map.lock().unwrap();
        map.values()
            .filter_map(|value| serde_json::from_slice::<serde_json::Value>(value).ok())
            .filter(|row| {
                row.get("reason")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    == "adversary_sim_control_audit"
            })
            .filter_map(|row| {
                let outcome = row
                    .get("outcome")
                    .and_then(|value| value.as_str())
                    .unwrap_or("{}");
                let details = serde_json::from_str::<serde_json::Value>(outcome).ok()?;
                details
                    .get("decision")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string())
            })
            .collect()
    }

    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl Default for TestStore {
        fn default() -> Self {
            let mut map = HashMap::new();
            let cfg = crate::config::defaults().clone();
            map.insert(
                "config:default".to_string(),
                serde_json::to_vec(&cfg).unwrap(),
            );
            Self {
                map: Mutex::new(map),
            }
        }
    }

    impl crate::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let m = self.map.lock().unwrap();
            Ok(m.get(key).cloned())
        }
        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.insert(key.to_string(), value.to_vec());
            Ok(())
        }
        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.remove(key);
            Ok(())
        }

        fn get_keys(&self) -> Result<Vec<String>, ()> {
            let m = self.map.lock().unwrap();
            Ok(m.keys().cloned().collect())
        }
    }

    impl crate::maze::state::MazeStateStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            crate::challenge::KeyValueStore::get(self, key)
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            crate::challenge::KeyValueStore::set(self, key, value)
        }
    }

    fn clear_env(keys: &[&str]) {
        for key in keys {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn admin_config_export_returns_non_secret_runtime_values() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_IP_ALLOWLIST", "203.0.113.0/24,198.51.100.8");
        std::env::set_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE", "17");
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "240");
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "false");
        std::env::set_var("SHUMA_ENFORCE_HTTPS", "true");
        std::env::set_var("SHUMA_DEBUG_HEADERS", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");
        std::env::set_var("SHUMA_FRONTIER_OPENAI_MODEL", "gpt-5-mini");
        std::env::set_var("SHUMA_FRONTIER_ANTHROPIC_MODEL", "claude-3-5-haiku-latest");
        std::env::set_var("SHUMA_FRONTIER_GOOGLE_MODEL", "gemini-2.0-flash-lite");
        std::env::set_var("SHUMA_FRONTIER_XAI_MODEL", "grok-3-mini");
        std::env::set_var("SHUMA_RATE_LIMITER_REDIS_URL", "redis://redis:6379");
        std::env::set_var("SHUMA_BAN_STORE_REDIS_URL", "redis://redis:6379");
        std::env::set_var("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN", "fail_open");
        std::env::set_var("SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH", "fail_closed");

        let store = TestStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.rate_limit = 321;
        cfg.honeypot_enabled = false;
        cfg.browser_policy_enabled = false;
        cfg.bypass_allowlists_enabled = false;
        cfg.path_allowlist_enabled = false;
        cfg.adversary_sim_enabled = true;
        cfg.tarpit_enabled = false;
        cfg.tarpit_progress_token_ttl_seconds = 140;
        cfg.tarpit_progress_replay_ttl_seconds = 360;
        cfg.tarpit_step_chunk_base_bytes = 4096;
        cfg.tarpit_step_chunk_max_bytes = 12288;
        cfg.tarpit_egress_global_bytes_per_window = 5_242_880;
        cfg.tarpit_max_concurrent_global = 40;
        cfg.tarpit_max_concurrent_per_ip_bucket = 3;
        cfg.tarpit_fallback_action = crate::config::TarpitFallbackAction::Block;
        cfg.challenge_puzzle_enabled = false;
        cfg.honeypots = vec!["/trap-a".to_string(), "/trap-b".to_string()];
        cfg.defence_modes.rate = crate::config::ComposabilityMode::Signal;
        cfg.provider_backends.fingerprint_signal = crate::config::ProviderBackend::External;
        cfg.edge_integration_mode = crate::config::EdgeIntegrationMode::Additive;
        store
            .set("config:default", &serde_json::to_vec(&cfg).unwrap())
            .unwrap();

        let req = make_request(Method::Get, "/admin/config/export", Vec::new());
        let resp = handle_admin_config_export(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);

        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let env = body.get("env").and_then(|v| v.as_object()).unwrap();
        assert_eq!(env.get("SHUMA_RATE_LIMIT"), Some(&serde_json::json!("321")));
        assert_eq!(
            env.get("SHUMA_HONEYPOTS"),
            Some(&serde_json::json!("[\"/trap-a\",\"/trap-b\"]"))
        );
        assert_eq!(
            env.get("SHUMA_MODE_RATE"),
            Some(&serde_json::json!("signal"))
        );
        assert_eq!(
            env.get("SHUMA_PROVIDER_FINGERPRINT_SIGNAL"),
            Some(&serde_json::json!("external"))
        );
        assert_eq!(
            env.get("SHUMA_EDGE_INTEGRATION_MODE"),
            Some(&serde_json::json!("additive"))
        );
        assert_eq!(
            env.get("SHUMA_HONEYPOT_ENABLED"),
            Some(&serde_json::json!("false"))
        );
        assert_eq!(
            env.get("SHUMA_BROWSER_POLICY_ENABLED"),
            Some(&serde_json::json!("false"))
        );
        assert_eq!(
            env.get("SHUMA_BYPASS_ALLOWLISTS_ENABLED"),
            Some(&serde_json::json!("false"))
        );
        assert_eq!(
            env.get("SHUMA_PATH_ALLOWLIST_ENABLED"),
            Some(&serde_json::json!("false"))
        );
        assert_eq!(
            env.get("SHUMA_TARPIT_ENABLED"),
            Some(&serde_json::json!("false"))
        );
        assert_eq!(
            env.get("SHUMA_TARPIT_PROGRESS_TOKEN_TTL_SECONDS"),
            Some(&serde_json::json!("140"))
        );
        assert_eq!(
            env.get("SHUMA_TARPIT_PROGRESS_REPLAY_TTL_SECONDS"),
            Some(&serde_json::json!("360"))
        );
        assert_eq!(
            env.get("SHUMA_TARPIT_STEP_CHUNK_BASE_BYTES"),
            Some(&serde_json::json!("4096"))
        );
        assert_eq!(
            env.get("SHUMA_TARPIT_STEP_CHUNK_MAX_BYTES"),
            Some(&serde_json::json!("12288"))
        );
        assert_eq!(
            env.get("SHUMA_TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW"),
            Some(&serde_json::json!("5242880"))
        );
        assert_eq!(
            env.get("SHUMA_TARPIT_MAX_CONCURRENT_GLOBAL"),
            Some(&serde_json::json!("40"))
        );
        assert_eq!(
            env.get("SHUMA_TARPIT_MAX_CONCURRENT_PER_IP_BUCKET"),
            Some(&serde_json::json!("3"))
        );
        assert_eq!(
            env.get("SHUMA_TARPIT_FALLBACK_ACTION"),
            Some(&serde_json::json!("block"))
        );
        assert_eq!(
            env.get("SHUMA_CHALLENGE_PUZZLE_ENABLED"),
            Some(&serde_json::json!("false"))
        );
        assert!(env.get("SHUMA_CHALLENGE_PUZZLE_SEED_TTL_SECONDS").is_some());
        assert!(env
            .get("SHUMA_CHALLENGE_PUZZLE_ATTEMPT_LIMIT_PER_WINDOW")
            .is_some());
        assert!(env
            .get("SHUMA_CHALLENGE_PUZZLE_ATTEMPT_WINDOW_SECONDS")
            .is_some());
        assert_eq!(
            env.get("SHUMA_ADMIN_IP_ALLOWLIST"),
            Some(&serde_json::json!("203.0.113.0/24,198.51.100.8"))
        );
        assert_eq!(
            env.get("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE"),
            Some(&serde_json::json!("17"))
        );
        assert_eq!(
            env.get("SHUMA_EVENT_LOG_RETENTION_HOURS"),
            Some(&serde_json::json!("240"))
        );
        assert_eq!(
            env.get("SHUMA_ADMIN_CONFIG_WRITE_ENABLED"),
            Some(&serde_json::json!("true"))
        );
        assert_eq!(
            env.get("SHUMA_KV_STORE_FAIL_OPEN"),
            Some(&serde_json::json!("false"))
        );
        assert_eq!(
            env.get("SHUMA_ENFORCE_HTTPS"),
            Some(&serde_json::json!("true"))
        );
        assert_eq!(
            env.get("SHUMA_DEBUG_HEADERS"),
            Some(&serde_json::json!("true"))
        );
        assert_eq!(
            env.get("SHUMA_RUNTIME_ENV"),
            Some(&serde_json::json!("runtime-dev"))
        );
        assert_eq!(
            env.get("SHUMA_ADVERSARY_SIM_AVAILABLE"),
            Some(&serde_json::json!("true"))
        );
        assert_eq!(
            env.get("SHUMA_FRONTIER_OPENAI_MODEL"),
            Some(&serde_json::json!("gpt-5-mini"))
        );
        assert_eq!(
            env.get("SHUMA_FRONTIER_ANTHROPIC_MODEL"),
            Some(&serde_json::json!("claude-3-5-haiku-latest"))
        );
        assert_eq!(
            env.get("SHUMA_FRONTIER_GOOGLE_MODEL"),
            Some(&serde_json::json!("gemini-2.0-flash-lite"))
        );
        assert_eq!(
            env.get("SHUMA_FRONTIER_XAI_MODEL"),
            Some(&serde_json::json!("grok-3-mini"))
        );
        assert!(env.get("SHUMA_FRONTIER_OPENAI_API_KEY").is_none());
        assert_eq!(
            env.get("SHUMA_ADVERSARY_SIM_ENABLED"),
            Some(&serde_json::json!("false"))
        );
        assert_eq!(
            env.get("SHUMA_ADVERSARY_SIM_DURATION_SECONDS"),
            Some(&serde_json::json!(cfg.adversary_sim_duration_seconds.to_string()))
        );
        assert!(env.get("SHUMA_RATE_LIMITER_REDIS_URL").is_none());
        assert!(env.get("SHUMA_BAN_STORE_REDIS_URL").is_none());
        assert_eq!(
            env.get("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN"),
            Some(&serde_json::json!("fail_open"))
        );
        assert_eq!(
            env.get("SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH"),
            Some(&serde_json::json!("fail_closed"))
        );

        let env_text = body.get("env_text").and_then(|v| v.as_str()).unwrap();
        assert!(env_text.contains("SHUMA_RATE_LIMIT=321"));
        assert!(env_text.contains("SHUMA_MODE_RATE=signal"));
        assert!(env_text.contains("SHUMA_PROVIDER_FINGERPRINT_SIGNAL=external"));
        assert!(env_text.contains("SHUMA_HONEYPOT_ENABLED=false"));
        assert!(env_text.contains("SHUMA_BROWSER_POLICY_ENABLED=false"));
        assert!(env_text.contains("SHUMA_BYPASS_ALLOWLISTS_ENABLED=false"));
        assert!(env_text.contains("SHUMA_PATH_ALLOWLIST_ENABLED=false"));
        assert!(env_text.contains("SHUMA_TARPIT_ENABLED=false"));
        assert!(env_text.contains("SHUMA_TARPIT_PROGRESS_TOKEN_TTL_SECONDS=140"));
        assert!(env_text.contains("SHUMA_TARPIT_PROGRESS_REPLAY_TTL_SECONDS=360"));
        assert!(env_text.contains("SHUMA_TARPIT_STEP_CHUNK_BASE_BYTES=4096"));
        assert!(env_text.contains("SHUMA_TARPIT_STEP_CHUNK_MAX_BYTES=12288"));
        assert!(env_text.contains("SHUMA_TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW=5242880"));
        assert!(env_text.contains("SHUMA_TARPIT_MAX_CONCURRENT_GLOBAL=40"));
        assert!(env_text.contains("SHUMA_TARPIT_MAX_CONCURRENT_PER_IP_BUCKET=3"));
        assert!(env_text.contains("SHUMA_TARPIT_FALLBACK_ACTION=block"));
        assert!(env_text.contains("SHUMA_CHALLENGE_PUZZLE_ENABLED=false"));
        assert!(env_text.contains("SHUMA_CHALLENGE_PUZZLE_SEED_TTL_SECONDS="));
        assert!(env_text.contains("SHUMA_CHALLENGE_PUZZLE_ATTEMPT_LIMIT_PER_WINDOW="));
        assert!(env_text.contains("SHUMA_CHALLENGE_PUZZLE_ATTEMPT_WINDOW_SECONDS="));
        assert!(env_text.contains("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE=17"));
        assert!(env_text.contains("SHUMA_RUNTIME_ENV=runtime-dev"));
        assert!(env_text.contains("SHUMA_ADVERSARY_SIM_AVAILABLE=true"));
        assert!(env_text.contains("SHUMA_FRONTIER_OPENAI_MODEL=gpt-5-mini"));
        assert!(env_text.contains("SHUMA_FRONTIER_ANTHROPIC_MODEL=claude-3-5-haiku-latest"));
        assert!(env_text.contains("SHUMA_FRONTIER_GOOGLE_MODEL=gemini-2.0-flash-lite"));
        assert!(env_text.contains("SHUMA_FRONTIER_XAI_MODEL=grok-3-mini"));
        assert!(!env_text.contains("SHUMA_FRONTIER_OPENAI_API_KEY="));
        assert!(env_text.contains("SHUMA_ADVERSARY_SIM_ENABLED=false"));
        assert!(env_text.contains("SHUMA_ADVERSARY_SIM_DURATION_SECONDS="));
        assert!(!env_text.contains("SHUMA_RATE_LIMITER_REDIS_URL="));
        assert!(!env_text.contains("SHUMA_BAN_STORE_REDIS_URL="));
        assert!(env_text.contains("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN=fail_open"));
        assert!(env_text.contains("SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH=fail_closed"));

        clear_env(&[
            "SHUMA_ADMIN_IP_ALLOWLIST",
            "SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE",
            "SHUMA_EVENT_LOG_RETENTION_HOURS",
            "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
            "SHUMA_KV_STORE_FAIL_OPEN",
            "SHUMA_ENFORCE_HTTPS",
            "SHUMA_DEBUG_HEADERS",
            "SHUMA_RUNTIME_ENV",
            "SHUMA_ADVERSARY_SIM_AVAILABLE",
            "SHUMA_FRONTIER_OPENAI_MODEL",
            "SHUMA_FRONTIER_ANTHROPIC_MODEL",
            "SHUMA_FRONTIER_GOOGLE_MODEL",
            "SHUMA_FRONTIER_XAI_MODEL",
            "SHUMA_RATE_LIMITER_REDIS_URL",
            "SHUMA_BAN_STORE_REDIS_URL",
            "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
            "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
        ]);
    }

    #[test]
    fn admin_config_export_omits_secret_values() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "admin-key-secret");
        std::env::set_var("SHUMA_JS_SECRET", "js-secret");
        std::env::set_var("SHUMA_POW_SECRET", "pow-secret");
        std::env::set_var("SHUMA_CHALLENGE_SECRET", "challenge-secret");
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "forwarded-secret");
        std::env::set_var("SHUMA_HEALTH_SECRET", "health-secret");
        std::env::set_var("SHUMA_SIM_TELEMETRY_SECRET", "sim-telemetry-secret");
        std::env::set_var("SHUMA_FRONTIER_OPENAI_API_KEY", "frontier-openai-secret");
        std::env::set_var("SHUMA_FRONTIER_ANTHROPIC_API_KEY", "frontier-anthropic-secret");
        std::env::set_var("SHUMA_FRONTIER_GOOGLE_API_KEY", "frontier-google-secret");
        std::env::set_var("SHUMA_FRONTIER_XAI_API_KEY", "frontier-xai-secret");
        std::env::set_var("SHUMA_RATE_LIMITER_REDIS_URL", "redis://secret@redis:6379");
        std::env::set_var("SHUMA_BAN_STORE_REDIS_URL", "redis://secret@redis:6379");

        let store = TestStore::default();
        let req = make_request(Method::Get, "/admin/config/export", Vec::new());
        let resp = handle_admin_config_export(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);

        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let env = body.get("env").and_then(|v| v.as_object()).unwrap();
        for secret_key in CONFIG_EXPORT_SECRET_KEYS {
            assert!(env.get(secret_key).is_none());
        }

        let env_text = body.get("env_text").and_then(|v| v.as_str()).unwrap();
        for secret_key in CONFIG_EXPORT_SECRET_KEYS {
            assert!(!env_text.contains(&format!("{}=", secret_key)));
        }

        let excluded = body
            .get("excluded_secrets")
            .and_then(|v| v.as_array())
            .unwrap();
        for secret_key in CONFIG_EXPORT_SECRET_KEYS {
            assert!(excluded
                .iter()
                .any(|item| item.as_str() == Some(secret_key)));
        }

        clear_env(&[
            "SHUMA_API_KEY",
            "SHUMA_JS_SECRET",
            "SHUMA_POW_SECRET",
            "SHUMA_CHALLENGE_SECRET",
            "SHUMA_FORWARDED_IP_SECRET",
            "SHUMA_HEALTH_SECRET",
            "SHUMA_SIM_TELEMETRY_SECRET",
            "SHUMA_FRONTIER_OPENAI_API_KEY",
            "SHUMA_FRONTIER_ANTHROPIC_API_KEY",
            "SHUMA_FRONTIER_GOOGLE_API_KEY",
            "SHUMA_FRONTIER_XAI_API_KEY",
            "SHUMA_RATE_LIMITER_REDIS_URL",
            "SHUMA_BAN_STORE_REDIS_URL",
        ]);
    }

    #[test]
    fn admin_config_includes_challenge_fields() {
        let _lock = crate::test_support::lock_env();
        let req = make_request(Method::Get, "/admin/config", Vec::new());
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert!(body.get("challenge_puzzle_risk_threshold").is_some());
        assert!(body.get("not_a_bot_risk_threshold").is_some());
        assert!(body.get("not_a_bot_enabled").is_some());
        assert!(body.get("not_a_bot_risk_threshold_default").is_some());
        assert!(body.get("challenge_puzzle_enabled").is_some());
        assert!(body.get("tarpit_enabled").is_some());
        assert!(body.get("tarpit_progress_token_ttl_seconds").is_some());
        assert!(body.get("tarpit_progress_replay_ttl_seconds").is_some());
        assert!(body.get("tarpit_hashcash_min_difficulty").is_some());
        assert!(body.get("tarpit_hashcash_max_difficulty").is_some());
        assert!(body.get("tarpit_hashcash_base_difficulty").is_some());
        assert!(body.get("tarpit_hashcash_adaptive").is_some());
        assert!(body.get("tarpit_step_chunk_base_bytes").is_some());
        assert!(body.get("tarpit_step_chunk_max_bytes").is_some());
        assert!(body.get("tarpit_step_jitter_percent").is_some());
        assert!(body.get("tarpit_shard_rotation_enabled").is_some());
        assert!(body.get("tarpit_egress_window_seconds").is_some());
        assert!(body.get("tarpit_egress_global_bytes_per_window").is_some());
        assert!(body.get("tarpit_egress_per_ip_bucket_bytes_per_window").is_some());
        assert!(body.get("tarpit_egress_per_flow_max_bytes").is_some());
        assert!(body.get("tarpit_egress_per_flow_max_duration_seconds").is_some());
        assert!(body.get("tarpit_max_concurrent_global").is_some());
        assert!(body.get("tarpit_max_concurrent_per_ip_bucket").is_some());
        assert!(body.get("tarpit_fallback_action").is_some());
        assert!(body.get("challenge_puzzle_risk_threshold_default").is_some());
        assert!(body.get("challenge_puzzle_transform_count").is_some());
        assert!(body.get("challenge_puzzle_seed_ttl_seconds").is_some());
        assert!(body.get("challenge_puzzle_attempt_limit_per_window").is_some());
        assert!(body.get("challenge_puzzle_attempt_window_seconds").is_some());
        assert!(body.get("ai_policy_block_training").is_some());
        assert!(body.get("ai_policy_block_search").is_some());
        assert!(body.get("ai_policy_allow_search_engines").is_some());
        assert!(body.get("botness_maze_threshold").is_some());
        assert!(body.get("js_required_enforced").is_some());
        assert!(body.get("kv_store_fail_open").is_some());
        assert!(body.get("botness_weights").is_some());
        assert!(body.get("defence_modes").is_some());
        assert!(body.get("provider_backends").is_some());
        assert!(body.get("edge_integration_mode").is_some());
        assert!(body.get("defence_modes_effective").is_some());
        assert!(body.get("defence_mode_warnings").is_some());
        assert!(body.get("enterprise_multi_instance").is_some());
        assert!(body
            .get("enterprise_unsynced_state_exception_confirmed")
            .is_some());
        assert!(body.get("enterprise_state_guardrail_warnings").is_some());
        assert!(body.get("enterprise_state_guardrail_error").is_some());
        assert!(body.get("botness_signal_definitions").is_some());
        assert!(body.get("honeypot_enabled").is_some());
        assert!(body.get("adversary_sim_enabled").is_some());
        assert!(body.get("adversary_sim_duration_seconds").is_some());
        assert!(body.get("runtime_environment").is_some());
        assert!(body.get("adversary_sim_available").is_some());
    }

    #[test]
    fn admin_config_includes_runtime_environment_and_adversary_sim_state() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let req = make_request(Method::Get, "/admin/config", Vec::new());
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();

        assert_eq!(
            body.get("runtime_environment").and_then(|v| v.as_str()),
            Some("runtime-dev")
        );
        assert_eq!(
            body.get("adversary_sim_available").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("adversary_sim_enabled").and_then(|v| v.as_bool()),
            Some(false)
        );

        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn admin_config_includes_frontier_summary_without_exposing_keys() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_FRONTIER_OPENAI_API_KEY", "sk-openai-test");
        std::env::set_var("SHUMA_FRONTIER_OPENAI_MODEL", "gpt-5-mini");

        let req = make_request(Method::Get, "/admin/config", Vec::new());
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body_raw = String::from_utf8_lossy(resp.body()).to_string();
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();

        assert_eq!(
            body.get("frontier_mode").and_then(|value| value.as_str()),
            Some("single_provider_self_play")
        );
        assert_eq!(
            body.get("frontier_provider_count")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            body.get("frontier_diversity_confidence")
                .and_then(|value| value.as_str()),
            Some("low")
        );
        assert_eq!(
            body.get("frontier_reduced_diversity_warning")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        let providers = body
            .get("frontier_providers")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert_eq!(providers.len(), 4);
        assert!(providers.iter().any(|entry| {
            entry.get("provider").and_then(|value| value.as_str()) == Some("openai")
                && entry.get("configured").and_then(|value| value.as_bool()) == Some(true)
        }));
        assert!(!body_raw.contains("sk-openai-test"));

        std::env::remove_var("SHUMA_FRONTIER_OPENAI_API_KEY");
        std::env::remove_var("SHUMA_FRONTIER_OPENAI_MODEL");
    }

    #[test]
    fn admin_config_rejects_enabling_adversary_sim_in_runtime_prod() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");

        let body = br#"{"adversary_sim_enabled":true}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 400u16);
        assert!(String::from_utf8_lossy(resp.body()).contains("runtime-prod"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
    }

    #[test]
    fn admin_config_allows_enabling_adversary_sim_in_runtime_dev() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");

        let body = br#"{"adversary_sim_enabled":true}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let json: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(
            json.get("config")
                .and_then(|v| v.get("adversary_sim_enabled"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.adversary_sim_enabled);
        let effective_cfg = crate::config::load_runtime_cached(&store, "default").unwrap();
        assert!(effective_cfg.adversary_sim_enabled);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
    }

    #[test]
    fn admin_config_updates_adversary_sim_duration_seconds() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");

        let body = br#"{"adversary_sim_duration_seconds":240}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let json: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(
            json.get("config")
                .and_then(|v| v.get("adversary_sim_duration_seconds"))
                .and_then(|v| v.as_u64()),
            Some(240)
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(saved_cfg.adversary_sim_duration_seconds, 240);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_test_mode_toggle_updates_runtime_config() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");

        let body = br#"{"test_mode":true}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let json: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(
            json.get("config")
                .and_then(|v| v.get("test_mode"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(saved_cfg.test_mode);
        let effective_cfg = crate::config::load_runtime_cached(&store, "default").unwrap();
        assert!(effective_cfg.test_mode);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_invalid_adversary_sim_duration_seconds() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");

        let body = br#"{"adversary_sim_duration_seconds":901}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 400u16);
        assert!(String::from_utf8_lossy(resp.body()).contains("out of range"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn adversary_sim_control_start_stop_and_status_round_trip() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();

        let on_req = make_request(
            Method::Post,
            "/admin/adversary-sim/control",
            br#"{"enabled":true}"#.to_vec(),
        );
        let on_resp = handle_admin_adversary_sim_control(&on_req, &store, "default", &auth);
        assert_eq!(*on_resp.status(), 200u16);
        let on_json: serde_json::Value = serde_json::from_slice(on_resp.body()).unwrap();
        assert_eq!(
            on_json
                .get("status")
                .and_then(|v| v.get("phase"))
                .and_then(|v| v.as_str()),
            Some("running")
        );
        assert_eq!(
            on_json
                .get("status")
                .and_then(|v| v.get("active_run_count"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            on_json
                .get("status")
                .and_then(|v| v.get("active_lane_count"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        // Simulate a new runtime instance where in-memory ephemeral overrides are not retained.
        crate::config::clear_runtime_cache_for_tests();

        let status_req = make_request(Method::Get, "/admin/adversary-sim/status", Vec::new());
        let status_resp = handle_admin_adversary_sim_status(&status_req, &store, "default", &auth);
        assert_eq!(*status_resp.status(), 200u16);
        let status_json: serde_json::Value = serde_json::from_slice(status_resp.body()).unwrap();
        assert_eq!(
            status_json
                .get("adversary_sim_enabled")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            status_json.get("phase").and_then(|v| v.as_str()),
            Some("running")
        );
        assert_eq!(
            status_json
                .get("guardrails")
                .and_then(|v| v.get("max_duration_seconds"))
                .and_then(|v| v.as_u64()),
            Some(900)
        );

        let off_req = make_request(
            Method::Post,
            "/admin/adversary-sim/control",
            br#"{"enabled":false}"#.to_vec(),
        );
        let off_resp = handle_admin_adversary_sim_control(&off_req, &store, "default", &auth);
        assert_eq!(*off_resp.status(), 200u16);
        let off_json: serde_json::Value = serde_json::from_slice(off_resp.body()).unwrap();
        assert_eq!(
            off_json
                .get("status")
                .and_then(|v| v.get("phase"))
                .and_then(|v| v.as_str()),
            Some("off")
        );
        assert_eq!(
            off_json
                .get("status")
                .and_then(|v| v.get("adversary_sim_enabled"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            off_json
                .get("status")
                .and_then(|v| v.get("generation_active"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            off_json
                .get("status")
                .and_then(|v| v.get("historical_data_visible"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            off_json
                .get("status")
                .and_then(|v| v.get("history_retention"))
                .and_then(|v| v.get("cleanup_command"))
                .and_then(|v| v.as_str()),
            Some("make adversary-sim-history-clean")
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.adversary_sim_enabled);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_internal_beat_updates_generation_diagnostics_contract() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");
        std::env::set_var("SHUMA_API_KEY", "sim-internal-beat-test-key");

        let store = TestStore::default();
        let auth = bearer_rw_auth();

        let on_resp = handle_admin_adversary_sim_control(
            &make_control_request(true, "tick-start"),
            &store,
            "default",
            &auth,
        );
        assert_eq!(*on_resp.status(), 200u16);
        let on_json: serde_json::Value = serde_json::from_slice(on_resp.body()).unwrap();
        assert_eq!(
            on_json
                .get("status")
                .and_then(|v| v.get("generation_diagnostics"))
                .and_then(|v| v.get("generated_tick_count"))
                .and_then(|v| v.as_u64()),
            Some(0)
        );

        let beat_req = make_internal_beat_request("sim-internal-beat-test-key");
        let beat_resp = handle_internal_adversary_sim_beat(&beat_req, &store, "default");
        assert_eq!(*beat_resp.status(), 200u16);
        let beat_json: serde_json::Value = serde_json::from_slice(beat_resp.body()).unwrap();
        assert_eq!(beat_json.get("accepted").and_then(|v| v.as_bool()), Some(true));
        assert!(
            beat_json
                .get("executed_ticks")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                >= 1
        );
        assert!(
            beat_json
                .get("generated_requests")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0
        );
        assert_eq!(
            beat_json
                .get("status")
                .and_then(|v| v.get("generation_diagnostics"))
                .and_then(|v| v.get("health"))
                .and_then(|v| v.as_str()),
            Some("ok")
        );
        assert_eq!(
            beat_json
                .get("status")
                .and_then(|v| v.get("supervisor"))
                .and_then(|v| v.get("owner"))
                .and_then(|v| v.as_str()),
            Some("backend_autonomous_supervisor")
        );
        assert_eq!(
            beat_json
                .get("status")
                .and_then(|v| v.get("lifecycle_diagnostics"))
                .and_then(|v| v.get("control"))
                .and_then(|v| v.get("desired_enabled"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            beat_json
                .get("status")
                .and_then(|v| v.get("lifecycle_diagnostics"))
                .and_then(|v| v.get("supervisor"))
                .and_then(|v| v.get("heartbeat_expected"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert!(
            beat_json
                .get("status")
                .and_then(|v| v.get("lifecycle_diagnostics"))
                .and_then(|v| v.get("supervisor"))
                .and_then(|v| v.get("last_successful_beat_at"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
        std::env::remove_var("SHUMA_API_KEY");
    }

    #[test]
    fn adversary_sim_control_enable_recovers_from_stale_expired_running_state() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let now = now_ts();

        let stale_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Running,
            desired_enabled: false,
            owner_instance_id: Some("simproc-stale".to_string()),
            run_id: Some("simrun-stale".to_string()),
            started_at: Some(now.saturating_sub(600)),
            ends_at: Some(now.saturating_sub(300)),
            stop_deadline: None,
            active_run_count: 1,
            active_lane_count: 2,
            last_transition_reason: Some("manual_on".to_string()),
            last_terminal_failure_reason: None,
            last_run_id: Some("simrun-stale".to_string()),
            generated_tick_count: 0,
            generated_request_count: 0,
            last_generated_at: None,
            last_generation_error: None,
            updated_at: now.saturating_sub(300),
        };
        crate::admin::adversary_sim::save_state(&store, "default", &stale_state).unwrap();
        let mut stale_cfg = crate::config::defaults().clone();
        stale_cfg.adversary_sim_enabled = false;
        store
            .set("config:default", serde_json::to_vec(&stale_cfg).unwrap().as_slice())
            .unwrap();

        let on_resp = handle_admin_adversary_sim_control(
            &make_control_request(true, "recover-stale-running-enable"),
            &store,
            "default",
            &auth,
        );
        assert_eq!(*on_resp.status(), 200u16);
        let on_json: serde_json::Value = serde_json::from_slice(on_resp.body()).unwrap();
        assert_eq!(
            on_json
                .get("status")
                .and_then(|value| value.get("adversary_sim_enabled"))
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            on_json
                .get("status")
                .and_then(|value| value.get("phase"))
                .and_then(|value| value.as_str()),
            Some("running")
        );
        assert_eq!(
            on_json
                .get("status")
                .and_then(|value| value.get("active_run_count"))
                .and_then(|value| value.as_u64()),
            Some(1)
        );

        let persisted_cfg: crate::config::Config =
            serde_json::from_slice(&store.get("config:default").unwrap().unwrap()).unwrap();
        assert!(!persisted_cfg.adversary_sim_enabled);
        let effective_cfg = crate::config::load_runtime_cached(&store, "default").unwrap();
        assert!(effective_cfg.adversary_sim_enabled);
        let persisted_state = crate::admin::adversary_sim::load_state(&store, "default");
        assert_eq!(persisted_state.phase, crate::admin::adversary_sim::ControlPhase::Running);
        assert_eq!(persisted_state.active_run_count, 1);
        assert_eq!(persisted_state.active_lane_count, 2);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_status_reports_no_traffic_diagnostics_when_running_without_ticks() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();

        let on_resp = handle_admin_adversary_sim_control(
            &make_control_request(true, "diag-start"),
            &store,
            "default",
            &auth,
        );
        assert_eq!(*on_resp.status(), 200u16);

        let mut state = crate::admin::adversary_sim::load_state(&store, "default");
        let now = now_ts();
        state.started_at = Some(now.saturating_sub(10));
        state.generated_tick_count = 0;
        state.generated_request_count = 0;
        state.last_generated_at = None;
        state.last_generation_error = None;
        crate::admin::adversary_sim::save_state(&store, "default", &state).unwrap();

        let status_req = make_request(Method::Get, "/admin/adversary-sim/status", Vec::new());
        let status_resp = handle_admin_adversary_sim_status(&status_req, &store, "default", &auth);
        assert_eq!(*status_resp.status(), 200u16);
        let status_json: serde_json::Value = serde_json::from_slice(status_resp.body()).unwrap();
        assert_eq!(
            status_json
                .get("generation_diagnostics")
                .and_then(|v| v.get("health"))
                .and_then(|v| v.as_str()),
            Some("no_traffic")
        );
        assert_eq!(
            status_json
                .get("generation_diagnostics")
                .and_then(|v| v.get("reason"))
                .and_then(|v| v.as_str()),
            Some("supervisor_no_traffic_yet")
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_auto_off_preserves_historical_monitoring_visibility() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let now = now_ts();

        let on_resp = handle_admin_adversary_sim_control(
            &make_control_request(true, "history-preserve-start"),
            &store,
            "default",
            &auth,
        );
        assert_eq!(*on_resp.status(), 200u16);

        {
            let _guard = crate::runtime::sim_telemetry::enter(Some(
                crate::runtime::sim_telemetry::SimulationRequestMetadata {
                    sim_run_id: "run_history_001".to_string(),
                    sim_profile: "fast_smoke".to_string(),
                    sim_lane: "deterministic_black_box".to_string(),
                },
            ));
            crate::observability::monitoring::record_challenge_failure(
                &store,
                "198.51.100.77",
                "incorrect",
            );
            log_event(
                &store,
                &EventLogEntry {
                    ts: now,
                    event: EventType::AdminAction,
                    ip: Some("198.51.100.77".to_string()),
                    reason: Some("sim_history_visibility_probe".to_string()),
                    outcome: Some("ok".to_string()),
                    admin: Some("me".to_string()),
                },
            );
        }

        let off_resp = handle_admin_adversary_sim_control(
            &make_control_request(false, "history-preserve-stop"),
            &store,
            "default",
            &auth,
        );
        assert_eq!(*off_resp.status(), 200u16);
        let off_json: serde_json::Value = serde_json::from_slice(off_resp.body()).unwrap();
        assert_eq!(
            off_json
                .get("status")
                .and_then(|v| v.get("phase"))
                .and_then(|v| v.as_str()),
            Some("off")
        );
        assert_eq!(
            off_json
                .get("status")
                .and_then(|v| v.get("generation_active"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            off_json
                .get("status")
                .and_then(|v| v.get("historical_data_visible"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        let monitoring_req = make_request(Method::Get, "/admin/monitoring?hours=24&limit=10", Vec::new());
        let monitoring_resp = handle_admin_monitoring(&monitoring_req, &store);
        assert_eq!(*monitoring_resp.status(), 200u16);
        let monitoring_json: serde_json::Value =
            serde_json::from_slice(monitoring_resp.body()).unwrap();
        assert!(
            monitoring_json
                .get("summary")
                .and_then(|v| v.get("challenge"))
                .and_then(|v| v.get("total_failures"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                >= 1
        );
        assert!(
            monitoring_json
                .get("details")
                .and_then(|v| v.get("events"))
                .and_then(|v| v.get("recent_events"))
                .and_then(|v| v.as_array())
                .map(|events| {
                    events.iter().any(|event| {
                        event
                            .get("is_simulation")
                            .and_then(|value| value.as_bool())
                            == Some(true)
                    })
                })
                .unwrap_or(false)
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_history_cleanup_endpoint_clears_retained_dev_telemetry() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let now = now_ts();

        {
            let _guard = crate::runtime::sim_telemetry::enter(Some(
                crate::runtime::sim_telemetry::SimulationRequestMetadata {
                    sim_run_id: "run_cleanup_001".to_string(),
                    sim_profile: "fast_smoke".to_string(),
                    sim_lane: "deterministic_black_box".to_string(),
                },
            ));
            crate::observability::monitoring::record_challenge_failure(
                &store,
                "198.51.100.88",
                "incorrect",
            );
            log_event(
                &store,
                &EventLogEntry {
                    ts: now,
                    event: EventType::AdminAction,
                    ip: Some("198.51.100.88".to_string()),
                    reason: Some("sim_history_cleanup_probe".to_string()),
                    outcome: Some("ok".to_string()),
                    admin: Some("me".to_string()),
                },
            );
        }

        let cleanup_req = make_request(
            Method::Post,
            "/admin/adversary-sim/history/cleanup",
            Vec::new(),
        );
        let cleanup_resp =
            handle_admin_adversary_sim_history_cleanup(&cleanup_req, &store, "default", &auth);
        assert_eq!(*cleanup_resp.status(), 200u16);
        let cleanup_json: serde_json::Value = serde_json::from_slice(cleanup_resp.body()).unwrap();
        assert_eq!(
            cleanup_json
                .get("cleaned")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert!(
            cleanup_json
                .get("deleted_keys")
                .and_then(|value| value.as_u64())
                .unwrap_or(0)
                >= 1
        );

        let monitoring_req = make_request(Method::Get, "/admin/monitoring?hours=24&limit=10", Vec::new());
        let monitoring_resp = handle_admin_monitoring(&monitoring_req, &store);
        assert_eq!(*monitoring_resp.status(), 200u16);
        let monitoring_json: serde_json::Value =
            serde_json::from_slice(monitoring_resp.body()).unwrap();
        assert_eq!(
            monitoring_json
                .get("summary")
                .and_then(|v| v.get("challenge"))
                .and_then(|v| v.get("total_failures"))
                .and_then(|v| v.as_u64()),
            Some(0)
        );
        let recent_events = monitoring_json
            .get("details")
            .and_then(|v| v.get("events"))
            .and_then(|v| v.get("recent_events"))
            .and_then(|v| v.as_array())
            .expect("recent_events");
        assert_eq!(recent_events.len(), 1);
        assert_eq!(
            recent_events[0]
                .get("reason")
                .and_then(|value| value.as_str()),
            Some("adversary_sim_history_cleanup")
        );
        assert_eq!(
            recent_events[0]
                .get("is_simulation")
                .and_then(|value| value.as_bool()),
            Some(false)
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_history_cleanup_fails_closed_outside_dev_surface() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "false");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let cleanup_req = make_request(
            Method::Post,
            "/admin/adversary-sim/history/cleanup",
            Vec::new(),
        );
        let cleanup_resp =
            handle_admin_adversary_sim_history_cleanup(&cleanup_req, &store, "default", &auth);
        assert_eq!(*cleanup_resp.status(), 404u16);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_status_reconciles_idle_enabled_state_to_off() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();

        let enable_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"adversary_sim_enabled":true}"#.to_vec(),
        );
        let enable_resp = handle_admin_config(&enable_req, &store, "default");
        assert_eq!(*enable_resp.status(), 200u16);

        let status_req = make_request(Method::Get, "/admin/adversary-sim/status", Vec::new());
        let status_resp = handle_admin_adversary_sim_status(&status_req, &store, "default", &auth);
        assert_eq!(*status_resp.status(), 200u16);
        let status_json: serde_json::Value = serde_json::from_slice(status_resp.body()).unwrap();
        assert_eq!(
            status_json
                .get("adversary_sim_enabled")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            status_json.get("phase").and_then(|value| value.as_str()),
            Some("off")
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.adversary_sim_enabled);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_status_reconciles_stale_running_state_when_disabled() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let now = now_ts();

        let stale_running_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Running,
            desired_enabled: false,
            owner_instance_id: Some("simproc-stale".to_string()),
            run_id: Some("simrun-stale-running".to_string()),
            started_at: Some(now.saturating_sub(30)),
            ends_at: Some(now.saturating_add(120)),
            stop_deadline: None,
            active_run_count: 1,
            active_lane_count: 2,
            last_transition_reason: Some("manual_on".to_string()),
            last_terminal_failure_reason: None,
            last_run_id: Some("simrun-stale-running".to_string()),
            generated_tick_count: 0,
            generated_request_count: 0,
            last_generated_at: None,
            last_generation_error: None,
            updated_at: now.saturating_sub(10),
        };
        crate::admin::adversary_sim::save_state(&store, "default", &stale_running_state).unwrap();

        let mut cfg = crate::config::defaults().clone();
        cfg.adversary_sim_enabled = false;
        store
            .set("config:default", serde_json::to_vec(&cfg).unwrap().as_slice())
            .unwrap();

        let status_req = make_request(Method::Get, "/admin/adversary-sim/status", Vec::new());
        let status_resp = handle_admin_adversary_sim_status(&status_req, &store, "default", &auth);
        assert_eq!(*status_resp.status(), 200u16);
        let status_json: serde_json::Value = serde_json::from_slice(status_resp.body()).unwrap();
        assert_eq!(
            status_json
                .get("adversary_sim_enabled")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            status_json.get("phase").and_then(|value| value.as_str()),
            Some("off")
        );
        assert_eq!(
            status_json
                .get("generation_active")
                .and_then(|value| value.as_bool()),
            Some(false)
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_status_forces_off_when_run_owned_by_previous_process_instance() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let now = now_ts();

        let stale_running_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Running,
            desired_enabled: true,
            owner_instance_id: Some("simproc-previous".to_string()),
            run_id: Some("simrun-prev-process".to_string()),
            started_at: Some(now.saturating_sub(5)),
            ends_at: Some(now.saturating_add(120)),
            active_run_count: 1,
            active_lane_count: 2,
            last_transition_reason: Some("manual_on".to_string()),
            updated_at: now.saturating_sub(5),
            ..crate::admin::adversary_sim::ControlState::default()
        };
        crate::admin::adversary_sim::save_state(&store, "default", &stale_running_state).unwrap();

        let status_req = make_request(Method::Get, "/admin/adversary-sim/status", Vec::new());
        let status_resp = handle_admin_adversary_sim_status(&status_req, &store, "default", &auth);
        assert_eq!(*status_resp.status(), 200u16);
        let status_json: serde_json::Value = serde_json::from_slice(status_resp.body()).unwrap();
        assert_eq!(
            status_json.get("phase").and_then(|value| value.as_str()),
            Some("off")
        );
        assert_eq!(
            status_json
                .get("adversary_sim_enabled")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            status_json
                .get("last_transition_reason")
                .and_then(|value| value.as_str()),
            Some("process_restart")
        );

        let persisted = crate::admin::adversary_sim::load_state(&store, "default");
        assert_eq!(persisted.phase, crate::admin::adversary_sim::ControlPhase::Off);
        assert!(!persisted.desired_enabled);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_status_auto_window_expiry_clears_runtime_enabled_override() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let now = now_ts();

        let on_resp = handle_admin_adversary_sim_control(
            &make_control_request(true, "auto-expiry-enable"),
            &store,
            "default",
            &auth,
        );
        assert_eq!(*on_resp.status(), 200u16);

        let mut state = crate::admin::adversary_sim::load_state(&store, "default");
        state.phase = crate::admin::adversary_sim::ControlPhase::Running;
        state.ends_at = Some(now.saturating_sub(1));
        state.active_run_count = 1;
        state.active_lane_count = 2;
        crate::admin::adversary_sim::save_state(&store, "default", &state).unwrap();

        let status_req = make_request(Method::Get, "/admin/adversary-sim/status", Vec::new());
        let status_resp = handle_admin_adversary_sim_status(&status_req, &store, "default", &auth);
        assert_eq!(*status_resp.status(), 200u16);
        let status_json: serde_json::Value = serde_json::from_slice(status_resp.body()).unwrap();
        assert_eq!(
            status_json.get("phase").and_then(|value| value.as_str()),
            Some("off")
        );
        assert_eq!(
            status_json
                .get("adversary_sim_enabled")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        let effective_cfg = crate::config::load_runtime_cached(&store, "default").unwrap();
        assert!(!effective_cfg.adversary_sim_enabled);
        let persisted_cfg = crate::config::Config::load(&store, "default").unwrap();
        assert!(!persisted_cfg.adversary_sim_enabled);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_exact_replay_returns_stable_operation_id() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();

        let req = make_control_request(true, "replay-key-1");
        let first = handle_admin_adversary_sim_control(&req, &store, "default", &auth);
        assert_eq!(*first.status(), 200u16);
        let first_json: serde_json::Value = serde_json::from_slice(first.body()).unwrap();
        let first_operation_id = first_json
            .get("operation_id")
            .and_then(|value| value.as_str())
            .expect("operation id");
        assert_eq!(
            first_json.get("decision").and_then(|value| value.as_str()),
            Some("accepted")
        );

        let replay = handle_admin_adversary_sim_control(&req, &store, "default", &auth);
        assert_eq!(*replay.status(), 200u16);
        let replay_json: serde_json::Value = serde_json::from_slice(replay.body()).unwrap();
        assert_eq!(
            replay_json
                .get("operation_id")
                .and_then(|value| value.as_str()),
            Some(first_operation_id)
        );
        assert_eq!(
            replay_json.get("decision").and_then(|value| value.as_str()),
            Some("replayed")
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_rejects_idempotency_payload_mismatch() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();

        let on_req = make_control_request(true, "mismatch-key-1");
        let on_resp = handle_admin_adversary_sim_control(&on_req, &store, "default", &auth);
        assert_eq!(*on_resp.status(), 200u16);

        let off_req = make_control_request(false, "mismatch-key-1");
        let off_resp = handle_admin_adversary_sim_control(&off_req, &store, "default", &auth);
        assert_eq!(*off_resp.status(), 409u16);
        assert!(String::from_utf8_lossy(off_resp.body()).contains("payload mismatch"));

        let status_req = make_request(Method::Get, "/admin/adversary-sim/status", Vec::new());
        let status_resp = handle_admin_adversary_sim_status(&status_req, &store, "default", &auth);
        assert_eq!(*status_resp.status(), 200u16);
        let status_json: serde_json::Value = serde_json::from_slice(status_resp.body()).unwrap();
        assert_eq!(
            status_json.get("phase").and_then(|value| value.as_str()),
            Some("running")
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_rejects_missing_origin_header() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/admin/adversary-sim/control")
            .header("host", "localhost:3000")
            .header("authorization", "Bearer changeme-dev-only-api-key")
            .header("sec-fetch-site", "same-origin")
            .header("idempotency-key", "missing-origin-key")
            .body(br#"{"enabled":true}"#.to_vec());
        let req = builder.build();

        let resp = handle_admin_adversary_sim_control(&req, &store, "default", &auth);
        assert_eq!(*resp.status(), 403u16);
        assert!(String::from_utf8_lossy(resp.body()).contains("trust boundary"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_rejects_origin_mismatch() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let req = make_control_request_with_trust_headers(
            true,
            "origin-mismatch-key",
            Some("https://malicious.example"),
            Some("same-origin"),
            None,
        );

        let resp = handle_admin_adversary_sim_control(&req, &store, "default", &auth);
        assert_eq!(*resp.status(), 403u16);
        assert!(String::from_utf8_lossy(resp.body()).contains("trust boundary"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_rejects_cross_site_fetch_metadata() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let req = make_control_request_with_trust_headers(
            true,
            "cross-site-key",
            Some("http://localhost:3000"),
            Some("cross-site"),
            None,
        );

        let resp = handle_admin_adversary_sim_control(&req, &store, "default", &auth);
        assert_eq!(*resp.status(), 403u16);
        assert!(String::from_utf8_lossy(resp.body()).contains("trust boundary"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_allows_authenticated_session_independent_of_age() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let now = now_ts();
        let auth = session_rw_auth("session-stale", "csrf-stale", now.saturating_add(300));
        let req = make_control_request_with_trust_headers(
            true,
            "stale-session-key",
            Some("http://localhost:3000"),
            Some("same-origin"),
            Some("csrf-stale"),
        );

        let resp = handle_admin_adversary_sim_control(&req, &store, "default", &auth);
        assert_eq!(*resp.status(), 200u16);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(
            payload.get("decision").and_then(|value| value.as_str()),
            Some("accepted")
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_allows_session_csrf_when_origin_header_is_missing() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let now = now_ts();
        let auth = session_rw_auth(
            "session-origin-missing",
            "csrf-origin-missing",
            now.saturating_add(crate::admin::auth::admin_session_ttl_seconds()),
        );
        let req = make_control_request_with_trust_headers(
            true,
            "session-origin-missing-key",
            None,
            Some("same-origin"),
            Some("csrf-origin-missing"),
        );

        let resp = handle_admin_adversary_sim_control(&req, &store, "default", &auth);
        assert_eq!(*resp.status(), 200u16);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(
            payload.get("decision").and_then(|value| value.as_str()),
            Some("accepted")
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_rejects_missing_or_invalid_session_csrf() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let now = now_ts();
        let auth = session_rw_auth(
            "session-csrf",
            "csrf-expected",
            now.saturating_add(crate::admin::auth::admin_session_ttl_seconds()),
        );
        let missing_csrf = make_control_request_with_trust_headers(
            true,
            "missing-csrf-key",
            Some("http://localhost:3000"),
            Some("same-origin"),
            None,
        );
        let missing_resp = handle_admin_adversary_sim_control(&missing_csrf, &store, "default", &auth);
        assert_eq!(*missing_resp.status(), 403u16);
        assert!(String::from_utf8_lossy(missing_resp.body()).contains("trust boundary"));

        let invalid_csrf = make_control_request_with_trust_headers(
            true,
            "invalid-csrf-key",
            Some("http://localhost:3000"),
            Some("same-origin"),
            Some("csrf-wrong"),
        );
        let invalid_resp = handle_admin_adversary_sim_control(&invalid_csrf, &store, "default", &auth);
        assert_eq!(*invalid_resp.status(), 403u16);
        assert!(String::from_utf8_lossy(invalid_resp.body()).contains("trust boundary"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_rejects_multi_controller_lease_contention() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let now = now_ts();
        let session_expires = now.saturating_add(crate::admin::auth::admin_session_ttl_seconds());
        let auth_a = session_rw_auth("session-owner-a", "csrf-a", session_expires);
        let auth_b = session_rw_auth("session-owner-b", "csrf-b", session_expires);

        let first = handle_admin_adversary_sim_control(
            &make_control_request_with_trust_headers(
                true,
                "lease-key-1",
                Some("http://localhost:3000"),
                Some("same-origin"),
                Some("csrf-a"),
            ),
            &store,
            "default",
            &auth_a,
        );
        assert_eq!(*first.status(), 200u16);

        let second = handle_admin_adversary_sim_control(
            &make_control_request_with_trust_headers(
                false,
                "lease-key-2",
                Some("http://localhost:3000"),
                Some("same-origin"),
                Some("csrf-b"),
            ),
            &store,
            "default",
            &auth_b,
        );
        assert_eq!(*second.status(), 409u16);
        assert!(String::from_utf8_lossy(second.body()).contains("lease is currently held"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_throttles_rapid_same_state_repeats() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();

        let first = handle_admin_adversary_sim_control(
            &make_control_request(true, "throttle-key-1"),
            &store,
            "default",
            &auth,
        );
        assert_eq!(*first.status(), 200u16);

        let rapid_repeat = handle_admin_adversary_sim_control(
            &make_control_request(true, "throttle-key-2"),
            &store,
            "default",
            &auth,
        );
        assert_eq!(*rapid_repeat.status(), 429u16);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_control_emits_audit_for_accept_reject_and_throttle() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let now = now_ts();
        let session_expires = now.saturating_add(crate::admin::auth::admin_session_ttl_seconds());
        let auth_session = session_rw_auth("session-audit", "csrf-audit", session_expires);
        let auth_bearer = bearer_rw_auth();

        let accepted = handle_admin_adversary_sim_control(
            &make_control_request_with_trust_headers(
                true,
                "audit-accept-key",
                Some("http://localhost:3000"),
                Some("same-origin"),
                Some("csrf-audit"),
            ),
            &store,
            "default",
            &auth_session,
        );
        assert_eq!(*accepted.status(), 200u16);

        let throttled = handle_admin_adversary_sim_control(
            &make_control_request_with_trust_headers(
                true,
                "audit-throttle-key",
                Some("http://localhost:3000"),
                Some("same-origin"),
                Some("csrf-audit"),
            ),
            &store,
            "default",
            &auth_session,
        );
        assert_eq!(*throttled.status(), 429u16);

        let rejected = handle_admin_adversary_sim_control(
            &make_control_request_with_trust_headers(
                true,
                "audit-reject-key",
                Some("https://evil.invalid"),
                Some("same-origin"),
                None,
            ),
            &store,
            "default",
            &auth_bearer,
        );
        assert_eq!(*rejected.status(), 403u16);

        let decisions = collect_control_audit_decisions(&store);
        assert!(decisions.contains(&"accepted".to_string()));
        assert!(decisions.contains(&"throttled".to_string()));
        assert!(decisions.contains(&"rejected".to_string()));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn adversary_sim_status_read_path_reconciles_stale_state() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");

        let store = TestStore::default();
        let auth = bearer_rw_auth();
        let now = now_ts();

        let mut cfg = crate::config::defaults().clone();
        cfg.adversary_sim_enabled = true;
        store
            .set("config:default", &serde_json::to_vec(&cfg).unwrap())
            .unwrap();

        let stale_running_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Running,
            desired_enabled: false,
            owner_instance_id: Some("simproc-stale".to_string()),
            run_id: Some("run-stale".to_string()),
            started_at: Some(now.saturating_sub(180)),
            ends_at: Some(now.saturating_sub(1)),
            stop_deadline: None,
            active_run_count: 1,
            active_lane_count: 2,
            last_transition_reason: Some("manual_on".to_string()),
            updated_at: now.saturating_sub(180),
            ..crate::admin::adversary_sim::ControlState::default()
        };
        crate::admin::adversary_sim::save_state(&store, "default", &stale_running_state).unwrap();

        let status_req = make_request(Method::Get, "/admin/adversary-sim/status", Vec::new());
        let status_resp = handle_admin_adversary_sim_status(&status_req, &store, "default", &auth);
        assert_eq!(*status_resp.status(), 200u16);
        let status_json: serde_json::Value = serde_json::from_slice(status_resp.body()).unwrap();
        assert_eq!(
            status_json
                .get("controller_reconciliation_required")
                .and_then(|value| value.as_bool()),
            Some(false)
        );

        let persisted = crate::admin::adversary_sim::load_state(&store, "default");
        assert_eq!(persisted.phase, crate::admin::adversary_sim::ControlPhase::Off);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    }

    #[test]
    fn admin_maze_seed_sources_round_trip_and_manual_refresh() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.maze_seed_provider = crate::config::MazeSeedProvider::Operator;
        cfg.maze_seed_refresh_rate_limit_per_hour = 3;
        cfg.maze_seed_refresh_max_sources = 4;
        store
            .set("config:default", &serde_json::to_vec(&cfg).unwrap())
            .unwrap();

        let post_req = make_request(
            Method::Post,
            "/admin/maze/seeds",
            br#"{
                "sources":[
                    {
                        "id":"headlines",
                        "url":"https://example.com/feed",
                        "title":"Signal routing update",
                        "description":"Metadata-only refresh for maze corpus",
                        "keywords":["maze","checkpoint","budget"],
                        "allow_seed_use":true,
                        "robots_allowed":true
                    }
                ]
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_maze_seed_sources(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);

        let get_req = make_request(Method::Get, "/admin/maze/seeds", Vec::new());
        let get_resp = handle_admin_maze_seed_sources(&get_req, &store, "default");
        assert_eq!(*get_resp.status(), 200u16);
        let get_json: serde_json::Value = serde_json::from_slice(get_resp.body()).unwrap();
        assert_eq!(
            get_json
                .get("sources")
                .and_then(|v| v.as_array())
                .map(|v| v.len()),
            Some(1)
        );

        let refresh_req = make_request(Method::Post, "/admin/maze/seeds/refresh", Vec::new());
        let refresh_resp = handle_admin_maze_seed_refresh(&refresh_req, &store, "default");
        assert_eq!(*refresh_resp.status(), 200u16);
        let refresh_json: serde_json::Value = serde_json::from_slice(refresh_resp.body()).unwrap();
        assert_eq!(
            refresh_json.get("refreshed"),
            Some(&serde_json::Value::Bool(true))
        );
        assert!(
            refresh_json
                .get("term_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0
        );
    }

    #[test]
    fn admin_maze_seed_refresh_requires_operator_provider() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        let refresh_req = make_request(Method::Post, "/admin/maze/seeds/refresh", Vec::new());
        let refresh_resp = handle_admin_maze_seed_refresh(&refresh_req, &store, "default");
        assert_eq!(*refresh_resp.status(), 409u16);
    }

    #[test]
    fn admin_maze_preview_returns_safe_non_operational_html() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        let preview_path = crate::maze::entry_path("preview-segment");
        let req = make_request(
            Method::Get,
            format!("/admin/maze/preview?path={}", preview_path).as_str(),
            Vec::new(),
        );
        let resp = handle_admin_maze_preview(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body = String::from_utf8_lossy(resp.body());
        assert!(!body.contains("Maze Preview"));
        assert!(!body.contains("Preview-only path."));
        assert!(!body.contains("mt="));
        assert!(!body.contains("data-shuma-covert-decoy"));
        assert!(body.contains("/admin/maze/preview?path="));
    }

    #[test]
    fn admin_maze_preview_does_not_mutate_live_maze_state() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();

        {
            let mut map = store.map.lock().unwrap();
            map.insert("maze:budget:active:global".to_string(), b"9".to_vec());
            map.insert("maze:risk:ip".to_string(), b"4".to_vec());
            map.insert("maze:token:seen:flow:op".to_string(), b"123456789".to_vec());
        }
        let before = {
            let map = store.map.lock().unwrap();
            map.iter()
                .filter(|(k, _)| k.starts_with("maze:"))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<std::collections::HashMap<_, _>>()
        };

        let req = make_request(Method::Get, "/admin/maze/preview", Vec::new());
        let resp = handle_admin_maze_preview(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);

        let after = {
            let map = store.map.lock().unwrap();
            map.iter()
                .filter(|(k, _)| k.starts_with("maze:"))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<std::collections::HashMap<_, _>>()
        };
        assert_eq!(before, after);
    }

    #[test]
    fn admin_maze_preview_is_get_only_read_path() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        let req = make_request(Method::Post, "/admin/maze/preview", Vec::new());
        let resp = handle_admin_maze_preview(&req, &store, "default");
        assert_eq!(*resp.status(), 405u16);
        assert!(!request_requires_admin_write(
            "/admin/maze/preview",
            &Method::Get
        ));
        assert!(sanitize_path("/admin/maze/preview"));
        assert!(sanitize_path("/admin/tarpit/preview"));
        assert!(sanitize_path("/admin/ip-range/suggestions"));
        assert!(sanitize_path("/admin/monitoring/stream"));
        assert!(sanitize_path("/admin/ip-bans/stream"));
        assert!(sanitize_path("/admin/adversary-sim/history/cleanup"));
    }

    #[test]
    fn admin_tarpit_preview_serves_progressive_bootstrap() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();

        let req = make_request(Method::Get, "/admin/tarpit/preview", Vec::new());
        let resp = handle_admin_tarpit_preview(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body = String::from_utf8_lossy(resp.body());
        assert!(body.contains("window.__shumaTarpit"));
        assert!(body.contains("/tarpit/progress"));
    }

    #[test]
    fn admin_tarpit_preview_is_non_mutating_get_only_path() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        {
            let mut map = store.map.lock().unwrap();
            map.insert("tarpit:budget:active:global:default".to_string(), b"7".to_vec());
            map.insert(
                "tarpit:budget:active:bucket:default:bucket-a".to_string(),
                b"2".to_vec(),
            );
            map.insert(
                "tarpit:persistence:default:bucket-a".to_string(),
                br#"{"count":4,"expires_at":9999999999}"#.to_vec(),
            );
        }

        let before = {
            let map = store.map.lock().unwrap();
            map.iter()
                .filter(|(k, _)| k.starts_with("tarpit:"))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<std::collections::HashMap<_, _>>()
        };

        let req = make_request(
            Method::Get,
            "/admin/tarpit/preview",
            Vec::new(),
        );
        let resp = handle_admin_tarpit_preview(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);

        let after = {
            let map = store.map.lock().unwrap();
            map.iter()
                .filter(|(k, _)| k.starts_with("tarpit:"))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<std::collections::HashMap<_, _>>()
        };
        assert_eq!(before, after);

        let post_req = make_request(Method::Post, "/admin/tarpit/preview", Vec::new());
        let post_resp = handle_admin_tarpit_preview(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 405u16);
        assert!(!request_requires_admin_write(
            "/admin/tarpit/preview",
            &Method::Get
        ));
    }

    #[test]
    fn admin_monitoring_returns_structured_summary_shape() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        crate::observability::monitoring::record_honeypot_hit(&store, "10.0.0.8", "/instaban");
        crate::observability::monitoring::record_challenge_failure(
            &store,
            "198.51.100.7",
            "incorrect",
        );
        crate::observability::monitoring::record_pow_failure(
            &store,
            "198.51.100.9",
            "invalid_proof",
        );
        crate::observability::monitoring::record_rate_violation_with_path(
            &store,
            "203.0.113.11",
            Some("/"),
            "limited",
        );
        crate::observability::monitoring::record_geo_violation(&store, Some("US"), "challenge");
        crate::observability::monitoring::record_not_a_bot_served(&store);
        crate::observability::monitoring::record_not_a_bot_submit(&store, "pass", Some(1400));

        let req = make_request(Method::Get, "/admin/monitoring?hours=24&limit=5", Vec::new());
        let resp = handle_admin_monitoring(&req, &store);
        assert_eq!(*resp.status(), 200u16);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let summary = body.get("summary").unwrap();
        let details = body.get("details").unwrap();

        assert!(summary.get("honeypot").is_some());
        assert!(summary.get("challenge").is_some());
        assert!(summary.get("not_a_bot").is_some());
        assert!(summary.get("pow").is_some());
        assert!(summary.get("rate").is_some());
        assert!(summary.get("geo").is_some());
        assert!(details.get("analytics").is_some());
        assert!(details.get("events").is_some());
        assert!(details.get("bans").is_some());
        assert!(details.get("maze").is_some());
        assert!(details.get("tarpit").is_some());
        assert!(details.get("cdp").is_some());
        assert!(details.get("cdp_events").is_some());
        assert!(body.get("freshness_slo").is_some());
        assert!(body.get("load_envelope").is_some());
        assert!(body.get("freshness").is_some());
        assert_eq!(
            body.get("prometheus")
                .and_then(|v| v.get("endpoint"))
                .and_then(|v| v.as_str()),
            Some("/metrics")
        );
        assert!(
            body.get("prometheus")
                .and_then(|v| v.get("notes"))
                .and_then(|v| v.as_array())
                .map(|notes| !notes.is_empty())
                .unwrap_or(false)
        );
        assert!(
            body.get("prometheus")
                .and_then(|v| v.get("example_js"))
                .and_then(|v| v.as_str())
                .map(|value| value.contains("fetch('/metrics')"))
                .unwrap_or(false)
        );
        assert!(
            body.get("prometheus")
                .and_then(|v| v.get("example_summary_stats"))
                .and_then(|v| v.as_str())
                .map(|value| value.contains("monitoring.summary"))
                .unwrap_or(false)
        );
        assert!(
            details
                .get("events")
                .and_then(|v| v.get("recent_events"))
                .map(|v| v.is_array())
                .unwrap_or(false)
        );
        assert!(
            summary
                .get("challenge")
                .and_then(|v| v.get("reasons"))
                .and_then(|v| v.get("incorrect"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                >= 1
        );
        assert!(
            summary
                .get("not_a_bot")
                .and_then(|v| v.get("pass"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                >= 1
        );
        assert!(
            summary
                .get("not_a_bot")
                .and_then(|v| v.get("solve_latency_buckets"))
                .and_then(|v| v.get("1_3s"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                >= 1
        );
        assert!(
            summary
                .get("pow")
                .and_then(|v| v.get("total_successes"))
                .and_then(|v| v.as_u64())
                .is_some()
        );
        assert!(
            summary
                .get("pow")
                .and_then(|v| v.get("success_ratio"))
                .and_then(|v| v.as_f64())
                .is_some()
        );
        assert!(
            summary
                .get("pow")
                .and_then(|v| v.get("outcomes"))
                .and_then(|v| v.get("failure"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                >= 1
        );
        assert!(
            summary
                .get("rate")
                .and_then(|v| v.get("top_paths"))
                .map(|v| v.is_array())
                .unwrap_or(false)
        );
    }

    #[test]
    fn admin_monitoring_includes_simulation_and_baseline_events() {
        let store = TestStore::default();
        let now = now_ts();

        crate::observability::monitoring::record_challenge_failure(
            &store,
            "198.51.100.7",
            "incorrect",
        );
        log_event(
            &store,
            &EventLogEntry {
                ts: now,
                event: EventType::AdminAction,
                ip: Some("198.51.100.7".to_string()),
                reason: Some("baseline_event".to_string()),
                outcome: Some("ok".to_string()),
                admin: Some("me".to_string()),
            },
        );

        {
            let _guard = crate::runtime::sim_telemetry::enter(Some(
                crate::runtime::sim_telemetry::SimulationRequestMetadata {
                    sim_run_id: "run_abc".to_string(),
                    sim_profile: "fast_smoke".to_string(),
                    sim_lane: "deterministic_black_box".to_string(),
                },
            ));
            crate::observability::monitoring::record_challenge_failure(
                &store,
                "198.51.100.8",
                "incorrect",
            );
            log_event(
                &store,
                &EventLogEntry {
                    ts: now,
                    event: EventType::AdminAction,
                    ip: Some("198.51.100.8".to_string()),
                    reason: Some("sim_event".to_string()),
                    outcome: Some("ok".to_string()),
                    admin: Some("me".to_string()),
                },
            );
        }

        let req_default = make_request(Method::Get, "/admin/monitoring?hours=24&limit=5", Vec::new());
        let resp_default = handle_admin_monitoring(&req_default, &store);
        assert_eq!(*resp_default.status(), 200u16);
        let body_default: serde_json::Value = serde_json::from_slice(resp_default.body()).unwrap();
        assert_eq!(
            body_default
                .get("summary")
                .and_then(|summary| summary.get("challenge"))
                .and_then(|challenge| challenge.get("total_failures"))
                .and_then(|value| value.as_u64()),
            Some(2)
        );
        let include_events = body_default
            .get("details")
            .and_then(|details| details.get("events"))
            .and_then(|events| events.get("recent_events"))
            .and_then(|events| events.as_array())
            .expect("recent_events");
        assert_eq!(include_events.len(), 2);
        assert!(include_events
            .iter()
            .any(|entry| entry.get("is_simulation").and_then(|value| value.as_bool()) == Some(true)));
    }

    #[test]
    fn admin_monitoring_keeps_simulation_event_parity_for_equivalent_outcomes() {
        let store = TestStore::default();
        let now = now_ts();

        crate::observability::monitoring::record_challenge_failure(
            &store,
            "198.51.100.31",
            "incorrect",
        );
        log_event(
            &store,
            &EventLogEntry {
                ts: now,
                event: EventType::Challenge,
                ip: Some("198.51.100.31".to_string()),
                reason: Some("challenge_submit".to_string()),
                outcome: Some("challenge_failed".to_string()),
                admin: Some("ops".to_string()),
            },
        );

        {
            let _guard = crate::runtime::sim_telemetry::enter(Some(
                crate::runtime::sim_telemetry::SimulationRequestMetadata {
                    sim_run_id: "run_parity".to_string(),
                    sim_profile: "runtime_toggle".to_string(),
                    sim_lane: "deterministic_black_box".to_string(),
                },
            ));
            crate::observability::monitoring::record_challenge_failure(
                &store,
                "198.51.100.31",
                "incorrect",
            );
            log_event(
                &store,
                &EventLogEntry {
                    ts: now.saturating_add(1),
                    event: EventType::Challenge,
                    ip: Some("198.51.100.31".to_string()),
                    reason: Some("challenge_submit".to_string()),
                    outcome: Some("challenge_failed".to_string()),
                    admin: Some("ops".to_string()),
                },
            );
        }

        let req = make_request(Method::Get, "/admin/monitoring?hours=24&limit=10", Vec::new());
        let resp = handle_admin_monitoring(&req, &store);
        assert_eq!(*resp.status(), 200u16);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(
            payload
                .get("summary")
                .and_then(|summary| summary.get("challenge"))
                .and_then(|challenge| challenge.get("total_failures"))
                .and_then(|value| value.as_u64()),
            Some(2)
        );

        let events = payload
            .get("details")
            .and_then(|details| details.get("events"))
            .and_then(|events| events.get("recent_events"))
            .and_then(|events| events.as_array())
            .cloned()
            .unwrap_or_default();
        let equivalent = events
            .iter()
            .filter(|entry| {
                entry.get("event").and_then(|value| value.as_str()) == Some("Challenge")
                    && entry.get("reason").and_then(|value| value.as_str())
                        == Some("challenge_submit")
                    && entry.get("outcome").and_then(|value| value.as_str())
                        == Some("challenge_failed")
            })
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(equivalent.len(), 2);
        assert_eq!(
            equivalent
                .iter()
                .filter(|entry| entry.get("is_simulation").and_then(|value| value.as_bool()) == Some(true))
                .count(),
            1
        );
        assert_eq!(
            equivalent
                .iter()
                .filter(|entry| entry.get("is_simulation").and_then(|value| value.as_bool()) == Some(false))
                .count(),
            1
        );
    }

    #[test]
    fn admin_monitoring_defaults_to_pseudonymized_view_and_supports_forensic_mode() {
        let store = TestStore::default();
        let now = now_ts();
        let raw_ip = "203.0.113.55";
        let pseudo_ip = pseudonymize_ip_identifier(raw_ip);
        log_event(
            &store,
            &EventLogEntry {
                ts: now,
                event: EventType::AdminAction,
                ip: Some(raw_ip.to_string()),
                reason: Some("security_mode_probe".to_string()),
                outcome: Some("ok".to_string()),
                admin: Some("operator@example.com".to_string()),
            },
        );

        let req_default = make_request(Method::Get, "/admin/monitoring?hours=24&limit=5", Vec::new());
        let resp_default = handle_admin_monitoring(&req_default, &store);
        assert_eq!(*resp_default.status(), 200u16);
        assert_eq!(
            resp_default
                .header("x-shuma-monitoring-security-mode")
                .and_then(|value| value.as_str()),
            Some("pseudonymized_default")
        );
        let body_default: serde_json::Value = serde_json::from_slice(resp_default.body()).unwrap();
        let event_default = body_default
            .get("details")
            .and_then(|value| value.get("events"))
            .and_then(|value| value.get("recent_events"))
            .and_then(|value| value.as_array())
            .and_then(|rows| {
                rows.iter().find(|row| {
                    row.get("reason").and_then(|value| value.as_str())
                        == Some("security_mode_probe")
                })
            })
            .cloned()
            .expect("expected monitoring event row");
        assert_eq!(
            event_default.get("ip").and_then(|value| value.as_str()),
            Some(pseudo_ip.as_str())
        );
        assert_eq!(
            event_default.get("admin").and_then(|value| value.as_str()),
            Some("[masked]")
        );
        assert_eq!(
            body_default
                .get("security_privacy")
                .and_then(|value| value.get("access_control"))
                .and_then(|value| value.get("view_mode"))
                .and_then(|value| value.as_str()),
            Some("pseudonymized_default")
        );

        let req_forensic = make_request(
            Method::Get,
            format!(
                "/admin/monitoring?hours=24&limit=5&forensic=1&forensic_ack={}",
                SECURITY_FORENSIC_ACK_VALUE
            )
            .as_str(),
            Vec::new(),
        );
        let resp_forensic = handle_admin_monitoring(&req_forensic, &store);
        assert_eq!(*resp_forensic.status(), 200u16);
        assert_eq!(
            resp_forensic
                .header("x-shuma-monitoring-security-mode")
                .and_then(|value| value.as_str()),
            Some("forensic_raw")
        );
        let body_forensic: serde_json::Value = serde_json::from_slice(resp_forensic.body()).unwrap();
        let event_forensic = body_forensic
            .get("details")
            .and_then(|value| value.get("events"))
            .and_then(|value| value.get("recent_events"))
            .and_then(|value| value.as_array())
            .and_then(|rows| {
                rows.iter().find(|row| {
                    row.get("reason").and_then(|value| value.as_str())
                        == Some("security_mode_probe")
                })
            })
            .cloned()
            .expect("expected forensic monitoring event row");
        assert_eq!(
            event_forensic.get("ip").and_then(|value| value.as_str()),
            Some(raw_ip)
        );
        assert_eq!(
            event_forensic.get("admin").and_then(|value| value.as_str()),
            Some("operator@example.com")
        );
    }

    #[test]
    fn admin_monitoring_delta_pseudonymizes_without_forensic_ack() {
        let store = TestStore::default();
        let now = now_ts();
        let raw_ip = "198.51.100.66";
        let pseudo_ip = pseudonymize_ip_identifier(raw_ip);
        log_event(
            &store,
            &EventLogEntry {
                ts: now,
                event: EventType::AdminAction,
                ip: Some(raw_ip.to_string()),
                reason: Some("delta_security_mode_probe".to_string()),
                outcome: Some("ok".to_string()),
                admin: Some("ops".to_string()),
            },
        );

        let req_default = make_request(
            Method::Get,
            "/admin/monitoring/delta?hours=24&limit=10",
            Vec::new(),
        );
        let resp_default = handle_admin_monitoring_delta(&req_default, &store);
        assert_eq!(*resp_default.status(), 200u16);
        let payload_default: serde_json::Value = serde_json::from_slice(resp_default.body()).unwrap();
        let events_default = payload_default
            .get("events")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(events_default.iter().any(|row| {
            row.get("reason").and_then(|value| value.as_str()) == Some("delta_security_mode_probe")
                && row.get("ip").and_then(|value| value.as_str())
                    == Some(pseudo_ip.as_str())
        }));

        let req_forensic = make_request(
            Method::Get,
            format!(
                "/admin/monitoring/delta?hours=24&limit=10&forensic=1&forensic_ack={}",
                SECURITY_FORENSIC_ACK_VALUE
            )
            .as_str(),
            Vec::new(),
        );
        let resp_forensic = handle_admin_monitoring_delta(&req_forensic, &store);
        assert_eq!(*resp_forensic.status(), 200u16);
        let payload_forensic: serde_json::Value = serde_json::from_slice(resp_forensic.body()).unwrap();
        let events_forensic = payload_forensic
            .get("events")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(events_forensic.iter().any(|row| {
            row.get("reason").and_then(|value| value.as_str()) == Some("delta_security_mode_probe")
                && row.get("ip").and_then(|value| value.as_str()) == Some(raw_ip)
        }));
    }

    #[test]
    fn admin_monitoring_cost_governance_surfaces_query_budget_degraded_state() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();

        let req = make_request(Method::Get, "/admin/monitoring?hours=720&limit=50", Vec::new());
        let resp = handle_admin_monitoring(&req, &store);
        assert_eq!(*resp.status(), 200u16);
        assert_eq!(
            resp.header("x-shuma-monitoring-cost-state")
                .and_then(|value| value.as_str()),
            Some("degraded")
        );
        assert_eq!(
            resp.header("x-shuma-monitoring-query-budget")
                .and_then(|value| value.as_str()),
            Some("exceeded")
        );

        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let cost = body
            .get("details")
            .and_then(|value| value.get("cost_governance"))
            .expect("cost_governance");
        assert_eq!(
            cost.get("query_budget_status").and_then(|value| value.as_str()),
            Some("exceeded")
        );
        assert_eq!(
            cost.get("degraded_state").and_then(|value| value.as_str()),
            Some("degraded")
        );
        assert!(
            cost.get("degraded_reasons")
                .and_then(|value| value.as_array())
                .map(|reasons| reasons.iter().any(|row| row.as_str() == Some("query_budget_exceeded")))
                .unwrap_or(false)
        );
        assert_eq!(
            body.get("details")
                .and_then(|value| value.get("events"))
                .and_then(|value| value.get("recent_events_window"))
                .and_then(|value| value.get("response_shaping_reason"))
                .and_then(|value| value.as_str()),
            Some("query_budget_guardrail")
        );
    }

    #[test]
    fn admin_monitoring_negotiates_gzip_and_reports_compression() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        let now = now_ts();
        for idx in 0..4000u64 {
            log_event(
                &store,
                &EventLogEntry {
                    ts: now.saturating_sub(idx),
                    event: EventType::AdminAction,
                    ip: Some(format!("198.51.100.{}", idx % 255)),
                    reason: Some(
                        format!(
                            "cdp_detected:large_payload_seed_reason_value_for_monitoring_compression_path:{}",
                            idx
                        ),
                    ),
                    outcome: Some(
                        "large_payload_seed_outcome_value_for_monitoring_compression_path".to_string(),
                    ),
                    admin: Some("ops".to_string()),
                },
            );
        }

        let mut builder = Request::builder();
        builder
            .method(Method::Get)
            .uri("/admin/monitoring?hours=24&limit=50")
            .header("host", "localhost:3000")
            .header("authorization", "Bearer changeme-dev-only-api-key")
            .header("origin", "http://localhost:3000")
            .header("sec-fetch-site", "same-origin")
            .header("idempotency-key", "compression-test-key")
            .header("accept-encoding", "gzip")
            .body(Vec::new());
        let req = builder.build();

        let resp = handle_admin_monitoring(&req, &store);
        assert_eq!(*resp.status(), 200u16);
        assert_eq!(
            resp.header("content-encoding")
                .and_then(|value| value.as_str()),
            Some("gzip")
        );

        let mut decoder = GzDecoder::new(resp.body());
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).unwrap();
        let body: serde_json::Value = serde_json::from_slice(decoded.as_slice()).unwrap();
        let compression = body
            .get("details")
            .and_then(|value| value.get("cost_governance"))
            .and_then(|value| value.get("compression"))
            .expect("compression payload");

        assert_eq!(
            compression.get("negotiated").and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            compression.get("algorithm").and_then(|value| value.as_str()),
            Some("gzip")
        );
        assert!(
            compression
                .get("reduction_percent")
                .and_then(|value| value.as_f64())
                .unwrap_or(0.0)
                > 0.0
        );
    }

    #[test]
    fn admin_ip_range_suggestions_returns_structured_payload() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();
        let now = now_ts();

        for host in 0..40usize {
            log_event(
                &store,
                &EventLogEntry {
                    ts: now,
                    event: EventType::Ban,
                    ip: Some(format!("198.51.100.{}", host)),
                    reason: Some("honeypot".to_string()),
                    outcome: Some("banned".to_string()),
                    admin: None,
                },
            );
        }

        let req = make_request(
            Method::Get,
            "/admin/ip-range/suggestions?hours=24&limit=5",
            Vec::new(),
        );
        let resp = handle_admin_ip_range_suggestions(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let summary = body.get("summary").expect("summary should exist");
        let suggestions = body
            .get("suggestions")
            .and_then(|value| value.as_array())
            .expect("suggestions array should exist");

        assert_eq!(
            body.get("hours").and_then(|value| value.as_u64()),
            Some(24u64)
        );
        assert!(!suggestions.is_empty());
        assert!(suggestions.len() <= 5);
        assert!(
            summary
                .get("suggestions_total")
                .and_then(|value| value.as_u64())
                .unwrap_or(0)
                >= 1
        );
        let first = suggestions.first().unwrap();
        assert!(first.get("cidr").and_then(|value| value.as_str()).is_some());
        assert!(
            first
                .get("recommended_action")
                .and_then(|value| value.as_str())
                .is_some()
        );
        assert!(
            first
                .get("recommended_mode")
                .and_then(|value| value.as_str())
                .is_some()
        );
    }

    #[test]
    fn admin_monitoring_tarpit_active_counters_are_site_scoped() {
        let _lock = crate::test_support::lock_env();
        let store = TestStore::default();

        store
            .set("tarpit:budget:active:global:default", b"3")
            .unwrap();
        store
            .set("tarpit:budget:active:global:other-site", b"9")
            .unwrap();
        store
            .set("tarpit:budget:active:bucket:default:bucket-a", b"2")
            .unwrap();
        store
            .set("tarpit:budget:active:bucket:default:bucket-b", b"1")
            .unwrap();
        store
            .set("tarpit:budget:active:bucket:other-site:bucket-z", b"7")
            .unwrap();

        let details = monitoring_details_payload(&store, "default", 24, 10, false);
        let tarpit = details.get("tarpit").unwrap();
        assert_eq!(
            tarpit
                .get("active")
                .and_then(|value| value.get("global"))
                .and_then(|value| value.as_u64()),
            Some(3)
        );
        let top_buckets = tarpit
            .get("active")
            .and_then(|value| value.get("top_buckets"))
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(top_buckets.iter().any(|entry| {
            entry.get("bucket").and_then(|value| value.as_str()) == Some("bucket-a")
                && entry.get("active").and_then(|value| value.as_u64()) == Some(2)
        }));
        assert!(top_buckets.iter().any(|entry| {
            entry.get("bucket").and_then(|value| value.as_str()) == Some("bucket-b")
                && entry.get("active").and_then(|value| value.as_u64()) == Some(1)
        }));
        assert!(!top_buckets.iter().any(|entry| {
            entry.get("bucket")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .contains("other-site")
        }));
        assert!(!top_buckets.iter().any(|entry| {
            entry.get("bucket").and_then(|value| value.as_str()) == Some("bucket-z")
        }));
    }

    #[test]
    fn admin_config_rejects_updates_when_admin_config_write_disabled() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
        let body = br#"{"test_mode":true}"#.to_vec();
        let req = make_request(Method::Post, "/admin/config", body);
        let store = TestStore::default();
        let resp = handle_admin_config(&req, &store, "default");
        assert_eq!(*resp.status(), 403u16);
        let msg = String::from_utf8_lossy(resp.body());
        assert!(msg.contains("SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false"));
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_geo_policy_lists() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        std::env::remove_var("SHUMA_GEO_RISK_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_ALLOW_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_CHALLENGE_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_MAZE_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_BLOCK_COUNTRIES");
        let store = TestStore::default();

        let body = br#"{
          "geo_risk": ["us", "CN", "us"],
          "geo_allow": ["gb"],
          "geo_challenge": ["br"],
          "geo_maze": ["ru"],
          "geo_block": ["kp"]
        }"#
        .to_vec();
        let post_req = make_request(Method::Post, "/admin/config", body);
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();

        assert_eq!(
            cfg.get("geo_risk").unwrap(),
            &serde_json::json!(["US", "CN"])
        );
        assert_eq!(cfg.get("geo_allow").unwrap(), &serde_json::json!(["GB"]));
        assert_eq!(
            cfg.get("geo_challenge").unwrap(),
            &serde_json::json!(["BR"])
        );
        assert_eq!(cfg.get("geo_maze").unwrap(), &serde_json::json!(["RU"]));
        assert_eq!(cfg.get("geo_block").unwrap(), &serde_json::json!(["KP"]));

        let get_req = make_request(Method::Get, "/admin/config", Vec::new());
        let get_resp = handle_admin_config(&get_req, &store, "default");
        assert_eq!(*get_resp.status(), 200u16);
        let get_json: serde_json::Value = serde_json::from_slice(get_resp.body()).unwrap();
        assert_eq!(
            get_json.get("geo_risk").unwrap(),
            &serde_json::json!(["US", "CN"])
        );
        assert_eq!(
            get_json.get("geo_allow").unwrap(),
            &serde_json::json!(["GB"])
        );
        assert_eq!(
            get_json.get("geo_challenge").unwrap(),
            &serde_json::json!(["BR"])
        );
        assert_eq!(
            get_json.get("geo_maze").unwrap(),
            &serde_json::json!(["RU"])
        );
        assert_eq!(
            get_json.get("geo_block").unwrap(),
            &serde_json::json!(["KP"])
        );
        std::env::remove_var("SHUMA_GEO_RISK_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_ALLOW_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_CHALLENGE_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_MAZE_COUNTRIES");
        std::env::remove_var("SHUMA_GEO_BLOCK_COUNTRIES");
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_non_iso_geo_country_codes() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let body = br#"{"geo_risk": ["US", "ZZ"]}"#.to_vec();
        let post_req = make_request(Method::Post, "/admin/config", body);
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("invalid country code"));
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_js_required_enforced_flag() {
        let _lock = crate::test_support::lock_env();
        let prior_js_required_env = std::env::var("SHUMA_JS_REQUIRED_ENFORCED").ok();
        std::env::remove_var("SHUMA_JS_REQUIRED_ENFORCED");
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"js_required_enforced":false}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("js_required_enforced"),
            Some(&serde_json::Value::Bool(false))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.js_required_enforced);

        if let Some(previous) = prior_js_required_env {
            std::env::set_var("SHUMA_JS_REQUIRED_ENFORCED", previous);
        } else {
            std::env::remove_var("SHUMA_JS_REQUIRED_ENFORCED");
        }
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_ai_policy_fields_via_first_class_keys() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "ai_policy_block_training": false,
                "ai_policy_block_search": true,
                "ai_policy_allow_search_engines": false
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json
            .get("config")
            .expect("config payload should exist");
        assert_eq!(
            cfg.get("ai_policy_block_training"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("ai_policy_block_search"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            cfg.get("ai_policy_allow_search_engines"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("robots_block_ai_training"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("robots_block_ai_search"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            cfg.get("robots_allow_search_engines"),
            Some(&serde_json::Value::Bool(false))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.robots_block_ai_training);
        assert!(saved_cfg.robots_block_ai_search);
        assert!(!saved_cfg.robots_allow_search_engines);
        let robots = crate::crawler_policy::robots::generate_robots_txt(&saved_cfg);
        assert!(robots.contains("# Content-Signal: ai-train=yes, search=no, ai-input=no"));
        assert!(!robots.contains("User-agent: GPTBot"));
        assert!(robots.contains("User-agent: PerplexityBot"));
        assert!(!robots.contains("User-agent: Googlebot"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn robots_preview_patch_applies_dirty_values_without_persisting_config() {
        let store = TestStore::default();
        let original_bytes = store.get("config:default").unwrap().unwrap();
        let mut cfg: crate::config::Config = serde_json::from_slice(&original_bytes).unwrap();
        let patch = json!({
            "robots_enabled": true,
            "ai_policy_block_training": false,
            "ai_policy_block_search": true,
            "ai_policy_allow_search_engines": false,
            "robots_crawl_delay": 4
        });

        apply_robots_preview_patch(&mut cfg, &patch);
        let payload = admin_robots_payload(&cfg);
        let preview = payload
            .get("preview")
            .and_then(|value| value.as_str())
            .expect("preview text should exist");

        assert!(preview.contains("# Content-Signal: ai-train=yes, search=no, ai-input=no"));
        assert!(!preview.contains("User-agent: GPTBot"));
        assert!(preview.contains("User-agent: PerplexityBot"));
        assert!(!preview.contains("User-agent: Googlebot"));
        assert!(preview.contains("User-agent: *"));
        assert!(preview.contains("Disallow: /"));

        let persisted_bytes = store.get("config:default").unwrap().unwrap();
        assert_eq!(persisted_bytes, original_bytes);
    }

    #[test]
    fn admin_config_rejects_out_of_range_rate_limit() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"rate_limit":0}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("rate_limit out of range"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_unknown_top_level_key() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"rate_limit":100,"unknown_key":true}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("unknown field `unknown_key`"));
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_deprecated_simulation_namespace_key() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"sim_telemetry_namespace":"legacy-sim-plane"}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("unknown field `sim_telemetry_namespace`"));
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_typed_field_type_mismatch() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"rate_limit":"100"}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("Invalid config payload"));
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_validate_endpoint_accepts_valid_patch_without_persisting() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let before = store.get("config:default").unwrap().unwrap();

        let req = make_request(
            Method::Post,
            "/admin/config/validate",
            br#"{"rate_limit":1234}"#.to_vec(),
        );
        let resp = handle_admin_config_validate(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body.get("valid"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(
            body.get("issues").and_then(|value| value.as_array()).map(|v| v.len()),
            Some(0)
        );

        let after = store.get("config:default").unwrap().unwrap();
        assert_eq!(before, after);
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_validate_endpoint_reports_structured_issue_details() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let req = make_request(
            Method::Post,
            "/admin/config/validate",
            br#"{"rate_limit":"oops"}"#.to_vec(),
        );
        let resp = handle_admin_config_validate(&req, &store, "default");
        assert_eq!(*resp.status(), 200u16);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body.get("valid"), Some(&serde_json::Value::Bool(false)));
        let issue = body
            .get("issues")
            .and_then(|value| value.as_array())
            .and_then(|issues| issues.first())
            .expect("expected validation issue");
        assert_eq!(
            issue.get("field").and_then(|value| value.as_str()),
            Some("rate_limit")
        );
        assert!(
            issue
                .get("expected")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .contains("Type mismatch")
        );
        assert_eq!(
            issue.get("received").and_then(|value| value.as_str()),
            Some("oops")
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_lists_and_cdp_ban_duration() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "honeypot_enabled": false,
                "honeypots": ["/instaban", "/trap-b"],
                "browser_policy_enabled": false,
                "browser_block": [["Chrome",126],["Firefox",120]],
                "browser_allowlist": [["Safari",16]],
                "bypass_allowlists_enabled": false,
                "allowlist": ["203.0.113.0/24", "198.51.100.9"],
                "path_allowlist_enabled": false,
                "path_allowlist": ["/status", "/assets/*"],
                "ban_durations": {"cdp": 777}
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(cfg.get("honeypot_enabled"), Some(&serde_json::Value::Bool(false)));
        assert_eq!(cfg.get("honeypots"), Some(&serde_json::json!(["/instaban", "/trap-b"])));
        assert_eq!(
            cfg.get("browser_block"),
            Some(&serde_json::json!([["Chrome", 126], ["Firefox", 120]]))
        );
        assert_eq!(
            cfg.get("browser_policy_enabled"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("browser_allowlist"),
            Some(&serde_json::json!([["Safari", 16]]))
        );
        assert_eq!(
            cfg.get("bypass_allowlists_enabled"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("allowlist"),
            Some(&serde_json::json!(["203.0.113.0/24", "198.51.100.9"]))
        );
        assert_eq!(
            cfg.get("path_allowlist_enabled"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("path_allowlist"),
            Some(&serde_json::json!(["/status", "/assets/*"]))
        );
        assert_eq!(
            cfg.get("ban_durations")
                .and_then(|v| v.get("cdp"))
                .and_then(|v| v.as_u64()),
            Some(777)
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.honeypot_enabled);
        assert_eq!(
            saved_cfg.honeypots,
            vec!["/instaban".to_string(), "/trap-b".to_string()]
        );
        assert_eq!(
            saved_cfg.browser_block,
            vec![("Chrome".to_string(), 126), ("Firefox".to_string(), 120)]
        );
        assert!(!saved_cfg.browser_policy_enabled);
        assert_eq!(
            saved_cfg.browser_allowlist,
            vec![("Safari".to_string(), 16)]
        );
        assert!(!saved_cfg.bypass_allowlists_enabled);
        assert_eq!(
            saved_cfg.allowlist,
            vec!["203.0.113.0/24".to_string(), "198.51.100.9".to_string()]
        );
        assert!(!saved_cfg.path_allowlist_enabled);
        assert_eq!(
            saved_cfg.path_allowlist,
            vec!["/status".to_string(), "/assets/*".to_string()]
        );
        assert_eq!(saved_cfg.ban_durations.cdp, 777);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_invalid_honeypot_path() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let invalid_payloads = [
            br#"{"honeypots":["instaban"]}"#.to_vec(),
            br#"{"honeypots":["/instaban.  gfdgfdgdfgderg.  egfsdfg"]}"#.to_vec(),
            br#"{"honeypots":["/trap?source=bot"]}"#.to_vec(),
            br#"{"honeypots":["/trap/%ZZ"]}"#.to_vec(),
        ];

        for payload in invalid_payloads {
            let post_req = make_request(Method::Post, "/admin/config", payload);
            let post_resp = handle_admin_config(&post_req, &store, "default");
            assert_eq!(*post_resp.status(), 400u16);
            let msg = String::from_utf8_lossy(post_resp.body());
            assert!(msg.contains("invalid path"));
            assert!(msg.contains("percent-encoded"));
        }
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_accepts_valid_honeypot_path_percent_encoding() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"honeypots":["/instaban","/trap/%7Ebot"]}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_pow_enabled() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"pow_enabled":false}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(cfg.get("pow_enabled"), Some(&serde_json::Value::Bool(false)));

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.pow_enabled);

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_updates_challenge_puzzle_transform_count() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"challenge_puzzle_transform_count":7}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("challenge_puzzle_transform_count"),
            Some(&serde_json::Value::Number(7.into()))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(saved_cfg.challenge_puzzle_transform_count, 7);

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_updates_challenge_puzzle_runtime_controls() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "challenge_puzzle_seed_ttl_seconds": 240,
                "challenge_puzzle_attempt_limit_per_window": 8,
                "challenge_puzzle_attempt_window_seconds": 420
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("challenge_puzzle_seed_ttl_seconds"),
            Some(&serde_json::Value::Number(240.into()))
        );
        assert_eq!(
            cfg.get("challenge_puzzle_attempt_limit_per_window"),
            Some(&serde_json::Value::Number(8.into()))
        );
        assert_eq!(
            cfg.get("challenge_puzzle_attempt_window_seconds"),
            Some(&serde_json::Value::Number(420.into()))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(saved_cfg.challenge_puzzle_seed_ttl_seconds, 240);
        assert_eq!(saved_cfg.challenge_puzzle_attempt_limit_per_window, 8);
        assert_eq!(saved_cfg.challenge_puzzle_attempt_window_seconds, 420);

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_updates_challenge_puzzle_enabled() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"challenge_puzzle_enabled":false}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("challenge_puzzle_enabled"),
            Some(&serde_json::Value::Bool(false))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.challenge_puzzle_enabled);

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_updates_tarpit_enabled() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"tarpit_enabled":false}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(cfg.get("tarpit_enabled"), Some(&serde_json::Value::Bool(false)));

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.tarpit_enabled);

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_updates_tarpit_runtime_controls() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "tarpit_progress_token_ttl_seconds": 140,
                "tarpit_progress_replay_ttl_seconds": 420,
                "tarpit_hashcash_min_difficulty": 9,
                "tarpit_hashcash_max_difficulty": 17,
                "tarpit_hashcash_base_difficulty": 12,
                "tarpit_hashcash_adaptive": false,
                "tarpit_step_chunk_base_bytes": 4096,
                "tarpit_step_chunk_max_bytes": 16384,
                "tarpit_step_jitter_percent": 20,
                "tarpit_shard_rotation_enabled": false,
                "tarpit_egress_window_seconds": 90,
                "tarpit_egress_global_bytes_per_window": 8388608,
                "tarpit_egress_per_ip_bucket_bytes_per_window": 1048576,
                "tarpit_egress_per_flow_max_bytes": 524288,
                "tarpit_egress_per_flow_max_duration_seconds": 180,
                "tarpit_max_concurrent_global": 24,
                "tarpit_max_concurrent_per_ip_bucket": 3,
                "tarpit_fallback_action":"block"
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("tarpit_progress_token_ttl_seconds"),
            Some(&serde_json::Value::Number(140.into()))
        );
        assert_eq!(
            cfg.get("tarpit_progress_replay_ttl_seconds"),
            Some(&serde_json::Value::Number(420.into()))
        );
        assert_eq!(
            cfg.get("tarpit_hashcash_min_difficulty"),
            Some(&serde_json::Value::Number(9.into()))
        );
        assert_eq!(
            cfg.get("tarpit_hashcash_max_difficulty"),
            Some(&serde_json::Value::Number(17.into()))
        );
        assert_eq!(
            cfg.get("tarpit_hashcash_base_difficulty"),
            Some(&serde_json::Value::Number(12.into()))
        );
        assert_eq!(
            cfg.get("tarpit_hashcash_adaptive"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("tarpit_step_chunk_base_bytes"),
            Some(&serde_json::Value::Number(4096.into()))
        );
        assert_eq!(
            cfg.get("tarpit_step_chunk_max_bytes"),
            Some(&serde_json::Value::Number(16384.into()))
        );
        assert_eq!(
            cfg.get("tarpit_step_jitter_percent"),
            Some(&serde_json::Value::Number(20.into()))
        );
        assert_eq!(
            cfg.get("tarpit_shard_rotation_enabled"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            cfg.get("tarpit_egress_window_seconds"),
            Some(&serde_json::Value::Number(90.into()))
        );
        assert_eq!(
            cfg.get("tarpit_egress_global_bytes_per_window"),
            Some(&serde_json::Value::Number(8_388_608u64.into()))
        );
        assert_eq!(
            cfg.get("tarpit_egress_per_ip_bucket_bytes_per_window"),
            Some(&serde_json::Value::Number(1_048_576u64.into()))
        );
        assert_eq!(
            cfg.get("tarpit_egress_per_flow_max_bytes"),
            Some(&serde_json::Value::Number(524_288u64.into()))
        );
        assert_eq!(
            cfg.get("tarpit_egress_per_flow_max_duration_seconds"),
            Some(&serde_json::Value::Number(180.into()))
        );
        assert_eq!(
            cfg.get("tarpit_max_concurrent_global"),
            Some(&serde_json::Value::Number(24.into()))
        );
        assert_eq!(
            cfg.get("tarpit_max_concurrent_per_ip_bucket"),
            Some(&serde_json::Value::Number(3.into()))
        );
        assert_eq!(
            cfg.get("tarpit_fallback_action"),
            Some(&serde_json::Value::String("block".to_string()))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(saved_cfg.tarpit_progress_token_ttl_seconds, 140);
        assert_eq!(saved_cfg.tarpit_progress_replay_ttl_seconds, 420);
        assert_eq!(saved_cfg.tarpit_hashcash_min_difficulty, 9);
        assert_eq!(saved_cfg.tarpit_hashcash_max_difficulty, 17);
        assert_eq!(saved_cfg.tarpit_hashcash_base_difficulty, 12);
        assert!(!saved_cfg.tarpit_hashcash_adaptive);
        assert_eq!(saved_cfg.tarpit_step_chunk_base_bytes, 4096);
        assert_eq!(saved_cfg.tarpit_step_chunk_max_bytes, 16384);
        assert_eq!(saved_cfg.tarpit_step_jitter_percent, 20);
        assert!(!saved_cfg.tarpit_shard_rotation_enabled);
        assert_eq!(saved_cfg.tarpit_egress_window_seconds, 90);
        assert_eq!(saved_cfg.tarpit_egress_global_bytes_per_window, 8_388_608);
        assert_eq!(saved_cfg.tarpit_egress_per_ip_bucket_bytes_per_window, 1_048_576);
        assert_eq!(saved_cfg.tarpit_egress_per_flow_max_bytes, 524_288);
        assert_eq!(saved_cfg.tarpit_egress_per_flow_max_duration_seconds, 180);
        assert_eq!(saved_cfg.tarpit_max_concurrent_global, 24);
        assert_eq!(saved_cfg.tarpit_max_concurrent_per_ip_bucket, 3);
        assert_eq!(
            saved_cfg.tarpit_fallback_action,
            crate::config::TarpitFallbackAction::Block
        );

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_rejects_invalid_tarpit_runtime_controls() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let invalid_token_ttl = make_request(
            Method::Post,
            "/admin/config",
            br#"{"tarpit_progress_token_ttl_seconds": 10}"#.to_vec(),
        );
        let invalid_token_ttl_resp = handle_admin_config(&invalid_token_ttl, &store, "default");
        assert_eq!(*invalid_token_ttl_resp.status(), 400u16);
        assert!(
            String::from_utf8_lossy(invalid_token_ttl_resp.body())
                .contains("tarpit_progress_token_ttl_seconds out of range")
        );

        let invalid_chunk_bounds = make_request(
            Method::Post,
            "/admin/config",
            br#"{"tarpit_step_chunk_base_bytes": 8192, "tarpit_step_chunk_max_bytes": 1024}"#
                .to_vec(),
        );
        let invalid_chunk_bounds_resp =
            handle_admin_config(&invalid_chunk_bounds, &store, "default");
        assert_eq!(*invalid_chunk_bounds_resp.status(), 400u16);
        assert!(
            String::from_utf8_lossy(invalid_chunk_bounds_resp.body())
                .contains("tarpit_step_chunk_max_bytes must be >=")
        );

        let invalid_budget = make_request(
            Method::Post,
            "/admin/config",
            br#"{"tarpit_max_concurrent_global": 2, "tarpit_max_concurrent_per_ip_bucket": 5}"#
                .to_vec(),
        );
        let invalid_budget_resp = handle_admin_config(&invalid_budget, &store, "default");
        assert_eq!(*invalid_budget_resp.status(), 400u16);
        assert!(
            String::from_utf8_lossy(invalid_budget_resp.body())
                .contains("tarpit_max_concurrent_per_ip_bucket must be <=")
        );

        let invalid_egress = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "tarpit_egress_global_bytes_per_window": 10000,
                "tarpit_egress_per_ip_bucket_bytes_per_window": 12000
            }"#
            .to_vec(),
        );
        let invalid_egress_resp = handle_admin_config(&invalid_egress, &store, "default");
        assert_eq!(*invalid_egress_resp.status(), 400u16);
        assert!(
            String::from_utf8_lossy(invalid_egress_resp.body())
                .contains("tarpit_egress_per_ip_bucket_bytes_per_window must be <=")
        );

        let invalid_fallback = make_request(
            Method::Post,
            "/admin/config",
            br#"{"tarpit_fallback_action":"challenge"}"#.to_vec(),
        );
        let invalid_fallback_resp = handle_admin_config(&invalid_fallback, &store, "default");
        assert_eq!(*invalid_fallback_resp.status(), 400u16);
        assert!(
            String::from_utf8_lossy(invalid_fallback_resp.body())
                .contains("tarpit_fallback_action must be one of")
        );

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_rejects_challenge_puzzle_transform_count_out_of_range() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"challenge_puzzle_transform_count":9}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("challenge_puzzle_transform_count out of range"));
        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_rejects_invalid_challenge_puzzle_runtime_controls() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let invalid_seed_ttl = make_request(
            Method::Post,
            "/admin/config",
            br#"{"challenge_puzzle_seed_ttl_seconds": 301}"#.to_vec(),
        );
        let invalid_seed_ttl_resp = handle_admin_config(&invalid_seed_ttl, &store, "default");
        assert_eq!(*invalid_seed_ttl_resp.status(), 400u16);
        assert!(String::from_utf8_lossy(invalid_seed_ttl_resp.body())
            .contains("challenge_puzzle_seed_ttl_seconds out of range"));

        let invalid_attempt_limit = make_request(
            Method::Post,
            "/admin/config",
            br#"{"challenge_puzzle_attempt_limit_per_window": 0}"#.to_vec(),
        );
        let invalid_attempt_limit_resp =
            handle_admin_config(&invalid_attempt_limit, &store, "default");
        assert_eq!(*invalid_attempt_limit_resp.status(), 400u16);
        assert!(String::from_utf8_lossy(invalid_attempt_limit_resp.body())
            .contains("challenge_puzzle_attempt_limit_per_window out of range"));

        let invalid_attempt_window = make_request(
            Method::Post,
            "/admin/config",
            br#"{"challenge_puzzle_attempt_window_seconds": 3601}"#.to_vec(),
        );
        let invalid_attempt_window_resp =
            handle_admin_config(&invalid_attempt_window, &store, "default");
        assert_eq!(*invalid_attempt_window_resp.status(), 400u16);
        assert!(String::from_utf8_lossy(invalid_attempt_window_resp.body())
            .contains("challenge_puzzle_attempt_window_seconds out of range"));

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_updates_not_a_bot_controls() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "not_a_bot_enabled": false,
                "not_a_bot_risk_threshold": 2,
                "not_a_bot_pass_score": 8,
                "not_a_bot_fail_score": 5,
                "not_a_bot_nonce_ttl_seconds": 150,
                "not_a_bot_marker_ttl_seconds": 900,
                "not_a_bot_attempt_limit_per_window": 9,
                "not_a_bot_attempt_window_seconds": 420
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(cfg.get("not_a_bot_enabled"), Some(&serde_json::Value::Bool(false)));
        assert_eq!(
            cfg.get("not_a_bot_risk_threshold"),
            Some(&serde_json::Value::Number(2.into()))
        );
        assert_eq!(
            cfg.get("not_a_bot_pass_score"),
            Some(&serde_json::Value::Number(8.into()))
        );
        assert_eq!(
            cfg.get("not_a_bot_fail_score"),
            Some(&serde_json::Value::Number(5.into()))
        );
        assert_eq!(
            cfg.get("not_a_bot_nonce_ttl_seconds"),
            Some(&serde_json::Value::Number(150.into()))
        );
        assert_eq!(
            cfg.get("not_a_bot_marker_ttl_seconds"),
            Some(&serde_json::Value::Number(900.into()))
        );
        assert_eq!(
            cfg.get("not_a_bot_attempt_limit_per_window"),
            Some(&serde_json::Value::Number(9.into()))
        );
        assert_eq!(
            cfg.get("not_a_bot_attempt_window_seconds"),
            Some(&serde_json::Value::Number(420.into()))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert!(!saved_cfg.not_a_bot_enabled);
        assert_eq!(saved_cfg.not_a_bot_risk_threshold, 2);
        assert_eq!(saved_cfg.not_a_bot_pass_score, 8);
        assert_eq!(saved_cfg.not_a_bot_fail_score, 5);
        assert_eq!(saved_cfg.not_a_bot_nonce_ttl_seconds, 150);
        assert_eq!(saved_cfg.not_a_bot_marker_ttl_seconds, 900);
        assert_eq!(saved_cfg.not_a_bot_attempt_limit_per_window, 9);
        assert_eq!(saved_cfg.not_a_bot_attempt_window_seconds, 420);

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_rejects_invalid_not_a_bot_controls() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let invalid_threshold = make_request(
            Method::Post,
            "/admin/config",
            br#"{"not_a_bot_risk_threshold": 11}"#.to_vec(),
        );
        let invalid_threshold_resp = handle_admin_config(&invalid_threshold, &store, "default");
        assert_eq!(*invalid_threshold_resp.status(), 400u16);
        assert!(String::from_utf8_lossy(invalid_threshold_resp.body()).contains("not_a_bot_risk_threshold out of range"));

        let invalid_score_order = make_request(
            Method::Post,
            "/admin/config",
            br#"{"not_a_bot_pass_score": 6, "not_a_bot_fail_score": 7}"#.to_vec(),
        );
        let invalid_score_order_resp = handle_admin_config(&invalid_score_order, &store, "default");
        assert_eq!(*invalid_score_order_resp.status(), 400u16);
        assert!(String::from_utf8_lossy(invalid_score_order_resp.body()).contains("not_a_bot_fail_score must be <= not_a_bot_pass_score"));

        clear_env(&["SHUMA_ADMIN_CONFIG_WRITE_ENABLED"]);
    }

    #[test]
    fn admin_config_updates_defence_modes() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"defence_modes":{"rate":"signal","geo":"enforce","js":"off"}}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("defence_modes"),
            Some(&serde_json::json!({"rate":"signal","geo":"enforce","js":"off"}))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(
            saved_cfg.defence_modes.rate,
            crate::config::ComposabilityMode::Signal
        );
        assert_eq!(
            saved_cfg.defence_modes.geo,
            crate::config::ComposabilityMode::Enforce
        );
        assert_eq!(
            saved_cfg.defence_modes.js,
            crate::config::ComposabilityMode::Off
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_ip_range_policy_fields() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "ip_range_policy_mode":"enforce",
                "ip_range_emergency_allowlist":["203.0.113.0/24"],
                "ip_range_custom_rules":[
                    {
                        "id":"dc_block",
                        "enabled":true,
                        "cidrs":["198.51.100.0/24"],
                        "action":"forbidden_403"
                    },
                    {
                        "id":"redirect_known",
                        "enabled":true,
                        "cidrs":["192.0.2.0/24"],
                        "action":"redirect_308",
                        "redirect_url":"https://example.com/security-check"
                    }
                ]
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let body: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = body.get("config").unwrap();
        assert_eq!(
            cfg.get("ip_range_policy_mode"),
            Some(&serde_json::json!("enforce"))
        );
        assert_eq!(
            cfg.get("ip_range_emergency_allowlist"),
            Some(&serde_json::json!(["203.0.113.0/24"]))
        );
        assert_eq!(
            cfg.get("ip_range_custom_rules")
                .and_then(|value| value.as_array())
                .map(|entries| entries.len()),
            Some(2)
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(
            saved_cfg.ip_range_policy_mode,
            crate::config::IpRangePolicyMode::Enforce
        );
        assert_eq!(
            saved_cfg.ip_range_emergency_allowlist,
            vec!["203.0.113.0/24".to_string()]
        );
        assert_eq!(saved_cfg.ip_range_custom_rules.len(), 2);

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_invalid_ip_range_payloads() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let invalid_cidr = make_request(
            Method::Post,
            "/admin/config",
            br#"{"ip_range_custom_rules":[{"id":"bad","enabled":true,"cidrs":["invalid"],"action":"forbidden_403"}]}"#
                .to_vec(),
        );
        let invalid_cidr_resp = handle_admin_config(&invalid_cidr, &store, "default");
        assert_eq!(*invalid_cidr_resp.status(), 400u16);
        assert!(String::from_utf8_lossy(invalid_cidr_resp.body()).contains("invalid"));

        let missing_redirect = make_request(
            Method::Post,
            "/admin/config",
            br#"{"ip_range_custom_rules":[{"id":"redir","enabled":true,"cidrs":["203.0.113.0/24"],"action":"redirect_308"}]}"#
                .to_vec(),
        );
        let missing_redirect_resp = handle_admin_config(&missing_redirect, &store, "default");
        assert_eq!(*missing_redirect_resp.status(), 400u16);
        assert!(String::from_utf8_lossy(missing_redirect_resp.body()).contains("redirect_url"));

        let custom_standard = make_request(
            Method::Post,
            "/admin/config",
            br#"{"ip_range_custom_rules":[{"id":"noop","enabled":true,"cidrs":["203.0.113.0/24"],"action":"standard"}]}"#
                .to_vec(),
        );
        let custom_standard_resp = handle_admin_config(&custom_standard, &store, "default");
        assert_eq!(*custom_standard_resp.status(), 400u16);
        assert!(String::from_utf8_lossy(custom_standard_resp.body()).contains("must be one of"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_invalid_defence_mode_value() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"defence_modes":{"rate":"invalid"}}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("defence_modes.rate must be one of"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_unknown_defence_mode_key() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"defence_modes":{"rate":"both","foo":"off"}}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("unknown field `foo`"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_updates_provider_backends_and_edge_mode() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{
                "provider_backends": {
                    "rate_limiter": "external",
                    "ban_store": "external",
                    "fingerprint_signal": "external"
                },
                "edge_integration_mode": "additive"
            }"#
            .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 200u16);
        let post_json: serde_json::Value = serde_json::from_slice(post_resp.body()).unwrap();
        let cfg = post_json.get("config").unwrap();
        assert_eq!(
            cfg.get("provider_backends"),
            Some(&serde_json::json!({
                "rate_limiter": "external",
                "ban_store": "external",
                "challenge_engine": "internal",
                "maze_tarpit": "internal",
                "fingerprint_signal": "external"
            }))
        );
        assert_eq!(
            cfg.get("edge_integration_mode"),
            Some(&serde_json::json!("additive"))
        );

        let saved_bytes = store.get("config:default").unwrap().unwrap();
        let saved_cfg: crate::config::Config = serde_json::from_slice(&saved_bytes).unwrap();
        assert_eq!(
            saved_cfg.provider_backends.rate_limiter,
            crate::config::ProviderBackend::External
        );
        assert_eq!(
            saved_cfg.provider_backends.ban_store,
            crate::config::ProviderBackend::External
        );
        assert_eq!(
            saved_cfg.provider_backends.fingerprint_signal,
            crate::config::ProviderBackend::External
        );
        assert_eq!(
            saved_cfg.edge_integration_mode,
            crate::config::EdgeIntegrationMode::Additive
        );

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_invalid_provider_backend_value() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"provider_backends":{"rate_limiter":"invalid"}}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("provider_backends.rate_limiter must be one of"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_invalid_edge_integration_mode() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"edge_integration_mode":"invalid"}"#.to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("edge_integration_mode must be one of"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }

    #[test]
    fn admin_config_rejects_unknown_provider_backend_key() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();

        let post_req = make_request(
            Method::Post,
            "/admin/config",
            br#"{"provider_backends":{"fingerprint_signal":"external","unknown":"external"}}"#
                .to_vec(),
        );
        let post_resp = handle_admin_config(&post_req, &store, "default");
        assert_eq!(*post_resp.status(), 400u16);
        let msg = String::from_utf8_lossy(post_resp.body());
        assert!(msg.contains("unknown field `unknown`"));

        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }
}

#[cfg(test)]
mod admin_auth_tests {
    use super::*;
    use spin_sdk::http::{Method, Request};
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl crate::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            self.map.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn login_request(api_key: &str) -> Request {
        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/admin/login")
            .header("content-type", "application/json")
            .body(format!(r#"{{"api_key":"{}"}}"#, api_key).into_bytes());
        builder.build()
    }

    fn logout_request() -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Post).uri("/admin/logout");
        builder.build()
    }

    #[test]
    fn login_invalid_api_key_is_rate_limited() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        std::env::set_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE", "2");
        let store = TestStore::default();

        let req = login_request("wrong-key");
        let first = handle_admin_login(&req, &store);
        assert_eq!(*first.status(), 401u16);

        let second = handle_admin_login(&req, &store);
        assert_eq!(*second.status(), 401u16);

        let third = handle_admin_login(&req, &store);
        assert_eq!(*third.status(), 429u16);

        std::env::remove_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE");
        std::env::remove_var("SHUMA_API_KEY");
    }

    #[test]
    fn logout_unauthorized_is_rate_limited() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE", "1");
        let store = TestStore::default();
        let req = logout_request();

        let first = handle_admin_logout(&req, &store);
        assert_eq!(*first.status(), 401u16);

        let second = handle_admin_logout(&req, &store);
        assert_eq!(*second.status(), 429u16);

        std::env::remove_var("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE");
    }

    #[test]
    fn write_access_matrix_covers_only_mutating_admin_routes() {
        assert!(request_requires_admin_write("/admin/config", &Method::Post));
        assert!(request_requires_admin_write(
            "/admin/config/validate",
            &Method::Post
        ));
        assert!(request_requires_admin_write(
            "/admin/adversary-sim/control",
            &Method::Post
        ));
        assert!(request_requires_admin_write(
            "/admin/adversary-sim/history/cleanup",
            &Method::Post
        ));
        assert!(request_requires_admin_write("/admin/ban", &Method::Post));
        assert!(request_requires_admin_write("/admin/unban", &Method::Post));
        assert!(!request_requires_admin_write(
            "/admin/maze/preview",
            &Method::Post
        ));
        assert!(!request_requires_admin_write(
            "/admin/tarpit/preview",
            &Method::Post
        ));
        assert!(!request_requires_admin_write(
            "/admin/events",
            &Method::Post
        ));
        assert!(!request_requires_admin_write(
            "/admin/monitoring",
            &Method::Post
        ));
        assert!(!request_requires_admin_write("/admin/config", &Method::Get));
        assert!(!request_requires_admin_write(
            "/admin/config/validate",
            &Method::Get
        ));
        assert!(!request_requires_admin_write(
            "/admin/adversary-sim/control",
            &Method::Get
        ));
        assert!(!request_requires_admin_write(
            "/admin/adversary-sim/status",
            &Method::Get
        ));
        assert!(!request_requires_admin_write(
            "/admin/adversary-sim/history/cleanup",
            &Method::Get
        ));
        assert!(!request_requires_admin_write(
            "/admin/analytics",
            &Method::Get
        ));
    }
}

/// Utility to get current unix timestamp
pub fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
// src/admin.rs
// Admin API endpoints for WASM Bot Defence
// Provides HTTP endpoints for ban management and analytics, protected by API key auth.

use serde_json::json;
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::key_value::Store;

const ADMIN_BAN_DURATION_MIN: u64 = 60;
const ADMIN_BAN_DURATION_MAX: u64 = 31_536_000;
const ADVERSARY_SIM_DURATION_SECONDS_MIN: u64 = crate::config::ADVERSARY_SIM_DURATION_SECONDS_MIN;
const ADVERSARY_SIM_DURATION_SECONDS_MAX: u64 = crate::config::ADVERSARY_SIM_DURATION_SECONDS_MAX;

/// Returns true if the path is a valid admin endpoint (prevents path traversal/abuse).
fn sanitize_path(path: &str) -> bool {
    matches!(
        path,
        "/admin"
            | "/admin/login"
            | "/admin/session"
            | "/admin/logout"
            | "/admin/ban"
            | "/admin/unban"
            | "/admin/analytics"
            | "/admin/events"
            | "/admin/config"
            | "/admin/config/validate"
            | "/admin/config/export"
            | "/admin/adversary-sim/control"
            | "/admin/adversary-sim/status"
            | "/admin/adversary-sim/history/cleanup"
            | "/admin/maze"
            | "/admin/maze/preview"
            | "/admin/tarpit/preview"
            | "/admin/maze/seeds"
            | "/admin/maze/seeds/refresh"
            | "/admin/robots"
            | "/admin/robots/preview"
            | "/admin/cdp"
            | "/admin/cdp/events"
            | "/admin/monitoring"
            | "/admin/monitoring/delta"
            | "/admin/monitoring/stream"
            | "/admin/ip-bans/delta"
            | "/admin/ip-bans/stream"
            | "/admin/ip-range/suggestions"
    )
}

fn session_cookie_value(session_id: &str) -> String {
    let max_age = crate::admin::auth::admin_session_ttl_seconds();
    let secure = if crate::config::https_enforced() {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{}",
        crate::admin::auth::admin_session_cookie_name(),
        session_id,
        max_age,
        secure
    )
}

fn clear_session_cookie_value() -> String {
    let secure = if crate::config::https_enforced() {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{}",
        crate::admin::auth::admin_session_cookie_name(),
        secure
    )
}

fn too_many_admin_auth_attempts_response() -> Response {
    Response::builder()
        .status(429)
        .header("Retry-After", "60")
        .header("Cache-Control", "no-store")
        .body("Too Many Requests")
        .build()
}

const ADMIN_EXPENSIVE_READ_SITE_ID: &str = "admin-read-expensive";
const ADMIN_EXPENSIVE_READ_LIMIT_PER_MINUTE: u32 = 120;
const ADMIN_EXPENSIVE_READ_SESSION_SITE_ID: &str = "admin-read-expensive-session";
const ADMIN_EXPENSIVE_READ_SESSION_LIMIT_PER_MINUTE: u32 = 120;
const ADMIN_DASHBOARD_REFRESH_SESSION_SITE_ID: &str = "admin-dashboard-refresh-session";
const ADMIN_DASHBOARD_REFRESH_SESSION_LIMIT_PER_MINUTE: u32 = 120;
const ADVERSARY_SIM_CONTROL_SESSION_SITE_ID: &str = "adversary-sim-control-session";
const ADVERSARY_SIM_CONTROL_IP_SITE_ID: &str = "adversary-sim-control-ip";
const MONITORING_FRESHNESS_SLO_P50_MS: u64 = 150;
const MONITORING_FRESHNESS_SLO_P95_MS: u64 = 300;
const MONITORING_FRESHNESS_SLO_P99_MS: u64 = 500;
const MONITORING_MANUAL_REFRESH_STALENESS_BOUND_MS: u64 = 60_000;
const MONITORING_MAX_ALLOWED_LAG_BEFORE_DEGRADED_MS: u64 = 2_000;
const INTERNAL_ADVERSARY_SIM_BEAT_PATH: &str = "/internal/adversary-sim/beat";
const MONITORING_STALE_LAG_THRESHOLD_MS: u64 = 10_000;
const MONITORING_LOAD_ENVELOPE_EVENTS_PER_SEC: u64 = 1_000;
const MONITORING_LOAD_ENVELOPE_OPERATOR_CLIENTS: u64 = 5;
const MONITORING_LOAD_ENVELOPE_QUERY_COST_CEILING_PER_MINUTE: u32 =
    ADMIN_DASHBOARD_REFRESH_SESSION_LIMIT_PER_MINUTE;
const MONITORING_QUERY_BUDGET_REQUESTS_PER_SECOND: u64 = 1;
const MONITORING_QUERY_BUDGET_STANDARD_MAX_COST_UNITS: u64 = 240;
const MONITORING_QUERY_BUDGET_ELEVATED_MAX_COST_UNITS: u64 = 1_200;
const MONITORING_QUERY_BUDGET_HEAVY_MAX_COST_UNITS: u64 = 3_600;
const MONITORING_PAYLOAD_BUDGET_P95_KB: f64 = 512.0;
const MONITORING_COMPRESSION_MIN_PAYLOAD_BYTES: usize = 64 * 1024;
const MONITORING_COMPRESSION_MIN_REDUCTION_PERCENT: f64 = 30.0;
const MONITORING_COST_ENVELOPE_INGEST_EVENTS_PER_SECOND_DEV: u64 =
    MONITORING_LOAD_ENVELOPE_EVENTS_PER_SEC;
const MONITORING_COST_ENVELOPE_INGEST_EVENTS_PER_SECOND_PROD: u64 =
    MONITORING_LOAD_ENVELOPE_EVENTS_PER_SEC;
const MONITORING_COST_ENVELOPE_QUERY_CALLS_PER_SECOND_CLIENT_DEV: f64 =
    MONITORING_QUERY_BUDGET_REQUESTS_PER_SECOND as f64;
const MONITORING_COST_ENVELOPE_QUERY_CALLS_PER_SECOND_CLIENT_PROD: f64 =
    MONITORING_QUERY_BUDGET_REQUESTS_PER_SECOND as f64;
const MONITORING_STREAM_RETRY_MS: u64 = 1_000;
const MONITORING_STREAM_MAX_BUFFER_EVENTS: usize = 250;

fn too_many_admin_read_requests_response() -> Response {
    Response::builder()
        .status(429)
        .header("Retry-After", "60")
        .header("Cache-Control", "no-store")
        .body("Too Many Requests")
        .build()
}

fn expensive_admin_read_is_limited(
    store: &Store,
    req: &Request,
    auth: &crate::admin::auth::AdminAuthResult,
    provider_registry: Option<&crate::providers::registry::ProviderRegistry>,
) -> bool {
    if expensive_admin_read_limit_check(
        store,
        req,
        ADMIN_EXPENSIVE_READ_SITE_ID,
        ADMIN_EXPENSIVE_READ_LIMIT_PER_MINUTE,
        provider_registry,
    ) {
        return true;
    }
    if auth.method == Some(crate::admin::auth::AdminAuthMethod::SessionCookie) {
        let session_scope = auth
            .session_id
            .as_deref()
            .map(|session_id| format!("session:{}", session_id));
        if let Some(session_scope) = session_scope {
            if expensive_admin_read_limit_check_with_identity(
                store,
                session_scope.as_str(),
                ADMIN_EXPENSIVE_READ_SESSION_SITE_ID,
                ADMIN_EXPENSIVE_READ_SESSION_LIMIT_PER_MINUTE,
                provider_registry,
            ) {
                return true;
            }
        }
    }
    false
}

fn dashboard_refresh_session_scope(
    auth: &crate::admin::auth::AdminAuthResult,
) -> Option<String> {
    if auth.method != Some(crate::admin::auth::AdminAuthMethod::SessionCookie) {
        return None;
    }
    auth.session_id
        .as_deref()
        .map(|session_id| format!("dashboard-session:{session_id}"))
}

fn dashboard_refresh_is_limited(
    store: &Store,
    auth: &crate::admin::auth::AdminAuthResult,
    provider_registry: Option<&crate::providers::registry::ProviderRegistry>,
) -> bool {
    let Some(session_scope) = dashboard_refresh_session_scope(auth) else {
        return false;
    };
    expensive_admin_read_limit_check_with_identity(
        store,
        session_scope.as_str(),
        ADMIN_DASHBOARD_REFRESH_SESSION_SITE_ID,
        ADMIN_DASHBOARD_REFRESH_SESSION_LIMIT_PER_MINUTE,
        provider_registry,
    )
}

fn expensive_admin_read_limit_check(
    store: &Store,
    req: &Request,
    site_id: &str,
    limit_per_minute: u32,
    provider_registry: Option<&crate::providers::registry::ProviderRegistry>,
) -> bool {
    let ip = crate::extract_client_ip(req);
    expensive_admin_read_limit_check_with_identity(
        store,
        &ip,
        site_id,
        limit_per_minute,
        provider_registry,
    )
}

fn expensive_admin_read_limit_check_with_identity(
    store: &Store,
    identity: &str,
    site_id: &str,
    limit_per_minute: u32,
    provider_registry: Option<&crate::providers::registry::ProviderRegistry>,
) -> bool {
    if let Some(registry) = provider_registry {
        return registry.rate_limiter_provider().check_rate_limit(
            store,
            site_id,
            identity,
            limit_per_minute,
        ) == crate::providers::contracts::RateLimitDecision::Limited;
    }
    expensive_admin_read_limit_check_internal_with_identity(
        store,
        identity,
        site_id,
        limit_per_minute,
    )
}

fn expensive_admin_read_limit_check_internal_with_identity<S: crate::challenge::KeyValueStore>(
    store: &S,
    identity: &str,
    site_id: &str,
    limit_per_minute: u32,
) -> bool {
    !crate::enforcement::rate::check_rate_limit(
        store,
        site_id,
        identity,
        limit_per_minute,
    )
}

fn adversary_sim_control_submission_is_limited<S: crate::challenge::KeyValueStore>(
    store: &S,
    session_scope: &str,
    client_ip: &str,
) -> bool {
    expensive_admin_read_limit_check_internal_with_identity(
        store,
        session_scope,
        ADVERSARY_SIM_CONTROL_SESSION_SITE_ID,
        crate::admin::adversary_sim_control::CONTROL_SESSION_LIMIT_PER_MINUTE,
    ) || expensive_admin_read_limit_check_internal_with_identity(
        store,
        client_ip,
        ADVERSARY_SIM_CONTROL_IP_SITE_ID,
        crate::admin::adversary_sim_control::CONTROL_IP_LIMIT_PER_MINUTE,
    )
}

fn request_requires_admin_write(path: &str, method: &Method) -> bool {
    if !matches!(
        method,
        Method::Post | Method::Put | Method::Patch | Method::Delete
    ) {
        return false;
    }
    matches!(
        path,
        "/admin/ban"
            | "/admin/unban"
            | "/admin/config"
            | "/admin/config/validate"
            | "/admin/adversary-sim/control"
            | "/admin/adversary-sim/history/cleanup"
            | "/admin/maze/seeds"
            | "/admin/maze/seeds/refresh"
    )
}

fn parse_unban_identity(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("unknown") {
        return Some("unknown".to_string());
    }
    crate::request_validation::parse_ip_addr(trimmed)
}

fn log_admin_write_denied<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    path: &str,
    auth: &crate::admin::auth::AdminAuthResult,
) {
    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: None,
            reason: Some("admin_write_denied".to_string()),
            outcome: Some(format!(
                "path={} method={} access={}",
                path,
                req.method(),
                auth.access_label()
            )),
            admin: Some(auth.audit_actor_label().to_string()),
        },
    );
}

fn register_admin_auth_failure_with_selected_rate_limiter(
    store: &Store,
    req: &Request,
    scope: crate::admin::auth::AdminAuthFailureScope,
    provider_registry: Option<&crate::providers::registry::ProviderRegistry>,
) -> bool {
    if let Some(registry) = provider_registry {
        return crate::admin::auth::register_admin_auth_failure_with_provider(
            store, req, scope, registry,
        );
    }
    crate::admin::auth::register_admin_auth_failure(store, req, scope)
}

fn handle_admin_login_with_failure_handler<S, F>(
    req: &Request,
    store: &S,
    mut register_failure: F,
) -> Response
where
    S: crate::challenge::KeyValueStore,
    F: FnMut() -> bool,
{
    if req.method() != &spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let json = match crate::request_validation::parse_json_body(req.body(), 2048) {
        Ok(v) => v,
        Err(msg) => return Response::new(400, msg),
    };
    let Some(api_key) = json.get("api_key").and_then(|v| v.as_str()) else {
        return Response::new(400, "Bad Request: api_key is required");
    };

    if !crate::admin::auth::verify_admin_api_key_candidate(api_key) {
        if register_failure() {
            return too_many_admin_auth_attempts_response();
        }
        return Response::new(401, "Unauthorized");
    }

    let (session_id, csrf_token, expires_at) = match crate::admin::auth::create_admin_session(store)
    {
        Ok(v) => v,
        Err(_) => return Response::new(500, "Key-value store error"),
    };

    let body = serde_json::to_string(&json!({
        "authenticated": true,
        "csrf_token": csrf_token,
        "expires_at": expires_at
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("Set-Cookie", session_cookie_value(&session_id))
        .body(body)
        .build()
}

#[cfg(test)]
fn handle_admin_login<S: crate::challenge::KeyValueStore>(req: &Request, store: &S) -> Response {
    handle_admin_login_with_failure_handler(req, store, || {
        crate::admin::auth::register_admin_auth_failure(
            store,
            req,
            crate::admin::auth::AdminAuthFailureScope::Login,
        )
    })
}

fn handle_admin_session<S: crate::challenge::KeyValueStore>(req: &Request, store: &S) -> Response {
    if req.method() != &spin_sdk::http::Method::Get {
        return Response::new(405, "Method Not Allowed");
    }

    let auth = crate::admin::auth::authenticate_admin(req, store);
    let (authenticated, method, csrf_token, access, expires_at) = match auth.method {
        Some(crate::admin::auth::AdminAuthMethod::SessionCookie) => (
            true,
            "session",
            auth.csrf_token.clone(),
            crate::admin::auth::AdminAccessLevel::ReadWrite.as_str(),
            auth.session_expires_at,
        ),
        Some(crate::admin::auth::AdminAuthMethod::BearerToken) => {
            (true, "bearer", None, auth.access_label(), None)
        }
        None => (false, "none", None, "none", None),
    };
    let body = serde_json::to_string(&json!({
        "authenticated": authenticated,
        "method": method,
        "csrf_token": csrf_token,
        "access": access,
        "expires_at": expires_at
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

fn handle_admin_logout_with_failure_handler<S, F>(
    req: &Request,
    store: &S,
    mut register_failure: F,
) -> Response
where
    S: crate::challenge::KeyValueStore,
    F: FnMut() -> bool,
{
    if req.method() != &spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let auth = crate::admin::auth::authenticate_admin(req, store);
    if !auth.is_authorized() {
        if register_failure() {
            return too_many_admin_auth_attempts_response();
        }
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }
    if auth.requires_csrf(req) {
        let expected = auth.csrf_token.as_deref().unwrap_or("");
        if !crate::admin::auth::validate_session_csrf(req, expected) {
            log_admin_csrf_denied(store, req, "/admin/logout", &auth);
            return Response::new(403, "Forbidden");
        }
    }

    if let Err(e) = crate::admin::auth::clear_admin_session(store, req) {
        eprintln!("[admin] failed to clear admin session on logout: {:?}", e);
    }
    let body = serde_json::to_string(&json!({ "ok": true })).unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("Set-Cookie", clear_session_cookie_value())
        .body(body)
        .build()
}

#[cfg(test)]
fn handle_admin_logout<S: crate::challenge::KeyValueStore>(req: &Request, store: &S) -> Response {
    handle_admin_logout_with_failure_handler(req, store, || {
        crate::admin::auth::register_admin_auth_failure(
            store,
            req,
            crate::admin::auth::AdminAuthFailureScope::Endpoint,
        )
    })
}

fn query_u64_param(query: &str, key: &str, default: u64) -> u64 {
    query
        .split('&')
        .find_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let k = parts.next()?;
            let v = parts.next().unwrap_or("");
            if k == key {
                v.parse::<u64>().ok()
            } else {
                None
            }
        })
        .unwrap_or(default)
}

fn apply_robots_preview_patch(cfg: &mut crate::config::Config, json: &serde_json::Value) {
    let ai_policy_block_training = json
        .get("ai_policy_block_training")
        .and_then(|v| v.as_bool())
        .or_else(|| json.get("robots_block_ai_training").and_then(|v| v.as_bool()));
    if let Some(value) = ai_policy_block_training {
        cfg.robots_block_ai_training = value;
    }

    let ai_policy_block_search = json
        .get("ai_policy_block_search")
        .and_then(|v| v.as_bool())
        .or_else(|| json.get("robots_block_ai_search").and_then(|v| v.as_bool()));
    if let Some(value) = ai_policy_block_search {
        cfg.robots_block_ai_search = value;
    }

    let ai_policy_allow_search_engines = json
        .get("ai_policy_allow_search_engines")
        .and_then(|v| v.as_bool())
        .or_else(|| json.get("robots_allow_search_engines").and_then(|v| v.as_bool()));
    if let Some(value) = ai_policy_allow_search_engines {
        cfg.robots_allow_search_engines = value;
    }

    if let Some(robots_enabled) = json.get("robots_enabled").and_then(|v| v.as_bool()) {
        cfg.robots_enabled = robots_enabled;
    }

    if let Some(robots_crawl_delay) = json.get("robots_crawl_delay").and_then(|v| v.as_u64()) {
        cfg.robots_crawl_delay = robots_crawl_delay.clamp(0, 60) as u32;
    }
}

fn admin_robots_payload(cfg: &crate::config::Config) -> serde_json::Value {
    let preview = crate::crawler_policy::robots::generate_robots_txt(cfg);
    let content_signal = crate::crawler_policy::robots::get_content_signal_header(cfg);
    json!({
        "config": {
            "enabled": cfg.robots_enabled,
            "ai_policy_block_training": cfg.robots_block_ai_training,
            "ai_policy_block_search": cfg.robots_block_ai_search,
            "ai_policy_allow_search_engines": cfg.robots_allow_search_engines,
            "block_ai_training": cfg.robots_block_ai_training,
            "block_ai_search": cfg.robots_block_ai_search,
            "allow_search_engines": cfg.robots_allow_search_engines,
            "crawl_delay": cfg.robots_crawl_delay
        },
        "content_signal_header": content_signal,
        "ai_training_bots": crate::crawler_policy::robots::AI_TRAINING_BOTS,
        "ai_search_bots": crate::crawler_policy::robots::AI_SEARCH_BOTS,
        "search_engine_bots": crate::crawler_policy::robots::SEARCH_ENGINE_BOTS,
        "preview": preview
    })
}

fn admin_robots_response(cfg: &crate::config::Config) -> Response {
    let body = serde_json::to_string(&admin_robots_payload(cfg)).unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

fn load_recent_events<S: crate::challenge::KeyValueStore>(
    store: &S,
    now: u64,
    hours: u64,
) -> Vec<EventLogEntry> {
    load_recent_event_records(store, now, hours)
        .into_iter()
        .map(|record| record.entry)
        .collect()
}

#[derive(Debug, Clone)]
struct StoredEventLogRecord {
    storage_key: String,
    record: EventLogRecord,
}

#[derive(Debug, Clone)]
struct CursorEventRecord {
    cursor: String,
    record: EventLogRecord,
}

fn build_event_cursor(ts: u64, storage_key: &str) -> String {
    format!("{:020}|{}", ts, storage_key)
}

fn cursor_event_row_payload(row: &CursorEventRecord) -> serde_json::Value {
    let mut payload = serde_json::to_value(&row.record).unwrap_or_else(|_| json!({}));
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(
            "cursor".to_string(),
            serde_json::Value::String(row.cursor.clone()),
        );
    }
    payload
}

fn validate_after_cursor(raw_cursor: &str) -> Result<(), String> {
    if raw_cursor.len() > 512 {
        return Err("after_cursor must be <= 512 chars".to_string());
    }
    if raw_cursor.contains('\n') || raw_cursor.contains('\r') {
        return Err("after_cursor must not contain newlines".to_string());
    }
    Ok(())
}

fn delta_page_etag(next_cursor: &str, count: usize, has_more: bool, overflow: &str) -> String {
    let signature = format!("{}|{}|{}|{}", next_cursor, count, has_more, overflow);
    let digest = crate::admin::adversary_sim_control::hash_hex(signature.as_str());
    format!("\"{}\"", digest)
}

fn request_if_none_match(req: &Request) -> Option<String> {
    req.header("if-none-match")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
}

fn request_last_event_id(req: &Request) -> Option<String> {
    req.header("last-event-id")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
}

fn resolve_after_cursor(req: &Request) -> String {
    let query_cursor =
        crate::request_validation::query_param(req.query(), "after_cursor").unwrap_or_default();
    if !query_cursor.trim().is_empty() {
        return query_cursor;
    }
    request_last_event_id(req).unwrap_or_default()
}

fn latest_event_ts(rows: &[CursorEventRecord]) -> Option<u64> {
    rows.iter().map(|row| row.record.entry.ts).max()
}

fn latest_monitoring_snapshot_ts(details: &serde_json::Value) -> Option<u64> {
    let event_ts = details
        .get("events")
        .and_then(|value| value.get("recent_events"))
        .and_then(|value| value.as_array())
        .and_then(|rows| {
            rows.iter()
                .filter_map(|row| row.get("ts").and_then(|value| value.as_u64()))
                .max()
        });
    let ban_ts = details
        .get("bans")
        .and_then(|value| value.get("bans"))
        .and_then(|value| value.as_array())
        .and_then(|rows| {
            rows.iter()
                .filter_map(|row| row.get("banned_at").and_then(|value| value.as_u64()))
                .max()
        });
    match (event_ts, ban_ts) {
        (Some(event_ts), Some(ban_ts)) => Some(event_ts.max(ban_ts)),
        (Some(event_ts), None) => Some(event_ts),
        (None, Some(ban_ts)) => Some(ban_ts),
        (None, None) => None,
    }
}

fn freshness_state_for_lag(lag_ms: Option<u64>) -> &'static str {
    let Some(lag_ms) = lag_ms else {
        return "stale";
    };
    if lag_ms <= MONITORING_MAX_ALLOWED_LAG_BEFORE_DEGRADED_MS {
        return "fresh";
    }
    if lag_ms <= MONITORING_STALE_LAG_THRESHOLD_MS {
        return "degraded";
    }
    "stale"
}

fn freshness_health_payload(
    now_ts: u64,
    latest_event_ts: Option<u64>,
    has_more: bool,
    overflow: &str,
    transport: &str,
) -> serde_json::Value {
    let lag_ms = latest_event_ts.map(|event_ts| now_ts.saturating_sub(event_ts).saturating_mul(1000));
    let state = freshness_state_for_lag(lag_ms);
    let slow_consumer_lag_state = if has_more || overflow == "limit_exceeded" {
        "lagged"
    } else {
        "normal"
    };
    json!({
        "state": state,
        "now_ts": now_ts,
        "last_event_ts": latest_event_ts,
        "lag_ms": lag_ms,
        "manual_refresh_staleness_bound_ms": MONITORING_MANUAL_REFRESH_STALENESS_BOUND_MS,
        "max_allowed_lag_before_degraded_ms": MONITORING_MAX_ALLOWED_LAG_BEFORE_DEGRADED_MS,
        "stale_lag_threshold_ms": MONITORING_STALE_LAG_THRESHOLD_MS,
        "slow_consumer_lag_state": slow_consumer_lag_state,
        "overflow": overflow,
        "transport": transport
    })
}

fn freshness_slo_payload() -> serde_json::Value {
    json!({
        "visibility_delay_ms": {
            "p50_target": MONITORING_FRESHNESS_SLO_P50_MS,
            "p95_target": MONITORING_FRESHNESS_SLO_P95_MS,
            "p99_target": MONITORING_FRESHNESS_SLO_P99_MS
        },
        "manual_refresh_staleness_bound_ms": MONITORING_MANUAL_REFRESH_STALENESS_BOUND_MS,
        "max_allowed_lag_before_degraded_ms": MONITORING_MAX_ALLOWED_LAG_BEFORE_DEGRADED_MS
    })
}

fn load_envelope_payload() -> serde_json::Value {
    json!({
        "event_ingest_rate_events_per_second": MONITORING_LOAD_ENVELOPE_EVENTS_PER_SEC,
        "operator_refresh_clients": MONITORING_LOAD_ENVELOPE_OPERATOR_CLIENTS,
        "query_cost_ceiling_per_minute": MONITORING_LOAD_ENVELOPE_QUERY_COST_CEILING_PER_MINUTE,
        "query_budget_requests_per_second_per_client": MONITORING_QUERY_BUDGET_REQUESTS_PER_SECOND
    })
}

fn stream_contract_payload() -> serde_json::Value {
    json!({
        "type": "one_shot_sse",
        "retry_ms": MONITORING_STREAM_RETRY_MS,
        "max_buffer_events": MONITORING_STREAM_MAX_BUFFER_EVENTS,
        "slow_consumer_lag_state_taxonomy": ["normal", "lagged"]
    })
}

#[derive(Debug, Clone)]
struct MonitoringQueryBudget {
    cost_units: u64,
    cost_class: &'static str,
    avg_req_per_sec_client: f64,
    max_req_per_sec_client: f64,
    status: &'static str,
}

#[derive(Debug, Clone)]
struct MonitoringCompressionReport {
    negotiated: bool,
    algorithm: &'static str,
    status: &'static str,
    reduction_percent: f64,
    input_bytes: usize,
    output_bytes: usize,
}

fn monitoring_query_budget(hours: u64, limit: usize) -> MonitoringQueryBudget {
    let cost_units = hours.saturating_mul(limit as u64);
    if cost_units <= MONITORING_QUERY_BUDGET_STANDARD_MAX_COST_UNITS {
        return MonitoringQueryBudget {
            cost_units,
            cost_class: "standard",
            avg_req_per_sec_client: 0.5,
            max_req_per_sec_client: MONITORING_QUERY_BUDGET_REQUESTS_PER_SECOND as f64,
            status: "within_budget",
        };
    }
    if cost_units <= MONITORING_QUERY_BUDGET_ELEVATED_MAX_COST_UNITS {
        return MonitoringQueryBudget {
            cost_units,
            cost_class: "elevated",
            avg_req_per_sec_client: 0.75,
            max_req_per_sec_client: MONITORING_QUERY_BUDGET_REQUESTS_PER_SECOND as f64,
            status: "within_budget",
        };
    }
    if cost_units <= MONITORING_QUERY_BUDGET_HEAVY_MAX_COST_UNITS {
        return MonitoringQueryBudget {
            cost_units,
            cost_class: "heavy",
            avg_req_per_sec_client: MONITORING_QUERY_BUDGET_REQUESTS_PER_SECOND as f64,
            max_req_per_sec_client: MONITORING_QUERY_BUDGET_REQUESTS_PER_SECOND as f64,
            status: "within_budget",
        };
    }
    MonitoringQueryBudget {
        cost_units,
        cost_class: "exceeded",
        avg_req_per_sec_client: 1.25,
        max_req_per_sec_client: MONITORING_QUERY_BUDGET_REQUESTS_PER_SECOND as f64,
        status: "exceeded",
    }
}

fn request_accepts_gzip(req: &Request) -> bool {
    let Some(value) = req
        .header("accept-encoding")
        .and_then(|header| header.as_str())
    else {
        return false;
    };
    for token in value.to_ascii_lowercase().split(',') {
        let mut parts = token.trim().split(';');
        let encoding = parts.next().unwrap_or("").trim();
        if encoding != "gzip" {
            continue;
        }
        let mut quality = 1.0f64;
        for part in parts {
            let trimmed = part.trim();
            if let Some(raw) = trimmed.strip_prefix("q=") {
                if let Ok(parsed) = raw.parse::<f64>() {
                    quality = parsed;
                }
            }
        }
        if quality > 0.0 {
            return true;
        }
    }
    false
}

fn gzip_bytes(payload: &[u8]) -> Option<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    if encoder.write_all(payload).is_err() {
        return None;
    }
    encoder.finish().ok()
}

fn monitoring_compression_report(payload: &[u8], supports_gzip: bool) -> MonitoringCompressionReport {
    if payload.len() <= MONITORING_COMPRESSION_MIN_PAYLOAD_BYTES {
        return MonitoringCompressionReport {
            negotiated: false,
            algorithm: "none",
            status: "not_required",
            reduction_percent: 0.0,
            input_bytes: payload.len(),
            output_bytes: payload.len(),
        };
    }
    if !supports_gzip {
        return MonitoringCompressionReport {
            negotiated: false,
            algorithm: "none",
            status: "not_negotiated",
            reduction_percent: 0.0,
            input_bytes: payload.len(),
            output_bytes: payload.len(),
        };
    }
    let Some(compressed) = gzip_bytes(payload) else {
        return MonitoringCompressionReport {
            negotiated: false,
            algorithm: "none",
            status: "compression_error",
            reduction_percent: 0.0,
            input_bytes: payload.len(),
            output_bytes: payload.len(),
        };
    };
    let input = payload.len();
    let output = compressed.len().max(1);
    let reduction_percent = ((input.saturating_sub(output) as f64) / input as f64) * 100.0;
    let status = if reduction_percent >= MONITORING_COMPRESSION_MIN_REDUCTION_PERCENT {
        "effective"
    } else {
        "below_target"
    };
    MonitoringCompressionReport {
        negotiated: true,
        algorithm: "gzip",
        status,
        reduction_percent,
        input_bytes: input,
        output_bytes: output,
    }
}

fn update_monitoring_cost_governance_transport_fields(
    payload: &mut serde_json::Value,
    payload_kb: f64,
    compression: &MonitoringCompressionReport,
) {
    let payload_status = if payload_kb <= MONITORING_PAYLOAD_BUDGET_P95_KB {
        "within_budget"
    } else {
        "exceeded"
    };
    let mut degraded_reasons: Vec<&str> = Vec::new();
    if payload_status == "exceeded" {
        degraded_reasons.push("payload_budget_exceeded");
    }
    if compression.status == "below_target" || compression.status == "compression_error" {
        degraded_reasons.push("compression_effectiveness_below_target");
    }

    let cost_governance = payload
        .get_mut("details")
        .and_then(|details| details.get_mut("cost_governance"))
        .and_then(|value| value.as_object_mut());
    let Some(cost_governance) = cost_governance else {
        return;
    };

    if let Some(query_budget_obj) = cost_governance
        .get("query_budget")
        .and_then(|value| value.as_object())
    {
        let query_status = query_budget_obj
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or("within_budget");
        if query_status == "exceeded" {
            degraded_reasons.push("query_budget_exceeded");
        }
        cost_governance.insert(
            "query_budget_status".to_string(),
            serde_json::Value::from(query_status),
        );
    }

    if let Some(payload_budget_obj) = cost_governance
        .entry("payload_budget".to_string())
        .or_insert_with(|| json!({}))
        .as_object_mut()
    {
        payload_budget_obj.insert(
            "p95_max_kb".to_string(),
            serde_json::Value::from(MONITORING_PAYLOAD_BUDGET_P95_KB),
        );
        payload_budget_obj.insert(
            "estimated_current_payload_kb".to_string(),
            serde_json::Value::from(payload_kb),
        );
        payload_budget_obj.insert(
            "status".to_string(),
            serde_json::Value::from(payload_status),
        );
    }
    cost_governance.insert(
        "payload_budget_status".to_string(),
        serde_json::Value::from(payload_status),
    );

    if let Some(compression_obj) = cost_governance
        .entry("compression".to_string())
        .or_insert_with(|| json!({}))
        .as_object_mut()
    {
        compression_obj.insert(
            "status".to_string(),
            serde_json::Value::from(compression.status),
        );
        compression_obj.insert(
            "negotiated".to_string(),
            serde_json::Value::from(compression.negotiated),
        );
        compression_obj.insert(
            "algorithm".to_string(),
            serde_json::Value::from(compression.algorithm),
        );
        compression_obj.insert(
            "input_bytes".to_string(),
            serde_json::Value::from(compression.input_bytes as u64),
        );
        compression_obj.insert(
            "output_bytes".to_string(),
            serde_json::Value::from(compression.output_bytes as u64),
        );
        compression_obj.insert(
            "reduction_percent".to_string(),
            serde_json::Value::from(compression.reduction_percent),
        );
        compression_obj.insert(
            "min_percent".to_string(),
            serde_json::Value::from(MONITORING_COMPRESSION_MIN_REDUCTION_PERCENT),
        );
    }

    let degraded_state = if degraded_reasons.is_empty() {
        "normal"
    } else {
        "degraded"
    };
    cost_governance.insert(
        "degraded_state".to_string(),
        serde_json::Value::from(degraded_state),
    );
    cost_governance.insert(
        "degraded_reasons".to_string(),
        serde_json::Value::Array(
            degraded_reasons
                .into_iter()
                .map(serde_json::Value::from)
                .collect(),
        ),
    );
}

fn sse_single_event_response(event_name: &str, event_id: &str, payload: &serde_json::Value) -> Response {
    let event_payload = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());
    let body = format!(
        "retry: {}\nevent: {}\nid: {}\ndata: {}\n\n",
        MONITORING_STREAM_RETRY_MS, event_name, event_id, event_payload
    );
    Response::builder()
        .status(200)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-store")
        .header("X-Accel-Buffering", "no")
        .body(body)
        .build()
}

fn paginate_cursor_rows(
    mut rows: Vec<CursorEventRecord>,
    after_cursor: &str,
    limit: usize,
) -> (Vec<CursorEventRecord>, String, bool, &'static str) {
    rows.sort_by(|a, b| a.cursor.cmp(&b.cursor));
    let mut filtered: Vec<CursorEventRecord> = rows
        .into_iter()
        .filter(|row| after_cursor.is_empty() || row.cursor.as_str() > after_cursor)
        .collect();
    let has_more = filtered.len() > limit;
    if has_more {
        filtered.truncate(limit);
    }
    let next_cursor = filtered
        .last()
        .map(|row| row.cursor.clone())
        .unwrap_or_else(|| after_cursor.to_string());
    let overflow = if has_more { "limit_exceeded" } else { "none" };
    (filtered, next_cursor, has_more, overflow)
}

fn load_recent_event_records_with_keys<S: crate::challenge::KeyValueStore>(
    store: &S,
    now: u64,
    hours: u64,
) -> Vec<StoredEventLogRecord> {
    let hours = effective_event_log_query_hours(hours);
    let now_hour = now / 3600;
    let mut events: Vec<StoredEventLogRecord> = Vec::new();
    let window_start = now.saturating_sub(hours.saturating_mul(3600));
    let window_start_hour = window_start / 3600;

    if let Ok(keys) = store.get_keys() {
        for key in keys {
            let Some(event_hour) = parse_v2_event_key(&key) else {
                continue;
            };
            if event_hour < window_start_hour || event_hour > now_hour {
                continue;
            }
            if let Ok(Some(val)) = store.get(&key) {
                let parsed_record = serde_json::from_slice::<EventLogRecord>(&val)
                    .ok()
                    .or_else(|| {
                        serde_json::from_slice::<EventLogEntry>(&val)
                            .ok()
                            .map(EventLogRecord::from_entry)
                    });
                let Some(record) = parsed_record else {
                    continue;
                };
                if record.entry.ts < window_start {
                    continue;
                }
                events.push(StoredEventLogRecord {
                    storage_key: key,
                    record,
                });
            }
        }
    }

    events
}

fn load_recent_event_records<S: crate::challenge::KeyValueStore>(
    store: &S,
    now: u64,
    hours: u64,
) -> Vec<EventLogRecord> {
    load_recent_event_records_with_keys(store, now, hours)
        .into_iter()
        .map(|stored| stored.record)
        .collect()
}

fn is_cdp_event_reason(reason: &str) -> bool {
    let lowered = reason.to_lowercase();
    lowered.starts_with("cdp_detected:") || lowered == "cdp_automation"
}

fn challenge_threshold_default() -> u8 {
    crate::config::defaults().challenge_puzzle_risk_threshold
}

fn not_a_bot_threshold_default() -> u8 {
    crate::config::defaults().not_a_bot_risk_threshold
}

fn maze_threshold_default() -> u8 {
    crate::config::defaults().botness_maze_threshold
}

fn botness_signal_definitions(cfg: &crate::config::Config) -> serde_json::Value {
    json!({
        "scored_signals": [
            {
                "key": "js_verification_required",
                "label": "JS verification required",
                "weight": cfg.botness_weights.js_required
            },
            {
                "key": "browser_outdated",
                "label": "Browser policy minimum-version match",
                "weight": 1
            },
            {
                "key": "geo_risk",
                "label": "High-risk geography",
                "weight": cfg.botness_weights.geo_risk
            },
            {
                "key": "rate_pressure_medium",
                "label": "Rate pressure (>=50%)",
                "weight": cfg.botness_weights.rate_medium
            },
            {
                "key": "rate_pressure_high",
                "label": "Rate pressure (>=80%)",
                "weight": cfg.botness_weights.rate_high
            },
            {
                "key": "fp_ua_ch_mismatch",
                "label": "Fingerprint UA/client-hint mismatch",
                "weight": 2
            },
            {
                "key": "fp_ua_transport_mismatch",
                "label": "Fingerprint UA/transport mismatch",
                "weight": 3
            },
            {
                "key": "fp_temporal_transition",
                "label": "Fingerprint impossible temporal transition",
                "weight": 2
            },
            {
                "key": "fp_flow_violation",
                "label": "Fingerprint flow-window violation",
                "weight": 2
            },
            {
                "key": "fp_persistence_marker_missing",
                "label": "Fingerprint persistence-marker missing",
                "weight": 1
            },
            {
                "key": "fp_untrusted_transport_header",
                "label": "Fingerprint untrusted transport header",
                "weight": 3
            },
            {
                "key": "fp_akamai_edge_additive",
                "label": "Fingerprint Akamai edge signal (additive)",
                "weight": 2
            }
        ],
        "terminal_signals": [
            { "key": "honeypot", "label": "Honeypot hit", "action": "Immediate ban" },
            { "key": "rate_limit_exceeded", "label": "Rate limit exceeded", "action": "Immediate ban" },
            { "key": "cdp_automation", "label": "CDP automation detected", "action": "Immediate ban (if enabled)" },
            { "key": "maze_crawler_threshold", "label": "Maze crawler threshold reached", "action": "Immediate ban (if enabled)" },
            { "key": "already_banned", "label": "Existing active ban", "action": "Block page" }
        ]
    })
}

fn bool_env(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn json_env<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

fn config_export_env_entries(cfg: &crate::config::Config) -> Vec<(String, String)> {
    let frontier = crate::config::frontier_summary();
    let frontier_model = |provider_name: &str| -> String {
        frontier
            .providers
            .iter()
            .find(|provider| provider.provider == provider_name)
            .map(|provider| provider.model_id.clone())
            .unwrap_or_default()
    };

    vec![
        (
            "SHUMA_ADMIN_IP_ALLOWLIST".to_string(),
            std::env::var("SHUMA_ADMIN_IP_ALLOWLIST").unwrap_or_default(),
        ),
        (
            "SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE".to_string(),
            crate::admin::auth::admin_auth_failure_limit_per_minute().to_string(),
        ),
        (
            "SHUMA_EVENT_LOG_RETENTION_HOURS".to_string(),
            crate::config::event_log_retention_hours().to_string(),
        ),
        (
            "SHUMA_ADMIN_CONFIG_WRITE_ENABLED".to_string(),
            bool_env(crate::config::admin_config_write_enabled()).to_string(),
        ),
        (
            "SHUMA_KV_STORE_FAIL_OPEN".to_string(),
            bool_env(crate::config::kv_store_fail_open()).to_string(),
        ),
        (
            "SHUMA_ENFORCE_HTTPS".to_string(),
            bool_env(crate::config::https_enforced()).to_string(),
        ),
        (
            "SHUMA_DEBUG_HEADERS".to_string(),
            bool_env(crate::config::debug_headers_enabled()).to_string(),
        ),
        (
            "SHUMA_RUNTIME_ENV".to_string(),
            crate::config::runtime_environment().as_str().to_string(),
        ),
        (
            "SHUMA_ADVERSARY_SIM_AVAILABLE".to_string(),
            bool_env(crate::config::adversary_sim_available()).to_string(),
        ),
        (
            "SHUMA_FRONTIER_OPENAI_MODEL".to_string(),
            frontier_model("openai"),
        ),
        (
            "SHUMA_FRONTIER_ANTHROPIC_MODEL".to_string(),
            frontier_model("anthropic"),
        ),
        (
            "SHUMA_FRONTIER_GOOGLE_MODEL".to_string(),
            frontier_model("google"),
        ),
        (
            "SHUMA_FRONTIER_XAI_MODEL".to_string(),
            frontier_model("xai"),
        ),
        (
            "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN".to_string(),
            crate::config::rate_limiter_outage_mode_main()
                .as_str()
                .to_string(),
        ),
        (
            "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH".to_string(),
            crate::config::rate_limiter_outage_mode_admin_auth()
                .as_str()
                .to_string(),
        ),
        (
            "SHUMA_TEST_MODE".to_string(),
            bool_env(cfg.test_mode).to_string(),
        ),
        (
            "SHUMA_ADVERSARY_SIM_ENABLED".to_string(),
            bool_env(cfg.adversary_sim_enabled).to_string(),
        ),
        (
            "SHUMA_ADVERSARY_SIM_DURATION_SECONDS".to_string(),
            cfg.adversary_sim_duration_seconds.to_string(),
        ),
        (
            "SHUMA_JS_REQUIRED_ENFORCED".to_string(),
            bool_env(cfg.js_required_enforced).to_string(),
        ),
        (
            "SHUMA_MODE_RATE".to_string(),
            cfg.defence_modes.rate.as_str().to_string(),
        ),
        (
            "SHUMA_MODE_GEO".to_string(),
            cfg.defence_modes.geo.as_str().to_string(),
        ),
        (
            "SHUMA_MODE_JS".to_string(),
            cfg.defence_modes.js.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_RATE_LIMITER".to_string(),
            cfg.provider_backends.rate_limiter.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_BAN_STORE".to_string(),
            cfg.provider_backends.ban_store.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_CHALLENGE_ENGINE".to_string(),
            cfg.provider_backends.challenge_engine.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_MAZE_TARPIT".to_string(),
            cfg.provider_backends.maze_tarpit.as_str().to_string(),
        ),
        (
            "SHUMA_PROVIDER_FINGERPRINT_SIGNAL".to_string(),
            cfg.provider_backends
                .fingerprint_signal
                .as_str()
                .to_string(),
        ),
        (
            "SHUMA_EDGE_INTEGRATION_MODE".to_string(),
            cfg.edge_integration_mode.as_str().to_string(),
        ),
        (
            "SHUMA_POW_ENABLED".to_string(),
            bool_env(cfg.pow_enabled).to_string(),
        ),
        (
            "SHUMA_POW_DIFFICULTY".to_string(),
            cfg.pow_difficulty.to_string(),
        ),
        (
            "SHUMA_POW_TTL_SECONDS".to_string(),
            cfg.pow_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_CHALLENGE_PUZZLE_ENABLED".to_string(),
            bool_env(cfg.challenge_puzzle_enabled).to_string(),
        ),
        (
            "SHUMA_CHALLENGE_PUZZLE_TRANSFORM_COUNT".to_string(),
            cfg.challenge_puzzle_transform_count.to_string(),
        ),
        (
            "SHUMA_CHALLENGE_PUZZLE_SEED_TTL_SECONDS".to_string(),
            cfg.challenge_puzzle_seed_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_CHALLENGE_PUZZLE_ATTEMPT_LIMIT_PER_WINDOW".to_string(),
            cfg.challenge_puzzle_attempt_limit_per_window.to_string(),
        ),
        (
            "SHUMA_CHALLENGE_PUZZLE_ATTEMPT_WINDOW_SECONDS".to_string(),
            cfg.challenge_puzzle_attempt_window_seconds.to_string(),
        ),
        (
            "SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD".to_string(),
            cfg.challenge_puzzle_risk_threshold.to_string(),
        ),
        (
            "SHUMA_NOT_A_BOT_ENABLED".to_string(),
            bool_env(cfg.not_a_bot_enabled).to_string(),
        ),
        (
            "SHUMA_NOT_A_BOT_RISK_THRESHOLD".to_string(),
            cfg.not_a_bot_risk_threshold.to_string(),
        ),
        (
            "SHUMA_NOT_A_BOT_PASS_SCORE".to_string(),
            cfg.not_a_bot_pass_score.to_string(),
        ),
        (
            "SHUMA_NOT_A_BOT_FAIL_SCORE".to_string(),
            cfg.not_a_bot_fail_score.to_string(),
        ),
        (
            "SHUMA_NOT_A_BOT_NONCE_TTL_SECONDS".to_string(),
            cfg.not_a_bot_nonce_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_NOT_A_BOT_MARKER_TTL_SECONDS".to_string(),
            cfg.not_a_bot_marker_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_NOT_A_BOT_ATTEMPT_LIMIT_PER_WINDOW".to_string(),
            cfg.not_a_bot_attempt_limit_per_window.to_string(),
        ),
        (
            "SHUMA_NOT_A_BOT_ATTEMPT_WINDOW_SECONDS".to_string(),
            cfg.not_a_bot_attempt_window_seconds.to_string(),
        ),
        (
            "SHUMA_BOTNESS_MAZE_THRESHOLD".to_string(),
            cfg.botness_maze_threshold.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_JS_REQUIRED".to_string(),
            cfg.botness_weights.js_required.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_GEO_RISK".to_string(),
            cfg.botness_weights.geo_risk.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM".to_string(),
            cfg.botness_weights.rate_medium.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_RATE_HIGH".to_string(),
            cfg.botness_weights.rate_high.to_string(),
        ),
        (
            "SHUMA_BOTNESS_WEIGHT_MAZE_BEHAVIOR".to_string(),
            cfg.botness_weights.maze_behavior.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION".to_string(),
            cfg.ban_duration.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION_HONEYPOT".to_string(),
            cfg.ban_durations.honeypot.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION_RATE_LIMIT".to_string(),
            cfg.ban_durations.rate_limit.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION_ADMIN".to_string(),
            cfg.ban_durations.admin.to_string(),
        ),
        (
            "SHUMA_BAN_DURATION_CDP".to_string(),
            cfg.ban_durations.cdp.to_string(),
        ),
        ("SHUMA_RATE_LIMIT".to_string(), cfg.rate_limit.to_string()),
        (
            "SHUMA_HONEYPOT_ENABLED".to_string(),
            bool_env(cfg.honeypot_enabled).to_string(),
        ),
        ("SHUMA_HONEYPOTS".to_string(), json_env(&cfg.honeypots)),
        (
            "SHUMA_BROWSER_POLICY_ENABLED".to_string(),
            bool_env(cfg.browser_policy_enabled).to_string(),
        ),
        (
            "SHUMA_BROWSER_BLOCK".to_string(),
            json_env(&cfg.browser_block),
        ),
        (
            "SHUMA_BROWSER_ALLOWLIST".to_string(),
            json_env(&cfg.browser_allowlist),
        ),
        (
            "SHUMA_GEO_RISK_COUNTRIES".to_string(),
            json_env(&cfg.geo_risk),
        ),
        (
            "SHUMA_GEO_ALLOW_COUNTRIES".to_string(),
            json_env(&cfg.geo_allow),
        ),
        (
            "SHUMA_GEO_CHALLENGE_COUNTRIES".to_string(),
            json_env(&cfg.geo_challenge),
        ),
        (
            "SHUMA_GEO_MAZE_COUNTRIES".to_string(),
            json_env(&cfg.geo_maze),
        ),
        (
            "SHUMA_GEO_BLOCK_COUNTRIES".to_string(),
            json_env(&cfg.geo_block),
        ),
        (
            "SHUMA_GEO_EDGE_HEADERS_ENABLED".to_string(),
            bool_env(cfg.geo_edge_headers_enabled).to_string(),
        ),
        (
            "SHUMA_BYPASS_ALLOWLISTS_ENABLED".to_string(),
            bool_env(cfg.bypass_allowlists_enabled).to_string(),
        ),
        ("SHUMA_ALLOWLIST".to_string(), json_env(&cfg.allowlist)),
        (
            "SHUMA_PATH_ALLOWLIST_ENABLED".to_string(),
            bool_env(cfg.path_allowlist_enabled).to_string(),
        ),
        (
            "SHUMA_PATH_ALLOWLIST".to_string(),
            json_env(&cfg.path_allowlist),
        ),
        (
            "SHUMA_IP_RANGE_POLICY_MODE".to_string(),
            cfg.ip_range_policy_mode.as_str().to_string(),
        ),
        (
            "SHUMA_IP_RANGE_EMERGENCY_ALLOWLIST".to_string(),
            json_env(&cfg.ip_range_emergency_allowlist),
        ),
        (
            "SHUMA_IP_RANGE_CUSTOM_RULES".to_string(),
            json_env(&cfg.ip_range_custom_rules),
        ),
        (
            "SHUMA_MAZE_ENABLED".to_string(),
            bool_env(cfg.maze_enabled).to_string(),
        ),
        (
            "SHUMA_TARPIT_ENABLED".to_string(),
            bool_env(cfg.tarpit_enabled).to_string(),
        ),
        (
            "SHUMA_TARPIT_PROGRESS_TOKEN_TTL_SECONDS".to_string(),
            cfg.tarpit_progress_token_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_TARPIT_PROGRESS_REPLAY_TTL_SECONDS".to_string(),
            cfg.tarpit_progress_replay_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_TARPIT_HASHCASH_MIN_DIFFICULTY".to_string(),
            cfg.tarpit_hashcash_min_difficulty.to_string(),
        ),
        (
            "SHUMA_TARPIT_HASHCASH_MAX_DIFFICULTY".to_string(),
            cfg.tarpit_hashcash_max_difficulty.to_string(),
        ),
        (
            "SHUMA_TARPIT_HASHCASH_BASE_DIFFICULTY".to_string(),
            cfg.tarpit_hashcash_base_difficulty.to_string(),
        ),
        (
            "SHUMA_TARPIT_HASHCASH_ADAPTIVE".to_string(),
            bool_env(cfg.tarpit_hashcash_adaptive).to_string(),
        ),
        (
            "SHUMA_TARPIT_STEP_CHUNK_BASE_BYTES".to_string(),
            cfg.tarpit_step_chunk_base_bytes.to_string(),
        ),
        (
            "SHUMA_TARPIT_STEP_CHUNK_MAX_BYTES".to_string(),
            cfg.tarpit_step_chunk_max_bytes.to_string(),
        ),
        (
            "SHUMA_TARPIT_STEP_JITTER_PERCENT".to_string(),
            cfg.tarpit_step_jitter_percent.to_string(),
        ),
        (
            "SHUMA_TARPIT_SHARD_ROTATION_ENABLED".to_string(),
            bool_env(cfg.tarpit_shard_rotation_enabled).to_string(),
        ),
        (
            "SHUMA_TARPIT_EGRESS_WINDOW_SECONDS".to_string(),
            cfg.tarpit_egress_window_seconds.to_string(),
        ),
        (
            "SHUMA_TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW".to_string(),
            cfg.tarpit_egress_global_bytes_per_window.to_string(),
        ),
        (
            "SHUMA_TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW".to_string(),
            cfg.tarpit_egress_per_ip_bucket_bytes_per_window.to_string(),
        ),
        (
            "SHUMA_TARPIT_EGRESS_PER_FLOW_MAX_BYTES".to_string(),
            cfg.tarpit_egress_per_flow_max_bytes.to_string(),
        ),
        (
            "SHUMA_TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS".to_string(),
            cfg.tarpit_egress_per_flow_max_duration_seconds.to_string(),
        ),
        (
            "SHUMA_TARPIT_MAX_CONCURRENT_GLOBAL".to_string(),
            cfg.tarpit_max_concurrent_global.to_string(),
        ),
        (
            "SHUMA_TARPIT_MAX_CONCURRENT_PER_IP_BUCKET".to_string(),
            cfg.tarpit_max_concurrent_per_ip_bucket.to_string(),
        ),
        (
            "SHUMA_TARPIT_FALLBACK_ACTION".to_string(),
            cfg.tarpit_fallback_action.as_str().to_string(),
        ),
        (
            "SHUMA_MAZE_AUTO_BAN".to_string(),
            bool_env(cfg.maze_auto_ban).to_string(),
        ),
        (
            "SHUMA_MAZE_AUTO_BAN_THRESHOLD".to_string(),
            cfg.maze_auto_ban_threshold.to_string(),
        ),
        (
            "SHUMA_MAZE_ROLLOUT_PHASE".to_string(),
            cfg.maze_rollout_phase.as_str().to_string(),
        ),
        (
            "SHUMA_MAZE_TOKEN_TTL_SECONDS".to_string(),
            cfg.maze_token_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_MAZE_TOKEN_MAX_DEPTH".to_string(),
            cfg.maze_token_max_depth.to_string(),
        ),
        (
            "SHUMA_MAZE_TOKEN_BRANCH_BUDGET".to_string(),
            cfg.maze_token_branch_budget.to_string(),
        ),
        (
            "SHUMA_MAZE_REPLAY_TTL_SECONDS".to_string(),
            cfg.maze_replay_ttl_seconds.to_string(),
        ),
        (
            "SHUMA_MAZE_ENTROPY_WINDOW_SECONDS".to_string(),
            cfg.maze_entropy_window_seconds.to_string(),
        ),
        (
            "SHUMA_MAZE_CLIENT_EXPANSION_ENABLED".to_string(),
            bool_env(cfg.maze_client_expansion_enabled).to_string(),
        ),
        (
            "SHUMA_MAZE_CHECKPOINT_EVERY_NODES".to_string(),
            cfg.maze_checkpoint_every_nodes.to_string(),
        ),
        (
            "SHUMA_MAZE_CHECKPOINT_EVERY_MS".to_string(),
            cfg.maze_checkpoint_every_ms.to_string(),
        ),
        (
            "SHUMA_MAZE_STEP_AHEAD_MAX".to_string(),
            cfg.maze_step_ahead_max.to_string(),
        ),
        (
            "SHUMA_MAZE_NO_JS_FALLBACK_MAX_DEPTH".to_string(),
            cfg.maze_no_js_fallback_max_depth.to_string(),
        ),
        (
            "SHUMA_MAZE_MICRO_POW_ENABLED".to_string(),
            bool_env(cfg.maze_micro_pow_enabled).to_string(),
        ),
        (
            "SHUMA_MAZE_MICRO_POW_DEPTH_START".to_string(),
            cfg.maze_micro_pow_depth_start.to_string(),
        ),
        (
            "SHUMA_MAZE_MICRO_POW_BASE_DIFFICULTY".to_string(),
            cfg.maze_micro_pow_base_difficulty.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_CONCURRENT_GLOBAL".to_string(),
            cfg.maze_max_concurrent_global.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_CONCURRENT_PER_IP_BUCKET".to_string(),
            cfg.maze_max_concurrent_per_ip_bucket.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_RESPONSE_BYTES".to_string(),
            cfg.maze_max_response_bytes.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_RESPONSE_DURATION_MS".to_string(),
            cfg.maze_max_response_duration_ms.to_string(),
        ),
        (
            "SHUMA_MAZE_SERVER_VISIBLE_LINKS".to_string(),
            cfg.maze_server_visible_links.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_LINKS".to_string(),
            cfg.maze_max_links.to_string(),
        ),
        (
            "SHUMA_MAZE_MAX_PARAGRAPHS".to_string(),
            cfg.maze_max_paragraphs.to_string(),
        ),
        (
            "SHUMA_MAZE_PATH_ENTROPY_SEGMENT_LEN".to_string(),
            cfg.maze_path_entropy_segment_len.to_string(),
        ),
        (
            "SHUMA_MAZE_COVERT_DECOYS_ENABLED".to_string(),
            bool_env(cfg.maze_covert_decoys_enabled).to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_PROVIDER".to_string(),
            cfg.maze_seed_provider.as_str().to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_REFRESH_INTERVAL_SECONDS".to_string(),
            cfg.maze_seed_refresh_interval_seconds.to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_REFRESH_RATE_LIMIT_PER_HOUR".to_string(),
            cfg.maze_seed_refresh_rate_limit_per_hour.to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_REFRESH_MAX_SOURCES".to_string(),
            cfg.maze_seed_refresh_max_sources.to_string(),
        ),
        (
            "SHUMA_MAZE_SEED_METADATA_ONLY".to_string(),
            bool_env(cfg.maze_seed_metadata_only).to_string(),
        ),
        (
            "SHUMA_ROBOTS_ENABLED".to_string(),
            bool_env(cfg.robots_enabled).to_string(),
        ),
        (
            "SHUMA_ROBOTS_BLOCK_AI_TRAINING".to_string(),
            bool_env(cfg.robots_block_ai_training).to_string(),
        ),
        (
            "SHUMA_ROBOTS_BLOCK_AI_SEARCH".to_string(),
            bool_env(cfg.robots_block_ai_search).to_string(),
        ),
        (
            "SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES".to_string(),
            bool_env(cfg.robots_allow_search_engines).to_string(),
        ),
        (
            "SHUMA_AI_POLICY_BLOCK_TRAINING".to_string(),
            bool_env(cfg.robots_block_ai_training).to_string(),
        ),
        (
            "SHUMA_AI_POLICY_BLOCK_SEARCH".to_string(),
            bool_env(cfg.robots_block_ai_search).to_string(),
        ),
        (
            "SHUMA_AI_POLICY_ALLOW_SEARCH_ENGINES".to_string(),
            bool_env(cfg.robots_allow_search_engines).to_string(),
        ),
        (
            "SHUMA_ROBOTS_CRAWL_DELAY".to_string(),
            cfg.robots_crawl_delay.to_string(),
        ),
        (
            "SHUMA_CDP_DETECTION_ENABLED".to_string(),
            bool_env(cfg.cdp_detection_enabled).to_string(),
        ),
        (
            "SHUMA_CDP_AUTO_BAN".to_string(),
            bool_env(cfg.cdp_auto_ban).to_string(),
        ),
        (
            "SHUMA_CDP_DETECTION_THRESHOLD".to_string(),
            cfg.cdp_detection_threshold.to_string(),
        ),
    ]
}

fn handle_admin_config_export(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != spin_sdk::http::Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let entries = config_export_env_entries(&cfg);
    let env_map: BTreeMap<String, String> = entries.iter().cloned().collect();
    let env_text = entries
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>()
        .join("\n");

    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: None,
            reason: Some("config_export".to_string()),
            outcome: Some(format!("{} keys", entries.len())),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );

    let body = serde_json::to_string(&json!({
        "format": "env",
        "site_id": site_id,
        "generated_at": now_ts(),
        "excluded_secrets": CONFIG_EXPORT_SECRET_KEYS,
        "env": env_map,
        "env_text": env_text
    }))
    .unwrap();
    Response::new(200, body)
}

fn parse_country_list_json(field: &str, value: &serde_json::Value) -> Result<Vec<String>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("{} must be an array of 2-letter country codes", field))?;
    let mut parsed = Vec::with_capacity(items.len());
    for item in items {
        let raw = item
            .as_str()
            .ok_or_else(|| format!("{} must contain only strings", field))?;
        let code = crate::signals::geo::normalize_country_code(raw)
            .ok_or_else(|| format!("{} contains invalid country code '{}'", field, raw))?;
        parsed.push(code);
    }
    Ok(crate::signals::geo::normalize_country_list(&parsed))
}

fn parse_string_list_json(field: &str, value: &serde_json::Value) -> Result<Vec<String>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("{} must be an array of strings", field))?;
    let mut parsed = Vec::with_capacity(items.len());
    let mut seen = HashSet::new();
    for item in items {
        let raw = item
            .as_str()
            .ok_or_else(|| format!("{} must contain only strings", field))?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            parsed.push(trimmed.to_string());
        }
    }
    Ok(parsed)
}

fn sanitize_redirect_url(
    field: &str,
    value: Option<&serde_json::Value>,
) -> Result<Option<String>, String> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let url = raw
        .as_str()
        .ok_or_else(|| format!("{}.redirect_url must be a string", field))?
        .trim();
    if url.is_empty() {
        return Ok(None);
    }
    if url.len() > IP_RANGE_REDIRECT_URL_MAX_CHARS {
        return Err(format!(
            "{}.redirect_url exceeds {} characters",
            field, IP_RANGE_REDIRECT_URL_MAX_CHARS
        ));
    }
    let lower = url.to_ascii_lowercase();
    if !lower.starts_with("https://") && !lower.starts_with("http://") {
        return Err(format!(
            "{}.redirect_url must start with http:// or https://",
            field
        ));
    }
    Ok(Some(url.to_string()))
}

fn sanitize_custom_message(
    field: &str,
    value: Option<&serde_json::Value>,
) -> Result<Option<String>, String> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let message = raw
        .as_str()
        .ok_or_else(|| format!("{}.custom_message must be a string", field))?
        .trim();
    if message.is_empty() {
        return Ok(None);
    }
    if message.chars().any(|ch| ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t') {
        return Err(format!(
            "{}.custom_message contains unsupported control characters",
            field
        ));
    }
    if message.chars().count() > IP_RANGE_CUSTOM_MESSAGE_MAX_CHARS {
        return Err(format!(
            "{}.custom_message exceeds {} characters",
            field, IP_RANGE_CUSTOM_MESSAGE_MAX_CHARS
        ));
    }
    Ok(Some(message.to_string()))
}

fn parse_cidr_list_json(
    field: &str,
    value: &serde_json::Value,
    max_len: usize,
) -> Result<Vec<String>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("{} must be an array of CIDR strings", field))?;
    if items.len() > max_len {
        return Err(format!(
            "{} exceeds max length {}",
            field, max_len
        ));
    }

    let mut parsed = Vec::with_capacity(items.len());
    let mut seen = HashSet::new();
    for item in items {
        let raw = item
            .as_str()
            .ok_or_else(|| format!("{} must contain only strings", field))?;
        let Some(net) = crate::signals::ip_range_policy::parse_acceptable_cidr(raw) else {
            return Err(format!(
                "{} contains invalid or unsupported CIDR '{}'",
                field, raw
            ));
        };
        let canonical = net.to_string();
        if seen.insert(canonical.clone()) {
            parsed.push(canonical);
        }
    }
    Ok(parsed)
}

fn parse_ip_range_policy_mode_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::IpRangePolicyMode, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: off, advisory, enforce", field))?;
    crate::config::parse_ip_range_policy_mode(raw)
        .ok_or_else(|| format!("{} must be one of: off, advisory, enforce", field))
}

fn parse_ip_range_policy_action_json(
    field: &str,
    value: Option<&serde_json::Value>,
    default_action: crate::config::IpRangePolicyAction,
) -> Result<crate::config::IpRangePolicyAction, String> {
    let Some(raw_value) = value else {
        return Ok(default_action);
    };
    let raw = raw_value
        .as_str()
        .ok_or_else(|| format!("{} action must be a string", field))?;
    let Some(action) = crate::config::parse_ip_range_policy_action(raw) else {
        return Err(format!(
            "{} action must be one of: forbidden_403, custom_message, drop_connection, redirect_308, rate_limit, honeypot, maze, tarpit",
            field
        ));
    };
    Ok(action)
}

fn validate_ip_range_action_params(
    field: &str,
    action: crate::config::IpRangePolicyAction,
    redirect_url: Option<&str>,
    custom_message: Option<&str>,
) -> Result<(), String> {
    if action == crate::config::IpRangePolicyAction::Redirect308 && redirect_url.is_none() {
        return Err(format!("{} action redirect_308 requires redirect_url", field));
    }
    if action == crate::config::IpRangePolicyAction::CustomMessage && custom_message.is_none() {
        return Err(format!("{} action custom_message requires custom_message", field));
    }
    Ok(())
}

fn parse_ip_range_custom_rules_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<Vec<crate::config::IpRangePolicyRule>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("{} must be an array of objects", field))?;
    if items.len() > IP_RANGE_MAX_RULES {
        return Err(format!("{} exceeds max rules {}", field, IP_RANGE_MAX_RULES));
    }

    let mut parsed = Vec::with_capacity(items.len());
    let mut seen_ids = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        let obj = item
            .as_object()
            .ok_or_else(|| format!("{}[{}] must be an object", field, index))?;
        let enabled = obj.get("enabled").and_then(|value| value.as_bool()).unwrap_or(true);
        let id = obj
            .get("id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("custom_rule_{}", index + 1));
        if !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            return Err(format!(
                "{}[{}].id must contain only [a-zA-Z0-9_-]",
                field, index
            ));
        }
        if !seen_ids.insert(id.clone()) {
            return Err(format!("{} contains duplicate id '{}'", field, id));
        }
        let cidrs = parse_cidr_list_json(
            format!("{}[{}].cidrs", field, index).as_str(),
            obj.get("cidrs")
                .ok_or_else(|| format!("{}[{}].cidrs is required", field, index))?,
            IP_RANGE_MAX_CIDRS_PER_RULE,
        )?;
        if enabled && cidrs.is_empty() {
            return Err(format!(
                "{}[{}].cidrs must not be empty when enabled=true",
                field, index
            ));
        }
        let action = parse_ip_range_policy_action_json(
            format!("{}[{}]", field, index).as_str(),
            obj.get("action"),
            crate::config::IpRangePolicyAction::Forbidden403,
        )?;
        let redirect_url = sanitize_redirect_url(
            format!("{}[{}]", field, index).as_str(),
            obj.get("redirect_url"),
        )?;
        let custom_message = sanitize_custom_message(
            format!("{}[{}]", field, index).as_str(),
            obj.get("custom_message"),
        )?;
        validate_ip_range_action_params(
            format!("{}[{}]", field, index).as_str(),
            action,
            redirect_url.as_deref(),
            custom_message.as_deref(),
        )?;

        parsed.push(crate::config::IpRangePolicyRule {
            id,
            enabled,
            cidrs,
            action,
            redirect_url,
            custom_message,
        });
    }
    Ok(parsed)
}

fn parse_honeypot_paths_json(field: &str, value: &serde_json::Value) -> Result<Vec<String>, String> {
    let paths = parse_string_list_json(field, value)?;
    for path in &paths {
        if !is_valid_honeypot_path(path) {
            return Err(format!(
                "{} contains invalid path '{}'; each path must start with '/'; allowed unencoded characters are letters, digits, '/', '-', '.', '_', '~', '!', '$', '&', '\\'', '(', ')', '*', '+', ',', ';', '=', ':', and '@'; query ('?') and fragment ('#') are not allowed; any other character must be percent-encoded as '%HH'",
                field, path,
            ));
        }
    }
    Ok(paths)
}

fn is_valid_honeypot_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    if bytes.is_empty() || bytes[0] != b'/' {
        return false;
    }

    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if !(0x21..=0x7e).contains(&byte) {
            return false;
        }

        if byte == b'%' {
            if index + 2 >= bytes.len() {
                return false;
            }
            if !is_ascii_hex_digit(bytes[index + 1]) || !is_ascii_hex_digit(bytes[index + 2]) {
                return false;
            }
            index += 3;
            continue;
        }

        if is_ascii_alphanumeric(byte) || is_allowed_honeypot_path_byte(byte) {
            index += 1;
            continue;
        }

        return false;
    }
    true
}

fn is_ascii_alphanumeric(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

fn is_ascii_hex_digit(byte: u8) -> bool {
    byte.is_ascii_hexdigit()
}

fn is_allowed_honeypot_path_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'/' | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b'!'
            | b'$'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'='
            | b':'
            | b'@'
    )
}

fn parse_browser_rules_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<Vec<(String, u32)>, String> {
    let rules: Vec<(String, u32)> = serde_json::from_value(value.clone())
        .map_err(|_| format!("{} must be an array of [browser, min_major] tuples", field))?;
    let mut sanitized = Vec::with_capacity(rules.len());
    for (name, version) in rules {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(format!("{} contains an empty browser name", field));
        }
        sanitized.push((trimmed.to_string(), version));
    }
    Ok(sanitized)
}

fn parse_composability_mode_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::ComposabilityMode, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: off, signal, enforce, both", field))?;
    crate::config::parse_composability_mode(raw)
        .ok_or_else(|| format!("{} must be one of: off, signal, enforce, both", field))
}

fn parse_provider_backend_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::ProviderBackend, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: internal, external", field))?;
    crate::config::parse_provider_backend(raw)
        .ok_or_else(|| format!("{} must be one of: internal, external", field))
}

fn parse_edge_integration_mode_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::EdgeIntegrationMode, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: off, additive, authoritative", field))?;
    crate::config::parse_edge_integration_mode(raw)
        .ok_or_else(|| format!("{} must be one of: off, additive, authoritative", field))
}

fn parse_cdp_probe_family_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::CdpProbeFamily, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: v1, v2, split", field))?;
    crate::config::parse_cdp_probe_family(raw)
        .ok_or_else(|| format!("{} must be one of: v1, v2, split", field))
}

fn parse_maze_rollout_phase_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::MazeRolloutPhase, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: instrument, advisory, enforce", field))?;
    crate::config::parse_maze_rollout_phase(raw)
        .ok_or_else(|| format!("{} must be one of: instrument, advisory, enforce", field))
}

fn parse_maze_seed_provider_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::MazeSeedProvider, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: internal, operator", field))?;
    crate::config::parse_maze_seed_provider(raw)
        .ok_or_else(|| format!("{} must be one of: internal, operator", field))
}

fn parse_tarpit_fallback_action_json(
    field: &str,
    value: &serde_json::Value,
) -> Result<crate::config::TarpitFallbackAction, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("{} must be one of: maze, block", field))?;
    crate::config::parse_tarpit_fallback_action(raw)
        .ok_or_else(|| format!("{} must be one of: maze, block", field))
}

fn admin_config_payload(
    cfg: &crate::config::Config,
    challenge_default: u8,
    not_a_bot_default: u8,
    maze_default: u8,
) -> serde_json::Value {
    let mut payload = serde_json::to_value(cfg).unwrap_or_else(|_| json!({}));
    let Some(obj) = payload.as_object_mut() else {
        return json!({});
    };

    obj.insert(
        "ai_policy_block_training".to_string(),
        serde_json::Value::Bool(cfg.robots_block_ai_training),
    );
    obj.insert(
        "ai_policy_block_search".to_string(),
        serde_json::Value::Bool(cfg.robots_block_ai_search),
    );
    obj.insert(
        "ai_policy_allow_search_engines".to_string(),
        serde_json::Value::Bool(cfg.robots_allow_search_engines),
    );
    obj.insert(
        "admin_config_write_enabled".to_string(),
        serde_json::Value::Bool(crate::config::admin_config_write_enabled()),
    );
    obj.insert(
        "kv_store_fail_open".to_string(),
        serde_json::Value::Bool(crate::config::kv_store_fail_open()),
    );
    obj.insert(
        "https_enforced".to_string(),
        serde_json::Value::Bool(crate::config::https_enforced()),
    );
    obj.insert(
        "forwarded_header_trust_configured".to_string(),
        serde_json::Value::Bool(crate::config::forwarded_header_trust_configured()),
    );
    obj.insert(
        "runtime_environment".to_string(),
        serde_json::Value::String(crate::config::runtime_environment().as_str().to_string()),
    );
    obj.insert(
        "adversary_sim_available".to_string(),
        serde_json::Value::Bool(crate::config::adversary_sim_available()),
    );
    let frontier = crate::config::frontier_summary();
    obj.insert(
        "frontier_mode".to_string(),
        serde_json::Value::String(frontier.mode.clone()),
    );
    obj.insert(
        "frontier_provider_count".to_string(),
        serde_json::Value::Number((frontier.provider_count as u64).into()),
    );
    obj.insert(
        "frontier_diversity_confidence".to_string(),
        serde_json::Value::String(frontier.diversity_confidence.clone()),
    );
    obj.insert(
        "frontier_reduced_diversity_warning".to_string(),
        serde_json::Value::Bool(frontier.reduced_diversity_warning),
    );
    obj.insert(
        "frontier_providers".to_string(),
        serde_json::to_value(frontier.providers).unwrap_or_else(|_| json!([])),
    );
    obj.insert(
        "challenge_puzzle_risk_threshold_default".to_string(),
        serde_json::Value::Number(challenge_default.into()),
    );
    obj.insert(
        "not_a_bot_risk_threshold_default".to_string(),
        serde_json::Value::Number(not_a_bot_default.into()),
    );
    obj.insert(
        "botness_maze_threshold_default".to_string(),
        serde_json::Value::Number(maze_default.into()),
    );
    obj.insert(
        "defence_modes_effective".to_string(),
        serde_json::to_value(cfg.defence_modes_effective()).unwrap_or_else(|_| json!({})),
    );
    obj.insert(
        "defence_mode_warnings".to_string(),
        serde_json::to_value(cfg.defence_mode_warnings()).unwrap_or_else(|_| json!([])),
    );
    obj.insert(
        "enterprise_multi_instance".to_string(),
        serde_json::Value::Bool(crate::config::enterprise_multi_instance_enabled()),
    );
    obj.insert(
        "enterprise_unsynced_state_exception_confirmed".to_string(),
        serde_json::Value::Bool(crate::config::enterprise_unsynced_state_exception_confirmed()),
    );
    obj.insert(
        "enterprise_state_guardrail_warnings".to_string(),
        serde_json::to_value(cfg.enterprise_state_guardrail_warnings()).unwrap_or_else(|_| json!([])),
    );
    obj.insert(
        "enterprise_state_guardrail_error".to_string(),
        match cfg.enterprise_state_guardrail_error() {
            Some(msg) => serde_json::Value::String(msg),
            None => serde_json::Value::Null,
        },
    );
    obj.insert("botness_signal_definitions".to_string(), botness_signal_definitions(cfg));
    payload
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct AdminBanDurationsPatch {
    honeypot: Option<u64>,
    rate_limit: Option<u64>,
    admin: Option<u64>,
    cdp: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct AdminBotnessWeightsPatch {
    js_required: Option<u64>,
    geo_risk: Option<u64>,
    rate_medium: Option<u64>,
    rate_high: Option<u64>,
    maze_behavior: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct AdminDefenceModesPatch {
    rate: Option<String>,
    geo: Option<String>,
    js: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct AdminProviderBackendsPatch {
    rate_limiter: Option<String>,
    ban_store: Option<String>,
    challenge_engine: Option<String>,
    maze_tarpit: Option<String>,
    fingerprint_signal: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct AdminConfigPatch {
    test_mode: Option<bool>,
    adversary_sim_enabled: Option<bool>,
    adversary_sim_duration_seconds: Option<u64>,
    ban_duration: Option<u64>,
    rate_limit: Option<u64>,
    js_required_enforced: Option<bool>,
    geo_risk: Option<serde_json::Value>,
    geo_allow: Option<serde_json::Value>,
    geo_challenge: Option<serde_json::Value>,
    geo_maze: Option<serde_json::Value>,
    geo_block: Option<serde_json::Value>,
    geo_edge_headers_enabled: Option<bool>,
    honeypot_enabled: Option<bool>,
    honeypots: Option<serde_json::Value>,
    browser_policy_enabled: Option<bool>,
    browser_block: Option<serde_json::Value>,
    browser_allowlist: Option<serde_json::Value>,
    bypass_allowlists_enabled: Option<bool>,
    allowlist: Option<serde_json::Value>,
    path_allowlist_enabled: Option<bool>,
    path_allowlist: Option<serde_json::Value>,
    ip_range_policy_mode: Option<String>,
    ip_range_emergency_allowlist: Option<serde_json::Value>,
    ip_range_custom_rules: Option<serde_json::Value>,
    ban_durations: Option<AdminBanDurationsPatch>,
    maze_enabled: Option<bool>,
    tarpit_enabled: Option<bool>,
    tarpit_progress_token_ttl_seconds: Option<u64>,
    tarpit_progress_replay_ttl_seconds: Option<u64>,
    tarpit_hashcash_min_difficulty: Option<u64>,
    tarpit_hashcash_max_difficulty: Option<u64>,
    tarpit_hashcash_base_difficulty: Option<u64>,
    tarpit_hashcash_adaptive: Option<bool>,
    tarpit_step_chunk_base_bytes: Option<u64>,
    tarpit_step_chunk_max_bytes: Option<u64>,
    tarpit_step_jitter_percent: Option<u64>,
    tarpit_shard_rotation_enabled: Option<bool>,
    tarpit_egress_window_seconds: Option<u64>,
    tarpit_egress_global_bytes_per_window: Option<u64>,
    tarpit_egress_per_ip_bucket_bytes_per_window: Option<u64>,
    tarpit_egress_per_flow_max_bytes: Option<u64>,
    tarpit_egress_per_flow_max_duration_seconds: Option<u64>,
    tarpit_max_concurrent_global: Option<u64>,
    tarpit_max_concurrent_per_ip_bucket: Option<u64>,
    tarpit_fallback_action: Option<String>,
    maze_auto_ban: Option<bool>,
    maze_auto_ban_threshold: Option<u64>,
    maze_rollout_phase: Option<String>,
    maze_token_ttl_seconds: Option<u64>,
    maze_token_max_depth: Option<u64>,
    maze_token_branch_budget: Option<u64>,
    maze_replay_ttl_seconds: Option<u64>,
    maze_entropy_window_seconds: Option<u64>,
    maze_client_expansion_enabled: Option<bool>,
    maze_checkpoint_every_nodes: Option<u64>,
    maze_checkpoint_every_ms: Option<u64>,
    maze_step_ahead_max: Option<u64>,
    maze_no_js_fallback_max_depth: Option<u64>,
    maze_micro_pow_enabled: Option<bool>,
    maze_micro_pow_depth_start: Option<u64>,
    maze_micro_pow_base_difficulty: Option<u64>,
    maze_max_concurrent_global: Option<u64>,
    maze_max_concurrent_per_ip_bucket: Option<u64>,
    maze_max_response_bytes: Option<u64>,
    maze_max_response_duration_ms: Option<u64>,
    maze_server_visible_links: Option<u64>,
    maze_max_links: Option<u64>,
    maze_max_paragraphs: Option<u64>,
    maze_path_entropy_segment_len: Option<u64>,
    maze_covert_decoys_enabled: Option<bool>,
    maze_seed_provider: Option<String>,
    maze_seed_refresh_interval_seconds: Option<u64>,
    maze_seed_refresh_rate_limit_per_hour: Option<u64>,
    maze_seed_refresh_max_sources: Option<u64>,
    maze_seed_metadata_only: Option<bool>,
    robots_enabled: Option<bool>,
    ai_policy_block_training: Option<bool>,
    robots_block_ai_training: Option<bool>,
    ai_policy_block_search: Option<bool>,
    robots_block_ai_search: Option<bool>,
    ai_policy_allow_search_engines: Option<bool>,
    robots_allow_search_engines: Option<bool>,
    robots_crawl_delay: Option<u64>,
    cdp_detection_enabled: Option<bool>,
    cdp_auto_ban: Option<bool>,
    cdp_detection_threshold: Option<f64>,
    cdp_probe_family: Option<String>,
    cdp_probe_rollout_percent: Option<u64>,
    fingerprint_signal_enabled: Option<bool>,
    fingerprint_state_ttl_seconds: Option<u64>,
    fingerprint_flow_window_seconds: Option<u64>,
    fingerprint_flow_violation_threshold: Option<u64>,
    fingerprint_pseudonymize: Option<bool>,
    fingerprint_entropy_budget: Option<u64>,
    fingerprint_family_cap_header_runtime: Option<u64>,
    fingerprint_family_cap_transport: Option<u64>,
    fingerprint_family_cap_temporal: Option<u64>,
    fingerprint_family_cap_persistence: Option<u64>,
    fingerprint_family_cap_behavior: Option<u64>,
    pow_enabled: Option<bool>,
    pow_difficulty: Option<u64>,
    pow_ttl_seconds: Option<u64>,
    challenge_puzzle_enabled: Option<bool>,
    challenge_puzzle_transform_count: Option<u64>,
    challenge_puzzle_seed_ttl_seconds: Option<u64>,
    challenge_puzzle_attempt_limit_per_window: Option<u64>,
    challenge_puzzle_attempt_window_seconds: Option<u64>,
    not_a_bot_enabled: Option<bool>,
    not_a_bot_risk_threshold: Option<u64>,
    not_a_bot_pass_score: Option<u64>,
    not_a_bot_fail_score: Option<u64>,
    not_a_bot_nonce_ttl_seconds: Option<u64>,
    not_a_bot_marker_ttl_seconds: Option<u64>,
    not_a_bot_attempt_limit_per_window: Option<u64>,
    not_a_bot_attempt_window_seconds: Option<u64>,
    provider_backends: Option<AdminProviderBackendsPatch>,
    edge_integration_mode: Option<String>,
    challenge_puzzle_risk_threshold: Option<u64>,
    botness_maze_threshold: Option<u64>,
    botness_weights: Option<AdminBotnessWeightsPatch>,
    defence_modes: Option<AdminDefenceModesPatch>,
}

fn validate_admin_config_patch_shape(json: &serde_json::Value) -> Result<(), String> {
    serde_json::from_value::<AdminConfigPatch>(json.clone())
        .map(|_| ())
        .map_err(|err| format!("Invalid config payload: {}", err))
}

#[derive(Serialize)]
struct AdminConfigValidationIssue {
    field: Option<String>,
    message: String,
    expected: Option<String>,
    received: Option<serde_json::Value>,
}

fn admin_config_validation_read_value(
    json: &serde_json::Value,
    field: &str,
) -> Option<serde_json::Value> {
    let mut cursor = json;
    for segment in field.split('.') {
        let obj = cursor.as_object()?;
        cursor = obj.get(segment)?;
    }
    Some(cursor.clone())
}

fn admin_config_validation_field(message: &str) -> Option<String> {
    if let Some(start) = message.find("unknown field `") {
        let rest = &message[start + "unknown field `".len()..];
        if let Some(end) = rest.find('`') {
            let field = rest[..end].trim();
            if !field.is_empty() {
                return Some(field.to_string());
            }
        }
    }

    if let Some(start) = message.find("`") {
        let rest = &message[start + 1..];
        if let Some(end) = rest.find('`') {
            let field = rest[..end].trim();
            if !field.is_empty() && field.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                return Some(field.to_string());
            }
        }
    }

    let first_token = message
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.');
    if first_token.is_empty() {
        return None;
    }
    if first_token
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
    {
        let looks_like_field = first_token.contains('_')
            || first_token.contains('.')
            || first_token
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit());
        if looks_like_field {
            return Some(first_token.to_string());
        }
    }
    None
}

fn admin_config_validation_expected(message: &str) -> Option<String> {
    if message.contains("unknown field `") {
        return Some(
            "Unknown key. Use only writable keys from docs/configuration.md.".to_string(),
        );
    }

    if let Some(start) = message.find("out of range (") {
        let rest = &message[start + "out of range (".len()..];
        if let Some(end) = rest.find(')') {
            return Some(format!("Value must be in range {}.", &rest[..end]));
        }
    }

    if let Some(start) = message.find("must be one of [") {
        let rest = &message[start + "must be one of [".len()..];
        if let Some(end) = rest.find(']') {
            return Some(format!("Value must be one of: {}.", &rest[..end]));
        }
    }

    if message.contains("must be an integer") {
        return Some("Value must be an integer.".to_string());
    }

    if message.contains("must be <= ") {
        return Some("Keep values within the documented ordering constraints.".to_string());
    }

    if message.contains("must be lower than") {
        return Some("Keep values within the documented ordering constraints.".to_string());
    }

    if message.starts_with("Invalid config payload: invalid type") {
        return Some("Type mismatch: use the value type required for that key.".to_string());
    }

    None
}

fn admin_config_validation_issue(
    patch: &serde_json::Value,
    message: String,
) -> AdminConfigValidationIssue {
    let mut field = admin_config_validation_field(&message);
    if field.is_none() {
        if let Some(object) = patch.as_object() {
            if object.len() == 1 {
                field = object.keys().next().map(|key| key.to_string());
            }
        }
    }
    let received = field
        .as_deref()
        .and_then(|field_name| admin_config_validation_read_value(patch, field_name));
    AdminConfigValidationIssue {
        field,
        expected: admin_config_validation_expected(&message),
        message,
        received,
    }
}

fn persist_site_config(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    cfg: &crate::config::Config,
) -> Result<(), ()> {
    let key = format!("config:{}", site_id);
    let encoded = serde_json::to_vec(cfg).map_err(|_| ())?;
    store.set(&key, &encoded).map_err(|_| ())?;
    crate::config::invalidate_runtime_cache(site_id);
    Ok(())
}

fn handle_admin_config_internal(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    validate_only: bool,
) -> Response {
    // GET: Return current config
    // POST: Update config (supports {"test_mode": true/false})
    if *req.method() == spin_sdk::http::Method::Post {
        if !crate::config::admin_config_write_enabled() {
            return Response::new(
                403,
                "Config updates are disabled when SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false",
            );
        }
        let json = match crate::request_validation::parse_json_body(
            req.body(),
            crate::request_validation::MAX_ADMIN_JSON_BYTES,
        ) {
            Ok(v) => v,
            Err(e) => return Response::new(400, e),
        };
        if let Err(err) = validate_admin_config_patch_shape(&json) {
            return Response::new(400, err);
        }
        // Load current config
        let mut cfg = match crate::config::Config::load(store, site_id) {
            Ok(cfg) => cfg,
            Err(err) => return Response::new(500, err.user_message()),
        };
        let mut changed = false;
        let mut effective_adversary_sim_enabled =
            crate::config::runtime_adversary_sim_enabled_for_site(site_id);

        // Update test_mode if provided.
        if let Some(test_mode) = json.get("test_mode").and_then(|v| v.as_bool()) {
            let old_value = cfg.test_mode;
            if old_value != test_mode {
                cfg.test_mode = test_mode;
                changed = true;
                if !validate_only {
                    log_event(
                        store,
                        &EventLogEntry {
                            ts: now_ts(),
                            event: EventType::AdminAction,
                            ip: None,
                            reason: Some("test_mode_toggle".to_string()),
                            outcome: Some(format!("{} -> {}", old_value, test_mode)),
                            admin: Some(crate::admin::auth::get_admin_id(req)),
                        },
                    );
                }
            }
        }
        // Update adversary_sim_enabled if provided.
        if let Some(adversary_sim_enabled) =
            json.get("adversary_sim_enabled").and_then(|v| v.as_bool())
        {
            if adversary_sim_enabled && crate::config::runtime_environment().is_prod() {
                return Response::new(
                    400,
                    "adversary_sim_enabled cannot be set to true when SHUMA_RUNTIME_ENV=runtime-prod",
                );
            }
            if !validate_only {
                crate::config::set_runtime_adversary_sim_enabled_override(
                    site_id,
                    adversary_sim_enabled,
                );
            }
            effective_adversary_sim_enabled = adversary_sim_enabled;
        }
        if let Some(adversary_sim_duration_seconds) = json
            .get("adversary_sim_duration_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(ADVERSARY_SIM_DURATION_SECONDS_MIN..=ADVERSARY_SIM_DURATION_SECONDS_MAX)
                .contains(&adversary_sim_duration_seconds)
            {
                return Response::new(
                    400,
                    format!(
                        "adversary_sim_duration_seconds out of range ({}-{})",
                        ADVERSARY_SIM_DURATION_SECONDS_MIN, ADVERSARY_SIM_DURATION_SECONDS_MAX
                    ),
                );
            }
            if cfg.adversary_sim_duration_seconds != adversary_sim_duration_seconds {
                cfg.adversary_sim_duration_seconds = adversary_sim_duration_seconds;
                changed = true;
            }
        }

        // Update other config fields if provided
        if let Some(ban_duration) = json.get("ban_duration").and_then(|v| v.as_u64()) {
            cfg.ban_duration = ban_duration;
            changed = true;
        }
        if let Some(rate_limit) = json.get("rate_limit").and_then(|v| v.as_u64()) {
            if !(1..=1_000_000).contains(&rate_limit) {
                return Response::new(400, "rate_limit out of range (1-1000000)");
            }
            cfg.rate_limit = rate_limit as u32;
            changed = true;
        }
        if let Some(js_required_enforced) =
            json.get("js_required_enforced").and_then(|v| v.as_bool())
        {
            cfg.js_required_enforced = js_required_enforced;
            changed = true;
        }

        // Update GEO policy lists if provided.
        if let Some(value) = json.get("geo_risk") {
            match parse_country_list_json("geo_risk", value) {
                Ok(list) => {
                    cfg.geo_risk = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("geo_allow") {
            match parse_country_list_json("geo_allow", value) {
                Ok(list) => {
                    cfg.geo_allow = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("geo_challenge") {
            match parse_country_list_json("geo_challenge", value) {
                Ok(list) => {
                    cfg.geo_challenge = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("geo_maze") {
            match parse_country_list_json("geo_maze", value) {
                Ok(list) => {
                    cfg.geo_maze = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("geo_block") {
            match parse_country_list_json("geo_block", value) {
                Ok(list) => {
                    cfg.geo_block = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(geo_edge_headers_enabled) =
            json.get("geo_edge_headers_enabled").and_then(|v| v.as_bool())
        {
            cfg.geo_edge_headers_enabled = geo_edge_headers_enabled;
            changed = true;
        }

        if let Some(honeypot_enabled) = json.get("honeypot_enabled").and_then(|v| v.as_bool()) {
            cfg.honeypot_enabled = honeypot_enabled;
            changed = true;
        }
        if let Some(value) = json.get("honeypots") {
            match parse_honeypot_paths_json("honeypots", value) {
                Ok(list) => {
                    cfg.honeypots = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(browser_policy_enabled) =
            json.get("browser_policy_enabled").and_then(|v| v.as_bool())
        {
            cfg.browser_policy_enabled = browser_policy_enabled;
            changed = true;
        }
        if let Some(value) = json.get("browser_block") {
            match parse_browser_rules_json("browser_block", value) {
                Ok(rules) => {
                    cfg.browser_block = rules;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("browser_allowlist") {
            match parse_browser_rules_json("browser_allowlist", value) {
                Ok(rules) => {
                    cfg.browser_allowlist = rules;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(bypass_allowlists_enabled) =
            json.get("bypass_allowlists_enabled").and_then(|v| v.as_bool())
        {
            cfg.bypass_allowlists_enabled = bypass_allowlists_enabled;
            changed = true;
        }
        if let Some(value) = json.get("allowlist") {
            match parse_string_list_json("allowlist", value) {
                Ok(list) => {
                    cfg.allowlist = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(path_allowlist_enabled) =
            json.get("path_allowlist_enabled").and_then(|v| v.as_bool())
        {
            cfg.path_allowlist_enabled = path_allowlist_enabled;
            changed = true;
        }
        if let Some(value) = json.get("path_allowlist") {
            match parse_string_list_json("path_allowlist", value) {
                Ok(list) => {
                    cfg.path_allowlist = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("ip_range_policy_mode") {
            match parse_ip_range_policy_mode_json("ip_range_policy_mode", value) {
                Ok(mode) => {
                    cfg.ip_range_policy_mode = mode;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("ip_range_emergency_allowlist") {
            match parse_cidr_list_json(
                "ip_range_emergency_allowlist",
                value,
                IP_RANGE_MAX_EMERGENCY_ALLOWLIST,
            ) {
                Ok(list) => {
                    cfg.ip_range_emergency_allowlist = list;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }
        if let Some(value) = json.get("ip_range_custom_rules") {
            match parse_ip_range_custom_rules_json("ip_range_custom_rules", value) {
                Ok(rules) => {
                    cfg.ip_range_custom_rules = rules;
                    changed = true;
                }
                Err(msg) => return Response::new(400, msg),
            }
        }

        // Update per-type ban durations if provided
        if let Some(ban_durations) = json.get("ban_durations") {
            if let Some(honeypot) = ban_durations.get("honeypot").and_then(|v| v.as_u64()) {
                cfg.ban_durations.honeypot = honeypot;
                changed = true;
            }
            if let Some(rate_limit) = ban_durations.get("rate_limit").and_then(|v| v.as_u64()) {
                cfg.ban_durations.rate_limit = rate_limit;
                changed = true;
            }
            if let Some(admin) = ban_durations.get("admin").and_then(|v| v.as_u64()) {
                cfg.ban_durations.admin = admin;
                changed = true;
            }
            if let Some(cdp) = ban_durations.get("cdp").and_then(|v| v.as_u64()) {
                cfg.ban_durations.cdp = cdp;
                changed = true;
            }
        }

        // Update maze settings if provided
        let old_tarpit_enabled = cfg.tarpit_enabled;
        let old_tarpit_progress_token_ttl_seconds = cfg.tarpit_progress_token_ttl_seconds;
        let old_tarpit_progress_replay_ttl_seconds = cfg.tarpit_progress_replay_ttl_seconds;
        let old_tarpit_hashcash_min_difficulty = cfg.tarpit_hashcash_min_difficulty;
        let old_tarpit_hashcash_max_difficulty = cfg.tarpit_hashcash_max_difficulty;
        let old_tarpit_hashcash_base_difficulty = cfg.tarpit_hashcash_base_difficulty;
        let old_tarpit_hashcash_adaptive = cfg.tarpit_hashcash_adaptive;
        let old_tarpit_step_chunk_base_bytes = cfg.tarpit_step_chunk_base_bytes;
        let old_tarpit_step_chunk_max_bytes = cfg.tarpit_step_chunk_max_bytes;
        let old_tarpit_step_jitter_percent = cfg.tarpit_step_jitter_percent;
        let old_tarpit_shard_rotation_enabled = cfg.tarpit_shard_rotation_enabled;
        let old_tarpit_egress_window_seconds = cfg.tarpit_egress_window_seconds;
        let old_tarpit_egress_global_bytes_per_window = cfg.tarpit_egress_global_bytes_per_window;
        let old_tarpit_egress_per_ip_bucket_bytes_per_window =
            cfg.tarpit_egress_per_ip_bucket_bytes_per_window;
        let old_tarpit_egress_per_flow_max_bytes = cfg.tarpit_egress_per_flow_max_bytes;
        let old_tarpit_egress_per_flow_max_duration_seconds =
            cfg.tarpit_egress_per_flow_max_duration_seconds;
        let old_tarpit_max_concurrent_global = cfg.tarpit_max_concurrent_global;
        let old_tarpit_max_concurrent_per_ip_bucket = cfg.tarpit_max_concurrent_per_ip_bucket;
        let old_tarpit_fallback_action = cfg.tarpit_fallback_action;
        let mut tarpit_changed = false;

        if let Some(maze_enabled) = json.get("maze_enabled").and_then(|v| v.as_bool()) {
            cfg.maze_enabled = maze_enabled;
            changed = true;
        }
        if let Some(tarpit_enabled) = json.get("tarpit_enabled").and_then(|v| v.as_bool()) {
            if cfg.tarpit_enabled != tarpit_enabled {
                cfg.tarpit_enabled = tarpit_enabled;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_progress_token_ttl_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MIN..=TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_progress_token_ttl_seconds out of range ({}-{})",
                        TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MIN, TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MAX
                    ),
                );
            }
            if cfg.tarpit_progress_token_ttl_seconds != value {
                cfg.tarpit_progress_token_ttl_seconds = value;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_progress_replay_ttl_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MIN..=TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_progress_replay_ttl_seconds out of range ({}-{})",
                        TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MIN,
                        TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MAX
                    ),
                );
            }
            if cfg.tarpit_progress_replay_ttl_seconds != value {
                cfg.tarpit_progress_replay_ttl_seconds = value;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_hashcash_min_difficulty")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_HASHCASH_DIFFICULTY_MIN..=TARPIT_HASHCASH_DIFFICULTY_MAX).contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_hashcash_min_difficulty out of range ({}-{})",
                        TARPIT_HASHCASH_DIFFICULTY_MIN, TARPIT_HASHCASH_DIFFICULTY_MAX
                    ),
                );
            }
            if cfg.tarpit_hashcash_min_difficulty != value as u8 {
                cfg.tarpit_hashcash_min_difficulty = value as u8;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_hashcash_max_difficulty")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_HASHCASH_DIFFICULTY_MIN..=TARPIT_HASHCASH_DIFFICULTY_MAX).contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_hashcash_max_difficulty out of range ({}-{})",
                        TARPIT_HASHCASH_DIFFICULTY_MIN, TARPIT_HASHCASH_DIFFICULTY_MAX
                    ),
                );
            }
            if cfg.tarpit_hashcash_max_difficulty != value as u8 {
                cfg.tarpit_hashcash_max_difficulty = value as u8;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_hashcash_base_difficulty")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_HASHCASH_DIFFICULTY_MIN..=TARPIT_HASHCASH_DIFFICULTY_MAX).contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_hashcash_base_difficulty out of range ({}-{})",
                        TARPIT_HASHCASH_DIFFICULTY_MIN, TARPIT_HASHCASH_DIFFICULTY_MAX
                    ),
                );
            }
            if cfg.tarpit_hashcash_base_difficulty != value as u8 {
                cfg.tarpit_hashcash_base_difficulty = value as u8;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json.get("tarpit_hashcash_adaptive").and_then(|v| v.as_bool()) {
            if cfg.tarpit_hashcash_adaptive != value {
                cfg.tarpit_hashcash_adaptive = value;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_step_chunk_base_bytes")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_STEP_CHUNK_BASE_BYTES_MIN..=TARPIT_STEP_CHUNK_BASE_BYTES_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_step_chunk_base_bytes out of range ({}-{})",
                        TARPIT_STEP_CHUNK_BASE_BYTES_MIN, TARPIT_STEP_CHUNK_BASE_BYTES_MAX
                    ),
                );
            }
            if cfg.tarpit_step_chunk_base_bytes != value as u32 {
                cfg.tarpit_step_chunk_base_bytes = value as u32;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_step_chunk_max_bytes")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_STEP_CHUNK_MAX_BYTES_MIN..=TARPIT_STEP_CHUNK_MAX_BYTES_MAX).contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_step_chunk_max_bytes out of range ({}-{})",
                        TARPIT_STEP_CHUNK_MAX_BYTES_MIN, TARPIT_STEP_CHUNK_MAX_BYTES_MAX
                    ),
                );
            }
            if cfg.tarpit_step_chunk_max_bytes != value as u32 {
                cfg.tarpit_step_chunk_max_bytes = value as u32;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_step_jitter_percent")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_STEP_JITTER_PERCENT_MIN..=TARPIT_STEP_JITTER_PERCENT_MAX).contains(&value) {
                return Response::new(
                    400,
                    format!(
                        "tarpit_step_jitter_percent out of range ({}-{})",
                        TARPIT_STEP_JITTER_PERCENT_MIN, TARPIT_STEP_JITTER_PERCENT_MAX
                    ),
                );
            }
            if cfg.tarpit_step_jitter_percent != value as u8 {
                cfg.tarpit_step_jitter_percent = value as u8;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_shard_rotation_enabled")
            .and_then(|v| v.as_bool())
        {
            if cfg.tarpit_shard_rotation_enabled != value {
                cfg.tarpit_shard_rotation_enabled = value;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_egress_window_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_EGRESS_WINDOW_SECONDS_MIN..=TARPIT_EGRESS_WINDOW_SECONDS_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_egress_window_seconds out of range ({}-{})",
                        TARPIT_EGRESS_WINDOW_SECONDS_MIN, TARPIT_EGRESS_WINDOW_SECONDS_MAX
                    ),
                );
            }
            if cfg.tarpit_egress_window_seconds != value {
                cfg.tarpit_egress_window_seconds = value;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_egress_global_bytes_per_window")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW_MIN..=TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_egress_global_bytes_per_window out of range ({}-{})",
                        TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW_MIN,
                        TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW_MAX
                    ),
                );
            }
            if cfg.tarpit_egress_global_bytes_per_window != value {
                cfg.tarpit_egress_global_bytes_per_window = value;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_egress_per_ip_bucket_bytes_per_window")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW_MIN
                ..=TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_egress_per_ip_bucket_bytes_per_window out of range ({}-{})",
                        TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW_MIN,
                        TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW_MAX
                    ),
                );
            }
            if cfg.tarpit_egress_per_ip_bucket_bytes_per_window != value {
                cfg.tarpit_egress_per_ip_bucket_bytes_per_window = value;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_egress_per_flow_max_bytes")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_EGRESS_PER_FLOW_MAX_BYTES_MIN..=TARPIT_EGRESS_PER_FLOW_MAX_BYTES_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_egress_per_flow_max_bytes out of range ({}-{})",
                        TARPIT_EGRESS_PER_FLOW_MAX_BYTES_MIN, TARPIT_EGRESS_PER_FLOW_MAX_BYTES_MAX
                    ),
                );
            }
            if cfg.tarpit_egress_per_flow_max_bytes != value {
                cfg.tarpit_egress_per_flow_max_bytes = value;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_egress_per_flow_max_duration_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS_MIN
                ..=TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_egress_per_flow_max_duration_seconds out of range ({}-{})",
                        TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS_MIN,
                        TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS_MAX
                    ),
                );
            }
            if cfg.tarpit_egress_per_flow_max_duration_seconds != value {
                cfg.tarpit_egress_per_flow_max_duration_seconds = value;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_max_concurrent_global")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_MAX_CONCURRENT_GLOBAL_MIN..=TARPIT_MAX_CONCURRENT_GLOBAL_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_max_concurrent_global out of range ({}-{})",
                        TARPIT_MAX_CONCURRENT_GLOBAL_MIN, TARPIT_MAX_CONCURRENT_GLOBAL_MAX
                    ),
                );
            }
            if cfg.tarpit_max_concurrent_global != value as u32 {
                cfg.tarpit_max_concurrent_global = value as u32;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(value) = json
            .get("tarpit_max_concurrent_per_ip_bucket")
            .and_then(|v| v.as_u64())
        {
            if !(TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MIN..=TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MAX)
                .contains(&value)
            {
                return Response::new(
                    400,
                    format!(
                        "tarpit_max_concurrent_per_ip_bucket out of range ({}-{})",
                        TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MIN,
                        TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MAX
                    ),
                );
            }
            if cfg.tarpit_max_concurrent_per_ip_bucket != value as u32 {
                cfg.tarpit_max_concurrent_per_ip_bucket = value as u32;
                changed = true;
                tarpit_changed = true;
            }
        }
        if cfg.tarpit_max_concurrent_per_ip_bucket > cfg.tarpit_max_concurrent_global {
            return Response::new(
                400,
                "tarpit_max_concurrent_per_ip_bucket must be <= tarpit_max_concurrent_global",
            );
        }
        if cfg.tarpit_hashcash_max_difficulty < cfg.tarpit_hashcash_min_difficulty {
            return Response::new(
                400,
                "tarpit_hashcash_max_difficulty must be >= tarpit_hashcash_min_difficulty",
            );
        }
        if cfg.tarpit_hashcash_base_difficulty < cfg.tarpit_hashcash_min_difficulty
            || cfg.tarpit_hashcash_base_difficulty > cfg.tarpit_hashcash_max_difficulty
        {
            return Response::new(
                400,
                "tarpit_hashcash_base_difficulty must be between min and max difficulty",
            );
        }
        if cfg.tarpit_step_chunk_max_bytes < cfg.tarpit_step_chunk_base_bytes {
            return Response::new(
                400,
                "tarpit_step_chunk_max_bytes must be >= tarpit_step_chunk_base_bytes",
            );
        }
        if cfg.tarpit_egress_per_ip_bucket_bytes_per_window > cfg.tarpit_egress_global_bytes_per_window {
            return Response::new(
                400,
                "tarpit_egress_per_ip_bucket_bytes_per_window must be <= tarpit_egress_global_bytes_per_window",
            );
        }
        if let Some(value) = json.get("tarpit_fallback_action") {
            let next = match parse_tarpit_fallback_action_json("tarpit_fallback_action", value) {
                Ok(action) => action,
                Err(msg) => return Response::new(400, msg),
            };
            if cfg.tarpit_fallback_action != next {
                cfg.tarpit_fallback_action = next;
                changed = true;
                tarpit_changed = true;
            }
        }
        if let Some(maze_auto_ban) = json.get("maze_auto_ban").and_then(|v| v.as_bool()) {
            cfg.maze_auto_ban = maze_auto_ban;
            changed = true;
        }
        if let Some(maze_auto_ban_threshold) =
            json.get("maze_auto_ban_threshold").and_then(|v| v.as_u64())
        {
            cfg.maze_auto_ban_threshold = maze_auto_ban_threshold as u32;
            changed = true;
        }
        if let Some(value) = json.get("maze_rollout_phase") {
            cfg.maze_rollout_phase =
                match parse_maze_rollout_phase_json("maze_rollout_phase", value) {
                    Ok(phase) => phase,
                    Err(msg) => return Response::new(400, msg),
                };
            changed = true;
        }
        if let Some(v) = json.get("maze_token_ttl_seconds").and_then(|v| v.as_u64()) {
            cfg.maze_token_ttl_seconds = v;
            changed = true;
        }
        if let Some(v) = json.get("maze_token_max_depth").and_then(|v| v.as_u64()) {
            cfg.maze_token_max_depth = v as u16;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_token_branch_budget")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_token_branch_budget = v as u8;
            changed = true;
        }
        if let Some(v) = json.get("maze_replay_ttl_seconds").and_then(|v| v.as_u64()) {
            cfg.maze_replay_ttl_seconds = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_entropy_window_seconds")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_entropy_window_seconds = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_client_expansion_enabled")
            .and_then(|v| v.as_bool())
        {
            cfg.maze_client_expansion_enabled = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_checkpoint_every_nodes")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_checkpoint_every_nodes = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_checkpoint_every_ms")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_checkpoint_every_ms = v;
            changed = true;
        }
        if let Some(v) = json.get("maze_step_ahead_max").and_then(|v| v.as_u64()) {
            cfg.maze_step_ahead_max = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_no_js_fallback_max_depth")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_no_js_fallback_max_depth = v as u16;
            changed = true;
        }
        if let Some(v) = json.get("maze_micro_pow_enabled").and_then(|v| v.as_bool()) {
            cfg.maze_micro_pow_enabled = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_micro_pow_depth_start")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_micro_pow_depth_start = v as u16;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_micro_pow_base_difficulty")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_micro_pow_base_difficulty = v as u8;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_max_concurrent_global")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_max_concurrent_global = v as u32;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_max_concurrent_per_ip_bucket")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_max_concurrent_per_ip_bucket = v as u32;
            changed = true;
        }
        if let Some(v) = json.get("maze_max_response_bytes").and_then(|v| v.as_u64()) {
            cfg.maze_max_response_bytes = v as u32;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_max_response_duration_ms")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_max_response_duration_ms = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_server_visible_links")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_server_visible_links = v as u32;
            changed = true;
        }
        if let Some(v) = json.get("maze_max_links").and_then(|v| v.as_u64()) {
            cfg.maze_max_links = v as u32;
            changed = true;
        }
        if let Some(v) = json.get("maze_max_paragraphs").and_then(|v| v.as_u64()) {
            cfg.maze_max_paragraphs = v as u32;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_path_entropy_segment_len")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_path_entropy_segment_len = v as u8;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_covert_decoys_enabled")
            .and_then(|v| v.as_bool())
        {
            cfg.maze_covert_decoys_enabled = v;
            changed = true;
        }
        if let Some(value) = json.get("maze_seed_provider") {
            cfg.maze_seed_provider =
                match parse_maze_seed_provider_json("maze_seed_provider", value) {
                    Ok(provider) => provider,
                    Err(msg) => return Response::new(400, msg),
                };
            changed = true;
        }
        if let Some(v) = json
            .get("maze_seed_refresh_interval_seconds")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_seed_refresh_interval_seconds = v;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_seed_refresh_rate_limit_per_hour")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_seed_refresh_rate_limit_per_hour = v as u32;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_seed_refresh_max_sources")
            .and_then(|v| v.as_u64())
        {
            cfg.maze_seed_refresh_max_sources = v as u32;
            changed = true;
        }
        if let Some(v) = json
            .get("maze_seed_metadata_only")
            .and_then(|v| v.as_bool())
        {
            cfg.maze_seed_metadata_only = v;
            changed = true;
        }
        if tarpit_changed && !validate_only {
            log_event(
                store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("tarpit_config_update".to_string()),
                    outcome: Some(format!(
                        "enabled:{}->{} token_ttl:{}->{} replay_ttl:{}->{} hashcash(min/max/base/adaptive):{}/{}/{}/{}->{}/{}/{}/{} chunk(base/max/jitter/rotation):{}/{}/{}/{}->{}/{}/{}/{} egress(window/global/per_bucket/flow_bytes/flow_duration):{}/{}/{}/{}/{}->{}/{}/{}/{}/{} max_global:{}->{} max_per_ip_bucket:{}->{} fallback_action:{}->{}",
                        old_tarpit_enabled,
                        cfg.tarpit_enabled,
                        old_tarpit_progress_token_ttl_seconds,
                        cfg.tarpit_progress_token_ttl_seconds,
                        old_tarpit_progress_replay_ttl_seconds,
                        cfg.tarpit_progress_replay_ttl_seconds,
                        old_tarpit_hashcash_min_difficulty,
                        old_tarpit_hashcash_max_difficulty,
                        old_tarpit_hashcash_base_difficulty,
                        old_tarpit_hashcash_adaptive,
                        cfg.tarpit_hashcash_min_difficulty,
                        cfg.tarpit_hashcash_max_difficulty,
                        cfg.tarpit_hashcash_base_difficulty,
                        cfg.tarpit_hashcash_adaptive,
                        old_tarpit_step_chunk_base_bytes,
                        old_tarpit_step_chunk_max_bytes,
                        old_tarpit_step_jitter_percent,
                        old_tarpit_shard_rotation_enabled,
                        cfg.tarpit_step_chunk_base_bytes,
                        cfg.tarpit_step_chunk_max_bytes,
                        cfg.tarpit_step_jitter_percent,
                        cfg.tarpit_shard_rotation_enabled,
                        old_tarpit_egress_window_seconds,
                        old_tarpit_egress_global_bytes_per_window,
                        old_tarpit_egress_per_ip_bucket_bytes_per_window,
                        old_tarpit_egress_per_flow_max_bytes,
                        old_tarpit_egress_per_flow_max_duration_seconds,
                        cfg.tarpit_egress_window_seconds,
                        cfg.tarpit_egress_global_bytes_per_window,
                        cfg.tarpit_egress_per_ip_bucket_bytes_per_window,
                        cfg.tarpit_egress_per_flow_max_bytes,
                        cfg.tarpit_egress_per_flow_max_duration_seconds,
                        old_tarpit_max_concurrent_global,
                        cfg.tarpit_max_concurrent_global,
                        old_tarpit_max_concurrent_per_ip_bucket,
                        cfg.tarpit_max_concurrent_per_ip_bucket,
                        old_tarpit_fallback_action.as_str(),
                        cfg.tarpit_fallback_action.as_str()
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
        }

        // Update robots.txt settings if provided
        if let Some(robots_enabled) = json.get("robots_enabled").and_then(|v| v.as_bool()) {
            cfg.robots_enabled = robots_enabled;
            changed = true;
        }
        let ai_policy_block_training = json
            .get("ai_policy_block_training")
            .and_then(|v| v.as_bool())
            .or_else(|| {
                json.get("robots_block_ai_training")
                    .and_then(|v| v.as_bool())
            });
        if let Some(robots_block_ai_training) = ai_policy_block_training {
            cfg.robots_block_ai_training = robots_block_ai_training;
            changed = true;
        }
        let ai_policy_block_search = json
            .get("ai_policy_block_search")
            .and_then(|v| v.as_bool())
            .or_else(|| json.get("robots_block_ai_search").and_then(|v| v.as_bool()));
        if let Some(robots_block_ai_search) = ai_policy_block_search {
            cfg.robots_block_ai_search = robots_block_ai_search;
            changed = true;
        }
        let ai_policy_allow_search_engines = json
            .get("ai_policy_allow_search_engines")
            .and_then(|v| v.as_bool())
            .or_else(|| {
                json.get("robots_allow_search_engines")
                    .and_then(|v| v.as_bool())
            });
        if let Some(robots_allow_search_engines) = ai_policy_allow_search_engines {
            cfg.robots_allow_search_engines = robots_allow_search_engines;
            changed = true;
        }
        if let Some(robots_crawl_delay) = json.get("robots_crawl_delay").and_then(|v| v.as_u64()) {
            cfg.robots_crawl_delay = robots_crawl_delay as u32;
            changed = true;
        }

        // Update CDP detection settings if provided
        if let Some(cdp_detection_enabled) =
            json.get("cdp_detection_enabled").and_then(|v| v.as_bool())
        {
            cfg.cdp_detection_enabled = cdp_detection_enabled;
            changed = true;
        }
        if let Some(cdp_auto_ban) = json.get("cdp_auto_ban").and_then(|v| v.as_bool()) {
            cfg.cdp_auto_ban = cdp_auto_ban;
            changed = true;
        }
        if let Some(cdp_detection_threshold) =
            json.get("cdp_detection_threshold").and_then(|v| v.as_f64())
        {
            cfg.cdp_detection_threshold = cdp_detection_threshold as f32;
            changed = true;
        }
        if let Some(value) = json.get("cdp_probe_family") {
            cfg.cdp_probe_family = match parse_cdp_probe_family_json("cdp_probe_family", value) {
                Ok(family) => family,
                Err(msg) => return Response::new(400, msg),
            };
            changed = true;
        }
        if let Some(value) = json.get("cdp_probe_rollout_percent").and_then(|v| v.as_u64()) {
            if value > 100 {
                return Response::new(400, "cdp_probe_rollout_percent out of range (0-100)");
            }
            cfg.cdp_probe_rollout_percent = value as u8;
            changed = true;
        }
        if let Some(value) = json.get("fingerprint_signal_enabled").and_then(|v| v.as_bool()) {
            cfg.fingerprint_signal_enabled = value;
            changed = true;
        }
        if let Some(value) = json
            .get("fingerprint_state_ttl_seconds")
            .and_then(|v| v.as_u64())
        {
            cfg.fingerprint_state_ttl_seconds = value;
            changed = true;
        }
        if let Some(value) = json
            .get("fingerprint_flow_window_seconds")
            .and_then(|v| v.as_u64())
        {
            cfg.fingerprint_flow_window_seconds = value;
            changed = true;
        }
        if let Some(value) = json
            .get("fingerprint_flow_violation_threshold")
            .and_then(|v| v.as_u64())
        {
            cfg.fingerprint_flow_violation_threshold = value as u8;
            changed = true;
        }
        if let Some(value) = json.get("fingerprint_pseudonymize").and_then(|v| v.as_bool()) {
            cfg.fingerprint_pseudonymize = value;
            changed = true;
        }
        if let Some(value) = json.get("fingerprint_entropy_budget").and_then(|v| v.as_u64()) {
            if value > 10 {
                return Response::new(400, "fingerprint_entropy_budget out of range (0-10)");
            }
            cfg.fingerprint_entropy_budget = value as u8;
            changed = true;
        }
        if let Some(value) = json
            .get("fingerprint_family_cap_header_runtime")
            .and_then(|v| v.as_u64())
        {
            if value > 10 {
                return Response::new(
                    400,
                    "fingerprint_family_cap_header_runtime out of range (0-10)",
                );
            }
            cfg.fingerprint_family_cap_header_runtime = value as u8;
            changed = true;
        }
        if let Some(value) = json
            .get("fingerprint_family_cap_transport")
            .and_then(|v| v.as_u64())
        {
            if value > 10 {
                return Response::new(400, "fingerprint_family_cap_transport out of range (0-10)");
            }
            cfg.fingerprint_family_cap_transport = value as u8;
            changed = true;
        }
        if let Some(value) = json
            .get("fingerprint_family_cap_temporal")
            .and_then(|v| v.as_u64())
        {
            if value > 10 {
                return Response::new(400, "fingerprint_family_cap_temporal out of range (0-10)");
            }
            cfg.fingerprint_family_cap_temporal = value as u8;
            changed = true;
        }
        if let Some(value) = json
            .get("fingerprint_family_cap_persistence")
            .and_then(|v| v.as_u64())
        {
            if value > 10 {
                return Response::new(
                    400,
                    "fingerprint_family_cap_persistence out of range (0-10)",
                );
            }
            cfg.fingerprint_family_cap_persistence = value as u8;
            changed = true;
        }
        if let Some(value) = json
            .get("fingerprint_family_cap_behavior")
            .and_then(|v| v.as_u64())
        {
            if value > 10 {
                return Response::new(400, "fingerprint_family_cap_behavior out of range (0-10)");
            }
            cfg.fingerprint_family_cap_behavior = value as u8;
            changed = true;
        }

        let old_pow_enabled = cfg.pow_enabled;
        let old_pow_difficulty = cfg.pow_difficulty;
        let old_pow_ttl = cfg.pow_ttl_seconds;
        let mut pow_changed = false;

        // Update PoW settings if provided.
        if let Some(pow_enabled) = json.get("pow_enabled").and_then(|v| v.as_bool()) {
            if cfg.pow_enabled != pow_enabled {
                cfg.pow_enabled = pow_enabled;
                changed = true;
                pow_changed = true;
            }
        }
        if let Some(pow_difficulty) = json.get("pow_difficulty").and_then(|v| v.as_u64()) {
            if pow_difficulty < POW_DIFFICULTY_MIN as u64
                || pow_difficulty > POW_DIFFICULTY_MAX as u64
            {
                return Response::new(400, "pow_difficulty out of range (12-20)");
            }
            cfg.pow_difficulty = pow_difficulty as u8;
            changed = true;
            pow_changed = true;
        }
        if let Some(pow_ttl_seconds) = json.get("pow_ttl_seconds").and_then(|v| v.as_u64()) {
            if pow_ttl_seconds < POW_TTL_MIN || pow_ttl_seconds > POW_TTL_MAX {
                return Response::new(400, "pow_ttl_seconds out of range (30-300)");
            }
            cfg.pow_ttl_seconds = pow_ttl_seconds;
            changed = true;
            pow_changed = true;
        }

        if pow_changed && !validate_only {
            log_event(
                store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("pow_config_update".to_string()),
                    outcome: Some(format!(
                        "enabled:{}->{} difficulty:{}->{} ttl:{}->{}",
                        old_pow_enabled,
                        cfg.pow_enabled,
                        old_pow_difficulty,
                        cfg.pow_difficulty,
                        old_pow_ttl,
                        cfg.pow_ttl_seconds
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
        }

        let old_challenge_puzzle_enabled = cfg.challenge_puzzle_enabled;
        let old_transform_count = cfg.challenge_puzzle_transform_count;
        let old_seed_ttl_seconds = cfg.challenge_puzzle_seed_ttl_seconds;
        let old_attempt_limit_per_window = cfg.challenge_puzzle_attempt_limit_per_window;
        let old_attempt_window_seconds = cfg.challenge_puzzle_attempt_window_seconds;
        let mut challenge_changed = false;
        if let Some(challenge_puzzle_enabled) = json.get("challenge_puzzle_enabled").and_then(|v| v.as_bool()) {
            if cfg.challenge_puzzle_enabled != challenge_puzzle_enabled {
                cfg.challenge_puzzle_enabled = challenge_puzzle_enabled;
                changed = true;
                challenge_changed = true;
            }
        }
        if let Some(transform_count) = json
            .get("challenge_puzzle_transform_count")
            .and_then(|v| v.as_u64())
        {
            if !(CHALLENGE_TRANSFORM_COUNT_MIN..=CHALLENGE_TRANSFORM_COUNT_MAX)
                .contains(&transform_count)
            {
                return Response::new(400, "challenge_puzzle_transform_count out of range (4-8)");
            }
            let next = transform_count as u8;
            if cfg.challenge_puzzle_transform_count != next {
                cfg.challenge_puzzle_transform_count = next;
                changed = true;
                challenge_changed = true;
            }
        }
        if let Some(seed_ttl_seconds) = json
            .get("challenge_puzzle_seed_ttl_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(CHALLENGE_PUZZLE_SEED_TTL_MIN..=CHALLENGE_PUZZLE_SEED_TTL_MAX)
                .contains(&seed_ttl_seconds)
            {
                return Response::new(400, "challenge_puzzle_seed_ttl_seconds out of range (30-300)");
            }
            if cfg.challenge_puzzle_seed_ttl_seconds != seed_ttl_seconds {
                cfg.challenge_puzzle_seed_ttl_seconds = seed_ttl_seconds;
                changed = true;
                challenge_changed = true;
            }
        }
        if let Some(attempt_limit_per_window) = json
            .get("challenge_puzzle_attempt_limit_per_window")
            .and_then(|v| v.as_u64())
        {
            if !(CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MIN..=CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MAX)
                .contains(&attempt_limit_per_window)
            {
                return Response::new(
                    400,
                    "challenge_puzzle_attempt_limit_per_window out of range (1-100)",
                );
            }
            let next = attempt_limit_per_window as u32;
            if cfg.challenge_puzzle_attempt_limit_per_window != next {
                cfg.challenge_puzzle_attempt_limit_per_window = next;
                changed = true;
                challenge_changed = true;
            }
        }
        if let Some(attempt_window_seconds) = json
            .get("challenge_puzzle_attempt_window_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MIN..=CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MAX)
                .contains(&attempt_window_seconds)
            {
                return Response::new(
                    400,
                    "challenge_puzzle_attempt_window_seconds out of range (30-3600)",
                );
            }
            if cfg.challenge_puzzle_attempt_window_seconds != attempt_window_seconds {
                cfg.challenge_puzzle_attempt_window_seconds = attempt_window_seconds;
                changed = true;
                challenge_changed = true;
            }
        }
        if challenge_changed && !validate_only {
            log_event(
                store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("challenge_config_update".to_string()),
                    outcome: Some(format!(
                        "enabled:{}->{} transform_count:{}->{} seed_ttl:{}->{} attempt_limit:{}->{} attempt_window:{}->{}",
                        old_challenge_puzzle_enabled,
                        cfg.challenge_puzzle_enabled,
                        old_transform_count,
                        cfg.challenge_puzzle_transform_count,
                        old_seed_ttl_seconds,
                        cfg.challenge_puzzle_seed_ttl_seconds,
                        old_attempt_limit_per_window,
                        cfg.challenge_puzzle_attempt_limit_per_window,
                        old_attempt_window_seconds,
                        cfg.challenge_puzzle_attempt_window_seconds
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
        }

        let old_not_a_bot_enabled = cfg.not_a_bot_enabled;
        let old_not_a_bot_threshold = cfg.not_a_bot_risk_threshold;
        let old_not_a_bot_pass_score = cfg.not_a_bot_pass_score;
        let old_not_a_bot_fail_score = cfg.not_a_bot_fail_score;
        let old_not_a_bot_nonce_ttl_seconds = cfg.not_a_bot_nonce_ttl_seconds;
        let old_not_a_bot_marker_ttl_seconds = cfg.not_a_bot_marker_ttl_seconds;
        let old_not_a_bot_attempt_limit_per_window = cfg.not_a_bot_attempt_limit_per_window;
        let old_not_a_bot_attempt_window_seconds = cfg.not_a_bot_attempt_window_seconds;
        let mut not_a_bot_changed = false;

        if let Some(not_a_bot_enabled) = json.get("not_a_bot_enabled").and_then(|v| v.as_bool()) {
            if cfg.not_a_bot_enabled != not_a_bot_enabled {
                cfg.not_a_bot_enabled = not_a_bot_enabled;
                changed = true;
                not_a_bot_changed = true;
            }
        }
        if let Some(value) = json
            .get("not_a_bot_risk_threshold")
            .and_then(|v| v.as_u64())
        {
            if !(NOT_A_BOT_THRESHOLD_MIN..=NOT_A_BOT_THRESHOLD_MAX).contains(&value) {
                return Response::new(400, "not_a_bot_risk_threshold out of range (1-10)");
            }
            let next = value as u8;
            if cfg.not_a_bot_risk_threshold != next {
                cfg.not_a_bot_risk_threshold = next;
                changed = true;
                not_a_bot_changed = true;
            }
        }
        if let Some(value) = json
            .get("not_a_bot_pass_score")
            .and_then(|v| v.as_u64())
        {
            if !(NOT_A_BOT_SCORE_MIN..=NOT_A_BOT_SCORE_MAX).contains(&value) {
                return Response::new(400, "not_a_bot_pass_score out of range (1-10)");
            }
            let next = value as u8;
            if cfg.not_a_bot_pass_score != next {
                cfg.not_a_bot_pass_score = next;
                changed = true;
                not_a_bot_changed = true;
            }
        }
        if let Some(value) = json
            .get("not_a_bot_fail_score")
            .and_then(|v| v.as_u64())
        {
            if !(NOT_A_BOT_SCORE_MIN..=NOT_A_BOT_SCORE_MAX).contains(&value) {
                return Response::new(400, "not_a_bot_fail_score out of range (1-10)");
            }
            let next = value as u8;
            if cfg.not_a_bot_fail_score != next {
                cfg.not_a_bot_fail_score = next;
                changed = true;
                not_a_bot_changed = true;
            }
        }
        if cfg.not_a_bot_fail_score > cfg.not_a_bot_pass_score {
            return Response::new(
                400,
                "not_a_bot_fail_score must be <= not_a_bot_pass_score",
            );
        }
        if let Some(value) = json
            .get("not_a_bot_nonce_ttl_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(NOT_A_BOT_NONCE_TTL_MIN..=NOT_A_BOT_NONCE_TTL_MAX).contains(&value) {
                return Response::new(400, "not_a_bot_nonce_ttl_seconds out of range (30-300)");
            }
            if cfg.not_a_bot_nonce_ttl_seconds != value {
                cfg.not_a_bot_nonce_ttl_seconds = value;
                changed = true;
                not_a_bot_changed = true;
            }
        }
        if let Some(value) = json
            .get("not_a_bot_marker_ttl_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(NOT_A_BOT_MARKER_TTL_MIN..=NOT_A_BOT_MARKER_TTL_MAX).contains(&value) {
                return Response::new(400, "not_a_bot_marker_ttl_seconds out of range (60-3600)");
            }
            if cfg.not_a_bot_marker_ttl_seconds != value {
                cfg.not_a_bot_marker_ttl_seconds = value;
                changed = true;
                not_a_bot_changed = true;
            }
        }
        if let Some(value) = json
            .get("not_a_bot_attempt_limit_per_window")
            .and_then(|v| v.as_u64())
        {
            if !(NOT_A_BOT_ATTEMPT_LIMIT_MIN..=NOT_A_BOT_ATTEMPT_LIMIT_MAX).contains(&value) {
                return Response::new(
                    400,
                    "not_a_bot_attempt_limit_per_window out of range (1-100)",
                );
            }
            let next = value as u32;
            if cfg.not_a_bot_attempt_limit_per_window != next {
                cfg.not_a_bot_attempt_limit_per_window = next;
                changed = true;
                not_a_bot_changed = true;
            }
        }
        if let Some(value) = json
            .get("not_a_bot_attempt_window_seconds")
            .and_then(|v| v.as_u64())
        {
            if !(NOT_A_BOT_ATTEMPT_WINDOW_MIN..=NOT_A_BOT_ATTEMPT_WINDOW_MAX).contains(&value) {
                return Response::new(
                    400,
                    "not_a_bot_attempt_window_seconds out of range (30-3600)",
                );
            }
            if cfg.not_a_bot_attempt_window_seconds != value {
                cfg.not_a_bot_attempt_window_seconds = value;
                changed = true;
                not_a_bot_changed = true;
            }
        }

        if not_a_bot_changed && !validate_only {
            log_event(
                store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("not_a_bot_config_update".to_string()),
                    outcome: Some(format!(
                        "enabled:{}->{} threshold:{}->{} score_pass:{}->{} score_escalate:{}->{} nonce_ttl:{}->{} marker_ttl:{}->{} attempts:{}->{} window:{}->{}",
                        old_not_a_bot_enabled,
                        cfg.not_a_bot_enabled,
                        old_not_a_bot_threshold,
                        cfg.not_a_bot_risk_threshold,
                        old_not_a_bot_pass_score,
                        cfg.not_a_bot_pass_score,
                        old_not_a_bot_fail_score,
                        cfg.not_a_bot_fail_score,
                        old_not_a_bot_nonce_ttl_seconds,
                        cfg.not_a_bot_nonce_ttl_seconds,
                        old_not_a_bot_marker_ttl_seconds,
                        cfg.not_a_bot_marker_ttl_seconds,
                        old_not_a_bot_attempt_limit_per_window,
                        cfg.not_a_bot_attempt_limit_per_window,
                        old_not_a_bot_attempt_window_seconds,
                        cfg.not_a_bot_attempt_window_seconds
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
        }

        let mut provider_selection_changed = false;
        let old_provider_backends = cfg.provider_backends.clone();
        let old_edge_integration_mode = cfg.edge_integration_mode;

        if let Some(provider_backends) = json.get("provider_backends") {
            let Some(backends_obj) = provider_backends.as_object() else {
                return Response::new(
                    400,
                    "provider_backends must be an object with optional keys: rate_limiter, ban_store, challenge_engine, maze_tarpit, fingerprint_signal",
                );
            };
            for key in backends_obj.keys() {
                if !matches!(
                    key.as_str(),
                    "rate_limiter"
                        | "ban_store"
                        | "challenge_engine"
                        | "maze_tarpit"
                        | "fingerprint_signal"
                ) {
                    return Response::new(
                        400,
                        format!("provider_backends.{} is not supported", key),
                    );
                }
            }

            if let Some(value) = backends_obj.get("rate_limiter") {
                cfg.provider_backends.rate_limiter =
                    match parse_provider_backend_json("provider_backends.rate_limiter", value) {
                        Ok(backend) => backend,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                provider_selection_changed = true;
            }
            if let Some(value) = backends_obj.get("ban_store") {
                cfg.provider_backends.ban_store =
                    match parse_provider_backend_json("provider_backends.ban_store", value) {
                        Ok(backend) => backend,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                provider_selection_changed = true;
            }
            if let Some(value) = backends_obj.get("challenge_engine") {
                cfg.provider_backends.challenge_engine = match parse_provider_backend_json(
                    "provider_backends.challenge_engine",
                    value,
                ) {
                    Ok(backend) => backend,
                    Err(msg) => return Response::new(400, msg),
                };
                changed = true;
                provider_selection_changed = true;
            }
            if let Some(value) = backends_obj.get("maze_tarpit") {
                cfg.provider_backends.maze_tarpit =
                    match parse_provider_backend_json("provider_backends.maze_tarpit", value) {
                        Ok(backend) => backend,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                provider_selection_changed = true;
            }
            if let Some(value) = backends_obj.get("fingerprint_signal") {
                cfg.provider_backends.fingerprint_signal = match parse_provider_backend_json(
                    "provider_backends.fingerprint_signal",
                    value,
                ) {
                    Ok(backend) => backend,
                    Err(msg) => return Response::new(400, msg),
                };
                changed = true;
                provider_selection_changed = true;
            }
        }

        if let Some(value) = json.get("edge_integration_mode") {
            cfg.edge_integration_mode =
                match parse_edge_integration_mode_json("edge_integration_mode", value) {
                    Ok(mode) => mode,
                    Err(msg) => return Response::new(400, msg),
                };
            changed = true;
            provider_selection_changed = true;
        }

        if provider_selection_changed && !validate_only {
            log_event(
                store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("provider_selection_update".to_string()),
                    outcome: Some(format!(
                        "providers(rate_limiter:{}->{} ban_store:{}->{} challenge_engine:{}->{} maze_tarpit:{}->{} fingerprint_signal:{}->{}) edge:{}->{}",
                        old_provider_backends.rate_limiter.as_str(),
                        cfg.provider_backends.rate_limiter.as_str(),
                        old_provider_backends.ban_store.as_str(),
                        cfg.provider_backends.ban_store.as_str(),
                        old_provider_backends.challenge_engine.as_str(),
                        cfg.provider_backends.challenge_engine.as_str(),
                        old_provider_backends.maze_tarpit.as_str(),
                        cfg.provider_backends.maze_tarpit.as_str(),
                        old_provider_backends.fingerprint_signal.as_str(),
                        cfg.provider_backends.fingerprint_signal.as_str(),
                        old_edge_integration_mode.as_str(),
                        cfg.edge_integration_mode.as_str(),
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
        }

        let mut botness_changed = false;
        let old_challenge_threshold = cfg.challenge_puzzle_risk_threshold;
        let old_maze_threshold = cfg.botness_maze_threshold;
        let old_weights = cfg.botness_weights.clone();
        let old_modes = cfg.defence_modes.clone();
        if let Some(challenge_threshold) = json
            .get("challenge_puzzle_risk_threshold")
            .and_then(|v| v.as_u64())
        {
            if challenge_threshold < 1 || challenge_threshold > 10 {
                return Response::new(400, "challenge_puzzle_risk_threshold out of range (1-10)");
            }
            cfg.challenge_puzzle_risk_threshold = challenge_threshold as u8;
            changed = true;
            botness_changed = true;
        }
        if let Some(maze_threshold) = json.get("botness_maze_threshold").and_then(|v| v.as_u64()) {
            if maze_threshold < 1 || maze_threshold > 10 {
                return Response::new(400, "botness_maze_threshold out of range (1-10)");
            }
            cfg.botness_maze_threshold = maze_threshold as u8;
            changed = true;
            botness_changed = true;
        }
        if let Some(weights) = json.get("botness_weights") {
            if let Some(js_required) = weights.get("js_required").and_then(|v| v.as_u64()) {
                if js_required > 10 {
                    return Response::new(400, "botness_weights.js_required out of range (0-10)");
                }
                cfg.botness_weights.js_required = js_required as u8;
                changed = true;
                botness_changed = true;
            }
            if let Some(geo_risk) = weights.get("geo_risk").and_then(|v| v.as_u64()) {
                if geo_risk > 10 {
                    return Response::new(400, "botness_weights.geo_risk out of range (0-10)");
                }
                cfg.botness_weights.geo_risk = geo_risk as u8;
                changed = true;
                botness_changed = true;
            }
            if let Some(rate_medium) = weights.get("rate_medium").and_then(|v| v.as_u64()) {
                if rate_medium > 10 {
                    return Response::new(400, "botness_weights.rate_medium out of range (0-10)");
                }
                cfg.botness_weights.rate_medium = rate_medium as u8;
                changed = true;
                botness_changed = true;
            }
            if let Some(rate_high) = weights.get("rate_high").and_then(|v| v.as_u64()) {
                if rate_high > 10 {
                    return Response::new(400, "botness_weights.rate_high out of range (0-10)");
                }
                cfg.botness_weights.rate_high = rate_high as u8;
                changed = true;
                botness_changed = true;
            }
            if let Some(maze_behavior) = weights.get("maze_behavior").and_then(|v| v.as_u64()) {
                if maze_behavior > 10 {
                    return Response::new(400, "botness_weights.maze_behavior out of range (0-10)");
                }
                cfg.botness_weights.maze_behavior = maze_behavior as u8;
                changed = true;
                botness_changed = true;
            }
        }
        if let Some(defence_modes) = json.get("defence_modes") {
            let Some(modes_obj) = defence_modes.as_object() else {
                return Response::new(
                    400,
                    "defence_modes must be an object with optional keys: rate, geo, js",
                );
            };
            for key in modes_obj.keys() {
                if !matches!(key.as_str(), "rate" | "geo" | "js") {
                    return Response::new(400, format!("defence_modes.{} is not supported", key));
                }
            }

            if let Some(value) = modes_obj.get("rate") {
                cfg.defence_modes.rate =
                    match parse_composability_mode_json("defence_modes.rate", value) {
                        Ok(mode) => mode,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                botness_changed = true;
            }
            if let Some(value) = modes_obj.get("geo") {
                cfg.defence_modes.geo =
                    match parse_composability_mode_json("defence_modes.geo", value) {
                        Ok(mode) => mode,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                botness_changed = true;
            }
            if let Some(value) = modes_obj.get("js") {
                cfg.defence_modes.js =
                    match parse_composability_mode_json("defence_modes.js", value) {
                        Ok(mode) => mode,
                        Err(msg) => return Response::new(400, msg),
                    };
                changed = true;
                botness_changed = true;
            }
        }

        if cfg.challenge_puzzle_risk_threshold > 1
            && cfg.not_a_bot_risk_threshold >= cfg.challenge_puzzle_risk_threshold
        {
            return Response::new(
                400,
                "not_a_bot_risk_threshold must be lower than challenge_puzzle_risk_threshold",
            );
        }

        if botness_changed && !validate_only {
            log_event(store, &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("botness_config_update".to_string()),
                    outcome: Some(format!(
                        "challenge:{}->{} maze:{}->{} weights(js:{}->{} geo:{}->{} rate_med:{}->{} rate_high:{}->{} maze_behavior:{}->{}) modes(rate:{:?}->{:?} geo:{:?}->{:?} js:{:?}->{:?})",
                        old_challenge_threshold,
                        cfg.challenge_puzzle_risk_threshold,
                        old_maze_threshold,
                        cfg.botness_maze_threshold,
                        old_weights.js_required,
                        cfg.botness_weights.js_required,
                        old_weights.geo_risk,
                        cfg.botness_weights.geo_risk,
                        old_weights.rate_medium,
                        cfg.botness_weights.rate_medium,
                        old_weights.rate_high,
                        cfg.botness_weights.rate_high,
                        old_weights.maze_behavior,
                        cfg.botness_weights.maze_behavior,
                        old_modes.rate,
                        cfg.defence_modes.rate,
                        old_modes.geo,
                        cfg.defence_modes.geo,
                        old_modes.js,
                        cfg.defence_modes.js
                    )),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                });
        }

        // Save config to KV store.
        if changed && !validate_only {
            if persist_site_config(store, site_id, &cfg).is_err() {
                return Response::new(500, "Key-value store error");
            }
        }

        let challenge_default = challenge_threshold_default();
        let not_a_bot_default = not_a_bot_threshold_default();
        let maze_default = maze_threshold_default();

        let mut effective_cfg = cfg.clone();
        crate::config::apply_runtime_ephemeral_overrides(site_id, &mut effective_cfg);
        effective_cfg.adversary_sim_enabled = effective_adversary_sim_enabled;
        let body = serde_json::to_string(&json!({
            "status": "updated",
            "config": admin_config_payload(&effective_cfg, challenge_default, not_a_bot_default, maze_default)
        }))
        .unwrap();
        return Response::new(200, body);
    }
    // GET: Return current config
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: None,
            reason: Some("config_view".to_string()),
            outcome: Some(format!("test_mode={}", cfg.test_mode)),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );
    let challenge_default = challenge_threshold_default();
    let not_a_bot_default = not_a_bot_threshold_default();
    let maze_default = maze_threshold_default();
    let body =
        serde_json::to_string(&admin_config_payload(
            &cfg,
            challenge_default,
            not_a_bot_default,
            maze_default,
        ))
            .unwrap();
    Response::new(200, body)
}

fn handle_admin_config(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    handle_admin_config_internal(req, store, site_id, false)
}

fn handle_admin_config_validate(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != spin_sdk::http::Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let patch = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_ADMIN_JSON_BYTES,
    ) {
        Ok(value) => value,
        Err(err) => {
            let issue = admin_config_validation_issue(
                &serde_json::Value::Null,
                format!("Invalid config payload: {}", err),
            );
            let body = serde_json::to_string(&json!({
                "valid": false,
                "issues": [issue]
            }))
            .unwrap();
            return Response::new(200, body);
        }
    };

    if let Err(err) = validate_admin_config_patch_shape(&patch) {
        let issue = admin_config_validation_issue(&patch, err);
        let body = serde_json::to_string(&json!({
            "valid": false,
            "issues": [issue]
        }))
        .unwrap();
        return Response::new(200, body);
    }

    let validation_response = handle_admin_config_internal(req, store, site_id, true);
    let status = *validation_response.status();
    if status == 200 {
        return Response::new(200, r#"{"valid":true,"issues":[]}"#);
    }

    let message = String::from_utf8_lossy(validation_response.body()).to_string();
    if status == 400 {
        let issue = admin_config_validation_issue(&patch, message.clone());
        let body = serde_json::to_string(&json!({
            "valid": false,
            "issues": [issue]
        }))
        .unwrap();
        return Response::new(200, body);
    }

    Response::new(status, message)
}

fn parse_operator_seed_sources_json(
    value: &serde_json::Value,
) -> Result<Vec<crate::maze::seeds::OperatorSeedSource>, String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "sources must be an array".to_string())?;
    let mut sources = Vec::with_capacity(entries.len());
    for entry in entries {
        let obj = entry
            .as_object()
            .ok_or_else(|| "each source must be an object".to_string())?;
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = obj
            .get("url")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let title = obj
            .get("title")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let keywords = obj
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let allow_seed_use = obj
            .get("allow_seed_use")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let robots_allowed = obj
            .get("robots_allowed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let body_excerpt = obj
            .get("body_excerpt")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        sources.push(crate::maze::seeds::OperatorSeedSource {
            id,
            url,
            title,
            description,
            keywords,
            allow_seed_use,
            robots_allowed,
            body_excerpt,
        });
    }
    Ok(sources)
}

fn handle_admin_maze_seed_sources<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore + crate::maze::state::MazeStateStore,
{
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };

    match *req.method() {
        Method::Get => {
            let sources = crate::maze::seeds::list_operator_sources(store);
            let cache = crate::maze::seeds::cached_seed_snapshot(store);
            let body = serde_json::to_string(&json!({
                "seed_provider": cfg.maze_seed_provider,
                "seed_refresh_interval_seconds": cfg.maze_seed_refresh_interval_seconds,
                "seed_refresh_rate_limit_per_hour": cfg.maze_seed_refresh_rate_limit_per_hour,
                "seed_refresh_max_sources": cfg.maze_seed_refresh_max_sources,
                "seed_metadata_only": cfg.maze_seed_metadata_only,
                "sources": sources,
                "cache": cache
            }))
            .unwrap();
            Response::new(200, body)
        }
        Method::Post => {
            let payload = match crate::request_validation::parse_json_body(
                req.body(),
                crate::request_validation::MAX_ADMIN_JSON_BYTES,
            ) {
                Ok(payload) => payload,
                Err(err) => return Response::new(400, err),
            };
            let Some(value) = payload.get("sources") else {
                return Response::new(400, "sources field is required");
            };
            let sources = match parse_operator_seed_sources_json(value) {
                Ok(sources) => sources,
                Err(err) => return Response::new(400, err),
            };
            if let Err(err) =
                crate::maze::seeds::save_operator_sources(store, &cfg, sources.clone())
            {
                return Response::new(400, err);
            }
            log_event(
                store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("maze_seed_sources_update".to_string()),
                    outcome: Some(format!("sources={}", sources.len())),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            let body = serde_json::to_string(&json!({
                "updated": true,
                "source_count": sources.len()
            }))
            .unwrap();
            Response::new(200, body)
        }
        _ => Response::new(405, "Method Not Allowed"),
    }
}

fn handle_admin_maze_preview<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let requested_path = crate::request_validation::query_param(req.query(), "path");
    let html = crate::maze::preview::render_admin_preview(&cfg, requested_path.as_deref());
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-store")
        .body(html)
        .build()
}

fn handle_admin_tarpit_preview<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };

    crate::tarpit::runtime::build_progressive_entry_response(
        &cfg,
        "admin-preview-ip-bucket",
        "admin-preview-ua-bucket",
        "/admin/tarpit/preview",
        crate::tarpit::progress_path(),
    )
}

fn handle_admin_maze_seed_refresh<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore + crate::maze::state::MazeStateStore,
{
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    if cfg.maze_seed_provider != crate::config::MazeSeedProvider::Operator {
        return Response::new(
            409,
            "maze_seed_provider must be 'operator' for manual seed refresh",
        );
    }

    let now = now_ts();
    let refreshed = match crate::maze::seeds::manual_refresh_operator_corpus(store, &cfg, now) {
        Ok(refreshed) => refreshed,
        Err(err) => {
            if err.contains("rate limit exceeded") {
                return Response::new(429, err);
            }
            return Response::new(400, err);
        }
    };
    log_event(
        store,
        &EventLogEntry {
            ts: now,
            event: EventType::AdminAction,
            ip: None,
            reason: Some("maze_seed_refresh".to_string()),
            outcome: Some(format!(
                "provider={} version={} terms={} sources={}",
                refreshed.provider,
                refreshed.version,
                refreshed.terms.len(),
                refreshed.source_count
            )),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );
    let body = serde_json::to_string(&json!({
        "refreshed": true,
        "provider": refreshed.provider,
        "version": refreshed.version,
        "metadata_only": refreshed.metadata_only,
        "source_count": refreshed.source_count,
        "term_count": refreshed.terms.len()
    }))
    .unwrap();
    Response::new(200, body)
}

fn handle_admin_monitoring<S>(req: &Request, store: &S) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    let hours = query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = query_u64_param(req.query(), "limit", 10).clamp(1, 50) as usize;
    let forensic_mode = forensic_access_mode(req.query());
    let now = now_ts();
    let query_budget = monitoring_query_budget(hours, limit);
    let summary = crate::observability::monitoring::summarize_with_store(store, hours, limit);
    let details = monitoring_details_payload(store, "default", hours, limit, forensic_mode);
    let snapshot_latest_ts = latest_monitoring_snapshot_ts(&details);
    let freshness = freshness_health_payload(now, snapshot_latest_ts, false, "none", "snapshot_poll");
    let retention_health = crate::observability::retention::retention_health(store);
    let security_privacy = details
        .get("security_privacy")
        .cloned()
        .unwrap_or_else(|| security_privacy_payload(store, now, hours, forensic_mode));
    let mut payload = json!({
        "summary": summary,
        "prometheus": monitoring_prometheus_helper_payload(),
        "details": details,
        "freshness_slo": freshness_slo_payload(),
        "load_envelope": load_envelope_payload(),
        "freshness": freshness,
        "retention_health": retention_health,
        "security_privacy": security_privacy
    });

    let supports_gzip = request_accepts_gzip(req);
    let initial_uncompressed =
        serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());
    let initial_payload_kb = (initial_uncompressed.len() as f64) / 1024.0;
    let initial_compression =
        monitoring_compression_report(initial_uncompressed.as_slice(), supports_gzip);
    update_monitoring_cost_governance_transport_fields(
        &mut payload,
        initial_payload_kb,
        &initial_compression,
    );

    let mut uncompressed = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());
    let final_payload_kb = (uncompressed.len() as f64) / 1024.0;
    let final_compression = monitoring_compression_report(uncompressed.as_slice(), supports_gzip);
    update_monitoring_cost_governance_transport_fields(
        &mut payload,
        final_payload_kb,
        &final_compression,
    );
    uncompressed = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());

    let body = if final_compression.negotiated {
        gzip_bytes(uncompressed.as_slice()).unwrap_or_else(|| uncompressed.clone())
    } else {
        uncompressed
    };
    let cost_state = payload
        .get("details")
        .and_then(|details| details.get("cost_governance"))
        .and_then(|cost| cost.get("degraded_state"))
        .and_then(|value| value.as_str())
        .unwrap_or("normal");

    let mut builder = Response::builder();
    builder
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("X-Shuma-Monitoring-Cost-State", cost_state)
        .header("X-Shuma-Monitoring-Security-Mode", security_view_mode_label(forensic_mode))
        .header("X-Shuma-Monitoring-Query-Budget", query_budget.status);
    if final_compression.negotiated {
        builder
            .header("Content-Encoding", "gzip")
            .header("Vary", "Accept-Encoding");
    }
    builder.body(body).build()
}

fn handle_admin_monitoring_delta<S>(req: &Request, store: &S) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    let hours = query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = query_u64_param(req.query(), "limit", 100)
        .clamp(1, MONITORING_STREAM_MAX_BUFFER_EVENTS as u64) as usize;
    let forensic_mode = forensic_access_mode(req.query());
    let after_cursor = resolve_after_cursor(req);
    if let Err(msg) = validate_after_cursor(after_cursor.as_str()) {
        return Response::new(400, msg);
    }

    let now = now_ts();
    let rows: Vec<CursorEventRecord> = load_recent_event_records_with_keys(store, now, hours)
        .into_iter()
        .map(|stored| CursorEventRecord {
            cursor: build_event_cursor(stored.record.entry.ts, stored.storage_key.as_str()),
            record: present_event_record(&stored.record, forensic_mode),
        })
        .collect();
    let latest_window_ts = latest_event_ts(rows.as_slice());
    let window_end_cursor = rows
        .iter()
        .map(|row: &CursorEventRecord| row.cursor.clone())
        .max()
        .unwrap_or_default();
    let (page_rows, next_cursor, has_more, overflow) =
        paginate_cursor_rows(rows, after_cursor.as_str(), limit);
    let event_rows: Vec<serde_json::Value> = page_rows.iter().map(cursor_event_row_payload).collect();
    let etag = delta_page_etag(next_cursor.as_str(), event_rows.len(), has_more, overflow);
    let freshness = freshness_health_payload(
        now,
        latest_window_ts,
        has_more,
        overflow,
        "cursor_delta_poll",
    );

    if request_if_none_match(req).as_deref() == Some(etag.as_str()) {
        return Response::builder()
            .status(304)
            .header("Cache-Control", "no-store")
            .header("ETag", etag.as_str())
            .body("")
            .build();
    }

    let body = serde_json::to_string(&json!({
        "cursor_contract": {
            "version": "monitoring-event-cursor.v1",
            "ordering": "strict_monotonic_cursor_ascending",
            "cursor_source": "eventlog:v2 key ordering",
            "overflow_taxonomy": ["none", "limit_exceeded"]
        },
        "security_mode": security_view_mode_label(forensic_mode),
        "security_privacy": security_privacy_payload(store, now, hours, forensic_mode),
        "freshness_slo": freshness_slo_payload(),
        "load_envelope": load_envelope_payload(),
        "hours": hours,
        "limit": limit,
        "after_cursor": after_cursor,
        "window_end_cursor": window_end_cursor,
        "next_cursor": next_cursor,
        "has_more": has_more,
        "overflow": overflow,
        "events": event_rows,
        "freshness": freshness,
        "stream_supported": true,
        "stream_endpoint": "/admin/monitoring/stream"
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("ETag", etag.as_str())
        .header("X-Shuma-Monitoring-Security-Mode", security_view_mode_label(forensic_mode))
        .body(body)
        .build()
}

fn handle_admin_monitoring_stream<S>(req: &Request, store: &S) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let hours = query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = query_u64_param(req.query(), "limit", 100)
        .clamp(1, MONITORING_STREAM_MAX_BUFFER_EVENTS as u64) as usize;
    let forensic_mode = forensic_access_mode(req.query());
    let after_cursor = resolve_after_cursor(req);
    if let Err(msg) = validate_after_cursor(after_cursor.as_str()) {
        return Response::new(400, msg);
    }

    let now = now_ts();
    let rows: Vec<CursorEventRecord> = load_recent_event_records_with_keys(store, now, hours)
        .into_iter()
        .map(|stored| CursorEventRecord {
            cursor: build_event_cursor(stored.record.entry.ts, stored.storage_key.as_str()),
            record: present_event_record(&stored.record, forensic_mode),
        })
        .collect();
    let latest_window_ts = latest_event_ts(rows.as_slice());
    let window_end_cursor = rows
        .iter()
        .map(|row: &CursorEventRecord| row.cursor.clone())
        .max()
        .unwrap_or_default();
    let (page_rows, next_cursor, has_more, overflow) =
        paginate_cursor_rows(rows, after_cursor.as_str(), limit);
    let freshness = freshness_health_payload(
        now,
        latest_window_ts,
        has_more,
        overflow,
        "sse",
    );
    let event_rows: Vec<serde_json::Value> = page_rows.iter().map(cursor_event_row_payload).collect();
    let payload = json!({
        "cursor_contract": {
            "version": "monitoring-event-cursor.v1",
            "ordering": "strict_monotonic_cursor_ascending",
            "cursor_source": "eventlog:v2 key ordering",
            "overflow_taxonomy": ["none", "limit_exceeded"]
        },
        "security_mode": security_view_mode_label(forensic_mode),
        "security_privacy": security_privacy_payload(store, now, hours, forensic_mode),
        "freshness_slo": freshness_slo_payload(),
        "load_envelope": load_envelope_payload(),
        "stream_contract": stream_contract_payload(),
        "hours": hours,
        "limit": limit,
        "after_cursor": after_cursor,
        "window_end_cursor": window_end_cursor,
        "next_cursor": next_cursor,
        "has_more": has_more,
        "overflow": overflow,
        "events": event_rows,
        "freshness": freshness
    });
    let event_id = payload
        .get("next_cursor")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("");
    sse_single_event_response("monitoring_delta", event_id, &payload)
}

fn handle_admin_ip_bans_delta<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    let hours = query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = query_u64_param(req.query(), "limit", 100)
        .clamp(1, MONITORING_STREAM_MAX_BUFFER_EVENTS as u64) as usize;
    let forensic_mode = forensic_access_mode(req.query());
    let after_cursor = resolve_after_cursor(req);
    if let Err(msg) = validate_after_cursor(after_cursor.as_str()) {
        return Response::new(400, msg);
    }

    let now = now_ts();
    let rows: Vec<CursorEventRecord> = load_recent_event_records_with_keys(store, now, hours)
        .into_iter()
        .filter(|stored| matches!(stored.record.entry.event, EventType::Ban | EventType::Unban))
        .map(|stored| CursorEventRecord {
            cursor: build_event_cursor(stored.record.entry.ts, stored.storage_key.as_str()),
            record: present_event_record(&stored.record, forensic_mode),
        })
        .collect();
    let latest_window_ts = latest_event_ts(rows.as_slice());
    let window_end_cursor = rows
        .iter()
        .map(|row: &CursorEventRecord| row.cursor.clone())
        .max()
        .unwrap_or_default();
    let (page_rows, next_cursor, has_more, overflow) =
        paginate_cursor_rows(rows, after_cursor.as_str(), limit);
    let event_rows: Vec<serde_json::Value> = page_rows.iter().map(cursor_event_row_payload).collect();

    let mut active_bans: Vec<_> = crate::enforcement::ban::list_active_bans(store, site_id)
        .into_iter()
        .collect();
    active_bans.sort_by(|a, b| a.0.cmp(&b.0));
    let active_bans_payload: Vec<serde_json::Value> = active_bans
        .into_iter()
        .map(|(ip, ban)| {
            let display_ip = if forensic_mode {
                ip
            } else {
                pseudonymize_ip_identifier(ip.as_str())
            };
            json!({
                "ip": display_ip,
                "reason": ban.reason,
                "expires": ban.expires,
                "banned_at": ban.banned_at,
                "fingerprint": ban.fingerprint
            })
        })
        .collect();

    let etag = delta_page_etag(
        next_cursor.as_str(),
        event_rows.len().saturating_add(active_bans_payload.len()),
        has_more,
        overflow,
    );
    if request_if_none_match(req).as_deref() == Some(etag.as_str()) {
        return Response::builder()
            .status(304)
            .header("Cache-Control", "no-store")
        .header("ETag", etag.as_str())
        .body("")
        .build();
    }

    let latest_ban_ts = active_bans_payload
        .iter()
        .filter_map(|ban| ban.get("banned_at").and_then(|value| value.as_u64()))
        .max();
    let latest_ts = match (latest_window_ts, latest_ban_ts) {
        (Some(event_ts), Some(ban_ts)) => Some(event_ts.max(ban_ts)),
        (Some(event_ts), None) => Some(event_ts),
        (None, Some(ban_ts)) => Some(ban_ts),
        (None, None) => None,
    };
    let freshness = freshness_health_payload(now, latest_ts, has_more, overflow, "cursor_delta_poll");

    let body = serde_json::to_string(&json!({
        "cursor_contract": {
            "version": "monitoring-event-cursor.v1",
            "ordering": "strict_monotonic_cursor_ascending",
            "cursor_source": "eventlog:v2 key ordering",
            "overflow_taxonomy": ["none", "limit_exceeded"]
        },
        "security_mode": security_view_mode_label(forensic_mode),
        "security_privacy": security_privacy_payload(store, now, hours, forensic_mode),
        "freshness_slo": freshness_slo_payload(),
        "load_envelope": load_envelope_payload(),
        "hours": hours,
        "limit": limit,
        "after_cursor": after_cursor,
        "window_end_cursor": window_end_cursor,
        "next_cursor": next_cursor,
        "has_more": has_more,
        "overflow": overflow,
        "events": event_rows,
        "active_bans": active_bans_payload,
        "freshness": freshness,
        "stream_supported": true,
        "stream_endpoint": "/admin/ip-bans/stream"
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("ETag", etag.as_str())
        .header("X-Shuma-Monitoring-Security-Mode", security_view_mode_label(forensic_mode))
        .body(body)
        .build()
}

fn handle_admin_ip_bans_stream<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let hours = query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = query_u64_param(req.query(), "limit", 100)
        .clamp(1, MONITORING_STREAM_MAX_BUFFER_EVENTS as u64) as usize;
    let forensic_mode = forensic_access_mode(req.query());
    let after_cursor = resolve_after_cursor(req);
    if let Err(msg) = validate_after_cursor(after_cursor.as_str()) {
        return Response::new(400, msg);
    }

    let now = now_ts();
    let rows: Vec<CursorEventRecord> = load_recent_event_records_with_keys(store, now, hours)
        .into_iter()
        .filter(|stored| matches!(stored.record.entry.event, EventType::Ban | EventType::Unban))
        .map(|stored| CursorEventRecord {
            cursor: build_event_cursor(stored.record.entry.ts, stored.storage_key.as_str()),
            record: present_event_record(&stored.record, forensic_mode),
        })
        .collect();
    let latest_window_ts = latest_event_ts(rows.as_slice());
    let window_end_cursor = rows
        .iter()
        .map(|row: &CursorEventRecord| row.cursor.clone())
        .max()
        .unwrap_or_default();
    let (page_rows, next_cursor, has_more, overflow) =
        paginate_cursor_rows(rows, after_cursor.as_str(), limit);
    let event_rows: Vec<serde_json::Value> = page_rows.iter().map(cursor_event_row_payload).collect();

    let mut active_bans: Vec<_> = crate::enforcement::ban::list_active_bans(store, site_id)
        .into_iter()
        .collect();
    active_bans.sort_by(|a, b| a.0.cmp(&b.0));
    let active_bans_payload: Vec<serde_json::Value> = active_bans
        .into_iter()
        .map(|(ip, ban)| {
            let display_ip = if forensic_mode {
                ip
            } else {
                pseudonymize_ip_identifier(ip.as_str())
            };
            json!({
                "ip": display_ip,
                "reason": ban.reason,
                "expires": ban.expires,
                "banned_at": ban.banned_at,
                "fingerprint": ban.fingerprint
            })
        })
        .collect();
    let latest_ban_ts = active_bans_payload
        .iter()
        .filter_map(|ban| ban.get("banned_at").and_then(|value| value.as_u64()))
        .max();
    let latest_ts = match (latest_window_ts, latest_ban_ts) {
        (Some(event_ts), Some(ban_ts)) => Some(event_ts.max(ban_ts)),
        (Some(event_ts), None) => Some(event_ts),
        (None, Some(ban_ts)) => Some(ban_ts),
        (None, None) => None,
    };
    let freshness = freshness_health_payload(now, latest_ts, has_more, overflow, "sse");
    let payload = json!({
        "cursor_contract": {
            "version": "monitoring-event-cursor.v1",
            "ordering": "strict_monotonic_cursor_ascending",
            "cursor_source": "eventlog:v2 key ordering",
            "overflow_taxonomy": ["none", "limit_exceeded"]
        },
        "security_mode": security_view_mode_label(forensic_mode),
        "security_privacy": security_privacy_payload(store, now, hours, forensic_mode),
        "freshness_slo": freshness_slo_payload(),
        "load_envelope": load_envelope_payload(),
        "stream_contract": stream_contract_payload(),
        "hours": hours,
        "limit": limit,
        "after_cursor": after_cursor,
        "window_end_cursor": window_end_cursor,
        "next_cursor": next_cursor,
        "has_more": has_more,
        "overflow": overflow,
        "events": event_rows,
        "active_bans": active_bans_payload,
        "freshness": freshness
    });
    let event_id = payload
        .get("next_cursor")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("");
    sse_single_event_response("ip_bans_delta", event_id, &payload)
}

fn handle_admin_ip_range_suggestions<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let requested_hours = query_u64_param(req.query(), "hours", 24);
    let requested_limit = query_u64_param(req.query(), "limit", 20);
    let hours = crate::signals::ip_range_suggestions::normalize_suggestion_hours(requested_hours);
    let safe_limit_u64 = requested_limit.min(usize::MAX as u64);
    let limit =
        crate::signals::ip_range_suggestions::normalize_suggestion_limit(safe_limit_u64 as usize);
    let now = now_ts();
    let events = load_recent_events(store, now, hours);
    let payload = crate::signals::ip_range_suggestions::build_ip_range_suggestions(
        store, &cfg, &events, now, hours, limit,
    );

    let body = serde_json::to_string(&payload).unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

fn read_u64_counter<S>(store: &S, key: &str) -> u64
where
    S: crate::challenge::KeyValueStore,
{
    store
        .get(key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

#[derive(Debug, Default, Serialize)]
struct TelemetryHistoryCleanupResult {
    deleted_keys: u64,
    deleted_by_family: BTreeMap<String, u64>,
}

fn classify_dev_telemetry_history_key(
    key: &str,
    tarpit_global_key: &str,
    tarpit_bucket_prefix: &str,
) -> Option<&'static str> {
    if key.starts_with("eventlog:v2:") {
        return Some("eventlog");
    }
    if key.starts_with("monitoring:v1:") {
        return Some("monitoring");
    }
    if key.starts_with("metrics:") {
        return Some("metrics");
    }
    if key.starts_with("cdp:") {
        return Some("cdp");
    }
    if key.starts_with("fingerprint:") {
        return Some("fingerprint");
    }
    if key.starts_with("maze_hits:") {
        return Some("maze_hits");
    }
    if key == tarpit_global_key || key.starts_with(tarpit_bucket_prefix) {
        return Some("tarpit_active");
    }
    None
}

fn clear_dev_telemetry_history<S>(store: &S, site_id: &str) -> TelemetryHistoryCleanupResult
where
    S: crate::challenge::KeyValueStore,
{
    let tarpit_global_key = crate::providers::internal::tarpit_budget_global_active_key(site_id);
    let tarpit_bucket_prefix = format!(
        "{}:",
        crate::providers::internal::tarpit_budget_bucket_active_prefix(site_id)
    );
    let mut result = TelemetryHistoryCleanupResult::default();
    if let Ok(keys) = store.get_keys() {
        for key in keys {
            let Some(family) = classify_dev_telemetry_history_key(
                key.as_str(),
                tarpit_global_key.as_str(),
                tarpit_bucket_prefix.as_str(),
            ) else {
                continue;
            };
            if store.delete(key.as_str()).is_err() {
                continue;
            }
            result.deleted_keys = result.deleted_keys.saturating_add(1);
            let entry = result
                .deleted_by_family
                .entry(family.to_string())
                .or_insert(0);
            *entry = entry.saturating_add(1);
        }
    }
    result
}

fn monitoring_details_payload<S>(
    store: &S,
    site_id: &str,
    hours: u64,
    limit: usize,
    forensic_mode: bool,
) -> serde_json::Value
where
    S: crate::challenge::KeyValueStore,
{
    let now = now_ts();
    let mut events = load_recent_event_records(store, now, hours);
    let query_budget = monitoring_query_budget(hours, limit);
    let mut ip_counts = std::collections::HashMap::new();
    let mut event_counts = std::collections::HashMap::new();

    for entry in &events {
        if let Some(ip) = &entry.entry.ip {
            let key = if forensic_mode {
                ip.clone()
            } else {
                pseudonymize_ip_identifier(ip.as_str())
            };
            *ip_counts.entry(key).or_insert(0u32) += 1;
        }
        *event_counts
            .entry(format!("{:?}", entry.entry.event))
            .or_insert(0u32) += 1;
    }
    events.sort_by(|a, b| b.entry.ts.cmp(&a.entry.ts));
    let unique_ips = ip_counts.len();
    let mut top_ips: Vec<_> = ip_counts.into_iter().collect();
    top_ips.sort_by(|a, b| b.1.cmp(&a.1));
    let top_ips: Vec<_> = top_ips.into_iter().take(10).collect();
    let recent_event_cap = if query_budget.status == "exceeded" {
        20
    } else {
        (limit.saturating_mul(10)).clamp(20, 100)
    };
    let total_recent_events_in_window = events.len();
    let recent_events_raw: Vec<_> = events.iter().take(recent_event_cap).cloned().collect();
    let recent_events = present_event_records(recent_events_raw.as_slice(), forensic_mode);
    let recent_events_has_more = total_recent_events_in_window > recent_events.len();

    let cdp_events_limit = 500usize;
    let mut cdp_events: Vec<EventLogRecord> = events
        .iter()
        .filter(|entry| {
            entry
                .entry
                .reason
                .as_deref()
                .map(is_cdp_event_reason)
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    cdp_events.sort_by(|a, b| b.entry.ts.cmp(&a.entry.ts));
    let total_matches = cdp_events.len();
    let detections = cdp_events
        .iter()
        .filter(|entry| {
            entry
                .entry
                .reason
                .as_deref()
                .map(|reason| reason.to_lowercase().starts_with("cdp_detected:"))
                .unwrap_or(false)
        })
        .count();
    let auto_bans = cdp_events
        .iter()
        .filter(|entry| {
            entry
                .entry
                .reason
                .as_deref()
                .map(|reason| reason.eq_ignore_ascii_case("cdp_automation"))
                .unwrap_or(false)
        })
        .count();
    cdp_events.truncate(cdp_events_limit);
    let cdp_events = present_event_records(cdp_events.as_slice(), forensic_mode);

    let active_bans = crate::enforcement::ban::list_active_bans(store, site_id);
    let bans: Vec<_> = active_bans
        .iter()
        .map(|(ip, ban)| {
            let display_ip = if forensic_mode {
                ip.clone()
            } else {
                pseudonymize_ip_identifier(ip.as_str())
            };
            json!({
                "ip": display_ip,
                "reason": ban.reason,
                "expires": ban.expires,
                "banned_at": ban.banned_at,
                "fingerprint": ban.fingerprint
            })
        })
        .collect();

    let mut maze_ips: Vec<(String, u32)> = Vec::new();
    let mut total_hits: u32 = 0;
    if let Ok(keys) = store.get_keys() {
        for key in keys {
            if key.starts_with("maze_hits:") {
                let ip = key
                    .strip_prefix("maze_hits:")
                    .unwrap_or("unknown")
                    .to_string();
                if let Ok(Some(value)) = store.get(&key) {
                    if let Ok(hits) = String::from_utf8_lossy(&value).parse::<u32>() {
                        total_hits += hits;
                        maze_ips.push((ip, hits));
                    }
                }
            }
        }
    }
    maze_ips.sort_by(|a, b| b.1.cmp(&a.1));
    let deepest = maze_ips
        .first()
        .map(|(ip, hits)| {
            let display_ip = if forensic_mode {
                ip.clone()
            } else {
                pseudonymize_ip_identifier(ip.as_str())
            };
            json!({"ip": display_ip, "hits": hits})
        });
    let top_crawlers: Vec<_> = maze_ips
        .iter()
        .take(10)
        .map(|(ip, hits)| {
            let display_ip = if forensic_mode {
                ip.clone()
            } else {
                pseudonymize_ip_identifier(ip.as_str())
            };
            json!({"ip": display_ip, "hits": hits})
        })
        .collect();
    let maze_bans = active_bans
        .iter()
        .filter(|(_, ban)| ban.reason == "maze_crawler")
        .count();

    let tarpit_bucket_prefix = crate::providers::internal::tarpit_budget_bucket_active_prefix(site_id);
    let tarpit_bucket_key_prefix = format!("{}:", tarpit_bucket_prefix);
    let mut tarpit_active_bucket_counts: Vec<(String, u64)> = Vec::new();
    if let Ok(keys) = store.get_keys() {
        for key in keys {
            if !key.starts_with(tarpit_bucket_key_prefix.as_str()) {
                continue;
            }
            let bucket = key
                .strip_prefix(tarpit_bucket_key_prefix.as_str())
                .unwrap_or("unknown")
                .to_string();
            let count = read_u64_counter(store, key.as_str());
            tarpit_active_bucket_counts.push((bucket, count));
        }
    }
    tarpit_active_bucket_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let tarpit_top_active_buckets: Vec<_> = tarpit_active_bucket_counts
        .iter()
        .take(10)
        .map(|(bucket, count)| json!({"bucket": bucket, "active": count}))
        .collect();
    let tarpit_global_active_key = crate::providers::internal::tarpit_budget_global_active_key(site_id);

    let cfg = crate::config::Config::load(store, site_id).ok();
    let fail_mode = if crate::config::kv_store_fail_open() {
        "open"
    } else {
        "closed"
    };
    let retention_health = crate::observability::retention::retention_health(store);
    let cost_governance =
        monitoring_cost_governance_payload(store, events.as_slice(), now, &query_budget);
    let security_privacy = security_privacy_payload(store, now, hours, forensic_mode);

    json!({
        "retention_health": retention_health,
        "cost_governance": cost_governance,
        "security_privacy": security_privacy,
        "analytics": {
            "ban_count": active_bans.len(),
            "test_mode": cfg.as_ref().map(|v| v.test_mode).unwrap_or(false),
            "fail_mode": fail_mode
        },
        "events": {
            "recent_events": recent_events,
            "security_mode": security_view_mode_label(forensic_mode),
            "event_counts": event_counts,
            "top_ips": top_ips,
            "unique_ips": unique_ips,
            "recent_events_window": {
                "hours": hours,
                "requested_limit": limit,
                "applied_recent_event_cap": recent_event_cap,
                "total_events_in_window": total_recent_events_in_window,
                "returned_events": recent_events.len(),
                "has_more": recent_events_has_more,
                "continue_via": format!("/admin/monitoring/delta?hours={hours}&limit={}", limit.clamp(1, MONITORING_STREAM_MAX_BUFFER_EVENTS)),
                "response_shaping_reason": if query_budget.status == "exceeded" { "query_budget_guardrail" } else { "requested" }
            }
        },
        "bans": {
            "bans": bans
        },
        "maze": {
            "total_hits": total_hits,
            "unique_crawlers": maze_ips.len(),
            "maze_auto_bans": maze_bans,
            "deepest_crawler": deepest,
            "top_crawlers": top_crawlers
        },
        "tarpit": {
            "enabled": cfg.as_ref().map(|value| value.tarpit_enabled).unwrap_or(false),
            "progress_token_ttl_seconds": cfg.as_ref().map(|value| value.tarpit_progress_token_ttl_seconds).unwrap_or(0),
            "progress_replay_ttl_seconds": cfg.as_ref().map(|value| value.tarpit_progress_replay_ttl_seconds).unwrap_or(0),
            "hashcash_min_difficulty": cfg.as_ref().map(|value| value.tarpit_hashcash_min_difficulty).unwrap_or(0),
            "hashcash_max_difficulty": cfg.as_ref().map(|value| value.tarpit_hashcash_max_difficulty).unwrap_or(0),
            "hashcash_base_difficulty": cfg.as_ref().map(|value| value.tarpit_hashcash_base_difficulty).unwrap_or(0),
            "hashcash_adaptive": cfg.as_ref().map(|value| value.tarpit_hashcash_adaptive).unwrap_or(false),
            "step_chunk_base_bytes": cfg.as_ref().map(|value| value.tarpit_step_chunk_base_bytes).unwrap_or(0),
            "step_chunk_max_bytes": cfg.as_ref().map(|value| value.tarpit_step_chunk_max_bytes).unwrap_or(0),
            "step_jitter_percent": cfg.as_ref().map(|value| value.tarpit_step_jitter_percent).unwrap_or(0),
            "shard_rotation_enabled": cfg.as_ref().map(|value| value.tarpit_shard_rotation_enabled).unwrap_or(false),
            "egress_window_seconds": cfg.as_ref().map(|value| value.tarpit_egress_window_seconds).unwrap_or(0),
            "egress_global_bytes_per_window": cfg.as_ref().map(|value| value.tarpit_egress_global_bytes_per_window).unwrap_or(0),
            "egress_per_ip_bucket_bytes_per_window": cfg.as_ref().map(|value| value.tarpit_egress_per_ip_bucket_bytes_per_window).unwrap_or(0),
            "egress_per_flow_max_bytes": cfg.as_ref().map(|value| value.tarpit_egress_per_flow_max_bytes).unwrap_or(0),
            "egress_per_flow_max_duration_seconds": cfg.as_ref().map(|value| value.tarpit_egress_per_flow_max_duration_seconds).unwrap_or(0),
            "max_concurrent_global": cfg.as_ref().map(|value| value.tarpit_max_concurrent_global).unwrap_or(0),
            "max_concurrent_per_ip_bucket": cfg.as_ref().map(|value| value.tarpit_max_concurrent_per_ip_bucket).unwrap_or(0),
            "fallback_action": cfg.as_ref().map(|value| value.tarpit_fallback_action.as_str()).unwrap_or("maze"),
            "active": {
                "global": read_u64_counter(store, tarpit_global_active_key.as_str()),
                "top_buckets": tarpit_top_active_buckets
            },
            "metrics": {
                "activations": {
                    "progressive": read_u64_counter(store, "metrics:tarpit_activations_total:progressive")
                },
                "progress_outcomes": {
                    "advanced": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:advanced"),
                    "tarpit_progress_malformed": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_malformed"),
                    "tarpit_progress_signature_mismatch": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_signature_mismatch"),
                    "tarpit_progress_invalid_version": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_invalid_version"),
                    "tarpit_progress_expired": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_expired"),
                    "tarpit_progress_invalid_window": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_invalid_window"),
                    "tarpit_progress_binding_ip_mismatch": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_binding_ip_mismatch"),
                    "tarpit_progress_binding_ua_mismatch": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_binding_ua_mismatch"),
                    "tarpit_progress_path_mismatch": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_path_mismatch"),
                    "tarpit_progress_step_out_of_order": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_step_out_of_order"),
                    "tarpit_progress_parent_chain_missing": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_parent_chain_missing"),
                    "tarpit_progress_replay": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_replay"),
                    "tarpit_progress_invalid_proof": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_invalid_proof"),
                    "tarpit_progress_budget_exhausted": read_u64_counter(store, "metrics:tarpit_progress_outcomes_total:tarpit_progress_budget_exhausted")
                },
                "budget_outcomes": {
                    "acquired": read_u64_counter(store, "metrics:tarpit_budget_outcomes_total:acquired"),
                    "saturated": read_u64_counter(store, "metrics:tarpit_budget_outcomes_total:saturated"),
                    "fallback_maze": read_u64_counter(store, "metrics:tarpit_budget_outcomes_total:fallback_maze"),
                    "fallback_block": read_u64_counter(store, "metrics:tarpit_budget_outcomes_total:fallback_block")
                },
                "escalation_outcomes": {
                    "none": read_u64_counter(store, "metrics:tarpit_escalation_outcomes_total:none"),
                    "short_ban": read_u64_counter(store, "metrics:tarpit_escalation_outcomes_total:short_ban"),
                    "block": read_u64_counter(store, "metrics:tarpit_escalation_outcomes_total:block")
                },
                "duration_buckets": {
                    "lt_1s": read_u64_counter(store, "metrics:tarpit_duration_buckets_total:lt_1s"),
                    "1_5s": read_u64_counter(store, "metrics:tarpit_duration_buckets_total:1_5s"),
                    "5_20s": read_u64_counter(store, "metrics:tarpit_duration_buckets_total:5_20s"),
                    "20s_plus": read_u64_counter(store, "metrics:tarpit_duration_buckets_total:20s_plus")
                },
                "bytes_buckets": {
                    "lt_8kb": read_u64_counter(store, "metrics:tarpit_bytes_buckets_total:lt_8kb"),
                    "8_32kb": read_u64_counter(store, "metrics:tarpit_bytes_buckets_total:8_32kb"),
                    "32_128kb": read_u64_counter(store, "metrics:tarpit_bytes_buckets_total:32_128kb"),
                    "128_512kb": read_u64_counter(store, "metrics:tarpit_bytes_buckets_total:128_512kb"),
                    "512kb_plus": read_u64_counter(store, "metrics:tarpit_bytes_buckets_total:512kb_plus")
                }
            }
        },
        "cdp": {
            "config": {
                "enabled": cfg.as_ref().map(|v| v.cdp_detection_enabled).unwrap_or(false),
                "auto_ban": cfg.as_ref().map(|v| v.cdp_auto_ban).unwrap_or(false),
                "detection_threshold": cfg.as_ref().map(|v| v.cdp_detection_threshold).unwrap_or(0.0),
                "probe_family": cfg.as_ref().map(|v| v.cdp_probe_family.as_str()).unwrap_or("legacy"),
                "probe_rollout_percent": cfg.as_ref().map(|v| v.cdp_probe_rollout_percent).unwrap_or(0),
                "fingerprint_signal_enabled": cfg.as_ref().map(|v| v.fingerprint_signal_enabled).unwrap_or(false),
                "fingerprint_state_ttl_seconds": cfg.as_ref().map(|v| v.fingerprint_state_ttl_seconds).unwrap_or(0),
                "fingerprint_flow_window_seconds": cfg.as_ref().map(|v| v.fingerprint_flow_window_seconds).unwrap_or(0),
                "fingerprint_flow_violation_threshold": cfg.as_ref().map(|v| v.fingerprint_flow_violation_threshold).unwrap_or(0),
                "fingerprint_pseudonymize": cfg.as_ref().map(|v| v.fingerprint_pseudonymize).unwrap_or(false),
                "fingerprint_entropy_budget": cfg.as_ref().map(|v| v.fingerprint_entropy_budget).unwrap_or(0),
                "fingerprint_family_cap_header_runtime": cfg.as_ref().map(|v| v.fingerprint_family_cap_header_runtime).unwrap_or(0),
                "fingerprint_family_cap_transport": cfg.as_ref().map(|v| v.fingerprint_family_cap_transport).unwrap_or(0),
                "fingerprint_family_cap_temporal": cfg.as_ref().map(|v| v.fingerprint_family_cap_temporal).unwrap_or(0),
                "fingerprint_family_cap_persistence": cfg.as_ref().map(|v| v.fingerprint_family_cap_persistence).unwrap_or(0),
                "fingerprint_family_cap_behavior": cfg.as_ref().map(|v| v.fingerprint_family_cap_behavior).unwrap_or(0)
            },
            "stats": {
                "total_detections": read_u64_counter(store, "cdp:detections"),
                "auto_bans": read_u64_counter(store, "cdp:auto_bans")
            },
            "fingerprint_stats": {
                "events": read_u64_counter(store, "fingerprint:events"),
                "ua_client_hint_mismatch": read_u64_counter(store, "fingerprint:ua_ch_mismatch"),
                "ua_transport_mismatch": read_u64_counter(store, "fingerprint:ua_transport_mismatch"),
                "temporal_transition": read_u64_counter(store, "fingerprint:temporal_transition"),
                "flow_violation": read_u64_counter(store, "fingerprint:flow_violation"),
                "persistence_marker_missing": read_u64_counter(store, "fingerprint:persistence_marker_missing"),
                "untrusted_transport_header": read_u64_counter(store, "fingerprint:untrusted_transport_header")
            }
        },
        "cdp_events": {
            "events": cdp_events,
            "hours": hours,
            "limit": cdp_events_limit,
            "total_matches": total_matches,
            "counts": {
                "detections": detections,
                "auto_bans": auto_bans
            }
        }
    })
}

fn monitoring_cost_governance_payload<S>(
    store: &S,
    events: &[EventLogRecord],
    now: u64,
    query_budget: &MonitoringQueryBudget,
) -> serde_json::Value
where
    S: crate::challenge::KeyValueStore,
{
    let now_hour = now / 3600;
    let cap_per_hour = crate::observability::monitoring::guarded_dimension_cardinality_cap_per_hour();
    let count_suffix = format!(":{}", now_hour);
    let count_prefix = "monitoring:v1:cardinality_guard_count:";
    let overflow_prefix = "monitoring:v1:cardinality_guard_overflow:";
    let mut observed_guarded_dimension_cardinality_max = 0u64;
    let mut overflow_bucket_count = 0u64;

    if let Ok(keys) = store.get_keys() {
        for key in keys {
            if key.starts_with(count_prefix) && key.ends_with(count_suffix.as_str()) {
                observed_guarded_dimension_cardinality_max =
                    observed_guarded_dimension_cardinality_max.max(read_u64_counter(store, key.as_str()));
                continue;
            }
            if key.starts_with(overflow_prefix) && key.ends_with(count_suffix.as_str()) {
                overflow_bucket_count =
                    overflow_bucket_count.saturating_add(read_u64_counter(store, key.as_str()));
            }
        }
    }

    let sampled_key = format!(
        "monitoring:v1:ip_range_suggestions:likely_human_sampled:{}",
        now_hour
    );
    let unsampled_key = format!(
        "monitoring:v1:ip_range_suggestions:likely_human_unsampled:{}",
        now_hour
    );
    let sampled_count = read_u64_counter(store, sampled_key.as_str());
    let unsampled_count = read_u64_counter(store, unsampled_key.as_str());
    let unsampleable_drop_count = 0u64;

    let one_min = now.saturating_sub(60);
    let five_min = now.saturating_sub(300);
    let one_hour = now.saturating_sub(3600);
    let rollup_1m = events
        .iter()
        .filter(|event| event.entry.ts >= one_min)
        .count() as u64;
    let rollup_5m = events
        .iter()
        .filter(|event| event.entry.ts >= five_min)
        .count() as u64;
    let rollup_1h = events
        .iter()
        .filter(|event| event.entry.ts >= one_hour)
        .count() as u64;

    json!({
        "cost_envelope_profiles": {
            "runtime_dev": {
                "ingest_events_per_second": MONITORING_COST_ENVELOPE_INGEST_EVENTS_PER_SECOND_DEV,
                "query_calls_per_second_per_client": MONITORING_COST_ENVELOPE_QUERY_CALLS_PER_SECOND_CLIENT_DEV,
                "payload_p95_kb": MONITORING_PAYLOAD_BUDGET_P95_KB,
                "guarded_dimension_cardinality_cap_per_hour": cap_per_hour,
                "compression_min_percent_for_payloads_over_64kb": MONITORING_COMPRESSION_MIN_REDUCTION_PERCENT
            },
            "runtime_prod": {
                "ingest_events_per_second": MONITORING_COST_ENVELOPE_INGEST_EVENTS_PER_SECOND_PROD,
                "query_calls_per_second_per_client": MONITORING_COST_ENVELOPE_QUERY_CALLS_PER_SECOND_CLIENT_PROD,
                "payload_p95_kb": MONITORING_PAYLOAD_BUDGET_P95_KB,
                "guarded_dimension_cardinality_cap_per_hour": cap_per_hour,
                "compression_min_percent_for_payloads_over_64kb": MONITORING_COMPRESSION_MIN_REDUCTION_PERCENT
            }
        },
        "guarded_dimension_cardinality_cap_per_hour": cap_per_hour,
        "observed_guarded_dimension_cardinality_max": observed_guarded_dimension_cardinality_max,
        "overflow_bucket_accounted": true,
        "overflow_bucket_count": overflow_bucket_count,
        "cardinality_pressure": if overflow_bucket_count > 0 { "pressure" } else { "normal" },
        "rollups": {
            "1m": rollup_1m,
            "5m": rollup_5m,
            "1h": rollup_1h,
            "raw_event_lineage_source": "eventlog:v2"
        },
        "unsampleable_event_classes": crate::observability::monitoring::unsampleable_security_event_classes(),
        "unsampleable_event_drop_count": unsampleable_drop_count,
        "sampling": {
            "eligible_low_risk_classes": ["ip_range_suggestions.likely_human_sample"],
            "sampled_count": sampled_count,
            "unsampled_count": unsampled_count
        },
        "sampling_status": if unsampleable_drop_count == 0 { "compliant" } else { "violation" },
        "payload_budget": {
            "p95_max_kb": MONITORING_PAYLOAD_BUDGET_P95_KB,
            "estimated_current_payload_kb": 0.0,
            "status": "within_budget"
        },
        "payload_budget_status": "within_budget",
        "compression": {
            "status": "pending",
            "negotiated": false,
            "algorithm": "none",
            "input_bytes": 0,
            "output_bytes": 0,
            "reduction_percent": 0.0,
            "min_percent": MONITORING_COMPRESSION_MIN_REDUCTION_PERCENT
        },
        "query_budget": {
            "cost_units": query_budget.cost_units,
            "cost_class": query_budget.cost_class,
            "avg_req_per_sec_client_target": query_budget.avg_req_per_sec_client,
            "max_req_per_sec_client": query_budget.max_req_per_sec_client,
            "status": query_budget.status
        },
        "query_budget_status": query_budget.status,
        "degraded_state": if query_budget.status == "exceeded" { "degraded" } else { "normal" },
        "degraded_reasons": if query_budget.status == "exceeded" { vec!["query_budget_exceeded"] } else { Vec::<&str>::new() }
    })
}

fn monitoring_prometheus_helper_payload() -> serde_json::Value {
    json!({
        "endpoint": "/metrics",
        "notes": [
            "/metrics returns one full Prometheus text payload and accepts no query arguments.",
            "For bounded JSON summaries use /admin/monitoring?hours=<1-720>&limit=<1-50>, then read summary.* fields."
        ],
        "example_js": "const metricsText = await fetch('/metrics').then(r => r.text());",
        "example_output": "# TYPE bot_defence_requests_total counter\nbot_defence_requests_total{path=\"main\"} 128\n# TYPE bot_defence_blocks_total counter\nbot_defence_blocks_total 9\n# TYPE bot_defence_bans_total counter\nbot_defence_bans_total{reason=\"honeypot\"} 3\n# TYPE bot_defence_active_bans gauge\nbot_defence_active_bans 2",
        "example_stats": "const lines = metricsText.split('\\n');\nconst metricValue = (prefix) => {\n  const line = lines.find((entry) => entry.startsWith(prefix));\n  return line ? Number(line.slice(prefix.length).trim()) : null;\n};\nconst stats = {\n  requestsMain: metricValue('bot_defence_requests_total{path=\\\"main\\\"} '),\n  honeypotBans: metricValue('bot_defence_bans_total{reason=\\\"honeypot\\\"} '),\n  blocksTotal: metricValue('bot_defence_blocks_total '),\n  activeBans: metricValue('bot_defence_active_bans ')\n};",
        "example_windowed": "const apiKey = 'YOUR_ADMIN_API_KEY';\nconst params = new URLSearchParams({ hours: '24', limit: '10' });\nconst monitoring = await fetch(`/admin/monitoring?${params}`, {\n  headers: { Authorization: `Bearer ${apiKey}` }\n}).then(r => r.json());",
        "example_summary_stats": "const stats = {\n  honeypotHits: monitoring.summary.honeypot.total_hits,\n  challengeFailures: monitoring.summary.challenge.total_failures,\n  notABotServed: monitoring.summary.not_a_bot.served,\n  notABotPass: monitoring.summary.not_a_bot.pass,\n  notABotAbandonmentRate: monitoring.summary.not_a_bot.abandonment_ratio,\n  powFailures: monitoring.summary.pow.total_failures,\n  powSuccesses: monitoring.summary.pow.total_successes,\n  powSuccessRatio: monitoring.summary.pow.success_ratio,\n  rateViolations: monitoring.summary.rate.total_violations,\n  geoViolations: monitoring.summary.geo.total_violations\n};",
        "docs": {
            "observability": "https://github.com/atomless/Shuma-Gorath/blob/main/docs/observability.md",
            "api": "https://github.com/atomless/Shuma-Gorath/blob/main/docs/api.md"
        }
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdminAdversarySimControlRequest {
    enabled: bool,
    #[serde(default)]
    reason: Option<String>,
}

fn log_admin_csrf_denied<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    path: &str,
    auth: &crate::admin::auth::AdminAuthResult,
) {
    let client_ip = crate::extract_client_ip(req);
    let session = auth.session_id.as_deref().unwrap_or("-");
    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: Some(client_ip),
            reason: Some("admin_csrf_denied".to_string()),
            outcome: Some(format!(
                "path={} method={} actor={} session={}",
                path,
                req.method(),
                auth.audit_actor_label(),
                session
            )),
            admin: Some(auth.audit_actor_label().to_string()),
        },
    );
}

fn log_adversary_sim_transition<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    auth: &crate::admin::auth::AdminAuthResult,
    transition: &crate::admin::adversary_sim::Transition,
    operation_id: Option<&str>,
) {
    let client_ip = crate::extract_client_ip(req);
    let session = auth.session_id.as_deref().unwrap_or("-");
    let run_id = transition.run_id.as_deref().unwrap_or("-");
    let operation = operation_id.unwrap_or("-");
    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: Some(client_ip),
            reason: Some("adversary_sim_transition".to_string()),
            outcome: Some(format!(
                "from={} to={} reason={} run_id={} operation_id={} actor={} session={}",
                transition.from.as_str(),
                transition.to.as_str(),
                transition.reason,
                run_id,
                operation,
                auth.audit_actor_label(),
                session
            )),
            admin: Some(auth.audit_actor_label().to_string()),
        },
    );
}

fn log_adversary_sim_control_audit<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    auth: &crate::admin::auth::AdminAuthResult,
    audit: &crate::admin::adversary_sim_control::ControlAuditRecord,
    _capability: &crate::admin::adversary_sim_control::AuditWriteCapability,
) {
    let client_ip = crate::extract_client_ip(req);
    let payload = json!({
        "operation_id": audit.operation_id.clone(),
        "actor_scope": audit.actor_scope.clone(),
        "session_scope_hash": crate::admin::adversary_sim_control::hash_hex(audit.session_scope.as_str()),
        "decision": audit.decision,
        "reason": audit.reason.clone(),
        "origin_verdict": audit.origin_verdict.clone(),
        "idempotency_key_hash": audit.idempotency_key_hash.clone(),
        "request_origin": audit.request_origin.clone(),
        "requested_state": audit.requested_state.clone(),
        "desired_state": audit.desired_state.clone(),
        "actual_state": audit.actual_state.clone()
    });
    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: Some(client_ip),
            reason: Some("adversary_sim_control_audit".to_string()),
            outcome: Some(payload.to_string()),
            admin: Some(auth.audit_actor_label().to_string()),
        },
    );
}

fn save_adversary_sim_state_with_capability<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    state: &crate::admin::adversary_sim::ControlState,
    _capability: &crate::admin::adversary_sim_control::StateWriteCapability,
) -> Result<(), ()> {
    crate::admin::adversary_sim::save_state(store, site_id, state)
}

fn adversary_sim_status_payload(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    cfg: &crate::config::Config,
    state: &crate::admin::adversary_sim::ControlState,
    now: u64,
) -> serde_json::Value {
    let mut payload = crate::admin::adversary_sim::status_payload(
        now,
        crate::config::runtime_environment(),
        crate::config::adversary_sim_available(),
        cfg.adversary_sim_enabled,
        cfg.adversary_sim_duration_seconds,
        state,
    );
    let reconciliation_required = crate::admin::adversary_sim_control::status_reconciliation_needed(
        now,
        cfg.adversary_sim_enabled,
        state,
    );
    let generation_diagnostics =
        crate::admin::adversary_sim::generation_diagnostics(now, cfg.adversary_sim_enabled, state);
    let supervisor = crate::admin::adversary_sim::supervisor_status_payload(
        now,
        cfg.adversary_sim_enabled,
        state,
    );
    let lease = crate::admin::adversary_sim_control::load_controller_lease(store, site_id);
    let lease_operation_id = lease.as_ref().map(|value| value.operation_id.clone());
    let lease_expires_at = lease.as_ref().map(|value| value.expires_at);
    let seconds_since_last_successful_beat = state
        .last_generated_at
        .map(|last_generated_at| now.saturating_sub(last_generated_at));
    let generation_active =
        cfg.adversary_sim_enabled && state.phase == crate::admin::adversary_sim::ControlPhase::Running;
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "desired_state".to_string(),
            serde_json::Value::String(if cfg.adversary_sim_enabled {
                "running".to_string()
            } else {
                "off".to_string()
            }),
        );
        object.insert(
            "actual_state".to_string(),
            serde_json::Value::String(
                crate::admin::adversary_sim_control::actual_phase_label(state.phase).to_string(),
            ),
        );
        object.insert(
            "controller_reconciliation_required".to_string(),
            serde_json::Value::Bool(reconciliation_required),
        );
        object.insert(
            "generation_active".to_string(),
            serde_json::Value::Bool(generation_active),
        );
        object.insert(
            "historical_data_visible".to_string(),
            serde_json::Value::Bool(true),
        );
        object.insert(
            "history_retention".to_string(),
            json!({
                "retention_hours": event_log_retention_hours(),
                "retention_health": crate::observability::retention::retention_health(store),
                "cleanup_supported": crate::config::admin_config_write_enabled(),
                "cleanup_endpoint": "/admin/adversary-sim/history/cleanup",
                "cleanup_command": "make adversary-sim-history-clean"
            }),
        );
        object.insert(
            "generation_diagnostics".to_string(),
            json!({
                "health": generation_diagnostics.health,
                "reason": generation_diagnostics.reason,
                "recommended_action": generation_diagnostics.recommended_action,
                "generated_tick_count": generation_diagnostics.generated_tick_count,
                "generated_request_count": generation_diagnostics.generated_request_count,
                "last_generated_at": generation_diagnostics.last_generated_at,
                "last_generation_error": generation_diagnostics.last_generation_error
            }),
        );
        object.insert("supervisor".to_string(), supervisor);
        object.insert(
            "control_contract".to_string(),
            json!({
                "contract": "adversary-sim-control.v1",
                "idempotency_ttl_seconds": crate::admin::adversary_sim_control::IDEMPOTENCY_TTL_SECONDS,
                "lease_ttl_seconds": crate::admin::adversary_sim_control::LEASE_TTL_SECONDS,
                "requires_idempotency_key": true
            }),
        );
        object.insert(
            "lifecycle_diagnostics".to_string(),
            json!({
                "control": {
                    "desired_enabled": cfg.adversary_sim_enabled,
                    "actual_phase": state.phase.as_str(),
                    "controller_reconciliation_required": reconciliation_required,
                    "runtime_instance_id": crate::admin::adversary_sim::process_instance_id(),
                    "owner_instance_id": state.owner_instance_id.clone(),
                    "last_transition_reason": state.last_transition_reason.clone(),
                    "last_terminal_failure_reason": state.last_terminal_failure_reason.clone(),
                    "last_control_operation_id": lease_operation_id,
                    "lease_expires_at": lease_expires_at
                },
                "supervisor": {
                    "heartbeat_expected": generation_active,
                    "generated_tick_count": state.generated_tick_count,
                    "generated_request_count": state.generated_request_count,
                    "last_successful_beat_at": state.last_generated_at,
                    "seconds_since_last_successful_beat": seconds_since_last_successful_beat,
                    "last_generation_error": state.last_generation_error.clone()
                }
            }),
        );
        object.insert(
            "controller_lease".to_string(),
            match lease {
                Some(current_lease) => json!({
                    "owner": current_lease.owner,
                    "fencing_token": current_lease.fencing_token,
                    "acquired_at": current_lease.acquired_at,
                    "expires_at": current_lease.expires_at,
                    "operation_id": current_lease.operation_id
                }),
                None => serde_json::Value::Null,
            },
        );
    }
    payload
}

fn reconcile_adversary_sim_enabled_runtime_override(
    site_id: &str,
    cfg: &mut crate::config::Config,
    state: &crate::admin::adversary_sim::ControlState,
) {
    if cfg.adversary_sim_enabled && state.phase == crate::admin::adversary_sim::ControlPhase::Off {
        crate::config::set_runtime_adversary_sim_enabled_override(site_id, false);
        cfg.adversary_sim_enabled = false;
    }
}

fn effective_adversary_sim_enabled(
    cfg: &crate::config::Config,
    state: &crate::admin::adversary_sim::ControlState,
) -> bool {
    cfg.adversary_sim_enabled || state.desired_enabled
}

fn handle_admin_adversary_sim_status(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    _auth: &crate::admin::auth::AdminAuthResult,
) -> Response {
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }

    let runtime_environment = crate::config::runtime_environment();
    let env_available = crate::config::adversary_sim_available();
    if !crate::admin::adversary_sim::control_surface_available(runtime_environment, env_available) {
        return Response::new(404, "Not Found");
    }

    let mut cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let now = now_ts();
    let mut state = crate::admin::adversary_sim::load_state(store, site_id);
    cfg.adversary_sim_enabled = effective_adversary_sim_enabled(&cfg, &state);
    let (reconciled_state, _) =
        crate::admin::adversary_sim::reconcile_state(now, cfg.adversary_sim_enabled, &state);
    if reconciled_state != state {
        state = reconciled_state;
        if crate::admin::adversary_sim::save_state(store, site_id, &state).is_err() {
            return Response::new(500, "Key-value store error");
        }
    }
    cfg.adversary_sim_enabled = effective_adversary_sim_enabled(&cfg, &state);
    reconcile_adversary_sim_enabled_runtime_override(site_id, &mut cfg, &state);

    let body = serde_json::to_string(&adversary_sim_status_payload(store, site_id, &cfg, &state, now))
        .unwrap();
    Response::new(200, body)
}

fn handle_admin_adversary_sim_history_cleanup(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    _auth: &crate::admin::auth::AdminAuthResult,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    if !crate::config::admin_config_write_enabled() {
        return Response::new(
            403,
            "Config updates are disabled when SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false",
        );
    }

    let runtime_environment = crate::config::runtime_environment();
    let env_available = crate::config::adversary_sim_available();
    if !crate::admin::adversary_sim::control_surface_available(runtime_environment, env_available) {
        return Response::new(404, "Not Found");
    }

    let cleanup_result = clear_dev_telemetry_history(store, site_id);
    log_event(
        store,
        &EventLogEntry {
            ts: now_ts(),
            event: EventType::AdminAction,
            ip: Some(crate::extract_client_ip(req)),
            reason: Some("adversary_sim_history_cleanup".to_string()),
            outcome: Some(format!(
                "deleted_keys={} families={}",
                cleanup_result.deleted_keys,
                cleanup_result.deleted_by_family.len()
            )),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );

    let body = serde_json::to_string(&json!({
        "cleaned": true,
        "deleted_keys": cleanup_result.deleted_keys,
        "deleted_by_family": cleanup_result.deleted_by_family,
        "retention_hours": event_log_retention_hours(),
        "cleanup_command": "make adversary-sim-history-clean"
    }))
    .unwrap();
    Response::new(200, body)
}

fn internal_adversary_supervisor_is_authorized(req: &Request) -> bool {
    matches!(
        crate::admin::auth::bearer_access_level(req),
        Some(crate::admin::auth::AdminAccessLevel::ReadWrite)
    )
}

fn handle_internal_adversary_sim_beat(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let runtime_environment = crate::config::runtime_environment();
    let env_available = crate::config::adversary_sim_available();
    if !crate::admin::adversary_sim::control_surface_available(runtime_environment, env_available) {
        return Response::new(404, "Not Found");
    }

    if !internal_adversary_supervisor_is_authorized(req) {
        return Response::new(401, "Unauthorized: Internal supervisor authorization required");
    }

    let mut cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let now = now_ts();
    let mut state = crate::admin::adversary_sim::load_state(store, site_id);
    let previous_state = state.clone();

    cfg.adversary_sim_enabled = effective_adversary_sim_enabled(&cfg, &state);
    let (reconciled_state, _) =
        crate::admin::adversary_sim::reconcile_state(now, cfg.adversary_sim_enabled, &state);
    state = reconciled_state;
    cfg.adversary_sim_enabled = effective_adversary_sim_enabled(&cfg, &state);
    reconcile_adversary_sim_enabled_runtime_override(site_id, &mut cfg, &state);

    let summary = crate::admin::adversary_sim::run_autonomous_supervisor_ticks(store, &mut state, now);
    if state != previous_state && crate::admin::adversary_sim::save_state(store, site_id, &state).is_err() {
        return Response::new(500, "Key-value store error");
    }

    if summary.executed_ticks > 0 {
        crate::log_line(&format!(
            "[adversary-sim-supervisor] executed_ticks={} generated_requests={} failed_requests={}",
            summary.executed_ticks, summary.generated_requests, summary.failed_requests
        ));
    }

    let generation_active =
        cfg.adversary_sim_enabled && state.phase == crate::admin::adversary_sim::ControlPhase::Running;
    let status = adversary_sim_status_payload(store, site_id, &cfg, &state, now);
    let body = serde_json::to_string(&json!({
        "accepted": true,
        "executed_ticks": summary.executed_ticks,
        "due_ticks": summary.due_ticks,
        "generated_requests": summary.generated_requests,
        "failed_requests": summary.failed_requests,
        "last_response_status": summary.last_response_status,
        "phase": state.phase.as_str(),
        "generation_active": generation_active,
        "should_exit": !generation_active,
        "status": status
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

fn handle_admin_adversary_sim_control(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    auth: &crate::admin::auth::AdminAuthResult,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    if !crate::config::admin_config_write_enabled() {
        return Response::new(
            403,
            "Config updates are disabled when SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false",
        );
    }

    let runtime_environment = crate::config::runtime_environment();
    let env_available = crate::config::adversary_sim_available();
    if !crate::admin::adversary_sim::control_surface_available(runtime_environment, env_available) {
        return Response::new(404, "Not Found");
    }

    if auth.requires_csrf(req) {
        let expected = auth.csrf_token.as_deref().unwrap_or("");
        if !crate::admin::auth::validate_session_csrf(req, expected) {
            log_admin_csrf_denied(store, req, "/admin/adversary-sim/control", auth);
            return Response::new(403, "Forbidden: control trust boundary violation");
        }
    }

    let body = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_ADMIN_JSON_BYTES,
    ) {
        Ok(v) => v,
        Err(err) => return Response::new(400, err),
    };
    let payload = match serde_json::from_value::<AdminAdversarySimControlRequest>(body) {
        Ok(parsed) => parsed,
        Err(err) => return Response::new(400, format!("Invalid control payload: {}", err)),
    };

    let now = now_ts();
    let requested_reason = crate::admin::adversary_sim_control::canonical_reason(payload.reason.as_deref());
    let Some(idempotency_key) = req
        .header("idempotency-key")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Response::new(428, "Idempotency-Key header is required");
    };
    let idempotency_key_hash = crate::admin::adversary_sim_control::hash_hex(idempotency_key);
    let payload_hash = crate::admin::adversary_sim_control::canonical_payload_hash(
        payload.enabled,
        requested_reason.as_str(),
    );

    let session_scope = crate::admin::adversary_sim_control::idempotency_scope(auth);
    let actor_scope = crate::admin::adversary_sim_control::actor_scope(auth);
    let client_ip = crate::extract_client_ip(req);

    let origin_validation = crate::admin::adversary_sim_control::validate_origin_and_fetch_metadata(req);
    let session_origin_fallback_allowed = matches!(
        auth.method,
        Some(crate::admin::auth::AdminAuthMethod::SessionCookie)
    );
    let (origin_verdict, request_origin, trust_reason, trust_decision) = match origin_validation {
        Ok(valid) => (
            valid.verdict,
            valid.request_origin,
            "trust_boundary_ok".to_string(),
            crate::admin::adversary_sim_control::TrustDecision::Allow,
        ),
        Err(reason @ ("origin_missing" | "fetch_metadata_missing"))
            if session_origin_fallback_allowed =>
        {
            (
                "session_csrf_origin_fallback".to_string(),
                None,
                "session_csrf_origin_fallback".to_string(),
                crate::admin::adversary_sim_control::TrustDecision::Allow,
            )
        }
        Err(reason) => (
            "origin_denied".to_string(),
            None,
            reason.to_string(),
            crate::admin::adversary_sim_control::TrustDecision::Reject,
        ),
    };

    let idempotency_store_key = crate::admin::adversary_sim_control::control_idempotency_key(
        site_id,
        session_scope.as_str(),
        idempotency_key_hash.as_str(),
    );
    let mut existing_idempotency =
        crate::admin::adversary_sim_control::load_idempotency_record(store, &idempotency_store_key);
    if existing_idempotency
        .as_ref()
        .map(|record| record.expires_at <= now)
        .unwrap_or(false)
    {
        existing_idempotency = None;
    }
    let idempotency_plan = match existing_idempotency.as_ref() {
        Some(record) if record.payload_hash == payload_hash => {
            crate::admin::adversary_sim_control::IdempotencyPlan::Replay
        }
        Some(_) => crate::admin::adversary_sim_control::IdempotencyPlan::PayloadMismatch,
        None => crate::admin::adversary_sim_control::IdempotencyPlan::NewSubmission,
    };

    let mut cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let mut state = crate::admin::adversary_sim::load_state(store, site_id);
    cfg.adversary_sim_enabled = effective_adversary_sim_enabled(&cfg, &state);
    let (reconciled_state, _) =
        crate::admin::adversary_sim::reconcile_state(now, cfg.adversary_sim_enabled, &state);
    if reconciled_state != state {
        state = reconciled_state;
        if crate::admin::adversary_sim::save_state(store, site_id, &state).is_err() {
            return Response::new(500, "Key-value store error");
        }
    }
    cfg.adversary_sim_enabled = effective_adversary_sim_enabled(&cfg, &state);
    reconcile_adversary_sim_enabled_runtime_override(site_id, &mut cfg, &state);

    let debounce_key =
        crate::admin::adversary_sim_control::control_debounce_key(site_id, session_scope.as_str());
    let last_submission_at = store
        .get(&debounce_key)
        .ok()
        .flatten()
        .as_deref()
        .and_then(crate::admin::adversary_sim_control::parse_debounce_timestamp);
    let debounce_throttled = crate::admin::adversary_sim_control::should_throttle_for_debounce(
        now,
        last_submission_at,
        crate::admin::adversary_sim_control::CONTROL_DEBOUNCE_SECONDS,
    ) && cfg.adversary_sim_enabled == payload.enabled;
    let rate_limited = adversary_sim_control_submission_is_limited(
        store,
        session_scope.as_str(),
        client_ip.as_str(),
    );
    let throttle_decision = if rate_limited || debounce_throttled {
        crate::admin::adversary_sim_control::ThrottleDecision::Throttle
    } else {
        crate::admin::adversary_sim_control::ThrottleDecision::Allow
    };
    let throttle_reason = if throttle_decision == crate::admin::adversary_sim_control::ThrottleDecision::Throttle {
        if debounce_throttled {
            "debounce_window"
        } else {
            "toggle_rate_limited"
        }
    } else {
        "throttle_ok"
    };

    let plan = crate::admin::adversary_sim_control::plan_submission(
        &crate::admin::adversary_sim_control::SubmissionPlanInput {
            trust: trust_decision,
            throttle: throttle_decision,
            idempotency: idempotency_plan,
        },
    );
    let capabilities = crate::admin::adversary_sim_control::ControlCapabilities::mint_for_trust_boundary();

    let current_actual_state = state.phase.as_str().to_string();
    match plan.decision {
        crate::admin::adversary_sim_control::SubmissionPlanDecision::RejectTrustBoundary => {
            let trust_reason = trust_reason.clone();
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: None,
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Rejected,
                    reason: trust_reason,
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash),
                    request_origin,
                    requested_state: Some(if payload.enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    desired_state: Some(if cfg.adversary_sim_enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    actual_state: current_actual_state,
                },
                capabilities.audit_write(),
            );
            return Response::new(403, "Forbidden: control trust boundary violation");
        }
        crate::admin::adversary_sim_control::SubmissionPlanDecision::RejectThrottled => {
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: None,
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Throttled,
                    reason: throttle_reason.to_string(),
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash),
                    request_origin,
                    requested_state: Some(if payload.enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    desired_state: Some(if cfg.adversary_sim_enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    actual_state: current_actual_state,
                },
                capabilities.audit_write(),
            );
            return Response::builder()
                .status(429)
                .header("Retry-After", "60")
                .body("Too Many Requests: adversary control throttled")
                .build();
        }
        crate::admin::adversary_sim_control::SubmissionPlanDecision::RejectPayloadMismatch => {
            let replayed_operation_id = existing_idempotency
                .as_ref()
                .map(|record| record.operation_id.clone());
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: replayed_operation_id,
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Rejected,
                    reason: "idempotency_payload_mismatch".to_string(),
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash),
                    request_origin,
                    requested_state: Some(if payload.enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    desired_state: Some(if cfg.adversary_sim_enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    actual_state: current_actual_state,
                },
                capabilities.audit_write(),
            );
            return Response::new(
                409,
                "Idempotency-Key replay rejected: payload mismatch for existing key",
            );
        }
        crate::admin::adversary_sim_control::SubmissionPlanDecision::ReturnReplay => {
            let Some(idempotency_record) = existing_idempotency.as_ref() else {
                return Response::new(500, "Idempotency state unavailable");
            };
            let operation_key = crate::admin::adversary_sim_control::control_operation_key(
                site_id,
                idempotency_record.operation_id.as_str(),
            );
            let operation = crate::admin::adversary_sim_control::load_operation_record(store, &operation_key);
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: Some(idempotency_record.operation_id.clone()),
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Replayed,
                    reason: "idempotency_exact_replay".to_string(),
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash.clone()),
                    request_origin,
                    requested_state: Some(if payload.enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    desired_state: Some(if cfg.adversary_sim_enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    actual_state: state.phase.as_str().to_string(),
                },
                capabilities.audit_write(),
            );
            let response = json!({
                "operation_id": idempotency_record.operation_id,
                "decision": "replayed",
                "requested_enabled": payload.enabled,
                "phase_trace": ["plan", "execute", "collect_evidence", "publish_report"],
                "requested_state": {
                    "enabled": payload.enabled,
                    "reason": requested_reason
                },
                "accepted_state": {
                    "desired_enabled": cfg.adversary_sim_enabled,
                    "actual_phase": state.phase.as_str()
                },
                "idempotency": {
                    "key_hash": idempotency_key_hash,
                    "replayed": true,
                    "ttl_seconds": crate::admin::adversary_sim_control::IDEMPOTENCY_TTL_SECONDS
                },
                "operation": operation,
                "status": adversary_sim_status_payload(store, site_id, &cfg, &state, now),
                "config": admin_config_payload(
                    &cfg,
                    challenge_threshold_default(),
                    not_a_bot_threshold_default(),
                    maze_threshold_default()
                )
            });
            return Response::new(200, serde_json::to_string(&response).unwrap());
        }
        crate::admin::adversary_sim_control::SubmissionPlanDecision::AcceptNew => {}
    }

    let operation_id = crate::admin::adversary_sim_control::operation_id(now);
    let current_lease = crate::admin::adversary_sim_control::load_controller_lease(store, site_id);
    let lease = match crate::admin::adversary_sim_control::acquire_controller_lease(
        now,
        session_scope.as_str(),
        Some(operation_id.as_str()),
        current_lease.as_ref(),
    ) {
        Ok(lease) => lease,
        Err(reason) => {
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: Some(operation_id),
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Throttled,
                    reason: reason.to_string(),
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash),
                    request_origin,
                    requested_state: Some(if payload.enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    desired_state: Some(if cfg.adversary_sim_enabled {
                        "running".to_string()
                    } else {
                        "off".to_string()
                    }),
                    actual_state: state.phase.as_str().to_string(),
                },
                capabilities.audit_write(),
            );
            return Response::new(409, "Adversary simulation controller lease is currently held");
        }
    };
    if crate::admin::adversary_sim_control::save_controller_lease(
        store,
        site_id,
        &lease,
        capabilities.state_write(),
    )
    .is_err()
    {
        return Response::new(500, "Key-value store error");
    }

    let mut transitions = Vec::new();
    let (preflight_state, mut preflight_transitions) =
        crate::admin::adversary_sim::reconcile_state(now, cfg.adversary_sim_enabled, &state);
    state = preflight_state;
    transitions.append(&mut preflight_transitions);
    let mut desired_enabled = payload.enabled;

    if payload.enabled {
        if state.phase != crate::admin::adversary_sim::ControlPhase::Running {
            let duration = crate::admin::adversary_sim::clamp_duration_seconds(
                cfg.adversary_sim_duration_seconds,
            );
            match crate::admin::adversary_sim::start_state(now, duration, &state) {
                Ok((next_state, mut started_transitions)) => {
                    state = next_state;
                    transitions.append(&mut started_transitions);
                }
                Err(crate::admin::adversary_sim::StartError::QueueFull) => {
                    return Response::new(
                        409,
                        "Adversary simulation queue is full (queue_policy=reject_new)",
                    );
                }
            }
        }
    } else {
        let (stopping_state, mut stop_transitions) =
            crate::admin::adversary_sim::stop_state(now, "manual_off", &state);
        state = stopping_state;
        transitions.append(&mut stop_transitions);
    }

    let (reconciled_state, mut reconciled_transitions) =
        crate::admin::adversary_sim::reconcile_state(now, desired_enabled, &state);
    state = reconciled_state;
    transitions.append(&mut reconciled_transitions);

    if state.phase == crate::admin::adversary_sim::ControlPhase::Off && desired_enabled {
        desired_enabled = false;
    }
    if save_adversary_sim_state_with_capability(store, site_id, &state, capabilities.state_write())
        .is_err()
    {
        return Response::new(500, "Key-value store error");
    }
    crate::config::set_runtime_adversary_sim_enabled_override(site_id, desired_enabled);
    cfg.adversary_sim_enabled = desired_enabled;
    for transition in &transitions {
        log_adversary_sim_transition(store, req, auth, transition, Some(operation_id.as_str()));
    }

    let operation_record = crate::admin::adversary_sim_control::ControlOperationRecord {
        operation_id: operation_id.clone(),
        requested_enabled: payload.enabled,
        requested_reason: requested_reason.clone(),
        desired_enabled,
        actual_phase: state.phase.as_str().to_string(),
        actor_scope: actor_scope.clone(),
        session_scope: session_scope.clone(),
        idempotency_key_hash: idempotency_key_hash.clone(),
        payload_hash: payload_hash.clone(),
        created_at: now,
        completed_at: now,
        decision: crate::admin::adversary_sim_control::ControlDecision::Accepted,
        decision_reason: "accepted".to_string(),
        origin_verdict: origin_verdict.clone(),
        lease_fencing_token: Some(lease.fencing_token),
    };
    let operation_key =
        crate::admin::adversary_sim_control::control_operation_key(site_id, operation_id.as_str());
    if crate::admin::adversary_sim_control::save_operation_record(
        store,
        &operation_key,
        &operation_record,
        capabilities.state_write(),
    )
    .is_err()
    {
        return Response::new(500, "Key-value store error");
    }

    let idempotency_record = crate::admin::adversary_sim_control::IdempotencyRecord {
        operation_id: operation_id.clone(),
        payload_hash,
        actor_scope: actor_scope.clone(),
        session_scope: session_scope.clone(),
        created_at: now,
        expires_at: now.saturating_add(crate::admin::adversary_sim_control::IDEMPOTENCY_TTL_SECONDS),
    };
    if crate::admin::adversary_sim_control::save_idempotency_record(
        store,
        &idempotency_store_key,
        &idempotency_record,
        capabilities.state_write(),
    )
    .is_err()
    {
        return Response::new(500, "Key-value store error");
    }
    if crate::admin::adversary_sim_control::save_debounce_timestamp(
        store,
        &debounce_key,
        now,
        capabilities.state_write(),
    )
    .is_err()
    {
        return Response::new(500, "Key-value store error");
    }

    log_adversary_sim_control_audit(
        store,
        req,
        auth,
        &crate::admin::adversary_sim_control::ControlAuditRecord {
            operation_id: Some(operation_id.clone()),
            actor_scope,
            session_scope,
            decision: crate::admin::adversary_sim_control::ControlDecision::Accepted,
            reason: "accepted".to_string(),
            origin_verdict,
            idempotency_key_hash: Some(idempotency_key_hash.clone()),
            request_origin,
            requested_state: Some(if payload.enabled {
                "running".to_string()
            } else {
                "off".to_string()
            }),
            desired_state: Some(if cfg.adversary_sim_enabled {
                "running".to_string()
            } else {
                "off".to_string()
            }),
            actual_state: state.phase.as_str().to_string(),
        },
        capabilities.audit_write(),
    );

    let response = json!({
        "operation_id": operation_id,
        "decision": "accepted",
        "requested_enabled": payload.enabled,
        "phase_trace": ["plan", "execute", "collect_evidence", "publish_report"],
        "requested_state": {
            "enabled": payload.enabled,
            "reason": requested_reason
        },
        "accepted_state": {
            "desired_enabled": desired_enabled,
            "actual_phase": state.phase.as_str()
        },
        "idempotency": {
            "key_hash": idempotency_key_hash,
            "replayed": false,
            "ttl_seconds": crate::admin::adversary_sim_control::IDEMPOTENCY_TTL_SECONDS
        },
        "status": adversary_sim_status_payload(store, site_id, &cfg, &state, now),
        "config": admin_config_payload(
            &cfg,
            challenge_threshold_default(),
            not_a_bot_threshold_default(),
            maze_threshold_default()
        ),
    });
    Response::new(200, serde_json::to_string(&response).unwrap())
}

/// Handles host-side internal control-plane endpoints (no browser/UI callers).
///
/// Currently supports:
///   - POST /internal/adversary-sim/beat: run one bounded autonomous supervisor beat
pub fn handle_internal(req: &Request) -> Response {
    if !crate::admin::auth::is_admin_ip_allowed(req) {
        return Response::new(403, "Forbidden");
    }
    if !crate::admin::auth::is_admin_api_key_configured() {
        return Response::new(503, "Internal API disabled: admin key not configured");
    }

    let path = req.path();
    match path {
        INTERNAL_ADVERSARY_SIM_BEAT_PATH => {
            let store = match Store::open_default() {
                Ok(s) => s,
                Err(_) => return Response::new(500, "Key-value store error"),
            };
            handle_internal_adversary_sim_beat(req, &store, "default")
        }
        _ => Response::new(404, "Not Found"),
    }
}

/// Handles all /admin API endpoints.
/// Supports:
///   - POST /admin/login: Exchange API key for short-lived admin session cookie
///   - GET /admin/session: Return current admin auth session state
///   - POST /admin/logout: Clear admin session cookie
///   - GET /admin/ban: List all bans for the site
///   - POST /admin/ban: Manually ban an IP (expects JSON body: {"ip": "1.2.3.4", "duration": 3600}; reason is fixed to "manual_ban")
///   - POST /admin/unban?ip=...: Remove a ban for an IP
///   - GET /admin/analytics: Return ban count and test_mode status
///   - GET /admin/events: Query event log
///   - GET /admin/cdp/events: Query CDP-only events
///   - GET /admin/monitoring: Query consolidated monitoring telemetry summaries
///   - GET /admin/monitoring/delta: Cursor-based monitoring event deltas (`after_cursor`, `limit`, `next_cursor`)
///   - GET /admin/monitoring/stream: One-shot SSE cursor delta (`Last-Event-ID` resume supported)
///   - GET /admin/ip-bans/delta: Cursor-based ban/unban deltas plus active-ban snapshot
///   - GET /admin/ip-bans/stream: One-shot SSE ban delta (`Last-Event-ID` resume supported)
///   - GET /admin/ip-range/suggestions: Query IP range recommendation suggestions
///   - GET /admin/config: Get current config including test_mode status
///   - POST /admin/config: Update config (e.g., toggle test_mode)
///   - POST /admin/config/validate: Validate a config patch without persisting changes
///   - GET /admin/config/export: Export non-secret runtime config for immutable deploy handoff
///   - POST /admin/adversary-sim/control: Start/stop dev adversary simulation orchestration
///   - GET /admin/adversary-sim/status: Read orchestration state and guardrails
///   - POST /admin/adversary-sim/history/cleanup: Explicitly clear retained dev telemetry history
///   - GET /admin/maze/preview: Render a non-operational maze preview for operators
///   - GET /admin/tarpit/preview: Render a non-operational progressive tarpit preview for operators
///   - GET /admin: API help
pub fn handle_admin(req: &Request) -> Response {
    // Optional admin IP allowlist
    if !crate::admin::auth::is_admin_ip_allowed(req) {
        return Response::new(403, "Forbidden");
    }
    if !crate::admin::auth::is_admin_api_key_configured() {
        return Response::new(503, "Admin API disabled: key not configured");
    }

    let path = req.path();
    if !sanitize_path(path) {
        return Response::new(400, "Bad Request: Invalid admin endpoint");
    }

    if path == "/admin/login" || path == "/admin/session" || path == "/admin/logout" {
        let store = match Store::open_default() {
            Ok(s) => s,
            Err(_) => return Response::new(500, "Key-value store error"),
        };
        let provider_registry = crate::config::load_runtime_cached(&store, "default")
            .ok()
            .map(|cfg| crate::providers::registry::ProviderRegistry::from_config(&cfg));
        return match path {
            "/admin/login" => handle_admin_login_with_failure_handler(req, &store, || {
                register_admin_auth_failure_with_selected_rate_limiter(
                    &store,
                    req,
                    crate::admin::auth::AdminAuthFailureScope::Login,
                    provider_registry.as_ref(),
                )
            }),
            "/admin/session" => handle_admin_session(req, &store),
            "/admin/logout" => handle_admin_logout_with_failure_handler(req, &store, || {
                register_admin_auth_failure_with_selected_rate_limiter(
                    &store,
                    req,
                    crate::admin::auth::AdminAuthFailureScope::Endpoint,
                    provider_registry.as_ref(),
                )
            }),
            _ => Response::new(400, "Bad Request: Invalid admin endpoint"),
        };
    }

    let has_bearer = crate::admin::auth::is_bearer_authorized(req);
    let has_session_cookie = crate::admin::auth::has_admin_session_cookie(req);
    if !has_bearer && !has_session_cookie {
        if matches!(
            path,
            "/admin/adversary-sim/control"
                | "/admin/adversary-sim/history/cleanup"
        ) {
            if let Ok(store) = Store::open_default() {
                let client_ip = crate::extract_client_ip(req);
                log_event(
                    &store,
                    &EventLogEntry {
                        ts: now_ts(),
                        event: EventType::AdminAction,
                        ip: Some(client_ip),
                        reason: Some("adversary_sim_auth_denied".to_string()),
                        outcome: Some(format!("path={} method={}", path, req.method())),
                        admin: Some("-".to_string()),
                    },
                );
            }
        }
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }

    let store = match Store::open_default() {
        Ok(s) => s,
        Err(_) => return Response::new(500, "Key-value store error"),
    };
    let provider_registry = crate::config::load_runtime_cached(&store, "default")
        .ok()
        .map(|cfg| crate::providers::registry::ProviderRegistry::from_config(&cfg));

    // Require either a valid bearer token or a valid admin session cookie.
    let auth = crate::admin::auth::authenticate_admin(req, &store);
    if !auth.is_authorized() {
        if matches!(
            path,
            "/admin/adversary-sim/control"
                | "/admin/adversary-sim/history/cleanup"
        ) {
            let client_ip = crate::extract_client_ip(req);
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: Some(client_ip),
                    reason: Some("adversary_sim_auth_denied".to_string()),
                    outcome: Some(format!("path={} method={}", path, req.method())),
                    admin: Some(auth.audit_actor_label().to_string()),
                },
            );
        }
        if register_admin_auth_failure_with_selected_rate_limiter(
            &store,
            req,
            crate::admin::auth::AdminAuthFailureScope::Endpoint,
            provider_registry.as_ref(),
        ) {
            return too_many_admin_auth_attempts_response();
        }
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }
    if auth.requires_csrf(req) {
        let expected = auth.csrf_token.as_deref().unwrap_or("");
        if !crate::admin::auth::validate_session_csrf(req, expected) {
            log_admin_csrf_denied(&store, req, path, &auth);
            return Response::new(403, "Forbidden");
        }
    }
    if request_requires_admin_write(path, req.method()) && !auth.is_write_authorized() {
        log_admin_write_denied(&store, req, path, &auth);
        return Response::new(403, "Forbidden: admin write access required");
    }

    let site_id = "default";

    match path {
        "/admin/events" => {
            if expensive_admin_read_is_limited(&store, req, &auth, provider_registry.as_ref()) {
                return too_many_admin_read_requests_response();
            }
            // Query event log for recent events, top IPs, and event statistics
            // Query params: ?hours=N (default 24, max 720)
            let hours = query_u64_param(req.query(), "hours", 24).clamp(1, 720);
            let forensic_mode = forensic_access_mode(req.query());
            let now = now_ts();
            let mut events = load_recent_event_records(&store, now, hours);
            let mut ip_counts = std::collections::HashMap::new();
            let mut event_counts = std::collections::HashMap::new();

            for e in &events {
                if let Some(ip) = &e.entry.ip {
                    let key = if forensic_mode {
                        ip.clone()
                    } else {
                        pseudonymize_ip_identifier(ip.as_str())
                    };
                    *ip_counts.entry(key).or_insert(0u32) += 1;
                }
                *event_counts
                    .entry(format!("{:?}", e.entry.event))
                    .or_insert(0u32) += 1;
            }
            // Sort events by timestamp descending
            events.sort_by(|a, b| b.entry.ts.cmp(&a.entry.ts));
            // Unique IP count before consuming the map
            let unique_ips = ip_counts.len();
            // Top 10 IPs
            let mut top_ips: Vec<_> = ip_counts.into_iter().collect();
            top_ips.sort_by(|a, b| b.1.cmp(&a.1));
            let top_ips: Vec<_> = top_ips.into_iter().take(10).collect();
            let recent_events_raw: Vec<_> = events.iter().take(100).cloned().collect();
            let recent_events = present_event_records(recent_events_raw.as_slice(), forensic_mode);
            let body = serde_json::to_string(&json!({
                "recent_events": recent_events,
                "event_counts": event_counts,
                "top_ips": top_ips,
                "unique_ips": unique_ips,
                "security_mode": security_view_mode_label(forensic_mode),
                "security_privacy": security_privacy_payload(&store, now, hours, forensic_mode)
            }))
            .unwrap();
            Response::new(200, body)
        }
        "/admin/cdp/events" => {
            if expensive_admin_read_is_limited(&store, req, &auth, provider_registry.as_ref()) {
                return too_many_admin_read_requests_response();
            }
            // Query params: ?hours=N&limit=M
            // hours default 24 (max 720), limit default 500 (max 5000)
            let hours = query_u64_param(req.query(), "hours", 24).clamp(1, 720);
            let limit = query_u64_param(req.query(), "limit", 500).clamp(1, 5000) as usize;
            let forensic_mode = forensic_access_mode(req.query());
            let now = now_ts();
            let mut cdp_events: Vec<EventLogRecord> =
                load_recent_event_records(&store, now, hours)
                .into_iter()
                .filter(|entry| {
                    entry
                        .entry
                        .reason
                        .as_deref()
                        .map(is_cdp_event_reason)
                        .unwrap_or(false)
                })
                .collect();

            cdp_events.sort_by(|a, b| b.entry.ts.cmp(&a.entry.ts));

            let total_matches = cdp_events.len();
            let detections = cdp_events
                .iter()
                .filter(|entry| {
                    entry
                        .entry
                        .reason
                        .as_deref()
                        .map(|reason| reason.to_lowercase().starts_with("cdp_detected:"))
                        .unwrap_or(false)
                })
                .count();
            let auto_bans = cdp_events
                .iter()
                .filter(|entry| {
                    entry
                        .entry
                        .reason
                        .as_deref()
                        .map(|reason| reason.eq_ignore_ascii_case("cdp_automation"))
                        .unwrap_or(false)
                })
                .count();

            cdp_events.truncate(limit);
            let cdp_events = present_event_records(cdp_events.as_slice(), forensic_mode);

            let body = serde_json::to_string(&json!({
                "events": cdp_events,
                "hours": hours,
                "limit": limit,
                "total_matches": total_matches,
                "security_mode": security_view_mode_label(forensic_mode),
                "security_privacy": security_privacy_payload(&store, now, hours, forensic_mode),
                "counts": {
                    "detections": detections,
                    "auto_bans": auto_bans
                }
            }))
            .unwrap();
            Response::new(200, body)
        }
        "/admin/monitoring" => {
            if dashboard_refresh_is_limited(&store, &auth, provider_registry.as_ref())
                || expensive_admin_read_is_limited(&store, req, &auth, provider_registry.as_ref())
            {
                return too_many_admin_read_requests_response();
            }
            handle_admin_monitoring(req, &store)
        }
        "/admin/monitoring/delta" => {
            if dashboard_refresh_is_limited(&store, &auth, provider_registry.as_ref())
                || expensive_admin_read_is_limited(&store, req, &auth, provider_registry.as_ref())
            {
                return too_many_admin_read_requests_response();
            }
            handle_admin_monitoring_delta(req, &store)
        }
        "/admin/monitoring/stream" => {
            if dashboard_refresh_is_limited(&store, &auth, provider_registry.as_ref())
                || expensive_admin_read_is_limited(&store, req, &auth, provider_registry.as_ref())
            {
                return too_many_admin_read_requests_response();
            }
            handle_admin_monitoring_stream(req, &store)
        }
        "/admin/ip-bans/delta" => {
            if dashboard_refresh_is_limited(&store, &auth, provider_registry.as_ref())
                || expensive_admin_read_is_limited(&store, req, &auth, provider_registry.as_ref())
            {
                return too_many_admin_read_requests_response();
            }
            handle_admin_ip_bans_delta(req, &store, site_id)
        }
        "/admin/ip-bans/stream" => {
            if dashboard_refresh_is_limited(&store, &auth, provider_registry.as_ref())
                || expensive_admin_read_is_limited(&store, req, &auth, provider_registry.as_ref())
            {
                return too_many_admin_read_requests_response();
            }
            handle_admin_ip_bans_stream(req, &store, site_id)
        }
        "/admin/ip-range/suggestions" => {
            if dashboard_refresh_is_limited(&store, &auth, provider_registry.as_ref())
                || expensive_admin_read_is_limited(&store, req, &auth, provider_registry.as_ref())
            {
                return too_many_admin_read_requests_response();
            }
            handle_admin_ip_range_suggestions(req, &store, site_id)
        }
        "/admin/ban" => {
            if *req.method() == spin_sdk::http::Method::Get
                && (dashboard_refresh_is_limited(&store, &auth, provider_registry.as_ref())
                    || expensive_admin_read_is_limited(
                        &store,
                        req,
                        &auth,
                        provider_registry.as_ref(),
                    ))
            {
                return too_many_admin_read_requests_response();
            }
            let cfg = match crate::config::load_runtime_cached(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };
            let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);

            // POST: Manually ban an IP
            if *req.method() == spin_sdk::http::Method::Post {
                let json = match crate::request_validation::parse_json_body(
                    req.body(),
                    crate::request_validation::MAX_ADMIN_JSON_BYTES,
                ) {
                    Ok(v) => v,
                    Err(e) => return Response::new(400, e),
                };

                let ip_raw = match json.get("ip").and_then(|v| v.as_str()) {
                    Some(v) => v,
                    None => return Response::new(400, "Missing 'ip' field in request body"),
                };
                let ip = match crate::request_validation::parse_ip_addr(ip_raw) {
                    Some(v) => v,
                    None => return Response::new(400, "Invalid IP address"),
                };
                // Manual bans are always tagged with a fixed reason to prevent client-side tampering.
                let reason = "manual_ban".to_string();
                let duration = json
                    .get("duration")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(21600)
                    .clamp(ADMIN_BAN_DURATION_MIN, ADMIN_BAN_DURATION_MAX);

                provider_registry
                    .ban_store_provider()
                    .ban_ip_with_fingerprint(
                        &store,
                        site_id,
                        ip.as_str(),
                        reason.as_str(),
                        duration,
                        Some(crate::enforcement::ban::BanFingerprint {
                            score: None,
                            signals: vec!["manual_admin".to_string()],
                            summary: Some("manual_admin_ban".to_string()),
                        }),
                    );
                // Log ban event
                log_event(
                    &store,
                    &EventLogEntry {
                        ts: now_ts(),
                        event: EventType::Ban,
                        ip: Some(ip.clone()),
                        reason: Some(reason.clone()),
                        outcome: Some("banned".to_string()),
                        admin: Some(crate::admin::auth::get_admin_id(req)),
                    },
                );
                return Response::new(200, json!({"status": "banned", "ip": ip}).to_string());
            }
            // GET: List all bans for this site (keys starting with ban:site_id:)
            let mut bans = vec![];
            for (ip, ban) in provider_registry
                .ban_store_provider()
                .list_active_bans(&store, site_id)
            {
                bans.push(json!({
                    "ip": ip,
                    "reason": ban.reason,
                    "expires": ban.expires,
                    "banned_at": ban.banned_at,
                    "fingerprint": ban.fingerprint
                }));
            }
            let body = serde_json::to_string(&json!({"bans": bans})).unwrap();
            Response::new(200, body)
        }
        "/admin/unban" => {
            if *req.method() != spin_sdk::http::Method::Post {
                return Response::new(405, "Method Not Allowed");
            }
            // Unban IP (expects ?ip=...)
            let ip_raw = match crate::request_validation::query_param(req.query(), "ip") {
                Some(v) => v,
                None => return Response::new(400, "Missing ip param"),
            };
            let ip = match parse_unban_identity(&ip_raw) {
                Some(v) => v,
                None => return Response::new(400, "Invalid IP address"),
            };
            if ip.is_empty() {
                return Response::new(400, "Missing ip param");
            }
            let cfg = match crate::config::load_runtime_cached(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };
            let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
            provider_registry
                .ban_store_provider()
                .unban_ip(&store, site_id, ip.as_str());
            // Log unban event
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::Unban,
                    ip: Some(ip.to_string()),
                    reason: Some("admin_unban".to_string()),
                    outcome: Some("unbanned".to_string()),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            Response::new(200, "Unbanned")
        }
        "/admin/analytics" => {
            // Return analytics: ban count and test_mode status
            let cfg = match crate::config::load_runtime_cached(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };
            let ban_count =
                crate::enforcement::ban::list_active_bans_with_scan(&store, site_id).len();
            let fail_mode = if crate::config::kv_store_fail_open() {
                "open"
            } else {
                "closed"
            };
            let body = serde_json::to_string(&json!({
                "ban_count": ban_count,
                "test_mode": cfg.test_mode,
                "fail_mode": fail_mode
            }))
            .unwrap();
            Response::new(200, body)
        }
        "/admin/config" => {
            return handle_admin_config(req, &store, site_id);
        }
        "/admin/config/validate" => {
            return handle_admin_config_validate(req, &store, site_id);
        }
        "/admin/config/export" => {
            return handle_admin_config_export(req, &store, site_id);
        }
        "/admin/adversary-sim/control" => {
            return handle_admin_adversary_sim_control(req, &store, site_id, &auth);
        }
        "/admin/adversary-sim/status" => {
            return handle_admin_adversary_sim_status(req, &store, site_id, &auth);
        }
        "/admin/adversary-sim/history/cleanup" => {
            return handle_admin_adversary_sim_history_cleanup(req, &store, site_id, &auth);
        }
        "/admin/maze/preview" => {
            return handle_admin_maze_preview(req, &store, site_id);
        }
        "/admin/tarpit/preview" => {
            return handle_admin_tarpit_preview(req, &store, site_id);
        }
        "/admin/maze/seeds" => {
            return handle_admin_maze_seed_sources(req, &store, site_id);
        }
        "/admin/maze/seeds/refresh" => {
            return handle_admin_maze_seed_refresh(req, &store, site_id);
        }
        "/admin" => {
            // API help endpoint
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("help".to_string()),
                    outcome: None,
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            Response::new(200, "WASM Bot Defence Admin API. Endpoints: /admin/ban, /admin/unban?ip=IP, /admin/analytics, /admin/events, /admin/monitoring, /admin/monitoring/delta, /admin/monitoring/stream, /admin/ip-bans/delta, /admin/ip-bans/stream, /admin/ip-range/suggestions, /admin/config, /admin/config/validate, /admin/config/export, /admin/adversary-sim/control, /admin/adversary-sim/status, /admin/adversary-sim/history/cleanup, /admin/maze (GET for maze stats), /admin/maze/preview (GET non-operational maze preview), /admin/tarpit/preview (GET non-operational progressive tarpit preview), /admin/maze/seeds (GET/POST seed source adapters), /admin/maze/seeds/refresh (POST manual seed refresh), /admin/robots (GET for robots.txt config & preview), /admin/robots/preview (POST unsaved robots preview patch), /admin/cdp (GET for CDP detection config & stats), /admin/cdp/events (GET for CDP detection and auto-ban events).")
        }
        "/admin/maze" => {
            // Return maze statistics
            // - Total unique IPs that have visited maze pages
            // - Per-IP hit counts (top crawlers)
            // - Total maze hits
            let mut maze_ips: Vec<(String, u32)> = Vec::new();
            let mut total_hits: u32 = 0;

            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with("maze_hits:") {
                        let ip = k
                            .strip_prefix("maze_hits:")
                            .unwrap_or("unknown")
                            .to_string();
                        if let Ok(Some(val)) = store.get(&k) {
                            if let Ok(hits) = String::from_utf8_lossy(&val).parse::<u32>() {
                                total_hits += hits;
                                maze_ips.push((ip, hits));
                            }
                        }
                    }
                }
            }

            // Sort by hits descending
            maze_ips.sort_by(|a, b| b.1.cmp(&a.1));

            // Get the deepest crawler (most maze page visits)
            let deepest = maze_ips
                .first()
                .map(|(ip, hits)| json!({"ip": ip, "hits": hits}));

            // Top 10 crawlers
            let top_crawlers: Vec<_> = maze_ips
                .iter()
                .take(10)
                .map(|(ip, hits)| json!({"ip": ip, "hits": hits}))
                .collect();

            // Count auto-bans from maze (check bans with reason "maze_crawler")
            let maze_bans = crate::enforcement::ban::list_active_bans_with_scan(&store, site_id)
                .into_iter()
                .filter(|(_, ban)| ban.reason == "maze_crawler")
                .count();

            // Log admin maze view
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("maze_stats_view".to_string()),
                    outcome: Some(format!("{} crawlers, {} hits", maze_ips.len(), total_hits)),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );

            let body = serde_json::to_string(&json!({
                "total_hits": total_hits,
                "unique_crawlers": maze_ips.len(),
                "maze_auto_bans": maze_bans,
                "deepest_crawler": deepest,
                "top_crawlers": top_crawlers
            }))
            .unwrap();
            Response::new(200, body)
        }
        "/admin/robots/preview" => {
            if req.method() != &Method::Post {
                return Response::new(405, "Method Not Allowed");
            }
            let patch = match crate::request_validation::parse_json_body(
                req.body(),
                crate::request_validation::MAX_ADMIN_JSON_BYTES,
            ) {
                Ok(value) => value,
                Err(msg) => return Response::new(400, msg),
            };
            let mut cfg = match crate::config::Config::load(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };
            apply_robots_preview_patch(&mut cfg, &patch);
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("robots_preview_patch".to_string()),
                    outcome: None,
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            admin_robots_response(&cfg)
        }
        "/admin/robots" => {
            // Return robots.txt configuration and preview
            let cfg = match crate::config::Config::load(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };

            // Log admin action
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("robots_config_view".to_string()),
                    outcome: None,
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            admin_robots_response(&cfg)
        }
        "/admin/cdp" => {
            // Return CDP detection configuration and stats
            let cfg = match crate::config::Config::load(&store, site_id) {
                Ok(cfg) => cfg,
                Err(err) => return Response::new(500, err.user_message()),
            };

            // Get CDP detection stats from KV store
            let cdp_detections: u64 = store
                .get("cdp:detections")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let cdp_auto_bans: u64 = store
                .get("cdp:auto_bans")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let fingerprint_events: u64 = store
                .get("fingerprint:events")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let fingerprint_ua_ch_mismatch: u64 = store
                .get("fingerprint:ua_ch_mismatch")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let fingerprint_ua_transport_mismatch: u64 = store
                .get("fingerprint:ua_transport_mismatch")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let fingerprint_temporal_transition: u64 = store
                .get("fingerprint:temporal_transition")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let fingerprint_flow_violation: u64 = store
                .get("fingerprint:flow_violation")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let fingerprint_persistence_marker_missing: u64 = store
                .get("fingerprint:persistence_marker_missing")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let fingerprint_untrusted_transport_header: u64 = store
                .get("fingerprint:untrusted_transport_header")
                .ok()
                .flatten()
                .and_then(|v| String::from_utf8(v).ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // Log admin action
            log_event(
                &store,
                &EventLogEntry {
                    ts: now_ts(),
                    event: EventType::AdminAction,
                    ip: None,
                    reason: Some("cdp_config_view".to_string()),
                    outcome: None,
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );

            let body = serde_json::to_string(&json!({
                "config": {
                    "enabled": cfg.cdp_detection_enabled,
                    "auto_ban": cfg.cdp_auto_ban,
                    "detection_threshold": cfg.cdp_detection_threshold,
                    "probe_family": cfg.cdp_probe_family,
                    "probe_rollout_percent": cfg.cdp_probe_rollout_percent,
                    "fingerprint_signal_enabled": cfg.fingerprint_signal_enabled,
                    "fingerprint_state_ttl_seconds": cfg.fingerprint_state_ttl_seconds,
                    "fingerprint_flow_window_seconds": cfg.fingerprint_flow_window_seconds,
                    "fingerprint_flow_violation_threshold": cfg.fingerprint_flow_violation_threshold,
                    "fingerprint_pseudonymize": cfg.fingerprint_pseudonymize,
                    "fingerprint_entropy_budget": cfg.fingerprint_entropy_budget,
                    "fingerprint_family_cap_header_runtime": cfg.fingerprint_family_cap_header_runtime,
                    "fingerprint_family_cap_transport": cfg.fingerprint_family_cap_transport,
                    "fingerprint_family_cap_temporal": cfg.fingerprint_family_cap_temporal,
                    "fingerprint_family_cap_persistence": cfg.fingerprint_family_cap_persistence,
                    "fingerprint_family_cap_behavior": cfg.fingerprint_family_cap_behavior
                },
                "stats": {
                    "total_detections": cdp_detections,
                    "auto_bans": cdp_auto_bans
                },
                "fingerprint_stats": {
                    "events": fingerprint_events,
                    "ua_client_hint_mismatch": fingerprint_ua_ch_mismatch,
                    "ua_transport_mismatch": fingerprint_ua_transport_mismatch,
                    "temporal_transition": fingerprint_temporal_transition,
                    "flow_violation": fingerprint_flow_violation,
                    "persistence_marker_missing": fingerprint_persistence_marker_missing,
                    "untrusted_transport_header": fingerprint_untrusted_transport_header
                },
                "detection_methods": [
                    "Error stack timing analysis (Runtime.Enable leak)",
                    "navigator.webdriver property check",
                    "Automation-specific window properties",
                    "Chrome object consistency verification",
                    "Plugin array anomaly detection"
                ]
            }))
            .unwrap();
            Response::new(200, body)
        }
        _ => Response::new(404, "Not found"),
    }
}
