use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::observability::benchmark_suite::BENCHMARK_SUITE_SCHEMA_VERSION;
use crate::observability::operator_snapshot::{
    OperatorBudgetDistanceRow, OperatorSnapshotHotReadPayload, OperatorSnapshotLane,
    OperatorSnapshotWindow,
};

pub(crate) const BENCHMARK_RESULTS_SCHEMA_VERSION: &str = "benchmark_results_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchmarkBaselineReference {
    pub reference_kind: String,
    pub status: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BenchmarkFamilyResult {
    pub family_id: String,
    pub status: String,
    pub capability_gate: String,
    pub note: String,
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

pub(crate) fn build_benchmark_results_payload(
    input_snapshot_generated_at: u64,
    snapshot: &OperatorSnapshotHotReadPayload,
) -> BenchmarkResultsPayload {
    let suspicious_family = suspicious_origin_cost_family(snapshot);
    let friction_family = likely_human_friction_family(snapshot);
    let adversary_family = representative_adversary_effectiveness_family(snapshot);
    let non_human_family = beneficial_non_human_posture_family();
    let families = vec![
        suspicious_family,
        friction_family,
        adversary_family,
        non_human_family,
    ];

    BenchmarkResultsPayload {
        schema_version: BENCHMARK_RESULTS_SCHEMA_VERSION.to_string(),
        suite_version: BENCHMARK_SUITE_SCHEMA_VERSION.to_string(),
        generated_at: snapshot.generated_at,
        input_snapshot_generated_at,
        subject_kind: "current_instance".to_string(),
        watch_window: snapshot.window.clone(),
        baseline_reference: BenchmarkBaselineReference {
            reference_kind: "prior_window".to_string(),
            status: "not_available".to_string(),
            note: "Baseline comparison materializes with benchmark-result history.".to_string(),
        },
        coverage_status: overall_coverage_status(families.as_slice()),
        overall_status: overall_status(families.as_slice()),
        improvement_status: "not_available".to_string(),
        escalation_hint: derive_escalation_hint(snapshot, families.as_slice()),
        families,
    }
}

fn suspicious_origin_cost_family(snapshot: &OperatorSnapshotHotReadPayload) -> BenchmarkFamilyResult {
    let lane = snapshot.live_traffic.suspicious_automation.as_ref();
    let request_budget =
        budget_row(snapshot, "suspicious_forwarded_request_rate");
    let byte_budget = budget_row(snapshot, "suspicious_forwarded_byte_rate");
    let metrics = vec![
        budget_metric_result(
            "suspicious_forwarded_request_rate",
            request_budget,
            "supported",
        ),
        budget_metric_result("suspicious_forwarded_byte_rate", byte_budget, "supported"),
        tracking_ratio_metric(
            "suspicious_short_circuit_rate",
            lane,
            lane.map(|row| ratio(row.short_circuited_requests, row.total_requests)),
        ),
        tracking_ratio_metric(
            "suspicious_locally_served_byte_share",
            lane,
            lane.map(|row| {
                let total_bytes = row
                    .forwarded_response_bytes
                    .saturating_add(row.shuma_served_response_bytes);
                ratio(row.shuma_served_response_bytes, total_bytes)
            }),
        ),
    ];
    BenchmarkFamilyResult {
        family_id: "suspicious_origin_cost".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: "supported".to_string(),
        note: "Derived from the live suspicious-automation lane and current budget-distance rows."
            .to_string(),
        metrics,
    }
}

fn likely_human_friction_family(snapshot: &OperatorSnapshotHotReadPayload) -> BenchmarkFamilyResult {
    let friction_budget = budget_row(snapshot, "likely_human_friction_rate");
    let metrics = vec![
        budget_metric_result("likely_human_friction_rate", friction_budget, "supported"),
        unsupported_metric("interactive_friction_rate"),
        unsupported_metric("likely_human_hard_block_rate"),
    ];
    BenchmarkFamilyResult {
        family_id: "likely_human_friction".to_string(),
        status: aggregate_budget_status(metrics.as_slice()),
        capability_gate: "partially_supported".to_string(),
        note: "Current results are budgeted on observed likely-human friction while interactive and hard-block breakdowns remain to be materialized.".to_string(),
        metrics,
    }
}

fn representative_adversary_effectiveness_family(
    snapshot: &OperatorSnapshotHotReadPayload,
) -> BenchmarkFamilyResult {
    let recent_run_count = snapshot.adversary_sim.recent_runs.len();
    BenchmarkFamilyResult {
        family_id: "representative_adversary_effectiveness".to_string(),
        status: "not_yet_supported".to_string(),
        capability_gate: "not_yet_supported".to_string(),
        note: format!(
            "Recent adversary-sim runs are visible (count={recent_run_count}), but scenario-family benchmark mapping and result deltas are not materialized yet."
        ),
        metrics: vec![
            unsupported_metric("scenario_goal_success_rate"),
            unsupported_metric("scenario_origin_reach_rate"),
            unsupported_metric("scenario_escalation_rate"),
            unsupported_metric("scenario_regression_status"),
        ],
    }
}

fn beneficial_non_human_posture_family() -> BenchmarkFamilyResult {
    BenchmarkFamilyResult {
        family_id: "beneficial_non_human_posture".to_string(),
        status: "not_yet_supported".to_string(),
        capability_gate: "not_yet_supported".to_string(),
        note: "Beneficial or authenticated non-human benchmarking waits for verified-identity and stance-aware allowance telemetry.".to_string(),
        metrics: vec![
            unsupported_metric("allowed_as_intended_rate"),
            unsupported_metric("friction_mismatch_rate"),
            unsupported_metric("deny_mismatch_rate"),
            unsupported_metric("coverage_status"),
        ],
    }
}

fn budget_row<'a>(
    snapshot: &'a OperatorSnapshotHotReadPayload,
    metric: &str,
) -> Option<&'a OperatorBudgetDistanceRow> {
    snapshot
        .budget_distance
        .rows
        .iter()
        .find(|row| row.metric == metric)
}

fn budget_metric_result(
    metric_id: &str,
    row: Option<&OperatorBudgetDistanceRow>,
    capability_gate: &str,
) -> BenchmarkMetricResult {
    match row {
        Some(value) => BenchmarkMetricResult {
            metric_id: value.metric.clone(),
            status: value.status.clone(),
            current: Some(value.current),
            target: Some(value.target),
            delta: Some(value.delta),
            exactness: value.exactness.clone(),
            basis: value.basis.clone(),
            capability_gate: capability_gate.to_string(),
        },
        None => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "insufficient_evidence".to_string(),
            current: None,
            target: None,
            delta: None,
            exactness: "derived".to_string(),
            basis: "mixed".to_string(),
            capability_gate: capability_gate.to_string(),
        },
    }
}

fn tracking_ratio_metric(
    metric_id: &str,
    lane: Option<&OperatorSnapshotLane>,
    current: Option<f64>,
) -> BenchmarkMetricResult {
    match (lane, current) {
        (Some(value), Some(current)) if value.total_requests > 0 => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "tracking_only".to_string(),
            current: Some(current),
            target: None,
            delta: None,
            exactness: value.exactness.clone(),
            basis: value.basis.clone(),
            capability_gate: "supported".to_string(),
        },
        (Some(value), _) => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "insufficient_evidence".to_string(),
            current: None,
            target: None,
            delta: None,
            exactness: value.exactness.clone(),
            basis: value.basis.clone(),
            capability_gate: "supported".to_string(),
        },
        (None, _) => BenchmarkMetricResult {
            metric_id: metric_id.to_string(),
            status: "insufficient_evidence".to_string(),
            current: None,
            target: None,
            delta: None,
            exactness: "derived".to_string(),
            basis: "observed".to_string(),
            capability_gate: "supported".to_string(),
        },
    }
}

fn unsupported_metric(metric_id: &str) -> BenchmarkMetricResult {
    BenchmarkMetricResult {
        metric_id: metric_id.to_string(),
        status: "not_yet_supported".to_string(),
        current: None,
        target: None,
        delta: None,
        exactness: "derived".to_string(),
        basis: "mixed".to_string(),
        capability_gate: "not_yet_supported".to_string(),
    }
}

fn aggregate_budget_status(metrics: &[BenchmarkMetricResult]) -> String {
    let budget_statuses: Vec<&str> = metrics
        .iter()
        .filter(|metric| matches!(
            metric.status.as_str(),
            "outside_budget" | "near_limit" | "inside_budget" | "insufficient_evidence"
        ))
        .map(|metric| metric.status.as_str())
        .collect();
    if budget_statuses.iter().any(|status| *status == "outside_budget") {
        "outside_budget".to_string()
    } else if budget_statuses.iter().any(|status| *status == "near_limit") {
        "near_limit".to_string()
    } else if budget_statuses.iter().any(|status| *status == "inside_budget") {
        "inside_budget".to_string()
    } else if budget_statuses
        .iter()
        .any(|status| *status == "insufficient_evidence")
    {
        "insufficient_evidence".to_string()
    } else {
        "not_yet_supported".to_string()
    }
}

fn overall_coverage_status(families: &[BenchmarkFamilyResult]) -> String {
    if families
        .iter()
        .all(|family| family.capability_gate == "supported")
    {
        "supported".to_string()
    } else if families
        .iter()
        .any(|family| family.capability_gate == "supported")
    {
        "partial_support".to_string()
    } else {
        "not_yet_supported".to_string()
    }
}

fn overall_status(families: &[BenchmarkFamilyResult]) -> String {
    if families.iter().any(|family| family.status == "outside_budget") {
        "outside_budget".to_string()
    } else if families.iter().any(|family| family.status == "near_limit") {
        "near_limit".to_string()
    } else if families.iter().any(|family| family.status == "inside_budget") {
        "inside_budget".to_string()
    } else if families
        .iter()
        .any(|family| family.status == "insufficient_evidence")
    {
        "insufficient_evidence".to_string()
    } else {
        "not_yet_supported".to_string()
    }
}

fn derive_escalation_hint(
    snapshot: &OperatorSnapshotHotReadPayload,
    families: &[BenchmarkFamilyResult],
) -> BenchmarkEscalationHint {
    let outside_budget_families: Vec<&BenchmarkFamilyResult> = families
        .iter()
        .filter(|family| family.status == "outside_budget")
        .collect();
    let near_limit_families: Vec<&BenchmarkFamilyResult> = families
        .iter()
        .filter(|family| family.status == "near_limit")
        .collect();
    let insufficient_families: Vec<&BenchmarkFamilyResult> = families
        .iter()
        .filter(|family| family.status == "insufficient_evidence")
        .collect();

    let availability = "partial_support".to_string();
    let review_status = "manual_review_required".to_string();

    if outside_budget_families.is_empty() {
        let mut blockers = Vec::new();
        let trigger_family_ids = if !near_limit_families.is_empty() {
            blockers.push("near_limit_only".to_string());
            family_ids(&near_limit_families)
        } else if !insufficient_families.is_empty() {
            blockers.push("insufficient_evidence".to_string());
            family_ids(&insufficient_families)
        } else {
            blockers.push("outside_budget_not_observed".to_string());
            Vec::new()
        };
        return BenchmarkEscalationHint {
            availability,
            decision: "observe_longer".to_string(),
            review_status,
            trigger_family_ids,
            candidate_action_families: Vec::new(),
            blockers,
            note:
                "Current benchmark evidence does not yet justify config or code escalation; keep observing additional windows."
                    .to_string(),
        };
    }

    let trigger_family_ids = family_ids(&outside_budget_families);
    let mut candidate_action_families = BTreeSet::new();
    let mut blockers = BTreeSet::new();

    for family in outside_budget_families {
        if family.capability_gate == "not_yet_supported" {
            blockers.insert("family_capability_gap".to_string());
        }

        let mapped_families = benchmark_action_families(family.family_id.as_str());
        if mapped_families.is_empty() {
            blockers.insert("no_matching_config_surface".to_string());
            continue;
        }

        let matching_surface_families: Vec<_> = snapshot
            .allowed_actions
            .families
            .iter()
            .filter(|allowed_family| mapped_families.contains(&allowed_family.family.as_str()))
            .collect();

        if matching_surface_families.is_empty() {
            blockers.insert("no_matching_config_surface".to_string());
            continue;
        }

        let has_addressable_surface = matching_surface_families.iter().any(|allowed_family| {
            matches!(
                allowed_family.controller_status.as_str(),
                "allowed" | "manual_only"
            )
        });

        if has_addressable_surface {
            for allowed_family in matching_surface_families {
                if matches!(
                    allowed_family.controller_status.as_str(),
                    "allowed" | "manual_only"
                ) {
                    candidate_action_families.insert(allowed_family.family.clone());
                }
            }
        } else {
            blockers.insert("no_matching_config_surface".to_string());
        }
    }

    if blockers.is_empty() && !candidate_action_families.is_empty() {
        return BenchmarkEscalationHint {
            availability,
            decision: "config_tuning_candidate".to_string(),
            review_status,
            trigger_family_ids,
            candidate_action_families: candidate_action_families.into_iter().collect(),
            blockers: Vec::new(),
            note: "Current-window benchmark misses align with existing config surfaces; manual review remains required before proposing a tuning change."
                .to_string(),
        };
    }

    BenchmarkEscalationHint {
        availability,
        decision: "code_evolution_candidate".to_string(),
        review_status,
        trigger_family_ids,
        candidate_action_families: candidate_action_families.into_iter().collect(),
        blockers: blockers.into_iter().collect(),
        note: "At least one outside-budget benchmark family is not addressable through the current config surface or requires missing capability; manual review should consider code evolution."
            .to_string(),
    }
}

fn benchmark_action_families(family_id: &str) -> &'static [&'static str] {
    match family_id {
        "suspicious_origin_cost" => &[
            "geo_policy",
            "ip_range_policy",
            "honeypot",
            "maze_core",
            "tarpit",
            "proof_of_work",
            "challenge",
            "not_a_bot",
            "botness",
            "cdp_detection",
            "fingerprint_signal",
        ],
        "likely_human_friction" => &[
            "core_policy",
            "browser_policy",
            "proof_of_work",
            "challenge",
            "not_a_bot",
            "botness",
            "maze_core",
        ],
        _ => &[],
    }
}

fn family_ids(families: &[&BenchmarkFamilyResult]) -> Vec<String> {
    families
        .iter()
        .map(|family| family.family_id.clone())
        .collect()
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_benchmark_results_payload, derive_escalation_hint, BenchmarkFamilyResult,
        BenchmarkMetricResult,
    };
    use crate::config::allowed_actions_v1;
    use crate::challenge::KeyValueStore;
    use crate::observability::monitoring::{record_request_outcome, summarize_with_store};
    use crate::observability::operator_snapshot::{
        build_operator_snapshot_payload, OperatorSnapshotRecentChanges, OperatorSnapshotRecentSimRun,
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

        let payload = build_benchmark_results_payload(1_700_000_100, &snapshot);
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
        assert_eq!(payload.escalation_hint.review_status, "manual_review_required");
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

        let payload = build_benchmark_results_payload(1_700_000_100, &snapshot);
        assert_eq!(payload.escalation_hint.decision, "config_tuning_candidate");
        assert_eq!(payload.escalation_hint.review_status, "manual_review_required");
        assert!(
            payload
                .escalation_hint
                .trigger_family_ids
                .contains(&"likely_human_friction".to_string())
        );
        assert!(
            payload
                .escalation_hint
                .candidate_action_families
                .contains(&"challenge".to_string())
        );
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
            metrics: vec![BenchmarkMetricResult {
                metric_id: "allowed_as_intended_rate".to_string(),
                status: "not_yet_supported".to_string(),
                current: None,
                target: None,
                delta: None,
                exactness: "derived".to_string(),
                basis: "mixed".to_string(),
                capability_gate: "not_yet_supported".to_string(),
            }],
        }];

        let hint = derive_escalation_hint(&snapshot, families.as_slice());
        assert_eq!(hint.decision, "code_evolution_candidate");
        assert_eq!(hint.review_status, "manual_review_required");
        assert!(
            hint.blockers
                .contains(&"no_matching_config_surface".to_string())
        );
        assert!(
            hint.blockers
                .contains(&"family_capability_gap".to_string())
        );
    }
}
