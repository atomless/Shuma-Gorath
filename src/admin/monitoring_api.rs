use serde_json::json;
use spin_sdk::http::{Request, Response};

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
