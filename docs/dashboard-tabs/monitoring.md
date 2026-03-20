# Dashboard Tab: Monitoring

Route: `#monitoring`  
Component: [`dashboard/src/lib/components/dashboard/MonitoringTab.svelte`](../../dashboard/src/lib/components/dashboard/MonitoringTab.svelte)

Purpose:

- Reserve a clean slate for the operator Monitoring overhaul.
- Avoid mixing the future operator decision surface with the legacy subsystem-by-subsystem diagnostics layout.

Current behavior:

- Shows a minimal transition panel explaining that the operator Monitoring overhaul is in progress.
- Directs operators and contributors to `#diagnostics` for the current deep-inspection surface.
- Keeps the tab truthful during the transition instead of pretending the old diagnostic surface is already the Monitoring contract Shuma wants long-term.

Refresh behavior:

- No dedicated auto-refresh surface behavior is exposed on this placeholder tab.
- Shared config/bootstrap state still loads through the dashboard runtime so global controls remain available.

Writes:

- Read-only tab (no config writes).
