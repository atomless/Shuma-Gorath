# SIM-SCR-FULL-1C3 Operator Truth Audit Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../docs/dashboard-tabs/game-loop.md`](../../docs/dashboard-tabs/game-loop.md)
- [`../../e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_scrapling_surface_contract.rs`](../../src/observability/benchmark_scrapling_surface_contract.rs)
- [`../../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../../docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Scope Reviewed

This closeout reviewed `SIM-SCR-FULL-1C3`: compare machine-first Scrapling receipts, Red Team truth, and Game Loop projection and answer whether the current operator-facing “Scrapling is fully blocked” reading is truthful or a projection artifact.

# Audit Finding

The bleak “Scrapling is 100% unable to pass Shuma’s defences” reading was misleading.

The underlying data planes were not saying the same thing:

1. `Category Posture Achievement` rows were scoring category-level posture alignment, such as whether `ai_scraper_bot` requests were short-circuited often enough to satisfy a `blocked` target.
2. Scrapling’s latest required defense-surface truth lived in the recent-run owned-surface coverage summary and in Red Team receipts.
3. After `SIM-SCR-FULL-1C2`, that same required-surface truth also entered benchmark pressure as the new `scrapling_surface_contract` family.

So a category row could legitimately read `Achieved 100.0%` while the latest Scrapling surface contract was still only `Partial` because a required surface such as `Maze Navigation` remained blocking.

# What Landed

1. Game Loop now labels the category section as `Category Posture Achievement` and states explicitly that it is category-level posture math, not direct Scrapling surface-contract proof.
2. The compact corroborating row now says `Latest Scrapling Surface Contract` and spells out `required surfaces` rather than a bare surface count.
3. A rendered proof case now freezes the exact confusing scenario:
   - category posture at `100.0%`,
   - latest Scrapling surface contract still `Partial`,
   - and the pressure section naming `Scrapling Surface Contract` plus the blocking locus.
4. The Game Loop operator doc now says more clearly that category posture and Scrapling surface-contract truth are distinct planes.

# Review Result

The repo now answers the audit question explicitly:

1. the old “fully blocked” reading was not a clean statement of attacker defeat,
2. it was a projection artifact caused by category posture math being easy to read as if it were the attacker-surface verdict,
3. and the current Game Loop now makes that distinction explicit enough that the page no longer silently collapses those truths together.

This also closes `SIM-SCR-FULL-1` as a mainline prerequisite:

1. full-spectrum Scrapling capability for the retained lane is implemented,
2. receipt-backed category and defense-surface truth is machine-first,
3. Red Team projects that evidence faithfully,
4. and Game Loop no longer implies more attacker maturity or defeat than the backend proves.

# Shortfalls Found

The full exploit-first redesign is still open:

1. Game Loop still does not expose the richer exploit-progress, evidence-quality, urgency, and move-selection planes planned for `RSI-SCORE-2E`.
2. Red Team remains the canonical place for full adversary-forensic detail; Game Loop still only carries a bounded corroborating projection by design.

So the next mainline is now `RSI-SCORE-2`, not more Scrapling maturity work.

# Verification

- `make test-benchmark-results-contract`
- `make test-adversary-sim-scrapling-coverage-receipts`
- `make test-dashboard-game-loop-accountability`
- `git diff --check`
