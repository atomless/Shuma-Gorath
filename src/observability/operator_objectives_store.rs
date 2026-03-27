use crate::challenge::KeyValueStore;

use super::operator_snapshot_objectives::{
    default_operator_objectives, validate_operator_objectives, OperatorObjectivesProfile,
    HUMAN_ONLY_PRIVATE_OBJECTIVE_PROFILE_ID, OPERATOR_OBJECTIVES_SCHEMA_VERSION,
};

const OPERATOR_OBJECTIVES_PREFIX: &str = "operator_objectives:v1";
const LEGACY_SITE_DEFAULT_OBJECTIVE_PROFILE_ID: &str = "site_default_v1";

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
        if should_upgrade_legacy_seeded_default(&profile) {
            let upgraded = default_operator_objectives(generated_at_ts);
            if save_operator_objectives(store, site_id, &upgraded).is_ok() {
                return upgraded;
            }
            return upgraded;
        }
        if should_upgrade_runtime_dev_seeded_default(&profile) {
            let upgraded = runtime_dev_seeded_default_operator_objectives(generated_at_ts);
            if save_operator_objectives(store, site_id, &upgraded).is_ok() {
                return upgraded;
            }
            return upgraded;
        }
        return profile;
    }

    let profile = if crate::config::runtime_environment().is_dev() {
        runtime_dev_seeded_default_operator_objectives(generated_at_ts)
    } else {
        default_operator_objectives(generated_at_ts)
    };
    if save_operator_objectives(store, site_id, &profile).is_ok() {
        profile
    } else {
        // Keep snapshot materialization progressing even if the first persistence attempt fails.
        profile
    }
}

fn should_upgrade_legacy_seeded_default(profile: &OperatorObjectivesProfile) -> bool {
    profile.source == "seeded_default_profile"
        && profile.profile_id == LEGACY_SITE_DEFAULT_OBJECTIVE_PROFILE_ID
        && profile.profile_id != HUMAN_ONLY_PRIVATE_OBJECTIVE_PROFILE_ID
}

fn should_upgrade_runtime_dev_seeded_default(profile: &OperatorObjectivesProfile) -> bool {
    crate::config::runtime_environment().is_dev()
        && profile.source == "seeded_default_profile"
        && profile.rollout_guardrails.automated_apply_status == "manual_only"
}

fn runtime_dev_seeded_default_operator_objectives(updated_at_ts: u64) -> OperatorObjectivesProfile {
    let mut profile = default_operator_objectives(updated_at_ts);
    profile.rollout_guardrails.automated_apply_status = "canary_only".to_string();
    profile
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
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        let store = TestStore::new();

        let seeded = load_or_seed_operator_objectives(&store, "default", 1_700_000_000);
        let loaded = load_or_seed_operator_objectives(&store, "default", 1_700_000_100);

        assert_eq!(seeded.profile_id, "human_only_private");
        assert_eq!(seeded.revision, "rev-1700000000");
        assert_eq!(
            seeded.rollout_guardrails.automated_apply_status,
            "manual_only"
        );
        assert_eq!(loaded.revision, "rev-1700000000");
        assert!(store
            .get(&operator_objectives_key("default"))
            .expect("lookup succeeds")
            .is_some());
    }

    #[test]
    fn load_or_seed_operator_objectives_upgrades_legacy_seeded_mixed_site_profile() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        let store = TestStore::new();
        let mut legacy = default_operator_objectives(1_699_999_900);
        legacy.profile_id = "site_default_v1".to_string();

        save_operator_objectives(&store, "default", &legacy).expect("save succeeds");

        let loaded = load_or_seed_operator_objectives(&store, "default", 1_700_000_000);

        assert_eq!(loaded.profile_id, "human_only_private");
        assert_eq!(loaded.revision, "rev-1700000000");
        assert_eq!(
            load_operator_objectives(&store, "default")
                .expect("profile loads")
                .profile_id,
            "human_only_private"
        );
    }

    #[test]
    fn save_operator_objectives_round_trips_valid_profile() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("SHUMA_RUNTIME_ENV");
        let store = TestStore::new();
        let profile = default_operator_objectives(1_700_000_000);

        save_operator_objectives(&store, "default", &profile).expect("save succeeds");
        let loaded = load_operator_objectives(&store, "default").expect("profile loads");

        assert_eq!(loaded, profile);
    }

    #[test]
    fn load_or_seed_operator_objectives_seeds_canary_only_in_runtime_dev() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        let store = TestStore::new();

        let seeded = load_or_seed_operator_objectives(&store, "default", 1_700_000_000);

        assert_eq!(seeded.profile_id, "human_only_private");
        assert_eq!(
            seeded.rollout_guardrails.automated_apply_status,
            "canary_only"
        );
    }

    #[test]
    fn load_or_seed_operator_objectives_upgrades_existing_seeded_manual_only_profile_in_runtime_dev() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        let store = TestStore::new();
        let profile = default_operator_objectives(1_699_999_900);

        save_operator_objectives(&store, "default", &profile).expect("save succeeds");

        let loaded = load_or_seed_operator_objectives(&store, "default", 1_700_000_000);

        assert_eq!(loaded.revision, "rev-1700000000");
        assert_eq!(
            loaded.rollout_guardrails.automated_apply_status,
            "canary_only"
        );
    }

    #[test]
    fn load_or_seed_operator_objectives_does_not_override_operator_owned_manual_only_profile_in_runtime_dev() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        let store = TestStore::new();
        let mut profile = default_operator_objectives(1_699_999_900);
        profile.source = "admin_api".to_string();

        save_operator_objectives(&store, "default", &profile).expect("save succeeds");

        let loaded = load_or_seed_operator_objectives(&store, "default", 1_700_000_000);

        assert_eq!(loaded.revision, "rev-1699999900");
        assert_eq!(
            loaded.rollout_guardrails.automated_apply_status,
            "manual_only"
        );
        assert_eq!(loaded.source, "admin_api");
    }
}
