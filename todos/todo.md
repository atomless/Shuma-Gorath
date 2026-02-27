# TODO Roadmap

Last updated: 2026-02-27

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

- [ ] SIM2-ARCH-1 Publish ADR for functional-core/imperative-shell orchestration and explicit capability model (trust boundaries, migration order, rollback).
- [ ] SIM2-ARCH-2 Add characterization test harness capturing current request-path decision outcomes for representative policy matrix.
- [ ] SIM2-ARCH-3 Extract side-effect-free `RequestFacts` builders from request/config/provider inputs.
- [ ] SIM2-ARCH-4 Extract first policy tranche into pure `PolicyDecisionGraph` stages (IP-range, honeypot, rate, existing-ban) with typed outputs.
- [ ] SIM2-ARCH-5 Extract second policy tranche into pure stages (GEO, botness, JS/challenge routing) with typed outputs.
- [ ] SIM2-ARCH-6 Introduce explicit effect-intent executor for bans, metrics, monitoring, and event logging side effects.
- [ ] SIM2-ARCH-7 Replace convention-based privileged operations with explicit capability objects/tokens at trust boundaries.
- [ ] SIM2-ARCH-8 Reduce `src/lib.rs` to thin orchestration shell (`facts -> decisions -> effects -> response`) while preserving behavior.
- [ ] SIM2-ARCH-9 Add policy-graph unit coverage and parity tests to prove no behavior regressions across migration slices.
- [ ] SIM2-ARCH-10 Update module-boundary docs and operator/developer architecture docs to reflect new orchestration model.
- [ ] SIM2-ARCH-11 Ensure all verification remains Makefile-driven (`make test`, `make build`) with no lane bypass.

Acceptance criteria:
1. Core policy decisions become predominantly pure and testable without side effects.
2. Privileged operations are blocked unless explicit capability objects are present.
3. Characterization parity tests prove behavior stability across extraction slices.
4. `src/lib.rs` orchestration complexity is materially reduced and role-focused.
5. Full required verification (`make test`, `make build`) remains green throughout migration.

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
