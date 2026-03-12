use std::env;

#[cfg(test)]
use std::{collections::HashMap, sync::Mutex};

#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
static TEST_SPIN_VARIABLES: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub(crate) fn spin_variable_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if !trimmed.starts_with("SHUMA_") {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

#[cfg(test)]
pub(crate) fn set_test_spin_variable(name: &str, value: &str) {
    let key = spin_variable_name(name).unwrap_or_else(|| name.trim().to_string());
    TEST_SPIN_VARIABLES
        .lock()
        .unwrap()
        .insert(key, value.to_string());
}

#[cfg(test)]
pub(crate) fn clear_test_spin_variables() {
    TEST_SPIN_VARIABLES.lock().unwrap().clear();
}

pub(crate) fn runtime_var_raw_optional(name: &str) -> Option<String> {
    env::var(name).ok().or_else(|| spin_variable_raw_optional(name))
}

pub(crate) fn runtime_var_trimmed_optional(name: &str) -> Option<String> {
    runtime_var_raw_optional(name)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
fn spin_variable_raw_optional(name: &str) -> Option<String> {
    let key = spin_variable_name(name).unwrap_or_else(|| name.trim().to_string());
    TEST_SPIN_VARIABLES.lock().unwrap().get(&key).cloned()
}

#[cfg(all(not(test), target_arch = "wasm32"))]
fn spin_variable_raw_optional(name: &str) -> Option<String> {
    let key = spin_variable_name(name)?;
    spin_sdk::variables::get(&key).ok()
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn spin_variable_raw_optional(_name: &str) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::{
        clear_test_spin_variables, runtime_var_raw_optional, runtime_var_trimmed_optional,
        set_test_spin_variable,
    };

    #[test]
    fn runtime_var_prefers_process_env_over_spin_variable() {
        clear_test_spin_variables();
        std::env::set_var("SHUMA_API_KEY", "env-secret");
        set_test_spin_variable("SHUMA_API_KEY", "spin-secret");

        let value = runtime_var_trimmed_optional("SHUMA_API_KEY");

        std::env::remove_var("SHUMA_API_KEY");
        clear_test_spin_variables();
        assert_eq!(value.as_deref(), Some("env-secret"));
    }

    #[test]
    fn runtime_var_uses_spin_variable_when_env_is_missing() {
        clear_test_spin_variables();
        std::env::remove_var("SHUMA_API_KEY");
        set_test_spin_variable("SHUMA_API_KEY", "spin-secret");

        let value = runtime_var_trimmed_optional("SHUMA_API_KEY");

        clear_test_spin_variables();
        assert_eq!(value.as_deref(), Some("spin-secret"));
    }

    #[test]
    fn runtime_var_trimmed_optional_discards_blank_spin_variable_values() {
        clear_test_spin_variables();
        std::env::remove_var("SHUMA_API_KEY");
        set_test_spin_variable("SHUMA_API_KEY", "   ");

        let value = runtime_var_trimmed_optional("SHUMA_API_KEY");

        clear_test_spin_variables();
        assert_eq!(value, None);
    }

    #[test]
    fn runtime_var_does_not_attempt_spin_lookup_for_non_shuma_keys() {
        clear_test_spin_variables();
        std::env::remove_var("RUNTIME_INSTANCE_ID");
        set_test_spin_variable("RUNTIME_INSTANCE_ID", "ignored");

        let value = runtime_var_raw_optional("RUNTIME_INSTANCE_ID");

        clear_test_spin_variables();
        assert_eq!(value, None);
    }
}
