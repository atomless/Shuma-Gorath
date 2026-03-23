# SIM-SCR-FIT-2 Request Personas Post-Implementation Review

Date: 2026-03-23
Status: Completed

Related context:

- [`2026-03-23-scrapling-non-human-category-capability-review.md`](2026-03-23-scrapling-non-human-category-capability-review.md)
- [`2026-03-23-sim-scr-fit-1-request-native-category-ownership-post-implementation-review.md`](2026-03-23-sim-scr-fit-1-request-native-category-ownership-post-implementation-review.md)
- [`../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)

# Delivered

`SIM-SCR-FIT-2` now implements the bounded request-native Scrapling personas that `SIM-SCR-FIT-1` only named:

1. `crawler`
   - keeps the existing bounded spider traversal flow
2. `bulk_scraper`
   - performs direct pagination and detail retrieval inside the existing scope fence
3. `http_agent`
   - performs bounded direct request traffic with method mix, cookies, JSON bodies, and in-scope redirect follow-up

The worker now validates `fulfillment_mode` plus `category_targets`, routes each mode through the appropriate executor, and returns typed worker results with `fulfillment_mode` included:

1. [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
2. [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
3. [`../../src/admin/api.rs`](../../src/admin/api.rs)

The focused tests now prove those persona differences directly:

1. bulk-scraper pagination/detail retrieval
2. http-agent method mix and redirect follow-up
3. lane-contract acceptance of the request-native headers these personas use

# Verification

The focused proof path for this tranche passed:

1. `make test-adversary-sim-scrapling-worker`
2. `make test-adversarial-lane-contract`
3. `make test-adversary-sim-scrapling-category-fit`
4. `git diff --check`

# Review Outcome

One real shortfall surfaced during closeout: the Rust worker-result fixture tests initially missed the new `fulfillment_mode` field in one manual result payload, which caused the focused worker and category-fit gates to fail at compile time instead of exercising the intended behavior.

That was corrected immediately in the same tranche by updating the manual result payloads in [`../../src/admin/api.rs`](../../src/admin/api.rs) to mirror the new schema exactly.

No tranche-local shortfall remains open after that correction.
