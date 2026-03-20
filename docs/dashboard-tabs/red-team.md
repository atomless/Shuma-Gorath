# Dashboard Tab: Red Team

Route: `#red-team`  
Component: [`dashboard/src/lib/components/dashboard/RedTeamTab.svelte`](../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte)

Purpose:

- Start or stop adversary simulation from the dedicated top-level operator surface.
- Keep operator intent, backend lifecycle truth, and retained-telemetry visibility understandable without mixing them into one toggle state.

Panel:

- `Adversary Simulation`:
  - on/off toggle backed by `POST /admin/adversary-sim/control`,
  - lifecycle copy rendered from backend status plus controller phase,
  - backend-timed run progress bar derived from `started_at`, `ends_at`, and `remaining_seconds`.
- `Recent Red Team Runs`:
  - recent adversary simulation run identifiers derived from a compact monitoring-backed run-history summary,
  - run-id linkage back to `Diagnostics` and `IP Bans`,
  - freshness-aware empty/degraded messaging so delayed telemetry is not misread as no activity.

Behavior:

- The switch reflects the latest operator intent immediately, even during the debounce window.
- The tab shares the dashboard refresh affordance used by `IP Bans`:
  - manual refresh and auto-refresh both hydrate the monitoring-backed run table,
  - the run table stays on the shared monitoring refresh path, but no longer infers run history from the bounded raw-event tail.
- Backend truth remains separate:
  - lifecycle copy uses backend phase/status,
  - the root `adversary-sim` class follows backend truth only,
  - submit/converge failures snap the switch back to the last backend-confirmed desired state.
- Enabling with zero configured frontier providers shows a confirmation dialog:
  - continue without frontier calls, or
  - cancel, add `SHUMA_FRONTIER_*_API_KEY` values, and restart the runtime.
- The controller is page-scoped, so switching away from `#red-team` does not pause convergence or running-state polling.

Reads and writes:

- Read: `GET /admin/adversary-sim/status`
- Write: `POST /admin/adversary-sim/control`

Notes:

- Retained simulation telemetry remains queryable after auto-off until retention expiry or explicit cleanup.
- Cleanup is intentionally not part of the tab UI; use `make telemetry-clean` or `POST /admin/adversary-sim/history/cleanup` when destructive retained-history removal is required.
