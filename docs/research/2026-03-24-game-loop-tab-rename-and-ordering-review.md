Date: 2026-03-24
Status: Completed

Related context:

- [`../plans/2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md`](../plans/2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md)
- [`../plans/2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-plan.md`](../plans/2026-03-24-traffic-tab-and-diagnostics-furniture-ownership-plan.md)
- [`../../dashboard/src/lib/domain/dashboard-state.js`](../../dashboard/src/lib/domain/dashboard-state.js)
- [`../../dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte)

# Review

The current dashboard tab still uses `Monitoring` as both the operator-facing label and the canonical tab id/hash, even though the tab's role has now been narrowed to proving the closed feedback loop rather than generic monitoring. The present user-facing label undersells that remit and competes semantically with the backend monitoring APIs, the `Traffic` tab, and `Diagnostics`.

The clean pre-launch answer is a canonical rename rather than a partial label swap:

1. rename the tab from `Monitoring` to `Game Loop`,
2. rename the canonical tab id and hash from `monitoring` to `game-loop`,
3. move the tab so it sits immediately after `Red Team`,
4. keep the underlying telemetry and admin API contract names as `monitoring`, because those backend reads remain broader data sources rather than the tab's operator-facing identity,
5. avoid compatibility aliases for the old hash or key, because the repo is pre-launch and should prefer one clean canonical contract over temporary drift.

# Resulting contract

The dashboard should read as:

1. `Traffic`
2. `IP Bans`
3. `Red Team`
4. `Game Loop`
5. remaining tabs

`Game Loop` remains the human-readable projection of the loop's independent judge. It continues to consume machine-first reads such as `operator_snapshot_v1`, `benchmark_results_v1`, and oversight history, but the tab itself should no longer be named or routed as `Monitoring`.
