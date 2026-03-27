Date: 2026-03-27
Status: Implemented

Related context:

- [`../research/2026-03-27-rsi-game-arch-1i-runtime-dev-canary-seed-review.md`](../research/2026-03-27-rsi-game-arch-1i-runtime-dev-canary-seed-review.md)
- [`../research/2026-03-27-rsi-game-arch-1h-live-protected-evidence-stale-guard-review.md`](../research/2026-03-27-rsi-game-arch-1h-live-protected-evidence-stale-guard-review.md)
- [`../plans/2026-03-22-ovr-apply-1-canary-apply-and-rollback-implementation-plan.md`](../plans/2026-03-22-ovr-apply-1-canary-apply-and-rollback-implementation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Make the live local strict Scrapling loop able to mutate config by default in runtime-dev without weakening the conservative production seed or overriding operator-edited objective profiles.

# Required contract

1. `default_operator_objectives()` may remain globally conservative.
2. `load_or_seed_operator_objectives()` must upgrade only seeded-default profiles in `runtime_dev` to `canary_only`.
3. `runtime_prod` must keep seeded defaults at `manual_only`.
4. Profiles saved by the operator must not be auto-rewritten.
5. The live local post-sim Scrapling loop must progress from `recommend_patch` to `canary_applied`.

# Execution tranche

## `RSI-GAME-ARCH-1I`

### Runtime-dev seeded rollout mode for the strict Scrapling proof loop

Implementation guidance:

1. add failing tests first for:
   - runtime-dev seeded default loading as `canary_only`,
   - runtime-prod seeded default staying `manual_only`,
   - operator-edited profiles staying untouched in runtime-dev,
2. keep the change in the seeded-objectives store seam rather than spreading environment checks through controller code,
3. update focused testing docs if the live local strict loop now assumes runtime-dev seeded `canary_only`,
4. verify with both focused unit and live runtime-surface proof.

Acceptance criteria:

1. `load_or_seed_operator_objectives()` upgrades a seeded-default runtime-dev profile to `canary_only`,
2. `default_operator_objectives()` itself remains conservative and test-stable,
3. runtime-prod or unset-runtime seeded defaults remain `manual_only`,
4. operator-edited objectives are not silently rewritten in runtime-dev,
5. focused proof exists through:
   - `make test-operator-objectives-contract`
   - `make test-oversight-apply`
   - `make test-adversary-sim-runtime-surface`
6. live local evidence shows:
   - `objectives.rollout_guardrails.automated_apply_status=canary_only`
   - and the latest post-sim oversight agent run reaches `apply.stage=canary_applied`.

# Sequencing

1. Land this only after `RSI-GAME-ARCH-1H`.
2. Re-check live local post-sim oversight status immediately after the seed upgrade.
3. Only then move on to the next true RSI gap: measured retained improvement across repeated changed-config runs.

# Definition Of Done

This tranche is complete when:

1. runtime-dev seeded defaults enable canary apply for the strict local Scrapling loop,
2. production defaults stay conservative,
3. operator-owned profiles remain respected,
4. and the live local loop advances into actual canary mutation rather than stopping at preview-only apply refusal.
