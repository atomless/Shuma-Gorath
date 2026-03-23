# TODO Roadmap

Last updated: 2026-03-23

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

## P0 First Closed Feedback Loop And Control-Plane Restructuring

Reference context:
- [`docs/research/2026-03-21-feedback-loop-and-architecture-debt-review.md`](../docs/research/2026-03-21-feedback-loop-and-architecture-debt-review.md)
- [`docs/research/2026-03-21-loop-closure-execution-readiness-review.md`](../docs/research/2026-03-21-loop-closure-execution-readiness-review.md)
- [`docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`docs/plans/2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md`](../docs/plans/2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md)
- [`docs/plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`](../docs/plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md)
- [`docs/plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md`](../docs/plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md)
- [`docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`docs/plans/2026-03-20-benchmark-suite-v1-design.md`](../docs/plans/2026-03-20-benchmark-suite-v1-design.md)
- [`docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`docs/plans/2026-03-15-agentic-era-oversight-design.md`](../docs/plans/2026-03-15-agentic-era-oversight-design.md)
- [`docs/plans/2026-03-15-agentic-era-oversight-implementation-plan.md`](../docs/plans/2026-03-15-agentic-era-oversight-implementation-plan.md)

Current stance:
- Observation and adversary generation are now ahead of objective, comparison, attribution, and recommend-only reconcile truth.
- Execute the structural decomposition tranches before landing more controller or operator-loop logic into the current hotspot files.
- Keep the decomposition slices behavior-preserving; semantic expansion begins only after the hotspot modules have focused seams.
- Land the first shared-host agent tweaker loop before `MON-OVERHAUL-1` so Monitoring and Tuning can project proven machine-first semantics instead of inventing them early.
- Treat the verified-identity observability and benchmarking gap as part of this loop-closure track rather than later polish.
- The first bounded shared-host closed config loop is now live-proven, and the adversary-sim status diagnostics follow-up is now delivered.
- Before `MON-OVERHAUL-1`, settle Scrapling's truthful request-native category ownership and proof so Monitoring does not project the older indexing-only lane semantics.
- Before `MON-OVERHAUL-1`, deepen verified identity from a coarse beneficial-traffic signal into a taxonomy-calibration and no-harm guardrail so Monitoring does not bake the current flattened Web Bot Auth semantics into the next operator-facing surface.
- The next prerequisite now begins one step earlier: Shuma must first define a canonical non-human taxonomy and shared classification contract before it can truthfully measure lane representativeness.
- The actively evolving near-term layer should be fingerprinting and categorization quality inside that taxonomy; taxonomy expansion is only a later contingency if important non-human traffic persistently falls outside the existing categories.
- The next bridge to a genuine LLM-in-the-loop system is not the full later `SIM-LLM-1` runtime actor; it is a bounded category-fulfillment LLM tranche that sits between the taxonomy/classifier work and the first closed config loop, with the later LLM diagnosis harness and code loop still downstream of `OVR-APPLY-1`.
- Before `MON-OVERHAUL-1`, extend suspicious-origin cost with a truthful host-impact proxy so Monitoring and later tuning do not project a request-and-byte-only model of unwanted non-human traffic cost.
- Before `MON-OVERHAUL-1`, expose the already-settled local control truths that no longer belong only in Advanced JSON or backend-only payloads: verified identity in `Verification` and adversary-sim status truth basis in `Red Team`.

### VID-TAX-2: Verified-Identity Versus Taxonomy Alignment Receipts
- Add explicit machine-first receipts that show whether high-confidence verified traffic aligned cleanly with the taxonomy crosswalk, fell back through a degraded mapping, or remained mismatched.
- Surface bounded alignment summaries in snapshot or benchmark contracts so later Monitoring can project them directly.
- Reference context: [`../docs/research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md`](../docs/research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md), [`../docs/plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](../docs/plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)

### VID-BOT-1: Verified-Identity Versus Botness Conflict Metrics
- Add benchmark and snapshot metrics that quantify when fingerprinting and cumulative abuse score (`botness`) disagree with high-confidence verified identity, especially for tolerated or allowed verified traffic.
- Keep the metrics bounded and operator-meaningful: conflict pressure, mismatch-driven short-circuiting, and user-triggered-agent friction drift.
- Reference context: [`../docs/research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md`](../docs/research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md), [`../docs/plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](../docs/plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)

### VID-GUARD-1: Hard Verified-Identity Tuning Guardrails
- Make diagnosis and bounded tuning fail closed when verified-identity friction mismatch or unresolved botness conflicts show likely harm against configured tolerated or allowed verified traffic.
- Keep this tranche restrictive-by-default: it must add safety guardrails, not implicit allow behavior.
- Reference context: [`../docs/research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md`](../docs/research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md), [`../docs/plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](../docs/plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)

### HOST-COST-1: Bounded Host-Impact Telemetry Foundation
- Add bounded forwarded-latency telemetry to the request-outcome and monitoring hot-read path so Shuma can measure host-impact proxies that go beyond suspicious forwarded request and byte ratios.
- Keep this tranche telemetry-first and proxy-first: no speculative route weights, no fake cost units, and no direct CPU or memory budgets yet.
- Reference context: [`../docs/research/2026-03-23-host-impact-cost-proxy-and-benchmark-review.md`](../docs/research/2026-03-23-host-impact-cost-proxy-and-benchmark-review.md), [`../docs/plans/2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md`](../docs/plans/2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md)

### HOST-COST-2: Host-Impact Snapshot And Benchmark Integration
- Thread the new host-impact proxy through `operator_snapshot_v1`, `operator_objectives_v1`, and `benchmark_results_v1`, including a budgetable `suspicious_forwarded_latency_share` metric and bounded comparison semantics.
- Keep this tranche inside the existing suspicious-origin family and existing controller model; do not invent a second cost loop or new patch-family surface.
- Reference context: [`../docs/research/2026-03-23-host-impact-cost-proxy-and-benchmark-review.md`](../docs/research/2026-03-23-host-impact-cost-proxy-and-benchmark-review.md), [`../docs/plans/2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md`](../docs/plans/2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md)

### UI-VID-1: Verified Identity Pane In Verification Tab
- Add a first-class `Verified Identity` pane to `Verification` so native Web Bot Auth and the broader verified-identity mechanics are no longer Advanced-only for operators.
- Keep the first pane focused on the already-stable basics: enablement, native versus provider verification toggles, replay and freshness windows, bounded directory controls, and a bounded read-only health summary.
- Reference context: [`../docs/research/2026-03-23-dashboard-operator-surfacing-gap-review.md`](../docs/research/2026-03-23-dashboard-operator-surfacing-gap-review.md), [`../docs/plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](../docs/plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)

### UI-RED-1: Red Team Truth-Basis Diagnostics
- Surface adversary-sim status `truth_basis` and bounded `persisted_event_evidence` in `Red Team` so operators can tell when a completed run summary is coming from direct counters versus recovered lower-bound event truth.
- Keep this tranche local to `Red Team`; do not turn it into a Monitoring substitute.
- Reference context: [`../docs/research/2026-03-23-dashboard-operator-surfacing-gap-review.md`](../docs/research/2026-03-23-dashboard-operator-surfacing-gap-review.md), [`../docs/plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](../docs/plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)

### MON-OVERHAUL-1: Monitoring As Thin Operator Projection Over Machine-First Contracts
- Redesign Monitoring as a thin operator decision surface over the machine-first snapshot contract, including truthful projection of `operator_snapshot_v1`, `benchmark_results_v1`, and the bounded oversight status or history contracts, with explicit live-versus-shadow-versus-adversary-sim separation, clear attacker-effectiveness versus human-friction visibility, and the settled host-impact cost proxy semantics from the delivered closed-loop backend.
- Reference context: [`../docs/plans/2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md`](../docs/plans/2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md), [`../docs/research/2026-03-23-dashboard-operator-surfacing-gap-review.md`](../docs/research/2026-03-23-dashboard-operator-surfacing-gap-review.md), [`../docs/research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md`](../docs/research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md), [`../docs/research/2026-03-23-scrapling-non-human-category-capability-review.md`](../docs/research/2026-03-23-scrapling-non-human-category-capability-review.md), [`../docs/research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md`](../docs/research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md), [`../docs/research/2026-03-23-host-impact-cost-proxy-and-benchmark-review.md`](../docs/research/2026-03-23-host-impact-cost-proxy-and-benchmark-review.md)

## P1 Verified Bot Identity And Web Bot Auth Foundation

Reference context:
- [`docs/research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](../docs/research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
- [`docs/research/2026-03-21-verified-identity-execution-readiness-refresh.md`](../docs/research/2026-03-21-verified-identity-execution-readiness-refresh.md)
- [`docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

Current stance:
- Verified identity now sits after the delivered machine-first telemetry/snapshot foundations and before mature adversary-sim expansion.
- The first execution slices should stay observe-only: identity contracts, config, provider normalization, telemetry, and request-path annotations without routing change.
- The primary product value is exact non-human restriction and exception management; looser treatment for named verified bots remains an explicit opt-in later policy choice.
- Do not bundle authorization policy, low-cost profiles, or dashboard control surfaces into the first tranche.

## P1 Production Adversary-Sim Operating Contract

Reference context:
- [`docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md`](../docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md)
- [`docs/research/2026-03-20-sim-deploy-2-readiness-review.md`](../docs/research/2026-03-20-sim-deploy-2-readiness-review.md)
- [`docs/adversarial-operator-guide.md`](../docs/adversarial-operator-guide.md)
- [`docs/plans/2026-03-20-sim-deploy-2-production-operating-envelope-implementation-plan.md`](../docs/plans/2026-03-20-sim-deploy-2-production-operating-envelope-implementation-plan.md)
- [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)

Current stance:
- Production adversary-sim control is part of Shuma's operating model and must not be runtime-prod-disabled.
- The remaining work is operating-envelope hardening, not approval for production availability.
- Execute this tranche in the order captured by the 2026-03-20 readiness review and implementation plan: verification-target truthfulness, desired-state unification, production posture codification, no-impact verification, then docs/evidence closure.

## P1 Shared-Host Discovery Baseline

Reference plan:
- [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`docs/plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](../docs/plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`docs/plans/2026-03-20-shared-host-scope-fence-contract.md`](../docs/plans/2026-03-20-shared-host-scope-fence-contract.md)
- [`docs/plans/2026-03-20-shared-host-seed-contract.md`](../docs/plans/2026-03-20-shared-host-seed-contract.md)
- [`docs/research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md`](../docs/research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md)

### SIM-SH-SURFACE-1: Minimal Shared-Host Scope And Seed Gate
Scope note:
- `SIM-SCR-LANE-1` should require only the minimal scope-and-seed gate in `SIM-SH-SURFACE-1-1..2` plus `SIM-DEPLOY-2`.
- The observed reachable surface should emerge from traversal telemetry once the emergent lane runs; Shuma should not build a rich precomputed public-surface catalog as the default discovery architecture.

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
- [ ] ADV-JSON-1 Audit the Dashboard Advanced tab runtime-variable inventory and Advanced JSON seed for full parity and organization.
  - Reference context:
    - [`docs/dashboard-tabs/advanced.md`](../docs/dashboard-tabs/advanced.md)
    - [`dashboard/src/lib/domain/config-schema.js`](../dashboard/src/lib/domain/config-schema.js)
    - [`dashboard/src/lib/components/dashboard/AdvancedTab.svelte`](../dashboard/src/lib/components/dashboard/AdvancedTab.svelte)
    - [`src/admin/api.rs`](../src/admin/api.rs)
    - [`docs/configuration.md`](../docs/configuration.md)
  - Acceptance criteria:
    - every admin-writable KV config var accepted by `POST /admin/config` appears in Advanced JSON,
    - runtime-visible read-only vars surfaced in Advanced are truthfully classified,
    - stale, missing, or misleading entries are corrected,
    - variables are logically grouped and ordered so the Advanced surface remains navigable as the config surface grows,
    - docs and parity tests are updated so drift fails fast.
- [ ] Resolve the `ip_range_suggestions_*` classification exception so the documented config model stays honest: either make those runtime-visible KV knobs admin-writable with Advanced JSON parity, or move them out of the persisted read-only exception path and document the chosen contract.
- [ ] Write objective criteria for future repository splits (API stability, release cadence, ownership, operational coupling).
- [ ] Design runtime-agnostic architecture that keeps core detection logic portable while preserving the shared-host-first control plane and a later thin edge-gateway adapter path.
- [ ] Evaluate renaming `SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD` to `SHUMA_BOTNESS_CHALLENGE_PUZZLE_THRESHOLD` to reflect botness semantics.
- [ ] Standardize terminology across code, UI, and docs so `honeypot` and `maze` are used consistently instead of interchangeably.
- [ ] Long-term option: integrate upstream identity or proxy auth (OIDC/SAML) for dashboard and admin instead of app-level key login.

## Final Pre-Launch Gate

- [ ] PERF-LAUNCH-1 Execute a final pre-launch performance and optimization pass (dashboard bundle-size budgets in strict mode, runtime latency/<abbr title="Central Processing Unit">CPU</abbr>/memory envelopes, and high-cost request-path profiling), then lock release thresholds and acceptance criteria.
