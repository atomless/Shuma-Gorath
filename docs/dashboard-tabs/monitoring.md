# Dashboard Tab: Monitoring

Route: `#monitoring`  
Component: [`dashboard/src/lib/components/dashboard/MonitoringTab.svelte`](../../dashboard/src/lib/components/dashboard/MonitoringTab.svelte)

Purpose:

- Operational visibility for live detections, bans, challenge outcomes, and trend data.

What it shows:

- Overview cards: total bans, active bans, events (24h), unique IPs.
- Shadow Mode summary: simulated action totals, pass-through totals, and top would-act outcomes when `test_mode` is enabled.
- Charts: event types, top IPs, and time-series events for `60m/24h/7d/30d`.
- Per-defense trend blocks (trigger volume, pass/fail/escalate mix, ban outcomes, execution mode, source-label breakdown).
- Recent Events table with fast filters (`origin`, `mode`, `scenario`, `lane`, `defense`, `outcome`).
- CDP detections table and summary cards.
- Maze, honeypot, challenge, PoW, rate-limiting, GEO, and IP-range monitoring sections.
- External monitoring helper with Prometheus and JSON API examples.

Refresh behavior:

- Supports manual refresh and optional auto-refresh.
- Auto-refresh is available on this tab, `IP Bans`, and `Red Team`.
- Uses consolidated `/admin/monitoring` snapshot refresh and bounded local cache.
- Simulation-tagged events are included in monitoring data whenever simulation traffic is present and remain distinguishable via per-event simulation metadata fields.
- Test-mode shadow events remain visually distinct from enforced events:
  - summary cards report `Would ...` action totals separately from real enforcement,
  - Recent Events include an explicit execution `Mode`,
  - defense trend rows split `Enforced` and `Shadow` counts.
- Recent-events empty states are explicit:
  - degraded/stale freshness uses warning language,
  - filter mismatch states are distinct from true no-data states,
  - tab-level fetch errors render explicit error context.

Writes:

- Read-only tab (no config writes).
