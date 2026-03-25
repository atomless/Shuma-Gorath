Date: 2026-03-24
Status: Completed

Related context:

- [`../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md)
- [`2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
- [`../../src/config/controller_mutability_policy.rs`](../../src/config/controller_mutability_policy.rs)
- [`../../src/config/tests.rs`](../../src/config/tests.rs)
- [`../../Makefile`](../../Makefile)

# What landed

`CTRL-SURFACE-1` now has a canonical mutability policy module for the two relevant rule or config surfaces:

1. `admin_config`
2. `operator_objectives_v1`

The repo can now answer, from one source of truth, whether a path is:

1. `never`
2. `manual_only`
3. `controller_tunable`

The policy explicitly freezes:

1. all operator objectives,
2. runtime harness controls,
3. provider and edge topology,
4. verified-identity policy,
5. robots and AI policy,
6. trust exceptions and allowlists,
7. privacy posture,
8. punishment horizons,
9. defender safety and resource budgets,
10. implementation-composition selectors.

It also freezes the intended narrow controller-tunable ring for bounded JS, PoW, challenge, not-a-bot, botness, maze-rollout, CDP, and fingerprint sensitivity controls, while leaving `rate_limit` as the current explicit `manual_only` example.

# Proof added

The new focused proof path is:

- `make test-controller-mutability-policy`

The narrow assertions now prove:

1. operator objectives are permanently out of bounds,
2. representative hard-never config paths resolve as `never`,
3. bounded sensitivity controls resolve as `controller_tunable`,
4. and the current intermediate manual-only ring is explicit rather than implicit.

# Documentation changes

The mutability rings are now called out directly in:

1. [`../../docs/configuration.md`](../../docs/configuration.md)
2. [`../../docs/api.md`](../../docs/api.md)
3. [`../../docs/testing.md`](../../docs/testing.md)

# Shortfalls and next work

This slice intentionally stops short of rewiring the live controller surfaces.

What remains for the next tranche:

1. `allowed_actions_v1` still derives its `controller_status` from the older action-catalog truth rather than this new canonical policy.
2. `oversight_patch_policy`, benchmark escalation, and reconcile have not yet been narrowed to the ratified ring.
3. dashboard and advanced-config classification still need to consume this canonical policy rather than local grouping logic.

That is the correct remaining scope for:

1. `CTRL-SURFACE-2`
2. `CTRL-SURFACE-3`

# Verification

- `make test-controller-mutability-policy`
- `git diff --check`

# CI

CI was not yet visible at closeout time, so remote CI remains unverified from this review.
