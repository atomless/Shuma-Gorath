# Scrapling Request-Native Category Fulfillment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expand the Scrapling lane from indexing-only behavior to the full set of request-native non-human categories it can credibly fulfill on its own, while keeping browser-like and agentic categories outside Scrapling ownership until they are separately proven.

**Architecture:** Keep the current shared-host Scrapling lane as a bounded worker behind the existing supervisor, scope, seed, and telemetry contracts. Extend that worker with typed request personas that map to `indexing_bot`, `ai_scraper_bot`, and `http_agent`, then prove the resulting behavior through the same canonical taxonomy, classification, and coverage receipts Shuma already uses for tuning gates. Do not widen Scrapling's contract to browser-agent or delegated-agent behavior in this plan.

**Tech Stack:** Rust adversary-sim control plane, Python Scrapling worker, repo-owned `.venv-scrapling` runtime, canonical taxonomy and coverage contracts, Makefile verification.

---

## Guardrails

1. Do not reassign `automated_browser`, `browser_agent`, or `agent_on_behalf_of_human` to Scrapling in this plan.
2. Do not claim raw request volume alone proves category fulfillment; the classifier and coverage receipts stay authoritative.
3. Keep the existing shared-host scope fence, seed contract, and signed sim telemetry intact.
4. Do not introduce browser-runtime dependencies into the current Scrapling lane as part of this request-native expansion.
5. Keep Monitoring work downstream of these settled lane semantics.

## Task 0: Focused Verification Prep

**Files:**
- Modify: `Makefile`
- Modify: `docs/testing.md`
- Modify: `scripts/tests/test_adversary_sim_make_targets.py`

**Work:**
1. Add a truthful focused target for Scrapling request-native fulfillment contract work, for example `test-adversary-sim-scrapling-category-fit`.
2. Add or refine a truthful focused target for the expanded Scrapling worker persona coverage, for example `test-adversary-sim-scrapling-worker`.
3. Add or refine a truthful focused target for the receipt-backed coverage follow-on, for example `test-adversarial-coverage-receipts`.
4. Add selector-truth regression coverage for the new category-fit target so later target drift fails fast.

**Acceptance criteria:**
1. Each request-native Scrapling slice has a narrow `make` proof path.
2. The new focused target name is enforced by a source-contract test rather than only convention.

## Task 1: `SIM-SCR-FIT-1`

**Files:**
- Modify: `src/observability/non_human_lane_fulfillment.rs`
- Modify: `src/admin/adversary_sim_worker_plan.rs`
- Modify: `src/admin/adversary_sim_lane_runtime.rs`
- Modify: `scripts/tests/adversarial/coverage_contract.v2.json`
- Modify: `scripts/tests/test_adversarial_coverage_contract.py`
- Modify: `docs/adversarial-operator-guide.md`

**Work:**
1. Freeze Scrapling's near-term category ownership to:
   - `indexing_bot`
   - `ai_scraper_bot`
   - `http_agent`
2. Keep `automated_browser`, `browser_agent`, and `agent_on_behalf_of_human` assigned away from Scrapling.
3. Extend the Scrapling worker plan with a typed fulfillment or persona field instead of relying only on broad lane names.
4. Make the coverage fixture and operator docs reflect the new bounded ownership.

**Acceptance criteria:**
1. Shuma can say, in machine-readable form, which taxonomy categories Scrapling is intended to fulfill now.
2. The worker plan can ask the Scrapling lane for a specific bounded request-native persona.

**Verification:**
1. `make test-adversary-sim-scrapling-category-fit`
2. `make test-adversarial-coverage-contract`
3. `git diff --check`

## Task 2: `SIM-SCR-FIT-2`

**Files:**
- Modify: `scripts/supervisor/scrapling_worker.py`
- Create: `scripts/supervisor/scrapling_request_personas.py`
- Modify: `scripts/tests/test_scrapling_worker.py`
- Modify: `scripts/tests/test_adversarial_lane_contract.py`
- Modify: `src/admin/adversary_sim.rs`
- Modify: `src/admin/adversary_sim_lane_runtime.rs`
- Modify: `src/admin/adversary_sim_worker_plan.rs`

**Work:**
1. Introduce typed request-native Scrapling personas:
   - `crawler` for `indexing_bot`
   - `bulk_scraper` for `ai_scraper_bot`
   - `http_agent` for `http_agent`
2. Keep the current spider traversal as the crawler persona.
3. Add a bounded bulk-scraper persona that emphasizes breadth of content retrieval and pagination-oriented discovery without pretending to be a browser agent.
4. Add a bounded direct request persona that exercises supported request-native behaviors such as method mix, cookies, headers, request bodies, redirects, and proxy-ready shape within Scrapling's actual capability surface.
5. Keep the worker's outputs typed and bounded so later classification receipts can distinguish which persona generated which requests.

**Acceptance criteria:**
1. The Scrapling lane can generate materially different request-native traffic shapes on purpose.
2. Those shapes stay inside the current shared-host worker boundary and do not depend on browser automation support.

**Verification:**
1. `make test-adversary-sim-scrapling-worker`
2. `make test-adversary-sim-scrapling-category-fit`
3. `make test-adversarial-lane-contract`
4. `git diff --check`

## Task 3: `SIM-SCR-COVER-2`

**Files:**
- Modify: `src/observability/non_human_coverage.rs`
- Modify: `src/observability/non_human_lane_fulfillment.rs`
- Modify: `src/observability/operator_snapshot_non_human.rs`
- Modify: `src/observability/benchmark_results.rs`
- Modify: `scripts/tests/adversarial/coverage_contract.v2.json`
- Modify: `scripts/tests/test_adversarial_coverage_contract.py`
- Modify: `scripts/tests/test_scrapling_worker.py`

**Work:**
1. Extend receipt-backed coverage so Scrapling can be proven for:
   - `indexing_bot`
   - `ai_scraper_bot`
   - `http_agent`
2. Keep category coverage based on classification receipts, not only worker-plan intent.
3. Make partial or unavailable Scrapling category coverage visible to the same benchmark and snapshot surfaces that later Monitoring will project.
4. Keep later tuning blockers fail-closed if the new request-native categories are claimed but not yet actually covered.

**Acceptance criteria:**
1. Shuma can prove which request-native categories Scrapling covers today.
2. Monitoring and later tuning surfaces can consume that truth without inventing new lane-local semantics.

**Verification:**
1. `make test-adversarial-coverage-receipts`
2. `make test-operator-snapshot-foundation`
3. `make test-benchmark-results-contract`
4. `git diff --check`

## Task 4: Docs And Operator Truth

**Files:**
- Modify: `docs/adversarial-operator-guide.md`
- Modify: `docs/testing.md`
- Modify: `docs/current-system-architecture.md`
- Modify: `docs/api.md`

**Work:**
1. Document Scrapling's expanded request-native role clearly.
2. State explicitly which categories are still not Scrapling-owned.
3. Update architecture and operator docs so Monitoring and later Tuning are not designed around stale indexing-only assumptions.

**Acceptance criteria:**
1. Operators and contributors can see what Scrapling does and does not own.
2. The docs stay aligned with the machine-readable lane-fulfillment contract.

## Out Of Scope And Blocked Follow-On

### `SIM-SCR-BROWSER-1`

Do not implement browser-like Scrapling fulfillment in this plan.

That follow-on should stay blocked until:

1. request-native Scrapling expansion is landed and proven,
2. a truthful shared-host runtime and deploy contract exists for browser dependencies,
3. and `automated_browser` can be covered without collapsing into `browser_agent` or `agent_on_behalf_of_human`.

## Exit Criteria

This plan is complete when:

1. Scrapling's truthful near-term ownership is frozen to request-native categories,
2. the Scrapling worker can generate bounded request personas for those categories,
3. the canonical taxonomy and coverage receipts prove those categories rather than only intending them,
4. and Monitoring can later project those settled semantics instead of the older indexing-only story.
