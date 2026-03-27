Date: 2026-03-27
Status: Implemented

Related context:

- [`2026-03-27-game-loop-architecture-alignment-gap-review.md`](2026-03-27-game-loop-architecture-alignment-gap-review.md)
- [`2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/operator_snapshot_non_human.rs`](../../src/observability/operator_snapshot_non_human.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)

# What landed

`RSI-GAME-ARCH-1A` is now implemented.

The non-human snapshot path no longer exposes one blended category receipt stream as if it were a single truth.

The split now works like this:

1. `restriction_readiness` and `restriction_receipts`
   - current Shuma-side evidence only,
   - no projected recent-sim category presence,
   - this is the rail benchmark eligibility and controller gating now read.
2. `recognition_evaluation`
   - mixed current plus projected sim-category evidence for evaluation and coverage,
   - explicit `simulator_ground_truth`,
   - this is the rail category posture and later recognition-quality work now read.

# What changed

1. `operator_snapshot_non_human` now materializes:
   - a restriction-grade rail,
   - a recognition-evaluation rail,
   - and a simulator-ground-truth summary.
2. The old projected recent-sim category presence no longer enters restriction readiness.
3. `benchmark_results` now treats:
   - `non_human_classification` as restriction-grade readiness,
   - `non_human_coverage` as recognition-evaluation coverage,
   - and bounded tuning eligibility no longer blocks just because degraded projected sim category receipts exist.
4. `non_human_category_posture` still exists, but it now reads from the recognition-evaluation rail instead of the restriction rail.
5. Verified-identity conflict checks now read restriction receipts rather than the mixed evaluation stream.

# Why this matters

This tranche closes the first cross-cutting architecture gap from the March 27 review.

Before this change, projected sim categories and current Shuma-side receipts shared the same summary lane, which meant the controller could end up looking at evaluation-only degraded evidence while deciding whether it was safe or eligible to act.

Now the repo says something much cleaner:

1. the restriction loop acts only on Shuma-side current evidence,
2. the recognition quest can still compare simulator truth against Shuma inference,
3. and the two rails are explicit instead of being blended through one category summary.

# Verification

The tranche was verified through:

1. `make test-benchmark-results-contract`
2. `make test-traffic-classification-contract`
3. `make test-dashboard-game-loop-accountability`

# Follow-on

The next immediate slice is `RSI-SCORE-2F2`: make the recognition-evaluation rail more useful for categorisation improvement by auditing which categories Shuma can infer from shared-path evidence today, which it still cannot, and how simulator ground truth should be used only to evaluate that inference quality after the fact.
