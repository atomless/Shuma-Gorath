use super::*;
use crate::challenge::KeyValueStore;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

#[derive(Default)]
struct CountingStore {
    map: Mutex<HashMap<String, Vec<u8>>>,
    get_count: AtomicUsize,
}

impl CountingStore {
    fn get_count(&self) -> usize {
        self.get_count.load(Ordering::SeqCst)
    }
}

impl KeyValueStore for CountingStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        self.get_count.fetch_add(1, Ordering::SeqCst);
        let map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(map.get(key).cloned())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        let mut map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), ()> {
        let mut map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        map.remove(key);
        Ok(())
    }
}

fn clear_env(keys: &[&str]) {
    for key in keys {
        std::env::remove_var(key);
    }
}

fn clear_gateway_env() {
    clear_env(&[
        "SHUMA_GATEWAY_UPSTREAM_ORIGIN",
        "SHUMA_GATEWAY_DEPLOYMENT_PROFILE",
        "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL",
        "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS",
        "SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST",
        "SHUMA_GATEWAY_PUBLIC_AUTHORITIES",
        "SHUMA_GATEWAY_LOOP_MAX_HOPS",
        "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED",
        "SHUMA_GATEWAY_ORIGIN_AUTH_MODE",
        "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME",
        "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE",
        "SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS",
        "SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS",
        "SHUMA_GATEWAY_TLS_STRICT",
        "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
    ]);
}

fn set_gateway_env_baseline() {
    std::env::set_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "https://origin.example.com");
    std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");
    std::env::set_var("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL", "false");
    std::env::set_var("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS", "false");
    std::env::set_var("SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST", "");
    std::env::set_var("SHUMA_GATEWAY_PUBLIC_AUTHORITIES", "shuma.example.com:443");
    std::env::set_var("SHUMA_GATEWAY_LOOP_MAX_HOPS", "3");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED", "true");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_MODE", "network_only");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME", "");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE", "");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS", "90");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS", "7");
    std::env::set_var("SHUMA_GATEWAY_TLS_STRICT", "true");
    std::env::set_var(
        "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
        "true",
    );
}

fn store_config_with_rate_limit(store: &CountingStore, rate_limit: u32) {
    let mut cfg = defaults().clone();
    cfg.rate_limit = rate_limit;
    store
        .set("config:default", &serde_json::to_vec(&cfg).unwrap())
        .unwrap();
}

#[test]
fn parse_challenge_threshold_defaults_to_3() {
    assert_eq!(parse_challenge_threshold(None), 3);
}

#[test]
fn parse_challenge_threshold_clamps_range() {
    assert_eq!(parse_challenge_threshold(Some("0")), 1);
    assert_eq!(parse_challenge_threshold(Some("99")), 10);
    assert_eq!(parse_challenge_threshold(Some("5")), 5);
    assert_eq!(parse_challenge_threshold(Some("junk")), 3);
}

#[test]
fn parse_maze_threshold_clamps_range() {
    assert_eq!(parse_maze_threshold(Some("0")), 1);
    assert_eq!(parse_maze_threshold(Some("99")), 10);
    assert_eq!(parse_maze_threshold(Some("6")), 6);
    assert_eq!(parse_maze_threshold(Some("junk")), 6);
}

#[test]
fn parse_botness_weight_clamps_range() {
    assert_eq!(parse_botness_weight(Some("0"), 3), 0);
    assert_eq!(parse_botness_weight(Some("11"), 3), 10);
    assert_eq!(parse_botness_weight(Some("4"), 3), 4);
    assert_eq!(parse_botness_weight(Some("junk"), 3), 3);
}

#[test]
fn parse_composability_mode_accepts_expected_values() {
    assert_eq!(
        parse_composability_mode("off"),
        Some(ComposabilityMode::Off)
    );
    assert_eq!(
        parse_composability_mode("signal"),
        Some(ComposabilityMode::Signal)
    );
    assert_eq!(
        parse_composability_mode("enforce"),
        Some(ComposabilityMode::Enforce)
    );
    assert_eq!(
        parse_composability_mode("both"),
        Some(ComposabilityMode::Both)
    );
    assert_eq!(
        parse_composability_mode("  BoTh "),
        Some(ComposabilityMode::Both)
    );
    assert_eq!(parse_composability_mode("invalid"), None);
    assert_eq!(ComposabilityMode::Off.as_str(), "off");
    assert_eq!(ComposabilityMode::Signal.as_str(), "signal");
    assert_eq!(ComposabilityMode::Enforce.as_str(), "enforce");
    assert_eq!(ComposabilityMode::Both.as_str(), "both");
}

#[test]
fn parse_ip_range_policy_mode_accepts_expected_values() {
    assert_eq!(
        parse_ip_range_policy_mode("off"),
        Some(IpRangePolicyMode::Off)
    );
    assert_eq!(
        parse_ip_range_policy_mode("advisory"),
        Some(IpRangePolicyMode::Advisory)
    );
    assert_eq!(
        parse_ip_range_policy_mode("enforce"),
        Some(IpRangePolicyMode::Enforce)
    );
    assert_eq!(
        parse_ip_range_policy_mode(" EnFoRcE "),
        Some(IpRangePolicyMode::Enforce)
    );
    assert_eq!(parse_ip_range_policy_mode("invalid"), None);
}

#[test]
fn parse_ip_range_policy_action_accepts_expected_values() {
    assert_eq!(
        parse_ip_range_policy_action("forbidden_403"),
        Some(IpRangePolicyAction::Forbidden403)
    );
    assert_eq!(
        parse_ip_range_policy_action("custom_message"),
        Some(IpRangePolicyAction::CustomMessage)
    );
    assert_eq!(
        parse_ip_range_policy_action("drop_connection"),
        Some(IpRangePolicyAction::DropConnection)
    );
    assert_eq!(
        parse_ip_range_policy_action("redirect_308"),
        Some(IpRangePolicyAction::Redirect308)
    );
    assert_eq!(
        parse_ip_range_policy_action("rate_limit"),
        Some(IpRangePolicyAction::RateLimit)
    );
    assert_eq!(
        parse_ip_range_policy_action("honeypot"),
        Some(IpRangePolicyAction::Honeypot)
    );
    assert_eq!(parse_ip_range_policy_action("maze"), Some(IpRangePolicyAction::Maze));
    assert_eq!(
        parse_ip_range_policy_action("tarpit"),
        Some(IpRangePolicyAction::Tarpit)
    );
    assert_eq!(parse_ip_range_policy_action("invalid"), None);
}

#[test]
fn parse_provider_backend_accepts_expected_values() {
    assert_eq!(
        parse_provider_backend("internal"),
        Some(ProviderBackend::Internal)
    );
    assert_eq!(
        parse_provider_backend("external"),
        Some(ProviderBackend::External)
    );
    assert_eq!(
        parse_provider_backend("  ExTeRnAl "),
        Some(ProviderBackend::External)
    );
    assert_eq!(parse_provider_backend("invalid"), None);
    assert_eq!(ProviderBackend::Internal.as_str(), "internal");
    assert_eq!(ProviderBackend::External.as_str(), "external");
}

#[test]
fn allowed_actions_v1_exposes_conservative_controller_write_surface() {
    let surface = allowed_actions_v1();
    assert_eq!(surface.schema_version, "allowed_actions_v1");
    assert_eq!(surface.write_surface, "admin_config");
    assert_eq!(surface.proposal_mode, "config_diff_only");
    assert!(surface
        .allowed_group_ids
        .contains(&"not_a_bot.policy".to_string()));
    assert!(surface
        .manual_only_group_ids
        .contains(&"shadow_mode.state".to_string()));
    assert!(surface
        .forbidden_group_ids
        .contains(&"provider_selection.backends".to_string()));

    let core_policy = surface
        .families
        .iter()
        .find(|family| family.family == "core_policy")
        .expect("core_policy family");
    assert_eq!(core_policy.controller_status, "mixed");
    assert!(core_policy
        .targets
        .contains(&"likely_human_friction".to_string()));

    let not_a_bot = surface
        .groups
        .iter()
        .find(|group| group.group_id == "not_a_bot.policy")
        .expect("not_a_bot policy group");
    assert_eq!(not_a_bot.controller_status, "allowed");
    assert_eq!(not_a_bot.canary_requirement, "required");
    assert!(not_a_bot
        .value_constraints
        .iter()
        .any(|constraint| constraint.path == "not_a_bot_risk_threshold"));
}

#[test]
fn controller_config_family_for_patch_key_reuses_allowed_action_catalog() {
    assert_eq!(
        controller_config_family_for_patch_key("js_required_enforced"),
        Some("core_policy")
    );
    assert_eq!(
        controller_config_family_for_patch_key("defence_modes"),
        Some("botness")
    );
    assert_eq!(
        controller_config_family_for_patch_key("edge_integration_mode"),
        Some("provider_selection")
    );
    assert_eq!(
        controller_config_family_for_patch_key("unknown_field"),
        None
    );
}

#[test]
fn parse_edge_integration_mode_accepts_expected_values() {
    assert_eq!(
        parse_edge_integration_mode("off"),
        Some(EdgeIntegrationMode::Off)
    );
    assert_eq!(
        parse_edge_integration_mode("additive"),
        Some(EdgeIntegrationMode::Additive)
    );
    assert_eq!(
        parse_edge_integration_mode("authoritative"),
        Some(EdgeIntegrationMode::Authoritative)
    );
    assert_eq!(
        parse_edge_integration_mode("  AuThOrItAtIvE "),
        Some(EdgeIntegrationMode::Authoritative)
    );
    assert_eq!(parse_edge_integration_mode("invalid"), None);
    assert_eq!(EdgeIntegrationMode::Off.as_str(), "off");
    assert_eq!(EdgeIntegrationMode::Additive.as_str(), "additive");
    assert_eq!(EdgeIntegrationMode::Authoritative.as_str(), "authoritative");
}

#[test]
fn parse_rate_limiter_outage_mode_accepts_expected_values() {
    assert_eq!(
        parse_rate_limiter_outage_mode("fallback_internal"),
        Some(RateLimiterOutageMode::FallbackInternal)
    );
    assert_eq!(
        parse_rate_limiter_outage_mode("fail_open"),
        Some(RateLimiterOutageMode::FailOpen)
    );
    assert_eq!(
        parse_rate_limiter_outage_mode("fail_closed"),
        Some(RateLimiterOutageMode::FailClosed)
    );
    assert_eq!(
        parse_rate_limiter_outage_mode("  FAIL_OPEN "),
        Some(RateLimiterOutageMode::FailOpen)
    );
    assert_eq!(parse_rate_limiter_outage_mode("invalid"), None);
    assert_eq!(
        RateLimiterOutageMode::FallbackInternal.as_str(),
        "fallback_internal"
    );
    assert_eq!(RateLimiterOutageMode::FailOpen.as_str(), "fail_open");
    assert_eq!(RateLimiterOutageMode::FailClosed.as_str(), "fail_closed");
}

#[test]
fn parse_tarpit_fallback_action_accepts_expected_values() {
    assert_eq!(
        parse_tarpit_fallback_action("maze"),
        Some(TarpitFallbackAction::Maze)
    );
    assert_eq!(
        parse_tarpit_fallback_action("block"),
        Some(TarpitFallbackAction::Block)
    );
    assert_eq!(
        parse_tarpit_fallback_action("  BLOCK "),
        Some(TarpitFallbackAction::Block)
    );
    assert_eq!(parse_tarpit_fallback_action("invalid"), None);
    assert_eq!(TarpitFallbackAction::Maze.as_str(), "maze");
    assert_eq!(TarpitFallbackAction::Block.as_str(), "block");
}

#[test]
fn parse_redis_url_accepts_expected_values() {
    assert_eq!(
        parse_redis_url("redis://localhost:6379"),
        Some("redis://localhost:6379".to_string())
    );
    assert_eq!(
        parse_redis_url(" rediss://cache.example:6379 "),
        Some("rediss://cache.example:6379".to_string())
    );
    assert_eq!(parse_redis_url("http://example.com"), None);
    assert_eq!(parse_redis_url(""), None);
}

#[test]
fn defaults_enable_both_signal_and_action_paths() {
    let cfg = defaults().clone();
    assert_eq!(cfg.edge_integration_mode, EdgeIntegrationMode::Off);
    assert!(cfg.js_required_enforced);
    assert!(cfg.honeypot_enabled);
    assert!(cfg.browser_policy_enabled);
    assert!(cfg.bypass_allowlists_enabled);
    assert!(cfg.path_allowlist_enabled);
    assert!(cfg.challenge_puzzle_enabled);
    assert_eq!(cfg.challenge_puzzle_seed_ttl_seconds, 300);
    assert_eq!(cfg.challenge_puzzle_attempt_limit_per_window, 4);
    assert_eq!(cfg.challenge_puzzle_attempt_window_seconds, 300);
    assert_eq!(cfg.not_a_bot_attempt_limit_per_window, 3);
    assert_eq!(cfg.not_a_bot_attempt_window_seconds, 300);
    assert_eq!(cfg.defence_modes.js, ComposabilityMode::Both);
    assert_eq!(cfg.defence_modes.geo, ComposabilityMode::Both);
    assert_eq!(cfg.defence_modes.rate, ComposabilityMode::Both);
    assert_eq!(cfg.ip_range_policy_mode, IpRangePolicyMode::Off);
    assert!(cfg.ip_range_emergency_allowlist.is_empty());
    assert!(cfg.ip_range_custom_rules.is_empty());
    assert_eq!(cfg.ip_range_suggestions_min_observations, 30);
    assert_eq!(cfg.ip_range_suggestions_min_bot_events, 8);
    assert_eq!(cfg.ip_range_suggestions_min_confidence_percent, 60);
    assert_eq!(cfg.ip_range_suggestions_low_collateral_percent, 10);
    assert_eq!(cfg.ip_range_suggestions_high_collateral_percent, 25);
    assert_eq!(cfg.ip_range_suggestions_ipv4_min_prefix_len, 24);
    assert_eq!(cfg.ip_range_suggestions_ipv6_min_prefix_len, 48);
    assert_eq!(cfg.ip_range_suggestions_likely_human_sample_percent, 10);
    assert!(cfg.tarpit_enabled);
    assert_eq!(cfg.tarpit_progress_token_ttl_seconds, 120);
    assert_eq!(cfg.tarpit_progress_replay_ttl_seconds, 300);
    assert_eq!(cfg.tarpit_hashcash_min_difficulty, 10);
    assert_eq!(cfg.tarpit_hashcash_max_difficulty, 16);
    assert_eq!(cfg.tarpit_hashcash_base_difficulty, 12);
    assert!(cfg.tarpit_hashcash_adaptive);
    assert_eq!(cfg.tarpit_step_chunk_base_bytes, 2048);
    assert_eq!(cfg.tarpit_step_chunk_max_bytes, 8192);
    assert_eq!(cfg.tarpit_step_jitter_percent, 15);
    assert!(cfg.tarpit_shard_rotation_enabled);
    assert_eq!(cfg.tarpit_egress_window_seconds, 60);
    assert_eq!(cfg.tarpit_egress_global_bytes_per_window, 4_194_304);
    assert_eq!(cfg.tarpit_egress_per_ip_bucket_bytes_per_window, 524_288);
    assert_eq!(cfg.tarpit_egress_per_flow_max_bytes, 262_144);
    assert_eq!(cfg.tarpit_egress_per_flow_max_duration_seconds, 120);
    assert_eq!(cfg.tarpit_max_concurrent_global, 64);
    assert_eq!(cfg.tarpit_max_concurrent_per_ip_bucket, 2);
    assert_eq!(cfg.tarpit_fallback_action, TarpitFallbackAction::Maze);
    assert!(cfg.rate_signal_enabled());
    assert!(cfg.rate_action_enabled());
    assert!(cfg.geo_signal_enabled());
    assert!(cfg.geo_action_enabled());
    assert!(cfg.js_signal_enabled());
    assert!(cfg.js_action_enabled());

    let effective = cfg.defence_modes_effective();
    assert!(effective.rate.signal_enabled);
    assert!(effective.rate.action_enabled);
    assert!(effective.geo.signal_enabled);
    assert!(effective.geo.action_enabled);
    assert!(effective.js.signal_enabled);
    assert!(effective.js.action_enabled);
    assert!(cfg.defence_mode_warnings().is_empty());
    assert_eq!(
        cfg.provider_backends.rate_limiter,
        ProviderBackend::Internal
    );
    assert_eq!(cfg.provider_backends.ban_store, ProviderBackend::Internal);
    assert_eq!(
        cfg.provider_backends.challenge_engine,
        ProviderBackend::Internal
    );
    assert_eq!(cfg.provider_backends.maze_tarpit, ProviderBackend::Internal);
    assert_eq!(
        cfg.provider_backends.fingerprint_signal,
        ProviderBackend::Internal
    );
}

#[test]
fn clamp_config_values_normalizes_ip_range_suggestion_bounds() {
    let mut cfg = defaults().clone();
    cfg.ip_range_suggestions_min_observations = 0;
    cfg.ip_range_suggestions_min_bot_events = 0;
    cfg.ip_range_suggestions_min_confidence_percent = 255;
    cfg.ip_range_suggestions_low_collateral_percent = 90;
    cfg.ip_range_suggestions_high_collateral_percent = 20;
    cfg.ip_range_suggestions_ipv4_min_prefix_len = 1;
    cfg.ip_range_suggestions_ipv6_min_prefix_len = 200;
    cfg.ip_range_suggestions_likely_human_sample_percent = 255;

    super::clamp_config_values(&mut cfg);

    assert_eq!(cfg.ip_range_suggestions_min_observations, 1);
    assert_eq!(cfg.ip_range_suggestions_min_bot_events, 1);
    assert_eq!(cfg.ip_range_suggestions_min_confidence_percent, 100);
    assert_eq!(cfg.ip_range_suggestions_low_collateral_percent, 20);
    assert_eq!(cfg.ip_range_suggestions_high_collateral_percent, 20);
    assert_eq!(cfg.ip_range_suggestions_ipv4_min_prefix_len, 8);
    assert_eq!(cfg.ip_range_suggestions_ipv6_min_prefix_len, 128);
    assert_eq!(cfg.ip_range_suggestions_likely_human_sample_percent, 100);
}

#[test]
fn enterprise_state_guardrail_errors_without_exception_for_unsynced_multi_instance() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");

    let cfg = defaults().clone();
    let error = cfg.enterprise_state_guardrail_error();
    assert!(error.is_some());
    assert!(error
        .unwrap()
        .contains("SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true"));
    assert!(cfg.enterprise_state_guardrail_warnings().is_empty());

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_warns_for_exceptioned_additive_unsynced_posture() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var(
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "true",
    );

    let mut cfg = defaults().clone();
    cfg.edge_integration_mode = EdgeIntegrationMode::Additive;
    assert_eq!(cfg.enterprise_state_guardrail_error(), None);
    let warnings = cfg.enterprise_state_guardrail_warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("explicit additive/off exception"));

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_errors_for_authoritative_unsynced_posture_even_with_exception() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var(
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "true",
    );

    let mut cfg = defaults().clone();
    cfg.edge_integration_mode = EdgeIntegrationMode::Authoritative;
    let error = cfg.enterprise_state_guardrail_error();
    assert!(error.is_some());
    assert!(error.unwrap().contains("authoritative mode"));
    assert!(cfg.enterprise_state_guardrail_warnings().is_empty());

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_is_clear_for_synced_multi_instance_posture() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var("SHUMA_RATE_LIMITER_REDIS_URL", "redis://redis:6379");
    std::env::set_var("SHUMA_BAN_STORE_REDIS_URL", "redis://redis:6379");

    let mut cfg = defaults().clone();
    cfg.provider_backends.rate_limiter = ProviderBackend::External;
    cfg.provider_backends.ban_store = ProviderBackend::External;
    cfg.edge_integration_mode = EdgeIntegrationMode::Authoritative;
    assert_eq!(cfg.enterprise_state_guardrail_error(), None);
    assert!(cfg.enterprise_state_guardrail_warnings().is_empty());

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_requires_redis_url_for_external_rate_limiter() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var(
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "true",
    );

    let mut cfg = defaults().clone();
    cfg.provider_backends.rate_limiter = ProviderBackend::External;
    cfg.edge_integration_mode = EdgeIntegrationMode::Additive;

    let error = cfg.enterprise_state_guardrail_error();
    assert!(error.is_some());
    assert!(error.unwrap().contains("SHUMA_RATE_LIMITER_REDIS_URL"));

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn enterprise_state_guardrail_requires_redis_url_for_external_ban_store() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "true");
    std::env::set_var(
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "true",
    );

    let mut cfg = defaults().clone();
    cfg.provider_backends.ban_store = ProviderBackend::External;
    cfg.edge_integration_mode = EdgeIntegrationMode::Additive;

    let error = cfg.enterprise_state_guardrail_error();
    assert!(error.is_some());
    assert!(error.unwrap().contains("SHUMA_BAN_STORE_REDIS_URL"));

    clear_env(&[
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn js_effective_mode_is_disabled_when_js_required_enforced_is_false() {
    let mut cfg = defaults().clone();
    cfg.js_required_enforced = false;
    cfg.defence_modes.js = ComposabilityMode::Both;

    assert!(!cfg.js_signal_enabled());
    assert!(!cfg.js_action_enabled());

    let effective = cfg.defence_modes_effective();
    assert_eq!(effective.js.configured, ComposabilityMode::Both);
    assert!(!effective.js.signal_enabled);
    assert!(!effective.js.action_enabled);
    assert!(effective.js.note.is_some());

    let warnings = cfg.defence_mode_warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("js_required_enforced=false"));
}

#[test]
fn parse_admin_config_write_defaults_to_enabled() {
    assert!(parse_admin_config_write_enabled(None));
    assert!(parse_admin_config_write_enabled(Some("junk")));
    assert!(parse_admin_config_write_enabled(Some("true")));
    assert!(parse_admin_config_write_enabled(Some("1")));
    assert!(!parse_admin_config_write_enabled(Some("false")));
}

#[test]
fn runtime_environment_defaults_to_runtime_prod_and_parses_runtime_dev() {
    let _lock = crate::test_support::lock_env();
    std::env::remove_var("SHUMA_RUNTIME_ENV");
    assert_eq!(runtime_environment(), RuntimeEnvironment::RuntimeProd);
    assert!(runtime_environment().is_prod());
    assert!(!runtime_environment().is_dev());

    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
    assert_eq!(runtime_environment(), RuntimeEnvironment::RuntimeDev);
    assert!(runtime_environment().is_dev());
    assert!(!runtime_environment().is_prod());

    std::env::remove_var("SHUMA_RUNTIME_ENV");
}

#[test]
fn adversary_sim_available_defaults_true_and_parses_bool_values() {
    let _lock = crate::test_support::lock_env();
    std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
    assert!(adversary_sim_available());

    std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");
    assert!(adversary_sim_available());

    std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "false");
    assert!(!adversary_sim_available());

    std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "invalid");
    assert!(adversary_sim_available());

    std::env::remove_var("SHUMA_ADVERSARY_SIM_AVAILABLE");
}

#[test]
fn parse_gateway_deployment_profile_accepts_expected_values() {
    assert_eq!(
        parse_gateway_deployment_profile("shared-server"),
        Some(GatewayDeploymentProfile::SharedServer)
    );
    assert_eq!(
        parse_gateway_deployment_profile("EDGE-FERMYON"),
        Some(GatewayDeploymentProfile::EdgeFermyon)
    );
    assert_eq!(parse_gateway_deployment_profile("invalid"), None);
}

#[test]
fn parse_gateway_origin_auth_mode_accepts_expected_values() {
    assert_eq!(
        parse_gateway_origin_auth_mode("network_only"),
        Some(GatewayOriginAuthMode::NetworkOnly)
    );
    assert_eq!(
        parse_gateway_origin_auth_mode("signed_header"),
        Some(GatewayOriginAuthMode::SignedHeader)
    );
    assert_eq!(parse_gateway_origin_auth_mode("invalid"), None);
}

#[test]
fn parse_gateway_upstream_origin_enforces_shape_and_canonicalization() {
    let https = super::parse_gateway_upstream_origin("https://Example.com").unwrap();
    assert_eq!(https.scheme, "https");
    assert_eq!(https.host, "example.com");
    assert_eq!(https.port, 443);
    assert_eq!(https.authority(), "example.com:443");

    let ipv6 = super::parse_gateway_upstream_origin("http://[::1]:8080").unwrap();
    assert_eq!(ipv6.host, "::1");
    assert_eq!(ipv6.port, 8080);
    assert_eq!(ipv6.authority(), "[::1]:8080");

    assert!(super::parse_gateway_upstream_origin("https://origin.example.com/path").is_err());
    assert!(super::parse_gateway_upstream_origin("ftp://origin.example.com").is_err());
}

#[test]
fn frontier_summary_defaults_to_disabled_without_provider_keys() {
    let _lock = crate::test_support::lock_env();
    std::env::remove_var("SHUMA_FRONTIER_OPENAI_API_KEY");
    std::env::remove_var("SHUMA_FRONTIER_ANTHROPIC_API_KEY");
    std::env::remove_var("SHUMA_FRONTIER_GOOGLE_API_KEY");
    std::env::remove_var("SHUMA_FRONTIER_XAI_API_KEY");
    std::env::remove_var("SHUMA_FRONTIER_OPENAI_MODEL");
    std::env::remove_var("SHUMA_FRONTIER_ANTHROPIC_MODEL");
    std::env::remove_var("SHUMA_FRONTIER_GOOGLE_MODEL");
    std::env::remove_var("SHUMA_FRONTIER_XAI_MODEL");

    let summary = frontier_summary();
    assert_eq!(summary.mode, "disabled");
    assert_eq!(summary.provider_count, 0);
    assert_eq!(summary.diversity_confidence, "none");
    assert!(!summary.reduced_diversity_warning);
    assert_eq!(summary.providers.len(), 4);
    assert_eq!(summary.providers[0].provider, "openai");
    assert_eq!(summary.providers[0].model_id, "gpt-5-mini");
    assert!(!summary.providers[0].configured);
}

#[test]
fn frontier_summary_tracks_single_vs_multi_provider_modes() {
    let _lock = crate::test_support::lock_env();
    std::env::remove_var("SHUMA_FRONTIER_OPENAI_API_KEY");
    std::env::remove_var("SHUMA_FRONTIER_ANTHROPIC_API_KEY");
    std::env::remove_var("SHUMA_FRONTIER_GOOGLE_API_KEY");
    std::env::remove_var("SHUMA_FRONTIER_XAI_API_KEY");
    std::env::set_var("SHUMA_FRONTIER_OPENAI_API_KEY", "test-openai-key");
    std::env::set_var("SHUMA_FRONTIER_OPENAI_MODEL", "gpt-custom-fast");

    let single = frontier_summary();
    assert_eq!(single.mode, "single_provider_self_play");
    assert_eq!(single.provider_count, 1);
    assert_eq!(single.diversity_confidence, "low");
    assert!(single.reduced_diversity_warning);
    assert_eq!(single.providers[0].model_id, "gpt-custom-fast");
    assert!(single.providers[0].configured);

    std::env::set_var("SHUMA_FRONTIER_ANTHROPIC_API_KEY", "test-anthropic-key");
    let multi = frontier_summary();
    assert_eq!(multi.mode, "multi_provider_playoff");
    assert_eq!(multi.provider_count, 2);
    assert_eq!(multi.diversity_confidence, "higher");
    assert!(!multi.reduced_diversity_warning);

    std::env::remove_var("SHUMA_FRONTIER_OPENAI_API_KEY");
    std::env::remove_var("SHUMA_FRONTIER_ANTHROPIC_API_KEY");
    std::env::remove_var("SHUMA_FRONTIER_OPENAI_MODEL");
}

#[test]
fn adversary_sim_duration_defaults_to_180_and_clamps_loaded_values() {
    let store = CountingStore::default();
    let mut cfg = defaults().clone();
    assert_eq!(cfg.adversary_sim_duration_seconds, 180);

    cfg.adversary_sim_duration_seconds = 5;
    store
        .set("config:default", &serde_json::to_vec(&cfg).unwrap())
        .unwrap();
    let loaded_low = Config::load(&store, "default").unwrap();
    assert_eq!(
        loaded_low.adversary_sim_duration_seconds,
        ADVERSARY_SIM_DURATION_SECONDS_MIN
    );

    cfg.adversary_sim_duration_seconds = 9_999;
    store
        .set("config:default", &serde_json::to_vec(&cfg).unwrap())
        .unwrap();
    let loaded_high = Config::load(&store, "default").unwrap();
    assert_eq!(
        loaded_high.adversary_sim_duration_seconds,
        ADVERSARY_SIM_DURATION_SECONDS_MAX
    );
}

#[test]
fn validate_env_rejects_invalid_optional_runtime_environment() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
        "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "invalid-runtime");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("SHUMA_RUNTIME_ENV"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);
}

#[test]
fn validate_env_accepts_sim_available_in_runtime_prod_when_gateway_contract_is_satisfied() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
        "SHUMA_ADVERSARY_SIM_AVAILABLE",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
    std::env::set_var("SHUMA_ADVERSARY_SIM_AVAILABLE", "true");
    set_gateway_env_baseline();

    let result = validate_env_only_once();
    assert!(result.is_ok());

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
        "SHUMA_ADVERSARY_SIM_AVAILABLE",
    ]);
}

#[test]
fn validate_env_rejects_frontier_model_id_with_whitespace() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_FRONTIER_OPENAI_MODEL",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_FRONTIER_OPENAI_MODEL", "gpt 5 mini");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("SHUMA_FRONTIER_OPENAI_MODEL"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_FRONTIER_OPENAI_MODEL",
    ]);
}

#[test]
fn validate_env_rejects_invalid_optional_enterprise_bool() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_ENTERPRISE_MULTI_INSTANCE", "definitely-not-bool");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("SHUMA_ENTERPRISE_MULTI_INSTANCE"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn validate_env_rejects_invalid_optional_redis_url() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RATE_LIMITER_REDIS_URL", "https://not-redis.example");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("SHUMA_RATE_LIMITER_REDIS_URL"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn validate_env_rejects_invalid_optional_ban_store_redis_url() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_BAN_STORE_REDIS_URL", "https://not-redis.example");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("SHUMA_BAN_STORE_REDIS_URL"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn validate_env_rejects_invalid_optional_rate_limiter_outage_mode() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN", "invalid-mode");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN"));

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);
}

#[test]
fn validate_env_accepts_empty_optional_redis_url() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
    std::env::set_var("SHUMA_RATE_LIMITER_REDIS_URL", "");

    let result = validate_env_only_once();
    assert!(result.is_ok());

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_ENTERPRISE_MULTI_INSTANCE",
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        "SHUMA_RATE_LIMITER_REDIS_URL",
        "SHUMA_BAN_STORE_REDIS_URL",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
        "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
        "SHUMA_RUNTIME_ENV",
    ]);
    clear_gateway_env();
}

#[test]
fn validate_env_rejects_missing_gateway_upstream_in_runtime_prod() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_LOCAL_PROD_DIRECT_MODE",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
    set_gateway_env_baseline();
    std::env::set_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("SHUMA_GATEWAY_UPSTREAM_ORIGIN must be set"));

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_LOCAL_PROD_DIRECT_MODE",
        "SHUMA_RUNTIME_ENV",
    ]);
}

#[test]
fn validate_env_accepts_runtime_prod_local_direct_mode_without_gateway_upstream() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_LOCAL_PROD_DIRECT_MODE",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
    std::env::set_var("SHUMA_LOCAL_PROD_DIRECT_MODE", "true");
    set_gateway_env_baseline();
    std::env::set_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "");

    let result = validate_env_only_once();
    assert!(result.is_ok());

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_LOCAL_PROD_DIRECT_MODE",
        "SHUMA_RUNTIME_ENV",
    ]);
}

#[test]
fn validate_env_rejects_insecure_public_http_upstream() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
    set_gateway_env_baseline();
    std::env::set_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "http://8.8.8.8:8080");
    std::env::set_var("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL", "true");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("must be loopback/private"));

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);
}

#[test]
fn validate_env_rejects_edge_profile_without_signed_header_origin_auth() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
    std::env::set_var("SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET", "edge-cron-secret");
    set_gateway_env_baseline();
    std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_MODE", "network_only");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("requires SHUMA_GATEWAY_ORIGIN_AUTH_MODE=signed_header"));

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
        "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET",
    ]);
}

#[test]
fn validate_env_rejects_gateway_public_authority_loop_collision() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
        "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
    set_gateway_env_baseline();
    std::env::set_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "https://origin.example.com");
    std::env::set_var("SHUMA_GATEWAY_PUBLIC_AUTHORITIES", "origin.example.com:443");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("must not match SHUMA_GATEWAY_PUBLIC_AUTHORITIES"));

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);
}

#[test]
fn validate_env_rejects_runtime_prod_when_route_collision_attestation_missing() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
    set_gateway_env_baseline();
    std::env::set_var(
        "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
        "false",
    );

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result.err().unwrap().contains(
        "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED must be true when SHUMA_RUNTIME_ENV=runtime-prod"
    ));

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);
}

#[test]
fn validate_env_accepts_edge_profile_with_signed_header_origin_auth_contract() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
    std::env::set_var("SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET", "edge-cron-secret");
    set_gateway_env_baseline();
    std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_MODE", "signed_header");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME", "x-origin-auth");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE", "edge-shared-secret");
    std::env::set_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "https://origin.example.com:443");

    let result = validate_env_only_once();
    assert!(result.is_ok());

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
        "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET",
    ]);
}

#[test]
fn validate_env_rejects_edge_profile_without_adversary_sim_edge_cron_secret() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
        "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-prod");
    set_gateway_env_baseline();
    std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_MODE", "signed_header");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME", "x-origin-auth");
    std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE", "edge-shared-secret");
    std::env::set_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "https://origin.example.com:443");

    let result = validate_env_only_once();
    assert!(result.is_err());
    assert!(result
        .err()
        .unwrap()
        .contains("SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET"));

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
        "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET",
    ]);
}

#[test]
fn validate_env_accepts_runtime_dev_without_gateway_upstream() {
    let _lock = crate::test_support::lock_env();
    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");

    let result = validate_env_only_once();
    assert!(result.is_ok());

    clear_gateway_env();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);
}

#[test]
fn https_enforced_reads_required_env_bool() {
    let _lock = crate::test_support::lock_env();
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    assert!(!https_enforced());

    std::env::set_var("SHUMA_ENFORCE_HTTPS", "true");
    assert!(https_enforced());

    std::env::remove_var("SHUMA_ENFORCE_HTTPS");
}

#[test]
fn forwarded_header_trust_configured_requires_non_empty_secret() {
    let _lock = crate::test_support::lock_env();
    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    assert!(!forwarded_header_trust_configured());

    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "");
    assert!(!forwarded_header_trust_configured());

    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-secret");
    assert!(forwarded_header_trust_configured());

    std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
}

#[test]
fn load_config_missing_returns_error() {
    let _lock = crate::test_support::lock_env();
    let store = crate::test_support::InMemoryStore::default();
    let result = Config::load(&store, "default");
    assert!(matches!(result, Err(ConfigLoadError::MissingConfig)));
}

#[test]
fn load_config_reads_kv_only_without_tunable_env_overrides() {
    let _lock = crate::test_support::lock_env();
    let keys = ["SHUMA_RATE_LIMIT", "SHUMA_HONEYPOTS"];
    clear_env(&keys);
    std::env::set_var("SHUMA_RATE_LIMIT", "222");
    std::env::set_var("SHUMA_HONEYPOTS", "[\"/trap-a\",\"/trap-b\"]");

    let store = crate::test_support::InMemoryStore::default();
    let mut kv_cfg = defaults().clone();
    kv_cfg.rate_limit = 111;
    kv_cfg.honeypots = vec!["/kv-trap".to_string()];
    store
        .set("config:default", &serde_json::to_vec(&kv_cfg).unwrap())
        .unwrap();

    let cfg = Config::load(&store, "default").unwrap();
    assert_eq!(cfg.rate_limit, 111);
    assert_eq!(cfg.honeypots, vec!["/kv-trap".to_string()]);

    clear_env(&keys);
}

#[test]
fn load_config_defaults_honeypot_enabled_when_key_missing() {
    let _lock = crate::test_support::lock_env();
    let store = crate::test_support::InMemoryStore::default();
    let mut kv_cfg_value = serde_json::to_value(defaults().clone()).unwrap();
    kv_cfg_value
        .as_object_mut()
        .expect("config json object")
        .remove("honeypot_enabled");
    store
        .set("config:default", &serde_json::to_vec(&kv_cfg_value).unwrap())
        .unwrap();

    let cfg = Config::load(&store, "default").unwrap();
    assert!(cfg.honeypot_enabled);
}

#[test]
fn runtime_config_cache_hits_within_ttl() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    let store = CountingStore::default();
    store_config_with_rate_limit(&store, 101);

    let first = load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
    let second = load_runtime_cached_for_tests(&store, "default", 101, 2).unwrap();

    assert_eq!(first.rate_limit, 101);
    assert_eq!(second.rate_limit, 101);
    assert_eq!(store.get_count(), 1);
    clear_runtime_cache_for_tests();
}

#[test]
fn runtime_config_cache_refreshes_after_ttl() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    let store = CountingStore::default();
    store_config_with_rate_limit(&store, 111);

    let _ = load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
    let _ = load_runtime_cached_for_tests(&store, "default", 103, 2).unwrap();

    assert_eq!(store.get_count(), 2);
    clear_runtime_cache_for_tests();
}

#[test]
fn runtime_config_cache_invalidation_forces_reload() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    let store = CountingStore::default();
    store_config_with_rate_limit(&store, 120);

    let first = load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
    assert_eq!(first.rate_limit, 120);
    assert_eq!(store.get_count(), 1);

    store_config_with_rate_limit(&store, 220);
    invalidate_runtime_cache("default");

    let refreshed = load_runtime_cached_for_tests(&store, "default", 101, 2).unwrap();

    assert_eq!(refreshed.rate_limit, 220);
    assert_eq!(store.get_count(), 2);
    clear_runtime_cache_for_tests();
}

#[test]
fn runtime_ephemeral_overrides_apply_without_persisting_to_kv() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    clear_env(&["SHUMA_SHADOW_MODE", "SHUMA_ADVERSARY_SIM_ENABLED"]);

    let store = crate::test_support::InMemoryStore::default();
    let mut persisted = defaults().clone();
    persisted.shadow_mode = false;
    persisted.adversary_sim_enabled = false;
    store
        .set("config:default", &serde_json::to_vec(&persisted).unwrap())
        .unwrap();

    let initial = load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
    assert!(!initial.shadow_mode);
    assert!(!initial.adversary_sim_enabled);

    set_runtime_shadow_mode_override("default", true);

    let effective = load_runtime_cached_for_tests(&store, "default", 101, 2).unwrap();
    assert!(effective.shadow_mode);
    assert!(!effective.adversary_sim_enabled);

    let raw = Config::load(&store, "default").unwrap();
    assert!(!raw.shadow_mode);
    assert!(!raw.adversary_sim_enabled);
    clear_runtime_cache_for_tests();
}

#[test]
fn runtime_ephemeral_defaults_honor_env_startup_overrides() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    std::env::set_var("SHUMA_SHADOW_MODE", "true");
    std::env::set_var("SHUMA_ADVERSARY_SIM_ENABLED", "true");

    let store = crate::test_support::InMemoryStore::default();
    let mut persisted = defaults().clone();
    persisted.shadow_mode = false;
    persisted.adversary_sim_enabled = false;
    store
        .set("config:default", &serde_json::to_vec(&persisted).unwrap())
        .unwrap();

    let effective = load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
    assert!(effective.shadow_mode);
    assert!(!effective.adversary_sim_enabled);

    std::env::remove_var("SHUMA_SHADOW_MODE");
    std::env::remove_var("SHUMA_ADVERSARY_SIM_ENABLED");
    clear_runtime_cache_for_tests();
}

#[test]
fn runtime_adversary_sim_enablement_uses_persisted_seeded_state_once_config_exists() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    clear_env(&["SHUMA_ADVERSARY_SIM_ENABLED"]);

    let store = crate::test_support::InMemoryStore::default();
    let mut persisted = defaults().clone();
    persisted.adversary_sim_enabled = true;
    store
        .set("config:default", &serde_json::to_vec(&persisted).unwrap())
        .unwrap();

    let effective = load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
    assert!(effective.adversary_sim_enabled);

    std::env::set_var("SHUMA_ADVERSARY_SIM_ENABLED", "true");
    clear_runtime_cache_for_tests();
    persisted.adversary_sim_enabled = false;
    store
        .set("config:default", &serde_json::to_vec(&persisted).unwrap())
        .unwrap();
    let env_effective = load_runtime_cached_for_tests(&store, "default", 101, 2).unwrap();
    assert!(!env_effective.adversary_sim_enabled);
    std::env::remove_var("SHUMA_ADVERSARY_SIM_ENABLED");
    clear_runtime_cache_for_tests();
}

#[test]
fn validate_env_only_accepts_spin_variables_in_tests() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    clear_test_spin_variables();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    set_test_spin_variable("SHUMA_API_KEY", "spin-admin-key");
    set_test_spin_variable("SHUMA_JS_SECRET", "spin-js-secret");
    set_test_spin_variable("SHUMA_FORWARDED_IP_SECRET", "spin-forwarded-secret");
    set_test_spin_variable("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    set_test_spin_variable("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
    set_test_spin_variable("SHUMA_KV_STORE_FAIL_OPEN", "true");
    set_test_spin_variable("SHUMA_ENFORCE_HTTPS", "false");
    set_test_spin_variable("SHUMA_DEBUG_HEADERS", "false");
    set_test_spin_variable("SHUMA_RUNTIME_ENV", "runtime-dev");

    let result = validate_env_only_once();

    std::env::remove_var("SHUMA_VALIDATE_ENV_IN_TESTS");
    clear_test_spin_variables();
    assert!(result.is_ok(), "expected spin variables to satisfy env validation, got {result:?}");
}

#[test]
fn validate_env_only_rejects_invalid_monitoring_retention_hours() {
    let _lock = crate::test_support::lock_env();
    clear_runtime_cache_for_tests();
    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_MONITORING_RETENTION_HOURS",
        "SHUMA_MONITORING_ROLLUP_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);

    std::env::set_var("SHUMA_VALIDATE_ENV_IN_TESTS", "true");
    std::env::set_var("SHUMA_API_KEY", "test-admin-key");
    std::env::set_var("SHUMA_JS_SECRET", "test-js-secret");
    std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
    std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "168");
    std::env::set_var("SHUMA_MONITORING_RETENTION_HOURS", "not-a-number");
    std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false");
    std::env::set_var("SHUMA_KV_STORE_FAIL_OPEN", "true");
    std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
    std::env::set_var("SHUMA_DEBUG_HEADERS", "false");
    std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");

    let result = validate_env_only_once();

    clear_env(&[
        "SHUMA_VALIDATE_ENV_IN_TESTS",
        "SHUMA_API_KEY",
        "SHUMA_JS_SECRET",
        "SHUMA_FORWARDED_IP_SECRET",
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_MONITORING_RETENTION_HOURS",
        "SHUMA_MONITORING_ROLLUP_RETENTION_HOURS",
        "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
        "SHUMA_KV_STORE_FAIL_OPEN",
        "SHUMA_ENFORCE_HTTPS",
        "SHUMA_DEBUG_HEADERS",
        "SHUMA_RUNTIME_ENV",
    ]);
    assert_eq!(
        result,
        Err("Invalid integer env var SHUMA_MONITORING_RETENTION_HOURS=not-a-number".to_string())
    );
}

#[test]
fn env_u64_defaulted_uses_defaults_when_env_is_missing() {
    let _lock = crate::test_support::lock_env();
    clear_env(&[
        "SHUMA_EVENT_LOG_RETENTION_HOURS",
        "SHUMA_MONITORING_RETENTION_HOURS",
        "SHUMA_MONITORING_ROLLUP_RETENTION_HOURS",
    ]);

    assert_eq!(env_u64_defaulted("SHUMA_EVENT_LOG_RETENTION_HOURS"), 168);
    assert_eq!(env_u64_defaulted("SHUMA_MONITORING_RETENTION_HOURS"), 168);
    assert_eq!(env_u64_defaulted("SHUMA_MONITORING_ROLLUP_RETENTION_HOURS"), 720);
}

#[test]
fn env_string_required_uses_spin_variable_when_env_missing() {
    let _lock = crate::test_support::lock_env();
    clear_test_spin_variables();
    std::env::remove_var("SHUMA_API_KEY");
    set_test_spin_variable("SHUMA_API_KEY", "spin-admin-key");

    let value = env_string_required("SHUMA_API_KEY");

    clear_test_spin_variables();
    assert_eq!(value, "spin-admin-key");
}

#[test]
fn default_seeded_config_matches_defaults_snapshot() {
    let cfg = default_seeded_config();
    let defaults_cfg = defaults().clone();

    assert_eq!(cfg.rate_limit, defaults_cfg.rate_limit);
    assert_eq!(cfg.js_required_enforced, defaults_cfg.js_required_enforced);
    assert_eq!(cfg.challenge_puzzle_enabled, defaults_cfg.challenge_puzzle_enabled);
    assert_eq!(cfg.not_a_bot_enabled, defaults_cfg.not_a_bot_enabled);
    assert_eq!(cfg.maze_enabled, defaults_cfg.maze_enabled);
    assert_eq!(cfg.provider_backends.rate_limiter, defaults_cfg.provider_backends.rate_limiter);
    assert_eq!(cfg.edge_integration_mode, defaults_cfg.edge_integration_mode);
}
