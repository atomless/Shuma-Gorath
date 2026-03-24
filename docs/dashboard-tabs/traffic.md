# Dashboard Tab: Traffic

Route: `#traffic`  
Component: [`dashboard/src/lib/components/dashboard/TrafficTab.svelte`](../../dashboard/src/lib/components/dashboard/TrafficTab.svelte)

Purpose:

- Live and recent traffic visibility surface.
- Home for the bounded traffic picture that no longer belongs in either loop-accountability Game Loop or furniture-focused Diagnostics.
- Proof that traffic telemetry collection is alive and readable without dropping into contributor-style subsystem diagnostics.

What it shows:

- `Traffic Overview` without extra framing copy:
  - bounded overview cards,
  - enforced-event charts,
  - time-range switching for the bounded event picture.
- `Recent Events`:
  - recent event rows,
  - shared filters for origin, mode, scenario, lane, defense, and outcome,
  - explicit empty-state handling when freshness is degraded or no events match filters.
- Bottom freshness/read-path strip:
  - freshness state,
  - lag and last-event timing,
  - transport/read-path truth,
  - slow-consumer and overflow summary.

Refresh behavior:

- Shares the top-level dashboard refresh bar.
- Supports manual refresh.
- Supports bounded auto-refresh.
- Reuses the existing bounded monitoring refresh path, including cache/bootstrap/delta behavior, so the tab stays cost-effective and does not introduce a second heavyweight traffic-read model.
- Does not force extra shared-config refreshes just to render the traffic picture.

Writes:

- Read-only tab (no config writes).
