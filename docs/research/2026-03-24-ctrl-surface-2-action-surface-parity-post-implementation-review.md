Date: 2026-03-24
Status: Completed

Related context:

- [`../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md)
- [`2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md`](2026-03-24-ctrl-surface-1-controller-mutability-policy-post-implementation-review.md)
- [`../../src/config/controller_action_surface.rs`](../../src/config/controller_action_surface.rs)
- [`../../src/config/controller_action_catalog.rs`](../../src/config/controller_action_catalog.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)

# What landed

`CTRL-SURFACE-2` now makes the live controller surface obey the canonical mutability policy rather than the older catalog-local status strings.

The main parity changes are:

1. `allowed_actions_v1` group statuses now derive from the canonical mutability policy.
2. benchmark escalation now considers only group-level `allowed` surfaces as candidate tuning families.
3. the patch proposer now treats only `allowed` groups as proposable.
4. the catalog no longer carries the read-only `ip_range_suggestions_*` knobs inside the controller write surface.
5. `challenge_puzzle_risk_threshold` now belongs to the `challenge` family end to end, including patch-family mapping and proposal metadata.

# Why it mattered

Before this slice, the repo still had three important drifts:

1. the live action surface still trusted catalog-local `controller_status` values instead of the new canonical policy,
2. benchmark escalation missed mixed families like `core_policy` and `cdp_detection` even when they contained real controller-tunable groups,
3. and the patch proposer still treated `manual_only` as proposable, which blurred the line between operator-only posture and legal controller moves.

# Proof added

The new focused parity target is:

- `make test-controller-action-surface-parity`

It proves:

1. `allowed_actions_v1` now reflects the canonical mutability rings,
2. verified-identity surfaces are forbidden in the controller envelope,
3. challenge-threshold ownership now points at the `challenge` family,
4. benchmark escalation includes addressable mixed families like `core_policy`,
5. benchmark escalation excludes forbidden families like `browser_policy`, `geo_policy`, and `ip_range_policy`,
6. and the patch proposer matches the new family ownership and allowed-only proposal semantics.

# Remaining work

This slice settles parity across the policy, action surface, escalation, and patch shaper, but `CTRL-SURFACE-3` still remains:

1. explicit hard-boundary enforcement tests beyond the current focused parity cases,
2. broader operator-facing documentation of the rings,
3. and later UI or explanation surfaces consuming the canonical policy directly.

# Verification

- `make test-controller-action-surface-parity`
- `make test-controller-mutability-policy`
- `make test-oversight-reconcile`
- `git diff --check`

# CI

CI was not yet visible at closeout time, so remote CI remains unverified from this review.
