Date: 2026-03-27
Status: Implemented

Related context:

- [`2026-03-27-game-loop-architecture-alignment-gap-review.md`](2026-03-27-game-loop-architecture-alignment-gap-review.md)
- [`2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md`](2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md`](../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md)

# Objective

Close the remaining missing-data honesty gap in board-state breach loci so the Game Loop stops fabricating certainty and starts projecting measured, derived, and not-materialized truth end to end.

# What landed

1. `BenchmarkExploitLocus` now carries explicit materialization state for:
   - `attempt_count`
   - `cost_channel_ids`
   - `repair_family_candidates`
2. exploit-progress and evidence-quality benchmark builders now mark:
   - measured attempt counts as `measured`
   - mapped host-cost and repair hints as `derived`
   - absent board-state hints as `not_materialized`
3. dashboard API adaptation no longer coerces missing exploit-locus fields into `0`.
4. the Game Loop breach-locus rendering now says:
   - `attempt count not materialized`
   - `Host cost not materialized`
   - `Repair candidates not materialized`
   instead of pretending those fields were observed as zero or merely “not available”.

# Why it mattered

Before this slice, the same missing breach-locus field could appear as:

1. missing in backend materialization,
2. `0` in the adapter,
3. and then operator-visible as false certainty like `0 attempts`.

That broke the board-state doctrine because the loop could no longer distinguish:

1. a measured failed attack,
2. a derived repair hint,
3. and a field that had never been materialized at all.

# Acceptance review

## `RSI-GAME-ARCH-1D`

Accepted.

- breach loci now carry explicit materialization truth rather than only raw values,
- API adapters preserve that truth instead of coercing missing values to zero-like defaults,
- and the blocker grouping from `RSI-GAME-BOARD-1F` remains intact as the typed downstream projection.

## `RSI-GAME-BOARD-1G`

Accepted.

- named breach loci stay surface-exact through the existing `locus_label`,
- sample request proof remains visible,
- and missing attempt, host-cost, and repair data now render honestly enough to guide further board-state repair instead of hiding the gap behind fabricated values.

# Evidence

- `make test-benchmark-results-contract`
- `make test-rsi-score-exploit-progress`
- `make test-rsi-score-evidence-quality`
- `make dashboard-build`
- `make test-dashboard-game-loop-accountability`
- `git diff --check`

# Remaining follow-on

1. `RSI-GAME-ARCH-1E` remains open:
   - retire or demote any still-live legacy category-first Game Loop surfaces only after full replacement proof.
2. the broader strict-loop mainline remains open:
   - continue toward a Game Loop that produces retained config improvements from Scrapling pressure, not just clearer diagnosis surfaces.
