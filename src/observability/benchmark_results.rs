use serde::{Deserialize, Serialize};

use crate::config::AllowedActionsSurface;
use crate::config::Config;
use crate::observability::benchmark_suite::BENCHMARK_SUITE_SCHEMA_VERSION;
use crate::observability::non_human_classification::NonHumanClassificationReadiness;
use crate::observability::non_human_coverage::NonHumanCoverageSummary;
use crate::observability::operator_snapshot::{
    OperatorBudgetDistanceSummary, OperatorSnapshotAdversarySim, OperatorSnapshotLiveTraffic,
    OperatorSnapshotNonHumanTrafficSummary, OperatorSnapshotWindow, ReplayPromotionSummary,
};
use crate::observability::operator_snapshot_objectives::OperatorObjectivesProfile;
use super::benchmark_adversary_effectiveness::representative_adversary_effectiveness_family;
use super::benchmark_beneficial_non_human::beneficial_non_human_posture_family;
use super::benchmark_non_human_categories::non_human_category_posture_family;
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
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkTuningEligibility {
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<String>,
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
    pub families: Vec<BenchmarkFamilyResult>,
    pub escalation_hint: BenchmarkEscalationHint,
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
    let suspicious_family =
        suspicious_origin_cost_family(live_traffic.suspicious_automation.as_ref(), budget_distance);
    let friction_family = likely_human_friction_family(budget_distance);
    let adversary_family = representative_adversary_effectiveness_family(adversary_sim);
    let verified_identity =
        super::operator_snapshot_verified_identity::verified_identity_summary(
            summary,
            cfg,
            non_human_traffic.receipts.as_slice(),
        );
    let non_human_family = beneficial_non_human_posture_family(
        summary,
        cfg,
        objectives,
        non_human_traffic,
        &verified_identity,
    );
    let category_posture_family = non_human_category_posture_family(objectives, non_human_traffic);
    let mut families = vec![
        suspicious_family,
        friction_family,
        adversary_family,
        non_human_family,
        category_posture_family,
    ];
    let (baseline_reference, improvement_status) = apply_prior_window_comparison(
        generated_at,
        families.as_mut_slice(),
        prior_window_reference,
    );
    let tuning_eligibility =
        tuning_eligibility(non_human_traffic, replay_promotion, &verified_identity, families.as_slice());
    let derived_escalation_hint = derive_escalation_hint(allowed_actions, families.as_slice());
    let escalation_hint = if tuning_eligibility.status != "eligible" {
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
            note: "Current benchmark pressure cannot justify tuning because category-aware protected evidence is not yet eligible for controller-grade judgment."
                .to_string(),
        }
    } else {
        derived_escalation_hint
    };

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
        non_human_classification: non_human_traffic.readiness.clone(),
        non_human_coverage: non_human_traffic.coverage.compact_for_benchmark(),
        tuning_eligibility,
        escalation_hint,
        replay_promotion: replay_promotion.clone(),
        families,
    }
}

fn tuning_eligibility(
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
    replay_promotion: &ReplayPromotionSummary,
    verified_identity: &OperatorSnapshotVerifiedIdentitySummary,
    families: &[BenchmarkFamilyResult],
) -> BenchmarkTuningEligibility {
    let mut blockers = if non_human_traffic.readiness.status != "ready" {
        let mut blockers = vec!["non_human_classification_not_ready".to_string()];
        blockers.extend(non_human_traffic.readiness.blockers.iter().cloned());
        blockers
    } else {
        non_human_traffic
            .coverage
            .protected_tuning_blockers(replay_promotion)
    };
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
        assert!(hint
            .blockers
            .contains(&"no_matching_config_surface".to_string()));
        assert!(hint.blockers.contains(&"family_capability_gap".to_string()));
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
                },
            ],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_200,
            1_700_000_200,
            1_700_000_200,
        );
        let mut cfg = defaults().clone();
        cfg.verified_identity.enabled = true;
        cfg.verified_identity.non_human_traffic_stance =
            crate::bot_identity::policy::NonHumanTrafficStance::AllowOnlyExplicitVerifiedIdentities;

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
    fn benchmark_results_materialize_host_impact_metrics_in_suspicious_origin_cost_family() {
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

        assert_eq!(latency_share.status, "outside_budget");
        assert!((latency_share.current.expect("latency share current") - 0.7).abs() < 0.000_001);
        assert_eq!(average_latency.status, "tracking_only");
        assert!((average_latency.current.expect("average latency current") - 70.0).abs() < 0.000_001);
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
        let objectives =
            crate::observability::operator_snapshot_objectives::default_operator_objectives(
                1_700_000_500,
            );

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
                decision_chain: Vec::new(),
                receipts: vec![crate::observability::non_human_classification::NonHumanClassificationReceipt {
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
            .contains(&"non_human_category_coverage_not_ready".to_string()));
        assert!(payload
            .escalation_hint
            .blockers
            .contains(&"mapped_categories_have_unavailable_coverage".to_string()));
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
    fn benchmark_results_surface_scrapling_request_native_category_coverage() {
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

        assert_eq!(payload.non_human_classification.status, "ready");
        assert_eq!(payload.non_human_coverage.covered_category_count, 3);
        assert_eq!(payload.non_human_coverage.overall_status, "partial");
        assert_eq!(payload.tuning_eligibility.status, "blocked");
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
        assert_eq!(beneficial.status, "inside_budget");
        assert_eq!(beneficial.current, Some(1.0));
        assert_eq!(beneficial.target, Some(1.0));

        let indexing = family
            .metrics
            .iter()
            .find(|metric| metric.metric_id == "category_posture_alignment:indexing_bot")
            .expect("indexing posture metric");
        assert_eq!(indexing.status, "inside_budget");
        assert_eq!(indexing.current, Some(0.75));
        assert_eq!(indexing.target, Some(0.5));
    }
}
