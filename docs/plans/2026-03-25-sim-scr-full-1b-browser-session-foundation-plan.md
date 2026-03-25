# Scrapling Full-Power Browser Session Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the first truthful browser-session foundation for full-power Scrapling by making dynamic and stealth sessions part of the runtime and worker contracts, with an explicit session-strategy seam for later browser-driven challenge work.

**Architecture:** Keep the current request-native personas working as-is, but stop pretending browser sessions can be bolted on later without any structural bridge. First widen the runtime import and worker strategy contracts, then use those explicit strategy helpers as the seam for later browser-driven interaction slices.

**Tech Stack:** Repo-owned Scrapling Python runtime, `scrapling_worker.py`, focused Scrapling worker unit tests, setup-runtime bootstrap tests, Makefile-focused worker verification.

---

## Task 1: Write the failing tests for browser-session foundation

**Files:**
- Modify: `scripts/tests/test_scrapling_worker.py`

**Step 1: Write the failing tests**

Add tests that require:

1. `_import_scrapling()` to expose `DynamicSession` and `StealthySession`,
2. browser-session kwargs helpers to exist for dynamic and stealth strategies,
3. and a worker strategy helper to classify which fulfillment mode or surface mix later routes through request-native versus browser-backed execution.

**Step 2: Run test to verify it fails**

Run:

```bash
make test-adversary-sim-scrapling-worker
```

Expected:

1. the new tests fail because the worker does not yet expose those imports or helpers,
2. and the failure is about missing behavior, not a typo.

## Task 2: Implement the minimal worker and runtime foundation

**Files:**
- Modify: `scripts/supervisor/scrapling_worker.py`
- Modify: `scripts/bootstrap/scrapling_runtime.sh`

**Step 1: Write minimal implementation**

Add:

1. dynamic and stealth session imports in `_import_scrapling()`,
2. explicit dynamic-session kwargs helper,
3. explicit stealth-session kwargs helper,
4. explicit browser-session strategy helper for the current fulfillment modes or owned surface groups,
5. runtime readiness checks that now verify those browser session classes are importable in the repo-owned Scrapling runtime.

Do not yet implement the full browser interaction logic in this slice.

**Step 2: Run focused verification**

Run:

```bash
make test-adversary-sim-scrapling-worker
make test-setup-runtime-bootstrap
```

Expected:

1. the new worker tests pass,
2. setup-runtime bootstrap still passes,
3. and no existing request-native proof regresses.

## Task 3: Update docs, backlog, and audit trail

**Files:**
- Modify: `docs/testing.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`
- Create: `docs/research/2026-03-25-sim-scr-full-1b-browser-session-foundation-post-implementation-review.md`

**Step 1: Record the new foundation slice**

Document that:

1. browser sessions are now part of the runtime and worker contract,
2. the actual browser-driven challenge behavior still remains for the next follow-on,
3. and the next slice should build on the new explicit session-strategy seam rather than bypass it.

**Step 2: Commit**

Commit message:

```bash
feat: add scrapling browser session foundation
```

# Definition Of Done

This slice is complete when:

1. the worker and runtime explicitly know about dynamic and stealth sessions,
2. focused tests prove that contract,
3. request-native behavior still passes,
4. and the repo has a clear seam for the next browser-driven interaction slice.
