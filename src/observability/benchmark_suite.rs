use serde::{Deserialize, Serialize};

pub(crate) const BENCHMARK_SUITE_SCHEMA_VERSION: &str = "benchmark_suite_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkMetricContract {
    pub metric_id: String,
    pub eligible_population: String,
    pub target_kind: String,
    pub capability_gate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkFamilyContract {
    pub id: String,
    pub decision_question: String,
    pub eligible_population: String,
    pub comparison_modes: Vec<String>,
    pub subject_kinds: Vec<String>,
    pub capability_gate: String,
    pub metrics: Vec<BenchmarkMetricContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkDecisionBoundary {
    pub decision: String,
    pub summary: String,
    pub review_posture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkSuiteContract {
    pub schema_version: String,
    pub input_contract: String,
    pub comparison_modes: Vec<String>,
    pub subject_kinds: Vec<String>,
    pub families: Vec<BenchmarkFamilyContract>,
    pub decision_boundaries: Vec<BenchmarkDecisionBoundary>,
}

fn family(
    id: &str,
    decision_question: &str,
    eligible_population: &str,
    capability_gate: &str,
    metrics: &[(&str, &str, &str, &str)],
) -> BenchmarkFamilyContract {
    BenchmarkFamilyContract {
        id: id.to_string(),
        decision_question: decision_question.to_string(),
        eligible_population: eligible_population.to_string(),
        comparison_modes: vec![
            "prior_window".to_string(),
            "baseline".to_string(),
            "candidate".to_string(),
        ],
        subject_kinds: vec![
            "current_instance".to_string(),
            "prior_baseline".to_string(),
            "candidate_config".to_string(),
            "candidate_code".to_string(),
        ],
        capability_gate: capability_gate.to_string(),
        metrics: metrics
            .iter()
            .map(
                |(metric_id, eligible_population, target_kind, capability_gate)| {
                    BenchmarkMetricContract {
                        metric_id: (*metric_id).to_string(),
                        eligible_population: (*eligible_population).to_string(),
                        target_kind: (*target_kind).to_string(),
                        capability_gate: (*capability_gate).to_string(),
                    }
                },
            )
            .collect(),
    }
}

pub(crate) fn benchmark_suite_v1() -> BenchmarkSuiteContract {
    BenchmarkSuiteContract {
        schema_version: BENCHMARK_SUITE_SCHEMA_VERSION.to_string(),
        input_contract: "operator_snapshot_v1".to_string(),
        comparison_modes: vec![
            "prior_window".to_string(),
            "baseline".to_string(),
            "candidate".to_string(),
        ],
        subject_kinds: vec![
            "current_instance".to_string(),
            "prior_baseline".to_string(),
            "candidate_config".to_string(),
            "candidate_code".to_string(),
        ],
        families: vec![
            family(
                "suspicious_origin_cost",
                "How much suspicious traffic is still consuming defended-site cost, and how much cost is being shifted back onto Shuma?",
                "live:ingress_primary:enforced:suspicious_automation",
                "supported",
                &[
                    (
                        "suspicious_forwarded_request_rate",
                        "live:ingress_primary:enforced:suspicious_automation",
                        "max_ratio_budget",
                        "supported",
                    ),
                    (
                        "suspicious_forwarded_byte_rate",
                        "live:ingress_primary:enforced:suspicious_automation",
                        "max_ratio_budget",
                        "supported",
                    ),
                    (
                        "suspicious_short_circuit_rate",
                        "live:ingress_primary:enforced:suspicious_automation",
                        "maximize_ratio",
                        "supported",
                    ),
                    (
                        "suspicious_locally_served_byte_share",
                        "live:ingress_primary:enforced:suspicious_automation",
                        "maximize_ratio",
                        "supported",
                    ),
                ],
            ),
            family(
                "likely_human_friction",
                "How much friction or denial is Shuma imposing on likely-human or interactive traffic, and is that within budget?",
                "live:ingress_primary:enforced:likely_human",
                "partially_supported",
                &[
                    (
                        "likely_human_friction_rate",
                        "live:ingress_primary:enforced:likely_human",
                        "max_ratio_budget",
                        "supported",
                    ),
                    (
                        "interactive_friction_rate",
                        "live:ingress_primary:enforced:unknown_interactive",
                        "max_ratio_budget",
                        "not_yet_supported",
                    ),
                    (
                        "likely_human_hard_block_rate",
                        "live:ingress_primary:enforced:likely_human",
                        "max_ratio_budget",
                        "not_yet_supported",
                    ),
                ],
            ),
            family(
                "representative_adversary_effectiveness",
                "Against representative hostile scenarios, how effective is the current Shuma posture?",
                "adversary_sim:scenario_family",
                "partially_supported",
                &[
                    (
                        "scenario_goal_success_rate",
                        "adversary_sim:scenario_family",
                        "max_ratio_budget",
                        "supported",
                    ),
                    (
                        "scenario_origin_reach_rate",
                        "adversary_sim:scenario_family",
                        "max_ratio_budget",
                        "supported",
                    ),
                    (
                        "scenario_escalation_rate",
                        "adversary_sim:scenario_family",
                        "min_ratio_budget",
                        "supported",
                    ),
                    (
                        "scenario_regression_status",
                        "adversary_sim:scenario_family",
                        "regression_flag",
                        "supported",
                    ),
                ],
            ),
            family(
                "beneficial_non_human_posture",
                "Is Shuma treating beneficial or authenticated non-human traffic in line with the site's declared stance?",
                "verified_or_declared_non_human",
                "partially_supported",
                &[
                    (
                        "allowed_as_intended_rate",
                        "verified_or_declared_non_human",
                        "stance_aware_ratio",
                        "supported",
                    ),
                    (
                        "friction_mismatch_rate",
                        "verified_or_declared_non_human",
                        "stance_aware_ratio",
                        "supported",
                    ),
                    (
                        "deny_mismatch_rate",
                        "verified_or_declared_non_human",
                        "stance_aware_ratio",
                        "supported",
                    ),
                    (
                        "coverage_status",
                        "verified_or_declared_non_human",
                        "capability_gate",
                        "supported",
                    ),
                ],
            ),
        ],
        decision_boundaries: vec![
            BenchmarkDecisionBoundary {
                decision: "config_tuning_candidate".to_string(),
                summary: "Benchmark misses appear addressable through the existing allowed action surface.".to_string(),
                review_posture: "controller_can_propose".to_string(),
            },
            BenchmarkDecisionBoundary {
                decision: "observe_longer".to_string(),
                summary: "Current evidence is incomplete, too recent, or too noisy to justify a config or code change.".to_string(),
                review_posture: "watch_window_required".to_string(),
            },
            BenchmarkDecisionBoundary {
                decision: "code_evolution_candidate".to_string(),
                summary: "Repeated misses suggest the current code or action surface cannot improve the tradeoff frontier sufficiently.".to_string(),
                review_posture: "review_required".to_string(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::benchmark_suite_v1;

    #[test]
    fn benchmark_suite_v1_exposes_small_machine_first_family_registry() {
        let suite = benchmark_suite_v1();
        assert_eq!(suite.schema_version, "benchmark_suite_v1");
        assert_eq!(suite.input_contract, "operator_snapshot_v1");
        assert_eq!(suite.comparison_modes.len(), 3);
        assert_eq!(suite.subject_kinds.len(), 4);
        assert_eq!(suite.families.len(), 4);
        assert!(suite
            .families
            .iter()
            .any(|family| family.id == "suspicious_origin_cost"));
        assert!(suite
            .decision_boundaries
            .iter()
            .any(|row| row.decision == "code_evolution_candidate"));
    }
}
