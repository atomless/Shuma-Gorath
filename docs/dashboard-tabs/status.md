# Dashboard Tab: Status

Route: `#status`  
Component: [`dashboard/src/lib/components/dashboard/StatusTab.svelte`](../../dashboard/src/lib/components/dashboard/StatusTab.svelte)

Purpose:

- Fast, human-readable status summary of key defense features plus operator health for the dashboard read path.

What it shows:

- Feature status cards derived from current runtime config snapshot (fail mode, HTTPS, PoW, challenges, CDP, maze, tarpit, JS required, GEO, IP range policy, rate limiting).
- Appended operational posture cards:
  - runtime and deployment posture (`runtime_environment`, gateway deployment profile, and local-direct localhost prod mode), with a link to the operator quick-reference matrix for dev vs local prod-like vs deployed production,
  - admin config write posture (`admin_config_write_enabled`).
- Operator health sections:
  - heartbeat-owned backend connection state, last heartbeat success/failure, and failure threshold posture,
  - monitoring-feed freshness and IP-ban-feed freshness, including lag, last event time, and partial-data warnings when present,
  - retention-worker health sourced from `/admin/monitoring` (`retention_health`).
- Runtime performance telemetry for dashboard refresh behavior:
  - thresholds apply to the dashboard auto-refresh tabs (`monitoring`, `ip-bans`, and `red-team`),
  - fetch latency (last, avg, p95),
  - render timing (last, avg, p95),
  - polling skip/resume counters.

Writes:

- Read-only tab (no config writes).
