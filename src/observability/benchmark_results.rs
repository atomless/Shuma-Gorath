use serde::{Deserialize, Serialize};

use crate::config::AllowedActionsSurface;
use crate::config::Config;
use crate::observability::benchmark_suite::BENCHMARK_SUITE_SCHEMA_VERSION;
use crate::observability::non_human_classification::NonHumanClassificationReadiness;
use crate::observability::non_human_coverage::NonHumanCoverageSummary;
use crate::observability::operator_snapshot::{
    OperatorBudgetDistanceSummary, OperatorSnapshotAdversarySim, OperatorSnapshotLane,
    OperatorSnapshotLiveTraffic, OperatorSnapshotNonHumanTrafficSummary, OperatorSnapshotWindow,
    ReplayPromotionSummary,
};
use crate::observability::operator_snapshot_objectives::{
    objective_profile_is_strict_human_only, OperatorObjectivesProfile,
};
use super::benchmark_adversary_effectiveness::representative_adversary_effectiveness_family;
use super::benchmark_beneficial_non_human::beneficial_non_human_posture_family;
use super::benchmark_mixed_attacker_evidence_quality::mixed_attacker_evidence_quality_assessment;
use super::benchmark_mixed_attacker_restriction_progress::mixed_attacker_restriction_progress_family;
use super::benchmark_non_human_categories::non_human_category_posture_family;
use super::benchmark_scrapling_exploit_progress::scrapling_exploit_progress_family;
use super::benchmark_scrapling_surface_contract::{
    scrapling_surface_contract_family, scrapling_surface_contract_tuning_blockers,
};
use super::benchmark_urgency::benchmark_urgency_summary;
use super::benchmark_comparison::{
    apply_prior_window_comparison, BenchmarkComparableSnapshot,
};
use super::benchmark_results_comparison::{
    derive_escalation_hint, overall_coverage_status, overall_status,
};
use super::benchmark_results_families::{
    likely_human_friction_family, suspicious_origin_cost_family,
};
use super::operator_snapshot_verified_identity::OperatorSnapshotVerifiedIdentitySummary;

pub(crate) const BENCHMARK_RESULTS_SCHEMA_VERSION: &str = "benchmark_results_v1";

fn benchmark_comparison_not_available() -> String {
    "not_available".to_string()
}

fn is_benchmark_comparison_not_available(value: &str) -> bool {
    value == "not_available"
}

pub(crate) fn unavailable_benchmark_diagnosis_evidence_quality() -> BenchmarkDiagnosisEvidenceQuality {
    BenchmarkDiagnosisEvidenceQuality {
        status: "not_available".to_string(),
        diagnosis_confidence: "not_available".to_string(),
        attribution_status: "not_available".to_string(),
        sample_status: "not_available".to_string(),
        freshness_status: "not_available".to_string(),
        recent_window_support_status: "not_available".to_string(),
        locality_status: "not_localized".to_string(),
        breach_loci: Vec::new(),
        note: "Exploit-evidence quality has not been attached to this benchmark hint yet."
            .to_string(),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn unavailable_benchmark_urgency_summary() -> BenchmarkUrgencySummary {
    BenchmarkUrgencySummary {
        status: "not_available".to_string(),
        exploit_short_window_status: "not_available".to_string(),
        exploit_long_window_status: "not_available".to_string(),
        restriction_confidence_status: "not_available".to_string(),
        abuse_backstop_status: "not_available".to_string(),
        likely_human_short_window_status: "not_available".to_string(),
        likely_human_long_window_status: "not_available".to_string(),
        homeostasis_break_status: "not_triggered".to_string(),
        homeostasis_break_reasons: Vec::new(),
        note: "Urgency has not been attached to this benchmark payload yet.".to_string(),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn unavailable_benchmark_controller_contract() -> BenchmarkControllerContract {
    BenchmarkControllerContract {
        restriction_diagnosis: BenchmarkRestrictionDiagnosis {
            problem_class: "not_available".to_string(),
            status: "not_available".to_string(),
            confidence: "not_available".to_string(),
            repair_surface_candidates: Vec::new(),
            breach_loci: Vec::new(),
            blockers: Vec::new(),
            note: "Restriction diagnosis is not available because the controller contract is not materialized yet."
                .to_string(),
        },
        recognition_evaluation: BenchmarkRecognitionEvaluationStatus {
            status: "not_available".to_string(),
            trigger_family_ids: Vec::new(),
            blockers: Vec::new(),
            note: "Recognition evaluation is not available because the controller contract is not materialized yet."
                .to_string(),
        },
        move_selection: BenchmarkMoveSelectionGuidance {
            decision: "observe_longer".to_string(),
            review_status: "manual_review_required".to_string(),
            guidance_status: "not_available".to_string(),
            tractability: "not_available".to_string(),
            expected_direction: "continue_observing".to_string(),
            trigger_family_ids: Vec::new(),
            candidate_action_families: Vec::new(),
            family_guidance: Vec::new(),
            blockers: Vec::new(),
            note: "Move-selection guidance is not available because the controller contract is not materialized yet."
                .to_string(),
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkBaselineReference {
    pub reference_kind: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<u64>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkMetricResult {
    pub metric_id: String,
    pub status: String,
    pub current: Option<f64>,
    pub target: Option<f64>,
    pub delta: Option<f64>,
    pub exactness: String,
    pub basis: String,
    pub capability_gate: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_current: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_delta: Option<f64>,
    #[serde(
        default = "benchmark_comparison_not_available",
        skip_serializing_if = "is_benchmark_comparison_not_available"
    )]
    pub comparison_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkExploitLocus {
    pub locus_id: String,
    pub locus_label: String,
    pub stage_id: String,
    pub evidence_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_count: Option<u64>,
    pub attempt_count_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cost_channel_ids: Vec<String>,
    pub cost_channel_status: String,
    pub sample_request_method: String,
    pub sample_request_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_response_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repair_family_candidates: Vec<String>,
    pub repair_family_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkFamilyResult {
    pub family_id: String,
    pub status: String,
    pub capability_gate: String,
    pub note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_status: Option<String>,
    #[serde(
        default = "benchmark_comparison_not_available",
        skip_serializing_if = "is_benchmark_comparison_not_available"
    )]
    pub comparison_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exploit_loci: Vec<BenchmarkExploitLocus>,
    pub metrics: Vec<BenchmarkMetricResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkEscalationFamilyGuidance {
    pub family: String,
    pub likely_human_risk: String,
    pub tolerated_non_human_risk: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkDiagnosisEvidenceQuality {
    pub status: String,
    pub diagnosis_confidence: String,
    pub attribution_status: String,
    pub sample_status: String,
    pub freshness_status: String,
    pub recent_window_support_status: String,
    pub locality_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breach_loci: Vec<BenchmarkExploitLocus>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkUrgencySummary {
    pub status: String,
    pub exploit_short_window_status: String,
    pub exploit_long_window_status: String,
    pub restriction_confidence_status: String,
    pub abuse_backstop_status: String,
    pub likely_human_short_window_status: String,
    pub likely_human_long_window_status: String,
    pub homeostasis_break_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub homeostasis_break_reasons: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkEscalationHint {
    pub availability: String,
    pub decision: String,
    pub review_status: String,
    pub problem_class: String,
    pub guidance_status: String,
    pub tractability: String,
    pub expected_direction: String,
    pub trigger_family_ids: Vec<String>,
    pub trigger_metric_ids: Vec<String>,
    pub candidate_action_families: Vec<String>,
    pub family_guidance: Vec<BenchmarkEscalationFamilyGuidance>,
    pub blockers: Vec<String>,
    pub evidence_quality: BenchmarkDiagnosisEvidenceQuality,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breach_loci: Vec<BenchmarkExploitLocus>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkControllerBlocker {
    pub blocker_id: String,
    pub blocker_group: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkRestrictionDiagnosis {
    pub problem_class: String,
    pub status: String,
    pub confidence: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repair_surface_candidates: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breach_loci: Vec<BenchmarkExploitLocus>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<BenchmarkControllerBlocker>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkRecognitionEvaluationStatus {
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trigger_family_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<BenchmarkControllerBlocker>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkMoveSelectionGuidance {
    pub decision: String,
    pub review_status: String,
    pub guidance_status: String,
    pub tractability: String,
    pub expected_direction: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trigger_family_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidate_action_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub family_guidance: Vec<BenchmarkEscalationFamilyGuidance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<BenchmarkControllerBlocker>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkControllerContract {
    pub restriction_diagnosis: BenchmarkRestrictionDiagnosis,
    pub recognition_evaluation: BenchmarkRecognitionEvaluationStatus,
    pub move_selection: BenchmarkMoveSelectionGuidance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkTuningEligibility {
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkProtectedEvidenceSummary {
    pub availability: String,
    pub evidence_status: String,
    pub tuning_eligible: bool,
    pub protected_basis: String,
    #[serde(default)]
    pub protected_lineage_count: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub eligibility_blockers: Vec<String>,
    pub note: String,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn unavailable_benchmark_protected_evidence_summary(
) -> BenchmarkProtectedEvidenceSummary {
    BenchmarkProtectedEvidenceSummary {
        availability: "not_materialized".to_string(),
        evidence_status: "not_materialized".to_string(),
        tuning_eligible: false,
        protected_basis: "none".to_string(),
        protected_lineage_count: 0,
        eligibility_blockers: vec!["protected_tuning_evidence_not_ready".to_string()],
        note: "Protected tuning evidence is not materialized yet.".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkResultsPayload {
    pub schema_version: String,
    pub suite_version: String,
    pub generated_at: u64,
    pub input_snapshot_generated_at: u64,
    pub subject_kind: String,
    pub watch_window: OperatorSnapshotWindow,
    pub baseline_reference: BenchmarkBaselineReference,
    pub coverage_status: String,
    pub overall_status: String,
    pub improvement_status: String,
    pub non_human_classification: NonHumanClassificationReadiness,
    pub non_human_coverage: NonHumanCoverageSummary,
    pub tuning_eligibility: BenchmarkTuningEligibility,
    pub protected_evidence: BenchmarkProtectedEvidenceSummary,
    pub families: Vec<BenchmarkFamilyResult>,
    pub escalation_hint: BenchmarkEscalationHint,
    pub controller_contract: BenchmarkControllerContract,
    pub urgency: BenchmarkUrgencySummary,
    pub replay_promotion: ReplayPromotionSummary,
}

pub(crate) fn build_benchmark_results_from_snapshot_sections(
    generated_at: u64,
    input_snapshot_generated_at: u64,
    watch_window: &OperatorSnapshotWindow,
    objectives: &OperatorObjectivesProfile,
    live_traffic: &OperatorSnapshotLiveTraffic,
    adversary_sim: &OperatorSnapshotAdversarySim,
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
    budget_distance: &OperatorBudgetDistanceSummary,
    summary: &crate::observability::monitoring::MonitoringSummary,
    cfg: &Config,
    allowed_actions: &AllowedActionsSurface,
    replay_promotion: &ReplayPromotionSummary,
    prior_window_reference: Option<&BenchmarkComparableSnapshot>,
) -> BenchmarkResultsPayload {
    let strict_human_only_private = objective_profile_is_strict_human_only(objectives);
    let suspicious_tracking_lane =
        suspicious_origin_tracking_lane(objectives, live_traffic, adversary_sim);
    let suspicious_family = suspicious_origin_cost_family(
        suspicious_tracking_lane.as_ref(),
        budget_distance,
        strict_human_only_private,
    );
    let friction_family = likely_human_friction_family(budget_distance);
    let scrapling_surface_family = scrapling_surface_contract_family(adversary_sim);
    let mixed_attacker_family = mixed_attacker_restriction_progress_family(adversary_sim);
    let scrapling_exploit_family = scrapling_exploit_progress_family(adversary_sim);
    let adversary_family = representative_adversary_effectiveness_family(adversary_sim);
    let verified_identity =
        super::operator_snapshot_verified_identity::verified_identity_summary(
            summary,
            cfg,
            objectives,
            non_human_traffic.restriction_receipts.as_slice(),
        );
    let non_human_family = beneficial_non_human_posture_family(
        summary,
        cfg,
        objectives,
        non_human_traffic,
        &verified_identity,
    );
    let category_posture_family = non_human_category_posture_family(objectives, non_human_traffic);
    let exploit_evidence_quality =
        mixed_attacker_evidence_quality_assessment(adversary_sim, non_human_traffic);
    let mut families = vec![
        suspicious_family,
        friction_family,
        mixed_attacker_family,
        scrapling_exploit_family,
        scrapling_surface_family,
        adversary_family,
        non_human_family,
        category_posture_family,
    ];
    let (baseline_reference, improvement_status) = apply_prior_window_comparison(
        generated_at,
        families.as_mut_slice(),
        prior_window_reference,
    );
    let protected_evidence = protected_tuning_evidence_summary(
        replay_promotion,
        families.as_slice(),
        &exploit_evidence_quality,
    );
    let tuning_eligibility =
        tuning_eligibility(
            adversary_sim,
            non_human_traffic,
            &verified_identity,
            &protected_evidence,
            families.as_slice(),
            &exploit_evidence_quality,
        );
    let derived_escalation_hint = derive_escalation_hint(allowed_actions, families.as_slice());
    let escalation_hint = if tuning_eligibility.status != "eligible" {
        attach_exploit_evidence_quality(
            BenchmarkEscalationHint {
            availability: derived_escalation_hint.availability.clone(),
            decision: "observe_longer".to_string(),
            review_status: "manual_review_required".to_string(),
            problem_class: derived_escalation_hint.problem_class.clone(),
            guidance_status: derived_escalation_hint.guidance_status.clone(),
            tractability: derived_escalation_hint.tractability.clone(),
            expected_direction: derived_escalation_hint.expected_direction.clone(),
            trigger_family_ids: derived_escalation_hint.trigger_family_ids.clone(),
            trigger_metric_ids: derived_escalation_hint.trigger_metric_ids.clone(),
            candidate_action_families: Vec::new(),
            family_guidance: derived_escalation_hint.family_guidance.clone(),
            blockers: tuning_eligibility.blockers.clone(),
            evidence_quality: unavailable_benchmark_diagnosis_evidence_quality(),
            breach_loci: Vec::new(),
            note: "Current benchmark pressure cannot justify tuning because restriction-grade confidence, protected evidence, or no-harm guardrails are not yet eligible for controller-grade judgment."
                .to_string(),
        },
            &exploit_evidence_quality,
        )
    } else {
        attach_exploit_evidence_quality(derived_escalation_hint, &exploit_evidence_quality)
    };
    let urgency = benchmark_urgency_summary(
        families.as_slice(),
        non_human_traffic.restriction_readiness.status.as_str(),
        &exploit_evidence_quality,
    );
    let controller_contract =
        benchmark_controller_contract(families.as_slice(), &tuning_eligibility, &escalation_hint);

    BenchmarkResultsPayload {
        schema_version: BENCHMARK_RESULTS_SCHEMA_VERSION.to_string(),
        suite_version: BENCHMARK_SUITE_SCHEMA_VERSION.to_string(),
        generated_at,
        input_snapshot_generated_at,
        subject_kind: "current_instance".to_string(),
        watch_window: watch_window.clone(),
        baseline_reference,
        coverage_status: overall_coverage_status(families.as_slice()),
        overall_status: overall_status(families.as_slice()),
        improvement_status,
        non_human_classification: non_human_traffic.restriction_readiness.clone(),
        non_human_coverage: non_human_traffic
            .recognition_evaluation
            .coverage
            .compact_for_benchmark(),
        tuning_eligibility,
        protected_evidence,
        escalation_hint,
        controller_contract,
        urgency,
        replay_promotion: replay_promotion.clone(),
        families,
    }
}

fn suspicious_origin_tracking_lane(
    objectives: &OperatorObjectivesProfile,
    live_traffic: &OperatorSnapshotLiveTraffic,
    adversary_sim: &OperatorSnapshotAdversarySim,
) -> Option<OperatorSnapshotLane> {
    if objective_profile_is_strict_human_only(objectives) {
        return Some(OperatorSnapshotLane {
            lane: "adversary_sim".to_string(),
            exactness: "derived".to_string(),
            basis: "observed".to_string(),
            total_requests: adversary_sim.total_requests,
            forwarded_requests: adversary_sim.forwarded_requests,
            short_circuited_requests: adversary_sim.short_circuited_requests,
            control_response_requests: adversary_sim.control_response_requests,
            forwarded_upstream_latency_ms_total: adversary_sim.forwarded_upstream_latency_ms_total,
            forwarded_response_bytes: adversary_sim.forwarded_response_bytes,
            shuma_served_response_bytes: adversary_sim.shuma_served_response_bytes,
        });
    }
    live_traffic.suspicious_automation.clone()
}

fn tuning_eligibility(
    adversary_sim: &OperatorSnapshotAdversarySim,
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
    verified_identity: &OperatorSnapshotVerifiedIdentitySummary,
    protected_evidence: &BenchmarkProtectedEvidenceSummary,
    families: &[BenchmarkFamilyResult],
    exploit_evidence_quality: &BenchmarkDiagnosisEvidenceQuality,
) -> BenchmarkTuningEligibility {
    let exploit_progress_outside_budget = families.iter().any(|family| {
        family.family_id == "mixed_attacker_restriction_progress"
            && family.status == "outside_budget"
    });
    let exploit_progress_surface_native_high_confidence = exploit_progress_outside_budget
        && exploit_evidence_quality.status == "high_confidence"
        && exploit_evidence_quality.attribution_status == "surface_native_shared_path";
    let mut blockers = if non_human_traffic.restriction_readiness.status != "ready"
        && !exploit_progress_surface_native_high_confidence
    {
        let mut blockers = vec!["non_human_classification_not_ready".to_string()];
        blockers.extend(non_human_traffic.restriction_readiness.blockers.iter().cloned());
        blockers
    } else {
        protected_evidence
            .eligibility_blockers
            .iter()
            .cloned()
            .chain(
                (!protected_evidence.tuning_eligible)
                    .then(|| "protected_tuning_evidence_not_ready".to_string())
                    .into_iter(),
            )
            .collect()
    };
    blockers.extend(scrapling_surface_contract_tuning_blockers(adversary_sim));
    blockers.extend(scrapling_exploit_evidence_quality_blockers(
        families,
        exploit_evidence_quality,
    ));
    blockers.extend(verified_identity_guardrail_blockers(
        verified_identity,
        families,
    ));
    blockers.sort();
    blockers.dedup();

    BenchmarkTuningEligibility {
        status: if blockers.is_empty() {
            "eligible".to_string()
        } else {
            "blocked".to_string()
        },
        blockers,
    }
}

fn protected_tuning_evidence_summary(
    replay_promotion: &ReplayPromotionSummary,
    families: &[BenchmarkFamilyResult],
    exploit_evidence_quality: &BenchmarkDiagnosisEvidenceQuality,
) -> BenchmarkProtectedEvidenceSummary {
    let exploit_progress_outside_budget = families.iter().any(|family| {
        family.family_id == "mixed_attacker_restriction_progress"
            && family.status == "outside_budget"
    });
    let live_runtime_protected = exploit_progress_outside_budget
        && exploit_evidence_quality.status == "high_confidence"
        && matches!(
            exploit_evidence_quality.attribution_status.as_str(),
            "surface_native_shared_path" | "category_and_surface_native"
        )
        && exploit_evidence_quality.recent_window_support_status == "reproduced_recently"
        && !exploit_evidence_quality.breach_loci.is_empty();
    if live_runtime_protected {
        return BenchmarkProtectedEvidenceSummary {
            availability: "materialized".to_string(),
            evidence_status: "protected".to_string(),
            tuning_eligible: true,
            protected_basis: "live_mixed_attacker_runtime".to_string(),
            protected_lineage_count: replay_promotion.protected_lineage_count,
            eligibility_blockers: Vec::new(),
            note: format!(
                "Protected tuning evidence is backed by strong live mixed-attacker runtime proof across the recent window at: {}.",
                exploit_evidence_quality
                    .breach_loci
                    .iter()
                    .map(|locus| locus.locus_label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        };
    }

    if replay_promotion.tuning_eligible {
        return BenchmarkProtectedEvidenceSummary {
            availability: replay_promotion.availability.clone(),
            evidence_status: if replay_promotion.evidence_status.is_empty() {
                "protected".to_string()
            } else {
                replay_promotion.evidence_status.clone()
            },
            tuning_eligible: true,
            protected_basis: if replay_promotion.protected_basis.is_empty() {
                "replay_promoted_lineage".to_string()
            } else {
                replay_promotion.protected_basis.clone()
            },
            protected_lineage_count: replay_promotion.protected_lineage_count,
            eligibility_blockers: replay_promotion.eligibility_blockers.clone(),
            note: if replay_promotion.protected_lineage_count > 0 {
                format!(
                    "Protected tuning evidence is backed by {} replay-promoted lineage row(s).",
                    replay_promotion.protected_lineage_count
                )
            } else {
                "Protected tuning evidence is backed by replay-promoted lineage."
                    .to_string()
            },
        };
    }

    let mut eligibility_blockers = replay_promotion.eligibility_blockers.clone();
    if exploit_progress_outside_budget {
        eligibility_blockers.push("live_mixed_attacker_runtime_not_protected_yet".to_string());
    }
    eligibility_blockers.sort();
    eligibility_blockers.dedup();

    BenchmarkProtectedEvidenceSummary {
        availability: replay_promotion.availability.clone(),
        evidence_status: if replay_promotion.evidence_status.is_empty() {
            if exploit_progress_outside_budget {
                "advisory_only".to_string()
            } else {
                "not_materialized".to_string()
            }
        } else {
            replay_promotion.evidence_status.clone()
        },
        tuning_eligible: false,
        protected_basis: "none".to_string(),
        protected_lineage_count: replay_promotion.protected_lineage_count,
        eligibility_blockers,
        note: if exploit_progress_outside_budget {
            format!(
                "Live mixed-attacker runtime pressure is visible, but it is not protected tuning evidence yet because exploit evidence status={}, attribution={}, recent window={}, and replay protection status={}.",
                exploit_evidence_quality.status,
                exploit_evidence_quality.attribution_status,
                exploit_evidence_quality.recent_window_support_status,
                replay_promotion.evidence_status
            )
        } else if replay_promotion.availability == "not_materialized" {
            "Protected tuning evidence is not materialized yet because neither replay lineage nor strong live mixed-attacker runtime proof is available."
                .to_string()
        } else {
            "Protected tuning evidence is not ready yet because replay lineage remains advisory or incomplete."
                .to_string()
        },
    }
}

fn attach_exploit_evidence_quality(
    mut hint: BenchmarkEscalationHint,
    exploit_evidence_quality: &BenchmarkDiagnosisEvidenceQuality,
) -> BenchmarkEscalationHint {
    hint.evidence_quality = exploit_evidence_quality.clone();
    hint.breach_loci = exploit_evidence_quality.breach_loci.clone();
    hint
}

fn benchmark_controller_contract(
    families: &[BenchmarkFamilyResult],
    tuning_eligibility: &BenchmarkTuningEligibility,
    escalation_hint: &BenchmarkEscalationHint,
) -> BenchmarkControllerContract {
    let move_selection_blockers = typed_controller_blockers(
        tuning_eligibility
            .blockers
            .iter()
            .chain(escalation_hint.blockers.iter())
            .cloned()
            .collect(),
    );
    let move_selection = BenchmarkMoveSelectionGuidance {
        decision: escalation_hint.decision.clone(),
        review_status: escalation_hint.review_status.clone(),
        guidance_status: escalation_hint.guidance_status.clone(),
        tractability: escalation_hint.tractability.clone(),
        expected_direction: escalation_hint.expected_direction.clone(),
        trigger_family_ids: escalation_hint.trigger_family_ids.clone(),
        candidate_action_families: escalation_hint.candidate_action_families.clone(),
        family_guidance: escalation_hint.family_guidance.clone(),
        blockers: move_selection_blockers.clone(),
        note: escalation_hint.note.clone(),
    };

    let restriction_diagnosis = BenchmarkRestrictionDiagnosis {
        problem_class: escalation_hint.problem_class.clone(),
        status: if escalation_hint.breach_loci.is_empty() {
            if move_selection_blockers.is_empty() {
                "aggregate_only".to_string()
            } else {
                "blocked_by_missing_truth".to_string()
            }
        } else if escalation_hint.breach_loci.len() == 1 {
            "localized".to_string()
        } else {
            "distributed".to_string()
        },
        confidence: escalation_hint.evidence_quality.diagnosis_confidence.clone(),
        repair_surface_candidates: escalation_hint.candidate_action_families.clone(),
        breach_loci: escalation_hint.breach_loci.clone(),
        blockers: move_selection_blockers.clone(),
        note: if escalation_hint.breach_loci.is_empty() {
            if move_selection_blockers.is_empty() {
                "Restriction diagnosis is still aggregate and does not yet localize a repair locus."
                    .to_string()
            } else {
                format!(
                    "Restriction diagnosis is blocked by {} typed controller blocker(s).",
                    move_selection_blockers.len()
                )
            }
        } else {
            format!(
                "Restriction diagnosis localizes the shortfall at: {}.",
                escalation_hint
                    .breach_loci
                    .iter()
                    .map(|locus| locus.locus_label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
    };

    BenchmarkControllerContract {
        restriction_diagnosis,
        recognition_evaluation: recognition_evaluation_status(families),
        move_selection,
    }
}

fn recognition_evaluation_status(
    families: &[BenchmarkFamilyResult],
) -> BenchmarkRecognitionEvaluationStatus {
    let Some(family) = families
        .iter()
        .find(|family| family.family_id == "non_human_category_posture")
    else {
        return BenchmarkRecognitionEvaluationStatus {
            status: "not_materialized".to_string(),
            trigger_family_ids: Vec::new(),
            blockers: Vec::new(),
            note: "Recognition evaluation is not materialized in the current benchmark family set."
                .to_string(),
        };
    };

    let blockers = if family.status == "outside_budget" {
        typed_controller_blockers(vec!["recognition_evaluation_outside_budget_only".to_string()])
    } else {
        Vec::new()
    };
    let status = match family.status.as_str() {
        "outside_budget" => "needs_work",
        "near_limit" => "watching",
        "inside_budget" => "steady",
        "insufficient_evidence" => "unscored",
        other => other,
    };

    BenchmarkRecognitionEvaluationStatus {
        status: status.to_string(),
        trigger_family_ids: vec![family.family_id.clone()],
        blockers,
        note: if family.status == "outside_budget" {
            "Recognition evaluation is outside budget, but remains a side quest rather than a bounded-tuning oracle for undeclared hostile traffic."
                .to_string()
        } else {
            "Recognition evaluation remains explicit and separate from restriction scoring."
                .to_string()
        },
    }
}

fn typed_controller_blockers(blocker_ids: Vec<String>) -> Vec<BenchmarkControllerBlocker> {
    let mut blocker_ids = blocker_ids;
    blocker_ids.sort();
    blocker_ids.dedup();
    blocker_ids
        .into_iter()
        .map(|blocker_id| BenchmarkControllerBlocker {
            blocker_group: controller_blocker_group(blocker_id.as_str()).to_string(),
            note: controller_blocker_note(blocker_id.as_str()).to_string(),
            blocker_id,
        })
        .collect()
}

fn controller_blocker_group(blocker_id: &str) -> &'static str {
    if blocker_id == "recognition_evaluation_outside_budget_only" {
        "recognition_evaluation"
    } else if blocker_id.starts_with("non_human_classification")
        || blocker_id.starts_with("restriction_receipts_")
        || blocker_id.starts_with("non_human_")
    {
        "shared_classification"
    } else if blocker_id.contains("surface_contract") {
        "surface_proof"
    } else if blocker_id.contains("evidence_quality") || blocker_id == "insufficient_evidence" {
        "evidence_quality"
    } else if blocker_id == "no_matching_config_surface"
        || blocker_id == "no_candidate_family"
        || blocker_id.starts_with("no_bounded_patch:")
    {
        "bounded_move"
    } else if blocker_id == "near_limit_only" || blocker_id == "outside_budget_not_observed" {
        "observation_window"
    } else if blocker_id.starts_with("verified_identity_")
        || blocker_id == "protected_tuning_evidence_not_ready"
    {
        "controller_guardrail"
    } else {
        "controller_guardrail"
    }
}

fn controller_blocker_note(blocker_id: &str) -> &'static str {
    match controller_blocker_group(blocker_id) {
        "recognition_evaluation" => {
            "Recognition quality still needs work, but this remains evaluator-only rather than a runtime or tuning shortcut."
        }
        "shared_classification" => {
            "Restriction-grade shared-path classification or receipt truth is not ready enough yet."
        }
        "surface_proof" => {
            "Surface-contract proof is still missing or blocking, so bounded repair cannot yet rely on this locus cleanly."
        }
        "evidence_quality" => {
            "Exploit evidence is not yet strong enough to justify a bounded config move."
        }
        "bounded_move" => {
            "No still-legal bounded config move maps cleanly to the current pressure surface."
        }
        "observation_window" => {
            "The loop still needs another observation window before it can justify a bounded move."
        }
        _ => "A controller guardrail is still blocking bounded config tuning.",
    }
}

fn scrapling_exploit_evidence_quality_blockers(
    families: &[BenchmarkFamilyResult],
    exploit_evidence_quality: &BenchmarkDiagnosisEvidenceQuality,
) -> Vec<String> {
    let exploit_progress_outside_budget = families.iter().any(|family| {
        family.family_id == "mixed_attacker_restriction_progress"
            && family.status == "outside_budget"
    });
    if exploit_progress_outside_budget && exploit_evidence_quality.status != "high_confidence" {
        vec!["mixed_attacker_exploit_evidence_quality_low".to_string()]
    } else {
        Vec::new()
    }
}

fn verified_identity_guardrail_blockers(
    verified_identity: &OperatorSnapshotVerifiedIdentitySummary,
    families: &[BenchmarkFamilyResult],
) -> Vec<String> {
    let mut blockers = Vec::new();
    if matches!(
        verified_identity.taxonomy_alignment.status.as_str(),
        "degraded" | "insufficient_evidence"
    ) {
        blockers.push("verified_identity_taxonomy_alignment_guardrail".to_string());
    }
    let Some(beneficial_family) = families
        .iter()
        .find(|family| family.family_id == "beneficial_non_human_posture")
    else {
        return blockers;
    };
    for metric in &beneficial_family.metrics {
        if metric.status != "outside_budget" {
            continue;
        }
        match metric.metric_id.as_str() {
            "verified_botness_conflict_rate" => {
                blockers.push("verified_identity_botness_conflict_guardrail".to_string());
            }
            "user_triggered_agent_friction_mismatch_rate" => {
                blockers.push("verified_identity_user_triggered_agent_guardrail".to_string());
            }
            "friction_mismatch_rate" => {
                blockers.push("verified_identity_friction_mismatch_guardrail".to_string());
            }
            "taxonomy_alignment_mismatch_rate" => {
                blockers.push("verified_identity_taxonomy_alignment_guardrail".to_string());
            }
            _ => {}
        }
    }
    blockers
}

#[cfg(test)]
mod tests {
    use super::{
        build_benchmark_results_from_snapshot_sections, derive_escalation_hint,
        BenchmarkFamilyResult, BenchmarkMetricResult,
    };
    use crate::challenge::KeyValueStore;
    use crate::config::allowed_actions_v1;
    use crate::config::defaults;
    use crate::observability::benchmark_comparison::comparable_snapshot_from_results;
    use crate::observability::monitoring::{record_request_outcome, summarize_with_store};
    use crate::observability::operator_snapshot::{
        build_operator_snapshot_payload, OperatorSnapshotRecentChanges,
        OperatorSnapshotRecentSimRun,
    };
    use crate::observability::replay_promotion::ReplayPromotionSummary;
    use crate::runtime::effect_intents::ExecutionMode;
    use crate::runtime::request_outcome::{
        RenderedRequestOutcome, RequestOutcomeClass, RequestOutcomeLane, ResponseKind,
        TrafficOrigin,
    };
    use crate::runtime::traffic_classification::{
        MeasurementScope, PolicySource, RouteActionFamily, TrafficLane,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    fn protected_replay_promotion_summary() -> ReplayPromotionSummary {
        let mut summary = ReplayPromotionSummary::not_materialized();
        summary.availability = "materialized".to_string();
        summary.evidence_status = "protected".to_string();
        summary.tuning_eligible = true;
        summary.protected_basis = "replay_promoted_lineage".to_string();
        summary.protected_lineage_count = 1;
        summary.ineligible_runtime_lanes = vec!["synthetic_traffic".to_string()];
        summary.eligibility_blockers.clear();
        summary
    }

    fn covered_non_human_summary() -> crate::observability::operator_snapshot::OperatorSnapshotNonHumanTrafficSummary {
        crate::observability::operator_snapshot::OperatorSnapshotNonHumanTrafficSummary {
            availability: "taxonomy_seeded".to_string(),
            taxonomy: crate::runtime::non_human_taxonomy::canonical_non_human_taxonomy(),
            coverage: crate::observability::non_human_coverage::NonHumanCoverageSummary {
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
            restriction_readiness: crate::observability::non_human_classification::NonHumanClassificationReadiness {
                status: "ready".to_string(),
                blockers: Vec::new(),
                live_receipt_count: 1,
                adversary_sim_receipt_count: 1,
            },
            decision_chain: Vec::new(),
            restriction_receipts: Vec::new(),
            recognition_evaluation:
                crate::observability::operator_snapshot_non_human::OperatorSnapshotNonHumanRecognitionEvaluationSummary {
                    readiness: crate::observability::non_human_classification::NonHumanClassificationReadiness {
                        status: "ready".to_string(),
                        blockers: Vec::new(),
                        live_receipt_count: 1,
                        adversary_sim_receipt_count: 1,
                    },
                    coverage: crate::observability::non_human_coverage::NonHumanCoverageSummary {
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
        }
    }

    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl TestStore {
        fn new() -> Self {
            Self {
                map: Mutex::new(HashMap::new()),
            }
        }
    }

    impl KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.lock().expect("map lock").get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map
                .lock()
                .expect("map lock")
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            self.map.lock().expect("map lock").remove(key);
            Ok(())
        }

        fn get_keys(&self) -> Result<Vec<String>, ()> {
            Ok(self.map.lock().expect("map lock").keys().cloned().collect())
        }
    }

    #[test]
    fn benchmark_results_payload_uses_snapshot_budget_rows_and_family_registry() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
                forwarded_upstream_latency_ms: None,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_100,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "run_001".to_string(),
                lane: "deterministic_black_box".to_string(),
                profile: "fast_smoke".to_string(),
                observed_fulfillment_modes: Vec::new(),
                observed_category_ids: Vec::new(),
                first_ts: 1_700_000_000,
                last_ts: 1_700_000_100,
                monitoring_event_count: 3,
                defense_delta_count: 2,
                ban_outcome_count: 0,
                owned_surface_coverage: None,
                llm_runtime_summary: None,
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_100,
            1_700_000_100,
            1_700_000_100,
        );

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_100,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &ReplayPromotionSummary::not_materialized(),
            None,
        );
        assert_eq!(payload.schema_version, "benchmark_results_v1");
        assert_eq!(payload.suite_version, "benchmark_suite_v1");
        assert_eq!(payload.subject_kind, "current_instance");
        assert!(payload
            .families
            .iter()
            .any(|family| family.family_id == "likely_human_friction"));
        assert_eq!(payload.coverage_status, "partial_support");
        assert_eq!(payload.improvement_status, "not_available");
        assert_eq!(payload.non_human_classification.status, "not_observed");
        assert_eq!(payload.non_human_coverage.overall_status, "unavailable");
        assert_eq!(payload.tuning_eligibility.status, "blocked");
        assert_eq!(payload.escalation_hint.availability, "partial_support");
        assert_eq!(payload.escalation_hint.decision, "observe_longer");
        assert_eq!(payload.replay_promotion.availability, "not_materialized");
        assert_eq!(
            payload.escalation_hint.review_status,
            "manual_review_required"
        );
        assert!(payload
            .families
            .iter()
            .any(|family| family.family_id == "non_human_category_posture"));
    }

    #[test]
    fn escalation_hint_promotes_supported_budget_breach_to_config_tuning_candidate() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
                forwarded_upstream_latency_ms: None,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::VerifiedBot,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphVerifiedIdentityTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::DeclaredCrawler,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let mut snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_100,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_100,
            1_700_000_100,
            1_700_000_100,
        );
        let row = snapshot
            .budget_distance
            .rows
            .iter_mut()
            .find(|row| row.metric == "likely_human_friction_rate")
            .expect("likely human friction budget row present");
        row.status = "outside_budget".to_string();
        row.current = 0.12;
        row.delta = 0.10;
        snapshot.allowed_actions = allowed_actions_v1();
        snapshot.non_human_traffic.coverage.overall_status = "covered".to_string();
        snapshot.non_human_traffic.coverage.blocking_reasons.clear();
        snapshot.non_human_traffic.coverage.blocking_category_ids.clear();
        snapshot.non_human_traffic.coverage.covered_category_count =
            snapshot.non_human_traffic.coverage.mapped_category_count;
        snapshot.non_human_traffic.coverage.partial_category_count = 0;
        snapshot.non_human_traffic.coverage.stale_category_count = 0;
        snapshot.non_human_traffic.coverage.unavailable_category_count = 0;
        snapshot.non_human_traffic.recognition_evaluation.coverage =
            snapshot.non_human_traffic.coverage.clone();

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_100,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            None,
        );
        assert_eq!(payload.non_human_classification.status, "ready");
        assert_eq!(payload.tuning_eligibility.status, "eligible");
        assert_eq!(payload.escalation_hint.decision, "config_tuning_candidate");
        assert_eq!(
            payload.escalation_hint.review_status,
            "manual_review_required"
        );
        assert!(payload
            .escalation_hint
            .trigger_family_ids
            .contains(&"likely_human_friction".to_string()));
        assert!(payload
            .escalation_hint
            .candidate_action_families
            .contains(&"challenge".to_string()));
    }

    #[test]
    fn escalation_hint_promotes_unaddressable_budget_breach_to_code_evolution_candidate() {
        let snapshot = build_operator_snapshot_payload(
            &TestStore::new(),
            "default",
            1_700_000_100,
            &crate::observability::monitoring::summarize_with_store(&TestStore::new(), 24, 10),
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_100,
            1_700_000_100,
            1_700_000_100,
        );
        let families = vec![BenchmarkFamilyResult {
            family_id: "beneficial_non_human_posture".to_string(),
            status: "outside_budget".to_string(),
            capability_gate: "not_yet_supported".to_string(),
            note: "identity posture is missing".to_string(),
            baseline_status: None,
            comparison_status: "not_available".to_string(),
            exploit_loci: Vec::new(),
            metrics: vec![BenchmarkMetricResult {
                metric_id: "allowed_as_intended_rate".to_string(),
                status: "not_yet_supported".to_string(),
                current: None,
                target: None,
                delta: None,
                exactness: "derived".to_string(),
                basis: "mixed".to_string(),
                capability_gate: "not_yet_supported".to_string(),
                baseline_current: None,
                comparison_delta: None,
                comparison_status: "not_available".to_string(),
            }],
        }];

        let hint = derive_escalation_hint(&snapshot.allowed_actions, families.as_slice());
        assert_eq!(hint.decision, "code_evolution_candidate");
        assert_eq!(hint.review_status, "manual_review_required");
        assert_eq!(hint.guidance_status, "code_evolution_only");
        assert_eq!(hint.tractability, "code_or_capability_gap");
        assert!(hint.blockers.contains(&"family_capability_gap".to_string()));
        assert!(!hint
            .blockers
            .contains(&"no_matching_config_surface".to_string()));
    }

    #[test]
    fn benchmark_results_materialize_supported_adversary_and_beneficial_non_human_families() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::SignedAgent,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 512,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphVerifiedIdentityTranche,
            },
        );
        crate::observability::monitoring::record_verified_identity_telemetry(
            &store,
            &crate::bot_identity::telemetry::IdentityVerificationTelemetryRecord {
                scheme: Some(crate::bot_identity::contracts::IdentityScheme::ProviderSignedAgent),
                category: Some(crate::bot_identity::contracts::IdentityCategory::UserTriggeredAgent),
                provenance: crate::bot_identity::contracts::IdentityProvenance::Provider,
                result_status:
                    crate::bot_identity::verification::IdentityVerificationResultStatus::Verified,
                failure: None,
                freshness: crate::bot_identity::verification::IdentityVerificationFreshness::Fresh,
                end_user_controlled: true,
                operator: Some("openai".to_string()),
                stable_identity: Some("chatgpt-agent".to_string()),
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_200,
            &summary,
            &[
                OperatorSnapshotRecentSimRun {
                    run_id: "run_001".to_string(),
                    lane: "synthetic_traffic".to_string(),
                    profile: "fast_smoke".to_string(),
                    observed_fulfillment_modes: Vec::new(),
                    observed_category_ids: Vec::new(),
                    first_ts: 1_700_000_120,
                    last_ts: 1_700_000_140,
                    monitoring_event_count: 4,
                    defense_delta_count: 2,
                    ban_outcome_count: 1,
                    owned_surface_coverage: None,
                llm_runtime_summary: None,
                },
                OperatorSnapshotRecentSimRun {
                    run_id: "run_002".to_string(),
                    lane: "synthetic_traffic".to_string(),
                    profile: "abuse_regression".to_string(),
                    observed_fulfillment_modes: Vec::new(),
                    observed_category_ids: Vec::new(),
                    first_ts: 1_700_000_150,
                    last_ts: 1_700_000_190,
                    monitoring_event_count: 6,
                    defense_delta_count: 3,
                    ban_outcome_count: 1,
                    owned_surface_coverage: None,
                llm_runtime_summary: None,
                },
            ],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_200,
            1_700_000_200,
            1_700_000_200,
        );
        let mut cfg = defaults().clone();
        cfg.verified_identity.enabled = true;

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_200,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &cfg,
            &snapshot.allowed_actions,
            &ReplayPromotionSummary::not_materialized(),
            None,
        );

        let adversary = payload
            .families
            .iter()
            .find(|family| family.family_id == "representative_adversary_effectiveness")
            .expect("adversary family");
        assert_ne!(adversary.status, "not_yet_supported");
        assert_ne!(adversary.capability_gate, "not_yet_supported");
        assert!(adversary
            .metrics
            .iter()
            .all(|metric| metric.status != "not_yet_supported"));

        let beneficial = payload
            .families
            .iter()
            .find(|family| family.family_id == "beneficial_non_human_posture")
            .expect("beneficial family");
        assert_ne!(beneficial.status, "not_yet_supported");
        assert_ne!(beneficial.capability_gate, "not_yet_supported");
        assert!(beneficial
            .metrics
            .iter()
            .all(|metric| metric.status != "not_yet_supported"));
        let category_posture = payload
            .families
            .iter()
            .find(|family| family.family_id == "non_human_category_posture")
            .expect("category posture family");
        assert!(category_posture
            .metrics
            .iter()
            .any(|metric| metric.metric_id == "category_posture_alignment:indexing_bot"));
    }

    #[test]
    fn benchmark_results_materialize_strict_human_only_suspicious_origin_metrics_from_adversary_sim() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 80,
                forwarded_upstream_latency_ms: Some(30),
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::SuspiciousAutomation,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Derived,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Mixed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: Some(70),
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        for _ in 0..3 {
            record_request_outcome(
                &store,
                &RenderedRequestOutcome {
                    traffic_origin: TrafficOrigin::AdversarySim,
                    measurement_scope: MeasurementScope::IngressPrimary,
                    route_action_family: RouteActionFamily::PublicContent,
                    execution_mode: ExecutionMode::Enforced,
                    traffic_lane: Some(RequestOutcomeLane {
                        lane: TrafficLane::SuspiciousAutomation,
                        exactness:
                            crate::observability::hot_read_contract::TelemetryExactness::Derived,
                        basis: crate::observability::hot_read_contract::TelemetryBasis::Mixed,
                    }),
                    non_human_category: None,
                    outcome_class: RequestOutcomeClass::ShortCircuited,
                    response_kind: ResponseKind::NotABot,
                    http_status: 200,
                    response_bytes: 55,
                    forwarded_upstream_latency_ms: None,
                    forward_attempted: false,
                    forward_failure_class: None,
                    intended_action: None,
                    policy_source: PolicySource::PolicyGraphSecondTranche,
                },
            );
        }
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::SuspiciousAutomation,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Derived,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Mixed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 90,
                forwarded_upstream_latency_ms: Some(50),
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_210,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_210,
            1_700_000_210,
            1_700_000_210,
        );

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_210,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &ReplayPromotionSummary::not_materialized(),
            None,
        );

        let suspicious = payload
            .families
            .iter()
            .find(|family| family.family_id == "suspicious_origin_cost")
            .expect("suspicious origin cost family");
        let request_rate = suspicious
            .metrics
            .iter()
            .find(|metric| metric.metric_id == "suspicious_forwarded_request_rate")
            .expect("request-rate metric");
        let latency_share = suspicious
            .metrics
            .iter()
            .find(|metric| metric.metric_id == "suspicious_forwarded_latency_share")
            .expect("latency share metric");
        let average_latency = suspicious
            .metrics
            .iter()
            .find(|metric| metric.metric_id == "suspicious_average_forward_latency_ms")
            .expect("average latency metric");

        assert!(suspicious.note.contains("adversary-sim scope"));
        assert_eq!(request_rate.target, Some(0.0));
        assert_eq!(request_rate.current, Some(0.25));
        assert_eq!(latency_share.status, "outside_budget");
        assert!((latency_share.current.expect("latency share current") - 1.0).abs() < 0.000_001);
        assert_eq!(average_latency.status, "tracking_only");
        assert!((average_latency.current.expect("average latency current") - 50.0).abs() < 0.000_001);
    }

    #[test]
    fn verified_identity_guardrails_block_tuning_when_conflicts_are_outside_budget() {
        let mut cfg = defaults().clone();
        cfg.verified_identity.enabled = true;
        let mut summary = crate::observability::monitoring::MonitoringSummary::default();
        summary.verified_identity.attempts = 6;
        summary.verified_identity.verified = 6;
        summary
            .verified_identity
            .top_verified_identities
            .push(crate::observability::monitoring::VerifiedIdentitySeenRow {
                operator: "openai".to_string(),
                stable_identity: "chatgpt-agent".to_string(),
                scheme: "provider_signed_agent".to_string(),
                category: "user_triggered_agent".to_string(),
                provenance: "provider".to_string(),
                end_user_controlled: true,
                count: 6,
            });
        summary.request_outcomes.by_policy_source.push(
            crate::observability::monitoring::RequestOutcomeBreakdownSummaryRow {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                value: "policy_graph_verified_identity_tranche".to_string(),
                total_requests: 6,
                forwarded_requests: 2,
                short_circuited_requests: 4,
                control_response_requests: 0,
            },
        );
        let mut objectives =
            crate::observability::operator_snapshot_objectives::default_operator_objectives(
                1_700_000_500,
            );
        objectives
            .category_postures
            .iter_mut()
            .find(|row| row.category_id.as_str() == "agent_on_behalf_of_human")
            .expect("agent-on-behalf-of-human posture")
            .posture = "tolerated".to_string();

        let payload = build_benchmark_results_from_snapshot_sections(
            1_700_000_500,
            1_700_000_500,
            &crate::observability::operator_snapshot::OperatorSnapshotWindow {
                start_ts: 1_700_000_000,
                end_ts: 1_700_000_500,
                duration_seconds: 500,
            },
            &objectives,
            &crate::observability::operator_snapshot_live_traffic::OperatorSnapshotLiveTraffic {
                traffic_origin: "live".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                total_requests: 6,
                forwarded_requests: 2,
                short_circuited_requests: 4,
                control_response_requests: 0,
                forwarded_upstream_latency_ms_total: 0,
                forwarded_response_bytes: 200,
                shuma_served_response_bytes: 400,
                likely_human: None,
                suspicious_automation: None,
                human_friction: None,
            },
            &crate::observability::operator_snapshot_live_traffic::OperatorSnapshotAdversarySim {
                traffic_origin: "adversary_sim".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                execution_mode: "enforced".to_string(),
                total_requests: 0,
                forwarded_requests: 0,
                short_circuited_requests: 0,
                control_response_requests: 0,
                forwarded_upstream_latency_ms_total: 0,
                forwarded_response_bytes: 0,
                shuma_served_response_bytes: 0,
                recent_runs: Vec::new(),
            },
            &crate::observability::operator_snapshot::OperatorSnapshotNonHumanTrafficSummary {
                availability: "taxonomy_seeded".to_string(),
                taxonomy: crate::runtime::non_human_taxonomy::canonical_non_human_taxonomy(),
                coverage: crate::observability::non_human_coverage::NonHumanCoverageSummary {
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
                restriction_readiness: crate::observability::non_human_classification::NonHumanClassificationReadiness {
                    status: "ready".to_string(),
                    blockers: Vec::new(),
                    live_receipt_count: 1,
                    adversary_sim_receipt_count: 1,
                },
                decision_chain: Vec::new(),
                restriction_receipts: vec![crate::observability::non_human_classification::NonHumanClassificationReceipt {
                    traffic_origin: "live".to_string(),
                    measurement_scope: "ingress_primary".to_string(),
                    execution_mode: "enforced".to_string(),
                    lane: "category_crosswalk".to_string(),
                    category_id: "agent_on_behalf_of_human".to_string(),
                    category_label: "Agent On Behalf Of Human".to_string(),
                    assignment_status: "classified".to_string(),
                    exactness: "exact".to_string(),
                    basis: "observed".to_string(),
                    degradation_status: "current".to_string(),
                    total_requests: 6,
                    forwarded_requests: 2,
                    short_circuited_requests: 4,
                    evidence_references: Vec::new(),
                }],
                recognition_evaluation:
                    crate::observability::operator_snapshot_non_human::OperatorSnapshotNonHumanRecognitionEvaluationSummary {
                        readiness: crate::observability::non_human_classification::NonHumanClassificationReadiness {
                            status: "ready".to_string(),
                            blockers: Vec::new(),
                            live_receipt_count: 1,
                            adversary_sim_receipt_count: 1,
                        },
                        coverage: crate::observability::non_human_coverage::NonHumanCoverageSummary {
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
            &crate::observability::operator_snapshot::OperatorBudgetDistanceSummary {
                rows: Vec::new(),
            },
            &summary,
            &cfg,
            &allowed_actions_v1(),
            &protected_replay_promotion_summary(),
            None,
        );

        assert_eq!(payload.tuning_eligibility.status, "blocked");
        assert!(payload
            .tuning_eligibility
            .blockers
            .contains(&"verified_identity_botness_conflict_guardrail".to_string()));
        assert!(payload
            .tuning_eligibility
            .blockers
            .contains(&"verified_identity_user_triggered_agent_guardrail".to_string()));
    }

    #[test]
    fn benchmark_results_fail_closed_when_non_human_classification_is_not_ready() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
                forwarded_upstream_latency_ms: None,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_300,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_300,
            1_700_000_300,
            1_700_000_300,
        );

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_300,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &ReplayPromotionSummary::not_materialized(),
            None,
        );

        assert_eq!(payload.non_human_classification.status, "not_observed");
        assert_eq!(payload.tuning_eligibility.status, "blocked");
        assert_eq!(payload.escalation_hint.decision, "observe_longer");
        assert!(payload
            .escalation_hint
            .blockers
            .contains(&"non_human_classification_not_ready".to_string()));
    }

    #[test]
    fn benchmark_results_fail_closed_when_adversary_sim_categories_are_only_projected_from_recent_runs() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::VerifiedBot,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphVerifiedIdentityTranche,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_360,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-request-native".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec![
                    "crawler".to_string(),
                    "bulk_scraper".to_string(),
                    "http_agent".to_string(),
                ],
                observed_category_ids: vec![
                    "indexing_bot".to_string(),
                    "ai_scraper_bot".to_string(),
                    "http_agent".to_string(),
                ],
                first_ts: 1_700_000_300,
                last_ts: 1_700_000_340,
                monitoring_event_count: 9,
                defense_delta_count: 2,
                ban_outcome_count: 0,
                owned_surface_coverage: None,
                llm_runtime_summary: None,
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_360,
            1_700_000_360,
            1_700_000_360,
        );

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_360,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &ReplayPromotionSummary::not_materialized(),
            None,
        );

        assert_eq!(payload.non_human_classification.status, "partial");
        assert!(payload
            .tuning_eligibility
            .blockers
            .contains(&"non_human_classification_not_ready".to_string()));
        let family = payload
            .families
            .iter()
            .find(|family| family.family_id == "non_human_category_posture")
            .expect("category posture family");
        let indexing = family
            .metrics
            .iter()
            .find(|metric| metric.metric_id == "category_posture_alignment:indexing_bot")
            .expect("indexing posture metric");
        assert_eq!(indexing.status, "insufficient_evidence");
        assert_eq!(indexing.current, None);
        assert_eq!(indexing.basis, "projected_recent_sim_run");
    }

    #[test]
    fn benchmark_results_fail_closed_when_non_human_coverage_is_not_ready() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::VerifiedBot,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphVerifiedIdentityTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::DeclaredCrawler,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_350,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_350,
            1_700_000_350,
            1_700_000_350,
        );

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_350,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &ReplayPromotionSummary::not_materialized(),
            None,
        );

        assert_eq!(payload.non_human_classification.status, "ready");
        assert_eq!(payload.non_human_coverage.overall_status, "partial");
        assert_eq!(payload.tuning_eligibility.status, "blocked");
        assert_eq!(payload.escalation_hint.decision, "observe_longer");
        assert!(payload
            .escalation_hint
            .blockers
            .contains(&"protected_tuning_evidence_not_ready".to_string()));
    }

    #[test]
    fn benchmark_results_fail_closed_when_protected_tuning_evidence_is_not_ready() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
                forwarded_upstream_latency_ms: None,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::VerifiedBot,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphVerifiedIdentityTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::DeclaredCrawler,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let mut snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_375,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_375,
            1_700_000_375,
            1_700_000_375,
        );
        let row = snapshot
            .budget_distance
            .rows
            .iter_mut()
            .find(|row| row.metric == "likely_human_friction_rate")
            .expect("likely human friction budget row present");
        row.status = "outside_budget".to_string();
        row.current = 0.12;
        row.delta = 0.10;
        snapshot.allowed_actions = allowed_actions_v1();
        snapshot.non_human_traffic.coverage.overall_status = "covered".to_string();
        snapshot.non_human_traffic.coverage.blocking_reasons.clear();
        snapshot.non_human_traffic.coverage.blocking_category_ids.clear();
        snapshot.non_human_traffic.coverage.covered_category_count =
            snapshot.non_human_traffic.coverage.mapped_category_count;
        snapshot.non_human_traffic.coverage.partial_category_count = 0;
        snapshot.non_human_traffic.coverage.stale_category_count = 0;
        snapshot.non_human_traffic.coverage.unavailable_category_count = 0;
        snapshot.non_human_traffic.recognition_evaluation.coverage =
            snapshot.non_human_traffic.coverage.clone();

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_375,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &ReplayPromotionSummary::not_materialized(),
            None,
        );

        assert_eq!(payload.non_human_classification.status, "ready");
        assert_eq!(payload.non_human_coverage.overall_status, "covered");
        assert_eq!(payload.tuning_eligibility.status, "blocked");
        assert_eq!(payload.escalation_hint.decision, "observe_longer");
        assert!(payload
            .escalation_hint
            .blockers
            .contains(&"protected_tuning_evidence_not_ready".to_string()));
        assert!(payload
            .escalation_hint
            .blockers
            .contains(&"replay_promotion_not_materialized".to_string()));
    }

    #[test]
    fn benchmark_results_fail_closed_when_latest_scrapling_surface_contract_is_blocking() {
        let store = TestStore::new();
        let summary = summarize_with_store(&store, 24, 10);
        let mut snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_380,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-scrapling-blocking".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec![
                    "browser_automation".to_string(),
                    "stealth_browser".to_string(),
                ],
                observed_category_ids: vec!["automated_browser".to_string()],
                first_ts: 1_700_000_300,
                last_ts: 1_700_000_370,
                monitoring_event_count: 11,
                defense_delta_count: 2,
                ban_outcome_count: 1,
                owned_surface_coverage: Some(
                    crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                        overall_status: "partial".to_string(),
                        canonical_surface_ids: vec![
                            "challenge_routing".to_string(),
                            "maze_navigation".to_string(),
                        ],
                        surface_labels: std::collections::BTreeMap::from([
                            (
                                "challenge_routing".to_string(),
                                "Challenge Routing".to_string(),
                            ),
                            (
                                "maze_navigation".to_string(),
                                "Maze Navigation".to_string(),
                            ),
                        ]),
                        required_surface_ids: vec![
                            "challenge_routing".to_string(),
                            "maze_navigation".to_string(),
                        ],
                        satisfied_surface_ids: vec!["challenge_routing".to_string()],
                        blocking_surface_ids: vec!["maze_navigation".to_string()],
                        receipts: vec![
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "challenge_routing".to_string(),
                                success_contract: "mixed_outcomes".to_string(),
                                dependency_kind: "independent".to_string(),
                                dependency_surface_ids: Vec::new(),
                                coverage_status: "pass_observed".to_string(),
                                surface_state: "satisfied".to_string(),
                                satisfied: true,
                                blocked_by_surface_ids: Vec::new(),
                                attempt_count: 2,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/challenge".to_string(),
                                sample_response_status: Some(200),
                            },
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "maze_navigation".to_string(),
                                success_contract: "should_pass_some".to_string(),
                                dependency_kind: "independent".to_string(),
                                dependency_surface_ids: Vec::new(),
                                coverage_status: "fail_observed".to_string(),
                                surface_state: "attempted_blocked".to_string(),
                                satisfied: false,
                                blocked_by_surface_ids: Vec::new(),
                                attempt_count: 2,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/maze".to_string(),
                                sample_response_status: Some(429),
                            },
                        ],
                    },
                ),
                llm_runtime_summary: None,
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_380,
            1_700_000_380,
            1_700_000_380,
        );
        snapshot.non_human_traffic = covered_non_human_summary();

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_380,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            None,
        );

        let family = payload
            .families
            .iter()
            .find(|family| family.family_id == "scrapling_surface_contract")
            .expect("scrapling surface contract family");
        assert_eq!(family.status, "outside_budget");
        assert!(family
            .note
            .contains("Maze Navigation (attempted and blocked | independent surface)"));
        assert_eq!(payload.overall_status, "outside_budget");
        assert_eq!(payload.tuning_eligibility.status, "blocked");
        assert!(payload
            .tuning_eligibility
            .blockers
            .contains(&"scrapling_surface_contract_not_ready".to_string()));
        assert!(payload
            .tuning_eligibility
            .blockers
            .contains(&"scrapling_surface_blocking:maze_navigation".to_string()));
        assert_eq!(payload.escalation_hint.decision, "observe_longer");
        assert!(payload
            .escalation_hint
            .blockers
            .contains(&"scrapling_surface_contract_not_ready".to_string()));
    }

    #[test]
    fn benchmark_results_accept_when_latest_scrapling_surface_contract_is_covered() {
        let store = TestStore::new();
        let summary = summarize_with_store(&store, 24, 10);
        let mut snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_390,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-scrapling-covered".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec!["crawler".to_string()],
                observed_category_ids: vec!["indexing_bot".to_string()],
                first_ts: 1_700_000_300,
                last_ts: 1_700_000_385,
                monitoring_event_count: 6,
                defense_delta_count: 0,
                ban_outcome_count: 0,
                owned_surface_coverage: Some(
                    crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                        overall_status: "covered".to_string(),
                        canonical_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        surface_labels: std::collections::BTreeMap::from([
                            (
                                "public_path_traversal".to_string(),
                                "Public Path Traversal".to_string(),
                            ),
                            (
                                "challenge_routing".to_string(),
                                "Challenge Routing".to_string(),
                            ),
                        ]),
                        required_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        satisfied_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        blocking_surface_ids: Vec::new(),
                        receipts: vec![
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "public_path_traversal".to_string(),
                                success_contract: "should_pass_some".to_string(),
                                dependency_kind: "independent".to_string(),
                                dependency_surface_ids: Vec::new(),
                                coverage_status: "pass_observed".to_string(),
                                surface_state: "satisfied".to_string(),
                                satisfied: true,
                                blocked_by_surface_ids: Vec::new(),
                                attempt_count: 2,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/".to_string(),
                                sample_response_status: Some(200),
                            },
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "challenge_routing".to_string(),
                                success_contract: "mixed_outcomes".to_string(),
                                dependency_kind: "independent".to_string(),
                                dependency_surface_ids: Vec::new(),
                                coverage_status: "pass_observed".to_string(),
                                surface_state: "satisfied".to_string(),
                                satisfied: true,
                                blocked_by_surface_ids: Vec::new(),
                                attempt_count: 1,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/challenge".to_string(),
                                sample_response_status: Some(200),
                            },
                        ],
                    },
                ),
                llm_runtime_summary: None,
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_390,
            1_700_000_390,
            1_700_000_390,
        );
        snapshot.non_human_traffic = covered_non_human_summary();

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_390,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            None,
        );

        let family = payload
            .families
            .iter()
            .find(|family| family.family_id == "scrapling_surface_contract")
            .expect("scrapling surface contract family");
        assert_eq!(family.status, "inside_budget");
        let exploit_progress = payload
            .families
            .iter()
            .find(|family| family.family_id == "scrapling_exploit_progress")
            .expect("scrapling exploit progress family");
        assert_eq!(exploit_progress.status, "outside_budget");
        assert!(exploit_progress.note.contains("Public Path Traversal"));
        assert!(exploit_progress.note.contains("Challenge Routing"));
        assert_eq!(exploit_progress.exploit_loci.len(), 2);
        assert_eq!(
            exploit_progress.exploit_loci[0].locus_id,
            "public_path_traversal"
        );
        assert_eq!(
            exploit_progress.exploit_loci[1].locus_id,
            "challenge_routing"
        );
        assert_eq!(payload.overall_status, "outside_budget");
        assert!(!payload
            .tuning_eligibility
            .blockers
            .contains(&"scrapling_surface_contract_not_ready".to_string()));
    }

    #[test]
    fn benchmark_results_materialize_mixed_attacker_restriction_family_from_scrapling_and_llm_receipts() {
        let store = TestStore::new();
        let summary = summarize_with_store(&store, 24, 10);
        let mut snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_399,
            &summary,
            &[
                OperatorSnapshotRecentSimRun {
                    run_id: "simrun-scrapling-current".to_string(),
                    lane: "scrapling_traffic".to_string(),
                    profile: "scrapling_runtime_lane".to_string(),
                    observed_fulfillment_modes: vec!["crawler".to_string()],
                    observed_category_ids: vec!["indexing_bot".to_string()],
                    first_ts: 1_700_000_350,
                    last_ts: 1_700_000_396,
                    monitoring_event_count: 8,
                    defense_delta_count: 0,
                    ban_outcome_count: 0,
                    owned_surface_coverage: Some(
                        crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                            overall_status: "covered".to_string(),
                            canonical_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            surface_labels: std::collections::BTreeMap::from([
                                (
                                    "public_path_traversal".to_string(),
                                    "Public Path Traversal".to_string(),
                                ),
                                (
                                    "challenge_routing".to_string(),
                                    "Challenge Routing".to_string(),
                                ),
                            ]),
                            required_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            satisfied_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            blocking_surface_ids: Vec::new(),
                            receipts: vec![
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "public_path_traversal".to_string(),
                                    success_contract: "should_pass_some".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/".to_string(),
                                    sample_response_status: Some(200),
                                },
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "challenge_routing".to_string(),
                                    success_contract: "mixed_outcomes".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 1,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/challenge".to_string(),
                                    sample_response_status: Some(200),
                                },
                            ],
                        },
                    ),
                    llm_runtime_summary: None,
                },
                OperatorSnapshotRecentSimRun {
                    run_id: "simrun-llm-current".to_string(),
                    lane: "bot_red_team".to_string(),
                    profile: "frontier_request_mode".to_string(),
                    observed_fulfillment_modes: vec!["request_mode".to_string()],
                    observed_category_ids: vec!["http_agent".to_string(), "ai_scraper_bot".to_string()],
                    first_ts: 1_700_000_360,
                    last_ts: 1_700_000_398,
                    monitoring_event_count: 4,
                    defense_delta_count: 0,
                    ban_outcome_count: 0,
                    owned_surface_coverage: None,
                    llm_runtime_summary: Some(crate::admin::adversary_sim_worker_plan::LlmRuntimeRecentRunSummary {
                        receipt_count: 1,
                        fulfillment_mode: "request_mode".to_string(),
                        category_targets: vec!["http_agent".to_string(), "ai_scraper_bot".to_string()],
                        backend_kind: "frontier_reference".to_string(),
                        backend_state: "available".to_string(),
                        generation_source: "provider_response".to_string(),
                        provider: "openai".to_string(),
                        model_id: "gpt-5-mini".to_string(),
                        fallback_reason: None,
                        generated_action_count: 3,
                        executed_action_count: 3,
                        failed_action_count: 0,
                        passed_tick_count: 1,
                        failed_tick_count: 0,
                        last_response_status: Some(200),
                        failure_class: None,
                        error: None,
                        terminal_failure: None,
                        latest_realism_receipt: None,
                        latest_action_receipts: vec![
                            crate::admin::adversary_sim_worker_plan::LlmRuntimeActionReceipt {
                                action_index: 0,
                                action_type: "http_get".to_string(),
                                path: "/".to_string(),
                                label: Some("root".to_string()),
                                status: Some(200),
                                error: None,
                            },
                            crate::admin::adversary_sim_worker_plan::LlmRuntimeActionReceipt {
                                action_index: 1,
                                action_type: "http_get".to_string(),
                                path: "/research/".to_string(),
                                label: Some("docs".to_string()),
                                status: Some(200),
                                error: None,
                            },
                        ],
                    }),
                },
            ],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_399,
            1_700_000_399,
            1_700_000_399,
        );
        snapshot.non_human_traffic = covered_non_human_summary();

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_399,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            None,
        );

        let family = payload
            .families
            .iter()
            .find(|family| family.family_id == "mixed_attacker_restriction_progress")
            .expect("mixed attacker restriction family");
        assert_eq!(family.status, "outside_budget");
        assert!(family.note.contains("scrapling_traffic"));
        assert!(family.note.contains("bot_red_team"));
        assert!(family
            .exploit_loci
            .iter()
            .any(|locus| locus.locus_id == "public_path_traversal"));
    }

    #[test]
    fn benchmark_results_block_tuning_when_exploit_progress_evidence_is_low_confidence() {
        let store = TestStore::new();
        let summary = summarize_with_store(&store, 24, 10);
        let mut snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_395,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-scrapling-low-confidence".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec!["crawler".to_string()],
                observed_category_ids: vec!["indexing_bot".to_string()],
                first_ts: 1_700_000_300,
                last_ts: 1_700_000_390,
                monitoring_event_count: 6,
                defense_delta_count: 0,
                ban_outcome_count: 0,
                owned_surface_coverage: Some(
                    crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                        overall_status: "covered".to_string(),
                        canonical_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        surface_labels: std::collections::BTreeMap::from([
                            (
                                "public_path_traversal".to_string(),
                                "Public Path Traversal".to_string(),
                            ),
                            (
                                "challenge_routing".to_string(),
                                "Challenge Routing".to_string(),
                            ),
                        ]),
                        required_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        satisfied_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        blocking_surface_ids: Vec::new(),
                        receipts: vec![
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "public_path_traversal".to_string(),
                                success_contract: "should_pass_some".to_string(),
                                dependency_kind: "independent".to_string(),
                                dependency_surface_ids: Vec::new(),
                                coverage_status: "pass_observed".to_string(),
                                surface_state: "satisfied".to_string(),
                                satisfied: true,
                                blocked_by_surface_ids: Vec::new(),
                                attempt_count: 1,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/".to_string(),
                                sample_response_status: Some(200),
                            },
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "challenge_routing".to_string(),
                                success_contract: "mixed_outcomes".to_string(),
                                dependency_kind: "independent".to_string(),
                                dependency_surface_ids: Vec::new(),
                                coverage_status: "pass_observed".to_string(),
                                surface_state: "satisfied".to_string(),
                                satisfied: true,
                                blocked_by_surface_ids: Vec::new(),
                                attempt_count: 1,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/challenge".to_string(),
                                sample_response_status: Some(200),
                            },
                        ],
                    },
                ),
                llm_runtime_summary: None,
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_395,
            1_700_000_395,
            1_700_000_395,
        );
        snapshot.non_human_traffic = covered_non_human_summary();

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_395,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            None,
        );

        assert_eq!(payload.escalation_hint.evidence_quality.status, "low_confidence");
        assert_eq!(payload.escalation_hint.evidence_quality.diagnosis_confidence, "low");
        assert_eq!(
            payload
                .escalation_hint
                .evidence_quality
                .recent_window_support_status,
            "single_run_only"
        );
        assert_eq!(payload.tuning_eligibility.status, "blocked");
        assert!(payload
            .tuning_eligibility
            .blockers
            .contains(&"mixed_attacker_exploit_evidence_quality_low".to_string()));
        assert_eq!(payload.escalation_hint.decision, "observe_longer");
        assert_eq!(payload.escalation_hint.breach_loci.len(), 2);
    }

    #[test]
    fn benchmark_results_mark_exploit_progress_evidence_high_confidence_when_reproduced_and_localized() {
        let store = TestStore::new();
        let summary = summarize_with_store(&store, 24, 10);
        let mut snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_396,
            &summary,
            &[
                OperatorSnapshotRecentSimRun {
                    run_id: "simrun-scrapling-prior".to_string(),
                    lane: "scrapling_traffic".to_string(),
                    profile: "scrapling_runtime_lane".to_string(),
                    observed_fulfillment_modes: vec!["crawler".to_string()],
                    observed_category_ids: vec!["indexing_bot".to_string()],
                    first_ts: 1_700_000_300,
                    last_ts: 1_700_000_340,
                    monitoring_event_count: 6,
                    defense_delta_count: 0,
                    ban_outcome_count: 0,
                    owned_surface_coverage: Some(
                        crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                            overall_status: "covered".to_string(),
                            canonical_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            surface_labels: std::collections::BTreeMap::from([
                                (
                                    "public_path_traversal".to_string(),
                                    "Public Path Traversal".to_string(),
                                ),
                                (
                                    "challenge_routing".to_string(),
                                    "Challenge Routing".to_string(),
                                ),
                            ]),
                            required_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            satisfied_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            blocking_surface_ids: Vec::new(),
                            receipts: vec![
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "public_path_traversal".to_string(),
                                    success_contract: "should_pass_some".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/".to_string(),
                                    sample_response_status: Some(200),
                                },
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "challenge_routing".to_string(),
                                    success_contract: "mixed_outcomes".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/challenge".to_string(),
                                    sample_response_status: Some(200),
                                },
                            ],
                        },
                    ),
                    llm_runtime_summary: None,
                },
                OperatorSnapshotRecentSimRun {
                    run_id: "simrun-scrapling-current".to_string(),
                    lane: "scrapling_traffic".to_string(),
                    profile: "scrapling_runtime_lane".to_string(),
                    observed_fulfillment_modes: vec![
                        "crawler".to_string(),
                        "bulk_scraper".to_string(),
                    ],
                    observed_category_ids: vec![
                        "indexing_bot".to_string(),
                        "ai_scraper_bot".to_string(),
                    ],
                    first_ts: 1_700_000_350,
                    last_ts: 1_700_000_394,
                    monitoring_event_count: 8,
                    defense_delta_count: 0,
                    ban_outcome_count: 0,
                    owned_surface_coverage: Some(
                        crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                            overall_status: "covered".to_string(),
                            canonical_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            surface_labels: std::collections::BTreeMap::from([
                                (
                                    "public_path_traversal".to_string(),
                                    "Public Path Traversal".to_string(),
                                ),
                                (
                                    "challenge_routing".to_string(),
                                    "Challenge Routing".to_string(),
                                ),
                            ]),
                            required_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            satisfied_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            blocking_surface_ids: Vec::new(),
                            receipts: vec![
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "public_path_traversal".to_string(),
                                    success_contract: "should_pass_some".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/".to_string(),
                                    sample_response_status: Some(200),
                                },
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "challenge_routing".to_string(),
                                    success_contract: "mixed_outcomes".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/challenge".to_string(),
                                    sample_response_status: Some(200),
                                },
                            ],
                        },
                    ),
                    llm_runtime_summary: None,
                },
            ],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_396,
            1_700_000_396,
            1_700_000_396,
        );
        snapshot.non_human_traffic = covered_non_human_summary();

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_396,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            None,
        );

        assert_eq!(payload.escalation_hint.evidence_quality.status, "high_confidence");
        assert_eq!(payload.escalation_hint.evidence_quality.diagnosis_confidence, "high");
        assert_eq!(
            payload
                .escalation_hint
                .evidence_quality
                .recent_window_support_status,
            "reproduced_recently"
        );
    }

    #[test]
    fn benchmark_results_allow_restriction_tuning_from_reproduced_recent_window_even_when_category_readiness_is_partial() {
        let store = TestStore::new();
        let summary = summarize_with_store(&store, 24, 10);
        let mut snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_397,
            &summary,
            &[
                OperatorSnapshotRecentSimRun {
                    run_id: "simrun-scrapling-prior".to_string(),
                    lane: "scrapling_traffic".to_string(),
                    profile: "scrapling_runtime_lane".to_string(),
                    observed_fulfillment_modes: vec!["crawler".to_string()],
                    observed_category_ids: vec!["indexing_bot".to_string()],
                    first_ts: 1_700_000_300,
                    last_ts: 1_700_000_340,
                    monitoring_event_count: 6,
                    defense_delta_count: 0,
                    ban_outcome_count: 0,
                    owned_surface_coverage: Some(
                        crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                            overall_status: "covered".to_string(),
                            canonical_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            surface_labels: std::collections::BTreeMap::from([
                                (
                                    "public_path_traversal".to_string(),
                                    "Public Path Traversal".to_string(),
                                ),
                                (
                                    "challenge_routing".to_string(),
                                    "Challenge Routing".to_string(),
                                ),
                            ]),
                            required_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            satisfied_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            blocking_surface_ids: Vec::new(),
                            receipts: vec![
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "public_path_traversal".to_string(),
                                    success_contract: "should_pass_some".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/".to_string(),
                                    sample_response_status: Some(200),
                                },
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "challenge_routing".to_string(),
                                    success_contract: "mixed_outcomes".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/challenge".to_string(),
                                    sample_response_status: Some(200),
                                },
                            ],
                        },
                    ),
                    llm_runtime_summary: None,
                },
                OperatorSnapshotRecentSimRun {
                    run_id: "simrun-scrapling-current".to_string(),
                    lane: "scrapling_traffic".to_string(),
                    profile: "scrapling_runtime_lane".to_string(),
                    observed_fulfillment_modes: vec!["crawler".to_string()],
                    observed_category_ids: vec!["indexing_bot".to_string()],
                    first_ts: 1_700_000_350,
                    last_ts: 1_700_000_396,
                    monitoring_event_count: 8,
                    defense_delta_count: 0,
                    ban_outcome_count: 0,
                    owned_surface_coverage: Some(
                        crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                            overall_status: "covered".to_string(),
                            canonical_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            surface_labels: std::collections::BTreeMap::from([
                                (
                                    "public_path_traversal".to_string(),
                                    "Public Path Traversal".to_string(),
                                ),
                                (
                                    "challenge_routing".to_string(),
                                    "Challenge Routing".to_string(),
                                ),
                            ]),
                            required_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            satisfied_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            blocking_surface_ids: Vec::new(),
                            receipts: vec![
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "public_path_traversal".to_string(),
                                    success_contract: "should_pass_some".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/".to_string(),
                                    sample_response_status: Some(200),
                                },
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "challenge_routing".to_string(),
                                    success_contract: "mixed_outcomes".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/challenge".to_string(),
                                    sample_response_status: Some(200),
                                },
                            ],
                        },
                    ),
                    llm_runtime_summary: None,
                },
            ],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_397,
            1_700_000_397,
            1_700_000_397,
        );
        snapshot.non_human_traffic = covered_non_human_summary();
        snapshot.non_human_traffic.restriction_readiness.status = "partial".to_string();
        snapshot.non_human_traffic.restriction_readiness.blockers = vec![
            "insufficient_category_evidence".to_string(),
            "degraded_category_receipts_present".to_string(),
        ];
        snapshot.non_human_traffic.recognition_evaluation.readiness =
            snapshot.non_human_traffic.restriction_readiness.clone();

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_397,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            None,
        );

        assert_eq!(payload.escalation_hint.evidence_quality.status, "high_confidence");
        assert_eq!(
            payload
                .escalation_hint
                .evidence_quality
                .recent_window_support_status,
            "reproduced_recently"
        );
        assert_eq!(payload.protected_evidence.evidence_status, "protected");
        assert_eq!(
            payload.protected_evidence.protected_basis,
            "live_mixed_attacker_runtime"
        );
        assert_eq!(payload.tuning_eligibility.status, "eligible");
        assert!(!payload
            .tuning_eligibility
            .blockers
            .contains(&"insufficient_category_evidence".to_string()));
        assert!(!payload
            .tuning_eligibility
            .blockers
            .contains(&"degraded_category_receipts_present".to_string()));
        assert_eq!(payload.escalation_hint.decision, "config_tuning_candidate");
    }

    #[test]
    fn benchmark_results_allow_strong_live_scrapling_runtime_without_replay_lineage() {
        let store = TestStore::new();
        let summary = summarize_with_store(&store, 24, 10);
        let mut snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_398,
            &summary,
            &[
                OperatorSnapshotRecentSimRun {
                    run_id: "simrun-scrapling-prior".to_string(),
                    lane: "scrapling_traffic".to_string(),
                    profile: "scrapling_runtime_lane".to_string(),
                    observed_fulfillment_modes: vec!["crawler".to_string()],
                    observed_category_ids: vec!["indexing_bot".to_string()],
                    first_ts: 1_700_000_300,
                    last_ts: 1_700_000_340,
                    monitoring_event_count: 6,
                    defense_delta_count: 0,
                    ban_outcome_count: 0,
                    owned_surface_coverage: Some(
                        crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                            overall_status: "covered".to_string(),
                            canonical_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            surface_labels: std::collections::BTreeMap::from([
                                (
                                    "public_path_traversal".to_string(),
                                    "Public Path Traversal".to_string(),
                                ),
                                (
                                    "challenge_routing".to_string(),
                                    "Challenge Routing".to_string(),
                                ),
                            ]),
                            required_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            satisfied_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            blocking_surface_ids: Vec::new(),
                            receipts: vec![
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "public_path_traversal".to_string(),
                                    success_contract: "should_pass_some".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/".to_string(),
                                    sample_response_status: Some(200),
                                },
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "challenge_routing".to_string(),
                                    success_contract: "mixed_outcomes".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/challenge".to_string(),
                                    sample_response_status: Some(200),
                                },
                            ],
                        },
                    ),
                    llm_runtime_summary: None,
                },
                OperatorSnapshotRecentSimRun {
                    run_id: "simrun-scrapling-current".to_string(),
                    lane: "scrapling_traffic".to_string(),
                    profile: "scrapling_runtime_lane".to_string(),
                    observed_fulfillment_modes: vec![
                        "crawler".to_string(),
                        "bulk_scraper".to_string(),
                    ],
                    observed_category_ids: vec![
                        "indexing_bot".to_string(),
                        "ai_scraper_bot".to_string(),
                    ],
                    first_ts: 1_700_000_350,
                    last_ts: 1_700_000_397,
                    monitoring_event_count: 8,
                    defense_delta_count: 0,
                    ban_outcome_count: 0,
                    owned_surface_coverage: Some(
                        crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                            overall_status: "covered".to_string(),
                            canonical_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            surface_labels: std::collections::BTreeMap::from([
                                (
                                    "public_path_traversal".to_string(),
                                    "Public Path Traversal".to_string(),
                                ),
                                (
                                    "challenge_routing".to_string(),
                                    "Challenge Routing".to_string(),
                                ),
                            ]),
                            required_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            satisfied_surface_ids: vec![
                                "public_path_traversal".to_string(),
                                "challenge_routing".to_string(),
                            ],
                            blocking_surface_ids: Vec::new(),
                            receipts: vec![
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "public_path_traversal".to_string(),
                                    success_contract: "should_pass_some".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/".to_string(),
                                    sample_response_status: Some(200),
                                },
                                crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                    surface_id: "challenge_routing".to_string(),
                                    success_contract: "mixed_outcomes".to_string(),
                                    dependency_kind: "independent".to_string(),
                                    dependency_surface_ids: Vec::new(),
                                    coverage_status: "pass_observed".to_string(),
                                    surface_state: "satisfied".to_string(),
                                    satisfied: true,
                                    blocked_by_surface_ids: Vec::new(),
                                    attempt_count: 2,
                                    sample_request_method: "GET".to_string(),
                                    sample_request_path: "/challenge".to_string(),
                                    sample_response_status: Some(200),
                                },
                            ],
                        },
                    ),
                    llm_runtime_summary: None,
                },
            ],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_398,
            1_700_000_398,
            1_700_000_398,
        );
        snapshot.non_human_traffic = covered_non_human_summary();
        snapshot.non_human_traffic.restriction_readiness.status = "partial".to_string();
        snapshot.non_human_traffic.restriction_readiness.blockers = vec![
            "insufficient_category_evidence".to_string(),
            "degraded_category_receipts_present".to_string(),
        ];
        snapshot.non_human_traffic.recognition_evaluation.readiness =
            snapshot.non_human_traffic.restriction_readiness.clone();

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_398,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &ReplayPromotionSummary::not_materialized(),
            None,
        );

        assert_eq!(payload.escalation_hint.evidence_quality.status, "high_confidence");
        assert_eq!(
            payload
                .escalation_hint
                .evidence_quality
                .recent_window_support_status,
            "reproduced_recently"
        );
        assert_eq!(payload.tuning_eligibility.status, "eligible");
        assert!(!payload
            .tuning_eligibility
            .blockers
            .contains(&"protected_lineage_missing".to_string()));
        assert!(!payload
            .tuning_eligibility
            .blockers
            .contains(&"protected_tuning_evidence_not_ready".to_string()));
        assert_eq!(payload.escalation_hint.decision, "config_tuning_candidate");
    }

    #[test]
    fn benchmark_results_materialize_critical_urgency_when_exploit_progress_regresses() {
        let store = TestStore::new();
        let summary = summarize_with_store(&store, 24, 10);
        let mut baseline_snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_395,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-scrapling-baseline".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec!["crawler".to_string()],
                observed_category_ids: vec!["indexing_bot".to_string()],
                first_ts: 1_700_000_300,
                last_ts: 1_700_000_390,
                monitoring_event_count: 4,
                defense_delta_count: 0,
                ban_outcome_count: 0,
                owned_surface_coverage: Some(
                    crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                        overall_status: "partial".to_string(),
                        canonical_surface_ids: vec!["public_path_traversal".to_string()],
                        surface_labels: std::collections::BTreeMap::from([(
                            "public_path_traversal".to_string(),
                            "Public Path Traversal".to_string(),
                        )]),
                        required_surface_ids: vec!["public_path_traversal".to_string()],
                        satisfied_surface_ids: Vec::new(),
                        blocking_surface_ids: vec!["public_path_traversal".to_string()],
                        receipts: vec![
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "public_path_traversal".to_string(),
                                success_contract: "should_pass_some".to_string(),
                                dependency_kind: "independent".to_string(),
                                dependency_surface_ids: Vec::new(),
                                coverage_status: "fail_observed".to_string(),
                                surface_state: "attempted_blocked".to_string(),
                                satisfied: false,
                                blocked_by_surface_ids: Vec::new(),
                                attempt_count: 1,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/".to_string(),
                                sample_response_status: Some(429),
                            },
                        ],
                    },
                ),
                llm_runtime_summary: None,
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_395,
            1_700_000_395,
            1_700_000_395,
        );
        baseline_snapshot.non_human_traffic = covered_non_human_summary();
        let baseline_payload = build_benchmark_results_from_snapshot_sections(
            baseline_snapshot.generated_at,
            1_700_000_395,
            &baseline_snapshot.window,
            &baseline_snapshot.objectives,
            &baseline_snapshot.live_traffic,
            &baseline_snapshot.adversary_sim,
            &baseline_snapshot.non_human_traffic,
            &baseline_snapshot.budget_distance,
            &summary,
            &defaults(),
            &baseline_snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            None,
        );
        let prior_window_reference = comparable_snapshot_from_results(&baseline_payload);

        let mut current_snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_396,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-scrapling-current".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec![
                    "crawler".to_string(),
                    "bulk_scraper".to_string(),
                ],
                observed_category_ids: vec![
                    "indexing_bot".to_string(),
                    "ai_scraper_bot".to_string(),
                ],
                first_ts: 1_700_000_350,
                last_ts: 1_700_000_394,
                monitoring_event_count: 8,
                defense_delta_count: 0,
                ban_outcome_count: 0,
                owned_surface_coverage: Some(
                    crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                        overall_status: "covered".to_string(),
                        canonical_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        surface_labels: std::collections::BTreeMap::from([
                            (
                                "public_path_traversal".to_string(),
                                "Public Path Traversal".to_string(),
                            ),
                            (
                                "challenge_routing".to_string(),
                                "Challenge Routing".to_string(),
                            ),
                        ]),
                        required_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        satisfied_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        blocking_surface_ids: Vec::new(),
                        receipts: vec![
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "public_path_traversal".to_string(),
                                success_contract: "should_pass_some".to_string(),
                                dependency_kind: "independent".to_string(),
                                dependency_surface_ids: Vec::new(),
                                coverage_status: "pass_observed".to_string(),
                                surface_state: "satisfied".to_string(),
                                satisfied: true,
                                blocked_by_surface_ids: Vec::new(),
                                attempt_count: 2,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/".to_string(),
                                sample_response_status: Some(200),
                            },
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "challenge_routing".to_string(),
                                success_contract: "mixed_outcomes".to_string(),
                                dependency_kind: "independent".to_string(),
                                dependency_surface_ids: Vec::new(),
                                coverage_status: "pass_observed".to_string(),
                                surface_state: "satisfied".to_string(),
                                satisfied: true,
                                blocked_by_surface_ids: Vec::new(),
                                attempt_count: 1,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/challenge".to_string(),
                                sample_response_status: Some(200),
                            },
                        ],
                    },
                ),
                llm_runtime_summary: None,
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_396,
            1_700_000_396,
            1_700_000_396,
        );
        current_snapshot.non_human_traffic = covered_non_human_summary();

        let payload = build_benchmark_results_from_snapshot_sections(
            current_snapshot.generated_at,
            1_700_000_396,
            &current_snapshot.window,
            &current_snapshot.objectives,
            &current_snapshot.live_traffic,
            &current_snapshot.adversary_sim,
            &current_snapshot.non_human_traffic,
            &current_snapshot.budget_distance,
            &summary,
            &defaults(),
            &current_snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            Some(&prior_window_reference),
        );

        assert_eq!(payload.urgency.status, "critical");
        assert_eq!(payload.urgency.exploit_short_window_status, "outside_budget");
        assert_eq!(payload.urgency.exploit_long_window_status, "regressed");
        assert_eq!(payload.urgency.homeostasis_break_status, "triggered");
        assert!(payload
            .urgency
            .homeostasis_break_reasons
            .contains(&"exploit_success_regressed".to_string()));
    }

    #[test]
    fn benchmark_results_surface_recent_run_only_scrapling_category_coverage_as_stale() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::VerifiedBot,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphVerifiedIdentityTranche,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_360,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-request-native".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec![
                    "crawler".to_string(),
                    "bulk_scraper".to_string(),
                    "http_agent".to_string(),
                ],
                observed_category_ids: vec![
                    "indexing_bot".to_string(),
                    "ai_scraper_bot".to_string(),
                    "http_agent".to_string(),
                ],
                first_ts: 1_700_000_300,
                last_ts: 1_700_000_350,
                monitoring_event_count: 9,
                defense_delta_count: 2,
                ban_outcome_count: 0,
                owned_surface_coverage: None,
                llm_runtime_summary: None,
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_360,
            1_700_000_360,
            1_700_000_360,
        );

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_360,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &ReplayPromotionSummary::not_materialized(),
            None,
        );

        assert_eq!(payload.non_human_classification.status, "partial");
        assert_eq!(payload.non_human_coverage.stale_category_count, 3);
        assert_eq!(payload.non_human_coverage.overall_status, "stale");
        assert_eq!(payload.tuning_eligibility.status, "blocked");
        assert!(payload
            .tuning_eligibility
            .blockers
            .contains(&"non_human_classification_not_ready".to_string()));
    }

    #[test]
    fn category_posture_family_tracks_alignment_against_persisted_operator_postures() {
        let store = TestStore::new();
        for _ in 0..2 {
            record_request_outcome(
                &store,
                &RenderedRequestOutcome {
                    traffic_origin: TrafficOrigin::Live,
                    measurement_scope: MeasurementScope::IngressPrimary,
                    route_action_family: RouteActionFamily::PublicContent,
                    execution_mode: ExecutionMode::Enforced,
                    traffic_lane: Some(RequestOutcomeLane {
                        lane: TrafficLane::VerifiedBot,
                        exactness:
                            crate::observability::hot_read_contract::TelemetryExactness::Exact,
                        basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                    }),
                non_human_category: None,
                    outcome_class: RequestOutcomeClass::Forwarded,
                    response_kind: ResponseKind::ForwardAllow,
                    http_status: 200,
                    response_bytes: 120,
                    forwarded_upstream_latency_ms: None,
                    forward_attempted: true,
                    forward_failure_class: None,
                    intended_action: None,
                    policy_source: PolicySource::PolicyGraphVerifiedIdentityTranche,
                },
            );
        }
        for _ in 0..3 {
            record_request_outcome(
                &store,
                &RenderedRequestOutcome {
                    traffic_origin: TrafficOrigin::AdversarySim,
                    measurement_scope: MeasurementScope::IngressPrimary,
                    route_action_family: RouteActionFamily::PublicContent,
                    execution_mode: ExecutionMode::Enforced,
                    traffic_lane: Some(RequestOutcomeLane {
                        lane: TrafficLane::DeclaredCrawler,
                        exactness:
                            crate::observability::hot_read_contract::TelemetryExactness::Exact,
                        basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                    }),
                non_human_category: None,
                    outcome_class: RequestOutcomeClass::ShortCircuited,
                    response_kind: ResponseKind::NotABot,
                    http_status: 200,
                    response_bytes: 45,
                    forwarded_upstream_latency_ms: None,
                    forward_attempted: false,
                    forward_failure_class: None,
                    intended_action: None,
                    policy_source: PolicySource::PolicyGraphSecondTranche,
                },
            );
        }
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::DeclaredCrawler,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );

        let summary = summarize_with_store(&store, 24, 10);
        let snapshot = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_450,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_450,
            1_700_000_450,
            1_700_000_450,
        );

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_450,
            &snapshot.window,
            &snapshot.objectives,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.non_human_traffic,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            &protected_replay_promotion_summary(),
            None,
        );

        let family = payload
            .families
            .iter()
            .find(|family| family.family_id == "non_human_category_posture")
            .expect("category posture family");

        let beneficial = family
            .metrics
            .iter()
            .find(|metric| metric.metric_id == "category_posture_alignment:verified_beneficial_bot")
            .expect("verified beneficial posture metric");
        assert_eq!(beneficial.status, "outside_budget");
        assert_eq!(beneficial.current, Some(0.0));
        assert_eq!(beneficial.target, Some(1.0));

        let indexing = family
            .metrics
            .iter()
            .find(|metric| metric.metric_id == "category_posture_alignment:indexing_bot")
            .expect("indexing posture metric");
        assert_eq!(indexing.status, "near_limit");
        assert_eq!(indexing.current, Some(0.75));
        assert_eq!(indexing.target, Some(1.0));
    }
}
