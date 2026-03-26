# SIM-SCR-FULL-1B Browser And Proxy Capability Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)
- [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)
- [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
- [`../../src/observability/operator_snapshot_non_human.rs`](../../src/observability/operator_snapshot_non_human.rs)
- [`../../scripts/tests/test_scrapling_worker.py`](../../scripts/tests/test_scrapling_worker.py)
- [`../../scripts/tests/test_llm_fulfillment.py`](../../scripts/tests/test_llm_fulfillment.py)
- [`../../scripts/tests/test_llm_runtime_worker.py`](../../scripts/tests/test_llm_runtime_worker.py)
- [`../../docs/plans/2026-03-26-sim-scr-full-spectrum-capability-implementation-plan.md`](../../docs/plans/2026-03-26-sim-scr-full-spectrum-capability-implementation-plan.md)

# Scope Reviewed

This closeout reviewed the implementation slice for:

1. `SIM-SCR-FULL-1B1` dynamic-browser and stealth-browser Scrapling personas plus truthful `automated_browser` ownership.
2. `SIM-SCR-FULL-1B2` explicit request and browser proxy-plan support with local proof.

# What Landed

1. The Scrapling lane now cycles five fulfillment modes instead of the old request-native trio:
   - `crawler`
   - `bulk_scraper`
   - `browser_automation`
   - `stealth_browser`
   - `http_agent`
2. `automated_browser` now belongs to `scrapling_traffic` rather than remaining parked outside the Scrapling lane.
3. Scrapling-owned browser surfaces now include:
   - `maze_navigation`
   - `js_verification_execution`
   - `browser_automation_detection`
4. The worker imports and uses `DynamicSession` and `StealthySession`, executes browser-native traversal against the PoW and maze paths, and emits owned-surface receipts for those browser interactions.
5. The runtime worker plan now carries the extra browser runtime paths needed for those browser-owned surfaces, including `pow` and `maze_entry`.
6. The repo-owned Scrapling bootstrap now proves the runtime imports the browser-capable Scrapling session classes it depends on.
7. The worker plan now accepts optional `request_proxy_url` and `browser_proxy_url` inputs, and the worker threads those into request-native and browser sessions for truthful local capability proof.
8. Snapshot and coverage contracts now project Scrapling full-spectrum category truth for:
   - `indexing_bot`
   - `ai_scraper_bot`
   - `automated_browser`
   - `http_agent`
9. The later LLM lane tests were updated so `automated_browser` no longer remains duplicated across Scrapling and LLM ownership.

# Review Result

The shipped slice matches the refreshed full-spectrum mandate:

1. attacker-relevant browser and stealth capability is now active in the Scrapling lane rather than parked behind a taxonomy-purity boundary,
2. `automated_browser` is now owned where the runtime can actually prove it,
3. browser-owned Shuma surfaces now have receipt-backed coverage in the same worker, recent-run, and snapshot chain as the earlier request-native personas,
4. and proxy support is now an implemented capability rather than an implicit omission.

The remaining explicit exclusions or deferrals from the refreshed matrix remain truthful:

1. `solve_cloudflare` stays excluded because it does not currently strengthen attacks against Shuma-native defenses,
2. `cdp_url` stays excluded because it changes transport control rather than attacker power,
3. and default `real_chrome` usage stays deferred unless later receipts show the adopted bundled browser or stealth runtime is still insufficient on current Scrapling-owned surfaces.

# Shortfalls Found

Two tranche-local shortfalls surfaced during the closeout pass and were corrected in the same slice:

1. operator and testing docs were still describing Scrapling as request-native only even after the runtime and receipts moved to full-spectrum coverage,
2. and the blocked roadmap still carried a separate browser parking-lot item that implied `automated_browser` ownership was undecided.

Those documentation and roadmap contradictions were corrected as part of this closeout. The remaining open work is `SIM-SCR-FULL-1C`, which is a later proof-surfacing and operator-evidence tranche rather than an unlanded part of `1B`.

# Verification

- `make test-adversary-sim-scrapling-category-fit`
- `make test-adversary-sim-scrapling-browser-capability`
- `make test-adversary-sim-scrapling-proxy-capability`
- `make test-adversary-sim-scrapling-coverage-receipts`
- `make test-adversarial-coverage-receipts`
- `make test-adversarial-llm-fit`
- `make test-adversarial-llm-runtime-dispatch`
- `git diff --check`

# Operational Note

This slice proves proxy capability locally, but it does not yet claim live distributed-origin effectiveness. Any later claim that proxy-backed Scrapling materially changed geo, IP, ban, or rate outcomes must be backed by a separate live proof ring rather than inferred from this local contract.
