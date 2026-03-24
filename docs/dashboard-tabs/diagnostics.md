# Dashboard Tab: Diagnostics

Route: `#diagnostics`  
Component: [`dashboard/src/lib/components/dashboard/DiagnosticsTab.svelte`](../../dashboard/src/lib/components/dashboard/DiagnosticsTab.svelte)

Purpose:

- Deep-inspection and contributor diagnostics surface.
- Home for subsystem-by-subsystem monitoring detail that no longer belongs in the top Monitoring accountability flow.
- External-traffic telemetry only: operator actions, dashboard/config interactions, and manual ban/unban events are excluded so the analysis stays focused on real incoming traffic.

What it shows:

- Intro panel clarifying that `Diagnostics` owns subsystem investigation while `Monitoring` owns closed-loop accountability.
- Overview cards: total bans, active bans, events (24h), unique IPs.
- Charts: enforced event types, top IPs by enforced events, and enforced-event time series for `60m/24h/7d/30d`.
- Per-defense trend blocks (trigger volume, pass/fail/escalate mix, ban outcomes, execution mode, source-label breakdown).
- Recent Events table with fast filters (`origin`, `mode`, `scenario`, `lane`, `defense`, `outcome`).
- CDP detections table and summary cards, including total detections, detection-triggered bans, and fingerprint mismatch/transition counters.
- Maze, honeypot, challenge, PoW, rate-limiting, GEO, and IP-range monitoring sections.
- Explicit subsection ownership:
  - `Traffic Overview`
  - `Defense Breakdown`
  - `Recent External Traffic`
  - `Defense-Specific Diagnostics`
  - `Telemetry Diagnostics`
  - `External Monitoring`
- Collapsed `Telemetry Diagnostics` section near the bottom:
  - low-level monitoring-feed freshness and read-path diagnostics,
  - low-level IP-ban-feed freshness and read-path diagnostics,
  - rolling raw external-traffic telemetry feed rows.
- External monitoring helper with Prometheus and JSON API examples.

Refresh behavior:

- Supports manual refresh only.
- Auto-refresh is intentionally not available on this tab.
- Uses consolidated `/admin/monitoring` snapshot refresh and bounded local cache.
- Retains the existing legacy monitoring read model in `MON-OVERHAUL-1A`; the change in this tranche is ownership and sectioning, not a backend contract rewrite.
- Simulation-tagged events are included whenever simulation traffic is present and remain distinguishable via per-event simulation metadata fields.
- Contributor-style freshness, transport, overflow, and raw-feed diagnostics are intentionally kept in the collapsed diagnostics section.
- Shadow-mode traffic remains visually distinct from enforced traffic:
  - primary charts stay focused on enforced activity,
  - Recent Events include an explicit execution `Mode`,
  - defense trend rows split `Enforced` and `Shadow` counts.
- Recent-events empty states are explicit:
  - degraded/stale freshness uses warning language,
  - filter mismatch states are distinct from true no-data states,
  - tab-level fetch errors render explicit error context.

Writes:

- Read-only tab (no config writes).
