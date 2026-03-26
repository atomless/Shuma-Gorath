Date: 2026-03-26
Status: Post-implementation review

Related context:

- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`../research/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md`](../research/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)

# Objective

Strengthen the strict-loop proof chain by showing that the current route-level game loop can do more than one judged episode, and ensure the machine-first archive records the proposal that was actually applied for each terminal canary outcome.

# What landed

1. `make test-rsi-game-mainline` now includes a repeated route-level proof where:
   - one post-sim episode applies and retains a bounded canary,
   - a later post-sim episode starts from that retained config,
   - the later episode applies a different bounded canary,
   - and the next periodic judgment rolls it back while both terminal outcomes remain visible in `oversight_history_v1` and `oversight_agent_status_v1`.
2. Shared Rust test support can now seed benchmark snapshots with explicit candidate action families, so repeated-episode proofs no longer need ad hoc snapshot duplication.
3. The episode-archive contract now records the active canary proposal and baseline snapshot when a cycle ends in `improved` or `rollback_applied`, instead of accidentally reusing whatever reconcile proposal happened to be present during watch-window close.
4. `make test-oversight-episode-archive` now also guards the rollback side of that truth contract, including the archived proposal family for a rolled-back canary.

# What this proves

1. The current mainline can do more than one judged cycle locally.
2. Later route-level episodes really do run against changed config, not only the original baseline.
3. Machine-first archive rows can now distinguish retained versus rolled-back episodes without drifting their proposal metadata away from the canary actually under judgment.

# Root-cause finding

The new repeated-cycle proof exposed a real archive-truth bug: terminal episode rows were being built from `reconcile.proposal`, which can describe a fresh recommendation for the current snapshot rather than the canary that was actually applied for the episode being judged.

That would have made repeated-cycle history misleading exactly where `RSI-GAME-HO-1B` needs trustworthy provenance. The fix was to reuse the active canary's stored proposal and baseline snapshot for terminal `improved` and `rollback_applied` rows.

# Verification

1. `make test-rsi-game-mainline`
2. `make test-oversight-episode-archive`
3. `git diff --check`

# Remaining gap

This does not close `RSI-GAME-HO-1B` or `RSI-GAME-HO-1C` by itself.

The repo still needs:

1. stronger repeated-cycle proof beyond the compressed local route path,
2. measured movement toward the strict `human_only_private` restriction target rather than only repeated plumbing,
3. and the explicit unlock condition showing retained improvement rather than merely mixed retained and rolled-back episodes.
