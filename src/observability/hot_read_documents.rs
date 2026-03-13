#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::observability::hot_read_contract::{
    monitoring_bootstrap_component_contracts, HotReadProjectionModel, TelemetryExactness,
};
use crate::observability::monitoring::MonitoringSummary;
use crate::observability::retention::RetentionHealth;

const HOT_READ_PREFIX: &str = "telemetry:hot_read:v1";
const HOT_READ_BOOTSTRAP_SCHEMA_VERSION: &str = "telemetry-hot-read-bootstrap.v1";
const HOT_READ_RETENTION_SCHEMA_VERSION: &str = "telemetry-hot-read-retention.v1";
const HOT_READ_SECURITY_PRIVACY_SCHEMA_VERSION: &str = "telemetry-hot-read-security-privacy.v1";
const HOT_READ_RECENT_EVENTS_TAIL_SCHEMA_VERSION: &str = "telemetry-hot-read-recent-events.v1";
const HOT_READ_MONITORING_SUMMARY_SCHEMA_VERSION: &str = "telemetry-hot-read-summary.v1";
const HOT_READ_BOOTSTRAP_WINDOW_HOURS: u64 = 24;
const HOT_READ_BOOTSTRAP_MAX_BYTES: usize = 64 * 1024;
const HOT_READ_SECURITY_PRIVACY_MAX_BYTES: usize = 16 * 1024;
const HOT_READ_RETENTION_MAX_BYTES: usize = 8 * 1024;
const HOT_READ_RECENT_EVENTS_TAIL_MAX_BYTES: usize = 32 * 1024;
const HOT_READ_MONITORING_SUMMARY_MAX_BYTES: usize = 24 * 1024;
const HOT_READ_RECENT_EVENTS_TAIL_MAX_RECORDS: usize = 40;
const HOT_READ_MONITORING_SUMMARY_TOP_LIMIT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HotReadUpdateTrigger {
    MonitoringFlush,
    EventAppend,
    RetentionWorker,
    AdminMutation,
    RepairRebuild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HotReadFreshnessBudget {
    pub stale_after_seconds: u64,
    pub rebuild_after_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HotReadRepairPolicy {
    pub rebuild_on_missing: bool,
    pub rebuild_on_schema_mismatch: bool,
    pub rebuild_on_decode_error: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HotReadDocumentContract {
    pub document_key: &'static str,
    pub schema_version: &'static str,
    pub max_serialized_bytes: usize,
    pub freshness: HotReadFreshnessBudget,
    pub repair_policy: HotReadRepairPolicy,
    pub projection_model: HotReadProjectionModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HotReadDocumentMetadata {
    pub schema_version: String,
    pub site_id: String,
    pub generated_at_ts: u64,
    pub trigger: HotReadUpdateTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HotReadComponentMetadata {
    pub exactness: TelemetryExactness,
    pub refreshed_at_ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HotReadDocumentEnvelope<T> {
    pub metadata: HotReadDocumentMetadata,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MonitoringBootstrapAnalyticsSummary {
    pub ban_count: u64,
    pub test_mode: bool,
    pub fail_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MonitoringRecentEventsWindowSummary {
    pub hours: u64,
    pub requested_limit: usize,
    pub applied_recent_event_cap: usize,
    pub total_events_in_window: usize,
    pub returned_events: usize,
    pub has_more: bool,
    pub continue_via: String,
    pub response_shaping_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MonitoringBootstrapHotReadPayload {
    pub component_metadata: BTreeMap<String, HotReadComponentMetadata>,
    pub analytics: MonitoringBootstrapAnalyticsSummary,
    pub retention_health: RetentionHealth,
    pub security_privacy: Value,
    pub security_mode: String,
    pub recent_events: Vec<Value>,
    pub recent_events_window: MonitoringRecentEventsWindowSummary,
    pub window_end_cursor: Option<String>,
    pub drill_down_only_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MonitoringRecentEventsTailPayload {
    pub recent_events: Vec<Value>,
    pub recent_events_window: MonitoringRecentEventsWindowSummary,
    pub window_end_cursor: Option<String>,
}

pub(crate) type MonitoringBootstrapHotReadDocument =
    HotReadDocumentEnvelope<MonitoringBootstrapHotReadPayload>;
pub(crate) type MonitoringRetentionSummaryDocument = HotReadDocumentEnvelope<RetentionHealth>;
pub(crate) type MonitoringSecurityPrivacySummaryDocument = HotReadDocumentEnvelope<Value>;
pub(crate) type MonitoringRecentEventsTailDocument =
    HotReadDocumentEnvelope<MonitoringRecentEventsTailPayload>;
pub(crate) type MonitoringSummaryHotReadDocument = HotReadDocumentEnvelope<MonitoringSummary>;

const MONITORING_BOOTSTRAP_UPDATE_TRIGGERS: [HotReadUpdateTrigger; 4] = [
    HotReadUpdateTrigger::MonitoringFlush,
    HotReadUpdateTrigger::EventAppend,
    HotReadUpdateTrigger::RetentionWorker,
    HotReadUpdateTrigger::AdminMutation,
];

const MONITORING_SUPPORTING_SUMMARY_UPDATE_TRIGGERS: [HotReadUpdateTrigger; 3] = [
    HotReadUpdateTrigger::MonitoringFlush,
    HotReadUpdateTrigger::RetentionWorker,
    HotReadUpdateTrigger::AdminMutation,
];

const MONITORING_RECENT_EVENTS_UPDATE_TRIGGERS: [HotReadUpdateTrigger; 2] = [
    HotReadUpdateTrigger::EventAppend,
    HotReadUpdateTrigger::RepairRebuild,
];

const MONITORING_BOOTSTRAP_DRILL_DOWN_ONLY_FIELDS: [&str; 8] = [
    "events.event_counts",
    "events.top_ips",
    "events.unique_ips",
    "bans.bans",
    "maze",
    "tarpit",
    "cdp",
    "cdp_events.events",
];

const MONITORING_BOOTSTRAP_DOCUMENT_CONTRACT: HotReadDocumentContract = HotReadDocumentContract {
    document_key: "telemetry:hot_read:v1:bootstrap:<site>",
    schema_version: HOT_READ_BOOTSTRAP_SCHEMA_VERSION,
    max_serialized_bytes: HOT_READ_BOOTSTRAP_MAX_BYTES,
    freshness: HotReadFreshnessBudget {
        stale_after_seconds: 15,
        rebuild_after_seconds: 90,
    },
    repair_policy: HotReadRepairPolicy {
        rebuild_on_missing: true,
        rebuild_on_schema_mismatch: true,
        rebuild_on_decode_error: true,
    },
    projection_model: HotReadProjectionModel::DeterministicRebuild,
};

const MONITORING_RETENTION_SUMMARY_DOCUMENT_CONTRACT: HotReadDocumentContract =
    HotReadDocumentContract {
        document_key: "telemetry:hot_read:v1:retention_summary:<site>",
        schema_version: HOT_READ_RETENTION_SCHEMA_VERSION,
        max_serialized_bytes: HOT_READ_RETENTION_MAX_BYTES,
        freshness: HotReadFreshnessBudget {
            stale_after_seconds: 30,
            rebuild_after_seconds: 180,
        },
        repair_policy: HotReadRepairPolicy {
            rebuild_on_missing: true,
            rebuild_on_schema_mismatch: true,
            rebuild_on_decode_error: true,
        },
        projection_model: HotReadProjectionModel::DeterministicRebuild,
    };

const MONITORING_SECURITY_PRIVACY_SUMMARY_DOCUMENT_CONTRACT: HotReadDocumentContract =
    HotReadDocumentContract {
        document_key: "telemetry:hot_read:v1:security_privacy_summary:<site>",
        schema_version: HOT_READ_SECURITY_PRIVACY_SCHEMA_VERSION,
        max_serialized_bytes: HOT_READ_SECURITY_PRIVACY_MAX_BYTES,
        freshness: HotReadFreshnessBudget {
            stale_after_seconds: 30,
            rebuild_after_seconds: 180,
        },
        repair_policy: HotReadRepairPolicy {
            rebuild_on_missing: true,
            rebuild_on_schema_mismatch: true,
            rebuild_on_decode_error: true,
        },
        projection_model: HotReadProjectionModel::DeterministicRebuild,
    };

const MONITORING_RECENT_EVENTS_TAIL_DOCUMENT_CONTRACT: HotReadDocumentContract =
    HotReadDocumentContract {
        document_key: "telemetry:hot_read:v1:recent_events_tail:<site>",
        schema_version: HOT_READ_RECENT_EVENTS_TAIL_SCHEMA_VERSION,
        max_serialized_bytes: HOT_READ_RECENT_EVENTS_TAIL_MAX_BYTES,
        freshness: HotReadFreshnessBudget {
            stale_after_seconds: 10,
            rebuild_after_seconds: 60,
        },
        repair_policy: HotReadRepairPolicy {
            rebuild_on_missing: true,
            rebuild_on_schema_mismatch: true,
            rebuild_on_decode_error: true,
        },
        projection_model: HotReadProjectionModel::DeterministicRebuild,
    };

const MONITORING_SUMMARY_DOCUMENT_CONTRACT: HotReadDocumentContract = HotReadDocumentContract {
    document_key: "telemetry:hot_read:v1:monitoring_summary:<site>",
    schema_version: HOT_READ_MONITORING_SUMMARY_SCHEMA_VERSION,
    max_serialized_bytes: HOT_READ_MONITORING_SUMMARY_MAX_BYTES,
    freshness: HotReadFreshnessBudget {
        stale_after_seconds: 15,
        rebuild_after_seconds: 90,
    },
    repair_policy: HotReadRepairPolicy {
        rebuild_on_missing: true,
        rebuild_on_schema_mismatch: true,
        rebuild_on_decode_error: true,
    },
    projection_model: HotReadProjectionModel::DeterministicRebuild,
};

pub(crate) fn monitoring_bootstrap_document_contract() -> HotReadDocumentContract {
    MONITORING_BOOTSTRAP_DOCUMENT_CONTRACT
}

pub(crate) fn monitoring_retention_summary_document_contract() -> HotReadDocumentContract {
    MONITORING_RETENTION_SUMMARY_DOCUMENT_CONTRACT
}

pub(crate) fn monitoring_security_privacy_summary_document_contract() -> HotReadDocumentContract {
    MONITORING_SECURITY_PRIVACY_SUMMARY_DOCUMENT_CONTRACT
}

pub(crate) fn monitoring_recent_events_tail_document_contract() -> HotReadDocumentContract {
    MONITORING_RECENT_EVENTS_TAIL_DOCUMENT_CONTRACT
}

pub(crate) fn monitoring_summary_document_contract() -> HotReadDocumentContract {
    MONITORING_SUMMARY_DOCUMENT_CONTRACT
}

pub(crate) fn monitoring_bootstrap_update_triggers() -> &'static [HotReadUpdateTrigger] {
    &MONITORING_BOOTSTRAP_UPDATE_TRIGGERS
}

pub(crate) fn monitoring_supporting_summary_update_triggers() -> &'static [HotReadUpdateTrigger] {
    &MONITORING_SUPPORTING_SUMMARY_UPDATE_TRIGGERS
}

pub(crate) fn monitoring_recent_events_tail_update_triggers() -> &'static [HotReadUpdateTrigger] {
    &MONITORING_RECENT_EVENTS_UPDATE_TRIGGERS
}

pub(crate) fn monitoring_bootstrap_drill_down_only_fields() -> &'static [&'static str] {
    &MONITORING_BOOTSTRAP_DRILL_DOWN_ONLY_FIELDS
}

pub(crate) fn monitoring_bootstrap_window_hours() -> u64 {
    HOT_READ_BOOTSTRAP_WINDOW_HOURS
}

pub(crate) fn monitoring_recent_events_tail_max_records() -> usize {
    HOT_READ_RECENT_EVENTS_TAIL_MAX_RECORDS
}

pub(crate) fn monitoring_summary_top_limit() -> usize {
    HOT_READ_MONITORING_SUMMARY_TOP_LIMIT
}

pub(crate) fn monitoring_bootstrap_document_key(site_id: &str) -> String {
    format!("{HOT_READ_PREFIX}:bootstrap:{site_id}")
}

pub(crate) fn monitoring_retention_summary_document_key(site_id: &str) -> String {
    format!("{HOT_READ_PREFIX}:retention_summary:{site_id}")
}

pub(crate) fn monitoring_security_privacy_summary_document_key(site_id: &str) -> String {
    format!("{HOT_READ_PREFIX}:security_privacy_summary:{site_id}")
}

pub(crate) fn monitoring_recent_events_tail_document_key(site_id: &str) -> String {
    format!("{HOT_READ_PREFIX}:recent_events_tail:{site_id}")
}

pub(crate) fn monitoring_summary_document_key(site_id: &str) -> String {
    format!("{HOT_READ_PREFIX}:monitoring_summary:{site_id}")
}

pub(crate) fn monitoring_hot_read_component_metadata(
    refreshed_at_ts: u64,
) -> BTreeMap<String, HotReadComponentMetadata> {
    monitoring_bootstrap_component_contracts()
        .iter()
        .map(|component| {
            (
                component.key.to_string(),
                HotReadComponentMetadata {
                    exactness: component.exactness,
                    refreshed_at_ts,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        monitoring_bootstrap_document_contract, monitoring_bootstrap_document_key,
        monitoring_bootstrap_drill_down_only_fields, monitoring_bootstrap_update_triggers,
        monitoring_bootstrap_window_hours, monitoring_hot_read_component_metadata,
        monitoring_recent_events_tail_document_contract, monitoring_recent_events_tail_max_records,
        monitoring_recent_events_tail_update_triggers,
        monitoring_retention_summary_document_contract,
        monitoring_summary_document_contract, monitoring_summary_document_key,
        monitoring_summary_top_limit,
        monitoring_security_privacy_summary_document_contract, HotReadUpdateTrigger,
    };
    use crate::observability::hot_read_contract::TelemetryExactness;

    #[test]
    fn monitoring_bootstrap_contract_is_bounded_to_payload_budget() {
        let contract = monitoring_bootstrap_document_contract();
        assert_eq!(contract.schema_version, "telemetry-hot-read-bootstrap.v1");
        assert_eq!(contract.max_serialized_bytes, 64 * 1024);
        assert_eq!(monitoring_bootstrap_window_hours(), 24);
        assert_eq!(monitoring_recent_events_tail_max_records(), 40);
        assert!(contract.freshness.stale_after_seconds < contract.freshness.rebuild_after_seconds);
    }

    #[test]
    fn supporting_summary_contracts_are_narrower_than_bootstrap_document() {
        let bootstrap = monitoring_bootstrap_document_contract();
        let retention = monitoring_retention_summary_document_contract();
        let security_privacy = monitoring_security_privacy_summary_document_contract();
        let recent_events = monitoring_recent_events_tail_document_contract();
        let monitoring_summary = monitoring_summary_document_contract();

        assert!(retention.max_serialized_bytes < bootstrap.max_serialized_bytes);
        assert!(security_privacy.max_serialized_bytes < bootstrap.max_serialized_bytes);
        assert!(recent_events.max_serialized_bytes < bootstrap.max_serialized_bytes);
        assert!(monitoring_summary.max_serialized_bytes < bootstrap.max_serialized_bytes);
        assert_eq!(recent_events.freshness.stale_after_seconds, 10);
        assert_eq!(monitoring_summary_top_limit(), 10);
    }

    #[test]
    fn bootstrap_drill_down_only_fields_keep_expensive_sections_out_of_hot_read() {
        let fields = monitoring_bootstrap_drill_down_only_fields();
        assert!(fields.contains(&"events.event_counts"));
        assert!(fields.contains(&"events.top_ips"));
        assert!(fields.contains(&"events.unique_ips"));
        assert!(fields.contains(&"bans.bans"));
        assert!(fields.contains(&"maze"));
        assert!(fields.contains(&"tarpit"));
        assert!(fields.contains(&"cdp"));
        assert!(fields.contains(&"cdp_events.events"));
    }

    #[test]
    fn bootstrap_component_metadata_preserves_exactness_contract() {
        let metadata = monitoring_hot_read_component_metadata(1_700_000_000);
        assert_eq!(
            metadata
                .get("recent_events_tail")
                .expect("recent events metadata")
                .exactness,
            TelemetryExactness::Exact
        );
        assert_eq!(
            metadata
                .get("security_privacy_summary")
                .expect("security privacy metadata")
                .exactness,
            TelemetryExactness::BestEffort
        );
    }

    #[test]
    fn bootstrap_and_tail_triggers_match_expected_write_paths() {
        let bootstrap_triggers = monitoring_bootstrap_update_triggers();
        let recent_event_triggers = monitoring_recent_events_tail_update_triggers();
        assert!(bootstrap_triggers.contains(&HotReadUpdateTrigger::MonitoringFlush));
        assert!(bootstrap_triggers.contains(&HotReadUpdateTrigger::EventAppend));
        assert!(bootstrap_triggers.contains(&HotReadUpdateTrigger::RetentionWorker));
        assert!(recent_event_triggers.contains(&HotReadUpdateTrigger::EventAppend));
        assert!(recent_event_triggers.contains(&HotReadUpdateTrigger::RepairRebuild));
    }

    #[test]
    fn document_keys_are_site_scoped_under_shared_prefix() {
        assert_eq!(
            monitoring_bootstrap_document_key("default"),
            "telemetry:hot_read:v1:bootstrap:default"
        );
        assert_eq!(
            monitoring_summary_document_key("default"),
            "telemetry:hot_read:v1:monitoring_summary:default"
        );
    }
}
