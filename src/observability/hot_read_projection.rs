#![allow(dead_code)]

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::challenge::KeyValueStore;
use crate::observability::hot_read_documents::{
    monitoring_bootstrap_document_contract, monitoring_bootstrap_document_key,
    monitoring_bootstrap_drill_down_only_fields, monitoring_bootstrap_window_hours,
    monitoring_hot_read_component_metadata, monitoring_recent_events_tail_document_contract,
    monitoring_recent_events_tail_document_key, monitoring_recent_events_tail_max_records,
    monitoring_recent_sim_runs_document_contract, monitoring_recent_sim_runs_document_key,
    monitoring_recent_sim_runs_max_records,
    monitoring_retention_summary_document_contract, monitoring_retention_summary_document_key,
    monitoring_security_privacy_summary_document_contract,
    monitoring_security_privacy_summary_document_key, monitoring_summary_document_contract,
    monitoring_summary_document_key, monitoring_summary_top_limit, HotReadDocumentEnvelope,
    HotReadDocumentMetadata, HotReadUpdateTrigger, MonitoringBootstrapAnalyticsSummary,
    MonitoringBootstrapHotReadDocument, MonitoringBootstrapHotReadPayload,
    MonitoringRecentEventsTailDocument, MonitoringRecentEventsTailPayload,
    MonitoringRecentSimRunsDocument, MonitoringRecentSimRunsPayload,
    MonitoringRetentionSummaryDocument, MonitoringSecurityPrivacySummaryDocument,
    MonitoringSummaryHotReadDocument, MonitoringRecentEventsWindowSummary,
};

fn write_document<S, T>(store: &S, key: String, document: &HotReadDocumentEnvelope<T>) -> Result<(), ()>
where
    S: KeyValueStore,
    T: Serialize,
{
    let bytes = serde_json::to_vec(document).map_err(|_| ())?;
    store.set(key.as_str(), bytes.as_slice()).map_err(|_| ())
}

fn read_document<S, T>(store: &S, key: String, expected_schema: &str) -> Option<HotReadDocumentEnvelope<T>>
where
    S: KeyValueStore,
    T: DeserializeOwned,
{
    let bytes = store.get(key.as_str()).ok().flatten()?;
    let document = serde_json::from_slice::<HotReadDocumentEnvelope<T>>(bytes.as_slice()).ok()?;
    (document.metadata.schema_version == expected_schema).then_some(document)
}

fn document_metadata(
    schema_version: &str,
    site_id: &str,
    generated_at_ts: u64,
    trigger: HotReadUpdateTrigger,
) -> HotReadDocumentMetadata {
    HotReadDocumentMetadata {
        schema_version: schema_version.to_string(),
        site_id: site_id.to_string(),
        generated_at_ts,
        trigger,
    }
}

fn analytics_summary<S: KeyValueStore>(store: &S, site_id: &str) -> MonitoringBootstrapAnalyticsSummary {
    let cfg = crate::config::Config::load(store, site_id).ok();
    let fail_mode = if crate::config::kv_store_fail_open() {
        "open"
    } else {
        "closed"
    };
    MonitoringBootstrapAnalyticsSummary {
        ban_count: crate::enforcement::ban::list_active_bans(store, site_id).len() as u64,
        shadow_mode: cfg.as_ref().map(|value| value.shadow_mode).unwrap_or(false),
        fail_mode: fail_mode.to_string(),
    }
}

fn build_monitoring_summary_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    trigger: HotReadUpdateTrigger,
) -> MonitoringSummaryHotReadDocument {
    let contract = monitoring_summary_document_contract();
    HotReadDocumentEnvelope {
        metadata: document_metadata(contract.schema_version, site_id, generated_at_ts, trigger),
        payload: crate::observability::monitoring::summarize_with_store(
            store,
            monitoring_bootstrap_window_hours(),
            monitoring_summary_top_limit(),
        ),
    }
}

fn build_security_privacy_summary_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    trigger: HotReadUpdateTrigger,
) -> MonitoringSecurityPrivacySummaryDocument {
    let contract = monitoring_security_privacy_summary_document_contract();
    HotReadDocumentEnvelope {
        metadata: document_metadata(contract.schema_version, site_id, generated_at_ts, trigger),
        payload: crate::admin::monitoring_security_privacy_payload(
            store,
            generated_at_ts,
            monitoring_bootstrap_window_hours(),
            false,
        ),
    }
}

fn build_retention_summary_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    trigger: HotReadUpdateTrigger,
) -> MonitoringRetentionSummaryDocument {
    let contract = monitoring_retention_summary_document_contract();
    HotReadDocumentEnvelope {
        metadata: document_metadata(contract.schema_version, site_id, generated_at_ts, trigger),
        payload: crate::observability::retention::retention_health(store),
    }
}

fn build_recent_events_tail_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    trigger: HotReadUpdateTrigger,
) -> MonitoringRecentEventsTailDocument {
    let contract = monitoring_recent_events_tail_document_contract();
    let recent = crate::admin::monitoring_presented_recent_event_tail(
        store,
        generated_at_ts,
        monitoring_bootstrap_window_hours(),
        monitoring_recent_events_tail_max_records(),
        false,
    );
    let continue_via = format!(
        "/admin/monitoring/delta?hours={}&limit={}",
        monitoring_bootstrap_window_hours(),
        monitoring_recent_events_tail_max_records()
    );
    HotReadDocumentEnvelope {
        metadata: document_metadata(contract.schema_version, site_id, generated_at_ts, trigger),
        payload: MonitoringRecentEventsTailPayload {
            recent_events: recent.recent_events,
            recent_event_rows: recent.recent_event_rows,
            recent_events_window: MonitoringRecentEventsWindowSummary {
                hours: monitoring_bootstrap_window_hours(),
                requested_limit: monitoring_recent_events_tail_max_records(),
                applied_recent_event_cap: monitoring_recent_events_tail_max_records(),
                total_events_in_window: recent.total_events_in_window,
                returned_events: recent.returned_events,
                has_more: recent.has_more,
                continue_via,
                response_shaping_reason: "bootstrap_recent_tail".to_string(),
            },
            window_end_cursor: recent.window_end_cursor,
        },
    }
}

fn build_recent_sim_runs_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    trigger: HotReadUpdateTrigger,
) -> MonitoringRecentSimRunsDocument {
    let contract = monitoring_recent_sim_runs_document_contract();
    HotReadDocumentEnvelope {
        metadata: document_metadata(contract.schema_version, site_id, generated_at_ts, trigger),
        payload: MonitoringRecentSimRunsPayload {
            recent_sim_runs: crate::admin::monitoring_recent_sim_run_summaries(
                store,
                generated_at_ts,
                monitoring_bootstrap_window_hours(),
                monitoring_recent_sim_runs_max_records(),
            ),
            window_hours: monitoring_bootstrap_window_hours(),
            applied_recent_run_cap: monitoring_recent_sim_runs_max_records(),
        },
    }
}

fn ensure_retention_summary_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringRetentionSummaryDocument {
    let contract = monitoring_retention_summary_document_contract();
    let key = monitoring_retention_summary_document_key(site_id);
    read_document(store, key.clone(), contract.schema_version).unwrap_or_else(|| {
        let document = build_retention_summary_document(
            store,
            site_id,
            generated_at_ts,
            HotReadUpdateTrigger::RepairRebuild,
        );
        let _ = write_document(store, key, &document);
        document
    })
}

fn ensure_security_privacy_summary_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringSecurityPrivacySummaryDocument {
    let contract = monitoring_security_privacy_summary_document_contract();
    let key = monitoring_security_privacy_summary_document_key(site_id);
    read_document(store, key.clone(), contract.schema_version).unwrap_or_else(|| {
        let document = build_security_privacy_summary_document(
            store,
            site_id,
            generated_at_ts,
            HotReadUpdateTrigger::RepairRebuild,
        );
        let _ = write_document(store, key, &document);
        document
    })
}

fn ensure_recent_events_tail_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringRecentEventsTailDocument {
    let contract = monitoring_recent_events_tail_document_contract();
    let key = monitoring_recent_events_tail_document_key(site_id);
    read_document(store, key.clone(), contract.schema_version).unwrap_or_else(|| {
        let document = build_recent_events_tail_document(
            store,
            site_id,
            generated_at_ts,
            HotReadUpdateTrigger::RepairRebuild,
        );
        let _ = write_document(store, key, &document);
        document
    })
}

fn ensure_recent_sim_runs_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringRecentSimRunsDocument {
    let contract = monitoring_recent_sim_runs_document_contract();
    let key = monitoring_recent_sim_runs_document_key(site_id);
    read_document(store, key.clone(), contract.schema_version).unwrap_or_else(|| {
        let document = build_recent_sim_runs_document(
            store,
            site_id,
            generated_at_ts,
            HotReadUpdateTrigger::RepairRebuild,
        );
        let _ = write_document(store, key, &document);
        document
    })
}

fn ensure_monitoring_summary_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringSummaryHotReadDocument {
    let contract = monitoring_summary_document_contract();
    let key = monitoring_summary_document_key(site_id);
    read_document(store, key.clone(), contract.schema_version).unwrap_or_else(|| {
        let document = build_monitoring_summary_document(
            store,
            site_id,
            generated_at_ts,
            HotReadUpdateTrigger::RepairRebuild,
        );
        let _ = write_document(store, key, &document);
        document
    })
}

fn ensure_bootstrap_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringBootstrapHotReadDocument {
    let contract = monitoring_bootstrap_document_contract();
    let key = monitoring_bootstrap_document_key(site_id);
    read_document(store, key.clone(), contract.schema_version).unwrap_or_else(|| {
        let document =
            rebuild_bootstrap_document(store, site_id, generated_at_ts, HotReadUpdateTrigger::RepairRebuild);
        let _ = write_document(store, key, &document);
        document
    })
}

pub(crate) fn load_monitoring_summary_hot_read<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringSummaryHotReadDocument {
    ensure_monitoring_summary_document(store, site_id, generated_at_ts)
}

pub(crate) fn load_monitoring_bootstrap_hot_read<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringBootstrapHotReadDocument {
    ensure_bootstrap_document(store, site_id, generated_at_ts)
}

pub(crate) fn load_monitoring_recent_events_tail_hot_read<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringRecentEventsTailDocument {
    ensure_recent_events_tail_document(store, site_id, generated_at_ts)
}

pub(crate) fn load_monitoring_recent_sim_runs_hot_read<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringRecentSimRunsDocument {
    ensure_recent_sim_runs_document(store, site_id, generated_at_ts)
}

pub(crate) fn load_monitoring_security_privacy_summary_hot_read<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> MonitoringSecurityPrivacySummaryDocument {
    ensure_security_privacy_summary_document(store, site_id, generated_at_ts)
}

fn rebuild_bootstrap_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    trigger: HotReadUpdateTrigger,
) -> MonitoringBootstrapHotReadDocument {
    let contract = monitoring_bootstrap_document_contract();
    let retention = ensure_retention_summary_document(store, site_id, generated_at_ts);
    let security_privacy = ensure_security_privacy_summary_document(store, site_id, generated_at_ts);
    let recent_events = ensure_recent_events_tail_document(store, site_id, generated_at_ts);
    let recent_sim_runs = ensure_recent_sim_runs_document(store, site_id, generated_at_ts);
    let mut component_metadata = monitoring_hot_read_component_metadata(generated_at_ts);
    if let Some(metadata) = component_metadata.get_mut("retention_health_summary") {
        metadata.refreshed_at_ts = retention.metadata.generated_at_ts;
    }
    if let Some(metadata) = component_metadata.get_mut("security_privacy_summary") {
        metadata.refreshed_at_ts = security_privacy.metadata.generated_at_ts;
    }
    if let Some(metadata) = component_metadata.get_mut("recent_events_tail") {
        metadata.refreshed_at_ts = recent_events.metadata.generated_at_ts;
    }
    if let Some(metadata) = component_metadata.get_mut("recent_sim_runs_summary") {
        metadata.refreshed_at_ts = recent_sim_runs.metadata.generated_at_ts;
    }
    let monitoring_summary = ensure_monitoring_summary_document(store, site_id, generated_at_ts);
    if let Some(metadata) = component_metadata.get_mut("monitoring_summary") {
        metadata.refreshed_at_ts = monitoring_summary.metadata.generated_at_ts;
    }
    if let Some(metadata) = component_metadata.get_mut("runtime_posture_summary") {
        metadata.refreshed_at_ts = generated_at_ts;
    }
    if let Some(metadata) = component_metadata.get_mut("active_ban_summary") {
        metadata.refreshed_at_ts = generated_at_ts;
    }

    HotReadDocumentEnvelope {
        metadata: document_metadata(contract.schema_version, site_id, generated_at_ts, trigger),
        payload: MonitoringBootstrapHotReadPayload {
            summary: monitoring_summary.payload,
            component_metadata,
            analytics: analytics_summary(store, site_id),
            retention_health: retention.payload,
            security_privacy: security_privacy.payload,
            security_mode: crate::admin::monitoring_security_view_mode_label(false).to_string(),
            recent_events: recent_events.payload.recent_events,
            recent_sim_runs: recent_sim_runs.payload.recent_sim_runs,
            recent_events_window: recent_events.payload.recent_events_window,
            window_end_cursor: recent_events.payload.window_end_cursor,
            drill_down_only_fields: monitoring_bootstrap_drill_down_only_fields()
                .iter()
                .map(|value| value.to_string())
                .collect(),
        },
    }
}

fn write_bootstrap_document<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    trigger: HotReadUpdateTrigger,
) {
    let key = monitoring_bootstrap_document_key(site_id);
    let document = rebuild_bootstrap_document(store, site_id, generated_at_ts, trigger);
    if let Err(err) = write_document(store, key, &document) {
        eprintln!(
            "[telemetry-hot-read] failed writing bootstrap document site={} trigger={:?} error={:?}",
            site_id, trigger, err
        );
    }
}

pub(crate) fn refresh_after_counter_flush<S: KeyValueStore>(store: &S, site_id: &str) {
    let now = crate::admin::now_ts();
    let summary_key = monitoring_summary_document_key(site_id);
    let summary = build_monitoring_summary_document(
        store,
        site_id,
        now,
        HotReadUpdateTrigger::MonitoringFlush,
    );
    if write_document(store, summary_key, &summary).is_err() {
        eprintln!(
            "[telemetry-hot-read] failed writing monitoring summary site={} trigger=monitoring_flush",
            site_id
        );
    }
    let security_privacy_key = monitoring_security_privacy_summary_document_key(site_id);
    let security_privacy = build_security_privacy_summary_document(
        store,
        site_id,
        now,
        HotReadUpdateTrigger::MonitoringFlush,
    );
    if write_document(store, security_privacy_key, &security_privacy).is_err() {
        eprintln!(
            "[telemetry-hot-read] failed writing security/privacy summary site={} trigger=monitoring_flush",
            site_id
        );
    }
    write_bootstrap_document(store, site_id, now, HotReadUpdateTrigger::MonitoringFlush);
}

pub(crate) fn refresh_after_event_append<S: KeyValueStore>(store: &S, site_id: &str) {
    let now = crate::admin::now_ts();
    let key = monitoring_recent_events_tail_document_key(site_id);
    let document =
        build_recent_events_tail_document(store, site_id, now, HotReadUpdateTrigger::EventAppend);
    if write_document(store, key, &document).is_err() {
        eprintln!(
            "[telemetry-hot-read] failed writing recent events tail site={} trigger=event_append",
            site_id
        );
    }
    let sim_runs_key = monitoring_recent_sim_runs_document_key(site_id);
    let sim_runs =
        build_recent_sim_runs_document(store, site_id, now, HotReadUpdateTrigger::EventAppend);
    if write_document(store, sim_runs_key, &sim_runs).is_err() {
        eprintln!(
            "[telemetry-hot-read] failed writing recent sim runs site={} trigger=event_append",
            site_id
        );
    }
    write_bootstrap_document(store, site_id, now, HotReadUpdateTrigger::EventAppend);
}

pub(crate) fn refresh_after_retention_worker<S: KeyValueStore>(store: &S, site_id: &str) {
    let now = crate::admin::now_ts();
    let key = monitoring_retention_summary_document_key(site_id);
    let document =
        build_retention_summary_document(store, site_id, now, HotReadUpdateTrigger::RetentionWorker);
    if write_document(store, key, &document).is_err() {
        eprintln!(
            "[telemetry-hot-read] failed writing retention summary site={} trigger=retention_worker",
            site_id
        );
    }
    write_bootstrap_document(store, site_id, now, HotReadUpdateTrigger::RetentionWorker);
}

pub(crate) fn refresh_after_admin_mutation<S: KeyValueStore>(store: &S, site_id: &str) {
    let now = crate::admin::now_ts();
    let retention_key = monitoring_retention_summary_document_key(site_id);
    let retention = build_retention_summary_document(
        store,
        site_id,
        now,
        HotReadUpdateTrigger::AdminMutation,
    );
    let _ = write_document(store, retention_key, &retention);
    let security_privacy_key = monitoring_security_privacy_summary_document_key(site_id);
    let security_privacy = build_security_privacy_summary_document(
        store,
        site_id,
        now,
        HotReadUpdateTrigger::AdminMutation,
    );
    let _ = write_document(store, security_privacy_key, &security_privacy);
    write_bootstrap_document(store, site_id, now, HotReadUpdateTrigger::AdminMutation);
}

#[cfg(test)]
mod tests {
    use super::{
        monitoring_bootstrap_document_key, monitoring_recent_events_tail_document_key,
        monitoring_recent_sim_runs_document_key,
        monitoring_retention_summary_document_key,
        monitoring_security_privacy_summary_document_key, monitoring_summary_document_key,
        read_document, refresh_after_admin_mutation, refresh_after_counter_flush,
        refresh_after_event_append, refresh_after_retention_worker,
    };
    use crate::admin::{log_event, EventLogEntry, EventType};
    use crate::challenge::KeyValueStore;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                map: Mutex::new(HashMap::new()),
            }
        }
    }

    impl KeyValueStore for MockStore {
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
    fn counter_flush_refresh_writes_summary_and_bootstrap_documents() {
        let store = MockStore::new();
        crate::observability::monitoring::record_shadow_action(
            &store,
            crate::runtime::effect_intents::ShadowAction::Challenge,
        );
        refresh_after_counter_flush(&store, "default");

        let summary = read_document::<_, crate::observability::monitoring::MonitoringSummary>(
            &store,
            monitoring_summary_document_key("default"),
            crate::observability::hot_read_documents::monitoring_summary_document_contract()
                .schema_version,
        );
        assert!(summary.is_some());
        let bootstrap = read_document::<
            _,
            crate::observability::hot_read_documents::MonitoringBootstrapHotReadPayload,
        >(
            &store,
            monitoring_bootstrap_document_key("default"),
            crate::observability::hot_read_documents::monitoring_bootstrap_document_contract()
                .schema_version,
        );
        assert!(bootstrap.is_some());
    }

    #[test]
    fn counter_flush_refresh_preserves_request_outcome_summary_rows_in_summary_and_bootstrap() {
        let store = MockStore::new();
        crate::observability::monitoring::record_request_outcome(
            &store,
            &crate::runtime::request_outcome::RenderedRequestOutcome {
                traffic_origin: crate::runtime::request_outcome::TrafficOrigin::Live,
                measurement_scope:
                    crate::runtime::traffic_classification::MeasurementScope::IngressPrimary,
                route_action_family:
                    crate::runtime::traffic_classification::RouteActionFamily::PublicContent,
                execution_mode: crate::runtime::effect_intents::ExecutionMode::Enforced,
                traffic_lane: Some(crate::runtime::request_outcome::RequestOutcomeLane {
                    lane: crate::runtime::traffic_classification::TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                outcome_class: crate::runtime::request_outcome::RequestOutcomeClass::Forwarded,
                response_kind: crate::runtime::request_outcome::ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 123,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: crate::runtime::traffic_classification::PolicySource::CleanAllow,
            },
        );
        crate::observability::monitoring::record_request_outcome(
            &store,
            &crate::runtime::request_outcome::RenderedRequestOutcome {
                traffic_origin: crate::runtime::request_outcome::TrafficOrigin::Live,
                measurement_scope:
                    crate::runtime::traffic_classification::MeasurementScope::IngressPrimary,
                route_action_family:
                    crate::runtime::traffic_classification::RouteActionFamily::PublicContent,
                execution_mode: crate::runtime::effect_intents::ExecutionMode::Enforced,
                traffic_lane: Some(crate::runtime::request_outcome::RequestOutcomeLane {
                    lane: crate::runtime::traffic_classification::TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                outcome_class:
                    crate::runtime::request_outcome::RequestOutcomeClass::ShortCircuited,
                response_kind: crate::runtime::request_outcome::ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source:
                    crate::runtime::traffic_classification::PolicySource::PolicyGraphSecondTranche,
            },
        );
        refresh_after_counter_flush(&store, "default");

        let summary = read_document::<_, crate::observability::monitoring::MonitoringSummary>(
            &store,
            monitoring_summary_document_key("default"),
            crate::observability::hot_read_documents::monitoring_summary_document_contract()
                .schema_version,
        )
        .expect("summary document");
        let live_scope = summary
            .payload
            .request_outcomes
            .by_scope
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "ingress_primary"
                    && row.execution_mode == "enforced"
            })
            .expect("live scope row in summary");
        assert_eq!(live_scope.total_requests, 2);
        assert_eq!(live_scope.forwarded_requests, 1);
        assert_eq!(live_scope.short_circuited_requests, 1);
        assert_eq!(live_scope.response_bytes, 168);
        let summary_human_friction = summary
            .payload
            .human_friction
            .segments
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.segment == "likely_human")
            .expect("likely human friction row in summary");
        assert_eq!(summary_human_friction.denominator_requests, 2);
        assert_eq!(summary_human_friction.not_a_bot_requests, 1);
        let summary_not_a_bot_funnel = summary
            .payload
            .defence_funnel
            .rows
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.family == "not_a_bot")
            .expect("not_a_bot funnel row in summary");
        assert_eq!(summary_not_a_bot_funnel.candidate_requests, Some(1));
        assert_eq!(summary_not_a_bot_funnel.passed_requests, None);
        assert_eq!(summary_not_a_bot_funnel.likely_human_affected_requests, Some(1));

        let bootstrap = read_document::<
            _,
            crate::observability::hot_read_documents::MonitoringBootstrapHotReadPayload,
        >(
            &store,
            monitoring_bootstrap_document_key("default"),
            crate::observability::hot_read_documents::monitoring_bootstrap_document_contract()
                .schema_version,
        )
        .expect("bootstrap document");
        let bootstrap_lane = bootstrap
            .payload
            .summary
            .request_outcomes
            .by_lane
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "ingress_primary"
                    && row.execution_mode == "enforced"
                    && row.lane == "likely_human"
            })
            .expect("live lane row in bootstrap");
        assert_eq!(bootstrap_lane.total_requests, 2);
        assert_eq!(bootstrap_lane.forwarded_requests, 1);
        assert_eq!(bootstrap_lane.short_circuited_requests, 1);
        assert_eq!(bootstrap_lane.response_bytes, 168);
        let bootstrap_human_friction = bootstrap
            .payload
            .summary
            .human_friction
            .segments
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.segment == "likely_human")
            .expect("likely human friction row in bootstrap");
        assert_eq!(bootstrap_human_friction.denominator_requests, 2);
        assert_eq!(bootstrap_human_friction.not_a_bot_requests, 1);
        let bootstrap_not_a_bot_funnel = bootstrap
            .payload
            .summary
            .defence_funnel
            .rows
            .iter()
            .find(|row| row.execution_mode == "enforced" && row.family == "not_a_bot")
            .expect("not_a_bot funnel row in bootstrap");
        assert_eq!(bootstrap_not_a_bot_funnel.candidate_requests, Some(1));
        assert_eq!(bootstrap_not_a_bot_funnel.passed_requests, None);
        assert_eq!(
            bootstrap_not_a_bot_funnel.likely_human_affected_requests,
            Some(1)
        );
    }

    #[test]
    fn event_append_refresh_writes_recent_events_tail_and_bootstrap_documents() {
        let store = MockStore::new();
        let now = crate::admin::now_ts();
        log_event(
            &store,
            &EventLogEntry {
                ts: now,
                event: EventType::Challenge,
                ip: Some("198.51.100.9".to_string()),
                reason: Some("example".to_string()),
                outcome: Some("served".to_string()),
                admin: None,
            },
        );
        refresh_after_event_append(&store, "default");

        let recent = read_document::<
            _,
            crate::observability::hot_read_documents::MonitoringRecentEventsTailPayload,
        >(
            &store,
            monitoring_recent_events_tail_document_key("default"),
            crate::observability::hot_read_documents::monitoring_recent_events_tail_document_contract()
                .schema_version,
        );
        assert!(recent.is_some());
        assert_eq!(recent.unwrap().payload.recent_events.len(), 1);
        let recent_sim_runs = read_document::<
            _,
            crate::observability::hot_read_documents::MonitoringRecentSimRunsPayload,
        >(
            &store,
            monitoring_recent_sim_runs_document_key("default"),
            crate::observability::hot_read_documents::monitoring_recent_sim_runs_document_contract()
                .schema_version,
        );
        assert!(recent_sim_runs.is_some());
        assert_eq!(recent_sim_runs.unwrap().payload.recent_sim_runs.len(), 0);
    }

    #[test]
    fn admin_mutation_refresh_rebuilds_supporting_summaries_and_bootstrap() {
        let store = MockStore::new();
        refresh_after_admin_mutation(&store, "default");
        assert!(read_document::<_, serde_json::Value>(
            &store,
            monitoring_security_privacy_summary_document_key("default"),
            crate::observability::hot_read_documents::monitoring_security_privacy_summary_document_contract()
                .schema_version,
        )
        .is_some());
        assert!(read_document::<_, crate::observability::retention::RetentionHealth>(
            &store,
            monitoring_retention_summary_document_key("default"),
            crate::observability::hot_read_documents::monitoring_retention_summary_document_contract()
                .schema_version,
        )
        .is_some());
    }

    #[test]
    fn retention_worker_refresh_writes_retention_summary_document() {
        let store = MockStore::new();
        refresh_after_retention_worker(&store, "default");
        assert!(read_document::<_, crate::observability::retention::RetentionHealth>(
            &store,
            monitoring_retention_summary_document_key("default"),
            crate::observability::hot_read_documents::monitoring_retention_summary_document_contract()
                .schema_version,
        )
        .is_some());
    }
}
