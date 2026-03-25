use serde::{Deserialize, Serialize};

use crate::config::FrontierSummary;

use super::adversary_sim::{ControlState, RuntimeLane};

pub(crate) const LLM_FULFILLMENT_PLAN_SCHEMA_VERSION: &str =
    "adversary-sim-llm-fulfillment-plan.v1";
pub(crate) const FRONTIER_ACTION_CONTRACT_ID: &str = "frontier_action_contract.v1";
pub(crate) const CONTAINER_RUNTIME_PROFILE_ID: &str = "container_runtime_profile.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LlmFulfillmentMode {
    BrowserMode,
    RequestMode,
}

impl LlmFulfillmentMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::BrowserMode => "browser_mode",
            Self::RequestMode => "request_mode",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LlmBackendKind {
    FrontierReference,
    LocalCandidate,
}

impl LlmBackendKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::FrontierReference => "frontier_reference",
            Self::LocalCandidate => "local_candidate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LlmCapabilityEnvelope {
    pub allowed_tools: Vec<String>,
    pub browser_automation_allowed: bool,
    pub direct_request_emission_allowed: bool,
    pub max_actions: u64,
    pub max_time_budget_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LlmReceiptRequirements {
    pub attack_trace_required: bool,
    pub observation_lineage_required: bool,
    pub category_objective_lineage_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LlmBlackBoxBoundary {
    pub position: String,
    pub host_root_only_entrypoint: bool,
    pub category_objective_required: bool,
    pub malicious_category_priming_required: bool,
    pub public_knowledge_only: bool,
    pub shuma_blind: bool,
    pub web_search_allowed: bool,
    pub repo_visibility_allowed: bool,
    pub judge_visibility_allowed: bool,
    pub public_host_hint_sources: Vec<String>,
    pub allowed_observation_families: Vec<String>,
    pub forbidden_knowledge_sources: Vec<String>,
    pub receipt_requirements: LlmReceiptRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LlmEpisodeHarness {
    pub initial_context_fields: Vec<String>,
    pub environment_reset_required: bool,
    pub environment_reset_policy: String,
    pub bounded_action_horizon_required: bool,
    pub terminal_conditions: Vec<String>,
    pub failure_states: Vec<String>,
    pub allowed_memory_sources: Vec<String>,
    pub forbidden_memory_sources: Vec<String>,
    pub max_retained_episode_summaries: u64,
    pub max_curriculum_items: u64,
    pub player_visible_protected_evidence_allowed: bool,
    pub held_out_evaluation_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LlmFulfillmentPlan {
    pub schema_version: String,
    pub run_id: String,
    pub tick_id: String,
    pub lane: RuntimeLane,
    pub fulfillment_mode: String,
    pub backend_kind: String,
    pub backend_state: String,
    pub backend_id: String,
    pub supported_backend_kinds: Vec<String>,
    pub category_targets: Vec<String>,
    pub frontier_action_contract_id: String,
    pub container_runtime_profile_id: String,
    pub black_box_boundary: LlmBlackBoxBoundary,
    pub episode_harness: LlmEpisodeHarness,
    pub capability_envelope: LlmCapabilityEnvelope,
}

pub(crate) fn next_llm_fulfillment_plan(
    now: u64,
    state: &ControlState,
    frontier: &FrontierSummary,
) -> LlmFulfillmentPlan {
    let run_id = state
        .run_id
        .clone()
        .or_else(|| state.last_run_id.clone())
        .unwrap_or_else(|| format!("simrun-runtime-{now}"));
    let mode = llm_fulfillment_mode_for_tick(state.generated_tick_count);
    let (backend_state, backend_id) = frontier_backend_state(frontier);

    LlmFulfillmentPlan {
        schema_version: LLM_FULFILLMENT_PLAN_SCHEMA_VERSION.to_string(),
        run_id,
        tick_id: format!("llm-fit-tick-{}-{:016x}", now, rand::random::<u64>()),
        lane: RuntimeLane::BotRedTeam,
        fulfillment_mode: mode.as_str().to_string(),
        backend_kind: LlmBackendKind::FrontierReference.as_str().to_string(),
        backend_state,
        backend_id,
        supported_backend_kinds: vec![
            LlmBackendKind::FrontierReference.as_str().to_string(),
            LlmBackendKind::LocalCandidate.as_str().to_string(),
        ],
        category_targets: category_targets_for_mode(mode),
        frontier_action_contract_id: FRONTIER_ACTION_CONTRACT_ID.to_string(),
        container_runtime_profile_id: CONTAINER_RUNTIME_PROFILE_ID.to_string(),
        black_box_boundary: black_box_boundary_contract(),
        episode_harness: episode_harness_contract(),
        capability_envelope: capability_envelope_for_mode(mode),
    }
}

pub(crate) fn llm_fulfillment_mode_for_tick(generated_tick_count: u64) -> LlmFulfillmentMode {
    if generated_tick_count % 2 == 0 {
        LlmFulfillmentMode::BrowserMode
    } else {
        LlmFulfillmentMode::RequestMode
    }
}

fn category_targets_for_mode(mode: LlmFulfillmentMode) -> Vec<String> {
    crate::observability::non_human_lane_fulfillment::llm_category_targets_for_mode(mode.as_str())
}

fn capability_envelope_for_mode(mode: LlmFulfillmentMode) -> LlmCapabilityEnvelope {
    match mode {
        LlmFulfillmentMode::BrowserMode => LlmCapabilityEnvelope {
            allowed_tools: vec![
                "browser_navigate".to_string(),
                "browser_snapshot".to_string(),
                "browser_click".to_string(),
            ],
            browser_automation_allowed: true,
            direct_request_emission_allowed: false,
            max_actions: 8,
            max_time_budget_seconds: 90,
        },
        LlmFulfillmentMode::RequestMode => LlmCapabilityEnvelope {
            allowed_tools: vec!["http_get".to_string()],
            browser_automation_allowed: false,
            direct_request_emission_allowed: true,
            max_actions: 24,
            max_time_budget_seconds: 120,
        },
    }
}

fn black_box_boundary_contract() -> LlmBlackBoxBoundary {
    LlmBlackBoxBoundary {
        position: "outside_attacker".to_string(),
        host_root_only_entrypoint: true,
        category_objective_required: true,
        malicious_category_priming_required: true,
        public_knowledge_only: true,
        shuma_blind: true,
        web_search_allowed: false,
        repo_visibility_allowed: false,
        judge_visibility_allowed: false,
        public_host_hint_sources: vec![
            "robots_txt".to_string(),
            "sitemap_references".to_string(),
            "traversal_visible_pages".to_string(),
        ],
        allowed_observation_families: vec![
            "root_response".to_string(),
            "public_host_hints".to_string(),
            "traversal_observations".to_string(),
            "response_metadata".to_string(),
            "content_snapshots".to_string(),
        ],
        forbidden_knowledge_sources: vec![
            "shuma_repo".to_string(),
            "shuma_docs".to_string(),
            "shuma_source_code".to_string(),
            "shuma_internal_routes".to_string(),
            "shuma_defense_inventory".to_string(),
            "admin_credentials".to_string(),
            "judge_state".to_string(),
            "web_search".to_string(),
        ],
        receipt_requirements: LlmReceiptRequirements {
            attack_trace_required: true,
            observation_lineage_required: true,
            category_objective_lineage_required: true,
        },
    }
}

fn episode_harness_contract() -> LlmEpisodeHarness {
    LlmEpisodeHarness {
        initial_context_fields: vec![
            "host_root_entrypoint".to_string(),
            "category_objective".to_string(),
            "black_box_boundary".to_string(),
            "capability_envelope".to_string(),
        ],
        environment_reset_required: true,
        environment_reset_policy: "fresh_episode_reset".to_string(),
        bounded_action_horizon_required: true,
        terminal_conditions: vec![
            "objective_completed".to_string(),
            "action_budget_exhausted".to_string(),
            "time_budget_exhausted".to_string(),
            "hard_guardrail_triggered".to_string(),
            "environment_unavailable".to_string(),
        ],
        failure_states: vec![
            "objective_unfulfilled".to_string(),
            "environment_error".to_string(),
            "guardrail_refusal".to_string(),
        ],
        allowed_memory_sources: vec![
            "prior_episode_summaries".to_string(),
            "player_visible_protected_evidence".to_string(),
            "public_host_discoveries".to_string(),
            "curriculum_strategy_notes".to_string(),
        ],
        forbidden_memory_sources: vec![
            "judge_held_out_evaluation".to_string(),
            "raw_regression_anchor_inventory".to_string(),
            "admin_or_secret_material".to_string(),
            "shuma_internal_knowledge".to_string(),
        ],
        max_retained_episode_summaries: 5,
        max_curriculum_items: 8,
        player_visible_protected_evidence_allowed: true,
        held_out_evaluation_visible: false,
    }
}

fn frontier_backend_state(frontier: &FrontierSummary) -> (String, String) {
    if frontier.provider_count == 0 {
        return (
            "unavailable".to_string(),
            "frontier_reference:unconfigured".to_string(),
        );
    }
    let backend_state = if frontier.reduced_diversity_warning {
        "degraded"
    } else {
        "configured"
    };
    (
        backend_state.to_string(),
        format!("frontier_reference:{}", frontier.mode),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        llm_fulfillment_mode_for_tick, next_llm_fulfillment_plan, LlmFulfillmentMode,
    };
    use crate::admin::adversary_sim::ControlState;
    use crate::config::frontier_summary;

    #[test]
    fn llm_fulfillment_modes_alternate_between_browser_and_request_contracts() {
        assert_eq!(llm_fulfillment_mode_for_tick(0), LlmFulfillmentMode::BrowserMode);
        assert_eq!(llm_fulfillment_mode_for_tick(1), LlmFulfillmentMode::RequestMode);
    }

    #[test]
    fn llm_fulfillment_plan_uses_frontier_reference_when_provider_keys_exist() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_FRONTIER_OPENAI_API_KEY", "test-key");
        std::env::set_var("SHUMA_FRONTIER_OPENAI_MODEL", "gpt-5-mini");
        let frontier = frontier_summary();
        let plan = next_llm_fulfillment_plan(1_700_000_000, &ControlState::default(), &frontier);

        assert_eq!(plan.backend_kind, "frontier_reference");
        assert_eq!(plan.backend_state, "degraded");
        assert_eq!(plan.fulfillment_mode, "browser_mode");
        assert!(plan
            .supported_backend_kinds
            .contains(&"local_candidate".to_string()));
        assert!(plan
            .capability_envelope
            .allowed_tools
            .contains(&"browser_navigate".to_string()));
        assert_eq!(plan.black_box_boundary.position, "outside_attacker");
        assert!(plan.black_box_boundary.host_root_only_entrypoint);
        assert!(plan.black_box_boundary.shuma_blind);
        assert!(!plan.black_box_boundary.web_search_allowed);
        assert!(plan
            .black_box_boundary
            .public_host_hint_sources
            .contains(&"robots_txt".to_string()));
        assert!(plan.episode_harness.environment_reset_required);
        assert_eq!(
            plan.episode_harness.initial_context_fields,
            vec![
                "host_root_entrypoint",
                "category_objective",
                "black_box_boundary",
                "capability_envelope",
            ]
        );
        assert!(plan
            .episode_harness
            .allowed_memory_sources
            .contains(&"player_visible_protected_evidence".to_string()));
        assert!(!plan.episode_harness.held_out_evaluation_visible);

        std::env::remove_var("SHUMA_FRONTIER_OPENAI_API_KEY");
        std::env::remove_var("SHUMA_FRONTIER_OPENAI_MODEL");
    }

    #[test]
    fn llm_fulfillment_plan_reports_unavailable_frontier_backend_without_provider_keys() {
        let frontier = frontier_summary();
        let mut state = ControlState::default();
        state.generated_tick_count = 1;
        let plan = next_llm_fulfillment_plan(1_700_000_000, &state, &frontier);

        assert_eq!(plan.backend_kind, "frontier_reference");
        assert_eq!(plan.backend_state, "unavailable");
        assert_eq!(plan.fulfillment_mode, "request_mode");
        assert_eq!(plan.category_targets, vec!["http_agent", "ai_scraper_bot"]);
        assert_eq!(plan.capability_envelope.allowed_tools, vec!["http_get"]);
        assert!(plan.black_box_boundary.public_knowledge_only);
        assert!(!plan.black_box_boundary.repo_visibility_allowed);
        assert!(!plan.black_box_boundary.judge_visibility_allowed);
        assert_eq!(
            plan.episode_harness.environment_reset_policy,
            "fresh_episode_reset"
        );
        assert!(plan
            .episode_harness
            .terminal_conditions
            .contains(&"objective_completed".to_string()));
        assert_eq!(plan.episode_harness.max_retained_episode_summaries, 5);
    }
}
