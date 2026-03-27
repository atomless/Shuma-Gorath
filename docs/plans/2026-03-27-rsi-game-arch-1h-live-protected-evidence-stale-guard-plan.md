Date: 2026-03-27
Status: Implemented

Related context:

- [`../research/2026-03-27-rsi-game-arch-1h-live-protected-evidence-stale-guard-review.md`](../research/2026-03-27-rsi-game-arch-1h-live-protected-evidence-stale-guard-review.md)
- [`../research/2026-03-27-rsi-game-arch-1g-live-protected-evidence-post-implementation-review.md`](../research/2026-03-27-rsi-game-arch-1g-live-protected-evidence-post-implementation-review.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Finish the live protected-evidence controller alignment by making reconcile and canary-apply stale-input guards respect the effective protected-evidence basis instead of assuming replay freshness is always mandatory.

# Required contract

1. `replay_promotion` freshness must remain mandatory when the current bounded move depends on replay-promoted lineage.
2. `replay_promotion` freshness must not block reconcile, apply, or rollback evaluation when:
   - the effective protected basis is `live_scrapling_runtime`,
   - the evidence is otherwise protected and tuning-eligible,
   - and all other required inputs are still fresh.
3. The stale-input rule must stay centralized so reconcile and apply cannot drift again.
4. The controller must stay fail-closed for synthetic or advisory-only evidence.

# Execution tranche

## `RSI-GAME-ARCH-1H`

### Align stale-input requirements to the effective protected basis

Implementation guidance:

1. add failing tests first for:
   - reconcile refusing replay-stale snapshot input today,
   - reconcile allowing replay-stale snapshots when protected basis is `live_scrapling_runtime`,
   - apply and rollback logic following the same conditional rule,
2. replace the unconditional stale-section list with a helper that derives required sections from the effective protected basis,
3. keep replay freshness mandatory for `replay_promoted_lineage`,
4. update docs and closure records to distinguish:
   - protected evidence becoming eligible,
   - from the controller actually honoring that basis live.

Acceptance criteria:

1. `oversight_reconcile` no longer returns `refuse_stale_evidence` solely because `replay_promotion` is stale when protected basis is `live_scrapling_runtime`,
2. `oversight_apply` no longer refuses or rolls back solely because `replay_promotion` is stale in the same live-runtime-protected case,
3. replay-promoted lineage still fails closed on stale replay metadata,
4. focused proof exists through:
   - `make test-protected-tuning-evidence`
   - `make test-rsi-score-move-selection`
   - `make test-oversight-apply`
   - `make test-adversary-sim-runtime-surface`
5. live local evidence shows the controller moves past `refuse_stale_evidence` when the current protected basis is `live_scrapling_runtime`.

# Sequencing

1. Land this before any attempt to change rollout guardrail defaults.
2. Re-run live local Scrapling proof after the stale-guard repair.
3. Only if the next blocker is still `automated_apply_manual_only`, decide separately whether local dev policy should move to `canary_only` for the strict Scrapling RSI proof path.

# Definition Of Done

This tranche is complete when:

1. the docs explicitly record the stale-guard inconsistency,
2. tests prove the conditional replay-freshness rule,
3. reconcile and apply share the same corrected rule,
4. live local evidence moves from replay-stale refusal into the next real controller gate,
5. and the completion record distinguishes this fix from the earlier protected-evidence eligibility slice.
