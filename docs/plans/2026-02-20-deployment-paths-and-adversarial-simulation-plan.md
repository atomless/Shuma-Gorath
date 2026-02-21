# Deployment Paths and Adversarial Simulation Excellence Plan

Date: 2026-02-20  
Status: Proposed (implementation-ready)

Related research:
- `docs/research/2026-02-20-deployment-and-adversarial-simulation-research-synthesis.md`

## Objectives

1. Publish a clear, low-friction single-host deployment path for `self_hosted_minimal`.
2. Define and implement a strict enterprise path for Akamai/Fermyon multi-instance deployments with full ban synchronization semantics across instances.
3. Establish a repeatable adversarial simulation program that covers crawler/scraper/bot behavior from low to high threat levels.

## Non-goals

1. Creating provider-specific policy forks.
2. Introducing Python on runtime request paths.
3. Replacing the existing Makefile-first workflow.

## Workstream A: Single-Host Deployment Excellence

### Target state

- One host, one Shuma instance, internal providers by default.
- Explicit production hardening checklist with secure defaults and minimal operator burden.

### Implementation slices

1. SH-1: Add a single-host runbook section with:
   - required env-only variables
   - minimal secure baseline
   - startup, health, and rollback commands
2. SH-2: Add deployment validation coverage for single-host posture (no enterprise drift warnings, secure default assertions).
3. SH-3: Add smoke verification script via Makefile target for post-deploy checks (<abbr title="Hypertext Transfer Protocol">HTTP</abbr> health, admin auth, metrics, and challenge route sanity).

### Exit criteria

- New operator can deploy single-host with only docs + `make setup` + deploy commands.
- Deployment checks fail fast on unsafe posture drift.

## Workstream B: Akamai/Fermyon Multi-Instance Full Ban Sync

### Target state

- `enterprise_akamai` multi-instance uses distributed external providers for `ban_store` and `rate_limiter`.
- Cross-instance ban visibility converges within a documented <abbr title="Service Level Objective">SLO</abbr>.
- Drift/outage behavior is observable, tested, and rollback-safe.

### Architecture requirements

1. Shared Redis-backed ban source of truth for active ban decisions.
2. Configurable strictness posture for distributed ban availability:
   - advisory fallback posture (current behavior)
   - strict sync posture (planned): no silent local divergence for enterprise authoritative operation
3. Convergence telemetry:
   - sync result counts
   - sync-lag histogram/buckets
   - external-backend error metrics
4. Operational guardrails:
   - explicit deploy-time checks for outbound host policy and Redis reachability posture
   - explicit runtime warnings/errors for unsynced posture

### Implementation slices

1. ENT-1: Implement strict distributed ban-store mode for enterprise authoritative operation (no local-only silent drift path).
2. ENT-2: Emit ban propagation/sync observability (result + lag + backend-failure dimensions).
3. ENT-3: Add two-instance integration harness:
   - Spin instance A + Spin instance B
   - shared Redis backend
   - deterministic ban/unban convergence assertions
4. ENT-4: Add outage and partition simulations (Redis unavailable, delayed writes, partial instance loss) with explicit expected behavior by mode.
5. ENT-5: Publish enterprise rollout/rollback runbook:
   - advisory to authoritative promotion gates
   - SLO thresholds
   - immediate rollback triggers and actions
6. ENT-6 (optional, additive): Design asynchronous high-confidence ban mirroring to Akamai Network Lists for perimeter-first suppression; keep Shuma policy as source of truth for app-layer decisions.

### Exit criteria

- Enterprise multi-instance mode has an enforced strict-sync posture for authoritative deployments.
- Multi-instance convergence and failure-mode tests are part of canonical verification.
- On-call runbook contains measurable promotion/rollback gates.

## Workstream C: Adversarial Simulation Program

### Target state

- A canonical simulation matrix covers botness/threat tiers and validates policy outcomes, false-positive posture, and cost controls.
- Results are reproducible and integrated into Make-based verification tiers.

### Simulation tiers

1. SIM-T0 Legitimate humans:
   - normal browsers with expected interaction timing
2. SIM-T1 Low-risk automation:
   - compliant crawlers, low-rate scripted clients
3. SIM-T2 Medium suspicion:
   - headless browser automation, partial JS capability, abnormal cadence
4. SIM-T3 High threat:
   - high-rate scrapers, replay attempts, token misuse, path-randomized crawling
5. SIM-T4 Active adversarial:
   - coordinated distributed load, low-and-slow evasion, retry storms, challenge bypass attempts

### Harness architecture

1. Browser-realistic flow runners (<abbr title="for example">e.g.</abbr>, Playwright).
2. Non-browser crawler runners (Crawlee/Scrapy class).
3. Load shapers for throughput and burst profiles (k6/Locust class).
4. Unified scenario manifest with expected Shuma outcomes:
   - allow
   - tag/monitor
   - not-a-bot
   - challenge
   - maze/deception
   - temporary deny

### Implementation slices

1. SIM-1: Define canonical profile manifest and expected outcome contract per scenario.
2. SIM-2: Implement simulation harness under `scripts/tests/` with deterministic seeds and bounded runtime.
3. SIM-3: Add regression suite for replay and sequence abuse paths.
4. SIM-4: Add performance/cost assertions:
   - response latency bands
   - ban/challenge ratios by scenario
   - write/read amplification guardrails for monitoring paths
5. SIM-5: Add Make targets with tiered execution:
   - fast smoke in default CI path
   - heavier adversarial soak as scheduled/manual gate
6. SIM-6: Document interpretation guide and tuning workflow for failed scenarios.

### Exit criteria

- Botness-threat coverage is explicit and versioned.
- Simulation failures are actionable with clear signal/action mismatch reports.
- CI has at least one mandatory adversarial slice beyond current baseline integration tests.

## Sequence and Priority

1. Start with Workstream B (`ENT-1` to `ENT-4`) because enterprise sync correctness is the largest current risk.
2. Execute Workstream C (`SIM-1` to `SIM-4`) in parallel once strict-sync semantics are defined.
3. Finalize Workstream A polish and docs once enterprise and simulation baselines are measurable.

## Risks and Mitigations

1. Risk: strict sync increases fail-closed behavior during backend outages.
   - Mitigation: explicit mode controls and documented rollback posture.
2. Risk: simulation breadth increases CI runtime.
   - Mitigation: tiered targets (fast mandatory slice + optional/scheduled deep soak).
3. Risk: tool sprawl fragments test ownership.
   - Mitigation: one scenario manifest and one reporting format across all generators.
