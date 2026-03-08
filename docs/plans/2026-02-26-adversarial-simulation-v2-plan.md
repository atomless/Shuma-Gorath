# Adversarial Simulation v2 Plan (Coverage + Realism)

Date: 2026-02-26  
Status: Proposed (implementation-ready)

Related:
- [`docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)

## Purpose

Design a strict, realistic adversary simulation program that:

1. Generates realistic mixed traffic (not only synthetic route pokes).
2. Triggers every monitoring and defense category before release.
3. Enforces quantitative gates (latency, ratio, amplification), not only route pass/fail.
4. Stays bounded enough that simulation does not become a larger project than core defense.

## Hard Requirements

1. `Coverage`: full pre-release profile must produce non-zero evidence across all monitored defense categories.
2. `Realism`: traffic must include benign humans, tolerated automation, and adversarial behavior with realistic pacing/cadence and session behavior.
3. `Black-box attacker`: attacker agents must not use admin credentials or privileged secrets.
4. `Bounded runtime`: profile tiers must stay deterministic and time-bounded.
5. `Actionability`: failed gates must point to a specific category/scenario mismatch.

## Research Inputs (Condensed)

1. OWASP WSTG maps web security testing to PTES-style phased methodology (planning, intelligence, execution, reporting), supporting explicit simulation objectives and reporting contracts.  
   Source: https://owasp.org/www-project-web-security-testing-guide/
2. PTES emphasizes explicit scope, threat modeling, and repeatable execution/reporting, matching a versioned manifest + deterministic runner model.  
   Source: https://www.pentest-standard.org/index.php/Main_Page
3. OWASP Automated Threat Handbook provides category-level taxonomy for automated abuse paths; this is a better basis for scenario families than ad hoc scripts.  
   Source: https://owasp.org/www-project-automated-threats-to-web-applications/
4. MITRE CTID emulation guidance emphasizes behavior-focused, objective-driven emulation plans over one-off checks.  
   Source: https://center-for-threat-informed-defense.github.io/adversary_emulation_library/
5. k6 threshold model shows practical quantitative gating patterns for CI-grade non-functional assertions.  
   Source: https://grafana.com/docs/k6/latest/using-k6/thresholds/
6. Locust documents user classes and wait-time modeling for realistic traffic pacing rather than flat request loops.  
   Source: https://docs.locust.io/en/stable/writing-a-locustfile.html
7. Scrapling demonstrates modern scraper evasion capabilities (stealth/fingerprint-aware crawling), useful as an optional attacker-pack reference without forcing scope explosion in core repo.  
   Source: https://github.com/D4Vinci/Scrapling

## Current Gaps (v1 Runner)

1. Strong deterministic route checks exist, but category coverage is incomplete.
2. Current traffic is mostly scenario-driven request scripts, not realistic mixed traffic cohorts.
3. Tarpit and some deep monitoring categories are underexercised in standard runs.
4. Attacker/control concerns are partially mixed (simulation framework can use privileged setup paths).

## Coverage Contract

The `full_coverage` profile must satisfy all rows below in one run.

| Category | Evidence Source | Gate |
|---|---|---|
| Honeypot | `summary.honeypot.total_hits` | `> 0` |
| Challenge failures | `summary.challenge.total_failures` | `> 0` |
| Not-a-Bot outcomes | `summary.not_a_bot.outcomes` | `pass > 0`, `fail/replay/escalate` covered by scenarios |
| PoW | `summary.pow.total_attempts` and `summary.pow.outcomes` | success and failure both `> 0` |
| Rate limiting | `summary.rate.total_violations` and `summary.rate.outcomes` | `> 0`, includes `limited` |
| GEO routing | `summary.geo.total_violations` and `summary.geo.actions` | `> 0`, challenge+maze+block covered across suite |
| Maze | `details.maze.total_hits` | `> 0` |
| Tarpit | `details.tarpit.metrics.*` | activations and at least one progression outcome `> 0` |
| CDP | `details.cdp.stats.total_detections` | `> 0` |
| Fingerprint ingestion | `details.cdp.fingerprint_stats.events` | `> 0` |
| Ban path | `details.analytics.ban_count` + event stream | ban produced during run |
| Event stream health | `details.events.recent_events` | includes expected event families |
| IP range suggestions | `/admin/ip-range/suggestions` | non-empty summary when suggestion prerequisites met |

Notes:
1. `fast_smoke` remains narrower and time-bounded.
2. `full_coverage` is mandatory pre-release; CI policy can keep it mandatory while pre-launch.

## Architecture v2

### 1. Plane separation

1. `Control plane` (trusted): setup/reset, config patching, telemetry reads.
2. `Attacker plane` (untrusted): sends only public traffic; no admin/API keys, no signing secrets.

### 2. Manifest v2

Extend scenario schema with:

1. `traffic_model`: persona, think-time distribution, retry strategy, cookie behavior.
2. `expected_defense_categories`: categories expected to fire.
3. `coverage_tags`: links scenario to coverage contract rows.
4. `cost_assertions`: per-scenario and profile-level latency/ratio/amplification gates.

### 3. Unified runner

Keep one orchestrator, but support multiple agent classes:

1. Browser-realistic (`playwright`-style flows).
2. HTTP scraper (stateful non-browser with cadence/evasion variants).
3. Edge-signal injection (canned Akamai fixtures).
4. Cost-imposition walkers (maze/tarpit progression attempts).

### 4. Realistic traffic composition

Default `full_coverage` blend:

1. Human-like cohort: page navigation, natural pauses, occasional abandon/back.
2. Benign automation cohort: low-rate consistent fetchers.
3. Suspicious automation cohort: cadence anomalies and partial capability gaps.
4. Adversarial cohort: replay/stale/order abuse, scraper loops, bypass attempts.

## Minimal Public Sim Surface (Dev/Test Only)

Add minimal public pages solely for simulation realism and crawl graph quality:

1. `/sim/public/landing`
2. `/sim/public/docs`
3. `/sim/public/pricing`
4. `/sim/public/contact`
5. `/sim/public/search?q=...`

Rules:

1. Enabled only in dev/test simulation mode.
2. No new runtime dependencies.
3. Static/minimal payloads; purpose is traffic-shape realism, not product UI.

## Make/CI Tier Design

1. `make test-adversarial-smoke`
   - Mandatory CI gate.
   - Runtime target: ~90s.
   - Core outcome + core cost gates.
2. `make test-adversarial-coverage`
   - Mandatory while pre-launch.
   - Runtime target: <= 6 minutes.
   - Enforces full category coverage contract.
3. `make test-adversarial-abuse`
   - Mandatory CI gate.
   - Replay/stale/order/cadence abuse regressions.
4. `make test-adversarial-akamai`
   - Mandatory CI gate.
   - Canned fixture ingestion and authoritative/additive behavior checks.
5. `make test-adversarial-soak`
   - Scheduled/manual.
   - Longer stability + amplification drift checks.
6. `make test-adversarial-live`
   - Operator flow (preserve state + rotating IPs by default) for live dashboard observation.

## Quantitative Gates (Profile-Level)

1. `Latency`: scenario max and profile p95/p99 bands.
2. `Defense mix`: challenge/maze/ban ratios by cohort must stay inside configured envelopes.
3. `Collateral`: human-cohort deny/challenge envelope must remain below threshold.
4. `Amplification`: monitoring and fingerprint writes per request must remain bounded.
5. `Determinism`: fixed seeds produce stable pass/fail outcomes.

## Scope Control (Avoid Project Bloat)

Default path: keep implementation in-repo.

Create sister adversary repo only if two or more criteria hold:

1. Required attacker dependencies meaningfully bloat core dev setup.
2. Adversary release cadence diverges from core runtime cadence.
3. Adversary logic exceeds agreed maintenance threshold (for example >25% of test code footprint).
4. CI runtime/cost becomes materially constrained by adversary framework complexity.

## Implementation Slices

1. `SIM-V2-1`: Manifest v2 schema + compatibility migration for existing scenarios.
2. `SIM-V2-2`: Coverage contract evaluator and report section (`coverage_gates`).
3. `SIM-V2-3`: Add missing scenario families (rate, GEO block, PoW fail/success, CDP, tarpit progression, IP-range-triggered behavior).
   Current status note:
   `full_coverage` now proves tarpit bootstrap entry, but advanced tarpit progression remains deferred until a dedicated progress-following scenario lands.
4. `SIM-V2-4`: Add minimal dev/test public sim pages and crawl graph.
5. `SIM-V2-5`: Add `full_coverage` profile and make target.
6. `SIM-V2-6`: CI policy update for mandatory vs scheduled tiers.
7. `SIM-V2-7`: Operator docs for interpreting failed coverage/cost gates.

## Definition of Done

1. `make test-adversarial-coverage` fails if any required category is untriggered.
2. Runner report includes per-category coverage evidence and missing-category diagnostics.
3. Attacker plane runs without admin/API key access.
4. Mandatory CI tiers are deterministic and bounded with clear failure output.
5. Docs explain what each profile proves and when to use each.
