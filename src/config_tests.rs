// src/config_tests.rs
// Unit tests for config defaults and parsing

#[cfg(test)]
mod tests {
    use crate::challenge::KeyValueStore;
    use std::collections::HashMap;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };

    use once_cell::sync::Lazy;

    static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[derive(Default)]
    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl crate::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let m = self.map.lock().unwrap();
            Ok(m.get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.remove(key);
            Ok(())
        }
    }

    #[derive(Default)]
    struct CountingStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
        get_count: AtomicUsize,
    }

    impl CountingStore {
        fn get_count(&self) -> usize {
            self.get_count.load(Ordering::SeqCst)
        }
    }

    impl crate::challenge::KeyValueStore for CountingStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            self.get_count.fetch_add(1, Ordering::SeqCst);
            let m = self.map.lock().unwrap();
            Ok(m.get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut m = self.map.lock().unwrap();
            m.remove(key);
            Ok(())
        }
    }

    fn clear_env(keys: &[&str]) {
        for key in keys {
            std::env::remove_var(key);
        }
    }

    fn store_config_with_rate_limit(store: &CountingStore, rate_limit: u32) {
        let mut cfg = crate::config::defaults().clone();
        cfg.rate_limit = rate_limit;
        store
            .set("config:default", &serde_json::to_vec(&cfg).unwrap())
            .unwrap();
    }

    #[test]
    fn parse_challenge_threshold_defaults_to_3() {
        assert_eq!(crate::config::parse_challenge_threshold(None), 3);
    }

    #[test]
    fn parse_challenge_threshold_clamps_range() {
        assert_eq!(crate::config::parse_challenge_threshold(Some("0")), 1);
        assert_eq!(crate::config::parse_challenge_threshold(Some("99")), 10);
        assert_eq!(crate::config::parse_challenge_threshold(Some("5")), 5);
        assert_eq!(crate::config::parse_challenge_threshold(Some("junk")), 3);
    }

    #[test]
    fn parse_maze_threshold_clamps_range() {
        assert_eq!(crate::config::parse_maze_threshold(Some("0")), 1);
        assert_eq!(crate::config::parse_maze_threshold(Some("99")), 10);
        assert_eq!(crate::config::parse_maze_threshold(Some("6")), 6);
        assert_eq!(crate::config::parse_maze_threshold(Some("junk")), 6);
    }

    #[test]
    fn parse_botness_weight_clamps_range() {
        assert_eq!(crate::config::parse_botness_weight(Some("0"), 3), 0);
        assert_eq!(crate::config::parse_botness_weight(Some("11"), 3), 10);
        assert_eq!(crate::config::parse_botness_weight(Some("4"), 3), 4);
        assert_eq!(crate::config::parse_botness_weight(Some("junk"), 3), 3);
    }

    #[test]
    fn challenge_config_mutable_from_env_parses_values() {
        assert!(crate::config::challenge_config_mutable_from_env(Some("1")));
        assert!(crate::config::challenge_config_mutable_from_env(Some(
            "true"
        )));
        assert!(crate::config::challenge_config_mutable_from_env(Some(
            "TRUE"
        )));
        assert!(!crate::config::challenge_config_mutable_from_env(Some("0")));
        assert!(!crate::config::challenge_config_mutable_from_env(Some(
            "false"
        )));
        assert!(!crate::config::challenge_config_mutable_from_env(None));
    }

    #[test]
    fn parse_admin_config_write_defaults_to_disabled() {
        assert!(!crate::config::parse_admin_config_write_enabled(None));
        assert!(!crate::config::parse_admin_config_write_enabled(Some(
            "junk"
        )));
        assert!(crate::config::parse_admin_config_write_enabled(Some(
            "true"
        )));
        assert!(crate::config::parse_admin_config_write_enabled(Some("1")));
        assert!(!crate::config::parse_admin_config_write_enabled(Some(
            "false"
        )));
    }

    #[test]
    fn https_enforced_reads_required_env_bool() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var("SHUMA_ENFORCE_HTTPS", "false");
        assert!(!crate::config::https_enforced());

        std::env::set_var("SHUMA_ENFORCE_HTTPS", "true");
        assert!(crate::config::https_enforced());

        std::env::remove_var("SHUMA_ENFORCE_HTTPS");
    }

    #[test]
    fn forwarded_header_trust_configured_requires_non_empty_secret() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
        assert!(!crate::config::forwarded_header_trust_configured());

        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "");
        assert!(!crate::config::forwarded_header_trust_configured());

        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-secret");
        assert!(crate::config::forwarded_header_trust_configured());

        std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    }

    #[test]
    fn load_config_missing_returns_error() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let store = TestStore::default();
        let result = crate::config::Config::load(&store, "default");
        assert!(matches!(
            result,
            Err(crate::config::ConfigLoadError::MissingConfig)
        ));
    }

    #[test]
    fn load_config_reads_kv_only_without_tunable_env_overrides() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let keys = ["SHUMA_RATE_LIMIT", "SHUMA_HONEYPOTS"];
        clear_env(&keys);
        std::env::set_var("SHUMA_RATE_LIMIT", "222");
        std::env::set_var("SHUMA_HONEYPOTS", "[\"/trap-a\",\"/trap-b\"]");

        let store = TestStore::default();
        let mut kv_cfg = crate::config::defaults().clone();
        kv_cfg.rate_limit = 111;
        kv_cfg.honeypots = vec!["/kv-trap".to_string()];
        let key = "config:default".to_string();
        store
            .set(&key, &serde_json::to_vec(&kv_cfg).unwrap())
            .unwrap();

        let cfg = crate::config::Config::load(&store, "default").unwrap();
        assert_eq!(cfg.rate_limit, 111);
        assert_eq!(cfg.honeypots, vec!["/kv-trap".to_string()]);

        clear_env(&keys);
    }

    #[test]
    fn runtime_config_cache_hits_within_ttl() {
        let _lock = ENV_MUTEX.lock().unwrap();
        crate::config::clear_runtime_cache_for_tests();
        let store = CountingStore::default();
        store_config_with_rate_limit(&store, 101);

        let first =
            crate::config::load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
        let second =
            crate::config::load_runtime_cached_for_tests(&store, "default", 101, 2).unwrap();

        assert_eq!(first.rate_limit, 101);
        assert_eq!(second.rate_limit, 101);
        assert_eq!(store.get_count(), 1);
        crate::config::clear_runtime_cache_for_tests();
    }

    #[test]
    fn runtime_config_cache_refreshes_after_ttl() {
        let _lock = ENV_MUTEX.lock().unwrap();
        crate::config::clear_runtime_cache_for_tests();
        let store = CountingStore::default();
        store_config_with_rate_limit(&store, 111);

        let _ = crate::config::load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
        let _ = crate::config::load_runtime_cached_for_tests(&store, "default", 103, 2).unwrap();

        assert_eq!(store.get_count(), 2);
        crate::config::clear_runtime_cache_for_tests();
    }

    #[test]
    fn runtime_config_cache_invalidation_forces_reload() {
        let _lock = ENV_MUTEX.lock().unwrap();
        crate::config::clear_runtime_cache_for_tests();
        let store = CountingStore::default();
        store_config_with_rate_limit(&store, 120);

        let first =
            crate::config::load_runtime_cached_for_tests(&store, "default", 100, 2).unwrap();
        assert_eq!(first.rate_limit, 120);
        assert_eq!(store.get_count(), 1);

        store_config_with_rate_limit(&store, 220);
        crate::config::invalidate_runtime_cache("default");

        let refreshed =
            crate::config::load_runtime_cached_for_tests(&store, "default", 101, 2).unwrap();

        assert_eq!(refreshed.rate_limit, 220);
        assert_eq!(store.get_count(), 2);
        crate::config::clear_runtime_cache_for_tests();
    }
}
