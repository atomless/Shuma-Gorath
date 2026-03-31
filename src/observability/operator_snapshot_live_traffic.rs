use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::observability::monitoring::{
    HumanFrictionSegmentRow, MonitoringSummary, RequestOutcomeLaneSummaryRow,
    RequestOutcomeScopeSummaryRow,
};
use crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary;
use crate::admin::adversary_sim::LlmRuntimeRecentRunSummary;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotLane {
    pub lane: String,
    pub exactness: String,
    pub basis: String,
    pub total_requests: u64,
    pub forwarded_requests: u64,
    pub short_circuited_requests: u64,
    pub control_response_requests: u64,
    pub forwarded_upstream_latency_ms_total: u64,
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
    pub forwarded_upstream_latency_ms_total: u64,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_fulfillment_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_category_ids: Vec<String>,
    pub first_ts: u64,
    pub last_ts: u64,
    pub monitoring_event_count: u64,
    pub defense_delta_count: u64,
    pub ban_outcome_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_surface_coverage: Option<ScraplingOwnedSurfaceCoverageSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_scrapling_realism_receipt:
        Option<crate::admin::adversary_sim::ScraplingRealismReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_runtime_summary: Option<LlmRuntimeRecentRunSummary>,
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
    pub forwarded_upstream_latency_ms_total: u64,
    pub forwarded_response_bytes: u64,
    pub shuma_served_response_bytes: u64,
    pub recent_runs: Vec<OperatorSnapshotRecentSimRun>,
}

pub(super) fn scope_row<'a>(
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

pub(super) fn lane_row<'a>(
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

pub(super) fn human_friction_row<'a>(
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
        forwarded_upstream_latency_ms_total: row.forwarded_upstream_latency_ms_total,
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

pub(super) fn live_traffic_section(
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
        forwarded_upstream_latency_ms_total: scope.forwarded_upstream_latency_ms_total,
        forwarded_response_bytes: scope.forwarded_response_bytes,
        shuma_served_response_bytes: scope
            .short_circuited_response_bytes
            .saturating_add(scope.control_response_bytes),
        likely_human: likely_human_lane.map(lane_snapshot),
        suspicious_automation: suspicious_lane.map(lane_snapshot),
        human_friction: human_friction.map(human_friction_snapshot),
    }
}

pub(super) fn adversary_sim_section(
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
        forwarded_upstream_latency_ms_total: scope.forwarded_upstream_latency_ms_total,
        forwarded_response_bytes: scope.forwarded_response_bytes,
        shuma_served_response_bytes: scope
            .short_circuited_response_bytes
            .saturating_add(scope.control_response_bytes),
        recent_runs: recent_sim_runs.to_vec(),
    }
}
