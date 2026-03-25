use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spin_sdk::http::Request;
#[cfg(not(test))]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::signals::botness::{BotSignal, SignalFamily, SignalProvenance};

const FP_UA_CH_MISMATCH_KEY: &str = "fp_ua_ch_mismatch";
const FP_UA_TRANSPORT_MISMATCH_KEY: &str = "fp_ua_transport_mismatch";
const FP_TEMPORAL_TRANSITION_KEY: &str = "fp_temporal_transition";
const FP_FLOW_VIOLATION_KEY: &str = "fp_flow_violation";
const FP_PERSISTENCE_MARKER_KEY: &str = "fp_persistence_marker_missing";
const FP_UNTRUSTED_TRANSPORT_HEADER_KEY: &str = "fp_untrusted_transport_header";
const FP_AKAMAI_EDGE_ADDITIVE_KEY: &str = "fp_akamai_edge_additive";

const FP_KEY_PREFIX_STATE: &str = "fp:state:";
const FP_KEY_PREFIX_FLOW: &str = "fp:flow:";
const FP_KEY_PREFIX_FLOW_LAST_BUCKET: &str = "fp:flow:last_bucket:";
const FP_KEY_PREFIX_EDGE_SIGNAL: &str = "fp:edge:";
const FP_CLEANUP_STATE_KEY: &str = "fp:cleanup:v1:last_run_ts";

const WEIGHT_UA_CH_MISMATCH: u8 = 2;
const WEIGHT_UA_TRANSPORT_MISMATCH: u8 = 3;
const WEIGHT_TEMPORAL_TRANSITION: u8 = 2;
const WEIGHT_FLOW_VIOLATION: u8 = 2;
const WEIGHT_PERSISTENCE_MARKER_MISSING: u8 = 1;
const WEIGHT_UNTRUSTED_TRANSPORT_HEADER: u8 = 3;
const WEIGHT_AKAMAI_EDGE_ADDITIVE: u8 = 2;
const AKAMAI_EDGE_ADDITIVE_CONFIDENCE_MIN: u8 = 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FingerprintState {
    ts: u64,
    ua_family: String,
    ja4_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExternalEdgeSignalState {
    ts: u64,
    confidence: u8,
    hard_signal: bool,
}

#[derive(Debug, Default)]
struct TransportEvidence {
    ja3: Option<String>,
    ja4: Option<String>,
    edge_browser_family: Option<String>,
    edge_score: Option<f32>,
    untrusted_headers_present: bool,
}

#[cfg(not(test))]
fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
fn now_ts() -> u64 {
    1_700_000_000
}

fn header_value(req: &Request, name: &str) -> Option<String> {
    req.header(name)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn normalize_browser_family(raw: &str) -> &'static str {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("edg/") || lower.contains("edge") || lower == "edge" {
        "edge"
    } else if lower == "chrome"
        || lower.contains("chrome/")
        || lower.contains("chromium")
        || lower.contains("crios")
    {
        "chrome"
    } else if lower == "firefox" || lower.contains("firefox/") || lower.contains("fxios") {
        "firefox"
    } else if lower == "safari" || lower.contains("safari/") {
        if lower.contains("chrome/") || lower.contains("chromium") || lower.contains("crios") {
            "chrome"
        } else {
            "safari"
        }
    } else {
        "other"
    }
}

fn extract_ua_family(req: &Request) -> &'static str {
    normalize_browser_family(
        req.header("user-agent")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    )
}

fn bool_from_client_hint_mobile(raw: Option<String>) -> Option<bool> {
    match raw.as_deref().map(str::trim) {
        Some("?1") => Some(true),
        Some("?0") => Some(false),
        _ => None,
    }
}

fn detect_ua_client_hint_mismatch(req: &Request) -> bool {
    let ua_family = extract_ua_family(req);
    let ch_ua = header_value(req, "sec-ch-ua")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let ch_mobile = bool_from_client_hint_mobile(header_value(req, "sec-ch-ua-mobile"));
    let ua_raw = header_value(req, "user-agent")
        .unwrap_or_default()
        .to_ascii_lowercase();

    let family_mismatch = if ch_ua.is_empty() {
        false
    } else {
        match ua_family {
            "chrome" => !ch_ua.contains("chrom"),
            "firefox" => !ch_ua.contains("firefox"),
            "safari" => !ch_ua.contains("safari"),
            "edge" => !(ch_ua.contains("edge") || ch_ua.contains("edg")),
            _ => false,
        }
    };

    let mobile_mismatch = match ch_mobile {
        Some(true) => !ua_raw.contains("mobile") && !ua_raw.contains("android"),
        Some(false) => ua_raw.contains("mobile") && !ua_raw.contains("ipad"),
        None => false,
    };

    family_mismatch || mobile_mismatch
}

fn sanitize_transport_token(value: Option<String>) -> Option<String> {
    let value = value?;
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 256 {
        return None;
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':')
    {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

fn parse_score(value: Option<String>) -> Option<f32> {
    let raw = value?;
    let score = raw.trim().parse::<f32>().ok()?;
    if score.is_finite() && (0.0..=100.0).contains(&score) {
        Some(score)
    } else {
        None
    }
}

fn extract_transport_evidence(req: &Request, headers_trusted: bool) -> TransportEvidence {
    let ja3 = sanitize_transport_token(header_value(req, "x-shuma-edge-ja3"));
    let ja4 = sanitize_transport_token(header_value(req, "x-shuma-edge-ja4"));
    let edge_browser_family = sanitize_transport_token(header_value(
        req,
        "x-shuma-edge-browser-family",
    ));
    let edge_score = parse_score(header_value(req, "x-shuma-edge-bot-score"));

    if headers_trusted {
        return TransportEvidence {
            ja3,
            ja4,
            edge_browser_family,
            edge_score,
            untrusted_headers_present: false,
        };
    }

    let untrusted_headers_present =
        ja3.is_some() || ja4.is_some() || edge_browser_family.is_some() || edge_score.is_some();
    TransportEvidence {
        ja3: None,
        ja4: None,
        edge_browser_family: None,
        edge_score: None,
        untrusted_headers_present,
    }
}

fn fingerprint_secret() -> String {
    crate::config::runtime_var_trimmed_optional("SHUMA_JS_SECRET")
        .unwrap_or_else(|| "shuma-fingerprint-default-secret".to_string())
}

fn hash_prefix(input: &str, chars: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(format!("{:02x}", byte).as_str());
    }
    hex.chars().take(chars).collect()
}

fn flow_identity(ip: &str, cfg: &crate::config::Config) -> String {
    if !cfg.fingerprint_pseudonymize {
        return ip
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == ':' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
    }
    hash_prefix(
        format!("{}|{}", fingerprint_secret(), ip.trim()).as_str(),
        24,
    )
}

fn external_edge_signal_ttl_seconds(cfg: &crate::config::Config) -> u64 {
    cfg.fingerprint_flow_window_seconds.max(1)
}

fn read_u64<S: crate::challenge::KeyValueStore>(store: &S, key: &str) -> Option<u64> {
    store
        .get(key)
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u64>().ok())
}

fn fingerprint_cleanup_cadence_seconds(cfg: &crate::config::Config) -> u64 {
    cfg.fingerprint_state_ttl_seconds
        .max(1)
        .min(cfg.fingerprint_flow_window_seconds.max(1))
        .clamp(10, 300)
}

fn cleanup_stale_fingerprint_state_key<S: crate::challenge::KeyValueStore>(
    store: &S,
    key: &str,
    now: u64,
    ttl_seconds: u64,
) {
    let Some(raw) = store.get(key).ok().flatten() else {
        return;
    };
    let Some(state) = serde_json::from_slice::<FingerprintState>(raw.as_slice()).ok() else {
        let _ = store.delete(key);
        return;
    };
    if now.saturating_sub(state.ts) > ttl_seconds.max(1) {
        let _ = store.delete(key);
    }
}

fn cleanup_stale_fingerprint_flow_key<S: crate::challenge::KeyValueStore>(
    store: &S,
    key: &str,
    current_bucket: u64,
) {
    let Some(suffix) = key.strip_prefix(FP_KEY_PREFIX_FLOW) else {
        return;
    };
    let Some((_, bucket_raw)) = suffix.rsplit_once(':') else {
        let _ = store.delete(key);
        return;
    };
    let Some(bucket) = bucket_raw.parse::<u64>().ok() else {
        let _ = store.delete(key);
        return;
    };
    if bucket != current_bucket {
        let _ = store.delete(key);
    }
}

fn cleanup_stale_fingerprint_flow_last_bucket_key<S: crate::challenge::KeyValueStore>(
    store: &S,
    key: &str,
    current_bucket: u64,
) {
    let Some(bucket) = read_u64(store, key) else {
        let _ = store.delete(key);
        return;
    };
    if bucket != current_bucket {
        let _ = store.delete(key);
    }
}

fn maybe_run_fingerprint_retention_cleanup<S: crate::challenge::KeyValueStore>(
    store: &S,
    cfg: &crate::config::Config,
    now: u64,
) {
    let cadence_seconds = fingerprint_cleanup_cadence_seconds(cfg);
    let last_run = read_u64(store, FP_CLEANUP_STATE_KEY).unwrap_or(0);
    if now.saturating_sub(last_run) < cadence_seconds {
        return;
    }

    let Ok(keys) = store.get_keys() else {
        return;
    };

    let state_ttl_seconds = cfg.fingerprint_state_ttl_seconds.max(1);
    let current_bucket = now / cfg.fingerprint_flow_window_seconds.max(1);

    for key in keys {
        if key.starts_with(FP_KEY_PREFIX_STATE) {
            cleanup_stale_fingerprint_state_key(store, key.as_str(), now, state_ttl_seconds);
            continue;
        }
        if key.starts_with(FP_KEY_PREFIX_FLOW_LAST_BUCKET) {
            cleanup_stale_fingerprint_flow_last_bucket_key(store, key.as_str(), current_bucket);
            continue;
        }
        if key.starts_with(FP_KEY_PREFIX_FLOW) {
            cleanup_stale_fingerprint_flow_key(store, key.as_str(), current_bucket);
        }
    }

    let _ = store.set(FP_CLEANUP_STATE_KEY, now.to_string().as_bytes());
}

fn load_external_edge_signal_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    identity: &str,
    now: u64,
    ttl_seconds: u64,
) -> Option<ExternalEdgeSignalState> {
    let key = format!("{}{}", FP_KEY_PREFIX_EDGE_SIGNAL, identity);
    let state = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_slice::<ExternalEdgeSignalState>(&raw).ok());

    let Some(state) = state else {
        return None;
    };

    if now.saturating_sub(state.ts) > ttl_seconds.max(1) {
        let _ = store.delete(key.as_str());
        return None;
    }

    Some(state)
}

fn store_external_edge_signal_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    identity: &str,
    state: &ExternalEdgeSignalState,
) {
    let key = format!("{}{}", FP_KEY_PREFIX_EDGE_SIGNAL, identity);
    let Ok(raw) = serde_json::to_vec(state) else {
        return;
    };
    let _ = store.set(key.as_str(), &raw);
}

pub(crate) fn record_akamai_edge_signal<S: crate::challenge::KeyValueStore>(
    store: &S,
    cfg: &crate::config::Config,
    ip: &str,
    confidence: u8,
    hard_signal: bool,
) {
    if cfg.provider_backends.fingerprint_signal != crate::config::ProviderBackend::External {
        return;
    }
    let identity = flow_identity(ip, cfg);
    let now = now_ts();
    let state = ExternalEdgeSignalState {
        ts: now,
        confidence: confidence.clamp(0, 10),
        hard_signal,
    };
    store_external_edge_signal_state(store, identity.as_str(), &state);
}

pub(crate) fn record_external_payload_rejection<S: crate::challenge::KeyValueStore>(store: &S) {
    increment_counter(store, "fingerprint:events");
    increment_counter(store, "fingerprint:external_invalid_payload");
}

fn load_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    identity: &str,
    now: u64,
    ttl_seconds: u64,
) -> Option<FingerprintState> {
    let key = format!("{}{}", FP_KEY_PREFIX_STATE, identity);
    let state = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_slice::<FingerprintState>(&raw).ok());

    let Some(state) = state else {
        return None;
    };

    if now.saturating_sub(state.ts) > ttl_seconds.max(1) {
        let _ = store.delete(key.as_str());
        return None;
    }

    Some(state)
}

fn store_state<S: crate::challenge::KeyValueStore>(store: &S, identity: &str, state: &FingerprintState) {
    let key = format!("{}{}", FP_KEY_PREFIX_STATE, identity);
    let Ok(raw) = serde_json::to_vec(state) else {
        return;
    };
    let _ = store.set(key.as_str(), &raw);
}

fn update_flow_mismatch_count<S: crate::challenge::KeyValueStore>(
    store: &S,
    identity: &str,
    now: u64,
    window_seconds: u64,
    mismatch_observed: bool,
) -> u32 {
    let safe_window_seconds = window_seconds.max(1);
    let bucket = now / safe_window_seconds;
    let counter_key = format!("{}{}:{}", FP_KEY_PREFIX_FLOW, identity, bucket);
    let last_bucket_key = format!("{}{}", FP_KEY_PREFIX_FLOW_LAST_BUCKET, identity);

    let prior_bucket = store
        .get(last_bucket_key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u64>().ok());
    if let Some(prior_bucket) = prior_bucket {
        if prior_bucket != bucket {
            let stale_key = format!("{}{}:{}", FP_KEY_PREFIX_FLOW, identity, prior_bucket);
            let _ = store.delete(stale_key.as_str());
        }
    }
    let _ = store.set(last_bucket_key.as_str(), bucket.to_string().as_bytes());

    let mut count = store
        .get(counter_key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(0);
    if mismatch_observed {
        count = count.saturating_add(1);
        let _ = store.set(counter_key.as_str(), count.to_string().as_bytes());
    }
    count
}

fn increment_counter<S: crate::challenge::KeyValueStore>(store: &S, key: &str) {
    let current = store
        .get(key)
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0);
    let _ = store.set(key, current.saturating_add(1).to_string().as_bytes());
}

fn has_cookie(req: &Request, key: &str) -> bool {
    let Some(cookie_header) = req.header("cookie").and_then(|value| value.as_str()) else {
        return false;
    };
    cookie_header
        .split(';')
        .map(str::trim)
        .filter_map(|part| part.split_once('='))
        .any(|(cookie_key, _)| cookie_key == key)
}

fn temporal_transition_impossible(
    prior: Option<&FingerprintState>,
    now: u64,
    window_seconds: u64,
    current_ua_family: &str,
    current_ja4_hash: Option<&str>,
) -> bool {
    let Some(prior) = prior else {
        return false;
    };
    let age = now.saturating_sub(prior.ts);
    if age > window_seconds {
        return false;
    }

    if prior.ua_family != current_ua_family
        && prior.ua_family != "other"
        && current_ua_family != "other"
    {
        return true;
    }

    match (prior.ja4_hash.as_deref(), current_ja4_hash) {
        (Some(previous), Some(current))
            if previous != current
                && age <= (window_seconds / 2).max(1)
                && prior.ua_family == current_ua_family =>
        {
            true
        }
        _ => false,
    }
}

fn ua_transport_family_mismatch(ua_family: &str, transport: &TransportEvidence) -> bool {
    let Some(edge_family) = transport.edge_browser_family.as_deref() else {
        return false;
    };
    let edge_family = normalize_browser_family(edge_family);
    edge_family != "other" && ua_family != "other" && edge_family != ua_family
}

fn fingerprint_signal_catalog() -> [(&'static str, &'static str, SignalFamily); 7] {
    [
        (
            FP_UA_CH_MISMATCH_KEY,
            "UA and client-hint mismatch",
            SignalFamily::FingerprintHeaderRuntime,
        ),
        (
            FP_UA_TRANSPORT_MISMATCH_KEY,
            "UA and transport/browser mismatch",
            SignalFamily::FingerprintTransport,
        ),
        (
            FP_TEMPORAL_TRANSITION_KEY,
            "Impossible short-window fingerprint transition",
            SignalFamily::FingerprintTemporal,
        ),
        (
            FP_FLOW_VIOLATION_KEY,
            "Flow-window mismatch threshold exceeded",
            SignalFamily::FingerprintBehavior,
        ),
        (
            FP_PERSISTENCE_MARKER_KEY,
            "Expected persistence marker missing",
            SignalFamily::FingerprintPersistence,
        ),
        (
            FP_UNTRUSTED_TRANSPORT_HEADER_KEY,
            "Untrusted transport fingerprint headers present",
            SignalFamily::FingerprintTransport,
        ),
        (
            FP_AKAMAI_EDGE_ADDITIVE_KEY,
            "Akamai edge bot signal (additive)",
            SignalFamily::FingerprintTransport,
        ),
    ]
}

fn disabled_fingerprint_signals() -> Vec<BotSignal> {
    fingerprint_signal_catalog()
        .iter()
        .map(|(key, label, family)| {
            BotSignal::disabled_with_metadata(
                key,
                label,
                SignalProvenance::Internal,
                10,
                *family,
            )
        })
        .collect()
}

pub(crate) fn collect_bot_signals<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    cfg: &crate::config::Config,
    ip: &str,
    headers_trusted: bool,
) -> Vec<BotSignal> {
    if !cfg.fingerprint_signal_enabled {
        return disabled_fingerprint_signals();
    }

    let identity = flow_identity(ip, cfg);
    let ua_family = extract_ua_family(req);
    let transport = extract_transport_evidence(req, headers_trusted);
    let ua_ch_mismatch = detect_ua_client_hint_mismatch(req);
    let ua_transport_mismatch = ua_transport_family_mismatch(ua_family, &transport);
    let now = now_ts();
    maybe_run_fingerprint_retention_cleanup(store, cfg, now);
    let edge_additive_state = if cfg.provider_backends.fingerprint_signal
        == crate::config::ProviderBackend::External
        && cfg.edge_integration_mode == crate::config::EdgeIntegrationMode::Additive
    {
        load_external_edge_signal_state(
            store,
            identity.as_str(),
            now,
            external_edge_signal_ttl_seconds(cfg),
        )
    } else {
        None
    };
    let akamai_edge_additive_active = edge_additive_state
        .as_ref()
        .map(|state| state.hard_signal || state.confidence >= AKAMAI_EDGE_ADDITIVE_CONFIDENCE_MIN)
        .unwrap_or(false);

    let ja4_hash = transport.ja4.as_deref().map(|ja4| hash_prefix(ja4, 16));
    let previous_state = load_state(
        store,
        identity.as_str(),
        now,
        cfg.fingerprint_state_ttl_seconds,
    );
    let temporal_transition = temporal_transition_impossible(
        previous_state.as_ref(),
        now,
        cfg.fingerprint_flow_window_seconds,
        ua_family,
        ja4_hash.as_deref(),
    );

    let mismatch_observed =
        ua_ch_mismatch || ua_transport_mismatch || temporal_transition || transport.untrusted_headers_present;
    let mismatch_count = update_flow_mismatch_count(
        store,
        identity.as_str(),
        now,
        cfg.fingerprint_flow_window_seconds,
        mismatch_observed,
    );
    let flow_violation = mismatch_count >= cfg.fingerprint_flow_violation_threshold as u32;

    let persistence_marker_missing =
        has_cookie(req, "js_verified") && !has_cookie(req, "shuma_fp") && mismatch_observed;

    store_state(
        store,
        identity.as_str(),
        &FingerprintState {
            ts: now,
            ua_family: ua_family.to_string(),
            ja4_hash,
        },
    );

    increment_counter(store, "fingerprint:events");
    if ua_ch_mismatch {
        increment_counter(store, "fingerprint:ua_ch_mismatch");
    }
    if ua_transport_mismatch {
        increment_counter(store, "fingerprint:ua_transport_mismatch");
    }
    if temporal_transition {
        increment_counter(store, "fingerprint:temporal_transition");
    }
    if flow_violation {
        increment_counter(store, "fingerprint:flow_violation");
    }
    if persistence_marker_missing {
        increment_counter(store, "fingerprint:persistence_marker_missing");
    }
    if transport.untrusted_headers_present {
        increment_counter(store, "fingerprint:untrusted_transport_header");
    }

    let transport_identity_present = transport.ja3.is_some() || transport.ja4.is_some();
    let edge_confidence = if transport.edge_score.unwrap_or(0.0) >= 80.0 {
        if transport_identity_present {
            9
        } else {
            8
        }
    } else {
        7
    };
    let mut signals = Vec::with_capacity(7);
    signals.push(BotSignal::scored_with_metadata(
        FP_UA_CH_MISMATCH_KEY,
        "UA and client-hint mismatch",
        ua_ch_mismatch,
        WEIGHT_UA_CH_MISMATCH,
        SignalProvenance::Internal,
        8,
        SignalFamily::FingerprintHeaderRuntime,
    ));
    signals.push(BotSignal::scored_with_metadata(
        FP_UA_TRANSPORT_MISMATCH_KEY,
        "UA and transport/browser mismatch",
        ua_transport_mismatch,
        WEIGHT_UA_TRANSPORT_MISMATCH,
        SignalProvenance::ExternalTrusted,
        edge_confidence,
        SignalFamily::FingerprintTransport,
    ));
    signals.push(BotSignal::scored_with_metadata(
        FP_TEMPORAL_TRANSITION_KEY,
        "Impossible short-window fingerprint transition",
        temporal_transition,
        WEIGHT_TEMPORAL_TRANSITION,
        SignalProvenance::Derived,
        8,
        SignalFamily::FingerprintTemporal,
    ));
    signals.push(BotSignal::scored_with_metadata(
        FP_FLOW_VIOLATION_KEY,
        "Flow-window mismatch threshold exceeded",
        flow_violation,
        WEIGHT_FLOW_VIOLATION,
        SignalProvenance::Derived,
        7,
        SignalFamily::FingerprintBehavior,
    ));
    signals.push(BotSignal::scored_with_metadata(
        FP_PERSISTENCE_MARKER_KEY,
        "Expected persistence marker missing",
        persistence_marker_missing,
        WEIGHT_PERSISTENCE_MARKER_MISSING,
        SignalProvenance::Internal,
        6,
        SignalFamily::FingerprintPersistence,
    ));
    signals.push(BotSignal::scored_with_metadata(
        FP_UNTRUSTED_TRANSPORT_HEADER_KEY,
        "Untrusted transport fingerprint headers present",
        transport.untrusted_headers_present,
        WEIGHT_UNTRUSTED_TRANSPORT_HEADER,
        SignalProvenance::ExternalUntrusted,
        9,
        SignalFamily::FingerprintTransport,
    ));
    if cfg.provider_backends.fingerprint_signal == crate::config::ProviderBackend::External
        && cfg.edge_integration_mode == crate::config::EdgeIntegrationMode::Additive
    {
        let additive_confidence = edge_additive_state
            .as_ref()
            .map(|state| state.confidence.max(6))
            .unwrap_or(7);
        signals.push(BotSignal::scored_with_metadata(
            FP_AKAMAI_EDGE_ADDITIVE_KEY,
            "Akamai edge bot signal (additive)",
            akamai_edge_additive_active,
            WEIGHT_AKAMAI_EDGE_ADDITIVE,
            SignalProvenance::ExternalTrusted,
            additive_confidence,
            SignalFamily::FingerprintTransport,
        ));
    } else {
        signals.push(BotSignal::disabled_with_metadata(
            FP_AKAMAI_EDGE_ADDITIVE_KEY,
            "Akamai edge bot signal (additive)",
            SignalProvenance::ExternalTrusted,
            10,
            SignalFamily::FingerprintTransport,
        ));
    }

    // Keep transport signal availability explicit when trusted transport ingestion is unavailable.
    if !headers_trusted && !transport.untrusted_headers_present {
        signals.push(BotSignal::unavailable_with_metadata(
            "fp_transport_signal_unavailable",
            "Transport fingerprint headers unavailable",
            SignalProvenance::ExternalTrusted,
            10,
            SignalFamily::FingerprintTransport,
        ));
    }

    signals
}

#[cfg(test)]
mod tests {
    use crate::challenge::KeyValueStore;
    use super::{
        collect_bot_signals, flow_identity, now_ts, record_akamai_edge_signal,
        record_external_payload_rejection, FingerprintState, FP_AKAMAI_EDGE_ADDITIVE_KEY,
        FP_FLOW_VIOLATION_KEY, FP_KEY_PREFIX_FLOW, FP_KEY_PREFIX_FLOW_LAST_BUCKET,
        FP_KEY_PREFIX_STATE, FP_TEMPORAL_TRANSITION_KEY,
        FP_UA_CH_MISMATCH_KEY, FP_UA_TRANSPORT_MISMATCH_KEY,
        FP_UNTRUSTED_TRANSPORT_HEADER_KEY,
    };
    use spin_sdk::http::Request;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
        get_keys_calls: Mutex<u64>,
    }

    impl crate::challenge::KeyValueStore for MockStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let map = self.map.lock().map_err(|_| ())?;
            Ok(map.get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut map = self.map.lock().map_err(|_| ())?;
            map.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut map = self.map.lock().map_err(|_| ())?;
            map.remove(key);
            Ok(())
        }

        fn get_keys(&self) -> Result<Vec<String>, ()> {
            let mut calls = self.get_keys_calls.lock().map_err(|_| ())?;
            *calls = calls.saturating_add(1);
            let map = self.map.lock().map_err(|_| ())?;
            Ok(map.keys().cloned().collect())
        }
    }

    fn request(path: &str, headers: &[(&str, &str)]) -> Request {
        let mut builder = Request::builder();
        builder.method(spin_sdk::http::Method::Get).uri(path);
        for (key, value) in headers {
            builder.header(*key, *value);
        }
        builder.build()
    }

    fn signal_active(signals: &[BotSignal], key: &str) -> bool {
        signals
            .iter()
            .find(|signal| signal.key == key)
            .map(|signal| signal.active)
            .unwrap_or(false)
    }

    fn get_keys_calls(store: &MockStore) -> u64 {
        *store
            .get_keys_calls
            .lock()
            .expect("get_keys call counter mutex poisoned")
    }

    use crate::signals::botness::BotSignal;

    #[test]
    fn detects_ua_client_hint_mismatch() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        let req = request(
            "/",
            &[
                ("user-agent", "Mozilla/5.0 Chrome/120.0"),
                ("sec-ch-ua", "\"Firefox\";v=\"122\""),
            ],
        );

        let signals = collect_bot_signals(&store, &req, &cfg, "203.0.113.10", false);
        assert!(signal_active(&signals, FP_UA_CH_MISMATCH_KEY));
    }

    #[test]
    fn detects_ua_transport_family_mismatch_when_headers_are_trusted() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        let req = request(
            "/",
            &[
                ("user-agent", "Mozilla/5.0 Chrome/120.0"),
                ("x-shuma-edge-browser-family", "firefox"),
            ],
        );

        let signals = collect_bot_signals(&store, &req, &cfg, "203.0.113.10", true);
        assert!(signal_active(&signals, FP_UA_TRANSPORT_MISMATCH_KEY));
    }

    #[test]
    fn marks_untrusted_transport_headers_as_signal() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        let req = request(
            "/",
            &[
                ("user-agent", "Mozilla/5.0 Chrome/120.0"),
                ("x-shuma-edge-ja4", "t13d1516h2_8daaf6152771_b6f405a00624"),
            ],
        );

        let signals = collect_bot_signals(&store, &req, &cfg, "203.0.113.11", false);
        assert!(signal_active(&signals, FP_UNTRUSTED_TRANSPORT_HEADER_KEY));
    }

    #[test]
    fn detects_temporal_impossible_transition_in_same_window() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        cfg.fingerprint_flow_window_seconds = 120;

        let req_chrome = request("/", &[("user-agent", "Mozilla/5.0 Chrome/120.0")]);
        let req_safari = request(
            "/",
            &[("user-agent", "Mozilla/5.0 Version/17.0 Safari/605.1.15")],
        );
        let _first = collect_bot_signals(&store, &req_chrome, &cfg, "203.0.113.12", true);
        let second = collect_bot_signals(&store, &req_safari, &cfg, "203.0.113.12", true);
        assert!(signal_active(&second, FP_TEMPORAL_TRANSITION_KEY));
    }

    #[test]
    fn detects_flow_violation_after_threshold() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        cfg.fingerprint_flow_violation_threshold = 2;
        let req = request(
            "/",
            &[
                ("user-agent", "Mozilla/5.0 Chrome/120.0"),
                ("sec-ch-ua", "\"Firefox\";v=\"122\""),
            ],
        );
        let _first = collect_bot_signals(&store, &req, &cfg, "203.0.113.13", true);
        let second = collect_bot_signals(&store, &req, &cfg, "203.0.113.13", true);
        assert!(signal_active(&second, FP_FLOW_VIOLATION_KEY));
    }

    #[test]
    fn stale_fingerprint_state_is_evicted_by_configured_ttl() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        cfg.fingerprint_state_ttl_seconds = 60;
        cfg.fingerprint_flow_window_seconds = 300;

        let ip = "203.0.113.14";
        let identity = flow_identity(ip, &cfg);
        let state_key = format!("{}{}", FP_KEY_PREFIX_STATE, identity);
        let stale_state = FingerprintState {
            ts: now_ts().saturating_sub(120),
            ua_family: "chrome".to_string(),
            ja4_hash: Some("legacyhash".to_string()),
        };
        let serialized = serde_json::to_vec(&stale_state).unwrap();
        store.set(state_key.as_str(), &serialized).unwrap();

        let req = request(
            "/",
            &[("user-agent", "Mozilla/5.0 Version/17.0 Safari/605.1.15")],
        );
        let observed_start = now_ts();
        let signals = collect_bot_signals(&store, &req, &cfg, ip, true);
        let observed_end = now_ts();
        assert!(!signal_active(&signals, FP_TEMPORAL_TRANSITION_KEY));

        let persisted_raw = store.get(state_key.as_str()).unwrap().unwrap();
        let persisted: FingerprintState = serde_json::from_slice(&persisted_raw).unwrap();
        assert!((observed_start..=observed_end).contains(&persisted.ts));
        assert_eq!(persisted.ua_family, "safari");
    }

    #[test]
    fn stale_fingerprint_state_cleanup_evicts_other_identities() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        cfg.fingerprint_state_ttl_seconds = 60;
        cfg.fingerprint_flow_window_seconds = 300;

        let stale_identity = flow_identity("203.0.113.30", &cfg);
        let stale_state_key = format!("{}{}", FP_KEY_PREFIX_STATE, stale_identity);
        let stale_state = FingerprintState {
            ts: now_ts().saturating_sub(120),
            ua_family: "chrome".to_string(),
            ja4_hash: Some("stalehash".to_string()),
        };
        store
            .set(
                stale_state_key.as_str(),
                serde_json::to_vec(&stale_state).unwrap().as_slice(),
            )
            .unwrap();

        let req = request("/", &[("user-agent", "Mozilla/5.0 Chrome/120.0")]);
        let _signals = collect_bot_signals(&store, &req, &cfg, "203.0.113.31", true);

        assert_eq!(store.get(stale_state_key.as_str()).unwrap(), None);
    }

    #[test]
    fn stale_fingerprint_flow_cleanup_evicts_old_buckets_but_keeps_current_window() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        cfg.fingerprint_flow_window_seconds = 120;

        let current_bucket = now_ts() / cfg.fingerprint_flow_window_seconds;
        let stale_bucket = current_bucket.saturating_sub(2);
        let stale_identity = flow_identity("203.0.113.40", &cfg);
        let active_identity = flow_identity("203.0.113.41", &cfg);
        let stale_flow_key = format!("{}{}:{}", FP_KEY_PREFIX_FLOW, stale_identity, stale_bucket);
        let stale_last_bucket_key = format!("{}{}", FP_KEY_PREFIX_FLOW_LAST_BUCKET, stale_identity);
        let active_flow_key = format!("{}{}:{}", FP_KEY_PREFIX_FLOW, active_identity, current_bucket);
        let active_last_bucket_key = format!("{}{}", FP_KEY_PREFIX_FLOW_LAST_BUCKET, active_identity);
        store.set(stale_flow_key.as_str(), b"2").unwrap();
        store
            .set(
                stale_last_bucket_key.as_str(),
                stale_bucket.to_string().as_bytes(),
            )
            .unwrap();
        store.set(active_flow_key.as_str(), b"1").unwrap();
        store
            .set(
                active_last_bucket_key.as_str(),
                current_bucket.to_string().as_bytes(),
            )
            .unwrap();

        let req = request("/", &[("user-agent", "Mozilla/5.0 Chrome/120.0")]);
        let _signals = collect_bot_signals(&store, &req, &cfg, "203.0.113.42", true);

        assert_eq!(store.get(stale_flow_key.as_str()).unwrap(), None);
        assert_eq!(store.get(stale_last_bucket_key.as_str()).unwrap(), None);
        assert_eq!(store.get(active_flow_key.as_str()).unwrap(), Some(b"1".to_vec()));
        assert_eq!(
            store.get(active_last_bucket_key.as_str()).unwrap(),
            Some(current_bucket.to_string().into_bytes())
        );
    }

    #[test]
    fn fingerprint_cleanup_scan_is_cadence_gated() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        cfg.fingerprint_state_ttl_seconds = 300;
        cfg.fingerprint_flow_window_seconds = 300;

        let req = request("/", &[("user-agent", "Mozilla/5.0 Chrome/120.0")]);
        let _first = collect_bot_signals(&store, &req, &cfg, "203.0.113.50", true);
        let _second = collect_bot_signals(&store, &req, &cfg, "203.0.113.51", true);
        let _third = collect_bot_signals(&store, &req, &cfg, "203.0.113.52", true);

        assert_eq!(get_keys_calls(&store), 1);
    }

    #[test]
    fn additive_akamai_signal_contributes_when_recent_high_confidence_state_exists() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        cfg.provider_backends.fingerprint_signal = crate::config::ProviderBackend::External;
        cfg.edge_integration_mode = crate::config::EdgeIntegrationMode::Additive;

        let ip = "203.0.113.15";
        record_akamai_edge_signal(&store, &cfg, ip, 9, false);
        let req = request("/", &[("user-agent", "Mozilla/5.0 Chrome/120.0")]);
        let signals = collect_bot_signals(&store, &req, &cfg, ip, true);

        let additive = signals
            .iter()
            .find(|signal| signal.key == FP_AKAMAI_EDGE_ADDITIVE_KEY)
            .expect("missing Akamai additive signal");
        assert!(additive.active);
        assert_eq!(additive.contribution, 2);
    }

    #[test]
    fn additive_akamai_signal_is_disabled_when_edge_mode_is_off() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.fingerprint_signal_enabled = true;
        cfg.provider_backends.fingerprint_signal = crate::config::ProviderBackend::External;
        cfg.edge_integration_mode = crate::config::EdgeIntegrationMode::Off;

        let ip = "203.0.113.16";
        record_akamai_edge_signal(&store, &cfg, ip, 10, true);
        let req = request("/", &[("user-agent", "Mozilla/5.0 Chrome/120.0")]);
        let signals = collect_bot_signals(&store, &req, &cfg, ip, true);

        let additive = signals
            .iter()
            .find(|signal| signal.key == FP_AKAMAI_EDGE_ADDITIVE_KEY)
            .expect("missing Akamai additive signal");
        assert_eq!(
            additive.availability,
            crate::signals::botness::SignalAvailability::Disabled
        );
        assert_eq!(additive.contribution, 0);
    }

    #[test]
    fn external_payload_rejection_increments_fingerprint_counters() {
        let store = MockStore::default();
        record_external_payload_rejection(&store);

        let events = store
            .get("fingerprint:events")
            .unwrap()
            .and_then(|raw| String::from_utf8(raw).ok())
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(0);
        let invalid_payload = store
            .get("fingerprint:external_invalid_payload")
            .unwrap()
            .and_then(|raw| String::from_utf8(raw).ok())
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(0);
        assert_eq!(events, 1);
        assert_eq!(invalid_payload, 1);
    }
}
