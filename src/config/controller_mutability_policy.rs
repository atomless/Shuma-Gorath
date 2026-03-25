#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};

pub(crate) const CONTROLLER_MUTABILITY_SCHEMA_VERSION: &str = "controller_mutability_v1";
pub(crate) const CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG: &str = "admin_config";
pub(crate) const CONTROLLER_MUTABILITY_SCOPE_OPERATOR_OBJECTIVES: &str = "operator_objectives_v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControllerMutabilityRing {
    Never,
    ManualOnly,
    ControllerTunable,
}

impl ControllerMutabilityRing {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::ManualOnly => "manual_only",
            Self::ControllerTunable => "controller_tunable",
        }
    }

    pub(crate) fn allowed_actions_status(self) -> &'static str {
        match self {
            Self::Never => "forbidden",
            Self::ManualOnly => "manual_only",
            Self::ControllerTunable => "allowed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControllerMutabilityGroup {
    pub scope: String,
    pub group_id: String,
    pub ring: String,
    pub paths: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControllerMutabilitySurface {
    pub schema_version: String,
    pub groups: Vec<ControllerMutabilityGroup>,
}

struct ControllerMutabilityGroupDefinition {
    scope: &'static str,
    group_id: &'static str,
    ring: ControllerMutabilityRing,
    paths: &'static [&'static str],
    note: &'static str,
}

const OPERATOR_OBJECTIVE_MUTABILITY_GROUP_DEFINITIONS: &[ControllerMutabilityGroupDefinition] = &[
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_OPERATOR_OBJECTIVES,
        group_id: "operator_objectives.identity",
        ring: ControllerMutabilityRing::Never,
        paths: &["profile_id", "window_hours", "compliance_semantics"],
        note: "Operator objectives define the game rules and must remain permanently outside the controller move set.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_OPERATOR_OBJECTIVES,
        group_id: "operator_objectives.category_postures",
        ring: ControllerMutabilityRing::Never,
        paths: &["category_postures.*"],
        note: "Category postures are operator-owned target policy and must never be controller-mutable.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_OPERATOR_OBJECTIVES,
        group_id: "operator_objectives.budgets",
        ring: ControllerMutabilityRing::Never,
        paths: &["budgets.*"],
        note: "Budget targets define success and must never be controller-mutable.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_OPERATOR_OBJECTIVES,
        group_id: "operator_objectives.adversary_sim_expectations",
        ring: ControllerMutabilityRing::Never,
        paths: &["adversary_sim_expectations.*"],
        note: "Adversary-sim expectations are rule-side evaluation settings, not controller moves.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_OPERATOR_OBJECTIVES,
        group_id: "operator_objectives.rollout_guardrails",
        ring: ControllerMutabilityRing::Never,
        paths: &["rollout_guardrails.*"],
        note: "Rollout guardrails are operator-owned safety policy and must never be controller-mutable.",
    },
];

const ADMIN_CONFIG_MUTABILITY_GROUP_DEFINITIONS: &[ControllerMutabilityGroupDefinition] = &[
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "shadow_mode.state",
        ring: ControllerMutabilityRing::Never,
        paths: &["shadow_mode"],
        note: "Execution-mode switches change the measurement harness and must remain permanently outside controller tuning.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "adversary_sim.duration",
        ring: ControllerMutabilityRing::Never,
        paths: &["adversary_sim_duration_seconds"],
        note: "Adversary-sim runtime posture changes the harness rather than the defence policy and must remain permanently controller-forbidden.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "core_policy.js_required_toggle",
        ring: ControllerMutabilityRing::ControllerTunable,
        paths: &["js_required_enforced"],
        note: "JS-required posture is a bounded friction-sensitive control and belongs in the narrow controller-tunable ring.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "core_policy.rate_limit",
        ring: ControllerMutabilityRing::ManualOnly,
        paths: &["rate_limit"],
        note: "Rate-limit posture is operator-editable but not yet ratified as a safe controller-tunable knob.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "core_policy.ban_durations",
        ring: ControllerMutabilityRing::Never,
        paths: &[
            "ban_duration",
            "ban_durations.honeypot",
            "ban_durations.ip_range_honeypot",
            "ban_durations.maze_crawler",
            "ban_durations.rate_limit",
            "ban_durations.admin",
            "ban_durations.cdp",
            "ban_durations.edge_fingerprint",
            "ban_durations.tarpit_persistence",
            "ban_durations.not_a_bot_abuse",
            "ban_durations.challenge_puzzle_abuse",
        ],
        note: "Punishment horizons are policy and sanction controls, not bounded tuning moves.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "geo_policy.country_lists",
        ring: ControllerMutabilityRing::Never,
        paths: &[
            "geo_risk",
            "geo_allow",
            "geo_challenge",
            "geo_maze",
            "geo_block",
            "geo_edge_headers_enabled",
        ],
        note: "Geo routing and trust posture are site-local policy and must remain permanently outside controller control.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "honeypot.surface",
        ring: ControllerMutabilityRing::Never,
        paths: &["honeypot_enabled", "honeypots"],
        note: "Honeypot placement is site-local deception policy and must remain controller-forbidden.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "browser_policy.allowlists",
        ring: ControllerMutabilityRing::Never,
        paths: &["browser_policy_enabled", "browser_block", "browser_allowlist"],
        note: "Browser allowlists and deny rules are explicit trust-boundary policy and must remain controller-forbidden.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "allowlists.surface",
        ring: ControllerMutabilityRing::Never,
        paths: &[
            "bypass_allowlists_enabled",
            "allowlist",
            "path_allowlist_enabled",
            "path_allowlist",
        ],
        note: "Explicit trust exceptions and allowlists must never be loop-mutable.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "ip_range_policy",
        ring: ControllerMutabilityRing::Never,
        paths: &[
            "ip_range_policy_mode",
            "ip_range_emergency_allowlist",
            "ip_range_custom_rules",
        ],
        note: "IP-range policy carries high collateral-risk and must remain permanently controller-forbidden.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "maze_core.rollout",
        ring: ControllerMutabilityRing::ControllerTunable,
        paths: &["maze_enabled", "maze_auto_ban", "maze_rollout_phase"],
        note: "Maze rollout posture stays in-bounds only through the explicit bounded rollout controls.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "maze_core.content_and_budget",
        ring: ControllerMutabilityRing::Never,
        paths: &[
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
        note: "Maze content, seed, and defender safety budgets must remain permanently controller-forbidden.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "tarpit.core",
        ring: ControllerMutabilityRing::Never,
        paths: &[
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
        note: "Tarpit resource ceilings, fallback posture, and cost budgets protect Shuma itself and must remain controller-forbidden.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "proof_of_work.policy",
        ring: ControllerMutabilityRing::ControllerTunable,
        paths: &["pow_enabled", "pow_difficulty", "pow_ttl_seconds"],
        note: "Proof-of-work posture is bounded by explicit ranges and belongs in the controller-tunable ring.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "challenge.policy",
        ring: ControllerMutabilityRing::ControllerTunable,
        paths: &[
            "challenge_puzzle_enabled",
            "challenge_puzzle_transform_count",
            "challenge_puzzle_seed_ttl_seconds",
            "challenge_puzzle_attempt_limit_per_window",
            "challenge_puzzle_attempt_window_seconds",
        ],
        note: "Challenge posture is controller-tunable only through bounded challenge settings.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "not_a_bot.policy",
        ring: ControllerMutabilityRing::ControllerTunable,
        paths: &[
            "not_a_bot_enabled",
            "not_a_bot_risk_threshold",
            "not_a_bot_pass_score",
            "not_a_bot_fail_score",
            "not_a_bot_nonce_ttl_seconds",
            "not_a_bot_marker_ttl_seconds",
            "not_a_bot_attempt_limit_per_window",
            "not_a_bot_attempt_window_seconds",
        ],
        note: "Not-a-bot posture is bounded and directly tied to human-friction tradeoffs, so it belongs in the controller-tunable ring.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "botness.thresholds",
        ring: ControllerMutabilityRing::ControllerTunable,
        paths: &[
            "challenge_puzzle_risk_threshold",
            "botness_maze_threshold",
            "botness_weights.js_required",
            "botness_weights.geo_risk",
            "botness_weights.rate_medium",
            "botness_weights.rate_high",
            "botness_weights.maze_behavior",
        ],
        note: "Botness thresholds and weights are bounded sensitivity controls and belong in the controller-tunable ring.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "botness.defence_modes",
        ring: ControllerMutabilityRing::Never,
        paths: &["defence_modes.rate", "defence_modes.geo", "defence_modes.js"],
        note: "Per-defence composability modes are implementation and rollout controls, not controller moves.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "robots_policy",
        ring: ControllerMutabilityRing::Never,
        paths: &[
            "robots_enabled",
            "ai_policy_block_training",
            "ai_policy_block_search",
            "ai_policy_allow_search_engines",
            "robots_crawl_delay",
        ],
        note: "Robots and AI policy are explicit authorization choices and must remain controller-forbidden.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "cdp_detection.policy",
        ring: ControllerMutabilityRing::ControllerTunable,
        paths: &[
            "cdp_detection_enabled",
            "cdp_auto_ban",
            "cdp_detection_threshold",
            "cdp_probe_rollout_percent",
        ],
        note: "CDP detection thresholds and rollout percent are bounded tuning knobs and belong in the controller-tunable ring.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "cdp_detection.probe_family",
        ring: ControllerMutabilityRing::Never,
        paths: &["cdp_probe_family"],
        note: "Probe-family rotation is implementation composition, not a safe controller-tunable setting.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "fingerprint_signal.policy",
        ring: ControllerMutabilityRing::ControllerTunable,
        paths: &[
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
        note: "Fingerprint sensitivity and bounded caps belong in the controller-tunable ring.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "fingerprint_signal.privacy",
        ring: ControllerMutabilityRing::Never,
        paths: &["fingerprint_pseudonymize"],
        note: "Fingerprint privacy posture changes observability policy and must remain controller-forbidden.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "provider_selection.backends",
        ring: ControllerMutabilityRing::Never,
        paths: &[
            "provider_backends.rate_limiter",
            "provider_backends.ban_store",
            "provider_backends.challenge_engine",
            "provider_backends.maze_tarpit",
            "provider_backends.fingerprint_signal",
            "edge_integration_mode",
        ],
        note: "Provider and edge topology define architecture and trust boundaries, not controller moves.",
    },
    ControllerMutabilityGroupDefinition {
        scope: CONTROLLER_MUTABILITY_SCOPE_ADMIN_CONFIG,
        group_id: "verified_identity.policy",
        ring: ControllerMutabilityRing::Never,
        paths: &[
            "verified_identity.enabled",
            "verified_identity.native_web_bot_auth_enabled",
            "verified_identity.provider_assertions_enabled",
            "verified_identity.replay_window_seconds",
            "verified_identity.clock_skew_seconds",
            "verified_identity.directory_cache_ttl_seconds",
            "verified_identity.directory_freshness_requirement_seconds",
            "verified_identity.named_policies",
            "verified_identity.category_defaults",
            "verified_identity.service_profiles",
        ],
        note: "Verified-identity trust posture and authorization policy must remain permanently controller-forbidden.",
    },
];

fn build_group(
    definition: &ControllerMutabilityGroupDefinition,
) -> ControllerMutabilityGroup {
    ControllerMutabilityGroup {
        scope: definition.scope.to_string(),
        group_id: definition.group_id.to_string(),
        ring: definition.ring.as_str().to_string(),
        paths: definition.paths.iter().map(|path| (*path).to_string()).collect(),
        note: definition.note.to_string(),
    }
}

fn ring_for_path(
    definitions: &[ControllerMutabilityGroupDefinition],
    path: &str,
) -> Option<ControllerMutabilityRing> {
    let exact = definitions
        .iter()
        .filter_map(|definition| {
            definition
                .paths
                .iter()
                .find(|pattern| **pattern == path)
                .map(|_| definition.ring)
        })
        .next();
    if exact.is_some() {
        return exact;
    }

    let prefix_matches = definitions
        .iter()
        .filter_map(|definition| {
            let matched = definition.paths.iter().any(|pattern| {
                if let Some(prefix) = pattern.strip_suffix(".*") {
                    path == prefix || path.starts_with(&format!("{prefix}."))
                } else {
                    path.starts_with(&format!("{pattern}."))
                        || pattern.starts_with(&format!("{path}."))
                }
            });
            matched.then_some(definition.ring)
        })
        .collect::<Vec<_>>();

    let first = prefix_matches.first().copied()?;
    prefix_matches
        .iter()
        .all(|candidate| *candidate == first)
        .then_some(first)
}

pub(crate) fn controller_mutability_ring_for_admin_config_path(
    path: &str,
) -> Option<ControllerMutabilityRing> {
    ring_for_path(ADMIN_CONFIG_MUTABILITY_GROUP_DEFINITIONS, path)
}

pub(crate) fn controller_mutability_ring_for_operator_objectives_path(
    path: &str,
) -> Option<ControllerMutabilityRing> {
    ring_for_path(OPERATOR_OBJECTIVE_MUTABILITY_GROUP_DEFINITIONS, path)
}

pub(crate) fn allowed_actions_status_for_admin_config_paths(
    paths: &[&str],
) -> Option<&'static str> {
    let rings = paths
        .iter()
        .map(|path| controller_mutability_ring_for_admin_config_path(path))
        .collect::<Option<Vec<_>>>()?;
    let first = *rings.first()?;
    rings
        .iter()
        .all(|candidate| *candidate == first)
        .then_some(first.allowed_actions_status())
}

pub(crate) fn controller_mutability_policy_v1() -> ControllerMutabilitySurface {
    let groups = OPERATOR_OBJECTIVE_MUTABILITY_GROUP_DEFINITIONS
        .iter()
        .chain(ADMIN_CONFIG_MUTABILITY_GROUP_DEFINITIONS.iter())
        .map(build_group)
        .collect();

    ControllerMutabilitySurface {
        schema_version: CONTROLLER_MUTABILITY_SCHEMA_VERSION.to_string(),
        groups,
    }
}
