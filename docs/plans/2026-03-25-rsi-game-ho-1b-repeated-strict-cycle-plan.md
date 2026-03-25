# `RSI-GAME-HO-1B` Repeated Strict-Cycle Plan

**Goal:** Prove that the strict `human_only_private` Scrapling-only loop can iterate through multiple bounded config-change cycles, with later cycles running against retained earlier config and watch windows judging both retain and rollback truthfully.

**Architecture:** Reuse the existing post-sim route path and canary/apply/watch/archive machinery. Do not create a second loop harness. Extend the test seeding surface so route-level proof can advertise more than one candidate family, then add a focused repeated-cycle proof and dedicated Make target that sits alongside the first-working-loop target.

**Tech Stack:** `src/test_support.rs`, `src/admin/api.rs`, `Makefile`, `scripts/tests/test_adversary_sim_make_targets.py`, `docs/testing.md`, planning indexes, and TODO history.

---

## Task 1: Write the failing repeated-cycle proof

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `scripts/tests/test_adversary_sim_make_targets.py`

**Work:**
1. Add a route-level test that requires:
   - a first strict-baseline post-sim cycle to retain one bounded config change,
   - a second strict-baseline post-sim cycle to run against that retained config,
   - and a later watch-window judgment to produce another truthful terminal outcome.
2. Add Makefile contract expectations for a new focused repeated-cycle target.

## Task 2: Extend the strict-cycle harness without inventing a new loop

**Files:**
- Modify: `src/test_support.rs`
- Modify: `Makefile`

**Work:**
1. Extend snapshot seeding so tests can supply multiple escalation hint candidate families.
2. Add a dedicated focused target, for example `test-rsi-game-human-only-cycles`, that proves repeated strict-baseline iteration rather than the first-working-loop milestone only.
3. Update `test-scrapling-game-loop-mainline` to include the new repeated-cycle gate, while keeping live remote proof out of that local/pre-merge bundle.

## Task 3: Close the tranche

**Files:**
- Modify: `docs/testing.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-25-rsi-game-ho-1b-repeated-strict-cycle-post-implementation-review.md`

**Verification:**

```bash
make test-rsi-game-human-only-cycles
make test-scrapling-game-loop-mainline
make test-adversary-sim-make-target-contract
git diff --check
```

## Definition Of Done

This slice is complete when:

1. the strict baseline is proven across multiple route-level cycles instead of one retained cycle,
2. a later cycle demonstrably runs against config retained from an earlier one,
3. the repeated-cycle proof includes truthful terminal judgment, not just repeated apply,
4. and the active local mainline bundle includes that repeated-cycle gate under a truthful target name.
