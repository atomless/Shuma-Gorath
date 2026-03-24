Date: 2026-03-24
Status: Completed

Related plan:

- [`../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md)

## Objective

`CTRL-SURFACE-1` was meant to replace the repo's fuzzy "admin-writable implies maybe controller-addressable" posture with one canonical controller mutability policy over:

1. every admin-writable config path,
2. the separate `operator_objectives_v1` rule surface,
3. and the existing `allowed_actions_v1` action envelope.

## What landed

1. Canonical path-level mutability rings now exist in [`../../src/config/controller_action_catalog.rs`](../../src/config/controller_action_catalog.rs):
   - `controller_tunable`
   - `manual_only`
   - `never`
2. `allowed_actions_v1` now carries that truth forward in [`../../src/config/controller_action_surface.rs`](../../src/config/controller_action_surface.rs):
   - `controller_mutability_schema_version`
   - `admin_config_path_mutability`
   - `operator_objectives_path_mutability`
   - per-group `controller_mutability`
   - per-family `controller_mutability`
3. The dashboard config schema now mirrors the admin-config mutability groupings in [`../../dashboard/src/lib/domain/config-schema.js`](../../dashboard/src/lib/domain/config-schema.js), with focused parity proof against the backend source of truth.
4. The old stale `ip_range_suggestions_*` entries were removed from the controller catalog's writable patch-path surface because those keys are not actually accepted by `POST /admin/config`.

## Why this is better

1. The repo now has one explicit answer to "may the controller touch this exact path?" rather than relying on family-level inference.
2. `operator_objectives_v1` is now explicitly carried as a `never` surface in the machine contract, which keeps the game rules separate from the move set.
3. The path-level split also makes later UI work safer: Advanced JSON, Tuning, and controller-explanation surfaces can consume a stable backend classification instead of inventing local mutability logic.

## Remaining gap

`CTRL-SURFACE-1` intentionally did **not** finish all legacy `allowed_actions_v1.controller_status` cleanup.

Some groups and families now expose the correct canonical `controller_mutability`, but their older `controller_status` values still reflect the earlier, coarser allowed/manual-only/forbidden framing. That is the remaining `CTRL-SURFACE-2` job: align `allowed_actions_v1`, benchmark escalation, and the patch proposer so there is no residual status drift between the legacy action envelope and the new canonical path-level policy.

## Verification

- `make test-controller-mutability-policy`
- `make test-controller-action-surface`
- `git diff --check`
