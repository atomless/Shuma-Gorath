# `RSI-GAME-HO-1A` Human-Only Private Loop Plan

**Goal:** Make the already-landed first-working machine-first loop prove the strict `human_only_private` baseline explicitly, including denied verified non-human traffic.

**Architecture:** Reuse the existing first-working-loop route proof and shared-host verifier rather than inventing a new loop path. Seed strict objectives through test support, then require the same preset and verified-identity mode from `operator_snapshot_v1` in both the Rust route-level proof and the live feedback-loop verifier report.

**Tech Stack:** `src/observability/operator_snapshot_objectives.rs`, `src/test_support.rs`, `src/admin/api.rs`, `scripts/tests/live_feedback_loop_remote.py`, `scripts/tests/test_live_feedback_loop_remote.py`, `docs/testing.md`, planning indexes, and TODO history.

---

## Task 1: Tighten the failing proof first

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `scripts/tests/test_live_feedback_loop_remote.py`

**Work:**
1. Require the first-working-loop Rust proof to see:
   - `non_human_stance_presets.active_preset_id == "human_only_private"`
   - `effective_non_human_policy.active_preset_id == "human_only_private"`
   - `effective_non_human_policy.verified_identity_mode == "verified_identities_denied"`
2. Require the local shared-host verifier tests to record the same fields in the operator-snapshot receipt.

## Task 2: Wire the strict baseline through the loop proof path

**Files:**
- Modify: `src/test_support.rs`
- Modify: `scripts/tests/live_feedback_loop_remote.py`

**Work:**
1. Seed the loop proof with strict `human_only_private` objectives rather than the balanced default.
2. Make the live feedback-loop verifier fail closed if `/admin/operator-snapshot` does not expose:
   - strict `human_only_private` as the active preset,
   - strict `human_only_private` as the effective policy preset,
   - `verified_identities_denied` as the resolved verified-identity mode.

## Task 3: Close the tranche

**Files:**
- Modify: `docs/testing.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-25-rsi-game-ho-1a-human-only-private-loop-post-implementation-review.md`

**Verification:**

```bash
make test-rsi-game-mainline
make test-live-feedback-loop-remote-unit
make test-live-feedback-loop-remote-contracts
git diff --check
```

## Definition Of Done

This slice is complete when:

1. the first-working-loop route proof now runs under `human_only_private`,
2. verified non-human traffic remains denied in the resolved effective policy under that strict baseline,
3. the shared-host verifier also proves the same strict baseline rather than only checking operator-snapshot schema presence,
4. and `RSI-GAME-HO-1B` is the next active strict-baseline tranche.
