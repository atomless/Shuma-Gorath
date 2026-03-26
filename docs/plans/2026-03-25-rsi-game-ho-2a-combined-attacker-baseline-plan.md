# RSI-GAME-HO-2A Combined-Attacker Baseline Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the first truthful strict `human_only_private` Game Loop proof that includes both Scrapling and live `bot_red_team` attacker pressure.

**Architecture:** Reuse the existing route-level oversight proof, recent-run lineage, and strict-baseline archive surfaces rather than inventing a second mixed-attacker harness. Extend the current strict-loop proof so both attacker lanes are present in the same machine-first story, then expose that through one focused Make target.

**Tech Stack:** Rust route-level tests in `src/admin/api.rs`, shared test helpers in `src/test_support.rs`, Makefile proof targets, Python make-target contract tests, and docs/TODO closeout.

---

## Task 1: Add the failing mixed-attacker strict-baseline proof

**Files:**
- Modify: `src/admin/api.rs`

**Step 1: Write the failing test**

Add one focused route-level test that proves:

1. the strict `human_only_private` stance is still active,
2. both Scrapling and `bot_red_team` attacker evidence are present in the same loop story,
3. and the post-sim oversight route can still progress under that combined pressure.

**Step 2: Run the focused proof to verify it fails**

Run:

```bash
make test-rsi-game-human-only-mixed-baseline
```

Expected: the new proof target fails because the route-level mixed-attacker contract is not implemented yet.

## Task 2: Implement the minimal mixed-attacker route proof support

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `src/test_support.rs` if shared helper setup is needed

**Step 1: Add the smallest shared helper support needed**

If the new proof needs seeded recent-run or attacker-lane evidence, add only the minimum reusable helper surface required for that route-level test.

**Step 2: Implement the route-level proof**

Extend the existing strict-loop test path so:

1. the proof remains under `human_only_private`,
2. Scrapling and `bot_red_team` evidence both exist in the recent strict-baseline story,
3. and the loop still reaches the expected bounded post-sim stage without inventing a parallel mixed-attacker controller path.

**Step 3: Run the focused proof to verify it passes**

Run:

```bash
make test-rsi-game-human-only-mixed-baseline
```

Expected: PASS.

## Task 3: Add the focused Make target and contract proof

**Files:**
- Modify: `Makefile`
- Modify: `scripts/tests/test_adversary_sim_make_targets.py`
- Modify: `docs/testing.md`

**Step 1: Add a focused Make target**

Add:

```make
test-rsi-game-human-only-mixed-baseline
```

It must run only the new combined-attacker strict-baseline proof, not the whole umbrella.

**Step 2: Add the make-target contract test**

Update the make-target contract suite so the new target name and contents are fail-closed and truthful.

**Step 3: Update testing docs**

Document exactly what the new target proves and what it does not yet prove.

**Step 4: Run the focused verification bundle**

Run:

```bash
make test-rsi-game-human-only-mixed-baseline
make test-adversary-sim-make-target-contract
```

Expected: PASS.

## Task 4: Close the tranche docs and backlog

**Files:**
- Create: `docs/research/2026-03-25-rsi-game-ho-2a-combined-attacker-baseline-post-implementation-review.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Write the post-implementation review**

Record:

1. what landed,
2. what the new mixed-attacker proof actually covers,
3. and what remains for `RSI-GAME-HO-2B` and `RSI-GAME-HO-2C`.

**Step 2: Update indexes and backlog**

Make `RSI-GAME-HO-2A` the completed slice and move the next sub-slices forward truthfully.

**Step 3: Run final docs hygiene**

Run:

```bash
git diff --check
```

Expected: no diff hygiene failures.
