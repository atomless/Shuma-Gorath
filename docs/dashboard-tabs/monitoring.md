# Dashboard Tab: Monitoring

Route: `#monitoring`  
Component: [`dashboard/src/lib/components/dashboard/MonitoringTab.svelte`](../../dashboard/src/lib/components/dashboard/MonitoringTab.svelte)

Purpose:

- Operational visibility for live detections, bans, challenge outcomes, and trend data.

What it shows:

- Overview cards: total bans, active bans, events (24h), unique IPs.
- Charts: event types, top IPs, and time-series events for `60m/24h/7d/30d`.
- Recent Events table.
- CDP detections table and summary cards.
- Maze, honeypot, challenge, PoW, rate-limiting, GEO, and IP-range monitoring sections.
- External monitoring helper with Prometheus and JSON API examples.

Refresh behavior:

- Supports manual refresh and optional auto-refresh.
- Auto-refresh is only available on this tab and `IP Bans`.
- Uses consolidated `/admin/monitoring` snapshot refresh and bounded local cache.
- Simulation-tagged events are included in monitoring data by default in runtime-dev and remain distinguishable via per-event simulation metadata fields.

Writes:

- Read-only tab (no config writes).
