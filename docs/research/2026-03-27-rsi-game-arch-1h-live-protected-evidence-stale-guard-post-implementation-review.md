# RSI-GAME-ARCH-1H Live Protected Evidence Stale-Guard Post-Implementation Review

Date: 2026-03-27  
Status: implemented

## Scope

Close the remaining controller inconsistency after `RSI-GAME-ARCH-1G`: reconcile and canary-apply must respect the effective protected-evidence basis instead of always requiring fresh replay-promotion metadata.

## What Landed

1. `oversight_reconcile::stale_evidence_reasons()` now derives required stale-input sections from the active protected basis rather than hard-coding replay freshness for every case.
2. `replay_promotion` remains required when the protected basis is `replay_promoted_lineage`.
3. `replay_promotion` is no longer treated as a universal stale blocker when the protected basis is `live_scrapling_runtime`.
4. Because apply and rollback logic already reuse `stale_evidence_reasons()`, the same correction now protects:
   - recommend-only reconcile,
   - canary-apply refusal,
   - and active-canary rollback evaluation.
5. The focused protected-evidence Make target now proves both halves of the rule:
   - replay-lineage fail-closed,
   - live-runtime allowance.

## Why This Matters

Before this slice, the benchmark layer could truthfully mark live Scrapling runtime evidence as protected and tuning-eligible while the controller still refused to act solely because replay metadata had aged out. That left the Game Loop stuck between diagnosis and action even though the newer protected-evidence rail was already ratified.

After this slice, the controller now honors the same trust model as the benchmark and patch-policy layers.

## Verification

- `make test-protected-tuning-evidence`
- `make test-rsi-score-move-selection`
- `make test-oversight-apply`
- `make test-adversary-sim-runtime-surface`
- live local evidence:
  - `POST /admin/oversight/reconcile` now returns `reconcile.outcome=recommend_patch`
  - `apply.refusal_reasons=["automated_apply_manual_only"]`
  - and no longer fails with `replay_promotion_stale` when the protected basis is `live_scrapling_runtime`

## Remaining Follow-On

1. The next live blocker is now explicit and narrower:
   - seeded or operator-owned rollout mode still decides whether the controller may mutate config.
2. `RSI-GAME-ARCH-1I` addresses the runtime-dev seeded-default part of that problem.
