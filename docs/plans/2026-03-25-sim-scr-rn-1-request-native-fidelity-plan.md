# SIM-SCR-RN-1 Request-Native Fidelity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make Shuma's request-native Scrapling lane stop advertising itself through a bespoke worker `User-Agent`, while explicitly locking the upstream request-native impersonation contract into local code and tests.

**Architecture:** Keep the current request-native lane and owned-surface matrix intact. Do not widen into browser-runtime capability. Instead, add one small helper for request-native `FetcherSession` construction, remove the internal `User-Agent` override from signed request headers, and prove both the configuration contract and live emitted header shape through focused worker tests.

**Tech Stack:** Python Scrapling worker, repo-local Scrapling runtime, existing worker unit tests, Makefile `test-adversary-sim-scrapling-worker`.

---

## Guardrails

1. Do not widen this into `DynamicFetcher`, `StealthyFetcher`, or category-ownership work.
2. Do not remove sim-tag telemetry headers; only stop making the attack traffic cosmetically internal.
3. Do not add browser-runtime dependencies.
4. Keep the fix local to the Scrapling worker and its focused proof path.

## Task 1: Tighten The Failing Proof First

**Files:**
- Modify: `scripts/tests/test_scrapling_worker.py`

**Work:**
1. Add a unit test for the explicit request-native session kwargs helper.
2. Add a rendered-request test proving the live request no longer carries the internal `ShumaScraplingWorker` `User-Agent`.
3. Verify the focused worker target fails for the right reason before implementation.

**Acceptance criteria:**
1. The tests fail because the worker still overrides the request `User-Agent` and lacks an explicit session helper.

## Task 2: Implement The Minimal Worker Fix

**Files:**
- Modify: `scripts/supervisor/scrapling_worker.py`

**Work:**
1. Add a small shared helper for request-native `FetcherSession` kwargs that explicitly pins:
   - `impersonate='chrome'`
   - `stealthy_headers=True`
   - bounded timeout, retries, and redirect behavior
2. Reuse that helper from:
   - the crawler spider session
   - the bulk-scraper direct session
   - the http-agent direct session
3. Remove the bespoke worker `User-Agent` override from signed per-request headers.

**Acceptance criteria:**
1. Shuma no longer advertises the request-native attacker as `ShumaScraplingWorker/...`.
2. The request-native impersonation contract is explicit in code rather than implicit in upstream defaults.

## Task 3: Verification And Paper Trail

**Files:**
- Modify: `docs/testing.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/completed-todo-history.md`

**Work:**
1. Re-run the focused worker proof.
2. Update docs or indexes only if the proof path or tranche discoverability changed.
3. Record the completion in the audit trail.

**Acceptance criteria:**
1. `make test-adversary-sim-scrapling-worker` passes.
2. The work leaves a truthful paper trail and no lingering ambiguity about the root cause.
