use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::runtime::non_human_taxonomy::{
    canonical_non_human_taxonomy, posture_scale, NonHumanCategoryId,
};

pub(crate) const OPERATOR_OBJECTIVES_SCHEMA_VERSION: &str = "operator_objectives_v1";

const SITE_DEFAULT_OBJECTIVE_PROFILE_ID: &str = "site_default_v1";
pub(super) const DEFAULT_WINDOW_HOURS: u64 = 24;
const DEFAULT_NEAR_LIMIT_RATIO: f64 = 0.75;
const LIKELY_HUMAN_FRICTION_TARGET: f64 = 0.02;
const SUSPICIOUS_FORWARDED_REQUEST_TARGET: f64 = 0.10;
const SUSPICIOUS_FORWARDED_BYTE_TARGET: f64 = 0.10;
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

pub(crate) fn operator_objectives_watch_window_seconds(
    profile: &OperatorObjectivesProfile,
) -> u64 {
    profile.window_hours.saturating_mul(3600)
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
        validate_operator_objectives, OperatorObjectiveAdversarySimExpectations,
        OperatorObjectiveBudget, OperatorObjectiveCategoryPosture,
        OperatorObjectivesRolloutGuardrails,
        OperatorObjectivesUpsertRequest, OPERATOR_OBJECTIVES_SCHEMA_VERSION,
    };

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
        assert_eq!(profile.budgets.len(), 3);
        assert_eq!(profile.budgets[0].budget_id, "likely_human_friction");
        assert_eq!(profile.budgets[1].metric, "suspicious_forwarded_request_rate");
        assert_eq!(profile.budgets[2].metric, "suspicious_forwarded_byte_rate");
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
}
