use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

pub(crate) fn handle_admin_events<S>(req: &Request, store: &S) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    let hours = super::api::query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let forensic_mode = super::api::forensic_access_mode(req.query());
    let now = crate::admin::now_ts();
    let mut events = super::api::load_recent_monitoring_event_records(store, now, hours);
    let mut ip_counts = std::collections::HashMap::new();
    let mut event_counts = std::collections::HashMap::new();

    for event in &events {
        if let Some(ip) = &event.entry.ip {
            let key = if forensic_mode {
                ip.clone()
            } else {
                super::api::pseudonymize_ip_identifier(ip.as_str())
            };
            *ip_counts.entry(key).or_insert(0u32) += 1;
        }
        *event_counts
            .entry(format!("{:?}", event.entry.event))
            .or_insert(0u32) += 1;
    }

    events.sort_by(|a, b| b.entry.ts.cmp(&a.entry.ts));
    let unique_ips = ip_counts.len();
    let mut top_ips: Vec<_> = ip_counts.into_iter().collect();
    top_ips.sort_by(|a, b| b.1.cmp(&a.1));
    let top_ips: Vec<_> = top_ips.into_iter().take(10).collect();
    let recent_events_raw: Vec<_> = events.iter().take(100).cloned().collect();
    let recent_events = super::api::present_event_records(recent_events_raw.as_slice(), forensic_mode);
    let body = serde_json::to_string(&json!({
        "recent_events": recent_events,
        "event_counts": event_counts,
        "top_ips": top_ips,
        "unique_ips": unique_ips,
        "security_mode": super::api::security_view_mode_label(forensic_mode),
        "security_privacy": super::api::security_privacy_payload(store, now, hours, forensic_mode)
    }))
    .unwrap();
    Response::new(200, body)
}

pub(crate) fn handle_admin_monitoring<S>(req: &Request, store: &S) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    let hours = super::api::query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = super::api::query_u64_param(req.query(), "limit", 10).clamp(1, 50) as usize;
    let forensic_mode = super::api::forensic_access_mode(req.query());
    let now = crate::admin::now_ts();
    let bootstrap_mode = crate::request_validation::query_param(req.query(), "bootstrap")
        .map(|value| matches!(value.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);
    let edge_bounded_details_mode =
        !bootstrap_mode && crate::config::gateway_deployment_profile().is_edge();
    let hot_read_eligible =
        super::api::monitoring_bootstrap_hot_read_request_eligible(hours, forensic_mode);
    let (summary, mut details, bootstrap_window_end_cursor) =
        if hot_read_eligible && (bootstrap_mode || edge_bounded_details_mode) {
            super::api::monitoring_bootstrap_hot_read_payload(store, "default", hours, limit)
        } else {
            let summary = crate::observability::monitoring::summarize_with_store(store, hours, limit);
            let (details, window_end_cursor) = if bootstrap_mode || edge_bounded_details_mode {
                super::api::monitoring_bootstrap_details_payload(
                    store,
                    "default",
                    hours,
                    limit,
                    forensic_mode,
                )
            } else {
                (
                    super::api::monitoring_details_payload(
                        store,
                        "default",
                        hours,
                        limit,
                        forensic_mode,
                    ),
                    None,
                )
            };
            (summary, details, window_end_cursor)
        };
    if edge_bounded_details_mode {
        if let Some(recent_events_window) = details
            .get_mut("events")
            .and_then(|events| events.get_mut("recent_events_window"))
            .and_then(|value| value.as_object_mut())
        {
            recent_events_window.insert(
                "response_shaping_reason".to_string(),
                serde_json::Value::String("edge_profile_bounded_details".to_string()),
            );
        }
    }
    let snapshot_latest_ts = super::api::latest_monitoring_snapshot_ts(&details);
    let freshness =
        super::api::freshness_health_payload(now, snapshot_latest_ts, false, "none", "snapshot_poll");
    let retention_health = details.get("retention_health").cloned().unwrap_or_else(|| {
        serde_json::to_value(crate::observability::retention::retention_health(store))
            .unwrap_or_else(|_| json!({}))
    });
    let security_privacy = details
        .get("security_privacy")
        .cloned()
        .unwrap_or_else(|| super::api::security_privacy_payload(store, now, hours, forensic_mode));
    let mut payload = json!({
        "summary": summary,
        "prometheus": super::api::monitoring_prometheus_helper_payload(),
        "details": details,
        "freshness_slo": super::api::freshness_slo_payload(),
        "load_envelope": super::api::load_envelope_payload(),
        "freshness": freshness,
        "retention_health": retention_health,
        "security_privacy": security_privacy
    });
    if let Some(window_end_cursor) = bootstrap_window_end_cursor {
        payload["window_end_cursor"] = serde_json::Value::String(window_end_cursor);
    }

    let supports_gzip = super::api::request_accepts_gzip(req);
    let initial_uncompressed = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());
    let initial_payload_kb = (initial_uncompressed.len() as f64) / 1024.0;
    let initial_compression =
        super::api::monitoring_compression_report(initial_uncompressed.as_slice(), supports_gzip);
    super::api::update_monitoring_cost_governance_transport_fields(
        &mut payload,
        initial_payload_kb,
        &initial_compression,
    );

    let mut uncompressed = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());
    let final_payload_kb = (uncompressed.len() as f64) / 1024.0;
    let final_compression =
        super::api::monitoring_compression_report(uncompressed.as_slice(), supports_gzip);
    super::api::update_monitoring_cost_governance_transport_fields(
        &mut payload,
        final_payload_kb,
        &final_compression,
    );
    uncompressed = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());

    let body = if final_compression.negotiated {
        super::api::gzip_bytes(uncompressed.as_slice()).unwrap_or_else(|| uncompressed.clone())
    } else {
        uncompressed
    };
    let cost_state = payload
        .get("details")
        .and_then(|details| details.get("cost_governance"))
        .and_then(|cost| cost.get("degraded_state"))
        .and_then(|value| value.as_str())
        .unwrap_or("normal");

    let mut builder = Response::builder();
    builder
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("X-Shuma-Monitoring-Cost-State", cost_state)
        .header(
            "X-Shuma-Monitoring-Security-Mode",
            super::api::security_view_mode_label(forensic_mode),
        )
        .header(
            "X-Shuma-Monitoring-Query-Budget",
            payload
                .get("details")
                .and_then(|details| details.get("cost_governance"))
                .and_then(|cost| cost.get("query_budget_status"))
                .and_then(|value| value.as_str())
                .unwrap_or("within_budget"),
        );
    if final_compression.negotiated {
        builder
            .header("Content-Encoding", "gzip")
            .header("Vary", "Accept-Encoding");
    }
    builder.body(body).build()
}

pub(crate) fn handle_admin_monitoring_delta<S>(req: &Request, store: &S) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    let hours = super::api::query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = super::api::query_u64_param(req.query(), "limit", 100)
        .clamp(1, super::api::MONITORING_STREAM_MAX_BUFFER_EVENTS as u64) as usize;
    let forensic_mode = super::api::forensic_access_mode(req.query());
    let after_cursor = super::api::resolve_after_cursor(req);
    if let Err(msg) = super::api::validate_after_cursor(after_cursor.as_str()) {
        return Response::new(400, msg);
    }

    let now = crate::admin::now_ts();
    let (
        event_rows,
        recent_sim_runs,
        latest_window_ts,
        window_end_cursor,
        next_cursor,
        has_more,
        overflow,
        security_privacy,
    ) = if super::api::monitoring_delta_hot_read_bootstrap_eligible(
        hours,
        limit,
        forensic_mode,
        after_cursor.as_str(),
    ) {
        super::api::monitoring_delta_hot_read_bootstrap_payload(store, "default", now, limit)
    } else {
        let selection =
            super::api::load_monitoring_cursor_page(store, now, hours, after_cursor.as_str(), limit, forensic_mode);
        let latest_window_ts = selection.latest_window_ts;
        let window_end_cursor = selection.window_end_cursor.clone();
        let next_cursor = selection.next_cursor.clone();
        let has_more = selection.has_more;
        let overflow = selection.overflow;
        let page_rows = selection.rows;
        let event_rows: Vec<serde_json::Value> =
            page_rows.iter().map(super::api::cursor_event_row_payload).collect();
        let recent_sim_runs =
            crate::observability::hot_read_projection::load_monitoring_recent_sim_runs_hot_read(
                store, "default", now,
            )
            .payload
            .recent_sim_runs;
        (
            event_rows,
            recent_sim_runs,
            latest_window_ts,
            window_end_cursor,
            next_cursor,
            has_more,
            overflow,
            super::api::security_privacy_payload(store, now, hours, forensic_mode),
        )
    };
    let etag = super::api::delta_page_etag(next_cursor.as_str(), event_rows.len(), has_more, overflow);
    let freshness = super::api::freshness_health_payload(
        now,
        latest_window_ts,
        has_more,
        overflow,
        "cursor_delta_poll",
    );

    if super::api::request_if_none_match(req).as_deref() == Some(etag.as_str()) {
        return Response::builder()
            .status(304)
            .header("Cache-Control", "no-store")
            .header("ETag", etag.as_str())
            .body("")
            .build();
    }

    let body = serde_json::to_string(&json!({
        "cursor_contract": {
            "version": "monitoring-event-cursor.v1",
            "ordering": "strict_monotonic_cursor_ascending",
            "cursor_source": "eventlog:v2 key ordering",
            "overflow_taxonomy": ["none", "limit_exceeded"]
        },
        "security_mode": super::api::security_view_mode_label(forensic_mode),
        "security_privacy": security_privacy,
        "freshness_slo": super::api::freshness_slo_payload(),
        "load_envelope": super::api::load_envelope_payload(),
        "hours": hours,
        "limit": limit,
        "after_cursor": after_cursor,
        "window_end_cursor": window_end_cursor,
        "next_cursor": next_cursor,
        "has_more": has_more,
        "overflow": overflow,
        "events": event_rows,
        "recent_sim_runs": recent_sim_runs,
        "freshness": freshness,
        "stream_supported": true,
        "stream_endpoint": "/admin/monitoring/stream"
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("ETag", etag.as_str())
        .header(
            "X-Shuma-Monitoring-Security-Mode",
            super::api::security_view_mode_label(forensic_mode),
        )
        .body(body)
        .build()
}

pub(crate) fn handle_admin_monitoring_stream<S>(req: &Request, store: &S) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let hours = super::api::query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = super::api::query_u64_param(req.query(), "limit", 100)
        .clamp(1, super::api::MONITORING_STREAM_MAX_BUFFER_EVENTS as u64) as usize;
    let forensic_mode = super::api::forensic_access_mode(req.query());
    let after_cursor = super::api::resolve_after_cursor(req);
    if let Err(msg) = super::api::validate_after_cursor(after_cursor.as_str()) {
        return Response::new(400, msg);
    }

    let now = crate::admin::now_ts();
    let selection =
        super::api::load_monitoring_cursor_page(store, now, hours, after_cursor.as_str(), limit, forensic_mode);
    let latest_window_ts = selection.latest_window_ts;
    let window_end_cursor = selection.window_end_cursor.clone();
    let next_cursor = selection.next_cursor.clone();
    let has_more = selection.has_more;
    let overflow = selection.overflow;
    let page_rows = selection.rows;
    let freshness =
        super::api::freshness_health_payload(now, latest_window_ts, has_more, overflow, "sse");
    let event_rows: Vec<serde_json::Value> =
        page_rows.iter().map(super::api::cursor_event_row_payload).collect();
    let payload = json!({
        "cursor_contract": {
            "version": "monitoring-event-cursor.v1",
            "ordering": "strict_monotonic_cursor_ascending",
            "cursor_source": "eventlog:v2 key ordering",
            "overflow_taxonomy": ["none", "limit_exceeded"]
        },
        "security_mode": super::api::security_view_mode_label(forensic_mode),
        "security_privacy": super::api::security_privacy_payload(store, now, hours, forensic_mode),
        "freshness_slo": super::api::freshness_slo_payload(),
        "load_envelope": super::api::load_envelope_payload(),
        "stream_contract": super::api::stream_contract_payload(),
        "hours": hours,
        "limit": limit,
        "after_cursor": after_cursor,
        "window_end_cursor": window_end_cursor,
        "next_cursor": next_cursor,
        "has_more": has_more,
        "overflow": overflow,
        "events": event_rows,
        "recent_sim_runs": crate::observability::hot_read_projection::load_monitoring_recent_sim_runs_hot_read(
            store, "default", now,
        ).payload.recent_sim_runs,
        "freshness": freshness
    });
    let event_id = payload
        .get("next_cursor")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("");
    super::api::sse_single_event_response("monitoring_delta", event_id, &payload)
}

pub(crate) fn handle_admin_ip_bans_delta<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    let hours = super::api::query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = super::api::query_u64_param(req.query(), "limit", 100)
        .clamp(1, super::api::MONITORING_STREAM_MAX_BUFFER_EVENTS as u64) as usize;
    let forensic_mode = super::api::forensic_access_mode(req.query());
    let after_cursor = super::api::resolve_after_cursor(req);
    if let Err(msg) = super::api::validate_after_cursor(after_cursor.as_str()) {
        return Response::new(400, msg);
    }

    let now = crate::admin::now_ts();
    let rows: Vec<super::api::CursorEventRecord> =
        super::api::load_recent_event_records_with_keys(store, now, hours)
            .into_iter()
            .filter(|stored| {
                matches!(
                    stored.record.entry.event,
                    crate::admin::EventType::Ban | crate::admin::EventType::Unban
                )
            })
            .map(|stored| super::api::CursorEventRecord {
                cursor: super::api::build_event_cursor(stored.record.entry.ts, stored.storage_key.as_str()),
                record: super::api::present_event_record(&stored.record, forensic_mode),
            })
            .collect();
    let latest_window_ts = super::api::latest_event_ts(rows.as_slice());
    let window_end_cursor = rows
        .iter()
        .map(|row| row.cursor.clone())
        .max()
        .unwrap_or_default();
    let (page_rows, next_cursor, has_more, overflow) =
        super::api::paginate_cursor_rows(rows, after_cursor.as_str(), limit);
    let event_rows: Vec<serde_json::Value> =
        page_rows.iter().map(super::api::cursor_event_row_payload).collect();

    let cfg = crate::config::Config::load(store, site_id).ok();
    let active_ban_snapshot =
        super::api::list_active_ban_snapshot_view(store, site_id, cfg.as_ref(), forensic_mode);
    let active_bans_payload = active_ban_snapshot.bans.clone();

    let etag = super::api::delta_page_etag(
        next_cursor.as_str(),
        event_rows.len().saturating_add(active_bans_payload.len()),
        has_more,
        overflow,
    );
    if super::api::request_if_none_match(req).as_deref() == Some(etag.as_str()) {
        return Response::builder()
            .status(304)
            .header("Cache-Control", "no-store")
            .header("ETag", etag.as_str())
            .body("")
            .build();
    }

    let latest_ts = match (latest_window_ts, active_ban_snapshot.latest_ban_ts) {
        (Some(event_ts), Some(ban_ts)) => Some(event_ts.max(ban_ts)),
        (Some(event_ts), None) => Some(event_ts),
        (None, Some(ban_ts)) => Some(ban_ts),
        (None, None) => None,
    };
    let freshness = super::api::freshness_health_payload(
        now,
        latest_ts,
        has_more,
        overflow,
        "cursor_delta_poll",
    );

    let body = serde_json::to_string(&json!({
        "cursor_contract": {
            "version": "monitoring-event-cursor.v1",
            "ordering": "strict_monotonic_cursor_ascending",
            "cursor_source": "eventlog:v2 key ordering",
            "overflow_taxonomy": ["none", "limit_exceeded"]
        },
        "security_mode": super::api::security_view_mode_label(forensic_mode),
        "security_privacy": super::api::security_privacy_payload(store, now, hours, forensic_mode),
        "freshness_slo": super::api::freshness_slo_payload(),
        "load_envelope": super::api::load_envelope_payload(),
        "hours": hours,
        "limit": limit,
        "after_cursor": after_cursor,
        "window_end_cursor": window_end_cursor,
        "next_cursor": next_cursor,
        "has_more": has_more,
        "overflow": overflow,
        "events": event_rows,
        "active_bans": active_bans_payload,
        "active_bans_status": active_ban_snapshot.status,
        "active_bans_message": active_ban_snapshot.message,
        "freshness": freshness,
        "stream_supported": true,
        "stream_endpoint": "/admin/ip-bans/stream"
    }))
    .unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .header("ETag", etag.as_str())
        .header(
            "X-Shuma-Monitoring-Security-Mode",
            super::api::security_view_mode_label(forensic_mode),
        )
        .body(body)
        .build()
}

pub(crate) fn handle_admin_ip_bans_stream<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let hours = super::api::query_u64_param(req.query(), "hours", 24).clamp(1, 720);
    let limit = super::api::query_u64_param(req.query(), "limit", 100)
        .clamp(1, super::api::MONITORING_STREAM_MAX_BUFFER_EVENTS as u64) as usize;
    let forensic_mode = super::api::forensic_access_mode(req.query());
    let after_cursor = super::api::resolve_after_cursor(req);
    if let Err(msg) = super::api::validate_after_cursor(after_cursor.as_str()) {
        return Response::new(400, msg);
    }

    let now = crate::admin::now_ts();
    let rows: Vec<super::api::CursorEventRecord> =
        super::api::load_recent_event_records_with_keys(store, now, hours)
            .into_iter()
            .filter(|stored| {
                matches!(
                    stored.record.entry.event,
                    crate::admin::EventType::Ban | crate::admin::EventType::Unban
                )
            })
            .map(|stored| super::api::CursorEventRecord {
                cursor: super::api::build_event_cursor(stored.record.entry.ts, stored.storage_key.as_str()),
                record: super::api::present_event_record(&stored.record, forensic_mode),
            })
            .collect();
    let latest_window_ts = super::api::latest_event_ts(rows.as_slice());
    let window_end_cursor = rows
        .iter()
        .map(|row| row.cursor.clone())
        .max()
        .unwrap_or_default();
    let (page_rows, next_cursor, has_more, overflow) =
        super::api::paginate_cursor_rows(rows, after_cursor.as_str(), limit);
    let event_rows: Vec<serde_json::Value> =
        page_rows.iter().map(super::api::cursor_event_row_payload).collect();

    let cfg = crate::config::Config::load(store, site_id).ok();
    let active_ban_snapshot =
        super::api::list_active_ban_snapshot_view(store, site_id, cfg.as_ref(), forensic_mode);
    let active_bans_payload = active_ban_snapshot.bans.clone();
    let latest_ts = match (latest_window_ts, active_ban_snapshot.latest_ban_ts) {
        (Some(event_ts), Some(ban_ts)) => Some(event_ts.max(ban_ts)),
        (Some(event_ts), None) => Some(event_ts),
        (None, Some(ban_ts)) => Some(ban_ts),
        (None, None) => None,
    };
    let freshness =
        super::api::freshness_health_payload(now, latest_ts, has_more, overflow, "sse");
    let payload = json!({
        "cursor_contract": {
            "version": "monitoring-event-cursor.v1",
            "ordering": "strict_monotonic_cursor_ascending",
            "cursor_source": "eventlog:v2 key ordering",
            "overflow_taxonomy": ["none", "limit_exceeded"]
        },
        "security_mode": super::api::security_view_mode_label(forensic_mode),
        "security_privacy": super::api::security_privacy_payload(store, now, hours, forensic_mode),
        "freshness_slo": super::api::freshness_slo_payload(),
        "load_envelope": super::api::load_envelope_payload(),
        "stream_contract": super::api::stream_contract_payload(),
        "hours": hours,
        "limit": limit,
        "after_cursor": after_cursor,
        "window_end_cursor": window_end_cursor,
        "next_cursor": next_cursor,
        "has_more": has_more,
        "overflow": overflow,
        "events": event_rows,
        "active_bans": active_bans_payload,
        "active_bans_status": active_ban_snapshot.status,
        "active_bans_message": active_ban_snapshot.message,
        "freshness": freshness
    });
    let event_id = payload
        .get("next_cursor")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("");
    super::api::sse_single_event_response("ip_bans_delta", event_id, &payload)
}
