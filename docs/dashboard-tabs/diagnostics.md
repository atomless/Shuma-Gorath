# Dashboard Tab: Diagnostics

Route: `#diagnostics`  
Component: [`dashboard/src/lib/components/dashboard/DiagnosticsTab.svelte`](../../dashboard/src/lib/components/dashboard/DiagnosticsTab.svelte)

Purpose:

- Deep-inspection and contributor diagnostics surface.
- Home for subsystem-by-subsystem monitoring detail that no longer belongs in either the Traffic picture or the top Game Loop accountability flow.
- Contributor-style proof that Shuma's telemetry and defence furniture is operational.

What it shows:

- CDP detections table and summary cards, including total detections, detection-triggered bans, and fingerprint mismatch/transition counters.
- Maze, honeypot, challenge, PoW, rate-limiting, GEO, and IP-range monitoring sections.
- Explicit subsection ownership:
  - `Telemetry Diagnostics`
  - `External Monitoring`
- `Telemetry Diagnostics` section near the bottom:
  - low-level monitoring-feed freshness and read-path diagnostics,
  - low-level IP-ban-feed freshness and read-path diagnostics,
  - rolling raw external-traffic telemetry feed rows.
- External monitoring helper with Prometheus and JSON API examples.

Refresh behavior:

- Supports manual refresh only.
- Auto-refresh is intentionally not available on this tab.
- Uses consolidated `/admin/monitoring` snapshot refresh and bounded local cache.
- Contributor-style freshness, transport, overflow, and raw-feed diagnostics are intentionally kept in the collapsed diagnostics section.

Writes:

- Read-only tab (no config writes).
