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
    let truth_projection =
        crate::admin::adversary_sim_status_truth::project_status_truth(store, site_id, now, state);
    let projected_state = &truth_projection.projected_state;
    let mut payload = crate::admin::adversary_sim::status_payload(
        now,
        crate::config::runtime_environment(),
        crate::config::adversary_sim_available(),
        cfg.adversary_sim_enabled,
        cfg.adversary_sim_duration_seconds,
        projected_state,
    );
    let reconciliation_required = crate::admin::adversary_sim_control::status_reconciliation_needed(
        now,
        cfg.adversary_sim_enabled,
        projected_state,
    );
    let generation_diagnostics = crate::admin::adversary_sim::generation_diagnostics(
        now,
        cfg.adversary_sim_enabled,
        projected_state,
    );
    let supervisor = crate::admin::adversary_sim::supervisor_status_payload(
        now,
        cfg.adversary_sim_enabled,
        projected_state,
    );
    let lease = crate::admin::adversary_sim_control::load_controller_lease(store, site_id);
    let lease_operation_id = lease.as_ref().map(|value| value.operation_id.clone());
    let lease_expires_at = lease.as_ref().map(|value| value.expires_at);
    let seconds_since_last_successful_beat = projected_state
        .last_generated_at
        .map(|last_generated_at| now.saturating_sub(last_generated_at));
    let generation_active = cfg.adversary_sim_enabled
        && projected_state.phase == crate::admin::adversary_sim::ControlPhase::Running;
    if let Some(object) = payload.as_object_mut() {
        if let Some(generation) = object.get_mut("generation").and_then(|value| value.as_object_mut()) {
            generation.insert(
                "truth_basis".to_string(),
                serde_json::Value::String(truth_projection.generation_truth_basis.to_string()),
            );
        }
        if let Some(lane_diagnostics) = object
            .get_mut("lane_diagnostics")
            .and_then(|value| value.as_object_mut())
        {
            lane_diagnostics.insert(
                "truth_basis".to_string(),
                serde_json::Value::String(
                    truth_projection.lane_diagnostics_truth_basis.to_string(),
                ),
            );
        }
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
                crate::admin::adversary_sim_control::actual_phase_label(projected_state.phase)
                    .to_string(),
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
                "last_generation_error": generation_diagnostics.last_generation_error,
                "truth_basis": truth_projection.generation_truth_basis
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
                    "actual_phase": projected_state.phase.as_str(),
                    "controller_reconciliation_required": reconciliation_required,
                    "runtime_instance_id": crate::admin::adversary_sim::process_instance_id(),
                    "owner_instance_id": projected_state.owner_instance_id.clone(),
                    "last_transition_reason": projected_state.last_transition_reason.clone(),
                    "last_terminal_failure_reason": projected_state.last_terminal_failure_reason.clone(),
                    "last_control_operation_id": lease_operation_id,
                    "lease_expires_at": lease_expires_at
                },
                "supervisor": {
                    "heartbeat_expected": generation_active,
                    "generated_tick_count": projected_state.generated_tick_count,
                    "generated_request_count": projected_state.generated_request_count,
                    "last_successful_beat_at": projected_state.last_generated_at,
                    "seconds_since_last_successful_beat": seconds_since_last_successful_beat,
                    "last_generation_error": projected_state.last_generation_error.clone(),
                    "truth_basis": truth_projection.generation_truth_basis
                }
            }),
        );
        object.insert(
            "persisted_event_evidence".to_string(),
            truth_projection
                .persisted_event_evidence
                .unwrap_or(serde_json::Value::Null),
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

    if let Err(()) = crate::admin::oversight_agent::maybe_trigger_post_sim_agent_cycle(
        store,
        site_id,
        &previous_state,
        &state,
        now,
    ) {
        crate::log_line(
            "[oversight-agent] post-sim trigger failed during internal beat; continuing without agent payload",
        );
    }

    let generation_active = cfg.adversary_sim_enabled
        && state.phase == crate::admin::adversary_sim::ControlPhase::Running;
    let status = adversary_sim_status_payload(store, site_id, &cfg, &state, now);
    let dispatch_mode = if summary.worker_plan.is_some() {
        "scrapling_worker"
    } else if summary.llm_fulfillment_plan.is_some() {
        "llm_fulfillment_plan"
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
        "llm_fulfillment_plan": summary.llm_fulfillment_plan,
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

    if !worker_result.surface_receipts.is_empty() {
        log_scrapling_surface_receipts_event(store, &worker_result);
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

fn log_scrapling_surface_receipts_event(
    store: &impl crate::challenge::KeyValueStore,
    worker_result: &crate::admin::adversary_sim::ScraplingWorkerResult,
) {
    let record = super::api::EventLogRecord {
        entry: super::api::EventLogEntry {
            ts: worker_result.tick_completed_at,
            event: super::api::EventType::AdminAction,
            ip: None,
            reason: Some("scrapling_surface_coverage".to_string()),
            outcome: Some(format!(
                "tick_id={} receipts={} generated_requests={} failed_requests={}",
                worker_result.tick_id,
                worker_result.surface_receipts.len(),
                worker_result.generated_requests,
                worker_result.failed_requests
            )),
            admin: None,
        },
        taxonomy: None,
        outcome_code: None,
        botness_score: None,
        sim_run_id: Some(worker_result.run_id.clone()),
        sim_profile: Some(format!(
            "{}.{}",
            crate::admin::adversary_sim::SCRAPLING_SIM_PROFILE,
            worker_result.fulfillment_mode
        )),
        sim_lane: Some(worker_result.lane.as_str().to_string()),
        is_simulation: true,
        scrapling_surface_receipts: worker_result.surface_receipts.clone(),
        execution: super::api::EventExecutionMetadata::default(),
    };
    super::api::persist_event_record(store, record);
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

pub(crate) fn handle_admin_adversary_sim_control(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    auth: &crate::admin::auth::AdminAuthResult,
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
    let env_available = crate::config::adversary_sim_available();
    if !crate::admin::adversary_sim::control_surface_available(runtime_environment, env_available) {
        return Response::new(404, "Not Found");
    }

    if auth.requires_csrf(req) {
        let expected = auth.csrf_token.as_deref().unwrap_or("");
        if !crate::admin::auth::validate_session_csrf(req, expected) {
            super::api::log_admin_csrf_denied(store, req, "/admin/adversary-sim/control", auth);
            return Response::new(403, "Forbidden: control trust boundary violation");
        }
    }

    let body = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_ADMIN_JSON_BYTES,
    ) {
        Ok(v) => v,
        Err(err) => return Response::new(400, err),
    };
    let payload = match serde_json::from_value::<AdminAdversarySimControlRequest>(body) {
        Ok(parsed) => parsed,
        Err(err) => return Response::new(400, format!("Invalid control payload: {}", err)),
    };

    let now = crate::admin::now_ts();
    let requested_reason =
        crate::admin::adversary_sim_control::canonical_reason(payload.reason.as_deref());
    let Some(idempotency_key) = req
        .header("idempotency-key")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Response::new(428, "Idempotency-Key header is required");
    };
    let idempotency_key_hash = crate::admin::adversary_sim_control::hash_hex(idempotency_key);
    let payload_hash = crate::admin::adversary_sim_control::canonical_payload_hash(
        payload.enabled,
        payload.lane.map(|lane| lane.as_str()),
        requested_reason.as_str(),
    );

    let session_scope = crate::admin::adversary_sim_control::idempotency_scope(auth);
    let actor_scope = crate::admin::adversary_sim_control::actor_scope(auth);
    let client_ip = crate::extract_client_ip(req);

    let origin_validation =
        crate::admin::adversary_sim_control::validate_origin_and_fetch_metadata(req);
    let session_origin_fallback_allowed = matches!(
        auth.method,
        Some(crate::admin::auth::AdminAuthMethod::SessionCookie)
    );
    let (origin_verdict, request_origin, trust_reason, trust_decision) = match origin_validation {
        Ok(valid) => (
            valid.verdict,
            valid.request_origin,
            "trust_boundary_ok".to_string(),
            crate::admin::adversary_sim_control::TrustDecision::Allow,
        ),
        Err("origin_missing" | "fetch_metadata_missing") if session_origin_fallback_allowed => (
            "session_csrf_origin_fallback".to_string(),
            None,
            "session_csrf_origin_fallback".to_string(),
            crate::admin::adversary_sim_control::TrustDecision::Allow,
        ),
        Err(reason) => (
            "origin_denied".to_string(),
            None,
            reason.to_string(),
            crate::admin::adversary_sim_control::TrustDecision::Reject,
        ),
    };

    let idempotency_store_key = crate::admin::adversary_sim_control::control_idempotency_key(
        site_id,
        session_scope.as_str(),
        idempotency_key_hash.as_str(),
    );
    let mut existing_idempotency =
        crate::admin::adversary_sim_control::load_idempotency_record(store, &idempotency_store_key);
    if existing_idempotency
        .as_ref()
        .map(|record| record.expires_at <= now)
        .unwrap_or(false)
    {
        existing_idempotency = None;
    }
    let idempotency_plan = match existing_idempotency.as_ref() {
        Some(record) if record.payload_hash == payload_hash => {
            crate::admin::adversary_sim_control::IdempotencyPlan::Replay
        }
        Some(_) => crate::admin::adversary_sim_control::IdempotencyPlan::PayloadMismatch,
        None => crate::admin::adversary_sim_control::IdempotencyPlan::NewSubmission,
    };

    let snapshot = match load_adversary_sim_lifecycle_snapshot(store, site_id) {
        Ok(snapshot) => snapshot,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let mut cfg = snapshot.cfg;
    let mut state = snapshot.state;
    let previous_state = state.clone();
    let (reconciled_state, _) =
        crate::admin::adversary_sim::reconcile_state(now, cfg.adversary_sim_enabled, &state);
    if reconciled_state != state {
        state = reconciled_state;
        if crate::admin::adversary_sim::save_state(store, site_id, &state).is_err() {
            return Response::new(500, "Key-value store error");
        }
    }
    crate::admin::adversary_sim::project_effective_desired_state(&mut cfg, &state);
    let requested_state_label = control_state_label(payload.enabled);
    let requested_lane_label = control_lane_label(payload.lane);
    let requested_lane = payload.lane.unwrap_or(state.desired_lane);
    let current_desired_state_label = control_state_label(cfg.adversary_sim_enabled);
    let current_desired_lane_label = control_desired_lane_label(&state);
    let current_actual_state = state.phase.as_str().to_string();
    let current_actual_lane_label = control_actual_lane_label(&state);

    let debounce_key =
        crate::admin::adversary_sim_control::control_debounce_key(site_id, session_scope.as_str());
    let last_submission_at = store
        .get(&debounce_key)
        .ok()
        .flatten()
        .as_deref()
        .and_then(crate::admin::adversary_sim_control::parse_debounce_timestamp);
    let debounce_throttled = crate::admin::adversary_sim_control::should_throttle_for_debounce(
        now,
        last_submission_at,
        crate::admin::adversary_sim_control::CONTROL_DEBOUNCE_SECONDS,
    ) && cfg.adversary_sim_enabled == payload.enabled
        && state.desired_lane == requested_lane;
    let rate_limited = super::api::adversary_sim_control_submission_is_limited(
        store,
        session_scope.as_str(),
        client_ip.as_str(),
    );
    let throttle_decision = if rate_limited || debounce_throttled {
        crate::admin::adversary_sim_control::ThrottleDecision::Throttle
    } else {
        crate::admin::adversary_sim_control::ThrottleDecision::Allow
    };
    let throttle_reason =
        if throttle_decision == crate::admin::adversary_sim_control::ThrottleDecision::Throttle {
            if debounce_throttled {
                "debounce_window"
            } else {
                "toggle_rate_limited"
            }
        } else {
            "throttle_ok"
        };

    let plan = crate::admin::adversary_sim_control::plan_submission(
        &crate::admin::adversary_sim_control::SubmissionPlanInput {
            trust: trust_decision,
            throttle: throttle_decision,
            idempotency: idempotency_plan,
        },
    );
    let capabilities =
        crate::admin::adversary_sim_control::ControlCapabilities::mint_for_trust_boundary();

    match plan.decision {
        crate::admin::adversary_sim_control::SubmissionPlanDecision::RejectTrustBoundary => {
            let trust_reason = trust_reason.clone();
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: None,
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Rejected,
                    reason: trust_reason,
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash),
                    request_origin,
                    requested_state: Some(requested_state_label.clone()),
                    requested_lane: requested_lane_label.clone(),
                    desired_state: Some(current_desired_state_label.clone()),
                    desired_lane: current_desired_lane_label.clone(),
                    actual_state: current_actual_state.clone(),
                    actual_lane: current_actual_lane_label.clone(),
                },
                capabilities.audit_write(),
            );
            return Response::new(403, "Forbidden: control trust boundary violation");
        }
        crate::admin::adversary_sim_control::SubmissionPlanDecision::RejectThrottled => {
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: None,
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Throttled,
                    reason: throttle_reason.to_string(),
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash),
                    request_origin,
                    requested_state: Some(requested_state_label.clone()),
                    requested_lane: requested_lane_label.clone(),
                    desired_state: Some(current_desired_state_label.clone()),
                    desired_lane: current_desired_lane_label.clone(),
                    actual_state: current_actual_state.clone(),
                    actual_lane: current_actual_lane_label.clone(),
                },
                capabilities.audit_write(),
            );
            return Response::builder()
                .status(429)
                .header("Retry-After", "60")
                .body("Too Many Requests: adversary control throttled")
                .build();
        }
        crate::admin::adversary_sim_control::SubmissionPlanDecision::RejectPayloadMismatch => {
            let replayed_operation_id = existing_idempotency
                .as_ref()
                .map(|record| record.operation_id.clone());
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: replayed_operation_id,
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Rejected,
                    reason: "idempotency_payload_mismatch".to_string(),
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash),
                    request_origin,
                    requested_state: Some(requested_state_label.clone()),
                    requested_lane: requested_lane_label.clone(),
                    desired_state: Some(current_desired_state_label.clone()),
                    desired_lane: current_desired_lane_label.clone(),
                    actual_state: current_actual_state.clone(),
                    actual_lane: current_actual_lane_label.clone(),
                },
                capabilities.audit_write(),
            );
            return Response::new(
                409,
                "Idempotency-Key replay rejected: payload mismatch for existing key",
            );
        }
        crate::admin::adversary_sim_control::SubmissionPlanDecision::ReturnReplay => {
            let Some(idempotency_record) = existing_idempotency.as_ref() else {
                return Response::new(500, "Idempotency state unavailable");
            };
            let operation_key = crate::admin::adversary_sim_control::control_operation_key(
                site_id,
                idempotency_record.operation_id.as_str(),
            );
            let operation =
                crate::admin::adversary_sim_control::load_operation_record(store, &operation_key);
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: Some(idempotency_record.operation_id.clone()),
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Replayed,
                    reason: "idempotency_exact_replay".to_string(),
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash.clone()),
                    request_origin,
                    requested_state: Some(requested_state_label.clone()),
                    requested_lane: requested_lane_label.clone(),
                    desired_state: Some(current_desired_state_label.clone()),
                    desired_lane: current_desired_lane_label.clone(),
                    actual_state: state.phase.as_str().to_string(),
                    actual_lane: control_actual_lane_label(&state),
                },
                capabilities.audit_write(),
            );
            let response = json!({
                "operation_id": idempotency_record.operation_id,
                "decision": "replayed",
                "requested_enabled": payload.enabled,
                "phase_trace": ["plan", "execute", "collect_evidence", "publish_report"],
                "requested_state": {
                    "enabled": payload.enabled,
                    "lane": requested_lane_label,
                    "reason": requested_reason
                },
                "accepted_state": {
                    "desired_enabled": cfg.adversary_sim_enabled,
                    "desired_lane": state.desired_lane.as_str(),
                    "actual_phase": state.phase.as_str(),
                    "active_lane": control_actual_lane_label(&state)
                },
                "idempotency": {
                    "key_hash": idempotency_key_hash,
                    "replayed": true,
                    "ttl_seconds": crate::admin::adversary_sim_control::IDEMPOTENCY_TTL_SECONDS
                },
                "operation": operation,
                "status": adversary_sim_status_payload(store, site_id, &cfg, &state, now),
                "config": super::api::admin_config_settings_payload(&cfg),
                "runtime": super::api::admin_config_runtime_payload(
                    &cfg,
                    super::api::challenge_threshold_default(),
                    super::api::not_a_bot_threshold_default(),
                    super::api::maze_threshold_default()
                )
            });
            return Response::new(200, serde_json::to_string(&response).unwrap());
        }
        crate::admin::adversary_sim_control::SubmissionPlanDecision::AcceptNew => {}
    }

    let operation_id = crate::admin::adversary_sim_control::operation_id(now);
    let current_lease = crate::admin::adversary_sim_control::load_controller_lease(store, site_id);
    let lease = match crate::admin::adversary_sim_control::acquire_controller_lease(
        now,
        session_scope.as_str(),
        Some(operation_id.as_str()),
        current_lease.as_ref(),
    ) {
        Ok(lease) => lease,
        Err(reason) => {
            log_adversary_sim_control_audit(
                store,
                req,
                auth,
                &crate::admin::adversary_sim_control::ControlAuditRecord {
                    operation_id: Some(operation_id),
                    actor_scope,
                    session_scope,
                    decision: crate::admin::adversary_sim_control::ControlDecision::Throttled,
                    reason: reason.to_string(),
                    origin_verdict,
                    idempotency_key_hash: Some(idempotency_key_hash),
                    request_origin,
                    requested_state: Some(requested_state_label.clone()),
                    requested_lane: requested_lane_label.clone(),
                    desired_state: Some(current_desired_state_label.clone()),
                    desired_lane: current_desired_lane_label.clone(),
                    actual_state: state.phase.as_str().to_string(),
                    actual_lane: control_actual_lane_label(&state),
                },
                capabilities.audit_write(),
            );
            let retry_after_seconds = current_lease
                .as_ref()
                .map(|lease| lease.expires_at.saturating_sub(now).max(1))
                .unwrap_or(crate::admin::adversary_sim_control::LEASE_TTL_SECONDS);
            let mut response = Response::builder();
            response
                .status(409)
                .header("Retry-After", retry_after_seconds.to_string())
                .body("Adversary simulation controller lease is currently held");
            return response.build();
        }
    };
    if crate::admin::adversary_sim_control::save_controller_lease(
        store,
        site_id,
        &lease,
        capabilities.state_write(),
    )
    .is_err()
    {
        return Response::new(500, "Key-value store error");
    }

    let mut transitions = Vec::new();
    let (preflight_state, mut preflight_transitions) =
        crate::admin::adversary_sim::reconcile_state(now, cfg.adversary_sim_enabled, &state);
    state = preflight_state;
    transitions.append(&mut preflight_transitions);
    state = crate::admin::adversary_sim::select_desired_lane(now, requested_lane, &state);
    let mut desired_enabled = payload.enabled;

    if payload.enabled {
        if state.phase != crate::admin::adversary_sim::ControlPhase::Running {
            let duration = crate::admin::adversary_sim::clamp_duration_seconds(
                cfg.adversary_sim_duration_seconds,
            );
            match crate::admin::adversary_sim::start_state(now, duration, &state) {
                Ok((next_state, mut started_transitions)) => {
                    state = next_state;
                    transitions.append(&mut started_transitions);
                }
                Err(crate::admin::adversary_sim::StartError::QueueFull) => {
                    return Response::new(
                        409,
                        "Adversary simulation queue is full (queue_policy=reject_new)",
                    );
                }
            }
        }
    } else {
        let (stopping_state, mut stop_transitions) =
            crate::admin::adversary_sim::stop_state(now, "manual_off", &state);
        state = stopping_state;
        transitions.append(&mut stop_transitions);
    }

    let (reconciled_state, mut reconciled_transitions) =
        crate::admin::adversary_sim::reconcile_state(now, desired_enabled, &state);
    state = reconciled_state;
    transitions.append(&mut reconciled_transitions);

    if state.phase == crate::admin::adversary_sim::ControlPhase::Off && desired_enabled {
        desired_enabled = false;
    }

    if save_adversary_sim_state_with_capability(store, site_id, &state, capabilities.state_write())
        .is_err()
    {
        return Response::new(500, "Key-value store error");
    }
    crate::admin::adversary_sim::project_effective_desired_state(&mut cfg, &state);
    let desired_state_label = control_state_label(desired_enabled);
    let desired_lane_label = control_desired_lane_label(&state);
    let actual_lane_label = control_actual_lane_label(&state);
    for transition in &transitions {
        log_adversary_sim_transition(
            store,
            site_id,
            req,
            auth,
            transition,
            Some(operation_id.as_str()),
        );
    }

    let operation_record = crate::admin::adversary_sim_control::ControlOperationRecord {
        operation_id: operation_id.clone(),
        requested_enabled: payload.enabled,
        requested_lane: requested_lane_label.clone(),
        requested_reason: requested_reason.clone(),
        desired_enabled,
        desired_lane: desired_lane_label.clone(),
        actual_phase: state.phase.as_str().to_string(),
        actual_lane: actual_lane_label.clone(),
        actor_scope: actor_scope.clone(),
        session_scope: session_scope.clone(),
        idempotency_key_hash: idempotency_key_hash.clone(),
        payload_hash: payload_hash.clone(),
        created_at: now,
        completed_at: now,
        decision: crate::admin::adversary_sim_control::ControlDecision::Accepted,
        decision_reason: "accepted".to_string(),
        origin_verdict: origin_verdict.clone(),
        lease_fencing_token: Some(lease.fencing_token),
    };
    let operation_key =
        crate::admin::adversary_sim_control::control_operation_key(site_id, operation_id.as_str());
    if crate::admin::adversary_sim_control::save_operation_record(
        store,
        &operation_key,
        &operation_record,
        capabilities.state_write(),
    )
    .is_err()
    {
        return Response::new(500, "Key-value store error");
    }

    let idempotency_record = crate::admin::adversary_sim_control::IdempotencyRecord {
        operation_id: operation_id.clone(),
        payload_hash,
        actor_scope: actor_scope.clone(),
        session_scope: session_scope.clone(),
        created_at: now,
        expires_at: now
            .saturating_add(crate::admin::adversary_sim_control::IDEMPOTENCY_TTL_SECONDS),
    };
    if crate::admin::adversary_sim_control::save_idempotency_record(
        store,
        &idempotency_store_key,
        &idempotency_record,
        capabilities.state_write(),
    )
    .is_err()
    {
        return Response::new(500, "Key-value store error");
    }
    if crate::admin::adversary_sim_control::save_debounce_timestamp(
        store,
        &debounce_key,
        now,
        capabilities.state_write(),
    )
    .is_err()
    {
        return Response::new(500, "Key-value store error");
    }

    log_adversary_sim_control_audit(
        store,
        req,
        auth,
        &crate::admin::adversary_sim_control::ControlAuditRecord {
            operation_id: Some(operation_id.clone()),
            actor_scope,
            session_scope,
            decision: crate::admin::adversary_sim_control::ControlDecision::Accepted,
            reason: "accepted".to_string(),
            origin_verdict,
            idempotency_key_hash: Some(idempotency_key_hash.clone()),
            request_origin,
            requested_state: Some(requested_state_label),
            requested_lane: requested_lane_label.clone(),
            desired_state: Some(desired_state_label.clone()),
            desired_lane: desired_lane_label.clone(),
            actual_state: state.phase.as_str().to_string(),
            actual_lane: actual_lane_label.clone(),
        },
        capabilities.audit_write(),
    );

    if let Err(()) = crate::admin::oversight_agent::maybe_trigger_post_sim_agent_cycle(
        store,
        site_id,
        &previous_state,
        &state,
        now,
    ) {
        crate::log_line(
            "[oversight-agent] post-sim trigger failed during control transition; continuing without agent payload",
        );
    }

    let response = json!({
        "operation_id": operation_id,
        "decision": "accepted",
        "requested_enabled": payload.enabled,
        "phase_trace": ["plan", "execute", "collect_evidence", "publish_report"],
        "requested_state": {
            "enabled": payload.enabled,
            "lane": requested_lane_label,
            "reason": requested_reason
        },
        "accepted_state": {
            "desired_enabled": desired_enabled,
            "desired_lane": desired_lane_label,
            "actual_phase": state.phase.as_str(),
            "active_lane": actual_lane_label
        },
        "idempotency": {
            "key_hash": idempotency_key_hash,
            "replayed": false,
            "ttl_seconds": crate::admin::adversary_sim_control::IDEMPOTENCY_TTL_SECONDS
        },
        "status": adversary_sim_status_payload(store, site_id, &cfg, &state, now),
        "config": super::api::admin_config_settings_payload(&cfg),
        "runtime": super::api::admin_config_runtime_payload(
            &cfg,
            super::api::challenge_threshold_default(),
            super::api::not_a_bot_threshold_default(),
            super::api::maze_threshold_default()
        ),
    });
    Response::new(200, serde_json::to_string(&response).unwrap())
}
