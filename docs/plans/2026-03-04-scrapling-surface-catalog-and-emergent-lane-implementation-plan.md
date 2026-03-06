# Scrapling Surface Catalog and Emergent Lane Implementation Plan

Date: 2026-03-04
Status: Proposed (implementation-ready)

Related:
- [`docs/adr/0010-adversary-sim-autonomous-heartbeat.md`](../adr/0010-adversary-sim-autonomous-heartbeat.md)
- [`docs/adr/0005-adversarial-lane-coexistence-policy.md`](../adr/0005-adversarial-lane-coexistence-policy.md)
- [`docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)
- [`scripts/tests/adversarial/hybrid_lane_contract.v1.json`](../../scripts/tests/adversarial/hybrid_lane_contract.v1.json)
- [`docs/adversarial-operator-guide.md`](../adversarial-operator-guide.md)
- [`todos/blocked-todo.md`](../../todos/blocked-todo.md) (`SIM-SCR-LANE-1`, `SIM-LLM-1`, `SIM-DEPLOY-2`, `SIM-BREACH-REPLAY-1`)

## Objective

Implement a hosted-site public-surface simulation model with:

1. A code-truth baseline that accurately reflects current runtime behavior today (toggle-only control, deterministic runtime generation, no lane selection control yet).
2. A production-setting shared-host discovery tool that maps host public surface from `robots.txt` and `sitemap.xml` first, then uses bounded Scrapling probing to discover additional in-scope pages.
3. A staged migration to a 3-lane runtime selector under the existing top-level Adversary Sim toggle:
   - `synthetic_traffic` (internal deterministic traffic),
   - `scrapling_traffic` (crawler/scraper traffic),
   - `bot_red_team` (LLM traffic, initially disabled/placeholder).
4. A strict boundary where deterministic oracle authority (CI/replay) remains separate from heartbeat runtime-lane scheduling.
5. A manual operator tuning loop (run, inspect breaches, tune, rerun) without automatic replay-promotion in this tranche.

## Decisions Locked In

1. Keep one cadence owner: backend autonomous supervisor heartbeat.
2. Keep global Adversary Sim toggle as the sole on/off control for starting/stopping simulation work.
3. Keep deterministic scripted oracle corpus as release-blocking authority (separate from runtime lane routing).
4. Preserve current runtime behavior as baseline truth until migration slices land.
5. Deliver lane selection as an explicit migration: exactly one selected runtime lane per beat when lane routing is introduced.
6. Target lane contract for migration end-state: `synthetic_traffic`, `scrapling_traffic`, `bot_red_team`.
7. Keep emergent findings advisory in this tranche; manual operator tune-and-rerun remains authoritative.
8. Keep automatic deterministic replay synthesis/promotion explicitly deferred.
9. Enforce scope safety fail-closed: crawler traffic must be confined to operator-approved hosted-site scope.
10. Keep simulation features production-eligible under existing feature gates and runtime budgets.

## Explicit Assumptions

1. Repository is pre-launch; avoid backward-compatibility shims unless explicitly requested.
2. Operators may run simulation in runtime-dev and approved production modes (subject to existing feature gates).
3. Hosted public surface can be discovered from sitemap/robots/crawl/telemetry, but final catalog activation remains operator-approved.

## Verified Current Runtime Baseline (Code-Truth, 2026-03-04)

1. Dashboard currently exposes a top-level Adversary Sim toggle only; there is no lane radio group yet.
2. `POST /admin/adversary-sim/control` currently accepts only `enabled` (plus optional `reason`) and does not accept a lane selection field.
3. Runtime status currently reports lane-phase labels under `lanes.deterministic` and `lanes.containerized` plus `active_lane_count`; it does not expose `desired_lane`/`active_lane`.
4. Runtime generation path is currently deterministic internal generation (`run_internal_generation_tick`) and not lane-routed by a selected lane enum.
5. This plan treats those facts as source-of-truth baseline and sequences all lane-selector work as forward migration, not as already implemented behavior.

## Non-goals (This Tranche)

1. Implementing full LLM red-team runtime lane (`SIM-LLM-1` remains follow-up).
2. Implementing automatic replay-confirmation synthesis and blocking promotion for emergent findings.
3. Redesigning monitoring transport architecture.

## Architecture Decision Traceability

| Decision | Enforcement in plan | Verification slice(s) |
|---|---|---|
| One cadence owner | Supervisor heartbeat is the only execution scheduler; dashboard polling is read-only | `SIM-SCR-1`, `SIM-SCR-6`, `SIM-SCR-7` |
| Global toggle owns run state | `POST /admin/adversary-sim/control` remains the single write path for enable/disable; lane selection is added via migration slices | `SIM-SCR-1`, `SIM-SCR-7` |
| Exclusive lane selection (target state) | Target lane enum/routing guarantee one active lane per beat when selector migration lands | `SIM-SCR-1`, `SIM-SCR-6`, `SIM-SCR-7` |
| Three-lane contract (target state) | Target active lane values are only `synthetic_traffic`, `scrapling_traffic`, or `bot_red_team` | `SIM-SCR-0`, `SIM-SCR-1`, `SIM-SCR-7` |
| Switches apply immediately at beat boundary | Router updates active lane at next beat and halts prior lane dispatch | `SIM-SCR-1`, `SIM-SCR-6` |
| Deterministic oracle is release-blocking authority | Catalog-backed scripted oracle stays the contractual gate | `SIM-SCR-4`, `SIM-SCR-5` |
| Internal deterministic runtime traffic remains baseline | Existing deterministic generation remains baseline until lane migration cutover | `SIM-SCR-5`, `SIM-SCR-7` |
| Scrapling lane is emergent/advisory (target state) | `scrapling_traffic` findings do not auto-promote to blocking in this tranche | `SIM-SCR-6`, `SIM-SCR-8`, `SIM-SCR-9` |
| Manual tune-and-rerun loop is authoritative | Operator workflow documents manual breach review and rerun loop | `SIM-SCR-8` |
| Hosted-site confinement is mandatory | Fail-closed scope policy + per-request/redirect enforcement | `SIM-SCR-2`, `SIM-SCR-3`, `SIM-SCR-6` |
| Request cancellation must not flip global connection state | Lane request failures are classified; `cancelled` is forbidden from mutating global connection state | `SIM-SCR-0`, `SIM-SCR-6`, dashboard regression tests |
| Production runtime viability | Lane budgets and defaults cap resource usage; OFF remains inert | `SIM-SCR-6`, `SIM-SCR-8` |

## Scrapling Capability Profile (Docs-Sourced)

This plan is grounded in the full Scrapling docs sitemap on 2026-03-04 (see Appendix A), including fetchers, spiders, sessions, blocking/proxy behavior, CLI, tutorials, API reference, and development docs.

### Capabilities We Will Use in This Tranche

1. Spider framework core: scheduler, deduplication, async callbacks, `start()`/`stream()`.
2. Hosted-scope confinement primitives: `allowed_domains` and offsite filtering counters (`offsite_requests_count`).
3. Crawl-pressure controls: `concurrent_requests`, `concurrent_requests_per_domain`, `download_delay`.
4. Session routing with `sid` and default `FetcherSession` for low-cost HTTP-first discovery.
5. Built-in blocked-request pipeline with custom `is_blocked()` and `retry_blocked_request()` hooks.
6. Optional proxy rotation (`ProxyRotator`) behind explicit operator config.
7. Checkpointing with `crawldir` for pause/resume and per-beat incremental progress.

### Capabilities Explicitly Deferred (Not This Tranche)

1. Automatic replay synthesis/promotion from emergent findings.
2. Full autonomous LLM lane behavior (`bot_red_team`) beyond placeholder routing and disabled UI state.
3. Stealth/browser-heavy default crawling (`AsyncDynamicSession`/`AsyncStealthySession`) as baseline behavior.
4. MCP server integration and AI-driven extraction flows.
5. Adaptive selector persistence as a required dependency for the emergent lane.

### What Scrapling Will Do For Us Now

1. Discover and traverse in-scope hosted public pages under strict budgets.
2. Produce advisory emergent findings with provenance and crawl diagnostics.
3. Feed manual operator tuning loops (inspect breach, tune defenses, rerun sim).

### What Scrapling Will Not Do For Us Now

1. It will not own global dashboard/backend connection state.
2. It will not bypass scope policy or permit offsite crawling.
3. It will not automatically change deterministic blocking catalogs without operator action.

## Architecture Summary

### Current Baseline (Implemented Today)

1. `POST /admin/adversary-sim/control` controls enabled/disabled state only (no lane field yet).
2. `GET /admin/adversary-sim/status` reports phase, counts, and lane-phase labels (`deterministic`, `containerized`) plus generation diagnostics.
3. Dashboard exposes a top-level Adversary Sim toggle only.
4. Supervisor heartbeat cadence is backend-owned; UI polling remains read-only.
5. Runtime generation is deterministic internal traffic; lane-selector routing is not yet implemented.

### Target End-State (Migration Goal)

1. Control API accepts explicit runtime lane selection with strict enum validation.
2. Status API reports target lane contract fields (`desired_lane`, `active_lane`, switch metadata) and per-lane diagnostics.
3. Dashboard keeps the top-level toggle and adds a 3-option radio group:
   - `Synthetic Traffic (Internal)`,
   - `Scrapling Crawler/Scraper`,
   - `Bot Red Team (LLM)` (disabled until implementation).
4. Exactly one selected lane executes per heartbeat beat boundary.

### Data Plane

1. Discovery inventory artifact: merged raw URL evidence from sitemap/robots/crawl/telemetry.
2. Compiled deterministic catalog artifact: canonicalized, templated, scoped route set consumed by deterministic oracle execution and runtime synthetic baselines.
3. Sim telemetry tags include run ID, lane label, tick ID, and worker provenance for auditability.

### Governance

1. Deterministic oracle (separate from runtime lane scheduler) remains blocking authority.
2. Internal runtime synthetic traffic remains deterministic baseline behavior.
3. Scrapling runtime traffic remains discovery/advisory in this tranche.
4. Manual operator loop is authoritative for mitigation validation in this tranche.

## Naming and Slice Mapping

1. Plan slices `SIM-SCR-2`, `SIM-SCR-3`, and `SIM-SCR-4` map to roadmap tranche `SIM-SH-SURFACE-1` (shared-host discovery first gate).
2. Plan slices `SIM-SCR-0`, `SIM-SCR-1`, `SIM-SCR-6`, `SIM-SCR-7`, and `SIM-SCR-8` map to roadmap tranche `SIM-SCR-LANE-1` (runtime lane integration after gate).
3. Plan slice `SIM-SCR-9` maps to roadmap tranche `SIM-BREACH-REPLAY-1` and later `SIM-LLM-1` follow-up.
4. Where older notes reference `SIM-SCR-6`, treat it as the runtime-lane implementation slice now sequenced under `SIM-SCR-LANE-1`.

## Milestone-First Implementation Order (Explicit)

### Milestone 1 (Current Priority): Shared-Host Public-Surface Discovery Tool

Goal:

1. Build an operator tool that discovers the host's public surface on a shared single-host deployment.
2. Discovery order must be:
   - `robots.txt` + `sitemap.xml` ingest and normalize first,
   - then bounded Scrapling probe augmentation for additional in-scope findings.
3. Prove it on a real shared host and produce inventory evidence artifacts before any non-deterministic lane implementation.

Slices in this milestone:

1. `SIM-SCR-2` Hosted-scope policy model and validation.
2. `SIM-SCR-3` Discovery inventory builder (robots/sitemap first, Scrapling augmentation second).
3. `SIM-SCR-4` Inventory-to-catalog compiler.

### Milestone 2: Deterministic Oracle and Synthetic Seed Adoption

Goal:

1. Consume the compiled hosted-surface catalog in deterministic oracle execution and synthetic-lane seeding.

Slices in this milestone:

1. `SIM-SCR-5` Deterministic oracle/baseline migration from fixed sim paths to catalog.

### Milestone 3: Lane Selector Migration + Scrapling Sim Lane

Goal:

1. Add explicit lane-selection contract/control/UI on top of the existing toggle-only baseline.
2. Introduce Scrapling as an autonomous heartbeat-driven non-deterministic runtime lane.

Slices in this milestone:

1. `SIM-SCR-0` Contracts and observability scaffolding (lane/state diagnostics).
2. `SIM-SCR-1` Control/state model for lane selection and switch semantics.
3. `SIM-SCR-6` Scrapling lane worker integration and bounded per-beat execution.
4. `SIM-SCR-7` Dashboard lane controls and diagnostics.
5. `SIM-SCR-8` Operator workflow docs, Make targets, and rollout/rollback playbook.
6. `SIM-SCR-9` Roadmap capture for replay automation and LLM lane follow-up.

Execution gate (non-negotiable):

1. `SIM-SCR-6` and later non-deterministic lane slices must not start until Milestone 1 has real shared-host evidence artifacts and acceptance checks passed.

## Execution Order (Slice-by-Slice)

1. `SIM-SCR-2` Hosted-scope policy model and validation.
2. `SIM-SCR-3` Public-surface discovery inventory builder (robots/sitemap first, Scrapling probe second).
3. `SIM-SCR-4` Inventory-to-catalog compiler for deterministic execution.
4. `SIM-SCR-5` Deterministic oracle/baseline migration from fixed sim paths to catalog.
5. `SIM-SCR-0` Contracts and observability scaffolding for lane diagnostics.
6. `SIM-SCR-1` Control/state model for lane selection.
7. `SIM-SCR-6` Scrapling non-deterministic lane worker integration.
8. `SIM-SCR-7` Dashboard lane controls and diagnostics.
9. `SIM-SCR-8` Operator workflow and Make targets hardening.
10. `SIM-SCR-9` Deferred roadmap capture.

## Slice Details

### SIM-SCR-0: Contracts and Observability Scaffolding

Scope:

1. Add a versioned migration contract that preserves current status fields while introducing new lane-schema fields.
2. Add diagnostics counters/events for lane routing and scope enforcement outcomes.
3. Add diagnostics schema for request failure classes: `cancelled`, `timeout`, `transport`, `http`.
4. Add tests that fail on missing/invalid lane payload fields and missing failure-class counters.

Primary touchpoints:

- `src/admin/adversary_sim.rs`
- `src/admin/api.rs`
- `scripts/tests/check_sim2_governance_contract.py`
- `scripts/tests/test_sim2_governance_contract.py`

Acceptance criteria:

1. Status payload keeps current baseline fields and also includes migration fields `desired_lane`, `active_lane`, `lane_switch_seq`, `last_lane_switch_at`, `last_lane_switch_reason`.
2. Diagnostics include per-lane beat attempts/successes/failures.
3. Diagnostics include failure-class counters and last-seen timestamps.
4. Contract explicitly forbids `cancelled` class from mutating global connection state.
5. No behavior change yet; tests validate new contract shape only.

### SIM-SCR-1: Control/State Model for Lane Selection

Scope:

1. Extend control state with explicit desired/active lane fields while retaining compatibility with current toggle-only payload.
2. Extend control API payload to accept lane changes with strict validation (`synthetic_traffic`, `scrapling_traffic`, `bot_red_team`).
3. Keep default lane as `synthetic_traffic` when lane is omitted.
4. Preserve existing `enabled` toggle semantics as first-class control behavior.

Primary touchpoints:

- `src/admin/adversary_sim.rs`
- `src/admin/adversary_sim_control.rs`
- `src/admin/api.rs`
- `src/lib.rs` (route wiring only)

Acceptance criteria:

1. Control endpoint continues to accept existing toggle payload and additionally enforces lane enum validation when lane is supplied.
2. Status endpoint reflects lane migration fields consistently without breaking existing status consumers.
3. Lane selection is idempotent and auditable through existing control audit logs.

### SIM-SCR-2: Hosted-Scope Policy Model and Validation

Scope:

1. Introduce explicit scope policy contract for crawler/catalog generation.
2. Enforce fail-closed validation for host/scheme/path constraints (`https` only, host allowlist only, no IP-literal URLs).
3. Enforce redirect revalidation (redirect target must pass the same scope gate).
4. Reject privileged/internal paths from scope.

Primary touchpoints:

- `config/defaults.env`
- `src/config/mod.rs`
- `scripts/config_seed.sh`
- `scripts/bootstrap/setup.sh`
- `dashboard/src/lib/domain/config-schema.js`
- `src/admin/api.rs` (config validation and status exposure)

Policy fields (initial proposal):

1. `adversary_surface_allowed_hosts` (required list)
2. `adversary_surface_allowed_path_prefixes` (required list)
3. `adversary_surface_denied_path_prefixes` (required list, default includes `/admin`, `/internal`, `/dashboard`, `/session`, `/auth`, `/login`)
4. `adversary_surface_require_https` (required bool, default `true`)
5. `adversary_surface_deny_ip_literals` (required bool, default `true`)
6. `adversary_surface_allow_query_patterns` (optional)
7. `adversary_surface_max_redirect_hops` (bounded)

Acceptance criteria:

1. Invalid scope policy is rejected with explicit errors.
2. Config/docs/advanced JSON path parity is maintained.
3. URL gate rejects non-HTTPS, IP-literal hosts, out-of-scope redirects, and denied path families with explicit reason codes.
4. Status exposes effective scope policy summary (redacted as needed).
5. Policy gate is fail-closed and has no permissive fallback path.

### SIM-SCR-3: Discovery Inventory Builder

Scope:

1. Build inventory seed from `robots.txt` and `sitemap.xml` first (including sitemap index expansion) before any probing.
2. Merge/canonicalize seed URLs and enforce scope policy prior to probing.
3. Use Scrapling spider as bounded probe augmenter only after seed ingest completes, with:
   - `allowed_domains` sourced from approved scope policy,
   - explicit concurrency and delay caps,
   - offsite filtering stats collection,
   - HTTP-first session profile (`FetcherSession`) as default.
4. Optionally merge monitoring telemetry URL evidence after seed + probe steps.
5. Persist inventory artifact with provenance and evidence metadata.
6. Expose inventory generation as operator workflow for shared-host deployments (Make target first, API follow-up optional).
7. Execute and evidence at least one real shared-host discovery run before lane work starts.

Primary touchpoints:

- `scripts/tests/adversarial_surface_inventory.py` (new)
- `scripts/tests/test_adversarial_surface_inventory.py` (new)
- `Makefile` (new target wiring)
- `docs/adversarial-operator-guide.md`

Artifact proposal:

- `scripts/tests/adversarial/public_surface_inventory.json`

Acceptance criteria:

1. Discovery execution order is enforced and test-covered: `robots/sitemap -> normalize/scope -> Scrapling probe`.
2. Inventory contains source provenance per URL (`sitemap`, `robots`, `crawl`, `telemetry`).
3. `crawl` provenance entries are additive only (they must not bypass scope or override seed exclusions).
4. Out-of-scope URLs are excluded and recorded as structured rejection reasons.
5. Build command is deterministic when input sources and seed are fixed.
6. Inventory run captures crawl stats (`requests_count`, `offsite_requests_count`, `response_status_count`, `response_bytes`).
7. Inventory command fails closed when scope policy is absent/invalid.
8. Real shared-host proof run outputs versioned artifact + summary report under `scripts/tests/adversarial/` and is linked from operator docs.

### SIM-SCR-4: Inventory-to-Catalog Compiler

Scope:

1. Canonicalize URLs and template dynamic segments/query values.
2. Compile normalized routes into deterministic execution specs.
3. Emit coverage and rejection diagnostics.

Primary touchpoints:

- `scripts/tests/adversarial_surface_catalog_compile.py` (new)
- `scripts/tests/test_adversarial_surface_catalog_compile.py` (new)
- `scripts/tests/adversarial/public_surface_catalog.schema.json` (new)
- `docs/adversarial-operator-guide.md`

Artifact proposal:

- `scripts/tests/adversarial/public_surface_catalog.v1.json`

Acceptance criteria:

1. Catalog includes stable `route_id`, `path_template`, `query_templates`, `priority_weight`, `risk_tags`.
2. Catalog includes `catalog_hash` and compile-time scope policy digest.
3. Compile output reports accepted/rejected counts and reason breakdown.

### SIM-SCR-5: Deterministic Oracle/Baseline Migration to Catalog Input

Scope:

1. Replace fixed `primary_public_paths` runtime selection with compiled hosted-surface catalog input for deterministic oracle and synthetic-lane seeding.
2. Keep deterministic selection (`run_id + tick + slot`) for reproducibility.
3. Keep existing supplemental defense probes (challenge/pow/tarpit/cdp/rate) while catalog coverage matures.

Primary touchpoints:

- `src/admin/adversary_sim.rs`
- `src/admin/api.rs` (status diagnostics for catalog version/hash)
- `scripts/tests/adversarial/deterministic_attack_corpus.v1.json` (contract update)
- `scripts/tests/test_adversarial_simulation_runner.py`
- `src/admin/api.rs` tests for generation diagnostics

Acceptance criteria:

1. Deterministic oracle reports catalog hash/version in status and telemetry metadata.
2. Coverage summary reports catalog route coverage percentage and uncovered routes.
3. Existing deterministic regression gates remain green.

### SIM-SCR-6: Scrapling Lane Worker Integration

Scope:

1. Start only after Milestone 1 shared-host discovery proof is complete and accepted.
2. Add bounded out-of-process Scrapling worker invoked by supervisor when `active_lane=scrapling_traffic`.
3. Define per-beat worker contract with strict `TickBudget` (`max_requests`, `max_depth`, `max_bytes`, `max_ms`).
4. Persist per-run crawl state (`crawldir`) to support incremental per-beat crawling and safe resume.
5. Enforce scope policy on every request and redirect.
6. Enforce header policy: deny privileged/internal headers (`Authorization`, internal Shuma headers).
7. Enforce egress policy: isolated worker runtime with allowlisted outbound only to approved host `:443` plus DNS.
8. Apply explicit failure classification (`cancelled`, `timeout`, `transport`, `http`) in lane diagnostics.
9. Emit full request provenance (`run_id`, `lane`, `tick_id`, `worker_id`) for every worker-emitted request/event.
10. Keep Scrapling lane session profile HTTP-first (`FetcherSession`), with optional proxy rotation via config.
11. Define explicit degraded-state semantics for worker unavailable/crash/timeout; fail closed and never report silent success.

Primary touchpoints:

- `scripts/supervisor/adversary_sim_supervisor.rs`
- `scripts/supervisor/scrapling_worker.py` (new)
- `scripts/tests/test_adversary_sim_supervisor.py` (new)
- `src/admin/adversary_sim.rs` (lane routing hooks and state)
- `src/admin/api.rs` (lane diagnostics status rendering)

Acceptance criteria:

1. `scrapling_traffic` lane executes bounded crawl slices per beat (`max_requests`, `max_depth`, `max_bytes`, `max_ms`).
2. Out-of-scope, non-HTTPS, IP-literal, denied-path, and out-of-scope-redirect targets are blocked with explicit diagnostics counters.
3. Lane switch between any lanes (`synthetic_traffic`, `scrapling_traffic`, `bot_red_team`) stops prior lane activity on next beat with zero concurrent-lane overlap.
4. Request failures from runtime lane traffic do not mutate dashboard global backend connection state.
5. `cancelled` failures are tracked but never treated as backend disconnect signals.
6. Lane diagnostics expose crawl-pressure and block data (`blocked_requests_count`, `offsite_requests_count`, per-status counts, proxy-use counts).
7. Worker crash/timeout/unavailable yields explicit degraded diagnostics and fail-closed behavior (no silent success).
8. Privileged headers are stripped/denied and cannot reach target host from worker requests.

### SIM-SCR-7: Dashboard Lane Controls and Diagnostics

Scope:

1. Keep global Adversary Sim toggle (existing behavior).
2. Add exclusive lane radio group under the toggle (new behavior):
   - `Synthetic Traffic (Internal)`
   - `Scrapling Crawler/Scraper`
   - `Bot Red Team (LLM)` disabled/annotated as follow-up
3. Show active vs desired lane and last switch reason/timestamp.
4. Keep monitoring refresh path read-only; no frontend polling path may drive lane execution.

Primary touchpoints:

- `dashboard/src/routes/+page.svelte`
- `dashboard/src/lib/domain/api-client.js`
- `dashboard/src/lib/domain/config-schema.js`
- `e2e/dashboard.modules.unit.test.js`
- `e2e/dashboard.smoke.spec.js`
- `docs/dashboard.md`

Acceptance criteria:

1. Lane selection persists via control API and survives remount/navigation.
2. UI always renders backend status truth (no optimistic lane drift).
3. Existing adversary sim toggle tests remain green with added lane cases.
4. Dashboard polling cadence does not create additional simulation scheduler loops.

### SIM-SCR-8: Operator Workflow and Make Targets

Scope:

1. Add make targets for inventory and catalog workflows.
2. Document manual tuning loop without automatic replay synthesis.
3. Document runbook for scope policy setup and safety checks.

Primary touchpoints:

- `Makefile`
- `docs/adversarial-operator-guide.md`
- `docs/testing.md`
- `docs/api.md`

Proposed targets:

1. `make adversary-surface-inventory`
2. `make adversary-surface-catalog-compile`
3. `make test-adversary-surface-catalog`

Acceptance criteria:

1. Operator can bootstrap catalog in <= 3 commands from fresh setup.
2. Docs specify mandatory scope-policy checks before enabling Scrapling lane execution.
3. Verification paths use Make targets only.

### SIM-SCR-9: Roadmap Capture (Deferred Work)

Scope:

1. Capture deferred automatic replay pipeline from emergent findings.
2. Capture deferred LLM lane implementation details.
3. Ensure deferred items are explicit TODO entries and non-blocking for this tranche.

Primary touchpoints:

- `todos/todo.md`
- `docs/plans/` follow-up plan stub(s)

Acceptance criteria:

1. Deferred scope is explicit and linked from this plan.
2. No implicit claims that replay automation is complete.

## Data Contracts (Proposed)

### Public Surface Inventory (`public-surface-inventory.v1`)

Required fields:

1. `schema_version`
2. `generated_at_unix`
3. `target_base_url`
4. `scope_policy_digest`
5. `urls[]` with `url`, `source`, `seen_count`, `last_seen_at`, `status_family`, `content_type`
6. `rejections[]` with `url`, `reason_code`, `source`

### Public Surface Catalog (`public-surface-catalog.v1`)

Required fields:

1. `schema_version`
2. `generated_at_unix`
3. `catalog_id`
4. `catalog_hash`
5. `scope_policy_digest`
6. `routes[]` with `route_id`, `method_set`, `path_template`, `query_templates`, `priority_weight`, `risk_tags`
7. `compile_stats` with accepted/rejected totals and reason breakdown

### Synthetic Lane Tick Summary (`synthetic-lane-tick-summary.v1`)

Required fields:

1. `schema_version`
2. `run_id`
3. `lane` (`synthetic_traffic`, `scrapling_traffic`, or `bot_red_team`)
4. `tick_id`
5. `worker_id`
6. `tick_budget` (`max_requests`, `max_depth`, `max_bytes`, `max_ms`)
7. `request_counts` (`attempted`, `completed`, `failed`, `blocked_policy`)
8. `failure_classes` (`cancelled`, `timeout`, `transport`, `http`)
9. `status` (`ok`, `degraded`, `failed_closed`)
10. `degraded_reason` (required when `status != ok`)

## Verification Gates (Non-negotiable)

1. Unit: URL/scheme/path/redirect confinement policy (`https` only, no IP-literal, denied paths, redirect revalidation).
2. Integration: worker cannot reach out-of-scope hosts (egress allowlist enforced).
3. Integration: lane switch leaves zero concurrent lane activity.
4. E2E: selected runtime lane requests (`synthetic_traffic` and `scrapling_traffic`) appear in normal monitoring telemetry surfaces.
5. Failure-path: worker crash, worker timeout, and heartbeat loss all fail closed with explicit degraded diagnostics (no silent success).
6. Connection-state ownership: global connection-state writer remains heartbeat-only and independent of lane request churn.

## Verification Strategy (Makefile Canonical)

1. `make test-adversarial-python-unit`
2. `make test-unit`
3. `make test-integration` (with `make dev` running)
4. `make test-dashboard-unit`
5. `make test-dashboard-e2e` (with `make dev` running)
6. `make test-adversarial-fast` (with `make dev` running)
7. `make test`

## Security and Abuse Posture

1. Scope policy defaults must fail closed and must not allow wildcard egress by default.
2. Scrapling worker must never receive admin credentials or internal signing secrets.
3. Out-of-scope and privileged-path attempts must emit explicit policy telemetry.
4. Worker egress must be isolated to approved host `:443` plus DNS only; all other outbound attempts are denied.
5. Worker request builder must deny privileged headers (`Authorization`, internal Shuma control headers).
6. Simulation off-state inertness guarantees remain mandatory.

## Resource and Operational Impact

1. Deterministic catalog generation adds bounded preprocessing time and artifact storage.
2. Scrapling lane introduces extra CPU/network load only while enabled.
3. Lane budgets and per-beat limits are required to cap runtime cost.
4. OFF state must continue to enforce near-zero adversary-sim resource usage.

## Rollback Strategy

1. Force adversary simulation OFF and verify no active lane execution.
2. Set default lane back to `synthetic_traffic` and disable `scrapling_traffic` and `bot_red_team` execution via config gate.
3. Keep catalog artifacts for audit but stop lane routing to Scrapling worker.
4. Revert lane-control UI to toggle-only if contract regression is detected.

## Definition of Done

1. Shared-host discovery tool runs in production setting with enforced order (`robots/sitemap` first, Scrapling probe second) and emits evidence artifacts from at least one real shared-host run.
2. Deterministic oracle executes against compiled hosted-surface catalog with explicit coverage diagnostics.
3. Runtime `scrapling_traffic` lane executes bounded, hosted-site-confined discovery traffic only.
4. Dashboard exposes stable, backend-authored lane state and switch diagnostics.
5. Request failure diagnostics classify `cancelled`, `timeout`, `transport`, and `http`; `cancelled` never mutates global connection state.
6. Operator can run manual tune-and-rerun workflow without replay automation.
7. Deferred replay automation and LLM lane work are explicitly documented as follow-up scope.
8. Documentation explicitly records Scrapling capabilities used now vs deferred, with source links.
9. Lane contract is enforced to `synthetic_traffic | scrapling_traffic | bot_red_team` with zero concurrent lane activity per beat.
10. Worker failure states (`crash`, `timeout`, `unavailable`) are explicit and fail closed.

## Appendix A: Scrapling Docs Reviewed (Sitemap Complete on 2026-03-04)

Core docs:

1. [Index](https://scrapling.readthedocs.io/en/latest/index.html)
2. [Overview](https://scrapling.readthedocs.io/en/latest/overview.html)
3. [Benchmarks](https://scrapling.readthedocs.io/en/latest/benchmarks.html)

Parsing:

1. [Selection methods](https://scrapling.readthedocs.io/en/latest/parsing/selection.html)
2. [Main classes](https://scrapling.readthedocs.io/en/latest/parsing/main_classes.html)
3. [Adaptive parsing](https://scrapling.readthedocs.io/en/latest/parsing/adaptive.html)

Fetching:

1. [Fetchers basics](https://scrapling.readthedocs.io/en/latest/fetching/choosing.html)
2. [HTTP requests](https://scrapling.readthedocs.io/en/latest/fetching/static.html)
3. [Dynamic fetcher](https://scrapling.readthedocs.io/en/latest/fetching/dynamic.html)
4. [Stealthy fetcher](https://scrapling.readthedocs.io/en/latest/fetching/stealthy.html)

Spiders:

1. [Architecture](https://scrapling.readthedocs.io/en/latest/spiders/architecture.html)
2. [Getting started](https://scrapling.readthedocs.io/en/latest/spiders/getting-started.html)
3. [Requests and responses](https://scrapling.readthedocs.io/en/latest/spiders/requests-responses.html)
4. [Sessions](https://scrapling.readthedocs.io/en/latest/spiders/sessions.html)
5. [Proxy management and blocked handling](https://scrapling.readthedocs.io/en/latest/spiders/proxy-blocking.html)
6. [Advanced spider features](https://scrapling.readthedocs.io/en/latest/spiders/advanced.html)

CLI:

1. [CLI overview](https://scrapling.readthedocs.io/en/latest/cli/overview.html)
2. [Interactive shell](https://scrapling.readthedocs.io/en/latest/cli/interactive-shell.html)
3. [Extract commands](https://scrapling.readthedocs.io/en/latest/cli/extract-commands.html)

AI and tutorials:

1. [MCP server](https://scrapling.readthedocs.io/en/latest/ai/mcp-server.html)
2. [Replacing AI workflows tutorial](https://scrapling.readthedocs.io/en/latest/tutorials/replacing_ai.html)
3. [Migrating from BeautifulSoup tutorial](https://scrapling.readthedocs.io/en/latest/tutorials/migrating_from_beautifulsoup.html)

API reference:

1. [Selector API](https://scrapling.readthedocs.io/en/latest/api-reference/selector.html)
2. [Fetchers API](https://scrapling.readthedocs.io/en/latest/api-reference/fetchers.html)
3. [MCP server API](https://scrapling.readthedocs.io/en/latest/api-reference/mcp-server.html)
4. [Custom types API](https://scrapling.readthedocs.io/en/latest/api-reference/custom-types.html)
5. [Response API](https://scrapling.readthedocs.io/en/latest/api-reference/response.html)
6. [Spiders API](https://scrapling.readthedocs.io/en/latest/api-reference/spiders.html)
7. [Proxy rotation API](https://scrapling.readthedocs.io/en/latest/api-reference/proxy-rotation.html)

Development:

1. [Adaptive storage system](https://scrapling.readthedocs.io/en/latest/development/adaptive_storage_system.html)
2. [Custom types internals](https://scrapling.readthedocs.io/en/latest/development/scrapling_custom_types.html)

Other:

1. [Donate](https://scrapling.readthedocs.io/en/latest/donate.html)
