Date: 2026-03-29
Status: Proposed

Related context:

- [`../plans/2026-03-29-dashboard-machine-diagnostic-surface-retirement-plan.md`](../plans/2026-03-29-dashboard-machine-diagnostic-surface-retirement-plan.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../research/2026-03-28-game-loop-observer-facing-presentation-review.md`](2026-03-28-game-loop-observer-facing-presentation-review.md)
- [`../docs/dashboard-tabs/game-loop.md`](../dashboard-tabs/game-loop.md)
- [`../docs/dashboard-tabs/red-team.md`](../dashboard-tabs/red-team.md)
- [`../docs/dashboard-tabs/diagnostics.md`](../dashboard-tabs/diagnostics.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Dashboard Machine-Diagnostic Surface Retirement Review

## Questions

1. Which current dashboard sections are now redundant because their responsibilities have been replaced by cleaner observer-facing or subsystem-owned surfaces?
2. If those sections are removed, does Shuma lose telemetry collection or internal diagnostic potential, or only a cluttered presentation layer?
3. What dashboard read paths, helpers, and tests become dead once the sections are removed?

## Findings

### 1. The marked sections are presentation surfaces, not telemetry or controller producers

The sections slated for removal all read from already-materialized dashboard snapshots:

1. `Red Team` machine-diagnostic sections read `adversarySimStatus`, `eventsSnapshot`, and oversight snapshots in `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`.
2. The lower `Game Loop` machine-first sections read `benchmarkResults`, `operatorSnapshot`, `oversightHistory`, and `oversightAgentStatus` in `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`.
3. `Diagnostics` `Defense Breakdown` is a client-derived summary over already-fetched diagnostics data in `dashboard/src/lib/components/dashboard/DiagnosticsTab.svelte` and `dashboard/src/lib/components/dashboard/monitoring-view-model.js`.

Removing those surfaces does not stop collection, persistence, judging, reconcile, or the underlying machine-first contracts from existing. It removes only dashboard projection.

### 2. The current UI still carries machine-diagnostic clutter that no longer earns its keep

The surviving product framing is now clearer elsewhere:

1. `Red Team` already has a clean control surface plus `Recent Red Team Runs`.
2. `Game Loop` already has the stronger observer-facing top sections:
   - `Recent Rounds`
   - `Adversaries In This Round`
   - `Defences In This Round`
3. `Diagnostics` still owns the deeper subsystem inspection surfaces below the overview rollup.

That makes the following sections candidates for clean retirement rather than demotion:

1. `Red Team`:
   - `Lane State`
   - `Lane Diagnostics`
   - `Status Truth`
   - `Judged Episode Basis`
   - `Scrapling`
2. `Game Loop`:
   - `Round Outcome`
   - `Loop Progress`
   - `Origin Leakage And Human Cost`
   - `Loop Actionability`
   - `Pressure Context`
   - `Trust And Blockers`
3. `Diagnostics`:
   - `Defense Breakdown`

### 3. The cleanup should simplify dashboard read paths, not only markup

The current `Game Loop` refresh still fetches `benchmarkResults` specifically to feed the lower machine-facing sections. If those sections are retired, the tab can drop that read path and its state wiring while continuing to load:

1. `operatorSnapshot`
2. `oversightHistory`
3. `oversightAgentStatus`

Likewise:

1. `Red Team` no longer needs `oversightHistory` or `oversightAgentStatus` once `Judged Episode Basis` is retired.
2. The Scrapling evidence helper and panel become dead if both the Red Team `Scrapling` surface and the Game Loop `Pressure Context` corroboration surface are retired.
3. `Diagnostics` can remove `DefenseTrendBlocks` and `deriveDefenseBreakdownRows` if no remaining surface consumes them.

### 4. Several current tests are section-specific and should be retired with the sections

The dashboard suite contains both source-contract and rendered tests that exist only to prove the sections being removed. Leaving those behind would be test debt rather than proof. The cleanup should therefore remove or rewrite:

1. section-title and section-id checks for the retiring `Game Loop` sections,
2. `Red Team` checks that explicitly assert `Status Truth`, `Judged Episode Basis`, or `Scrapling`,
3. `Diagnostics` checks that explicitly assert `Defense Breakdown`,
4. and any focused `make` target or selector patterns whose only purpose is to prove deleted UI.

### 5. Removing these UI sections does not authorize deletion of backend benchmark or oversight contracts

This retirement is a dashboard-surface cleanup, not a benchmark-stack deletion. `benchmark_results_v1`, oversight history, and adversary-sim status still matter to the machine loop and later architecture. Only dashboard reads, adapters, helpers, tests, and docs that exist solely for the retired presentation should be removed in this tranche.

## Conclusions

1. The cleanup is safe from a telemetry and controller-correctness standpoint.
2. It should be executed as a full-path dashboard retirement slice:
   - UI sections removed,
   - dead dashboard read paths removed,
   - dead helpers/components removed,
   - dead tests removed or rewritten,
   - docs updated to describe only the surviving surfaces.
3. The surviving architecture should be:
   - `Red Team`: controls plus recent run history,
   - `Game Loop`: observer-facing recent rounds, adversaries, and defences,
   - `Diagnostics`: deeper subsystem inspection without the top furniture rollup.

## Acceptance boundary for the implementation tranche

This tranche is complete only when all of the following are true:

1. The listed sections are removed from the dashboard and docs.
2. `Game Loop` no longer fetches or consumes `benchmarkResults` if no kept surface needs it.
3. `Red Team` no longer receives oversight snapshots if no kept surface needs them.
4. Dead dashboard-only helpers and components tied exclusively to the removed sections are deleted.
5. Focused dashboard proof passes and no remaining test exists solely to assert a removed surface.
