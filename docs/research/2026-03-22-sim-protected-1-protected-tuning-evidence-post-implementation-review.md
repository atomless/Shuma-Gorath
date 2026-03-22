# SIM-PROTECTED-1 Post-Implementation Review

Date: 2026-03-22

## Scope reviewed

- `SIM-PROTECTED-1`
- Plan reference: [`../plans/2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md`](../plans/2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md)

## What landed

1. `ReplayPromotionSummary` now materializes protected-evidence state in [`../../src/observability/replay_promotion.rs`](../../src/observability/replay_promotion.rs), including `evidence_status`, `tuning_eligible`, `protected_basis`, protected-versus-advisory lineage counts, `eligibility_blockers`, and explicit `ineligible_runtime_lanes`.
2. `benchmark_results_v1` now fails escalation closed when replay lineage is still advisory even if classification and category coverage are otherwise ready, via [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs) and [`../../src/observability/non_human_coverage.rs`](../../src/observability/non_human_coverage.rs).
3. Oversight patch shaping now treats advisory replay lineage as requiring explicit replay review before any later proposal can proceed, via [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs).
4. The protected-evidence contract is now asserted across the replay-promotion API, operator snapshot, benchmark endpoint, docs, and a focused Make gate in [`../../Makefile`](../../Makefile), [`../../docs/api.md`](../../docs/api.md), [`../../docs/configuration.md`](../../docs/configuration.md), and [`../../docs/testing.md`](../../docs/testing.md).

## Acceptance check

### 1. Reconcile and later apply logic can distinguish contract-test evidence from tuning-grade evidence

Passed.

- The replay-promotion summary now exposes an explicit machine-readable distinction between `not_materialized`, `advisory_only`, and `protected`.
- Benchmark escalation now emits `protected_tuning_evidence_not_ready` and the underlying replay-promotion blockers instead of silently allowing category-covered but advisory evidence to authorize tuning.
- Patch shaping now keeps replay review in the required verification set whenever replay lineage is not yet tuning-eligible.

### 2. Synthetic ineligibility and replay-promotion promotion are explicit backend facts

Passed.

- `ineligible_runtime_lanes` now always includes `synthetic_traffic`.
- `tuning_eligible=true` is only materialized when replay-promoted lineage is present, thresholds have passed, and owner review is no longer pending.

## Verification run

1. `make test-protected-tuning-evidence`
2. `make test-benchmark-results-contract`
3. `make test-oversight-reconcile`
4. `git diff --check`

## Architectural review

The tranche stayed in the intended boundary:

- it reused the existing replay-promotion summary as the controller-facing evidence seam instead of inventing a parallel protected-evidence document,
- it kept synthetic exclusion as an explicit machine-readable fact rather than a comment or doc-only convention,
- it made the benchmark gate fail closed before any auto-apply work exists,
- and it avoided broadening the raw persisted replay-promotion payload shape used by the adversarial tooling.

That is the right shape for this stage because later objective, benchmark, and apply-loop work can all read one bounded evidence-status contract from the snapshot and benchmark surfaces while the raw replay-promotion payload remains the persisted lineage source of truth.

## Shortfalls found

One real tranche-local issue surfaced during closeout review and was fixed immediately inside `SIM-PROTECTED-1`:

1. The first doc/test draft overstated the `GET /admin/replay-promotion` surface by assuming the derived protected-evidence summary was present on the raw persisted payload read.
   - Fix: keep `GET /admin/replay-promotion` stable as the persisted `replay_promotion_v1` payload contract, and make the docs explicit that the controller-facing protected-evidence summary is exposed through `operator_snapshot_v1`, `benchmark_results_v1`, and the `POST /admin/replay-promotion` response.
   - Evidence: [`../../src/admin/replay_promotion_api.rs`](../../src/admin/replay_promotion_api.rs), [`../../docs/api.md`](../../docs/api.md)

No remaining tranche-local shortfall is left open.

## Next step

Proceed to `OPS-OBJECTIVES-3`.
