# SIM-SCR-COVER-2 Request-Native Coverage Post-Implementation Review

Date: 2026-03-23
Status: Completed

Related context:

- [`2026-03-23-scrapling-non-human-category-capability-review.md`](2026-03-23-scrapling-non-human-category-capability-review.md)
- [`2026-03-23-sim-scr-fit-1-request-native-category-ownership-post-implementation-review.md`](2026-03-23-sim-scr-fit-1-request-native-category-ownership-post-implementation-review.md)
- [`2026-03-23-sim-scr-fit-2-request-personas-post-implementation-review.md`](2026-03-23-sim-scr-fit-2-request-personas-post-implementation-review.md)
- [`../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)

# Delivered

`SIM-SCR-COVER-2` now closes the gap between Scrapling persona intent and machine-first coverage proof:

1. real Scrapling requests now emit mode-specific signed sim profiles (`scrapling_runtime_lane.<mode>`),
2. recent-sim hot-read summaries normalize those observed profiles back into bounded `observed_fulfillment_modes` and `observed_category_ids`,
3. non-human classification receipts now use that recent-sim evidence to materialize request-native Scrapling category receipts for:
   - `indexing_bot`
   - `ai_scraper_bot`
   - `http_agent`
4. operator snapshot and benchmark results now project that receipt-backed coverage directly.

The key files are:

1. [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
2. [`../../src/admin/api.rs`](../../src/admin/api.rs)
3. [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
4. [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
5. [`../../src/observability/non_human_coverage.rs`](../../src/observability/non_human_coverage.rs)
6. [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
7. [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)

# Verification

The focused proof path for this tranche passed:

1. `make test-adversarial-coverage-receipts`
2. `make test-operator-snapshot-foundation`
3. `make test-benchmark-results-contract`
4. `make test-adversary-sim-scrapling-worker`
5. `make test-adversary-sim-scrapling-category-fit`
6. `make test-traffic-classification-contract`
7. `git diff --check`

# Review Outcome

One real shortfall surfaced during closeout: the Scrapling worker’s `_signed_headers(...)` helper was returning before it merged persona-specific extra headers, which would have weakened the fidelity of direct request personas and obscured the intended request shape.

That was corrected immediately in the same tranche, and the worker proof gate now verifies the mode-specific signed sim-profile headers on real requests as part of the end-to-end hosted-scope fixture.

No tranche-local shortfall remains open after that correction.
