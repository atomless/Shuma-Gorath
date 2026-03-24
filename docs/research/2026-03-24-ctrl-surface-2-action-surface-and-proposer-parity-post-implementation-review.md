Date: 2026-03-24
Status: Completed

Related plan:

- [`../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md)

## Objective

`CTRL-SURFACE-2` was meant to remove the remaining drift between:

1. the canonical mutability rings from `CTRL-SURFACE-1`,
2. the machine-first `allowed_actions_v1` surface,
3. benchmark escalation candidates,
4. and the bounded oversight patch proposer.

The main rule was that admin writability and even controller-tunability are not enough on their own: the current bounded loop must expose which paths are actually auto-proposable today.

## What landed

1. Hard-never groups in [`../../src/config/controller_action_catalog.rs`](../../src/config/controller_action_catalog.rs) now carry `controller_status="forbidden"` instead of stale `manual_only` values when the canonical mutability ring is `never`.
2. The catalog now declares exact `proposable_patch_paths` per group, and [`../../src/config/controller_action_surface.rs`](../../src/config/controller_action_surface.rs) carries that truth forward as:
   - per-group `auto_proposal_status`
   - per-group `proposable_patch_paths`
   - per-family `auto_proposal_status`
   - per-family `proposable_patch_paths`
3. [`../../src/config/controller_action_guardrails.rs`](../../src/config/controller_action_guardrails.rs) now summarizes family-level auto-proposal support from the controller-tunable portion of each family rather than from raw family membership.
4. [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs) now surfaces config-tuning candidates only from explicitly auto-proposable controller-tunable families. Operator-owned or hard-never families now block visibly instead of leaking back into tuning candidates.
5. [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs) now validates patches against `proposable_patch_paths` and routes `challenge_puzzle_risk_threshold` through the `botness` family instead of the old `challenge` drift.
6. The benchmark-results contract test in [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs) was tightened so its "config tuning candidate" scenario now isolates an actually addressable breach rather than depending on pre-verified-identity defaults.

## Why this is better

1. `allowed_actions_v1`, benchmark escalation, and the bounded proposer now tell the same truth about what the current loop can actually move.
2. The repo can now distinguish three different states explicitly:
   - controller-tunable and auto-proposable now,
   - controller-tunable but only partially auto-proposable,
   - not controller-tunable at all.
3. The old family-path mismatch for `challenge_puzzle_risk_threshold` is gone, so the patch proposer and catalog no longer disagree about which family owns that knob.

## Remaining gap

`CTRL-SURFACE-2` makes the surface truthful, but it does not yet enforce the hard boundaries end to end.

`CTRL-SURFACE-3` is still required to:

1. add explicit tests that the controller cannot propose or apply hard-never surfaces,
2. thread the canonical mutability and auto-proposal truth through later operator-facing explanation surfaces,
3. and make those hard boundaries depend on code and tests rather than only on contract shape.

## Verification

- `make test-controller-action-surface-parity`
- `make test-oversight-reconcile`
- `make test-oversight-apply`
- `make test-benchmark-results-contract`
- `git diff --check`
