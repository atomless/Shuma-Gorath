use serde::{Deserialize, Serialize};

use crate::config::AllowedActionsSurface;
use crate::observability::benchmark_suite::BENCHMARK_SUITE_SCHEMA_VERSION;
use crate::config::Config;
use crate::observability::operator_snapshot::{
    OperatorBudgetDistanceSummary, OperatorSnapshotAdversarySim, OperatorSnapshotLiveTraffic,
    OperatorSnapshotWindow,
};
use super::benchmark_adversary_effectiveness::representative_adversary_effectiveness_family;
use super::benchmark_beneficial_non_human::beneficial_non_human_posture_family;
use super::benchmark_comparison::{
    apply_prior_window_comparison, BenchmarkComparableSnapshot,
};
use super::benchmark_results_comparison::{
    derive_escalation_hint, overall_coverage_status, overall_status,
};
use super::benchmark_results_families::{
    likely_human_friction_family, suspicious_origin_cost_family,
};

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
pub(crate) struct BenchmarkEscalationHint {
    pub availability: String,
    pub decision: String,
    pub review_status: String,
    pub trigger_family_ids: Vec<String>,
    pub candidate_action_families: Vec<String>,
    pub blockers: Vec<String>,
    pub note: String,
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
    pub families: Vec<BenchmarkFamilyResult>,
    pub escalation_hint: BenchmarkEscalationHint,
}

pub(crate) fn build_benchmark_results_from_snapshot_sections(
    generated_at: u64,
    input_snapshot_generated_at: u64,
    watch_window: &OperatorSnapshotWindow,
    live_traffic: &OperatorSnapshotLiveTraffic,
    adversary_sim: &OperatorSnapshotAdversarySim,
    budget_distance: &OperatorBudgetDistanceSummary,
    summary: &crate::observability::monitoring::MonitoringSummary,
    cfg: &Config,
    allowed_actions: &AllowedActionsSurface,
    prior_window_reference: Option<&BenchmarkComparableSnapshot>,
) -> BenchmarkResultsPayload {
    let suspicious_family =
        suspicious_origin_cost_family(live_traffic.suspicious_automation.as_ref(), budget_distance);
    let friction_family = likely_human_friction_family(budget_distance);
    let adversary_family = representative_adversary_effectiveness_family(adversary_sim);
    let non_human_family = beneficial_non_human_posture_family(summary, cfg);
    let mut families = vec![
        suspicious_family,
        friction_family,
        adversary_family,
        non_human_family,
    ];
    let (baseline_reference, improvement_status) = apply_prior_window_comparison(
        generated_at,
        families.as_mut_slice(),
        prior_window_reference,
    );

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
        escalation_hint: derive_escalation_hint(allowed_actions, families.as_slice()),
        families,
    }
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
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
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
                first_ts: 1_700_000_000,
                last_ts: 1_700_000_100,
                monitoring_event_count: 3,
                defense_delta_count: 2,
                ban_outcome_count: 0,
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
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
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
        assert_eq!(payload.escalation_hint.availability, "partial_support");
        assert_eq!(payload.escalation_hint.decision, "config_tuning_candidate");
        assert_eq!(
            payload.escalation_hint.review_status,
            "manual_review_required"
        );
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
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
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

        let payload = build_benchmark_results_from_snapshot_sections(
            snapshot.generated_at,
            1_700_000_100,
            &snapshot.window,
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.budget_distance,
            &summary,
            &defaults(),
            &snapshot.allowed_actions,
            None,
        );
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
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 512,
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
                provenance: crate::bot_identity::contracts::IdentityProvenance::Provider,
                result_status:
                    crate::bot_identity::verification::IdentityVerificationResultStatus::Verified,
                failure: None,
                freshness: crate::bot_identity::verification::IdentityVerificationFreshness::Fresh,
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
                    first_ts: 1_700_000_120,
                    last_ts: 1_700_000_140,
                    monitoring_event_count: 4,
                    defense_delta_count: 2,
                    ban_outcome_count: 1,
                },
                OperatorSnapshotRecentSimRun {
                    run_id: "run_002".to_string(),
                    lane: "synthetic_traffic".to_string(),
                    profile: "abuse_regression".to_string(),
                    first_ts: 1_700_000_150,
                    last_ts: 1_700_000_190,
                    monitoring_event_count: 6,
                    defense_delta_count: 3,
                    ban_outcome_count: 1,
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
            &snapshot.live_traffic,
            &snapshot.adversary_sim,
            &snapshot.budget_distance,
            &summary,
            &cfg,
            &snapshot.allowed_actions,
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
    }
}
