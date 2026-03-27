Date: 2026-03-27
Status: Implemented

Related context:

- [`2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md`](2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md)
- [`2026-03-27-sim-scr-full-1c4-surface-exercise-proof-post-implementation-review.md`](2026-03-27-sim-scr-full-1c4-surface-exercise-proof-post-implementation-review.md)
- [`../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md`](../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)
- [`../../src/observability/benchmark_scrapling_surface_contract.rs`](../../src/observability/benchmark_scrapling_surface_contract.rs)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../dashboard/src/lib/components/dashboard/monitoring/ScraplingEvidencePanel.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/ScraplingEvidencePanel.svelte)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

# What landed

`SIM-SCR-FULL-1C5` is now implemented.

The core outcome is that the canonical Scrapling owned-surface contract now says, explicitly and operator-visibly, whether a surface is independent, co-materialized with another surface, or blocked behind a prerequisite.

For the current matrix, the main finding was not hidden dependency. It was that several surfaces the operator might reasonably assume were chained are actually independent in the present Shuma request flow and Scrapling worker behavior:

1. `pow_verify_abuse`
2. `tarpit_progress_abuse`
3. `maze_navigation`
4. `js_verification_execution`

Those now render as independent surfaces instead of leaving the operator to guess. `browser_automation_detection` is now rendered as co-materialized with `js_verification_execution`, which matches the current browser-lane proof path more honestly than implying a generic prerequisite chain.

# What changed

1. The canonical owned-surface matrix now carries dependency semantics directly on each surface row.
2. Coverage receipts now preserve:
   - `surface_state`
   - `dependency_kind`
   - `dependency_surface_ids`
   - `blocked_by_surface_ids`
3. Operator-facing proof states now distinguish:
   - `satisfied`
   - `attempted and blocked`
   - `blocked by prerequisite`
   - `required but unreached`
4. The Red Team checklist and receipts now show dependency labels when they matter, so a blocking or unreached row is no longer just a naked miss.
5. The Game Loop surface-contract blocker view now carries the same dependency-aware detail instead of reducing every problem to a bare surface name.

# Why this matters

This tranche closes the second Scrapling rigor gap the user called out:

- the operator no longer has to reverse-engineer maze, JS, PoW, tarpit, or browser-detection relationships from code,
- independent surfaces are now called independent instead of being left suspiciously implicit,
- and the repo is now able to say "this surface was absent because of an earlier prerequisite" without fabricating that explanation where it does not belong.

That matters because a real board-state loop cannot localize repairs cleanly if it is fuzzy about whether a missing later surface is a local failure or just fallout from an earlier block.

# Verification

The tranche was verified through:

1. `make test-adversary-sim-scrapling-coverage-receipts`
2. `make test-dashboard-scrapling-evidence`
3. `make test-dashboard-game-loop-accountability`

# Follow-on

The next immediate tranche is now `RSI-GAME-ARCH-1A`: split the current mixed snapshot and benchmark evidence into clean restriction-grade and recognition-evaluation rails so dependency-aware Scrapling proof no longer has to travel through the same blended path as category-recognition evaluation.
