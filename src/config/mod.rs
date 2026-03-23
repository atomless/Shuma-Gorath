// src/config.rs
// Configuration and site settings for WASM Bot Defence.
// Tunables are loaded from KV; defaults are defined in config/defaults.env.

#[cfg(not(test))]
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::{HashMap, HashSet},
    env,
    net::{IpAddr, Ipv4Addr},
    sync::Mutex,
};

use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::challenge::KeyValueStore;

mod controller_action_catalog;
mod controller_action_guardrails;
mod controller_action_surface;
mod runtime_env;

#[cfg(test)]
pub(crate) use runtime_env::{clear_test_spin_variables, set_test_spin_variable};
pub(crate) use controller_action_surface::{
    allowed_actions_v1, controller_action_family_targets, controller_config_family_for_patch_key,
    AllowedActionsSurface,
};
pub(crate) use runtime_env::{runtime_var_raw_optional, runtime_var_trimmed_optional};

const DEFAULTS_ENV_TEXT: &str = include_str!("../../config/defaults.env");

pub const POW_DIFFICULTY_MIN: u8 = 12;
pub const POW_DIFFICULTY_MAX: u8 = 20;
pub const POW_TTL_MIN: u64 = 30;
pub const POW_TTL_MAX: u64 = 300;
pub const ADVERSARY_SIM_DURATION_SECONDS_MIN: u64 = 30;
pub const ADVERSARY_SIM_DURATION_SECONDS_MAX: u64 = 900;
const FRONTIER_OPENAI_MODEL_DEFAULT: &str = "gpt-5-mini";
const FRONTIER_ANTHROPIC_MODEL_DEFAULT: &str = "claude-3-5-haiku-latest";
const FRONTIER_GOOGLE_MODEL_DEFAULT: &str = "gemini-2.0-flash-lite";
const FRONTIER_XAI_MODEL_DEFAULT: &str = "grok-3-mini";
const MAZE_MICRO_POW_DIFFICULTY_MIN: u8 = 10;
const MAZE_MICRO_POW_DIFFICULTY_MAX: u8 = 24;
const CHALLENGE_THRESHOLD_MIN: u8 = 1;
const CHALLENGE_THRESHOLD_MAX: u8 = 10;
const MAZE_THRESHOLD_MIN: u8 = 1;
const MAZE_THRESHOLD_MAX: u8 = 10;
const NOT_A_BOT_SCORE_MIN: u8 = 1;
const NOT_A_BOT_SCORE_MAX: u8 = 10;
const BOTNESS_WEIGHT_MIN: u8 = 0;
const BOTNESS_WEIGHT_MAX: u8 = 10;
const CHALLENGE_TRANSFORM_COUNT_MIN: u8 = 4;
const CHALLENGE_TRANSFORM_COUNT_MAX: u8 = 8;
const CHALLENGE_PUZZLE_SEED_TTL_MIN: u64 = 30;
const CHALLENGE_PUZZLE_SEED_TTL_MAX: u64 = 300;
const CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MIN: u32 = 1;
const CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MAX: u32 = 100;
const CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MIN: u64 = 30;
const CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MAX: u64 = 3600;
const NOT_A_BOT_NONCE_TTL_MIN: u64 = 30;
const NOT_A_BOT_NONCE_TTL_MAX: u64 = 300;
const NOT_A_BOT_MARKER_TTL_MIN: u64 = 60;
const NOT_A_BOT_MARKER_TTL_MAX: u64 = 3600;
const NOT_A_BOT_ATTEMPT_LIMIT_MIN: u32 = 1;
const NOT_A_BOT_ATTEMPT_LIMIT_MAX: u32 = 100;
const NOT_A_BOT_ATTEMPT_WINDOW_MIN: u64 = 30;
const NOT_A_BOT_ATTEMPT_WINDOW_MAX: u64 = 3600;
const TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MIN: u64 = 20;
const TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MAX: u64 = 300;
const TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MIN: u64 = 30;
const TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MAX: u64 = 3_600;
const TARPIT_HASHCASH_DIFFICULTY_MIN: u8 = 4;
const TARPIT_HASHCASH_DIFFICULTY_MAX: u8 = 28;
const TARPIT_STEP_CHUNK_BASE_BYTES_MIN: u32 = 256;
const TARPIT_STEP_CHUNK_BASE_BYTES_MAX: u32 = 65_536;
const TARPIT_STEP_CHUNK_MAX_BYTES_MIN: u32 = 512;
const TARPIT_STEP_CHUNK_MAX_BYTES_MAX: u32 = 131_072;
const TARPIT_STEP_JITTER_PERCENT_MIN: u8 = 0;
const TARPIT_STEP_JITTER_PERCENT_MAX: u8 = 40;
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
const TARPIT_MAX_CONCURRENT_GLOBAL_MIN: u32 = 1;
const TARPIT_MAX_CONCURRENT_GLOBAL_MAX: u32 = 10_000;
const TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MIN: u32 = 1;
const TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MAX: u32 = 256;
const IP_RANGE_SUGGESTIONS_MIN_OBSERVATIONS_MIN: u32 = 1;
const IP_RANGE_SUGGESTIONS_MIN_OBSERVATIONS_MAX: u32 = 50_000;
const IP_RANGE_SUGGESTIONS_MIN_BOT_EVENTS_MIN: u32 = 1;
const IP_RANGE_SUGGESTIONS_MIN_BOT_EVENTS_MAX: u32 = 10_000;
const IP_RANGE_SUGGESTIONS_CONFIDENCE_PERCENT_MIN: u8 = 0;
const IP_RANGE_SUGGESTIONS_CONFIDENCE_PERCENT_MAX: u8 = 100;
const IP_RANGE_SUGGESTIONS_COLLATERAL_PERCENT_MIN: u8 = 0;
const IP_RANGE_SUGGESTIONS_COLLATERAL_PERCENT_MAX: u8 = 100;
const IP_RANGE_SUGGESTIONS_IPV4_MIN_PREFIX_LEN_MIN: u8 = 8;
const IP_RANGE_SUGGESTIONS_IPV4_MIN_PREFIX_LEN_MAX: u8 = 32;
const IP_RANGE_SUGGESTIONS_IPV6_MIN_PREFIX_LEN_MIN: u8 = 24;
const IP_RANGE_SUGGESTIONS_IPV6_MIN_PREFIX_LEN_MAX: u8 = 128;
const IP_RANGE_SUGGESTIONS_LIKELY_HUMAN_SAMPLE_PERCENT_MIN: u8 = 0;
const IP_RANGE_SUGGESTIONS_LIKELY_HUMAN_SAMPLE_PERCENT_MAX: u8 = 100;
const GATEWAY_LOOP_MAX_HOPS_MIN: u8 = 1;
const GATEWAY_LOOP_MAX_HOPS_MAX: u8 = 10;
const GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS_MIN: u32 = 1;
const GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS_MAX: u32 = 365;
const GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS_MIN: u32 = 1;
const GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS_MAX: u32 = 120;
const VERIFIED_IDENTITY_REPLAY_WINDOW_SECONDS_MIN: u64 = 30;
const VERIFIED_IDENTITY_REPLAY_WINDOW_SECONDS_MAX: u64 = 3_600;
const VERIFIED_IDENTITY_CLOCK_SKEW_SECONDS_MAX: u64 = 300;
const VERIFIED_IDENTITY_DIRECTORY_CACHE_TTL_SECONDS_MIN: u64 = 60;
const VERIFIED_IDENTITY_DIRECTORY_CACHE_TTL_SECONDS_MAX: u64 = 86_400;
const VERIFIED_IDENTITY_DIRECTORY_FRESHNESS_REQUIREMENT_SECONDS_MIN: u64 = 60;
const VERIFIED_IDENTITY_DIRECTORY_FRESHNESS_REQUIREMENT_SECONDS_MAX: u64 = 604_800;
#[cfg(not(test))]
const CONFIG_CACHE_TTL_SECONDS: u64 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigLoadError {
    StoreUnavailable,
    MissingConfig,
    InvalidConfig,
}

impl ConfigLoadError {
    pub fn user_message(&self) -> &'static str {
        match self {
            ConfigLoadError::StoreUnavailable => "Configuration unavailable (KV store error)",
            ConfigLoadError::MissingConfig => {
                "Configuration unavailable (missing KV config; run setup/config-seed)"
            }
            ConfigLoadError::InvalidConfig => "Configuration unavailable (invalid KV config)",
        }
    }
}

/// Weighted signal contributions for the unified botness score.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotnessWeights {
    #[serde(default = "default_botness_weight_js_required")]
    pub js_required: u8,
    #[serde(default = "default_botness_weight_geo_risk")]
    pub geo_risk: u8,
    #[serde(default = "default_botness_weight_rate_medium")]
    pub rate_medium: u8,
    #[serde(default = "default_botness_weight_rate_high")]
    pub rate_high: u8,
    #[serde(default = "default_botness_weight_maze_behavior")]
    pub maze_behavior: u8,
}

impl Default for BotnessWeights {
    fn default() -> Self {
        BotnessWeights {
            js_required: default_botness_weight_js_required(),
            geo_risk: default_botness_weight_geo_risk(),
            rate_medium: default_botness_weight_rate_medium(),
            rate_high: default_botness_weight_rate_high(),
            maze_behavior: default_botness_weight_maze_behavior(),
        }
    }
}

/// Per-module composability modes for signal and enforcement/action paths.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComposabilityMode {
    Off,
    Signal,
    Enforce,
    Both,
}

impl ComposabilityMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ComposabilityMode::Off => "off",
            ComposabilityMode::Signal => "signal",
            ComposabilityMode::Enforce => "enforce",
            ComposabilityMode::Both => "both",
        }
    }

    pub fn signal_enabled(self) -> bool {
        matches!(self, ComposabilityMode::Signal | ComposabilityMode::Both)
    }

    pub fn action_enabled(self) -> bool {
        matches!(self, ComposabilityMode::Enforce | ComposabilityMode::Both)
    }
}

/// Composability controls for hybrid/eligible defenses.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefenceModes {
    #[serde(default = "default_mode_rate")]
    pub rate: ComposabilityMode,
    #[serde(default = "default_mode_geo")]
    pub geo: ComposabilityMode,
    #[serde(default = "default_mode_js")]
    pub js: ComposabilityMode,
}

impl Default for DefenceModes {
    fn default() -> Self {
        Self {
            rate: default_mode_rate(),
            geo: default_mode_geo(),
            js: default_mode_js(),
        }
    }
}

/// Provider backend selection for swappable capabilities.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderBackend {
    Internal,
    External,
}

impl ProviderBackend {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderBackend::Internal => "internal",
            ProviderBackend::External => "external",
        }
    }
}

/// Integration precedence for managed-edge outcomes versus internal policy.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeIntegrationMode {
    Off,
    Additive,
    Authoritative,
}

impl EdgeIntegrationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            EdgeIntegrationMode::Off => "off",
            EdgeIntegrationMode::Additive => "additive",
            EdgeIntegrationMode::Authoritative => "authoritative",
        }
    }
}

/// Detector-surface rotation family for JS/CDP probe script.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CdpProbeFamily {
    V1,
    V2,
    Split,
}

impl CdpProbeFamily {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            CdpProbeFamily::V1 => "v1",
            CdpProbeFamily::V2 => "v2",
            CdpProbeFamily::Split => "split",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MazeRolloutPhase {
    Instrument,
    Advisory,
    Enforce,
}

impl MazeRolloutPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            MazeRolloutPhase::Instrument => "instrument",
            MazeRolloutPhase::Advisory => "advisory",
            MazeRolloutPhase::Enforce => "enforce",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MazeSeedProvider {
    Internal,
    Operator,
}

impl MazeSeedProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            MazeSeedProvider::Internal => "internal",
            MazeSeedProvider::Operator => "operator",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TarpitFallbackAction {
    Maze,
    Block,
}

impl TarpitFallbackAction {
    pub fn as_str(self) -> &'static str {
        match self {
            TarpitFallbackAction::Maze => "maze",
            TarpitFallbackAction::Block => "block",
        }
    }
}

/// Outage posture for external distributed rate limiter degradation.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RateLimiterOutageMode {
    FallbackInternal,
    FailOpen,
    FailClosed,
}

impl RateLimiterOutageMode {
    pub fn as_str(self) -> &'static str {
        match self {
            RateLimiterOutageMode::FallbackInternal => "fallback_internal",
            RateLimiterOutageMode::FailOpen => "fail_open",
            RateLimiterOutageMode::FailClosed => "fail_closed",
        }
    }
}

/// Outage posture for external distributed ban-store degradation.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BanStoreOutageMode {
    FallbackInternal,
    FailOpen,
    FailClosed,
}

impl BanStoreOutageMode {
    pub fn as_str(self) -> &'static str {
        match self {
            BanStoreOutageMode::FallbackInternal => "fallback_internal",
            BanStoreOutageMode::FailOpen => "fail_open",
            BanStoreOutageMode::FailClosed => "fail_closed",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeEnvironment {
    RuntimeDev,
    RuntimeProd,
}

impl RuntimeEnvironment {
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeEnvironment::RuntimeDev => "runtime-dev",
            RuntimeEnvironment::RuntimeProd => "runtime-prod",
        }
    }

    #[allow(dead_code)]
    pub fn is_dev(self) -> bool {
        matches!(self, RuntimeEnvironment::RuntimeDev)
    }

    pub fn is_prod(self) -> bool {
        matches!(self, RuntimeEnvironment::RuntimeProd)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GatewayDeploymentProfile {
    SharedServer,
    EdgeFermyon,
}

impl GatewayDeploymentProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            GatewayDeploymentProfile::SharedServer => "shared-server",
            GatewayDeploymentProfile::EdgeFermyon => "edge-fermyon",
        }
    }

    pub fn is_edge(self) -> bool {
        matches!(self, GatewayDeploymentProfile::EdgeFermyon)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayOriginAuthMode {
    NetworkOnly,
    SignedHeader,
}

impl GatewayOriginAuthMode {
    pub fn as_str(self) -> &'static str {
        match self {
            GatewayOriginAuthMode::NetworkOnly => "network_only",
            GatewayOriginAuthMode::SignedHeader => "signed_header",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GatewayUpstreamOrigin {
    scheme: String,
    host: String,
    port: u16,
    host_is_ip_literal: bool,
}

impl GatewayUpstreamOrigin {
    fn authority(&self) -> String {
        if self.host_is_ip_literal && self.host.contains(':') {
            format!("[{}]:{}", self.host, self.port)
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IpRangePolicyMode {
    Off,
    Advisory,
    Enforce,
}

impl IpRangePolicyMode {
    pub fn as_str(self) -> &'static str {
        match self {
            IpRangePolicyMode::Off => "off",
            IpRangePolicyMode::Advisory => "advisory",
            IpRangePolicyMode::Enforce => "enforce",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum IpRangePolicyAction {
    #[default]
    Forbidden403,
    CustomMessage,
    DropConnection,
    Redirect308,
    RateLimit,
    Honeypot,
    Maze,
    Tarpit,
}

impl IpRangePolicyAction {
    pub fn as_str(self) -> &'static str {
        match self {
            IpRangePolicyAction::Forbidden403 => "forbidden_403",
            IpRangePolicyAction::CustomMessage => "custom_message",
            IpRangePolicyAction::DropConnection => "drop_connection",
            IpRangePolicyAction::Redirect308 => "redirect_308",
            IpRangePolicyAction::RateLimit => "rate_limit",
            IpRangePolicyAction::Honeypot => "honeypot",
            IpRangePolicyAction::Maze => "maze",
            IpRangePolicyAction::Tarpit => "tarpit",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct IpRangePolicyRule {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub cidrs: Vec<String>,
    #[serde(default)]
    pub action: IpRangePolicyAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_message: Option<String>,
}

/// Per-capability provider backend selections.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProviderBackends {
    #[serde(default = "default_provider_rate_limiter")]
    pub rate_limiter: ProviderBackend,
    #[serde(default = "default_provider_ban_store")]
    pub ban_store: ProviderBackend,
    #[serde(default = "default_provider_challenge_engine")]
    pub challenge_engine: ProviderBackend,
    #[serde(default = "default_provider_maze_tarpit")]
    pub maze_tarpit: ProviderBackend,
    #[serde(default = "default_provider_fingerprint_signal")]
    pub fingerprint_signal: ProviderBackend,
}

impl Default for ProviderBackends {
    fn default() -> Self {
        Self {
            rate_limiter: default_provider_rate_limiter(),
            ban_store: default_provider_ban_store(),
            challenge_engine: default_provider_challenge_engine(),
            maze_tarpit: default_provider_maze_tarpit(),
            fingerprint_signal: default_provider_fingerprint_signal(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VerifiedIdentityConfig {
    #[serde(default = "default_verified_identity_enabled")]
    pub enabled: bool,
    #[serde(default = "default_verified_identity_native_web_bot_auth_enabled")]
    pub native_web_bot_auth_enabled: bool,
    #[serde(default = "default_verified_identity_provider_assertions_enabled")]
    pub provider_assertions_enabled: bool,
    #[serde(default = "default_verified_identity_non_human_traffic_stance")]
    pub non_human_traffic_stance: crate::bot_identity::policy::NonHumanTrafficStance,
    #[serde(default = "default_verified_identity_replay_window_seconds")]
    pub replay_window_seconds: u64,
    #[serde(default = "default_verified_identity_clock_skew_seconds")]
    pub clock_skew_seconds: u64,
    #[serde(default = "default_verified_identity_directory_cache_ttl_seconds")]
    pub directory_cache_ttl_seconds: u64,
    #[serde(default = "default_verified_identity_directory_freshness_requirement_seconds")]
    pub directory_freshness_requirement_seconds: u64,
    #[serde(default = "default_verified_identity_named_policies")]
    pub named_policies: Vec<crate::bot_identity::policy::IdentityPolicyEntry>,
    #[serde(default = "default_verified_identity_category_defaults")]
    pub category_defaults: Vec<crate::bot_identity::policy::IdentityCategoryDefaultAction>,
    #[serde(default = "default_verified_identity_service_profiles")]
    pub service_profiles: Vec<crate::bot_identity::policy::IdentityServiceProfileBinding>,
}

impl Default for VerifiedIdentityConfig {
    fn default() -> Self {
        Self {
            enabled: default_verified_identity_enabled(),
            native_web_bot_auth_enabled: default_verified_identity_native_web_bot_auth_enabled(),
            provider_assertions_enabled: default_verified_identity_provider_assertions_enabled(),
            non_human_traffic_stance: default_verified_identity_non_human_traffic_stance(),
            replay_window_seconds: default_verified_identity_replay_window_seconds(),
            clock_skew_seconds: default_verified_identity_clock_skew_seconds(),
            directory_cache_ttl_seconds: default_verified_identity_directory_cache_ttl_seconds(),
            directory_freshness_requirement_seconds:
                default_verified_identity_directory_freshness_requirement_seconds(),
            named_policies: default_verified_identity_named_policies(),
            category_defaults: default_verified_identity_category_defaults(),
            service_profiles: default_verified_identity_service_profiles(),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct DefenceModeEffective {
    pub configured: ComposabilityMode,
    pub signal_enabled: bool,
    pub action_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct DefenceModesEffective {
    pub rate: DefenceModeEffective,
    pub geo: DefenceModeEffective,
    pub js: DefenceModeEffective,
}

/// Ban duration settings per ban type (in seconds)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BanDurations {
    #[serde(default = "default_ban_duration_honeypot")]
    pub honeypot: u64,
    #[serde(default = "default_ban_duration_ip_range_honeypot")]
    pub ip_range_honeypot: u64,
    #[serde(default = "default_ban_duration_maze_crawler")]
    pub maze_crawler: u64,
    #[serde(default = "default_ban_duration_rate_limit")]
    pub rate_limit: u64,
    #[serde(default = "default_ban_duration_admin")]
    pub admin: u64,
    #[serde(default = "default_ban_duration_cdp")]
    pub cdp: u64,
    #[serde(default = "default_ban_duration_edge_fingerprint")]
    pub edge_fingerprint: u64,
    #[serde(default = "default_ban_duration_tarpit_persistence")]
    pub tarpit_persistence: u64,
    #[serde(default = "default_ban_duration_not_a_bot_abuse")]
    pub not_a_bot_abuse: u64,
    #[serde(default = "default_ban_duration_challenge_puzzle_abuse")]
    pub challenge_puzzle_abuse: u64,
}

impl Default for BanDurations {
    fn default() -> Self {
        BanDurations {
            honeypot: default_ban_duration_honeypot(),
            ip_range_honeypot: default_ban_duration_ip_range_honeypot(),
            maze_crawler: default_ban_duration_maze_crawler(),
            rate_limit: default_ban_duration_rate_limit(),
            admin: default_ban_duration_admin(),
            cdp: default_ban_duration_cdp(),
            edge_fingerprint: default_ban_duration_edge_fingerprint(),
            tarpit_persistence: default_ban_duration_tarpit_persistence(),
            not_a_bot_abuse: default_ban_duration_not_a_bot_abuse(),
            challenge_puzzle_abuse: default_ban_duration_challenge_puzzle_abuse(),
        }
    }
}

impl BanDurations {
    /// Get the configured duration for a specific ban family.
    pub fn get(&self, ban_type: &str) -> Option<u64> {
        match ban_type {
            "honeypot" => Some(self.honeypot),
            "ip_range_honeypot" => Some(self.ip_range_honeypot),
            "maze_crawler" => Some(self.maze_crawler),
            "rate" | "rate_limit" => Some(self.rate_limit),
            "cdp" | "cdp_automation" => Some(self.cdp),
            "edge_fingerprint" | "edge_fingerprint_automation" => Some(self.edge_fingerprint),
            "tarpit_persistence" => Some(self.tarpit_persistence),
            "not_a_bot_abuse" => Some(self.not_a_bot_abuse),
            "challenge_puzzle_abuse" | "challenge_submit_abuse" => {
                Some(self.challenge_puzzle_abuse)
            }
            "admin" | "manual_ban" => Some(self.admin),
            _ => None,
        }
    }
}

/// Configuration struct for a site, loaded from KV.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_ban_duration")]
    pub ban_duration: u64,
    #[serde(default)]
    pub ban_durations: BanDurations,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    #[serde(default = "default_honeypot_enabled")]
    pub honeypot_enabled: bool,
    #[serde(default = "default_honeypots")]
    pub honeypots: Vec<String>,
    #[serde(default = "default_browser_policy_enabled")]
    pub browser_policy_enabled: bool,
    #[serde(default = "default_browser_block")]
    pub browser_block: Vec<(String, u32)>,
    #[serde(default = "default_browser_allowlist")]
    pub browser_allowlist: Vec<(String, u32)>,
    #[serde(default = "default_geo_risk")]
    pub geo_risk: Vec<String>,
    #[serde(default = "default_geo_allow")]
    pub geo_allow: Vec<String>,
    #[serde(default = "default_geo_challenge")]
    pub geo_challenge: Vec<String>,
    #[serde(default = "default_geo_maze")]
    pub geo_maze: Vec<String>,
    #[serde(default = "default_geo_block")]
    pub geo_block: Vec<String>,
    #[serde(default = "default_geo_edge_headers_enabled")]
    pub geo_edge_headers_enabled: bool,
    #[serde(default = "default_bypass_allowlists_enabled")]
    pub bypass_allowlists_enabled: bool,
    #[serde(default = "default_allowlist")]
    pub allowlist: Vec<String>,
    #[serde(default = "default_path_allowlist_enabled")]
    pub path_allowlist_enabled: bool,
    #[serde(default = "default_path_allowlist")]
    pub path_allowlist: Vec<String>,
    #[serde(default = "default_ip_range_policy_mode")]
    pub ip_range_policy_mode: IpRangePolicyMode,
    #[serde(default = "default_ip_range_emergency_allowlist")]
    pub ip_range_emergency_allowlist: Vec<String>,
    #[serde(default = "default_ip_range_custom_rules")]
    pub ip_range_custom_rules: Vec<IpRangePolicyRule>,
    #[serde(default = "default_ip_range_suggestions_min_observations")]
    pub ip_range_suggestions_min_observations: u32,
    #[serde(default = "default_ip_range_suggestions_min_bot_events")]
    pub ip_range_suggestions_min_bot_events: u32,
    #[serde(default = "default_ip_range_suggestions_min_confidence_percent")]
    pub ip_range_suggestions_min_confidence_percent: u8,
    #[serde(default = "default_ip_range_suggestions_low_collateral_percent")]
    pub ip_range_suggestions_low_collateral_percent: u8,
    #[serde(default = "default_ip_range_suggestions_high_collateral_percent")]
    pub ip_range_suggestions_high_collateral_percent: u8,
    #[serde(default = "default_ip_range_suggestions_ipv4_min_prefix_len")]
    pub ip_range_suggestions_ipv4_min_prefix_len: u8,
    #[serde(default = "default_ip_range_suggestions_ipv6_min_prefix_len")]
    pub ip_range_suggestions_ipv6_min_prefix_len: u8,
    #[serde(default = "default_ip_range_suggestions_likely_human_sample_percent")]
    pub ip_range_suggestions_likely_human_sample_percent: u8,
    #[serde(default = "default_shadow_mode")]
    pub shadow_mode: bool,
    #[serde(default = "default_adversary_sim_enabled")]
    pub adversary_sim_enabled: bool,
    #[serde(default = "default_adversary_sim_duration_seconds")]
    pub adversary_sim_duration_seconds: u64,
    #[serde(default = "default_maze_enabled")]
    pub maze_enabled: bool,
    #[serde(default = "default_tarpit_enabled")]
    pub tarpit_enabled: bool,
    #[serde(default = "default_tarpit_progress_token_ttl_seconds")]
    pub tarpit_progress_token_ttl_seconds: u64,
    #[serde(default = "default_tarpit_progress_replay_ttl_seconds")]
    pub tarpit_progress_replay_ttl_seconds: u64,
    #[serde(default = "default_tarpit_hashcash_min_difficulty")]
    pub tarpit_hashcash_min_difficulty: u8,
    #[serde(default = "default_tarpit_hashcash_max_difficulty")]
    pub tarpit_hashcash_max_difficulty: u8,
    #[serde(default = "default_tarpit_hashcash_base_difficulty")]
    pub tarpit_hashcash_base_difficulty: u8,
    #[serde(default = "default_tarpit_hashcash_adaptive")]
    pub tarpit_hashcash_adaptive: bool,
    #[serde(default = "default_tarpit_step_chunk_base_bytes")]
    pub tarpit_step_chunk_base_bytes: u32,
    #[serde(default = "default_tarpit_step_chunk_max_bytes")]
    pub tarpit_step_chunk_max_bytes: u32,
    #[serde(default = "default_tarpit_step_jitter_percent")]
    pub tarpit_step_jitter_percent: u8,
    #[serde(default = "default_tarpit_shard_rotation_enabled")]
    pub tarpit_shard_rotation_enabled: bool,
    #[serde(default = "default_tarpit_egress_window_seconds")]
    pub tarpit_egress_window_seconds: u64,
    #[serde(default = "default_tarpit_egress_global_bytes_per_window")]
    pub tarpit_egress_global_bytes_per_window: u64,
    #[serde(default = "default_tarpit_egress_per_ip_bucket_bytes_per_window")]
    pub tarpit_egress_per_ip_bucket_bytes_per_window: u64,
    #[serde(default = "default_tarpit_egress_per_flow_max_bytes")]
    pub tarpit_egress_per_flow_max_bytes: u64,
    #[serde(default = "default_tarpit_egress_per_flow_max_duration_seconds")]
    pub tarpit_egress_per_flow_max_duration_seconds: u64,
    #[serde(default = "default_tarpit_max_concurrent_global")]
    pub tarpit_max_concurrent_global: u32,
    #[serde(default = "default_tarpit_max_concurrent_per_ip_bucket")]
    pub tarpit_max_concurrent_per_ip_bucket: u32,
    #[serde(default = "default_tarpit_fallback_action")]
    pub tarpit_fallback_action: TarpitFallbackAction,
    #[serde(default = "default_maze_auto_ban")]
    pub maze_auto_ban: bool,
    #[serde(default = "default_maze_auto_ban_threshold")]
    pub maze_auto_ban_threshold: u32,
    #[serde(default = "default_maze_rollout_phase")]
    pub maze_rollout_phase: MazeRolloutPhase,
    #[serde(default = "default_maze_token_ttl_seconds")]
    pub maze_token_ttl_seconds: u64,
    #[serde(default = "default_maze_token_max_depth")]
    pub maze_token_max_depth: u16,
    #[serde(default = "default_maze_token_branch_budget")]
    pub maze_token_branch_budget: u8,
    #[serde(default = "default_maze_replay_ttl_seconds")]
    pub maze_replay_ttl_seconds: u64,
    #[serde(default = "default_maze_entropy_window_seconds")]
    pub maze_entropy_window_seconds: u64,
    #[serde(default = "default_maze_client_expansion_enabled")]
    pub maze_client_expansion_enabled: bool,
    #[serde(default = "default_maze_checkpoint_every_nodes")]
    pub maze_checkpoint_every_nodes: u64,
    #[serde(default = "default_maze_checkpoint_every_ms")]
    pub maze_checkpoint_every_ms: u64,
    #[serde(default = "default_maze_step_ahead_max")]
    pub maze_step_ahead_max: u64,
    #[serde(default = "default_maze_no_js_fallback_max_depth")]
    pub maze_no_js_fallback_max_depth: u16,
    #[serde(default = "default_maze_micro_pow_enabled")]
    pub maze_micro_pow_enabled: bool,
    #[serde(default = "default_maze_micro_pow_depth_start")]
    pub maze_micro_pow_depth_start: u16,
    #[serde(default = "default_maze_micro_pow_base_difficulty")]
    pub maze_micro_pow_base_difficulty: u8,
    #[serde(default = "default_maze_max_concurrent_global")]
    pub maze_max_concurrent_global: u32,
    #[serde(default = "default_maze_max_concurrent_per_ip_bucket")]
    pub maze_max_concurrent_per_ip_bucket: u32,
    #[serde(default = "default_maze_max_response_bytes")]
    pub maze_max_response_bytes: u32,
    #[serde(default = "default_maze_max_response_duration_ms")]
    pub maze_max_response_duration_ms: u64,
    #[serde(default = "default_maze_server_visible_links")]
    pub maze_server_visible_links: u32,
    #[serde(default = "default_maze_max_links")]
    pub maze_max_links: u32,
    #[serde(default = "default_maze_max_paragraphs")]
    pub maze_max_paragraphs: u32,
    #[serde(default = "default_maze_path_entropy_segment_len")]
    pub maze_path_entropy_segment_len: u8,
    #[serde(default = "default_maze_covert_decoys_enabled")]
    pub maze_covert_decoys_enabled: bool,
    #[serde(default = "default_maze_seed_provider")]
    pub maze_seed_provider: MazeSeedProvider,
    #[serde(default = "default_maze_seed_refresh_interval_seconds")]
    pub maze_seed_refresh_interval_seconds: u64,
    #[serde(default = "default_maze_seed_refresh_rate_limit_per_hour")]
    pub maze_seed_refresh_rate_limit_per_hour: u32,
    #[serde(default = "default_maze_seed_refresh_max_sources")]
    pub maze_seed_refresh_max_sources: u32,
    #[serde(default = "default_maze_seed_metadata_only")]
    pub maze_seed_metadata_only: bool,
    #[serde(default = "default_robots_enabled")]
    pub robots_enabled: bool,
    #[serde(default = "default_robots_block_ai_training")]
    pub robots_block_ai_training: bool,
    #[serde(default = "default_robots_block_ai_search")]
    pub robots_block_ai_search: bool,
    #[serde(default = "default_robots_allow_search_engines")]
    pub robots_allow_search_engines: bool,
    #[serde(default = "default_robots_crawl_delay")]
    pub robots_crawl_delay: u32,
    #[serde(default = "default_cdp_detection_enabled")]
    pub cdp_detection_enabled: bool,
    #[serde(default = "default_cdp_auto_ban")]
    pub cdp_auto_ban: bool,
    #[serde(default = "default_cdp_threshold")]
    pub cdp_detection_threshold: f32,
    #[serde(default = "default_cdp_probe_family")]
    pub cdp_probe_family: CdpProbeFamily,
    #[serde(default = "default_cdp_probe_rollout_percent")]
    pub cdp_probe_rollout_percent: u8,
    #[serde(default = "default_fingerprint_signal_enabled")]
    pub fingerprint_signal_enabled: bool,
    #[serde(default = "default_fingerprint_state_ttl_seconds")]
    pub fingerprint_state_ttl_seconds: u64,
    #[serde(default = "default_fingerprint_flow_window_seconds")]
    pub fingerprint_flow_window_seconds: u64,
    #[serde(default = "default_fingerprint_flow_violation_threshold")]
    pub fingerprint_flow_violation_threshold: u8,
    #[serde(default = "default_fingerprint_pseudonymize")]
    pub fingerprint_pseudonymize: bool,
    #[serde(default = "default_fingerprint_entropy_budget")]
    pub fingerprint_entropy_budget: u8,
    #[serde(default = "default_fingerprint_family_cap_header_runtime")]
    pub fingerprint_family_cap_header_runtime: u8,
    #[serde(default = "default_fingerprint_family_cap_transport")]
    pub fingerprint_family_cap_transport: u8,
    #[serde(default = "default_fingerprint_family_cap_temporal")]
    pub fingerprint_family_cap_temporal: u8,
    #[serde(default = "default_fingerprint_family_cap_persistence")]
    pub fingerprint_family_cap_persistence: u8,
    #[serde(default = "default_fingerprint_family_cap_behavior")]
    pub fingerprint_family_cap_behavior: u8,
    #[serde(default = "default_js_required_enforced")]
    pub js_required_enforced: bool,
    #[serde(default = "default_pow_enabled")]
    pub pow_enabled: bool,
    #[serde(default = "default_pow_difficulty")]
    pub pow_difficulty: u8,
    #[serde(default = "default_pow_ttl_seconds")]
    pub pow_ttl_seconds: u64,
    #[serde(default = "default_challenge_puzzle_enabled")]
    pub challenge_puzzle_enabled: bool,
    #[serde(default = "default_challenge_puzzle_transform_count")]
    pub challenge_puzzle_transform_count: u8,
    #[serde(default = "default_challenge_puzzle_seed_ttl_seconds")]
    pub challenge_puzzle_seed_ttl_seconds: u64,
    #[serde(default = "default_challenge_puzzle_attempt_limit_per_window")]
    pub challenge_puzzle_attempt_limit_per_window: u32,
    #[serde(default = "default_challenge_puzzle_attempt_window_seconds")]
    pub challenge_puzzle_attempt_window_seconds: u64,
    #[serde(default = "default_challenge_threshold")]
    pub challenge_puzzle_risk_threshold: u8,
    #[serde(default = "default_not_a_bot_enabled")]
    pub not_a_bot_enabled: bool,
    #[serde(default = "default_not_a_bot_risk_threshold")]
    pub not_a_bot_risk_threshold: u8,
    #[serde(default = "default_not_a_bot_pass_score")]
    pub not_a_bot_pass_score: u8,
    #[serde(default = "default_not_a_bot_fail_score")]
    pub not_a_bot_fail_score: u8,
    #[serde(default = "default_not_a_bot_nonce_ttl_seconds")]
    pub not_a_bot_nonce_ttl_seconds: u64,
    #[serde(default = "default_not_a_bot_marker_ttl_seconds")]
    pub not_a_bot_marker_ttl_seconds: u64,
    #[serde(default = "default_not_a_bot_attempt_limit_per_window")]
    pub not_a_bot_attempt_limit_per_window: u32,
    #[serde(default = "default_not_a_bot_attempt_window_seconds")]
    pub not_a_bot_attempt_window_seconds: u64,
    #[serde(default = "default_maze_threshold")]
    pub botness_maze_threshold: u8,
    #[serde(default)]
    pub botness_weights: BotnessWeights,
    #[serde(default)]
    pub defence_modes: DefenceModes,
    #[serde(default)]
    pub provider_backends: ProviderBackends,
    #[serde(default = "default_edge_integration_mode")]
    pub edge_integration_mode: EdgeIntegrationMode,
    #[serde(default)]
    pub verified_identity: VerifiedIdentityConfig,
}

#[derive(Debug, Clone)]
struct CachedConfig {
    loaded_at: u64,
    config: Config,
}

#[derive(Debug, Clone, Copy, Default)]
struct RuntimeEphemeralFlags {
    shadow_mode_override: Option<bool>,
}

impl Config {
    /// Loads config for a site from KV only.
    pub fn load(store: &impl KeyValueStore, site_id: &str) -> Result<Self, ConfigLoadError> {
        let key = format!("config:{}", site_id);
        let val = store
            .get(&key)
            .map_err(|_| ConfigLoadError::StoreUnavailable)?
            .ok_or(ConfigLoadError::MissingConfig)?;

        let mut cfg =
            serde_json::from_slice::<Config>(&val).map_err(|_| ConfigLoadError::InvalidConfig)?;
        clamp_config_values(&mut cfg);
        validate_persisted_config(&cfg).map_err(|_| ConfigLoadError::InvalidConfig)?;
        Ok(cfg)
    }

    /// Returns ban duration for a specific ban type.
    pub fn get_ban_duration(&self, ban_type: &str) -> u64 {
        self.ban_durations.get(ban_type).unwrap_or(self.ban_duration)
    }

    pub fn rate_signal_enabled(&self) -> bool {
        self.defence_modes.rate.signal_enabled()
    }

    pub fn rate_action_enabled(&self) -> bool {
        self.defence_modes.rate.action_enabled()
    }

    pub fn geo_signal_enabled(&self) -> bool {
        self.defence_modes.geo.signal_enabled()
    }

    pub fn geo_action_enabled(&self) -> bool {
        self.defence_modes.geo.action_enabled()
    }

    pub fn js_signal_enabled(&self) -> bool {
        self.js_required_enforced && self.defence_modes.js.signal_enabled()
    }

    pub fn js_action_enabled(&self) -> bool {
        self.js_required_enforced && self.defence_modes.js.action_enabled()
    }

    pub fn defence_mode_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if !self.js_required_enforced && self.defence_modes.js != ComposabilityMode::Off {
            warnings.push(
                "js_required_enforced=false disables JS signal/action regardless of defence_modes.js"
                    .to_string(),
            );
        }
        warnings
    }

    fn enterprise_unsynced_state_active(&self) -> bool {
        self.provider_backends.rate_limiter != ProviderBackend::External
            || self.provider_backends.ban_store != ProviderBackend::External
    }

    pub fn enterprise_state_guardrail_error(&self) -> Option<String> {
        if !enterprise_multi_instance_enabled() {
            return None;
        }

        if self.provider_backends.rate_limiter == ProviderBackend::External
            && rate_limiter_redis_url().is_none()
        {
            return Some(
                "enterprise multi-instance rollout with SHUMA_PROVIDER_RATE_LIMITER=external requires SHUMA_RATE_LIMITER_REDIS_URL (redis:// or rediss://)"
                    .to_string(),
            );
        }
        if self.provider_backends.ban_store == ProviderBackend::External
            && ban_store_redis_url().is_none()
        {
            return Some(
                "enterprise multi-instance rollout with SHUMA_PROVIDER_BAN_STORE=external requires SHUMA_BAN_STORE_REDIS_URL (redis:// or rediss://)"
                    .to_string(),
            );
        }
        if self.edge_integration_mode == EdgeIntegrationMode::Authoritative
            && self.provider_backends.ban_store == ProviderBackend::External
            && ban_store_outage_mode() != BanStoreOutageMode::FailClosed
        {
            return Some(
                "enterprise authoritative external ban store requires SHUMA_BAN_STORE_OUTAGE_MODE=fail_closed"
                    .to_string(),
            );
        }

        if !self.enterprise_unsynced_state_active() {
            return None;
        }

        if self.edge_integration_mode == EdgeIntegrationMode::Authoritative {
            return Some(
                "enterprise multi-instance rollout cannot run with local-only rate/ban state in authoritative mode"
                    .to_string(),
            );
        }

        if !enterprise_unsynced_state_exception_confirmed() {
            return Some(
                "enterprise multi-instance rollout using local-only rate/ban state requires SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true"
                    .to_string(),
            );
        }

        None
    }

    pub fn enterprise_state_guardrail_warnings(&self) -> Vec<String> {
        if !enterprise_multi_instance_enabled() || !self.enterprise_unsynced_state_active() {
            return Vec::new();
        }

        if self.edge_integration_mode == EdgeIntegrationMode::Authoritative {
            return Vec::new();
        }

        if enterprise_unsynced_state_exception_confirmed() {
            vec!["enterprise multi-instance rollout is using local-only rate/ban state under explicit additive/off exception; keep this temporary until distributed state is enabled".to_string()]
        } else {
            Vec::new()
        }
    }

    pub fn defence_modes_effective(&self) -> DefenceModesEffective {
        let js_note =
            if !self.js_required_enforced && self.defence_modes.js != ComposabilityMode::Off {
                Some("Overridden by js_required_enforced=false".to_string())
            } else {
                None
            };

        DefenceModesEffective {
            rate: DefenceModeEffective {
                configured: self.defence_modes.rate,
                signal_enabled: self.rate_signal_enabled(),
                action_enabled: self.rate_action_enabled(),
                note: None,
            },
            geo: DefenceModeEffective {
                configured: self.defence_modes.geo,
                signal_enabled: self.geo_signal_enabled(),
                action_enabled: self.geo_action_enabled(),
                note: None,
            },
            js: DefenceModeEffective {
                configured: self.defence_modes.js,
                signal_enabled: self.js_signal_enabled(),
                action_enabled: self.js_action_enabled(),
                note: js_note,
            },
        }
    }
}

pub fn default_seeded_config() -> Config {
    let mut cfg =
        serde_json::from_str::<Config>("{}").expect("config defaults JSON must deserialize");
    clamp_config_values(&mut cfg);
    validate_persisted_config(&cfg).expect("default seeded config must be valid");
    cfg
}

pub(crate) fn normalize_persisted_config(cfg: &mut Config) -> Result<(), String> {
    clamp_config_values(cfg);
    validate_persisted_config(cfg)
}

pub(crate) fn apply_persisted_patch(
    cfg: &Config,
    patch: &serde_json::Value,
) -> Result<Config, String> {
    let Some(_) = patch.as_object() else {
        return Err("config patch must be a JSON object".to_string());
    };
    let mut merged = serde_json::to_value(cfg)
        .map_err(|err| format!("Unable to serialize current config for patching: {}", err))?;
    merge_json_value(&mut merged, patch)?;
    let mut updated = serde_json::from_value::<Config>(merged)
        .map_err(|err| format!("Invalid config payload: {}", err))?;
    normalize_persisted_config(&mut updated)?;
    Ok(updated)
}

fn merge_json_value(target: &mut serde_json::Value, patch: &serde_json::Value) -> Result<(), String> {
    match (target, patch) {
        (serde_json::Value::Object(target_object), serde_json::Value::Object(patch_object)) => {
            for (key, patch_value) in patch_object {
                match target_object.get_mut(key) {
                    Some(target_value)
                        if target_value.is_object() && patch_value.is_object() =>
                    {
                        merge_json_value(target_value, patch_value)?;
                    }
                    _ => {
                        target_object.insert(key.clone(), patch_value.clone());
                    }
                }
            }
            Ok(())
        }
        _ => Err("config patch must be a JSON object".to_string()),
    }
}

pub(crate) fn validate_persisted_config(cfg: &Config) -> Result<(), String> {
    validate_verified_identity_config(&cfg.verified_identity)
}

fn validate_verified_identity_config(cfg: &VerifiedIdentityConfig) -> Result<(), String> {
    if cfg.enabled && !cfg.native_web_bot_auth_enabled && !cfg.provider_assertions_enabled {
        return Err(
            "verified_identity.enabled=true requires at least one verifier path: native_web_bot_auth_enabled or provider_assertions_enabled"
                .to_string(),
        );
    }
    if !(VERIFIED_IDENTITY_REPLAY_WINDOW_SECONDS_MIN
        ..=VERIFIED_IDENTITY_REPLAY_WINDOW_SECONDS_MAX)
        .contains(&cfg.replay_window_seconds)
    {
        return Err(format!(
            "verified_identity.replay_window_seconds out of range ({}-{})",
            VERIFIED_IDENTITY_REPLAY_WINDOW_SECONDS_MIN,
            VERIFIED_IDENTITY_REPLAY_WINDOW_SECONDS_MAX
        ));
    }
    if cfg.clock_skew_seconds > VERIFIED_IDENTITY_CLOCK_SKEW_SECONDS_MAX {
        return Err(format!(
            "verified_identity.clock_skew_seconds out of range (0-{})",
            VERIFIED_IDENTITY_CLOCK_SKEW_SECONDS_MAX
        ));
    }
    if cfg.clock_skew_seconds > cfg.replay_window_seconds {
        return Err(
            "verified_identity.clock_skew_seconds must be <= verified_identity.replay_window_seconds"
                .to_string(),
        );
    }
    if !(VERIFIED_IDENTITY_DIRECTORY_CACHE_TTL_SECONDS_MIN
        ..=VERIFIED_IDENTITY_DIRECTORY_CACHE_TTL_SECONDS_MAX)
        .contains(&cfg.directory_cache_ttl_seconds)
    {
        return Err(format!(
            "verified_identity.directory_cache_ttl_seconds out of range ({}-{})",
            VERIFIED_IDENTITY_DIRECTORY_CACHE_TTL_SECONDS_MIN,
            VERIFIED_IDENTITY_DIRECTORY_CACHE_TTL_SECONDS_MAX
        ));
    }
    if !(VERIFIED_IDENTITY_DIRECTORY_FRESHNESS_REQUIREMENT_SECONDS_MIN
        ..=VERIFIED_IDENTITY_DIRECTORY_FRESHNESS_REQUIREMENT_SECONDS_MAX)
        .contains(&cfg.directory_freshness_requirement_seconds)
    {
        return Err(format!(
            "verified_identity.directory_freshness_requirement_seconds out of range ({}-{})",
            VERIFIED_IDENTITY_DIRECTORY_FRESHNESS_REQUIREMENT_SECONDS_MIN,
            VERIFIED_IDENTITY_DIRECTORY_FRESHNESS_REQUIREMENT_SECONDS_MAX
        ));
    }

    let mut profile_ids = HashSet::new();
    for (index, profile) in cfg.service_profiles.iter().enumerate() {
        let profile_id = profile.profile_id.trim();
        if profile_id.is_empty() {
            return Err(format!(
                "verified_identity.service_profiles[{}].profile_id must not be empty",
                index
            ));
        }
        if !profile_ids.insert(profile_id.to_string()) {
            return Err(format!(
                "verified_identity.service_profiles[{}].profile_id duplicates {}",
                index, profile_id
            ));
        }
    }

    let mut category_defaults = HashSet::new();
    for (index, category_default) in cfg.category_defaults.iter().enumerate() {
        if !category_defaults.insert(category_default.category) {
            return Err(format!(
                "verified_identity.category_defaults[{}].category duplicates {}",
                index,
                category_default.category.as_str()
            ));
        }
        validate_verified_identity_policy_action(
            format!("verified_identity.category_defaults[{}].action", index).as_str(),
            &category_default.action,
            &profile_ids,
        )?;
    }

    let mut policy_ids = HashSet::new();
    for (index, policy) in cfg.named_policies.iter().enumerate() {
        let policy_id = policy.policy_id.trim();
        if policy_id.is_empty() {
            return Err(format!(
                "verified_identity.named_policies[{}].policy_id must not be empty",
                index
            ));
        }
        if !policy_ids.insert(policy_id.to_string()) {
            return Err(format!(
                "verified_identity.named_policies[{}].policy_id duplicates {}",
                index, policy_id
            ));
        }
        validate_verified_identity_policy_matcher(
            format!("verified_identity.named_policies[{}].matcher", index).as_str(),
            &policy.matcher,
        )?;
        validate_verified_identity_policy_action(
            format!("verified_identity.named_policies[{}].action", index).as_str(),
            &policy.action,
            &profile_ids,
        )?;
    }

    Ok(())
}

fn validate_verified_identity_policy_matcher(
    field: &str,
    matcher: &crate::bot_identity::policy::IdentityPolicyMatcher,
) -> Result<(), String> {
    if matcher.is_empty() {
        return Err(format!(
            "{} must match at least one of: scheme, stable_identity, operator, category, or path_prefixes",
            field
        ));
    }
    if matcher
        .stable_identity
        .as_deref()
        .map(str::trim)
        .map(|value| value.is_empty())
        .unwrap_or(false)
    {
        return Err(format!("{}.stable_identity must not be empty", field));
    }
    if matcher
        .operator
        .as_deref()
        .map(str::trim)
        .map(|value| value.is_empty())
        .unwrap_or(false)
    {
        return Err(format!("{}.operator must not be empty", field));
    }
    for (index, prefix) in matcher.path_prefixes.iter().enumerate() {
        if !prefix.starts_with('/') {
            return Err(format!(
                "{}.path_prefixes[{}] must start with /",
                field, index
            ));
        }
    }
    Ok(())
}

fn validate_verified_identity_policy_action(
    field: &str,
    action: &crate::bot_identity::policy::IdentityPolicyAction,
    profile_ids: &HashSet<String>,
) -> Result<(), String> {
    let Some(profile_id) = action.referenced_service_profile_id() else {
        return Ok(());
    };
    let trimmed = profile_id.trim();
    if trimmed.is_empty() {
        return Err(format!("{}.value must not be empty", field));
    }
    if !profile_ids.contains(trimmed) {
        return Err(format!(
            "{} references unknown service profile {}",
            field, trimmed
        ));
    }
    Ok(())
}

static RUNTIME_CONFIG_CACHE: Lazy<Mutex<HashMap<String, CachedConfig>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static RUNTIME_EPHEMERAL_FLAGS: Lazy<Mutex<HashMap<String, RuntimeEphemeralFlags>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn runtime_ephemeral_flags(site_id: &str) -> RuntimeEphemeralFlags {
    let cache = RUNTIME_EPHEMERAL_FLAGS.lock().unwrap();
    cache.get(site_id).copied().unwrap_or_default()
}

fn runtime_env_shadow_mode_override() -> Option<bool> {
    runtime_var_raw_optional("SHUMA_SHADOW_MODE")
        .and_then(|value| parse_bool_like(value.as_str()))
}

#[cfg(test)]
pub fn set_runtime_shadow_mode_override(site_id: &str, enabled: bool) {
    let mut cache = RUNTIME_EPHEMERAL_FLAGS.lock().unwrap();
    let entry = cache.entry(site_id.to_string()).or_default();
    entry.shadow_mode_override = Some(enabled);
}

pub fn apply_runtime_ephemeral_overrides(site_id: &str, cfg: &mut Config) {
    let overrides = runtime_ephemeral_flags(site_id);
    if let Some(value) = overrides
        .shadow_mode_override
        .or_else(runtime_env_shadow_mode_override)
    {
        cfg.shadow_mode = value;
    }
}

#[cfg(not(test))]
fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn load_cached_with_now(
    store: &impl KeyValueStore,
    site_id: &str,
    now: u64,
    ttl_seconds: u64,
) -> Result<Config, ConfigLoadError> {
    {
        let cache = RUNTIME_CONFIG_CACHE.lock().unwrap();
        if let Some(entry) = cache.get(site_id) {
            let age = now.saturating_sub(entry.loaded_at);
            if age <= ttl_seconds {
                let mut effective = entry.config.clone();
                apply_runtime_ephemeral_overrides(site_id, &mut effective);
                return Ok(effective);
            }
        }
    }

    let config = Config::load(store, site_id)?;
    let mut cache = RUNTIME_CONFIG_CACHE.lock().unwrap();
    cache.insert(
        site_id.to_string(),
        CachedConfig {
            loaded_at: now,
            config: config.clone(),
        },
    );
    let mut effective = config;
    apply_runtime_ephemeral_overrides(site_id, &mut effective);
    Ok(effective)
}

pub fn load_runtime_cached(
    store: &impl KeyValueStore,
    site_id: &str,
) -> Result<Config, ConfigLoadError> {
    #[cfg(test)]
    {
        let mut cfg = Config::load(store, site_id)?;
        apply_runtime_ephemeral_overrides(site_id, &mut cfg);
        return Ok(cfg);
    }
    #[cfg(not(test))]
    {
        load_cached_with_now(store, site_id, now_ts(), CONFIG_CACHE_TTL_SECONDS)
    }
}

pub fn invalidate_runtime_cache(site_id: &str) {
    let mut cache = RUNTIME_CONFIG_CACHE.lock().unwrap();
    cache.remove(site_id);
}

#[cfg(test)]
pub(crate) fn clear_runtime_cache_for_tests() {
    let mut cache = RUNTIME_CONFIG_CACHE.lock().unwrap();
    cache.clear();
    let mut ephemeral = RUNTIME_EPHEMERAL_FLAGS.lock().unwrap();
    ephemeral.clear();
}

#[cfg(test)]
pub(crate) fn load_runtime_cached_for_tests(
    store: &impl KeyValueStore,
    site_id: &str,
    now: u64,
    ttl_seconds: u64,
) -> Result<Config, ConfigLoadError> {
    load_cached_with_now(store, site_id, now, ttl_seconds)
}

static DEFAULTS_MAP: Lazy<Result<HashMap<String, String>, String>> =
    Lazy::new(|| parse_defaults_env_map(DEFAULTS_ENV_TEXT));

static DEFAULT_CONFIG: Lazy<Config> = Lazy::new(|| {
    let mut cfg = Config {
        ban_duration: defaults_u64("SHUMA_BAN_DURATION"),
        ban_durations: BanDurations {
            honeypot: defaults_u64("SHUMA_BAN_DURATION_HONEYPOT"),
            ip_range_honeypot: defaults_u64("SHUMA_BAN_DURATION_IP_RANGE_HONEYPOT"),
            maze_crawler: defaults_u64("SHUMA_BAN_DURATION_MAZE_CRAWLER"),
            rate_limit: defaults_u64("SHUMA_BAN_DURATION_RATE_LIMIT"),
            admin: defaults_u64("SHUMA_BAN_DURATION_ADMIN"),
            cdp: defaults_u64("SHUMA_BAN_DURATION_CDP"),
            edge_fingerprint: defaults_u64("SHUMA_BAN_DURATION_EDGE_FINGERPRINT"),
            tarpit_persistence: defaults_u64("SHUMA_BAN_DURATION_TARPIT_PERSISTENCE"),
            not_a_bot_abuse: defaults_u64("SHUMA_BAN_DURATION_NOT_A_BOT_ABUSE"),
            challenge_puzzle_abuse: defaults_u64("SHUMA_BAN_DURATION_CHALLENGE_PUZZLE_ABUSE"),
        },
        rate_limit: defaults_u32("SHUMA_RATE_LIMIT"),
        honeypot_enabled: defaults_bool("SHUMA_HONEYPOT_ENABLED"),
        honeypots: defaults_string_list("SHUMA_HONEYPOTS"),
        browser_policy_enabled: defaults_bool("SHUMA_BROWSER_POLICY_ENABLED"),
        browser_block: defaults_browser_rules("SHUMA_BROWSER_BLOCK"),
        browser_allowlist: defaults_browser_rules("SHUMA_BROWSER_ALLOWLIST"),
        geo_risk: defaults_country_list("SHUMA_GEO_RISK_COUNTRIES"),
        geo_allow: defaults_country_list("SHUMA_GEO_ALLOW_COUNTRIES"),
        geo_challenge: defaults_country_list("SHUMA_GEO_CHALLENGE_COUNTRIES"),
        geo_maze: defaults_country_list("SHUMA_GEO_MAZE_COUNTRIES"),
        geo_block: defaults_country_list("SHUMA_GEO_BLOCK_COUNTRIES"),
        geo_edge_headers_enabled: defaults_bool("SHUMA_GEO_EDGE_HEADERS_ENABLED"),
        bypass_allowlists_enabled: defaults_bool("SHUMA_BYPASS_ALLOWLISTS_ENABLED"),
        allowlist: defaults_string_list("SHUMA_ALLOWLIST"),
        path_allowlist_enabled: defaults_bool("SHUMA_PATH_ALLOWLIST_ENABLED"),
        path_allowlist: defaults_string_list("SHUMA_PATH_ALLOWLIST"),
        ip_range_policy_mode: default_ip_range_policy_mode(),
        ip_range_emergency_allowlist: defaults_string_list("SHUMA_IP_RANGE_EMERGENCY_ALLOWLIST"),
        ip_range_custom_rules: defaults_json("SHUMA_IP_RANGE_CUSTOM_RULES"),
        ip_range_suggestions_min_observations: default_ip_range_suggestions_min_observations(),
        ip_range_suggestions_min_bot_events: default_ip_range_suggestions_min_bot_events(),
        ip_range_suggestions_min_confidence_percent:
            default_ip_range_suggestions_min_confidence_percent(),
        ip_range_suggestions_low_collateral_percent:
            default_ip_range_suggestions_low_collateral_percent(),
        ip_range_suggestions_high_collateral_percent:
            default_ip_range_suggestions_high_collateral_percent(),
        ip_range_suggestions_ipv4_min_prefix_len:
            default_ip_range_suggestions_ipv4_min_prefix_len(),
        ip_range_suggestions_ipv6_min_prefix_len:
            default_ip_range_suggestions_ipv6_min_prefix_len(),
        ip_range_suggestions_likely_human_sample_percent:
            default_ip_range_suggestions_likely_human_sample_percent(),
        shadow_mode: defaults_bool("SHUMA_SHADOW_MODE"),
        adversary_sim_enabled: defaults_bool("SHUMA_ADVERSARY_SIM_ENABLED"),
        adversary_sim_duration_seconds: default_adversary_sim_duration_seconds(),
        maze_enabled: defaults_bool("SHUMA_MAZE_ENABLED"),
        tarpit_enabled: defaults_bool("SHUMA_TARPIT_ENABLED"),
        tarpit_progress_token_ttl_seconds: defaults_u64("SHUMA_TARPIT_PROGRESS_TOKEN_TTL_SECONDS"),
        tarpit_progress_replay_ttl_seconds: defaults_u64(
            "SHUMA_TARPIT_PROGRESS_REPLAY_TTL_SECONDS",
        ),
        tarpit_hashcash_min_difficulty: defaults_u8("SHUMA_TARPIT_HASHCASH_MIN_DIFFICULTY"),
        tarpit_hashcash_max_difficulty: defaults_u8("SHUMA_TARPIT_HASHCASH_MAX_DIFFICULTY"),
        tarpit_hashcash_base_difficulty: defaults_u8("SHUMA_TARPIT_HASHCASH_BASE_DIFFICULTY"),
        tarpit_hashcash_adaptive: defaults_bool("SHUMA_TARPIT_HASHCASH_ADAPTIVE"),
        tarpit_step_chunk_base_bytes: defaults_u32("SHUMA_TARPIT_STEP_CHUNK_BASE_BYTES"),
        tarpit_step_chunk_max_bytes: defaults_u32("SHUMA_TARPIT_STEP_CHUNK_MAX_BYTES"),
        tarpit_step_jitter_percent: defaults_u8("SHUMA_TARPIT_STEP_JITTER_PERCENT"),
        tarpit_shard_rotation_enabled: defaults_bool("SHUMA_TARPIT_SHARD_ROTATION_ENABLED"),
        tarpit_egress_window_seconds: defaults_u64("SHUMA_TARPIT_EGRESS_WINDOW_SECONDS"),
        tarpit_egress_global_bytes_per_window: defaults_u64(
            "SHUMA_TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW",
        ),
        tarpit_egress_per_ip_bucket_bytes_per_window: defaults_u64(
            "SHUMA_TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW",
        ),
        tarpit_egress_per_flow_max_bytes: defaults_u64("SHUMA_TARPIT_EGRESS_PER_FLOW_MAX_BYTES"),
        tarpit_egress_per_flow_max_duration_seconds: defaults_u64(
            "SHUMA_TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS",
        ),
        tarpit_max_concurrent_global: defaults_u32("SHUMA_TARPIT_MAX_CONCURRENT_GLOBAL"),
        tarpit_max_concurrent_per_ip_bucket: defaults_u32(
            "SHUMA_TARPIT_MAX_CONCURRENT_PER_IP_BUCKET",
        ),
        tarpit_fallback_action: default_tarpit_fallback_action(),
        maze_auto_ban: defaults_bool("SHUMA_MAZE_AUTO_BAN"),
        maze_auto_ban_threshold: defaults_u32("SHUMA_MAZE_AUTO_BAN_THRESHOLD"),
        maze_rollout_phase: default_maze_rollout_phase(),
        maze_token_ttl_seconds: defaults_u64("SHUMA_MAZE_TOKEN_TTL_SECONDS"),
        maze_token_max_depth: defaults_u16("SHUMA_MAZE_TOKEN_MAX_DEPTH"),
        maze_token_branch_budget: defaults_u8("SHUMA_MAZE_TOKEN_BRANCH_BUDGET"),
        maze_replay_ttl_seconds: defaults_u64("SHUMA_MAZE_REPLAY_TTL_SECONDS"),
        maze_entropy_window_seconds: defaults_u64("SHUMA_MAZE_ENTROPY_WINDOW_SECONDS"),
        maze_client_expansion_enabled: defaults_bool("SHUMA_MAZE_CLIENT_EXPANSION_ENABLED"),
        maze_checkpoint_every_nodes: defaults_u64("SHUMA_MAZE_CHECKPOINT_EVERY_NODES"),
        maze_checkpoint_every_ms: defaults_u64("SHUMA_MAZE_CHECKPOINT_EVERY_MS"),
        maze_step_ahead_max: defaults_u64("SHUMA_MAZE_STEP_AHEAD_MAX"),
        maze_no_js_fallback_max_depth: defaults_u16("SHUMA_MAZE_NO_JS_FALLBACK_MAX_DEPTH"),
        maze_micro_pow_enabled: defaults_bool("SHUMA_MAZE_MICRO_POW_ENABLED"),
        maze_micro_pow_depth_start: defaults_u16("SHUMA_MAZE_MICRO_POW_DEPTH_START"),
        maze_micro_pow_base_difficulty: defaults_u8("SHUMA_MAZE_MICRO_POW_BASE_DIFFICULTY"),
        maze_max_concurrent_global: defaults_u32("SHUMA_MAZE_MAX_CONCURRENT_GLOBAL"),
        maze_max_concurrent_per_ip_bucket: defaults_u32("SHUMA_MAZE_MAX_CONCURRENT_PER_IP_BUCKET"),
        maze_max_response_bytes: defaults_u32("SHUMA_MAZE_MAX_RESPONSE_BYTES"),
        maze_max_response_duration_ms: defaults_u64("SHUMA_MAZE_MAX_RESPONSE_DURATION_MS"),
        maze_server_visible_links: defaults_u32("SHUMA_MAZE_SERVER_VISIBLE_LINKS"),
        maze_max_links: defaults_u32("SHUMA_MAZE_MAX_LINKS"),
        maze_max_paragraphs: defaults_u32("SHUMA_MAZE_MAX_PARAGRAPHS"),
        maze_path_entropy_segment_len: defaults_u8("SHUMA_MAZE_PATH_ENTROPY_SEGMENT_LEN"),
        maze_covert_decoys_enabled: defaults_bool("SHUMA_MAZE_COVERT_DECOYS_ENABLED"),
        maze_seed_provider: default_maze_seed_provider(),
        maze_seed_refresh_interval_seconds: defaults_u64(
            "SHUMA_MAZE_SEED_REFRESH_INTERVAL_SECONDS",
        ),
        maze_seed_refresh_rate_limit_per_hour: defaults_u32(
            "SHUMA_MAZE_SEED_REFRESH_RATE_LIMIT_PER_HOUR",
        ),
        maze_seed_refresh_max_sources: defaults_u32("SHUMA_MAZE_SEED_REFRESH_MAX_SOURCES"),
        maze_seed_metadata_only: defaults_bool("SHUMA_MAZE_SEED_METADATA_ONLY"),
        robots_enabled: defaults_bool("SHUMA_ROBOTS_ENABLED"),
        robots_block_ai_training: defaults_bool("SHUMA_ROBOTS_BLOCK_AI_TRAINING"),
        robots_block_ai_search: defaults_bool("SHUMA_ROBOTS_BLOCK_AI_SEARCH"),
        robots_allow_search_engines: defaults_bool("SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES"),
        robots_crawl_delay: defaults_u32("SHUMA_ROBOTS_CRAWL_DELAY"),
        cdp_detection_enabled: defaults_bool("SHUMA_CDP_DETECTION_ENABLED"),
        cdp_auto_ban: defaults_bool("SHUMA_CDP_AUTO_BAN"),
        cdp_detection_threshold: defaults_f32("SHUMA_CDP_DETECTION_THRESHOLD"),
        cdp_probe_family: default_cdp_probe_family(),
        cdp_probe_rollout_percent: defaults_u8("SHUMA_CDP_PROBE_ROLLOUT_PERCENT"),
        fingerprint_signal_enabled: defaults_bool("SHUMA_FINGERPRINT_SIGNAL_ENABLED"),
        fingerprint_state_ttl_seconds: defaults_u64("SHUMA_FINGERPRINT_STATE_TTL_SECONDS"),
        fingerprint_flow_window_seconds: defaults_u64("SHUMA_FINGERPRINT_FLOW_WINDOW_SECONDS"),
        fingerprint_flow_violation_threshold: defaults_u8(
            "SHUMA_FINGERPRINT_FLOW_VIOLATION_THRESHOLD",
        ),
        fingerprint_pseudonymize: defaults_bool("SHUMA_FINGERPRINT_PSEUDONYMIZE"),
        fingerprint_entropy_budget: defaults_u8("SHUMA_FINGERPRINT_ENTROPY_BUDGET"),
        fingerprint_family_cap_header_runtime: defaults_u8(
            "SHUMA_FINGERPRINT_FAMILY_CAP_HEADER_RUNTIME",
        ),
        fingerprint_family_cap_transport: defaults_u8("SHUMA_FINGERPRINT_FAMILY_CAP_TRANSPORT"),
        fingerprint_family_cap_temporal: defaults_u8("SHUMA_FINGERPRINT_FAMILY_CAP_TEMPORAL"),
        fingerprint_family_cap_persistence: defaults_u8(
            "SHUMA_FINGERPRINT_FAMILY_CAP_PERSISTENCE",
        ),
        fingerprint_family_cap_behavior: defaults_u8("SHUMA_FINGERPRINT_FAMILY_CAP_BEHAVIOR"),
        js_required_enforced: defaults_bool("SHUMA_JS_REQUIRED_ENFORCED"),
        pow_enabled: defaults_bool("SHUMA_POW_ENABLED"),
        pow_difficulty: defaults_u8("SHUMA_POW_DIFFICULTY"),
        pow_ttl_seconds: defaults_u64("SHUMA_POW_TTL_SECONDS"),
        challenge_puzzle_enabled: defaults_bool("SHUMA_CHALLENGE_PUZZLE_ENABLED"),
        challenge_puzzle_transform_count: defaults_u8("SHUMA_CHALLENGE_PUZZLE_TRANSFORM_COUNT"),
        challenge_puzzle_seed_ttl_seconds: defaults_u64("SHUMA_CHALLENGE_PUZZLE_SEED_TTL_SECONDS"),
        challenge_puzzle_attempt_limit_per_window: defaults_u32(
            "SHUMA_CHALLENGE_PUZZLE_ATTEMPT_LIMIT_PER_WINDOW",
        ),
        challenge_puzzle_attempt_window_seconds: defaults_u64(
            "SHUMA_CHALLENGE_PUZZLE_ATTEMPT_WINDOW_SECONDS",
        ),
        challenge_puzzle_risk_threshold: defaults_u8("SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD"),
        not_a_bot_enabled: defaults_bool("SHUMA_NOT_A_BOT_ENABLED"),
        not_a_bot_risk_threshold: defaults_u8("SHUMA_NOT_A_BOT_RISK_THRESHOLD"),
        not_a_bot_pass_score: defaults_u8("SHUMA_NOT_A_BOT_PASS_SCORE"),
        not_a_bot_fail_score: defaults_u8("SHUMA_NOT_A_BOT_FAIL_SCORE"),
        not_a_bot_nonce_ttl_seconds: defaults_u64("SHUMA_NOT_A_BOT_NONCE_TTL_SECONDS"),
        not_a_bot_marker_ttl_seconds: defaults_u64("SHUMA_NOT_A_BOT_MARKER_TTL_SECONDS"),
        not_a_bot_attempt_limit_per_window: defaults_u32("SHUMA_NOT_A_BOT_ATTEMPT_LIMIT_PER_WINDOW"),
        not_a_bot_attempt_window_seconds: defaults_u64("SHUMA_NOT_A_BOT_ATTEMPT_WINDOW_SECONDS"),
        botness_maze_threshold: defaults_u8("SHUMA_BOTNESS_MAZE_THRESHOLD"),
        botness_weights: BotnessWeights {
            js_required: defaults_u8("SHUMA_BOTNESS_WEIGHT_JS_REQUIRED"),
            geo_risk: defaults_u8("SHUMA_BOTNESS_WEIGHT_GEO_RISK"),
            rate_medium: defaults_u8("SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM"),
            rate_high: defaults_u8("SHUMA_BOTNESS_WEIGHT_RATE_HIGH"),
            maze_behavior: defaults_u8("SHUMA_BOTNESS_WEIGHT_MAZE_BEHAVIOR"),
        },
        defence_modes: DefenceModes::default(),
        provider_backends: ProviderBackends::default(),
        edge_integration_mode: default_edge_integration_mode(),
        verified_identity: VerifiedIdentityConfig::default(),
    };
    clamp_config_values(&mut cfg);
    validate_persisted_config(&cfg).expect("config defaults must be valid");
    cfg
});

static ENV_VALIDATION_RESULT: Lazy<Result<(), String>> = Lazy::new(validate_env_only_impl);

pub fn defaults() -> &'static Config {
    &DEFAULT_CONFIG
}

pub fn serialize_persisted_kv_config(cfg: &Config) -> Result<Vec<u8>, serde_json::Error> {
    let mut value = serde_json::to_value(cfg)?;
    if let Some(obj) = value.as_object_mut() {
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
    }
    serde_json::to_vec(&value)
}

pub fn validate_env_only_once() -> Result<(), String> {
    if cfg!(test) {
        if validate_env_in_tests_enabled() {
            return validate_env_only_impl();
        }
        return Ok(());
    }
    match &*ENV_VALIDATION_RESULT {
        Ok(()) => Ok(()),
        Err(msg) => Err(msg.clone()),
    }
}

// Env-only runtime guardrails. Keep this list aligned with:
// - config/defaults.env (env-only section),
// - Makefile Spin env injection lists,
// - docs/configuration.md env-only reference table.
fn validate_env_only_impl() -> Result<(), String> {
    validate_non_empty("SHUMA_API_KEY")?;
    validate_non_empty("SHUMA_JS_SECRET")?;
    validate_non_empty("SHUMA_FORWARDED_IP_SECRET")?;
    validate_u64_var("SHUMA_EVENT_LOG_RETENTION_HOURS")?;
    validate_optional_u64_var("SHUMA_MONITORING_RETENTION_HOURS")?;
    validate_optional_u64_var("SHUMA_MONITORING_ROLLUP_RETENTION_HOURS")?;

    validate_bool_like_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED")?;
    validate_bool_like_var("SHUMA_KV_STORE_FAIL_OPEN")?;
    validate_bool_like_var("SHUMA_ENFORCE_HTTPS")?;
    validate_bool_like_var("SHUMA_DEBUG_HEADERS")?;
    validate_optional_runtime_environment_var("SHUMA_RUNTIME_ENV")?;
    validate_optional_bool_like_var("SHUMA_LOCAL_PROD_DIRECT_MODE")?;
    validate_optional_bool_like_var("SHUMA_ADVERSARY_SIM_AVAILABLE")?;
    validate_optional_secret_var("SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET")?;
    validate_optional_secret_var("SHUMA_SIM_TELEMETRY_SECRET")?;
    validate_optional_model_id_var("SHUMA_FRONTIER_OPENAI_MODEL")?;
    validate_optional_model_id_var("SHUMA_FRONTIER_ANTHROPIC_MODEL")?;
    validate_optional_model_id_var("SHUMA_FRONTIER_GOOGLE_MODEL")?;
    validate_optional_model_id_var("SHUMA_FRONTIER_XAI_MODEL")?;
    validate_optional_bool_like_var("SHUMA_ENTERPRISE_MULTI_INSTANCE")?;
    validate_optional_bool_like_var("SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED")?;
    validate_optional_redis_url_var("SHUMA_RATE_LIMITER_REDIS_URL")?;
    validate_optional_redis_url_var("SHUMA_BAN_STORE_REDIS_URL")?;
    validate_optional_rate_limiter_outage_mode_var("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN")?;
    validate_optional_rate_limiter_outage_mode_var("SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH")?;
    validate_optional_ban_store_outage_mode_var("SHUMA_BAN_STORE_OUTAGE_MODE")?;
    validate_gateway_contract_env()?;

    Ok(())
}

fn validate_gateway_contract_env() -> Result<(), String> {
    let profile_raw = runtime_var_raw_optional("SHUMA_GATEWAY_DEPLOYMENT_PROFILE")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_DEPLOYMENT_PROFILE"));
    let profile = parse_gateway_deployment_profile(profile_raw.as_str()).ok_or_else(|| {
        format!(
            "Invalid gateway deployment profile env var SHUMA_GATEWAY_DEPLOYMENT_PROFILE={} (expected shared-server or edge-fermyon)",
            profile_raw
        )
    })?;

    let origin_auth_mode_raw = runtime_var_raw_optional("SHUMA_GATEWAY_ORIGIN_AUTH_MODE")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_ORIGIN_AUTH_MODE"));
    let origin_auth_mode =
        parse_gateway_origin_auth_mode(origin_auth_mode_raw.as_str()).ok_or_else(|| {
            format!(
                "Invalid gateway origin auth mode env var SHUMA_GATEWAY_ORIGIN_AUTH_MODE={} (expected network_only or signed_header)",
                origin_auth_mode_raw
            )
        })?;

    let allow_insecure_http_local = parse_env_bool_optional(
        "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL",
        defaults_bool("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL"),
    )
    .ok_or_else(|| {
        invalid_boolean_env(
            "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL",
            defaults_raw("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL").as_str(),
        )
    })?;
    let allow_insecure_http_special_use_ips = parse_env_bool_optional(
        "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS",
        defaults_bool("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS"),
    )
    .ok_or_else(|| {
        invalid_boolean_env(
            "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS",
            defaults_raw("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS").as_str(),
        )
    })?;
    let tls_strict = parse_env_bool_optional(
        "SHUMA_GATEWAY_TLS_STRICT",
        defaults_bool("SHUMA_GATEWAY_TLS_STRICT"),
    )
    .ok_or_else(|| {
        invalid_boolean_env(
            "SHUMA_GATEWAY_TLS_STRICT",
            defaults_raw("SHUMA_GATEWAY_TLS_STRICT").as_str(),
        )
    })?;

    if !tls_strict {
        return Err(
            "Invalid gateway TLS posture: SHUMA_GATEWAY_TLS_STRICT must be true; insecure upstream TLS modes are not supported"
                .to_string(),
        );
    }

    let loop_max_hops = env_u8_optional(
        "SHUMA_GATEWAY_LOOP_MAX_HOPS",
        defaults_u8("SHUMA_GATEWAY_LOOP_MAX_HOPS"),
    )
    .ok_or_else(|| invalid_integer_env("SHUMA_GATEWAY_LOOP_MAX_HOPS"))?;
    if !(GATEWAY_LOOP_MAX_HOPS_MIN..=GATEWAY_LOOP_MAX_HOPS_MAX).contains(&loop_max_hops) {
        return Err(format!(
            "Invalid gateway loop guard env var SHUMA_GATEWAY_LOOP_MAX_HOPS={} (expected {}..={})",
            loop_max_hops, GATEWAY_LOOP_MAX_HOPS_MIN, GATEWAY_LOOP_MAX_HOPS_MAX
        ));
    }

    let origin_auth_max_age_days = env_u32_optional(
        "SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS",
        defaults_u32("SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS"),
    )
    .ok_or_else(|| invalid_integer_env("SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS"))?;
    if !(GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS_MIN..=GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS_MAX)
        .contains(&origin_auth_max_age_days)
    {
        return Err(format!(
            "Invalid gateway origin auth max-age env var SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS={} (expected {}..={})",
            origin_auth_max_age_days,
            GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS_MIN,
            GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS_MAX
        ));
    }

    let origin_auth_rotation_overlap_days = env_u32_optional(
        "SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS",
        defaults_u32("SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS"),
    )
    .ok_or_else(|| invalid_integer_env("SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS"))?;
    if !(GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS_MIN
        ..=GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS_MAX)
        .contains(&origin_auth_rotation_overlap_days)
    {
        return Err(format!(
            "Invalid gateway origin auth overlap env var SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS={} (expected {}..={})",
            origin_auth_rotation_overlap_days,
            GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS_MIN,
            GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS_MAX
        ));
    }
    if origin_auth_rotation_overlap_days >= origin_auth_max_age_days {
        return Err(
            "Invalid gateway origin auth lifecycle: SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS must be less than SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS"
                .to_string(),
        );
    }

    let public_authorities_raw = env_trimmed_optional("SHUMA_GATEWAY_PUBLIC_AUTHORITIES")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_PUBLIC_AUTHORITIES"));
    let public_authorities = parse_gateway_authority_list(public_authorities_raw.as_str())
        .map_err(|reason| {
            format!(
                "Invalid gateway public authorities env var SHUMA_GATEWAY_PUBLIC_AUTHORITIES={} ({})",
                public_authorities_raw, reason
            )
        })?;

    let special_use_allowlist_raw =
        env_trimmed_optional("SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST")
            .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST"));
    let special_use_allowlist = parse_gateway_ip_allowlist(special_use_allowlist_raw.as_str())
        .map_err(|reason| {
            format!(
                "Invalid gateway special-use allowlist env var SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST={} ({})",
                special_use_allowlist_raw, reason
            )
        })?;

    let runtime_env = runtime_environment();
    let local_prod_direct_mode = local_prod_direct_mode();
    let upstream_origin_raw = env_trimmed_optional("SHUMA_GATEWAY_UPSTREAM_ORIGIN")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_UPSTREAM_ORIGIN"));
    if upstream_origin_raw.is_empty() {
        if runtime_env.is_prod() {
            if local_prod_direct_mode {
                if profile.is_edge() {
                    return Err(
                        "Invalid gateway posture: SHUMA_LOCAL_PROD_DIRECT_MODE=true only supports SHUMA_GATEWAY_DEPLOYMENT_PROFILE=shared-server"
                            .to_string(),
                    );
                }
                return Ok(());
            }
            return Err(
                "Invalid gateway posture: SHUMA_GATEWAY_UPSTREAM_ORIGIN must be set when SHUMA_RUNTIME_ENV=runtime-prod"
                    .to_string(),
            );
        }
        return Ok(());
    }

    let upstream = parse_gateway_upstream_origin(upstream_origin_raw.as_str()).map_err(|reason| {
        format!(
            "Invalid gateway upstream origin env var SHUMA_GATEWAY_UPSTREAM_ORIGIN={} ({})",
            upstream_origin_raw, reason
        )
    })?;

    if profile.is_edge() && upstream.scheme != "https" {
        return Err(
            "Invalid gateway posture: SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon requires SHUMA_GATEWAY_UPSTREAM_ORIGIN to use https://"
                .to_string(),
        );
    }

    if upstream.scheme == "http" {
        if profile.is_edge() {
            return Err(
                "Invalid gateway posture: SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon does not allow insecure http:// upstream origins"
                    .to_string(),
            );
        }
        if !allow_insecure_http_local {
            return Err(
                "Invalid gateway posture: insecure http:// upstream requires SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL=true"
                    .to_string(),
            );
        }
        if !upstream.host_is_ip_literal {
            return Err(
                "Invalid gateway posture: insecure http:// upstream must use an IP-literal host (DNS hostnames are not allowed)"
                    .to_string(),
            );
        }
        let host_ip = upstream
            .host
            .parse::<IpAddr>()
            .map_err(|_| "Invalid gateway posture: insecure http:// upstream host must be a valid IP literal".to_string())?;
        if is_insecure_special_use_ip(host_ip) {
            if !(allow_insecure_http_special_use_ips && special_use_allowlist.contains(&host_ip)) {
                return Err(
                    "Invalid gateway posture: insecure http:// upstream in special-use range requires SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS=true and explicit SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST entry"
                        .to_string(),
                );
            }
        } else if !is_private_or_loopback_ip(host_ip) {
            return Err(
                "Invalid gateway posture: insecure http:// upstream must be loopback/private IP-literal and must not be public-routable"
                    .to_string(),
            );
        }
    }

    if public_authorities
        .iter()
        .any(|authority| authority == &upstream.authority())
    {
        return Err(format!(
            "Invalid gateway loop posture: SHUMA_GATEWAY_UPSTREAM_ORIGIN authority {} must not match SHUMA_GATEWAY_PUBLIC_AUTHORITIES",
            upstream.authority()
        ));
    }

    let origin_auth_header_name = env_trimmed_optional("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME"));
    let origin_auth_header_value = env_trimmed_optional("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE"));

    if profile.is_edge() && origin_auth_mode != GatewayOriginAuthMode::SignedHeader {
        return Err(
            "Invalid gateway origin auth posture: SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon requires SHUMA_GATEWAY_ORIGIN_AUTH_MODE=signed_header"
                .to_string(),
        );
    }

    match origin_auth_mode {
        GatewayOriginAuthMode::NetworkOnly => {
            if !origin_auth_header_name.is_empty() || !origin_auth_header_value.is_empty() {
                return Err(
                    "Invalid gateway origin auth posture: SHUMA_GATEWAY_ORIGIN_AUTH_MODE=network_only must not set SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME/SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE"
                        .to_string(),
                );
            }
        }
        GatewayOriginAuthMode::SignedHeader => {
            if origin_auth_header_name.is_empty() || origin_auth_header_value.is_empty() {
                return Err(
                    "Invalid gateway origin auth posture: SHUMA_GATEWAY_ORIGIN_AUTH_MODE=signed_header requires SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME and SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE"
                        .to_string(),
                );
            }
            if !valid_header_name(origin_auth_header_name.as_str()) {
                return Err(
                    "Invalid gateway origin auth posture: SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME must be a valid HTTP header token"
                        .to_string(),
                );
            }
            let lowered = origin_auth_header_name.to_ascii_lowercase();
            if matches!(
                lowered.as_str(),
                "authorization"
                    | "host"
                    | "forwarded"
                    | "x-forwarded-for"
                    | "x-forwarded-host"
                    | "x-forwarded-proto"
                    | "x-forwarded-port"
                    | "x-shuma-forwarded-secret"
                    | "connection"
                    | "transfer-encoding"
            ) {
                return Err(
                    "Invalid gateway origin auth posture: SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME must not be a transport/provenance or privileged header"
                        .to_string(),
                );
            }
        }
    }

    let origin_lock_confirmed = parse_env_bool_optional(
        "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED",
        defaults_bool("SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED"),
    )
    .ok_or_else(|| {
        invalid_boolean_env(
            "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED",
            defaults_raw("SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED").as_str(),
        )
    })?;
    if runtime_env.is_prod() && !origin_lock_confirmed {
        return Err(
            "Invalid gateway posture: SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED must be true when SHUMA_RUNTIME_ENV=runtime-prod"
                .to_string(),
        );
    }

    let route_collision_check_passed = parse_env_bool_optional(
        "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
        defaults_bool("SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED"),
    )
    .ok_or_else(|| {
        invalid_boolean_env(
            "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
            defaults_raw("SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED").as_str(),
        )
    })?;
    if runtime_env.is_prod() && !route_collision_check_passed {
        return Err(
            "Invalid gateway posture: SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED must be true when SHUMA_RUNTIME_ENV=runtime-prod"
                .to_string(),
        );
    }

    if profile.is_edge() {
        let edge_cron_secret = env_trimmed_optional("SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET")
            .unwrap_or_else(|| defaults_raw("SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET"));
        if edge_cron_secret.is_empty() {
            return Err(
                "Invalid adversary-sim edge posture: SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon requires SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET"
                    .to_string(),
            );
        }
    }

    Ok(())
}

fn validate_env_in_tests_enabled() -> bool {
    if !cfg!(test) {
        return false;
    }
    env::var("SHUMA_VALIDATE_ENV_IN_TESTS")
        .ok()
        .and_then(|v| parse_bool_like(v.as_str()))
        .unwrap_or(false)
}

fn validate_non_empty(name: &str) -> Result<(), String> {
    let value = runtime_var_raw_optional(name).ok_or_else(|| format!("Missing required env var {}", name))?;
    if value.trim().is_empty() {
        return Err(format!("Invalid empty env var {}", name));
    }
    Ok(())
}

fn validate_bool_like_var(name: &str) -> Result<(), String> {
    let value = runtime_var_raw_optional(name).ok_or_else(|| format!("Missing required env var {}", name))?;
    if parse_bool_like(&value).is_none() {
        return Err(format!("Invalid boolean env var {}={}", name, value));
    }
    Ok(())
}

fn validate_optional_bool_like_var(name: &str) -> Result<(), String> {
    let Some(value) = runtime_var_raw_optional(name) else {
        return Ok(());
    };
    if parse_bool_like(&value).is_none() {
        return Err(format!("Invalid boolean env var {}={}", name, value));
    }
    Ok(())
}

fn validate_optional_runtime_environment_var(name: &str) -> Result<(), String> {
    let Some(value) = runtime_var_raw_optional(name) else {
        return Ok(());
    };
    if value.trim().is_empty() {
        return Ok(());
    }
    if parse_runtime_environment(&value).is_none() {
        return Err(format!(
            "Invalid runtime environment env var {}={} (expected runtime-dev or runtime-prod)",
            name, value
        ));
    }
    Ok(())
}

fn validate_optional_redis_url_var(name: &str) -> Result<(), String> {
    let Some(value) = runtime_var_raw_optional(name) else {
        return Ok(());
    };
    if value.trim().is_empty() {
        return Ok(());
    }
    if parse_redis_url(&value).is_none() {
        return Err(format!(
            "Invalid Redis URL env var {}={} (expected redis://... or rediss://...)",
            name, value
        ));
    }
    Ok(())
}

fn validate_optional_rate_limiter_outage_mode_var(name: &str) -> Result<(), String> {
    let Some(value) = runtime_var_raw_optional(name) else {
        return Ok(());
    };
    if value.trim().is_empty() {
        return Ok(());
    }
    if parse_rate_limiter_outage_mode(&value).is_none() {
        return Err(format!(
            "Invalid outage mode env var {}={} (expected fallback_internal, fail_open, or fail_closed)",
            name, value
        ));
    }
    Ok(())
}

fn validate_optional_ban_store_outage_mode_var(name: &str) -> Result<(), String> {
    let Some(value) = runtime_var_raw_optional(name) else {
        return Ok(());
    };
    if value.trim().is_empty() {
        return Ok(());
    }
    if parse_ban_store_outage_mode(&value).is_none() {
        return Err(format!(
            "Invalid outage mode env var {}={} (expected fallback_internal, fail_open, or fail_closed)",
            name, value
        ));
    }
    Ok(())
}

fn validate_optional_model_id_var(name: &str) -> Result<(), String> {
    let Some(value) = runtime_var_raw_optional(name) else {
        return Ok(());
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(format!(
            "Invalid model id env var {}={} (model ids must not contain whitespace)",
            name, value
        ));
    }
    Ok(())
}

fn validate_optional_secret_var(name: &str) -> Result<(), String> {
    let Some(value) = runtime_var_raw_optional(name) else {
        return Ok(());
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(format!(
            "Invalid secret env var {} (must not contain newlines)",
            name
        ));
    }
    Ok(())
}

fn validate_u64_var(name: &str) -> Result<(), String> {
    let value = runtime_var_raw_optional(name).ok_or_else(|| format!("Missing required env var {}", name))?;
    if value.trim().parse::<u64>().is_err() {
        return Err(format!("Invalid integer env var {}={}", name, value));
    }
    Ok(())
}

fn validate_optional_u64_var(name: &str) -> Result<(), String> {
    let Some(value) = runtime_var_raw_optional(name) else {
        return Ok(());
    };
    if value.trim().parse::<u64>().is_err() {
        return Err(invalid_integer_env(name));
    }
    Ok(())
}

fn parse_env_bool_optional(name: &str, default: bool) -> Option<bool> {
    let raw = runtime_var_raw_optional(name)
        .unwrap_or_else(|| if default { "true" } else { "false" }.to_string());
    parse_bool_like(raw.as_str())
}

fn env_u8_optional(name: &str, default: u8) -> Option<u8> {
    runtime_var_raw_optional(name)
        .unwrap_or_else(|| default.to_string())
        .trim()
        .parse::<u8>()
        .ok()
}

fn env_u32_optional(name: &str, default: u32) -> Option<u32> {
    runtime_var_raw_optional(name)
        .unwrap_or_else(|| default.to_string())
        .trim()
        .parse::<u32>()
        .ok()
}

fn invalid_boolean_env(name: &str, fallback_raw: &str) -> String {
    let value = runtime_var_raw_optional(name).unwrap_or_else(|| fallback_raw.to_string());
    format!("Invalid boolean env var {}={}", name, value)
}

fn invalid_integer_env(name: &str) -> String {
    let value = runtime_var_raw_optional(name).unwrap_or_default();
    if value.trim().is_empty() {
        format!("Invalid integer env var {}=<empty>", name)
    } else {
        format!("Invalid integer env var {}={}", name, value)
    }
}

fn parse_gateway_authority_list(raw: &str) -> Result<Vec<String>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    trimmed
        .split(',')
        .map(|entry| {
            let (host, port, is_ip_literal) = parse_gateway_authority(entry, 443)?;
            Ok(gateway_authority_to_string(
                host.as_str(),
                port,
                is_ip_literal,
            ))
        })
        .collect()
}

fn parse_gateway_ip_allowlist(raw: &str) -> Result<Vec<IpAddr>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let mut allowlist = Vec::new();
    for entry in trimmed.split(',') {
        let candidate = entry.trim();
        if candidate.is_empty() {
            continue;
        }
        let parsed = candidate.parse::<IpAddr>().map_err(|_| {
            format!(
                "invalid IP literal {}; expected comma-separated IPv4/IPv6 literals",
                candidate
            )
        })?;
        allowlist.push(parsed);
    }
    Ok(allowlist)
}

fn parse_gateway_upstream_origin(value: &str) -> Result<GatewayUpstreamOrigin, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("must not be empty".to_string());
    }
    let (scheme_raw, authority_raw) = trimmed
        .split_once("://")
        .ok_or_else(|| "must include scheme://authority".to_string())?;
    let scheme = scheme_raw.trim().to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https") {
        return Err("scheme must be http or https".to_string());
    }
    if authority_raw.contains('/') || authority_raw.contains('?') || authority_raw.contains('#') {
        return Err("must not include path, query, or fragment".to_string());
    }
    if authority_raw.contains('@') {
        return Err("must not include userinfo".to_string());
    }
    let default_port = if scheme == "https" { 443 } else { 80 };
    let (host, port, host_is_ip_literal) = parse_gateway_authority(authority_raw, default_port)?;
    Ok(GatewayUpstreamOrigin {
        scheme,
        host,
        port,
        host_is_ip_literal,
    })
}

fn parse_gateway_authority(raw: &str, default_port: u16) -> Result<(String, u16, bool), String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("authority must not be empty".to_string());
    }
    if trimmed.contains('/') || trimmed.contains('?') || trimmed.contains('#') || trimmed.contains('@') {
        return Err("authority must not include path/query/fragment/userinfo".to_string());
    }

    let (host_raw, port) = if let Some(rest) = trimmed.strip_prefix('[') {
        let close = rest
            .find(']')
            .ok_or_else(|| "IPv6 literals must be closed with ]".to_string())?;
        let host_part = &rest[..close];
        let suffix = rest[close + 1..].trim();
        let port_value = if suffix.is_empty() {
            default_port
        } else if let Some(port_raw) = suffix.strip_prefix(':') {
            parse_port(port_raw)?
        } else {
            return Err("unexpected characters after IPv6 literal".to_string());
        };
        (host_part.to_string(), port_value)
    } else {
        let last_colon = trimmed.rfind(':');
        let (host_part, port_value) = match last_colon {
            Some(index) => {
                let host_part = &trimmed[..index];
                let port_candidate = &trimmed[index + 1..];
                if host_part.contains(':') {
                    return Err("IPv6 literals must be bracketed when specifying a port".to_string());
                }
                if port_candidate.is_empty() {
                    return Err("port must not be empty when ':' is present".to_string());
                }
                (host_part.to_string(), parse_port(port_candidate)?)
            }
            None => (trimmed.to_string(), default_port),
        };
        (host_part, port_value)
    };

    let host_trimmed = host_raw.trim();
    if host_trimmed.is_empty() {
        return Err("host must not be empty".to_string());
    }
    let host_is_ip_literal = host_trimmed.parse::<IpAddr>().is_ok();
    let canonical_host = if host_is_ip_literal {
        host_trimmed
            .parse::<IpAddr>()
            .map_err(|_| "invalid IP literal host".to_string())?
            .to_string()
    } else {
        host_trimmed.to_ascii_lowercase()
    };
    Ok((canonical_host, port, host_is_ip_literal))
}

fn parse_port(raw: &str) -> Result<u16, String> {
    let parsed = raw
        .trim()
        .parse::<u16>()
        .map_err(|_| "invalid port".to_string())?;
    if parsed == 0 {
        return Err("port must be greater than zero".to_string());
    }
    Ok(parsed)
}

fn gateway_authority_to_string(host: &str, port: u16, host_is_ip_literal: bool) -> String {
    if host_is_ip_literal && host.contains(':') {
        format!("[{}]:{}", host, port)
    } else {
        format!("{}:{}", host, port)
    }
}

fn is_private_or_loopback_ip(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => v4.is_private() || v4.is_loopback(),
        IpAddr::V6(v6) => v6.is_unique_local() || v6.is_loopback(),
    }
}

fn is_insecure_special_use_ip(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(v4) => {
            v4.is_link_local()
                || v4.is_multicast()
                || v4.is_unspecified()
                || v4 == Ipv4Addr::BROADCAST
                || v4.octets()[0] == 0
        }
        IpAddr::V6(v6) => v6.is_unicast_link_local() || v6.is_multicast() || v6.is_unspecified(),
    }
}

fn valid_header_name(name: &str) -> bool {
    !name.is_empty()
        && name.as_bytes().iter().all(|byte| {
            matches!(
                byte,
                b'!' | b'#'
                    | b'$'
                    | b'%'
                    | b'&'
                    | b'\''
                    | b'*'
                    | b'+'
                    | b'-'
                    | b'.'
                    | b'^'
                    | b'_'
                    | b'`'
                    | b'|'
                    | b'~'
                    | b'0'..=b'9'
                    | b'a'..=b'z'
                    | b'A'..=b'Z'
            )
        })
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FrontierProviderSummary {
    pub provider: String,
    pub model_id: String,
    pub configured: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FrontierSummary {
    pub mode: String,
    pub diversity_confidence: String,
    pub provider_count: u8,
    pub reduced_diversity_warning: bool,
    pub providers: Vec<FrontierProviderSummary>,
}

#[cfg(test)]
pub(crate) fn parse_admin_config_write_enabled(value: Option<&str>) -> bool {
    value.and_then(parse_bool_like).unwrap_or(true)
}

pub fn admin_config_write_enabled() -> bool {
    env_bool_required("SHUMA_ADMIN_CONFIG_WRITE_ENABLED")
}

pub fn https_enforced() -> bool {
    env_bool_required("SHUMA_ENFORCE_HTTPS")
}

pub fn debug_headers_enabled() -> bool {
    env_bool_required("SHUMA_DEBUG_HEADERS")
}

pub fn forwarded_header_trust_configured() -> bool {
    runtime_var_trimmed_optional("SHUMA_FORWARDED_IP_SECRET").is_some()
}

pub fn kv_store_fail_open() -> bool {
    env_bool_required("SHUMA_KV_STORE_FAIL_OPEN")
}

fn env_bool_optional(name: &str, default: bool) -> bool {
    runtime_var_raw_optional(name)
        .and_then(|v| parse_bool_like(v.as_str()))
        .unwrap_or(default)
}

fn env_trimmed_optional(name: &str) -> Option<String> {
    runtime_var_trimmed_optional(name)
}

fn frontier_provider_model(name: &str, default_value: &str) -> String {
    env_trimmed_optional(name).unwrap_or_else(|| default_value.to_string())
}

fn frontier_provider_configured(name: &str) -> bool {
    env_trimmed_optional(name).is_some()
}

pub fn enterprise_multi_instance_enabled() -> bool {
    env_bool_optional("SHUMA_ENTERPRISE_MULTI_INSTANCE", false)
}

pub fn enterprise_unsynced_state_exception_confirmed() -> bool {
    env_bool_optional("SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED", false)
}

pub fn gateway_deployment_profile() -> GatewayDeploymentProfile {
    runtime_var_raw_optional("SHUMA_GATEWAY_DEPLOYMENT_PROFILE")
        .and_then(|value| parse_gateway_deployment_profile(value.as_str()))
        .unwrap_or_else(|| {
            parse_gateway_deployment_profile(defaults_raw("SHUMA_GATEWAY_DEPLOYMENT_PROFILE").as_str())
                .unwrap_or(GatewayDeploymentProfile::SharedServer)
        })
}

pub fn gateway_origin_auth_mode() -> GatewayOriginAuthMode {
    runtime_var_raw_optional("SHUMA_GATEWAY_ORIGIN_AUTH_MODE")
        .and_then(|value| parse_gateway_origin_auth_mode(value.as_str()))
        .unwrap_or_else(|| {
            parse_gateway_origin_auth_mode(defaults_raw("SHUMA_GATEWAY_ORIGIN_AUTH_MODE").as_str())
                .unwrap_or(GatewayOriginAuthMode::NetworkOnly)
        })
}

pub fn runtime_environment() -> RuntimeEnvironment {
    runtime_var_raw_optional("SHUMA_RUNTIME_ENV")
        .and_then(|value| parse_runtime_environment(value.as_str()))
        .unwrap_or(RuntimeEnvironment::RuntimeProd)
}

pub fn local_prod_direct_mode() -> bool {
    env_bool_optional(
        "SHUMA_LOCAL_PROD_DIRECT_MODE",
        defaults_bool("SHUMA_LOCAL_PROD_DIRECT_MODE"),
    )
}

pub fn adversary_sim_available() -> bool {
    env_bool_optional(
        "SHUMA_ADVERSARY_SIM_AVAILABLE",
        defaults_bool("SHUMA_ADVERSARY_SIM_AVAILABLE"),
    )
}

pub fn adversary_sim_available_default() -> bool {
    defaults_bool("SHUMA_ADVERSARY_SIM_AVAILABLE")
}

pub fn gateway_upstream_origin() -> Option<String> {
    env_trimmed_optional("SHUMA_GATEWAY_UPSTREAM_ORIGIN")
}

pub fn gateway_allow_insecure_http_local() -> bool {
    env_bool_optional(
        "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL",
        defaults_bool("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL"),
    )
}

pub fn gateway_allow_insecure_http_special_use_ips() -> bool {
    env_bool_optional(
        "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS",
        defaults_bool("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS"),
    )
}

pub fn gateway_insecure_http_special_use_ip_allowlist() -> String {
    runtime_var_raw_optional("SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST"))
        .trim()
        .to_string()
}

pub fn gateway_public_authorities() -> String {
    runtime_var_raw_optional("SHUMA_GATEWAY_PUBLIC_AUTHORITIES")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_PUBLIC_AUTHORITIES"))
        .trim()
        .to_string()
}

pub fn gateway_loop_max_hops() -> u8 {
    env_u8_optional(
        "SHUMA_GATEWAY_LOOP_MAX_HOPS",
        defaults_u8("SHUMA_GATEWAY_LOOP_MAX_HOPS"),
    )
    .unwrap_or(3)
}

pub fn gateway_tls_strict() -> bool {
    env_bool_optional("SHUMA_GATEWAY_TLS_STRICT", defaults_bool("SHUMA_GATEWAY_TLS_STRICT"))
}

pub fn gateway_origin_lock_confirmed() -> bool {
    env_bool_optional(
        "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED",
        defaults_bool("SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED"),
    )
}

pub fn gateway_origin_auth_header_name() -> String {
    runtime_var_raw_optional("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME"))
        .trim()
        .to_string()
}

pub fn gateway_origin_auth_header_value() -> String {
    runtime_var_raw_optional("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE")
        .unwrap_or_else(|| defaults_raw("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE"))
        .trim()
        .to_string()
}

pub fn gateway_origin_auth_max_age_days() -> u32 {
    env_u32_optional(
        "SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS",
        defaults_u32("SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS"),
    )
    .unwrap_or(90)
}

pub fn gateway_origin_auth_rotation_overlap_days() -> u32 {
    env_u32_optional(
        "SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS",
        defaults_u32("SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS"),
    )
    .unwrap_or(7)
}

pub fn gateway_reserved_route_collision_check_passed() -> bool {
    env_bool_optional(
        "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
        defaults_bool("SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED"),
    )
}

pub fn sim_telemetry_secret() -> Option<String> {
    env_trimmed_optional("SHUMA_SIM_TELEMETRY_SECRET")
}

pub fn frontier_summary() -> FrontierSummary {
    let providers = vec![
        FrontierProviderSummary {
            provider: "openai".to_string(),
            model_id: frontier_provider_model("SHUMA_FRONTIER_OPENAI_MODEL", FRONTIER_OPENAI_MODEL_DEFAULT),
            configured: frontier_provider_configured("SHUMA_FRONTIER_OPENAI_API_KEY"),
        },
        FrontierProviderSummary {
            provider: "anthropic".to_string(),
            model_id: frontier_provider_model(
                "SHUMA_FRONTIER_ANTHROPIC_MODEL",
                FRONTIER_ANTHROPIC_MODEL_DEFAULT,
            ),
            configured: frontier_provider_configured("SHUMA_FRONTIER_ANTHROPIC_API_KEY"),
        },
        FrontierProviderSummary {
            provider: "google".to_string(),
            model_id: frontier_provider_model("SHUMA_FRONTIER_GOOGLE_MODEL", FRONTIER_GOOGLE_MODEL_DEFAULT),
            configured: frontier_provider_configured("SHUMA_FRONTIER_GOOGLE_API_KEY"),
        },
        FrontierProviderSummary {
            provider: "xai".to_string(),
            model_id: frontier_provider_model("SHUMA_FRONTIER_XAI_MODEL", FRONTIER_XAI_MODEL_DEFAULT),
            configured: frontier_provider_configured("SHUMA_FRONTIER_XAI_API_KEY"),
        },
    ];

    let provider_count = providers
        .iter()
        .filter(|provider| provider.configured)
        .count() as u8;
    let mode = if provider_count == 0 {
        "disabled"
    } else if provider_count == 1 {
        "single_provider_self_play"
    } else {
        "multi_provider_playoff"
    };
    let diversity_confidence = if provider_count >= 2 {
        "higher"
    } else if provider_count == 1 {
        "low"
    } else {
        "none"
    };

    FrontierSummary {
        mode: mode.to_string(),
        diversity_confidence: diversity_confidence.to_string(),
        provider_count,
        reduced_diversity_warning: provider_count == 1,
        providers,
    }
}

pub fn rate_limiter_redis_url() -> Option<String> {
    runtime_var_raw_optional("SHUMA_RATE_LIMITER_REDIS_URL").and_then(|value| parse_redis_url(&value))
}

pub fn ban_store_redis_url() -> Option<String> {
    runtime_var_raw_optional("SHUMA_BAN_STORE_REDIS_URL").and_then(|value| parse_redis_url(&value))
}

pub fn ban_store_outage_mode() -> BanStoreOutageMode {
    runtime_var_raw_optional("SHUMA_BAN_STORE_OUTAGE_MODE")
        .and_then(|value| parse_ban_store_outage_mode(value.as_str()))
        .unwrap_or_else(default_ban_store_outage_mode)
}

fn env_rate_limiter_outage_mode(
    name: &str,
    default: RateLimiterOutageMode,
) -> RateLimiterOutageMode {
    runtime_var_raw_optional(name)
        .and_then(|value| parse_rate_limiter_outage_mode(value.as_str()))
        .unwrap_or(default)
}

pub fn rate_limiter_outage_mode_main() -> RateLimiterOutageMode {
    env_rate_limiter_outage_mode(
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        default_rate_limiter_outage_mode_main(),
    )
}

pub fn rate_limiter_outage_mode_admin_auth() -> RateLimiterOutageMode {
    env_rate_limiter_outage_mode(
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
        default_rate_limiter_outage_mode_admin_auth(),
    )
}

fn parse_bool_like(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_redis_url(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("redis://") || lower.starts_with("rediss://") {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub(crate) fn parse_composability_mode(value: &str) -> Option<ComposabilityMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "off" => Some(ComposabilityMode::Off),
        "signal" => Some(ComposabilityMode::Signal),
        "enforce" => Some(ComposabilityMode::Enforce),
        "both" => Some(ComposabilityMode::Both),
        _ => None,
    }
}

pub(crate) fn parse_ip_range_policy_mode(value: &str) -> Option<IpRangePolicyMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "off" => Some(IpRangePolicyMode::Off),
        "advisory" => Some(IpRangePolicyMode::Advisory),
        "enforce" => Some(IpRangePolicyMode::Enforce),
        _ => None,
    }
}

pub(crate) fn parse_ip_range_policy_action(value: &str) -> Option<IpRangePolicyAction> {
    match value.trim().to_ascii_lowercase().as_str() {
        "forbidden_403" => Some(IpRangePolicyAction::Forbidden403),
        "custom_message" => Some(IpRangePolicyAction::CustomMessage),
        "drop_connection" => Some(IpRangePolicyAction::DropConnection),
        "redirect_308" => Some(IpRangePolicyAction::Redirect308),
        "rate_limit" => Some(IpRangePolicyAction::RateLimit),
        "honeypot" => Some(IpRangePolicyAction::Honeypot),
        "maze" => Some(IpRangePolicyAction::Maze),
        "tarpit" => Some(IpRangePolicyAction::Tarpit),
        _ => None,
    }
}

pub(crate) fn parse_provider_backend(value: &str) -> Option<ProviderBackend> {
    match value.trim().to_ascii_lowercase().as_str() {
        "internal" => Some(ProviderBackend::Internal),
        "external" => Some(ProviderBackend::External),
        _ => None,
    }
}

pub(crate) fn parse_non_human_traffic_stance(
    value: &str,
) -> Option<crate::bot_identity::policy::NonHumanTrafficStance> {
    match value.trim().to_ascii_lowercase().as_str() {
        "deny_all_non_human" => Some(crate::bot_identity::policy::NonHumanTrafficStance::DenyAllNonHuman),
        "allow_only_explicit_verified_identities" => Some(
            crate::bot_identity::policy::NonHumanTrafficStance::AllowOnlyExplicitVerifiedIdentities,
        ),
        "allow_verified_by_category" => Some(
            crate::bot_identity::policy::NonHumanTrafficStance::AllowVerifiedByCategory,
        ),
        "allow_verified_with_low_cost_profiles_only" => Some(
            crate::bot_identity::policy::NonHumanTrafficStance::AllowVerifiedWithLowCostProfilesOnly,
        ),
        _ => None,
    }
}

pub(crate) fn parse_edge_integration_mode(value: &str) -> Option<EdgeIntegrationMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "off" => Some(EdgeIntegrationMode::Off),
        "additive" => Some(EdgeIntegrationMode::Additive),
        "authoritative" => Some(EdgeIntegrationMode::Authoritative),
        _ => None,
    }
}

pub(crate) fn parse_cdp_probe_family(value: &str) -> Option<CdpProbeFamily> {
    match value.trim().to_ascii_lowercase().as_str() {
        "v1" => Some(CdpProbeFamily::V1),
        "v2" => Some(CdpProbeFamily::V2),
        "split" => Some(CdpProbeFamily::Split),
        _ => None,
    }
}

pub(crate) fn parse_maze_rollout_phase(value: &str) -> Option<MazeRolloutPhase> {
    match value.trim().to_ascii_lowercase().as_str() {
        "instrument" => Some(MazeRolloutPhase::Instrument),
        "advisory" => Some(MazeRolloutPhase::Advisory),
        "enforce" => Some(MazeRolloutPhase::Enforce),
        _ => None,
    }
}

pub(crate) fn parse_maze_seed_provider(value: &str) -> Option<MazeSeedProvider> {
    match value.trim().to_ascii_lowercase().as_str() {
        "internal" => Some(MazeSeedProvider::Internal),
        "operator" => Some(MazeSeedProvider::Operator),
        _ => None,
    }
}

pub(crate) fn parse_tarpit_fallback_action(value: &str) -> Option<TarpitFallbackAction> {
    match value.trim().to_ascii_lowercase().as_str() {
        "maze" => Some(TarpitFallbackAction::Maze),
        "block" => Some(TarpitFallbackAction::Block),
        _ => None,
    }
}

pub(crate) fn parse_rate_limiter_outage_mode(value: &str) -> Option<RateLimiterOutageMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "fallback_internal" => Some(RateLimiterOutageMode::FallbackInternal),
        "fail_open" => Some(RateLimiterOutageMode::FailOpen),
        "fail_closed" => Some(RateLimiterOutageMode::FailClosed),
        _ => None,
    }
}

pub(crate) fn parse_ban_store_outage_mode(value: &str) -> Option<BanStoreOutageMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "fallback_internal" => Some(BanStoreOutageMode::FallbackInternal),
        "fail_open" => Some(BanStoreOutageMode::FailOpen),
        "fail_closed" => Some(BanStoreOutageMode::FailClosed),
        _ => None,
    }
}

pub(crate) fn parse_runtime_environment(value: &str) -> Option<RuntimeEnvironment> {
    match value.trim().to_ascii_lowercase().as_str() {
        "runtime-dev" => Some(RuntimeEnvironment::RuntimeDev),
        "runtime-prod" => Some(RuntimeEnvironment::RuntimeProd),
        _ => None,
    }
}

pub(crate) fn parse_gateway_deployment_profile(value: &str) -> Option<GatewayDeploymentProfile> {
    match value.trim().to_ascii_lowercase().as_str() {
        "shared-server" => Some(GatewayDeploymentProfile::SharedServer),
        "edge-fermyon" => Some(GatewayDeploymentProfile::EdgeFermyon),
        _ => None,
    }
}

pub(crate) fn parse_gateway_origin_auth_mode(value: &str) -> Option<GatewayOriginAuthMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "network_only" => Some(GatewayOriginAuthMode::NetworkOnly),
        "signed_header" => Some(GatewayOriginAuthMode::SignedHeader),
        _ => None,
    }
}

pub fn event_log_retention_hours() -> u64 {
    env_u64_defaulted("SHUMA_EVENT_LOG_RETENTION_HOURS")
}

pub fn monitoring_retention_hours() -> u64 {
    env_u64_defaulted("SHUMA_MONITORING_RETENTION_HOURS")
}

pub fn monitoring_rollup_retention_hours() -> u64 {
    env_u64_defaulted("SHUMA_MONITORING_ROLLUP_RETENTION_HOURS")
}

pub fn env_string_required(name: &str) -> String {
    if cfg!(test) {
        return runtime_var_raw_optional(name).unwrap_or_else(|| defaults_raw(name));
    }
    runtime_var_raw_optional(name).unwrap_or_else(|| panic!("Missing required env var {}", name))
}

fn env_bool_required(name: &str) -> bool {
    if cfg!(test) {
        return runtime_var_raw_optional(name)
            .and_then(|v| parse_bool_like(v.as_str()))
            .unwrap_or_else(|| defaults_bool(name));
    }
    let value = runtime_var_raw_optional(name).unwrap_or_else(|| panic!("Missing required env var {}", name));
    parse_bool_like(&value).unwrap_or_else(|| panic!("Invalid boolean env var {}={}", name, value))
}

fn env_u64_defaulted(name: &str) -> u64 {
    runtime_var_raw_optional(name)
        .map(|value| {
            value
                .trim()
                .parse::<u64>()
                .unwrap_or_else(|_| panic!("Invalid integer env var {}={}", name, value))
        })
        .unwrap_or_else(|| defaults_u64(name))
}

fn clamp_pow_difficulty(value: u8) -> u8 {
    value.clamp(POW_DIFFICULTY_MIN, POW_DIFFICULTY_MAX)
}

fn clamp_maze_micro_pow_difficulty(value: u8) -> u8 {
    value.clamp(MAZE_MICRO_POW_DIFFICULTY_MIN, MAZE_MICRO_POW_DIFFICULTY_MAX)
}

fn clamp_pow_ttl(value: u64) -> u64 {
    value.clamp(POW_TTL_MIN, POW_TTL_MAX)
}

fn clamp_challenge_threshold(value: u8) -> u8 {
    value.clamp(CHALLENGE_THRESHOLD_MIN, CHALLENGE_THRESHOLD_MAX)
}

fn clamp_maze_threshold(value: u8) -> u8 {
    value.clamp(MAZE_THRESHOLD_MIN, MAZE_THRESHOLD_MAX)
}

fn clamp_botness_weight(value: u8) -> u8 {
    value.clamp(BOTNESS_WEIGHT_MIN, BOTNESS_WEIGHT_MAX)
}

fn clamp_challenge_puzzle_transform_count(value: u8) -> u8 {
    value.clamp(CHALLENGE_TRANSFORM_COUNT_MIN, CHALLENGE_TRANSFORM_COUNT_MAX)
}

fn clamp_challenge_puzzle_seed_ttl(value: u64) -> u64 {
    value.clamp(CHALLENGE_PUZZLE_SEED_TTL_MIN, CHALLENGE_PUZZLE_SEED_TTL_MAX)
}

fn clamp_challenge_puzzle_attempt_limit(value: u32) -> u32 {
    value.clamp(
        CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MIN,
        CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MAX,
    )
}

fn clamp_challenge_puzzle_attempt_window(value: u64) -> u64 {
    value.clamp(
        CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MIN,
        CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MAX,
    )
}

fn clamp_not_a_bot_score(value: u8) -> u8 {
    value.clamp(NOT_A_BOT_SCORE_MIN, NOT_A_BOT_SCORE_MAX)
}

fn clamp_not_a_bot_nonce_ttl(value: u64) -> u64 {
    value.clamp(NOT_A_BOT_NONCE_TTL_MIN, NOT_A_BOT_NONCE_TTL_MAX)
}

fn clamp_not_a_bot_marker_ttl(value: u64) -> u64 {
    value.clamp(NOT_A_BOT_MARKER_TTL_MIN, NOT_A_BOT_MARKER_TTL_MAX)
}

fn clamp_not_a_bot_attempt_limit(value: u32) -> u32 {
    value.clamp(NOT_A_BOT_ATTEMPT_LIMIT_MIN, NOT_A_BOT_ATTEMPT_LIMIT_MAX)
}

fn clamp_not_a_bot_attempt_window(value: u64) -> u64 {
    value.clamp(NOT_A_BOT_ATTEMPT_WINDOW_MIN, NOT_A_BOT_ATTEMPT_WINDOW_MAX)
}

fn clamp_tarpit_progress_token_ttl_seconds(value: u64) -> u64 {
    value.clamp(
        TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MIN,
        TARPIT_PROGRESS_TOKEN_TTL_SECONDS_MAX,
    )
}

fn clamp_tarpit_progress_replay_ttl_seconds(value: u64) -> u64 {
    value.clamp(
        TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MIN,
        TARPIT_PROGRESS_REPLAY_TTL_SECONDS_MAX,
    )
}

fn clamp_tarpit_hashcash_difficulty(value: u8) -> u8 {
    value.clamp(TARPIT_HASHCASH_DIFFICULTY_MIN, TARPIT_HASHCASH_DIFFICULTY_MAX)
}

fn clamp_tarpit_step_chunk_base_bytes(value: u32) -> u32 {
    value.clamp(
        TARPIT_STEP_CHUNK_BASE_BYTES_MIN,
        TARPIT_STEP_CHUNK_BASE_BYTES_MAX,
    )
}

fn clamp_tarpit_step_chunk_max_bytes(value: u32) -> u32 {
    value.clamp(TARPIT_STEP_CHUNK_MAX_BYTES_MIN, TARPIT_STEP_CHUNK_MAX_BYTES_MAX)
}

fn clamp_tarpit_step_jitter_percent(value: u8) -> u8 {
    value.clamp(TARPIT_STEP_JITTER_PERCENT_MIN, TARPIT_STEP_JITTER_PERCENT_MAX)
}

fn clamp_tarpit_egress_window_seconds(value: u64) -> u64 {
    value.clamp(
        TARPIT_EGRESS_WINDOW_SECONDS_MIN,
        TARPIT_EGRESS_WINDOW_SECONDS_MAX,
    )
}

fn clamp_tarpit_egress_global_bytes_per_window(value: u64) -> u64 {
    value.clamp(
        TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW_MIN,
        TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW_MAX,
    )
}

fn clamp_tarpit_egress_per_ip_bucket_bytes_per_window(value: u64) -> u64 {
    value.clamp(
        TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW_MIN,
        TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW_MAX,
    )
}

fn clamp_tarpit_egress_per_flow_max_bytes(value: u64) -> u64 {
    value.clamp(
        TARPIT_EGRESS_PER_FLOW_MAX_BYTES_MIN,
        TARPIT_EGRESS_PER_FLOW_MAX_BYTES_MAX,
    )
}

fn clamp_tarpit_egress_per_flow_max_duration_seconds(value: u64) -> u64 {
    value.clamp(
        TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS_MIN,
        TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS_MAX,
    )
}

fn clamp_tarpit_max_concurrent_global(value: u32) -> u32 {
    value.clamp(
        TARPIT_MAX_CONCURRENT_GLOBAL_MIN,
        TARPIT_MAX_CONCURRENT_GLOBAL_MAX,
    )
}

fn clamp_tarpit_max_concurrent_per_ip_bucket(value: u32) -> u32 {
    value.clamp(
        TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MIN,
        TARPIT_MAX_CONCURRENT_PER_IP_BUCKET_MAX,
    )
}

fn clamp_ip_range_suggestions_min_observations(value: u32) -> u32 {
    value.clamp(
        IP_RANGE_SUGGESTIONS_MIN_OBSERVATIONS_MIN,
        IP_RANGE_SUGGESTIONS_MIN_OBSERVATIONS_MAX,
    )
}

fn clamp_ip_range_suggestions_min_bot_events(value: u32) -> u32 {
    value.clamp(
        IP_RANGE_SUGGESTIONS_MIN_BOT_EVENTS_MIN,
        IP_RANGE_SUGGESTIONS_MIN_BOT_EVENTS_MAX,
    )
}

fn clamp_ip_range_suggestions_confidence_percent(value: u8) -> u8 {
    value.clamp(
        IP_RANGE_SUGGESTIONS_CONFIDENCE_PERCENT_MIN,
        IP_RANGE_SUGGESTIONS_CONFIDENCE_PERCENT_MAX,
    )
}

fn clamp_ip_range_suggestions_collateral_percent(value: u8) -> u8 {
    value.clamp(
        IP_RANGE_SUGGESTIONS_COLLATERAL_PERCENT_MIN,
        IP_RANGE_SUGGESTIONS_COLLATERAL_PERCENT_MAX,
    )
}

fn clamp_ip_range_suggestions_ipv4_min_prefix_len(value: u8) -> u8 {
    value.clamp(
        IP_RANGE_SUGGESTIONS_IPV4_MIN_PREFIX_LEN_MIN,
        IP_RANGE_SUGGESTIONS_IPV4_MIN_PREFIX_LEN_MAX,
    )
}

fn clamp_ip_range_suggestions_ipv6_min_prefix_len(value: u8) -> u8 {
    value.clamp(
        IP_RANGE_SUGGESTIONS_IPV6_MIN_PREFIX_LEN_MIN,
        IP_RANGE_SUGGESTIONS_IPV6_MIN_PREFIX_LEN_MAX,
    )
}

fn clamp_ip_range_suggestions_likely_human_sample_percent(value: u8) -> u8 {
    value.clamp(
        IP_RANGE_SUGGESTIONS_LIKELY_HUMAN_SAMPLE_PERCENT_MIN,
        IP_RANGE_SUGGESTIONS_LIKELY_HUMAN_SAMPLE_PERCENT_MAX,
    )
}

fn clamp_adversary_sim_duration_seconds(value: u64) -> u64 {
    value.clamp(
        ADVERSARY_SIM_DURATION_SECONDS_MIN,
        ADVERSARY_SIM_DURATION_SECONDS_MAX,
    )
}

fn clamp_config_values(cfg: &mut Config) {
    cfg.pow_difficulty = clamp_pow_difficulty(cfg.pow_difficulty);
    cfg.pow_ttl_seconds = clamp_pow_ttl(cfg.pow_ttl_seconds);
    cfg.challenge_puzzle_transform_count =
        clamp_challenge_puzzle_transform_count(cfg.challenge_puzzle_transform_count);
    cfg.challenge_puzzle_seed_ttl_seconds =
        clamp_challenge_puzzle_seed_ttl(cfg.challenge_puzzle_seed_ttl_seconds);
    cfg.challenge_puzzle_attempt_limit_per_window =
        clamp_challenge_puzzle_attempt_limit(cfg.challenge_puzzle_attempt_limit_per_window);
    cfg.challenge_puzzle_attempt_window_seconds =
        clamp_challenge_puzzle_attempt_window(cfg.challenge_puzzle_attempt_window_seconds);
    cfg.challenge_puzzle_risk_threshold = clamp_challenge_threshold(cfg.challenge_puzzle_risk_threshold);
    cfg.not_a_bot_risk_threshold = clamp_challenge_threshold(cfg.not_a_bot_risk_threshold);
    cfg.not_a_bot_pass_score = clamp_not_a_bot_score(cfg.not_a_bot_pass_score);
    cfg.not_a_bot_fail_score = clamp_not_a_bot_score(cfg.not_a_bot_fail_score);
    cfg.not_a_bot_nonce_ttl_seconds = clamp_not_a_bot_nonce_ttl(cfg.not_a_bot_nonce_ttl_seconds);
    cfg.not_a_bot_marker_ttl_seconds = clamp_not_a_bot_marker_ttl(cfg.not_a_bot_marker_ttl_seconds);
    cfg.not_a_bot_attempt_limit_per_window =
        clamp_not_a_bot_attempt_limit(cfg.not_a_bot_attempt_limit_per_window);
    cfg.not_a_bot_attempt_window_seconds =
        clamp_not_a_bot_attempt_window(cfg.not_a_bot_attempt_window_seconds);
    cfg.adversary_sim_duration_seconds =
        clamp_adversary_sim_duration_seconds(cfg.adversary_sim_duration_seconds);
    cfg.tarpit_progress_token_ttl_seconds =
        clamp_tarpit_progress_token_ttl_seconds(cfg.tarpit_progress_token_ttl_seconds);
    cfg.tarpit_progress_replay_ttl_seconds =
        clamp_tarpit_progress_replay_ttl_seconds(cfg.tarpit_progress_replay_ttl_seconds);
    cfg.tarpit_hashcash_min_difficulty =
        clamp_tarpit_hashcash_difficulty(cfg.tarpit_hashcash_min_difficulty);
    cfg.tarpit_hashcash_max_difficulty =
        clamp_tarpit_hashcash_difficulty(cfg.tarpit_hashcash_max_difficulty);
    cfg.tarpit_hashcash_base_difficulty =
        clamp_tarpit_hashcash_difficulty(cfg.tarpit_hashcash_base_difficulty);
    if cfg.tarpit_hashcash_max_difficulty < cfg.tarpit_hashcash_min_difficulty {
        cfg.tarpit_hashcash_max_difficulty = cfg.tarpit_hashcash_min_difficulty;
    }
    cfg.tarpit_hashcash_base_difficulty = cfg
        .tarpit_hashcash_base_difficulty
        .clamp(cfg.tarpit_hashcash_min_difficulty, cfg.tarpit_hashcash_max_difficulty);
    cfg.tarpit_step_chunk_base_bytes =
        clamp_tarpit_step_chunk_base_bytes(cfg.tarpit_step_chunk_base_bytes);
    cfg.tarpit_step_chunk_max_bytes =
        clamp_tarpit_step_chunk_max_bytes(cfg.tarpit_step_chunk_max_bytes);
    if cfg.tarpit_step_chunk_max_bytes < cfg.tarpit_step_chunk_base_bytes {
        cfg.tarpit_step_chunk_max_bytes = cfg.tarpit_step_chunk_base_bytes;
    }
    cfg.tarpit_step_jitter_percent =
        clamp_tarpit_step_jitter_percent(cfg.tarpit_step_jitter_percent);
    cfg.tarpit_egress_window_seconds =
        clamp_tarpit_egress_window_seconds(cfg.tarpit_egress_window_seconds);
    cfg.tarpit_egress_global_bytes_per_window =
        clamp_tarpit_egress_global_bytes_per_window(cfg.tarpit_egress_global_bytes_per_window);
    cfg.tarpit_egress_per_ip_bucket_bytes_per_window =
        clamp_tarpit_egress_per_ip_bucket_bytes_per_window(
            cfg.tarpit_egress_per_ip_bucket_bytes_per_window,
        );
    if cfg.tarpit_egress_per_ip_bucket_bytes_per_window > cfg.tarpit_egress_global_bytes_per_window {
        cfg.tarpit_egress_per_ip_bucket_bytes_per_window =
            cfg.tarpit_egress_global_bytes_per_window;
    }
    cfg.tarpit_egress_per_flow_max_bytes =
        clamp_tarpit_egress_per_flow_max_bytes(cfg.tarpit_egress_per_flow_max_bytes);
    cfg.tarpit_egress_per_flow_max_duration_seconds =
        clamp_tarpit_egress_per_flow_max_duration_seconds(
            cfg.tarpit_egress_per_flow_max_duration_seconds,
        );
    cfg.tarpit_max_concurrent_global =
        clamp_tarpit_max_concurrent_global(cfg.tarpit_max_concurrent_global);
    cfg.tarpit_max_concurrent_per_ip_bucket =
        clamp_tarpit_max_concurrent_per_ip_bucket(cfg.tarpit_max_concurrent_per_ip_bucket);
    if cfg.tarpit_max_concurrent_per_ip_bucket > cfg.tarpit_max_concurrent_global {
        cfg.tarpit_max_concurrent_per_ip_bucket = cfg.tarpit_max_concurrent_global;
    }
    if cfg.not_a_bot_fail_score > cfg.not_a_bot_pass_score {
        cfg.not_a_bot_fail_score = cfg.not_a_bot_pass_score;
    }
    if cfg.challenge_puzzle_risk_threshold > CHALLENGE_THRESHOLD_MIN
        && cfg.not_a_bot_risk_threshold >= cfg.challenge_puzzle_risk_threshold
    {
        cfg.not_a_bot_risk_threshold = cfg.challenge_puzzle_risk_threshold.saturating_sub(1);
    }
    cfg.ip_range_suggestions_min_observations = clamp_ip_range_suggestions_min_observations(
        cfg.ip_range_suggestions_min_observations,
    );
    cfg.ip_range_suggestions_min_bot_events =
        clamp_ip_range_suggestions_min_bot_events(cfg.ip_range_suggestions_min_bot_events);
    cfg.ip_range_suggestions_min_confidence_percent =
        clamp_ip_range_suggestions_confidence_percent(
            cfg.ip_range_suggestions_min_confidence_percent,
        );
    cfg.ip_range_suggestions_low_collateral_percent = clamp_ip_range_suggestions_collateral_percent(
        cfg.ip_range_suggestions_low_collateral_percent,
    );
    cfg.ip_range_suggestions_high_collateral_percent = clamp_ip_range_suggestions_collateral_percent(
        cfg.ip_range_suggestions_high_collateral_percent,
    );
    if cfg.ip_range_suggestions_low_collateral_percent > cfg.ip_range_suggestions_high_collateral_percent
    {
        cfg.ip_range_suggestions_low_collateral_percent =
            cfg.ip_range_suggestions_high_collateral_percent;
    }
    cfg.ip_range_suggestions_ipv4_min_prefix_len = clamp_ip_range_suggestions_ipv4_min_prefix_len(
        cfg.ip_range_suggestions_ipv4_min_prefix_len,
    );
    cfg.ip_range_suggestions_ipv6_min_prefix_len = clamp_ip_range_suggestions_ipv6_min_prefix_len(
        cfg.ip_range_suggestions_ipv6_min_prefix_len,
    );
    cfg.ip_range_suggestions_likely_human_sample_percent =
        clamp_ip_range_suggestions_likely_human_sample_percent(
            cfg.ip_range_suggestions_likely_human_sample_percent,
        );
    cfg.botness_maze_threshold = clamp_maze_threshold(cfg.botness_maze_threshold);
    cfg.botness_weights.js_required = clamp_botness_weight(cfg.botness_weights.js_required);
    cfg.botness_weights.geo_risk = clamp_botness_weight(cfg.botness_weights.geo_risk);
    cfg.botness_weights.rate_medium = clamp_botness_weight(cfg.botness_weights.rate_medium);
    cfg.botness_weights.rate_high = clamp_botness_weight(cfg.botness_weights.rate_high);
    cfg.botness_weights.maze_behavior = clamp_botness_weight(cfg.botness_weights.maze_behavior);
    cfg.maze_token_ttl_seconds = cfg.maze_token_ttl_seconds.clamp(30, 600);
    cfg.maze_token_max_depth = cfg.maze_token_max_depth.clamp(1, 32);
    cfg.maze_token_branch_budget = cfg.maze_token_branch_budget.clamp(1, 12);
    cfg.maze_replay_ttl_seconds = cfg.maze_replay_ttl_seconds.clamp(30, 3600);
    cfg.maze_entropy_window_seconds = cfg.maze_entropy_window_seconds.clamp(10, 600);
    cfg.maze_checkpoint_every_nodes = cfg.maze_checkpoint_every_nodes.clamp(1, 16);
    cfg.maze_checkpoint_every_ms = cfg.maze_checkpoint_every_ms.clamp(200, 10_000);
    cfg.maze_step_ahead_max = cfg.maze_step_ahead_max.clamp(1, 16);
    cfg.maze_no_js_fallback_max_depth = cfg.maze_no_js_fallback_max_depth.clamp(1, 12);
    cfg.maze_micro_pow_depth_start = cfg.maze_micro_pow_depth_start.clamp(1, 24);
    cfg.maze_micro_pow_base_difficulty =
        clamp_maze_micro_pow_difficulty(cfg.maze_micro_pow_base_difficulty);
    cfg.maze_max_concurrent_global = cfg.maze_max_concurrent_global.clamp(1, 10_000);
    cfg.maze_max_concurrent_per_ip_bucket = cfg.maze_max_concurrent_per_ip_bucket.clamp(1, 256);
    cfg.maze_max_response_bytes = cfg.maze_max_response_bytes.clamp(1_024, 512 * 1024);
    cfg.maze_max_response_duration_ms = cfg.maze_max_response_duration_ms.clamp(100, 120_000);
    cfg.maze_server_visible_links = cfg.maze_server_visible_links.clamp(1, 32);
    cfg.maze_max_links = cfg.maze_max_links.clamp(1, 64);
    cfg.maze_max_paragraphs = cfg.maze_max_paragraphs.clamp(1, 24);
    cfg.maze_path_entropy_segment_len = cfg.maze_path_entropy_segment_len.clamp(8, 64);
    cfg.maze_seed_refresh_interval_seconds = cfg
        .maze_seed_refresh_interval_seconds
        .clamp(60, 7 * 24 * 3600);
    cfg.maze_seed_refresh_rate_limit_per_hour =
        cfg.maze_seed_refresh_rate_limit_per_hour.clamp(1, 1000);
    cfg.maze_seed_refresh_max_sources = cfg.maze_seed_refresh_max_sources.clamp(1, 500);
    cfg.cdp_detection_threshold = cfg.cdp_detection_threshold.clamp(0.0, 1.0);
    cfg.cdp_probe_rollout_percent = cfg.cdp_probe_rollout_percent.clamp(0, 100);
    cfg.fingerprint_state_ttl_seconds = cfg.fingerprint_state_ttl_seconds.clamp(30, 24 * 3600);
    cfg.fingerprint_flow_window_seconds = cfg.fingerprint_flow_window_seconds.clamp(10, 3600);
    cfg.fingerprint_flow_violation_threshold = cfg.fingerprint_flow_violation_threshold.clamp(1, 20);
    cfg.fingerprint_entropy_budget = clamp_botness_weight(cfg.fingerprint_entropy_budget);
    cfg.fingerprint_family_cap_header_runtime =
        clamp_botness_weight(cfg.fingerprint_family_cap_header_runtime);
    cfg.fingerprint_family_cap_transport =
        clamp_botness_weight(cfg.fingerprint_family_cap_transport);
    cfg.fingerprint_family_cap_temporal =
        clamp_botness_weight(cfg.fingerprint_family_cap_temporal);
    cfg.fingerprint_family_cap_persistence =
        clamp_botness_weight(cfg.fingerprint_family_cap_persistence);
    cfg.fingerprint_family_cap_behavior = clamp_botness_weight(cfg.fingerprint_family_cap_behavior);
}

#[cfg(test)]
pub(crate) fn parse_challenge_threshold(value: Option<&str>) -> u8 {
    let parsed = value
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or_else(default_challenge_threshold);
    clamp_challenge_threshold(parsed)
}

#[cfg(test)]
pub(crate) fn parse_maze_threshold(value: Option<&str>) -> u8 {
    let parsed = value
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or_else(default_maze_threshold);
    clamp_maze_threshold(parsed)
}

#[cfg(test)]
pub(crate) fn parse_botness_weight(value: Option<&str>, default_value: u8) -> u8 {
    let parsed = value
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(default_value);
    clamp_botness_weight(parsed)
}

fn parse_defaults_env_map(input: &str) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for (index, raw_line) in input.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("Invalid defaults line {}: missing '='", line_no))?;

        let key = key.trim();
        if key.is_empty() {
            return Err(format!("Invalid defaults line {}: empty key", line_no));
        }
        if !key
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        {
            return Err(format!(
                "Invalid defaults key '{}' on line {}",
                key, line_no
            ));
        }

        let mut value = raw_value.trim().to_string();
        if let Some((head, _)) = value.split_once(" #") {
            value = head.trim().to_string();
        }
        if value.len() >= 2 {
            let first = value.as_bytes()[0] as char;
            let last = value.as_bytes()[value.len() - 1] as char;
            if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
                value = value[1..value.len() - 1].to_string();
            }
        }

        map.insert(key.to_string(), value);
    }
    Ok(map)
}

fn defaults_map() -> &'static HashMap<String, String> {
    match &*DEFAULTS_MAP {
        Ok(map) => map,
        Err(err) => panic!("Invalid config/defaults.env: {}", err),
    }
}

fn defaults_raw(key: &str) -> String {
    defaults_map()
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("Missing required defaults key {}", key))
}

fn defaults_bool(key: &str) -> bool {
    parse_bool_like(defaults_raw(key).as_str())
        .unwrap_or_else(|| panic!("Invalid boolean default for {}", key))
}

fn defaults_u64(key: &str) -> u64 {
    defaults_raw(key)
        .trim()
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("Invalid integer default for {}", key))
}

fn defaults_u32(key: &str) -> u32 {
    defaults_raw(key)
        .trim()
        .parse::<u32>()
        .unwrap_or_else(|_| panic!("Invalid integer default for {}", key))
}

fn defaults_u16(key: &str) -> u16 {
    defaults_raw(key)
        .trim()
        .parse::<u16>()
        .unwrap_or_else(|_| panic!("Invalid integer default for {}", key))
}

fn defaults_u8(key: &str) -> u8 {
    defaults_raw(key)
        .trim()
        .parse::<u8>()
        .unwrap_or_else(|_| panic!("Invalid integer default for {}", key))
}

fn defaults_f32(key: &str) -> f32 {
    defaults_raw(key)
        .trim()
        .parse::<f32>()
        .unwrap_or_else(|_| panic!("Invalid float default for {}", key))
}

fn defaults_json<T>(key: &str) -> T
where
    T: DeserializeOwned,
{
    serde_json::from_str(defaults_raw(key).as_str())
        .unwrap_or_else(|_| panic!("Invalid JSON default for {}", key))
}

fn parse_string_list_value(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }
    if let Ok(v) = serde_json::from_str::<Vec<String>>(trimmed) {
        return Some(
            v.into_iter()
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect(),
        );
    }
    Some(
        trimmed
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
    )
}

fn parse_browser_rules_value(raw: &str) -> Option<Vec<(String, u32)>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }
    if let Ok(v) = serde_json::from_str::<Vec<(String, u32)>>(trimmed) {
        return Some(
            v.into_iter()
                .filter(|(name, _)| !name.trim().is_empty())
                .collect(),
        );
    }
    let mut parsed = Vec::new();
    for entry in trimmed.split(',') {
        let item = entry.trim();
        if item.is_empty() {
            continue;
        }
        let (name, version) = item.split_once(':')?;
        let name = name.trim();
        if name.is_empty() {
            return None;
        }
        let version = version.trim().parse::<u32>().ok()?;
        parsed.push((name.to_string(), version));
    }
    Some(parsed)
}

fn defaults_string_list(key: &str) -> Vec<String> {
    parse_string_list_value(defaults_raw(key).as_str())
        .unwrap_or_else(|| panic!("Invalid list default for {}", key))
}

fn defaults_country_list(key: &str) -> Vec<String> {
    crate::signals::geo::normalize_country_list(&defaults_string_list(key))
}

fn defaults_browser_rules(key: &str) -> Vec<(String, u32)> {
    parse_browser_rules_value(defaults_raw(key).as_str())
        .unwrap_or_else(|| panic!("Invalid browser rules default for {}", key))
}

fn default_ban_duration() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION")
}

fn default_ban_duration_honeypot() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_HONEYPOT")
}

fn default_ban_duration_ip_range_honeypot() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_IP_RANGE_HONEYPOT")
}

fn default_ban_duration_maze_crawler() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_MAZE_CRAWLER")
}

fn default_ban_duration_rate_limit() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_RATE_LIMIT")
}

fn default_ban_duration_admin() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_ADMIN")
}

fn default_ban_duration_cdp() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_CDP")
}

fn default_ban_duration_edge_fingerprint() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_EDGE_FINGERPRINT")
}

fn default_ban_duration_tarpit_persistence() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_TARPIT_PERSISTENCE")
}

fn default_ban_duration_not_a_bot_abuse() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_NOT_A_BOT_ABUSE")
}

fn default_ban_duration_challenge_puzzle_abuse() -> u64 {
    defaults_u64("SHUMA_BAN_DURATION_CHALLENGE_PUZZLE_ABUSE")
}

fn default_rate_limit() -> u32 {
    defaults_u32("SHUMA_RATE_LIMIT")
}

fn default_honeypot_enabled() -> bool {
    defaults_bool("SHUMA_HONEYPOT_ENABLED")
}

fn default_honeypots() -> Vec<String> {
    defaults_string_list("SHUMA_HONEYPOTS")
}

fn default_browser_policy_enabled() -> bool {
    defaults_bool("SHUMA_BROWSER_POLICY_ENABLED")
}

fn default_browser_block() -> Vec<(String, u32)> {
    defaults_browser_rules("SHUMA_BROWSER_BLOCK")
}

fn default_browser_allowlist() -> Vec<(String, u32)> {
    defaults_browser_rules("SHUMA_BROWSER_ALLOWLIST")
}

fn default_geo_risk() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_RISK_COUNTRIES")
}

fn default_geo_allow() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_ALLOW_COUNTRIES")
}

fn default_geo_challenge() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_CHALLENGE_COUNTRIES")
}

fn default_geo_maze() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_MAZE_COUNTRIES")
}

fn default_geo_block() -> Vec<String> {
    defaults_country_list("SHUMA_GEO_BLOCK_COUNTRIES")
}

fn default_geo_edge_headers_enabled() -> bool {
    defaults_bool("SHUMA_GEO_EDGE_HEADERS_ENABLED")
}

fn default_bypass_allowlists_enabled() -> bool {
    defaults_bool("SHUMA_BYPASS_ALLOWLISTS_ENABLED")
}

fn default_allowlist() -> Vec<String> {
    defaults_string_list("SHUMA_ALLOWLIST")
}

fn default_path_allowlist_enabled() -> bool {
    defaults_bool("SHUMA_PATH_ALLOWLIST_ENABLED")
}

fn default_path_allowlist() -> Vec<String> {
    defaults_string_list("SHUMA_PATH_ALLOWLIST")
}

fn default_ip_range_policy_mode() -> IpRangePolicyMode {
    let raw = defaults_raw("SHUMA_IP_RANGE_POLICY_MODE");
    parse_ip_range_policy_mode(raw.as_str())
        .unwrap_or_else(|| panic!("Invalid IP range policy mode default for {}", raw))
}

fn default_ip_range_emergency_allowlist() -> Vec<String> {
    defaults_string_list("SHUMA_IP_RANGE_EMERGENCY_ALLOWLIST")
}

fn default_ip_range_custom_rules() -> Vec<IpRangePolicyRule> {
    defaults_json("SHUMA_IP_RANGE_CUSTOM_RULES")
}

fn default_ip_range_suggestions_min_observations() -> u32 {
    clamp_ip_range_suggestions_min_observations(defaults_u32(
        "SHUMA_IP_RANGE_SUGGESTIONS_MIN_OBSERVATIONS",
    ))
}

fn default_ip_range_suggestions_min_bot_events() -> u32 {
    clamp_ip_range_suggestions_min_bot_events(defaults_u32(
        "SHUMA_IP_RANGE_SUGGESTIONS_MIN_BOT_EVENTS",
    ))
}

fn default_ip_range_suggestions_min_confidence_percent() -> u8 {
    clamp_ip_range_suggestions_confidence_percent(defaults_u8(
        "SHUMA_IP_RANGE_SUGGESTIONS_MIN_CONFIDENCE_PERCENT",
    ))
}

fn default_ip_range_suggestions_low_collateral_percent() -> u8 {
    clamp_ip_range_suggestions_collateral_percent(defaults_u8(
        "SHUMA_IP_RANGE_SUGGESTIONS_LOW_COLLATERAL_PERCENT",
    ))
}

fn default_ip_range_suggestions_high_collateral_percent() -> u8 {
    clamp_ip_range_suggestions_collateral_percent(defaults_u8(
        "SHUMA_IP_RANGE_SUGGESTIONS_HIGH_COLLATERAL_PERCENT",
    ))
}

fn default_ip_range_suggestions_ipv4_min_prefix_len() -> u8 {
    clamp_ip_range_suggestions_ipv4_min_prefix_len(defaults_u8(
        "SHUMA_IP_RANGE_SUGGESTIONS_IPV4_MIN_PREFIX_LEN",
    ))
}

fn default_ip_range_suggestions_ipv6_min_prefix_len() -> u8 {
    clamp_ip_range_suggestions_ipv6_min_prefix_len(defaults_u8(
        "SHUMA_IP_RANGE_SUGGESTIONS_IPV6_MIN_PREFIX_LEN",
    ))
}

fn default_ip_range_suggestions_likely_human_sample_percent() -> u8 {
    clamp_ip_range_suggestions_likely_human_sample_percent(defaults_u8(
        "SHUMA_IP_RANGE_SUGGESTIONS_LIKELY_HUMAN_SAMPLE_PERCENT",
    ))
}

fn default_shadow_mode() -> bool {
    defaults_bool("SHUMA_SHADOW_MODE")
}

fn default_adversary_sim_enabled() -> bool {
    defaults_bool("SHUMA_ADVERSARY_SIM_ENABLED")
}

fn default_adversary_sim_duration_seconds() -> u64 {
    clamp_adversary_sim_duration_seconds(defaults_u64("SHUMA_ADVERSARY_SIM_DURATION_SECONDS"))
}

fn default_maze_enabled() -> bool {
    defaults_bool("SHUMA_MAZE_ENABLED")
}

fn default_tarpit_enabled() -> bool {
    defaults_bool("SHUMA_TARPIT_ENABLED")
}

fn default_tarpit_progress_token_ttl_seconds() -> u64 {
    clamp_tarpit_progress_token_ttl_seconds(defaults_u64("SHUMA_TARPIT_PROGRESS_TOKEN_TTL_SECONDS"))
}

fn default_tarpit_progress_replay_ttl_seconds() -> u64 {
    clamp_tarpit_progress_replay_ttl_seconds(defaults_u64(
        "SHUMA_TARPIT_PROGRESS_REPLAY_TTL_SECONDS",
    ))
}

fn default_tarpit_hashcash_min_difficulty() -> u8 {
    clamp_tarpit_hashcash_difficulty(defaults_u8("SHUMA_TARPIT_HASHCASH_MIN_DIFFICULTY"))
}

fn default_tarpit_hashcash_max_difficulty() -> u8 {
    clamp_tarpit_hashcash_difficulty(defaults_u8("SHUMA_TARPIT_HASHCASH_MAX_DIFFICULTY"))
}

fn default_tarpit_hashcash_base_difficulty() -> u8 {
    clamp_tarpit_hashcash_difficulty(defaults_u8("SHUMA_TARPIT_HASHCASH_BASE_DIFFICULTY"))
}

fn default_tarpit_hashcash_adaptive() -> bool {
    defaults_bool("SHUMA_TARPIT_HASHCASH_ADAPTIVE")
}

fn default_tarpit_step_chunk_base_bytes() -> u32 {
    clamp_tarpit_step_chunk_base_bytes(defaults_u32("SHUMA_TARPIT_STEP_CHUNK_BASE_BYTES"))
}

fn default_tarpit_step_chunk_max_bytes() -> u32 {
    clamp_tarpit_step_chunk_max_bytes(defaults_u32("SHUMA_TARPIT_STEP_CHUNK_MAX_BYTES"))
}

fn default_tarpit_step_jitter_percent() -> u8 {
    clamp_tarpit_step_jitter_percent(defaults_u8("SHUMA_TARPIT_STEP_JITTER_PERCENT"))
}

fn default_tarpit_shard_rotation_enabled() -> bool {
    defaults_bool("SHUMA_TARPIT_SHARD_ROTATION_ENABLED")
}

fn default_tarpit_egress_window_seconds() -> u64 {
    clamp_tarpit_egress_window_seconds(defaults_u64("SHUMA_TARPIT_EGRESS_WINDOW_SECONDS"))
}

fn default_tarpit_egress_global_bytes_per_window() -> u64 {
    clamp_tarpit_egress_global_bytes_per_window(defaults_u64(
        "SHUMA_TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW",
    ))
}

fn default_tarpit_egress_per_ip_bucket_bytes_per_window() -> u64 {
    clamp_tarpit_egress_per_ip_bucket_bytes_per_window(defaults_u64(
        "SHUMA_TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW",
    ))
}

fn default_tarpit_egress_per_flow_max_bytes() -> u64 {
    clamp_tarpit_egress_per_flow_max_bytes(defaults_u64("SHUMA_TARPIT_EGRESS_PER_FLOW_MAX_BYTES"))
}

fn default_tarpit_egress_per_flow_max_duration_seconds() -> u64 {
    clamp_tarpit_egress_per_flow_max_duration_seconds(defaults_u64(
        "SHUMA_TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS",
    ))
}

fn default_tarpit_max_concurrent_global() -> u32 {
    clamp_tarpit_max_concurrent_global(defaults_u32("SHUMA_TARPIT_MAX_CONCURRENT_GLOBAL"))
}

fn default_tarpit_max_concurrent_per_ip_bucket() -> u32 {
    clamp_tarpit_max_concurrent_per_ip_bucket(defaults_u32("SHUMA_TARPIT_MAX_CONCURRENT_PER_IP_BUCKET"))
}

fn default_tarpit_fallback_action() -> TarpitFallbackAction {
    let raw = defaults_raw("SHUMA_TARPIT_FALLBACK_ACTION");
    parse_tarpit_fallback_action(raw.as_str()).unwrap_or_else(|| {
        panic!(
            "Invalid tarpit fallback action default for SHUMA_TARPIT_FALLBACK_ACTION={}",
            raw
        )
    })
}

fn default_maze_auto_ban() -> bool {
    defaults_bool("SHUMA_MAZE_AUTO_BAN")
}

fn default_maze_auto_ban_threshold() -> u32 {
    defaults_u32("SHUMA_MAZE_AUTO_BAN_THRESHOLD")
}

fn default_maze_rollout_phase() -> MazeRolloutPhase {
    let raw = defaults_raw("SHUMA_MAZE_ROLLOUT_PHASE");
    parse_maze_rollout_phase(raw.as_str()).unwrap_or_else(|| {
        panic!(
            "Invalid maze rollout phase default for SHUMA_MAZE_ROLLOUT_PHASE={}",
            raw
        )
    })
}

fn default_maze_token_ttl_seconds() -> u64 {
    defaults_u64("SHUMA_MAZE_TOKEN_TTL_SECONDS")
}

fn default_maze_token_max_depth() -> u16 {
    defaults_u16("SHUMA_MAZE_TOKEN_MAX_DEPTH")
}

fn default_maze_token_branch_budget() -> u8 {
    defaults_u8("SHUMA_MAZE_TOKEN_BRANCH_BUDGET")
}

fn default_maze_replay_ttl_seconds() -> u64 {
    defaults_u64("SHUMA_MAZE_REPLAY_TTL_SECONDS")
}

fn default_maze_entropy_window_seconds() -> u64 {
    defaults_u64("SHUMA_MAZE_ENTROPY_WINDOW_SECONDS")
}

fn default_maze_client_expansion_enabled() -> bool {
    defaults_bool("SHUMA_MAZE_CLIENT_EXPANSION_ENABLED")
}

fn default_maze_checkpoint_every_nodes() -> u64 {
    defaults_u64("SHUMA_MAZE_CHECKPOINT_EVERY_NODES")
}

fn default_maze_checkpoint_every_ms() -> u64 {
    defaults_u64("SHUMA_MAZE_CHECKPOINT_EVERY_MS")
}

fn default_maze_step_ahead_max() -> u64 {
    defaults_u64("SHUMA_MAZE_STEP_AHEAD_MAX")
}

fn default_maze_no_js_fallback_max_depth() -> u16 {
    defaults_u16("SHUMA_MAZE_NO_JS_FALLBACK_MAX_DEPTH")
}

fn default_maze_micro_pow_enabled() -> bool {
    defaults_bool("SHUMA_MAZE_MICRO_POW_ENABLED")
}

fn default_maze_micro_pow_depth_start() -> u16 {
    defaults_u16("SHUMA_MAZE_MICRO_POW_DEPTH_START")
}

fn default_maze_micro_pow_base_difficulty() -> u8 {
    clamp_maze_micro_pow_difficulty(defaults_u8("SHUMA_MAZE_MICRO_POW_BASE_DIFFICULTY"))
}

fn default_maze_max_concurrent_global() -> u32 {
    defaults_u32("SHUMA_MAZE_MAX_CONCURRENT_GLOBAL")
}

fn default_maze_max_concurrent_per_ip_bucket() -> u32 {
    defaults_u32("SHUMA_MAZE_MAX_CONCURRENT_PER_IP_BUCKET")
}

fn default_maze_max_response_bytes() -> u32 {
    defaults_u32("SHUMA_MAZE_MAX_RESPONSE_BYTES")
}

fn default_maze_max_response_duration_ms() -> u64 {
    defaults_u64("SHUMA_MAZE_MAX_RESPONSE_DURATION_MS")
}

fn default_maze_server_visible_links() -> u32 {
    defaults_u32("SHUMA_MAZE_SERVER_VISIBLE_LINKS")
}

fn default_maze_max_links() -> u32 {
    defaults_u32("SHUMA_MAZE_MAX_LINKS")
}

fn default_maze_max_paragraphs() -> u32 {
    defaults_u32("SHUMA_MAZE_MAX_PARAGRAPHS")
}

fn default_maze_path_entropy_segment_len() -> u8 {
    defaults_u8("SHUMA_MAZE_PATH_ENTROPY_SEGMENT_LEN")
}

fn default_maze_covert_decoys_enabled() -> bool {
    defaults_bool("SHUMA_MAZE_COVERT_DECOYS_ENABLED")
}

fn default_maze_seed_provider() -> MazeSeedProvider {
    let raw = defaults_raw("SHUMA_MAZE_SEED_PROVIDER");
    parse_maze_seed_provider(raw.as_str()).unwrap_or_else(|| {
        panic!(
            "Invalid maze seed provider default for SHUMA_MAZE_SEED_PROVIDER={}",
            raw
        )
    })
}

fn default_maze_seed_refresh_interval_seconds() -> u64 {
    defaults_u64("SHUMA_MAZE_SEED_REFRESH_INTERVAL_SECONDS")
}

fn default_maze_seed_refresh_rate_limit_per_hour() -> u32 {
    defaults_u32("SHUMA_MAZE_SEED_REFRESH_RATE_LIMIT_PER_HOUR")
}

fn default_maze_seed_refresh_max_sources() -> u32 {
    defaults_u32("SHUMA_MAZE_SEED_REFRESH_MAX_SOURCES")
}

fn default_maze_seed_metadata_only() -> bool {
    defaults_bool("SHUMA_MAZE_SEED_METADATA_ONLY")
}

fn default_robots_enabled() -> bool {
    defaults_bool("SHUMA_ROBOTS_ENABLED")
}

fn default_robots_block_ai_training() -> bool {
    defaults_bool("SHUMA_ROBOTS_BLOCK_AI_TRAINING")
}

fn default_robots_block_ai_search() -> bool {
    defaults_bool("SHUMA_ROBOTS_BLOCK_AI_SEARCH")
}

fn default_robots_allow_search_engines() -> bool {
    defaults_bool("SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES")
}

fn default_robots_crawl_delay() -> u32 {
    defaults_u32("SHUMA_ROBOTS_CRAWL_DELAY")
}

fn default_cdp_detection_enabled() -> bool {
    defaults_bool("SHUMA_CDP_DETECTION_ENABLED")
}

fn default_cdp_auto_ban() -> bool {
    defaults_bool("SHUMA_CDP_AUTO_BAN")
}

fn default_cdp_threshold() -> f32 {
    defaults_f32("SHUMA_CDP_DETECTION_THRESHOLD")
}

fn default_cdp_probe_family() -> CdpProbeFamily {
    let raw = defaults_raw("SHUMA_CDP_PROBE_FAMILY");
    parse_cdp_probe_family(raw.as_str()).unwrap_or_else(|| {
        panic!(
            "Invalid CDP probe family default for SHUMA_CDP_PROBE_FAMILY={}",
            raw
        )
    })
}

fn default_cdp_probe_rollout_percent() -> u8 {
    defaults_u8("SHUMA_CDP_PROBE_ROLLOUT_PERCENT").clamp(0, 100)
}

fn default_fingerprint_signal_enabled() -> bool {
    defaults_bool("SHUMA_FINGERPRINT_SIGNAL_ENABLED")
}

fn default_fingerprint_state_ttl_seconds() -> u64 {
    defaults_u64("SHUMA_FINGERPRINT_STATE_TTL_SECONDS")
}

fn default_fingerprint_flow_window_seconds() -> u64 {
    defaults_u64("SHUMA_FINGERPRINT_FLOW_WINDOW_SECONDS")
}

fn default_fingerprint_flow_violation_threshold() -> u8 {
    defaults_u8("SHUMA_FINGERPRINT_FLOW_VIOLATION_THRESHOLD")
}

fn default_fingerprint_pseudonymize() -> bool {
    defaults_bool("SHUMA_FINGERPRINT_PSEUDONYMIZE")
}

fn default_fingerprint_entropy_budget() -> u8 {
    defaults_u8("SHUMA_FINGERPRINT_ENTROPY_BUDGET")
}

fn default_fingerprint_family_cap_header_runtime() -> u8 {
    defaults_u8("SHUMA_FINGERPRINT_FAMILY_CAP_HEADER_RUNTIME")
}

fn default_fingerprint_family_cap_transport() -> u8 {
    defaults_u8("SHUMA_FINGERPRINT_FAMILY_CAP_TRANSPORT")
}

fn default_fingerprint_family_cap_temporal() -> u8 {
    defaults_u8("SHUMA_FINGERPRINT_FAMILY_CAP_TEMPORAL")
}

fn default_fingerprint_family_cap_persistence() -> u8 {
    defaults_u8("SHUMA_FINGERPRINT_FAMILY_CAP_PERSISTENCE")
}

fn default_fingerprint_family_cap_behavior() -> u8 {
    defaults_u8("SHUMA_FINGERPRINT_FAMILY_CAP_BEHAVIOR")
}

fn default_js_required_enforced() -> bool {
    defaults_bool("SHUMA_JS_REQUIRED_ENFORCED")
}

fn default_verified_identity_enabled() -> bool {
    defaults_bool("SHUMA_VERIFIED_IDENTITY_ENABLED")
}

fn default_verified_identity_native_web_bot_auth_enabled() -> bool {
    defaults_bool("SHUMA_VERIFIED_IDENTITY_NATIVE_WEB_BOT_AUTH_ENABLED")
}

fn default_verified_identity_provider_assertions_enabled() -> bool {
    defaults_bool("SHUMA_VERIFIED_IDENTITY_PROVIDER_ASSERTIONS_ENABLED")
}

fn default_verified_identity_non_human_traffic_stance(
) -> crate::bot_identity::policy::NonHumanTrafficStance {
    let raw = defaults_raw("SHUMA_VERIFIED_IDENTITY_NON_HUMAN_TRAFFIC_STANCE");
    parse_non_human_traffic_stance(raw.as_str()).unwrap_or_else(|| {
        panic!(
            "Invalid verified identity non-human traffic stance default for SHUMA_VERIFIED_IDENTITY_NON_HUMAN_TRAFFIC_STANCE={}",
            raw
        )
    })
}

fn default_verified_identity_replay_window_seconds() -> u64 {
    defaults_u64("SHUMA_VERIFIED_IDENTITY_REPLAY_WINDOW_SECONDS")
}

fn default_verified_identity_clock_skew_seconds() -> u64 {
    defaults_u64("SHUMA_VERIFIED_IDENTITY_CLOCK_SKEW_SECONDS")
}

fn default_verified_identity_directory_cache_ttl_seconds() -> u64 {
    defaults_u64("SHUMA_VERIFIED_IDENTITY_DIRECTORY_CACHE_TTL_SECONDS")
}

fn default_verified_identity_directory_freshness_requirement_seconds() -> u64 {
    defaults_u64("SHUMA_VERIFIED_IDENTITY_DIRECTORY_FRESHNESS_REQUIREMENT_SECONDS")
}

fn default_verified_identity_named_policies(
) -> Vec<crate::bot_identity::policy::IdentityPolicyEntry> {
    defaults_json("SHUMA_VERIFIED_IDENTITY_NAMED_POLICIES")
}

fn default_verified_identity_category_defaults(
) -> Vec<crate::bot_identity::policy::IdentityCategoryDefaultAction> {
    defaults_json("SHUMA_VERIFIED_IDENTITY_CATEGORY_DEFAULTS")
}

fn default_verified_identity_service_profiles(
) -> Vec<crate::bot_identity::policy::IdentityServiceProfileBinding> {
    defaults_json("SHUMA_VERIFIED_IDENTITY_SERVICE_PROFILES")
}

fn default_pow_enabled() -> bool {
    defaults_bool("SHUMA_POW_ENABLED")
}

fn default_pow_difficulty() -> u8 {
    clamp_pow_difficulty(defaults_u8("SHUMA_POW_DIFFICULTY"))
}

fn default_pow_ttl_seconds() -> u64 {
    clamp_pow_ttl(defaults_u64("SHUMA_POW_TTL_SECONDS"))
}

fn default_challenge_puzzle_enabled() -> bool {
    defaults_bool("SHUMA_CHALLENGE_PUZZLE_ENABLED")
}

fn default_challenge_puzzle_transform_count() -> u8 {
    clamp_challenge_puzzle_transform_count(defaults_u8("SHUMA_CHALLENGE_PUZZLE_TRANSFORM_COUNT"))
}

fn default_challenge_puzzle_seed_ttl_seconds() -> u64 {
    clamp_challenge_puzzle_seed_ttl(defaults_u64("SHUMA_CHALLENGE_PUZZLE_SEED_TTL_SECONDS"))
}

fn default_challenge_puzzle_attempt_limit_per_window() -> u32 {
    clamp_challenge_puzzle_attempt_limit(defaults_u32(
        "SHUMA_CHALLENGE_PUZZLE_ATTEMPT_LIMIT_PER_WINDOW",
    ))
}

fn default_challenge_puzzle_attempt_window_seconds() -> u64 {
    clamp_challenge_puzzle_attempt_window(defaults_u64(
        "SHUMA_CHALLENGE_PUZZLE_ATTEMPT_WINDOW_SECONDS",
    ))
}

fn default_challenge_threshold() -> u8 {
    clamp_challenge_threshold(defaults_u8("SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD"))
}

fn default_not_a_bot_enabled() -> bool {
    defaults_bool("SHUMA_NOT_A_BOT_ENABLED")
}

fn default_not_a_bot_risk_threshold() -> u8 {
    clamp_challenge_threshold(defaults_u8("SHUMA_NOT_A_BOT_RISK_THRESHOLD"))
}

fn default_not_a_bot_pass_score() -> u8 {
    clamp_not_a_bot_score(defaults_u8("SHUMA_NOT_A_BOT_PASS_SCORE"))
}

fn default_not_a_bot_fail_score() -> u8 {
    clamp_not_a_bot_score(defaults_u8("SHUMA_NOT_A_BOT_FAIL_SCORE"))
}

fn default_not_a_bot_nonce_ttl_seconds() -> u64 {
    clamp_not_a_bot_nonce_ttl(defaults_u64("SHUMA_NOT_A_BOT_NONCE_TTL_SECONDS"))
}

fn default_not_a_bot_marker_ttl_seconds() -> u64 {
    clamp_not_a_bot_marker_ttl(defaults_u64("SHUMA_NOT_A_BOT_MARKER_TTL_SECONDS"))
}

fn default_not_a_bot_attempt_limit_per_window() -> u32 {
    clamp_not_a_bot_attempt_limit(defaults_u32("SHUMA_NOT_A_BOT_ATTEMPT_LIMIT_PER_WINDOW"))
}

fn default_not_a_bot_attempt_window_seconds() -> u64 {
    clamp_not_a_bot_attempt_window(defaults_u64("SHUMA_NOT_A_BOT_ATTEMPT_WINDOW_SECONDS"))
}

fn default_maze_threshold() -> u8 {
    clamp_maze_threshold(defaults_u8("SHUMA_BOTNESS_MAZE_THRESHOLD"))
}

fn default_botness_weight_js_required() -> u8 {
    clamp_botness_weight(defaults_u8("SHUMA_BOTNESS_WEIGHT_JS_REQUIRED"))
}

fn default_botness_weight_geo_risk() -> u8 {
    clamp_botness_weight(defaults_u8("SHUMA_BOTNESS_WEIGHT_GEO_RISK"))
}

fn default_botness_weight_rate_medium() -> u8 {
    clamp_botness_weight(defaults_u8("SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM"))
}

fn default_botness_weight_rate_high() -> u8 {
    clamp_botness_weight(defaults_u8("SHUMA_BOTNESS_WEIGHT_RATE_HIGH"))
}

fn default_botness_weight_maze_behavior() -> u8 {
    clamp_botness_weight(defaults_u8("SHUMA_BOTNESS_WEIGHT_MAZE_BEHAVIOR"))
}

fn defaults_composability_mode(key: &str) -> ComposabilityMode {
    let raw = defaults_raw(key);
    parse_composability_mode(raw.as_str())
        .unwrap_or_else(|| panic!("Invalid composability mode default for {}={}", key, raw))
}

fn default_mode_rate() -> ComposabilityMode {
    defaults_composability_mode("SHUMA_MODE_RATE")
}

fn default_mode_geo() -> ComposabilityMode {
    defaults_composability_mode("SHUMA_MODE_GEO")
}

fn default_mode_js() -> ComposabilityMode {
    defaults_composability_mode("SHUMA_MODE_JS")
}

fn defaults_provider_backend(key: &str) -> ProviderBackend {
    let raw = defaults_raw(key);
    parse_provider_backend(raw.as_str())
        .unwrap_or_else(|| panic!("Invalid provider backend default for {}={}", key, raw))
}

fn default_provider_rate_limiter() -> ProviderBackend {
    defaults_provider_backend("SHUMA_PROVIDER_RATE_LIMITER")
}

fn default_provider_ban_store() -> ProviderBackend {
    defaults_provider_backend("SHUMA_PROVIDER_BAN_STORE")
}

fn default_provider_challenge_engine() -> ProviderBackend {
    defaults_provider_backend("SHUMA_PROVIDER_CHALLENGE_ENGINE")
}

fn default_provider_maze_tarpit() -> ProviderBackend {
    defaults_provider_backend("SHUMA_PROVIDER_MAZE_TARPIT")
}

fn default_provider_fingerprint_signal() -> ProviderBackend {
    defaults_provider_backend("SHUMA_PROVIDER_FINGERPRINT_SIGNAL")
}

fn defaults_rate_limiter_outage_mode(key: &str) -> RateLimiterOutageMode {
    let raw = defaults_raw(key);
    parse_rate_limiter_outage_mode(raw.as_str()).unwrap_or_else(|| {
        panic!(
            "Invalid rate limiter outage mode default for {}={}",
            key, raw
        )
    })
}

fn default_rate_limiter_outage_mode_main() -> RateLimiterOutageMode {
    defaults_rate_limiter_outage_mode("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN")
}

fn default_rate_limiter_outage_mode_admin_auth() -> RateLimiterOutageMode {
    defaults_rate_limiter_outage_mode("SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH")
}

fn default_ban_store_outage_mode() -> BanStoreOutageMode {
    let raw = defaults_raw("SHUMA_BAN_STORE_OUTAGE_MODE");
    parse_ban_store_outage_mode(raw.as_str()).unwrap_or_else(|| {
        panic!(
            "Invalid ban store outage mode default for SHUMA_BAN_STORE_OUTAGE_MODE={}",
            raw
        )
    })
}

fn defaults_edge_integration_mode(key: &str) -> EdgeIntegrationMode {
    let raw = defaults_raw(key);
    parse_edge_integration_mode(raw.as_str())
        .unwrap_or_else(|| panic!("Invalid edge integration mode default for {}={}", key, raw))
}

fn default_edge_integration_mode() -> EdgeIntegrationMode {
    defaults_edge_integration_mode("SHUMA_EDGE_INTEGRATION_MODE")
}

#[cfg(test)]
mod tests;
