# TODO Roadmap

Last updated: 2026-03-01

This is the active work queue.
`todos/security-review.md` tracks security finding validity and closure status.
Completed items are archived in `todos/completed-todo-history.md`.

## P0 Immediate Next-Agent Start (Highest Priority): Adversarial Simulation v2
Status: Execution complete on 2026-02-27. See `todos/completed-todo-history.md` for `SIM-V2-*` closure details.

### Required background (read first)
1. [`docs/plans/2026-02-26-adversarial-simulation-v2-plan.md`](../docs/plans/2026-02-26-adversarial-simulation-v2-plan.md)
2. [`docs/research/2026-02-25-llm-adversarial-testing-research-synthesis.md`](../docs/research/2026-02-25-llm-adversarial-testing-research-synthesis.md)
3. This TODO section `SIM-V2-*` below (authoritative task contract)

### Non-negotiable policy decisions (already agreed)
1. Frontier LLM adversary is central for adaptive discovery, but deterministic replay remains the blocking regression oracle.
2. Black-box-only adversary posture: no white-box adversary lane.
3. `SHUMA_ADVERSARY_SIM_AVAILABLE` is env-only, defaults to `true` in `make dev`, defaults to `false` in production.
4. Production must fail closed for adversary sim (no sim control/start path in prod).
5. Dev UI Adversary Sim toggle must be placed directly under the `test_mode` toggle and must start/stop full orchestration (no second toggle, no extra terminal command after `make dev`), with admin auth, CSRF protection, and audit logging.
6. Missing frontier keys in dev toggle-on flow must show warning with explicit continue-without-frontier option.
7. Toggle-on run must be resource-guarded (default 3-minute window + top progress line + auto-off teardown), with hard-coded max duration/concurrency/cpu-memory/queue guardrails.
8. Mandatory CI/release blockers remain deterministic; frontier outage is advisory degraded status, not a hard release blocker, but persistent unavailability must trigger supported-model-list refresh under an explicit threshold policy.
9. Frontier data governance is required (strict outbound allowlist + redaction/minimization + retention rules).
10. Sim events must be telemetry-tagged and dev/prod data planes must remain separated.

### First implementation sequence (next agent execution order)
1. `SIM-V2-*` execution order has completed for this phase; use completed history for implementation references and acceptance coverage evidence.

### Critical files likely touched (for rapid orientation)
1. `Makefile` (dev defaults, new make targets, CI/release invocation wiring)
2. `config/defaults.env` (new env-only defaults)
3. `scripts/bootstrap/setup.sh`, `scripts/bootstrap/setup-runtime.sh`, `docs/configuration.md` (setup/config docs lifecycle)
4. `src/config/mod.rs`, `src/admin/api.rs`, `src/lib.rs` (runtime config + endpoint behavior)
5. `dashboard/src/routes/+page.svelte`, `dashboard/src/lib/domain/config-schema.js` (toggle UX + body classes + warnings)
6. `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/adversarial/*` (manifest/runner/report/promotion flow)
7. CI workflow files under `.github/workflows/*` (PR/release policy enforcement)

### Definition of successful handoff
1. Next agent can start implementation immediately from `SIM-V2-*` without re-deriving policy decisions.
2. No ambiguity remains on frontier-vs-deterministic responsibilities or dev-vs-prod availability semantics.

## P0 SIM2 Post-Implementation Shortfall Remediation (Execution Priority)

Research bundle:
1. [`docs/research/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md`](../docs/research/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md)
2. [`docs/research/2026-02-27-sim2-shortfall-2-coverage-contract-governance.md`](../docs/research/2026-02-27-sim2-shortfall-2-coverage-contract-governance.md)
3. [`docs/research/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md`](../docs/research/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md)
4. [`docs/research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md`](../docs/research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md)
5. [`docs/research/2026-02-27-sim2-architecture-shortfall-orchestration-capability.md`](../docs/research/2026-02-27-sim2-architecture-shortfall-orchestration-capability.md)

Plan bundle:
1. [`docs/plans/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement-plan.md`](../docs/plans/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement-plan.md)
2. [`docs/plans/2026-02-27-sim2-shortfall-2-coverage-contract-governance-plan.md`](../docs/plans/2026-02-27-sim2-shortfall-2-coverage-contract-governance-plan.md)
3. [`docs/plans/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism-plan.md`](../docs/plans/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism-plan.md)
4. [`docs/plans/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity-plan.md`](../docs/plans/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity-plan.md)
5. [`docs/plans/2026-02-27-sim2-orchestration-capability-architecture-plan.md`](../docs/plans/2026-02-27-sim2-orchestration-capability-architecture-plan.md)

### SIM2-ARCH: Functional Orchestration and Capability-by-Construction Uplift

Status: Execution complete on 2026-02-27. See `todos/completed-todo-history.md` for `SIM2-ARCH-*` closure details.

## P0 SIM2 Excellence Remediation Wave 2 (Architecture + Adversary Evolution)

Objective: close all remaining SIM2 excellence gaps so Shuma behaves as a real adversary-vs-defender evolution system, not only a deterministic conformance suite.

Non-negotiable implementation demands for every `SIM2-EX*` item:
1. Preserve strict separation of concerns and functional-core boundaries; policy decision modules must remain side-effect free.
2. Preserve black-box trust boundaries; attacker plane must not gain privileged credentials, headers, endpoints, or secret material.
3. Prefer pure functions and typed data-flow over centralized imperative orchestration.
4. Keep deterministic replay as the blocking oracle while strengthening adaptive discovery.
5. Keep Makefile as the single contributor workflow surface (`make setup`, `make test`, `make build`, focused `make test-adversarial-*` targets).
6. Require explicit rollback notes, security impact notes, and resource-impact notes for each merged slice.
7. Merge only atomic slices with passing required verification for that slice.
8. Update docs/ADRs/module boundaries whenever architecture or trust-boundary semantics change.

### SIM2-EX1: Complete Functional-Core Migration and Decompose Imperative Hot Paths

Scope: remove remaining centralized imperative seams from request orchestration and finish migration to explicit `facts -> decisions -> effects -> response` flow.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-EX1-*` closure details.

### SIM2-EX2: Enforce Least-Authority Capability-by-Construction Across Privileged Effects

Scope: replace coarse capability minting with explicit least-authority capability sets and ensure privileged operations are impossible without capability possession.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-EX2-*` closure details.

### SIM2-EX3: Increase Black-Box Realism by Removing Per-Scenario Control-Plane Preconditioning

Scope: keep deterministic reproducibility while reducing artificial scenario-by-scenario config patching that weakens “real attacker” fidelity.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-EX3-*` closure details.

### SIM2-EX4: Deliver True Browser-Executed “Browser Realistic” Drivers

Scope: replace HTTP emulation for browser-realistic cohorts with actual browser execution semantics.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-EX4-*` closure details.

### SIM2-EX5: Upgrade Frontier Discovery from Advisory Probe to Adaptive Attack Generation Program

Scope: evolve frontier lane from provider-health probing and scenario metadata packaging into a structured adaptive discovery loop that still defers blocking authority to deterministic replay.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-EX5-*` closure details.

### SIM2-EX6: Deepen Coverage Contract Governance to Enforce Full Plan Intent

Scope: close contract-depth gaps so full coverage reflects true plan-row enforcement, including tarpit progression depth and event-stream health quality.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-EX6-*` closure details.

### SIM2-EX7: Harden Simulation-Telemetry Secret Ergonomics Without Weakening Security

Scope: preserve fail-closed sim-tag authenticity while removing setup sharp edges for local development and CI reliability.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-EX7-*` closure details.

### SIM2-EX8: Establish Continuous Defender-Adversary Evolution Loop as First-Class Program

Scope: operationalize SIM2 so defense tuning and adversary evolution become an explicit closed-loop engineering process.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-EX8-*` closure details.

## P0 SIM2 Gap-Closure Program: Real Execution + Realtime Monitoring

Objective: close the remaining mismatch between SIM2 intent and runtime behavior so adversary simulation produces real bot traffic against real defenses and those effects are visible in monitoring/IP-ban workflows in near-real time.

Non-negotiable delivery rules for every `SIM2-GC-*` slice:
1. Do not synthesize monitoring-only data; only persist telemetry generated by actual request handling and defense execution.
2. Keep strict trust boundaries: attacker plane stays black-box and never receives privileged secrets, headers, or control-plane capabilities.
3. Preserve functional-core separation (`facts -> decisions -> effects`) and keep policy logic pure where possible.
4. Keep deterministic replay as release-blocking oracle; adaptive/frontier lanes remain discovery inputs until deterministically confirmed.
5. Maintain dev/prod isolation without introducing extra simulation-only namespaces.
6. Each slice must ship atomically with tests, docs, and explicit security/resource impact notes.

### SIM2-GCR: Mandatory Research Program for Gap-Closure Execution

Scope: run rigorous research before implementing high-risk SIM2 gap-closure slices so architecture choices are evidence-backed, especially for UI-triggered black-box LLM adversary control, Rust realtime monitoring, and telemetry retention/cost/security posture.

Execution order (must be followed):
1. `SIM2-GCR-1` UI toggle orchestration architecture (completed 2026-02-28).
2. `SIM2-GCR-3` UI-toggle trust-boundary controls (completed 2026-02-28).
3. `SIM2-GCR-2` containerized black-box capability orchestration (completed 2026-02-28).
4. `SIM2-GCR-4` Rust realtime monitoring architecture candidates (completed 2026-02-28).
5. `SIM2-GCR-9` Rust prototype/benchmark comparison for realtime candidates (completed 2026-02-28).
6. `SIM2-GCR-5` telemetry retention/storage lifecycle best practices (completed 2026-02-28).
7. `SIM2-GCR-6` monitoring cost-efficiency patterns (completed 2026-02-28).
8. `SIM2-GCR-7` telemetry/adversary-artifact security and privacy controls (completed 2026-02-28).
9. `SIM2-GCR-10` ADR-backed architecture decision capture (completed 2026-02-28).
10. `SIM2-GCR-8` synthesize outcomes into implementation plans and TODO updates (completed 2026-02-28).

Per-track workflow (must be followed before marking each track complete):
1. Publish a dated research doc in `docs/research/` with sources, options considered, and decision matrix.
2. Publish a dated implementation plan in `docs/plans/` derived from that research.
3. Update `todos/todo.md` to reflect plan-derived changes (tightened acceptance criteria, reordered execution where needed, and any new required todos).
4. Only then mark that `SIM2-GCR-*` track complete and move to the next ordered track.


### SIM2-GC-1: Define End-to-End Contract for “Real Adversary Traffic”

Scope: codify exactly what qualifies as real simulated adversary execution and what telemetry must exist when it runs.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-1-*` closure details.

### SIM2-GC-2: Re-architect Host Orchestration into Capability-Gated Functional Flow

Scope: remove remaining centralized imperative orchestration seams that let traffic/reporting drift apart.
ADR reference: [`docs/adr/0007-adversary-sim-toggle-command-controller.md`](../docs/adr/0007-adversary-sim-toggle-command-controller.md)

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-2-*` closure details.

### SIM2-GC-3: Fix Runtime Toggle/Session Lifecycle So Traffic Persists Beyond Auto-Off

Scope: ensure auto-off terminates generation only, not observability of already-generated traffic.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-3-*` closure details.

### SIM2-GC-4: Guarantee Monitoring Ingest Uses Real Request Pipeline by Default

Scope: make runtime telemetry emission mandatory and uniform for SIM and non-SIM traffic in dev environment.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-4-*` closure details.

### SIM2-GC-5: Remove Simulation Telemetry Namespace Architecture Completely

Scope: simplify data model to dev/prod separation only, with no separate SIM namespace semantics.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-5-*` closure details.

### SIM2-GC-6: Deliver Realtime Monitoring Refresh Semantics and Backpressure Safety

Scope: ensure monitoring and IP-ban views reflect new activity quickly in both dev and production (simulated and real traffic) without destabilizing runtime.
ADR reference: [`docs/adr/0008-realtime-monitoring-cursor-sse-hybrid.md`](../docs/adr/0008-realtime-monitoring-cursor-sse-hybrid.md)

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-6-*` closure details.

### SIM2-GC-7: Upgrade Browser-Adversary Lane to True Browser Execution

Scope: ensure “browser realistic” scenarios are executed by real browser runtime and can trigger browser-only defenses.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-7-*` closure details.

### SIM2-GC-8: Containerized Frontier Integration as Real Actor (Not Metadata Generator)

Scope: ensure frontier-model-driven adversary lane produces concrete HTTP/browser actions through constrained containerized actors.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-8-*` closure details.

- [x] SIM2-GC-8-1 Define frontier action contract (`allowed tools`, `network constraints`, `time/resource budgets`, `forbidden data access`).
- [x] SIM2-GC-8-2 Define reject-by-default action grammar/DSL and validation engine so only explicitly permitted action types are executable.
- [x] SIM2-GC-8-3 Implement container execution path that converts model output to validated actionable steps, then executes against target endpoints.
- [x] SIM2-GC-8-4 Enforce egress allowlist and capability boundaries at runtime with explicit deny/audit paths for policy violations.
- [x] SIM2-GC-8-5 Add strict sanitization/validation so unsafe or out-of-policy model outputs are rejected before execution.
- [x] SIM2-GC-8-6 Add negative-path security tests (secret-exfiltration canaries, out-of-scope URL attempts, privileged header injection attempts, replay envelope misuse).
- [x] SIM2-GC-8-7 Add trace lineage from model suggestion -> executed action -> runtime telemetry -> monitoring view.
- [x] SIM2-GC-8-8 Add degraded-mode behavior for key outages that remains explicit, does not fake execution success, and surfaces degraded state within one monitoring refresh/stream cycle.
- [x] SIM2-GC-8-9 Add operator kill-switch and deterministic emergency stop flow for active frontier runs with `p95 <= 10s` stop-latency target.
- [x] SIM2-GC-8-10 Enforce hardened container runtime profile for frontier workers (`non-root/rootless`, `no_new_privileges`, capability allowlist only, read-only rootfs with explicit scratch mounts, no privileged mode/host namespace joins).
- [x] SIM2-GC-8-11 Block sensitive host-control surfaces by policy (forbid daemon-socket mounts and disallowed host bind mounts; fail launch when isolation profile is violated).
- [x] SIM2-GC-8-12 Implement signed host-issued capability envelopes for executable worker actions (`run_id`, `step_id`, action scope, nonce, `issued_at`, `expires_at`, `key_id`) with strict signature/expiry/replay validation.
- [x] SIM2-GC-8-13 Implement bounded one-way command channel semantics (host -> worker command queue with backpressure; worker output restricted to append-only evidence/events without control-plane mutation rights).
- [x] SIM2-GC-8-14 Implement deterministic fail-closed teardown contract (hard runtime deadline, heartbeat timeout, forced process-tree kill, and terminal run-failed semantics on teardown failure).
- [x] SIM2-GC-8-15 Add lifecycle cleanup policy for frontier run artifacts/resources (TTL-driven cleanup, bounded retention, and explicit cleanup failure diagnostics).

### SIM2-GC-9: Scenario Design Realism and Defense Exercise Guarantees

Scope: ensure scenario catalog consistently drives targeted defenses under realistic attacker progression.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-9-*` closure details.

### SIM2-GC-10: Dashboard UX for Arms-Race Operations (Evidence-First)

Scope: make monitoring UI operationally useful for defense evolution loops.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-10-*` closure details.

### SIM2-GC-11: Verification Suite Expansion for End-to-End Truthfulness

Scope: enforce non-regression with tests that prove real traffic -> real defenses -> real monitoring visibility.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-11-*` closure details.

### SIM2-GC-12: Program Governance for Continuous Defense Evolution

Scope: operationalize a repeatable closed-loop process using SIM2 as real adversary pressure.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-12-*` closure details.

### SIM2-GC-13: Remove Adversary Sim Progress Bar and Eliminate Dead UI Runtime Paths

Scope: remove the top progress bar UX that implies choreographed sequence progression and keep only clear ON/OFF + lifecycle state semantics.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-13-*` closure details.

### SIM2-GC-14: Formalize Hybrid Adversary Model (Deterministic Oracle + Emergent Exploration)

Scope: resolve ambiguity between choreographed simulation and emergent adversary behavior by defining and enforcing a two-lane architecture with explicit promotion bridge.

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-14-*` closure details.

### SIM2-GC-15: Telemetry Retention Lifecycle Determinism and Health Visibility

Scope: enforce deterministic retention/purge semantics for monitoring/event telemetry without read-path scan amplification.
ADR reference: [`docs/adr/0009-telemetry-lifecycle-retention-cost-security.md`](../docs/adr/0009-telemetry-lifecycle-retention-cost-security.md)

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-15-*` closure details.

### SIM2-GC-16: Monitoring Cost Governance and Resource Efficiency Envelope

Scope: enforce layered cost controls across telemetry ingest, storage, query, and transport while preserving security-critical evidence integrity.
ADR reference: [`docs/adr/0009-telemetry-lifecycle-retention-cost-security.md`](../docs/adr/0009-telemetry-lifecycle-retention-cost-security.md)

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-16-*` closure details.

### SIM2-GC-17: Telemetry and Adversary-Artifact Security/Privacy-by-Construction

Scope: enforce classification, minimization, pseudonymization, and incident-response controls so telemetry/artifacts cannot leak secrets or over-retain sensitive data by default.
ADR reference: [`docs/adr/0009-telemetry-lifecycle-retention-cost-security.md`](../docs/adr/0009-telemetry-lifecycle-retention-cost-security.md)

Status: Execution complete on 2026-02-28. See `todos/completed-todo-history.md` for `SIM2-GC-17-*` closure details.

## P0 SIM2 Round 4 Stabilization: Monitoring Truthfulness + UX Consistency

Objective: fix post-SIM2 regressions where monitoring appears disabled/unpopulated, refresh controls do not function, adversary simulation traffic is not visible in monitoring, and monitoring filter controls drift from shared dashboard design conventions.

Non-negotiable delivery rules for every `SIM2-R4-*` slice:
1. Monitoring must represent real request-pipeline telemetry and must not rely on synthetic chart/event entries.
2. Monitoring data fetch/render behavior must not depend on whether adversary simulation is enabled.
3. Historical data must remain visible while new live activity appends in near real time.
4. Dashboard UI controls must reuse canonical shared styles/components; one-off styling is not acceptable.
5. Config-seeding lifecycle decisions must be explicit, documented, and consistent across setup/dev/prod paths.

### SIM2-R4-1: Restore Monitoring Initial Load and Refresh Control Correctness
- [ ] SIM2-R4-1-1 Fix monitoring page bootstrap so charts/recent events initialize populated from the latest available snapshot on first load (without requiring adversary sim toggle-on).
- [ ] SIM2-R4-1-2 Fix auto-refresh toggle semantics so enabling/disabling refresh actually starts/stops polling and updates view state deterministically.
- [ ] SIM2-R4-1-3 Fix manual refresh semantics so button clicks trigger immediate reload when auto-refresh is off and do not no-op.
- [ ] SIM2-R4-1-4 Ensure loading/empty/error states are explicit and recoverable (no stuck disabled/unpopulated state after transient failures).

Acceptance criteria:
- Monitoring page renders usable initial data and controls on first entry in dev and prod mode with adversary sim disabled.
- Auto-refresh and manual refresh both execute the same validated fetch/update path and visibly update last-refresh state.
- Dashboard unit + e2e coverage proves refresh controls are behaviorally effective and non-no-op.

### SIM2-R4-2: Decouple Monitoring Render Pipeline from Adversary-Sim Toggle State
- [ ] SIM2-R4-2-1 Remove any runtime/dashboard gating that suppresses monitoring fetch/render unless adversary sim is enabled.
- [ ] SIM2-R4-2-2 Preserve historical telemetry visibility while appending newly ingested telemetry points without wiping history.
- [ ] SIM2-R4-2-3 Validate cursor/SSE/polling interplay so real-time updates continue without requiring toggle transitions.

Acceptance criteria:
- Monitoring charts and recent events update from real incoming traffic regardless of adversary-sim toggle state.
- Historical baseline remains visible and new points/events append in-order with no duplicate or dropped-window regressions.
- Integration tests prove monitoring updates under normal traffic generation with adversary sim both OFF and ON.

### SIM2-R4-3: Prove Adversary-Simulation Traffic Is Real, Generated, and Observable End-to-End
- [ ] SIM2-R4-3-1 Verify adversary-sim execution path emits real HTTP/browser requests through the same request pipeline used for organic traffic.
- [ ] SIM2-R4-3-2 Ensure emitted telemetry from adversary-sim traffic reaches monitoring ingest, chart aggregation, and recent-events feeds.
- [ ] SIM2-R4-3-3 Add diagnostics path for “sim enabled but no traffic generated” so operators receive explicit cause/reason instead of silent success.

Acceptance criteria:
- Enabling adversary sim produces measurable request/event deltas visible in monitoring within one refresh interval/SSE cycle.
- Recent events and chart series show adversary-sim-attributed activity alongside non-sim traffic without synthetic-only artifacts.
- End-to-end verification (`make test` path + focused SIM2 monitoring checks) fails if sim run does not produce observable telemetry.

### SIM2-R4-4: Reassess and Correct KV Config-Seed Lifecycle Boundaries (`setup` vs `dev`/`run`)
- [ ] SIM2-R4-4-1 Decide and document intended lifecycle: which entry points may seed/backfill defaults, and which must be read-only.
- [ ] SIM2-R4-4-2 Ensure `make dev`/watch restarts avoid unnecessary reseed/update churn when KV config is already present and schema-complete.
- [ ] SIM2-R4-4-3 Ensure production-oriented start paths do not silently mutate persistent config unless explicitly requested by operator workflow.
- [ ] SIM2-R4-4-4 Align `Makefile` help text, setup docs, and operational runbooks with the final lifecycle semantics.

Acceptance criteria:
- Startup logs and behavior match documented lifecycle policy for `make setup`, `make dev`, `make run`, and production entrypoints.
- KV seed/backfill runs are idempotent, bounded, and only occur on the intended commands/conditions.
- Regression tests cover both missing-KV bootstrap path and existing-KV no-op path.

### SIM2-R4-5: Enforce Monitoring-Page UI Control Style Parity with Canonical Dashboard Design System
- [ ] SIM2-R4-5-1 Replace monitoring recent-events field/select controls that diverge from shared styling with canonical reusable controls/classes.
- [ ] SIM2-R4-5-2 Remove duplicated/ad-hoc local CSS rules for those controls; reuse existing design tokens/patterns from shared dashboard style surfaces.
- [ ] SIM2-R4-5-3 Add dashboard regression coverage (unit/visual/e2e as appropriate) that detects style/structure drift for monitoring form controls.

Acceptance criteria:
- Monitoring recent-events controls visually and behaviorally match canonical dashboard form controls.
- No new one-off control style rules are introduced where an existing shared style/component already exists.
- Documentation/policy references updated so future agents/operators understand the style-reuse requirement.

## P0 CI + E2E Stability (Top Priority)
- [ ] CI-E2E-1 Resume point for next Codex session: start from `scripts/tests/run_dashboard_e2e.sh`, `scripts/tests/verify_playwright_launch.mjs`, `playwright.config.mjs`, `Makefile` (`test-dashboard-e2e`), and `e2e/run_dashboard_e2e.unit.test.js`; run `make dev` (terminal 1) plus `make test-dashboard-e2e` (terminal 2) and capture per-stage timings (unit, bundle budget, seed, preflight, Playwright) to prove there is no loop/stall; then run `DEBUG=pw:browser corepack pnpm exec node scripts/tests/verify_playwright_launch.mjs` to diagnose Chromium launch path and fix root cause so browser e2e runs without `PLAYWRIGHT_SANDBOX_ALLOW_SKIP`; finally, harden CI behavior so skip mode is never silently used in mandatory checks, retries are bounded and deterministic, and acceptance criteria are met: full `make test` completes in bounded time, Chromium e2e actually executes, and every failing step returns actionable diagnostics rather than hanging.

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

## P0 Adversarial Traffic Simulation Program
Reference plan: [`docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](../docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)
Refinement plan: [`docs/plans/2026-02-26-adversarial-simulation-v2-plan.md`](../docs/plans/2026-02-26-adversarial-simulation-v2-plan.md)

## P1 Dashboard IA: Promote Rate Limiting and GEO to Top-Level Tabs
- [ ] DSH-RG-1 Define dashboard information architecture update and tab order for new top-level `Rate Limiting` and `GEO` tabs (including hash-route mapping and back/forward behavior).
- [ ] DSH-RG-2 Move Rate Limiting controls from Config into a dedicated top-level `Rate Limiting` tab while preserving existing design language, save behavior, and validation rules.
- [ ] DSH-RG-3 Move GEO controls from Config into a dedicated top-level `GEO` tab while preserving existing design language, save behavior, and validation rules.
- [ ] DSH-RG-4 Keep config persistence and dirty-state semantics correct after the split (cross-tab unsaved summary, invalid-field tracking, and section-local warnings).
- [ ] DSH-RG-5 Update status/monitoring cross-navigation so operators can jump directly into the new `Rate Limiting` and `GEO` tabs for tuning.
- [ ] DSH-RG-6 Add/refresh dashboard unit + e2e coverage for tab routing, control enablement/disablement, save flows, and regression of existing config payload shape.
- [ ] DSH-RG-7 Update operator docs and screenshots so tab layout and control locations match the shipped dashboard.

## P1 Akamai Integration Controls Expansion (Rate Limiting + GEO)
- [ ] AK-RG-1 Write a concise architecture note (or ADR if scope broadens) that defines exact semantics for Akamai controls on Rate Limiting and GEO (`off`, `additive`, `authoritative` behavior, precedence, fallback, and trust boundaries).
- [ ] AK-RG-2 Define config surface and naming for Rate/GEO Akamai integration controls, including defaults and whether each is a simple toggle or toggle+mode control.
- [ ] AK-RG-3 Implement admin API + runtime config validation for the new Rate/GEO Akamai controls with explicit guardrails and clear validation errors.
- [ ] AK-RG-4 Implement runtime behavior wiring so Akamai Rate/GEO signals can influence decisions according to the defined mode semantics without bypassing Shuma’s policy ownership.
- [ ] AK-RG-5 Add dashboard controls and help text for Rate/GEO Akamai integration in the new top-level tabs, including disabled-state behavior and operator warnings.
- [ ] AK-RG-6 Add observability and policy-event taxonomy coverage for Rate/GEO Akamai decisions (source, mode, action, fallback reason, and downgrade behavior).
- [ ] AK-RG-7 Add integration/e2e tests for mode precedence, downgrade/fallback safety, and regression against internal-only behavior.
- [ ] AK-RG-8 Document rollout/rollback guidance for enabling Rate/GEO Akamai integration in enterprise deployments, including promotion gates and emergency disable steps.

## P1 Research Dossiers (Paper-by-Paper TODOs)
Completion rule for every paper TODO below: capture key findings, map to `self_hosted_minimal` vs `enterprise_akamai` ownership, and propose concrete Shuma TODO updates.

### Fingerprinting, JS Verification, and CDP-Adjacent Detection
- Completed research tranche (`R-FP-01`..`R-FP-09`) archived in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md) and `todos/completed-todo-history.md`.
- [ ] Run a Finch comparison spike to see if Shuma might benefit from enabling enhancing its internal capabilities with allowing users to integrate finch alongside it(no direct dependency in core runtime).

### Challenges: PoW, Not-a-Bot, and Puzzle Escalation
- [ ] R-CH-01 Review Dwork/Naor, "Pricing via Processing or Combatting Junk Mail" (CRYPTO 1992) and extract adaptive requester-cost principles for modern web bot defence. https://www.microsoft.com/en-us/research/publication/pricing-via-processing-or-combatting-junk-mail/
- [ ] R-CH-02 Review Juels/Brainard, "Client Puzzles" (NDSS 1999) and define stateless verification patterns for Shuma PoW endpoints. https://www.ndss-symposium.org/ndss1999/cryptographic-defense-against-connection-depletion-attacks/
- [ ] R-CH-03 Review Adam Back, "Hashcash: A Denial of Service Counter-Measure" (2002) and assess modern browser-side PoW cost tuning constraints. https://nakamotoinstitute.org/library/hashcash/
- [ ] R-CH-04 Review von Ahn et al., "CAPTCHA: Using Hard AI Problems for Security" (EUROCRYPT 2003) and capture challenge-design principles still valid for the Not-a-Bot checkbox step. https://doi.org/10.1007/3-540-39200-9_18
- [ ] R-CH-05 Review von Ahn et al., "reCAPTCHA: Human-based character recognition via Web security measures" (Science 2008) and extract lessons for useful-human-work and abuse resistance tradeoffs. https://doi.org/10.1126/science.1160379
- [ ] R-CH-06 Review Bursztein et al., "Easy Does It: More Usable CAPTCHAs" (CHI 2014) and derive practical usability thresholds/metrics for Shuma challenge UX. https://doi.org/10.1145/2556288.2557322
- [ ] R-CH-07 Review Golle, "Machine Learning Attacks Against the ASIRRA CAPTCHA" (CCS 2008) and define anti-ML solvability requirements for puzzle challenge variants. https://doi.org/10.1145/1455770.1455838
- [ ] R-CH-08 Review AI_Adaptive_POW (Software Impacts 2022) and evaluate adaptive-difficulty policies for botness-tiered PoW in Shuma. https://doi.org/10.1016/j.simpa.2022.100335
- [ ] R-CH-09 Review Alsuhibany, "A Survey on Adversarial Perturbations and Attacks on CAPTCHAs" (Applied Sciences 2023) and map attack classes to Shuma challenge threat model updates. https://doi.org/10.3390/app13074602
- [ ] R-CH-10 Review Uysal, "Revisiting Text-Based CAPTCHAs" (Electronics 2025) and evaluate current CNN-solvability implications for fallback challenge modes. https://doi.org/10.3390/electronics14224403

### Rate Limiting, Tarpit, and Cost-Imposition
- Research synthesis recorded in [`docs/research/2026-02-22-http-tarpit-cost-shift-research-synthesis.md`](../docs/research/2026-02-22-http-tarpit-cost-shift-research-synthesis.md) (includes source mapping, implementation implications, and cost-shift analysis).
- [ ] OUT-1 Add explicit deployment guardrails that fail when `provider_backends.rate_limiter=external` or `provider_backends.ban_store=external` but required Redis outbound hosts are not allowlisted in `spin.toml` `allowed_outbound_hosts`.
- [ ] OUT-2 Add a provider-to-outbound-requirements matrix in public docs (internal vs external backend, required host capabilities, required outbound host allowlists, fallback behavior).
- [ ] OUT-3 Add integration verification that exercises external Redis provider selection under restricted outbound policy and confirms safe fallback/guardrail behavior is deterministic.
- [ ] OUT-5 Before implementing non-stub `challenge_engine=external` and `maze_tarpit=external`, complete design work for their external transport path through Spin host capabilities or sidecar/adapter boundary, with rollback and security posture defined.
- [ ] (Enterprise/hybrid track) Extend distributed-state monitoring with ban sync-lag metrics (rate-limiter fallback/drift monitoring is implemented).

#### Tarpit Asymmetry Hardening (`work-gated`, `token-chained`, `egress-budgeted`)

Architecture alignment reference: [`docs/plans/2026-02-23-maze-tarpit-architecture-alignment-plan.md`](../docs/plans/2026-02-23-maze-tarpit-architecture-alignment-plan.md)

- [ ] TAH-11 Expand observability: progression admissions/denials, proof verify outcomes, chain violations, bytes sent, duration, budget exhaustion reason, fallback action, and escalation outcomes (including top offender buckets with cardinality guardrails).
- [ ] TAH-12 Add dashboard/admin visibility for the new tarpit progression + egress metrics and operator guidance for safe tuning (recommended starting ranges and rollback thresholds).
- [ ] TAH-19 Before launch, tighten collateral-risk controls (especially bucket-based persistence escalation), then re-evaluate tarpit defaults.

Execution order for remaining tarpit work:
1. `TAH-11`
2. `TAH-12`

### IP Range Policy, Reputation Feeds, and GEO Fencing
- Research synthesis recorded in [`docs/research/archive/2026-02-20-ip-range-policy-research-synthesis.md`](../docs/research/archive/2026-02-20-ip-range-policy-research-synthesis.md) (includes source mapping and implementation implications).
- [ ] R-GEO-01 Review Hu/Heidemann/Pradkin, "Towards Geolocation of Millions of IP Addresses" (IMC 2012) and capture scalability/error-tradeoff implications for GEO policy confidence scoring. https://doi.org/10.1145/2398776.2398790
- [ ] R-GEO-02 Review Dan/Parikh/Davison, "Improving IP Geolocation using Query Logs" (WSDM 2016) and define data-quality assumptions for geo-based enforcement. https://doi.org/10.1145/2835776.2835820
- [ ] R-GEO-03 Review Mazel et al., "Smartphone-based geolocation of Internet hosts" (Computer Networks 2017) and assess delay-model caveats for operational geofencing. https://doi.org/10.1016/j.comnet.2017.02.006
- [ ] R-GEO-04 Review Saxon/Feamster, "GPS-Based Geolocation of Consumer IP Addresses" (2021) and define confidence thresholds for city-level policy decisions. https://arxiv.org/abs/2105.13389

## P1 Distributed State and Limiter Correctness
- [ ] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Close distributed-state correctness remaining risks from `DEP-ENT-*` tasks and promote enterprise authoritative posture only after convergence <abbr title="Service Level Objective">SLO</abbr> evidence is stable.

### P1 Outbound Capability and External Provider Constraints
- [ ] OUT-4 Create an ADR for non-Redis external integrations (for example webhook notifications or cross-service sync) that defines the approved pattern in Spin (`allowed_outbound_hosts` expansion vs sidecar/bridge service).


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
