# Dashboard Tab: Monitoring

Route: `#monitoring`  
Component: [`dashboard/src/lib/components/dashboard/MonitoringTab.svelte`](../../dashboard/src/lib/components/dashboard/MonitoringTab.svelte)

Purpose:

- Human-readable accountability surface for Shuma's closed feedback loop.
- Keep loop outcome and controller judgment visible without collapsing into subsystem-forensics detail.
- Avoid mixing operator accountability with the deeper diagnostics workflow that now lives in `#diagnostics`.

Current behavior:

- Shows a top-level `Closed-Loop Accountability` framing panel that makes the tab's role explicit.
- Exposes the first bounded Monitoring information architecture for the overhaul:
  - `Current Status`
  - `Recent Loop Progress`
  - `Outcome Frontier`
  - `What The Loop Decided`
  - `Where The Pressure Sits`
  - `Trust And Blockers`
- Keeps these sections intentionally explanatory and static in `MON-OVERHAUL-1A`; machine-first data projection belongs to `MON-OVERHAUL-1B` and `MON-OVERHAUL-1C`.
- Directs operators and contributors to `#diagnostics` for deep subsystem inspection and rawer contributor-facing telemetry.

Refresh behavior:

- No tab-local manual or auto-refresh controls are exposed yet.
- Shared config/bootstrap state still loads through the dashboard runtime so global controls remain available while the Monitoring contract is being rebuilt.

Writes:

- Read-only tab (no config writes).
