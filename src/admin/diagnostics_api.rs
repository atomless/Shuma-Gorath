use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

use super::recent_changes_ledger::{
    operator_snapshot_manual_change_row, record_operator_snapshot_recent_change_rows,
};

fn parse_operator_seed_sources_json(
    value: &serde_json::Value,
) -> Result<Vec<crate::maze::seeds::OperatorSeedSource>, String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "sources must be an array".to_string())?;
    let mut sources = Vec::with_capacity(entries.len());
    for entry in entries {
        let obj = entry
            .as_object()
            .ok_or_else(|| "each source must be an object".to_string())?;
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = obj
            .get("url")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let title = obj
            .get("title")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let keywords = obj
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let allow_seed_use = obj
            .get("allow_seed_use")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let robots_allowed = obj
            .get("robots_allowed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let body_excerpt = obj
            .get("body_excerpt")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        sources.push(crate::maze::seeds::OperatorSeedSource {
            id,
            url,
            title,
            description,
            keywords,
            allow_seed_use,
            robots_allowed,
            body_excerpt,
        });
    }
    Ok(sources)
}

pub(crate) fn handle_admin_maze_seed_sources<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore + crate::maze::state::MazeStateStore,
{
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };

    match *req.method() {
        Method::Get => {
            let sources = crate::maze::seeds::list_operator_sources(store);
            let cache = crate::maze::seeds::cached_seed_snapshot(store);
            let body = serde_json::to_string(&json!({
                "seed_provider": cfg.maze_seed_provider,
                "seed_refresh_interval_seconds": cfg.maze_seed_refresh_interval_seconds,
                "seed_refresh_rate_limit_per_hour": cfg.maze_seed_refresh_rate_limit_per_hour,
                "seed_refresh_max_sources": cfg.maze_seed_refresh_max_sources,
                "seed_metadata_only": cfg.maze_seed_metadata_only,
                "sources": sources,
                "cache": cache
            }))
            .unwrap();
            Response::new(200, body)
        }
        Method::Post => {
            let payload = match crate::request_validation::parse_json_body(
                req.body(),
                crate::request_validation::MAX_ADMIN_JSON_BYTES,
            ) {
                Ok(payload) => payload,
                Err(err) => return Response::new(400, err),
            };
            let Some(value) = payload.get("sources") else {
                return Response::new(400, "sources field is required");
            };
            let sources = match parse_operator_seed_sources_json(value) {
                Ok(sources) => sources,
                Err(err) => return Response::new(400, err),
            };
            if let Err(err) =
                crate::maze::seeds::save_operator_sources(store, &cfg, sources.clone())
            {
                return Response::new(400, err);
            }
            let changed_at_ts = crate::admin::now_ts();
            record_operator_snapshot_recent_change_rows(
                store,
                site_id,
                &[operator_snapshot_manual_change_row(
                    changed_at_ts,
                    "maze_seed_sources_update",
                    &["maze_seed_sources"],
                    &[
                        "suspicious_forwarded_bytes",
                        "suspicious_forwarded_requests",
                    ],
                    crate::admin::auth::get_admin_id(req).as_str(),
                    format!(
                        "operator maze seed sources updated: {} sources",
                        sources.len()
                    )
                    .as_str(),
                )],
                changed_at_ts,
            );
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: changed_at_ts,
                    event: crate::admin::EventType::AdminAction,
                    ip: None,
                    reason: Some("maze_seed_sources_update".to_string()),
                    outcome: Some(format!("sources={}", sources.len())),
                    admin: Some(crate::admin::auth::get_admin_id(req)),
                },
            );
            let body = serde_json::to_string(&json!({
                "updated": true,
                "source_count": sources.len()
            }))
            .unwrap();
            Response::new(200, body)
        }
        _ => Response::new(405, "Method Not Allowed"),
    }
}

pub(crate) fn handle_admin_maze_preview<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    let requested_path = crate::request_validation::query_param(req.query(), "path");
    let html = crate::maze::preview::render_admin_preview(&cfg, requested_path.as_deref());
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-store")
        .body(html)
        .build()
}

pub(crate) fn handle_admin_tarpit_preview<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore,
{
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };

    crate::tarpit::runtime::build_progressive_entry_response(
        &cfg,
        "admin-preview-ip-bucket",
        "admin-preview-ua-bucket",
        "/admin/tarpit/preview",
        crate::tarpit::progress_path(),
    )
}

pub(crate) fn handle_admin_maze_seed_refresh<S>(req: &Request, store: &S, site_id: &str) -> Response
where
    S: crate::challenge::KeyValueStore + crate::maze::state::MazeStateStore,
{
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    let cfg = match crate::config::load_runtime_cached(store, site_id) {
        Ok(cfg) => cfg,
        Err(err) => return Response::new(500, err.user_message()),
    };
    if cfg.maze_seed_provider != crate::config::MazeSeedProvider::Operator {
        return Response::new(
            409,
            "maze_seed_provider must be 'operator' for manual seed refresh",
        );
    }

    let now = crate::admin::now_ts();
    let refreshed = match crate::maze::seeds::manual_refresh_operator_corpus(store, &cfg, now) {
        Ok(refreshed) => refreshed,
        Err(err) => {
            if err.contains("rate limit exceeded") {
                return Response::new(429, err);
            }
            return Response::new(400, err);
        }
    };
    record_operator_snapshot_recent_change_rows(
        store,
        site_id,
        &[operator_snapshot_manual_change_row(
            now,
            "maze_seed_refresh",
            &["maze_seed_refresh"],
            &[
                "suspicious_forwarded_bytes",
                "suspicious_forwarded_requests",
            ],
            crate::admin::auth::get_admin_id(req).as_str(),
            format!(
                "maze seed corpus refreshed: provider={} version={} sources={}",
                refreshed.provider, refreshed.version, refreshed.source_count
            )
            .as_str(),
        )],
        now,
    );
    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: now,
            event: crate::admin::EventType::AdminAction,
            ip: None,
            reason: Some("maze_seed_refresh".to_string()),
            outcome: Some(format!(
                "provider={} version={} terms={} sources={}",
                refreshed.provider,
                refreshed.version,
                refreshed.terms.len(),
                refreshed.source_count
            )),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );
    let body = serde_json::to_string(&json!({
        "refreshed": true,
        "provider": refreshed.provider,
        "version": refreshed.version,
        "metadata_only": refreshed.metadata_only,
        "source_count": refreshed.source_count,
        "term_count": refreshed.terms.len()
    }))
    .unwrap();
    Response::new(200, body)
}
