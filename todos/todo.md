# TODO Roadmap

Last updated: 2026-03-05

This is the active work queue.
`todos/security-review.md` tracks security finding validity and closure status.

## P0 SIM2 Round 4 Stabilization: Monitoring Truthfulness + UX Consistency

Objective: fix post-SIM2 regressions where monitoring appears disabled/unpopulated, refresh controls do not function, adversary simulation traffic is not visible in monitoring, and monitoring filter controls drift from shared dashboard design conventions.

Non-negotiable delivery rules for every `SIM2-R4-*` slice:
1. Monitoring must represent real request-pipeline telemetry and must not rely on synthetic chart/event entries.
2. Monitoring data fetch/render behavior must not depend on whether adversary simulation is enabled.
3. Historical data must remain visible while new live activity appends in near real time.
4. Dashboard UI controls must reuse canonical shared styles/components; one-off styling is not acceptable.
5. Config-seeding lifecycle decisions must be explicit, documented, and consistent across setup/dev/prod paths.

### SIM2-R4-4: Re-Architect Config Seeding Lifecycle and Test-Mode Persistence Semantics
- [ ] SIM2-R4-4-1 Establish and document a strict lifecycle policy:
  - `make setup` and `make setup-runtime` may initialize/backfill KV defaults.
  - `make config-seed` is the explicit operator/developer migration/backfill command.
  - Runtime start commands (`make dev`, `make dev-closed`, `make run`, `make run-prebuilt`, `make prod`) are read-only by default and must not mutate persisted KV config.
- [ ] SIM2-R4-4-2 Remove implicit seed/backfill calls from runtime start and watch-restart paths; add explicit diagnostics that distinguish “config present”, “config missing”, and “migration required”.
- [ ] SIM2-R4-4-3 Add an explicit low-surprise recovery path for local workflows when config is missing (clear command guidance and optional explicit `config-ensure` helper if required), without silent mutation during normal runtime start.
- [ ] SIM2-R4-4-4 Guarantee production start paths never silently write config unless an operator explicitly invokes seeding/migration commands.
- [ ] SIM2-R4-4-5 Introduce deterministic migration tests for defaults evolution:
  - missing `config:default` bootstrap path,
  - existing config backfill of newly introduced keys,
  - no-op behavior when config is already schema-complete,
  - failure-path diagnostics for invalid existing config JSON.
- [ ] SIM2-R4-4-6 Resolve `test_mode` persistence contract explicitly and implement the chosen model end-to-end (default target for this round: non-persisted ephemeral runtime/session state so it cannot linger across runs), while preserving explicit startup override via `SHUMA_TEST_MODE=true` in both `runtime-dev` and `runtime-prod` entrypoints.
- [ ] SIM2-R4-4-7 If `test_mode` becomes ephemeral, remove its persistence coupling from KV/default seeding and align all control paths (`/admin/config` contract, dashboard behavior, runtime toggles, export payloads, docs, tests) to the new semantics.
- [ ] SIM2-R4-4-8 Publish an operator-facing architecture note (or ADR if scope widens) that records the final lifecycle + test-mode semantics, rationale, rollback plan, and risk tradeoffs.

Acceptance criteria:
- Running `make dev`, `make dev-closed`, `make run`, `make run-prebuilt`, and `make prod` performs zero implicit KV config writes under normal operation.
- Only explicit setup/migration commands mutate seeded KV defaults, and those mutations are idempotent and bounded.
- Command/output messaging clearly tells operators what happened and what to run next when config is absent or stale; no silent repair during runtime start.
- Runtime behavior and docs are consistent for every entrypoint: `make setup`, `make setup-runtime`, `make config-seed`, dev starts, and prod starts.
- Regression coverage proves both migration correctness and read-only start semantics, including watch-restart paths.
- `test_mode` semantics are unambiguous, documented, and test-enforced:
  - if ephemeral: toggling test mode does not persist across process restart and cannot be reintroduced by seed/backfill;
  - if ephemeral: explicit environment startup override (`SHUMA_TEST_MODE=true`) remains supported for both `make dev` and `make prod` flows, applies only to current runtime/session posture, and must not be written into persisted KV config by runtime or dashboard control paths;
  - if persisted (only by explicit exception): persistence scope, reset policy, and safety guardrails are fully documented and tested.
- Dashboard/admin API contracts match the chosen `test_mode` model and reject/flag invalid legacy assumptions.

Definition of done:
- Makefile target matrix reflects final policy with no contradictory seed invocations in runtime start targets.
- Bootstrap, runtime, and deployment docs all reflect the same lifecycle and `test_mode` contract.
- Required Makefile verification passes for the slice, including unit/integration/dashboard coverage that exercises lifecycle and `test_mode` behavior.
- Completion notes include security impact, operational impact, resource impact, and rollback steps.

### SIM2-R4-5: Dashboard Connection-State Hardening (Instrumentation-First, Heartbeat-Owned)

Objective: eliminate flip-flop/disconnect churn by moving global connection-state authority to one dedicated heartbeat signal path, classifying all request failures, and explicitly forbidding cancelled client-side requests from mutating global connection state.

Non-negotiable delivery rules for every `SIM2-R4-5-*` slice:
1. Instrumentation and evidence capture must land before any global state-machine rewiring.
2. Global connection state must be heartbeat-owned only (single writer).
3. Request failures must be classified as `cancelled`, `timeout`, `transport`, or `http`.
4. `cancelled` failures (including `AbortError`) must not mutate global connection state.
5. Connection state model for this tranche is `disconnected -> connected -> degraded -> disconnected` (no `unknown` state).
6. Initial dashboard connection state must be `disconnected` until the first successful heartbeat confirms backend health.
7. Hysteresis is mandatory: transition to `disconnected` only after `N` consecutive heartbeat failures (default target `N=3`, configurable only if explicitly justified).
8. Lost-connection CSS/style behavior is out of scope for this tranche and must not be changed here.

Execution gate:
1. `SIM2-R4-5-1`, `SIM2-R4-5-2`, and `SIM2-R4-5-3` must be complete and green before `SIM2-R4-5-4+` implementation slices start.

- [ ] SIM2-R4-5-1 Add request-failure instrumentation contract (read-only behavior slice):
  - Define a shared failure-classification helper in the dashboard runtime domain (`cancelled`, `timeout`, `transport`, `http`).
  - Emit structured per-request telemetry fields: `request_id`, `path`, `method`, `tab`, `reason`, `started_at`, `duration_ms`, `outcome`, `failure_class`, `status_code`, `aborted`.
  - Add bounded in-memory ring-buffer diagnostics for recent failures (fixed max length, no unbounded growth).
  - Expose diagnostics through existing runtime telemetry surfaces used by status/debug views and tests.
  - Ensure instrumentation is passive only (no behavior/state transition changes in this slice).

- [ ] SIM2-R4-5-2 Add dedicated heartbeat telemetry surface (still no state-machine change):
  - Add explicit heartbeat diagnostics fields: `last_heartbeat_success_at`, `last_heartbeat_failure_at`, `consecutive_failures`, `last_failure_class`, `last_failure_error`, `last_transition_reason`.
  - Include event-level breadcrumbs for heartbeat attempts (`attempt_started`, `attempt_succeeded`, `attempt_failed`, `retry_scheduled`).
  - Ensure heartbeat telemetry and generic request telemetry are clearly separated so failure attribution is unambiguous.
  - Add explicit guardrail counters for ignored failures (`ignored_cancelled_count`, `ignored_non_heartbeat_failure_count`).

- [ ] SIM2-R4-5-3 Add instrumentation-proof tests before architecture rewiring:
  - Unit-test failure classifier across representative error shapes (`AbortError`, timeout-generated abort, transport/network exceptions, HTTP non-OK responses).
  - Unit-test telemetry emission shape and retention window behavior (append, trim, ordering, timestamp population).
  - Add regression test reproducing auto-refresh abort churn and assert instrumentation captures the specific aborted request path(s).
  - Add regression test asserting no global connection mutation occurs in instrumentation-only phase.
  - Require red/green evidence in `make test-dashboard-unit` before moving to `SIM2-R4-5-4`.

- [ ] SIM2-R4-5-4 Define and lock the connection-state state-machine spec (contract slice):
  - Document exact states and transitions: boot `disconnected`, heartbeat success `connected`, heartbeat failure threshold behavior (`degraded` then `disconnected` after `N` failures).
  - Define recovery transitions (`degraded -> connected` on success, `disconnected -> connected` on success).
  - Define transition reasons taxonomy (`heartbeat_ok`, `heartbeat_timeout`, `heartbeat_transport_error`, `heartbeat_http_error`, `heartbeat_cancelled_ignored`, etc.).
  - Specify hysteresis constants and where they are configured.
  - Record this as a short architecture note/ADR addendum before implementation changes.

- [ ] SIM2-R4-5-5 Implement heartbeat-owned global connection controller:
  - Introduce a dedicated module responsible for connection-state transitions and hysteresis accounting.
  - Remove generic API-client callback ownership of global connection state updates.
  - Route global-state mutation calls only through the dedicated heartbeat pathway.
  - Preserve existing route/refresh functionality while re-homing only connection ownership.
  - Keep state writes idempotent and transition-guarded (no repeated no-op churn writes).

- [ ] SIM2-R4-5-6 Rewire request paths to classification + local handling rules:
  - Ensure non-heartbeat request failures update only local tab/runtime diagnostics.
  - Ensure `cancelled` class is treated as expected control-flow, not connectivity failure.
  - Ensure timeout/transport/http from non-heartbeat paths do not directly flip global state.
  - Add explicit hooks so heartbeat scheduler can consume classified heartbeat failures for hysteresis accounting.

- [ ] SIM2-R4-5-7 Integrate heartbeat ownership cleanly with monitoring and adversary-sim ticks:
  - Verify monitoring auto-refresh cadence and adversary-sim status polling can run concurrently without competing state ownership.
  - Keep existing single-flight guards where they prevent abort churn, but ensure they are subordinate to classification telemetry (no hidden suppression).
  - Confirm one writer for global connection state even when multiple polling loops are active.
  - Add explicit assertions for “multiple loops active, one global connection owner” behavior.

- [ ] SIM2-R4-5-8 Expand regression/e2e coverage for state transitions and churn scenarios:
  - Add deterministic tests for full transition graph (`disconnected -> connected -> degraded -> disconnected -> connected`).
  - Add tests for hysteresis thresholds (`N-1` failures stays non-disconnected, `N`th failure transitions).
  - Add tests for steady abort cadence proving connection state remains stable when only cancellations occur.
  - Add tests for mixed failures showing only heartbeat-classified failures affect global connection state.
  - Add smoke/e2e assertions that dashboard no longer flip-flops under monitoring auto-refresh churn.

- [ ] SIM2-R4-5-9 Documentation, operator diagnostics, and rollout safeguards:
  - Update dashboard/runtime docs with new ownership model and failure taxonomy.
  - Document how to inspect request/heartbeat diagnostics during incidents.
  - Add rollback guidance (feature flag or narrow rollback steps) for connection-controller changes.
  - Update testing docs with canonical verification path and expected evidence artifacts.

Acceptance criteria:
- Reproduced abort cadence shows stable global connection state unless heartbeat failures cross hysteresis threshold.
- Global connection state has exactly one writer path (dedicated heartbeat controller), validated by tests.
- `cancelled` failures are visible in diagnostics but never trigger `degraded`/`disconnected`.
- Initial state is `disconnected`, first successful heartbeat promotes to `connected`, and failures progress through `degraded` before `disconnected`.
- Monitoring auto-refresh and adversary-sim status polling operate concurrently without global-state contention.
- No lost-connection CSS changes are included in this slice.

Definition of done:
- Instrumentation-first slices (`SIM2-R4-5-1..3`) are merged and evidenced before rewiring slices (`SIM2-R4-5-4..9`).
- Unit/integration/e2e coverage captures transition behavior, failure classification, and churn resilience.
- `make test-dashboard-unit`, `make test-integration`, `make test-dashboard-e2e`, and `make test` pass for the final integrated slice.
- Completion notes include resource impact (extra telemetry overhead), operational impact (incident diagnosis), and rollback steps.

## P0 Launch-Readiness Performance Pass
- [ ] PERF-LAUNCH-1 Execute a final pre-launch performance and optimization pass (dashboard bundle-size budgets in strict mode, runtime latency/<abbr title="Central Processing Unit">CPU</abbr>/memory envelopes, and high-cost request-path profiling), then lock release thresholds and acceptance criteria.

## P1 Privacy and Data-Protection Follow-up
- [ ] SEC-GDPR-2 Enforce deterministic cleanup/expiry for stale fingerprint state keys (`fp:state:*`, `fp:flow:*`, `fp:flow:last_bucket:*`) aligned to configured fingerprint TTL/window controls.
- [ ] SEC-GDPR-3 Add an optional event-log IP minimization mode (raw vs masked/pseudonymized) for privacy-sensitive deployments, with explicit tradeoff documentation.
- [ ] SEC-GDPR-4 Add a deployer-ready privacy/cookie disclosure template in docs (lawful basis, retention table, storage inventory, and rights-handling workflow).

## P0 Deployment Path Excellence (Single-Host + Akamai/Fermyon)
Reference plan: [`docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](../docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)

- [ ] DEP-ENT-1 Implement strict enterprise distributed ban-sync mode for authoritative multi-instance posture (no silent local-only divergence path).
- [ ] DEP-ENT-2 Add ban-sync observability (<abbr title="Service Level Objective">SLO</abbr> metrics for sync result and lag) to support promotion/rollback decisions.
- [ ] DEP-ENT-3 Add two-instance Spin integration coverage with shared Redis to prove ban/unban convergence behavior.
- [ ] DEP-ENT-4 Add outage/partition tests for distributed state (Redis unavailable/degraded) and assert explicit configured behavior by mode.
- [ ] DEP-ENT-5 Add deploy/runtime guardrails that validate enterprise distributed-state posture against outbound and backend requirements before authoritative operation.
- [ ] DEP-ENT-6 Design optional asynchronous mirror of high-confidence bans to Akamai Network Lists (additive perimeter control; Shuma policy remains source-of-truth).

### DEP-GW-1: Gateway-Only Existing-Site Integration (Only Production Mode)

Completed on 2026-03-05 and archived in `todos/completed-todo-history.md`.

Completion evidence:
1. Conformance review: [`docs/research/2026-03-05-gateway-first-tranche-conformance-review.md`](../docs/research/2026-03-05-gateway-first-tranche-conformance-review.md)
2. Cleanup review: [`docs/research/2026-03-05-gateway-first-post-tranche-cleanup-review.md`](../docs/research/2026-03-05-gateway-first-post-tranche-cleanup-review.md)

### DEP-GW-POST: Gateway Follow-On Hardening

Completed on 2026-03-05 and archived in `todos/completed-todo-history.md`.

Completion evidence:
1. wasm TLS cert-failure harness: `scripts/tests/gateway_tls_wasm_harness.py` + `make test-gateway-wasm-tls-harness`
2. optional active origin-bypass probe: `scripts/deploy/probe_gateway_origin_bypass.py` + `make test-gateway-origin-bypass-probe`

## P0 Adversarial Traffic Simulation Program
Reference plan: [`docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](../docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)
Refinement plan: [`docs/plans/2026-02-26-adversarial-simulation-v2-plan.md`](../docs/plans/2026-02-26-adversarial-simulation-v2-plan.md)
Heartbeat decoupling plan: [`docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](../docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)
Mandatory pre-read architecture report for all open SIM work: [`docs/research/2026-03-02-sim-runtime-architecture-overview-and-gap-report.md`](../docs/research/2026-03-02-sim-runtime-architecture-overview-and-gap-report.md)
Prime directive report (shared corpus + out-of-process heartbeat): [`docs/research/2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md`](../docs/research/2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md)
Production-availability decision criteria (`SIM-DEPLOY-1`): [`docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md`](../docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md)
Execution gate: no open `SIM-*` TODO in this section may move forward until both reports above have been read and the implementation slice is explicitly checked against runtime/test-lane boundary findings and prime-directive constraints.

Ongoing objectives for this tranche:
1. Keep deterministic oracle as regression authority; LLM lane as discovery/promotable corpus.
2. Share one canonical deterministic attack corpus across runtime and CI oracle executors.
3. Keep CI Python oracle focused on setup/gates/evidence (separate system), while sharing attacker action definitions with runtime.
4. Keep strict control-plane and attacker-plane separation.
5. Keep resource caps and explicit degraded states so frontier outages do not fake success.
6. Ensure all simulation traffic is indistinguishable in enforcement and telemetry paths from external traffic.
7. Move runtime generation cadence ownership to an out-of-process supervisor (not request lifecycle, not dashboard refresh loop).
8. Enforce strict off-state inertness: when adversary simulation is toggled off, no simulator heartbeat loop runs, no generator/supervisor process remains active, and no simulation traffic is emitted.
9. Use spawn-on-enable/teardown-on-disable lifecycle for simulator execution so runtime resource usage attributable to adversary simulation is effectively zero while off.
10. Simplify freshness ownership to one source of truth (backend), with the UI acting as renderer only.
11. Treat adversary simulation as potential core production value, enabling operators to red-team defenses immediately after deployment rather than waiting for external attackers.

### SIM-SH-SURFACE-1: Shared-Host Public Surface Discovery First (Gate Before Non-Deterministic Lane)

Reference plan: [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)

Objective: deliver a production-setting toolchain that discovers Shuma host public surface on a shared single server, with strict execution order (`robots.txt` + `sitemap.xml` first, then bounded Scrapling probe augmentation), and produce real-host evidence artifacts before any non-deterministic Scrapling sim-lane runtime implementation.

Naming note:
1. This roadmap uses `SIM-SCR-LANE-1` as the canonical runtime lane tranche name.
2. Earlier shorthand references to `SIM-SCR-6` in draft planning docs map to this same runtime-lane tranche.

Non-negotiable delivery rules for every `SIM-SH-SURFACE-1-*` slice:
1. Discovery order must be `robots/sitemap ingest -> normalize/scope -> Scrapling probe augmentation`; probe must not run first.
2. Scope policy must be fail-closed and enforced on every discovered/probed URL.
3. Scrapling in this milestone is discovery tooling only, not heartbeat-owned sim-lane execution.
4. Real shared-host execution evidence is mandatory before lane runtime work.
5. No hidden dashboard-driven scheduling loops; discovery is explicit operator-run workflow.

Execution gate:
1. `SIM-SCR-LANE-1`, `SIM-LLM-1`, and any non-deterministic lane runtime wiring are blocked until `SIM-SH-SURFACE-1-1..SIM-SH-SURFACE-1-10` are complete and verified.

- [ ] SIM-SH-SURFACE-1-1 Define shared-host target descriptor + scope contract:
  - canonical host base URL, allowed hosts, allowed path prefixes, denied path prefixes, redirect-hop limits;
  - explicit rejection taxonomy and fail-closed behavior for missing/invalid policy.
- [ ] SIM-SH-SURFACE-1-2 Implement `robots.txt` discovery step with tests:
  - fetch + parse `Allow`/`Disallow` and declared sitemap links;
  - capture provenance and parse diagnostics.
- [ ] SIM-SH-SURFACE-1-3 Implement `sitemap.xml` discovery step with tests:
  - support sitemap indexes, nested maps, compressed sitemap payloads, and deterministic URL extraction;
  - capture fetch/parse diagnostics and invalid-source reporting.
- [ ] SIM-SH-SURFACE-1-4 Implement seed inventory merger (robots+sitemaps) with deterministic canonicalization:
  - URL normalization, dedupe, stable ordering, scope filtering, and structured rejection reasons.
- [ ] SIM-SH-SURFACE-1-5 Add operator make target for seed-only discovery path:
  - proposed target: `make adversary-surface-discover-shared-host-seed`;
  - output artifact under `scripts/tests/adversarial/` with provenance and summary counters.
- [ ] SIM-SH-SURFACE-1-6 Add bounded Scrapling probe augmentation step (discovery-only):
  - `allowed_domains` from scope policy,
  - explicit crawl budgets (requests/depth/bytes/time),
  - offsite and blocked-request stats surfaced in artifact metadata.
- [ ] SIM-SH-SURFACE-1-7 Add operator make target for seed+probe combined workflow:
  - proposed target: `make adversary-surface-discover-shared-host`;
  - ensure deterministic behavior when seed inputs and crawl seed are fixed.
- [ ] SIM-SH-SURFACE-1-8 Run the combined discovery workflow against a real shared-host deployment:
  - produce timestamped evidence artifacts (`inventory`, `summary`, `rejections`, `crawl stats`);
  - confirm only in-scope public URLs are present.
- [ ] SIM-SH-SURFACE-1-9 Compile discovered inventory into hosted-surface catalog baseline:
  - generate catalog hash/version and compile diagnostics;
  - record inventory-to-catalog lineage in artifact metadata.
- [ ] SIM-SH-SURFACE-1-10 Document operator runbook + signoff checklist for shared-host discovery:
  - preflight checks, execution commands, artifact interpretation, safety checks, and rollback steps.

Acceptance criteria:
- Shared-host discovery works end-to-end in production setting with explicit command path and reproducible artifacts.
- Discovery artifacts prove execution order and provenance split (`robots`, `sitemap`, `crawl`).
- Scope policy exclusions are visible, auditable, and enforced for both seed ingest and probe phases.
- Catalog baseline can be produced directly from shared-host discovery outputs.
- Non-deterministic Scrapling lane work remains blocked until this milestone is marked complete.

Definition of done:
- New/updated make targets exist and are documented in operator docs.
- Required unit/integration verification for discovery toolchain passes via Makefile targets.
- At least one real shared-host discovery evidence set is archived in the expected artifact location and referenced in completion notes.

### SIM-SCR-LANE-1: Three-Lane Runtime Migration + Scrapling Integration (Post Shared-Host Discovery Gate)

Reference plan: [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)

Objective: migrate from the current toggle-only deterministic runtime baseline to a 3-lane runtime selector model, then implement bounded out-of-process Scrapling lane execution driven only by adversary-sim supervisor heartbeat, with strict fail-closed confinement and explicit degraded-state semantics.

Non-negotiable delivery rules for every `SIM-SCR-LANE-1-*` slice:
1. Current toggle-only control path must continue to work during migration (`POST /admin/adversary-sim/control` with `enabled` only).
2. Target lane contract is three lanes: `synthetic_traffic`, `scrapling_traffic`, `bot_red_team`.
3. Exactly one active lane per beat; zero concurrent lane activity.
4. Monitoring UI remains read-only and must not drive generation cadence.
5. Global connection-state ownership remains heartbeat-only (aligned with `SIM2-R4-5` contract).
6. Worker policy is fail-closed: scope violation, egress violation, timeout, or worker crash must not degrade into permissive execution.

Execution gate:
1. `SIM-SCR-LANE-1-*` is blocked until `SIM-SH-SURFACE-1` milestone is complete and evidenced.

- [ ] SIM-SCR-LANE-1-1 Define and enforce lane-state contract in control/status API:
  - required fields: `desired_lane`, `active_lane`, `lane_switch_seq`, `last_lane_switch_at`, `last_lane_switch_reason`;
  - allowed lane enum: `synthetic_traffic | scrapling_traffic | bot_red_team`;
  - preserve baseline status compatibility fields during migration (`active_lane_count`, existing `lanes.*` payload shape) until dashboard migration cutover.
- [ ] SIM-SCR-LANE-1-2 Extend control API payload compatibility:
  - keep existing `enabled` toggle payload behavior;
  - add optional lane selection field with strict enum validation;
  - keep idempotency/audit semantics unchanged.
- [ ] SIM-SCR-LANE-1-3 Implement supervisor lane router semantics:
  - route exactly one lane per beat;
  - lane switch applies on next beat and prior lane receives zero subsequent ticks.
- [ ] SIM-SCR-LANE-1-4 Implement Scrapling worker beat contract:
  - out-of-process worker invocation with strict `TickBudget` (`max_requests`, `max_depth`, `max_bytes`, `max_ms`);
  - resumable frontier state persistence between beats.
- [ ] SIM-SCR-LANE-1-5 Implement strict confinement gate:
  - `https` only, no IP-literal targets, allowlisted hostnames only;
  - redirect target must re-pass allowlist/scope gate;
  - deny privileged/internal path families (`/admin`, `/internal`, `/dashboard`, `/session`, `/auth`, `/login`).
- [ ] SIM-SCR-LANE-1-6 Implement worker egress isolation:
  - isolated worker runtime/network namespace/container;
  - outbound allowlist restricted to approved host `:443` plus DNS.
- [ ] SIM-SCR-LANE-1-7 Implement request header policy:
  - block privileged headers (`Authorization`, internal Shuma headers) from worker requests.
- [ ] SIM-SCR-LANE-1-8 Implement per-request provenance + degraded-state reporting:
  - mandatory lineage fields: `run_id`, `lane`, `tick_id`, `worker_id`;
  - explicit statuses for `ok`, `degraded`, `failed_closed` and reason taxonomy.
- [ ] SIM-SCR-LANE-1-9 Implement failure classification guardrails:
  - classify failures as `cancelled`, `timeout`, `transport`, `http`;
  - forbid `cancelled` from mutating global connection state.
- [ ] SIM-SCR-LANE-1-10 Implement dashboard lane selector migration:
  - keep top-level Adversary Sim toggle;
  - add exclusive radio group for `Synthetic Traffic (Internal)`, `Scrapling Crawler/Scraper`, `Bot Red Team (LLM)` (disabled/annotated until ready);
  - ensure status rendering uses backend truth (no optimistic local lane state).
- [ ] SIM-SCR-LANE-1-11 Add required verification gates:
  - unit: URL/path/redirect confinement policy;
  - integration: worker cannot reach out-of-scope hosts;
  - integration: lane switch leaves zero concurrent lane activity;
  - e2e: selected runtime lane traffic (`synthetic_traffic`, `scrapling_traffic`) appears in normal monitoring telemetry;
  - failure tests: worker crash/timeout and heartbeat loss fail closed.
- [ ] SIM-SCR-LANE-1-12 Document rollout/rollback + operator diagnostics:
  - lane contract, worker degraded-state semantics, investigation commands, and rollback switches.

Acceptance criteria:
- Supervisor heartbeat is sole cadence writer for lane execution.
- Runtime lane contract is explicit, test-enforced, and constrained to `synthetic_traffic | scrapling_traffic | bot_red_team`.
- Existing toggle-only control behavior remains valid during migration until final cutover.
- Scrapling worker is bounded, resumable, and fail-closed under policy/runtime failure.
- Confinement, egress, header policy, provenance, and degraded-state diagnostics are all observable and test-covered.

Definition of done:
- All non-negotiable verification gates pass through Makefile targets.
- Docs include clear operator guidance and rollback for lane runtime failures.
- Completion evidence shows no UI-timer generation path and no multi-lane beat overlap.

### SIM-BREACH-REPLAY-1: External Breach -> Replayable Attack Pipeline (Planning + Research Required)

Objective: design and then implement a safe, auditable pipeline that turns real external bot-defence breaches into replayable attack artifacts usable by deterministic gates and Scrapling discovery profiles.

Non-negotiable delivery rules for every `SIM-BREACH-REPLAY-1-*` slice:
1. Planning/research outputs must be completed before implementation begins.
2. Privacy and security controls are mandatory (no secret/token/body leakage into replay artifacts).
3. Deterministic replay remains release-blocking authority; generated/emergent artifacts remain advisory until deterministic confirmation.
4. Artifact lineage must be explicit end-to-end (`raw event -> normalized breach -> replay candidate -> confirmation result -> promotion decision`).

Execution gate:
1. No runtime implementation for this pipeline may begin until `SIM-BREACH-REPLAY-1-1..SIM-BREACH-REPLAY-1-4` are complete and approved.

- [ ] SIM-BREACH-REPLAY-1-1 Research and document prior art:
  - survey practical approaches for attack-capture normalization, replay generation, and promotion governance in bot-defence systems;
  - compare deterministic replay versus crawler-profile replay tradeoffs for false confidence and maintenance cost.
- [ ] SIM-BREACH-REPLAY-1-2 Write architecture plan for breach-capture and normalization:
  - define capture contract (`method`, `path`, query, selected headers, payload fingerprint, timing, response/policy outcomes);
  - define sanitization/redaction policy and retention rules for replay-safe artifacts.
- [ ] SIM-BREACH-REPLAY-1-3 Write architecture plan for replay generation and confirmation:
  - deterministic scenario generation path (blocking oracle);
  - Scrapling seed/profile generation path (emergent stress lane);
  - reproducibility requirements and confirmation thresholds before promotion.
- [ ] SIM-BREACH-REPLAY-1-4 Define governance + operator workflow:
  - owner review/disposition requirements, severity-based SLA, and rollback semantics;
  - promotion criteria from advisory findings into deterministic blocking corpus.
- [ ] SIM-BREACH-REPLAY-1-5 Implement normalized breach artifact schema + validators:
  - schema versioning, required lineage fields, and strict validation failures on missing/unsafe data.
- [ ] SIM-BREACH-REPLAY-1-6 Implement deterministic replay candidate generator:
  - map normalized breach artifacts to deterministic scenario entries with stable IDs and compile diagnostics.
- [ ] SIM-BREACH-REPLAY-1-7 Implement Scrapling replay-profile generator:
  - map normalized breach artifacts to bounded scope/cadence/session profiles for emergent reruns.
- [ ] SIM-BREACH-REPLAY-1-8 Add verification and docs:
  - unit/integration tests for redaction, normalization, replay generation, and governance gates;
  - operator runbook for triage, replay, confirmation, and promotion decisions.

Acceptance criteria:
- Planning/research outputs clearly define architecture, governance, and safety posture before implementation.
- Raw breach evidence can be converted into sanitized, versioned replay artifacts with traceable lineage.
- Both replay targets are supported: deterministic (blocking authority) and Scrapling profile (advisory stress lane).
- Promotion to blocking corpus requires explicit deterministic confirmation and owner disposition.

Definition of done:
- Planning docs are merged and linked from this TODO entry.
- Implementation slices pass required Makefile verification targets.
- Operator documentation covers incident-to-replay workflow, review gates, and rollback.

- [ ] SIM-DEPLOY-2 If production availability is approved, design and implement production-safe adversary-sim operating modes (explicit opt-in, spawn-on-enable execution lifecycle, strict rate/resource envelopes, kill switch, auditability, and no-impact guarantees for normal user traffic).
- [ ] SIM-LLM-1 Realize full LLM-orchestrated, instruction-driven, containerized adversary lane as first-class runtime actor on top of the same runtime heartbeat ownership model: run capability-constrained action plans against public HTTP surface, emit normal request-pipeline telemetry, preserve deterministic replay bridge, and surface explicit degraded-state diagnostics when frontier execution is unavailable.
- [ ] SIM-DET-L1 (Low priority) Add optional deterministic seed input for runtime-toggle runs to support exact tune-confirm-repeat replay when desired; keep default behavior non-seeded.

## P1 Akamai Integration Controls Expansion (Rate Limiting + GEO)
- [ ] AK-RG-1 Write a concise architecture note (or ADR if scope broadens) that defines exact semantics for Akamai controls on Rate Limiting and GEO (`off`, `additive`, `authoritative` behavior, precedence, fallback, and trust boundaries).
- [ ] AK-RG-2 Define config surface and naming for Rate/GEO Akamai integration controls, including defaults and whether each is a simple toggle or toggle+mode control.
- [ ] AK-RG-3 Implement admin API + runtime config validation for the new Rate/GEO Akamai controls with explicit guardrails and clear validation errors.
- [ ] AK-RG-4 Implement runtime behavior wiring so Akamai Rate/GEO signals can influence decisions according to the defined mode semantics without bypassing Shuma’s policy ownership.
- [ ] AK-RG-5 Add dashboard controls and help text for Rate/GEO Akamai integration in the new top-level tabs, including disabled-state behavior and operator warnings.
- [ ] AK-RG-6 Add observability and policy-event taxonomy coverage for Rate/GEO Akamai decisions (source, mode, action, fallback reason, and downgrade behavior).
- [ ] AK-RG-7 Add integration/e2e tests for mode precedence, downgrade/fallback safety, and regression against internal-only behavior.
- [ ] AK-RG-8 Document rollout/rollback guidance for enabling Rate/GEO Akamai integration in enterprise deployments, including promotion gates and emergency disable steps.

## P1 Outbound and Tarpit Hardening
- Research synthesis recorded in [`docs/research/2026-02-22-http-tarpit-cost-shift-research-synthesis.md`](../docs/research/2026-02-22-http-tarpit-cost-shift-research-synthesis.md) (includes source mapping, implementation implications, and cost-shift analysis).
- [ ] OUT-1 Add explicit deployment guardrails that fail when `provider_backends.rate_limiter=external` or `provider_backends.ban_store=external` but required Redis outbound hosts are not allowlisted in `spin.toml` `allowed_outbound_hosts`.
- [ ] OUT-2 Add a provider-to-outbound-requirements matrix in public docs (internal vs external backend, required host capabilities, required outbound host allowlists, fallback behavior).
- [ ] OUT-3 Add integration verification that exercises external Redis provider selection under restricted outbound policy and confirms safe fallback/guardrail behavior is deterministic.
- [ ] OUT-4 Create an ADR for non-Redis external integrations (for example webhook notifications or cross-service sync) that defines the approved pattern in Spin (`allowed_outbound_hosts` expansion vs sidecar/bridge service).
- [ ] OUT-5 Before implementing non-stub `challenge_engine=external` and `maze_tarpit=external`, complete design work for their external transport path through Spin host capabilities or sidecar/adapter boundary, with rollback and security posture defined.

#### Tarpit Asymmetry Hardening (`work-gated`, `token-chained`, `egress-budgeted`)

Architecture alignment reference: [`docs/plans/2026-02-23-maze-tarpit-architecture-alignment-plan.md`](../docs/plans/2026-02-23-maze-tarpit-architecture-alignment-plan.md)

- [ ] TAH-11 Expand observability: progression admissions/denials, proof verify outcomes, chain violations, bytes sent, duration, budget exhaustion reason, fallback action, and escalation outcomes (including top offender buckets with cardinality guardrails).
- [ ] TAH-12 Add dashboard/admin visibility for the new tarpit progression + egress metrics and operator guidance for safe tuning (recommended starting ranges and rollback thresholds).
- [ ] TAH-19 Before launch, tighten collateral-risk controls (especially bucket-based persistence escalation), then re-evaluate tarpit defaults.

Execution order for remaining tarpit work:
1. `TAH-11`
2. `TAH-12`

### Stage 2.6 follow-up: Maze test coverage closure
- [ ] MZ-T1 Add Spin integration coverage for live opaque maze traversal across multiple hops: entry -> tokenized link follow -> checkpoint submit -> `<maze_path_prefix>issue-links` progression -> fallback/escalation branches, with assertions for deterministic fallback action/reason semantics.
- [ ] MZ-T2 Add browser E2E coverage for live maze behavior (not just dashboard config): JS-enabled and JS-disabled cohorts, checkpoint/micro-PoW flow, replay rejection, and high-confidence escalation outcomes under real HTTP/session behavior.
- [ ] MZ-T3 Add concurrency/soak coverage for maze state/budget primitives (replay keys, checkpoint keys, global/per-bucket budget caps) to detect contention/regression under burst traversal and verify bounded host-write behavior.
- [ ] MZ-T4 Wire the new maze integration + E2E + soak tests into canonical Makefile/CI verification paths (`make test`, focused rerun targets, and CI failure gates) so maze behavior regressions fail fast before merge.

## P2 Challenge Roadmap
- [ ] NAB-12 Evaluate optional PAT-style private attestation signal ingestion as additive evidence only (non-blocking).
- [ ] NAB-13 Execute short Not-a-Bot hardening sprint per [`docs/plans/2026-02-21-not-a-bot-hardening-sprint.md`](../docs/plans/2026-02-21-not-a-bot-hardening-sprint.md) (unknown-modality cap, stronger pass corroboration, cross-attempt/session consistency gating, accessibility-safe anti-fast-path tightening).

## P2 GEO Defence Maturity
- [ ] Add ASN/network dimensions in GEO policy logic (not just country list). (`src/signals/geo/mod.rs`, `src/config/mod.rs`, `src/admin/api.rs`)
- [ ] Add GEO/ASN observability and alerting (metrics, dashboard panels, docs). (`src/observability/metrics.rs`, dashboard, docs)

## P2 Modularization and Future Repository Boundaries
- [ ] Write objective criteria for future repo splits (API stability, release cadence, ownership, operational coupling).
## P3 Platform and Configuration Clarity
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths.
- [ ] Evaluate renaming `SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_PUZZLE_THRESHOLD` to reflect botness semantics.
- [ ] Standardize terminology across code/UI/docs so `honeypot` and `maze` are used consistently instead of interchangeably.
- [ ] Document setup-time config bootstrapping clearly: how `make setup` creates/populates local env, how env-only vars are sourced, and how KV defaults are seeded and later overridden.
- [ ] Long-term option: integrate upstream identity/proxy auth (OIDC/SAML) for dashboard/admin instead of app-level key login.
