Date: 2026-03-24
Status: Proposed

Related context:

- [`../research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](../research/2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)
- [`../../src/config/controller_action_catalog.rs`](../../src/config/controller_action_catalog.rs)
- [`../../src/config/controller_action_surface.rs`](../../src/config/controller_action_surface.rs)
- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../dashboard/src/lib/domain/config-schema.js`](../../dashboard/src/lib/domain/config-schema.js)

# Objective

Make the controller action space explicit, narrow, and enforceable so the feedback loop can optimize toward operator-defined policy targets without ever mutating:

1. the targets themselves,
2. hard security or trust-boundary config,
3. or Shuma's own safety-critical operating envelope.

# Core Decisions

1. `operator_objectives_v1` is the rule set for the game and must never be controller-mutable.
2. Admin writability is not controller eligibility.
3. The controller needs one canonical mutability policy that classifies every writable path as `never`, `manual_only`, or `controller_tunable`.
4. The current loop should remain narrow: only ratified bounded sensitivity and rollout knobs are in-bounds.
5. Any future reopening of currently out-of-bounds paths must require explicit new research and plan-level signoff; no path should become controller-tunable merely because it is already admin-writable.

# Task 0: Focused Verification Prep

**Files:**

- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**

1. Add a focused proof target for the mutability policy contract, for example `test-controller-mutability-policy`.
2. Add or refine a focused proof target for `allowed_actions_v1` and patch-policy parity, for example `test-controller-action-surface-parity`.
3. Add or refine a focused proof target for "hard-never" rejection behavior, for example `test-controller-hard-boundaries`.

**Acceptance criteria:**

1. the mutability policy, allowed-action surface, and hard-boundary behavior each have narrow truthful proof paths.

## Task 1: `CTRL-SURFACE-1`

### Canonical controller mutability policy and full writable-surface classification

**Files:**

- Modify: `src/config/controller_action_catalog.rs`
- Modify: `src/config/controller_action_surface.rs`
- Modify: `src/config/controller_action_guardrails.rs`
- Modify: `src/admin/api.rs`
- Modify: `dashboard/src/lib/domain/config-schema.js`
- Modify: `docs/configuration.md`
- Modify: `docs/api.md`

**Work:**

1. Define one canonical mutability policy over the full writable surface:
   1. `never`
   2. `manual_only`
   3. `controller_tunable`
2. Classify every admin-writable config path accepted by `POST /admin/config`.
3. Classify the separate `operator_objectives_v1` surface explicitly as out-of-bounds for the controller.
4. Make the hard-never categories explicit in code and docs:
   1. operator objectives,
   2. runtime harness and shadow-mode controls,
   3. provider and edge topology,
   4. verified-identity policy,
   5. `robots.txt` and AI policy,
   6. trust exceptions and allowlists,
   7. privacy posture,
   8. punishment horizons,
   9. defender resource or safety budgets,
   10. implementation-composition selectors.

**Acceptance criteria:**

1. there is one canonical answer to "may the controller touch this path?",
2. operator targets are explicitly excluded from the controller move set,
3. and docs, API truth, and Advanced JSON classification can derive from the same policy.

**Verification:**

1. `make test-controller-mutability-policy`
2. `git diff --check`

## Task 2: `CTRL-SURFACE-2`

### Align `allowed_actions_v1` and the patch proposer with the ratified tunable surface

**Files:**

- Modify: `src/config/controller_action_catalog.rs`
- Modify: `src/config/controller_action_surface.rs`
- Modify: `src/admin/oversight_patch_policy.rs`
- Modify: `src/observability/benchmark_results_comparison.rs`
- Modify: `src/admin/oversight_reconcile.rs`
- Modify: `docs/configuration.md`
- Modify: `docs/api.md`

**Work:**

1. Narrow the candidate controller-tunable set to the ratified bounded config families and paths.
2. Fix catalog/proposer drift so declared controller-tunable paths and actually proposable paths no longer disagree silently.
3. Fix family-path mismatches, including the current `challenge_puzzle_risk_threshold` classification drift between `challenge` and `botness`.
4. Ensure benchmark escalation and reconcile only surface candidate families that still sit inside the ratified controller-tunable ring.
5. Make "controller-tunable but not yet auto-proposable" impossible without explicit status and tests.

**Acceptance criteria:**

1. `allowed_actions_v1` and the patch proposer tell the same truth about what the current loop can tune,
2. family and path ownership are internally consistent,
3. and the controller no longer inherits accidental eligibility from the broader admin-config surface.

**Verification:**

1. `make test-controller-action-surface-parity`
2. `make test-oversight-reconcile`
3. `make test-oversight-apply`
4. `git diff --check`

## Task 3: `CTRL-SURFACE-3`

### Enforce hard boundaries and surface mutability truthfully

**Files:**

- Modify: `src/admin/oversight_patch_policy.rs`
- Modify: `src/config/tests.rs`
- Modify: `src/admin/oversight_api.rs`
- Modify: `docs/configuration.md`
- Modify: `docs/dashboard-tabs/advanced.md`
- Modify: `docs/dashboard-tabs/tuning.md`
- Modify: `docs/testing.md`

**Work:**

1. Add explicit tests proving the controller cannot propose or apply:
   1. operator objectives,
   2. provider selection,
   3. verified-identity policy,
   4. `robots.txt` and AI policy,
   5. trust exceptions and allowlists,
   6. hard safety or resource-budget controls.
2. Document the canonical mutability rings in operator and contributor docs.
3. Make later Monitoring or Tuning controller-explanation work depend on this canonical mutability policy instead of inferring mutability from admin writability.
4. Make `ADV-JSON-1` consume the new classification rather than inventing a second local grouping truth.

**Acceptance criteria:**

1. hard-never paths are defended by code and tests, not just prose,
2. operator-facing docs can explain what the controller may tune and what remains operator-owned,
3. and later UI surfaces have one canonical mutability source of truth.

**Verification:**

1. `make test-controller-hard-boundaries`
2. `make test-dashboard-config-parity`
3. `git diff --check`

# Sequencing

1. Execute `CTRL-SURFACE-1` first.
2. Then execute `CTRL-SURFACE-2`.
3. Then execute `CTRL-SURFACE-3`.
4. Keep `MON-OVERHAUL-1` and `TUNE-SURFACE-1A` independent of this work where possible, but require this mutability policy before later controller-explanation or patch-family-surfacing work in `TUNE-SURFACE-1B`, `OVR-AGENT-2`, and `OVR-CODE-1`.

# Definition Of Done

This plan is complete when:

1. the repo has one canonical mutability policy across the writable config and objective surfaces,
2. the controller action surface is narrower than admin writability and explicitly documented,
3. hard-never paths are enforced and tested,
4. `allowed_actions_v1` and the patch proposer no longer drift,
5. and later Monitoring, Tuning, and controller phases can project mutability truth without inventing a second policy model.
