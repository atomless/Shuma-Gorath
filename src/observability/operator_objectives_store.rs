use crate::challenge::KeyValueStore;

use super::operator_snapshot_objectives::{
    default_operator_objectives, validate_operator_objectives, OperatorObjectivesProfile,
    OPERATOR_OBJECTIVES_SCHEMA_VERSION,
};

const OPERATOR_OBJECTIVES_PREFIX: &str = "operator_objectives:v1";

pub(crate) fn operator_objectives_key(site_id: &str) -> String {
    format!("{OPERATOR_OBJECTIVES_PREFIX}:{site_id}")
}

pub(crate) fn load_operator_objectives<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> Option<OperatorObjectivesProfile> {
    let bytes = store
        .get(&operator_objectives_key(site_id))
        .ok()
        .flatten()?;
    let profile = serde_json::from_slice::<OperatorObjectivesProfile>(bytes.as_slice()).ok()?;
    if profile.schema_version != OPERATOR_OBJECTIVES_SCHEMA_VERSION {
        return None;
    }
    validate_operator_objectives(&profile).ok()?;
    Some(profile)
}

pub(crate) fn load_or_seed_operator_objectives<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
) -> OperatorObjectivesProfile {
    if let Some(profile) = load_operator_objectives(store, site_id) {
        return profile;
    }

    let profile = default_operator_objectives(generated_at_ts);
    if save_operator_objectives(store, site_id, &profile).is_ok() {
        profile
    } else {
        // Keep snapshot materialization progressing even if the first persistence attempt fails.
        profile
    }
}

pub(crate) fn save_operator_objectives<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    profile: &OperatorObjectivesProfile,
) -> Result<(), ()> {
    validate_operator_objectives(profile).map_err(|_| ())?;
    let payload = serde_json::to_vec(profile).map_err(|_| ())?;
    store
        .set(&operator_objectives_key(site_id), payload.as_slice())
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::{
        load_operator_objectives, load_or_seed_operator_objectives, operator_objectives_key,
        save_operator_objectives,
    };
    use crate::challenge::KeyValueStore;
    use crate::observability::operator_snapshot_objectives::default_operator_objectives;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl TestStore {
        fn new() -> Self {
            Self {
                map: Mutex::new(HashMap::new()),
            }
        }
    }

    impl KeyValueStore for TestStore {
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
    fn load_or_seed_operator_objectives_persists_default_profile_once() {
        let store = TestStore::new();

        let seeded = load_or_seed_operator_objectives(&store, "default", 1_700_000_000);
        let loaded = load_or_seed_operator_objectives(&store, "default", 1_700_000_100);

        assert_eq!(seeded.profile_id, "site_default_v1");
        assert_eq!(seeded.revision, "rev-1700000000");
        assert_eq!(loaded.revision, "rev-1700000000");
        assert!(store
            .get(&operator_objectives_key("default"))
            .expect("lookup succeeds")
            .is_some());
    }

    #[test]
    fn save_operator_objectives_round_trips_valid_profile() {
        let store = TestStore::new();
        let profile = default_operator_objectives(1_700_000_000);

        save_operator_objectives(&store, "default", &profile).expect("save succeeds");
        let loaded = load_operator_objectives(&store, "default").expect("profile loads");

        assert_eq!(loaded, profile);
    }
}
