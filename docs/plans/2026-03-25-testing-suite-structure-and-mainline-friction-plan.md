# TEST-MAINLINE-1 Active Mainline Verification Ergonomics Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the current attacker-faithful Scrapling -> first-working-game-loop path cheap, truthful, and obvious to verify during development.

**Architecture:** Add one truthful aggregate verification path for the currently active mainline, refresh `Makefile` help and `docs/testing.md` so contributors can find and trust that path quickly, and fix any remaining active-path churn that undermines day-to-day iteration. Keep the broader testing-rationalization backlog separate so this stays a small enabling tranche rather than a whole test-system redesign.

**Tech Stack:** `Makefile`, `docs/testing.md`, existing focused Scrapling and game-loop targets, TODO backlog, and related planning docs.

---

## Why This Tranche Exists

The repo already has meaningful proof for the active mainline, but it is spread across several focused commands. That is good for proof composition, but not ideal for fast repeated development. The next leverage point is to make the active path easy to run and easy to trust without diluting the stronger broader suite structure.

## Task 1: Freeze The Active Mainline Verification Bundle

**Files:**
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add one truthful aggregate target for the current active mainline, for example `make test-scrapling-game-loop-mainline`.
2. Keep that target local and pre-merge in scope. It must not silently imply live remote proof.
3. Compose it only from behavior-meaningful focused targets for the current mainline, expected to include:
   - `test-adversary-sim-scrapling-owned-surface-contract`
   - `test-adversary-sim-scrapling-malicious-request-native`
   - `test-adversary-sim-scrapling-coverage-receipts`
   - `test-rsi-game-mainline`
4. Do not hide selector-only, wrapper-only, or source-shape-only proof inside this new bundle unless that proof is truly part of the active behavior contract.

**Acceptance criteria:**
1. One plainly named command answers "what should I run for the current active mainline?"
2. The target name does not overclaim live, remote, or broader coverage it does not actually include.
3. The bundle is built from already-truthful focused proof lanes rather than a new opaque mega-target.

## Task 2: Refresh Help Text And Testing Guide Around The Active Path

**Files:**
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Refresh stale or misleading `Makefile` help text that affects test-target trust.
2. Add one short "current active mainline" subsection near the top of [`docs/testing.md`](../testing.md).
3. State explicitly how the new aggregate differs from:
   - `make test`
   - the deeper adversarial coverage gates
   - live/shared-host operational proof
4. Preserve the canonical tier split:
   - local pre-merge proof
   - rendered UI proof
   - live/shared-host operational proof

**Acceptance criteria:**
1. A contributor can find the active mainline command in under a minute.
2. The docs no longer force readers to infer the correct command sequence from scattered focused-target notes.
3. The target-scope wording remains truthful and specific.

## Task 3: Remove Remaining Active-Path Churn If It Still Exists

**Files:**
- Modify: `Makefile`
- Modify: active-path helper scripts or tests only if required
- Modify: `docs/testing.md`

**Work:**
1. Confirm whether the new active-mainline aggregate or its component targets still rewrite tracked receipts or artifacts.
2. If yes, move those outputs to untracked/runtime locations or make them reproducible without worktree churn.
3. Keep this scoped to the active-mainline path. Do not widen it into the full broader `TEST-HYGIENE-2` tranche unless the same fix naturally closes both.

**Acceptance criteria:**
1. Re-running the active-mainline command does not leave routine tracked artifact noise behind.
2. The result is documented truthfully if any broader full-suite churn still remains outside this narrower path.

## Task 4: Reorder The Backlog So This Comes Before Deferred Dashboard Cleanup

**Files:**
- Modify: `todos/todo.md`
- Modify: `docs/plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`
- Modify: `docs/plans/2026-03-23-testing-surface-rationalization-plan.md`
- Modify: `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`

**Work:**
1. Add `TEST-MAINLINE-1` as the immediate next active tranche.
2. Keep `DIAG-CLEANUP-1` and `MON-OVERHAUL-1C` deferred behind it.
3. Keep the broader testing-rationalization items (`TEST-HYGIENE-6C`, `TEST-HYGIENE-3`, `TEST-HYGIENE-4`, `TEST-HYGIENE-5`, `TEST-HYGIENE-2`) queued as follow-on hygiene, not as the immediate blocker.

**Acceptance criteria:**
1. The active queue clearly shows one narrow testing tranche next.
2. The backlog no longer implies that deferred dashboard cleanup should happen before active-mainline test ergonomics.

## Recommended Implementation Order

1. `TEST-MAINLINE-1`
   - add the active-mainline aggregate target,
   - refresh help/docs,
   - remove any remaining active-path churn if present.
2. return to product work on the current mainline once that faster truthful proof path exists.
3. keep broader follow-on testing hygiene in this order:
   - `TEST-HYGIENE-6C`
   - `TEST-HYGIENE-3`
   - `TEST-HYGIENE-4`
   - `TEST-HYGIENE-5`
   - `TEST-HYGIENE-2`

## Verification

This planning tranche is docs-only.

Use:

```bash
git diff --check
```
