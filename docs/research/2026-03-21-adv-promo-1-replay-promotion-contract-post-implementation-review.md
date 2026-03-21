# `ADV-PROMO-1` Replay-Promotion Contract Post-Implementation Review

Date: 2026-03-21  
Status: complete

## Scope reviewed

`ADV-PROMO-1` was meant to pull replay-promotion lineage out of Python-side sidecar artifacts and into the backend control-plane contract chain so later reconcile and agent work can consume one bounded machine-first truth surface.

Expected outcomes:

1. bounded persisted replay-promotion contract,
2. admin read/write surface for materialization,
3. snapshot and benchmark visibility,
4. promotion-lane materialization into backend state during real runs,
5. no uncontrolled deterministic-corpus mutation.

## What landed

1. `src/observability/replay_promotion.rs` now defines the bounded persisted `replay_promotion_v1` contract, ingest path, summary projection, and lineage truncation rules.
2. `src/admin/replay_promotion_api.rs` now exposes `GET/POST /admin/replay-promotion`.
3. `src/observability/operator_snapshot.rs`, `src/observability/benchmark_results.rs`, `src/observability/hot_read_contract.rs`, and `src/admin/api.rs` now project replay-promotion state into the machine-first snapshot and benchmark surfaces and route/auth the new admin endpoint correctly.
4. `scripts/tests/adversarial_promote_candidates.py` now fails closed if backend materialization cannot complete, and `scripts/tests/adversarial_simulation_runner.py` gained a shared read helper for the new contract.
5. `Makefile`, `docs/api.md`, `docs/adversarial-operator-guide.md`, and `docs/testing.md` now document and verify the new backend contract.

## Verification performed

1. `make test-replay-promotion-contract`
2. `make test-adversarial-python-unit`
3. `make test-adversarial-promote-candidates`
4. `git diff --check`

The live promotion-lane run completed successfully against the running local server and would have failed if the new `POST /admin/replay-promotion` materialization step had not succeeded.

## Shortfall found during review

### `ADV-PROMO-1-REVIEW-1`

Initial implementation mapped all replay-promotion persistence failures to `400`, which incorrectly treated backend serialization/storage faults as caller mistakes.

Fix executed immediately:

1. introduced typed replay-promotion persist errors in `src/observability/replay_promotion.rs`,
2. mapped invalid payloads to `400` and persistence faults to `500` in `src/admin/replay_promotion_api.rs`,
3. reran the focused replay-promotion contract gate.

## Final assessment

`ADV-PROMO-1` now meets the plan intent:

1. replay-promotion lineage is a first-class backend contract,
2. later reconcile and agent code can read it from `GET /admin/replay-promotion`, `operator_snapshot_v1`, and `benchmark_results_v1`,
3. promotion tooling no longer stops at sidecar JSON,
4. and no tranche-local shortfall remains open before `OVR-RECON-1`.
