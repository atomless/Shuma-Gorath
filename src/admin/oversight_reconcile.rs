use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::observability::benchmark_results::{
    BenchmarkControllerBlocker, BenchmarkExploitLocus, BenchmarkRecognitionEvaluationStatus,
};
use crate::observability::operator_snapshot::{
    OperatorBudgetDistanceRow, OperatorSnapshotHotReadPayload, OperatorSnapshotRecentSimRun,
};

use super::oversight_patch_policy::{
    rank_patch_candidates, OversightPatchCandidate, OversightPatchPolicyError,
    OversightPatchProposal, OversightProblemClass,
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
    pub judge: OversightJudgeState,
    pub diagnosis: OversightDiagnosis,
    pub recognition_evaluation: BenchmarkRecognitionEvaluationStatus,
    pub move_selection: OversightMoveSelection,
    pub evidence_references: Vec<OversightEvidenceReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightJudgeState {
    pub overall_status: String,
    pub improvement_status: String,
    pub urgency_status: String,
    pub evidence_quality_status: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightDiagnosis {
    pub status: String,
    pub problem_class: String,
    pub confidence: String,
    pub distributed_failure_status: String,
    pub repair_surface_status: String,
    pub repair_surface_candidates: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breach_loci: Vec<BenchmarkExploitLocus>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightMoveSelection {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_family: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_breach_locus_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounded_repair_surface: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ranked_candidates: Vec<OversightPatchCandidate>,
    pub config_ring_status: String,
    pub code_evolution_status: String,
    pub note: String,
}

pub(crate) fn reconcile(
    cfg: &Config,
    snapshot: &OperatorSnapshotHotReadPayload,
    trigger_source: &str,
) -> OversightReconcileResult {
    let benchmark = &snapshot.benchmark_results;
    let controller_move_selection = &benchmark.controller_contract.move_selection;
    let judge = judge_state(snapshot);
    let stale_reasons = stale_evidence_reasons(snapshot);
    if !stale_reasons.is_empty() {
        return result_without_proposal(
            snapshot,
            trigger_source,
            "refuse_stale_evidence",
            "Oversight refused to propose change because at least one required input section is stale.",
            stale_reasons,
            controller_problem_class(snapshot).as_str(),
            "blocked_by_guardrail",
            Vec::new(),
            None,
            "not_evaluated",
            "not_required",
            judge,
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
            controller_problem_class(snapshot).as_str(),
            "blocked_by_guardrail",
            Vec::new(),
            None,
            "not_evaluated",
            "not_required",
            judge,
        );
    }

    if benchmark.overall_status == "inside_budget" {
        return result_without_proposal(
            snapshot,
            trigger_source,
            "within_budget",
            "Current benchmark summary is inside budget; no config recommendation is justified.",
            Vec::new(),
            controller_problem_class(snapshot).as_str(),
            "not_required",
            Vec::new(),
            None,
            "not_applicable",
            "not_required",
            judge,
        );
    }
    if controller_move_selection.decision == "observe_longer"
        || benchmark.overall_status == "near_limit"
    {
        return result_without_proposal(
            snapshot,
            trigger_source,
            "observe_longer",
            "Current evidence does not yet justify a bounded config recommendation; continue observing the next window.",
            controller_blocker_ids(controller_move_selection.blockers.as_slice()),
            controller_problem_class(snapshot).as_str(),
            "observe_longer",
            Vec::new(),
            None,
            "not_evaluated",
            "not_required",
            judge,
        );
    }
    if controller_move_selection.decision == "code_evolution_candidate" {
        return result_without_proposal(
            snapshot,
            trigger_source,
            "code_evolution_referral",
            "Current outside-budget evidence points to a code-evolution gap rather than a bounded config repair.",
            controller_blocker_ids(controller_move_selection.blockers.as_slice()),
            controller_problem_class(snapshot).as_str(),
            "code_evolution_referral",
            Vec::new(),
            None,
            "not_applicable",
            "required",
            judge,
        );
    }
    if controller_move_selection.decision != "config_tuning_candidate" {
        let mut reasons = controller_blocker_ids(controller_move_selection.blockers.as_slice());
        if reasons.is_empty() {
            reasons.push("config_surface_not_authoritative".to_string());
        }
        return result_without_proposal(
            snapshot,
            trigger_source,
            "no_change",
            "Current outside-budget evidence does not map cleanly to a bounded config recommendation.",
            reasons,
            controller_problem_class(snapshot).as_str(),
            "not_selected",
            Vec::new(),
            None,
            "not_evaluated",
            "not_required",
            judge,
        );
    }

    let problem_class = primary_problem_class(snapshot)
        .unwrap_or(OversightProblemClass::SuspiciousOriginReachOverspend);
    let ranked_candidates = match rank_patch_candidates(
        cfg,
        &snapshot.allowed_actions,
        controller_move_selection.candidate_action_families.as_slice(),
        problem_class,
        &snapshot.replay_promotion,
    ) {
        Ok(candidates) => candidates,
        Err(OversightPatchPolicyError::NoCandidateFamily) => {
            return result_without_proposal(
                snapshot,
                trigger_source,
                "no_change",
                "No bounded config family candidates are currently available for the observed benchmark pressure.",
                vec!["no_candidate_family".to_string()],
                problem_class.as_str(),
                "not_selected",
                Vec::new(),
                None,
                "not_evaluated",
                "not_required",
                judge,
            );
        }
        Err(OversightPatchPolicyError::UnsupportedCandidateFamily(family)) => {
            return result_without_proposal(
                snapshot,
                trigger_source,
                "no_change",
                "The benchmark hint referenced a family that is not currently proposal-safe.",
                vec![format!("unsupported_candidate_family:{family}")],
                problem_class.as_str(),
                "not_selected",
                Vec::new(),
                None,
                "not_evaluated",
                "not_required",
                judge,
            );
        }
        Err(OversightPatchPolicyError::NoBoundedPatch(families)) => {
            return result_without_proposal(
                snapshot,
                trigger_source,
                "code_evolution_referral",
                "The selected candidate families did not yield a smaller bounded patch from the current config state, so this gap must escalate beyond bounded config tuning.",
                vec![format!("no_bounded_patch:{families}")],
                problem_class.as_str(),
                "code_evolution_referral",
                Vec::new(),
                None,
                "not_applicable",
                "required",
                judge,
            );
        }
        Err(OversightPatchPolicyError::InvalidPatch(reason)) => {
            return result_without_proposal(
                snapshot,
                trigger_source,
                "refuse_contradictory_evidence",
                "Patch shaping failed because the bounded controller action surface and the proposed patch disagreed.",
                vec![format!("invalid_patch:{reason}")],
                problem_class.as_str(),
                "blocked_by_guardrail",
                Vec::new(),
                None,
                "not_evaluated",
                "not_required",
                judge,
            );
        }
    };
    if let Some(exhausted_family) = config_ring_exhausted_family(snapshot, ranked_candidates.as_slice())
    {
        return result_without_proposal(
            snapshot,
            trigger_source,
            "config_ring_exhausted",
            "Recent bounded move history shows the current config ring has already failed repeatedly at this repair surface; escalate to code review instead of repeating near-equivalent config moves.",
            vec![format!("config_ring_exhausted:{exhausted_family}")],
            problem_class.as_str(),
            "config_ring_exhausted",
            ranked_candidates.clone(),
            None,
            "exhausted",
            "review_required",
            judge,
        );
    }
    let selected = ranked_candidates
        .first()
        .expect("ranked candidates must not be empty")
        .clone();
    let selected_family = selected.family.clone();
    let proposal = selected.proposal.clone();
    let diagnosis = diagnosis(snapshot, problem_class.as_str());

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
        guidance_status: controller_move_selection.guidance_status.clone(),
        tractability: controller_move_selection.tractability.clone(),
        trigger_family_ids: controller_move_selection.trigger_family_ids.clone(),
        candidate_action_families: controller_move_selection.candidate_action_families.clone(),
        refusal_reasons: Vec::new(),
        proposal: Some(proposal),
        latest_sim_run_id: latest_recent_sim_run_id(snapshot),
        replay_promotion_availability: snapshot.replay_promotion.availability.clone(),
        snapshot_generated_at: snapshot.generated_at,
        judge,
        diagnosis: diagnosis.clone(),
        recognition_evaluation: snapshot
            .benchmark_results
            .controller_contract
            .recognition_evaluation
            .clone(),
        move_selection: move_selection(
            snapshot,
            "selected",
            ranked_candidates,
            Some(selected_family.clone()),
            Some(selected_family),
            "bounded_ring_available",
            "not_required",
        ),
        evidence_references: evidence_references(snapshot),
    }
}

fn result_without_proposal(
    snapshot: &OperatorSnapshotHotReadPayload,
    trigger_source: &str,
    outcome: &str,
    summary: &str,
    refusal_reasons: Vec<String>,
    problem_class: &str,
    move_selection_status: &str,
    ranked_candidates: Vec<OversightPatchCandidate>,
    selected_family: Option<String>,
    config_ring_status: &str,
    code_evolution_status: &str,
    judge: OversightJudgeState,
) -> OversightReconcileResult {
    let diagnosis = diagnosis(snapshot, problem_class);
    OversightReconcileResult {
        schema_version: OVERSIGHT_RECONCILE_SCHEMA_VERSION.to_string(),
        generated_at: snapshot.generated_at,
        trigger_source: trigger_source.to_string(),
        outcome: outcome.to_string(),
        summary: summary.to_string(),
        objective_revision: snapshot.objectives.revision.clone(),
        benchmark_overall_status: snapshot.benchmark_results.overall_status.clone(),
        improvement_status: snapshot.benchmark_results.improvement_status.clone(),
        problem_class: controller_problem_class(snapshot),
        guidance_status: snapshot
            .benchmark_results
            .controller_contract
            .move_selection
            .guidance_status
            .clone(),
        tractability: snapshot
            .benchmark_results
            .controller_contract
            .move_selection
            .tractability
            .clone(),
        trigger_family_ids: snapshot
            .benchmark_results
            .controller_contract
            .move_selection
            .trigger_family_ids
            .clone(),
        candidate_action_families: snapshot
            .benchmark_results
            .controller_contract
            .move_selection
            .candidate_action_families
            .clone(),
        refusal_reasons,
        proposal: None,
        latest_sim_run_id: latest_recent_sim_run_id(snapshot),
        replay_promotion_availability: snapshot.replay_promotion.availability.clone(),
        snapshot_generated_at: snapshot.generated_at,
        judge,
        diagnosis,
        recognition_evaluation: snapshot
            .benchmark_results
            .controller_contract
            .recognition_evaluation
            .clone(),
        move_selection: move_selection(
            snapshot,
            move_selection_status,
            ranked_candidates,
            selected_family.clone(),
            selected_family,
            config_ring_status,
            code_evolution_status,
        ),
        evidence_references: evidence_references(snapshot),
    }
}

fn judge_state(snapshot: &OperatorSnapshotHotReadPayload) -> OversightJudgeState {
    OversightJudgeState {
        overall_status: snapshot.benchmark_results.overall_status.clone(),
        improvement_status: snapshot.benchmark_results.improvement_status.clone(),
        urgency_status: snapshot.benchmark_results.urgency.status.clone(),
        evidence_quality_status: snapshot
            .benchmark_results
            .escalation_hint
            .evidence_quality
            .status
            .clone(),
        note: "Judge state is copied directly from benchmark results so diagnosis and move selection cannot silently rewrite scored truth."
            .to_string(),
    }
}

fn diagnosis(snapshot: &OperatorSnapshotHotReadPayload, problem_class: &str) -> OversightDiagnosis {
    let controller_diagnosis = &snapshot.benchmark_results.controller_contract.restriction_diagnosis;
    let breach_loci = controller_diagnosis.breach_loci.clone();
    let repair_surface_candidates = controller_diagnosis.repair_surface_candidates.clone();
    let distributed_failure_status = if breach_loci.is_empty() {
        if controller_diagnosis.status == "blocked_by_missing_truth" {
            "blocked_by_missing_truth"
        } else {
            "not_localized"
        }
    } else if breach_loci.len() == 1 {
        "single_locus"
    } else {
        "distributed_failure_evidence"
    };
    let repair_surface_status = match repair_surface_candidates.len() {
        0 => "not_available",
        1 => "single_family",
        _ => "multiple_candidate_families",
    };

    OversightDiagnosis {
        status: controller_diagnosis.status.clone(),
        problem_class: problem_class.to_string(),
        confidence: controller_diagnosis.confidence.clone(),
        distributed_failure_status: distributed_failure_status.to_string(),
        repair_surface_status: repair_surface_status.to_string(),
        repair_surface_candidates,
        breach_loci: breach_loci.clone(),
        note: controller_diagnosis.note.clone(),
    }
}

fn move_selection(
    snapshot: &OperatorSnapshotHotReadPayload,
    status: &str,
    ranked_candidates: Vec<OversightPatchCandidate>,
    selected_family: Option<String>,
    bounded_repair_surface: Option<String>,
    config_ring_status: &str,
    code_evolution_status: &str,
) -> OversightMoveSelection {
    OversightMoveSelection {
        status: status.to_string(),
        selected_breach_locus_ids: snapshot
            .benchmark_results
            .controller_contract
            .restriction_diagnosis
            .breach_loci
            .iter()
            .map(|locus| locus.locus_id.clone())
            .collect(),
        selected_family,
        bounded_repair_surface,
        ranked_candidates,
        config_ring_status: config_ring_status.to_string(),
        code_evolution_status: code_evolution_status.to_string(),
        note: "Move selection is kept explicit so the repo can distinguish selected bounded config moves from code referrals and exhausted rings."
            .to_string(),
    }
}

fn config_ring_exhausted_family(
    snapshot: &OperatorSnapshotHotReadPayload,
    ranked_candidates: &[OversightPatchCandidate],
) -> Option<String> {
    let primary = ranked_candidates.first()?;
    let repeated_failures = snapshot
        .episode_archive
        .rows
        .iter()
        .filter(|row| row.homeostasis_eligible && row.retain_or_rollback == "rolled_back")
        .filter(|row| {
            row.proposal
                .as_ref()
                .map(|proposal| proposal.patch_family.as_str() == primary.family.as_str())
                .unwrap_or(false)
        })
        .take(2)
        .count();
    (repeated_failures >= 2).then(|| primary.family.clone())
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
    } else if controller_problem_class(snapshot)
        == "scrapling_exploit_progress_gap"
    {
        Some(OversightProblemClass::ScraplingExploitProgressGap)
    } else if controller_problem_class(snapshot)
        == "likely_human_friction_overspend"
    {
        Some(OversightProblemClass::LikelyHumanFrictionOverspend)
    } else if controller_problem_class(snapshot)
        == "suspicious_forwarded_latency_overspend"
    {
        Some(OversightProblemClass::SuspiciousOriginLatencyOverspend)
    } else if controller_problem_class(snapshot)
        == "suspicious_forwarded_reach_overspend"
    {
        Some(OversightProblemClass::SuspiciousOriginReachOverspend)
    } else if snapshot
        .benchmark_results
        .controller_contract
        .move_selection
        .trigger_family_ids
        .iter()
        .any(|family| family == "scrapling_exploit_progress")
    {
        Some(OversightProblemClass::ScraplingExploitProgressGap)
    } else if snapshot
        .benchmark_results
        .controller_contract
        .move_selection
        .trigger_family_ids
        .iter()
        .any(|family| family == "likely_human_friction")
    {
        Some(OversightProblemClass::LikelyHumanFrictionOverspend)
    } else if snapshot
        .benchmark_results
        .controller_contract
        .move_selection
        .trigger_family_ids
        .iter()
        .any(|family| family == "suspicious_origin_cost")
    {
        Some(OversightProblemClass::SuspiciousOriginReachOverspend)
    } else {
        None
    }
}

fn controller_problem_class(snapshot: &OperatorSnapshotHotReadPayload) -> String {
    snapshot
        .benchmark_results
        .controller_contract
        .restriction_diagnosis
        .problem_class
        .clone()
}

fn controller_blocker_ids(blockers: &[BenchmarkControllerBlocker]) -> Vec<String> {
    blockers
        .iter()
        .map(|blocker| blocker.blocker_id.clone())
        .collect()
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
        unavailable_benchmark_diagnosis_evidence_quality, unavailable_benchmark_urgency_summary,
        BenchmarkBaselineReference, BenchmarkControllerBlocker, BenchmarkControllerContract,
        BenchmarkEscalationHint, BenchmarkExploitLocus, BenchmarkFamilyResult,
        BenchmarkMetricResult, BenchmarkMoveSelectionGuidance,
        BenchmarkRecognitionEvaluationStatus, BenchmarkRestrictionDiagnosis, BenchmarkResultsPayload,
        BenchmarkTuningEligibility,
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
        BenchmarkComparableSnapshot, BenchmarkHomeostasisRestartBaseline,
        OperatorBudgetDistanceRow, OperatorBudgetDistanceSummary, OperatorSnapshotEpisodeArchive,
        OperatorSnapshotEpisodeEvaluationContext, OperatorSnapshotEpisodeProposal,
        OperatorSnapshotEpisodeRecord, OperatorSnapshotHotReadPayload,
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
                exploit_loci: Vec::new(),
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
                evidence_quality: unavailable_benchmark_diagnosis_evidence_quality(),
                breach_loci: Vec::new(),
                note: "Config tuning candidate.".to_string(),
            },
            controller_contract: BenchmarkControllerContract {
                restriction_diagnosis: BenchmarkRestrictionDiagnosis {
                    problem_class: "suspicious_forwarded_reach_overspend".to_string(),
                    status: "aggregate_only".to_string(),
                    confidence: "medium".to_string(),
                    repair_surface_candidates: vec!["fingerprint_signal".to_string()],
                    breach_loci: Vec::new(),
                    blockers: Vec::new(),
                    note: "Restriction diagnosis is still aggregate and does not yet localize a repair locus."
                        .to_string(),
                },
                recognition_evaluation: BenchmarkRecognitionEvaluationStatus {
                    status: "steady".to_string(),
                    trigger_family_ids: vec!["non_human_category_posture".to_string()],
                    blockers: Vec::new(),
                    note: "Recognition evaluation remains explicit and separate from restriction scoring."
                        .to_string(),
                },
                move_selection: BenchmarkMoveSelectionGuidance {
                    decision: "config_tuning_candidate".to_string(),
                    review_status: "manual_review_required".to_string(),
                    guidance_status: "bounded_family_guidance".to_string(),
                    tractability: "family_level_policy_choice".to_string(),
                    expected_direction: "tighten_suspicious_origin_controls".to_string(),
                    trigger_family_ids: vec!["suspicious_origin_cost".to_string()],
                    candidate_action_families: vec!["fingerprint_signal".to_string()],
                    family_guidance: Vec::new(),
                    blockers: Vec::new(),
                    note: "Config tuning candidate.".to_string(),
                },
            },
            urgency: unavailable_benchmark_urgency_summary(),
            replay_promotion: ReplayPromotionSummary::not_materialized(),
        };
        let mut section_metadata = BTreeMap::new();
        for key in [
            "objectives",
            "live_traffic",
            "adversary_sim",
            "game_contract",
            "episode_archive",
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
                restriction_readiness: NonHumanClassificationReadiness {
                    status: "ready".to_string(),
                    blockers: Vec::new(),
                    live_receipt_count: 1,
                    adversary_sim_receipt_count: 1,
                },
                decision_chain: non_human_decision_chain(),
                restriction_receipts: Vec::new(),
                recognition_evaluation:
                    crate::observability::operator_snapshot_non_human::OperatorSnapshotNonHumanRecognitionEvaluationSummary {
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
                        simulator_ground_truth:
                            crate::observability::non_human_classification::NonHumanSimulatorGroundTruthSummary::default(),
                        comparison_status: "not_observed".to_string(),
                        current_exact_match_count: 0,
                        degraded_match_count: 0,
                        collapsed_to_unknown_count: 0,
                        not_materialized_count: 0,
                        comparison_rows: Vec::new(),
                        receipts: Vec::new(),
                    },
            },
            allowed_actions: allowed_actions_v1(),
            game_contract:
                crate::observability::operator_snapshot_objectives::recursive_improvement_game_contract_v1(
                    &objectives,
                    &crate::config::controller_legal_move_ring_v1(),
                ),
            episode_archive: crate::observability::operator_snapshot::OperatorSnapshotEpisodeArchive {
                schema_version: "oversight_episode_archive_v1".to_string(),
                homeostasis: crate::observability::benchmark_comparison::classify_homeostasis(
                    &[],
                    10,
                ),
                rows: Vec::new(),
            },
            benchmark_results,
            verified_identity: OperatorSnapshotVerifiedIdentitySummary {
                availability: "not_configured".to_string(),
                enabled: false,
                native_web_bot_auth_enabled: false,
                provider_assertions_enabled: false,
                effective_non_human_policy:
                    crate::runtime::non_human_policy::effective_non_human_policy_summary(
                        &objectives,
                    ),
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

    fn controller_blocker(blocker_id: &str, blocker_group: &str) -> BenchmarkControllerBlocker {
        BenchmarkControllerBlocker {
            blocker_id: blocker_id.to_string(),
            blocker_group: blocker_group.to_string(),
            note: "test".to_string(),
        }
    }

    #[test]
    fn recommend_patch_when_outside_budget_maps_to_bounded_candidate_family() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        let snapshot = sample_snapshot();

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "recommend_patch");
        assert_eq!(result.problem_class, "suspicious_forwarded_reach_overspend");
        assert_eq!(result.guidance_status, "bounded_family_guidance");
        assert_eq!(result.tractability, "family_level_policy_choice");
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
    fn reconcile_prefers_explicit_controller_move_selection_over_legacy_escalation_hint() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        let mut snapshot = sample_snapshot();
        snapshot.benchmark_results.escalation_hint.decision = "observe_longer".to_string();
        snapshot.benchmark_results.escalation_hint.candidate_action_families.clear();

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "recommend_patch");
        assert_eq!(result.move_selection.status, "selected");
        assert_eq!(result.recognition_evaluation.status, "steady");
        assert_eq!(
            result
                .proposal
                .as_ref()
                .expect("proposal present")
                .patch_family,
            "fingerprint_signal"
        );
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
        snapshot.benchmark_results.controller_contract.move_selection.decision =
            "observe_longer".to_string();
        snapshot.benchmark_results.controller_contract.move_selection.blockers = vec![
            controller_blocker(
                "verified_identity_botness_conflict_guardrail",
                "controller_guardrail",
            ),
        ];

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "observe_longer");
        assert!(result
            .refusal_reasons
            .contains(&"verified_identity_botness_conflict_guardrail".to_string()));
    }

    #[test]
    fn observe_longer_when_scrapling_surface_contract_is_not_ready() {
        let cfg = defaults().clone();
        let mut snapshot = sample_snapshot();
        snapshot.benchmark_results.overall_status = "outside_budget".to_string();
        snapshot.benchmark_results.escalation_hint.decision = "observe_longer".to_string();
        snapshot.benchmark_results.escalation_hint.problem_class =
            "scrapling_surface_contract_gap".to_string();
        snapshot.benchmark_results.tuning_eligibility.status = "blocked".to_string();
        snapshot.benchmark_results.tuning_eligibility.blockers = vec![
            "scrapling_surface_contract_not_ready".to_string(),
            "scrapling_surface_blocking:maze_navigation".to_string(),
        ];
        snapshot.benchmark_results.escalation_hint.blockers =
            snapshot.benchmark_results.tuning_eligibility.blockers.clone();
        snapshot
            .benchmark_results
            .controller_contract
            .restriction_diagnosis
            .problem_class = "scrapling_surface_contract_gap".to_string();
        snapshot.benchmark_results.controller_contract.move_selection.decision =
            "observe_longer".to_string();
        snapshot.benchmark_results.controller_contract.move_selection.blockers = vec![
            controller_blocker("scrapling_surface_contract_not_ready", "surface_proof"),
            controller_blocker("scrapling_surface_blocking:maze_navigation", "surface_proof"),
        ];

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "observe_longer");
        assert!(result
            .refusal_reasons
            .contains(&"scrapling_surface_contract_not_ready".to_string()));
        assert!(result
            .refusal_reasons
            .contains(&"scrapling_surface_blocking:maze_navigation".to_string()));
    }

    #[test]
    fn observe_longer_when_exploit_progress_evidence_is_low_confidence() {
        let cfg = defaults().clone();
        let mut snapshot = sample_snapshot();
        snapshot.benchmark_results.overall_status = "outside_budget".to_string();
        snapshot.benchmark_results.escalation_hint.decision = "observe_longer".to_string();
        snapshot.benchmark_results.escalation_hint.problem_class =
            "scrapling_exploit_progress_gap".to_string();
        snapshot.benchmark_results.tuning_eligibility.status = "blocked".to_string();
        snapshot.benchmark_results.tuning_eligibility.blockers =
            vec!["scrapling_exploit_evidence_quality_low".to_string()];
        snapshot.benchmark_results.escalation_hint.blockers =
            snapshot.benchmark_results.tuning_eligibility.blockers.clone();
        snapshot.benchmark_results.escalation_hint.evidence_quality =
            crate::observability::benchmark_results::BenchmarkDiagnosisEvidenceQuality {
                status: "low_confidence".to_string(),
                diagnosis_confidence: "low".to_string(),
                attribution_status: "category_native".to_string(),
                sample_status: "sufficient".to_string(),
                freshness_status: "fresh_recent_run".to_string(),
                persona_diversity_status: "single_persona".to_string(),
                reproducibility_status: "single_run_only".to_string(),
                locality_status: "localized".to_string(),
                breach_loci: vec![BenchmarkExploitLocus {
                    locus_id: "public_path_traversal".to_string(),
                    locus_label: "Public Path Traversal".to_string(),
                    stage_id: "exposure".to_string(),
                    evidence_status: "progress_observed".to_string(),
                    attempt_count: Some(2),
                    attempt_count_status: "measured".to_string(),
                    cost_channel_ids: vec![
                        "public_content_exposure".to_string(),
                        "shuma_served_bytes".to_string(),
                    ],
                    cost_channel_status: "derived".to_string(),
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/landing".to_string(),
                    sample_response_status: Some(200),
                    repair_family_candidates: vec![
                        "fingerprint_signal".to_string(),
                        "botness".to_string(),
                        "core_policy".to_string(),
                    ],
                    repair_family_status: "derived".to_string(),
                }],
                note: "Single-persona exploit evidence is not yet strong enough for bounded tuning."
                    .to_string(),
            };
        snapshot.benchmark_results.escalation_hint.breach_loci = snapshot
            .benchmark_results
            .escalation_hint
            .evidence_quality
            .breach_loci
            .clone();
        snapshot
            .benchmark_results
            .controller_contract
            .restriction_diagnosis
            .problem_class = "scrapling_exploit_progress_gap".to_string();
        snapshot.benchmark_results.controller_contract.restriction_diagnosis.status =
            "blocked_by_missing_truth".to_string();
        snapshot.benchmark_results.controller_contract.restriction_diagnosis.confidence =
            "low".to_string();
        snapshot.benchmark_results.controller_contract.restriction_diagnosis.breach_loci =
            snapshot.benchmark_results.escalation_hint.breach_loci.clone();
        snapshot.benchmark_results.controller_contract.move_selection.decision =
            "observe_longer".to_string();
        snapshot.benchmark_results.controller_contract.move_selection.blockers = vec![
            controller_blocker("scrapling_exploit_evidence_quality_low", "evidence_quality"),
        ];

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(reconcile_outcome(&result), "observe_longer");
        assert!(result
            .refusal_reasons
            .contains(&"scrapling_exploit_evidence_quality_low".to_string()));
    }

    #[test]
    fn reconcile_surfaces_selected_move_lineage_for_localized_gap() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        let mut snapshot = sample_snapshot();
        snapshot.benchmark_results.escalation_hint.breach_loci = vec![BenchmarkExploitLocus {
            locus_id: "public_path_traversal".to_string(),
            locus_label: "Public Path Traversal".to_string(),
            stage_id: "exposure".to_string(),
            evidence_status: "progress_observed".to_string(),
            attempt_count: Some(2),
            attempt_count_status: "measured".to_string(),
            cost_channel_ids: vec![
                "public_content_exposure".to_string(),
                "shuma_served_bytes".to_string(),
            ],
            cost_channel_status: "derived".to_string(),
            sample_request_method: "GET".to_string(),
            sample_request_path: "/sim/public/landing".to_string(),
            sample_response_status: Some(200),
            repair_family_candidates: vec![
                "fingerprint_signal".to_string(),
                "botness".to_string(),
                "core_policy".to_string(),
            ],
            repair_family_status: "derived".to_string(),
        }];
        snapshot.benchmark_results.escalation_hint.evidence_quality.status =
            "high_confidence".to_string();
        snapshot.benchmark_results.escalation_hint.evidence_quality.diagnosis_confidence =
            "high".to_string();
        snapshot.benchmark_results.controller_contract.restriction_diagnosis.status =
            "localized".to_string();
        snapshot.benchmark_results.controller_contract.restriction_diagnosis.confidence =
            "high".to_string();
        snapshot.benchmark_results.controller_contract.restriction_diagnosis.breach_loci =
            snapshot.benchmark_results.escalation_hint.breach_loci.clone();

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(result.outcome, "recommend_patch");
        assert_eq!(result.diagnosis.status, "localized");
        assert_eq!(result.diagnosis.breach_loci.len(), 1);
        assert_eq!(result.move_selection.status, "selected");
        assert_eq!(
            result.move_selection.selected_family.as_deref(),
            Some("fingerprint_signal")
        );
        assert_eq!(
            result.move_selection.selected_breach_locus_ids,
            vec!["public_path_traversal".to_string()]
        );
        assert_eq!(result.move_selection.config_ring_status, "bounded_ring_available");
    }

    #[test]
    fn reconcile_promotes_code_evolution_candidate_to_first_class_referral() {
        let cfg = defaults().clone();
        let mut snapshot = sample_snapshot();
        snapshot.benchmark_results.escalation_hint.decision =
            "code_evolution_candidate".to_string();
        snapshot.benchmark_results.escalation_hint.guidance_status =
            "code_evolution_only".to_string();
        snapshot.benchmark_results.escalation_hint.candidate_action_families.clear();
        snapshot.benchmark_results.controller_contract.move_selection.decision =
            "code_evolution_candidate".to_string();
        snapshot.benchmark_results.controller_contract.move_selection.guidance_status =
            "code_evolution_only".to_string();
        snapshot.benchmark_results.controller_contract.move_selection.candidate_action_families
            .clear();

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(result.outcome, "code_evolution_referral");
        assert_eq!(result.move_selection.status, "code_evolution_referral");
        assert_eq!(result.move_selection.code_evolution_status, "required");
        assert_eq!(result.move_selection.config_ring_status, "not_applicable");
        assert!(result.proposal.is_none());
    }

    #[test]
    fn reconcile_emits_config_ring_exhausted_after_repeated_failed_bounded_moves() {
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        let mut snapshot = sample_snapshot();
        snapshot.benchmark_results.escalation_hint.breach_loci = vec![BenchmarkExploitLocus {
            locus_id: "public_path_traversal".to_string(),
            locus_label: "Public Path Traversal".to_string(),
            stage_id: "exposure".to_string(),
            evidence_status: "progress_observed".to_string(),
            attempt_count: Some(2),
            attempt_count_status: "measured".to_string(),
            cost_channel_ids: vec![
                "public_content_exposure".to_string(),
                "shuma_served_bytes".to_string(),
            ],
            cost_channel_status: "derived".to_string(),
            sample_request_method: "GET".to_string(),
            sample_request_path: "/sim/public/landing".to_string(),
            sample_response_status: Some(200),
            repair_family_candidates: vec![
                "fingerprint_signal".to_string(),
                "botness".to_string(),
                "core_policy".to_string(),
            ],
            repair_family_status: "derived".to_string(),
        }];
        snapshot.benchmark_results.controller_contract.restriction_diagnosis.status =
            "localized".to_string();
        snapshot.benchmark_results.controller_contract.restriction_diagnosis.breach_loci =
            snapshot.benchmark_results.escalation_hint.breach_loci.clone();
        snapshot.episode_archive = OperatorSnapshotEpisodeArchive {
            schema_version: "oversight_episode_archive_v1".to_string(),
            homeostasis: crate::observability::benchmark_comparison::classify_homeostasis(
                &[],
                snapshot
                    .game_contract
                    .evaluator_scorecard
                    .comparison_contract
                    .minimum_completed_cycles_for_homeostasis,
            ),
            rows: vec![
                failed_episode_row("fingerprint_signal", 1_700_000_090),
                failed_episode_row("fingerprint_signal", 1_700_000_080),
            ],
        };

        let result = reconcile(&cfg, &snapshot, "manual_admin");

        assert_eq!(result.outcome, "config_ring_exhausted");
        assert_eq!(result.move_selection.status, "config_ring_exhausted");
        assert_eq!(result.move_selection.config_ring_status, "exhausted");
        assert_eq!(result.move_selection.code_evolution_status, "review_required");
        assert!(result.proposal.is_none());
    }

    fn failed_episode_row(family: &str, completed_at_ts: u64) -> OperatorSnapshotEpisodeRecord {
        OperatorSnapshotEpisodeRecord {
            episode_id: format!("episode-{family}-{completed_at_ts}"),
            proposal_id: Some(format!("proposal-{family}-{completed_at_ts}")),
            completed_at_ts,
            trigger_source: "periodic_supervisor".to_string(),
            evaluation_context: OperatorSnapshotEpisodeEvaluationContext {
                objective_revision: "revision-1".to_string(),
                profile_id: "human_only_private".to_string(),
                subject_kind: "current_instance".to_string(),
                comparison_mode: "prior_window".to_string(),
            },
            baseline_scorecard: BenchmarkComparableSnapshot {
                generated_at: completed_at_ts.saturating_sub(10),
                subject_kind: "current_instance".to_string(),
                watch_window: OperatorSnapshotWindow {
                    start_ts: completed_at_ts.saturating_sub(86_399),
                    end_ts: completed_at_ts,
                    duration_seconds: 86_400,
                },
                coverage_status: "supported".to_string(),
                overall_status: "outside_budget".to_string(),
                families: Vec::new(),
            },
            proposal: Some(OperatorSnapshotEpisodeProposal {
                patch_family: family.to_string(),
                patch: serde_json::json!({"fingerprint_signal_enabled": true}),
                expected_impact: "test".to_string(),
                confidence: "high".to_string(),
                controller_status: "allowed".to_string(),
                canary_requirement: "required".to_string(),
                matched_group_ids: vec!["fingerprint_signal.policy".to_string()],
                note: "test".to_string(),
            }),
            proposal_status: "accepted".to_string(),
            watch_window_result: "rollback_applied".to_string(),
            retain_or_rollback: "rolled_back".to_string(),
            benchmark_deltas: Vec::new(),
            hard_guardrail_triggers: Vec::new(),
            cycle_judgment: "regressed".to_string(),
            homeostasis_eligible: true,
            benchmark_urgency_status: "critical".to_string(),
            homeostasis_break_status: "triggered".to_string(),
            homeostasis_break_reasons: vec!["candidate_baseline_regressed".to_string()],
            restart_baseline: BenchmarkHomeostasisRestartBaseline {
                status: "available".to_string(),
                generated_at: Some(completed_at_ts.saturating_sub(10)),
                subject_kind: Some("current_instance".to_string()),
                source: "pre_canary_baseline".to_string(),
                note: "test".to_string(),
            },
            evidence_references: Vec::new(),
        }
    }
}
