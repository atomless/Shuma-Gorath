Date: 2026-03-27
Status: Implemented

Related context:

- [`2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md`](2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md)
- [`../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md`](../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)
- [`../../src/observability/benchmark_scrapling_surface_contract.rs`](../../src/observability/benchmark_scrapling_surface_contract.rs)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../dashboard/src/lib/components/dashboard/monitoring/ScraplingEvidencePanel.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/ScraplingEvidencePanel.svelte)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

# What landed

`SIM-SCR-FULL-1C4` is now implemented.

The key finding from execution was that current Scrapling was already attempting the contested surfaces in the worker:

1. `js_verification_execution`
2. `browser_automation_detection`
3. `pow_verify_abuse`
4. `tarpit_progress_abuse`
5. `maze_navigation`

The real fault was not missing attacker behavior. It was that the projection path flattened operator truth too aggressively once those receipts reached owned-surface coverage and the dashboard.

# What changed

1. The canonical coverage helpers now expose explicit required-surface proof states derived from the existing coverage receipts:
   - `satisfied`
   - `attempted and blocked`
   - `required but unreached`
2. The Scrapling surface-contract benchmark note now names blocking surfaces with that proof state instead of emitting only bare surface labels.
3. The Red Team `Scrapling` panel now renders the checklist and receipts using those explicit proof states instead of collapsing every required miss into one generic `blocking` bucket.
4. The Game Loop `Surface Contract Satisfaction` panel now shows blocking surfaces with the same state-specific wording when receipts are available.

# Why this matters

This tranche closes the first operator-trust gap the user called out:

- a surface that Scrapling truly tried and failed is no longer rendered as indistinguishable from a surface it never reached,
- and the board-state view is now closer to a real game with explicit local truth rather than a vague blocking list.

It does **not** yet settle dependency semantics such as whether a later surface was absent only because an earlier prerequisite blocked progression. That remains `SIM-SCR-FULL-1C5`.

# Verification

The tranche was verified through:

1. `make test-benchmark-results-contract`
2. `make test-adversary-sim-scrapling-coverage-receipts`
3. `make test-dashboard-scrapling-evidence`
4. `make test-dashboard-game-loop-accountability`
5. `make test-adversary-sim-runtime-surface`

# Follow-on

The next immediate slice remains `SIM-SCR-FULL-1C5`: make prerequisite and independence truth explicit so `required but unreached` can later split into truly unreached versus blocked-by-earlier-surface where needed.
