use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::config::ControllerLegalMoveRingSurface;
use crate::observability::benchmark_results::BENCHMARK_RESULTS_SCHEMA_VERSION;
use crate::observability::benchmark_suite::benchmark_suite_v1;
use crate::runtime::non_human_taxonomy::{
    canonical_non_human_taxonomy, posture_scale, NonHumanCategoryId,
};

pub(crate) const OPERATOR_OBJECTIVES_SCHEMA_VERSION: &str = "operator_objectives_v1";
pub(crate) const RECURSIVE_IMPROVEMENT_GAME_CONTRACT_SCHEMA_VERSION: &str =
    "recursive_improvement_game_contract_v1";
const RECURSIVE_IMPROVEMENT_GAME_CONTRACT_ID: &str = "shuma_recursive_improvement_game_v1";

const SITE_DEFAULT_OBJECTIVE_PROFILE_ID: &str = "site_default_v1";
pub(super) const DEFAULT_WINDOW_HOURS: u64 = 24;
const DEFAULT_NEAR_LIMIT_RATIO: f64 = 0.75;
const LIKELY_HUMAN_FRICTION_TARGET: f64 = 0.02;
const SUSPICIOUS_FORWARDED_REQUEST_TARGET: f64 = 0.10;
const SUSPICIOUS_FORWARDED_BYTE_TARGET: f64 = 0.10;
const SUSPICIOUS_FORWARDED_LATENCY_SHARE_TARGET: f64 = 0.10;
const DEFAULT_MAX_GOAL_SUCCESS_RATE: f64 = 0.0;
const DEFAULT_MIN_ESCALATION_RATE: f64 = 0.25;
const MAX_OBJECTIVE_BUDGET_ROWS: usize = 8;
const MAX_WINDOW_HOURS: u64 = 24 * 30;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorObjectiveBudget {
    pub budget_id: String,
    pub metric: String,
    pub comparator: String,
    pub target: f64,
    pub near_limit_ratio: f64,
    pub eligible_population: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorObjectiveAdversarySimExpectations {
    pub comparison_mode: String,
    pub max_goal_success_rate: f64,
    pub min_escalation_rate: f64,
    pub regression_status_required: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorObjectivesRolloutGuardrails {
    pub automated_apply_status: String,
    pub code_evolution_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorObjectiveCategoryPosture {
    pub category_id: NonHumanCategoryId,
    pub posture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorObjectivesProfile {
    pub schema_version: String,
    pub profile_id: String,
    pub revision: String,
    pub updated_at_ts: u64,
    pub source: String,
    pub window_hours: u64,
    pub compliance_semantics: String,
    pub category_postures: Vec<OperatorObjectiveCategoryPosture>,
    pub budgets: Vec<OperatorObjectiveBudget>,
    pub adversary_sim_expectations: OperatorObjectiveAdversarySimExpectations,
    pub rollout_guardrails: OperatorObjectivesRolloutGuardrails,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorObjectivesUpsertRequest {
    pub profile_id: String,
    pub window_hours: u64,
    pub compliance_semantics: String,
    pub category_postures: Vec<OperatorObjectiveCategoryPosture>,
    pub budgets: Vec<OperatorObjectiveBudget>,
    pub adversary_sim_expectations: OperatorObjectiveAdversarySimExpectations,
    pub rollout_guardrails: OperatorObjectivesRolloutGuardrails,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecursiveImprovementImmutableRules {
    pub operator_rule_surface: String,
    pub objective_revision: String,
    pub compliance_semantics: String,
    pub watch_window_hours: u64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecursiveImprovementScorecardEntry {
    pub score_id: String,
    pub source_contract: String,
    pub family_id: String,
    pub metric_ids: Vec<String>,
    pub judgment_role: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecursiveImprovementEpisodeComparisonContract {
    pub source_contract: String,
    pub comparison_bases: Vec<String>,
    pub judgment_inputs: Vec<String>,
    pub rollback_input_ids: Vec<String>,
    pub homeostasis_input_ids: Vec<String>,
    pub minimum_completed_cycles_for_homeostasis: u64,
    pub scalarization: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecursiveImprovementEvaluatorScorecard {
    pub benchmark_suite_schema_version: String,
    pub benchmark_results_schema_version: String,
    pub input_contract: String,
    pub comparison_modes: Vec<String>,
    pub subject_kinds: Vec<String>,
    pub family_ids: Vec<String>,
    pub decision_boundaries: Vec<String>,
    pub optimization_targets: Vec<RecursiveImprovementScorecardEntry>,
    pub hard_guardrails: Vec<RecursiveImprovementScorecardEntry>,
    pub regression_inputs: Vec<RecursiveImprovementScorecardEntry>,
    pub diagnostic_contexts: Vec<RecursiveImprovementScorecardEntry>,
    pub comparison_contract: RecursiveImprovementEpisodeComparisonContract,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecursiveImprovementSafetyGate {
    pub gate_id: String,
    pub source_contract: String,
    pub requirement: String,
    pub failure_outcome: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecursiveImprovementRegressionAnchor {
    pub anchor_id: String,
    pub source_contract: String,
    pub availability: String,
    pub requirement: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecursiveImprovementGameContract {
    pub schema_version: String,
    pub contract_id: String,
    pub immutable_rules: RecursiveImprovementImmutableRules,
    pub evaluator_scorecard: RecursiveImprovementEvaluatorScorecard,
    pub legal_move_ring: ControllerLegalMoveRingSurface,
    pub safety_gates: Vec<RecursiveImprovementSafetyGate>,
    pub regression_anchors: Vec<RecursiveImprovementRegressionAnchor>,
}

pub(crate) fn operator_objectives_watch_window_seconds(
    profile: &OperatorObjectivesProfile,
) -> u64 {
    profile.window_hours.saturating_mul(3600)
}

pub(crate) fn recursive_improvement_game_contract_v1(
    objectives: &OperatorObjectivesProfile,
    legal_move_ring: &ControllerLegalMoveRingSurface,
) -> RecursiveImprovementGameContract {
    let benchmark_suite = benchmark_suite_v1();
    let category_metric_ids: Vec<_> = objectives
        .category_postures
        .iter()
        .map(|row| format!("category_posture_alignment:{}", row.category_id.as_str()))
        .collect();

    RecursiveImprovementGameContract {
        schema_version: RECURSIVE_IMPROVEMENT_GAME_CONTRACT_SCHEMA_VERSION.to_string(),
        contract_id: RECURSIVE_IMPROVEMENT_GAME_CONTRACT_ID.to_string(),
        immutable_rules: RecursiveImprovementImmutableRules {
            operator_rule_surface: objectives.schema_version.clone(),
            objective_revision: objectives.revision.clone(),
            compliance_semantics: objectives.compliance_semantics.clone(),
            watch_window_hours: objectives.window_hours,
            note: "operator_objectives_v1 is the site-owned rule surface for the game and must never be widened or rewritten by the controller.".to_string(),
        },
        evaluator_scorecard: RecursiveImprovementEvaluatorScorecard {
            benchmark_suite_schema_version: benchmark_suite.schema_version,
            benchmark_results_schema_version: BENCHMARK_RESULTS_SCHEMA_VERSION.to_string(),
            input_contract: benchmark_suite.input_contract,
            comparison_modes: benchmark_suite.comparison_modes,
            subject_kinds: benchmark_suite.subject_kinds,
            family_ids: benchmark_suite
                .families
                .into_iter()
                .map(|family| family.id)
                .collect(),
            decision_boundaries: benchmark_suite
                .decision_boundaries
                .into_iter()
                .map(|boundary| boundary.decision)
                .collect(),
            optimization_targets: vec![
                scorecard_entry(
                    "likely_human_friction_budget",
                    "operator_objectives_v1.budgets + benchmark_results_v1",
                    "likely_human_friction",
                    budget_metric_ids(
                        objectives,
                        "likely_human_friction",
                        "likely_human_friction_rate",
                    ),
                    "optimization_target",
                    "Likely-human friction remains a first-class optimization target and must not be hidden behind aggregate verdict text only.",
                ),
                scorecard_entry(
                    "suspicious_origin_request_budget",
                    "operator_objectives_v1.budgets + benchmark_results_v1",
                    "suspicious_origin_cost",
                    budget_metric_ids(
                        objectives,
                        "suspicious_forwarded_requests",
                        "suspicious_forwarded_request_rate",
                    ),
                    "optimization_target",
                    "Suspicious forwarded request reach is a direct cost-control target for the judge.",
                ),
                scorecard_entry(
                    "suspicious_origin_byte_budget",
                    "operator_objectives_v1.budgets + benchmark_results_v1",
                    "suspicious_origin_cost",
                    budget_metric_ids(
                        objectives,
                        "suspicious_forwarded_bytes",
                        "suspicious_forwarded_byte_rate",
                    ),
                    "optimization_target",
                    "Suspicious forwarded byte volume is judged separately so the loop cannot hide bandwidth regressions inside broader progress claims.",
                ),
                scorecard_entry(
                    "suspicious_origin_latency_budget",
                    "operator_objectives_v1.budgets + benchmark_results_v1",
                    "suspicious_origin_cost",
                    budget_metric_ids(
                        objectives,
                        "suspicious_forwarded_latency",
                        "suspicious_forwarded_latency_share",
                    ),
                    "optimization_target",
                    "Suspicious forwarded latency share remains distinct from request reach so high-latency leakage cannot hide behind request-count improvements.",
                ),
                scorecard_entry(
                    "category_target_achievement",
                    "operator_objectives_v1.category_postures + benchmark_results_v1",
                    "non_human_category_posture",
                    category_metric_ids.clone(),
                    "optimization_target",
                    "Per-category target achievement is judged as explicit target-vs-achieved outcome, not as a fake scalar budget.",
                ),
            ],
            hard_guardrails: vec![scorecard_entry(
                "beneficial_non_human_no_harm",
                "operator_objectives_v1.category_postures + benchmark_results_v1",
                "beneficial_non_human_posture",
                vec![
                    "friction_mismatch_rate".to_string(),
                    "deny_mismatch_rate".to_string(),
                    "coverage_status".to_string(),
                ],
                "hard_guardrail",
                "Beneficial and verified non-human traffic must remain protected from harmful regression even when hostile-traffic metrics improve.",
            )],
            regression_inputs: vec![
                scorecard_entry(
                    "representative_adversary_regression",
                    "operator_objectives_v1.adversary_sim_expectations + benchmark_results_v1",
                    "representative_adversary_effectiveness",
                    vec![
                        "scenario_goal_success_rate".to_string(),
                        "scenario_escalation_rate".to_string(),
                        "scenario_regression_status".to_string(),
                    ],
                    "regression_input",
                    "Representative adversary regression remains an explicit judge-side input rather than an explanatory footnote.",
                ),
                scorecard_entry(
                    "prior_window_progress",
                    "benchmark_results_v1",
                    "benchmark_progress_comparison",
                    vec![
                        "overall_improvement_status".to_string(),
                        "family_comparison_status:suspicious_origin_cost".to_string(),
                        "family_comparison_status:likely_human_friction".to_string(),
                        "family_comparison_status:non_human_category_posture".to_string(),
                    ],
                    "regression_input",
                    "The judge preserves machine-first prior-window progress semantics instead of allowing later players to invent their own trend story.",
                ),
            ],
            diagnostic_contexts: vec![
                scorecard_entry(
                    "suspicious_origin_diagnostics",
                    "benchmark_results_v1",
                    "suspicious_origin_cost",
                    vec![
                        "suspicious_short_circuit_rate".to_string(),
                        "suspicious_locally_served_byte_share".to_string(),
                        "suspicious_average_forward_latency_ms".to_string(),
                    ],
                    "diagnostic_context",
                    "These suspicious-origin metrics explain how progress was achieved, but they must not collapse the judge into one opaque reward.",
                ),
                scorecard_entry(
                    "representative_adversary_diagnostics",
                    "benchmark_results_v1",
                    "representative_adversary_effectiveness",
                    vec![
                        "scenario_origin_reach_rate".to_string(),
                        "scenario_escalation_rate".to_string(),
                    ],
                    "diagnostic_context",
                    "Representative adversary diagnostics explain pressure shape without replacing the canonical regression inputs.",
                ),
            ],
            comparison_contract: RecursiveImprovementEpisodeComparisonContract {
                source_contract: "benchmark_results_v1 + benchmark_comparison_v1".to_string(),
                comparison_bases: vec![
                    "prior_window".to_string(),
                    "baseline".to_string(),
                    "last_accepted_episode".to_string(),
                ],
                judgment_inputs: vec![
                    "optimization_targets".to_string(),
                    "hard_guardrails".to_string(),
                    "regression_inputs".to_string(),
                ],
                rollback_input_ids: vec![
                    "likely_human_friction_budget".to_string(),
                    "suspicious_origin_request_budget".to_string(),
                    "suspicious_origin_byte_budget".to_string(),
                    "suspicious_origin_latency_budget".to_string(),
                    "category_target_achievement".to_string(),
                    "beneficial_non_human_no_harm".to_string(),
                    "representative_adversary_regression".to_string(),
                ],
                homeostasis_input_ids: vec![
                    "likely_human_friction_budget".to_string(),
                    "suspicious_origin_request_budget".to_string(),
                    "suspicious_origin_byte_budget".to_string(),
                    "suspicious_origin_latency_budget".to_string(),
                    "category_target_achievement".to_string(),
                    "representative_adversary_regression".to_string(),
                    "prior_window_progress".to_string(),
                ],
                minimum_completed_cycles_for_homeostasis: 10,
                scalarization: "forbidden".to_string(),
                note: "Episode judgment, rollback-or-retain, and homeostasis must all reuse the same explicit scorecard partitions; no later player may replace them with a hidden scalar reward.".to_string(),
            },
            note: "benchmark_results_v1 remains the independent machine-first judge surface. This scorecard makes explicit which parts are optimization targets, hard guardrails, regression inputs, and diagnostics without replacing the evaluator boundary."
                .to_string(),
        },
        legal_move_ring: legal_move_ring.clone(),
        safety_gates: vec![
            RecursiveImprovementSafetyGate {
                gate_id: "stale_evidence_refusal".to_string(),
                source_contract: "operator_snapshot_v1.section_metadata".to_string(),
                requirement:
                    "live_traffic, adversary_sim, benchmark_results, and replay_promotion must remain fresh for the current watch window.".to_string(),
                failure_outcome: "refuse_stale_evidence".to_string(),
                note: "The defender must fail closed when required evidence is older than the active watch window."
                    .to_string(),
            },
            RecursiveImprovementSafetyGate {
                gate_id: "contradictory_evidence_refusal".to_string(),
                source_contract: "operator_snapshot_v1 + benchmark_results_v1".to_string(),
                requirement:
                    "benchmark input snapshot and watch-window identity must agree with the snapshot under review.".to_string(),
                failure_outcome: "refuse_contradictory_evidence".to_string(),
                note: "The defender must not act when bounded evidence surfaces disagree about the current subject."
                    .to_string(),
            },
            RecursiveImprovementSafetyGate {
                gate_id: "tuning_eligibility_guardrail".to_string(),
                source_contract: "benchmark_results_v1.tuning_eligibility".to_string(),
                requirement:
                    "protected evidence, category coverage, and verified-identity no-harm checks must remain eligible before tuning.".to_string(),
                failure_outcome: "observe_longer".to_string(),
                note: "Outside-budget pressure alone is insufficient when protected or category-aware evidence is not yet trustworthy."
                    .to_string(),
            },
            RecursiveImprovementSafetyGate {
                gate_id: "manual_review_guardrail".to_string(),
                source_contract:
                    "operator_objectives_v1.rollout_guardrails + benchmark_results_v1.escalation_hint.review_status"
                        .to_string(),
                requirement: "config recommendations remain manual-review bounded until rollout guardrails explicitly relax."
                    .to_string(),
                failure_outcome: "manual_review_required".to_string(),
                note: "The current game contract is recommend-only and must not silently widen into autonomous apply."
                    .to_string(),
            },
        ],
        regression_anchors: vec![
            RecursiveImprovementRegressionAnchor {
                anchor_id: "prior_window_comparison".to_string(),
                source_contract: "benchmark_results_v1".to_string(),
                availability: "active".to_string(),
                requirement: format!(
                    "Episode progress must remain comparable through the {} comparison mode.",
                    objectives.adversary_sim_expectations.comparison_mode
                ),
                note: "Improvement_status and baseline deltas remain authoritative machine-first progress anchors."
                    .to_string(),
            },
            RecursiveImprovementRegressionAnchor {
                anchor_id: "representative_adversary_regression_status".to_string(),
                source_contract: "operator_objectives_v1.adversary_sim_expectations".to_string(),
                availability: "active".to_string(),
                requirement: format!(
                    "Representative adversary episodes must continue to satisfy {}.",
                    objectives.adversary_sim_expectations.regression_status_required
                ),
                note: "Adversary-sim regression is already part of the rule contract and must remain independent of player preference."
                    .to_string(),
            },
            RecursiveImprovementRegressionAnchor {
                anchor_id: "strict_reference_stance".to_string(),
                source_contract: "RSI-METH-1".to_string(),
                availability: "deferred".to_string(),
                requirement:
                    "Later code evolution must continue to pass the strict Human-only / private reference stance."
                        .to_string(),
                note: "The stricter development reference stance is intentionally deferred, but the game contract names it now so later autonomy cannot omit it."
                    .to_string(),
            },
        ],
    }
}

fn scorecard_entry(
    score_id: &str,
    source_contract: &str,
    family_id: &str,
    metric_ids: Vec<String>,
    judgment_role: &str,
    note: &str,
) -> RecursiveImprovementScorecardEntry {
    RecursiveImprovementScorecardEntry {
        score_id: score_id.to_string(),
        source_contract: source_contract.to_string(),
        family_id: family_id.to_string(),
        metric_ids,
        judgment_role: judgment_role.to_string(),
        note: note.to_string(),
    }
}

fn budget_metric_ids(
    objectives: &OperatorObjectivesProfile,
    budget_id: &str,
    fallback_metric_id: &str,
) -> Vec<String> {
    vec![
        objectives
            .budgets
            .iter()
            .find(|budget| budget.budget_id == budget_id)
            .map(|budget| budget.metric.clone())
            .unwrap_or_else(|| fallback_metric_id.to_string()),
    ]
}

pub(crate) fn default_operator_objectives(updated_at_ts: u64) -> OperatorObjectivesProfile {
    OperatorObjectivesProfile {
        schema_version: OPERATOR_OBJECTIVES_SCHEMA_VERSION.to_string(),
        profile_id: SITE_DEFAULT_OBJECTIVE_PROFILE_ID.to_string(),
        revision: objective_revision(updated_at_ts),
        updated_at_ts,
        source: "seeded_default_profile".to_string(),
        window_hours: DEFAULT_WINDOW_HOURS,
        compliance_semantics: "max_ratio_budget".to_string(),
        category_postures: default_category_postures(),
        budgets: vec![
            OperatorObjectiveBudget {
                budget_id: "likely_human_friction".to_string(),
                metric: "likely_human_friction_rate".to_string(),
                comparator: "max_ratio".to_string(),
                target: LIKELY_HUMAN_FRICTION_TARGET,
                near_limit_ratio: DEFAULT_NEAR_LIMIT_RATIO,
                eligible_population: "live:ingress_primary:enforced:likely_human".to_string(),
            },
            OperatorObjectiveBudget {
                budget_id: "suspicious_forwarded_requests".to_string(),
                metric: "suspicious_forwarded_request_rate".to_string(),
                comparator: "max_ratio".to_string(),
                target: SUSPICIOUS_FORWARDED_REQUEST_TARGET,
                near_limit_ratio: DEFAULT_NEAR_LIMIT_RATIO,
                eligible_population: "live:ingress_primary:enforced:suspicious_automation"
                    .to_string(),
            },
            OperatorObjectiveBudget {
                budget_id: "suspicious_forwarded_bytes".to_string(),
                metric: "suspicious_forwarded_byte_rate".to_string(),
                comparator: "max_ratio".to_string(),
                target: SUSPICIOUS_FORWARDED_BYTE_TARGET,
                near_limit_ratio: DEFAULT_NEAR_LIMIT_RATIO,
                eligible_population: "live:ingress_primary:enforced:suspicious_automation"
                    .to_string(),
            },
            OperatorObjectiveBudget {
                budget_id: "suspicious_forwarded_latency".to_string(),
                metric: "suspicious_forwarded_latency_share".to_string(),
                comparator: "max_ratio".to_string(),
                target: SUSPICIOUS_FORWARDED_LATENCY_SHARE_TARGET,
                near_limit_ratio: DEFAULT_NEAR_LIMIT_RATIO,
                eligible_population: "live:ingress_primary:enforced:suspicious_automation"
                    .to_string(),
            },
        ],
        adversary_sim_expectations: OperatorObjectiveAdversarySimExpectations {
            comparison_mode: "prior_window".to_string(),
            max_goal_success_rate: DEFAULT_MAX_GOAL_SUCCESS_RATE,
            min_escalation_rate: DEFAULT_MIN_ESCALATION_RATE,
            regression_status_required: "no_goal_successes".to_string(),
        },
        rollout_guardrails: OperatorObjectivesRolloutGuardrails {
            automated_apply_status: "manual_only".to_string(),
            code_evolution_status: "review_required".to_string(),
        },
    }
}

pub(crate) fn persisted_operator_objectives_from_request(
    request: OperatorObjectivesUpsertRequest,
    updated_at_ts: u64,
    source: &str,
) -> Result<OperatorObjectivesProfile, String> {
    let profile = OperatorObjectivesProfile {
        schema_version: OPERATOR_OBJECTIVES_SCHEMA_VERSION.to_string(),
        profile_id: request.profile_id,
        revision: objective_revision(updated_at_ts),
        updated_at_ts,
        source: source.to_string(),
        window_hours: request.window_hours,
        compliance_semantics: request.compliance_semantics,
        category_postures: request.category_postures,
        budgets: request.budgets,
        adversary_sim_expectations: request.adversary_sim_expectations,
        rollout_guardrails: request.rollout_guardrails,
    };
    validate_operator_objectives(&profile)?;
    Ok(profile)
}

pub(crate) fn validate_operator_objectives(
    profile: &OperatorObjectivesProfile,
) -> Result<(), String> {
    if profile.schema_version != OPERATOR_OBJECTIVES_SCHEMA_VERSION {
        return Err("schema_version must be operator_objectives_v1".to_string());
    }
    if profile.profile_id.trim().is_empty() {
        return Err("profile_id must not be empty".to_string());
    }
    if profile.revision.trim().is_empty() {
        return Err("revision must not be empty".to_string());
    }
    if !(1..=MAX_WINDOW_HOURS).contains(&profile.window_hours) {
        return Err(format!(
            "window_hours must be between 1 and {}",
            MAX_WINDOW_HOURS
        ));
    }
    if profile.compliance_semantics != "max_ratio_budget" {
        return Err("compliance_semantics must be max_ratio_budget".to_string());
    }
    validate_category_postures(profile.category_postures.as_slice())?;
    if profile.budgets.is_empty() {
        return Err("budgets must not be empty".to_string());
    }
    if profile.budgets.len() > MAX_OBJECTIVE_BUDGET_ROWS {
        return Err(format!(
            "budgets must contain at most {} entries",
            MAX_OBJECTIVE_BUDGET_ROWS
        ));
    }
    let mut budget_ids = BTreeSet::new();
    let mut metrics = BTreeSet::new();
    for budget in &profile.budgets {
        if budget.budget_id.trim().is_empty() {
            return Err("budget_id must not be empty".to_string());
        }
        if !budget_ids.insert(budget.budget_id.as_str()) {
            return Err(format!("duplicate budget_id {}", budget.budget_id));
        }
        if !metrics.insert(budget.metric.as_str()) {
            return Err(format!("duplicate metric {}", budget.metric));
        }
        if budget.comparator != "max_ratio" {
            return Err(format!(
                "budget {} comparator must be max_ratio",
                budget.budget_id
            ));
        }
        if !(0.0..=1.0).contains(&budget.target) {
            return Err(format!(
                "budget {} target must be between 0.0 and 1.0",
                budget.budget_id
            ));
        }
        if !(0.0..=1.0).contains(&budget.near_limit_ratio) || budget.near_limit_ratio == 0.0 {
            return Err(format!(
                "budget {} near_limit_ratio must be greater than 0.0 and at most 1.0",
                budget.budget_id
            ));
        }
        if budget.eligible_population.trim().is_empty() {
            return Err(format!(
                "budget {} eligible_population must not be empty",
                budget.budget_id
            ));
        }
    }
    if !matches!(
        profile.adversary_sim_expectations.comparison_mode.as_str(),
        "prior_window" | "baseline"
    ) {
        return Err(
            "adversary_sim_expectations.comparison_mode must be prior_window or baseline"
                .to_string(),
        );
    }
    if !(0.0..=1.0).contains(&profile.adversary_sim_expectations.max_goal_success_rate) {
        return Err(
            "adversary_sim_expectations.max_goal_success_rate must be between 0.0 and 1.0"
                .to_string(),
        );
    }
    if !(0.0..=1.0).contains(&profile.adversary_sim_expectations.min_escalation_rate) {
        return Err(
            "adversary_sim_expectations.min_escalation_rate must be between 0.0 and 1.0"
                .to_string(),
        );
    }
    if profile.adversary_sim_expectations.regression_status_required != "no_goal_successes" {
        return Err(
            "adversary_sim_expectations.regression_status_required must be no_goal_successes"
                .to_string(),
        );
    }
    if !matches!(
        profile.rollout_guardrails.automated_apply_status.as_str(),
        "manual_only" | "canary_only"
    ) {
        return Err(
            "rollout_guardrails.automated_apply_status must be manual_only or canary_only"
                .to_string(),
        );
    }
    if profile.rollout_guardrails.code_evolution_status != "review_required" {
        return Err(
            "rollout_guardrails.code_evolution_status must be review_required".to_string(),
        );
    }
    Ok(())
}

fn objective_revision(updated_at_ts: u64) -> String {
    format!("rev-{updated_at_ts}")
}

fn default_category_postures() -> Vec<OperatorObjectiveCategoryPosture> {
    vec![
        category_posture(NonHumanCategoryId::IndexingBot, "cost_reduced"),
        category_posture(NonHumanCategoryId::AiScraperBot, "blocked"),
        category_posture(NonHumanCategoryId::AutomatedBrowser, "blocked"),
        category_posture(NonHumanCategoryId::HttpAgent, "restricted"),
        category_posture(NonHumanCategoryId::BrowserAgent, "restricted"),
        category_posture(NonHumanCategoryId::AgentOnBehalfOfHuman, "tolerated"),
        category_posture(NonHumanCategoryId::VerifiedBeneficialBot, "allowed"),
        category_posture(NonHumanCategoryId::UnknownNonHuman, "restricted"),
    ]
}

fn category_posture(
    category_id: NonHumanCategoryId,
    posture: &str,
) -> OperatorObjectiveCategoryPosture {
    OperatorObjectiveCategoryPosture {
        category_id,
        posture: posture.to_string(),
    }
}

fn validate_category_postures(
    rows: &[OperatorObjectiveCategoryPosture],
) -> Result<(), String> {
    let taxonomy = canonical_non_human_taxonomy();
    let expected_categories: BTreeSet<_> = taxonomy.categories.iter().map(|row| row.category_id).collect();
    if rows.len() != expected_categories.len() {
        return Err(format!(
            "category_postures must contain exactly {} canonical categories",
            expected_categories.len()
        ));
    }

    let posture_scale = posture_scale();
    let mut seen_categories = BTreeSet::new();
    for row in rows {
        if !expected_categories.contains(&row.category_id) {
            return Err(format!(
                "category_postures contains unknown category {}",
                row.category_id.as_str()
            ));
        }
        if !seen_categories.insert(row.category_id) {
            return Err(format!(
                "category_postures contains duplicate category {}",
                row.category_id.as_str()
            ));
        }
        if !posture_scale.iter().any(|value| value == &row.posture) {
            return Err(format!(
                "category_postures {} posture must be one of the canonical posture scale values",
                row.category_id.as_str()
            ));
        }
    }

    for category_id in expected_categories {
        if !seen_categories.contains(&category_id) {
            return Err(format!(
                "category_postures missing canonical category {}",
                category_id.as_str()
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        default_operator_objectives, persisted_operator_objectives_from_request,
        recursive_improvement_game_contract_v1, validate_operator_objectives,
        OperatorObjectiveAdversarySimExpectations, OperatorObjectiveBudget,
        OperatorObjectiveCategoryPosture, OperatorObjectivesRolloutGuardrails,
        OperatorObjectivesUpsertRequest, OPERATOR_OBJECTIVES_SCHEMA_VERSION,
    };
    use crate::config::controller_legal_move_ring_v1;

    #[test]
    fn default_operator_objectives_expose_site_owned_profile_and_budget_catalog() {
        let profile = default_operator_objectives(1_700_000_000);

        assert_eq!(profile.schema_version, OPERATOR_OBJECTIVES_SCHEMA_VERSION);
        assert_eq!(profile.profile_id, "site_default_v1");
        assert_eq!(profile.revision, "rev-1700000000");
        assert_eq!(profile.updated_at_ts, 1_700_000_000);
        assert_eq!(profile.source, "seeded_default_profile");
        assert_eq!(profile.window_hours, 24);
        assert_eq!(profile.compliance_semantics, "max_ratio_budget");
        assert_eq!(profile.category_postures.len(), 8);
        assert_eq!(profile.category_postures[0].category_id.as_str(), "indexing_bot");
        assert_eq!(profile.category_postures[0].posture, "cost_reduced");
        assert_eq!(
            profile
                .category_postures
                .iter()
                .find(|row| row.category_id.as_str() == "verified_beneficial_bot")
                .expect("verified beneficial bot row")
                .posture,
            "allowed"
        );
        assert_eq!(profile.budgets.len(), 4);
        assert_eq!(profile.budgets[0].budget_id, "likely_human_friction");
        assert_eq!(profile.budgets[1].metric, "suspicious_forwarded_request_rate");
        assert_eq!(profile.budgets[2].metric, "suspicious_forwarded_byte_rate");
        assert_eq!(profile.budgets[3].metric, "suspicious_forwarded_latency_share");
    }

    #[test]
    fn default_operator_objectives_keep_typed_expectations_and_manual_guardrails_explicit() {
        let profile = default_operator_objectives(1_700_000_000);

        assert_eq!(
            profile.adversary_sim_expectations.comparison_mode,
            "prior_window"
        );
        assert_eq!(
            profile.adversary_sim_expectations.max_goal_success_rate,
            0.0
        );
        assert_eq!(
            profile.adversary_sim_expectations.min_escalation_rate,
            0.25
        );
        assert_eq!(
            profile.adversary_sim_expectations.regression_status_required,
            "no_goal_successes"
        );
        assert_eq!(
            profile.rollout_guardrails.automated_apply_status,
            "manual_only"
        );
        assert_eq!(
            profile.rollout_guardrails.code_evolution_status,
            "review_required"
        );
    }

    #[test]
    fn persisted_operator_objectives_from_request_assigns_revisioned_server_metadata() {
        let request = OperatorObjectivesUpsertRequest {
            profile_id: "custom_profile".to_string(),
            window_hours: 12,
            compliance_semantics: "max_ratio_budget".to_string(),
            category_postures: vec![
                OperatorObjectiveCategoryPosture {
                    category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::IndexingBot,
                    posture: "cost_reduced".to_string(),
                },
                OperatorObjectiveCategoryPosture {
                    category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::AiScraperBot,
                    posture: "blocked".to_string(),
                },
                OperatorObjectiveCategoryPosture {
                    category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::AutomatedBrowser,
                    posture: "blocked".to_string(),
                },
                OperatorObjectiveCategoryPosture {
                    category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::HttpAgent,
                    posture: "restricted".to_string(),
                },
                OperatorObjectiveCategoryPosture {
                    category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::BrowserAgent,
                    posture: "restricted".to_string(),
                },
                OperatorObjectiveCategoryPosture {
                    category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::AgentOnBehalfOfHuman,
                    posture: "tolerated".to_string(),
                },
                OperatorObjectiveCategoryPosture {
                    category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::VerifiedBeneficialBot,
                    posture: "allowed".to_string(),
                },
                OperatorObjectiveCategoryPosture {
                    category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::UnknownNonHuman,
                    posture: "restricted".to_string(),
                },
            ],
            budgets: vec![OperatorObjectiveBudget {
                budget_id: "likely_human_friction".to_string(),
                metric: "likely_human_friction_rate".to_string(),
                comparator: "max_ratio".to_string(),
                target: 0.03,
                near_limit_ratio: 0.8,
                eligible_population: "live:ingress_primary:enforced:likely_human".to_string(),
            }],
            adversary_sim_expectations: OperatorObjectiveAdversarySimExpectations {
                comparison_mode: "prior_window".to_string(),
                max_goal_success_rate: 0.0,
                min_escalation_rate: 0.5,
                regression_status_required: "no_goal_successes".to_string(),
            },
            rollout_guardrails: OperatorObjectivesRolloutGuardrails {
                automated_apply_status: "manual_only".to_string(),
                code_evolution_status: "review_required".to_string(),
            },
        };

        let persisted = persisted_operator_objectives_from_request(
            request,
            1_700_000_100,
            "manual_admin_profile",
        )
        .expect("request validates");

        assert_eq!(persisted.schema_version, OPERATOR_OBJECTIVES_SCHEMA_VERSION);
        assert_eq!(persisted.revision, "rev-1700000100");
        assert_eq!(persisted.updated_at_ts, 1_700_000_100);
        assert_eq!(persisted.source, "manual_admin_profile");
        assert_eq!(persisted.category_postures.len(), 8);
    }

    #[test]
    fn validate_operator_objectives_rejects_duplicate_metric_and_bad_category_posture() {
        let mut invalid = default_operator_objectives(1_700_000_000);
        invalid.budgets.push(invalid.budgets[0].clone());
        invalid.category_postures[0].posture = "unknown".to_string();

        let error = validate_operator_objectives(&invalid).expect_err("profile rejected");

        assert!(error.contains("category_postures") || error.contains("duplicate metric"));
    }

    #[test]
    fn recursive_improvement_game_contract_names_rules_evaluator_moves_gates_and_anchors() {
        let objectives = default_operator_objectives(1_700_000_000);
        let legal_move_ring = controller_legal_move_ring_v1();
        let contract = recursive_improvement_game_contract_v1(&objectives, &legal_move_ring);

        assert_eq!(
            contract.schema_version,
            "recursive_improvement_game_contract_v1"
        );
        assert_eq!(contract.contract_id, "shuma_recursive_improvement_game_v1");
        assert_eq!(
            contract.immutable_rules.operator_rule_surface,
            "operator_objectives_v1"
        );
        assert_eq!(
            contract.evaluator_scorecard.benchmark_results_schema_version,
            "benchmark_results_v1"
        );
        assert_eq!(
            contract.evaluator_scorecard.optimization_targets.len(),
            5
        );
        assert_eq!(contract.evaluator_scorecard.hard_guardrails.len(), 1);
        assert_eq!(contract.evaluator_scorecard.regression_inputs.len(), 2);
        assert_eq!(contract.evaluator_scorecard.diagnostic_contexts.len(), 2);
        assert!(contract
            .evaluator_scorecard
            .family_ids
            .contains(&"representative_adversary_effectiveness".to_string()));
        assert!(contract
            .evaluator_scorecard
            .optimization_targets
            .iter()
            .any(|entry| entry.score_id == "likely_human_friction_budget"));
        assert!(contract
            .evaluator_scorecard
            .optimization_targets
            .iter()
            .any(|entry| entry.score_id == "category_target_achievement"));
        assert!(contract
            .evaluator_scorecard
            .hard_guardrails
            .iter()
            .any(|entry| entry.score_id == "beneficial_non_human_no_harm"));
        assert!(contract
            .evaluator_scorecard
            .regression_inputs
            .iter()
            .any(|entry| entry.score_id == "representative_adversary_regression"));
        assert!(contract
            .evaluator_scorecard
            .regression_inputs
            .iter()
            .any(|entry| entry.score_id == "prior_window_progress"));
        assert!(contract
            .evaluator_scorecard
            .diagnostic_contexts
            .iter()
            .any(|entry| entry.score_id == "suspicious_origin_diagnostics"));
        assert!(contract
            .evaluator_scorecard
            .diagnostic_contexts
            .iter()
            .any(|entry| entry.score_id == "representative_adversary_diagnostics"));
        assert_eq!(
            contract
                .evaluator_scorecard
                .comparison_contract
                .minimum_completed_cycles_for_homeostasis,
            10
        );
        assert_eq!(
            contract
                .evaluator_scorecard
                .comparison_contract
                .judgment_inputs,
            vec![
                "optimization_targets".to_string(),
                "hard_guardrails".to_string(),
                "regression_inputs".to_string(),
            ]
        );
        assert!(contract
            .evaluator_scorecard
            .comparison_contract
            .homeostasis_input_ids
            .contains(&"likely_human_friction_budget".to_string()));
        assert!(contract
            .evaluator_scorecard
            .comparison_contract
            .homeostasis_input_ids
            .contains(&"representative_adversary_regression".to_string()));
        assert_eq!(
            contract.legal_move_ring.legal_ring,
            "controller_tunable"
        );
        assert!(contract
            .legal_move_ring
            .controller_tunable_group_ids
            .contains(&"not_a_bot.policy".to_string()));
        assert!(contract
            .safety_gates
            .iter()
            .any(|gate| gate.gate_id == "stale_evidence_refusal"));
        assert!(contract
            .regression_anchors
            .iter()
            .any(|anchor| anchor.anchor_id == "prior_window_comparison"));
    }

    #[test]
    fn recursive_improvement_game_contract_partitions_metric_ids_without_collapsing_to_scalar() {
        let objectives = default_operator_objectives(1_700_000_000);
        let legal_move_ring = controller_legal_move_ring_v1();
        let contract = recursive_improvement_game_contract_v1(&objectives, &legal_move_ring);

        let category_target = contract
            .evaluator_scorecard
            .optimization_targets
            .iter()
            .find(|entry| entry.score_id == "category_target_achievement")
            .expect("category target achievement present");
        let beneficial_guardrail = contract
            .evaluator_scorecard
            .hard_guardrails
            .iter()
            .find(|entry| entry.score_id == "beneficial_non_human_no_harm")
            .expect("beneficial non-human no-harm present");
        let suspicious_diagnostics = contract
            .evaluator_scorecard
            .diagnostic_contexts
            .iter()
            .find(|entry| entry.score_id == "suspicious_origin_diagnostics")
            .expect("suspicious origin diagnostics present");

        assert_eq!(
            category_target.family_id,
            "non_human_category_posture"
        );
        assert_eq!(category_target.metric_ids.len(), objectives.category_postures.len());
        assert!(category_target.metric_ids.contains(
            &"category_posture_alignment:indexing_bot".to_string()
        ));
        assert!(category_target.metric_ids.contains(
            &"category_posture_alignment:verified_beneficial_bot".to_string()
        ));
        assert_eq!(
            beneficial_guardrail.metric_ids,
            vec![
                "friction_mismatch_rate".to_string(),
                "deny_mismatch_rate".to_string(),
                "coverage_status".to_string(),
            ]
        );
        assert_eq!(
            suspicious_diagnostics.metric_ids,
            vec![
                "suspicious_short_circuit_rate".to_string(),
                "suspicious_locally_served_byte_share".to_string(),
                "suspicious_average_forward_latency_ms".to_string(),
            ]
        );
        assert_eq!(
            contract.evaluator_scorecard.comparison_contract.scalarization,
            "forbidden"
        );
        assert_eq!(
            contract.evaluator_scorecard.comparison_contract.rollback_input_ids,
            vec![
                "likely_human_friction_budget".to_string(),
                "suspicious_origin_request_budget".to_string(),
                "suspicious_origin_byte_budget".to_string(),
                "suspicious_origin_latency_budget".to_string(),
                "category_target_achievement".to_string(),
                "beneficial_non_human_no_harm".to_string(),
                "representative_adversary_regression".to_string(),
            ]
        );
    }
}
