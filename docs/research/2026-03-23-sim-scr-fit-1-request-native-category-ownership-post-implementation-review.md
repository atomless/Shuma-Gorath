# SIM-SCR-FIT-1 Request-Native Category Ownership Post-Implementation Review

Date: 2026-03-23
Status: Completed

Related context:

- [`2026-03-23-scrapling-non-human-category-capability-review.md`](2026-03-23-scrapling-non-human-category-capability-review.md)
- [`2026-03-23-scrapling-and-verified-identity-sequence-readiness-review.md`](2026-03-23-scrapling-and-verified-identity-sequence-readiness-review.md)
- [`../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)

# Delivered

`SIM-SCR-FIT-1` now freezes Scrapling's near-term ownership to the request-native categories it can credibly own on the current shared-host boundary:

1. `indexing_bot` via `crawler`
2. `ai_scraper_bot` via `bulk_scraper`
3. `http_agent` via `http_agent`

The canonical lane-fulfillment summary now reports those mappings directly in [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs), while `automated_browser`, `browser_agent`, and `agent_on_behalf_of_human` remain outside Scrapling ownership.

The Scrapling worker plan now carries a typed `fulfillment_mode` plus bounded `category_targets` in [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs) and [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs), so the lane can ask for a specific request-native persona before the later worker-behavior tranche implements those personas in depth.

The frozen full-coverage contract and operator guidance now reflect the same ownership:

1. [`../../scripts/tests/adversarial/coverage_contract.v2.json`](../../scripts/tests/adversarial/coverage_contract.v2.json)
2. [`../../scripts/tests/check_adversarial_coverage_contract.py`](../../scripts/tests/check_adversarial_coverage_contract.py)
3. [`../../docs/adversarial-operator-guide.md`](../../docs/adversarial-operator-guide.md)

# Verification

The focused proof path for this tranche passed:

1. `make test-adversary-sim-scrapling-category-fit`
2. `make test-adversarial-coverage-contract`
3. `make test-adversary-sim-scrapling-worker`
4. `git diff --check`

# Review Outcome

One real shortfall surfaced during closeout: the new `test-adversary-sim-scrapling-category-fit` target initially used the wrong qualified Rust selector for the internal beat contract test, so the selector was being filtered out instead of executed.

That was corrected immediately in the same tranche by:

1. fixing the selector in [`../../Makefile`](../../Makefile),
2. tightening the source-contract check in [`../../scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py),
3. and re-running the focused target to confirm the API beat assertion was really executed.

No tranche-local shortfall remains open after that correction.
