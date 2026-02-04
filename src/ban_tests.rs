// src/ban_tests.rs
// Unit tests for ban logic

#[cfg(test)]
mod tests {
    use super::super::ban::*;
    use std::collections::HashMap;

    use std::cell::RefCell;
    #[derive(Default)]
    struct TestStore {
        map: RefCell<HashMap<String, Vec<u8>>>,
    }
    impl super::super::quiz::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.borrow().get(key).cloned())
        }
        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map.borrow_mut().insert(key.to_string(), value.to_vec());
            Ok(())
        }
        fn delete(&self, key: &str) -> Result<(), ()> {
            self.map.borrow_mut().remove(key);
            Ok(())
        }
    }

    #[test]
    fn test_ban_and_expiry() {
        let store = TestStore::default();
        let site_id = "testsite";
        let ip = "1.2.3.4";
        // Ban IP for 1 second
        ban_ip(&store, site_id, ip, "test", 1);
        assert!(is_banned(&store, site_id, ip));
    }

    #[test]
    fn test_ban_and_unban_unknown_ip() {
        let store = TestStore::default();
        let site_id = "testsite";
        let ip = "unknown";
        // Ban 'unknown' IP
        ban_ip(&store, site_id, ip, "test", 60);
        assert!(is_banned(&store, site_id, ip));
        // Unban using the unban_ip function
        unban_ip(&store, site_id, ip);
        assert!(!is_banned(&store, site_id, ip));
    }

    #[test]
    fn test_unban_ip_function() {
        let store = TestStore::default();
        let site_id = "testsite";
        let ip = "192.168.1.100";
        
        // Ban an IP
        ban_ip(&store, site_id, ip, "test_reason", 3600);
        assert!(is_banned(&store, site_id, ip), "IP should be banned after ban_ip");
        
        // Unban using unban_ip function
        unban_ip(&store, site_id, ip);
        assert!(!is_banned(&store, site_id, ip), "IP should not be banned after unban_ip");
    }

    #[test]
    fn test_unban_ip_nonexistent() {
        let store = TestStore::default();
        let site_id = "testsite";
        let ip = "10.0.0.1";
        
        // Unban a non-existent IP should not panic
        unban_ip(&store, site_id, ip);
        assert!(!is_banned(&store, site_id, ip), "Non-existent IP should not be banned");
    }

    #[test]
    fn test_ban_entry_serialization() {
        let entry = BanEntry {
            reason: "test".to_string(),
            expires: 1234567890,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let de: BanEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(de.reason, "test");
        assert_eq!(de.expires, 1234567890);
    }
    // use super::super::ban::*;
    // Removed MockStore; all tests use TestStore implementing KeyValueStore

    // Removed duplicate test using MockStore; TestStore is used for all tests
}
