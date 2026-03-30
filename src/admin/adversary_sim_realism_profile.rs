use serde::{Deserialize, Serialize};

pub(crate) const LANE_REALISM_PROFILE_SCHEMA_VERSION: &str = "sim-lane-realism-profile.v1";
pub(crate) const LANE_REALISM_RECEIPT_SCHEMA_VERSION: &str = "sim-lane-realism-receipt.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneRealismRange {
    pub min: u64,
    pub max: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneRealismIdentityRotation {
    pub strategy: String,
    pub min_every_n_activities: u64,
    pub max_every_n_activities: u64,
    pub stable_session_per_tick: bool,
    pub proxy_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneRealismReceiptContract {
    pub schema_version: String,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneRealismProfile {
    pub schema_version: String,
    pub profile_id: String,
    pub activity_unit: String,
    pub activity_budget: LaneRealismRange,
    pub burst_size: LaneRealismRange,
    pub intra_burst_jitter_ms: LaneRealismRange,
    pub between_burst_pause_ms: LaneRealismRange,
    pub navigation_dwell_ms: LaneRealismRange,
    pub identity_rotation: LaneRealismIdentityRotation,
    pub browser_propensity: String,
    pub javascript_execution: String,
    pub retry_ceiling: u64,
    pub receipt_contract: LaneRealismReceiptContract,
}

fn range(min: u64, max: u64) -> LaneRealismRange {
    LaneRealismRange { min, max }
}

fn receipt_contract(required_fields: &[&str]) -> LaneRealismReceiptContract {
    LaneRealismReceiptContract {
        schema_version: LANE_REALISM_RECEIPT_SCHEMA_VERSION.to_string(),
        required_fields: required_fields
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
    }
}

fn identity_rotation(
    strategy: &str,
    min_every_n_activities: u64,
    max_every_n_activities: u64,
    stable_session_per_tick: bool,
    proxy_required: bool,
) -> LaneRealismIdentityRotation {
    LaneRealismIdentityRotation {
        strategy: strategy.to_string(),
        min_every_n_activities,
        max_every_n_activities,
        stable_session_per_tick,
        proxy_required,
    }
}

fn request_native_receipt_contract() -> LaneRealismReceiptContract {
    receipt_contract(&[
        "activity_count",
        "burst_count",
        "burst_sizes",
        "inter_activity_gaps_ms",
        "identity_handles",
        "identity_rotation_count",
        "stop_reason",
    ])
}

fn browser_receipt_contract() -> LaneRealismReceiptContract {
    receipt_contract(&[
        "activity_count",
        "top_level_action_count",
        "dwell_intervals_ms",
        "session_handles",
        "identity_rotation_count",
        "stop_reason",
    ])
}

pub(crate) fn scrapling_realism_profile_for_mode(fulfillment_mode: &str) -> LaneRealismProfile {
    match fulfillment_mode {
        "crawler" => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.crawler.v1".to_string(),
            activity_unit: "request".to_string(),
            activity_budget: range(6, 14),
            burst_size: range(1, 2),
            intra_burst_jitter_ms: range(300, 1_200),
            between_burst_pause_ms: range(0, 0),
            navigation_dwell_ms: range(0, 0),
            identity_rotation: identity_rotation(
                "per_n_activities_when_proxy_available",
                2,
                4,
                false,
                true,
            ),
            browser_propensity: "none".to_string(),
            javascript_execution: "disabled".to_string(),
            retry_ceiling: 2,
            receipt_contract: request_native_receipt_contract(),
        },
        "bulk_scraper" => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.bulk_scraper.v1".to_string(),
            activity_unit: "request".to_string(),
            activity_budget: range(18, 45),
            burst_size: range(2, 5),
            intra_burst_jitter_ms: range(200, 800),
            between_burst_pause_ms: range(1_000, 3_000),
            navigation_dwell_ms: range(0, 0),
            identity_rotation: identity_rotation(
                "per_n_activities_when_proxy_available",
                1,
                3,
                false,
                true,
            ),
            browser_propensity: "none".to_string(),
            javascript_execution: "disabled".to_string(),
            retry_ceiling: 2,
            receipt_contract: request_native_receipt_contract(),
        },
        "browser_automation" => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.browser_automation.v1".to_string(),
            activity_unit: "action".to_string(),
            activity_budget: range(4, 9),
            burst_size: range(1, 1),
            intra_burst_jitter_ms: range(0, 0),
            between_burst_pause_ms: range(0, 0),
            navigation_dwell_ms: range(800, 2_500),
            identity_rotation: identity_rotation("none", 0, 0, true, false),
            browser_propensity: "required".to_string(),
            javascript_execution: "required".to_string(),
            retry_ceiling: 1,
            receipt_contract: browser_receipt_contract(),
        },
        "stealth_browser" => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.stealth_browser.v1".to_string(),
            activity_unit: "action".to_string(),
            activity_budget: range(3, 6),
            burst_size: range(1, 1),
            intra_burst_jitter_ms: range(0, 0),
            between_burst_pause_ms: range(0, 0),
            navigation_dwell_ms: range(1_400, 3_200),
            identity_rotation: identity_rotation("none", 0, 0, true, false),
            browser_propensity: "required".to_string(),
            javascript_execution: "required".to_string(),
            retry_ceiling: 1,
            receipt_contract: browser_receipt_contract(),
        },
        _ => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.http_agent.v1".to_string(),
            activity_unit: "request".to_string(),
            activity_budget: range(10, 24),
            burst_size: range(2, 4),
            intra_burst_jitter_ms: range(100, 500),
            between_burst_pause_ms: range(500, 2_000),
            navigation_dwell_ms: range(0, 0),
            identity_rotation: identity_rotation(
                "per_burst_when_proxy_available",
                1,
                1,
                false,
                true,
            ),
            browser_propensity: "none".to_string(),
            javascript_execution: "disabled".to_string(),
            retry_ceiling: 2,
            receipt_contract: request_native_receipt_contract(),
        },
    }
}

pub(crate) fn llm_realism_profile_for_mode(fulfillment_mode: &str) -> LaneRealismProfile {
    match fulfillment_mode {
        "browser_mode" => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "agentic.browser_mode.v1".to_string(),
            activity_unit: "action".to_string(),
            activity_budget: range(4, 8),
            burst_size: range(1, 1),
            intra_burst_jitter_ms: range(0, 0),
            between_burst_pause_ms: range(0, 0),
            navigation_dwell_ms: range(2_000, 7_000),
            identity_rotation: identity_rotation("none", 0, 0, true, false),
            browser_propensity: "required".to_string(),
            javascript_execution: "required".to_string(),
            retry_ceiling: 1,
            receipt_contract: browser_receipt_contract(),
        },
        _ => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "agentic.request_mode.v1".to_string(),
            activity_unit: "action".to_string(),
            activity_budget: range(8, 20),
            burst_size: range(3, 8),
            intra_burst_jitter_ms: range(125, 350),
            between_burst_pause_ms: range(1_000, 4_000),
            navigation_dwell_ms: range(0, 0),
            identity_rotation: identity_rotation("none", 0, 0, true, false),
            browser_propensity: "none".to_string(),
            javascript_execution: "disabled".to_string(),
            retry_ceiling: 2,
            receipt_contract: receipt_contract(&[
                "activity_count",
                "burst_count",
                "burst_sizes",
                "inter_activity_gaps_ms",
                "focused_page_set_size",
                "session_handles",
                "stop_reason",
            ]),
        },
    }
}
