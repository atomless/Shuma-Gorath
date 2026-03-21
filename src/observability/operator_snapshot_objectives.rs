use serde::{Deserialize, Serialize};

use super::operator_snapshot_verified_identity::{
    placeholder_section, OperatorSnapshotPlaceholderSection,
};

const BACKEND_DEFAULT_OBJECTIVE_PROFILE_ID: &str = "backend_default_v1";
pub(super) const DEFAULT_WINDOW_HOURS: u64 = 24;
const DEFAULT_NEAR_LIMIT_RATIO: f64 = 0.75;
const LIKELY_HUMAN_FRICTION_TARGET: f64 = 0.02;
const SUSPICIOUS_FORWARDED_REQUEST_TARGET: f64 = 0.10;
const SUSPICIOUS_FORWARDED_BYTE_TARGET: f64 = 0.10;

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
pub(crate) struct OperatorObjectivesRolloutGuardrails {
    pub automated_apply_status: String,
    pub code_evolution_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorObjectivesProfile {
    pub profile_id: String,
    pub source: String,
    pub window_hours: u64,
    pub compliance_semantics: String,
    pub non_human_posture: String,
    pub budgets: Vec<OperatorObjectiveBudget>,
    pub adversary_sim_expectations: OperatorSnapshotPlaceholderSection,
    pub rollout_guardrails: OperatorObjectivesRolloutGuardrails,
}

pub(super) fn default_operator_objectives() -> OperatorObjectivesProfile {
    OperatorObjectivesProfile {
        profile_id: BACKEND_DEFAULT_OBJECTIVE_PROFILE_ID.to_string(),
        source: "backend_default_profile".to_string(),
        window_hours: DEFAULT_WINDOW_HOURS,
        compliance_semantics: "max_ratio_budget".to_string(),
        non_human_posture: "treat_as_untrusted_until_identity_foundation".to_string(),
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
        adversary_sim_expectations: placeholder_section(
            "not_yet_materialized",
            "Scenario-family benchmark expectations land with benchmark result materialization.",
        ),
        rollout_guardrails: OperatorObjectivesRolloutGuardrails {
            automated_apply_status: "manual_only".to_string(),
            code_evolution_status: "review_required".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::default_operator_objectives;

    #[test]
    fn default_operator_objectives_expose_backend_default_profile_and_budget_catalog() {
        let profile = default_operator_objectives();

        assert_eq!(profile.profile_id, "backend_default_v1");
        assert_eq!(profile.source, "backend_default_profile");
        assert_eq!(profile.window_hours, 24);
        assert_eq!(profile.compliance_semantics, "max_ratio_budget");
        assert_eq!(
            profile.non_human_posture,
            "treat_as_untrusted_until_identity_foundation"
        );
        assert_eq!(profile.budgets.len(), 3);
        assert_eq!(profile.budgets[0].budget_id, "likely_human_friction");
        assert_eq!(profile.budgets[1].metric, "suspicious_forwarded_request_rate");
        assert_eq!(profile.budgets[2].metric, "suspicious_forwarded_byte_rate");
    }

    #[test]
    fn default_operator_objectives_keep_unmaterialized_expectations_and_manual_guardrails_explicit() {
        let profile = default_operator_objectives();

        assert_eq!(
            profile.adversary_sim_expectations.availability,
            "not_yet_materialized"
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
}
