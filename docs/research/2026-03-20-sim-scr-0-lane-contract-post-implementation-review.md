# SIM-SCR-0 Lane Contract Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md`](../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md)
- [`2026-03-20-sim-scr-lane-1-readiness-review.md`](./2026-03-20-sim-scr-lane-1-readiness-review.md)
- [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../Makefile`](../../Makefile)
- [`../../docs/api.md`](../../docs/api.md)
- [`../../docs/testing.md`](../../docs/testing.md)

## Review Goal

Confirm that `SIM-SCR-0` landed as the first additive runtime-lane migration slice:

1. new desired-versus-active lane fields in persisted state and status,
2. zeroed lane-diagnostics scaffolding,
3. legacy status compatibility preserved,
4. and a focused `make` gate that proves the new contract without starting Scrapling routing yet.

## What Was Intended

This slice was supposed to do one narrow thing well:

1. make the future three-lane model visible in the backend contract,
2. keep current deterministic runtime behavior unchanged,
3. avoid breaking existing dashboard/status consumers,
4. and give later `SIM-SCR-1`, `SIM-SCR-6`, and `SIM-SCR-7` work one stable backend contract to build on.

## What Landed

1. `ControlState` now carries additive lane-migration fields: `desired_lane`, `active_lane`, `lane_switch_seq`, `last_lane_switch_at`, and `last_lane_switch_reason`.
2. New status payload fields expose that migration contract while preserving legacy `active_lane_count` and `lanes.{deterministic,containerized}` compatibility.
3. Older running state remains readable through a safe synthetic-lane fallback on the status path, so pre-migration persisted rows do not render as lane-less running sessions.
4. The status payload now also includes a bounded zeroed `lane_diagnostics` scaffold covering per-lane beat counters and request failure classes `cancelled`, `timeout`, `transport`, and `http`.
5. `make test-adversary-sim-lane-contract` now provides a truthful focused gate for the additive contract and keeps the first migration slice out of the broader lifecycle and runtime-surface targets.
6. API and testing docs now describe the additive lane fields and the new focused verification path.

## Architectural Assessment

### 1. The migration stays additive

This is the most important property of the slice.

The backend now publishes the future lane contract without forcing the dashboard or operator flow to adopt it in the same patch series.

### 2. Desired versus active lane is now a first-class concept

The future Scrapling integration no longer has to invent this split while also landing worker routing.

That reduces the risk of recreating the same class of status/control ambiguity that earlier lifecycle work removed for `desired_enabled`.

### 3. The diagnostics shape exists before the worker

This means later worker slices can fill in real counters without renegotiating schema with tests, docs, or dashboard consumers.

## Shortfalls Found During Review

No new architectural shortfall was found inside `SIM-SCR-0`.

Two explicit boundaries remain and are intentional, not missing work in this slice:

1. the new lane fields are backend-only for now and are not yet projected by dashboard adapters or controls,
2. and `lane_diagnostics` is scaffolding with zeroed counters until worker-routing slices start producing real per-lane outcomes.

Those remain the planned work of `SIM-SCR-1`, `SIM-SCR-6`, and `SIM-SCR-7`.

## Result

Treat `SIM-SCR-0` as complete.

The next optimal tranche is `SIM-SCR-1`:

1. persist lane selection through the control API,
2. keep `enabled` as the first-class ON/OFF intent,
3. and make desired-versus-active lane divergence auditable through control/status responses.
