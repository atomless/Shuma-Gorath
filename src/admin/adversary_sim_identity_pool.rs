use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IdentityPoolEntry {
    pub label: String,
    pub proxy_url: String,
    pub identity_class: String,
    pub country_code: String,
}

fn is_valid_identity_class(value: &str) -> bool {
    matches!(value, "residential" | "mobile" | "datacenter")
}

fn normalize_country_code(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.len() != 2 || !trimmed.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    Some(trimmed.to_ascii_uppercase())
}

fn normalize_pool_entry(raw: serde_json::Value) -> Option<IdentityPoolEntry> {
    let entry = raw.as_object()?;
    let label = entry.get("label")?.as_str()?.trim().to_string();
    let proxy_url = entry.get("proxy_url")?.as_str()?.trim().to_string();
    let identity_class = entry.get("identity_class")?.as_str()?.trim().to_string();
    let country_code = normalize_country_code(entry.get("country_code")?.as_str()?.trim())?;
    if label.is_empty()
        || proxy_url.is_empty()
        || proxy_url.contains('\n')
        || proxy_url.contains('\r')
        || !is_valid_identity_class(&identity_class)
    {
        return None;
    }
    Some(IdentityPoolEntry {
        label,
        proxy_url,
        identity_class,
        country_code,
    })
}

pub(crate) fn load_identity_pool_from_env(name: &str) -> Vec<IdentityPoolEntry> {
    let raw = match std::env::var(name) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let parsed: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let Some(entries) = parsed.as_array() else {
        return Vec::new();
    };
    entries
        .iter()
        .cloned()
        .filter_map(normalize_pool_entry)
        .collect()
}
