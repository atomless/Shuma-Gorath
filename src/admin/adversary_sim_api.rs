use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

pub(crate) struct AdversarySimLifecycleSnapshot {
    pub(crate) cfg: crate::config::Config,
    pub(crate) state: crate::admin::adversary_sim::ControlState,
}

pub(crate) fn adversary_sim_lifecycle_snapshot_from_cfg(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    mut cfg: crate::config::Config,
) -> AdversarySimLifecycleSnapshot {
    let state = crate::admin::adversary_sim::load_state(store, site_id);
    crate::admin::adversary_sim::project_effective_desired_state(&mut cfg, &state);
    AdversarySimLifecycleSnapshot { cfg, state }
}

pub(crate) fn load_adversary_sim_lifecycle_snapshot(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Result<AdversarySimLifecycleSnapshot, crate::config::ConfigLoadError> {
    let cfg = crate::config::load_runtime_cached(store, site_id)?;
    Ok(adversary_sim_lifecycle_snapshot_from_cfg(
        store, site_id, cfg,
    ))
}

pub(crate) fn adversary_sim_status_payload(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    cfg: &crate::config::Config,
    state: &crate::admin::adversary_sim::ControlState,
    now: u64,
) -> serde_json::Value {
    let mut payload = crate::admin::adversary_sim::status_payload(
        now,
        crate::config::runtime_environment(),
        crate::config::adversary_sim_available(),
        cfg.adversary_sim_enabled,
        cfg.adversary_sim_duration_seconds,
        state,
    );
    let reconciliation_required = crate::admin::adversary_sim_control::status_reconciliation_needed(
        now,
        cfg.adversary_sim_enabled,
        state,
    );
    let generation_diagnostics =
        crate::admin::adversary_sim::generation_diagnostics(now, cfg.adversary_sim_enabled, state);
    let supervisor = crate::admin::adversary_sim::supervisor_status_payload(
        now,
        cfg.adversary_sim_enabled,
        state,
    );
    let lease = crate::admin::adversary_sim_control::load_controller_lease(store, site_id);
    let lease_operation_id = lease.as_ref().map(|value| value.operation_id.clone());
    let lease_expires_at = lease.as_ref().map(|value| value.expires_at);
    let seconds_since_last_successful_beat = state
        .last_generated_at
        .map(|last_generated_at| now.saturating_sub(last_generated_at));
    let generation_active = cfg.adversary_sim_enabled
        && state.phase == crate::admin::adversary_sim::ControlPhase::Running;
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "desired_state".to_string(),
            serde_json::Value::String(if cfg.adversary_sim_enabled {
                "running".to_string()
            } else {
                "off".to_string()
            }),
        );
        object.insert(
            "actual_state".to_string(),
            serde_json::Value::String(
                crate::admin::adversary_sim_control::actual_phase_label(state.phase).to_string(),
            ),
        );
        object.insert(
            "controller_reconciliation_required".to_string(),
            serde_json::Value::Bool(reconciliation_required),
        );
        object.insert(
            "generation_active".to_string(),
            serde_json::Value::Bool(generation_active),
        );
        object.insert(
            "historical_data_visible".to_string(),
            serde_json::Value::Bool(true),
        );
        object.insert(
            "history_retention".to_string(),
            json!({
                "retention_hours": super::api::event_log_retention_hours(),
                "retention_health": crate::observability::retention::retention_health(store),
                "cleanup_supported": crate::config::admin_config_write_enabled(),
                "cleanup_endpoint": "/admin/adversary-sim/history/cleanup",
                "cleanup_command": "make telemetry-clean"
            }),
        );
        object.insert(
            "generation_diagnostics".to_string(),
            json!({
                "health": generation_diagnostics.health,
                "reason": generation_diagnostics.reason,
                "recommended_action": generation_diagnostics.recommended_action,
                "generated_tick_count": generation_diagnostics.generated_tick_count,
                "generated_request_count": generation_diagnostics.generated_request_count,
                "last_generated_at": generation_diagnostics.last_generated_at,
                "last_generation_error": generation_diagnostics.last_generation_error
            }),
        );
        object.insert("supervisor".to_string(), supervisor);
        object.insert(
            "control_contract".to_string(),
            json!({
                "contract": "adversary-sim-control.v1",
                "idempotency_ttl_seconds": crate::admin::adversary_sim_control::IDEMPOTENCY_TTL_SECONDS,
                "lease_ttl_seconds": crate::admin::adversary_sim_control::LEASE_TTL_SECONDS,
                "requires_idempotency_key": true
            }),
        );
        object.insert(
            "lifecycle_diagnostics".to_string(),
            json!({
                "control": {
                    "desired_enabled": cfg.adversary_sim_enabled,
                    "actual_phase": state.phase.as_str(),
                    "controller_reconciliation_required": reconciliation_required,
                    "runtime_instance_id": crate::admin::adversary_sim::process_instance_id(),
                    "owner_instance_id": state.owner_instance_id.clone(),
                    "last_transition_reason": state.last_transition_reason.clone(),
                    "last_terminal_failure_reason": state.last_terminal_failure_reason.clone(),
                    "last_control_operation_id": lease_operation_id,
                    "lease_expires_at": lease_expires_at
                },
                "supervisor": {
                    "heartbeat_expected": generation_active,
                    "generated_tick_count": state.generated_tick_count,
                    "generated_request_count": state.generated_request_count,
                    "last_successful_beat_at": state.last_generated_at,
                    "seconds_since_last_successful_beat": seconds_since_last_successful_beat,
                    "last_generation_error": state.last_generation_error.clone()
                }
            }),
        );
        object.insert(
            "controller_lease".to_string(),
            match lease {
                Some(current_lease) => json!({
                    "owner": current_lease.owner,
                    "fencing_token": current_lease.fencing_token,
                    "acquired_at": current_lease.acquired_at,
                    "expires_at": current_lease.expires_at,
                    "operation_id": current_lease.operation_id
                }),
                None => serde_json::Value::Null,
            },
        );
    }
    payload
}

pub(crate) fn handle_admin_adversary_sim_status(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    _auth: &crate::admin::auth::AdminAuthResult,
) -> Response {
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }

    let runtime_environment = crate::config::runtime_environment();
    let env_available = crate::config::adversary_sim_available();
    if !crate::admin::adversary_sim::control_surface_available(runtime_environment, env_available) {
        return Response::new(404, "Not Found");
    }

    let snapshot = match load_adversary_sim_lifecycle_snapshot(store, site_id) {
        Ok(snapshot) => snapshot,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let now = crate::admin::now_ts();
    let cfg = snapshot.cfg;
    let state = snapshot.state;

    let body = serde_json::to_string(&adversary_sim_status_payload(
        store, site_id, &cfg, &state, now,
    ))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}
