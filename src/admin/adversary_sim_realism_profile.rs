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
pub(crate) struct LaneRealismIdentityEnvelope {
    pub identity_classes: Vec<String>,
    pub geo_affinity_mode: String,
    pub session_stickiness: String,
    pub degraded_without_pool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneRealismTransportEnvelope {
    pub request_client_posture: String,
    pub browser_client_posture: String,
    pub accept_language_strategy: String,
    pub browser_locale_strategy: String,
    pub request_transport_profile: String,
    pub browser_transport_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneRealismReceiptContract {
    pub schema_version: String,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneRealismPressureEnvelope {
    pub max_activities: u64,
    pub max_time_budget_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneRealismExplorationEnvelope {
    pub max_depth: u64,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneRealismRecurrenceEnvelope {
    pub strategy: String,
    pub reentry_scope: String,
    pub dormant_gap_seconds: LaneRealismRange,
    pub representative_dormant_gap_seconds: LaneRealismRange,
    pub max_reentries_per_run: u64,
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
    pub identity_envelope: LaneRealismIdentityEnvelope,
    pub transport_envelope: LaneRealismTransportEnvelope,
    pub browser_propensity: String,
    pub javascript_execution: String,
    pub retry_ceiling: u64,
    pub pressure_envelope: LaneRealismPressureEnvelope,
    pub exploration_envelope: LaneRealismExplorationEnvelope,
    pub recurrence_envelope: LaneRealismRecurrenceEnvelope,
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

fn pressure_envelope(max_activities: u64, max_time_budget_ms: u64) -> LaneRealismPressureEnvelope {
    LaneRealismPressureEnvelope {
        max_activities,
        max_time_budget_ms,
    }
}

fn exploration_envelope(max_depth: u64, max_bytes: u64) -> LaneRealismExplorationEnvelope {
    LaneRealismExplorationEnvelope {
        max_depth,
        max_bytes,
    }
}

fn recurrence_envelope(
    strategy: &str,
    reentry_scope: &str,
    dormant_gap_min_seconds: u64,
    dormant_gap_max_seconds: u64,
    representative_dormant_gap_min_seconds: u64,
    representative_dormant_gap_max_seconds: u64,
    max_reentries_per_run: u64,
) -> LaneRealismRecurrenceEnvelope {
    LaneRealismRecurrenceEnvelope {
        strategy: strategy.to_string(),
        reentry_scope: reentry_scope.to_string(),
        dormant_gap_seconds: range(dormant_gap_min_seconds, dormant_gap_max_seconds),
        representative_dormant_gap_seconds: range(
            representative_dormant_gap_min_seconds,
            representative_dormant_gap_max_seconds,
        ),
        max_reentries_per_run,
    }
}

fn identity_envelope(
    identity_classes: &[&str],
    geo_affinity_mode: &str,
    session_stickiness: &str,
    degraded_without_pool: &str,
) -> LaneRealismIdentityEnvelope {
    LaneRealismIdentityEnvelope {
        identity_classes: identity_classes
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        geo_affinity_mode: geo_affinity_mode.to_string(),
        session_stickiness: session_stickiness.to_string(),
        degraded_without_pool: degraded_without_pool.to_string(),
    }
}

fn transport_envelope(
    request_client_posture: &str,
    browser_client_posture: &str,
    accept_language_strategy: &str,
    browser_locale_strategy: &str,
    request_transport_profile: &str,
    browser_transport_profile: &str,
) -> LaneRealismTransportEnvelope {
    LaneRealismTransportEnvelope {
        request_client_posture: request_client_posture.to_string(),
        browser_client_posture: browser_client_posture.to_string(),
        accept_language_strategy: accept_language_strategy.to_string(),
        browser_locale_strategy: browser_locale_strategy.to_string(),
        request_transport_profile: request_transport_profile.to_string(),
        browser_transport_profile: browser_transport_profile.to_string(),
    }
}

fn request_native_receipt_contract() -> LaneRealismReceiptContract {
    receipt_contract(&[
        "activity_count",
        "burst_count",
        "burst_sizes",
        "inter_activity_gaps_ms",
        "transport_profile",
        "transport_realism_class",
        "transport_emission_basis",
        "transport_degraded_reason",
        "observed_user_agent_families",
        "observed_accept_languages",
        "identity_realism_status",
        "identity_envelope_classes",
        "geo_affinity_mode",
        "session_stickiness",
        "observed_country_codes",
        "identity_handles",
        "identity_rotation_count",
        "recurrence_strategy",
        "reentry_scope",
        "dormancy_truth_mode",
        "session_index",
        "reentry_count",
        "max_reentries_per_run",
        "planned_dormant_gap_seconds",
        "representative_dormant_gap_seconds",
        "stop_reason",
    ])
}

fn exploration_receipt_fields() -> [&'static str; 6] {
    [
        "visited_url_count",
        "discovered_url_count",
        "deepest_depth_reached",
        "sitemap_documents_seen",
        "frontier_remaining_count",
        "canonical_public_pages_reached",
    ]
}

fn request_native_exploration_receipt_contract() -> LaneRealismReceiptContract {
    let mut required_fields = vec![
        "activity_count",
        "burst_count",
        "burst_sizes",
        "inter_activity_gaps_ms",
        "transport_profile",
        "transport_realism_class",
        "transport_emission_basis",
        "transport_degraded_reason",
        "observed_user_agent_families",
        "observed_accept_languages",
        "identity_realism_status",
        "identity_envelope_classes",
        "geo_affinity_mode",
        "session_stickiness",
        "observed_country_codes",
        "identity_handles",
        "identity_rotation_count",
        "recurrence_strategy",
        "reentry_scope",
        "dormancy_truth_mode",
        "session_index",
        "reentry_count",
        "max_reentries_per_run",
        "planned_dormant_gap_seconds",
        "representative_dormant_gap_seconds",
    ];
    required_fields.extend(exploration_receipt_fields());
    required_fields.push("stop_reason");
    receipt_contract(&required_fields)
}

fn browser_receipt_contract() -> LaneRealismReceiptContract {
    receipt_contract(&[
        "activity_count",
        "top_level_action_count",
        "action_types_attempted",
        "capability_state",
        "targeting_strategy",
        "dwell_intervals_ms",
        "transport_profile",
        "transport_realism_class",
        "transport_emission_basis",
        "transport_degraded_reason",
        "observed_user_agent_families",
        "observed_accept_languages",
        "observed_browser_locales",
        "secondary_capture_mode",
        "secondary_request_count",
        "background_request_count",
        "subresource_request_count",
        "identity_realism_status",
        "identity_envelope_classes",
        "geo_affinity_mode",
        "session_stickiness",
        "observed_country_codes",
        "session_handles",
        "identity_rotation_count",
        "recurrence_strategy",
        "reentry_scope",
        "dormancy_truth_mode",
        "session_index",
        "reentry_count",
        "max_reentries_per_run",
        "planned_dormant_gap_seconds",
        "representative_dormant_gap_seconds",
        "stop_reason",
    ])
}

pub(crate) fn scrapling_realism_profile_for_mode(fulfillment_mode: &str) -> LaneRealismProfile {
    match fulfillment_mode {
        "crawler" => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.crawler.v1".to_string(),
            activity_unit: "request".to_string(),
            activity_budget: range(24, 48),
            burst_size: range(6, 12),
            intra_burst_jitter_ms: range(10, 50),
            between_burst_pause_ms: range(0, 0),
            navigation_dwell_ms: range(0, 0),
            identity_rotation: identity_rotation(
                "per_n_activities_when_proxy_available",
                2,
                4,
                false,
                true,
            ),
            identity_envelope: identity_envelope(
                &["datacenter", "residential"],
                "pool_aligned",
                "stable_per_identity",
                "local_session_only",
            ),
            transport_envelope: transport_envelope(
                "desktop_browser_like",
                "desktop_browser_like",
                "identity_geo_aligned",
                "identity_geo_aligned",
                "curl_impersonate",
                "playwright_chromium",
            ),
            browser_propensity: "none".to_string(),
            javascript_execution: "disabled".to_string(),
            retry_ceiling: 3,
            pressure_envelope: pressure_envelope(48, 1_800),
            exploration_envelope: exploration_envelope(4, 393_216),
            recurrence_envelope: recurrence_envelope(
                "bounded_campaign_return",
                "cross_window_campaign",
                3,
                8,
                14_400,
                43_200,
                2,
            ),
            receipt_contract: request_native_exploration_receipt_contract(),
        },
        "bulk_scraper" => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.bulk_scraper.v1".to_string(),
            activity_unit: "request".to_string(),
            activity_budget: range(96, 180),
            burst_size: range(18, 32),
            intra_burst_jitter_ms: range(0, 25),
            between_burst_pause_ms: range(0, 25),
            navigation_dwell_ms: range(0, 0),
            identity_rotation: identity_rotation(
                "per_n_activities_when_proxy_available",
                1,
                3,
                false,
                true,
            ),
            identity_envelope: identity_envelope(
                &["residential", "mobile"],
                "pool_aligned",
                "stable_per_identity",
                "local_session_only",
            ),
            transport_envelope: transport_envelope(
                "mobile_browser_like",
                "desktop_browser_like",
                "identity_geo_aligned",
                "identity_geo_aligned",
                "curl_impersonate",
                "playwright_chromium",
            ),
            browser_propensity: "none".to_string(),
            javascript_execution: "disabled".to_string(),
            retry_ceiling: 3,
            pressure_envelope: pressure_envelope(180, 3_200),
            exploration_envelope: exploration_envelope(6, 786_432),
            recurrence_envelope: recurrence_envelope(
                "bounded_campaign_return",
                "cross_window_campaign",
                2,
                6,
                7_200,
                28_800,
                3,
            ),
            receipt_contract: request_native_exploration_receipt_contract(),
        },
        "browser_automation" => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.browser_automation.v1".to_string(),
            activity_unit: "action".to_string(),
            activity_budget: range(10, 18),
            burst_size: range(1, 1),
            intra_burst_jitter_ms: range(0, 0),
            between_burst_pause_ms: range(0, 0),
            navigation_dwell_ms: range(40, 150),
            identity_rotation: identity_rotation("none", 0, 0, true, false),
            identity_envelope: identity_envelope(
                &["residential", "mobile"],
                "pool_aligned",
                "stable_per_tick",
                "local_browser_session_only",
            ),
            transport_envelope: transport_envelope(
                "desktop_browser_like",
                "desktop_browser_like",
                "identity_geo_aligned",
                "identity_geo_aligned",
                "curl_impersonate",
                "playwright_chromium",
            ),
            browser_propensity: "required".to_string(),
            javascript_execution: "required".to_string(),
            retry_ceiling: 2,
            pressure_envelope: pressure_envelope(18, 3_000),
            exploration_envelope: exploration_envelope(3, 393_216),
            recurrence_envelope: recurrence_envelope(
                "bounded_campaign_return",
                "cross_window_campaign",
                4,
                10,
                21_600,
                86_400,
                2,
            ),
            receipt_contract: browser_receipt_contract(),
        },
        "stealth_browser" => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.stealth_browser.v1".to_string(),
            activity_unit: "action".to_string(),
            activity_budget: range(8, 14),
            burst_size: range(1, 1),
            intra_burst_jitter_ms: range(0, 0),
            between_burst_pause_ms: range(0, 0),
            navigation_dwell_ms: range(60, 175),
            identity_rotation: identity_rotation("none", 0, 0, true, false),
            identity_envelope: identity_envelope(
                &["residential", "mobile"],
                "pool_aligned",
                "stable_per_tick",
                "local_browser_session_only",
            ),
            transport_envelope: transport_envelope(
                "desktop_browser_like",
                "desktop_browser_like",
                "identity_geo_aligned",
                "identity_geo_aligned",
                "curl_impersonate",
                "playwright_chromium",
            ),
            browser_propensity: "required".to_string(),
            javascript_execution: "required".to_string(),
            retry_ceiling: 2,
            pressure_envelope: pressure_envelope(14, 3_000),
            exploration_envelope: exploration_envelope(3, 393_216),
            recurrence_envelope: recurrence_envelope(
                "bounded_campaign_return",
                "cross_window_campaign",
                6,
                12,
                43_200,
                172_800,
                1,
            ),
            receipt_contract: browser_receipt_contract(),
        },
        _ => LaneRealismProfile {
            schema_version: LANE_REALISM_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "scrapling.http_agent.v1".to_string(),
            activity_unit: "request".to_string(),
            activity_budget: range(72, 144),
            burst_size: range(18, 30),
            intra_burst_jitter_ms: range(0, 20),
            between_burst_pause_ms: range(0, 20),
            navigation_dwell_ms: range(0, 0),
            identity_rotation: identity_rotation(
                "per_burst_when_proxy_available",
                1,
                1,
                false,
                true,
            ),
            identity_envelope: identity_envelope(
                &["residential", "mobile"],
                "pool_aligned",
                "stable_per_identity",
                "local_session_only",
            ),
            transport_envelope: transport_envelope(
                "desktop_browser_like",
                "desktop_browser_like",
                "identity_geo_aligned",
                "identity_geo_aligned",
                "curl_impersonate",
                "playwright_chromium",
            ),
            browser_propensity: "none".to_string(),
            javascript_execution: "disabled".to_string(),
            retry_ceiling: 3,
            pressure_envelope: pressure_envelope(144, 10_000),
            exploration_envelope: exploration_envelope(4, 524_288),
            recurrence_envelope: recurrence_envelope(
                "bounded_campaign_return",
                "cross_window_campaign",
                2,
                5,
                3_600,
                21_600,
                3,
            ),
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
            identity_envelope: identity_envelope(
                &["residential", "mobile"],
                "pool_aligned",
                "stable_per_tick",
                "local_browser_session_only",
            ),
            transport_envelope: transport_envelope(
                "desktop_browser_like",
                "desktop_browser_like",
                "identity_geo_aligned",
                "identity_geo_aligned",
                "urllib_direct",
                "playwright_chromium",
            ),
            browser_propensity: "required".to_string(),
            javascript_execution: "required".to_string(),
            retry_ceiling: 1,
            pressure_envelope: pressure_envelope(8, 90_000),
            exploration_envelope: exploration_envelope(3, 393_216),
            recurrence_envelope: recurrence_envelope(
                "bounded_campaign_return",
                "cross_window_campaign",
                5,
                12,
                21_600,
                86_400,
                2,
            ),
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
            identity_envelope: identity_envelope(
                &["residential", "mobile"],
                "pool_aligned",
                "stable_per_identity",
                "local_session_only",
            ),
            transport_envelope: transport_envelope(
                "mobile_browser_like",
                "desktop_browser_like",
                "identity_geo_aligned",
                "identity_geo_aligned",
                "urllib_direct",
                "playwright_chromium",
            ),
            browser_propensity: "none".to_string(),
            javascript_execution: "disabled".to_string(),
            retry_ceiling: 2,
            pressure_envelope: pressure_envelope(24, 120_000),
            exploration_envelope: exploration_envelope(4, 524_288),
            recurrence_envelope: recurrence_envelope(
                "bounded_campaign_return",
                "cross_window_campaign",
                3,
                9,
                7_200,
                43_200,
                3,
            ),
            receipt_contract: receipt_contract(&[
                "activity_count",
                "burst_count",
                "burst_sizes",
                "inter_activity_gaps_ms",
                "action_types_attempted",
                "capability_state",
                "targeting_strategy",
                "focused_page_set_size",
                "concurrency_group_sizes",
                "peak_concurrent_activities",
                "transport_profile",
                "transport_realism_class",
                "transport_emission_basis",
                "transport_degraded_reason",
                "observed_user_agent_families",
                "observed_accept_languages",
                "identity_realism_status",
                "identity_envelope_classes",
                "geo_affinity_mode",
                "session_stickiness",
                "observed_country_codes",
                "identity_rotation_count",
                "session_handles",
                "recurrence_strategy",
                "reentry_scope",
                "dormancy_truth_mode",
                "session_index",
                "reentry_count",
                "max_reentries_per_run",
                "planned_dormant_gap_seconds",
                "representative_dormant_gap_seconds",
                "stop_reason",
            ]),
        },
    }
}
