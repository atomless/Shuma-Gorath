# Operator Snapshot Allowed-Actions Slice Post-Implementation Review

Date: 2026-03-20

## Scope Reviewed

- `src/config/controller_action_surface.rs`
- `src/config/mod.rs`
- `src/config/tests.rs`
- `src/admin/api.rs`
- `src/observability/operator_snapshot.rs`
- `src/observability/hot_read_contract.rs`
- `src/observability/hot_read_projection.rs`
- `Makefile`
- `docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`
- `docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-implementation-plan.md`

## Objective Check

`OPS-SNAPSHOT-1-5` was meant to materialize `allowed_actions_v1` as the explicit controller action envelope so later scheduled-controller planning, benchmark work, and Monitoring projection would all inherit one truthful bounded write surface.

That objective is now met.

## What Landed Well

1. The implementation stayed machine-first. `allowed_actions_v1` is a typed backend contract inside `operator_snapshot_v1`, not a dashboard-only interpretation.
2. The implementation stayed conservative. The first controller write surface is narrow by design, with many groups explicitly marked `manual_only` or `forbidden` rather than pretending Shuma is ready for broader autonomous tuning.
3. The implementation improved shared truth, not just snapshot shape. The config-family catalog now drives both `allowed_actions_v1` and recent-change family targeting, which removed a subtle drift in `core_policy` target attribution.
4. The implementation stayed aligned with the plans. The surface enumerates allowed groups, manual-only groups, forbidden groups, canary requirements, patch paths, and bounded value envelopes where Shuma already has explicit clamp semantics.

## Review Findings

No new architectural blocker was found after implementation.

Two choices are especially worth preserving:

1. Families without explicit safe numeric envelopes, trust-boundary semantics, or rollout-safe automation posture remain `manual_only` rather than being forced into the autonomous surface prematurely.
2. Backend/provider selection remains explicitly `forbidden`, which keeps the first controller loop inside the intended config-diff-only boundary.

## Follow-On Implications

1. `OPS-SNAPSHOT-1` is now complete. The machine-first operator snapshot foundation is no longer the blocker for later Monitoring work.
2. `OPS-BENCH-1` is now the right next tranche, because Monitoring should project benchmark-aware operator truth rather than invent a human-only success model.
3. Future expansion of `allowed_actions_v1` should require explicit new safe envelopes and plan-level signoff for any family that is currently `manual_only`.

## Recommendation

Treat `OPS-SNAPSHOT-1` as complete.

The next work should follow the planned order:

1. implement `OPS-BENCH-1`,
2. review that benchmark contract against the machine-first snapshot base,
3. then move into `MON-OVERHAUL-1` as a thin projection over the completed backend contracts.

## Evidence

- `make test-operator-snapshot-foundation`
- `make test-monitoring-telemetry-foundation-unit`
- `git diff --check`
