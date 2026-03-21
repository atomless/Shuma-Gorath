use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;

const OPERATOR_SNAPSHOT_RECENT_CHANGES_SCHEMA_VERSION: &str = "operator_snapshot_recent_changes.v1";
const OPERATOR_SNAPSHOT_RECENT_CHANGES_MAX_ROWS: usize = 24;
const OPERATOR_SNAPSHOT_RECENT_CHANGES_SUMMARY_MAX_CHARS: usize = 240;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotRecentChangeLedgerRow {
    pub(crate) changed_at_ts: u64,
    pub(crate) change_reason: String,
    pub(crate) changed_families: Vec<String>,
    pub(crate) source: String,
    pub(crate) targets: Vec<String>,
    pub(crate) change_summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) decision_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OperatorSnapshotRecentChangeLedgerState {
    schema_version: String,
    updated_at_ts: u64,
    rows: Vec<OperatorSnapshotRecentChangeLedgerRow>,
}

impl Default for OperatorSnapshotRecentChangeLedgerState {
    fn default() -> Self {
        Self {
            schema_version: OPERATOR_SNAPSHOT_RECENT_CHANGES_SCHEMA_VERSION.to_string(),
            updated_at_ts: 0,
            rows: Vec::new(),
        }
    }
}

fn operator_snapshot_recent_changes_state_key(site_id: &str) -> String {
    format!("operator_snapshot:recent_changes:v1:{}", site_id)
}

fn load_operator_snapshot_recent_changes_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
) -> OperatorSnapshotRecentChangeLedgerState {
    let key = operator_snapshot_recent_changes_state_key(site_id);
    store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|value| {
            serde_json::from_slice::<OperatorSnapshotRecentChangeLedgerState>(value.as_slice()).ok()
        })
        .unwrap_or_default()
}

fn save_operator_snapshot_recent_changes_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    state: &OperatorSnapshotRecentChangeLedgerState,
) {
    let key = operator_snapshot_recent_changes_state_key(site_id);
    let Ok(payload) = serde_json::to_vec(state) else {
        eprintln!(
            "[operator-snapshot] failed serializing recent change ledger site={}",
            site_id
        );
        return;
    };
    if store.set(key.as_str(), payload.as_slice()).is_err() {
        eprintln!(
            "[operator-snapshot] failed persisting recent change ledger site={}",
            site_id
        );
    }
}

fn truncate_operator_snapshot_change_summary(summary: &str) -> String {
    if summary.chars().count() <= OPERATOR_SNAPSHOT_RECENT_CHANGES_SUMMARY_MAX_CHARS {
        return summary.to_string();
    }
    summary
        .chars()
        .take(OPERATOR_SNAPSHOT_RECENT_CHANGES_SUMMARY_MAX_CHARS.saturating_sub(3))
        .collect::<String>()
        + "..."
}

pub(crate) fn record_operator_snapshot_recent_change_rows<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    rows: &[OperatorSnapshotRecentChangeLedgerRow],
    updated_at_ts: u64,
) {
    if rows.is_empty() {
        return;
    }

    let mut state = load_operator_snapshot_recent_changes_state(store, site_id);
    for row in rows.iter().cloned() {
        state.rows.push(row);
    }
    state.rows.sort_by(|left, right| {
        right
            .changed_at_ts
            .cmp(&left.changed_at_ts)
            .then_with(|| left.change_reason.cmp(&right.change_reason))
            .then_with(|| left.change_summary.cmp(&right.change_summary))
    });
    state
        .rows
        .truncate(OPERATOR_SNAPSHOT_RECENT_CHANGES_MAX_ROWS);
    state.updated_at_ts = updated_at_ts;
    state.schema_version = OPERATOR_SNAPSHOT_RECENT_CHANGES_SCHEMA_VERSION.to_string();
    save_operator_snapshot_recent_changes_state(store, site_id, &state);
}

pub(crate) fn load_operator_snapshot_recent_changes<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    watch_window_hours: u64,
    max_rows: usize,
) -> (
    crate::observability::operator_snapshot::OperatorSnapshotRecentChanges,
    u64,
) {
    let state = load_operator_snapshot_recent_changes_state(store, site_id);
    let decision_map = crate::observability::decision_ledger::load_recent_decision_map(store, site_id);
    let watch_window_seconds = watch_window_hours.saturating_mul(3600);
    let lookback_seconds = watch_window_seconds
        .saturating_mul(3)
        .max(watch_window_seconds);
    let lookback_start_ts = generated_at_ts.saturating_sub(lookback_seconds.saturating_sub(1));
    let rows = state
        .rows
        .iter()
        .filter(|row| row.changed_at_ts >= lookback_start_ts)
        .take(max_rows)
        .map(|row| operator_snapshot_change_from_ledger_row(row, decision_map.get(row.decision_id.as_deref().unwrap_or_default()), generated_at_ts, watch_window_seconds))
        .collect();

    (
        crate::observability::operator_snapshot::OperatorSnapshotRecentChanges {
            lookback_seconds,
            watch_window_seconds,
            rows,
        },
        if state.updated_at_ts == 0 {
            generated_at_ts
        } else {
            state.updated_at_ts
        },
    )
}

fn operator_snapshot_change_from_ledger_row(
    row: &OperatorSnapshotRecentChangeLedgerRow,
    decision: Option<&crate::observability::decision_ledger::OperatorDecisionRecord>,
    generated_at_ts: u64,
    default_watch_window_seconds: u64,
) -> crate::observability::operator_snapshot::OperatorSnapshotRecentChange {
    let watch_window_seconds = decision
        .map(|decision| decision.watch_window_seconds)
        .filter(|seconds| *seconds > 0)
        .unwrap_or(default_watch_window_seconds);
    let elapsed = generated_at_ts.saturating_sub(row.changed_at_ts);
    let bounded_elapsed = elapsed.min(watch_window_seconds);
    let remaining = watch_window_seconds.saturating_sub(bounded_elapsed);
    let watch_window_status = if remaining == 0 {
        "watch_window_complete"
    } else {
        "collecting_post_change_window"
    };
    crate::observability::operator_snapshot::OperatorSnapshotRecentChange {
        changed_at_ts: row.changed_at_ts,
        change_reason: row.change_reason.clone(),
        changed_families: row.changed_families.clone(),
        source: row.source.clone(),
        targets: row.targets.clone(),
        decision_id: row.decision_id.clone(),
        decision_kind: decision.map(|decision| decision.decision_kind.clone()),
        decision_status: decision.map(|decision| decision.decision_status.clone()),
        objective_revision: decision.map(|decision| decision.objective_revision.clone()),
        expected_impact_summary: decision
            .map(|decision| decision.expected_impact_summary.clone()),
        evidence_references: decision
            .map(|decision| decision.evidence_references.clone())
            .unwrap_or_default(),
        watch_window_status: watch_window_status.to_string(),
        watch_window_elapsed_seconds: bounded_elapsed,
        watch_window_remaining_seconds: remaining,
        change_summary: row.change_summary.clone(),
    }
}

fn operator_snapshot_recent_change_source(admin_id: &str) -> String {
    if admin_id.starts_with("controller:") || admin_id == "scheduled_controller" {
        "scheduled_controller".to_string()
    } else {
        "manual_admin".to_string()
    }
}

fn operator_snapshot_patch_requested_families(patch: &serde_json::Value) -> Vec<&'static str> {
    let Some(object) = patch.as_object() else {
        return Vec::new();
    };
    let mut families = Vec::new();
    for key in object.keys() {
        let family = crate::config::controller_config_family_for_patch_key(key.as_str());
        if let Some(family) = family {
            if !families.contains(&family) {
                families.push(family);
            }
        }
    }
    families
}

fn operator_snapshot_family_snapshot(
    cfg: &crate::config::Config,
    family: &str,
) -> serde_json::Value {
    match family {
        "shadow_mode" => json!({
            "shadow_mode": cfg.shadow_mode,
        }),
        "adversary_sim_config" => json!({
            "adversary_sim_duration_seconds": cfg.adversary_sim_duration_seconds,
        }),
        "core_policy" => json!({
            "ban_duration": cfg.ban_duration,
            "ban_durations": cfg.ban_durations,
            "rate_limit": cfg.rate_limit,
            "js_required_enforced": cfg.js_required_enforced,
        }),
        "geo_policy" => json!({
            "geo_risk": cfg.geo_risk,
            "geo_allow": cfg.geo_allow,
            "geo_challenge": cfg.geo_challenge,
            "geo_maze": cfg.geo_maze,
            "geo_block": cfg.geo_block,
            "geo_edge_headers_enabled": cfg.geo_edge_headers_enabled,
        }),
        "honeypot" => json!({
            "honeypot_enabled": cfg.honeypot_enabled,
            "honeypots": cfg.honeypots,
        }),
        "browser_policy" => json!({
            "browser_policy_enabled": cfg.browser_policy_enabled,
            "browser_block": cfg.browser_block,
            "browser_allowlist": cfg.browser_allowlist,
        }),
        "allowlists" => json!({
            "bypass_allowlists_enabled": cfg.bypass_allowlists_enabled,
            "allowlist": cfg.allowlist,
            "path_allowlist_enabled": cfg.path_allowlist_enabled,
            "path_allowlist": cfg.path_allowlist,
        }),
        "ip_range_policy" => json!({
            "ip_range_policy_mode": cfg.ip_range_policy_mode,
            "ip_range_emergency_allowlist": cfg.ip_range_emergency_allowlist,
            "ip_range_custom_rules": cfg.ip_range_custom_rules,
            "ip_range_suggestions_min_observations": cfg.ip_range_suggestions_min_observations,
            "ip_range_suggestions_min_bot_events": cfg.ip_range_suggestions_min_bot_events,
            "ip_range_suggestions_min_confidence_percent": cfg.ip_range_suggestions_min_confidence_percent,
            "ip_range_suggestions_low_collateral_percent": cfg.ip_range_suggestions_low_collateral_percent,
            "ip_range_suggestions_high_collateral_percent": cfg.ip_range_suggestions_high_collateral_percent,
            "ip_range_suggestions_ipv4_min_prefix_len": cfg.ip_range_suggestions_ipv4_min_prefix_len,
            "ip_range_suggestions_ipv6_min_prefix_len": cfg.ip_range_suggestions_ipv6_min_prefix_len,
            "ip_range_suggestions_likely_human_sample_percent": cfg.ip_range_suggestions_likely_human_sample_percent,
        }),
        "maze_core" => json!({
            "maze_enabled": cfg.maze_enabled,
            "maze_auto_ban": cfg.maze_auto_ban,
            "maze_auto_ban_threshold": cfg.maze_auto_ban_threshold,
            "maze_rollout_phase": cfg.maze_rollout_phase,
            "maze_token_ttl_seconds": cfg.maze_token_ttl_seconds,
            "maze_token_max_depth": cfg.maze_token_max_depth,
            "maze_token_branch_budget": cfg.maze_token_branch_budget,
            "maze_replay_ttl_seconds": cfg.maze_replay_ttl_seconds,
            "maze_entropy_window_seconds": cfg.maze_entropy_window_seconds,
            "maze_client_expansion_enabled": cfg.maze_client_expansion_enabled,
            "maze_checkpoint_every_nodes": cfg.maze_checkpoint_every_nodes,
            "maze_checkpoint_every_ms": cfg.maze_checkpoint_every_ms,
            "maze_step_ahead_max": cfg.maze_step_ahead_max,
            "maze_no_js_fallback_max_depth": cfg.maze_no_js_fallback_max_depth,
            "maze_micro_pow_enabled": cfg.maze_micro_pow_enabled,
            "maze_micro_pow_depth_start": cfg.maze_micro_pow_depth_start,
            "maze_micro_pow_base_difficulty": cfg.maze_micro_pow_base_difficulty,
            "maze_max_concurrent_global": cfg.maze_max_concurrent_global,
            "maze_max_concurrent_per_ip_bucket": cfg.maze_max_concurrent_per_ip_bucket,
            "maze_max_response_bytes": cfg.maze_max_response_bytes,
            "maze_max_response_duration_ms": cfg.maze_max_response_duration_ms,
            "maze_server_visible_links": cfg.maze_server_visible_links,
            "maze_max_links": cfg.maze_max_links,
            "maze_max_paragraphs": cfg.maze_max_paragraphs,
            "maze_path_entropy_segment_len": cfg.maze_path_entropy_segment_len,
            "maze_covert_decoys_enabled": cfg.maze_covert_decoys_enabled,
            "maze_seed_provider": cfg.maze_seed_provider,
            "maze_seed_refresh_interval_seconds": cfg.maze_seed_refresh_interval_seconds,
            "maze_seed_refresh_rate_limit_per_hour": cfg.maze_seed_refresh_rate_limit_per_hour,
            "maze_seed_refresh_max_sources": cfg.maze_seed_refresh_max_sources,
            "maze_seed_metadata_only": cfg.maze_seed_metadata_only,
        }),
        "tarpit" => json!({
            "tarpit_enabled": cfg.tarpit_enabled,
            "tarpit_progress_token_ttl_seconds": cfg.tarpit_progress_token_ttl_seconds,
            "tarpit_progress_replay_ttl_seconds": cfg.tarpit_progress_replay_ttl_seconds,
            "tarpit_hashcash_min_difficulty": cfg.tarpit_hashcash_min_difficulty,
            "tarpit_hashcash_max_difficulty": cfg.tarpit_hashcash_max_difficulty,
            "tarpit_hashcash_base_difficulty": cfg.tarpit_hashcash_base_difficulty,
            "tarpit_hashcash_adaptive": cfg.tarpit_hashcash_adaptive,
            "tarpit_step_chunk_base_bytes": cfg.tarpit_step_chunk_base_bytes,
            "tarpit_step_chunk_max_bytes": cfg.tarpit_step_chunk_max_bytes,
            "tarpit_step_jitter_percent": cfg.tarpit_step_jitter_percent,
            "tarpit_shard_rotation_enabled": cfg.tarpit_shard_rotation_enabled,
            "tarpit_egress_window_seconds": cfg.tarpit_egress_window_seconds,
            "tarpit_egress_global_bytes_per_window": cfg.tarpit_egress_global_bytes_per_window,
            "tarpit_egress_per_ip_bucket_bytes_per_window": cfg.tarpit_egress_per_ip_bucket_bytes_per_window,
            "tarpit_egress_per_flow_max_bytes": cfg.tarpit_egress_per_flow_max_bytes,
            "tarpit_egress_per_flow_max_duration_seconds": cfg.tarpit_egress_per_flow_max_duration_seconds,
            "tarpit_max_concurrent_global": cfg.tarpit_max_concurrent_global,
            "tarpit_max_concurrent_per_ip_bucket": cfg.tarpit_max_concurrent_per_ip_bucket,
            "tarpit_fallback_action": cfg.tarpit_fallback_action,
        }),
        "proof_of_work" => json!({
            "pow_enabled": cfg.pow_enabled,
            "pow_difficulty": cfg.pow_difficulty,
            "pow_ttl_seconds": cfg.pow_ttl_seconds,
        }),
        "challenge" => json!({
            "challenge_puzzle_enabled": cfg.challenge_puzzle_enabled,
            "challenge_puzzle_transform_count": cfg.challenge_puzzle_transform_count,
            "challenge_puzzle_seed_ttl_seconds": cfg.challenge_puzzle_seed_ttl_seconds,
            "challenge_puzzle_attempt_limit_per_window": cfg.challenge_puzzle_attempt_limit_per_window,
            "challenge_puzzle_attempt_window_seconds": cfg.challenge_puzzle_attempt_window_seconds,
        }),
        "not_a_bot" => json!({
            "not_a_bot_enabled": cfg.not_a_bot_enabled,
            "not_a_bot_risk_threshold": cfg.not_a_bot_risk_threshold,
            "not_a_bot_pass_score": cfg.not_a_bot_pass_score,
            "not_a_bot_fail_score": cfg.not_a_bot_fail_score,
            "not_a_bot_nonce_ttl_seconds": cfg.not_a_bot_nonce_ttl_seconds,
            "not_a_bot_marker_ttl_seconds": cfg.not_a_bot_marker_ttl_seconds,
            "not_a_bot_attempt_limit_per_window": cfg.not_a_bot_attempt_limit_per_window,
            "not_a_bot_attempt_window_seconds": cfg.not_a_bot_attempt_window_seconds,
        }),
        "provider_selection" => json!({
            "provider_backends": cfg.provider_backends,
            "edge_integration_mode": cfg.edge_integration_mode,
        }),
        "verified_identity" => json!({
            "verified_identity": cfg.verified_identity,
        }),
        "botness" => json!({
            "challenge_puzzle_risk_threshold": cfg.challenge_puzzle_risk_threshold,
            "botness_maze_threshold": cfg.botness_maze_threshold,
            "botness_weights": cfg.botness_weights,
            "defence_modes": cfg.defence_modes,
        }),
        "robots_policy" => json!({
            "robots_enabled": cfg.robots_enabled,
            "ai_policy_block_training": cfg.robots_block_ai_training,
            "ai_policy_block_search": cfg.robots_block_ai_search,
            "ai_policy_allow_search_engines": cfg.robots_allow_search_engines,
            "robots_crawl_delay": cfg.robots_crawl_delay,
        }),
        "cdp_detection" => json!({
            "cdp_detection_enabled": cfg.cdp_detection_enabled,
            "cdp_auto_ban": cfg.cdp_auto_ban,
            "cdp_detection_threshold": cfg.cdp_detection_threshold,
            "cdp_probe_family": cfg.cdp_probe_family,
            "cdp_probe_rollout_percent": cfg.cdp_probe_rollout_percent,
        }),
        "fingerprint_signal" => json!({
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
            "fingerprint_family_cap_behavior": cfg.fingerprint_family_cap_behavior,
        }),
        _ => serde_json::Value::Null,
    }
}

fn operator_snapshot_patch_changed_families(
    old_cfg: &crate::config::Config,
    new_cfg: &crate::config::Config,
    patch: &serde_json::Value,
) -> Vec<String> {
    let mut families = operator_snapshot_patch_requested_families(patch)
        .into_iter()
        .filter(|family| {
            operator_snapshot_family_snapshot(old_cfg, family)
                != operator_snapshot_family_snapshot(new_cfg, family)
        })
        .map(|family| family.to_string())
        .collect::<Vec<_>>();
    families.sort();
    families
}

fn operator_snapshot_targets_for_families(families: &[String]) -> Vec<String> {
    let mut targets = BTreeSet::new();
    for family in families {
        for target in crate::config::controller_action_family_targets(family.as_str()) {
            targets.insert(target);
        }
    }
    targets.into_iter().collect()
}

pub(crate) fn operator_snapshot_config_patch_recent_change_row(
    old_cfg: &crate::config::Config,
    new_cfg: &crate::config::Config,
    patch: &serde_json::Value,
    admin_id: &str,
    changed_at_ts: u64,
) -> Option<OperatorSnapshotRecentChangeLedgerRow> {
    let changed_families = operator_snapshot_patch_changed_families(old_cfg, new_cfg, patch);
    if changed_families.is_empty() {
        return None;
    }
    Some(OperatorSnapshotRecentChangeLedgerRow {
        changed_at_ts,
        change_reason: "config_patch".to_string(),
        changed_families: changed_families.clone(),
        source: operator_snapshot_recent_change_source(admin_id),
        targets: operator_snapshot_targets_for_families(&changed_families),
        change_summary: truncate_operator_snapshot_change_summary(
            format!("config families updated: {}", changed_families.join(", ")).as_str(),
        ),
        decision_id: None,
    })
}

pub(crate) fn operator_snapshot_manual_change_row(
    changed_at_ts: u64,
    change_reason: &str,
    changed_families: &[&str],
    targets: &[&str],
    admin_id: &str,
    change_summary: &str,
) -> OperatorSnapshotRecentChangeLedgerRow {
    OperatorSnapshotRecentChangeLedgerRow {
        changed_at_ts,
        change_reason: change_reason.to_string(),
        changed_families: changed_families
            .iter()
            .map(|family| family.to_string())
            .collect(),
        source: operator_snapshot_recent_change_source(admin_id),
        targets: targets.iter().map(|target| target.to_string()).collect(),
        change_summary: truncate_operator_snapshot_change_summary(change_summary),
        decision_id: None,
    }
}

pub(crate) fn operator_snapshot_recent_change_with_decision_id(
    row: &OperatorSnapshotRecentChangeLedgerRow,
    decision_id: &str,
) -> OperatorSnapshotRecentChangeLedgerRow {
    let mut updated = row.clone();
    updated.decision_id = Some(decision_id.to_string());
    updated
}
