# TODO Roadmap

Last updated: 2026-03-02

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
Heartbeat decoupling plan: [`docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](../docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)
Mandatory pre-read architecture report for all open SIM work: [`docs/research/2026-03-02-sim-runtime-architecture-overview-and-gap-report.md`](../docs/research/2026-03-02-sim-runtime-architecture-overview-and-gap-report.md)
Prime directive report (shared corpus + out-of-process heartbeat): [`docs/research/2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md`](../docs/research/2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md)
Execution gate: no open `SIM-*` TODO in this section may move forward until both reports above have been read and the implementation slice is explicitly checked against runtime/test-lane boundary findings and prime-directive constraints.

Ongoing objectives for this tranche:
1. Keep deterministic lane as regression oracle; LLM lane as discovery/promotable corpus.
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

- [ ] SIM-DEPLOY-1 Re-evaluate current dev-only adversary-sim availability posture and deployment-path split (`runtime-dev` vs production) against product ambition; define decision criteria, abuse safeguards, tenant/isolation controls, explicit operator consent model, cost controls, and rollback strategy for possible production enablement.
- [ ] SIM-DEPLOY-2 If production availability is approved, design and implement production-safe adversary-sim operating modes (explicit opt-in, spawn-on-enable execution lifecycle, strict rate/resource envelopes, kill switch, auditability, and no-impact guarantees for normal user traffic).

- [ ] SIM-CLEAN-1 After `SIM-ARCH-2`/`SIM-ARCH-3`, run a rigorous runtime+CI dead-code sweep and remove superseded deterministic-generation code paths (obsolete hardcoded batch builders, duplicate action definitions, and unused helper utilities) introduced before shared-corpus convergence.
- [ ] SIM-HB-OOP-1 Introduce a dedicated internal adversary beat endpoint and move generation execution out of `/admin/adversary-sim/control` response path.
- [ ] SIM-HB-OOP-2 Remove request-lifecycle-driven heartbeat execution from runtime entrypoint and make status diagnostics report explicit out-of-process heartbeat ownership.
- [ ] SIM-HB-OOP-3 Implement transient Rust supervisor worker (`spawn-on-enable`, 1s cadence default, bounded retries/backoff) that exits on toggle-off, run-window expiry, or server unreachability.
- [ ] SIM-HB-OOP-4 Add host launch adapters and operator docs for supervisor execution across target environments (local `make dev`, systemd/single-host, container sidecar, and external edge supervisor service).
- [ ] SIM-HB-OOP-5 Enforce strict shutdown/off reconciliation and ephemeral toggle semantics across stop/restart paths: after server stop, state reconciles to `off`, no generator activity remains, and next start defaults to off.
- [ ] SIM-HB-OOP-6 Deprecate and remove dashboard/runtime reliance on `POST /admin/adversary-sim/tick` once out-of-process beat ownership is live.
- [ ] SIM-LEARN-1 Capture a concise adversary-toggle incident report and lifecycle invariants doc (what previously broke, why, and non-negotiable state semantics for toggle-on/off, auto-window expiry, server stop, and restart) and link it from SIM operator docs.
- [ ] SIM-LEARN-2 Add targeted regression tests for the exact failure modes previously seen: toggle no-op, on->off bounce, stale enabled state after server restart, control/status disagreement, and supervisor-not-running while UI claims enabled.
- [ ] SIM-LEARN-3 Add a fast deterministic verification target (single command) that validates toggle lifecycle end-to-end in runtime-dev before any SIM tranche merge.
- [ ] SIM-LEARN-4 Add explicit structured diagnostics for toggle lifecycle troubleshooting (control decision, state transitions, supervisor heartbeat, last successful beat) so failures can be triaged without deep code spelunking.
- [ ] SIM-CLEAN-2 After `SIM-HB-OOP-6`, run a rigorous dead-code sweep for heartbeat migration fallout: remove request-loop supervisor remnants, deprecated tick endpoint wiring, stale dashboard runtime adapters, and superseded diagnostics fields/contracts.
- [ ] SIM-LLM-1 Realize full LLM-orchestrated, instruction-driven, containerized adversary lane as first-class runtime actor on top of the same runtime heartbeat ownership model: run capability-constrained action plans against public HTTP surface, emit normal request-pipeline telemetry, preserve deterministic replay bridge, and surface explicit degraded-state diagnostics when frontier execution is unavailable.
- [ ] SIM-DET-2 Add deterministic config-profiled coverage pass for config-dependent surfaces (GEO and optional IP-range actions) in automated verification only (CI/test harness), so category-level event emission is guaranteed without mutating operator runtime simulation configuration.
- [ ] SIM-DET-3 Add runtime-toggle integration assertions that fail when required deterministic surface categories (challenge, JS, PoW, maze/tarpit, rate, fingerprint/CDP, ban, GEO-configured) are missing from observed event telemetry.
- [ ] SIM-DET-7 Ensure automated verification telemetry remains ephemeral: CI/test adversarial traffic must not pollute operator runtime telemetry history (ephemeral stores and/or mandatory teardown cleanup).
- [ ] SIM-TRUST-1 Remove simulation-context forwarded-IP trust bypass and require simulation requests to satisfy the same trust-boundary conditions as external traffic.
- [ ] SIM-TRUST-2 Add enforcement/telemetry parity tests proving simulated and external-equivalent requests follow identical policy decisions and event accounting (differing only by simulation metadata tags).
- [ ] SIM-DET-L1 (Low priority) Add optional deterministic seed input for runtime-toggle runs to support exact tune-confirm-repeat replay when desired; keep default behavior non-seeded.
- [ ] SIM-CLEAN-3 End-of-tranche final code hygiene pass for all SIM surfaces (runtime, dashboard, CI harness, docs/tests): remove dead modules/branches/contracts, collapse temporary compatibility shims, and fail verification if any open TODO references code paths already removed or renamed.

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
