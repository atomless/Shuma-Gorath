# RSI-GAME-ARCH-1I Runtime-Dev Canary Seed Review

Date: 2026-03-27  
Status: proposed

## Scope

Decide how the live local strict Scrapling loop should move from diagnosis into actual bounded config mutation now that:

1. strong live Scrapling runtime evidence is protected,
2. reconcile and apply stale guards now honor the `live_scrapling_runtime` protected basis,
3. and the remaining live blocker is the seeded rollout mode `automated_apply_status=manual_only`.

## Findings

1. The current live local loop now reaches:
   - `protected_evidence.protected_basis=live_scrapling_runtime`
   - `tuning_eligibility.status=eligible`
   - `controller_contract.move_selection.decision=config_tuning_candidate`
   - `reconcile.outcome=recommend_patch`
2. The only remaining live blocker in `POST /admin/oversight/reconcile` and `GET /admin/oversight/agent/status` is:
   - `apply.stage=refused`
   - `apply.refusal_reasons=["automated_apply_manual_only"]`
3. `default_operator_objectives()` still seeds `rollout_guardrails.automated_apply_status="manual_only"` universally.
4. Existing canary-apply proof already treats `canary_only` as the required gate for the first real bounded mutation loop.

## Decision

Do not flip the global seeded default to `canary_only`.

Instead:

1. keep runtime-prod seeded defaults conservative at `manual_only`,
2. but auto-upgrade runtime-dev seeded defaults to `canary_only`,
3. only when the profile is still a seeded default rather than an operator-edited profile.

That gives local development a real self-improving Scrapling loop without silently broadening the production default or overriding explicit operator-owned decisions.

## Why This Is The Cleanest Path

1. The user has been explicit that the current development focus is proving the strict Scrapling RSI loop locally.
2. Production-safe defaults still matter, and `manual_only` is the conservative baseline there.
3. The repo already distinguishes runtime environments, so the dev-only seed upgrade can stay narrow and explicit.
4. Operator-owned profiles must remain operator-owned; seeded-default upgrade logic is the right seam for this change.

## Acceptance Direction

This follow-on is complete only when:

1. `runtime_dev` seeded defaults load as `canary_only`,
2. `runtime_prod` seeded defaults remain `manual_only`,
3. operator-edited objectives are not silently rewritten,
4. and the live local post-sim Scrapling loop progresses from `recommend_patch` into `canary_applied` rather than stopping at `automated_apply_manual_only`.
