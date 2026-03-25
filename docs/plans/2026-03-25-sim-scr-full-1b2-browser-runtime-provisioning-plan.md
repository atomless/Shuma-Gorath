# Scrapling Browser Runtime Provisioning Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Provision the Playwright browser runtime inside the repo-owned Scrapling environment and make readiness fail closed until browser-backed Scrapling sessions are actually executable.

**Architecture:** Keep the newly landed browser-session foundation in place, but close the next runtime blocker before any browser-driven challenge logic lands. Extend the existing Scrapling runtime bootstrap rather than inventing a parallel installer path, and make readiness check the browser executable path rather than only import availability.

**Tech Stack:** `scripts/bootstrap/scrapling_runtime.sh`, `scripts/bootstrap/setup-runtime.sh`, setup-runtime shell tests, focused Scrapling runtime verification through Makefile.

---

## Task 1: Write the failing provisioning tests

**Files:**
- Modify: `scripts/tests/test_setup_runtime_spin_install.py`

**Step 1: Write the failing tests**

Add tests that require:

1. the runtime installer to invoke `python -m playwright install chromium`,
2. and the runtime readiness script to check the Playwright Chromium executable path rather than only import availability.

**Step 2: Run test to verify it fails**

Run:

```bash
make test-setup-runtime-bootstrap
```

Expected:

1. the new test fails because the runtime bootstrap does not yet provision browser binaries,
2. and the readiness script does not yet check executable-path existence.

## Task 2: Implement minimal runtime provisioning

**Files:**
- Modify: `scripts/bootstrap/scrapling_runtime.sh`

**Step 1: Write minimal implementation**

Add:

1. a browser package selection constant for the current Playwright browser install target,
2. a helper that runs `python -m playwright install <browser>`,
3. an installer path that provisions that browser after the Python package install,
4. and a readiness check that uses `playwright.sync_api` to assert the browser executable path exists.

**Step 2: Run focused verification**

Run:

```bash
make test-setup-runtime-bootstrap
make test-adversary-sim-scrapling-worker
```

Expected:

1. setup-runtime bootstrap passes,
2. the Scrapling worker gate still passes,
3. and the browser-session foundation remains intact.

## Task 3: Update docs and audit trail

**Files:**
- Modify: `docs/testing.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Create: `docs/research/2026-03-25-sim-scr-full-1b2-browser-runtime-provisioning-post-implementation-review.md`

**Step 1: Record the slice**

Document that:

1. browser-backed Scrapling sessions are now executable prerequisites, not only importable classes,
2. and the next follow-on is the first real browser-driven challenge interaction slice.

**Step 2: Commit**

Commit message:

```bash
feat: provision scrapling browser runtime
```

# Definition Of Done

This slice is complete when:

1. the repo-owned Scrapling runtime provisions the required Playwright browser,
2. readiness fails closed if the browser executable is missing,
3. the focused runtime-bootstrap and worker gates pass,
4. and the next `SIM-SCR-FULL-1B2B` slice can assume browser sessions are actually runnable.
