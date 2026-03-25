# Browser Challenge Interactions Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the first real browser-backed Scrapling interactions land on owned challenge surfaces where request-native execution is no longer truthful enough.

**Architecture:** Keep the current request-native lane intact for public traversal, routing pressure, PoW abuse, and tarpit abuse. Add a browser-backed seam only for the owned DOM challenge surfaces first: `not_a_bot_submit` and `puzzle_submit_or_escalation`. Reuse the existing DOM selectors and behavioral expectations already established by the adversarial browser driver instead of inventing a parallel challenge idiom.

**Tech Stack:** `scripts/supervisor/scrapling_worker.py`, `scripts/tests/test_scrapling_worker.py`, `src/observability/scrapling_owned_surface.rs`, focused Scrapling Makefile verification, existing Playwright-backed Scrapling runtime, current challenge DOM in `src/challenge/not_a_bot` and `src/challenge/puzzle`.

---

## Task 1: Write the failing browser-behavior and contract tests

**Files:**
- Modify: `scripts/tests/test_scrapling_worker.py`
- Modify: `src/observability/scrapling_owned_surface.rs`

**Step 1: Add failing tests that require:**

1. browser-backed `bulk_scraper` or `http_agent` not-a-bot interaction to submit the real form path with trusted checkbox telemetry and surface `pass_observed`,
2. browser-backed puzzle interaction to submit through the DOM and surface `fail_observed` when routed to the maze or other failure page,
3. and the owned-surface summary to declare those surfaces `browser_or_stealth` rather than `request_native`, with truthful success contracts.

**Step 2: Run the focused tests and confirm the red phase**

Run:

```bash
make test-adversary-sim-scrapling-worker
```

Expected:

1. the worker tests fail because the current direct-request path still drives those surfaces request-natively,
2. and the owned-surface contract tests fail because the required transport and success contract are still stale.

## Task 2: Implement browser-backed challenge actions

**Files:**
- Modify: `scripts/supervisor/scrapling_worker.py`

**Step 1: Add minimal implementation**

Add:

1. a browser-request execution path for request specs that require dynamic or stealth browser sessions,
2. a `not_a_bot` browser action that mirrors the existing human-like checkbox activation path and captures semantic submit evidence,
3. a puzzle browser action that writes a wrong output via the DOM and captures maze or failure evidence,
4. receipt recording that classifies browser challenge outcomes from the actual observed interaction result rather than only the final HTTP status,
5. and routing so the current request-native session remains in use for the unaffected surfaces.

**Step 2: Update the owned-surface contract**

Adjust the owned-surface summary so:

1. `not_a_bot_submit` is `browser_or_stealth` and expects honest pass evidence where Scrapling should be able to clear that defense,
2. `puzzle_submit_or_escalation` is `browser_or_stealth` and expects honest fail or escalation evidence through the DOM,
3. while PoW and tarpit remain request-native for this tranche.

**Step 3: Run focused verification**

Run:

```bash
make test-adversary-sim-scrapling-worker
make test-adversary-sim-scrapling-owned-surface-contract
```

Expected:

1. the worker now proves real browser-backed challenge interactions,
2. and the owned-surface contract reflects the new transport and success expectations truthfully.

## Task 3: Update docs and audit trail

**Files:**
- Modify: `docs/testing.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Create: `docs/research/2026-03-25-sim-scr-full-1b2b-browser-challenge-interactions-post-implementation-review.md`

**Step 1: Record the slice**

Document that:

1. the first browser-backed Scrapling challenge interactions are now live,
2. and `SIM-SCR-FULL-1B3` is now about the remaining full-power gaps after these first browser-backed surfaces.

**Step 2: Commit**

Commit message:

```bash
feat: add scrapling browser challenge interactions
```

# Definition Of Done

This slice is complete when:

1. Scrapling uses real browser-backed execution on the first owned DOM challenge surfaces,
2. not-a-bot and puzzle receipts reflect the real browser outcome instead of only direct POST status,
3. the owned-surface contract matches the new transport and success truth,
4. and the focused worker plus owned-surface contract gates pass.
