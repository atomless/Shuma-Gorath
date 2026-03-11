use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const KEY_CATALOG_SCHEMA_VERSION: &str = "telemetry-key-catalog.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyCatalog {
    schema_version: String,
    keys: Vec<String>,
}

impl Default for KeyCatalog {
    fn default() -> Self {
        Self {
            schema_version: KEY_CATALOG_SCHEMA_VERSION.to_string(),
            keys: Vec::new(),
        }
    }
}

fn read_catalog(
    get_value: &mut dyn FnMut(&str) -> Result<Option<Vec<u8>>, ()>,
    catalog_key: &str,
) -> KeyCatalog {
    get_value(catalog_key)
        .ok()
        .flatten()
        .and_then(|value| serde_json::from_slice::<KeyCatalog>(value.as_slice()).ok())
        .unwrap_or_default()
}

fn write_catalog(
    set_value: &mut dyn FnMut(&str, &[u8]) -> Result<(), ()>,
    catalog_key: &str,
    catalog: &KeyCatalog,
) -> Result<(), String> {
    let payload = serde_json::to_vec(catalog).map_err(|_| "serialize_error".to_string())?;
    set_value(catalog_key, payload.as_slice())
        .map_err(|_| "kv_write_error".to_string())
}

fn register_key_with_io(
    get_value: &mut dyn FnMut(&str) -> Result<Option<Vec<u8>>, ()>,
    set_value: &mut dyn FnMut(&str, &[u8]) -> Result<(), ()>,
    catalog_key: &str,
    key: &str,
) -> Result<(), String> {
    let mut catalog = read_catalog(get_value, catalog_key);
    if catalog.keys.iter().any(|item| item == key) {
        return Ok(());
    }
    catalog.keys.push(key.to_string());
    catalog.keys.sort();
    write_catalog(set_value, catalog_key, &catalog)
}

fn list_keys_with_io(
    get_value: &mut dyn FnMut(&str) -> Result<Option<Vec<u8>>, ()>,
    catalog_key: &str,
) -> Vec<String> {
    let catalog = read_catalog(get_value, catalog_key);
    let unique: BTreeSet<String> = catalog.keys.into_iter().collect();
    unique.into_iter().collect()
}

pub(crate) fn register_key(
    store: &(impl crate::challenge::KeyValueStore + ?Sized),
    catalog_key: &str,
    key: &str,
) -> Result<(), String> {
    let mut get_value = |lookup_key: &str| crate::challenge::KeyValueStore::get(store, lookup_key);
    let mut set_value =
        |lookup_key: &str, value: &[u8]| crate::challenge::KeyValueStore::set(store, lookup_key, value);
    register_key_with_io(&mut get_value, &mut set_value, catalog_key, key)
}

pub(crate) fn register_key_with_deception_store(
    store: &(impl crate::deception::primitives::DeceptionStateStore + ?Sized),
    catalog_key: &str,
    key: &str,
) -> Result<(), String> {
    let mut get_value =
        |lookup_key: &str| crate::deception::primitives::DeceptionStateStore::get(store, lookup_key);
    let mut set_value = |lookup_key: &str, value: &[u8]| {
        crate::deception::primitives::DeceptionStateStore::set(store, lookup_key, value)
    };
    register_key_with_io(&mut get_value, &mut set_value, catalog_key, key)
}

pub(crate) fn list_keys(
    store: &(impl crate::challenge::KeyValueStore + ?Sized),
    catalog_key: &str,
) -> Vec<String> {
    let mut get_value = |lookup_key: &str| crate::challenge::KeyValueStore::get(store, lookup_key);
    list_keys_with_io(&mut get_value, catalog_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenge::KeyValueStore;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
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
    fn register_key_deduplicates_catalog_entries() {
        let store = MockStore::default();
        register_key(&store, "catalog:test", "maze_hits:bucket-a").expect("register bucket a");
        register_key(&store, "catalog:test", "maze_hits:bucket-b").expect("register bucket b");
        register_key(&store, "catalog:test", "maze_hits:bucket-a").expect("register duplicate");

        assert_eq!(
            list_keys(&store, "catalog:test"),
            vec![
                "maze_hits:bucket-a".to_string(),
                "maze_hits:bucket-b".to_string()
            ]
        );
    }
}
