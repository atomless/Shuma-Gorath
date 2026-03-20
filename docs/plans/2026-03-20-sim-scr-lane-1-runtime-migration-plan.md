# SIM-SCR-LANE-1 Runtime Migration Plan

Date: 2026-03-20
Status: Active implementation plan

Related context:

- [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](./2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](./2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](./2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`../research/2026-03-20-sim-scr-lane-1-readiness-review.md`](../research/2026-03-20-sim-scr-lane-1-readiness-review.md)
- [`../research/2026-03-20-shared-host-seed-contract-post-implementation-review.md`](../research/2026-03-20-shared-host-seed-contract-post-implementation-review.md)
- [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/admin/adversary_sim_control.rs`](../../src/admin/adversary_sim_control.rs)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/runtime/dashboard-adversary-sim.js`](../../dashboard/src/lib/runtime/dashboard-adversary-sim.js)

## Purpose

Turn `SIM-SCR-LANE-1` into an execution-ready runtime migration plan that:

1. preserves the current toggle-only contract as baseline truth,
2. adds explicit desired-versus-active lane semantics,
3. introduces Scrapling under backend heartbeat ownership,
4. and keeps traversal telemetry as the reachable-surface map.

## Why This Needs Its Own Narrow Plan

The 2026-03-04 Scrapling plan already defines the broad tranche, but the codebase has moved since then:

1. `SIM-DEPLOY-2` settled the production lifecycle contract on 2026-03-20,
2. `SIM-SH-SURFACE-1-1..2` settled the minimal shared-host gate on 2026-03-20,
3. and the dashboard still normalizes the older `lanes.{deterministic,containerized}` shape.

So the next work is no longer a generic Scrapling discussion.

It is a migration problem with one critical rule:

1. land the lane-state contract first,
2. then land the lane-selection write path,
3. then attach the Scrapling worker,
4. and only then project the new model into dashboard controls.

## Code-Truth Baseline On 2026-03-20

Today Shuma still behaves like this:

1. `ControlState` persists `phase`, `desired_enabled`, ownership, counts, and generation diagnostics, but no lane enum.
2. `POST /admin/adversary-sim/control` accepts `enabled` plus optional `reason` only.
3. status exposes `active_lane_count` plus `lanes.deterministic` and `lanes.containerized`, both derived from lifecycle phase rather than from a selected runtime lane.
4. the dashboard adapters expect that legacy shape and do not yet read `desired_lane`, `active_lane`, or switch metadata.
5. runtime generation remains deterministic internal traffic and is not yet dispatched through a selected lane router.

This plan treats all lane-selection work as forward migration from that baseline.

## Core Decisions

### 1. Additive-first migration

The first backend slice must add new lane fields without removing the legacy ones.

Status must continue to publish:

1. `active_lane_count`,
2. `lanes.deterministic`,
3. and `lanes.containerized`

until the dashboard and any other consumers are migrated.

### 2. Desired and active lane are separate authorities

The migration should introduce:

1. `desired_lane`: the persisted operator-selected lane,
2. `active_lane`: the lane actually executing on the current beat,
3. `lane_switch_seq`: monotonically increasing switch sequence number,
4. `last_lane_switch_at`: last successful active-lane change timestamp,
5. and `last_lane_switch_reason`: last successful active-lane change reason.

This mirrors the existing `desired_enabled` versus actual lifecycle split and keeps temporary divergence observable rather than implicit.

### 3. Stable lane vocabulary

The target runtime lane enum is:

1. `synthetic_traffic`
2. `scrapling_traffic`
3. `bot_red_team`

No other runtime-lane labels should be accepted in control payloads or emitted in new status fields.

### 4. Off-state semantics must stay honest

Recommended contract:

1. `desired_lane` is always present and persists the operator choice even while disabled,
2. `active_lane` is `null` when no lane is currently executing,
3. `desired_enabled=false` remains the authoritative OFF intent,
4. and `phase` remains the lifecycle truth for running, stopping, or off.

### 5. Lane selection is separate from enablement

The control path should support operator preselection of the next lane without forcing an immediate run.

That means a payload with `enabled=false` and `lane="scrapling_traffic"` is valid and should persist the next desired lane while keeping the simulator off.

### 6. Beat boundary applies the switch

Lane changes should not mutate work mid-beat.

The scheduler contract is:

1. control writes `desired_lane`,
2. the supervisor reconciles `active_lane` at the next safe beat boundary,
3. the prior lane stops dispatching before the new lane starts,
4. and the switch metadata records the change.

## Status Contract Shape

The additive migration fields should look like this:

```json
{
  "desired_lane": "synthetic_traffic",
  "active_lane": "synthetic_traffic",
  "lane_switch_seq": 0,
  "last_lane_switch_at": null,
  "last_lane_switch_reason": null
}
```

When the simulator is off:

```json
{
  "desired_lane": "scrapling_traffic",
  "active_lane": null,
  "lane_switch_seq": 0,
  "last_lane_switch_at": null,
  "last_lane_switch_reason": null
}
```

The first slice keeps legacy fields alongside these migration fields.

## Diagnostics Contract

The `SIM-SCR-0` scaffolding slice should also add bounded lane diagnostics that later worker slices can fill in without changing schema:

1. per-lane beat attempts,
2. per-lane beat successes,
3. per-lane beat failures,
4. failure classes `cancelled`, `timeout`, `transport`, `http`,
5. and last-seen timestamps for each failure class.

Non-negotiable rule:

1. `cancelled` is an observational worker outcome only,
2. and it must not mutate global dashboard connection state or any shared control-plane availability state.

## Audit And Control Contract

The later `SIM-SCR-1` slice should extend the control surface, not replace it.

Required behavior:

1. `enabled` remains required and first-class.
2. `lane` becomes an optional field with strict enum validation.
3. omitting `lane` preserves the current desired lane.
4. the default lane for a never-written state is `synthetic_traffic`.
5. audit logs and operation records must include requested, desired, and actual lane information once the field exists.

## Execution Order

### 1. `SIM-SCR-0` Backend contract and diagnostics scaffolding

Scope:

1. add the new migration fields to persisted state and status payload shape,
2. add empty or baseline lane diagnostics documents/counters,
3. keep runtime behavior unchanged,
4. and keep existing dashboard/status consumers working.

Acceptance:

1. status includes `desired_lane`, `active_lane`, `lane_switch_seq`, `last_lane_switch_at`, `last_lane_switch_reason`,
2. legacy `active_lane_count` and `lanes.*` fields still exist,
3. deterministic baseline reports `desired_lane=synthetic_traffic`,
4. and focused tests fail if the migration fields disappear or use invalid values.

### 2. `SIM-SCR-1` Persisted lane-selection control model

Scope:

1. extend `ControlState` with desired and active lane semantics,
2. extend the control request with optional `lane`,
3. persist lane changes even when the simulator is off,
4. and extend control/status/audit payloads accordingly.

Acceptance:

1. control accepts existing toggle-only payloads unchanged,
2. control rejects invalid lane values,
3. lane selection is visible in persisted state and status,
4. and status truthfully reflects desired-versus-active divergence during lifecycle transitions.

### 3. `SIM-SCR-6` Scrapling worker integration

Scope:

1. route heartbeat work through the selected lane,
2. keep deterministic generation as the `synthetic_traffic` baseline,
3. add bounded Scrapling worker execution for `scrapling_traffic`,
4. and do not auto-enable `bot_red_team`.

Acceptance:

1. one lane executes per beat,
2. `active_lane` changes only at beat boundary,
3. Scrapling stays inside the shared-host scope-and-seed contract,
4. and traversal telemetry, not a catalog, becomes the working surface map.

### 4. `SIM-SCR-7` Dashboard lane controls and diagnostics

Scope:

1. add the 3-lane operator selector,
2. project desired versus active lane honestly,
3. keep `bot_red_team` visibly disabled until implemented,
4. and preserve the existing top-level ON/OFF semantics.

Acceptance:

1. rendered UI matches backend contract fields,
2. lane selection changes remain read-after-write consistent,
3. and no legacy consumer regresses while migration fields coexist.

### 5. `SIM-SCR-8` Operator workflow and Make targets

Scope:

1. document the new lane-selection contract,
2. add focused Make targets or refine existing ones where verification naming/scope needs tightening,
3. and capture rollout, rollback, and no-impact guidance for the new runtime lane.

Acceptance:

1. docs describe the settled contract rather than the migration intermediate,
2. verification targets truthfully match their scope,
3. and the operator guide explains how the new lane model interacts with shared-host seeds and traversal telemetry.

## Verification Strategy

Use focused `make` targets per slice and add new ones before relying on ad hoc commands.

Initial expected verification path:

1. a focused Rust/admin target for status/control migration tests,
2. existing adversary-sim lifecycle verification where still relevant,
3. then later worker, dashboard, and Make-target proofs as those slices land.

The first backend slices should avoid broad runtime or browser churn until they actually touch those surfaces.

## What This Tranche Must Not Do

This tranche must not:

1. remove legacy status keys before dashboard migration lands,
2. start with Scrapling worker execution before the state contract exists,
3. introduce a new discovery catalog or static route inventory,
4. claim `bot_red_team` is implemented,
5. or let request-cancellation behavior mutate global connection or control availability state.

## Result

`SIM-SCR-LANE-1` should proceed as a backend-first lane migration:

1. additive contract,
2. persisted desired/active lane state,
3. bounded Scrapling worker routing,
4. dashboard projection,
5. then operator and verification closeout.
