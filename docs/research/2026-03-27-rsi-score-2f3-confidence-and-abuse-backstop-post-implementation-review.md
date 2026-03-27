Date: 2026-03-27
Status: Implemented

Related plan:

- [`../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md`](../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)

# Objective

Close `RSI-SCORE-2F3` by making the restriction scorer and Game Loop say one explicit thing:

1. restriction scoring is no longer category-first,
2. Shuma confidence must weigh restriction urgency,
3. low-confidence but high-cost hostile pressure must still stay urgent through an abuse backstop,
4. and simulator-known category labels still must not leak into runtime or tuning.

# What Landed

1. [`../../src/observability/benchmark_urgency.rs`](../../src/observability/benchmark_urgency.rs) now materializes:
   1. `restriction_confidence_status`,
   2. `abuse_backstop_status`,
   3. and urgency notes that explain those states alongside homeostasis-break reasons.
2. [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs) now carries those urgency fields in the machine-first benchmark payload.
3. [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js) now adapts those new machine-first fields without inventing local dashboard-only semantics.
4. [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte) now projects:
   1. `Restriction Confidence`,
   2. `Abuse Backstop`,
   3. and a richer `Loop Actionability` urgency split that no longer hides those two planes behind a single urgency summary.
5. [`../../Makefile`](../../Makefile) now puts the new urgency tests on a named proof path through `make test-rsi-score-urgency-and-homeostasis` so the new behavior is executed, not merely compiled.

# Why This Matters

Before this tranche, the repo had already removed category posture from the primary restriction spine, but the urgency model still did not say clearly:

1. how much the loop trusted its own hostile-confidence state,
2. or how low-confidence but still-expensive traffic stayed urgent enough to matter.

That left the restriction scorer too easy to read as:

1. exploit progress plus human friction only,
2. with suspicious-origin cost still present,
3. but without an explicit backstop narrative that matched the new architecture doctrine.

This tranche closes that gap.

# Proof

The closure gate for `RSI-SCORE-2F3` is now satisfied through:

1. `make test-traffic-classification-contract`
2. `make test-benchmark-results-contract`
3. `make test-rsi-score-exploit-progress`
4. `make test-rsi-score-evidence-quality`
5. `make test-rsi-score-urgency-and-homeostasis`
6. `make test-rsi-score-move-selection`
7. `make dashboard-build`
8. `make test-dashboard-game-loop-accountability`
9. `git diff --check`

# Remaining Follow-On

`RSI-SCORE-2F3` is closed, but the next Game Loop gaps are still architectural and presentation-grade:

1. `RSI-GAME-ARCH-1C`
   Reconcile must stop depending on one monolithic escalation-hint oracle.
2. `RSI-GAME-BOARD-1F`
   `Loop Actionability` still needs typed blocker groups and exact next-fix surfaces.
3. `RSI-GAME-ARCH-1D` and `RSI-GAME-BOARD-1G`
   breach-locus and blocker contracts still need stricter missing-data honesty and end-to-end modular cleanup.
