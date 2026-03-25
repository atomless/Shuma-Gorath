use crate::config::AllowedActionsSurface;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::runtime::non_human_taxonomy::{
    canonical_non_human_taxonomy, posture_scale, NonHumanCategoryId,
};

pub(crate) const OPERATOR_OBJECTIVES_SCHEMA_VERSION: &str = "operator_objectives_v1";
pub(crate) const RECURSIVE_IMPROVEMENT_GAME_CONTRACT_SCHEMA_VERSION: &str =
    "game_contract_v1";
pub(crate) const RECURSIVE_IMPROVEMENT_JUDGE_SCORECARD_SCHEMA_VERSION: &str =
    "judge_scorecard_v1";

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RecursiveImprovementGameRules {
    pub immutable_rule_surface: String,
    pub objective_surface_schema_version: String,
    pub objective_revision: String,
    pub objective_profile_id: String,
    pub compliance_semantics: String,
    pub window_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RecursiveImprovementGameFixedPayoffs {
    pub independent_judge: String,
    pub benchmark_results_schema_version: String,
    pub benchmark_suite_schema_version: String,
    pub comparison_mode: String,
    pub optimization_budget_ids: Vec<String>,
    pub category_target_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RecursiveImprovementGameLegalMoves {
    pub game_role: String,
    pub immutable_rule_surface: String,
    pub allowed_actions_schema_version: String,
    pub controller_mutability_schema_version: String,
    pub write_surface: String,
    pub proposal_mode: String,
    pub controller_tunable_group_ids: Vec<String>,
    pub manual_only_group_ids: Vec<String>,
    pub forbidden_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RecursiveImprovementGameSafetyGates {
    pub automated_apply_status: String,
    pub code_evolution_status: String,
    pub tuning_gate_surface: String,
    pub protected_evidence_surface: String,
    pub fail_closed_conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RecursiveImprovementGameRegressionAnchors {
    pub anchor_ids: Vec<String>,
    pub anchor_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecursiveImprovementJudgeScorecardEntry {
    pub scorecard_id: String,
    pub subject_kind: String,
    pub source_surface: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metric_ids: Vec<String>,
    pub evaluation_mode: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecursiveImprovementJudgeHomeostasisInputs {
    pub cycle_window: String,
    pub comparison_basis: String,
    pub status_surface: String,
    pub required_scorecard_entry_ids: Vec<String>,
    pub held_out_override_surface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RecursiveImprovementJudgeScorecard {
    pub scorecard_surface_schema_version: String,
    pub optimization_targets: Vec<RecursiveImprovementJudgeScorecardEntry>,
    pub hard_guardrails: Vec<RecursiveImprovementJudgeScorecardEntry>,
    pub regression_anchors: Vec<RecursiveImprovementJudgeScorecardEntry>,
    pub explanatory_diagnostics: Vec<RecursiveImprovementJudgeScorecardEntry>,
    pub homeostasis_inputs: RecursiveImprovementJudgeHomeostasisInputs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RecursiveImprovementGameContract {
    pub schema_version: String,
    pub contract_revision: String,
    pub rules: RecursiveImprovementGameRules,
    pub fixed_payoffs: RecursiveImprovementGameFixedPayoffs,
    pub judge_scorecard: RecursiveImprovementJudgeScorecard,
    pub legal_moves: RecursiveImprovementGameLegalMoves,
    pub safety_gates: RecursiveImprovementGameSafetyGates,
    pub regression_anchors: RecursiveImprovementGameRegressionAnchors,
}

pub(crate) fn operator_objectives_watch_window_seconds(
    profile: &OperatorObjectivesProfile,
) -> u64 {
    profile.window_hours.saturating_mul(3600)
}

pub(crate) fn recursive_improvement_game_contract_v1(
    objectives: &OperatorObjectivesProfile,
    allowed_actions: &AllowedActionsSurface,
) -> RecursiveImprovementGameContract {
    let judge_scorecard = recursive_improvement_judge_scorecard_v1(objectives);
    RecursiveImprovementGameContract {
        schema_version: RECURSIVE_IMPROVEMENT_GAME_CONTRACT_SCHEMA_VERSION.to_string(),
        contract_revision: format!(
            "{}:{}:{}:{}",
            objectives.revision,
            allowed_actions.schema_version,
            allowed_actions.controller_mutability_schema_version,
            judge_scorecard.scorecard_surface_schema_version
        ),
        rules: RecursiveImprovementGameRules {
            immutable_rule_surface: "operator_objectives_v1".to_string(),
            objective_surface_schema_version: objectives.schema_version.clone(),
            objective_revision: objectives.revision.clone(),
            objective_profile_id: objectives.profile_id.clone(),
            compliance_semantics: objectives.compliance_semantics.clone(),
            window_hours: objectives.window_hours,
        },
        fixed_payoffs: RecursiveImprovementGameFixedPayoffs {
            independent_judge: "machine_first_benchmark_stack".to_string(),
            benchmark_results_schema_version: "benchmark_results_v1".to_string(),
            benchmark_suite_schema_version: "benchmark_suite_v1".to_string(),
            comparison_mode: objectives.adversary_sim_expectations.comparison_mode.clone(),
            optimization_budget_ids: objectives
                .budgets
                .iter()
                .map(|budget| budget.budget_id.clone())
                .collect(),
            category_target_family: "non_human_category_posture".to_string(),
        },
        judge_scorecard,
        legal_moves: RecursiveImprovementGameLegalMoves {
            game_role: allowed_actions.game_role.clone(),
            immutable_rule_surface: allowed_actions.immutable_rule_surface.clone(),
            allowed_actions_schema_version: allowed_actions.schema_version.clone(),
            controller_mutability_schema_version: allowed_actions
                .controller_mutability_schema_version
                .clone(),
            write_surface: allowed_actions.write_surface.clone(),
            proposal_mode: allowed_actions.proposal_mode.clone(),
            controller_tunable_group_ids: allowed_actions.allowed_group_ids.clone(),
            manual_only_group_ids: allowed_actions.manual_only_group_ids.clone(),
            forbidden_group_ids: allowed_actions.forbidden_group_ids.clone(),
        },
        safety_gates: RecursiveImprovementGameSafetyGates {
            automated_apply_status: objectives
                .rollout_guardrails
                .automated_apply_status
                .clone(),
            code_evolution_status: objectives
                .rollout_guardrails
                .code_evolution_status
                .clone(),
            tuning_gate_surface: "benchmark_results_v1.tuning_eligibility".to_string(),
            protected_evidence_surface: "replay_promotion_v1".to_string(),
            fail_closed_conditions: vec![
                "operator_snapshot_not_materialized".to_string(),
                "config_unavailable".to_string(),
                "verified_identity_no_harm_guardrails".to_string(),
                "protected_replay_evidence_required".to_string(),
            ],
        },
        regression_anchors: RecursiveImprovementGameRegressionAnchors {
            anchor_ids: vec![
                "likely_human_friction".to_string(),
                "verified_identity_no_harm".to_string(),
                "protected_evidence_required".to_string(),
            ],
            anchor_sources: vec![
                "operator_objectives_v1.budgets".to_string(),
                "benchmark_results_v1.tuning_eligibility".to_string(),
                "replay_promotion_v1".to_string(),
            ],
        },
    }
}

fn recursive_improvement_judge_scorecard_v1(
    objectives: &OperatorObjectivesProfile,
) -> RecursiveImprovementJudgeScorecard {
    let mut optimization_targets = objective_budget_scorecard_entries(objectives);
    optimization_targets.push(RecursiveImprovementJudgeScorecardEntry {
        scorecard_id: "family:representative_adversary_effectiveness".to_string(),
        subject_kind: "benchmark_family".to_string(),
        source_surface:
            "benchmark_results_v1.families.representative_adversary_effectiveness".to_string(),
        metric_ids: vec![
            "scenario_goal_success_rate".to_string(),
            "scenario_escalation_rate".to_string(),
        ],
        evaluation_mode: "minimize_goal_success_and_preserve_escalation".to_string(),
        note: "The judge must treat representative adversary success as a primary optimization target so the loop cannot improve only by shifting collateral cost onto humans or tolerated traffic."
            .to_string(),
    });
    optimization_targets.push(RecursiveImprovementJudgeScorecardEntry {
        scorecard_id: "family:non_human_category_posture".to_string(),
        subject_kind: "benchmark_family".to_string(),
        source_surface: "benchmark_results_v1.families.non_human_category_posture".to_string(),
        metric_ids: objectives
            .category_postures
            .iter()
            .map(|row| format!("category_posture_alignment:{}", row.category_id.as_str()))
            .collect(),
        evaluation_mode: "maximize_category_target_achievement".to_string(),
        note: "Taxonomy-category target achievement remains a first-class optimization target even though it is expressed as achieved-vs-target outcomes rather than numeric resource budgets."
            .to_string(),
    });

    let hard_guardrails = vec![
        RecursiveImprovementJudgeScorecardEntry {
            scorecard_id: "guardrail:beneficial_non_human_posture".to_string(),
            subject_kind: "benchmark_family".to_string(),
            source_surface:
                "benchmark_results_v1.families.beneficial_non_human_posture".to_string(),
            metric_ids: vec![
                "friction_mismatch_rate".to_string(),
                "deny_mismatch_rate".to_string(),
                "taxonomy_alignment_mismatch_rate".to_string(),
                "verified_botness_conflict_rate".to_string(),
                "user_triggered_agent_friction_mismatch_rate".to_string(),
            ],
            evaluation_mode: "hard_no_harm_gate".to_string(),
            note: "Verified and tolerated non-human traffic must remain protected by explicit no-harm gates rather than being traded away for better hostile-bot pressure."
                .to_string(),
        },
        RecursiveImprovementJudgeScorecardEntry {
            scorecard_id: "guardrail:protected_evidence_required".to_string(),
            subject_kind: "guardrail_surface".to_string(),
            source_surface:
                "benchmark_results_v1.tuning_eligibility+replay_promotion_v1".to_string(),
            metric_ids: vec![
                "tuning_eligibility.status".to_string(),
                "replay_promotion.availability".to_string(),
            ],
            evaluation_mode: "hard_evidence_gate".to_string(),
            note: "The judge must refuse player-side progress claims when protected evidence and replay-promotion prerequisites are not satisfied."
                .to_string(),
        },
    ];

    let regression_anchors = vec![
        RecursiveImprovementJudgeScorecardEntry {
            scorecard_id: "anchor:likely_human_friction".to_string(),
            subject_kind: "regression_anchor".to_string(),
            source_surface: "benchmark_results_v1.families.likely_human_friction".to_string(),
            metric_ids: vec!["likely_human_friction_rate".to_string()],
            evaluation_mode: "must_not_regress".to_string(),
            note: "Likely-human friction remains both an optimization target and a regression anchor so later broader search cannot normalize human harm."
                .to_string(),
        },
        RecursiveImprovementJudgeScorecardEntry {
            scorecard_id: "anchor:verified_identity_no_harm".to_string(),
            subject_kind: "regression_anchor".to_string(),
            source_surface:
                "benchmark_results_v1.families.beneficial_non_human_posture".to_string(),
            metric_ids: vec![
                "friction_mismatch_rate".to_string(),
                "deny_mismatch_rate".to_string(),
                "verified_botness_conflict_rate".to_string(),
            ],
            evaluation_mode: "must_not_regress".to_string(),
            note: "Verified-identity no-harm obligations remain a regression anchor even when later episodes optimize more permissive product stances."
                .to_string(),
        },
        RecursiveImprovementJudgeScorecardEntry {
            scorecard_id: "anchor:protected_evidence_required".to_string(),
            subject_kind: "regression_anchor".to_string(),
            source_surface:
                "benchmark_results_v1.tuning_eligibility+replay_promotion_v1".to_string(),
            metric_ids: vec![
                "tuning_eligibility.status".to_string(),
                "replay_promotion.availability".to_string(),
            ],
            evaluation_mode: "must_remain_satisfied".to_string(),
            note: "Protected evidence and replay-promotion readiness remain a regression anchor so later automation cannot quietly degrade the truth basis of evaluation."
                .to_string(),
        },
    ];

    let explanatory_diagnostics = vec![
        RecursiveImprovementJudgeScorecardEntry {
            scorecard_id: "diagnostic:non_human_classification_readiness".to_string(),
            subject_kind: "diagnostic_surface".to_string(),
            source_surface: "benchmark_results_v1.non_human_classification".to_string(),
            metric_ids: vec!["status".to_string(), "blockers".to_string()],
            evaluation_mode: "explain_but_do_not_optimize_directly".to_string(),
            note: "Classification readiness explains why the loop may refuse to act, but it is not itself a direct episode reward."
                .to_string(),
        },
        RecursiveImprovementJudgeScorecardEntry {
            scorecard_id: "diagnostic:non_human_coverage_summary".to_string(),
            subject_kind: "diagnostic_surface".to_string(),
            source_surface: "benchmark_results_v1.non_human_coverage".to_string(),
            metric_ids: vec!["status".to_string(), "covered_categories".to_string()],
            evaluation_mode: "explain_but_do_not_optimize_directly".to_string(),
            note: "Coverage diagnostics explain confidence and representativeness, but they should not collapse into the reward signal in place of observed outcome truth."
                .to_string(),
        },
    ];

    let mut required_scorecard_entry_ids = optimization_targets
        .iter()
        .map(|entry| entry.scorecard_id.clone())
        .collect::<Vec<_>>();
    required_scorecard_entry_ids.extend(
        hard_guardrails
            .iter()
            .map(|entry| entry.scorecard_id.clone()),
    );

    RecursiveImprovementJudgeScorecard {
        scorecard_surface_schema_version:
            RECURSIVE_IMPROVEMENT_JUDGE_SCORECARD_SCHEMA_VERSION.to_string(),
        optimization_targets,
        hard_guardrails,
        regression_anchors,
        explanatory_diagnostics,
        homeostasis_inputs: RecursiveImprovementJudgeHomeostasisInputs {
            cycle_window: "last_10_completed_cycles".to_string(),
            comparison_basis: "prior_window_and_baseline_reference".to_string(),
            status_surface: "benchmark_results_v1.overall_status+improvement_status"
                .to_string(),
            required_scorecard_entry_ids,
            held_out_override_surface: "held_out_eval_ring_v1".to_string(),
        },
    }
}

fn objective_budget_scorecard_entries(
    objectives: &OperatorObjectivesProfile,
) -> Vec<RecursiveImprovementJudgeScorecardEntry> {
    objectives
        .budgets
        .iter()
        .map(|budget| RecursiveImprovementJudgeScorecardEntry {
            scorecard_id: format!("budget:{}", budget.budget_id),
            subject_kind: "numeric_budget".to_string(),
            source_surface: format!("operator_objectives_v1.budgets:{}", budget.budget_id),
            metric_ids: vec![budget.metric.clone()],
            evaluation_mode: "optimize_within_numeric_budget".to_string(),
            note: format!(
                "Budget `{}` remains a primary judge target over `{}` for `{}`.",
                budget.budget_id, budget.metric, budget.eligible_population
            ),
        })
        .collect()
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
        OperatorObjectiveAdversarySimExpectations,
        OperatorObjectiveBudget, OperatorObjectiveCategoryPosture,
        OperatorObjectivesRolloutGuardrails, OperatorObjectivesUpsertRequest,
        OPERATOR_OBJECTIVES_SCHEMA_VERSION,
        RECURSIVE_IMPROVEMENT_GAME_CONTRACT_SCHEMA_VERSION,
    };
    use crate::config::allowed_actions_v1;

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
    fn recursive_improvement_game_contract_names_rules_judge_and_legal_move_ring() {
        let objectives = default_operator_objectives(1_700_000_000);
        let allowed_actions = allowed_actions_v1();

        let contract =
            recursive_improvement_game_contract_v1(&objectives, &allowed_actions);

        assert_eq!(
            contract.schema_version,
            RECURSIVE_IMPROVEMENT_GAME_CONTRACT_SCHEMA_VERSION
        );
        assert_eq!(contract.rules.immutable_rule_surface, "operator_objectives_v1");
        assert_eq!(contract.rules.objective_revision, objectives.revision);
        assert_eq!(
            contract.fixed_payoffs.independent_judge,
            "machine_first_benchmark_stack"
        );
        assert_eq!(
            contract.fixed_payoffs.benchmark_results_schema_version,
            "benchmark_results_v1"
        );
        assert_eq!(
            contract.fixed_payoffs.benchmark_suite_schema_version,
            "benchmark_suite_v1"
        );
        assert_eq!(contract.legal_moves.game_role, "legal_move_ring");
        assert_eq!(
            contract.legal_moves.allowed_actions_schema_version,
            "allowed_actions_v1"
        );
        assert!(contract
            .legal_moves
            .controller_tunable_group_ids
            .contains(&"not_a_bot.policy".to_string()));
        assert_eq!(
            contract.safety_gates.automated_apply_status,
            objectives.rollout_guardrails.automated_apply_status
        );
        assert_eq!(
            contract.judge_scorecard.scorecard_surface_schema_version,
            "judge_scorecard_v1"
        );
        assert_eq!(
            contract.judge_scorecard.optimization_targets.len(),
            objectives.budgets.len() + 2
        );
        assert!(contract
            .judge_scorecard
            .optimization_targets
            .iter()
            .any(|entry| entry.scorecard_id == "budget:likely_human_friction"));
        assert!(contract
            .judge_scorecard
            .optimization_targets
            .iter()
            .any(|entry| entry.scorecard_id == "family:representative_adversary_effectiveness"));
        assert!(contract
            .judge_scorecard
            .hard_guardrails
            .iter()
            .any(|entry| entry.scorecard_id == "guardrail:beneficial_non_human_posture"));
        assert!(contract
            .judge_scorecard
            .hard_guardrails
            .iter()
            .any(|entry| entry.scorecard_id == "guardrail:protected_evidence_required"));
        assert!(contract
            .regression_anchors
            .anchor_ids
            .contains(&"likely_human_friction".to_string()));
        assert!(contract
            .judge_scorecard
            .regression_anchors
            .iter()
            .any(|entry| entry.scorecard_id == "anchor:verified_identity_no_harm"));
        assert_eq!(
            contract.judge_scorecard.homeostasis_inputs.cycle_window,
            "last_10_completed_cycles"
        );
        assert!(contract
            .judge_scorecard
            .homeostasis_inputs
            .required_scorecard_entry_ids
            .contains(&"budget:likely_human_friction".to_string()));
        assert!(contract
            .judge_scorecard
            .homeostasis_inputs
            .required_scorecard_entry_ids
            .contains(&"family:non_human_category_posture".to_string()));
    }
}
