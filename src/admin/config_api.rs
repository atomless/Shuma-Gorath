use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

pub(crate) fn handle_admin_config_export(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let entries = super::api::config_export_env_entries(&cfg);
    let env_map: std::collections::BTreeMap<String, String> = entries.iter().cloned().collect();
    let env_text = entries
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>()
        .join("\n");

    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::AdminAction,
            ip: None,
            reason: Some("config_export".to_string()),
            outcome: Some(format!("{} keys", entries.len())),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );

    let body = serde_json::to_string(&json!({
        "format": "env",
        "site_id": site_id,
        "generated_at": crate::admin::now_ts(),
        "excluded_secrets": super::api::CONFIG_EXPORT_SECRET_KEYS,
        "env": env_map,
        "env_text": env_text
    }))
    .unwrap();
    Response::new(200, body)
}

pub(crate) fn handle_admin_config(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    super::api::handle_admin_config_internal(req, store, site_id, false)
}

pub(crate) fn handle_admin_config_bootstrap(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    if !crate::config::admin_config_write_enabled() {
        return Response::new(
            403,
            "Config updates are disabled when SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false",
        );
    }

    match crate::config::Config::load(store, site_id) {
        Ok(_) => return Response::new(409, "Config already seeded"),
        Err(crate::config::ConfigLoadError::MissingConfig) => {}
        Err(err) => return Response::new(500, err.user_message()),
    }

    let body = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_ADMIN_JSON_BYTES,
    ) {
        Ok(value) => value,
        Err(err) => return Response::new(400, format!("Invalid config payload: {}", err)),
    };

    let mut cfg = match serde_json::from_value::<crate::config::Config>(body) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(400, format!("Invalid config payload: {}", err)),
    };
    if let Err(msg) = crate::config::normalize_persisted_config(&mut cfg) {
        return Response::new(400, msg);
    }

    if super::api::persist_site_config(store, site_id, &cfg, &[]).is_err() {
        return Response::new(500, "Key-value store error");
    }

    let challenge_default = super::api::challenge_threshold_default();
    let not_a_bot_default = super::api::not_a_bot_threshold_default();
    let maze_default = super::api::maze_threshold_default();
    let response_body = serde_json::to_string(&json!({
        "bootstrapped": true,
        "config": super::api::admin_config_settings_payload(&cfg),
        "runtime": super::api::admin_config_runtime_payload(
            &cfg,
            challenge_default,
            not_a_bot_default,
            maze_default
        )
    }))
    .unwrap();
    Response::new(200, response_body)
}

pub(crate) fn handle_admin_config_validate(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }

    let patch = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_ADMIN_JSON_BYTES,
    ) {
        Ok(value) => value,
        Err(err) => {
            let issue = super::api::admin_config_validation_issue(
                &serde_json::Value::Null,
                format!("Invalid config payload: {}", err),
            );
            let body = serde_json::to_string(&json!({
                "valid": false,
                "issues": [issue]
            }))
            .unwrap();
            return Response::new(200, body);
        }
    };

    if let Err(err) = super::api::validate_admin_config_patch_shape(&patch) {
        let issue = super::api::admin_config_validation_issue(&patch, err);
        let body = serde_json::to_string(&json!({
            "valid": false,
            "issues": [issue]
        }))
        .unwrap();
        return Response::new(200, body);
    }

    let validation_response = super::api::handle_admin_config_internal(req, store, site_id, true);
    let status = *validation_response.status();
    if status == 200 {
        return Response::new(200, r#"{"valid":true,"issues":[]}"#);
    }

    let message = String::from_utf8_lossy(validation_response.body()).to_string();
    if status == 400 {
        let issue = super::api::admin_config_validation_issue(&patch, message.clone());
        let body = serde_json::to_string(&json!({
            "valid": false,
            "issues": [issue]
        }))
        .unwrap();
        return Response::new(200, body);
    }

    Response::new(status, message)
}
