Date: 2026-03-24
Status: Completed

Related plan:

- [`../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md)

## Objective

`CTRL-SURFACE-3` was meant to turn the mutability policy from a descriptive contract into an enforced one by proving:

1. operator objectives stay outside the controller move ring,
2. hard-never families cannot be proposed,
3. non-proposable hard-boundary keys inside mixed families cannot leak through patch validation,
4. and operator-facing docs consume the canonical mutability truth instead of inferring it from admin writability.

## What landed

1. [`../../src/config/tests.rs`](../../src/config/tests.rs) now proves `operator_objectives_v1` is entirely `never` and has no controller patch-family mapping.
2. [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs) now has explicit tests proving:
   - hard-never families such as `provider_selection`, `verified_identity`, `robots_policy`, `allowlists`, `tarpit`, `ip_range_policy`, `geo_policy`, and `honeypot` are not proposable,
   - and mixed families reject non-proposable hard-boundary keys such as `ban_duration`, `maze_token_ttl_seconds`, `cdp_probe_family`, and `fingerprint_pseudonymize`.
3. The focused verification surface now exposes:
   - `make test-controller-hard-boundaries`
   - `make test-dashboard-config-parity`
4. Operator docs now state the separation cleanly:
   - [`../../docs/configuration.md`](../../docs/configuration.md)
   - [`../../docs/testing.md`](../../docs/testing.md)
   - [`../../docs/dashboard-tabs/advanced.md`](../../docs/dashboard-tabs/advanced.md)
   - [`../../docs/dashboard-tabs/tuning.md`](../../docs/dashboard-tabs/tuning.md)

## Why this is better

1. The controller boundary is now enforced by targeted tests at the exact seams where drift could re-enter:
   - candidate-family proposal
   - mixed-family patch-key validation
   - operator-objective exclusion
2. Advanced and Tuning now describe the right ownership split:
   - Advanced reflects admin writability,
   - Tuning reflects operator tuning posture,
   - neither is allowed to infer controller eligibility without `allowed_actions_v1`.
3. Later controller-explanation work now has a stable backend contract to project instead of having to guess from writable config shape.

## Remaining gap

`CTRL-SURFACE-3` closes the mutability-policy tranche.

The remaining follow-on is not another mutability cleanup slice, but downstream adoption:

1. `ADV-JSON-1` should consume the canonical mutability and auto-proposal truth directly in the Advanced UI.
2. Later Game Loop and Tuning explanations should reuse the same contract when showing why the loop may or may not touch a given surface.

## Verification

- `make test-controller-hard-boundaries`
- `make test-dashboard-config-parity`
- `git diff --check`
