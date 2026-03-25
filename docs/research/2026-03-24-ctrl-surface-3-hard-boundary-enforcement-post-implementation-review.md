Date: 2026-03-24
Status: Completed

Related context:

- [`../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md)
- [`2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md`](2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md)
- [`2026-03-24-ctrl-surface-2-action-surface-parity-post-implementation-review.md`](2026-03-24-ctrl-surface-2-action-surface-parity-post-implementation-review.md)
- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)

# What landed

`CTRL-SURFACE-3` now adds explicit fail-closed enforcement and operator-surface documentation over the canonical mutability policy.

The concrete enforcement changes are:

1. the patch-policy layer now rejects known-but-forbidden candidate families explicitly,
2. the apply path now refuses any proposal whose `controller_status` is not `allowed`,
3. and the focused hard-boundary proof target exercises representative forbidden families plus the apply-side refusal seam.

# Why it mattered

After `CTRL-SURFACE-1` and `CTRL-SURFACE-2`, the legal move ring was explicit and the live action surface obeyed it, but the repo still needed one more hardening step:

1. direct proof that representative forbidden families are rejected rather than silently ignored,
2. and one more apply-side guard so non-tunable proposals cannot be auto-applied even if one were ever constructed incorrectly.

# Proof added

The new focused hard-boundary target is:

- `make test-controller-hard-boundaries`

It proves:

1. verified identity is forbidden in the action surface,
2. provider selection is rejected,
3. robots policy is rejected,
4. allowlists are rejected,
5. tarpit is rejected,
6. and apply refuses a proposal that is not controller-tunable.

The parity target still passes afterward:

- `make test-controller-action-surface-parity`

# Operator-surface documentation

The canonical rings are now explicitly called out in:

1. [`../../docs/dashboard-tabs/advanced.md`](../../docs/dashboard-tabs/advanced.md)
2. [`../../docs/dashboard-tabs/tuning.md`](../../docs/dashboard-tabs/tuning.md)
3. [`../../docs/testing.md`](../../docs/testing.md)

This keeps later UI work grounded in the canonical policy even though the dashboard implementation work is intentionally deferred.

# Verification

- `make test-controller-hard-boundaries`
- `make test-controller-action-surface-parity`
- `git diff --check`

# CI

CI was not yet visible at closeout time, so remote CI remains unverified from this review.
