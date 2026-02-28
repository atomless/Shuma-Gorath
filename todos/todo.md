# TODO Roadmap

Last updated: 2026-02-28

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

Acceptance criteria:
1. Core policy decisions become predominantly pure and testable without side effects.
2. Privileged operations are blocked unless explicit capability objects are present.
3. Characterization parity tests prove behavior stability across extraction slices.
4. `src/lib.rs` orchestration complexity is materially reduced and role-focused.
5. Full required verification (`make test`, `make build`) remains green throughout migration.

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

- [ ] SIM2-EX1-1 Produce an architecture inventory of all remaining direct side-effect callsites in request handling (`metrics`, `monitoring`, `event log`, `ban writes`) and classify each as `retain`, `migrate`, or `delete`.
- [ ] SIM2-EX1-2 Move all remaining request-path side effects still executed directly from `src/lib.rs` into effect-intent execution paths behind typed intents.
- [ ] SIM2-EX1-3 Split `src/runtime/effect_intents.rs` into responsibility-focused modules (`intent_types`, `plan_builder`, `intent_executor`, `response_renderer`) with explicit dependency direction.
- [ ] SIM2-EX1-4 Remove or fully migrate legacy `#[allow(dead_code)]` policy handlers in `src/runtime/policy_pipeline.rs`; keep no dead-code rollback seam in active request path.
- [ ] SIM2-EX1-5 Introduce architectural guard tests/lints that fail if pure decision modules depend on `Store`, provider side effects, event logging, or mutable global state.
- [ ] SIM2-EX1-6 Add characterization parity tests around migrated seams and require parity snapshots before and after each extraction slice.
- [ ] SIM2-EX1-7 Reduce `src/lib.rs` orchestration surface to route setup, trust-boundary setup, and tranche wiring only; move policy behavior decisions out of entrypoint logic.
- [ ] SIM2-EX1-8 Document final orchestration ownership map in `docs/module-boundaries.md` and update ADR references where boundaries changed.

Acceptance criteria:
1. No request-path privileged side effects are emitted directly from `src/lib.rs`; all flow through intent executor boundaries.
2. `src/runtime/policy_pipeline.rs` contains only active graph orchestration paths; legacy dead-code handlers are removed or isolated outside runtime path.
3. Pure decision modules compile and test without KV/provider dependencies.
4. Characterization parity suite shows no unintended behavior drift across extracted slices.
5. `src/lib.rs` becomes a thin orchestration shell with materially reduced complexity and clearly documented responsibilities.
6. `make test-unit`, `make test-integration`, `make test-dashboard-e2e`, `make test`, and `make build` pass after migration slices.
7. Updated docs make dependency direction and side-effect boundaries unambiguous to next contributors.

### SIM2-EX2: Enforce Least-Authority Capability-by-Construction Across Privileged Effects

Scope: replace coarse capability minting with explicit least-authority capability sets and ensure privileged operations are impossible without capability possession.

- [ ] SIM2-EX2-1 Define capability lattice by operation class (`metrics_write`, `monitoring_write`, `event_log_write`, `ban_write`, optional `response_privileged`) and by orchestration phase.
- [ ] SIM2-EX2-2 Replace single coarse `RuntimeCapabilities::for_request_path()` minting with phase-specific capability construction and explicit capability passing per execution step.
- [ ] SIM2-EX2-3 Eliminate direct privileged helper calls that bypass capability checks; route every write path through capability-gated executor APIs.
- [ ] SIM2-EX2-4 Add compile-time sealing for capability constructors so capabilities can only be minted at trust-boundary entrypoints.
- [ ] SIM2-EX2-5 Add negative-path tests proving privileged effects fail/are impossible when capability is absent.
- [ ] SIM2-EX2-6 Add regression tests ensuring no fallback path silently executes privileged writes outside capability-guarded APIs.
- [ ] SIM2-EX2-7 Add architecture assertions (search-based CI guard or compile checks) preventing direct calls to privileged write APIs from disallowed modules.
- [ ] SIM2-EX2-8 Update architecture docs and ADR notes with final capability model and enforcement guarantees.

Acceptance criteria:
1. Privileged side effects are capability-gated everywhere in request path, without convention-only exceptions.
2. Capability minting occurs only at explicit trust boundaries.
3. Least-authority capability scope is demonstrated by tests for each effect class.
4. Missing-capability scenarios fail deterministically and observably.
5. No privileged write API is reachable from pure decision modules.
6. CI guardrails fail fast on capability-bypass regressions.

### SIM2-EX3: Increase Black-Box Realism by Removing Per-Scenario Control-Plane Preconditioning

Scope: keep deterministic reproducibility while reducing artificial scenario-by-scenario config patching that weakens “real attacker” fidelity.

- [ ] SIM2-EX3-1 Define runner execution contract separating `suite_setup`, `attacker_execution`, and `suite_teardown`; forbid control-plane config writes during `attacker_execution`.
- [ ] SIM2-EX3-2 Replace per-scenario `admin_patch` choreography with baseline profile presets loaded before attacker execution starts.
- [ ] SIM2-EX3-3 Add explicit runner guardrail that fails the run if control-plane mutation occurs after attacker phase begins (except approved teardown/reset hooks).
- [ ] SIM2-EX3-4 Rework scenarios so expected defenses are triggered by attacker behavior and traffic progression, not repeated runtime reconfiguration.
- [ ] SIM2-EX3-5 Extend report schema with control-plane mutation audit trail (`count`, `phase`, `reason`) and fail criteria when mutation policy is violated.
- [ ] SIM2-EX3-6 Add deterministic tests for mutation-contract compliance in smoke/coverage profiles.
- [ ] SIM2-EX3-7 Update operator docs to distinguish deterministic reproducibility controls from attacker realism constraints.

Acceptance criteria:
1. During attacker phase, control-plane config mutation count is zero by policy and verified by tests.
2. Coverage profile still passes without per-scenario config patching.
3. Gate failures clearly identify realism-contract violations vs defense regressions.
4. Deterministic reproducibility remains stable across repeated runs with fixed seeds.
5. Black-box realism improves without granting attacker plane privileged controls.

### SIM2-EX4: Deliver True Browser-Executed “Browser Realistic” Drivers

Scope: replace HTTP emulation for browser-realistic cohorts with actual browser execution semantics.

- [ ] SIM2-EX4-1 Define browser-driver architecture (`playwright`/equivalent) with deterministic seed control, bounded runtime, and resource budgets.
- [ ] SIM2-EX4-2 Implement real browser execution path for `browser_realistic` class (navigation, DOM, JS execution, storage/cookie behavior, challenge interaction hooks).
- [ ] SIM2-EX4-3 Keep non-browser drivers for scraper/load cohorts; enforce driver-class-specific capability boundaries and telemetry labels.
- [ ] SIM2-EX4-4 Add browser-lane observability fields (`js_executed`, `dom_events`, `storage_mode`, `challenge_dom_path`) to report evidence.
- [ ] SIM2-EX4-5 Add deterministic replay harness for browser scenarios including strict timeout, retry policy, and anti-flake constraints.
- [ ] SIM2-EX4-6 Add CI-safe fallback semantics only for unsupported environments, with explicit lane status reporting and no silent pass-through.
- [ ] SIM2-EX4-7 Expand E2E/adversarial tests to validate that browser-only defenses are exercised by real browser lanes.

Acceptance criteria:
1. `browser_realistic` scenarios are executed by a real browser runtime, not raw HTTP request emulation.
2. Browser-only defense surfaces (JS verification/CDP/client-runtime checks) are exercised with explicit evidence in reports.
3. Browser lane remains deterministic enough for CI gating within bounded flake tolerance and declared retry policy.
4. Fallback behavior is explicit and cannot silently mask missing browser execution.
5. Required Makefile gates remain bounded and pass on supported CI lanes.

### SIM2-EX5: Upgrade Frontier Discovery from Advisory Probe to Adaptive Attack Generation Program

Scope: evolve frontier lane from provider-health probing and scenario metadata packaging into a structured adaptive discovery loop that still defers blocking authority to deterministic replay.

- [ ] SIM2-EX5-1 Define attack-generation contract for frontier lane (`objective`, `constraints`, `allowed actions`, `forbidden data`, `resource budgets`, `novelty expectations`).
- [ ] SIM2-EX5-2 Implement candidate generation pipeline that proposes new attack variants/mutations instead of only rewrapping existing deterministic scenarios.
- [ ] SIM2-EX5-3 Add diversity scoring (`cross-provider agreement`, `novelty`, `behavioral class coverage`) with deterministic normalization for triage.
- [ ] SIM2-EX5-4 Add automatic sanitization and governance checks for generated payloads before any replay/promotion path.
- [ ] SIM2-EX5-5 Upgrade promotion pipeline to ingest generated candidates, replay them deterministically, and produce lineage from `generated candidate -> deterministic confirmation -> promoted scenario`.
- [ ] SIM2-EX5-6 Add protected-lane metrics for discovery quality (`candidate count`, `novel confirmed regressions`, `false discovery rate`, `provider outage impact`).
- [ ] SIM2-EX5-7 Keep blocking policy deterministic: no stochastic frontier output can block release without deterministic confirmation.
- [ ] SIM2-EX5-8 Publish operator workflow for evaluating and curating generated candidates into canonical manifests.

Acceptance criteria:
1. Frontier lane produces novel candidate attacks beyond existing deterministic scenario catalog.
2. All promoted regressions show deterministic confirmation lineage.
3. Governance/redaction checks remain enforced and audited before replay.
4. Release-blocking semantics remain deterministic and policy-stable.
5. Operators can track discovery efficacy with explicit quality metrics, not only provider-health status.

### SIM2-EX6: Deepen Coverage Contract Governance to Enforce Full Plan Intent

Scope: close contract-depth gaps so full coverage reflects true plan-row enforcement, including tarpit progression depth and event-stream health quality.

- [ ] SIM2-EX6-1 Define `coverage_contract.v2` with explicit minima for currently under-specified plan intents (including tarpit progression and event-stream health depth metrics).
- [ ] SIM2-EX6-2 Add schema migration and compatibility handling for contract v1/v2 while pre-launch migration completes.
- [ ] SIM2-EX6-3 Add strict drift checks among plan rows, manifest expectations, runner extracted metrics, and contract requirements.
- [ ] SIM2-EX6-4 Extend gate diagnostics with row-level failure output showing `required`, `observed`, `missing evidence`, and scenario contribution mapping.
- [ ] SIM2-EX6-5 Add focused tests for each new v2 coverage key and threshold boundary behavior.
- [ ] SIM2-EX6-6 Wire v2 governance into mandatory Makefile and CI coverage gates with fail-fast messaging.
- [ ] SIM2-EX6-7 Update docs/runbooks with contract evolution protocol and backwards-compatibility removal date.

Acceptance criteria:
1. Canonical coverage contract enforces every required plan-row intent with explicit measurable thresholds.
2. Tarpit progression and event-stream health rows cannot pass with shallow/partial evidence.
3. Drift across plan/manifest/runner/contract fails deterministically with actionable output.
4. Coverage contract versioning and migration are documented and test-backed.
5. Mandatory coverage gates continue to run via canonical Makefile paths.

### SIM2-EX7: Harden Simulation-Telemetry Secret Ergonomics Without Weakening Security

Scope: preserve fail-closed sim-tag authenticity while removing setup sharp edges for local development and CI reliability.

- [ ] SIM2-EX7-1 Add `make setup` and `make verify` checks that guarantee `SHUMA_SIM_TELEMETRY_SECRET` is created, non-placeholder, and surfaced clearly to operators.
- [ ] SIM2-EX7-2 Add explicit adversarial preflight command/target that validates all required secrets and prints actionable remediation before runner execution.
- [ ] SIM2-EX7-3 Add CI workflow explicit env wiring for `SHUMA_SIM_TELEMETRY_SECRET` in lanes that run adversarial coverage/promote jobs.
- [ ] SIM2-EX7-4 Improve runner failure diagnostics with structured, copy-paste-safe setup guidance and clear distinction between missing secret vs invalid signature vs replay failure.
- [ ] SIM2-EX7-5 Add docs for local rotation and CI secret lifecycle, including cadence and compromise-response workflow.
- [ ] SIM2-EX7-6 Add automated tests for setup/preflight behavior ensuring missing/placeholder secret states fail early with deterministic guidance.
- [ ] SIM2-EX7-7 Confirm security posture remains fail-closed: no unsigned sim metadata acceptance path is introduced.

Acceptance criteria:
1. Local `make setup` leaves adversarial runs ready by default with valid sim telemetry secret material.
2. CI adversarial lanes explicitly provision required secret env and do not rely on implicit setup state.
3. Missing/invalid secret states fail before scenario execution with clear remediation output.
4. No change weakens sim-tag authenticity enforcement or introduces permissive bypass.
5. Operator docs clearly define setup, rotation, and incident-response steps.

### SIM2-EX8: Establish Continuous Defender-Adversary Evolution Loop as First-Class Program

Scope: operationalize SIM2 so defense tuning and adversary evolution become an explicit closed-loop engineering process.

- [ ] SIM2-EX8-1 Define canonical cycle contract: `run adversary -> analyze failures -> tune defenses -> replay -> promote scenarios -> repeat`.
- [ ] SIM2-EX8-2 Add report diff tooling that highlights defense deltas between runs (new passes, new regressions, cost shifts, collateral changes).
- [ ] SIM2-EX8-3 Add backlog automation guidance for converting confirmed novel regressions into prioritized implementation todos with ownership and SLA.
- [ ] SIM2-EX8-4 Add promotion hygiene rules so stale scenarios are retired, merged, or reclassified with explicit rationale.
- [ ] SIM2-EX8-5 Define excellence KPIs for the loop (`time to regression confirmation`, `time to mitigation`, `collateral ceiling`, `cost asymmetry trend`) and expose them in operator docs.
- [ ] SIM2-EX8-6 Add governance checkpoint requiring periodic architecture review against this cycle contract and documented outcomes.

Acceptance criteria:
1. Shuma has a documented and testable closed-loop process for adversary-driven defense evolution.
2. Novel regressions move from discovery to deterministic confirmation to TODO execution without manual ambiguity.
3. Scenario corpus quality is maintained through promotion and retirement rules.
4. Excellence KPIs are measurable, reported, and used for release readiness decisions.
5. The loop preserves core project principles: low human friction, rising attacker cost, bounded defender resource cost.

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
8. `SIM2-GCR-7` telemetry/adversary-artifact security and privacy controls.
9. `SIM2-GCR-10` ADR-backed architecture decision capture.
10. `SIM2-GCR-8` synthesize outcomes into implementation plans and TODO updates.

Per-track workflow (must be followed before marking each track complete):
1. Publish a dated research doc in `docs/research/` with sources, options considered, and decision matrix.
2. Publish a dated implementation plan in `docs/plans/` derived from that research.
3. Update `todos/todo.md` to reflect plan-derived changes (tightened acceptance criteria, reordered execution where needed, and any new required todos).
4. Only then mark that `SIM2-GCR-*` track complete and move to the next ordered track.

- [ ] SIM2-GCR-7 Research security/privacy best practices for telemetry and adversary artifacts (secret-exposure prevention, data minimization, pseudonymization options, artifact retention risk controls, incident-response hooks).
- [ ] SIM2-GCR-8 Produce research synthesis docs and implementation plans for `GC-6`, `GC-8`, `GC-11`, and `GC-14`, then update todos with quantitative thresholds derived from research outcomes.
- [ ] SIM2-GCR-10 Convert selected research outcomes into ADR-backed architecture decisions for: (a) UI-toggle-driven black-box adversary orchestration, (b) monitoring realtime data architecture, and (c) retention/cost/security lifecycle policies.

Acceptance criteria:
1. Each `SIM2-GCR-*` track produces a dated research doc in `docs/research/` with source-backed recommendations.
2. Every research doc includes a decision matrix (`option`, `benefits`, `risks`, `resource cost`, `security impact`, `rollback complexity`).
3. Each track produces an implementation plan in `docs/plans/` mapped to specific `SIM2-GC-*` todos.
4. `GC-6`, `GC-8`, `GC-11`, and `GC-14` acceptance criteria are upgraded with explicit quantitative thresholds before implementation begins.
5. Research outputs explicitly justify why chosen approaches are preferred over rejected alternatives.
6. Realtime monitoring architecture decision is backed by benchmark evidence from `SIM2-GCR-9`, not only qualitative preference.
7. Final selected approaches are codified in ADR artifacts before high-risk implementation slices begin.
8. Each completed research track has linked artifacts in all three layers: research doc, plan doc, and TODO updates.

### SIM2-GC-1: Define End-to-End Contract for “Real Adversary Traffic”

Scope: codify exactly what qualifies as real simulated adversary execution and what telemetry must exist when it runs.

- [ ] SIM2-GC-1-1 Write architecture contract doc that defines required invariants for `traffic source`, `execution lane`, `defense path`, `telemetry emission`, and `monitoring visibility`.
- [ ] SIM2-GC-1-2 Define explicit prohibited patterns (mock telemetry injection, out-of-band metrics writes, control-plane-only “success” signals).
- [ ] SIM2-GC-1-3 Define evidence schema for each run (`request id lineage`, `scenario id`, `lane`, `defenses touched`, `decision outcomes`, `latency/cost`).
- [ ] SIM2-GC-1-4 Add contract tests that fail if runner marks scenario success without corresponding runtime telemetry evidence.
- [ ] SIM2-GC-1-5 Publish operator-facing definition of done for SIM runs (what must appear in Monitoring and IP Ban views).
- [ ] SIM2-GC-1-6 Extend evidence schema with control-plane lineage fields (`control_operation_id`, `requested_state`, `desired_state`, `actual_state`, `actor/session`) for toggle-driven orchestration traceability.

Acceptance criteria:
1. Contract exists in docs and is referenced by runner, runtime, and dashboard modules.
2. Any run lacking required telemetry evidence is marked invalid/failed.
3. Architecture docs clearly separate “executed traffic” from “report-only metadata.”
4. Contributors can no longer pass SIM coverage with synthetic-only monitoring artifacts.

### SIM2-GC-2: Re-architect Host Orchestration into Capability-Gated Functional Flow

Scope: remove remaining centralized imperative orchestration seams that let traffic/reporting drift apart.

- [ ] SIM2-GC-2-1 Refactor host orchestration into explicit phases: `plan`, `execute`, `collect evidence`, `publish report`.
- [ ] SIM2-GC-2-2 Require capability tokens for privileged operations (config mutation, telemetry writes, admin APIs) and forbid implicit fallbacks.
- [ ] SIM2-GC-2-3 Move phase decision logic into pure functions with typed inputs/outputs and side effects only in executor boundary.
- [ ] SIM2-GC-2-4 Add characterization tests proving behavior parity before/after extraction and proving no telemetry side path bypasses runtime flow.
- [ ] SIM2-GC-2-5 Update module-boundary docs with explicit dependency direction and trust-boundary ownership.
- [ ] SIM2-GC-2-6 Introduce explicit command contract for adversary toggle control (`operation_id`, idempotency key semantics, requested/accepted state model).
- [ ] SIM2-GC-2-7 Separate desired lifecycle state from actual lifecycle state and move reconciliation authority out of read-path status handlers.
- [ ] SIM2-GC-2-8 Add controller lease/fencing ownership model so only one reconciler can mutate adversary lifecycle state at a time.
- [ ] SIM2-GC-2-9 Add endpoint-specific trust-boundary gate for adversary control submissions (`admin auth`, `csrf token`, strict `origin/referer`, and fetch-metadata policy for unsafe methods).
- [ ] SIM2-GC-2-10 Implement payload-bound idempotency replay policy (`Idempotency-Key` required, actor/session scoping, canonical payload hash binding, deterministic TTL expiry behavior).
- [ ] SIM2-GC-2-11 Add control-plane abuse throttling envelope (per-session and per-IP ceilings, bounded debounce/queue semantics, explicit throttled outcomes).
- [ ] SIM2-GC-2-12 Add structured control-operation security audit schema (`operation_id`, actor/session, decision, reason, origin verdict, idempotency-hash) with sensitive-field redaction rules.

Acceptance criteria:
1. Orchestration no longer has hidden write paths that can fabricate monitoring outcomes.
2. Privileged effects are impossible without capability possession.
3. Pure-policy modules have no direct dependency on storage/providers.
4. Control submissions fail closed on trust-boundary violations (auth/CSRF/origin/replay abuse) with explicit reason taxonomy.
5. Control operations are audit-complete and incident-reconstructable without exposing sensitive material.
6. Tests prove evidence publication is coupled to actual execution outputs.

### SIM2-GC-3: Fix Runtime Toggle/Session Lifecycle So Traffic Persists Beyond Auto-Off

Scope: ensure auto-off terminates generation only, not observability of already-generated traffic.

- [ ] SIM2-GC-3-1 Split lifecycle semantics into `generation active` vs `historical data visible`.
- [ ] SIM2-GC-3-2 Ensure toggle auto-off only stops producers and does not delete or hide prior records from monitoring queries.
- [ ] SIM2-GC-3-3 Add explicit retention controls for dev telemetry history with safe defaults and cleanup commands.
- [ ] SIM2-GC-3-4 Add regression tests for “run -> auto-off -> refresh monitoring” showing historical adversary traffic remains visible.
- [ ] SIM2-GC-3-5 Update UI copy to communicate active-state vs retained-history semantics.

Acceptance criteria:
1. Operators can inspect SIM-generated defense events after auto-off without rerunning simulation.
2. No monitoring view silently filters out prior SIM traffic solely because session ended.
3. Retention behavior is deterministic, documented, and tested.

### SIM2-GC-4: Guarantee Monitoring Ingest Uses Real Request Pipeline by Default

Scope: make runtime telemetry emission mandatory and uniform for SIM and non-SIM traffic in dev environment.

- [ ] SIM2-GC-4-1 Audit all monitoring emitters and remove SIM-specific alternative emit paths that bypass request processing.
- [ ] SIM2-GC-4-2 Ensure adversary requests traverse the same defense middleware/pipeline used for ordinary traffic.
- [ ] SIM2-GC-4-3 Add per-defense telemetry assertions for PoW, challenge, maze, honeypot, CDP, rate-limit, and GEO decisions.
- [ ] SIM2-GC-4-4 Add integration tests that run SIM traffic and assert monitoring counters/events increase through standard endpoints.
- [ ] SIM2-GC-4-5 Add “no-op defense” detector tests that fail if a configured defense never emits events under targeted scenario load.

Acceptance criteria:
1. SIM traffic hits the same runtime defense stack as real traffic.
2. Monitoring data for SIM runs comes from normal runtime telemetry, not synthetic ingestion.
3. Missing per-defense signals under expected attack scenarios fails tests.
4. Operators can refresh monitoring and immediately observe defense activity deltas.

### SIM2-GC-5: Remove Simulation Telemetry Namespace Architecture Completely

Scope: simplify data model to dev/prod separation only, with no separate SIM namespace semantics.

- [ ] SIM2-GC-5-1 Remove simulation namespace config flags, query paths, schema branches, and UI toggles.
- [ ] SIM2-GC-5-2 Consolidate dev telemetry queries so SIM and manual dev traffic coexist in same dev plane with source labels.
- [ ] SIM2-GC-5-3 Preserve source attribution fields (`origin=sim|manual|other`) for filtering without namespace-level partitioning.
- [ ] SIM2-GC-5-4 Add migration/compat tests to ensure old namespace settings are rejected or ignored safely in pre-launch mode.
- [ ] SIM2-GC-5-5 Update docs and runbooks to remove all namespace-era instructions and diagrams.

Acceptance criteria:
1. No runtime, dashboard, or docs references remain to simulation telemetry namespace.
2. Dev/prod split is the only data-plane separation model.
3. Source filtering remains possible without schema or storage partition complexity.
4. Cleanup leaves no dead config/code remnants.

### SIM2-GC-6: Deliver Realtime Monitoring Refresh Semantics and Backpressure Safety

Scope: ensure monitoring and IP-ban views reflect new activity quickly in both dev and production (simulated and real traffic) without destabilizing runtime.

- [ ] SIM2-GC-6-1 Define quantitative freshness SLOs for runtime-dev and runtime-prod (`p50/p95/p99 visibility delay`, `manual refresh staleness bound`, `max allowed lag before degraded state`).
- [ ] SIM2-GC-6-2 Define and enforce a load envelope for freshness SLO compliance (event ingest rate, operator refresh concurrency, query cost ceiling) with benchmark methodology.
- [ ] SIM2-GC-6-3 Implement selected realtime delivery model (from `SIM2-GCR-4`) with deterministic ordering, cursor semantics, and bounded payload windows.
- [ ] SIM2-GC-6-4 Add cache invalidation rules so high-signal events (new bans, challenge failures, maze escalations) invalidate stale views immediately without cache stampede behavior.
- [ ] SIM2-GC-6-5 Add backend and UI rate-limit/backpressure controls to avoid self-induced load from aggressive refresh loops.
- [ ] SIM2-GC-6-6 Add tests for freshness, monotonic ordering, deduplication, and behavior under bursty adversary runs.
- [ ] SIM2-GC-6-7 Add explicit freshness-health telemetry and UI state (`fresh`, `degraded`, `stale`) with operator-facing lag indicators.
- [ ] SIM2-GC-6-8 Replace run-active-only cache-bypass assumptions with a global freshness policy that preserves near-realtime visibility for real production attacker traffic.
- [ ] SIM2-GC-6-9 Define canonical monitoring event cursor contract (strict monotonic sequence id, resume semantics, and overflow taxonomy shared by polling and stream paths).
- [ ] SIM2-GC-6-10 Implement cursor-delta endpoint(s) for monitoring/IP-bans (`after_cursor`, bounded `limit`, `next_cursor`, `has_more`, `overflow`) with deterministic ordering.
- [ ] SIM2-GC-6-11 Add conditional polling optimization (`If-None-Match`/`304`) on cursor-delta reads where unchanged windows can be proven safely.
- [ ] SIM2-GC-6-12 Implement optional SSE delivery path (`text/event-stream`) that reuses the same cursor namespace and supports `Last-Event-ID` resume.
- [ ] SIM2-GC-6-13 Add bounded server-side fan-out buffers/queues and explicit slow-consumer lag signaling (no unbounded memory growth).
- [ ] SIM2-GC-6-14 Update dashboard refresh runtime to prefer cursor path (and SSE when available) with deterministic fallback to polling on stream failure.

Acceptance criteria:
1. Under declared load envelope (`>=1000 events/s`, `>=5 active operator clients`), active live path achieves `p95 <= 300ms` and `p99 <= 500ms` freshness in runtime-dev and runtime-prod verification profiles.
2. Non-degraded active path has zero overflow/drop for monitored events within declared bounded buffer window.
3. Refresh behavior is deterministic (monotonic cursor progression, no silent loss, no duplicate replay beyond documented replay window rules).
4. Backpressure and cache behavior are bounded and benchmarked under expected burst load.
5. Freshness regressions fail automated tests with actionable diagnostics naming violated percentile/budget threshold.
6. Production monitoring freshness is independent of adversary-sim active state.
7. Operators can see explicit freshness health and lag state at all times.
8. Cursor-resume semantics are deterministic across manual refresh, auto-refresh, and reconnect flows.
9. When streaming path is available, active live updates stay within query budget `<=1 request/sec/client` average (excluding initial bootstrap requests); degraded fallback polling above that budget must surface explicit degraded state.

### SIM2-GC-7: Upgrade Browser-Adversary Lane to True Browser Execution

Scope: ensure “browser realistic” scenarios are executed by real browser runtime and can trigger browser-only defenses.

- [ ] SIM2-GC-7-1 Replace HTTP-emulated browser lane with deterministic real-browser driver path.
- [ ] SIM2-GC-7-2 Add challenge interaction primitives (DOM read/write, click/submit flows, storage/session behavior) with strict capability limits.
- [ ] SIM2-GC-7-3 Ensure browser-only defenses (client runtime checks/CDP detections/challenge scripts) emit evidence when exercised.
- [ ] SIM2-GC-7-4 Add anti-flake constraints, retries, and diagnostics that preserve CI reliability while proving real execution occurred.
- [ ] SIM2-GC-7-5 Include per-run browser evidence fields in reports and monitoring correlation IDs.

Acceptance criteria:
1. Browser lane traffic is generated by actual browser runtime, not request emulation.
2. Browser-only defenses register events under targeted scenarios.
3. Evidence ties browser actions to monitoring events and outcomes.
4. CI remains deterministic enough for gated verification.

### SIM2-GC-8: Containerized Frontier Integration as Real Actor (Not Metadata Generator)

Scope: ensure frontier-model-driven adversary lane produces concrete HTTP/browser actions through constrained containerized actors.

- [ ] SIM2-GC-8-1 Define frontier action contract (`allowed tools`, `network constraints`, `time/resource budgets`, `forbidden data access`).
- [ ] SIM2-GC-8-2 Define reject-by-default action grammar/DSL and validation engine so only explicitly permitted action types are executable.
- [ ] SIM2-GC-8-3 Implement container execution path that converts model output to validated actionable steps, then executes against target endpoints.
- [ ] SIM2-GC-8-4 Enforce egress allowlist and capability boundaries at runtime with explicit deny/audit paths for policy violations.
- [ ] SIM2-GC-8-5 Add strict sanitization/validation so unsafe or out-of-policy model outputs are rejected before execution.
- [ ] SIM2-GC-8-6 Add negative-path security tests (secret-exfiltration canaries, out-of-scope URL attempts, privileged header injection attempts, replay envelope misuse).
- [ ] SIM2-GC-8-7 Add trace lineage from model suggestion -> executed action -> runtime telemetry -> monitoring view.
- [ ] SIM2-GC-8-8 Add degraded-mode behavior for key outages that remains explicit and does not fake execution success.
- [ ] SIM2-GC-8-9 Add operator kill-switch and deterministic emergency stop flow for active frontier runs.
- [ ] SIM2-GC-8-10 Enforce hardened container runtime profile for frontier workers (`non-root/rootless`, `no_new_privileges`, capability allowlist only, read-only rootfs with explicit scratch mounts, no privileged mode/host namespace joins).
- [ ] SIM2-GC-8-11 Block sensitive host-control surfaces by policy (forbid daemon-socket mounts and disallowed host bind mounts; fail launch when isolation profile is violated).
- [ ] SIM2-GC-8-12 Implement signed host-issued capability envelopes for executable worker actions (`run_id`, `step_id`, action scope, nonce, `issued_at`, `expires_at`, `key_id`) with strict signature/expiry/replay validation.
- [ ] SIM2-GC-8-13 Implement bounded one-way command channel semantics (host -> worker command queue with backpressure; worker output restricted to append-only evidence/events without control-plane mutation rights).
- [ ] SIM2-GC-8-14 Implement deterministic fail-closed teardown contract (hard runtime deadline, heartbeat timeout, forced process-tree kill, and terminal run-failed semantics on teardown failure).
- [ ] SIM2-GC-8-15 Add lifecycle cleanup policy for frontier run artifacts/resources (TTL-driven cleanup, bounded retention, and explicit cleanup failure diagnostics).

Acceptance criteria:
1. Frontier lane emits real traffic actions, not report-only suggestions.
2. Every executed frontier action has full lineage and monitoring evidence.
3. Out-of-policy model outputs are blocked deterministically by reject-by-default validation.
4. Egress allowlist and capability restrictions are enforced and test-covered.
5. Security abuse-path tests fail closed for exfiltration, privilege escalation, and envelope misuse attempts.
6. Key outage mode is visible, bounded, and non-deceptive.
7. Operators can force-stop frontier execution with deterministic teardown behavior.
8. Frontier workers cannot launch unless hardened runtime isolation profile is satisfied.
9. Signed capability envelopes are mandatory for execution and replay/signature/expiry failures are rejected deterministically.
10. Command/evidence channel semantics preserve one-way authority boundaries and bounded backpressure behavior.
11. Timeout/crash/heartbeat-loss paths fail closed with deterministic teardown and terminal diagnostics.

### SIM2-GC-9: Scenario Design Realism and Defense Exercise Guarantees

Scope: ensure scenario catalog consistently drives targeted defenses under realistic attacker progression.

- [ ] SIM2-GC-9-1 Add scenario intent matrix mapping each scenario to required defense signals and minimum evidence thresholds.
- [ ] SIM2-GC-9-2 Remove scenario success criteria that can pass without exercising intended defenses.
- [ ] SIM2-GC-9-3 Add progression logic for crawler/scraper/browser cohorts that models realistic retries, pacing, and evasion attempts.
- [ ] SIM2-GC-9-4 Add contract tests that fail if scenarios labeled for a defense category do not generate corresponding events.
- [ ] SIM2-GC-9-5 Add periodic coverage review process for stale, redundant, or non-realistic scenarios.

Acceptance criteria:
1. Each scenario has explicit, test-backed defense exercise expectations.
2. Scenario passes without required defense evidence are impossible.
3. Coverage includes realistic multi-step adversary behavior, not single-request probes only.
4. Catalog quality is actively governed and measurable.

### SIM2-GC-10: Dashboard UX for Arms-Race Operations (Evidence-First)

Scope: make monitoring UI operationally useful for defense evolution loops.

- [ ] SIM2-GC-10-1 Add “recent adversary run” panel linking run ids to observed defense deltas in monitoring and IP-ban surfaces.
- [ ] SIM2-GC-10-2 Add per-defense trend blocks (trigger count, pass/fail ratio, escalations, ban outcomes) keyed by source labels.
- [ ] SIM2-GC-10-3 Add fast filters for `origin`, `scenario`, `lane`, `defense`, and `outcome` without introducing new visual language.
- [ ] SIM2-GC-10-4 Add explicit empty/error/degraded states so missing data is never mistaken for “no attacks.”
- [ ] SIM2-GC-10-5 Add operator workflow docs for triage, replay, tuning, and validation loops from the dashboard.
- [ ] SIM2-GC-10-6 Add explicit monitoring freshness indicators (`last event at`, `current lag`, `state: fresh/degraded/stale`) on monitoring and IP-ban tabs.

Acceptance criteria:
1. Operators can directly correlate adversary runs with defense responses from the UI.
2. Missing/late telemetry is explicit and actionable.
3. Filtering and trends support fast tuning decisions without data ambiguity.
4. UI remains consistent with existing dashboard design system.
5. Operators can distinguish “no attacks observed” from “data is stale/degraded” without ambiguity.

### SIM2-GC-11: Verification Suite Expansion for End-to-End Truthfulness

Scope: enforce non-regression with tests that prove real traffic -> real defenses -> real monitoring visibility.

- [ ] SIM2-GC-11-1 Define and publish mandatory verification matrix mapping each defense category to required scenarios, lanes, and evidence assertions.
- [ ] SIM2-GC-11-2 Add e2e test suite that executes matrix-required crawler/scraper/browser/frontier scenarios and asserts monitoring/IP-ban updates.
- [ ] SIM2-GC-11-3 Add contract tests for telemetry lineage integrity and monotonic event ordering across refresh cycles.
- [ ] SIM2-GC-11-4 Add failure-injection tests (telemetry store delay, partial write failure, refresh race) with expected operator-visible outcomes.
- [ ] SIM2-GC-11-5 Add Makefile targets for focused SIM2 realtime verification and wire them into `make test` gating policy.
- [ ] SIM2-GC-11-6 Add CI diagnostics artifacts (timeline snapshots, event counts, refresh traces) for fast triage.
- [ ] SIM2-GC-11-7 Add explicit prod-mode monitoring checks using non-sim traffic profiles to verify near-realtime visibility without adversary-sim toggle dependence.
- [ ] SIM2-GC-11-8 Require failure diagnostics to name missing matrix row(s), missing evidence type(s), and failing telemetry lineage segment.
- [ ] SIM2-GC-11-9 Add control-plane race/idempotency tests for repeated UI toggle submissions, duplicate command replay, and multi-controller lease contention.
- [ ] SIM2-GC-11-10 Add trust-boundary negative-path tests for adversary control endpoint (`csrf missing/invalid`, `origin mismatch`, `fetch-metadata cross-site`, `stale session`) and assert fail-closed behavior.
- [ ] SIM2-GC-11-11 Add idempotency misuse tests proving key reuse with payload mismatch is rejected and exact retries map to stable `operation_id`.
- [ ] SIM2-GC-11-12 Add throttling + audit tests proving rapid toggle storms are bounded and every accept/reject/throttle decision emits structured audit evidence.
- [ ] SIM2-GC-11-13 Add container isolation regression tests for frontier lane (reject privileged mode, daemon-socket mount, disallowed host mount, and missing runtime hardening flags).
- [ ] SIM2-GC-11-14 Add signed-envelope negative tests (`invalid signature`, `nonce replay`, `expiry exceeded`, `scope mismatch`) proving worker execution is blocked.
- [ ] SIM2-GC-11-15 Add teardown determinism tests (`deadline exceeded`, `heartbeat loss`, forced-kill path) and assert terminal failure taxonomy plus cleanup completion.
- [ ] SIM2-GC-11-16 Add cursor-contract tests for monotonic ordering, resume-after-cursor correctness, overflow signaling, and deduped replay windows.
- [ ] SIM2-GC-11-17 Add SSE-path tests for event-id ordering, `Last-Event-ID` reconnect behavior, and fallback-to-polling continuity when stream drops.
- [ ] SIM2-GC-11-18 Add reproducible realtime benchmark verification target (`make test-sim2-realtime-bench`) and CI artifact outputs for latency percentiles, overflow/drop counts, and request-budget metrics.
- [ ] SIM2-GC-11-19 Add retention lifecycle regression tests for bucket cutoff correctness, purge-watermark progression, purge-lag threshold, and no read-path full-keyspace cleanup scans.
- [ ] SIM2-GC-11-20 Add cost-governance regression tests for cardinality caps, overflow-bucket accounting, unsampleable-event protection, payload-size budget, and compression effectiveness thresholds.

Acceptance criteria:
1. Mandatory verification fails if any matrix-required defense/lane evidence is missing.
2. Tests cover happy path, degraded path, race path, and prod-mode non-sim freshness path.
3. CI artifacts make telemetry-visibility failures actionable at defense-row granularity.
4. `make test` remains canonical and sufficient for contributor verification.
5. Release gates cannot pass with partial coverage hidden by aggregate pass/fail summaries.
6. Toggle control regressions (duplicate starts, lease split-brain, missing operation lineage) fail deterministically in CI.
7. Trust-boundary regressions (CSRF/origin/session/replay/throttle/audit) fail deterministically in CI with explicit failure taxonomy.
8. Frontier isolation/envelope/teardown regressions fail deterministically in CI with explicit failure taxonomy.
9. Realtime cursor/stream ordering and resume regressions fail deterministically in CI with explicit failure taxonomy.
10. Benchmark-threshold regressions (`p95`, `p99`, overflow/drop, request-budget) fail deterministically in CI with scenario-specific diagnostics.
11. Retention-lifecycle regressions (purge lag, bucket cutoff drift, read-path scan fallback) fail deterministically in CI with explicit failure taxonomy.
12. Cost-governance regressions (cardinality/payload/sampling/compression/query-budget) fail deterministically in CI with explicit failure taxonomy.

### SIM2-GC-12: Program Governance for Continuous Defense Evolution

Scope: operationalize a repeatable closed-loop process using SIM2 as real adversary pressure.

- [ ] SIM2-GC-12-1 Define weekly/iteration cadence for `run -> review -> tune -> replay -> promote` cycle with ownership and SLA.
- [ ] SIM2-GC-12-2 Define promotion rubric for new adversary techniques (severity, reproducibility, collateral risk, mitigation readiness).
- [ ] SIM2-GC-12-3 Add KPI dashboard/reporting for attacker cost shift, human-friction impact, detection latency, and mitigation lead time.
- [ ] SIM2-GC-12-4 Add explicit rollback playbooks for defensive changes that over-trigger on legitimate traffic.
- [ ] SIM2-GC-12-5 Add periodic architecture review checkpoint to ensure orchestration remains decentralized, capability-safe, and evidence-driven.

Acceptance criteria:
1. SIM2 is operated as an ongoing engineering system, not a one-off test feature.
2. New adversary discoveries have clear promotion and mitigation pathways.
3. KPIs demonstrate whether defenses are improving without unacceptable collateral.
4. Governance enforces architectural excellence and prevents drift back to imperative/convention-only controls.

### SIM2-GC-13: Remove Adversary Sim Progress Bar and Eliminate Dead UI Runtime Paths

Scope: remove the top progress bar UX that implies choreographed sequence progression and keep only clear ON/OFF + lifecycle state semantics.

- [ ] SIM2-GC-13-1 Remove progress-line markup (`#adversary-sim-progress-line`) and related style hooks from dashboard route/templates.
- [ ] SIM2-GC-13-2 Remove progress-timer state (`adversarySimProgressNowMs`, tick interval) and associated scheduling/cleanup logic.
- [ ] SIM2-GC-13-3 Delete `deriveAdversarySimProgress` runtime helper and remove any no-longer-used status fields from UI-only normalization contracts.
- [ ] SIM2-GC-13-4 Keep lifecycle semantics explicit in UI copy/status (`off`, `running`, `stopping`) without representing run as procedural progress.
- [ ] SIM2-GC-13-5 Update unit/e2e tests to assert control behavior and lifecycle state visibility without any progress-width assertions.
- [ ] SIM2-GC-13-6 Update docs to remove “top progress line” references and describe auto-off as a guardrail window, not scenario progression.
- [ ] SIM2-GC-13-7 Run dead-code sweep for dashboard/runtime modules to remove imports, helpers, and selectors no longer referenced after progress-line removal.

Acceptance criteria:
1. Dashboard no longer renders or references a top adversary-sim progress bar.
2. No progress-timer/tick code remains in dashboard route/runtime modules.
3. Tests verify ON/OFF + lifecycle behavior only and pass without progress assumptions.
4. Operator docs describe sim as enabled/disabled attacker activity bounded by guardrail duration, not choreographed progression.
5. Removal leaves no dead selectors, helper exports, or stale tests.

### SIM2-GC-14: Formalize Hybrid Adversary Model (Deterministic Oracle + Emergent Exploration)

Scope: resolve ambiguity between choreographed simulation and emergent adversary behavior by defining and enforcing a two-lane architecture with explicit promotion bridge.

- [ ] SIM2-GC-14-1 Write architecture contract distinguishing `deterministic conformance lane` (blocking) and `emergent exploration lane` (non-blocking discovery).
- [ ] SIM2-GC-14-2 Define what remains intentionally choreographed (seed scenarios, invariant assertions, resource guardrails) vs what must be emergent (crawl strategy, attack sequencing, adaptation).
- [ ] SIM2-GC-14-3 Define emergent-lane objective model (target assets, success functions, allowed adaptation space, stop conditions) with bounded runtime budgets.
- [ ] SIM2-GC-14-4 Define novelty scoring and triage policy (`novelty`, `severity`, `confidence`, `replayability`) for emergent findings.
- [ ] SIM2-GC-14-5 Add lane metadata and report lineage fields so operators can see whether evidence came from deterministic or emergent execution.
- [ ] SIM2-GC-14-6 Define promotion pipeline from emergent finding -> deterministic replay case -> blocking regression with explicit acceptance contract.
- [ ] SIM2-GC-14-7 Add governance tests that fail if release-blocking decisions depend on stochastic-only emergent outcomes without deterministic confirmation.
- [ ] SIM2-GC-14-8 Set and enforce quantitative promotion thresholds (`minimum deterministic confirmation rate`, `maximum tolerated false-discovery rate`, `owner-review requirements`).
- [ ] SIM2-GC-14-9 Update operator docs/runbooks so monitoring expectations reflect “real attacker behavior while enabled,” with deterministic replay used for release confidence.

Acceptance criteria:
1. Deterministic and emergent lanes are explicit, testable, and operationally visible.
2. Blocking gates depend only on deterministic confirmation, never stochastic one-off outcomes.
3. Emergent lane drives realistic crawl/scrape/attack exploration without privileged control-plane access and within bounded budgets.
4. Promotion decisions use quantitative thresholds and are auditable from lineage artifacts.
5. False-discovery behavior is measured and kept within declared limits.
6. Operator documentation and UI terminology no longer conflate guardrail duration with procedural adversary progress.

### SIM2-GC-15: Telemetry Retention Lifecycle Determinism and Health Visibility

Scope: enforce deterministic retention/purge semantics for monitoring/event telemetry without read-path scan amplification.

- [ ] SIM2-GC-15-1 Define canonical telemetry bucket/index schema for monitoring/event retention operations (`bucket_id`, `window_start`, `window_end`, `record_count`, `state`).
- [ ] SIM2-GC-15-2 Migrate telemetry writes to update bucket/index metadata so expired windows are purge-addressable without full keyspace scans.
- [ ] SIM2-GC-15-3 Implement background purge worker cadence with bounded batch budget and persisted purge watermark (`last_purged_bucket`, `last_attempt_ts`, `last_success_ts`).
- [ ] SIM2-GC-15-4 Remove opportunistic retention cleanup from monitoring/admin read paths and replace with worker-triggered retention lifecycle.
- [ ] SIM2-GC-15-5 Add retention health surface in admin/monitoring payloads (`retention_hours`, `oldest_retained_ts`, `purge_lag_hours`, `pending_expired_buckets`, `last_error`).
- [ ] SIM2-GC-15-6 Add degraded-state signaling and operator guidance when retention drift exceeds thresholds.
- [ ] SIM2-GC-15-7 Add deterministic failure-recovery behavior for purge partial failures (retry safety, idempotent bucket cleanup, explicit failure taxonomy).
- [ ] SIM2-GC-15-8 Add docs/runbook updates for retention tuning, purge troubleshooting, and operational rollback.

Acceptance criteria:
1. Retention enforcement no longer relies on monitoring refresh read paths performing keyspace-wide cleanup work.
2. Purge lag remains `<=1 hour` beyond configured retention window under declared normal envelope.
3. Healthy state reports `pending_expired_buckets == 0`; non-zero state is operator-visible with degraded status.
4. Bucket cutoff semantics are deterministic and test-backed across repeated purge cycles.
5. Purge worker remains bounded (`<=500ms` budget per cadence tick) and failure-retry behavior is idempotent.
6. Retention health telemetry is visible in dashboard/admin surfaces and included in CI diagnostics artifacts.

### SIM2-GC-16: Monitoring Cost Governance and Resource Efficiency Envelope

Scope: enforce layered cost controls across telemetry ingest, storage, query, and transport while preserving security-critical evidence integrity.

- [ ] SIM2-GC-16-1 Define formal monitoring cost envelope (`ingest events/sec`, `query calls/sec`, `payload bytes`, `cardinality budget`, `compression ratio`) for dev/prod verification profiles.
- [ ] SIM2-GC-16-2 Enforce guarded-dimension cardinality caps (`<=1000` distinct values/hour per guarded dimension) with deterministic `other` overflow bucket behavior.
- [ ] SIM2-GC-16-3 Implement rollup windows (`1m`, `5m`, `1h`) for dashboard-default queries and preserve raw-event drill-down lineage.
- [ ] SIM2-GC-16-4 Define unsampleable security-event class list and enforce `0` sampling/drop for those classes.
- [ ] SIM2-GC-16-5 Add deterministic low-risk telemetry sampling policy for eligible high-volume classes with explicit sampled/unsampled counters.
- [ ] SIM2-GC-16-6 Add payload budget controls (`p95 <= 512KB` default monitoring response) via pagination/cursor windowing and response shaping.
- [ ] SIM2-GC-16-7 Add compression negotiation/reporting for monitoring payloads and enforce `>=30%` transfer reduction target for payloads `>64KB`.
- [ ] SIM2-GC-16-8 Extend admin query budgets to cost-class aware controls and degraded-state signaling when budgets are exceeded.
- [ ] SIM2-GC-16-9 Add operator-facing cost health telemetry (`cardinality_pressure`, `payload_budget_status`, `sampling_status`, `query_budget_status`) and runbook guidance.

Acceptance criteria:
1. Monitoring pipeline remains within declared cost envelope under realtime benchmark scenarios.
2. Guarded dimensions respect cardinality caps with explicit overflow accounting and no unbounded growth.
3. Unsampleable defense-event classes are never sampled or dropped.
4. Default monitoring payloads meet size budget and expose pagination/window continuation when capped.
5. Compression and query-budget controls provide measurable transport/query cost savings without freshness regressions.
6. Cost health status is operator-visible and CI-enforced with threshold diagnostics.

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
