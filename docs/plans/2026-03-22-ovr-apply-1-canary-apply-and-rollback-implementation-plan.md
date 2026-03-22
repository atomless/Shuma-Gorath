# OVR-APPLY-1 Canary Apply And Rollback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the first shared-host config loop by letting a bounded oversight recommendation move into canary apply, watch-window comparison, and exact rollback over protected category-aware evidence.

**Architecture:** Extend the existing recommend-only reconcile and shared-host agent paths instead of introducing a second controller. Keep the manual admin route recommend-only, add one persisted active-canary state document per site, capture a pre-apply benchmark baseline and exact pre-canary config, and judge the watch window against that stored baseline before either retaining the canary or rolling back.

**Tech Stack:** Rust admin and observability modules, existing config validation/persistence helpers, operator and oversight decision ledgers, Makefile verification, shared-host remote proof tooling.

---

## Guardrails

1. Do not auto-apply unless `operator_objectives_v1.rollout_guardrails.automated_apply_status == canary_only`.
2. Keep the first auto-apply path shared-host-only and config-only.
3. Allow at most one active canary per site.
4. Restore the exact pre-canary config on rollback; do not attempt inverse-patch synthesis.
5. Fail closed on stale, contradictory, or non-eligible evidence.
6. Keep `/admin/oversight/reconcile` recommend-only even after this tranche lands.

## Task 1: Add the failing apply-loop contract tests

**Files:**
- Modify: `src/admin/oversight_api.rs`
- Modify: `src/admin/oversight_agent.rs`
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Step 1: Write the failing Rust tests**

Add focused tests that prove:

1. manual reconcile stays recommend-only and exposes apply eligibility without mutating config,
2. agent execution refuses apply while rollout guardrails are `manual_only`,
3. agent execution can apply one canary when rollout guardrails are `canary_only`,
4. a later cycle keeps the canary open while the watch window is still running,
5. a later cycle rolls back when candidate comparison regresses or evidence is no longer eligible,
6. a later cycle keeps the canary only when candidate comparison improves.

**Step 2: Add the focused Make target**

Add `test-oversight-apply` that runs only the new apply-state machine proof plus its immediate contract dependencies.

**Step 3: Run the new focused target to verify RED**

Run: `make test-oversight-apply`
Expected: FAIL because the apply-state machine does not exist yet.

## Task 2: Add a bounded persisted-config patch helper

**Files:**
- Modify: `src/config/mod.rs`
- Test: existing config tests in `src/config/tests.rs` if needed

**Step 1: Write the failing test**

Add a focused test for a helper that:

1. merges a bounded JSON patch into an existing persisted config,
2. validates the resulting config,
3. rejects non-object patches or invalid merged configs.

**Step 2: Run the focused failing test**

Run: `make test-oversight-apply`
Expected: FAIL on missing helper behavior.

**Step 3: Write the minimal implementation**

Add a helper suitable for internal controller patches, for example:

1. serialize config to JSON,
2. recursively merge the bounded patch,
3. deserialize back to `Config`,
4. run `validate_persisted_config`.

**Step 4: Re-run the focused target**

Run: `make test-oversight-apply`
Expected: still FAIL on missing canary state machine, but patch-helper assertions now pass.

## Task 3: Implement persisted active-canary state and watch-window judgment

**Files:**
- Create: `src/admin/oversight_apply.rs`
- Modify: `src/admin/mod.rs`
- Modify: `src/admin/oversight_decision_ledger.rs`

**Step 1: Write the failing test**

Add focused tests in the new module for:

1. saving and loading one active canary per site,
2. exact pre-canary config preservation,
3. candidate comparison against a stored `BenchmarkComparableSnapshot`,
4. fail-closed rollback on lost eligibility or unavailable comparison.

**Step 2: Run the focused target to verify RED**

Run: `make test-oversight-apply`
Expected: FAIL on missing module and state-machine behavior.

**Step 3: Write the minimal implementation**

Implement:

1. active-canary persistence,
2. apply eligibility evaluation,
3. watch-window-open evaluation,
4. improved-versus-rollback judgment,
5. rollback reason capture,
6. oversight decision-ledger apply-stage payloads.

**Step 4: Re-run the focused target**

Run: `make test-oversight-apply`
Expected: apply-state tests move closer to green, with API and agent wiring still pending.

## Task 4: Wire apply behavior through oversight API and shared-host agent execution

**Files:**
- Modify: `src/admin/oversight_api.rs`
- Modify: `src/admin/oversight_agent.rs`
- Modify: `src/admin/recent_changes_ledger.rs`
- Modify: `src/observability/decision_ledger.rs`

**Step 1: Write the failing integration-style tests**

Prove:

1. `execute_reconcile_cycle` keeps manual admin recommend-only,
2. agent execution can apply a canary and persist the active state,
3. rollback restores the exact pre-canary config,
4. recent-change and operator-decision lineage reflect canary apply and rollback,
5. oversight decision history exposes apply stages and reasons.

**Step 2: Run the focused target to verify RED**

Run: `make test-oversight-apply`
Expected: FAIL on missing wiring.

**Step 3: Write the minimal implementation**

Wire:

1. manual preview path,
2. shared-host agent auto-apply path,
3. config persistence plus cache invalidation and hot-read refresh,
4. recent-change and operator-decision entries for apply and rollback,
5. oversight execution payloads that now include apply-stage truth.

**Step 4: Re-run the focused target**

Run: `make test-oversight-apply`
Expected: PASS

## Task 5: Extend documentation and live-proof tooling

**Files:**
- Modify: `scripts/tests/live_feedback_loop_remote.py`
- Modify: `scripts/tests/test_live_feedback_loop_remote.py`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`
- Modify: `docs/deployment.md`

**Step 1: Write the failing live-proof unit test**

Extend the remote-proof contract so it expects apply-stage truth rather than only recommend-only execution.

**Step 2: Run the local live-proof verifier test**

Run: `make test-live-feedback-loop-remote-unit`
Expected: FAIL until the remote verifier understands the apply loop.

**Step 3: Update the verifier and docs**

Document:

1. the canary-only rollout guardrail,
2. the shared-host-only execution boundary,
3. the apply/watch/rollback stages,
4. the exact rollback expectation.

**Step 4: Re-run focused verification**

Run:

1. `make test-oversight-apply`
2. `make test-oversight-agent`
3. `make test-live-feedback-loop-remote-unit`
4. `git diff --check`

Expected: PASS

## Task 6: Live proof and closeout

**Files:**
- Create: `docs/research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md`
- Modify: `docs/research/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Run live proof**

Run:

1. `make test-live-feedback-loop-remote`

Expected: PASS with a receipt proving shared-host apply, watch-window evaluation, and rollback or retained-canary semantics as appropriate for the live target.

**Step 2: Write the post-implementation review**

Capture:

1. what landed,
2. acceptance checks,
3. verification,
4. shortfalls found and immediately fixed if any,
5. next-step readiness for the later blocked stages.

**Step 3: Update TODO history**

Move `OVR-APPLY-1` to completed history and keep `OVR-AGENT-2` and `OVR-CODE-1` blocked.

**Step 4: Commit**

```bash
git add src/admin/oversight_apply.rs src/admin/oversight_api.rs src/admin/oversight_agent.rs src/admin/oversight_decision_ledger.rs src/admin/recent_changes_ledger.rs src/observability/decision_ledger.rs src/config/mod.rs Makefile docs/testing.md scripts/tests/live_feedback_loop_remote.py scripts/tests/test_live_feedback_loop_remote.py docs/api.md docs/configuration.md docs/deployment.md docs/research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md docs/research/README.md todos/todo.md todos/completed-todo-history.md
git commit -m "feat: add oversight canary apply and rollback loop"
```

## Later Work Kept Blocked

### `OVR-AGENT-2`

Keep the later LLM diagnosis/config harness blocked until this plan is complete, live-proved, and projected through Monitoring and Tuning surfaces.

### `OVR-CODE-1`

Keep the later benchmark-driven code-evolution loop blocked until `OVR-AGENT-2` exists and the closed config loop has already demonstrated stable utility.
