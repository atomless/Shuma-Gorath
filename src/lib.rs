#![recursion_limit = "256"]

#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod test_support;
// src/lib.rs
// Entry point for the WASM Stealth Bot Defence Spin app

use crate::enforcement::block_page;
use crate::signals::{geo, js_verification as js};
use serde::Serialize;
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::http_component;
use spin_sdk::key_value::Store;
use std::env;
use std::io::Write;

mod admin; // Admin API endpoints
mod boundaries; // Domain boundary adapters for future repo splits
mod challenge; // Interactive math challenge for banned users
mod config; // Config loading and defaults
mod crawler_policy; // Crawler-facing policy surfaces (robots.txt)
mod deception; // Shared deception primitives (maze+tarpit)
mod enforcement; // Enforcement actions (ban, block page, honeypot, rate limiting)
mod maze; // maze crawler trap
mod observability; // Metrics and monitoring surfaces
mod providers; // Provider contracts for swappable implementations
mod request_validation; // Request validation/parsing helpers
mod runtime; // request-time orchestration helpers
mod signals; // Risk and identity signals (browser/CDP/GEO/IP/JS/allowlist)
mod tarpit; // tarpit progressive endpoint/runtime

#[derive(Clone, Copy)]
pub(crate) struct LibCapabilityToken(());

impl LibCapabilityToken {
    fn new() -> Self {
        Self(())
    }
}

/// Main HTTP handler for the bot defence. This function is invoked for every HTTP request.
/// It applies a series of anti-bot checks in order of cost and effectiveness, returning early on block/allow.

/// Returns true if forwarded IP headers should be trusted for this request.
/// If SHUMA_FORWARDED_IP_SECRET is set, require a matching X-Shuma-Forwarded-Secret header.
pub(crate) fn forwarded_ip_trusted(req: &Request) -> bool {
    match env::var("SHUMA_FORWARDED_IP_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => req
            .header("x-shuma-forwarded-secret")
            .and_then(|v| v.as_str())
            .map(|v| v == secret)
            .unwrap_or(false),
        _ => false,
    }
}

fn forwarded_proto_is_https(req: &Request) -> bool {
    if !forwarded_ip_trusted(req) {
        return false;
    }

    if let Some(proto) = req.header("x-forwarded-proto").and_then(|v| v.as_str()) {
        let first = proto.split(',').next().unwrap_or("").trim();
        if first.eq_ignore_ascii_case("https") {
            return true;
        }
    }

    if let Some(forwarded) = req.header("forwarded").and_then(|v| v.as_str()) {
        for part in forwarded.split(',') {
            for segment in part.split(';') {
                let segment = segment.trim();
                let lower = segment.to_ascii_lowercase();
                let Some(value) = lower.strip_prefix("proto=") else {
                    continue;
                };
                let value = value.trim().trim_matches('"');
                if value.eq_ignore_ascii_case("https") {
                    return true;
                }
            }
        }
    }

    false
}

pub(crate) fn request_is_https(req: &Request) -> bool {
    if req.uri().trim_start().starts_with("https://") {
        return true;
    }
    forwarded_proto_is_https(req)
}

const STATIC_BYPASS_PREFIXES: [&str; 8] = [
    "/assets/",
    "/static/",
    "/images/",
    "/img/",
    "/js/",
    "/css/",
    "/fonts/",
    "/_next/static/",
];
const STATIC_BYPASS_EXACT_PATHS: [&str; 7] = [
    "/favicon.ico",
    "/favicon.svg",
    "/apple-touch-icon.png",
    "/manifest.json",
    "/site.webmanifest",
    "/sitemap.xml",
    "/browserconfig.xml",
];
const STATIC_BYPASS_EXTENSIONS: [&str; 18] = [
    "css",
    "js",
    "mjs",
    "map",
    "png",
    "jpg",
    "jpeg",
    "gif",
    "webp",
    "svg",
    "ico",
    "woff",
    "woff2",
    "ttf",
    "otf",
    "eot",
    "webmanifest",
    "xml",
];

fn has_static_bypass_extension(path: &str) -> bool {
    let leaf = path.rsplit('/').next().unwrap_or("");
    let Some((_, ext)) = leaf.rsplit_once('.') else {
        return false;
    };
    STATIC_BYPASS_EXTENSIONS
        .iter()
        .any(|candidate| ext.eq_ignore_ascii_case(candidate))
}

fn is_obvious_static_asset_path(path: &str) -> bool {
    if STATIC_BYPASS_EXACT_PATHS.contains(&path) {
        return true;
    }
    if STATIC_BYPASS_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
    {
        return true;
    }
    has_static_bypass_extension(path)
}

pub(crate) fn should_bypass_expensive_bot_checks_for_static(req: &Request, path: &str) -> bool {
    if !matches!(req.method(), Method::Get | Method::Head) {
        return false;
    }
    if matches!(
        path,
        "/health"
            | "/metrics"
            | "/robots.txt"
            | "/pow"
            | "/pow/verify"
            | "/tarpit/progress"
            | "/challenge/puzzle"
            | "/challenge/not-a-bot-checkbox"
    ) {
        return false;
    }
    if path.starts_with("/admin") {
        return false;
    }
    if path == "/sim/public" || path.starts_with("/sim/public/") {
        return false;
    }
    is_obvious_static_asset_path(path)
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Extract the best available client IP from the request.
pub(crate) fn extract_client_ip(req: &Request) -> String {
    // Prefer X-Forwarded-For (may be a comma-separated list) when trusted
    if forwarded_ip_trusted(req) {
        if let Some(h) = req.header("x-forwarded-for") {
            let val = h.as_str().unwrap_or("");
            // Take the first IP in the list
            if let Some(ip) = val.split(',').next() {
                let ip = ip.trim();
                if !ip.is_empty() && ip != "unknown" {
                    return ip.to_string();
                }
            }
        }
        // Fallback: X-Real-IP
        if let Some(h) = req.header("x-real-ip") {
            let val = h.as_str().unwrap_or("");
            if !val.is_empty() && val != "unknown" {
                return val.to_string();
            }
        }
    }
    // Fallback: remote_addr (Spin SDK may not expose this, but placeholder for future)
    // If available: req.remote_addr().unwrap_or("")

    // Last resort:
    "unknown".to_string()
}

/// Extract client IP for `/health` checks.
///
/// Security posture:
/// - Only trust forwarded headers when `forwarded_ip_trusted` is true.
/// - Reject multi-hop XFF chains for health checks to avoid accepting attacker-
///   supplied left-most values when an upstream proxy appends addresses.
fn extract_health_client_ip(req: &Request) -> String {
    if forwarded_ip_trusted(req) {
        if let Some(h) = req.header("x-forwarded-for") {
            let mut entries = h
                .as_str()
                .unwrap_or("")
                .split(',')
                .map(|ip| ip.trim())
                .filter(|ip| !ip.is_empty() && *ip != "unknown");

            if let Some(first) = entries.next() {
                if entries.next().is_some() {
                    return "unknown".to_string();
                }
                return first.to_string();
            }
        }

        if let Some(h) = req.header("x-real-ip") {
            let val = h.as_str().unwrap_or("").trim();
            if !val.is_empty() && val != "unknown" {
                return val.to_string();
            }
        }
    }

    "unknown".to_string()
}

fn health_secret_authorized(req: &Request) -> bool {
    let expected = match env::var("SHUMA_HEALTH_SECRET") {
        Ok(secret) => secret.trim().to_string(),
        Err(_) => return true,
    };
    if expected.is_empty() {
        return true;
    }

    let presented = req
        .header("x-shuma-health-secret")
        .and_then(|v| v.as_str())
        .map(|v| v.trim())
        .unwrap_or("");

    constant_time_eq(presented, expected.as_str())
}

/// Return true when KV outage policy is fail-open.
fn shuma_fail_open() -> bool {
    config::kv_store_fail_open()
}

fn fail_mode_label(fail_open: bool) -> &'static str {
    if fail_open {
        "open"
    } else {
        "closed"
    }
}

fn debug_headers_enabled() -> bool {
    env::var("SHUMA_DEBUG_HEADERS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn response_with_optional_debug_headers(
    status: u16,
    body: &str,
    kv_status: &str,
    fail_mode: &str,
) -> Response {
    let mut response_builder = Response::builder();
    let builder = response_builder.status(status);
    if debug_headers_enabled() {
        builder
            .header("X-KV-Status", kv_status)
            .header("X-Shuma-Fail-Mode", fail_mode)
            .body(body)
            .build()
    } else {
        builder.body(body).build()
    }
}

pub(crate) fn maze_response(served: crate::maze::runtime::MazeRenderResult) -> Response {
    let mut response_builder = Response::builder();
    response_builder
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-store, no-cache, must-revalidate")
        .header("X-Robots-Tag", "noindex, nofollow")
        .body(served.html)
        .build()
}

fn config_error_response(err: config::ConfigLoadError, path: &str) -> Response {
    log_line(&format!(
        "[CONFIG ERROR] path={} error={}",
        path,
        err.user_message()
    ));
    Response::new(500, "Configuration unavailable")
}

pub(crate) fn load_runtime_config(
    store: &Store,
    site_id: &str,
    path: &str,
) -> Result<config::Config, Response> {
    let cfg = config::load_runtime_cached(store, site_id)
        .map_err(|err| config_error_response(err, path))?;
    if let Some(guardrail_error) = cfg.enterprise_state_guardrail_error() {
        log_line(&format!(
            "[ENTERPRISE STATE ERROR] path={} {}",
            path, guardrail_error
        ));
        return Err(Response::new(503, "Server configuration error"));
    }
    Ok(cfg)
}

fn rate_proximity_score(rate_count: u32, rate_limit: u32) -> u8 {
    if rate_limit == 0 {
        return 0;
    }
    let ratio = rate_count as f32 / rate_limit as f32;
    if ratio >= 0.8 {
        2
    } else if ratio >= 0.5 {
        1
    } else {
        0
    }
}

#[allow(dead_code)]
pub(crate) fn compute_risk_score(
    js_needed: bool,
    geo_risk: bool,
    rate_count: u32,
    rate_limit: u32,
) -> u8 {
    let mut score = 0u8;
    if js_needed {
        score += 1;
    }
    if geo_risk {
        score += 2;
    }
    score += rate_proximity_score(rate_count, rate_limit);
    score
}

pub type BotnessContribution = crate::signals::botness::BotSignal;
const BROWSER_POLICY_SIGNAL_WEIGHT: u8 = 1;
const BROWSER_POLICY_SIGNAL_KEY: &str = "browser_outdated";
const BROWSER_POLICY_SIGNAL_LABEL: &str = "Browser policy minimum-version match";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BotnessAssessment {
    pub score: u8,
    pub contributions: Vec<BotnessContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotnessSignalContext {
    pub js_needed: bool,
    pub browser_outdated: bool,
    pub geo_signal_available: bool,
    pub geo_risk: bool,
    pub rate_count: u32,
    pub rate_limit: u32,
    pub maze_behavior_score: u8,
    pub fingerprint_signals: Vec<BotnessContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoAssessment {
    pub country: Option<String>,
    pub headers_trusted: bool,
    pub route: geo::GeoPolicyRoute,
    pub scored_risk: bool,
}

pub(crate) fn assess_geo_request(req: &Request, cfg: &config::Config) -> GeoAssessment {
    let headers_trusted = cfg.geo_edge_headers_enabled && forwarded_ip_trusted(req);
    let country = geo::extract_geo_country(req, headers_trusted);
    let route = geo::evaluate_geo_policy(country.as_deref(), cfg);
    let scored_risk = if route == geo::GeoPolicyRoute::Allow {
        false
    } else {
        country
            .as_deref()
            .map(|value| geo::country_in_list(value, &cfg.geo_risk))
            .unwrap_or(false)
    };
    GeoAssessment {
        country,
        headers_trusted,
        route,
        scored_risk,
    }
}

pub(crate) fn collect_botness_contributions(
    context: BotnessSignalContext,
    cfg: &config::Config,
) -> Vec<BotnessContribution> {
    let signal_capacity = 6 + context.fingerprint_signals.len();
    let mut accumulator = crate::signals::botness::SignalAccumulator::with_capacity_and_policy(
        signal_capacity,
        crate::signals::botness::SignalBudgetPolicy {
            fingerprint_total_cap: cfg.fingerprint_entropy_budget,
            fingerprint_header_runtime_cap: cfg.fingerprint_family_cap_header_runtime,
            fingerprint_transport_cap: cfg.fingerprint_family_cap_transport,
            fingerprint_temporal_cap: cfg.fingerprint_family_cap_temporal,
            fingerprint_persistence_cap: cfg.fingerprint_family_cap_persistence,
            fingerprint_behavior_cap: cfg.fingerprint_family_cap_behavior,
        },
    );

    accumulator.push(js::bot_signal(
        cfg.js_signal_enabled(),
        context.js_needed,
        cfg.botness_weights.js_required,
    ));

    let browser_signal = if cfg.browser_policy_enabled {
        crate::signals::botness::BotSignal::scored_with_metadata(
            BROWSER_POLICY_SIGNAL_KEY,
            BROWSER_POLICY_SIGNAL_LABEL,
            context.browser_outdated,
            BROWSER_POLICY_SIGNAL_WEIGHT,
            crate::signals::botness::SignalProvenance::Internal,
            10,
            crate::signals::botness::SignalFamily::RequestIntegrity,
        )
    } else {
        crate::signals::botness::BotSignal::disabled_with_metadata(
            BROWSER_POLICY_SIGNAL_KEY,
            BROWSER_POLICY_SIGNAL_LABEL,
            crate::signals::botness::SignalProvenance::Internal,
            10,
            crate::signals::botness::SignalFamily::RequestIntegrity,
        )
    };
    accumulator.push(browser_signal);

    let geo_signal = if cfg.geo_signal_enabled() {
        geo::bot_signal(
            context.geo_signal_available,
            context.geo_risk,
            cfg.botness_weights.geo_risk,
        )
    } else {
        geo::disabled_bot_signal()
    };
    accumulator.push(geo_signal);

    let rate_signals = if cfg.rate_signal_enabled() {
        crate::signals::rate_pressure::bot_signals(
            context.rate_count,
            context.rate_limit,
            cfg.botness_weights.rate_medium,
            cfg.botness_weights.rate_high,
        )
    } else {
        crate::signals::rate_pressure::disabled_bot_signals()
    };

    for rate_signal in rate_signals {
        accumulator.push(rate_signal);
    }

    let maze_behavior_signal = if cfg.maze_enabled {
        crate::signals::botness::BotSignal::scored_with_metadata(
            "maze_behavior",
            "Maze traversal behavior",
            context.maze_behavior_score >= 2,
            cfg.botness_weights.maze_behavior,
            crate::signals::botness::SignalProvenance::Derived,
            8,
            crate::signals::botness::SignalFamily::Deception,
        )
    } else {
        crate::signals::botness::BotSignal::disabled_with_metadata(
            "maze_behavior",
            "Maze traversal behavior",
            crate::signals::botness::SignalProvenance::Derived,
            8,
            crate::signals::botness::SignalFamily::Deception,
        )
    };
    accumulator.push(maze_behavior_signal);

    for fingerprint_signal in context.fingerprint_signals {
        accumulator.push(fingerprint_signal);
    }

    let (_score, contributions) = accumulator.finish();
    contributions
}

pub(crate) fn compute_botness_assessment_from_contributions(
    contributions: Vec<BotnessContribution>,
) -> BotnessAssessment {
    let mut accumulator =
        crate::signals::botness::SignalAccumulator::with_capacity(contributions.len());
    for contribution in contributions {
        accumulator.push(contribution);
    }
    let (score, contributions) = accumulator.finish();
    BotnessAssessment {
        score,
        contributions,
    }
}

pub(crate) fn compute_botness_assessment(
    context: BotnessSignalContext,
    cfg: &config::Config,
) -> BotnessAssessment {
    let contributions = collect_botness_contributions(context, cfg);
    compute_botness_assessment_from_contributions(contributions)
}

pub(crate) fn botness_signals_summary(assessment: &BotnessAssessment) -> String {
    let active = assessment
        .contributions
        .iter()
        .filter(|c| c.active)
        .map(|c| format!("{}:{}", c.key, c.contribution))
        .collect::<Vec<_>>();
    if active.is_empty() {
        "none".to_string()
    } else {
        active.join(",")
    }
}

pub(crate) fn botness_signal_states_summary(assessment: &BotnessAssessment) -> String {
    assessment
        .contributions
        .iter()
        .map(|signal| {
            format!(
                "{}:{}:{}",
                signal.key,
                signal.availability.as_str(),
                signal.contribution
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn defence_modes_effective_summary(cfg: &config::Config) -> String {
    let effective = cfg.defence_modes_effective();
    format!(
        "rate={}/{}/{} geo={}/{}/{} js={}/{}/{}",
        effective.rate.configured.as_str(),
        effective.rate.signal_enabled,
        effective.rate.action_enabled,
        effective.geo.configured.as_str(),
        effective.geo.signal_enabled,
        effective.geo.action_enabled,
        effective.js.configured.as_str(),
        effective.js.signal_enabled,
        effective.js.action_enabled
    )
}

pub(crate) fn defence_runtime_metadata_summary(cfg: &config::Config) -> String {
    format!(
        "modes={} edge={}",
        defence_modes_effective_summary(cfg),
        cfg.edge_integration_mode.as_str()
    )
}

pub(crate) fn provider_implementations_summary(
    registry: &providers::registry::ProviderRegistry,
) -> String {
    let capabilities = [
        providers::registry::ProviderCapability::RateLimiter,
        providers::registry::ProviderCapability::BanStore,
        providers::registry::ProviderCapability::ChallengeEngine,
        providers::registry::ProviderCapability::MazeTarpit,
        providers::registry::ProviderCapability::FingerprintSignal,
    ];

    capabilities
        .iter()
        .map(|capability| {
            format!(
                "{}={}/{}",
                capability.as_str(),
                registry.backend_for(*capability).as_str(),
                registry.implementation_for(*capability)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn write_log_line(out: &mut impl Write, msg: &str) {
    let _ = writeln!(out, "{}", msg);
}

pub(crate) fn log_line(msg: &str) {
    let mut out = std::io::stdout();
    write_log_line(&mut out, msg);
}

pub(crate) fn increment_metric_intent(
    metric: observability::metrics::MetricName,
    label: Option<String>,
) -> runtime::effect_intents::EffectIntent {
    runtime::effect_intents::EffectIntent::IncrementMetric { metric, label }
}

pub(crate) fn policy_signal_intent(
    signal_id: runtime::policy_taxonomy::SignalId,
) -> runtime::effect_intents::EffectIntent {
    increment_metric_intent(
        observability::metrics::MetricName::PolicySignals,
        Some(signal_id.as_str().to_string()),
    )
}

pub(crate) fn provider_backend_visibility_intents(
    provider_registry: &providers::registry::ProviderRegistry,
) -> Vec<runtime::effect_intents::EffectIntent> {
    [
        providers::registry::ProviderCapability::RateLimiter,
        providers::registry::ProviderCapability::BanStore,
        providers::registry::ProviderCapability::ChallengeEngine,
        providers::registry::ProviderCapability::MazeTarpit,
        providers::registry::ProviderCapability::FingerprintSignal,
    ]
    .iter()
    .map(|capability| {
        let backend = provider_registry.backend_for(*capability);
        let implementation = provider_registry.implementation_for(*capability);
        increment_metric_intent(
            observability::metrics::MetricName::ProviderImplementationEffective,
            Some(format!(
                "{}:{}:{}",
                capability.as_str(),
                backend.as_str(),
                implementation
            )),
        )
    })
    .collect()
}

pub(crate) fn serve_maze_with_tracking(
    req: &Request,
    store: &Store,
    cfg: &config::Config,
    ip: &str,
    user_agent: &str,
    path: &str,
    event_reason: &str,
    event_outcome: &str,
    botness_hint: Option<u8>,
) -> Response {
    let provider_registry = crate::providers::registry::ProviderRegistry::from_config(cfg);
    let capabilities = runtime::capabilities::RuntimeCapabilities::for_policy_execution_phase(
        LibCapabilityToken::new(),
    );
    let context = runtime::effect_intents::EffectExecutionContext {
        req,
        store,
        cfg,
        provider_registry: &provider_registry,
        site_id: "default",
        ip,
        ua: user_agent,
    };
    let execute_intents = |intents: Vec<runtime::effect_intents::EffectIntent>| {
        runtime::effect_intents::execute_effect_intents(intents, &context, &capabilities);
    };
    let maze_decision =
        crate::maze::runtime::serve(store, cfg, req, ip, user_agent, path, botness_hint);
    let served = match maze_decision {
        crate::maze::runtime::MazeServeDecision::Serve(served) => served,
        crate::maze::runtime::MazeServeDecision::Fallback(fallback) => {
            let reason = fallback.reason;
            let transition = match reason {
                crate::maze::runtime::MazeFallbackReason::TokenInvalid => {
                    runtime::policy_taxonomy::PolicyTransition::MazeTokenInvalid
                }
                crate::maze::runtime::MazeFallbackReason::TokenExpired => {
                    runtime::policy_taxonomy::PolicyTransition::MazeTokenExpired
                }
                crate::maze::runtime::MazeFallbackReason::TokenReplay => {
                    runtime::policy_taxonomy::PolicyTransition::MazeTokenReplay
                }
                crate::maze::runtime::MazeFallbackReason::TokenBindingMismatch => {
                    runtime::policy_taxonomy::PolicyTransition::MazeTokenBindingMismatch
                }
                crate::maze::runtime::MazeFallbackReason::TokenDepthExceeded => {
                    runtime::policy_taxonomy::PolicyTransition::MazeDepthExceeded
                }
                crate::maze::runtime::MazeFallbackReason::BudgetExceeded => {
                    runtime::policy_taxonomy::PolicyTransition::MazeBudgetExceeded
                }
                crate::maze::runtime::MazeFallbackReason::CheckpointMissing => {
                    runtime::policy_taxonomy::PolicyTransition::MazeCheckpointMissing
                }
                crate::maze::runtime::MazeFallbackReason::MicroPowFailed => {
                    runtime::policy_taxonomy::PolicyTransition::MazeMicroPowFailed
                }
            };
            let token_outcome = match reason {
                crate::maze::runtime::MazeFallbackReason::TokenInvalid => "invalid",
                crate::maze::runtime::MazeFallbackReason::TokenExpired => "expired",
                crate::maze::runtime::MazeFallbackReason::TokenReplay => "replay",
                crate::maze::runtime::MazeFallbackReason::TokenBindingMismatch => {
                    "binding_mismatch"
                }
                crate::maze::runtime::MazeFallbackReason::TokenDepthExceeded => "depth_exceeded",
                crate::maze::runtime::MazeFallbackReason::BudgetExceeded => "budget_exceeded",
                crate::maze::runtime::MazeFallbackReason::CheckpointMissing => "checkpoint_missing",
                crate::maze::runtime::MazeFallbackReason::MicroPowFailed => "micro_pow_failed",
            };
            let mut fallback_intents = vec![increment_metric_intent(
                observability::metrics::MetricName::MazeTokenOutcomes,
                Some(token_outcome.to_string()),
            )];
            match reason {
                crate::maze::runtime::MazeFallbackReason::BudgetExceeded => {
                    fallback_intents.push(increment_metric_intent(
                        observability::metrics::MetricName::MazeBudgetOutcomes,
                        Some("saturated".to_string()),
                    ));
                }
                crate::maze::runtime::MazeFallbackReason::MicroPowFailed => {
                    fallback_intents.push(increment_metric_intent(
                        observability::metrics::MetricName::MazeProofOutcomes,
                        Some("required".to_string()),
                    ));
                    fallback_intents.push(increment_metric_intent(
                        observability::metrics::MetricName::MazeProofOutcomes,
                        Some("failed".to_string()),
                    ));
                }
                crate::maze::runtime::MazeFallbackReason::CheckpointMissing => {
                    fallback_intents.push(increment_metric_intent(
                        observability::metrics::MetricName::MazeCheckpointOutcomes,
                        Some("invalid".to_string()),
                    ));
                }
                _ => {}
            }
            fallback_intents.push(runtime::effect_intents::EffectIntent::RecordPolicyMatch(
                match reason {
                    crate::maze::runtime::MazeFallbackReason::TokenInvalid => {
                        runtime::policy_taxonomy::PolicyTransition::MazeTokenInvalid
                    }
                    crate::maze::runtime::MazeFallbackReason::TokenExpired => {
                        runtime::policy_taxonomy::PolicyTransition::MazeTokenExpired
                    }
                    crate::maze::runtime::MazeFallbackReason::TokenReplay => {
                        runtime::policy_taxonomy::PolicyTransition::MazeTokenReplay
                    }
                    crate::maze::runtime::MazeFallbackReason::TokenBindingMismatch => {
                        runtime::policy_taxonomy::PolicyTransition::MazeTokenBindingMismatch
                    }
                    crate::maze::runtime::MazeFallbackReason::TokenDepthExceeded => {
                        runtime::policy_taxonomy::PolicyTransition::MazeDepthExceeded
                    }
                    crate::maze::runtime::MazeFallbackReason::BudgetExceeded => {
                        runtime::policy_taxonomy::PolicyTransition::MazeBudgetExceeded
                    }
                    crate::maze::runtime::MazeFallbackReason::CheckpointMissing => {
                        runtime::policy_taxonomy::PolicyTransition::MazeCheckpointMissing
                    }
                    crate::maze::runtime::MazeFallbackReason::MicroPowFailed => {
                        runtime::policy_taxonomy::PolicyTransition::MazeMicroPowFailed
                    }
                },
            ));
            execute_intents(fallback_intents);
            let policy_match = runtime::policy_taxonomy::resolve_policy_match(transition);
            let outcome = format!(
                "{} action={}",
                policy_match.annotate_outcome(reason.detection_label()),
                fallback.action.label()
            );
            match fallback.action {
                crate::maze::runtime::MazeFallbackAction::Block => {
                    execute_intents(vec![
                        runtime::effect_intents::EffectIntent::LogEvent {
                            event: crate::admin::EventType::Block,
                            reason: "maze_runtime_fallback".to_string(),
                            outcome,
                        },
                        increment_metric_intent(observability::metrics::MetricName::BlocksTotal, None),
                    ]);
                    return Response::new(
                        403,
                        block_page::render_block_page(block_page::BlockReason::Honeypot),
                    );
                }
                crate::maze::runtime::MazeFallbackAction::Challenge => {
                    execute_intents(vec![
                        runtime::effect_intents::EffectIntent::LogEvent {
                            event: crate::admin::EventType::Challenge,
                            reason: "maze_runtime_fallback".to_string(),
                            outcome,
                        },
                        increment_metric_intent(
                            observability::metrics::MetricName::ChallengeServedTotal,
                            None,
                        ),
                        increment_metric_intent(
                            observability::metrics::MetricName::ChallengesTotal,
                            None,
                        ),
                    ]);
                    let report_endpoint =
                        provider_registry.fingerprint_signal_provider().report_path();
                    return crate::signals::js_verification::inject_js_challenge(
                        ip,
                        user_agent,
                        report_endpoint,
                        cfg.pow_enabled,
                        cfg.pow_difficulty,
                        cfg.pow_ttl_seconds,
                        cfg.cdp_probe_family,
                        cfg.cdp_probe_rollout_percent,
                    );
                }
            }
        }
    };

    let mut served_intents = vec![
        increment_metric_intent(observability::metrics::MetricName::MazeHits, None),
        increment_metric_intent(
            observability::metrics::MetricName::MazeTokenOutcomes,
            Some(
                if served.token_validated {
                    "validated"
                } else {
                    "entry"
                }
                .to_string(),
            ),
        ),
        increment_metric_intent(
            observability::metrics::MetricName::MazeBudgetOutcomes,
            Some("acquired".to_string()),
        ),
    ];
    if served.response_cap_exceeded {
        served_intents.push(increment_metric_intent(
            observability::metrics::MetricName::MazeBudgetOutcomes,
            Some("response_cap_exceeded".to_string()),
        ));
    }
    if cfg.maze_micro_pow_enabled
        && served.depth >= cfg.maze_micro_pow_depth_start
        && served.token_validated
    {
        served_intents.push(increment_metric_intent(
            observability::metrics::MetricName::MazeProofOutcomes,
            Some("required".to_string()),
        ));
        served_intents.push(increment_metric_intent(
            observability::metrics::MetricName::MazeProofOutcomes,
            Some("passed".to_string()),
        ));
    }
    let variant_family = served
        .variant_id
        .split('-')
        .take(2)
        .collect::<Vec<_>>()
        .join("-");
    served_intents.push(increment_metric_intent(
        observability::metrics::MetricName::MazeEntropyVariants,
        Some(format!(
            "{}:{}:{}",
            variant_family,
            served.seed_provider,
            served.seed_metadata_only as u8
        )),
    ));
    served_intents.push(policy_signal_intent(
        runtime::policy_taxonomy::SignalId::MazeTraversal,
    ));
    if crate::request_validation::query_param(req.query(), "dc").is_some() {
        served_intents.push(policy_signal_intent(
            runtime::policy_taxonomy::SignalId::DecoyInteraction,
        ));
    }
    served_intents.push(runtime::effect_intents::EffectIntent::LogEvent {
        event: crate::admin::EventType::Challenge,
        reason: event_reason.to_string(),
        outcome: format!(
            "{} variant={} depth={} flow={} bytes={} render_ms={}",
            event_outcome, served.variant_id, served.depth, served.flow_id, served.bytes, served.render_ms
        ),
    });
    execute_intents(served_intents);

    // Bucket the IP to reduce KV cardinality and avoid per-IP explosion.
    let maze_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let maze_key = format!("maze_hits:{}", maze_bucket);
    let hits: u32 = store
        .get(&maze_key)
        .ok()
        .flatten()
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if let Err(e) = store.set(&maze_key, (hits + 1).to_string().as_bytes()) {
        log_line(&format!(
            "[maze] failed to persist hit counter {}: {:?}",
            maze_key, e
        ));
    } else if let Err(err) = crate::observability::key_catalog::register_key(
        store,
        crate::maze::maze_hits_catalog_key(),
        maze_key.as_str(),
    ) {
        log_line(&format!(
            "[maze] failed to register hit counter catalog key={} error={}",
            maze_key, err
        ));
    }

    if hits >= cfg.maze_auto_ban_threshold && cfg.maze_auto_ban {
        let policy_match = runtime::policy_taxonomy::resolve_policy_match(
            runtime::policy_taxonomy::PolicyTransition::MazeThresholdBan,
        );
        execute_intents(vec![
            runtime::effect_intents::EffectIntent::RecordPolicyMatch(
                runtime::policy_taxonomy::PolicyTransition::MazeThresholdBan,
            ),
            runtime::effect_intents::EffectIntent::Ban(runtime::effect_intents::BanIntent {
                reason: "maze_crawler".to_string(),
                duration_seconds: cfg.get_ban_duration("honeypot"),
                score: None,
                signals: vec!["maze_crawler_threshold".to_string()],
                summary: Some(format!(
                    "maze_hits={} threshold={}",
                    hits + 1,
                    cfg.maze_auto_ban_threshold
                )),
            }),
            increment_metric_intent(
                observability::metrics::MetricName::BansTotal,
                Some("maze_crawler".to_string()),
            ),
            runtime::effect_intents::EffectIntent::LogEvent {
                event: crate::admin::EventType::Ban,
                reason: "maze_crawler".to_string(),
                outcome: policy_match.annotate_outcome(
                    format!("banned_after_{}_maze_pages", cfg.maze_auto_ban_threshold).as_str(),
                ),
            },
        ]);
    }

    maze_response(served)
}

/// Main handler logic, testable as a plain Rust function.
pub fn handle_bot_defence_impl(req: &Request) -> Response {
    runtime::request_flow::handle_request(req)
}

#[http_component]
pub fn spin_entrypoint(req: Request) -> Response {
    let response = handle_bot_defence_impl(&req);
    if let Ok(store) = Store::open_default() {
        let capabilities = runtime::capabilities::RuntimeCapabilities::for_post_response_flush_phase(
            LibCapabilityToken::new(),
        );
        runtime::effect_intents::execute_monitoring_store_intents(
            vec![runtime::effect_intents::EffectIntent::FlushPendingMonitoringCounters],
            &store,
            &capabilities,
        );
    }
    response
}
