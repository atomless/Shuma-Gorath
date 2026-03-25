# Baseline Repair After MZ-T4 Full-Suite Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restore a truthful green baseline after `MZ-T4` by fixing the non-maze failures exposed by the canonical `make test` path before resuming the next scheduled Scrapling and game-loop tranches.

**Architecture:** Keep this slice narrowly focused on baseline repair. Do not widen it into new feature work. Align tests with already-landed contracts where expectations drifted, make one explicit operator-snapshot hot-read budget decision, and only debug deeper into Scrapling owned-surface behavior if the broad suite still reproduces that failure after the other repairs land.

**Tech Stack:** Rust unit tests, Makefile verification targets, machine-first operator snapshot and hot-read contracts, verified-identity provider seam, benchmark escalation helper, adversary-sim Scrapling coverage receipts.

## Guardrails

1. Do not fold new Scrapling feature work into this repair tranche.
2. Do not weaken the operator snapshot content just to make the size assertion pass unless the removed data is clearly non-canonical for the machine-first contract.
3. Do not update expectations blindly; each expectation change must be justified by already-landed code and plans.
4. Do not assume the full-suite-only Scrapling failure is gone until `make test` passes.

## Task 1: Align stale test expectations with landed contracts

**Files:**

- Modify: `src/observability/benchmark_results.rs`
- Modify: `src/providers/external.rs`

**Work:**

1. Update the benchmark escalation test to match the current move-selection contract for `beneficial_non_human_posture`.
2. Update the verified-identity provider seam test to match the post-default-enable provider-path-off semantics.

**Acceptance criteria:**

1. `make test-benchmark-results-contract` passes.
2. `make test-verified-identity-provider` passes.
3. The updated assertions match the current implementation and planning intent, not a guessed workaround.

## Task 2: Rebaseline or trim the operator-snapshot hot-read budget truthfully

**Files:**

- Modify: `src/observability/hot_read_documents.rs`
- Modify: `src/observability/hot_read_projection.rs`
- Modify: `docs/testing.md`
- Modify: `docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md` if the contract budget itself changes

**Work:**

1. Decide whether the operator snapshot should stay at its current machine-first breadth in hot-read form.
2. If yes, rebaseline the hot-read size contract to a realistic bound that still preserves the hierarchy against the bootstrap document.
3. If no, trim only clearly non-canonical fields and prove the machine-first contract remains intact.
4. Keep the decision explicit in docs.

**Acceptance criteria:**

1. `make test-telemetry-hot-read-projection` passes.
2. The operator snapshot size budget is explicitly justified.
3. The hot-read contract tests remain truthful.

## Task 3: Re-run the broad verification path and close the Scrapling uncertainty

**Files:**

- Modify only if still needed after repro.

**Work:**

1. Re-run `make test-unit`.
2. Re-run `make test`.
3. If the Scrapling owned-surface failure reappears only in the broad run, continue with focused order/shared-state debugging and fix it in the same tranche.
4. If it does not reappear, record that the earlier failure was cleared by the baseline repair and full broad-path proof now holds.

**Acceptance criteria:**

1. `make test-unit` passes.
2. `make test` passes.
3. `.spin/last-full-test-pass.json` is refreshed only after a successful full-suite pass.

## Task 4: Keep the paper trail current

**Files:**

- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-24-baseline-repair-after-mz-t4-full-suite-post-implementation-review.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`

**Work:**

1. Add an execution-ready baseline-repair TODO before further mainline work.
2. Move it to completed history immediately on delivery.
3. Record the post-implementation review and verification evidence.
4. Index the new review and plan docs.

**Acceptance criteria:**

1. The repair tranche is auditable from research through completion.
2. The active backlog no longer hides the baseline issue.
