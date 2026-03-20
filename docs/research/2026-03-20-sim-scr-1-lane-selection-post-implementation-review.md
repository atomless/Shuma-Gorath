# SIM-SCR-1 Lane Selection Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md`](../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md)
- [`2026-03-20-sim-scr-0-lane-contract-post-implementation-review.md`](./2026-03-20-sim-scr-0-lane-contract-post-implementation-review.md)
- [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
- [`../../src/admin/adversary_sim_control.rs`](../../src/admin/adversary_sim_control.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../Makefile`](../../Makefile)
- [`../../docs/api.md`](../../docs/api.md)
- [`../../docs/testing.md`](../../docs/testing.md)

## Review Goal

Confirm that `SIM-SCR-1` landed as the control/state slice of the lane migration:

1. control payload accepts strict optional lane selection,
2. desired lane persists separately from ON/OFF intent,
3. desired-versus-active lane divergence is visible and auditable,
4. and Shuma still does not pretend the worker has switched lanes before `SIM-SCR-6`.

## What Was Intended

This slice was meant to add lane selection to the existing control contract without bundling in worker routing.

That means:

1. `enabled` stays the first-class lifecycle intent,
2. `lane` becomes a strict optional selector,
3. idempotency and audit surfaces must include lane information,
4. and running-state lane changes may update desired lane before active lane changes are possible.

## What Landed

1. `POST /admin/adversary-sim/control` now accepts optional `lane` values `synthetic_traffic`, `scrapling_traffic`, and `bot_red_team`, and rejects invalid lane values at payload parse time.
2. Control payload hashing and idempotency now include lane selection, so a same-key replay with only a lane difference is rejected as a payload mismatch.
3. Desired lane can now be persisted while the simulator is off, which lets operators preselect the next lane before the runtime is started.
4. Operation records, audit payloads, and control responses now include requested, desired, and actual lane information alongside the existing phase and enabled-state fields.
5. When a non-synthetic lane is selected while the simulator is already running, `desired_lane` changes immediately but `active_lane` remains `synthetic_traffic`, and `controller_reconciliation_required` becomes true.
6. `make test-adversary-sim-lane-selection` now provides a focused proof for strict lane validation, off-state persistence, lane-aware idempotency, and desired-versus-active divergence.

## Architectural Assessment

### 1. Control truth is now richer without becoming dishonest

This slice does not claim Scrapling is running just because the operator asked for it.

That is the right behavior before `SIM-SCR-6`.

### 2. The control plane now matches the planned migration boundary

The backend has one explicit place to store:

1. operator lane intent,
2. current actual lane,
3. and the gap between them.

That gives the later heartbeat-routing slice a clean state model instead of forcing it to infer desired lane from API payload timing.

### 3. Idempotency is now lane-aware

This was the most important low-level control-plane hardening inside the slice.

Without it, identical `enabled` and `reason` values could have let same-key replays silently collapse distinct lane-selection intents.

## Shortfalls Found During Review

No new architectural shortfall was found inside `SIM-SCR-1`.

One explicit migration boundary remains and is intentional:

1. `active_lane` remains `synthetic_traffic` until `SIM-SCR-6` lands the actual heartbeat router and bounded Scrapling worker dispatch.

That is not a missing fix in this tranche. It is the truthful current runtime contract.

## Result

Treat `SIM-SCR-1` as complete.

The next optimal tranche is `SIM-SCR-6`:

1. route beat execution through the selected lane,
2. keep actual lane changes at beat boundary,
3. and make desired-versus-active convergence real instead of planned-only.
