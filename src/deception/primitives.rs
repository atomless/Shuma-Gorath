use once_cell::sync::Lazy;
use crate::maze::state::MazeStateStore;
use std::sync::Mutex;

static SHARED_BUDGET_STATE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub(crate) trait DeceptionStateStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()>;
}

impl<T> DeceptionStateStore for T
where
    T: MazeStateStore + ?Sized,
{
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        MazeStateStore::get(self, key)
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        MazeStateStore::set(self, key, value)
    }
}

pub(crate) struct SharedBudgetGovernor<'a> {
    pub global_active_key: &'a str,
    pub bucket_active_prefix: &'a str,
    pub bucket_catalog_key: Option<&'a str>,
    pub max_concurrent_global: u32,
    pub max_concurrent_per_ip_bucket: u32,
}

pub(crate) struct BudgetLease<'a, S: DeceptionStateStore + ?Sized> {
    store: &'a S,
    global_key: String,
    bucket_key: String,
    active: bool,
}

impl<'a, S: DeceptionStateStore + ?Sized> BudgetLease<'a, S> {
    pub(crate) fn release(&mut self) {
        if !self.active {
            return;
        }
        let _guard = SHARED_BUDGET_STATE_LOCK
            .lock()
            .expect("shared budget state lock poisoned");
        decrement_counter(self.store, self.global_key.as_str());
        decrement_counter(self.store, self.bucket_key.as_str());
        self.active = false;
    }
}

impl<S: DeceptionStateStore + ?Sized> Drop for BudgetLease<'_, S> {
    fn drop(&mut self) {
        self.release();
    }
}

pub(crate) fn read_counter(store: &(impl DeceptionStateStore + ?Sized), key: &str) -> u32 {
    store
        .get(key)
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(0)
}

pub(crate) fn write_counter(store: &(impl DeceptionStateStore + ?Sized), key: &str, value: u32) {
    if let Err(err) = store.set(key, value.to_string().as_bytes()) {
        eprintln!(
            "[deception] failed to persist counter key={} err={:?}",
            key, err
        );
    }
}

pub(crate) fn increment_counter(store: &(impl DeceptionStateStore + ?Sized), key: &str) -> u32 {
    let next = read_counter(store, key).saturating_add(1);
    write_counter(store, key, next);
    next
}

pub(crate) fn decrement_counter(store: &(impl DeceptionStateStore + ?Sized), key: &str) -> u32 {
    let current = read_counter(store, key);
    let next = current.saturating_sub(1);
    write_counter(store, key, next);
    next
}

pub(crate) fn budget_bucket_key(bucket_prefix: &str, ip_bucket: &str) -> String {
    format!("{}:{}", bucket_prefix, ip_bucket)
}

pub(crate) fn try_acquire_shared_budget<'a, S: DeceptionStateStore + ?Sized>(
    store: &'a S,
    governor: SharedBudgetGovernor<'_>,
    ip_bucket: &str,
) -> Option<BudgetLease<'a, S>> {
    try_acquire_shared_budget_with_hook(store, governor, ip_bucket, || {})
}

fn try_acquire_shared_budget_with_hook<'a, S, F>(
    store: &'a S,
    governor: SharedBudgetGovernor<'_>,
    ip_bucket: &str,
    before_increment: F,
) -> Option<BudgetLease<'a, S>>
where
    S: DeceptionStateStore + ?Sized,
    F: FnOnce(),
{
    if governor.max_concurrent_global == 0 || governor.max_concurrent_per_ip_bucket == 0 {
        return None;
    }

    let _guard = SHARED_BUDGET_STATE_LOCK
        .lock()
        .expect("shared budget state lock poisoned");
    let global = read_counter(store, governor.global_active_key);
    let bucket_key = budget_bucket_key(governor.bucket_active_prefix, ip_bucket);
    let bucket = read_counter(store, bucket_key.as_str());
    if global >= governor.max_concurrent_global || bucket >= governor.max_concurrent_per_ip_bucket
    {
        return None;
    }

    before_increment();
    increment_counter(store, governor.global_active_key);
    increment_counter(store, bucket_key.as_str());
    if let Some(catalog_key) = governor.bucket_catalog_key {
        if let Err(err) = crate::observability::key_catalog::register_key_with_deception_store(
            store,
            catalog_key,
            bucket_key.as_str(),
        ) {
            eprintln!(
                "[deception] failed to register budget bucket key={} catalog={} err={}",
                bucket_key, catalog_key, err
            );
        }
    }
    Some(BudgetLease {
        store,
        global_key: governor.global_active_key.to_string(),
        bucket_key,
        active: true,
    })
}

pub(crate) fn progression_replay_key(prefix: &str, flow_id: &str, operation_id: &str) -> String {
    format!("{}:{}:{}", prefix, flow_id, operation_id)
}

pub(crate) fn progression_issue_key(prefix: &str, flow_id: &str, operation_id: &str) -> String {
    format!("{}:{}:{}", prefix, flow_id, operation_id)
}

pub(crate) fn progression_chain_key(prefix: &str, flow_id: &str, digest: &str) -> String {
    format!("{}:{}:{}", prefix, flow_id, digest)
}

pub(crate) fn marker_seen(
    store: &(impl DeceptionStateStore + ?Sized),
    key: &str,
    now: u64,
) -> bool {
    let seen_until = store
        .get(key)
        .ok()
        .flatten()
        .and_then(|raw| String::from_utf8(raw).ok())
        .and_then(|raw| raw.parse::<u64>().ok());
    matches!(seen_until, Some(until) if now <= until)
}

pub(crate) fn mark_marker(
    store: &(impl DeceptionStateStore + ?Sized),
    key: &str,
    seen_until: u64,
) {
    if let Err(err) = store.set(key, seen_until.to_string().as_bytes()) {
        eprintln!(
            "[deception] failed to persist marker key={} err={:?}",
            key, err
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    use std::thread;
    use std::time::Duration;

    #[derive(Default)]
    struct FakeStore {
        values: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl MazeStateStore for FakeStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.values.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.values
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }
    }

    #[test]
    fn shared_budget_enforces_global_and_bucket_caps() {
        let store = FakeStore::default();
        let governor = SharedBudgetGovernor {
            global_active_key: "budget:global",
            bucket_active_prefix: "budget:bucket",
            bucket_catalog_key: None,
            max_concurrent_global: 1,
            max_concurrent_per_ip_bucket: 1,
        };
        let lease =
            try_acquire_shared_budget(&store, governor, "bucket-a").expect("first should acquire");
        assert!(
            try_acquire_shared_budget(
                &store,
                SharedBudgetGovernor {
                    global_active_key: "budget:global",
                    bucket_active_prefix: "budget:bucket",
                    bucket_catalog_key: None,
                    max_concurrent_global: 1,
                    max_concurrent_per_ip_bucket: 1,
                },
                "bucket-a"
            )
            .is_none()
        );
        drop(lease);
        assert!(
            try_acquire_shared_budget(
                &store,
                SharedBudgetGovernor {
                    global_active_key: "budget:global",
                    bucket_active_prefix: "budget:bucket",
                    bucket_catalog_key: None,
                    max_concurrent_global: 1,
                    max_concurrent_per_ip_bucket: 1,
                },
                "bucket-a"
            )
            .is_some()
        );
    }

    #[test]
    fn progression_keys_are_stable() {
        assert_eq!(
            progression_replay_key("maze:token:seen", "f1", "o2"),
            "maze:token:seen:f1:o2"
        );
        assert_eq!(
            progression_issue_key("maze:token:issue", "f1", "o2"),
            "maze:token:issue:f1:o2"
        );
        assert_eq!(
            progression_chain_key("maze:token:chain", "f1", "abc"),
            "maze:token:chain:f1:abc"
        );
    }

    #[test]
    fn shared_budget_parallel_acquire_stays_bounded() {
        let store = Arc::new(FakeStore::default());
        let acquired = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();

        for _ in 0..8 {
            let store = Arc::clone(&store);
            let acquired = Arc::clone(&acquired);
            handles.push(thread::spawn(move || {
                let governor = SharedBudgetGovernor {
                    global_active_key: "budget:global",
                    bucket_active_prefix: "budget:bucket",
                    bucket_catalog_key: None,
                    max_concurrent_global: 1,
                    max_concurrent_per_ip_bucket: 1,
                };
                let lease = try_acquire_shared_budget_with_hook(
                    store.as_ref(),
                    governor,
                    "bucket-a",
                    || {
                        thread::sleep(Duration::from_millis(15));
                    },
                );
                if let Some(mut lease) = lease {
                    acquired.fetch_add(1, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(20));
                    lease.release();
                }
            }));
        }

        for handle in handles {
            handle.join().expect("budget burst thread should finish");
        }

        assert_eq!(
            acquired.load(Ordering::SeqCst),
            1,
            "cap=1 budget burst should admit exactly one lease"
        );
        assert_eq!(read_counter(store.as_ref(), "budget:global"), 0);
        assert_eq!(read_counter(store.as_ref(), "budget:bucket:bucket-a"), 0);
    }
}
