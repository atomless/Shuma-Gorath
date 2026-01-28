use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
/// Event types for activity logging
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventType {
    Ban,
    Unban,
    Challenge,
    Block,
    AdminAction,
}

/// Event log entry
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventLogEntry {
    pub ts: u64, // unix timestamp
    pub event: EventType,
    pub ip: Option<String>,
    pub reason: Option<String>,
    pub outcome: Option<String>,
    pub admin: Option<String>,
}

/// Append an event to the event log (simple append-only, time-bucketed by hour)
pub fn log_event(store: &Store, entry: &EventLogEntry) {
    let hour = entry.ts / 3600;
    let key = format!("eventlog:{}", hour);
    let mut log: Vec<EventLogEntry> = store.get(&key)
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_else(Vec::new);
    log.push(entry.clone());
    let _ = store.set(&key, serde_json::to_vec(&log).unwrap().as_slice());
}

/// Utility to get current unix timestamp
pub fn now_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
// src/admin.rs
// Admin API endpoints for WASM Bot Trap
// Provides HTTP endpoints for ban management and analytics, protected by API key auth.

use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;
use serde_json::json;

/// Returns true if the path is a valid admin endpoint (prevents path traversal/abuse).
fn sanitize_path(path: &str) -> bool {
    matches!(path, "/admin" | "/admin/ban" | "/admin/unban" | "/admin/analytics" | "/admin/events")
}

/// Handles all /admin API endpoints. Requires valid API key in Authorization header.
/// Supports:
///   - /admin/ban: List all bans for the site
///   - /admin/unban?ip=...: Remove a ban for an IP
///   - /admin/analytics: Return ban count
///   - /admin: API help
pub fn handle_admin(req: &Request) -> Response {
    // Require valid API key
    if !crate::auth::is_authorized(req) {
        return Response::new(401, "Unauthorized: Invalid or missing API key");
    }
    let path = req.path();
    if !sanitize_path(path) {
        return Response::new(400, "Bad Request: Invalid admin endpoint");
    }
    let store = Store::open_default().expect("open default store");
    let site_id = "default";

    match path {
                "/admin/events" => {
                    // Query event log for recent events, top IPs, and event statistics
                    // Query params: ?hours=N (default 24)
                    let hours: u64 = req.query().strip_prefix("hours=").and_then(|v| v.parse().ok()).unwrap_or(24);
                    let now = now_ts();
                    let mut events: Vec<EventLogEntry> = Vec::new();
                    let mut ip_counts = std::collections::HashMap::new();
                    let mut event_counts = std::collections::HashMap::new();
                    let store = &store;
                    for h in 0..hours {
                        let hour = (now / 3600).saturating_sub(h);
                        let key = format!("eventlog:{}", hour);
                        if let Ok(Some(val)) = store.get(&key) {
                            if let Ok(log) = serde_json::from_slice::<Vec<EventLogEntry>>(&val) {
                                for e in &log {
                                    // Only include events within the time window
                                    if e.ts >= now - hours * 3600 {
                                        if let Some(ip) = &e.ip {
                                            *ip_counts.entry(ip.clone()).or_insert(0u32) += 1;
                                        }
                                        *event_counts.entry(format!("{:?}", e.event)).or_insert(0u32) += 1;
                                        events.push(e.clone());
                                    }
                                }
                            }
                        }
                    }
                    // Sort events by timestamp descending
                    events.sort_by(|a, b| b.ts.cmp(&a.ts));
                    // Top 10 IPs
                    let mut top_ips: Vec<_> = ip_counts.into_iter().collect();
                    top_ips.sort_by(|a, b| b.1.cmp(&a.1));
                    let top_ips: Vec<_> = top_ips.into_iter().take(10).collect();
                    let body = serde_json::to_string(&json!({
                        "recent_events": events.iter().take(100).collect::<Vec<_>>(),
                        "event_counts": event_counts,
                        "top_ips": top_ips,
                    })).unwrap();
                    // Log admin analytics view
                    log_event(store, &EventLogEntry {
                        ts: now_ts(),
                        event: EventType::AdminAction,
                        ip: None,
                        reason: Some("events_view".to_string()),
                        outcome: Some(format!("{} events", events.len())),
                        admin: Some(crate::auth::get_admin_id(req)),
                    });
                    Response::new(200, body)
                }
        "/admin/ban" => {
            // List all bans for this site (keys starting with ban:site_id:)
            let mut bans = vec![];
            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with(&format!("ban:{}:", site_id)) {
                        if let Ok(Some(val)) = store.get(&k) {
                            if let Ok(ban) = serde_json::from_slice::<crate::ban::BanEntry>(&val) {
                                bans.push(json!({"ip": k.split(':').last().unwrap_or("?"), "reason": ban.reason, "expires": ban.expires}));
                            }
                        }
                    }
                }
            }
            // Log admin action
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("ban_list".to_string()),
                outcome: Some(format!("{} bans listed", bans.len())),
                admin: Some(crate::auth::get_admin_id(req)),
            });
            let body = serde_json::to_string(&json!({"bans": bans})).unwrap();
            Response::new(200, body)
        }
        "/admin/unban" => {
            // Unban IP (expects ?ip=...)
            let ip = req.query().strip_prefix("ip=").unwrap_or("");
            if ip.is_empty() {
                return Response::new(400, "Missing ip param");
            }
            let key = format!("ban:{}:{}", site_id, ip);
            let _ = store.delete(&key);
            // Log unban event
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::Unban,
                ip: Some(ip.to_string()),
                reason: Some("admin_unban".to_string()),
                outcome: Some("unbanned".to_string()),
                admin: Some(crate::auth::get_admin_id(req)),
            });
            Response::new(200, "Unbanned")
        }
        "/admin/analytics" => {
            // Return simple analytics: ban count
            let mut ban_count = 0;
            if let Ok(keys) = store.get_keys() {
                for k in keys {
                    if k.starts_with(&format!("ban:{}:", site_id)) {
                        ban_count += 1;
                    }
                }
            }
            // Log admin analytics view
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("analytics_view".to_string()),
                outcome: Some(format!("ban_count={}", ban_count)),
                admin: Some(crate::auth::get_admin_id(req)),
            });
            let body = serde_json::to_string(&json!({"ban_count": ban_count})).unwrap();
            Response::new(200, body)
        }
        "/admin" => {
            // API help endpoint
            log_event(&store, &EventLogEntry {
                ts: now_ts(),
                event: EventType::AdminAction,
                ip: None,
                reason: Some("help".to_string()),
                outcome: None,
                admin: Some(crate::auth::get_admin_id(req)),
            });
            Response::new(200, "WASM Bot Trap Admin API. Use /admin/ban, /admin/unban?ip=IP, /admin/analytics.")
        }
        _ => Response::new(404, "Not found"),
    }
}
