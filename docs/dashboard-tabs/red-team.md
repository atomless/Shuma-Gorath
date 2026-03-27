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
  - backend-timed run progress bar derived from `started_at`, `ends_at`, and `remaining_seconds`,
  - `Status Truth` readout showing whether generation counters and lane diagnostics come directly from runtime control state or from recovered persisted-event lower-bound evidence,
  - bounded persisted-event evidence summary when recent monitoring facts were used to recover completed-run truth.
- `Recent Red Team Runs`:
  - recent adversary simulation run identifiers derived from a compact monitoring-backed run-history summary,
  - observed fulfillment modes, category coverage, and defense-surface closure summaries for each bounded run row,
  - freshness-aware empty/degraded messaging so delayed telemetry is not misread as no activity.
- `Scrapling`:
  - receipt-backed projection of the most recent Scrapling run visible in the bounded monitoring window,
  - observed Scrapling personas, observed non-human taxonomy categories, and high-level defense-surface closure counts,
  - a full surface checklist that shows every canonical surface row in the current Scrapling defense-surface matrix with:
    - a tick when the latest run was required to hit it and did hit it,
    - a cross when the latest run was required to hit it but did not,
    - a dash when the latest run was not expected to hit it,
    - explicit state text so a required miss is no longer ambiguous:
      - `attempted and blocked` means Scrapling reached the surface and failed its contract there,
      - `blocked by prerequisite` means the surface was required but an earlier prerequisite surface did not satisfy its required pass contract,
      - `required but unreached` means the latest run never produced an attempt receipt for that required surface,
    - dependency labels where they matter:
      - `independent surface` means the row is not modeled as downstream of another owned surface,
      - `co-materialized with ...` means the row is expected to show up as part of the same browser interaction path as another owned surface,
      - `blocked by prerequisite ...` means the current coverage miss is explicitly downstream of an earlier required pass surface,
  - per-surface sample receipts so operators can inspect why a required surface satisfied or blocked.

Behavior:

- The switch reflects the latest operator intent immediately, even during the debounce window.
- The tab shares the dashboard refresh affordance used by `IP Bans`:
  - manual refresh and auto-refresh both hydrate the monitoring-backed run table,
  - the run table stays on the shared monitoring refresh path, but no longer infers run history from the bounded raw-event tail.
- Backend truth remains separate:
  - lifecycle copy uses backend phase/status,
  - the root `adversary-sim` class follows backend truth only,
  - submit/converge failures snap the switch back to the last backend-confirmed desired state,
  - truth-basis markers now distinguish direct runtime counters from recovered persisted-event lower-bound evidence instead of leaving operators to infer that distinction.
- Enabling with zero configured frontier providers shows a confirmation dialog:
  - continue without frontier calls, or
  - cancel, add `SHUMA_FRONTIER_*_API_KEY` values, and restart the runtime.
- The controller is page-scoped, so switching away from `#red-team` does not pause convergence or running-state polling.

Reads and writes:

- Read: `GET /admin/adversary-sim/status`
- Write: `POST /admin/adversary-sim/control`

Notes:

- Retained simulation telemetry remains queryable after auto-off until retention expiry or explicit cleanup.
- Persisted-event evidence is intentionally bounded and lower-bound only; it proves observed monitoring facts for a run, not exact full runtime totals.
- The detailed Scrapling proof lives here on purpose:
  - `Red Team` is the primary operator surface for adversary evidence,
  - `Game Loop` only carries a compact corroborating readiness row.
- Cleanup is intentionally not part of the tab UI; use `make telemetry-clean` or `POST /admin/adversary-sim/history/cleanup` when destructive retained-history removal is required.
