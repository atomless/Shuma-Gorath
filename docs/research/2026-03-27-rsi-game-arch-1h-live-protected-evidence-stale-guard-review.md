# RSI-GAME-ARCH-1H Live Protected Evidence Stale-Guard Alignment Review

Date: 2026-03-27  
Status: proposed

## Scope

Audit the remaining controller inconsistency after `RSI-GAME-ARCH-1G`: strong live Scrapling runtime evidence can now become protected tuning evidence, but the reconcile and apply stale-input guard still hard-requires fresh `replay_promotion` metadata even when the effective protected basis is `live_scrapling_runtime`.

## Findings

1. `benchmark_results_v1` can now materialize:
   - `protected_evidence.evidence_status=protected`
   - `protected_evidence.protected_basis=live_scrapling_runtime`
   - `protected_evidence.tuning_eligible=true`
2. `oversight_reconcile::stale_evidence_reasons()` still treats `replay_promotion` as an unconditional required section alongside `live_traffic`, `adversary_sim`, and `benchmark_results`.
3. A live local read on `GET /admin/benchmark-results` now shows the protected evidence rail is ready, while `GET /admin/oversight/agent/status` still reports:
   - `outcome=refuse_stale_evidence`
   - `refusal_reasons=["replay_promotion_stale"]`
4. That means the repo currently has one inconsistent truth:
   - the benchmark and patch-policy layers accept live protected Scrapling runtime evidence,
   - but reconcile and apply still fail closed on a replay-only freshness assumption from the older architecture.

## Why This Matters

The Game Loop cannot become a real Scrapling-driven RSI path while the controller still refuses to act on the very protected evidence basis the benchmark layer now ratifies. This is not a dashboard-only or wording-only issue. It is a controller-contract mismatch that keeps the live loop stuck between diagnosis and bounded action.

## Decision

1. Keep `live_traffic`, `adversary_sim`, and `benchmark_results` as always-required stale-input sections.
2. Make `replay_promotion` conditionally required:
   - required when the effective protected basis is `replay_promoted_lineage`,
   - not required when the effective protected basis is `live_scrapling_runtime`,
   - and not treated as a universal stale-input blocker merely because the section exists in the snapshot.
3. Apply the same conditional rule to:
   - reconcile refusal,
   - canary-apply refusal,
   - and active-canary rollback evaluation.
4. Preserve fail-closed behavior for replay-lineage-dependent tuning.

## Acceptance Direction

This follow-on is complete only when:

1. stale replay metadata no longer blocks reconcile or apply when protected live Scrapling runtime evidence is already eligible and all other controller gates pass,
2. replay-promoted lineage still requires fresh replay metadata,
3. the live local Scrapling loop can progress beyond `refuse_stale_evidence` on that basis,
4. and the repo docs and TODO chain stop overstating `RSI-GAME-ARCH-1G` as the whole live blocker.
