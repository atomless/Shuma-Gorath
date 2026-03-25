# Fingerprint State Retention Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enforce deterministic cleanup and expiry for stale fingerprint state and flow keys so persisted fingerprint lifecycle matches Shuma's configured TTL and flow-window controls.

**Architecture:** Keep the fix inside `src/signals/fingerprint.rs`. Add one bounded, cadence-gated cleanup path that scans only fingerprint prefixes and evicts stale `fp:state:*`, `fp:flow:*`, and `fp:flow:last_bucket:*` entries. Do not invent a second retention worker or widen this into broader telemetry retention work.

**Tech Stack:** Rust fingerprint signal module, crate-local unit tests, focused Make target, testing and research docs.

---

## Guardrails

1. Do not add a full keyspace scan on every request.
2. Do not widen this into event-log or monitoring retention work.
3. Do not weaken the existing fingerprint scoring semantics while adding cleanup.
4. Keep the lifecycle contract aligned to the already-configured TTL and flow-window values.

## Task 1: Add failing retention-cleanup tests first

**Files:**
- Modify: `src/signals/fingerprint.rs`
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add focused failing tests that prove:
   - stale `fp:state:*` keys are removed when older than `fingerprint_state_ttl_seconds`,
   - stale `fp:flow:*` and `fp:flow:last_bucket:*` keys are removed when no longer in the current flow window,
   - repeated requests inside the cleanup cadence do not rescan the full store every time.
2. Add a narrow Make target for the fingerprint retention cleanup proof.
3. Run the new target and confirm it fails for the right reason before implementation.

**Acceptance criteria:**
1. The new tests fail against the current code.
2. The expected lifecycle behavior is explicit and machine-checked.

## Task 2: Implement bounded cleanup in the fingerprint module

**Files:**
- Modify: `src/signals/fingerprint.rs`

**Work:**
1. Add a small cleanup-state key for cadence gating.
2. Implement bounded prefix cleanup for:
   - `fp:state:*`
   - `fp:flow:*`
   - `fp:flow:last_bucket:*`
3. Keep parsing robust for identities that may themselves contain `:`.
4. Invoke cleanup from the existing fingerprint request path without changing the user-visible signal contract.

**Acceptance criteria:**
1. Stale fingerprint keys are evicted deterministically on the bounded cleanup cadence.
2. Active current-window keys are preserved.
3. The request path remains bounded rather than scanning on every call.

## Task 3: Close the tranche and sync docs/backlog

**Files:**
- Modify: `docs/testing.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-24-sec-gdpr-2-fingerprint-retention-cleanup-post-implementation-review.md`

**Work:**
1. Move `SEC-GDPR-2` to completed history.
2. Add the post-implementation review.
3. Update the research and plan indexes so the new lifecycle contract is discoverable.

**Acceptance criteria:**
1. The backlog and paper trail show `SEC-GDPR-2` as delivered.
2. The new cleanup contract is discoverable from testing and research indexes.

## Verification

1. `make test-fingerprint-retention-cleanup`
2. `git diff --check`

## Exit Criteria

This tranche is complete when:

1. stale fingerprint state and flow keys are cleaned up according to configured TTL/window,
2. the cleanup path is cadence-gated and test-covered,
3. the narrow Make proof exists,
4. and the docs or backlog reflect the delivered lifecycle contract.
