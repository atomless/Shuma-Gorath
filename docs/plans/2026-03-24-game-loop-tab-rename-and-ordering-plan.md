Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-game-loop-tab-rename-and-ordering-review.md`](../research/2026-03-24-game-loop-tab-rename-and-ordering-review.md)
- [`2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md`](2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md)
- [`2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)
- [`../../dashboard/src/lib/domain/dashboard-state.js`](../../dashboard/src/lib/domain/dashboard-state.js)
- [`../../dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte)

# Goal

Canonically rename the dashboard `Monitoring` tab to `Game Loop` and move it to sit immediately after `Red Team`, while leaving the backend monitoring read contracts untouched.

# Required behavior

1. The canonical dashboard tab id becomes `game-loop`.
2. The canonical hash route becomes `#game-loop`.
3. The visible tab label becomes `Game Loop`.
4. The tab appears immediately after `Red Team` in all shared registries and rendered tab order.
5. The tab component, route wiring, loading copy, smoke coverage, and docs all use the new `Game Loop` naming.
6. The underlying dashboard snapshot keys and admin API reads may continue to use `monitoring` where they refer to telemetry data contracts rather than the tab identity.
7. Because the repo is pre-launch, do not keep a fallback alias for the old hash or a second tab key.

# Files to update

## Dashboard runtime and UI

- `dashboard/src/lib/domain/dashboard-state.js`
- `dashboard/src/lib/state/dashboard-store.js`
- `dashboard/src/lib/runtime/dashboard-native-runtime.js`
- `dashboard/src/lib/runtime/dashboard-route-controller.js`
- `dashboard/src/routes/+page.svelte`
- `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`

## Tests and verification

- `e2e/dashboard.modules.unit.test.js`
- `e2e/dashboard.smoke.spec.js`
- `Makefile`

## Docs and backlog

- `docs/dashboard.md`
- `docs/dashboard-tabs/README.md`
- `docs/dashboard-tabs/game-loop.md`
- `docs/plans/2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md`
- `docs/plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`
- `todos/todo.md`
- `todos/completed-todo-history.md`

# Verification

Use focused Make targets only:

1. `make test-dashboard-tab-information-architecture`
2. `make test-dashboard-game-loop-accountability`
3. `git diff --check`

Add or update assertions so they prove:

1. the canonical tab registry now includes `game-loop` after `red-team`,
2. the tab route and panel ids use `game-loop`,
3. the rendered label says `Game Loop`,
4. and the accountability tab still renders the expected loop sections after the rename.
