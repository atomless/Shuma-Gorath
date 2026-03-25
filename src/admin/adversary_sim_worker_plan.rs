use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::adversary_sim::{RuntimeLane, WorkerFailureClass};
use super::adversary_sim_llm_lane::LlmFulfillmentPlan;
use crate::observability::scrapling_owned_surface::ScraplingSurfaceObservationReceipt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScraplingCrawlStats {
    #[serde(default)]
    pub requests_count: u64,
    #[serde(default)]
    pub offsite_requests_count: u64,
    #[serde(default)]
    pub blocked_requests_count: u64,
    #[serde(default)]
    pub response_status_count: BTreeMap<String, u64>,
    #[serde(default)]
    pub response_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScraplingRuntimePaths {
    pub public_search: String,
    pub not_a_bot_checkbox: String,
    pub challenge_submit: String,
    pub pow_verify: String,
    pub tarpit_progress: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScraplingWorkerPlan {
    pub schema_version: String,
    pub run_id: String,
    pub tick_id: String,
    pub lane: RuntimeLane,
    pub sim_profile: String,
    pub fulfillment_mode: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category_targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surface_targets: Vec<String>,
    pub runtime_paths: ScraplingRuntimePaths,
    pub tick_started_at: u64,
    pub max_requests: u64,
    pub max_depth: u64,
    pub max_bytes: u64,
    pub max_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScraplingWorkerResult {
    pub schema_version: String,
    pub run_id: String,
    pub tick_id: String,
    pub lane: RuntimeLane,
    pub fulfillment_mode: String,
    pub worker_id: String,
    pub tick_started_at: u64,
    pub tick_completed_at: u64,
    pub generated_requests: u64,
    pub failed_requests: u64,
    pub last_response_status: Option<u16>,
    #[serde(default)]
    pub failure_class: Option<WorkerFailureClass>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub crawl_stats: ScraplingCrawlStats,
    #[serde(default)]
    pub scope_rejections: BTreeMap<String, u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surface_receipts: Vec<ScraplingSurfaceObservationReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerationTickResult {
    pub generated_requests: u64,
    pub failed_requests: u64,
    pub last_response_status: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AutonomousHeartbeatTickSummary {
    pub due_ticks: u64,
    pub executed_ticks: u64,
    pub generated_requests: u64,
    pub failed_requests: u64,
    pub last_response_status: Option<u16>,
    pub worker_pending: bool,
    pub worker_plan: Option<ScraplingWorkerPlan>,
    pub llm_fulfillment_plan: Option<LlmFulfillmentPlan>,
}
