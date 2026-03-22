# OPS-OBJECTIVES-3 Post-Implementation Review

Date: 2026-03-22

## Scope reviewed

- `OPS-OBJECTIVES-3`
- Plan reference: [`../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md`](../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md)

## What landed

1. `operator_objectives_v1` now persists canonical per-category posture rows through [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs), using the bounded posture scale `allowed`, `tolerated`, `cost_reduced`, `restricted`, and `blocked` keyed by canonical non-human category ids.
2. The operator-objectives admin write surface now accepts and records category posture state as first-class decision lineage in [`../../src/admin/operator_objectives_api.rs`](../../src/admin/operator_objectives_api.rs), including explicit decision-ledger and recent-change targets for `operator_objectives.category_postures`.
3. `operator_snapshot_v1` now projects the persisted category posture rows through the same machine-first snapshot contract in [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs) and keeps the hot-read contract description aligned in [`../../src/observability/hot_read_contract.rs`](../../src/observability/hot_read_contract.rs).
4. The docs and focused verification surface now treat category-aware objectives as the authoritative operator truth in [`../../docs/api.md`](../../docs/api.md), [`../../docs/configuration.md`](../../docs/configuration.md), [`../../docs/testing.md`](../../docs/testing.md), and [`../../Makefile`](../../Makefile).

## Acceptance check

### 1. The controller has a truthful per-category utility function

Passed.

- The persisted objective contract no longer collapses non-human posture into one coarse field.
- Validation now requires exactly one posture row for every canonical non-human category and rejects duplicates, unknown categories, or postures outside the bounded scale.
- The default seeded profile now distinguishes beneficial and tolerated categories from higher-cost or hostile ones in backend truth.

### 2. Operators can distinguish beneficial non-human categories from hostile or expensive ones in backend truth

Passed.

- The snapshot now exposes category posture rows directly alongside the canonical taxonomy and classification chain.
- The default seeded profile explicitly keeps `verified_beneficial_bot` at `allowed`, `agent_on_behalf_of_human` at `tolerated`, and more expensive or hostile categories at `restricted` or `blocked`.
- The later benchmark and apply work can now read this category posture truth from the same persisted objective contract rather than inferring intent from prose or a single coarse stance.

## Verification run

1. `make test-operator-objectives-category-contract`
2. `make test-operator-snapshot-foundation`
3. `git diff --check`

## Architectural review

The tranche stayed within the intended boundary:

- it extended the existing `operator_objectives_v1` contract instead of creating a second category-policy document,
- it reused the canonical runtime taxonomy as the source of category ids rather than duplicating category metadata into the objective rows,
- it kept the operator-facing posture scale bounded and explicit for later benchmarking and tuning work,
- and it preserved the pre-launch cleanliness rule by not adding backward-compatibility aliases for the retired coarse posture field.

That is the right shape here because the benchmark and apply loop now have one persisted, site-owned objective document to consult for category posture, while the canonical taxonomy remains the separate source of labels, descriptions, and category meaning.

## Shortfalls found

One real tranche-local issue surfaced during closeout review and was fixed immediately inside `OPS-OBJECTIVES-3`:

1. The first draft pushed `operator_snapshot_v1` beyond its bounded hot-read budget once the full category posture rows were projected into the snapshot.
   - Fix: keep the new objective rows in the snapshot and raise the operator-snapshot hot-read budget modestly from `36 KiB` to `38 KiB` in [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs), then re-run the snapshot foundation gate.
   - Evidence: [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs), [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)

No remaining tranche-local shortfall is left open.

## Next step

Proceed to `OPS-BENCH-3`.
