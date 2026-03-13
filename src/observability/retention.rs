use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const RETENTION_SCHEMA_VERSION: &str = "telemetry-retention.v1";
pub(crate) const RETENTION_DOMAIN_MONITORING: &str = "monitoring";
pub(crate) const RETENTION_DOMAIN_EVENTLOG: &str = "eventlog";
pub(crate) const RETENTION_DOMAIN_MONITORING_ROLLUP: &str = "monitoring_rollup";
const RETENTION_BUCKET_INDEX_PREFIX: &str = "telemetry:retention:v1:bucket";
const RETENTION_BUCKET_CATALOG_PREFIX: &str = "telemetry:retention:v1:catalog";
const RETENTION_WORKER_STATE_KEY: &str = "telemetry:retention:v1:worker_state";
const RETENTION_WORKER_CADENCE_SECONDS: u64 = 30;
const RETENTION_WORKER_BATCH_BUCKET_BUDGET: usize = 8;
const RETENTION_LAG_DEGRADED_THRESHOLD_HOURS: f64 = 1.0;
const EVENT_LOG_HIGH_RISK_RETENTION_MAX_HOURS: u64 = 72;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TelemetryBucketIndex {
    schema_version: String,
    bucket_id: String,
    domain: String,
    window_start: u64,
    window_end: u64,
    record_count: u64,
    state: String,
    keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TelemetryBucketCatalog {
    schema_version: String,
    domain: String,
    hours: Vec<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BucketWindowStats {
    pub bucket_hours: Vec<u64>,
    pub bucket_count: u64,
    pub key_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetentionWorkerState {
    schema_version: String,
    last_purged_bucket: String,
    last_attempt_ts: u64,
    last_success_ts: u64,
    pending_expired_buckets: u64,
    oldest_retained_ts: u64,
    last_error: Option<String>,
    last_error_code: Option<String>,
}

impl Default for RetentionWorkerState {
    fn default() -> Self {
        Self {
            schema_version: RETENTION_SCHEMA_VERSION.to_string(),
            last_purged_bucket: String::new(),
            last_attempt_ts: 0,
            last_success_ts: 0,
            pending_expired_buckets: 0,
            oldest_retained_ts: 0,
            last_error: None,
            last_error_code: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RetentionHealth {
    pub retention_hours: u64,
    pub high_risk_retention_hours: u64,
    pub high_risk_retention_max_hours: u64,
    pub redacted_summary_retention_hours: u64,
    pub oldest_retained_ts: u64,
    pub purge_lag_hours: f64,
    pub pending_expired_buckets: u64,
    pub last_purge_success_ts: u64,
    pub last_attempt_ts: u64,
    pub last_purged_bucket: String,
    pub last_error: Option<String>,
    pub state: String,
    pub guidance: String,
    pub bucket_schema: Vec<String>,
}

pub(crate) fn monitoring_retention_hours() -> u64 {
    crate::config::monitoring_retention_hours()
}

pub(crate) fn event_log_high_risk_retention_hours() -> u64 {
    crate::config::event_log_retention_hours().min(EVENT_LOG_HIGH_RISK_RETENTION_MAX_HOURS)
}

pub(crate) fn monitoring_rollup_retention_hours() -> u64 {
    crate::config::monitoring_rollup_retention_hours()
}

fn retention_hours_for_domain(domain: &str) -> u64 {
    match domain {
        RETENTION_DOMAIN_EVENTLOG => event_log_high_risk_retention_hours(),
        RETENTION_DOMAIN_MONITORING_ROLLUP => monitoring_rollup_retention_hours(),
        _ => monitoring_retention_hours(),
    }
}

fn cutoff_hour_for_domain(now_hour: u64, domain: &str) -> u64 {
    now_hour.saturating_sub(retention_hours_for_domain(domain))
}

fn bucket_id(domain: &str, hour: u64) -> String {
    format!("{}:{}", domain, hour)
}

fn bucket_index_key(domain: &str, hour: u64) -> String {
    format!("{}:{}:{}", RETENTION_BUCKET_INDEX_PREFIX, domain, hour)
}

fn bucket_catalog_key(domain: &str) -> String {
    format!("{}:{}", RETENTION_BUCKET_CATALOG_PREFIX, domain)
}

fn read_json<T: for<'de> Deserialize<'de>>(store: &impl crate::challenge::KeyValueStore, key: &str) -> Option<T> {
    store
        .get(key)
        .ok()
        .flatten()
        .and_then(|value| serde_json::from_slice::<T>(value.as_slice()).ok())
}

fn write_json<T: Serialize>(
    store: &impl crate::challenge::KeyValueStore,
    key: &str,
    value: &T,
) -> Result<(), String> {
    let payload = serde_json::to_vec(value).map_err(|_| "serialize_error".to_string())?;
    store
        .set(key, payload.as_slice())
        .map_err(|_| "kv_write_error".to_string())
}

fn load_bucket_index(
    store: &impl crate::challenge::KeyValueStore,
    domain: &str,
    hour: u64,
) -> Option<TelemetryBucketIndex> {
    read_json::<TelemetryBucketIndex>(store, bucket_index_key(domain, hour).as_str())
}

fn save_bucket_index(
    store: &impl crate::challenge::KeyValueStore,
    index: &TelemetryBucketIndex,
    hour: u64,
) -> Result<(), String> {
    write_json(store, bucket_index_key(index.domain.as_str(), hour).as_str(), index)
}

fn load_bucket_catalog(
    store: &impl crate::challenge::KeyValueStore,
    domain: &str,
) -> TelemetryBucketCatalog {
    read_json::<TelemetryBucketCatalog>(store, bucket_catalog_key(domain).as_str()).unwrap_or(
        TelemetryBucketCatalog {
            schema_version: RETENTION_SCHEMA_VERSION.to_string(),
            domain: domain.to_string(),
            hours: Vec::new(),
        },
    )
}

fn save_bucket_catalog(
    store: &impl crate::challenge::KeyValueStore,
    catalog: &TelemetryBucketCatalog,
) -> Result<(), String> {
    write_json(
        store,
        bucket_catalog_key(catalog.domain.as_str()).as_str(),
        catalog,
    )
}

fn load_worker_state(store: &impl crate::challenge::KeyValueStore) -> RetentionWorkerState {
    read_json::<RetentionWorkerState>(store, RETENTION_WORKER_STATE_KEY).unwrap_or_default()
}

fn save_worker_state(
    store: &impl crate::challenge::KeyValueStore,
    state: &RetentionWorkerState,
) -> Result<(), String> {
    write_json(store, RETENTION_WORKER_STATE_KEY, state)
}

fn insert_hour_unique_sorted(hours: &mut Vec<u64>, hour: u64) {
    if hours.binary_search(&hour).is_err() {
        hours.push(hour);
        hours.sort_unstable();
    }
}

pub(crate) fn bucket_window_stats(
    store: &impl crate::challenge::KeyValueStore,
    domain: &str,
    start_hour: u64,
    end_hour: u64,
) -> BucketWindowStats {
    if start_hour > end_hour {
        return BucketWindowStats::default();
    }

    let catalog = load_bucket_catalog(store, domain);
    let bucket_hours: Vec<u64> = catalog
        .hours
        .into_iter()
        .filter(|hour| *hour >= start_hour && *hour <= end_hour)
        .collect();

    let mut key_count = 0u64;
    for hour in &bucket_hours {
        if let Some(index) = load_bucket_index(store, domain, *hour) {
            let unique_keys: BTreeSet<String> = index.keys.into_iter().collect();
            key_count = key_count.saturating_add(unique_keys.len() as u64);
        }
    }

    BucketWindowStats {
        bucket_count: bucket_hours.len() as u64,
        key_count,
        bucket_hours,
    }
}

pub(crate) fn bucket_window_keys(
    store: &impl crate::challenge::KeyValueStore,
    domain: &str,
    start_hour: u64,
    end_hour: u64,
) -> Vec<String> {
    let stats = bucket_window_stats(store, domain, start_hour, end_hour);
    let mut unique_keys = BTreeSet::new();
    for hour in stats.bucket_hours {
        if let Some(index) = load_bucket_index(store, domain, hour) {
            unique_keys.extend(index.keys.into_iter());
        }
    }
    unique_keys.into_iter().collect()
}

fn register_bucket_key(
    store: &impl crate::challenge::KeyValueStore,
    domain: &str,
    hour: u64,
    key: &str,
    increment: u64,
) -> Result<(), String> {
    let mut index = load_bucket_index(store, domain, hour).unwrap_or(TelemetryBucketIndex {
        schema_version: RETENTION_SCHEMA_VERSION.to_string(),
        bucket_id: bucket_id(domain, hour),
        domain: domain.to_string(),
        window_start: hour.saturating_mul(3600),
        window_end: hour.saturating_add(1).saturating_mul(3600).saturating_sub(1),
        record_count: 0,
        state: "active".to_string(),
        keys: Vec::new(),
    });
    if !index.keys.iter().any(|item| item == key) {
        index.keys.push(key.to_string());
    }
    index.record_count = index.record_count.saturating_add(increment);
    index.state = "active".to_string();
    save_bucket_index(store, &index, hour)?;

    let mut catalog = load_bucket_catalog(store, domain);
    insert_hour_unique_sorted(&mut catalog.hours, hour);
    save_bucket_catalog(store, &catalog)?;
    Ok(())
}

pub(crate) fn register_event_log_key(
    store: &impl crate::challenge::KeyValueStore,
    hour: u64,
    key: &str,
) {
    if let Err(err) = register_bucket_key(store, RETENTION_DOMAIN_EVENTLOG, hour, key, 1) {
        eprintln!(
            "[telemetry-retention] failed to register eventlog key domain={} hour={} key={} error={}",
            RETENTION_DOMAIN_EVENTLOG, hour, key, err
        );
    }
}

pub(crate) fn register_monitoring_key(
    store: &impl crate::challenge::KeyValueStore,
    hour: u64,
    key: &str,
) {
    if let Err(err) = register_bucket_key(store, RETENTION_DOMAIN_MONITORING, hour, key, 1) {
        eprintln!(
            "[telemetry-retention] failed to register monitoring key domain={} hour={} key={} error={}",
            RETENTION_DOMAIN_MONITORING, hour, key, err
        );
    }
}

pub(crate) fn register_monitoring_rollup_key(
    store: &impl crate::challenge::KeyValueStore,
    hour: u64,
    key: &str,
) {
    if let Err(err) = register_bucket_key(store, RETENTION_DOMAIN_MONITORING_ROLLUP, hour, key, 1) {
        eprintln!(
            "[telemetry-retention] failed to register monitoring rollup key domain={} hour={} key={} error={}",
            RETENTION_DOMAIN_MONITORING_ROLLUP, hour, key, err
        );
    }
}

fn purge_bucket(
    store: &impl crate::challenge::KeyValueStore,
    domain: &str,
    hour: u64,
) -> Result<(), String> {
    let Some(index) = load_bucket_index(store, domain, hour) else {
        return Ok(());
    };
    let unique_keys: BTreeSet<String> = index.keys.into_iter().collect();
    for key in unique_keys {
        store
            .delete(key.as_str())
            .map_err(|_| format!("delete_key_failed:{key}"))?;
    }
    store
        .delete(bucket_index_key(domain, hour).as_str())
        .map_err(|_| format!("delete_bucket_index_failed:{}:{}", domain, hour))?;
    Ok(())
}

fn catalog_oldest_retained_hour(catalogs: &[TelemetryBucketCatalog]) -> Option<u64> {
    catalogs
        .iter()
        .flat_map(|catalog| catalog.hours.iter().copied())
        .min()
}

fn catalog_pending_expired_buckets(catalogs: &[TelemetryBucketCatalog], now_hour: u64) -> u64 {
    catalogs
        .iter()
        .map(|catalog| {
            let cutoff_hour = cutoff_hour_for_domain(now_hour, catalog.domain.as_str());
            catalog
                .hours
                .iter()
                .filter(|hour| **hour < cutoff_hour)
                .count() as u64
        })
        .sum()
}

fn compute_health(
    store: &impl crate::challenge::KeyValueStore,
    now: u64,
) -> RetentionHealth {
    let state = load_worker_state(store);
    let now_hour = now / 3600;
    let retention_hours = monitoring_retention_hours();
    let high_risk_retention_hours = event_log_high_risk_retention_hours();
    let catalogs = vec![
        load_bucket_catalog(store, RETENTION_DOMAIN_MONITORING),
        load_bucket_catalog(store, RETENTION_DOMAIN_EVENTLOG),
        load_bucket_catalog(store, RETENTION_DOMAIN_MONITORING_ROLLUP),
    ];
    let pending_expired_buckets = catalog_pending_expired_buckets(&catalogs, now_hour);
    let oldest_retained_hour = catalog_oldest_retained_hour(&catalogs).unwrap_or(0);
    let purge_lag_hours = catalogs
        .iter()
        .map(|catalog| {
            let cutoff_hour = cutoff_hour_for_domain(now_hour, catalog.domain.as_str());
            let domain_oldest_expired = catalog
                .hours
                .iter()
                .copied()
                .filter(|hour| *hour < cutoff_hour)
                .min();
            domain_oldest_expired
                .map(|hour| cutoff_hour.saturating_sub(hour) as f64)
                .unwrap_or(0.0)
        })
        .fold(0.0f64, f64::max);
    let oldest_retained_ts = if oldest_retained_hour == 0 {
        state.oldest_retained_ts
    } else {
        oldest_retained_hour.saturating_mul(3600)
    };

    let state_label = if state.last_error.is_some() {
        "stalled"
    } else if pending_expired_buckets > 0 || purge_lag_hours > RETENTION_LAG_DEGRADED_THRESHOLD_HOURS {
        "degraded"
    } else {
        "healthy"
    };
    let guidance = match state_label {
        "healthy" => "Retention worker healthy; no expired telemetry buckets pending purge.",
        "degraded" => {
            "Retention lag detected; verify purge worker cadence and investigate pending expired buckets."
        }
        _ => "Retention worker stalled; inspect last_error and resolve purge failures before relying on retention guarantees.",
    };

    RetentionHealth {
        retention_hours,
        high_risk_retention_hours,
        high_risk_retention_max_hours: EVENT_LOG_HIGH_RISK_RETENTION_MAX_HOURS,
        redacted_summary_retention_hours: retention_hours,
        oldest_retained_ts,
        purge_lag_hours,
        pending_expired_buckets,
        last_purge_success_ts: state.last_success_ts,
        last_attempt_ts: state.last_attempt_ts,
        last_purged_bucket: state.last_purged_bucket,
        last_error: state.last_error,
        state: state_label.to_string(),
        guidance: guidance.to_string(),
        bucket_schema: vec![
            "bucket_id".to_string(),
            "window_start".to_string(),
            "window_end".to_string(),
            "record_count".to_string(),
            "state".to_string(),
        ],
    }
}

fn run_worker_if_due_at(store: &impl crate::challenge::KeyValueStore, now: u64) -> bool {
    let retention_hours = monitoring_retention_hours();
    if retention_hours == 0 {
        return false;
    }
    let mut state = load_worker_state(store);
    if now.saturating_sub(state.last_attempt_ts) < RETENTION_WORKER_CADENCE_SECONDS {
        return false;
    }
    state.last_attempt_ts = now;
    state.last_error = None;
    state.last_error_code = None;

    let now_hour = now / 3600;
    let mut catalogs = vec![
        load_bucket_catalog(store, RETENTION_DOMAIN_MONITORING),
        load_bucket_catalog(store, RETENTION_DOMAIN_EVENTLOG),
        load_bucket_catalog(store, RETENTION_DOMAIN_MONITORING_ROLLUP),
    ];

    let mut processed = 0usize;
    for catalog in catalogs.iter_mut() {
        let domain = catalog.domain.clone();
        let cutoff_hour = cutoff_hour_for_domain(now_hour, domain.as_str());
        let expired_hours: Vec<u64> = catalog
            .hours
            .iter()
            .copied()
            .filter(|hour| *hour < cutoff_hour)
            .collect();
        for hour in expired_hours {
            if processed >= RETENTION_WORKER_BATCH_BUCKET_BUDGET {
                break;
            }
            match purge_bucket(store, domain.as_str(), hour) {
                Ok(()) => {
                    catalog.hours.retain(|value| *value != hour);
                    state.last_purged_bucket = bucket_id(domain.as_str(), hour);
                    processed = processed.saturating_add(1);
                }
                Err(error_code) => {
                    state.last_error =
                        Some(format!("domain={} bucket={} error={}", domain, hour, error_code));
                    state.last_error_code = Some(error_code);
                    break;
                }
            }
        }
        if state.last_error.is_some() || processed >= RETENTION_WORKER_BATCH_BUCKET_BUDGET {
            break;
        }
    }

    let mut save_failed = false;
    for catalog in catalogs.iter() {
        if let Err(err) = save_bucket_catalog(store, catalog) {
            state.last_error = Some(format!(
                "domain={} error=saving_catalog_failed:{}",
                catalog.domain, err
            ));
            state.last_error_code = Some("save_catalog_failed".to_string());
            save_failed = true;
            break;
        }
    }
    if !save_failed && processed > 0 && state.last_error.is_none() {
        state.last_success_ts = now;
    }

    state.pending_expired_buckets = catalog_pending_expired_buckets(&catalogs, now_hour);
    state.oldest_retained_ts = catalog_oldest_retained_hour(&catalogs)
        .map(|hour| hour.saturating_mul(3600))
        .unwrap_or(0);
    if let Err(err) = save_worker_state(store, &state) {
        eprintln!(
            "[telemetry-retention] failed writing worker state key={} error={}",
            RETENTION_WORKER_STATE_KEY, err
        );
    }
    true
}

pub(crate) fn run_worker_if_due(store: &impl crate::challenge::KeyValueStore) {
    if run_worker_if_due_at(store, crate::admin::now_ts()) {
        crate::observability::hot_read_projection::refresh_after_retention_worker(store, "default");
    }
}

pub(crate) fn retention_health(store: &impl crate::challenge::KeyValueStore) -> RetentionHealth {
    compute_health(store, crate::admin::now_ts())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenge::KeyValueStore;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
        fail_delete_key: Mutex<Option<String>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                map: Mutex::new(HashMap::new()),
                fail_delete_key: Mutex::new(None),
            }
        }

        fn fail_delete_for(&self, key: &str) {
            *self.fail_delete_key.lock().expect("fail-delete lock") = Some(key.to_string());
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
            let mut fail_key = self.fail_delete_key.lock().expect("fail-delete lock");
            if fail_key.as_deref() == Some(key) {
                *fail_key = None;
                return Err(());
            }
            self.map.lock().expect("map lock").remove(key);
            Ok(())
        }

        fn get_keys(&self) -> Result<Vec<String>, ()> {
            Ok(self
                .map
                .lock()
                .expect("map lock")
                .keys()
                .cloned()
                .collect())
        }
    }

    #[test]
    fn register_bucket_key_writes_schema_fields() {
        let store = MockStore::new();
        let hour = 42u64;
        register_event_log_key(&store, hour, "eventlog:v2:42:1-a");

        let index: TelemetryBucketIndex = read_json(&store, bucket_index_key("eventlog", hour).as_str())
            .expect("bucket index");
        assert_eq!(index.schema_version, RETENTION_SCHEMA_VERSION);
        assert_eq!(index.bucket_id, "eventlog:42");
        assert_eq!(index.window_start, 42 * 3600);
        assert_eq!(index.window_end, (43 * 3600) - 1);
        assert_eq!(index.record_count, 1);
        assert_eq!(index.state, "active");
        assert_eq!(index.keys, vec!["eventlog:v2:42:1-a".to_string()]);
    }

    #[test]
    fn eventlog_retention_is_capped_while_monitoring_retention_tracks_config() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "240");
        std::env::set_var("SHUMA_MONITORING_RETENTION_HOURS", "336");
        std::env::set_var("SHUMA_MONITORING_ROLLUP_RETENTION_HOURS", "720");
        assert_eq!(monitoring_retention_hours(), 336);
        assert_eq!(event_log_high_risk_retention_hours(), 72);
        assert_eq!(monitoring_rollup_retention_hours(), 720);
        assert_eq!(
            retention_hours_for_domain(RETENTION_DOMAIN_MONITORING),
            336
        );
        assert_eq!(retention_hours_for_domain(RETENTION_DOMAIN_EVENTLOG), 72);
        assert_eq!(
            retention_hours_for_domain(RETENTION_DOMAIN_MONITORING_ROLLUP),
            720
        );
        std::env::remove_var("SHUMA_MONITORING_ROLLUP_RETENTION_HOURS");
        std::env::remove_var("SHUMA_MONITORING_RETENTION_HOURS");
        std::env::remove_var("SHUMA_EVENT_LOG_RETENTION_HOURS");
    }

    #[test]
    fn worker_applies_domain_specific_retention_windows() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "240");

        let store = MockStore::new();
        let now_hour = 10_000u64;
        let stale_hour = now_hour.saturating_sub(100);

        let eventlog_key = format!("eventlog:v2:{}:1-stale", stale_hour);
        let monitoring_key = format!("monitoring:v1:{}:stale", stale_hour);
        store
            .set(eventlog_key.as_str(), br#"{"eventlog":true}"#)
            .expect("set eventlog stale");
        store
            .set(monitoring_key.as_str(), br#"{"monitoring":true}"#)
            .expect("set monitoring stale");
        register_event_log_key(&store, stale_hour, eventlog_key.as_str());
        register_monitoring_key(&store, stale_hour, monitoring_key.as_str());

        run_worker_if_due_at(&store, now_hour * 3600);

        assert!(store.get(eventlog_key.as_str()).expect("get eventlog").is_none());
        assert!(store
            .get(monitoring_key.as_str())
            .expect("get monitoring")
            .is_some());

        std::env::remove_var("SHUMA_EVENT_LOG_RETENTION_HOURS");
    }

    #[test]
    fn worker_purges_expired_bucket_and_updates_watermark() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "2");

        let store = MockStore::new();
        let now_hour = 10_000u64;
        let stale_hour = now_hour.saturating_sub(5);
        let retained_hour = now_hour.saturating_sub(1);
        let stale_key = format!("eventlog:v2:{}:1-stale", stale_hour);
        let retained_key = format!("eventlog:v2:{}:1-retained", retained_hour);
        store
            .set(stale_key.as_str(), br#"{"stale":true}"#)
            .expect("set stale");
        store
            .set(retained_key.as_str(), br#"{"retained":true}"#)
            .expect("set retained");
        register_event_log_key(&store, stale_hour, stale_key.as_str());
        register_event_log_key(&store, retained_hour, retained_key.as_str());

        run_worker_if_due_at(&store, now_hour * 3600);

        assert!(store.get(stale_key.as_str()).expect("get stale").is_none());
        assert!(store.get(retained_key.as_str()).expect("get retained").is_some());
        let state = load_worker_state(&store);
        assert_eq!(state.last_purged_bucket, format!("eventlog:{stale_hour}"));
        assert!(state.last_success_ts > 0);

        std::env::remove_var("SHUMA_EVENT_LOG_RETENTION_HOURS");
    }

    #[test]
    fn worker_partial_failure_is_retry_safe() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "1");

        let store = MockStore::new();
        let now_hour = 30_000u64;
        let stale_hour = now_hour.saturating_sub(4);
        let stale_key = format!("eventlog:v2:{}:1-stale", stale_hour);
        store
            .set(stale_key.as_str(), br#"{"stale":true}"#)
            .expect("set stale");
        register_event_log_key(&store, stale_hour, stale_key.as_str());
        store.fail_delete_for(stale_key.as_str());

        run_worker_if_due_at(&store, now_hour * 3600);
        let failed_state = load_worker_state(&store);
        assert!(failed_state.last_error.is_some());
        assert!(store.get(stale_key.as_str()).expect("get stale").is_some());

        run_worker_if_due_at(&store, (now_hour * 3600) + RETENTION_WORKER_CADENCE_SECONDS + 1);
        let recovered_state = load_worker_state(&store);
        assert!(recovered_state.last_error.is_none());
        assert!(store.get(stale_key.as_str()).expect("get stale").is_none());

        std::env::remove_var("SHUMA_EVENT_LOG_RETENTION_HOURS");
    }

    #[test]
    fn run_worker_if_due_refreshes_hot_read_retention_projection() {
        let store = MockStore::new();
        run_worker_if_due(&store);

        let key =
            crate::observability::hot_read_documents::monitoring_retention_summary_document_key(
                "default",
            );
        let bytes = store.get(key.as_str()).expect("retention doc read");
        assert!(bytes.is_some());
    }

    #[test]
    fn retention_health_reports_degraded_with_pending_expired_buckets() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_EVENT_LOG_RETENTION_HOURS", "1");

        let store = MockStore::new();
        let now_hour = 25_000u64;
        let stale_hour = now_hour.saturating_sub(3);
        let stale_key = format!("eventlog:v2:{}:1-stale", stale_hour);
        store
            .set(stale_key.as_str(), br#"{"stale":true}"#)
            .expect("set stale");
        register_event_log_key(&store, stale_hour, stale_key.as_str());

        let health = compute_health(&store, now_hour * 3600);
        assert_eq!(health.state, "degraded");
        assert!(health.pending_expired_buckets > 0);
        assert!(health.purge_lag_hours > 0.0);
        assert_eq!(
            health.bucket_schema,
            vec![
                "bucket_id".to_string(),
                "window_start".to_string(),
                "window_end".to_string(),
                "record_count".to_string(),
                "state".to_string()
            ]
        );

        std::env::remove_var("SHUMA_EVENT_LOG_RETENTION_HOURS");
    }
}
