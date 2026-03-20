use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::challenge::KeyValueStore;
use crate::observability::hot_read_contract::{
    operator_snapshot_component_contracts, HotReadOwnershipTier, TelemetryBasis,
    TelemetryExactness,
};
use crate::observability::monitoring::{
    HumanFrictionSegmentRow, MonitoringSummary, RequestOutcomeLaneSummaryRow,
    RequestOutcomeScopeSummaryRow,
};

pub(crate) const OPERATOR_SNAPSHOT_SCHEMA_VERSION: &str = "operator_snapshot_v1";
const BACKEND_DEFAULT_OBJECTIVE_PROFILE_ID: &str = "backend_default_v1";
const DEFAULT_WINDOW_HOURS: u64 = 24;
const DEFAULT_RECENT_CHANGE_ROWS: usize = 6;
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
pub(crate) struct OperatorSnapshotPlaceholderSection {
    pub availability: String,
    pub note: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotWindow {
    pub start_ts: u64,
    pub end_ts: u64,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotSectionMetadata {
    pub exactness: TelemetryExactness,
    pub basis: TelemetryBasis,
    pub ownership_tier: HotReadOwnershipTier,
    pub refreshed_at_ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotLane {
    pub lane: String,
    pub exactness: String,
    pub basis: String,
    pub total_requests: u64,
    pub forwarded_requests: u64,
    pub short_circuited_requests: u64,
    pub control_response_requests: u64,
    pub forwarded_response_bytes: u64,
    pub shuma_served_response_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotHumanFriction {
    pub segment: String,
    pub denominator_requests: u64,
    pub friction_requests: u64,
    pub friction_rate: f64,
    pub not_a_bot_requests: u64,
    pub challenge_requests: u64,
    pub js_challenge_requests: u64,
    pub maze_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotLiveTraffic {
    pub traffic_origin: String,
    pub measurement_scope: String,
    pub execution_mode: String,
    pub total_requests: u64,
    pub forwarded_requests: u64,
    pub short_circuited_requests: u64,
    pub control_response_requests: u64,
    pub forwarded_response_bytes: u64,
    pub shuma_served_response_bytes: u64,
    pub likely_human: Option<OperatorSnapshotLane>,
    pub suspicious_automation: Option<OperatorSnapshotLane>,
    pub human_friction: Option<OperatorSnapshotHumanFriction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotShadowMode {
    pub enabled: bool,
    pub total_actions: u64,
    pub pass_through_total: u64,
    pub actions: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotRecentSimRun {
    pub run_id: String,
    pub lane: String,
    pub profile: String,
    pub first_ts: u64,
    pub last_ts: u64,
    pub monitoring_event_count: u64,
    pub defense_delta_count: u64,
    pub ban_outcome_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotAdversarySim {
    pub traffic_origin: String,
    pub measurement_scope: String,
    pub execution_mode: String,
    pub total_requests: u64,
    pub forwarded_requests: u64,
    pub short_circuited_requests: u64,
    pub control_response_requests: u64,
    pub forwarded_response_bytes: u64,
    pub shuma_served_response_bytes: u64,
    pub recent_runs: Vec<OperatorSnapshotRecentSimRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotRuntimePosture {
    pub shadow_mode: bool,
    pub fail_mode: String,
    pub runtime_environment: String,
    pub gateway_deployment_profile: String,
    pub adversary_sim_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotRecentChange {
    pub changed_at_ts: u64,
    pub change_reason: String,
    pub changed_families: Vec<String>,
    pub source: String,
    pub targets: Vec<String>,
    pub watch_window_status: String,
    pub watch_window_elapsed_seconds: u64,
    pub watch_window_remaining_seconds: u64,
    pub change_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct OperatorSnapshotRecentChanges {
    pub lookback_seconds: u64,
    pub watch_window_seconds: u64,
    pub rows: Vec<OperatorSnapshotRecentChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorBudgetDistanceRow {
    pub budget_id: String,
    pub metric: String,
    pub eligible_requests: u64,
    pub current: f64,
    pub target: f64,
    pub delta: f64,
    pub near_limit: f64,
    pub status: String,
    pub exactness: String,
    pub basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct OperatorBudgetDistanceSummary {
    pub rows: Vec<OperatorBudgetDistanceRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotHotReadPayload {
    pub schema_version: String,
    pub generated_at: u64,
    pub window: OperatorSnapshotWindow,
    pub section_metadata: BTreeMap<String, OperatorSnapshotSectionMetadata>,
    pub objectives: OperatorObjectivesProfile,
    pub live_traffic: OperatorSnapshotLiveTraffic,
    pub shadow_mode: OperatorSnapshotShadowMode,
    pub adversary_sim: OperatorSnapshotAdversarySim,
    pub runtime_posture: OperatorSnapshotRuntimePosture,
    pub recent_changes: OperatorSnapshotRecentChanges,
    pub budget_distance: OperatorBudgetDistanceSummary,
    pub allowed_actions: OperatorSnapshotPlaceholderSection,
    pub verified_identity: OperatorSnapshotPlaceholderSection,
}

pub(crate) fn operator_snapshot_watch_window_hours(summary_hours: u64) -> u64 {
    summary_hours.max(DEFAULT_WINDOW_HOURS)
}

pub(crate) fn operator_snapshot_recent_changes_limit() -> usize {
    DEFAULT_RECENT_CHANGE_ROWS
}

pub(crate) fn build_operator_snapshot_payload<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    summary: &MonitoringSummary,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
    recent_changes: OperatorSnapshotRecentChanges,
    summary_refreshed_at_ts: u64,
    recent_sim_runs_refreshed_at_ts: u64,
    recent_changes_refreshed_at_ts: u64,
) -> OperatorSnapshotHotReadPayload {
    let objectives = default_operator_objectives();
    let window_hours = operator_snapshot_watch_window_hours(summary.hours);
    let live_scope = scope_row(summary, "live", "ingress_primary", "enforced").cloned();
    let sim_scope = scope_row(summary, "adversary_sim", "ingress_primary", "enforced").cloned();
    let likely_human_lane =
        lane_row(summary, "live", "ingress_primary", "enforced", "likely_human").cloned();
    let suspicious_lane = lane_row(
        summary,
        "live",
        "ingress_primary",
        "enforced",
        "suspicious_automation",
    )
    .cloned();
    let human_friction = human_friction_row(summary, "enforced", "likely_human").cloned();

    OperatorSnapshotHotReadPayload {
        schema_version: OPERATOR_SNAPSHOT_SCHEMA_VERSION.to_string(),
        generated_at: generated_at_ts,
        window: snapshot_window(generated_at_ts, window_hours),
        section_metadata: operator_snapshot_section_metadata(
            generated_at_ts,
            summary_refreshed_at_ts,
            recent_sim_runs_refreshed_at_ts,
            recent_changes_refreshed_at_ts,
        ),
        objectives: objectives.clone(),
        live_traffic: live_traffic_section(
            live_scope.as_ref(),
            likely_human_lane.as_ref(),
            suspicious_lane.as_ref(),
            human_friction.as_ref(),
        ),
        shadow_mode: OperatorSnapshotShadowMode {
            enabled: runtime_shadow_mode(store, site_id),
            total_actions: summary.shadow.total_actions,
            pass_through_total: summary.shadow.pass_through_total,
            actions: summary.shadow.actions.clone(),
        },
        adversary_sim: adversary_sim_section(sim_scope.as_ref(), recent_sim_runs),
        runtime_posture: runtime_posture(store, site_id),
        recent_changes,
        budget_distance: budget_distance_summary(
            &objectives,
            likely_human_lane.as_ref(),
            suspicious_lane.as_ref(),
            human_friction.as_ref(),
        ),
        allowed_actions: placeholder_section(
            "not_yet_materialized",
            "Allowed controller action envelope lands in a later operator-snapshot slice.",
        ),
        verified_identity: placeholder_section(
            "not_yet_supported",
            "Verified identity summaries land with the verified bot identity foundation.",
        ),
    }
}

fn default_operator_objectives() -> OperatorObjectivesProfile {
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
                eligible_population:
                    "live:ingress_primary:enforced:suspicious_automation".to_string(),
            },
            OperatorObjectiveBudget {
                budget_id: "suspicious_forwarded_bytes".to_string(),
                metric: "suspicious_forwarded_byte_rate".to_string(),
                comparator: "max_ratio".to_string(),
                target: SUSPICIOUS_FORWARDED_BYTE_TARGET,
                near_limit_ratio: DEFAULT_NEAR_LIMIT_RATIO,
                eligible_population:
                    "live:ingress_primary:enforced:suspicious_automation".to_string(),
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

fn snapshot_window(generated_at_ts: u64, hours: u64) -> OperatorSnapshotWindow {
    let duration_seconds = hours.saturating_mul(3600);
    OperatorSnapshotWindow {
        start_ts: generated_at_ts.saturating_sub(duration_seconds.saturating_sub(1)),
        end_ts: generated_at_ts,
        duration_seconds,
    }
}

fn operator_snapshot_section_metadata(
    generated_at_ts: u64,
    summary_refreshed_at_ts: u64,
    recent_sim_runs_refreshed_at_ts: u64,
    recent_changes_refreshed_at_ts: u64,
) -> BTreeMap<String, OperatorSnapshotSectionMetadata> {
    operator_snapshot_component_contracts()
        .iter()
        .map(|component| {
            let refreshed_at_ts = match component.key {
                "live_traffic" | "shadow_mode" | "budget_distance" => summary_refreshed_at_ts,
                "adversary_sim" => recent_sim_runs_refreshed_at_ts,
                "recent_changes" => recent_changes_refreshed_at_ts,
                _ => generated_at_ts,
            };
            (
                component.key.to_string(),
                OperatorSnapshotSectionMetadata {
                    exactness: component.exactness,
                    basis: component.basis,
                    ownership_tier: component.ownership_tier,
                    refreshed_at_ts,
                },
            )
        })
        .collect()
}

fn scope_row<'a>(
    summary: &'a MonitoringSummary,
    origin: &str,
    scope: &str,
    execution_mode: &str,
) -> Option<&'a RequestOutcomeScopeSummaryRow> {
    summary.request_outcomes.by_scope.iter().find(|row| {
        row.traffic_origin == origin
            && row.measurement_scope == scope
            && row.execution_mode == execution_mode
    })
}

fn lane_row<'a>(
    summary: &'a MonitoringSummary,
    origin: &str,
    scope: &str,
    execution_mode: &str,
    lane: &str,
) -> Option<&'a RequestOutcomeLaneSummaryRow> {
    summary.request_outcomes.by_lane.iter().find(|row| {
        row.traffic_origin == origin
            && row.measurement_scope == scope
            && row.execution_mode == execution_mode
            && row.lane == lane
    })
}

fn human_friction_row<'a>(
    summary: &'a MonitoringSummary,
    execution_mode: &str,
    segment: &str,
) -> Option<&'a HumanFrictionSegmentRow> {
    summary
        .human_friction
        .segments
        .iter()
        .find(|row| row.execution_mode == execution_mode && row.segment == segment)
}

fn lane_snapshot(row: &RequestOutcomeLaneSummaryRow) -> OperatorSnapshotLane {
    OperatorSnapshotLane {
        lane: row.lane.clone(),
        exactness: row.exactness.clone(),
        basis: row.basis.clone(),
        total_requests: row.total_requests,
        forwarded_requests: row.forwarded_requests,
        short_circuited_requests: row.short_circuited_requests,
        control_response_requests: row.control_response_requests,
        forwarded_response_bytes: row.forwarded_response_bytes,
        shuma_served_response_bytes: row
            .short_circuited_response_bytes
            .saturating_add(row.control_response_bytes),
    }
}

fn human_friction_snapshot(row: &HumanFrictionSegmentRow) -> OperatorSnapshotHumanFriction {
    OperatorSnapshotHumanFriction {
        segment: row.segment.clone(),
        denominator_requests: row.denominator_requests,
        friction_requests: row.friction_requests,
        friction_rate: row.friction_rate,
        not_a_bot_requests: row.not_a_bot_requests,
        challenge_requests: row.challenge_requests,
        js_challenge_requests: row.js_challenge_requests,
        maze_requests: row.maze_requests,
    }
}

fn live_traffic_section(
    scope_row: Option<&RequestOutcomeScopeSummaryRow>,
    likely_human_lane: Option<&RequestOutcomeLaneSummaryRow>,
    suspicious_lane: Option<&RequestOutcomeLaneSummaryRow>,
    human_friction: Option<&HumanFrictionSegmentRow>,
) -> OperatorSnapshotLiveTraffic {
    let scope = scope_row.cloned().unwrap_or_default();
    OperatorSnapshotLiveTraffic {
        traffic_origin: if scope.traffic_origin.is_empty() {
            "live".to_string()
        } else {
            scope.traffic_origin
        },
        measurement_scope: if scope.measurement_scope.is_empty() {
            "ingress_primary".to_string()
        } else {
            scope.measurement_scope
        },
        execution_mode: if scope.execution_mode.is_empty() {
            "enforced".to_string()
        } else {
            scope.execution_mode
        },
        total_requests: scope.total_requests,
        forwarded_requests: scope.forwarded_requests,
        short_circuited_requests: scope.short_circuited_requests,
        control_response_requests: scope.control_response_requests,
        forwarded_response_bytes: scope.forwarded_response_bytes,
        shuma_served_response_bytes: scope
            .short_circuited_response_bytes
            .saturating_add(scope.control_response_bytes),
        likely_human: likely_human_lane.map(lane_snapshot),
        suspicious_automation: suspicious_lane.map(lane_snapshot),
        human_friction: human_friction.map(human_friction_snapshot),
    }
}

fn adversary_sim_section(
    scope_row: Option<&RequestOutcomeScopeSummaryRow>,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
) -> OperatorSnapshotAdversarySim {
    let scope = scope_row.cloned().unwrap_or_default();
    OperatorSnapshotAdversarySim {
        traffic_origin: if scope.traffic_origin.is_empty() {
            "adversary_sim".to_string()
        } else {
            scope.traffic_origin
        },
        measurement_scope: if scope.measurement_scope.is_empty() {
            "ingress_primary".to_string()
        } else {
            scope.measurement_scope
        },
        execution_mode: if scope.execution_mode.is_empty() {
            "enforced".to_string()
        } else {
            scope.execution_mode
        },
        total_requests: scope.total_requests,
        forwarded_requests: scope.forwarded_requests,
        short_circuited_requests: scope.short_circuited_requests,
        control_response_requests: scope.control_response_requests,
        forwarded_response_bytes: scope.forwarded_response_bytes,
        shuma_served_response_bytes: scope
            .short_circuited_response_bytes
            .saturating_add(scope.control_response_bytes),
        recent_runs: recent_sim_runs.to_vec(),
    }
}

fn runtime_shadow_mode<S: KeyValueStore>(store: &S, site_id: &str) -> bool {
    crate::config::load_runtime_cached(store, site_id)
        .map(|cfg| cfg.shadow_mode)
        .unwrap_or(false)
}

fn runtime_posture<S: KeyValueStore>(store: &S, site_id: &str) -> OperatorSnapshotRuntimePosture {
    OperatorSnapshotRuntimePosture {
        shadow_mode: runtime_shadow_mode(store, site_id),
        fail_mode: if crate::config::kv_store_fail_open() {
            "open".to_string()
        } else {
            "closed".to_string()
        },
        runtime_environment: crate::config::runtime_environment().as_str().to_string(),
        gateway_deployment_profile: crate::config::gateway_deployment_profile()
            .as_str()
            .to_string(),
        adversary_sim_available: crate::config::adversary_sim_available(),
    }
}

fn placeholder_section(availability: &str, note: &str) -> OperatorSnapshotPlaceholderSection {
    OperatorSnapshotPlaceholderSection {
        availability: availability.to_string(),
        note: note.to_string(),
    }
}

fn budget_distance_summary(
    objectives: &OperatorObjectivesProfile,
    likely_human_lane: Option<&RequestOutcomeLaneSummaryRow>,
    suspicious_lane: Option<&RequestOutcomeLaneSummaryRow>,
    human_friction: Option<&HumanFrictionSegmentRow>,
) -> OperatorBudgetDistanceSummary {
    let mut rows = Vec::new();
    for budget in &objectives.budgets {
        let row = match budget.metric.as_str() {
            "likely_human_friction_rate" => {
                build_friction_budget_row(budget, likely_human_lane, human_friction)
            }
            "suspicious_forwarded_request_rate" => {
                build_suspicious_forwarded_request_budget_row(budget, suspicious_lane)
            }
            "suspicious_forwarded_byte_rate" => {
                build_suspicious_forwarded_byte_budget_row(budget, suspicious_lane)
            }
            _ => None,
        };
        if let Some(row) = row {
            rows.push(row);
        }
    }
    OperatorBudgetDistanceSummary { rows }
}

fn build_friction_budget_row(
    budget: &OperatorObjectiveBudget,
    likely_human_lane: Option<&RequestOutcomeLaneSummaryRow>,
    human_friction: Option<&HumanFrictionSegmentRow>,
) -> Option<OperatorBudgetDistanceRow> {
    let friction = human_friction?;
    let (exactness, basis) = if let Some(lane) = likely_human_lane {
        (lane.exactness.clone(), lane.basis.clone())
    } else {
        ("derived".to_string(), "observed".to_string())
    };
    Some(budget_row(
        budget,
        friction.denominator_requests,
        friction.friction_rate,
        exactness,
        basis,
    ))
}

fn build_suspicious_forwarded_request_budget_row(
    budget: &OperatorObjectiveBudget,
    suspicious_lane: Option<&RequestOutcomeLaneSummaryRow>,
) -> Option<OperatorBudgetDistanceRow> {
    let lane = suspicious_lane?;
    let current = ratio(lane.forwarded_requests, lane.total_requests);
    Some(budget_row(
        budget,
        lane.total_requests,
        current,
        lane.exactness.clone(),
        lane.basis.clone(),
    ))
}

fn build_suspicious_forwarded_byte_budget_row(
    budget: &OperatorObjectiveBudget,
    suspicious_lane: Option<&RequestOutcomeLaneSummaryRow>,
) -> Option<OperatorBudgetDistanceRow> {
    let lane = suspicious_lane?;
    let total_bytes = lane
        .forwarded_response_bytes
        .saturating_add(lane.short_circuited_response_bytes)
        .saturating_add(lane.control_response_bytes);
    let current = ratio(lane.forwarded_response_bytes, total_bytes);
    Some(budget_row(
        budget,
        lane.total_requests,
        current,
        lane.exactness.clone(),
        lane.basis.clone(),
    ))
}

fn budget_row(
    budget: &OperatorObjectiveBudget,
    eligible_requests: u64,
    current: f64,
    exactness: String,
    basis: String,
) -> OperatorBudgetDistanceRow {
    let near_limit = budget.target * budget.near_limit_ratio;
    let status = if eligible_requests == 0 {
        "insufficient_evidence".to_string()
    } else if current <= near_limit {
        "inside_budget".to_string()
    } else if current <= budget.target {
        "near_limit".to_string()
    } else {
        "outside_budget".to_string()
    };
    OperatorBudgetDistanceRow {
        budget_id: budget.budget_id.clone(),
        metric: budget.metric.clone(),
        eligible_requests,
        current,
        target: budget.target,
        delta: current - budget.target,
        near_limit,
        status,
        exactness,
        basis,
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
    use super::{
        build_operator_snapshot_payload, OperatorSnapshotRecentChanges,
        OperatorSnapshotRecentSimRun, operator_snapshot_watch_window_hours,
        OPERATOR_SNAPSHOT_SCHEMA_VERSION,
    };
    use crate::challenge::KeyValueStore;
    use crate::observability::monitoring::{record_request_outcome, summarize_with_store};
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
    fn snapshot_payload_uses_backend_default_objective_profile_and_budget_statuses() {
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
        let watch_window_hours = operator_snapshot_watch_window_hours(summary.hours);
        let payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_000,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "run_001".to_string(),
                lane: "deterministic_black_box".to_string(),
                profile: "fast_smoke".to_string(),
                first_ts: 1_699_999_900,
                last_ts: 1_700_000_000,
                monitoring_event_count: 3,
                defense_delta_count: 2,
                ban_outcome_count: 0,
            }],
            OperatorSnapshotRecentChanges {
                lookback_seconds: watch_window_hours.saturating_mul(3).saturating_mul(3600),
                watch_window_seconds: watch_window_hours.saturating_mul(3600),
                rows: Vec::new(),
            },
            1_700_000_000,
            1_700_000_000,
            1_700_000_000,
        );

        assert_eq!(payload.schema_version, OPERATOR_SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(payload.objectives.profile_id, "backend_default_v1");
        assert!(payload
            .budget_distance
            .rows
            .iter()
            .any(|row| row.metric == "likely_human_friction_rate"));
        assert!(payload.recent_changes.rows.is_empty());
        assert_eq!(payload.allowed_actions.availability, "not_yet_materialized");
        assert_eq!(payload.verified_identity.availability, "not_yet_supported");
    }

    #[test]
    fn snapshot_payload_keeps_live_and_adversary_sim_sections_separate() {
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
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 256,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );

        let summary = summarize_with_store(&store, 24, 10);
        let watch_window_hours = operator_snapshot_watch_window_hours(summary.hours);
        let payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_000,
            &summary,
            &[],
            OperatorSnapshotRecentChanges {
                lookback_seconds: watch_window_hours.saturating_mul(3).saturating_mul(3600),
                watch_window_seconds: watch_window_hours.saturating_mul(3600),
                rows: Vec::new(),
            },
            1_700_000_000,
            1_700_000_000,
            1_700_000_000,
        );

        assert_eq!(payload.live_traffic.traffic_origin, "live");
        assert_eq!(payload.live_traffic.total_requests, 1);
        assert_eq!(
            payload
                .live_traffic
                .likely_human
                .as_ref()
                .expect("likely human lane")
                .total_requests,
            1
        );
        assert!(payload.live_traffic.suspicious_automation.is_none());
        assert_eq!(payload.adversary_sim.traffic_origin, "adversary_sim");
        assert_eq!(payload.adversary_sim.total_requests, 1);
        assert_eq!(
            payload
                .section_metadata
                .get("budget_distance")
                .expect("budget distance metadata")
                .exactness,
            crate::observability::hot_read_contract::TelemetryExactness::Derived
        );
    }
}
