# TODO Roadmap

Last updated: 2026-02-24

This is the active work queue.
`todos/security-review.md` tracks security finding validity and closure status.
Completed items are archived in `todos/completed-todo-history.md`.

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

- [ ] SIM-1 Define canonical scenario manifest for botness/threat tiers (`SIM-T0`..`SIM-T4`) and expected outcomes (`allow`, `monitor`, `not-a-bot`, `challenge`, `maze`, `deny_temp`).
- [ ] SIM-2 Build a unified simulation harness in `scripts/tests/` that combines browser-realistic, scraper, crawler, and load-generator traffic profiles with deterministic seeds.
- [ ] SIM-3 Add replay/sequence-evasion simulation paths (token replay, stale token, order violation, cadence anomalies) to close current threat-coverage gaps.
- [ ] SIM-4 Add simulation assertions for effectiveness and cost (`challenge/ban` ratios, false-positive envelope, monitoring write/read amplification guardrails).
- [ ] SIM-5 Add tiered Make targets and CI policy (fast mandatory adversarial smoke + scheduled/deep soak profiles).
- [ ] SIM-6 Document operator interpretation workflow for simulation failures and tuning actions.

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

- [x] TAH-ARCH-1 Publish the target module/file map for maze+tarpit ownership and boundaries (provider adapter vs domain runtime vs route handler) and pin it in [`docs/module-boundaries.md`](../docs/module-boundaries.md).
- [x] TAH-ARCH-2 Move tarpit runtime logic out of `src/providers/internal.rs` into dedicated tarpit domain modules (`src/tarpit/runtime.rs`, `src/tarpit/types.rs`) so provider internals remain thin adapters.
- [x] TAH-ARCH-3 Introduce dedicated tarpit HTTP handler ownership (`src/tarpit/http.rs`) and route wiring for progression endpoints; remove ad-hoc route handling from non-domain layers.
- [x] TAH-ARCH-4 Extract shared maze+tarpit primitives for replay/chain/budget key handling into one shared module and migrate both maze runtime and tarpit runtime to consume it.
- [x] TAH-ARCH-5 Define one typed tarpit progression reason taxonomy (enum -> metrics/admin labels) and make request-router/admin/runtime emit only that normalized set.
- [x] TAH-ARCH-6 Reconcile tarpit config model for v2 progression (difficulty bounds + egress budgets) and explicitly deprecate/remove static-drip-only assumptions where no longer valid pre-launch.

- [x] TAH-1 Publish a concise design note/ADR for the v2 tarpit contract: tiny initial response, work-gated progression, token-chain continuity, strict byte/time budgets, and deterministic fallback matrix (`maze`, `block`, short-ban escalation). (See [`docs/adr/0004-tarpit-v2-progression-contract.md`](../docs/adr/0004-tarpit-v2-progression-contract.md).)
- [x] TAH-2 Define and document the tarpit progression envelope schema (signed, short-lived, single-use operation token with `flow_id`, `step`, `issued_at`, `expires_at`, `ip_bucket`, and optional work parameters), including replay and binding rules. (See [`docs/plans/2026-02-23-tarpit-v2-progression-envelope.md`](../docs/plans/2026-02-23-tarpit-v2-progression-envelope.md).)
- [x] TAH-3 Add a dedicated tarpit progression endpoint that verifies work proofs before issuing the next step token/content chunk; reject stale/replayed/out-of-order operations with explicit reason codes and monitoring labels.
- [x] TAH-4 Implement a low-cost server-verified work gate (hashcash-style by default; configurable bounded difficulty window), with clear hooks for future memory-hard alternatives.
- [x] TAH-5 Add adaptive work policy that raises/lower difficulty only within bounded limits based on abuse evidence and budget pressure; keep accessibility-safe fallback path to avoid impossible human flows.
- [x] TAH-6 Replace static large filler payload behavior with iterative gated progression so server sends small chunks per verified step rather than prebuilding a large in-memory body.
- [x] TAH-7 Add strict per-flow token-chain continuity checks (step monotonicity, parent-child digest linkage, single-use operation IDs, and replay TTL semantics) reusing maze shared token primitives where possible.
- [x] TAH-8 Add tarpit egress budget controls in config/admin: global bytes-per-window, per-IP-bucket bytes-per-window, per-flow max-bytes, and per-flow max-duration; classify as KV-tunable vs env-only according to project policy.
- [x] TAH-9 Enforce egress budgets before every tarpit step emission (admission + post-send accounting), with deterministic fallback when any budget is exhausted and explicit event/metric outcomes.
- [x] TAH-10 Add budget state keys and lifecycle semantics (TTL/reset window, counter granularity, site scoping) and ensure enterprise/distributed-state behavior is explicit (no silent divergence in authoritative mode).
- [ ] TAH-11 Expand observability: progression admissions/denials, proof verify outcomes, chain violations, bytes sent, duration, budget exhaustion reason, fallback action, and escalation outcomes (including top offender buckets with cardinality guardrails).
- [ ] TAH-12 Add dashboard/admin visibility for the new tarpit progression + egress metrics and operator guidance for safe tuning (recommended starting ranges and rollback thresholds).
- [ ] TAH-19 Before launch, tighten collateral-risk controls (especially bucket-based persistence escalation), then re-evaluate tarpit defaults.
- [x] TAH-13 Add tests:
  unit tests for envelope/proof/chain/budget logic;
  integration tests for abuse route -> gated progression -> fallback/escalation;
  concurrency/soak tests for egress-budget correctness under burst traffic.
- [x] TAH-14 Wire new tarpit asymmetry tests into Makefile/CI verification lanes (`make test-unit`, `make test-integration`, `make test`) and add explicit failure-gate criteria for regressions.
- [x] TAH-15 Update operator docs ([`docs/configuration.md`](../docs/configuration.md), [`docs/dashboard.md`](../docs/dashboard.md), runbooks) to explain that tarpit cost-imposition is now progression-gated and budget-capped, including tradeoffs between attacker cost, host egress, and false-positive risk.
- [x] TAH-16 Add bounded timing/content variability (jitter windows and template rotation with strict envelopes) to reduce tarpit fingerprintability without violating budget caps.
- [x] TAH-17 Add pre-generated/cached tarpit content shards (text/media templates) so per-request host compute stays low while preserving high bot ingestion cost.
- [x] TAH-18 Add an explicit crawler-safety policy path for known legitimate bots and sensitive endpoints (for example `robots.txt`) to avoid accidental tarpit impact on benign indexing/ops workflows.

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
