# TODO Roadmap

Last updated: 2026-03-20

This is the active execution-ready work queue.
Blocked or contingent work lives in `todos/blocked-todo.md`.
Completed work lives in `todos/completed-todo-history.md`.
Security finding validity and closure status live in `todos/security-review.md`.
Keep durable workflow and policy guidance in `docs/project-principles.md`, `CONTRIBUTING.md`, and `AGENTS.md`, not in this file.

## P0 Monitoring and Config Lifecycle Stabilization

Reference context:
- [`docs/plans/2026-02-26-adversarial-simulation-v2-plan.md`](../docs/plans/2026-02-26-adversarial-simulation-v2-plan.md)
- [`docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](../docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)
- [`docs/plans/2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md`](../docs/plans/2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md)
- [`docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](../docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`docs/plans/2026-03-13-compact-event-telemetry-implementation-plan.md`](../docs/plans/2026-03-13-compact-event-telemetry-implementation-plan.md)
- [`docs/plans/2026-03-14-telemetry-retention-rebaseline-implementation-plan.md`](../docs/plans/2026-03-14-telemetry-retention-rebaseline-implementation-plan.md)
- [`docs/plans/2026-03-12-shadow-mode-telemetry-monitoring-truthfulness-plan.md`](../docs/plans/2026-03-12-shadow-mode-telemetry-monitoring-truthfulness-plan.md)
- [`docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`](../docs/research/2026-03-14-compact-event-telemetry-live-evidence.md)
- [`docs/configuration.md`](../docs/configuration.md)
- [`docs/testing.md`](../docs/testing.md)

### SIM2-R4-4: Config Seeding Lifecycle and Shadow-Mode Semantics

## P0 Monitoring Telemetry Foundations

Reference context:
- [`docs/research/2026-03-17-operator-decision-support-telemetry-audit.md`](../docs/research/2026-03-17-operator-decision-support-telemetry-audit.md)
- [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](../docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`docs/plans/2026-03-18-monitoring-traffic-lane-and-denominator-contract.md`](../docs/plans/2026-03-18-monitoring-traffic-lane-and-denominator-contract.md)
- [`docs/plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](../docs/plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`docs/plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md`](../docs/plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md)
- [`docs/plans/2026-03-19-monitoring-human-friction-denominator-plan.md`](../docs/plans/2026-03-19-monitoring-human-friction-denominator-plan.md)
- [`docs/plans/2026-03-19-monitoring-defence-funnel-plan.md`](../docs/plans/2026-03-19-monitoring-defence-funnel-plan.md)
- [`docs/plans/2026-03-18-monitoring-operator-summary-exactness-contract.md`](../docs/plans/2026-03-18-monitoring-operator-summary-exactness-contract.md)
- [`docs/plans/2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md`](../docs/plans/2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md)
- [`docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](../docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`docs/plans/2026-03-13-compact-event-telemetry-implementation-plan.md`](../docs/plans/2026-03-13-compact-event-telemetry-implementation-plan.md)
- [`docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`](../docs/research/2026-03-14-compact-event-telemetry-live-evidence.md)

### MON-TEL-1: Cost-Aware Operator Telemetry Foundation
- Prerequisite foundation landed on 2026-03-18:
  - reuse the settled contracts in [`docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](../docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md) and its linked lane, outcome, exactness, and ownership notes while implementing the counter, summary, and read-surface parts of this tranche.
- Controller-readiness review addendum (2026-03-19):
  - treat the remaining telemetry work below as first-order foundation, not later polish, because future operator benchmarks and bounded inside-controller loops will need truthful byte attribution, richer bounded backend summaries, and fuller control/fail-path outcome coverage before the Monitoring UI overhaul should start.
  - reference: [`docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md)

## P0 Machine-First Operator Snapshot Foundations

Reference context:
- [`docs/research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](../docs/research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`docs/research/2026-03-20-benchmark-suite-v1-research-synthesis.md`](../docs/research/2026-03-20-benchmark-suite-v1-research-synthesis.md)
- [`docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-implementation-plan.md`](../docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-implementation-plan.md)
- [`docs/plans/2026-03-20-benchmark-suite-v1-design.md`](../docs/plans/2026-03-20-benchmark-suite-v1-design.md)
- [`docs/plans/2026-03-20-benchmark-suite-v1-implementation-plan.md`](../docs/plans/2026-03-20-benchmark-suite-v1-implementation-plan.md)
- [`docs/plans/2026-03-15-agentic-era-oversight-design.md`](../docs/plans/2026-03-15-agentic-era-oversight-design.md)
- [`docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

### OPS-BENCH-1: Benchmark Contract For Project Evolution
- [ ] OPS-BENCH-1-2 Materialize `benchmark_results_v1` as the bounded comparison envelope for subject kind, baseline reference, watch window, per-family statuses, metric deltas, and exactness or capability metadata.
- [ ] OPS-BENCH-1-3 Materialize the benchmark-driven escalation hint boundary between `config_tuning_candidate`, `observe_longer`, and `code_evolution_candidate`, keeping that gate explicit, review-aware, and benchmark-driven.
- [ ] OPS-BENCH-1-4 Project `benchmark_suite_v1` and `benchmark_results_v1` into `operator_snapshot_v1` and later Monitoring without letting the human UI invent a parallel semantic model.
- [ ] OPS-BENCH-1-5 Capture the later fleet or central-intelligence enrichment contract for benchmark scenario selection, weighting, and priority without turning the Git repository itself into the shared-intelligence transport.

## P1 Production Adversary-Sim Operating Contract

Reference context:
- [`docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md`](../docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md)
- [`docs/adversarial-operator-guide.md`](../docs/adversarial-operator-guide.md)
- [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)

Current stance:
- Production adversary-sim control is part of Shuma's operating model and must not be runtime-prod-disabled.
- The remaining work is operating-envelope hardening, not approval for production availability.

### SIM-DEPLOY-2: Production Adversary-Sim Operating Envelope
- [ ] SIM-DEPLOY-2-1 Define the production-default runtime lane/resource posture now that adversary-sim surfaces are production-capable by default.
- [ ] SIM-DEPLOY-2-2 Add explicit production kill-switch, diagnostics, and no-impact verification for normal user traffic under live operator use.
- [ ] SIM-DEPLOY-2-3 Update deployment/operator docs and evidence receipts so production adversary-sim usage is documented as a first-class operating path rather than a gated exception.
- [ ] SIM-DEPLOY-2-5 Collapse adversary-sim desired state to one backend source of truth instead of the current runtime-override plus `ControlState.desired_enabled` split, then factor status/control/beat handlers onto one shared lifecycle snapshot helper so status/control contract drift cannot reappear.

## P1 Shared-Host Discovery Baseline

Reference plan:
- [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)

### SIM-SH-SURFACE-1: Shared-Host Public Surface Discovery First
- [ ] SIM-SH-SURFACE-1-1 Define the shared-host descriptor and fail-closed scope contract.
- [ ] SIM-SH-SURFACE-1-2 Implement `robots.txt` and `sitemap.xml` discovery with provenance and diagnostics.
- [ ] SIM-SH-SURFACE-1-3 Merge seed inventory with deterministic canonicalization, dedupe, scope filtering, and rejection reporting.
- [ ] SIM-SH-SURFACE-1-4 Add operator make workflows for seed-only discovery and seed-plus-bounded-Scrapling probe augmentation.
- [ ] SIM-SH-SURFACE-1-5 Run the combined workflow against a real shared-host deployment and archive timestamped evidence artifacts.
- [ ] SIM-SH-SURFACE-1-6 Compile the hosted-surface catalog baseline and publish the operator runbook/signoff checklist.

Runtime Scrapling and LLM lanes remain blocked until this tranche is complete; see `todos/blocked-todo.md`.

## P1 Enterprise Distributed-State Deployment Baseline

Reference plan:
- [`docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](../docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)

- [ ] DEP-ENT-1 Implement strict enterprise distributed ban-sync mode for authoritative multi-instance posture (no silent local-only divergence path).
- [ ] DEP-ENT-2 Add ban-sync observability (<abbr title="Service Level Objective">SLO</abbr> metrics for sync result and lag) to support promotion and rollback decisions.
- [ ] DEP-ENT-3 Add two-instance Spin integration coverage with shared Redis to prove ban and unban convergence behavior.
- [ ] DEP-ENT-4 Add outage and partition tests for distributed state (Redis unavailable or degraded) and assert explicit configured behavior by mode.
- [ ] DEP-ENT-5 Add deployment and runtime guardrails that validate enterprise distributed-state posture against outbound and backend requirements before authoritative operation.
- [ ] OUT-1 Add explicit deployment guardrails that fail when `provider_backends.rate_limiter=external` or `provider_backends.ban_store=external` but required Redis outbound hosts are not allowlisted in `spin.toml` `allowed_outbound_hosts`.
- [ ] OUT-2 Add a provider-to-outbound-requirements matrix in public docs (internal vs external backend, required host capabilities, required outbound host allowlists, fallback behavior).
- [ ] OUT-3 Add integration verification that exercises external Redis provider selection under restricted outbound policy and confirms safe fallback and guardrail behavior is deterministic.

## P1 Akamai Edge Control Expansion

Reference context:
- [`docs/plans/2026-03-09-akamai-rate-geo-integration-semantics-note.md`](../docs/plans/2026-03-09-akamai-rate-geo-integration-semantics-note.md)
- [`docs/plans/2026-03-09-fermyon-akamai-edge-baseline-prerequisite-plan.md`](../docs/plans/2026-03-09-fermyon-akamai-edge-baseline-prerequisite-plan.md)
- [`docs/research/2026-03-12-fermyon-akamai-edge-live-proof.md`](../docs/research/2026-03-12-fermyon-akamai-edge-live-proof.md)

- [ ] AK-RG-2 Define config surface and naming for Rate and GEO Akamai integration controls, including defaults and whether each is a simple toggle or toggle-plus-mode control.
- [ ] AK-RG-3 Implement admin API and runtime config validation for the new Rate and GEO Akamai controls with explicit guardrails and clear validation errors.
- [ ] AK-RG-4 Implement runtime behavior wiring so Akamai Rate and GEO signals can influence decisions according to the defined mode semantics without bypassing Shuma policy ownership.
- [ ] AK-RG-5 Add dashboard controls and help text for Rate and GEO Akamai integration in the top-level tabs, including disabled-state behavior and operator warnings.
- [ ] AK-RG-6 Add observability and policy-event taxonomy coverage for Rate and GEO Akamai decisions (source, mode, action, fallback reason, and downgrade behavior).
- [ ] AK-RG-7 Add integration and end-to-end tests for mode precedence, downgrade/fallback safety, and regression against internal-only behavior.
- [ ] AK-RG-8 Document rollout and rollback guidance for enabling Rate and GEO Akamai integration in enterprise deployments, including promotion gates and emergency disable steps.

## P1 Privacy and Data-Protection Follow-up

- [ ] SEC-GDPR-2 Enforce deterministic cleanup and expiry for stale fingerprint state keys (`fp:state:*`, `fp:flow:*`, `fp:flow:last_bucket:*`) aligned to configured fingerprint TTL and window controls.
- [ ] SEC-GDPR-3 Add an optional event-log IP minimization mode (raw vs masked or pseudonymized) for privacy-sensitive deployments, with explicit tradeoff documentation.
- [ ] SEC-GDPR-4 Add a deployer-ready privacy and cookie disclosure template in docs (lawful basis, retention table, storage inventory, and rights-handling workflow).

## P2 Hardening and Coverage

Architecture alignment reference:
- [`docs/plans/2026-02-23-maze-tarpit-architecture-alignment-plan.md`](../docs/plans/2026-02-23-maze-tarpit-architecture-alignment-plan.md)

- [ ] TEST-HYGIENE-2 Keep canonical `make test` worktree-clean by moving routine adversarial/SIM2 generated receipts out of tracked fixture paths (or otherwise making them reproducible without churn), so verification does not rewrite committed JSON artifacts like `preflight_report.json`, `sim2_ci_diagnostics.json`, `sim2_operational_regressions_report.json`, and `sim2_realtime_bench_report.json`.
- [ ] TEST-HYGIENE-3 Replace the remaining dashboard source-contract archaeology checks with rendered-behavior coverage where practical, especially for tab-surface ownership and monitoring section composition, so tests prove operator-visible behavior instead of string-level absence of old implementations.
- [ ] TEST-HYGIENE-4 Add a focused dashboard behavior test proving two distinct adversary-simulation `sim_run_id` values render as two `Recent Red Team Runs` rows when both runs are still present in the bounded monitoring window.
- [ ] TEST-HYGIENE-5 Add dashboard coverage that proves Monitoring headline charts remain enforced-only while shadow-mode labeling stays explicit in the raw/recent-event surfaces, so shadow truthfulness is verified at the rendered UI level instead of inferred from source structure.
- [ ] TEST-ENV-1 Enforce repo-wide Rust test env-isolation discipline so any test that mutates process env must hold `lock_env()` (and fix the remaining offender in `src/runtime/shadow_mode/tests.rs`).
- [ ] BUILD-HYGIENE-1 Restore warning-free canonical verification by fixing cfg/dead-code hygiene in native test builds, including `src/config/runtime_env.rs::spin_variable_name`, so `make test` output stays signal-rich and release gates do not normalize away compiler warnings.
- [ ] CI-WF-1 Refresh GitHub Actions dependencies off the Node 20-backed majors (`actions/checkout@v4`, `actions/setup-node@v4`, `actions/upload-artifact@v4`) and re-prove the workflows without deprecation annotations before the hosted-runner cutoff forces emergency maintenance.
- [ ] TAH-11 Expand tarpit observability: progression admissions and denials, proof verify outcomes, chain violations, bytes sent, duration, budget exhaustion reason, fallback action, and escalation outcomes (including top offender buckets with cardinality guardrails).
- [ ] TAH-12 Add dashboard and admin visibility for the new tarpit progression and egress metrics plus operator guidance for safe tuning.
- [ ] TAH-19 Before launch, tighten collateral-risk controls (especially bucket-based persistence escalation), then re-evaluate tarpit defaults.
- [ ] MZ-T1 Add Spin integration coverage for live opaque maze traversal across multiple hops with deterministic fallback action and reason assertions.
- [ ] MZ-T2 Add browser end-to-end coverage for live maze behavior (JS-enabled and JS-disabled cohorts, checkpoint and micro-PoW flow, replay rejection, and high-confidence escalation outcomes under real HTTP and session behavior).
- [ ] MZ-T3 Add concurrency and soak coverage for maze state and budget primitives to detect contention or regression under burst traversal and verify bounded host-write behavior.
- [ ] MZ-T4 Wire the new maze integration, end-to-end, and soak tests into canonical Makefile and CI verification paths so maze behavior regressions fail fast before merge.

## P2 Later Product Work

- [ ] INSPECT-1: Ephemeral Admin Defence Inspection Mode
  - Reference context:
    - [`docs/challenge-verification.md`](../docs/challenge-verification.md)
    - [`docs/tarpit.md`](../docs/tarpit.md)
    - [`docs/dashboard-tabs/tuning.md`](../docs/dashboard-tabs/tuning.md)
    - [`src/runtime/policy_graph.rs`](../src/runtime/policy_graph.rs)
    - [`src/runtime/request_flow.rs`](../src/runtime/request_flow.rs)
  - [ ] INSPECT-1-1 Write a short design note that defines the exact contract: admin-only inspection controls in the Tuning tab, current authed admin IP derived server-side, ephemeral state with explicit expiry, no freeform IP entry, and no persisted `always_challenge_ips` config surface in normal config or Advanced JSON.
  - [ ] INSPECT-1-2 Add runtime state and admin API primitives for activate/deactivate/status so inspection binds to the currently authed admin IP under trusted admin auth, survives page refreshes for a bounded TTL, and expires/cleans up deterministically.
  - [ ] INSPECT-1-3 Model inspection as an explicit next-request entry point rather than a fake botness score or threshold override: the operator arms one inspection target (`Not-a-Bot`, `Puzzle`, `Maze`, or `Challenge-abuse escalation`) for the next eligible request, and the system then behaves normally from that point onward.
  - [ ] INSPECT-1-4 Keep the implementation at the policy/response boundary so existing defence modules remain truthful and unchanged internally; do not add walkthrough-specific scoring or success/failure branches inside Not-a-Bot, Puzzle, Maze, or Tarpit modules.
  - [ ] INSPECT-1-5 Keep direct trap rendering out of inspection mode: Maze and Tarpit previews remain the direct surfaces for previewing those traps. For tarpit-related inspection, add a truthful operator path that exercises the confirmed challenge-abuse routing logic that would escalate to tarpit, without requiring the operator to hand-craft replay/tamper abuse manually.
  - [ ] INSPECT-1-6 Add the Tuning-tab control surface as simple arm/disarm inspection actions with status/expiry copy, no operator IP input field, and clear explanation of which actions inspect human-path routing versus challenge-abuse escalation routing.
  - [ ] INSPECT-1-7 Add unit, integration, and dashboard end-to-end coverage proving activation, expiry, trusted admin IP rebinding, non-admin isolation, consumption of armed entry points, correct normal post-entry behaviour, challenge-abuse escalation inspection, and cleanup without persistent bans/collateral state leakage.
  - [ ] INSPECT-1-8 Update operator docs and verification guidance so admins know exactly what each inspection entry point exercises, what still follows normal runtime semantics after entry, and how inspection differs from the existing direct Maze/Tarpit preview links on local and deployed environments.
- [ ] SIM-DET-L1 Add optional deterministic seed input for runtime-toggle runs to support exact tune-confirm-repeat replay when desired; keep default behavior non-seeded.
- [ ] NAB-12 Evaluate optional PAT-style private attestation signal ingestion as additive evidence only (non-blocking).
- [ ] NAB-13 Execute short Not-a-Bot hardening sprint per [`docs/plans/2026-02-21-not-a-bot-hardening-sprint.md`](../docs/plans/2026-02-21-not-a-bot-hardening-sprint.md).
- [ ] Add ASN and network dimensions in GEO policy logic, not just country list. (`src/signals/geo/mod.rs`, `src/config/mod.rs`, `src/admin/api.rs`)
- [ ] Add GEO and ASN observability and alerting (metrics, dashboard panels, docs). (`src/observability/metrics.rs`, dashboard, docs)

## P3 Platform and Configuration Clarity

- [ ] Centralize dashboard tab metadata (tab ids, loading copy, refresh defaults, and invalidation scopes) into one shared registry consumed by route, route-controller, refresh-runtime, and native-runtime code so stale fallback tabs/messages cannot drift.
- [ ] ADV-RUN-ARCH-1 Break `scripts/tests/adversarial_simulation_runner.py` into focused modules/helpers for contract loading, attacker/control-plane execution, evidence shaping, and governance/report checks so the current `6881`-line multi-responsibility runner stops concentrating unrelated concerns in one file.
- [ ] Resolve the `ip_range_suggestions_*` classification exception so the documented config model stays honest: either make those runtime-visible KV knobs admin-writable with Advanced JSON parity, or move them out of the persisted read-only exception path and document the chosen contract.
- [ ] Write objective criteria for future repository splits (API stability, release cadence, ownership, operational coupling).
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving Fermyon-first performance paths.
- [ ] Evaluate renaming `SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_PUZZLE_THRESHOLD` to reflect botness semantics.
- [ ] Standardize terminology across code, UI, and docs so `honeypot` and `maze` are used consistently instead of interchangeably.
- [ ] Long-term option: integrate upstream identity or proxy auth (OIDC/SAML) for dashboard and admin instead of app-level key login.

## Final Pre-Launch Gate

- [ ] PERF-LAUNCH-1 Execute a final pre-launch performance and optimization pass (dashboard bundle-size budgets in strict mode, runtime latency/<abbr title="Central Processing Unit">CPU</abbr>/memory envelopes, and high-cost request-path profiling), then lock release thresholds and acceptance criteria.
