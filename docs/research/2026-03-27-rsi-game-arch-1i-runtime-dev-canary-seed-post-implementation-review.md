# RSI-GAME-ARCH-1I Runtime-Dev Canary Seed Post-Implementation Review

Date: 2026-03-27  
Status: implemented

## Scope

Enable the local strict Scrapling loop to move from `recommend_patch` into actual bounded canary apply without weakening the conservative production default or overriding operator-owned profiles.

## What Landed

1. `default_operator_objectives()` remains globally conservative and still seeds `automated_apply_status=manual_only`.
2. `load_or_seed_operator_objectives()` now upgrades seeded-default profiles to `canary_only` only in `runtime_dev`.
3. Existing operator-owned profiles are left untouched, even in `runtime_dev`.
4. Focused tests now prove:
   - runtime-dev seeded defaults become `canary_only`,
   - runtime-prod or unset-runtime defaults stay `manual_only`,
   - operator-owned profiles are not silently rewritten.

## Why This Matters

This keeps the production default conservative while giving local development a real bounded self-improving Scrapling loop. It also preserves the important ownership boundary: once a profile has been explicitly set by the operator, runtime-dev convenience logic must not silently override it.

## Live Proof Outcome

The running local site already had an operator-owned `manual_admin_profile`, so the new runtime-dev seeded-default upgrade correctly did not rewrite it. To continue the live proof honestly, the current local objectives were explicitly updated through the admin objectives API to:

- `rollout_guardrails.automated_apply_status=canary_only`

After that explicit operator action:

1. `GET /admin/operator-snapshot` showed:
   - `objectives.rollout_guardrails.automated_apply_status=canary_only`
   - `benchmark_results.protected_evidence.protected_basis=live_scrapling_runtime`
   - `benchmark_results.tuning_eligibility.status=eligible`
   - `benchmark_results.controller_contract.move_selection.decision=config_tuning_candidate`
2. `make test-adversary-sim-runtime-surface` then produced:
   - `oversight.apply_stage=canary_applied`
3. `GET /admin/oversight/agent/status` then showed:
   - `latest_run.execution.reconcile.outcome=recommend_patch`
   - `latest_run.execution.apply.stage=canary_applied`
   - `latest_run.execution.apply.patch_family=not_a_bot`

That is the first truthful live local proof in this repo that the strict Scrapling loop can progress from localized breach diagnosis into an actual bounded config mutation.

## Verification

- `make test-operator-objectives-contract`
- `make test-oversight-apply`
- `make test-adversary-sim-runtime-surface`
- live local API evidence from:
  - `GET /admin/operator-snapshot`
  - `GET /admin/oversight/agent/status`

## Remaining Follow-On

1. The live loop now reaches `canary_applied`, but live retain-vs-rollback judgment still depends on the real watch window elapsing.
2. The code-level repeated judged-cycle proof already exists; the remaining live-operational follow-on is about practical local watch-window iteration, not first mutation capability.
