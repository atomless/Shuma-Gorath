use serde::Deserialize;
use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;
use std::time::{SystemTime, UNIX_EPOCH};

use super::contracts::{
    BanListResult, BanLookupResult, BanStoreProvider, BanSyncResult, ChallengeEngineProvider,
    FingerprintSignalProvider, MazeTarpitProvider, RateLimitDecision, RateLimiterProvider,
    VerifiedIdentityProvider,
};
use super::internal;

const EXTERNAL_RATE_WINDOW_TTL_SECONDS: u64 = 120;
const RATE_ROUTE_CLASS_MAIN_TRAFFIC: &str = "main_traffic";
const RATE_ROUTE_CLASS_ADMIN_AUTH: &str = "admin_auth";
const RATE_DRIFT_BAND_DELTA_0: &str = "delta_0";
const RATE_DRIFT_BAND_DELTA_1_5: &str = "delta_1_5";
const RATE_DRIFT_BAND_DELTA_6_20: &str = "delta_6_20";
const RATE_DRIFT_BAND_DELTA_21_PLUS: &str = "delta_21_plus";
const MAX_AKAMAI_DETECTION_IDS: usize = 16;
const MAX_AKAMAI_TAGS: usize = 16;
const MAX_VERIFIED_IDENTITY_FIELD_CHARS: usize = 256;
const MAX_VERIFIED_IDENTITY_URI_CHARS: usize = 1024;
const VERIFIED_IDENTITY_SCHEME_HEADER: &str = "x-shuma-edge-verified-identity-scheme";
const VERIFIED_IDENTITY_VALUE_HEADER: &str = "x-shuma-edge-verified-identity";
const VERIFIED_IDENTITY_OPERATOR_HEADER: &str = "x-shuma-edge-verified-identity-operator";
const VERIFIED_IDENTITY_CATEGORY_HEADER: &str = "x-shuma-edge-verified-identity-category";
const VERIFIED_IDENTITY_END_USER_CONTROLLED_HEADER: &str =
    "x-shuma-edge-verified-identity-end-user-controlled";
const VERIFIED_IDENTITY_DIRECTORY_SOURCE_ID_HEADER: &str =
    "x-shuma-edge-verified-identity-directory-source-id";
const VERIFIED_IDENTITY_DIRECTORY_SOURCE_URI_HEADER: &str =
    "x-shuma-edge-verified-identity-directory-source-uri";

pub(crate) struct ExternalRateLimiterProvider;
pub(crate) struct ExternalBanStoreProvider;
pub(crate) struct UnsupportedExternalChallengeEngineProvider;
pub(crate) struct UnsupportedExternalMazeTarpitProvider;
pub(crate) struct ExternalFingerprintSignalProvider;
pub(crate) struct ExternalVerifiedIdentityProvider;

pub(crate) const RATE_LIMITER: ExternalRateLimiterProvider = ExternalRateLimiterProvider;
pub(crate) const BAN_STORE: ExternalBanStoreProvider = ExternalBanStoreProvider;
pub(crate) const UNSUPPORTED_CHALLENGE_ENGINE: UnsupportedExternalChallengeEngineProvider =
    UnsupportedExternalChallengeEngineProvider;
pub(crate) const UNSUPPORTED_MAZE_TARPIT: UnsupportedExternalMazeTarpitProvider =
    UnsupportedExternalMazeTarpitProvider;
pub(crate) const FINGERPRINT_SIGNAL: ExternalFingerprintSignalProvider =
    ExternalFingerprintSignalProvider;
pub(crate) const VERIFIED_IDENTITY: ExternalVerifiedIdentityProvider =
    ExternalVerifiedIdentityProvider;

#[derive(Debug, Clone, Deserialize)]
struct AkamaiEdgeOutcome {
    #[serde(default)]
    bot_score: Option<f32>,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    detection_ids: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Clone)]
struct NormalizedFingerprintSignal {
    confidence: f32,
    hard_signal: bool,
    checks: Vec<String>,
    summary: String,
}

#[derive(Debug, Clone)]
struct RawVerifiedIdentityHeaders {
    scheme: Option<String>,
    stable_identity: Option<String>,
    operator: Option<String>,
    category: Option<String>,
    end_user_controlled: Option<String>,
    directory_source_id: Option<String>,
    directory_source_uri: Option<String>,
}

impl RawVerifiedIdentityHeaders {
    fn any_present(&self) -> bool {
        self.scheme.is_some()
            || self.stable_identity.is_some()
            || self.operator.is_some()
            || self.category.is_some()
            || self.end_user_controlled.is_some()
            || self.directory_source_id.is_some()
            || self.directory_source_uri.is_some()
    }
}

fn normalize_akamai_edge_outcome(
    outcome: AkamaiEdgeOutcome,
) -> Result<NormalizedFingerprintSignal, &'static str> {
    if let Some(score) = outcome.bot_score {
        if !score.is_finite() || !(0.0..=100.0).contains(&score) {
            return Err("Invalid edge bot score");
        }
    }

    let normalized_action = outcome
        .action
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| "unknown".to_string());

    let mut confidence = outcome.bot_score.unwrap_or(0.0) / 10.0;
    let mut hard_signal = false;
    let mut checks = vec!["akamai_signal".to_string()];

    match normalized_action.as_str() {
        "deny" | "block" => {
            hard_signal = true;
            confidence = confidence.max(9.5);
            checks.push("automation_props".to_string());
        }
        "challenge" => {
            confidence = confidence.max(6.5);
            checks.push("cdp_timing".to_string());
        }
        "monitor" => {
            confidence = confidence.max(3.5);
        }
        "allow" => {
            confidence = confidence.max(1.0);
        }
        _ => {
            confidence = confidence.max(2.0);
        }
    }

    if let Some(action_check) = crate::request_validation::sanitize_check_name(
        format!("akamai_action:{normalized_action}").as_str(),
    ) {
        checks.push(action_check);
    }

    let detection_ids = outcome
        .detection_ids
        .into_iter()
        .take(MAX_AKAMAI_DETECTION_IDS)
        .filter_map(|id| crate::request_validation::sanitize_check_name(id.as_str()))
        .collect::<Vec<_>>();
    if !detection_ids.is_empty() {
        checks.push("akamai_detection_ids".to_string());
        confidence = (confidence + 1.0).min(10.0);
    }

    let tags = outcome
        .tags
        .into_iter()
        .take(MAX_AKAMAI_TAGS)
        .filter_map(|tag| crate::request_validation::sanitize_check_name(tag.as_str()))
        .collect::<Vec<_>>();
    if !tags.is_empty() {
        checks.push("akamai_tags".to_string());
        confidence = (confidence + 0.5).min(10.0);
    }

    for id in &detection_ids {
        if let Some(check) =
            crate::request_validation::sanitize_check_name(format!("akamai_id:{id}").as_str())
        {
            checks.push(check);
        }
    }
    for tag in &tags {
        if let Some(check) =
            crate::request_validation::sanitize_check_name(format!("akamai_tag:{tag}").as_str())
        {
            checks.push(check);
        }
    }

    checks.sort();
    checks.dedup();

    let summary = crate::request_validation::sanitize_ban_summary(
        format!(
            "provider=akamai action={} score={:.1} ids={} tags={}",
            normalized_action,
            confidence,
            if detection_ids.is_empty() {
                "none".to_string()
            } else {
                detection_ids.join(",")
            },
            if tags.is_empty() {
                "none".to_string()
            } else {
                tags.join(",")
            }
        )
        .as_str(),
    )
    .unwrap_or_else(|| "provider=akamai".to_string());

    Ok(NormalizedFingerprintSignal {
        confidence,
        hard_signal,
        checks,
        summary,
    })
}

fn map_normalized_fingerprint_to_cdp_report(
    normalized: &NormalizedFingerprintSignal,
) -> crate::signals::cdp::CdpReport {
    crate::signals::cdp::CdpReport {
        cdp_detected: normalized.hard_signal || normalized.confidence >= 4.0,
        score: (normalized.confidence / 2.0).clamp(0.0, 5.0),
        checks: normalized.checks.clone(),
    }
}

fn cdp_tier_label(tier: crate::signals::cdp::CdpTier) -> &'static str {
    match tier {
        crate::signals::cdp::CdpTier::Low => "low",
        crate::signals::cdp::CdpTier::Medium => "medium",
        crate::signals::cdp::CdpTier::Strong => "strong",
    }
}

fn looks_like_akamai_payload(outcome: &AkamaiEdgeOutcome) -> bool {
    outcome.bot_score.is_some()
        || outcome
            .action
            .as_deref()
            .map(str::trim)
            .map(|value| !value.is_empty())
            .unwrap_or(false)
        || !outcome.detection_ids.is_empty()
        || !outcome.tags.is_empty()
}

fn fingerprint_authoritative_mode_enabled(mode: crate::config::EdgeIntegrationMode) -> bool {
    mode == crate::config::EdgeIntegrationMode::Authoritative
}

fn header_value(req: &Request, name: &str) -> Option<String> {
    req.header(name)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_identity_text(raw: &str, max_chars: usize) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.len() > max_chars
        || trimmed.chars().any(|ch| ch.is_ascii_control())
    {
        return None;
    }

    Some(trimmed.to_string())
}

fn parse_identity_category(raw: &str) -> Option<crate::bot_identity::contracts::IdentityCategory> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "training" => Some(crate::bot_identity::contracts::IdentityCategory::Training),
        "search" => Some(crate::bot_identity::contracts::IdentityCategory::Search),
        "user_triggered_agent" => {
            Some(crate::bot_identity::contracts::IdentityCategory::UserTriggeredAgent)
        }
        "preview" => Some(crate::bot_identity::contracts::IdentityCategory::Preview),
        "service_agent" => Some(crate::bot_identity::contracts::IdentityCategory::ServiceAgent),
        "other" => Some(crate::bot_identity::contracts::IdentityCategory::Other),
        _ => None,
    }
}

fn parse_provider_identity_scheme(
    raw: &str,
) -> Result<
    crate::bot_identity::contracts::IdentityScheme,
    crate::bot_identity::verification::IdentityVerificationFailure,
> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "provider_verified_bot" => {
            Ok(crate::bot_identity::contracts::IdentityScheme::ProviderVerifiedBot)
        }
        "provider_signed_agent" => {
            Ok(crate::bot_identity::contracts::IdentityScheme::ProviderSignedAgent)
        }
        "http_message_signatures" | "mtls" => {
            Err(crate::bot_identity::verification::IdentityVerificationFailure::UnsupportedScheme)
        }
        _ => Err(crate::bot_identity::verification::IdentityVerificationFailure::ProviderRejected),
    }
}

fn parse_bool_header(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn extract_verified_identity_headers(req: &Request) -> RawVerifiedIdentityHeaders {
    RawVerifiedIdentityHeaders {
        scheme: header_value(req, VERIFIED_IDENTITY_SCHEME_HEADER),
        stable_identity: header_value(req, VERIFIED_IDENTITY_VALUE_HEADER),
        operator: header_value(req, VERIFIED_IDENTITY_OPERATOR_HEADER),
        category: header_value(req, VERIFIED_IDENTITY_CATEGORY_HEADER),
        end_user_controlled: header_value(req, VERIFIED_IDENTITY_END_USER_CONTROLLED_HEADER),
        directory_source_id: header_value(req, VERIFIED_IDENTITY_DIRECTORY_SOURCE_ID_HEADER),
        directory_source_uri: header_value(req, VERIFIED_IDENTITY_DIRECTORY_SOURCE_URI_HEADER),
    }
}

fn normalize_verified_identity_headers(
    headers: RawVerifiedIdentityHeaders,
) -> Result<
    crate::bot_identity::contracts::VerifiedIdentityEvidence,
    crate::bot_identity::verification::IdentityVerificationFailure,
> {
    let scheme = headers
        .scheme
        .as_deref()
        .ok_or(crate::bot_identity::verification::IdentityVerificationFailure::MissingAssertion)
        .and_then(parse_provider_identity_scheme)?;
    let stable_identity = headers
        .stable_identity
        .as_deref()
        .and_then(|value| parse_identity_text(value, MAX_VERIFIED_IDENTITY_FIELD_CHARS))
        .ok_or(crate::bot_identity::verification::IdentityVerificationFailure::MissingAssertion)?;
    let operator = headers
        .operator
        .as_deref()
        .and_then(|value| parse_identity_text(value, MAX_VERIFIED_IDENTITY_FIELD_CHARS))
        .ok_or(crate::bot_identity::verification::IdentityVerificationFailure::MissingAssertion)?;
    let category = headers
        .category
        .as_deref()
        .and_then(parse_identity_category)
        .ok_or(crate::bot_identity::verification::IdentityVerificationFailure::MissingAssertion)?;
    let end_user_controlled = match headers.end_user_controlled.as_deref() {
        Some(value) => parse_bool_header(value).ok_or(
            crate::bot_identity::verification::IdentityVerificationFailure::ProviderRejected,
        )?,
        None => false,
    };
    let directory_source = match (
        headers.directory_source_id.as_deref(),
        headers.directory_source_uri.as_deref(),
    ) {
        (None, None) => None,
        (Some(source_id), source_uri) => {
            let source_id = parse_identity_text(source_id, MAX_VERIFIED_IDENTITY_FIELD_CHARS)
                .ok_or(
                crate::bot_identity::verification::IdentityVerificationFailure::ProviderRejected,
            )?;
            let source_uri = match source_uri {
                Some(uri) => Some(
                    parse_identity_text(uri, MAX_VERIFIED_IDENTITY_URI_CHARS).ok_or(
                        crate::bot_identity::verification::IdentityVerificationFailure::ProviderRejected,
                    )?,
                ),
                None => None,
            };
            Some(crate::bot_identity::contracts::IdentityDirectorySource {
                source_id,
                source_uri,
            })
        }
        (None, Some(_)) => {
            return Err(
                crate::bot_identity::verification::IdentityVerificationFailure::ProviderRejected,
            )
        }
    };

    Ok(crate::bot_identity::contracts::VerifiedIdentityEvidence {
        scheme,
        stable_identity,
        operator,
        category,
        verification_strength:
            crate::bot_identity::contracts::VerificationStrength::ProviderAsserted,
        end_user_controlled,
        directory_source,
        provenance: crate::bot_identity::contracts::IdentityProvenance::Provider,
    })
}

trait DistributedRateCounter {
    fn current_usage(&self, key: &str) -> Result<u32, String>;
    fn increment_and_get(&self, key: &str, ttl_seconds: u64) -> Result<u32, String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RateLimiterOutageAction {
    FallbackInternal,
    Allow,
    Deny,
}

impl RateLimiterOutageAction {
    fn as_str(self) -> &'static str {
        match self {
            RateLimiterOutageAction::FallbackInternal => "fallback_internal",
            RateLimiterOutageAction::Allow => "allow",
            RateLimiterOutageAction::Deny => "deny",
        }
    }
}

struct RedisDistributedRateCounter {
    address: String,
}

impl RedisDistributedRateCounter {
    fn from_env() -> Option<Self> {
        crate::config::rate_limiter_redis_url().map(|address| Self { address })
    }

    fn open_connection(&self) -> Result<spin_sdk::redis::Connection, String> {
        spin_sdk::redis::Connection::open(&self.address)
            .map_err(|err| format!("redis connection failed ({:?})", err))
    }
}

impl DistributedRateCounter for RedisDistributedRateCounter {
    fn current_usage(&self, key: &str) -> Result<u32, String> {
        let conn = self.open_connection()?;
        let payload = conn
            .get(key)
            .map_err(|err| format!("redis GET failed ({:?})", err))?;
        let Some(bytes) = payload else {
            return Ok(0);
        };
        let raw =
            String::from_utf8(bytes).map_err(|_| "redis payload was not UTF-8".to_string())?;
        raw.trim()
            .parse::<u32>()
            .map_err(|_| "redis payload was not a valid u32 counter".to_string())
    }

    fn increment_and_get(&self, key: &str, ttl_seconds: u64) -> Result<u32, String> {
        let conn = self.open_connection()?;
        let next = conn
            .incr(key)
            .map_err(|err| format!("redis INCR failed ({:?})", err))?;

        if next == 1 {
            let ttl = i64::try_from(ttl_seconds).unwrap_or(i64::MAX);
            let args = [
                spin_sdk::redis::RedisParameter::Binary(key.as_bytes().to_vec()),
                spin_sdk::redis::RedisParameter::Int64(ttl),
            ];
            if let Err(err) = conn.execute("EXPIRE", &args) {
                eprintln!(
                    "[providers][rate] redis EXPIRE failed for key {} ({:?})",
                    key, err
                );
            }
        }

        if next < 0 {
            return Err("redis INCR returned a negative counter".to_string());
        }
        u32::try_from(next).map_err(|_| "redis INCR exceeded u32 counter range".to_string())
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn current_window_key(site_id: &str, ip: &str, window: u64) -> String {
    let bucket = crate::signals::ip_identity::bucket_ip(ip);
    format!("rate:{}:{}:{}", site_id, bucket, window)
}

fn current_window_rate_key(site_id: &str, ip: &str) -> String {
    current_window_key(site_id, ip, now_ts() / 60)
}

fn rate_route_class(site_id: &str) -> &'static str {
    if site_id.starts_with("admin-auth-") {
        RATE_ROUTE_CLASS_ADMIN_AUTH
    } else {
        RATE_ROUTE_CLASS_MAIN_TRAFFIC
    }
}

fn rate_outage_mode_for_route_class(route_class: &str) -> crate::config::RateLimiterOutageMode {
    if route_class == RATE_ROUTE_CLASS_ADMIN_AUTH {
        crate::config::rate_limiter_outage_mode_admin_auth()
    } else {
        crate::config::rate_limiter_outage_mode_main()
    }
}

fn decide_rate_limit_on_outage(
    outage_mode: crate::config::RateLimiterOutageMode,
    fallback: impl FnOnce() -> RateLimitDecision,
) -> (RateLimitDecision, RateLimiterOutageAction) {
    match outage_mode {
        crate::config::RateLimiterOutageMode::FallbackInternal => {
            (fallback(), RateLimiterOutageAction::FallbackInternal)
        }
        crate::config::RateLimiterOutageMode::FailOpen => {
            (RateLimitDecision::Allowed, RateLimiterOutageAction::Allow)
        }
        crate::config::RateLimiterOutageMode::FailClosed => {
            (RateLimitDecision::Limited, RateLimiterOutageAction::Deny)
        }
    }
}

fn rate_drift_band(delta: u32) -> &'static str {
    match delta {
        0 => RATE_DRIFT_BAND_DELTA_0,
        1..=5 => RATE_DRIFT_BAND_DELTA_1_5,
        6..=20 => RATE_DRIFT_BAND_DELTA_6_20,
        _ => RATE_DRIFT_BAND_DELTA_21_PLUS,
    }
}

fn record_rate_backend_error_metric(store: &Store, route_class: &str) {
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::RateLimiterBackendErrors,
        Some(route_class),
    );
}

fn record_rate_outage_decision_metric(
    store: &Store,
    route_class: &str,
    outage_mode: crate::config::RateLimiterOutageMode,
    action: RateLimiterOutageAction,
    decision: RateLimitDecision,
) {
    let label = format!(
        "{}:{}:{}:{}",
        route_class,
        outage_mode.as_str(),
        action.as_str(),
        decision.as_str()
    );
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::RateLimiterOutageDecisions,
        Some(label.as_str()),
    );
    if action == RateLimiterOutageAction::Allow && decision == RateLimitDecision::Allowed {
        crate::observability::monitoring::record_rate_outcome(store, "fallback_allow");
    }
    if action == RateLimiterOutageAction::Deny && decision == RateLimitDecision::Limited {
        crate::observability::monitoring::record_rate_outcome(store, "fallback_deny");
    }
}

fn record_rate_usage_fallback_metric(store: &Store, route_class: &str, reason: &str) {
    let label = format!("{}:{}", route_class, reason);
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::RateLimiterUsageFallback,
        Some(label.as_str()),
    );
}

fn record_rate_drift_metric(store: &Store, route_class: &str, delta: u32) {
    let band = rate_drift_band(delta);
    let label = format!("{}:{}", route_class, band);
    crate::observability::metrics::increment(
        store,
        crate::observability::metrics::MetricName::RateLimiterStateDriftObservations,
        Some(label.as_str()),
    );
}

#[cfg(test)]
fn current_rate_usage_with_backend<B: DistributedRateCounter>(
    backend: Option<&B>,
    site_id: &str,
    ip: &str,
    fallback: impl FnOnce() -> u32,
) -> u32 {
    if let Some(distributed_backend) = backend {
        let key = current_window_rate_key(site_id, ip);
        match distributed_backend.current_usage(&key) {
            Ok(count) => return count,
            Err(err) => eprintln!(
                "[providers][rate] external distributed usage read failed for key {} ({}); falling back to internal",
                key, err
            ),
        }
    }

    fallback()
}

#[cfg(test)]
fn check_rate_limit_with_backend<B: DistributedRateCounter>(
    backend: Option<&B>,
    site_id: &str,
    ip: &str,
    limit: u32,
    fallback: impl FnOnce() -> RateLimitDecision,
) -> RateLimitDecision {
    if limit == 0 {
        return RateLimitDecision::Limited;
    }

    if let Some(distributed_backend) = backend {
        let key = current_window_rate_key(site_id, ip);
        match distributed_backend.increment_and_get(&key, EXTERNAL_RATE_WINDOW_TTL_SECONDS) {
            Ok(next) => {
                if next > limit {
                    RateLimitDecision::Limited
                } else {
                    RateLimitDecision::Allowed
                }
            }
            Err(err) => {
                eprintln!(
                    "[providers][rate] external distributed limiter failed for key {} ({}); falling back to internal",
                    key, err
                );
                fallback()
            }
        }
    } else {
        fallback()
    }
}

impl RateLimiterProvider for ExternalRateLimiterProvider {
    fn current_rate_usage(&self, store: &Store, site_id: &str, ip: &str) -> u32 {
        let route_class = rate_route_class(site_id);
        let distributed_backend = RedisDistributedRateCounter::from_env();
        let Some(backend) = distributed_backend.as_ref() else {
            record_rate_usage_fallback_metric(store, route_class, "backend_missing");
            return internal::RATE_LIMITER.current_rate_usage(store, site_id, ip);
        };

        let key = current_window_rate_key(site_id, ip);
        match backend.current_usage(&key) {
            Ok(count) => count,
            Err(err) => {
                eprintln!(
                    "[providers][rate] external distributed usage read failed for key {} ({}); falling back to internal",
                    key, err
                );
                record_rate_backend_error_metric(store, route_class);
                record_rate_usage_fallback_metric(store, route_class, "backend_error");
                internal::RATE_LIMITER.current_rate_usage(store, site_id, ip)
            }
        }
    }

    fn check_rate_limit(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        limit: u32,
    ) -> RateLimitDecision {
        if limit == 0 {
            return RateLimitDecision::Limited;
        }

        let route_class = rate_route_class(site_id);
        let outage_mode = rate_outage_mode_for_route_class(route_class);
        let distributed_backend = RedisDistributedRateCounter::from_env();

        let Some(backend) = distributed_backend.as_ref() else {
            let (decision, action) = decide_rate_limit_on_outage(outage_mode, || {
                internal::RATE_LIMITER.check_rate_limit(store, site_id, ip, limit)
            });
            record_rate_outage_decision_metric(store, route_class, outage_mode, action, decision);
            return decision;
        };

        let key = current_window_rate_key(site_id, ip);
        match backend.increment_and_get(&key, EXTERNAL_RATE_WINDOW_TTL_SECONDS) {
            Ok(next) => {
                let decision = if next > limit {
                    RateLimitDecision::Limited
                } else {
                    RateLimitDecision::Allowed
                };
                // Shadow local counter for drift observability without changing enforcement path.
                let local_shadow_next = internal::RATE_LIMITER
                    .current_rate_usage(store, site_id, ip)
                    .saturating_add(1);
                let drift_delta = next.abs_diff(local_shadow_next);
                record_rate_drift_metric(store, route_class, drift_delta);
                decision
            }
            Err(err) => {
                eprintln!(
                    "[providers][rate] external distributed limiter failed for key {} ({}); applying outage posture",
                    key, err
                );
                record_rate_backend_error_metric(store, route_class);
                let (decision, action) = decide_rate_limit_on_outage(outage_mode, || {
                    internal::RATE_LIMITER.check_rate_limit(store, site_id, ip, limit)
                });
                record_rate_outage_decision_metric(
                    store,
                    route_class,
                    outage_mode,
                    action,
                    decision,
                );
                decision
            }
        }
    }
}

trait DistributedBanStore {
    fn is_banned(&self, site_id: &str, ip: &str) -> Result<bool, String>;
    fn list_active_bans(
        &self,
        site_id: &str,
    ) -> Result<Vec<(String, crate::enforcement::ban::BanEntry)>, String>;
    fn ban_ip_with_fingerprint(
        &self,
        site_id: &str,
        ip: &str,
        reason: &str,
        duration_secs: u64,
        fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
    ) -> Result<(), String>;
    fn unban_ip(&self, site_id: &str, ip: &str) -> Result<(), String>;
}

struct RedisDistributedBanStore {
    address: String,
}

impl RedisDistributedBanStore {
    fn from_env() -> Option<Self> {
        crate::config::ban_store_redis_url().map(|address| Self { address })
    }

    fn open_connection(&self) -> Result<spin_sdk::redis::Connection, String> {
        spin_sdk::redis::Connection::open(&self.address)
            .map_err(|err| format!("redis connection failed ({:?})", err))
    }
}

fn distributed_ban_key(site_id: &str, ip: &str) -> String {
    format!("ban:{}:{}", site_id, ip)
}

fn distributed_ban_key_pattern(site_id: &str) -> String {
    format!("ban:{}:*", site_id)
}

fn redis_result_as_string(result: &spin_sdk::redis::RedisResult) -> Option<String> {
    match result {
        spin_sdk::redis::RedisResult::Binary(bytes) => String::from_utf8(bytes.clone()).ok(),
        spin_sdk::redis::RedisResult::Status(value) => Some(value.clone()),
        _ => None,
    }
}

fn ban_lookup_result(is_banned: bool) -> BanLookupResult {
    if is_banned {
        BanLookupResult::Banned
    } else {
        BanLookupResult::NotBanned
    }
}

impl DistributedBanStore for RedisDistributedBanStore {
    fn is_banned(&self, site_id: &str, ip: &str) -> Result<bool, String> {
        let conn = self.open_connection()?;
        let key = distributed_ban_key(site_id, ip);
        let payload = conn
            .get(&key)
            .map_err(|err| format!("redis GET failed ({:?})", err))?;
        let Some(bytes) = payload else {
            return Ok(false);
        };

        let entry = match serde_json::from_slice::<crate::enforcement::ban::BanEntry>(&bytes) {
            Ok(entry) => entry,
            Err(_) => {
                if let Err(err) = conn.del(&[key.clone()]) {
                    eprintln!(
                        "[providers][ban] failed to delete invalid redis ban {} ({:?})",
                        key, err
                    );
                }
                return Ok(false);
            }
        };

        if entry.expires > now_ts() {
            return Ok(true);
        }

        if let Err(err) = conn.del(&[key.clone()]) {
            eprintln!(
                "[providers][ban] failed to delete expired redis ban {} ({:?})",
                key, err
            );
        }
        Ok(false)
    }

    fn list_active_bans(
        &self,
        site_id: &str,
    ) -> Result<Vec<(String, crate::enforcement::ban::BanEntry)>, String> {
        let conn = self.open_connection()?;
        let pattern = distributed_ban_key_pattern(site_id);
        let keys = conn
            .execute(
                "KEYS",
                &[spin_sdk::redis::RedisParameter::Binary(
                    pattern.as_bytes().to_vec(),
                )],
            )
            .map_err(|err| format!("redis KEYS failed ({:?})", err))?;

        let mut bans = Vec::new();
        let now = now_ts();

        for key in keys.iter().filter_map(redis_result_as_string) {
            let ip = key.split(':').next_back().unwrap_or("").to_string();
            if ip.is_empty() {
                continue;
            }

            let payload = match conn.get(&key) {
                Ok(payload) => payload,
                Err(err) => {
                    eprintln!(
                        "[providers][ban] redis GET failed for key {} ({:?})",
                        key, err
                    );
                    continue;
                }
            };
            let Some(bytes) = payload else {
                continue;
            };

            match serde_json::from_slice::<crate::enforcement::ban::BanEntry>(&bytes) {
                Ok(entry) if entry.expires > now => bans.push((ip, entry)),
                Ok(_) | Err(_) => {
                    if let Err(err) = conn.del(&[key.clone()]) {
                        eprintln!(
                            "[providers][ban] failed to delete stale redis ban {} ({:?})",
                            key, err
                        );
                    }
                }
            }
        }

        bans.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(bans)
    }

    fn ban_ip_with_fingerprint(
        &self,
        site_id: &str,
        ip: &str,
        reason: &str,
        duration_secs: u64,
        fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
    ) -> Result<(), String> {
        let conn = self.open_connection()?;
        let key = distributed_ban_key(site_id, ip);
        let ts = now_ts();
        let normalized_reason = crate::request_validation::sanitize_ban_reason(reason);
        let normalized_fingerprint = fingerprint.map(|mut fp| {
            fp.summary = fp
                .summary
                .as_deref()
                .and_then(crate::request_validation::sanitize_ban_summary);
            fp
        });
        let entry = crate::enforcement::ban::BanEntry {
            reason: normalized_reason,
            expires: ts.saturating_add(duration_secs),
            banned_at: ts,
            fingerprint: normalized_fingerprint,
        };
        let payload = serde_json::to_vec(&entry)
            .map_err(|err| format!("serialize ban failed ({:?})", err))?;
        conn.set(&key, &payload)
            .map_err(|err| format!("redis SET failed ({:?})", err))?;

        let ttl = i64::try_from(duration_secs.max(1)).unwrap_or(i64::MAX);
        let args = [
            spin_sdk::redis::RedisParameter::Binary(key.as_bytes().to_vec()),
            spin_sdk::redis::RedisParameter::Int64(ttl),
        ];
        if let Err(err) = conn.execute("EXPIRE", &args) {
            eprintln!(
                "[providers][ban] redis EXPIRE failed for key {} ({:?})",
                key, err
            );
        }
        Ok(())
    }

    fn unban_ip(&self, site_id: &str, ip: &str) -> Result<(), String> {
        let conn = self.open_connection()?;
        let key = distributed_ban_key(site_id, ip);
        conn.del(&[key])
            .map_err(|err| format!("redis DEL failed ({:?})", err))?;
        Ok(())
    }
}

fn is_banned_with_backend<B: DistributedBanStore>(
    backend: Option<&B>,
    outage_mode: crate::config::BanStoreOutageMode,
    site_id: &str,
    ip: &str,
    fallback: impl FnOnce() -> bool,
) -> BanLookupResult {
    if let Some(distributed_backend) = backend {
        match distributed_backend.is_banned(site_id, ip) {
            Ok(is_banned) => return ban_lookup_result(is_banned),
            Err(err) => eprintln!(
                "[providers][ban] external distributed ban check failed for site={} ip={} ({}); applying outage posture",
                site_id, ip, err
            ),
        }
    }
    match outage_mode {
        crate::config::BanStoreOutageMode::FallbackInternal => ban_lookup_result(fallback()),
        crate::config::BanStoreOutageMode::FailOpen
        | crate::config::BanStoreOutageMode::FailClosed => BanLookupResult::Unavailable,
    }
}

fn list_active_bans_with_backend<B: DistributedBanStore>(
    backend: Option<&B>,
    outage_mode: crate::config::BanStoreOutageMode,
    site_id: &str,
    fallback: impl FnOnce() -> Vec<(String, crate::enforcement::ban::BanEntry)>,
) -> BanListResult {
    if let Some(distributed_backend) = backend {
        match distributed_backend.list_active_bans(site_id) {
            Ok(bans) => return BanListResult::Available(bans),
            Err(err) => eprintln!(
                "[providers][ban] external distributed ban listing failed for site={} ({}); applying outage posture",
                site_id, err
            ),
        }
    }
    match outage_mode {
        crate::config::BanStoreOutageMode::FallbackInternal => BanListResult::Available(fallback()),
        crate::config::BanStoreOutageMode::FailOpen
        | crate::config::BanStoreOutageMode::FailClosed => BanListResult::Unavailable,
    }
}

fn ban_with_backend<B: DistributedBanStore>(
    backend: Option<&B>,
    outage_mode: crate::config::BanStoreOutageMode,
    site_id: &str,
    ip: &str,
    reason: &str,
    duration_secs: u64,
    fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
    fallback: impl FnOnce(),
) -> BanSyncResult {
    if let Some(distributed_backend) = backend {
        match distributed_backend
            .ban_ip_with_fingerprint(site_id, ip, reason, duration_secs, fingerprint.clone())
        {
            Ok(()) => return BanSyncResult::Synced,
            Err(err) => eprintln!(
                "[providers][ban] external distributed ban write failed for site={} ip={} ({}); applying outage posture",
                site_id, ip, err
            ),
        }
    }
    match outage_mode {
        crate::config::BanStoreOutageMode::FallbackInternal => {
            fallback();
            BanSyncResult::Deferred
        }
        crate::config::BanStoreOutageMode::FailOpen
        | crate::config::BanStoreOutageMode::FailClosed => BanSyncResult::Failed,
    }
}

fn unban_with_backend<B: DistributedBanStore>(
    backend: Option<&B>,
    outage_mode: crate::config::BanStoreOutageMode,
    site_id: &str,
    ip: &str,
    fallback: impl FnOnce(),
) -> BanSyncResult {
    if let Some(distributed_backend) = backend {
        match distributed_backend.unban_ip(site_id, ip) {
            Ok(()) => return BanSyncResult::Synced,
            Err(err) => eprintln!(
                "[providers][ban] external distributed unban failed for site={} ip={} ({}); applying outage posture",
                site_id, ip, err
            ),
        }
    }
    match outage_mode {
        crate::config::BanStoreOutageMode::FallbackInternal => {
            fallback();
            BanSyncResult::Deferred
        }
        crate::config::BanStoreOutageMode::FailOpen
        | crate::config::BanStoreOutageMode::FailClosed => BanSyncResult::Failed,
    }
}

impl BanStoreProvider for ExternalBanStoreProvider {
    fn is_banned(&self, store: &Store, site_id: &str, ip: &str) -> BanLookupResult {
        let outage_mode = crate::config::ban_store_outage_mode();
        let distributed_backend = RedisDistributedBanStore::from_env();
        is_banned_with_backend(
            distributed_backend.as_ref(),
            outage_mode,
            site_id,
            ip,
            || {
                matches!(
                    internal::BAN_STORE.is_banned(store, site_id, ip),
                    BanLookupResult::Banned
                )
            },
        )
    }

    fn list_active_bans(&self, store: &Store, site_id: &str) -> BanListResult {
        list_active_bans_with_runtime_contract(store, site_id)
    }

    fn ban_ip_with_fingerprint(
        &self,
        store: &Store,
        site_id: &str,
        ip: &str,
        reason: &str,
        duration_secs: u64,
        fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
    ) -> BanSyncResult {
        let outage_mode = crate::config::ban_store_outage_mode();
        let distributed_backend = RedisDistributedBanStore::from_env();
        ban_with_backend(
            distributed_backend.as_ref(),
            outage_mode,
            site_id,
            ip,
            reason,
            duration_secs,
            fingerprint.clone(),
            || {
                internal::BAN_STORE.ban_ip_with_fingerprint(
                    store,
                    site_id,
                    ip,
                    reason,
                    duration_secs,
                    fingerprint,
                );
            },
        )
    }

    fn unban_ip(&self, store: &Store, site_id: &str, ip: &str) -> BanSyncResult {
        let outage_mode = crate::config::ban_store_outage_mode();
        let distributed_backend = RedisDistributedBanStore::from_env();
        unban_with_backend(
            distributed_backend.as_ref(),
            outage_mode,
            site_id,
            ip,
            || {
                internal::BAN_STORE.unban_ip(store, site_id, ip);
            },
        )
    }
}

pub(crate) fn list_active_bans_with_runtime_contract<S>(store: &S, site_id: &str) -> BanListResult
where
    S: crate::challenge::KeyValueStore,
{
    let outage_mode = crate::config::ban_store_outage_mode();
    let distributed_backend = RedisDistributedBanStore::from_env();
    let fallback = || crate::enforcement::ban::list_active_bans(store, site_id);
    list_active_bans_with_backend(distributed_backend.as_ref(), outage_mode, site_id, fallback)
}

impl ChallengeEngineProvider for UnsupportedExternalChallengeEngineProvider {
    fn puzzle_path(&self) -> &'static str {
        internal::CHALLENGE_ENGINE.puzzle_path()
    }

    fn not_a_bot_path(&self) -> &'static str {
        internal::CHALLENGE_ENGINE.not_a_bot_path()
    }

    fn render_challenge(
        &self,
        req: &Request,
        transform_count: usize,
        seed_ttl_seconds: u64,
    ) -> Response {
        internal::CHALLENGE_ENGINE.render_challenge(req, transform_count, seed_ttl_seconds)
    }

    fn render_not_a_bot(&self, req: &Request, cfg: &crate::config::Config) -> Response {
        internal::CHALLENGE_ENGINE.render_not_a_bot(req, cfg)
    }

    fn serve_challenge_page(
        &self,
        req: &Request,
        shadow_mode: bool,
        transform_count: usize,
        seed_ttl_seconds: u64,
    ) -> Response {
        internal::CHALLENGE_ENGINE.serve_challenge_page(
            req,
            shadow_mode,
            transform_count,
            seed_ttl_seconds,
        )
    }

    fn serve_not_a_bot_page(
        &self,
        req: &Request,
        shadow_mode: bool,
        cfg: &crate::config::Config,
    ) -> Response {
        internal::CHALLENGE_ENGINE.serve_not_a_bot_page(req, shadow_mode, cfg)
    }

    fn handle_challenge_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
        challenge_puzzle_attempt_window_seconds: u64,
        challenge_puzzle_attempt_limit_per_window: u32,
    ) -> (Response, crate::challenge::ChallengeSubmitOutcome) {
        internal::CHALLENGE_ENGINE.handle_challenge_submit_with_outcome(
            store,
            req,
            challenge_puzzle_attempt_window_seconds,
            challenge_puzzle_attempt_limit_per_window,
        )
    }

    fn handle_not_a_bot_submit_with_outcome(
        &self,
        store: &Store,
        req: &Request,
        cfg: &crate::config::Config,
    ) -> crate::challenge::NotABotSubmitResult {
        internal::CHALLENGE_ENGINE.handle_not_a_bot_submit_with_outcome(store, req, cfg)
    }

    fn handle_pow_challenge(
        &self,
        ip: &str,
        user_agent: &str,
        enabled: bool,
        difficulty: u8,
        ttl_seconds: u64,
    ) -> Response {
        internal::CHALLENGE_ENGINE.handle_pow_challenge(
            ip,
            user_agent,
            enabled,
            difficulty,
            ttl_seconds,
        )
    }

    fn handle_pow_verify(&self, req: &Request, ip: &str, enabled: bool) -> Response {
        internal::CHALLENGE_ENGINE.handle_pow_verify(req, ip, enabled)
    }
}

impl MazeTarpitProvider for UnsupportedExternalMazeTarpitProvider {
    fn is_maze_path(&self, path: &str) -> bool {
        internal::MAZE_TARPIT.is_maze_path(path)
    }

    fn tarpit_progress_path(&self) -> &'static str {
        internal::MAZE_TARPIT.tarpit_progress_path()
    }

    fn serve_maze_with_tracking(
        &self,
        req: &Request,
        store: &Store,
        cfg: &crate::config::Config,
        ip: &str,
        user_agent: &str,
        path: &str,
        event_reason: &str,
        event_outcome: &str,
        botness_hint: Option<u8>,
    ) -> Response {
        internal::MAZE_TARPIT.serve_maze_with_tracking(
            req,
            store,
            cfg,
            ip,
            user_agent,
            path,
            event_reason,
            event_outcome,
            botness_hint,
        )
    }

    fn maybe_handle_tarpit(
        &self,
        req: &Request,
        store: &Store,
        cfg: &crate::config::Config,
        site_id: &str,
        ip: &str,
    ) -> Option<Response> {
        internal::MAZE_TARPIT.maybe_handle_tarpit(req, store, cfg, site_id, ip)
    }

    fn handle_tarpit_progress(
        &self,
        req: &Request,
        store: &Store,
        cfg: &crate::config::Config,
        site_id: &str,
        ip: &str,
        user_agent: &str,
    ) -> Response {
        internal::MAZE_TARPIT.handle_tarpit_progress(req, store, cfg, site_id, ip, user_agent)
    }
}

impl FingerprintSignalProvider for ExternalFingerprintSignalProvider {
    fn report_path(&self) -> &'static str {
        "/fingerprint-report"
    }

    fn source_availability(
        &self,
        cfg: &crate::config::Config,
    ) -> crate::signals::botness::SignalAvailability {
        if cfg.cdp_detection_enabled {
            crate::signals::botness::SignalAvailability::Active
        } else {
            crate::signals::botness::SignalAvailability::Disabled
        }
    }

    fn handle_report(&self, store: &Store, req: &Request) -> Response {
        let cfg = match crate::config::load_runtime_cached(store, "default") {
            Ok(cfg) => cfg,
            Err(_) => return Response::new(500, "Configuration unavailable"),
        };
        if !cfg.cdp_detection_enabled {
            return Response::new(200, "External fingerprint detection disabled");
        }

        if let Err(err) = crate::request_validation::enforce_body_size(
            req.body(),
            crate::request_validation::MAX_CDP_REPORT_BYTES,
        ) {
            return Response::new(400, err);
        }

        let parsed = match serde_json::from_slice::<AkamaiEdgeOutcome>(req.body()) {
            Ok(outcome) => outcome,
            Err(_) => {
                return internal::FINGERPRINT_SIGNAL.handle_report(store, req);
            }
        };
        if !looks_like_akamai_payload(&parsed) {
            return internal::FINGERPRINT_SIGNAL.handle_report(store, req);
        }
        if !crate::forwarded_ip_trusted(req) {
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::AdminAction,
                    ip: Some(crate::extract_client_ip(req)),
                    reason: Some("external_fingerprint_rejected_untrusted_source".to_string()),
                    outcome: Some("forbidden".to_string()),
                    admin: None,
                },
            );
            return Response::new(
                403,
                "External fingerprint report rejected (untrusted source)",
            );
        }
        if cfg.edge_integration_mode == crate::config::EdgeIntegrationMode::Off {
            return Response::new(200, "External fingerprint report ignored (edge mode off)");
        }

        let normalized = match normalize_akamai_edge_outcome(parsed) {
            Ok(outcome) => outcome,
            Err(err) => {
                let ip = crate::extract_client_ip(req);
                crate::signals::fingerprint::record_external_payload_rejection(store);
                crate::admin::log_event(
                    store,
                    &crate::admin::EventLogEntry {
                        ts: crate::admin::now_ts(),
                        event: crate::admin::EventType::Challenge,
                        ip: Some(ip),
                        reason: Some("external_fingerprint_invalid_payload".to_string()),
                        outcome: Some(err.to_string()),
                        admin: None,
                    },
                );
                return Response::new(400, err);
            }
        };
        let cdp_report = map_normalized_fingerprint_to_cdp_report(&normalized);
        let cdp_tier =
            crate::signals::cdp::classify_cdp_tier(&cdp_report, cfg.cdp_detection_threshold);
        let tier_label = cdp_tier_label(cdp_tier);
        let ip = crate::extract_client_ip(req);
        if cfg.edge_integration_mode == crate::config::EdgeIntegrationMode::Additive {
            crate::signals::fingerprint::record_akamai_edge_signal(
                store,
                &cfg,
                ip.as_str(),
                normalized.confidence.round().clamp(0.0, 10.0) as u8,
                normalized.hard_signal,
            );
        }
        let detection_policy_match = if cdp_tier == crate::signals::cdp::CdpTier::Strong {
            crate::runtime::policy_taxonomy::resolve_policy_match(
                crate::runtime::policy_taxonomy::PolicyTransition::EdgeFingerprintStrong,
            )
        } else {
            crate::runtime::policy_taxonomy::resolve_policy_match(
                crate::runtime::policy_taxonomy::PolicyTransition::EdgeFingerprintAdditive,
            )
        };
        crate::observability::metrics::record_policy_match(store, &detection_policy_match);

        crate::admin::log_event(
            store,
            &crate::admin::EventLogEntry {
                ts: crate::admin::now_ts(),
                event: crate::admin::EventType::Challenge,
                ip: Some(ip.clone()),
                reason: Some(format!(
                    "external_fingerprint_detected:tier={} score={:.2}",
                    tier_label, cdp_report.score
                )),
                outcome: Some(detection_policy_match.annotate_outcome(normalized.summary.as_str())),
                admin: None,
            },
        );
        crate::observability::metrics::increment(
            store,
            crate::observability::metrics::MetricName::CdpDetections,
            None,
        );

        if fingerprint_authoritative_mode_enabled(cfg.edge_integration_mode)
            && cfg.cdp_auto_ban
            && cdp_tier == crate::signals::cdp::CdpTier::Strong
        {
            let ban_policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                crate::runtime::policy_taxonomy::PolicyTransition::EdgeFingerprintAuthoritativeBan,
            );
            crate::observability::metrics::record_policy_match(store, &ban_policy_match);
            let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
            provider_registry
                .ban_store_provider()
                .ban_ip_with_fingerprint(
                    store,
                    "default",
                    &ip,
                    "edge_fingerprint_automation",
                    cfg.get_ban_duration("cdp"),
                    Some(crate::enforcement::ban::BanFingerprint {
                        score: Some((cdp_report.score * 2.0).round().clamp(0.0, 10.0) as u8),
                        signals: vec!["edge_fingerprint".to_string()],
                        summary: Some(normalized.summary.clone()),
                    }),
                );
            crate::observability::metrics::increment(
                store,
                crate::observability::metrics::MetricName::BansTotal,
                Some("cdp_automation"),
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event: crate::admin::EventType::Ban,
                    ip: Some(ip),
                    reason: Some("edge_fingerprint_automation".to_string()),
                    outcome: Some(
                        ban_policy_match.annotate_outcome(
                            format!("banned:tier={} score={:.2}", tier_label, cdp_report.score)
                                .as_str(),
                        ),
                    ),
                    admin: None,
                },
            );
            return Response::new(200, "External fingerprint automation detected - banned");
        }

        if cfg.edge_integration_mode == crate::config::EdgeIntegrationMode::Additive {
            return Response::new(200, "External fingerprint report received (additive)");
        }

        Response::new(200, "External fingerprint report received")
    }

    fn detection_script(&self) -> &'static str {
        ""
    }

    fn report_script(&self, _report_endpoint: &str) -> String {
        String::new()
    }

    fn inject_detection(&self, html: &str, _report_endpoint: Option<&str>) -> String {
        html.to_string()
    }
}

impl VerifiedIdentityProvider for ExternalVerifiedIdentityProvider {
    fn verify_identity(
        &self,
        _store: &dyn crate::challenge::KeyValueStore,
        _site_id: &str,
        req: &Request,
        cfg: &crate::config::Config,
    ) -> crate::bot_identity::verification::IdentityVerificationResult {
        if !cfg.verified_identity.enabled {
            return crate::bot_identity::verification::IdentityVerificationResult::disabled();
        }
        if !cfg.verified_identity.provider_assertions_enabled {
            return crate::bot_identity::verification::IdentityVerificationResult::not_attempted();
        }
        if cfg.edge_integration_mode == crate::config::EdgeIntegrationMode::Off {
            return crate::bot_identity::verification::IdentityVerificationResult::not_attempted();
        }

        let headers = extract_verified_identity_headers(req);
        if !headers.any_present() {
            return crate::bot_identity::verification::IdentityVerificationResult::not_attempted();
        }
        if !crate::forwarded_ip_trusted(req) {
            return crate::bot_identity::verification::IdentityVerificationResult::failed(
                crate::bot_identity::verification::IdentityVerificationFailure::ProviderRejected,
                crate::bot_identity::verification::IdentityVerificationFreshness::NotApplicable,
            );
        }

        match normalize_verified_identity_headers(headers) {
            Ok(identity) => {
                crate::bot_identity::verification::IdentityVerificationResult::verified(
                    identity,
                    crate::bot_identity::verification::IdentityVerificationFreshness::NotApplicable,
                )
            }
            Err(failure) => crate::bot_identity::verification::IdentityVerificationResult::failed(
                failure,
                crate::bot_identity::verification::IdentityVerificationFreshness::NotApplicable,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ban_with_backend, check_rate_limit_with_backend, current_rate_usage_with_backend,
        decide_rate_limit_on_outage, extract_verified_identity_headers,
        fingerprint_authoritative_mode_enabled, is_banned_with_backend,
        list_active_bans_with_backend, map_normalized_fingerprint_to_cdp_report,
        normalize_akamai_edge_outcome, normalize_verified_identity_headers, rate_drift_band,
        rate_route_class, unban_with_backend, AkamaiEdgeOutcome, DistributedBanStore,
        DistributedRateCounter, RateLimiterOutageAction, RATE_DRIFT_BAND_DELTA_0,
        RATE_DRIFT_BAND_DELTA_1_5, RATE_DRIFT_BAND_DELTA_21_PLUS, RATE_DRIFT_BAND_DELTA_6_20,
        RATE_ROUTE_CLASS_ADMIN_AUTH, RATE_ROUTE_CLASS_MAIN_TRAFFIC, VERIFIED_IDENTITY,
        VERIFIED_IDENTITY_CATEGORY_HEADER, VERIFIED_IDENTITY_DIRECTORY_SOURCE_ID_HEADER,
        VERIFIED_IDENTITY_DIRECTORY_SOURCE_URI_HEADER,
        VERIFIED_IDENTITY_END_USER_CONTROLLED_HEADER, VERIFIED_IDENTITY_OPERATOR_HEADER,
        VERIFIED_IDENTITY_SCHEME_HEADER, VERIFIED_IDENTITY_VALUE_HEADER,
    };
    use crate::providers::contracts::{
        BanListResult, BanLookupResult, BanSyncResult, RateLimitDecision, VerifiedIdentityProvider,
    };
    use std::cell::Cell;

    #[test]
    fn fingerprint_authoritative_mode_only_enabled_for_authoritative_setting() {
        assert!(!fingerprint_authoritative_mode_enabled(
            crate::config::EdgeIntegrationMode::Off
        ));
        assert!(!fingerprint_authoritative_mode_enabled(
            crate::config::EdgeIntegrationMode::Additive
        ));
        assert!(fingerprint_authoritative_mode_enabled(
            crate::config::EdgeIntegrationMode::Authoritative
        ));
    }

    #[test]
    fn normalize_akamai_edge_outcome_marks_deny_as_hard_signal() {
        let outcome = AkamaiEdgeOutcome {
            bot_score: Some(92.0),
            action: Some("deny".to_string()),
            detection_ids: vec!["bm_automation".to_string()],
            tags: vec!["ja3_mismatch".to_string()],
        };

        let normalized = normalize_akamai_edge_outcome(outcome).expect("valid outcome");
        assert!(normalized.hard_signal);
        assert!(normalized.confidence >= 9.0);
        assert!(normalized.checks.contains(&"automation_props".to_string()));
        assert!(normalized
            .checks
            .contains(&"akamai_action:deny".to_string()));
    }

    #[test]
    fn normalize_akamai_edge_outcome_rejects_out_of_range_scores() {
        let outcome = AkamaiEdgeOutcome {
            bot_score: Some(101.0),
            action: Some("allow".to_string()),
            detection_ids: vec![],
            tags: vec![],
        };

        assert!(normalize_akamai_edge_outcome(outcome).is_err());
    }

    #[test]
    fn normalized_akamai_signal_maps_into_cdp_report_contract() {
        let outcome = AkamaiEdgeOutcome {
            bot_score: Some(78.0),
            action: Some("challenge".to_string()),
            detection_ids: vec!["bm_signal".to_string()],
            tags: vec!["ua_mismatch".to_string()],
        };

        let normalized = normalize_akamai_edge_outcome(outcome).expect("valid outcome");
        let report = map_normalized_fingerprint_to_cdp_report(&normalized);

        assert!(report.cdp_detected);
        assert!(report.score >= 1.0);
        assert!(!report.checks.is_empty());
    }

    #[test]
    fn verified_identity_headers_normalize_provider_verified_bot_assertion() {
        let headers =
            extract_verified_identity_headers(&crate::test_support::request_with_headers(
                "/",
                &[
                    (VERIFIED_IDENTITY_SCHEME_HEADER, "provider_verified_bot"),
                    (VERIFIED_IDENTITY_VALUE_HEADER, "search.example"),
                    (VERIFIED_IDENTITY_OPERATOR_HEADER, "example"),
                    (VERIFIED_IDENTITY_CATEGORY_HEADER, "search"),
                ],
            ));

        let identity = normalize_verified_identity_headers(headers).expect("normalized identity");

        assert_eq!(
            identity.scheme,
            crate::bot_identity::contracts::IdentityScheme::ProviderVerifiedBot
        );
        assert_eq!(
            identity.verification_strength,
            crate::bot_identity::contracts::VerificationStrength::ProviderAsserted
        );
        assert_eq!(
            identity.provenance,
            crate::bot_identity::contracts::IdentityProvenance::Provider
        );
        assert!(!identity.end_user_controlled);
    }

    #[test]
    fn verified_identity_headers_normalize_signed_agent_assertion() {
        let headers =
            extract_verified_identity_headers(&crate::test_support::request_with_headers(
                "/",
                &[
                    (VERIFIED_IDENTITY_SCHEME_HEADER, "provider_signed_agent"),
                    (VERIFIED_IDENTITY_VALUE_HEADER, "chatgpt-agent"),
                    (VERIFIED_IDENTITY_OPERATOR_HEADER, "openai"),
                    (VERIFIED_IDENTITY_CATEGORY_HEADER, "user_triggered_agent"),
                    (VERIFIED_IDENTITY_END_USER_CONTROLLED_HEADER, "true"),
                    (
                        VERIFIED_IDENTITY_DIRECTORY_SOURCE_ID_HEADER,
                        "openai-http-message-signatures-directory",
                    ),
                    (
                        VERIFIED_IDENTITY_DIRECTORY_SOURCE_URI_HEADER,
                        "https://chatgpt.com/.well-known/http-message-signatures-directory",
                    ),
                ],
            ));

        let identity = normalize_verified_identity_headers(headers).expect("normalized identity");
        let directory_source = identity.directory_source.expect("directory source");

        assert_eq!(
            identity.scheme,
            crate::bot_identity::contracts::IdentityScheme::ProviderSignedAgent
        );
        assert!(identity.end_user_controlled);
        assert_eq!(
            directory_source.source_id,
            "openai-http-message-signatures-directory"
        );
        assert_eq!(
            directory_source.source_uri.as_deref(),
            Some("https://chatgpt.com/.well-known/http-message-signatures-directory")
        );
    }

    #[test]
    fn verified_identity_headers_reject_missing_required_fields() {
        let headers =
            extract_verified_identity_headers(&crate::test_support::request_with_headers(
                "/",
                &[
                    (VERIFIED_IDENTITY_SCHEME_HEADER, "provider_verified_bot"),
                    (VERIFIED_IDENTITY_VALUE_HEADER, "search.example"),
                    (VERIFIED_IDENTITY_OPERATOR_HEADER, "example"),
                ],
            ));

        let failure = normalize_verified_identity_headers(headers).expect_err("missing category");

        assert_eq!(
            failure,
            crate::bot_identity::verification::IdentityVerificationFailure::MissingAssertion
        );
    }

    #[test]
    fn verified_identity_provider_returns_disabled_when_provider_path_is_off() {
        let cfg = crate::config::defaults().clone();
        let req = crate::test_support::request_with_headers("/", &[]);
        let store = crate::test_support::InMemoryStore::default();

        let result = VERIFIED_IDENTITY.verify_identity(&store, "default", &req, &cfg);

        assert_eq!(
            result.status,
            crate::bot_identity::verification::IdentityVerificationResultStatus::Disabled
        );
    }

    #[test]
    fn verified_identity_provider_verifies_trusted_edge_assertions() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");

        let mut cfg = crate::config::defaults().clone();
        cfg.verified_identity.enabled = true;
        cfg.edge_integration_mode = crate::config::EdgeIntegrationMode::Additive;
        let store = crate::test_support::InMemoryStore::default();
        let req = crate::test_support::request_with_headers(
            "/",
            &[
                ("x-shuma-forwarded-secret", "test-forwarded-secret"),
                (VERIFIED_IDENTITY_SCHEME_HEADER, "provider_signed_agent"),
                (VERIFIED_IDENTITY_VALUE_HEADER, "chatgpt-agent"),
                (VERIFIED_IDENTITY_OPERATOR_HEADER, "openai"),
                (VERIFIED_IDENTITY_CATEGORY_HEADER, "user_triggered_agent"),
                (VERIFIED_IDENTITY_END_USER_CONTROLLED_HEADER, "true"),
            ],
        );

        let result = VERIFIED_IDENTITY.verify_identity(&store, "default", &req, &cfg);
        let identity = result.identity.expect("verified identity");

        assert_eq!(
            result.status,
            crate::bot_identity::verification::IdentityVerificationResultStatus::Verified
        );
        assert_eq!(
            identity.scheme,
            crate::bot_identity::contracts::IdentityScheme::ProviderSignedAgent
        );
        assert!(identity.end_user_controlled);
    }

    #[test]
    fn verified_identity_provider_rejects_untrusted_assertion_headers() {
        let mut cfg = crate::config::defaults().clone();
        cfg.verified_identity.enabled = true;
        cfg.edge_integration_mode = crate::config::EdgeIntegrationMode::Additive;
        let store = crate::test_support::InMemoryStore::default();
        let req = crate::test_support::request_with_headers(
            "/",
            &[
                (VERIFIED_IDENTITY_SCHEME_HEADER, "provider_verified_bot"),
                (VERIFIED_IDENTITY_VALUE_HEADER, "search.example"),
                (VERIFIED_IDENTITY_OPERATOR_HEADER, "example"),
                (VERIFIED_IDENTITY_CATEGORY_HEADER, "search"),
            ],
        );

        let result = VERIFIED_IDENTITY.verify_identity(&store, "default", &req, &cfg);

        assert_eq!(
            result.status,
            crate::bot_identity::verification::IdentityVerificationResultStatus::Failed
        );
        assert_eq!(
            result.failure,
            Some(crate::bot_identity::verification::IdentityVerificationFailure::ProviderRejected)
        );
    }

    #[derive(Clone)]
    struct MockDistributedRateCounter {
        current_result: Result<u32, String>,
        increment_result: Result<u32, String>,
        current_calls: Cell<u32>,
        increment_calls: Cell<u32>,
    }

    impl MockDistributedRateCounter {
        fn with_results(
            current_result: Result<u32, String>,
            increment_result: Result<u32, String>,
        ) -> Self {
            Self {
                current_result,
                increment_result,
                current_calls: Cell::new(0),
                increment_calls: Cell::new(0),
            }
        }
    }

    impl DistributedRateCounter for MockDistributedRateCounter {
        fn current_usage(&self, _key: &str) -> Result<u32, String> {
            self.current_calls.set(self.current_calls.get() + 1);
            self.current_result.clone()
        }

        fn increment_and_get(&self, _key: &str, _ttl_seconds: u64) -> Result<u32, String> {
            self.increment_calls.set(self.increment_calls.get() + 1);
            self.increment_result.clone()
        }
    }

    #[derive(Clone)]
    struct MockDistributedBanStore {
        is_banned_result: Result<bool, String>,
        list_result: Result<Vec<(String, crate::enforcement::ban::BanEntry)>, String>,
        ban_result: Result<(), String>,
        unban_result: Result<(), String>,
        is_banned_calls: Cell<u32>,
        list_calls: Cell<u32>,
        ban_calls: Cell<u32>,
        unban_calls: Cell<u32>,
    }

    impl MockDistributedBanStore {
        fn with_results(
            is_banned_result: Result<bool, String>,
            list_result: Result<Vec<(String, crate::enforcement::ban::BanEntry)>, String>,
            ban_result: Result<(), String>,
            unban_result: Result<(), String>,
        ) -> Self {
            Self {
                is_banned_result,
                list_result,
                ban_result,
                unban_result,
                is_banned_calls: Cell::new(0),
                list_calls: Cell::new(0),
                ban_calls: Cell::new(0),
                unban_calls: Cell::new(0),
            }
        }
    }

    impl DistributedBanStore for MockDistributedBanStore {
        fn is_banned(&self, _site_id: &str, _ip: &str) -> Result<bool, String> {
            self.is_banned_calls.set(self.is_banned_calls.get() + 1);
            self.is_banned_result.clone()
        }

        fn list_active_bans(
            &self,
            _site_id: &str,
        ) -> Result<Vec<(String, crate::enforcement::ban::BanEntry)>, String> {
            self.list_calls.set(self.list_calls.get() + 1);
            self.list_result.clone()
        }

        fn ban_ip_with_fingerprint(
            &self,
            _site_id: &str,
            _ip: &str,
            _reason: &str,
            _duration_secs: u64,
            _fingerprint: Option<crate::enforcement::ban::BanFingerprint>,
        ) -> Result<(), String> {
            self.ban_calls.set(self.ban_calls.get() + 1);
            self.ban_result.clone()
        }

        fn unban_ip(&self, _site_id: &str, _ip: &str) -> Result<(), String> {
            self.unban_calls.set(self.unban_calls.get() + 1);
            self.unban_result.clone()
        }
    }

    #[test]
    fn distributed_rate_usage_prefers_backend_when_available() {
        let backend = MockDistributedRateCounter::with_results(Ok(7), Ok(0));
        let fallback_called = Cell::new(false);
        let usage = current_rate_usage_with_backend(Some(&backend), "default", "1.2.3.4", || {
            fallback_called.set(true);
            3
        });
        assert_eq!(usage, 7);
        assert!(!fallback_called.get());
        assert_eq!(backend.current_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_usage_falls_back_when_backend_errors() {
        let backend =
            MockDistributedRateCounter::with_results(Err("backend unavailable".to_string()), Ok(0));
        let fallback_called = Cell::new(false);
        let usage = current_rate_usage_with_backend(Some(&backend), "default", "1.2.3.4", || {
            fallback_called.set(true);
            5
        });
        assert_eq!(usage, 5);
        assert!(fallback_called.get());
        assert_eq!(backend.current_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_limit_prefers_backend_when_available() {
        let backend = MockDistributedRateCounter::with_results(Ok(0), Ok(3));
        let fallback_called = Cell::new(false);
        let decision =
            check_rate_limit_with_backend(Some(&backend), "default", "1.2.3.4", 3, || {
                fallback_called.set(true);
                RateLimitDecision::Limited
            });
        assert_eq!(decision, RateLimitDecision::Allowed);
        assert!(!fallback_called.get());
        assert_eq!(backend.increment_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_limit_blocks_when_backend_counter_exceeds_limit() {
        let backend = MockDistributedRateCounter::with_results(Ok(0), Ok(4));
        let decision =
            check_rate_limit_with_backend(Some(&backend), "default", "1.2.3.4", 3, || {
                RateLimitDecision::Allowed
            });
        assert_eq!(decision, RateLimitDecision::Limited);
        assert_eq!(backend.increment_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_limit_falls_back_on_backend_error() {
        let backend =
            MockDistributedRateCounter::with_results(Ok(0), Err("backend unavailable".to_string()));
        let fallback_called = Cell::new(false);
        let decision =
            check_rate_limit_with_backend(Some(&backend), "default", "1.2.3.4", 3, || {
                fallback_called.set(true);
                RateLimitDecision::Allowed
            });
        assert_eq!(decision, RateLimitDecision::Allowed);
        assert!(fallback_called.get());
        assert_eq!(backend.increment_calls.get(), 1);
    }

    #[test]
    fn distributed_rate_limit_zero_limit_blocks_without_backend_or_fallback() {
        let backend = MockDistributedRateCounter::with_results(Ok(0), Ok(1));
        let fallback_called = Cell::new(false);
        let decision =
            check_rate_limit_with_backend(Some(&backend), "default", "1.2.3.4", 0, || {
                fallback_called.set(true);
                RateLimitDecision::Allowed
            });
        assert_eq!(decision, RateLimitDecision::Limited);
        assert!(!fallback_called.get());
        assert_eq!(backend.increment_calls.get(), 0);
    }

    #[test]
    fn outage_decision_uses_fallback_internal_mode() {
        let fallback_called = Cell::new(false);
        let (decision, action) = decide_rate_limit_on_outage(
            crate::config::RateLimiterOutageMode::FallbackInternal,
            || {
                fallback_called.set(true);
                RateLimitDecision::Limited
            },
        );
        assert_eq!(decision, RateLimitDecision::Limited);
        assert_eq!(action, RateLimiterOutageAction::FallbackInternal);
        assert!(fallback_called.get());
    }

    #[test]
    fn outage_decision_uses_fail_open_mode() {
        let fallback_called = Cell::new(false);
        let (decision, action) =
            decide_rate_limit_on_outage(crate::config::RateLimiterOutageMode::FailOpen, || {
                fallback_called.set(true);
                RateLimitDecision::Limited
            });
        assert_eq!(decision, RateLimitDecision::Allowed);
        assert_eq!(action, RateLimiterOutageAction::Allow);
        assert!(!fallback_called.get());
    }

    #[test]
    fn outage_decision_uses_fail_closed_mode() {
        let fallback_called = Cell::new(false);
        let (decision, action) =
            decide_rate_limit_on_outage(crate::config::RateLimiterOutageMode::FailClosed, || {
                fallback_called.set(true);
                RateLimitDecision::Allowed
            });
        assert_eq!(decision, RateLimitDecision::Limited);
        assert_eq!(action, RateLimiterOutageAction::Deny);
        assert!(!fallback_called.get());
    }

    #[test]
    fn rate_route_class_maps_admin_and_main_sites() {
        assert_eq!(rate_route_class("default"), RATE_ROUTE_CLASS_MAIN_TRAFFIC);
        assert_eq!(
            rate_route_class("admin-auth-login"),
            RATE_ROUTE_CLASS_ADMIN_AUTH
        );
        assert_eq!(
            rate_route_class("admin-auth-endpoint"),
            RATE_ROUTE_CLASS_ADMIN_AUTH
        );
    }

    #[test]
    fn rate_drift_band_groups_expected_ranges() {
        assert_eq!(rate_drift_band(0), RATE_DRIFT_BAND_DELTA_0);
        assert_eq!(rate_drift_band(1), RATE_DRIFT_BAND_DELTA_1_5);
        assert_eq!(rate_drift_band(5), RATE_DRIFT_BAND_DELTA_1_5);
        assert_eq!(rate_drift_band(6), RATE_DRIFT_BAND_DELTA_6_20);
        assert_eq!(rate_drift_band(20), RATE_DRIFT_BAND_DELTA_6_20);
        assert_eq!(rate_drift_band(21), RATE_DRIFT_BAND_DELTA_21_PLUS);
    }

    #[test]
    fn distributed_ban_lookup_prefers_backend_when_available() {
        let backend =
            MockDistributedBanStore::with_results(Ok(true), Ok(Vec::new()), Ok(()), Ok(()));
        let fallback_called = Cell::new(false);
        let banned = is_banned_with_backend(
            Some(&backend),
            crate::config::BanStoreOutageMode::FallbackInternal,
            "default",
            "1.2.3.4",
            || {
                fallback_called.set(true);
                false
            },
        );
        assert_eq!(banned, BanLookupResult::Banned);
        assert!(!fallback_called.get());
        assert_eq!(backend.is_banned_calls.get(), 1);
    }

    #[test]
    fn distributed_ban_lookup_uses_fallback_internal_mode_when_backend_errors() {
        let backend = MockDistributedBanStore::with_results(
            Err("backend unavailable".to_string()),
            Ok(Vec::new()),
            Ok(()),
            Ok(()),
        );
        let fallback_called = Cell::new(false);
        let banned = is_banned_with_backend(
            Some(&backend),
            crate::config::BanStoreOutageMode::FallbackInternal,
            "default",
            "1.2.3.4",
            || {
                fallback_called.set(true);
                true
            },
        );
        assert_eq!(banned, BanLookupResult::Banned);
        assert!(fallback_called.get());
        assert_eq!(backend.is_banned_calls.get(), 1);
    }

    #[test]
    fn distributed_ban_lookup_returns_unavailable_without_fallback_in_strict_modes() {
        let fallback_called = Cell::new(false);
        let banned = is_banned_with_backend(
            None::<&MockDistributedBanStore>,
            crate::config::BanStoreOutageMode::FailClosed,
            "default",
            "1.2.3.4",
            || {
                fallback_called.set(true);
                true
            },
        );
        assert_eq!(banned, BanLookupResult::Unavailable);
        assert!(!fallback_called.get());
    }

    #[test]
    fn distributed_ban_listing_prefers_backend_when_available() {
        let entries = vec![(
            "1.2.3.4".to_string(),
            crate::enforcement::ban::BanEntry {
                reason: "test".to_string(),
                expires: 999_999,
                banned_at: 1,
                fingerprint: None,
            },
        )];
        let backend =
            MockDistributedBanStore::with_results(Ok(false), Ok(entries.clone()), Ok(()), Ok(()));
        let fallback_called = Cell::new(false);
        let bans = list_active_bans_with_backend(
            Some(&backend),
            crate::config::BanStoreOutageMode::FallbackInternal,
            "default",
            || {
                fallback_called.set(true);
                Vec::new()
            },
        );
        match bans {
            BanListResult::Available(bans) => {
                assert_eq!(bans.len(), 1);
                assert_eq!(bans[0].0, entries[0].0);
            }
            BanListResult::Unavailable => panic!("expected available bans"),
        }
        assert!(!fallback_called.get());
        assert_eq!(backend.list_calls.get(), 1);
    }

    #[test]
    fn distributed_ban_listing_returns_unavailable_without_fallback_in_strict_modes() {
        let backend = MockDistributedBanStore::with_results(
            Ok(false),
            Err("backend unavailable".to_string()),
            Ok(()),
            Ok(()),
        );
        let fallback_called = Cell::new(false);
        let bans = list_active_bans_with_backend(
            Some(&backend),
            crate::config::BanStoreOutageMode::FailOpen,
            "default",
            || {
                fallback_called.set(true);
                vec![(
                    "2.3.4.5".to_string(),
                    crate::enforcement::ban::BanEntry {
                        reason: "fallback".to_string(),
                        expires: 999_999,
                        banned_at: 1,
                        fingerprint: None,
                    },
                )]
            },
        );
        assert!(matches!(bans, BanListResult::Unavailable));
        assert!(!fallback_called.get());
        assert_eq!(backend.list_calls.get(), 1);
    }

    #[test]
    fn distributed_ban_write_prefers_backend_when_available() {
        let backend =
            MockDistributedBanStore::with_results(Ok(false), Ok(Vec::new()), Ok(()), Ok(()));
        let fallback_called = Cell::new(false);
        let outcome = ban_with_backend(
            Some(&backend),
            crate::config::BanStoreOutageMode::FallbackInternal,
            "default",
            "1.2.3.4",
            "test",
            60,
            None,
            || fallback_called.set(true),
        );
        assert_eq!(outcome, BanSyncResult::Synced);
        assert!(!fallback_called.get());
        assert_eq!(backend.ban_calls.get(), 1);
    }

    #[test]
    fn distributed_ban_write_returns_deferred_when_fallback_internal_is_used() {
        let backend = MockDistributedBanStore::with_results(
            Ok(false),
            Ok(Vec::new()),
            Err("backend unavailable".to_string()),
            Ok(()),
        );
        let fallback_called = Cell::new(false);
        let outcome = ban_with_backend(
            Some(&backend),
            crate::config::BanStoreOutageMode::FallbackInternal,
            "default",
            "1.2.3.4",
            "test",
            60,
            None,
            || fallback_called.set(true),
        );
        assert_eq!(outcome, BanSyncResult::Deferred);
        assert!(fallback_called.get());
        assert_eq!(backend.ban_calls.get(), 1);
    }

    #[test]
    fn distributed_ban_write_returns_failed_without_local_fallback_in_strict_modes() {
        let fallback_called = Cell::new(false);
        let outcome = ban_with_backend(
            None::<&MockDistributedBanStore>,
            crate::config::BanStoreOutageMode::FailClosed,
            "default",
            "1.2.3.4",
            "test",
            60,
            None,
            || fallback_called.set(true),
        );
        assert_eq!(outcome, BanSyncResult::Failed);
        assert!(!fallback_called.get());
    }

    #[test]
    fn distributed_unban_prefers_backend_when_available() {
        let backend =
            MockDistributedBanStore::with_results(Ok(false), Ok(Vec::new()), Ok(()), Ok(()));
        let fallback_called = Cell::new(false);
        let outcome = unban_with_backend(
            Some(&backend),
            crate::config::BanStoreOutageMode::FallbackInternal,
            "default",
            "1.2.3.4",
            || fallback_called.set(true),
        );
        assert_eq!(outcome, BanSyncResult::Synced);
        assert!(!fallback_called.get());
        assert_eq!(backend.unban_calls.get(), 1);
    }

    #[test]
    fn distributed_unban_returns_failed_without_local_fallback_in_strict_modes() {
        let backend = MockDistributedBanStore::with_results(
            Ok(false),
            Ok(Vec::new()),
            Ok(()),
            Err("backend unavailable".to_string()),
        );
        let fallback_called = Cell::new(false);
        let outcome = unban_with_backend(
            Some(&backend),
            crate::config::BanStoreOutageMode::FailOpen,
            "default",
            "1.2.3.4",
            || fallback_called.set(true),
        );
        assert_eq!(outcome, BanSyncResult::Failed);
        assert!(!fallback_called.get());
        assert_eq!(backend.unban_calls.get(), 1);
    }
}
