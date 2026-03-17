# Dashboard Tab: Status

Route: `#status`  
Component: [`dashboard/src/lib/components/dashboard/StatusTab.svelte`](../../dashboard/src/lib/components/dashboard/StatusTab.svelte)

Purpose:

- Fast, human-readable status summary of key defense features plus operator health for the dashboard read path.

What it shows:

- Feature status cards derived from the split admin config envelope:
  - writable settings from `GET /admin/config.config`,
  - read-only posture/runtime facts from `GET /admin/config.runtime`.
- Appended operational posture cards:
  - runtime and deployment posture (`runtime_environment`, gateway deployment profile, and local-direct localhost prod mode), with a link to the operator quick-reference matrix for dev vs local prod-like vs deployed production,
  - admin config write posture (`admin_config_write_enabled`).
- Operator health sections:
  - heartbeat-owned backend connection state, last heartbeat success/failure, last heartbeat failure class, and failure threshold posture,
  - monitoring-feed freshness and IP-ban-feed freshness, including lag, last event time, and partial-data warnings when present,
  - retention-worker health sourced from `/admin/monitoring` (`retention_health`).
- Runtime performance telemetry for dashboard refresh behavior:
  - thresholds apply to the dashboard auto-refresh tabs (`monitoring`, `ip-bans`, and `red-team`),
  - fetch latency (last, avg, p95),
  - render timing (last, avg, p95),
  - polling skip/resume counters.

How to interpret it:

- If `Dashboard Connectivity` stays healthy while `Telemetry Delivery Health` degrades, the problem is in monitoring or IP-ban delivery/read freshness rather than the admin-session heartbeat.
- If `Ignored non-heartbeat failures` rises while connection stays healthy, tab-local endpoint failures are being contained correctly and must not be mistaken for a backend disconnect.
- If `Ignored cancelled requests` rises during navigation/remount churn, the dashboard is correctly treating client-side aborts as local noise instead of connection loss.
- If `Retention Health` degrades or stalls, recent traffic may still be live while older evidence ages out less reliably; treat that as a telemetry-lifecycle issue, not a connection issue.

Writes:

- Read-only tab (no config writes).
