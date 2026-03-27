Date: 2026-03-27
Status: Implemented

Related plan:

- [`../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md`](../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md)

Related code:

- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js)
- [`../../e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js)

# Objective

Close `RSI-GAME-BOARD-1F` by turning `Loop Actionability` from a flat blocker dump into a grouped repair view under the newer three-rail Game Loop model.

# What Landed

1. the dashboard benchmark adapter now carries `controller_contract` all the way into the Game Loop view model,
2. `Loop Actionability` now separates:
   1. restriction quest state,
   2. recognition quest state,
   3. root-cause blocker groups,
   4. controller-outcome groups,
   5. and next fix surfaces,
3. the older flat `Decision blockers:` line is gone from the primary operator view,
4. and the grouped output now names exact blocked surfaces such as `Browser CDP Automation Detection` and `JS Verification Execution` instead of reducing everything to an opaque blocker string.

# Why This Was Necessary

Before this tranche, the Game Loop still forced operators to read one blended line like:

1. degraded category receipts present,
2. insufficient category evidence,
3. surface blocking,
4. and controller guardrails

as if they were equivalent peers.

That was not good enough for a real RSI loop. The operator needs to know:

1. what is a root-cause truth gap,
2. what is merely a downstream controller consequence,
3. and which surfaces are the next exact places to fix.

# Acceptance Criteria Check

## 1. The machine payload preserves typed blocker groups or equivalent structured fields

Passed.

- `controller_contract` is now adapted explicitly in the dashboard client.
- blocker groups remain typed through the dashboard path rather than being flattened back to strings.

## 2. The dashboard projects those groups distinctly enough to show what must be fixed first

Passed.

- `Loop Actionability` now has separate `Root Cause Blockers` and `Controller Outcome` lists.
- the section also names `Next Fix Surfaces` rather than burying them inside one sentence.

## 3. The current flat blocker line is removed or demoted to a secondary raw-detail view

Passed.

- the old `Decision blockers:` paragraph is removed from the main operator surface.

# Verification

- `make dashboard-build`
- `make test-benchmark-results-contract`
- `make test-rsi-score-evidence-quality`
- `make test-dashboard-game-loop-accountability`
- `git diff --check`

# Follow-On Work

The remaining actionability and board-state work is now narrower:

1. `RSI-GAME-ARCH-1D`
   Keep missing vs derived vs materialized board-state data explicit end to end.
2. `RSI-GAME-BOARD-1G`
   Make `Named Breach Loci` fully honest about missing counts and host-cost materialization so the operator can trust the board-state panel with no hidden coercions.
