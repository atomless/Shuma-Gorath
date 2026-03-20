use serde::{Deserialize, Serialize};

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
        families,
        escalation_hint: BenchmarkEscalationHint {
            availability: "not_yet_materialized".to_string(),
            decision: "observe_longer".to_string(),
            note: "Explicit config-versus-code escalation lands with the next benchmark tranche."
                .to_string(),
        },
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

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::build_benchmark_results_payload;
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
    }
}
