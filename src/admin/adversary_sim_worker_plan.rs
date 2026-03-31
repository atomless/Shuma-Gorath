use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::adversary_sim::{RuntimeLane, WorkerFailureClass};
use super::adversary_sim_identity_pool::IdentityPoolEntry;
use super::adversary_sim_llm_lane::LlmFulfillmentPlan;
use super::adversary_sim_realism_profile::LaneRealismProfile;
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
pub(crate) struct LaneRealismRecurrenceContext {
    pub strategy: String,
    pub reentry_scope: String,
    pub dormancy_truth_mode: String,
    pub session_index: u64,
    pub reentry_count: u64,
    pub max_reentries_per_run: u64,
    pub planned_dormant_gap_seconds: u64,
    pub representative_dormant_gap_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScraplingRealismReceipt {
    pub schema_version: String,
    pub profile_id: String,
    pub activity_unit: String,
    pub planned_activity_budget: u64,
    pub effective_activity_budget: u64,
    pub planned_burst_size: u64,
    pub effective_burst_size: u64,
    pub activity_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub burst_sizes: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inter_activity_gaps_ms: Vec<u64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transport_profile: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transport_realism_class: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transport_emission_basis: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transport_degraded_reason: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_user_agent_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_accept_languages: Vec<String>,
    pub identity_realism_status: String,
    pub identity_provenance_mode: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identity_envelope_classes: Vec<String>,
    pub geo_affinity_mode: String,
    pub session_stickiness: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_country_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_level_action_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dwell_intervals_ms: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_browser_locales: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub secondary_capture_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_request_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_request_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subresource_request_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identity_handles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub session_handles: Vec<String>,
    pub identity_rotation_count: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub recurrence_strategy: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reentry_scope: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub dormancy_truth_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reentry_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_reentries_per_run: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_dormant_gap_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub representative_dormant_gap_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visited_url_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovered_url_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deepest_depth_reached: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sitemap_documents_seen: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontier_remaining_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_public_pages_reached: Option<u64>,
    pub stop_reason: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_proxy_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_proxy_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub request_identity_pool: Vec<IdentityPoolEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub browser_identity_pool: Vec<IdentityPoolEntry>,
    pub tick_started_at: u64,
    pub realism_profile: LaneRealismProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurrence_context: Option<LaneRealismRecurrenceContext>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category_targets: Vec<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realism_receipt: Option<ScraplingRealismReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surface_receipts: Vec<ScraplingSurfaceObservationReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LlmRuntimeActionReceipt {
    pub action_index: u64,
    pub action_type: String,
    pub path: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub status: Option<u16>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LlmRuntimeRealismReceipt {
    pub schema_version: String,
    pub profile_id: String,
    pub planned_activity_budget: u64,
    pub effective_activity_budget: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_burst_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_burst_size: Option<u64>,
    pub activity_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub burst_sizes: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inter_activity_gaps_ms: Vec<u64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transport_profile: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transport_realism_class: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transport_emission_basis: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transport_degraded_reason: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_user_agent_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_accept_languages: Vec<String>,
    pub identity_realism_status: String,
    pub identity_provenance_mode: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identity_envelope_classes: Vec<String>,
    pub geo_affinity_mode: String,
    pub session_stickiness: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_country_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_page_set_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_level_action_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dwell_intervals_ms: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_browser_locales: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub secondary_capture_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_request_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_request_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subresource_request_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub concurrency_group_sizes: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak_concurrent_activities: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub session_handles: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_rotation_count: Option<u64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub recurrence_strategy: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reentry_scope: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub dormancy_truth_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reentry_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_reentries_per_run: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_dormant_gap_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub representative_dormant_gap_seconds: Option<u64>,
    pub stop_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LlmRuntimeRecentRunSummary {
    pub receipt_count: u64,
    pub fulfillment_mode: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category_targets: Vec<String>,
    pub backend_kind: String,
    pub backend_state: String,
    pub generation_source: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub provider: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    pub generated_action_count: u64,
    pub executed_action_count: u64,
    pub failed_action_count: u64,
    pub passed_tick_count: u64,
    pub failed_tick_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_response_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<super::adversary_sim::WorkerFailureClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_failure: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_realism_receipt: Option<LlmRuntimeRealismReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub latest_action_receipts: Vec<LlmRuntimeActionReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LlmRuntimeResult {
    pub schema_version: String,
    pub run_id: String,
    pub tick_id: String,
    pub lane: RuntimeLane,
    pub fulfillment_mode: String,
    pub worker_id: String,
    pub tick_started_at: u64,
    pub tick_completed_at: u64,
    pub backend_kind: String,
    pub backend_state: String,
    pub generation_source: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model_id: String,
    #[serde(default)]
    pub fallback_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category_targets: Vec<String>,
    pub generated_action_count: u64,
    pub executed_action_count: u64,
    pub failed_action_count: u64,
    #[serde(default)]
    pub last_response_status: Option<u16>,
    pub passed: bool,
    #[serde(default)]
    pub failure_class: Option<super::adversary_sim::WorkerFailureClass>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub terminal_failure: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realism_receipt: Option<LlmRuntimeRealismReceipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_receipts: Vec<LlmRuntimeActionReceipt>,
}

impl LlmRuntimeRecentRunSummary {
    pub fn from_runtime_result(result: &LlmRuntimeResult) -> Self {
        let passed = result.passed && result.failure_class.is_none() && result.terminal_failure.is_none();
        LlmRuntimeRecentRunSummary {
            receipt_count: 1,
            fulfillment_mode: result.fulfillment_mode.clone(),
            category_targets: result.category_targets.clone(),
            backend_kind: result.backend_kind.clone(),
            backend_state: result.backend_state.clone(),
            generation_source: result.generation_source.clone(),
            provider: result.provider.clone(),
            model_id: result.model_id.clone(),
            fallback_reason: result.fallback_reason.clone(),
            generated_action_count: result.generated_action_count,
            executed_action_count: result.executed_action_count,
            failed_action_count: result.failed_action_count,
            passed_tick_count: if passed { 1 } else { 0 },
            failed_tick_count: if passed { 0 } else { 1 },
            last_response_status: result.last_response_status,
            failure_class: result.failure_class,
            error: result.error.clone(),
            terminal_failure: result.terminal_failure.clone(),
            latest_realism_receipt: result.realism_receipt.clone(),
            latest_action_receipts: result.action_receipts.clone(),
        }
    }

    pub fn merge_summary(&mut self, summary: &LlmRuntimeRecentRunSummary) {
        self.receipt_count = self.receipt_count.saturating_add(summary.receipt_count);
        self.fulfillment_mode = summary.fulfillment_mode.clone();
        self.backend_kind = summary.backend_kind.clone();
        self.backend_state = summary.backend_state.clone();
        self.generation_source = summary.generation_source.clone();
        self.provider = summary.provider.clone();
        self.model_id = summary.model_id.clone();
        self.fallback_reason = summary.fallback_reason.clone();
        self.generated_action_count = self
            .generated_action_count
            .saturating_add(summary.generated_action_count);
        self.executed_action_count = self
            .executed_action_count
            .saturating_add(summary.executed_action_count);
        self.failed_action_count = self
            .failed_action_count
            .saturating_add(summary.failed_action_count);
        self.passed_tick_count = self
            .passed_tick_count
            .saturating_add(summary.passed_tick_count);
        self.failed_tick_count = self
            .failed_tick_count
            .saturating_add(summary.failed_tick_count);
        self.last_response_status = summary.last_response_status;
        self.failure_class = summary.failure_class;
        self.error = summary.error.clone();
        self.terminal_failure = summary.terminal_failure.clone();
        self.latest_realism_receipt = summary.latest_realism_receipt.clone();
        self.latest_action_receipts = summary.latest_action_receipts.clone();
        for category_id in &summary.category_targets {
            if !self.category_targets.iter().any(|value| value == category_id) {
                self.category_targets.push(category_id.clone());
            }
        }
    }
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
    pub pending_dispatch_mode: Option<String>,
    pub worker_plan: Option<ScraplingWorkerPlan>,
    pub llm_fulfillment_plan: Option<LlmFulfillmentPlan>,
}
