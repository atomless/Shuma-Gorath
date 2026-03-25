use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::observability::operator_snapshot::{
    OperatorBudgetDistanceRow, OperatorSnapshotHotReadPayload, OperatorSnapshotRecentSimRun,
};

use super::oversight_patch_policy::{
    propose_patch, OversightPatchPolicyError, OversightPatchProposal, OversightProblemClass,
};

pub(crate) const OVERSIGHT_RECONCILE_SCHEMA_VERSION: &str = "oversight_reconcile_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightEvidenceReference {
    pub kind: String,
    pub reference: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightReconcileResult {
    pub schema_version: String,
    pub generated_at: u64,
    pub trigger_source: String,
    pub outcome: String,
    pub summary: String,
    pub objective_revision: String,
    pub benchmark_overall_status: String,
    pub improvement_status: String,
    pub problem_class: String,
    pub guidance_status: String,
    pub tractability: String,
    pub trigger_family_ids: Vec<String>,
    pub candidate_action_families: Vec<String>,
    pub refusal_reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal: Option<OversightPatchProposal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_sim_run_id: Option<String>,
    pub replay_promotion_availability: String,
    pub snapshot_generated_at: u64,
    pub evidence_references: Vec<OversightEvidenceReference>,
}

pub(crate) fn reconcile(
    cfg: &Config,
    snapshot: &OperatorSnapshotHotReadPayload,
    trigger_source: &str,
) -> OversightReconcileResult {
    let stale_reasons = stale_evidence_reasons(snapshot);
    if !stale_reasons.is_empty() {
        return result_without_proposal(
            snapshot,
            trigger_source,
            "refuse_stale_evidence",
            "Oversight refused to propose change because at least one required input section is stale.",
            stale_reasons,
        );
    }

    let contradictions = contradictory_evidence_reasons(snapshot);
    if !contradictions.is_empty() {
        return result_without_proposal(
            snapshot,
            trigger_source,
            "refuse_contradictory_evidence",
            "Oversight refused to propose change because the bounded evidence surfaces disagree about the current subject.",
            contradictions,
        );
    }

    let benchmark = &snapshot.benchmark_results;
    if benchmark.overall_status == "inside_budget" {
        return result_without_proposal(
            snapshot,
            trigger_source,
            "within_budget",
            "Current benchmark summary is inside budget; no config recommendation is justified.",
            Vec::new(),
        );
    }
    if benchmark.escalation_hint.decision == "observe_longer"
        || benchmark.overall_status == "near_limit"
    {
        return result_without_proposal(
            snapshot,
            trigger_source,
            "observe_longer",
            "Current evidence does not yet justify a bounded config recommendation; continue observing the next window.",
            benchmark.escalation_hint.blockers.clone(),
        );
    }
    if benchmark.escalation_hint.decision != "config_tuning_candidate" {
        let mut reasons = benchmark.escalation_hint.blockers.clone();
        if reasons.is_empty() {
            reasons.push("config_surface_not_authoritative".to_string());
        }
        return result_without_proposal(
            snapshot,
            trigger_source,
            "no_change",
            "Current outside-budget evidence does not map cleanly to a bounded config recommendation.",
            reasons,
        );
    }

    let problem_class = primary_problem_class(snapshot)
        .unwrap_or(OversightProblemClass::SuspiciousOriginReachOverspend);
    let proposal = match propose_patch(
        cfg,
        &snapshot.allowed_actions,
        benchmark.escalation_hint.candidate_action_families.as_slice(),
        problem_class,
        &snapshot.replay_promotion,
    ) {
        Ok(proposal) => proposal,
        Err(OversightPatchPolicyError::NoCandidateFamily) => {
            return result_without_proposal(
                snapshot,
                trigger_source,
                "no_change",
                "No bounded config family candidates are currently available for the observed benchmark pressure.",
                vec!["no_candidate_family".to_string()],
            );
        }
        Err(OversightPatchPolicyError::UnsupportedCandidateFamily(family)) => {
            return result_without_proposal(
                snapshot,
                trigger_source,
                "no_change",
                "The benchmark hint referenced a family that is not currently proposal-safe.",
                vec![format!("unsupported_candidate_family:{family}")],
            );
        }
        Err(OversightPatchPolicyError::NoBoundedPatch(families)) => {
            return result_without_proposal(
                snapshot,
                trigger_source,
                "no_change",
                "The selected candidate families did not yield a smaller bounded patch from the current config state.",
                vec![format!("no_bounded_patch:{families}")],
            );
        }
        Err(OversightPatchPolicyError::InvalidPatch(reason)) => {
            return result_without_proposal(
                snapshot,
                trigger_source,
                "refuse_contradictory_evidence",
                "Patch shaping failed because the bounded controller action surface and the proposed patch disagreed.",
                vec![format!("invalid_patch:{reason}")],
            );
        }
    };

    OversightReconcileResult {
        schema_version: OVERSIGHT_RECONCILE_SCHEMA_VERSION.to_string(),
        generated_at: snapshot.generated_at,
        trigger_source: trigger_source.to_string(),
        outcome: "recommend_patch".to_string(),
        summary: "Current benchmark pressure maps to a bounded config recommendation that still requires manual review and verification.".to_string(),
        objective_revision: snapshot.objectives.revision.clone(),
        benchmark_overall_status: benchmark.overall_status.clone(),
        improvement_status: benchmark.improvement_status.clone(),
        problem_class: problem_class.as_str().to_string(),
        guidance_status: "exact_bounded_move".to_string(),
        tractability: "exact_bounded_config_move".to_string(),
        trigger_family_ids: benchmark.escalation_hint.trigger_family_ids.clone(),
        candidate_action_families: benchmark.escalation_hint.candidate_action_families.clone(),
        refusal_reasons: Vec::new(),
        proposal: Some(proposal),
        latest_sim_run_id: latest_recent_sim_run_id(snapshot),
        replay_promotion_availability: snapshot.replay_promotion.availability.clone(),
        snapshot_generated_at: snapshot.generated_at,
        evidence_references: evidence_references(snapshot),
    }
}

fn result_without_proposal(
    snapshot: &OperatorSnapshotHotReadPayload,
    trigger_source: &str,
    outcome: &str,
    summary: &str,
    refusal_reasons: Vec<String>,
) -> OversightReconcileResult {
    OversightReconcileResult {
        schema_version: OVERSIGHT_RECONCILE_SCHEMA_VERSION.to_string(),
        generated_at: snapshot.generated_at,
        trigger_source: trigger_source.to_string(),
        outcome: outcome.to_string(),
        summary: summary.to_string(),
        objective_revision: snapshot.objectives.revision.clone(),
        benchmark_overall_status: snapshot.benchmark_results.overall_status.clone(),
        improvement_status: snapshot.benchmark_results.improvement_status.clone(),
        problem_class: snapshot.benchmark_results.escalation_hint.problem_class.clone(),
        guidance_status: snapshot
            .benchmark_results
            .escalation_hint
            .guidance_status
            .clone(),
        tractability: snapshot.benchmark_results.escalation_hint.tractability.clone(),
        trigger_family_ids: snapshot
            .benchmark_results
            .escalation_hint
            .trigger_family_ids
            .clone(),
        candidate_action_families: snapshot
            .benchmark_results
            .escalation_hint
            .candidate_action_families
            .clone(),
        refusal_reasons,
        proposal: None,
        latest_sim_run_id: latest_recent_sim_run_id(snapshot),
        replay_promotion_availability: snapshot.replay_promotion.availability.clone(),
        snapshot_generated_at: snapshot.generated_at,
        evidence_references: evidence_references(snapshot),
    }
}

pub(crate) fn stale_evidence_reasons(snapshot: &OperatorSnapshotHotReadPayload) -> Vec<String> {
    let max_age_seconds = snapshot.window.duration_seconds.max(1);
    ["live_traffic", "adversary_sim", "benchmark_results", "replay_promotion"]
        .iter()
        .filter_map(|key| {
            let metadata = snapshot.section_metadata.get(*key)?;
            let age_seconds = snapshot.generated_at.saturating_sub(metadata.refreshed_at_ts);
            if metadata.refreshed_at_ts == 0 || age_seconds > max_age_seconds {
                Some(format!("{key}_stale"))
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn contradictory_evidence_reasons(
    snapshot: &OperatorSnapshotHotReadPayload,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if snapshot.benchmark_results.input_snapshot_generated_at != snapshot.generated_at {
        reasons.push("benchmark_input_snapshot_mismatch".to_string());
    }
    if snapshot.benchmark_results.watch_window != snapshot.window {
        reasons.push("benchmark_watch_window_mismatch".to_string());
    }
    if snapshot.benchmark_results.generated_at < snapshot.generated_at {
        reasons.push("benchmark_generated_before_snapshot".to_string());
    }
    reasons
}

fn primary_problem_class(
    snapshot: &OperatorSnapshotHotReadPayload,
) -> Option<OversightProblemClass> {
    let likely_human_outside_budget = budget_row_status(
        snapshot.budget_distance.rows.as_slice(),
        "likely_human_friction_rate",
    ) == Some("outside_budget");
    let suspicious_reach_outside_budget = [
        "suspicious_forwarded_request_rate",
        "suspicious_forwarded_byte_rate",
    ]
    .iter()
    .any(|metric| budget_row_status(snapshot.budget_distance.rows.as_slice(), metric) == Some("outside_budget"));
    let suspicious_latency_outside_budget = budget_row_status(
        snapshot.budget_distance.rows.as_slice(),
        "suspicious_forwarded_latency_share",
    ) == Some("outside_budget");

    if likely_human_outside_budget {
        Some(OversightProblemClass::LikelyHumanFrictionOverspend)
    } else if suspicious_latency_outside_budget {
        Some(OversightProblemClass::SuspiciousOriginLatencyOverspend)
    } else if suspicious_reach_outside_budget {
        Some(OversightProblemClass::SuspiciousOriginReachOverspend)
    } else if snapshot.benchmark_results.escalation_hint.problem_class
        == "likely_human_friction_overspend"
    {
        Some(OversightProblemClass::LikelyHumanFrictionOverspend)
    } else if snapshot.benchmark_results.escalation_hint.problem_class
        == "suspicious_forwarded_latency_overspend"
    {
        Some(OversightProblemClass::SuspiciousOriginLatencyOverspend)
    } else if snapshot.benchmark_results.escalation_hint.problem_class
        == "suspicious_forwarded_reach_overspend"
    {
        Some(OversightProblemClass::SuspiciousOriginReachOverspend)
    } else if snapshot
        .benchmark_results
        .escalation_hint
        .trigger_family_ids
        .iter()
        .any(|family| family == "likely_human_friction")
    {
        Some(OversightProblemClass::LikelyHumanFrictionOverspend)
    } else if snapshot
        .benchmark_results
        .escalation_hint
        .trigger_family_ids
        .iter()
        .any(|family| family == "suspicious_origin_cost")
    {
        Some(OversightProblemClass::SuspiciousOriginReachOverspend)
    } else {
        None
    }
}

fn budget_row_status<'a>(rows: &'a [OperatorBudgetDistanceRow], metric: &str) -> Option<&'a str> {
    rows.iter()
        .find(|row| row.metric == metric)
        .map(|row| row.status.as_str())
}

pub(crate) fn latest_recent_sim_run_id(
    snapshot: &OperatorSnapshotHotReadPayload,
) -> Option<String> {
    latest_recent_sim_run(snapshot).map(|run| run.run_id.clone())
}

pub(crate) fn latest_recent_sim_run(
    snapshot: &OperatorSnapshotHotReadPayload,
) -> Option<&OperatorSnapshotRecentSimRun> {
    snapshot
        .adversary_sim
        .recent_runs
        .iter()
        .max_by(|left, right| left.last_ts.cmp(&right.last_ts))
}

fn evidence_references(snapshot: &OperatorSnapshotHotReadPayload) -> Vec<OversightEvidenceReference> {
    let mut references = vec![
        OversightEvidenceReference {
            kind: "operator_snapshot".to_string(),
            reference: format!("generated_at:{}", snapshot.generated_at),
            note: "Machine-first snapshot subject used by this recommend-only reconcile cycle."
                .to_string(),
        },
        OversightEvidenceReference {
            kind: "benchmark_results".to_string(),
            reference: format!("generated_at:{}", snapshot.benchmark_results.generated_at),
            note: "Nested benchmark summary used to derive trigger families and escalation direction."
                .to_string(),
        },
    ];
    if let Some(run_id) = latest_recent_sim_run_id(snapshot) {
        references.push(OversightEvidenceReference {
            kind: "adversary_sim_recent_run".to_string(),
            reference: run_id,
            note: "Latest bounded adversary-sim run visible in operator snapshot.".to_string(),
        });
    }
    references
}

#[cfg(test)]
mod tests {
    use super::{latest_recent_sim_run_id, reconcile, OversightReconcileResult};
    use crate::config::{allowed_actions_v1, defaults};
    use crate::observability::benchmark_results::{
        BenchmarkBaselineReference, BenchmarkEscalationHint, BenchmarkFamilyResult,
        BenchmarkMetricResult, BenchmarkResultsPayload, BenchmarkTuningEligibility,
        BENCHMARK_RESULTS_SCHEMA_VERSION,
    };
    use crate::observability::benchmark_suite::BENCHMARK_SUITE_SCHEMA_VERSION;
    use crate::observability::hot_read_contract::{
        HotReadOwnershipTier, TelemetryBasis, TelemetryExactness,
    };
    use crate::observability::non_human_classification::{
        non_human_decision_chain, NonHumanClassificationReadiness,
    };
    use crate::observability::non_human_coverage::NonHumanCoverageSummary;
    use crate::observability::operator_snapshot::{
        OperatorBudgetDistanceRow, OperatorBudgetDistanceSummary, OperatorSnapshotHotReadPayload,
        OperatorSnapshotSectionMetadata, OperatorSnapshotWindow,
    };
    use crate::observability::operator_snapshot_live_traffic::{
        OperatorSnapshotAdversarySim, OperatorSnapshotLiveTraffic, OperatorSnapshotRecentSimRun,
        OperatorSnapshotShadowMode,
    };
    use crate::observability::operator_snapshot_objectives::default_operator_objectives;
    use crate::observability::operator_snapshot_non_human::OperatorSnapshotNonHumanTrafficSummary;
    use crate::observability::operator_snapshot_recent_changes::OperatorSnapshotRecentChanges;
    use crate::observability::operator_snapshot_runtime_posture::OperatorSnapshotRuntimePosture;
    use crate::observability::operator_snapshot_verified_identity::{
        OperatorSnapshotVerifiedIdentityPolicySummary, OperatorSnapshotVerifiedIdentitySummary,
    };
    use crate::observability::replay_promotion::ReplayPromotionSummary;
    use std::collections::BTreeMap;

    fn sample_snapshot() -> OperatorSnapshotHotReadPayload {
        let generated_at = 1_700_000_100;
        let window = OperatorSnapshotWindow {
            start_ts: generated_at - 86_399,
            end_ts: generated_at,
            duration_seconds: 86_400,
        };
        let objectives = default_operator_objectives(generated_at);
        let benchmark_results = BenchmarkResultsPayload {
            schema_version: BENCHMARK_RESULTS_SCHEMA_VERSION.to_string(),
            suite_version: BENCHMARK_SUITE_SCHEMA_VERSION.to_string(),
            generated_at,
            input_snapshot_generated_at: generated_at,
            subject_kind: "current_instance".to_string(),
            watch_window: window.clone(),
            baseline_reference: BenchmarkBaselineReference {
                reference_kind: "prior_window".to_string(),
                status: "available".to_string(),
                subject_kind: Some("prior_window".to_string()),
                generated_at: Some(generated_at - 86_400),
                note: "Prior window reference".to_string(),
            },
            coverage_status: "partial_support".to_string(),
            overall_status: "outside_budget".to_string(),
            improvement_status: "regressed".to_string(),
            non_human_classification: NonHumanClassificationReadiness {
                status: "ready".to_string(),
                blockers: Vec::new(),
                live_receipt_count: 1,
                adversary_sim_receipt_count: 1,
            },
            non_human_coverage: NonHumanCoverageSummary {
                schema_version: "non_human_coverage_v1".to_string(),
                overall_status: "covered".to_string(),
                blocking_reasons: Vec::new(),
                blocking_category_ids: Vec::new(),
                mapped_category_count: 6,
                gap_category_count: 2,
                covered_category_count: 6,
                partial_category_count: 0,
                stale_category_count: 0,
                unavailable_category_count: 0,
                uncovered_category_count: 2,
                receipts: Vec::new(),
            },
            tuning_eligibility: BenchmarkTuningEligibility {
                status: "eligible".to_string(),
                blockers: Vec::new(),
            },
            families: vec![BenchmarkFamilyResult {
                family_id: "suspicious_origin_cost".to_string(),
                status: "outside_budget".to_string(),
                capability_gate: "supported".to_string(),
                note: "Suspicious origin cost outside budget.".to_string(),
                baseline_status: Some("outside_budget".to_string()),
                comparison_status: "regressed".to_string(),
                metrics: vec![BenchmarkMetricResult {
                    metric_id: "suspicious_forwarded_request_rate".to_string(),
                    status: "outside_budget".to_string(),
                    current: Some(0.42),
                    target: Some(0.10),
                    delta: Some(0.32),
                    exactness: "exact".to_string(),
                    basis: "observed".to_string(),
                    capability_gate: "supported".to_string(),
                    baseline_current: Some(0.30),
                    comparison_delta: Some(0.12),
                    comparison_status: "regressed".to_string(),
                }],
            }],
            escalation_hint: BenchmarkEscalationHint {
                availability: "partial_support".to_string(),
                decision: "config_tuning_candidate".to_string(),
                review_status: "manual_review_required".to_string(),
                problem_class: "suspicious_forwarded_reach_overspend".to_string(),
                guidance_status: "bounded_family_guidance".to_string(),
                tractability: "family_level_policy_choice".to_string(),
                expected_direction: "tighten_suspicious_origin_controls".to_string(),
                trigger_family_ids: vec!["suspicious_origin_cost".to_string()],
                trigger_metric_ids: vec!["suspicious_forwarded_request_rate".to_string()],
                candidate_action_families: vec!["fingerprint_signal".to_string()],
                family_guidance: vec![],
                blockers: Vec::new(),
                note: "Config tuning candidate.".to_string(),
            },
            replay_promotion: ReplayPromotionSummary::not_materialized(),
        };
        let mut section_metadata = BTreeMap::new();
        for key in [
            "objectives",
            "live_traffic",
            "adversary_sim",
            "game_contract",
            "benchmark_results",
            "non_human_traffic",
            "replay_promotion",
        ] {
            section_metadata.insert(
                key.to_string(),
                OperatorSnapshotSectionMetadata {
                    exactness: TelemetryExactness::Exact,
                    basis: TelemetryBasis::Observed,
                    ownership_tier: HotReadOwnershipTier::BootstrapCritical,
                    refreshed_at_ts: generated_at,
                },
            );
        }
        OperatorSnapshotHotReadPayload {
            schema_version: "operator_snapshot_v1".to_string(),
            generated_at,
            window: window.clone(),
            section_metadata,
            objectives: objectives.clone(),
            live_traffic: OperatorSnapshotLiveTraffic {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                total_requests: 120,
                forwarded_requests: 60,
                short_circuited_requests: 60,
                control_response_requests: 0,
                forwarded_upstream_latency_ms_total: 0,
                forwarded_response_bytes: 12_000,
                shuma_served_response_bytes: 8_000,
                likely_human: None,
                suspicious_automation: None,
                human_friction: None,
            },
            shadow_mode: OperatorSnapshotShadowMode {
                enabled: false,
                total_actions: 0,
                pass_through_total: 0,
                actions: BTreeMap::new(),
            },
            adversary_sim: OperatorSnapshotAdversarySim {
                traffic_origin: "adversary_sim".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                total_requests: 40,
                forwarded_requests: 20,
                short_circuited_requests: 20,
                control_response_requests: 0,
                forwarded_upstream_latency_ms_total: 0,
                forwarded_response_bytes: 4_000,
                shuma_served_response_bytes: 3_000,
                recent_runs: vec![OperatorSnapshotRecentSimRun {
                    run_id: "simrun-001".to_string(),
                    lane: "deterministic_black_box".to_string(),
                    profile: "fast_smoke".to_string(),
                    observed_fulfillment_modes: Vec::new(),
                    observed_category_ids: Vec::new(),
                    first_ts: generated_at - 120,
                    last_ts: generated_at - 30,
                    monitoring_event_count: 8,
                    defense_delta_count: 3,
                    ban_outcome_count: 0,
                    owned_surface_coverage: None,
                }],
            },
            runtime_posture: OperatorSnapshotRuntimePosture {
                shadow_mode: false,
                fail_mode: "closed".to_string(),
                runtime_environment: "runtime_dev".to_string(),
                gateway_deployment_profile: "shared_server".to_string(),
                adversary_sim_available: true,
            },
            recent_changes: OperatorSnapshotRecentChanges::default(),
            budget_distance: OperatorBudgetDistanceSummary {
                rows: vec![OperatorBudgetDistanceRow {
                    budget_id: "suspicious_forwarded_requests".to_string(),
                    metric: "suspicious_forwarded_request_rate".to_string(),
                    eligible_requests: 40,
                    current: 0.42,
                    target: 0.10,
                    delta: 0.32,
                    near_limit: 0.075,
                    status: "outside_budget".to_string(),
                    exactness: "exact".to_string(),
                    basis: "observed".to_string(),
                }],
            },
            non_human_traffic: OperatorSnapshotNonHumanTrafficSummary {
                availability: "taxonomy_seeded".to_string(),
                taxonomy: crate::runtime::non_human_taxonomy::canonical_non_human_taxonomy(),
                readiness: NonHumanClassificationReadiness {
                    status: "ready".to_string(),
                    blockers: Vec::new(),
                    live_receipt_count: 1,
                    adversary_sim_receipt_count: 1,
                },
                coverage: NonHumanCoverageSummary {
                    schema_version: "non_human_coverage_v1".to_string(),
                    overall_status: "covered".to_string(),
                    blocking_reasons: Vec::new(),
                    blocking_category_ids: Vec::new(),
                    mapped_category_count: 6,
                    gap_category_count: 2,
                    covered_category_count: 6,
                    partial_category_count: 0,
                    stale_category_count: 0,
                    unavailable_category_count: 0,
                    uncovered_category_count: 2,
                    receipts: Vec::new(),
                },
                decision_chain: non_human_decision_chain(),
                receipts: Vec::new(),
            },
            allowed_actions: allowed_actions_v1(),
            game_contract:
                crate::observability::operator_snapshot_objectives::recursive_improvement_game_contract_v1(
                    &objectives,
                    &crate::config::controller_legal_move_ring_v1(),
                ),
            benchmark_results,
            verified_identity: OperatorSnapshotVerifiedIdentitySummary {
                availability: "not_configured".to_string(),
                enabled: false,
                native_web_bot_auth_enabled: false,
                provider_assertions_enabled: false,
                non_human_traffic_stance: "allow_only_named_verified_identities".to_string(),
                named_policy_count: 0,
                service_profile_count: 0,
                attempts: 0,
                verified: 0,
                failed: 0,
                unique_verified_identities: 0,
                top_failure_reasons: Vec::new(),
                top_schemes: Vec::new(),
                top_categories: Vec::new(),
                top_provenance: Vec::new(),
                taxonomy_alignment: crate::observability::non_human_classification::VerifiedIdentityTaxonomyAlignmentSummary::default(),
                policy_tranche: OperatorSnapshotVerifiedIdentityPolicySummary::default(),
            },
            replay_promotion: ReplayPromotionSummary::not_materialized(),
        }
    }

    fn reconcile_outcome(result: &OversightReconcileResult) -> &str {
        result.outcome.as_str()
    }

    #[test]
    fn recommend_patch_when_outside_budget_maps_to_bounded_candidate_family() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        let snapshot = sample_snapshot();

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "recommend_patch");
        assert_eq!(result.problem_class, "suspicious_forwarded_reach_overspend");
        assert_eq!(result.guidance_status, "exact_bounded_move");
        assert_eq!(result.tractability, "exact_bounded_config_move");
        assert_eq!(
            result
                .proposal
                .as_ref()
                .expect("proposal present")
                .patch_family,
            "fingerprint_signal"
        );
        assert_eq!(latest_recent_sim_run_id(&snapshot).as_deref(), Some("simrun-001"));
    }

    #[test]
    fn refuse_stale_evidence_when_required_snapshot_sections_age_out() {
        let cfg = defaults().clone();
        let mut snapshot = sample_snapshot();
        snapshot
            .section_metadata
            .get_mut("benchmark_results")
            .expect("benchmark metadata present")
            .refreshed_at_ts = snapshot.generated_at - snapshot.window.duration_seconds - 1;

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "refuse_stale_evidence");
        assert!(result
            .refusal_reasons
            .contains(&"benchmark_results_stale".to_string()));
    }

    #[test]
    fn refuse_contradictory_evidence_when_nested_benchmark_subject_mismatches_snapshot() {
        let cfg = defaults().clone();
        let mut snapshot = sample_snapshot();
        snapshot.benchmark_results.input_snapshot_generated_at = snapshot.generated_at - 60;

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "refuse_contradictory_evidence");
        assert!(result
            .refusal_reasons
            .contains(&"benchmark_input_snapshot_mismatch".to_string()));
    }

    #[test]
    fn returns_within_budget_when_benchmarks_no_longer_need_escalation() {
        let cfg = defaults().clone();
        let mut snapshot = sample_snapshot();
        snapshot.benchmark_results.overall_status = "inside_budget".to_string();
        snapshot.benchmark_results.escalation_hint.decision = "observe_longer".to_string();

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "within_budget");
        assert!(result.proposal.is_none());
    }

    #[test]
    fn primary_problem_class_treats_latency_share_budget_miss_as_latency_overspend() {
        let mut snapshot = sample_snapshot();
        snapshot.budget_distance.rows.push(OperatorBudgetDistanceRow {
            budget_id: "suspicious_forwarded_latency".to_string(),
            metric: "suspicious_forwarded_latency_share".to_string(),
            eligible_requests: 1,
            current: 0.6,
            target: 0.1,
            delta: 0.5,
            near_limit: 0.075,
            status: "outside_budget".to_string(),
            exactness: "derived".to_string(),
            basis: "observed".to_string(),
        });

        assert_eq!(
            super::primary_problem_class(&snapshot),
            Some(super::OversightProblemClass::SuspiciousOriginLatencyOverspend)
        );
    }

    #[test]
    fn observe_longer_when_verified_identity_guardrail_blocks_candidate() {
        let cfg = defaults().clone();
        let mut snapshot = sample_snapshot();
        snapshot.benchmark_results.overall_status = "outside_budget".to_string();
        snapshot.benchmark_results.escalation_hint.decision = "observe_longer".to_string();
        snapshot.benchmark_results.escalation_hint.blockers =
            vec!["verified_identity_botness_conflict_guardrail".to_string()];

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "observe_longer");
        assert!(result
            .refusal_reasons
            .contains(&"verified_identity_botness_conflict_guardrail".to_string()));
    }
}
