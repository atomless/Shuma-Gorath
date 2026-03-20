use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) const ALLOWED_ACTIONS_SCHEMA_VERSION: &str = "allowed_actions_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionValueConstraint {
    pub path: String,
    pub value_kind: String,
    pub min_inclusive: Option<f64>,
    pub max_inclusive: Option<f64>,
    pub allowed_values: Vec<String>,
    pub rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionGroup {
    pub group_id: String,
    pub family: String,
    pub controller_status: String,
    pub canary_requirement: String,
    pub patch_paths: Vec<String>,
    pub targets: Vec<String>,
    pub value_constraints: Vec<AllowedActionValueConstraint>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionFamily {
    pub family: String,
    pub controller_status: String,
    pub group_ids: Vec<String>,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionsSurface {
    pub schema_version: String,
    pub write_surface: String,
    pub proposal_mode: String,
    pub groups: Vec<AllowedActionGroup>,
    pub families: Vec<AllowedActionFamily>,
    pub allowed_group_ids: Vec<String>,
    pub manual_only_group_ids: Vec<String>,
    pub forbidden_group_ids: Vec<String>,
}

struct AllowedActionValueConstraintDefinition {
    path: &'static str,
    value_kind: &'static str,
    min_inclusive: Option<f64>,
    max_inclusive: Option<f64>,
    allowed_values: &'static [&'static str],
    rule: Option<&'static str>,
}

struct AllowedActionGroupDefinition {
    group_id: &'static str,
    family: &'static str,
    controller_status: &'static str,
    canary_requirement: &'static str,
    patch_paths: &'static [&'static str],
    targets: &'static [&'static str],
    value_constraints: &'static [AllowedActionValueConstraintDefinition],
    note: &'static str,
}

const BOOLEAN_VALUES: &[&str] = &["false", "true"];
const MAZE_ROLLOUT_VALUES: &[&str] = &["instrument", "advisory", "enforce"];

const CORE_POLICY_JS_REQUIRED_CONSTRAINTS: &[AllowedActionValueConstraintDefinition] = &[AllowedActionValueConstraintDefinition {
    path: "js_required_enforced",
    value_kind: "bool",
    min_inclusive: None,
    max_inclusive: None,
    allowed_values: BOOLEAN_VALUES,
    rule: Some("Controller proposals that change human-facing friction toggles must be canary-reviewed before apply."),
}];

const MAZE_ROLLOUT_CONSTRAINTS: &[AllowedActionValueConstraintDefinition] = &[
    AllowedActionValueConstraintDefinition {
        path: "maze_enabled",
        value_kind: "bool",
        min_inclusive: None,
        max_inclusive: None,
        allowed_values: BOOLEAN_VALUES,
        rule: Some("Controller proposals that enable or disable enforcement surfaces must be canary-reviewed before apply."),
    },
    AllowedActionValueConstraintDefinition {
        path: "maze_auto_ban",
        value_kind: "bool",
        min_inclusive: None,
        max_inclusive: None,
        allowed_values: BOOLEAN_VALUES,
        rule: Some("Automatic ban posture changes must remain bounded to canary-reviewed config diffs."),
    },
    AllowedActionValueConstraintDefinition {
        path: "maze_rollout_phase",
        value_kind: "enum",
        min_inclusive: None,
        max_inclusive: None,
        allowed_values: MAZE_ROLLOUT_VALUES,
        rule: Some("Rollout-phase changes must stay within the explicit instrument/advisory/enforce progression."),
    },
];

const PROOF_OF_WORK_CONSTRAINTS: &[AllowedActionValueConstraintDefinition] = &[
    AllowedActionValueConstraintDefinition {
        path: "pow_enabled",
        value_kind: "bool",
        min_inclusive: None,
        max_inclusive: None,
        allowed_values: BOOLEAN_VALUES,
        rule: Some("PoW enablement changes must be canary-reviewed because they directly affect human friction."),
    },
    AllowedActionValueConstraintDefinition {
        path: "pow_difficulty",
        value_kind: "u8",
        min_inclusive: Some(POW_DIFFICULTY_MIN as f64),
        max_inclusive: Some(POW_DIFFICULTY_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "pow_ttl_seconds",
        value_kind: "u64",
        min_inclusive: Some(POW_TTL_MIN as f64),
        max_inclusive: Some(POW_TTL_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
];

const CHALLENGE_CONSTRAINTS: &[AllowedActionValueConstraintDefinition] = &[
    AllowedActionValueConstraintDefinition {
        path: "challenge_puzzle_enabled",
        value_kind: "bool",
        min_inclusive: None,
        max_inclusive: None,
        allowed_values: BOOLEAN_VALUES,
        rule: Some("Challenge-surface enablement changes must be canary-reviewed before apply."),
    },
    AllowedActionValueConstraintDefinition {
        path: "challenge_puzzle_transform_count",
        value_kind: "u8",
        min_inclusive: Some(CHALLENGE_TRANSFORM_COUNT_MIN as f64),
        max_inclusive: Some(CHALLENGE_TRANSFORM_COUNT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "challenge_puzzle_seed_ttl_seconds",
        value_kind: "u64",
        min_inclusive: Some(CHALLENGE_PUZZLE_SEED_TTL_MIN as f64),
        max_inclusive: Some(CHALLENGE_PUZZLE_SEED_TTL_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "challenge_puzzle_attempt_limit_per_window",
        value_kind: "u32",
        min_inclusive: Some(CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MIN as f64),
        max_inclusive: Some(CHALLENGE_PUZZLE_ATTEMPT_LIMIT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "challenge_puzzle_attempt_window_seconds",
        value_kind: "u64",
        min_inclusive: Some(CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MIN as f64),
        max_inclusive: Some(CHALLENGE_PUZZLE_ATTEMPT_WINDOW_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
];

const NOT_A_BOT_CONSTRAINTS: &[AllowedActionValueConstraintDefinition] = &[
    AllowedActionValueConstraintDefinition {
        path: "not_a_bot_enabled",
        value_kind: "bool",
        min_inclusive: None,
        max_inclusive: None,
        allowed_values: BOOLEAN_VALUES,
        rule: Some("Not-a-bot enablement changes must be canary-reviewed before apply."),
    },
    AllowedActionValueConstraintDefinition {
        path: "not_a_bot_risk_threshold",
        value_kind: "u8",
        min_inclusive: Some(CHALLENGE_THRESHOLD_MIN as f64),
        max_inclusive: Some(CHALLENGE_THRESHOLD_MAX as f64),
        allowed_values: &[],
        rule: Some("When challenge_puzzle_risk_threshold is enabled, not_a_bot_risk_threshold must stay below it."),
    },
    AllowedActionValueConstraintDefinition {
        path: "not_a_bot_pass_score",
        value_kind: "u8",
        min_inclusive: Some(NOT_A_BOT_SCORE_MIN as f64),
        max_inclusive: Some(NOT_A_BOT_SCORE_MAX as f64),
        allowed_values: &[],
        rule: Some("not_a_bot_fail_score must not exceed not_a_bot_pass_score."),
    },
    AllowedActionValueConstraintDefinition {
        path: "not_a_bot_fail_score",
        value_kind: "u8",
        min_inclusive: Some(NOT_A_BOT_SCORE_MIN as f64),
        max_inclusive: Some(NOT_A_BOT_SCORE_MAX as f64),
        allowed_values: &[],
        rule: Some("not_a_bot_fail_score must not exceed not_a_bot_pass_score."),
    },
    AllowedActionValueConstraintDefinition {
        path: "not_a_bot_nonce_ttl_seconds",
        value_kind: "u64",
        min_inclusive: Some(NOT_A_BOT_NONCE_TTL_MIN as f64),
        max_inclusive: Some(NOT_A_BOT_NONCE_TTL_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "not_a_bot_marker_ttl_seconds",
        value_kind: "u64",
        min_inclusive: Some(NOT_A_BOT_MARKER_TTL_MIN as f64),
        max_inclusive: Some(NOT_A_BOT_MARKER_TTL_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "not_a_bot_attempt_limit_per_window",
        value_kind: "u32",
        min_inclusive: Some(NOT_A_BOT_ATTEMPT_LIMIT_MIN as f64),
        max_inclusive: Some(NOT_A_BOT_ATTEMPT_LIMIT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "not_a_bot_attempt_window_seconds",
        value_kind: "u64",
        min_inclusive: Some(NOT_A_BOT_ATTEMPT_WINDOW_MIN as f64),
        max_inclusive: Some(NOT_A_BOT_ATTEMPT_WINDOW_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
];

const BOTNESS_CONSTRAINTS: &[AllowedActionValueConstraintDefinition] = &[
    AllowedActionValueConstraintDefinition {
        path: "challenge_puzzle_risk_threshold",
        value_kind: "u8",
        min_inclusive: Some(CHALLENGE_THRESHOLD_MIN as f64),
        max_inclusive: Some(CHALLENGE_THRESHOLD_MAX as f64),
        allowed_values: &[],
        rule: Some("challenge_puzzle_risk_threshold must remain above not_a_bot_risk_threshold when both are active."),
    },
    AllowedActionValueConstraintDefinition {
        path: "botness_maze_threshold",
        value_kind: "u8",
        min_inclusive: Some(MAZE_THRESHOLD_MIN as f64),
        max_inclusive: Some(MAZE_THRESHOLD_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "botness_weights.js_required",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "botness_weights.geo_risk",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "botness_weights.rate_medium",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "botness_weights.rate_high",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "botness_weights.maze_behavior",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
];

const CDP_POLICY_CONSTRAINTS: &[AllowedActionValueConstraintDefinition] = &[
    AllowedActionValueConstraintDefinition {
        path: "cdp_detection_enabled",
        value_kind: "bool",
        min_inclusive: None,
        max_inclusive: None,
        allowed_values: BOOLEAN_VALUES,
        rule: Some("CDP-surface enablement changes must be canary-reviewed before apply."),
    },
    AllowedActionValueConstraintDefinition {
        path: "cdp_auto_ban",
        value_kind: "bool",
        min_inclusive: None,
        max_inclusive: None,
        allowed_values: BOOLEAN_VALUES,
        rule: Some("Automatic-ban posture changes must be canary-reviewed before apply."),
    },
    AllowedActionValueConstraintDefinition {
        path: "cdp_detection_threshold",
        value_kind: "f32",
        min_inclusive: Some(0.0),
        max_inclusive: Some(1.0),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "cdp_probe_rollout_percent",
        value_kind: "u8",
        min_inclusive: Some(0.0),
        max_inclusive: Some(100.0),
        allowed_values: &[],
        rule: None,
    },
];

const FINGERPRINT_POLICY_CONSTRAINTS: &[AllowedActionValueConstraintDefinition] = &[
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_signal_enabled",
        value_kind: "bool",
        min_inclusive: None,
        max_inclusive: None,
        allowed_values: BOOLEAN_VALUES,
        rule: Some("Fingerprint-signal enablement changes must be canary-reviewed before apply."),
    },
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_state_ttl_seconds",
        value_kind: "u64",
        min_inclusive: Some(30.0),
        max_inclusive: Some((24 * 3600) as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_flow_window_seconds",
        value_kind: "u64",
        min_inclusive: Some(10.0),
        max_inclusive: Some(3600.0),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_flow_violation_threshold",
        value_kind: "u8",
        min_inclusive: Some(1.0),
        max_inclusive: Some(20.0),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_entropy_budget",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_family_cap_header_runtime",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_family_cap_transport",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_family_cap_temporal",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_family_cap_persistence",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
    AllowedActionValueConstraintDefinition {
        path: "fingerprint_family_cap_behavior",
        value_kind: "u8",
        min_inclusive: Some(BOTNESS_WEIGHT_MIN as f64),
        max_inclusive: Some(BOTNESS_WEIGHT_MAX as f64),
        allowed_values: &[],
        rule: None,
    },
];

const ALLOWED_ACTION_GROUP_DEFINITIONS: &[AllowedActionGroupDefinition] = &[
    AllowedActionGroupDefinition {
        group_id: "shadow_mode.state",
        family: "shadow_mode",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &["shadow_mode"],
        targets: &[],
        value_constraints: &[],
        note: "Execution-mode switches must remain a deliberate manual operator decision.",
    },
    AllowedActionGroupDefinition {
        group_id: "adversary_sim.duration",
        family: "adversary_sim_config",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &["adversary_sim_duration_seconds"],
        targets: &["representative_adversary_effectiveness"],
        value_constraints: &[],
        note: "Adversary-sim scheduling and runtime posture must remain manual until the benchmark loop is defined end to end.",
    },
    AllowedActionGroupDefinition {
        group_id: "core_policy.js_required_toggle",
        family: "core_policy",
        controller_status: "allowed",
        canary_requirement: "required",
        patch_paths: &["js_required_enforced"],
        targets: &["likely_human_friction", "suspicious_forwarded_requests"],
        value_constraints: CORE_POLICY_JS_REQUIRED_CONSTRAINTS,
        note: "Controller may propose bounded JS-required posture changes because they directly shape likely-human friction and suspicious leakage.",
    },
    AllowedActionGroupDefinition {
        group_id: "core_policy.rate_limit",
        family: "core_policy",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &["rate_limit"],
        targets: &["suspicious_forwarded_requests"],
        value_constraints: &[],
        note: "Rate-limit thresholds do not yet expose a dedicated safe numeric envelope, so they remain manual-only.",
    },
    AllowedActionGroupDefinition {
        group_id: "core_policy.ban_durations",
        family: "core_policy",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &["ban_duration", "ban_durations"],
        targets: &["suspicious_forwarded_requests"],
        value_constraints: &[],
        note: "Ban-duration posture remains manual-only because it changes punishment horizons rather than bounded detection sensitivity.",
    },
    AllowedActionGroupDefinition {
        group_id: "geo_policy.country_lists",
        family: "geo_policy",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &[
            "geo_risk",
            "geo_allow",
            "geo_challenge",
            "geo_maze",
            "geo_block",
            "geo_edge_headers_enabled",
        ],
        targets: &["likely_human_friction", "suspicious_forwarded_requests"],
        value_constraints: &[],
        note: "Country and edge-header posture remains manual-only because it is site- and deployment-specific trust policy.",
    },
    AllowedActionGroupDefinition {
        group_id: "honeypot.surface",
        family: "honeypot",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &["honeypot_enabled", "honeypots"],
        targets: &["suspicious_forwarded_requests"],
        value_constraints: &[],
        note: "Honeypot paths and surface placement remain manual-only because they depend on the protected site's navigation model.",
    },
    AllowedActionGroupDefinition {
        group_id: "browser_policy.allowlists",
        family: "browser_policy",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &["browser_policy_enabled", "browser_block", "browser_allowlist"],
        targets: &["likely_human_friction", "suspicious_forwarded_requests"],
        value_constraints: &[],
        note: "Browser allowlists and deny rules remain manual-only because they sit on a trust-boundary surface.",
    },
    AllowedActionGroupDefinition {
        group_id: "allowlists.surface",
        family: "allowlists",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &[
            "bypass_allowlists_enabled",
            "allowlist",
            "path_allowlist_enabled",
            "path_allowlist",
        ],
        targets: &["likely_human_friction", "suspicious_forwarded_requests"],
        value_constraints: &[],
        note: "Emergency and path allowlists remain manual-only because they are explicit trust exceptions.",
    },
    AllowedActionGroupDefinition {
        group_id: "ip_range_policy",
        family: "ip_range_policy",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &[
            "ip_range_policy_mode",
            "ip_range_emergency_allowlist",
            "ip_range_custom_rules",
            "ip_range_suggestions_min_observations",
            "ip_range_suggestions_min_bot_events",
            "ip_range_suggestions_min_confidence_percent",
            "ip_range_suggestions_low_collateral_percent",
            "ip_range_suggestions_high_collateral_percent",
            "ip_range_suggestions_ipv4_min_prefix_len",
            "ip_range_suggestions_ipv6_min_prefix_len",
            "ip_range_suggestions_likely_human_sample_percent",
        ],
        targets: &["likely_human_friction", "suspicious_forwarded_requests"],
        value_constraints: &[],
        note: "IP-range policy remains manual-only because it carries high collateral-risk and trust-boundary implications.",
    },
    AllowedActionGroupDefinition {
        group_id: "maze_core.rollout",
        family: "maze_core",
        controller_status: "allowed",
        canary_requirement: "required",
        patch_paths: &["maze_enabled", "maze_auto_ban", "maze_rollout_phase"],
        targets: &["suspicious_forwarded_bytes", "suspicious_forwarded_requests"],
        value_constraints: MAZE_ROLLOUT_CONSTRAINTS,
        note: "Controller may propose bounded maze rollout posture changes, but only through the explicit canary-required rollout surface.",
    },
    AllowedActionGroupDefinition {
        group_id: "maze_core.content_and_budget",
        family: "maze_core",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &[
            "maze_auto_ban_threshold",
            "maze_token_ttl_seconds",
            "maze_token_max_depth",
            "maze_token_branch_budget",
            "maze_replay_ttl_seconds",
            "maze_entropy_window_seconds",
            "maze_client_expansion_enabled",
            "maze_checkpoint_every_nodes",
            "maze_checkpoint_every_ms",
            "maze_step_ahead_max",
            "maze_no_js_fallback_max_depth",
            "maze_micro_pow_enabled",
            "maze_micro_pow_depth_start",
            "maze_micro_pow_base_difficulty",
            "maze_max_concurrent_global",
            "maze_max_concurrent_per_ip_bucket",
            "maze_max_response_bytes",
            "maze_max_response_duration_ms",
            "maze_server_visible_links",
            "maze_max_links",
            "maze_max_paragraphs",
            "maze_path_entropy_segment_len",
            "maze_covert_decoys_enabled",
            "maze_seed_provider",
            "maze_seed_refresh_interval_seconds",
            "maze_seed_refresh_rate_limit_per_hour",
            "maze_seed_refresh_max_sources",
            "maze_seed_metadata_only",
        ],
        targets: &["suspicious_forwarded_bytes", "suspicious_forwarded_requests"],
        value_constraints: &[],
        note: "Maze content, seed, and capacity tuning remain manual-only until Shuma has explicit benchmark evidence for safe autonomous adjustment.",
    },
    AllowedActionGroupDefinition {
        group_id: "tarpit.core",
        family: "tarpit",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &[
            "tarpit_enabled",
            "tarpit_progress_token_ttl_seconds",
            "tarpit_progress_replay_ttl_seconds",
            "tarpit_hashcash_min_difficulty",
            "tarpit_hashcash_max_difficulty",
            "tarpit_hashcash_base_difficulty",
            "tarpit_hashcash_adaptive",
            "tarpit_step_chunk_base_bytes",
            "tarpit_step_chunk_max_bytes",
            "tarpit_step_jitter_percent",
            "tarpit_shard_rotation_enabled",
            "tarpit_egress_window_seconds",
            "tarpit_egress_global_bytes_per_window",
            "tarpit_egress_per_ip_bucket_bytes_per_window",
            "tarpit_egress_per_flow_max_bytes",
            "tarpit_egress_per_flow_max_duration_seconds",
            "tarpit_max_concurrent_global",
            "tarpit_max_concurrent_per_ip_bucket",
            "tarpit_fallback_action",
        ],
        targets: &["suspicious_forwarded_bytes", "suspicious_forwarded_requests"],
        value_constraints: &[],
        note: "Tarpit cost-shaping remains manual-only until benchmark loops can prove safe asymmetric gains across real traffic and adversary-sim evidence.",
    },
    AllowedActionGroupDefinition {
        group_id: "proof_of_work.policy",
        family: "proof_of_work",
        controller_status: "allowed",
        canary_requirement: "required",
        patch_paths: &["pow_enabled", "pow_difficulty", "pow_ttl_seconds"],
        targets: &[
            "likely_human_friction",
            "suspicious_forwarded_bytes",
            "suspicious_forwarded_requests",
        ],
        value_constraints: PROOF_OF_WORK_CONSTRAINTS,
        note: "Proof-of-work posture is controller-tunable because it is already bounded by explicit clamp ranges.",
    },
    AllowedActionGroupDefinition {
        group_id: "challenge.policy",
        family: "challenge",
        controller_status: "allowed",
        canary_requirement: "required",
        patch_paths: &[
            "challenge_puzzle_enabled",
            "challenge_puzzle_transform_count",
            "challenge_puzzle_seed_ttl_seconds",
            "challenge_puzzle_attempt_limit_per_window",
            "challenge_puzzle_attempt_window_seconds",
        ],
        targets: &[
            "likely_human_friction",
            "suspicious_forwarded_bytes",
            "suspicious_forwarded_requests",
        ],
        value_constraints: CHALLENGE_CONSTRAINTS,
        note: "Challenge posture is controller-tunable because it is bounded by explicit numeric envelopes and canary review.",
    },
    AllowedActionGroupDefinition {
        group_id: "not_a_bot.policy",
        family: "not_a_bot",
        controller_status: "allowed",
        canary_requirement: "required",
        patch_paths: &[
            "not_a_bot_enabled",
            "not_a_bot_risk_threshold",
            "not_a_bot_pass_score",
            "not_a_bot_fail_score",
            "not_a_bot_nonce_ttl_seconds",
            "not_a_bot_marker_ttl_seconds",
            "not_a_bot_attempt_limit_per_window",
            "not_a_bot_attempt_window_seconds",
        ],
        targets: &["likely_human_friction"],
        value_constraints: NOT_A_BOT_CONSTRAINTS,
        note: "Not-a-bot posture is controller-tunable because it is bounded and directly tied to likely-human friction budgets.",
    },
    AllowedActionGroupDefinition {
        group_id: "botness.thresholds",
        family: "botness",
        controller_status: "allowed",
        canary_requirement: "required",
        patch_paths: &[
            "challenge_puzzle_risk_threshold",
            "botness_maze_threshold",
            "botness_weights",
        ],
        targets: &[
            "likely_human_friction",
            "suspicious_forwarded_bytes",
            "suspicious_forwarded_requests",
        ],
        value_constraints: BOTNESS_CONSTRAINTS,
        note: "Botness scoring thresholds are controller-tunable because they already live inside explicit clamp ranges.",
    },
    AllowedActionGroupDefinition {
        group_id: "botness.defence_modes",
        family: "botness",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &["defence_modes"],
        targets: &[
            "likely_human_friction",
            "suspicious_forwarded_bytes",
            "suspicious_forwarded_requests",
        ],
        value_constraints: &[],
        note: "Per-defence composability modes remain manual-only until Shuma has richer rollout semantics for autonomous mode switching.",
    },
    AllowedActionGroupDefinition {
        group_id: "robots_policy",
        family: "robots_policy",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &[
            "robots_enabled",
            "ai_policy_block_training",
            "ai_policy_block_search",
            "ai_policy_allow_search_engines",
            "robots_crawl_delay",
        ],
        targets: &["beneficial_non_human_posture"],
        value_constraints: &[],
        note: "Beneficial non-human posture and robots policy remain manual-only because they are explicit local authorization choices.",
    },
    AllowedActionGroupDefinition {
        group_id: "cdp_detection.policy",
        family: "cdp_detection",
        controller_status: "allowed",
        canary_requirement: "required",
        patch_paths: &[
            "cdp_detection_enabled",
            "cdp_auto_ban",
            "cdp_detection_threshold",
            "cdp_probe_rollout_percent",
        ],
        targets: &[
            "likely_human_friction",
            "suspicious_forwarded_bytes",
            "suspicious_forwarded_requests",
        ],
        value_constraints: CDP_POLICY_CONSTRAINTS,
        note: "CDP-surface thresholds are controller-tunable because they already have bounded numeric envelopes and explicit rollout review.",
    },
    AllowedActionGroupDefinition {
        group_id: "cdp_detection.probe_family",
        family: "cdp_detection",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &["cdp_probe_family"],
        targets: &[
            "likely_human_friction",
            "suspicious_forwarded_bytes",
            "suspicious_forwarded_requests",
        ],
        value_constraints: &[],
        note: "Probe-family rotation remains manual-only because it is effectively code/asset rollout posture rather than a safe numeric tuning action.",
    },
    AllowedActionGroupDefinition {
        group_id: "fingerprint_signal.policy",
        family: "fingerprint_signal",
        controller_status: "allowed",
        canary_requirement: "required",
        patch_paths: &[
            "fingerprint_signal_enabled",
            "fingerprint_state_ttl_seconds",
            "fingerprint_flow_window_seconds",
            "fingerprint_flow_violation_threshold",
            "fingerprint_entropy_budget",
            "fingerprint_family_cap_header_runtime",
            "fingerprint_family_cap_transport",
            "fingerprint_family_cap_temporal",
            "fingerprint_family_cap_persistence",
            "fingerprint_family_cap_behavior",
        ],
        targets: &[
            "likely_human_friction",
            "suspicious_forwarded_bytes",
            "suspicious_forwarded_requests",
        ],
        value_constraints: FINGERPRINT_POLICY_CONSTRAINTS,
        note: "Fingerprint thresholds are controller-tunable because they are bounded by explicit clamp ranges and stay inside the existing config write surface.",
    },
    AllowedActionGroupDefinition {
        group_id: "fingerprint_signal.privacy",
        family: "fingerprint_signal",
        controller_status: "manual_only",
        canary_requirement: "not_applicable",
        patch_paths: &["fingerprint_pseudonymize"],
        targets: &[],
        value_constraints: &[],
        note: "Fingerprint privacy posture remains manual-only because it changes observability/privacy policy, not tuning sensitivity.",
    },
    AllowedActionGroupDefinition {
        group_id: "provider_selection.backends",
        family: "provider_selection",
        controller_status: "forbidden",
        canary_requirement: "not_applicable",
        patch_paths: &["provider_backends", "edge_integration_mode"],
        targets: &[],
        value_constraints: &[],
        note: "Provider and edge-backend selection is permanently outside the first controller write surface.",
    },
];

fn build_value_constraint(
    definition: &AllowedActionValueConstraintDefinition,
) -> AllowedActionValueConstraint {
    AllowedActionValueConstraint {
        path: definition.path.to_string(),
        value_kind: definition.value_kind.to_string(),
        min_inclusive: definition.min_inclusive,
        max_inclusive: definition.max_inclusive,
        allowed_values: definition
            .allowed_values
            .iter()
            .map(|value| value.to_string())
            .collect(),
        rule: definition.rule.map(|rule| rule.to_string()),
    }
}

fn build_group(definition: &AllowedActionGroupDefinition) -> AllowedActionGroup {
    AllowedActionGroup {
        group_id: definition.group_id.to_string(),
        family: definition.family.to_string(),
        controller_status: definition.controller_status.to_string(),
        canary_requirement: definition.canary_requirement.to_string(),
        patch_paths: definition
            .patch_paths
            .iter()
            .map(|path| path.to_string())
            .collect(),
        targets: definition
            .targets
            .iter()
            .map(|target| target.to_string())
            .collect(),
        value_constraints: definition
            .value_constraints
            .iter()
            .map(build_value_constraint)
            .collect(),
        note: definition.note.to_string(),
    }
}

fn family_status(statuses: &[String]) -> String {
    let all_allowed = statuses.iter().all(|status| status == "allowed");
    let all_manual_only = statuses.iter().all(|status| status == "manual_only");
    let all_forbidden = statuses.iter().all(|status| status == "forbidden");
    if all_allowed {
        "allowed".to_string()
    } else if all_manual_only {
        "manual_only".to_string()
    } else if all_forbidden {
        "forbidden".to_string()
    } else {
        "mixed".to_string()
    }
}

pub(crate) fn controller_config_family_for_patch_key(key: &str) -> Option<&'static str> {
    ALLOWED_ACTION_GROUP_DEFINITIONS
        .iter()
        .find(|definition| definition.patch_paths.contains(&key))
        .map(|definition| definition.family)
}

pub(crate) fn controller_action_family_targets(family: &str) -> Vec<String> {
    let mut targets = BTreeMap::<String, ()>::new();
    for definition in ALLOWED_ACTION_GROUP_DEFINITIONS
        .iter()
        .filter(|definition| definition.family == family)
    {
        for target in definition.targets {
            targets.insert((*target).to_string(), ());
        }
    }
    targets.into_keys().collect()
}

pub(crate) fn allowed_actions_v1() -> AllowedActionsSurface {
    let groups = ALLOWED_ACTION_GROUP_DEFINITIONS
        .iter()
        .map(build_group)
        .collect::<Vec<_>>();
    let allowed_group_ids = groups
        .iter()
        .filter(|group| group.controller_status == "allowed")
        .map(|group| group.group_id.clone())
        .collect::<Vec<_>>();
    let manual_only_group_ids = groups
        .iter()
        .filter(|group| group.controller_status == "manual_only")
        .map(|group| group.group_id.clone())
        .collect::<Vec<_>>();
    let forbidden_group_ids = groups
        .iter()
        .filter(|group| group.controller_status == "forbidden")
        .map(|group| group.group_id.clone())
        .collect::<Vec<_>>();

    let mut family_groups = BTreeMap::<String, Vec<&AllowedActionGroup>>::new();
    for group in &groups {
        family_groups
            .entry(group.family.clone())
            .or_default()
            .push(group);
    }

    let families = family_groups
        .into_iter()
        .map(|(family, family_groups)| AllowedActionFamily {
            controller_status: family_status(
                family_groups
                    .iter()
                    .map(|group| group.controller_status.clone())
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            group_ids: family_groups
                .iter()
                .map(|group| group.group_id.clone())
                .collect(),
            targets: controller_action_family_targets(family.as_str()),
            family,
        })
        .collect::<Vec<_>>();

    AllowedActionsSurface {
        schema_version: ALLOWED_ACTIONS_SCHEMA_VERSION.to_string(),
        write_surface: "admin_config".to_string(),
        proposal_mode: "config_diff_only".to_string(),
        groups,
        families,
        allowed_group_ids,
        manual_only_group_ids,
        forbidden_group_ids,
    }
}
