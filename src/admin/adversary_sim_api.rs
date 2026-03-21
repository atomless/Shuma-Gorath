use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

use super::recent_changes_ledger::{
    operator_snapshot_manual_change_row, record_operator_snapshot_recent_change_rows,
};

pub(crate) struct AdversarySimLifecycleSnapshot {
    pub(crate) cfg: crate::config::Config,
    pub(crate) state: crate::admin::adversary_sim::ControlState,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdminAdversarySimControlRequest {
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) lane: Option<crate::admin::adversary_sim::RuntimeLane>,
    #[serde(default)]
    pub(crate) reason: Option<String>,
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

fn save_adversary_sim_beat_state_if_unchanged<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    previous_state: &crate::admin::adversary_sim::ControlState,
    next_state: &crate::admin::adversary_sim::ControlState,
) -> Result<bool, ()> {
    let current_state = crate::admin::adversary_sim::load_state(store, site_id);
    if current_state != *previous_state {
        return Ok(false);
    }
    crate::admin::adversary_sim::save_state(store, site_id, next_state)?;
    Ok(true)
}

fn internal_adversary_sim_beat_is_authorized(req: &Request) -> bool {
    crate::admin::auth::is_internal_adversary_sim_beat_request(req)
}

fn internal_adversary_sim_worker_result_is_authorized(req: &Request) -> bool {
    crate::admin::auth::is_internal_adversary_sim_supervisor_request(req)
}

pub(crate) fn handle_internal_adversary_sim_beat(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    let edge_cron_request = crate::admin::auth::is_internal_adversary_sim_edge_cron_request(req);
    if *req.method() != Method::Post && !(edge_cron_request && *req.method() == Method::Get) {
        return Response::new(405, "Method Not Allowed");
    }

    let runtime_environment = crate::config::runtime_environment();
    let env_available = crate::config::adversary_sim_available();
    if !crate::admin::adversary_sim::control_surface_available(runtime_environment, env_available) {
        return Response::new(404, "Not Found");
    }

    if !internal_adversary_sim_beat_is_authorized(req) {
        return Response::new(
            401,
            "Unauthorized: Internal adversary-sim beat authorization required",
        );
    }

    let snapshot = match load_adversary_sim_lifecycle_snapshot(store, site_id) {
        Ok(snapshot) => snapshot,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let now = crate::admin::now_ts();
    let mut cfg = snapshot.cfg;
    let mut state = snapshot.state;
    let previous_state = state.clone();

    let (reconciled_state, _) =
        crate::admin::adversary_sim::reconcile_state(now, cfg.adversary_sim_enabled, &state);
    state = reconciled_state;
    crate::admin::adversary_sim::project_effective_desired_state(&mut cfg, &state);

    let summary =
        crate::admin::adversary_sim::run_autonomous_supervisor_ticks(store, &mut state, now);
    if state != previous_state {
        match save_adversary_sim_beat_state_if_unchanged(store, site_id, &previous_state, &state) {
            Ok(true) => {}
            Ok(false) => {
                let snapshot = match load_adversary_sim_lifecycle_snapshot(store, site_id) {
                    Ok(snapshot) => snapshot,
                    Err(err) => return Response::new(500, err.user_message()),
                };
                cfg = snapshot.cfg;
                state = snapshot.state;
                let (reconciled_state, _) = crate::admin::adversary_sim::reconcile_state(
                    now,
                    cfg.adversary_sim_enabled,
                    &state,
                );
                state = reconciled_state;
                crate::admin::adversary_sim::project_effective_desired_state(&mut cfg, &state);
            }
            Err(()) => return Response::new(500, "Key-value store error"),
        }
    }

    if summary.executed_ticks > 0 {
        crate::log_line(&format!(
            "[adversary-sim-supervisor] executed_ticks={} generated_requests={} failed_requests={}",
            summary.executed_ticks, summary.generated_requests, summary.failed_requests
        ));
    }

    let generation_active = cfg.adversary_sim_enabled
        && state.phase == crate::admin::adversary_sim::ControlPhase::Running;
    let status = adversary_sim_status_payload(store, site_id, &cfg, &state, now);
    let dispatch_mode = if summary.worker_plan.is_some() {
        "scrapling_worker"
    } else if summary.worker_pending {
        "scrapling_worker_pending"
    } else {
        "internal"
    };
    let body = serde_json::to_string(&json!({
        "accepted": true,
        "dispatch_mode": dispatch_mode,
        "executed_ticks": summary.executed_ticks,
        "due_ticks": summary.due_ticks,
        "generated_requests": summary.generated_requests,
        "failed_requests": summary.failed_requests,
        "last_response_status": summary.last_response_status,
        "worker_plan": summary.worker_plan,
        "phase": state.phase.as_str(),
        "generation_active": generation_active,
        "should_exit": !generation_active,
        "status": status
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

pub(crate) fn handle_internal_adversary_sim_worker_result(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    let runtime_environment = crate::config::runtime_environment();
    let env_available = crate::config::adversary_sim_available();
    if !crate::admin::adversary_sim::control_surface_available(runtime_environment, env_available) {
        return Response::new(404, "Not Found");
    }
    if !internal_adversary_sim_worker_result_is_authorized(req) {
        return Response::new(
            401,
            "Unauthorized: Internal adversary-sim worker authorization required",
        );
    }

    let worker_result = match serde_json::from_slice::<
        crate::admin::adversary_sim::ScraplingWorkerResult,
    >(req.body())
    {
        Ok(parsed) => parsed,
        Err(_) => return Response::new(400, "Invalid Scrapling worker result payload"),
    };
    if worker_result.schema_version
        != crate::admin::adversary_sim::SCRAPLING_WORKER_RESULT_SCHEMA_VERSION
    {
        return Response::new(400, "Invalid Scrapling worker result schema_version");
    }

    let snapshot = match load_adversary_sim_lifecycle_snapshot(store, site_id) {
        Ok(snapshot) => snapshot,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let cfg = snapshot.cfg;
    let mut state = snapshot.state;
    let previous_state = state.clone();

    let active_lane = crate::admin::adversary_sim::effective_active_lane(&state);
    let worker_tick_matches =
        state.pending_worker_tick_id.as_deref() == Some(worker_result.tick_id.as_str());
    let run_matches = state.run_id.as_deref() == Some(worker_result.run_id.as_str());
    let lane_matches = active_lane == Some(worker_result.lane);
    if !matches!(
        state.phase,
        crate::admin::adversary_sim::ControlPhase::Running
    ) || !state.desired_enabled
        || !worker_tick_matches
        || !run_matches
        || !lane_matches
    {
        return Response::new(409, "stale_worker_result");
    }

    crate::admin::adversary_sim::apply_scrapling_worker_result(&mut state, &worker_result);
    match save_adversary_sim_beat_state_if_unchanged(store, site_id, &previous_state, &state) {
        Ok(true) => {}
        Ok(false) => return Response::new(409, "stale_worker_result"),
        Err(()) => return Response::new(500, "Key-value store error"),
    }

    let now = worker_result.tick_completed_at;
    let status = adversary_sim_status_payload(store, site_id, &cfg, &state, now);
    let body = serde_json::to_string(&json!({
        "accepted": true,
        "status": status
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

pub(crate) fn handle_admin_adversary_sim_history_cleanup(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    _auth: &crate::admin::auth::AdminAuthResult,
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

    let runtime_environment = crate::config::runtime_environment();
    let cleanup_command = "make telemetry-clean";
    if runtime_environment.is_prod() {
        if !super::api::telemetry_cleanup_acknowledged(req) {
            return Response::new(
                403,
                format!(
                    "Forbidden: runtime-prod telemetry cleanup requires header X-Shuma-Telemetry-Cleanup-Ack: {}",
                    super::api::TELEMETRY_CLEANUP_ACK_VALUE
                ),
            );
        }
    } else {
        let env_available = crate::config::adversary_sim_available();
        if !crate::admin::adversary_sim::control_surface_available(
            runtime_environment,
            env_available,
        ) {
            return Response::new(404, "Not Found");
        }
    }

    let cleanup_result = super::api::clear_telemetry_history(store, site_id);
    super::api::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::AdminAction,
            ip: Some(crate::extract_client_ip(req)),
            reason: Some("telemetry_history_cleanup".to_string()),
            outcome: Some(format!(
                "deleted_keys={} families={}",
                cleanup_result.deleted_keys,
                cleanup_result.deleted_by_family.len()
            )),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );

    let body = serde_json::to_string(&json!({
        "cleaned": true,
        "deleted_keys": cleanup_result.deleted_keys,
        "deleted_by_family": cleanup_result.deleted_by_family,
        "retention_hours": super::api::event_log_retention_hours(),
        "cleanup_command": cleanup_command
    }))
    .unwrap();
    Response::new(200, body)
}

pub(crate) fn control_state_label(enabled: bool) -> String {
    if enabled {
        "running".to_string()
    } else {
        "off".to_string()
    }
}

pub(crate) fn control_lane_label(
    lane: Option<crate::admin::adversary_sim::RuntimeLane>,
) -> Option<String> {
    lane.map(|value| value.as_str().to_string())
}

pub(crate) fn control_desired_lane_label(
    state: &crate::admin::adversary_sim::ControlState,
) -> Option<String> {
    Some(state.desired_lane.as_str().to_string())
}

pub(crate) fn control_actual_lane_label(
    state: &crate::admin::adversary_sim::ControlState,
) -> Option<String> {
    control_lane_label(crate::admin::adversary_sim::effective_active_lane(state))
}

pub(crate) fn log_adversary_sim_transition<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    req: &Request,
    auth: &crate::admin::auth::AdminAuthResult,
    transition: &crate::admin::adversary_sim::Transition,
    operation_id: Option<&str>,
) {
    let client_ip = crate::extract_client_ip(req);
    let session = auth.session_id.as_deref().unwrap_or("-");
    let run_id = transition.run_id.as_deref().unwrap_or("-");
    let operation = operation_id.unwrap_or("-");
    record_operator_snapshot_recent_change_rows(
        store,
        site_id,
        &[operator_snapshot_manual_change_row(
            crate::admin::now_ts(),
            "adversary_sim_transition",
            &["adversary_sim_control"],
            &["representative_adversary_effectiveness"],
            auth.audit_actor_label(),
            format!(
                "adversary sim transition {} -> {} ({})",
                transition.from.as_str(),
                transition.to.as_str(),
                transition.reason
            )
            .as_str(),
        )],
        crate::admin::now_ts(),
    );
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::AdminAction,
            ip: Some(client_ip),
            reason: Some("adversary_sim_transition".to_string()),
            outcome: Some(format!(
                "from={} to={} reason={} run_id={} operation_id={} actor={} session={}",
                transition.from.as_str(),
                transition.to.as_str(),
                transition.reason,
                run_id,
                operation,
                auth.audit_actor_label(),
                session
            )),
            admin: Some(auth.audit_actor_label().to_string()),
        },
    );
}

pub(crate) fn log_adversary_sim_control_audit<S: crate::challenge::KeyValueStore>(
    store: &S,
    req: &Request,
    auth: &crate::admin::auth::AdminAuthResult,
    audit: &crate::admin::adversary_sim_control::ControlAuditRecord,
    _capability: &crate::admin::adversary_sim_control::AuditWriteCapability,
) {
    let client_ip = crate::extract_client_ip(req);
    let payload = json!({
        "operation_id": audit.operation_id.clone(),
        "actor_scope": audit.actor_scope.clone(),
        "session_scope_hash": crate::admin::adversary_sim_control::hash_hex(audit.session_scope.as_str()),
        "decision": audit.decision,
        "reason": audit.reason.clone(),
        "origin_verdict": audit.origin_verdict.clone(),
        "idempotency_key_hash": audit.idempotency_key_hash.clone(),
        "request_origin": audit.request_origin.clone(),
        "requested_state": audit.requested_state.clone(),
        "requested_lane": audit.requested_lane.clone(),
        "desired_state": audit.desired_state.clone(),
        "desired_lane": audit.desired_lane.clone(),
        "actual_state": audit.actual_state.clone(),
        "actual_lane": audit.actual_lane.clone()
    });
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: crate::admin::now_ts(),
            event: crate::admin::EventType::AdminAction,
            ip: Some(client_ip),
            reason: Some("adversary_sim_control_audit".to_string()),
            outcome: Some(payload.to_string()),
            admin: Some(auth.audit_actor_label().to_string()),
        },
    );
}

pub(crate) fn save_adversary_sim_state_with_capability<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    state: &crate::admin::adversary_sim::ControlState,
    _capability: &crate::admin::adversary_sim_control::StateWriteCapability,
) -> Result<(), ()> {
    crate::admin::adversary_sim::save_state(store, site_id, state)
}
