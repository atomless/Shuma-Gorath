# 🐙 Research Index

This folder contains research syntheses, comparative analysis, and design references used to guide Shuma implementation choices.

Top-level files in `docs/research/` are intended to remain active for upcoming work.
Completed tranches that no longer guide active implementation are moved to `docs/research/archive/`.

## 🐙 Tarpit Research Collection

Use this list as the canonical index for Shuma tarpit research and design context.

1. [`tarpit-research-2026-02-11.md`](tarpit-research-2026-02-11.md)  
   Initial tarpit landscape scan (Deadlocked concept + ecosystem pattern review + Shuma fit).
2. [`2026-02-14-maze-tarpit-research-synthesis.md`](2026-02-14-maze-tarpit-research-synthesis.md)  
   Maze/tarpit synthesis gate (`MZ-R1`/`MZ-R2`/`MZ-R3`) and enforceable guardrails.
3. [`2026-02-22-http-tarpit-cost-shift-research-synthesis.md`](2026-02-22-http-tarpit-cost-shift-research-synthesis.md)  
   Paper + implementation evidence synthesis for bounded attacker-cost shifting.
4. [`2026-02-23-tarpit-docs-rereview-addendum.md`](2026-02-23-tarpit-docs-rereview-addendum.md)  
   Focused re-review of Deadlocked and linked repositories with concrete implications for next tarpit hardening steps.

Related implementation plan:

- [`../plans/2026-02-22-http-tarpit-cost-shift-implementation-plan.md`](../plans/2026-02-22-http-tarpit-cost-shift-implementation-plan.md)

Related active backlog:

- [`../../todos/todo.md`](../../todos/todo.md) (`TAH-*` items under “Tarpit Asymmetry Hardening”)

Completed archive index:

- [`archive/README.md`](archive/README.md)

## 🐙 Adversarial Simulation and LLM Testing

1. [`2026-02-25-llm-adversarial-testing-research-synthesis.md`](2026-02-25-llm-adversarial-testing-research-synthesis.md)  
   LLM-driven adversarial testing methods, benchmark lessons, and direct mapping to `SIM-1`..`SIM-4` simulation design.
2. [`2026-03-02-sim-runtime-architecture-overview-and-gap-report.md`](2026-03-02-sim-runtime-architecture-overview-and-gap-report.md)  
   Runtime architecture inventory and gap analysis for dashboard-toggle lane vs deterministic/containerized test lanes; required pre-read for open `SIM-*` execution.
3. [`2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md`](2026-03-02-sim-prime-directive-shared-corpus-and-out-of-process-heartbeat.md)  
   Prime directive for one shared deterministic corpus plus out-of-process runtime heartbeat ownership.
4. [`2026-03-02-adversary-toggle-incident-report-and-lifecycle-invariants.md`](2026-03-02-adversary-toggle-incident-report-and-lifecycle-invariants.md)  
   Incident capture and non-negotiable lifecycle invariants for toggle/restart/off-state behavior, with required fast regression gate.
5. [`2026-03-20-sim-deploy-2-readiness-review.md`](2026-03-20-sim-deploy-2-readiness-review.md)
   Readiness review confirming `SIM-DEPLOY-2` could start immediately, identifying the stale lifecycle verification target as the first local prerequisite, and recommending the execution order for the production operating-envelope tranche.
6. [`2026-03-20-sim-deploy-2-post-implementation-review.md`](2026-03-20-sim-deploy-2-post-implementation-review.md)
   Closeout review confirming the production adversary-sim operating envelope now has truthful lifecycle verification, one desired-state authority, explicit production posture and kill-switch semantics, no-impact proof, and first-class deployment/operator guidance.
7. [`2026-03-20-sim-scr-lane-1-readiness-review.md`](2026-03-20-sim-scr-lane-1-readiness-review.md)
   Readiness review confirming `SIM-SCR-LANE-1` can start immediately after the 2026-03-20 deploy and shared-host closeouts, and recommending additive contract migration before worker or dashboard work.
8. [`2026-03-20-sim-scr-0-lane-contract-post-implementation-review.md`](2026-03-20-sim-scr-0-lane-contract-post-implementation-review.md)
   Post-implementation review confirming the first additive lane-migration slice landed as backend state/status scaffolding plus a focused `make test-adversary-sim-lane-contract` gate without changing runtime routing yet.
9. [`2026-03-20-sim-scr-1-lane-selection-post-implementation-review.md`](2026-03-20-sim-scr-1-lane-selection-post-implementation-review.md)
   Post-implementation review confirming the control path now persists strict lane selection, exposes requested/desired/actual lane auditability, keeps idempotency lane-aware, and still truthfully reports `synthetic_traffic` as the active lane until worker routing lands.
10. [`2026-03-20-sim-scr-6-scrapling-worker-post-implementation-review.md`](2026-03-20-sim-scr-6-scrapling-worker-post-implementation-review.md)
   Post-implementation review confirming heartbeat lane routing now dispatches the real bounded Scrapling worker, records live lane diagnostics through the internal worker-result contract, and leaves deployment-level egress isolation as the remaining follow-on hardening note for the operator-workflow tranche.
11. [`2026-03-20-sim-scr-7-dashboard-lane-controls-post-implementation-review.md`](2026-03-20-sim-scr-7-dashboard-lane-controls-post-implementation-review.md)
   Post-implementation review confirming the dashboard now exposes the settled lane selector, desired-versus-active lane truth, focused module + Playwright verification, and leaves the operator-rollout tranche as the next remaining slice.

## 🐙 Gateway Deployment Research

1. [`2026-03-05-gateway-only-spin-architecture-research-synthesis.md`](2026-03-05-gateway-only-spin-architecture-research-synthesis.md)  
   Two-pass synthesis for gateway-only production posture, combining Spin/Fermyon platform constraints, reverse-proxy trust-boundary best practice, and current Shuma codebase impact mapping.
2. [`2026-03-05-gateway-first-tranche-conformance-review.md`](2026-03-05-gateway-first-tranche-conformance-review.md)  
   Implementation conformance review against `DEP-GW-1` acceptance criteria with completion evidence.
3. [`2026-03-05-gateway-first-post-tranche-cleanup-review.md`](2026-03-05-gateway-first-post-tranche-cleanup-review.md)  
   Post-tranche cleanup and knock-on architecture review, including follow-on hardening opportunities.
4. [`2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md)
   Architecture review concluding that Shuma should remain shared-host-first for the full adaptive loop and defer Fermyon to a later gateway-only track.
5. [`2026-03-21-fermyon-shelving-roadmap-and-docs-cleanup-post-implementation-review.md`](2026-03-21-fermyon-shelving-roadmap-and-docs-cleanup-post-implementation-review.md)
   Post-implementation review for the roadmap, backlog, and public-doc cleanup that followed the shared-host-first direction update.

## 🐙 Agentic-Era Oversight

1. [`2026-03-15-agentic-era-oversight-research-synthesis.md`](2026-03-15-agentic-era-oversight-research-synthesis.md)
   Research synthesis for Shuma's long-horizon operating model in the agentic era, covering crawler economics, verified-agent identity, scheduler patterns, low-cost agent handling, and bounded autonomous oversight.
2. [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md)
   Research synthesis for banded ban jitter, local repeat-offender escalation, and central intelligence as coordinated cost-shaping features within Shuma's agentic-era defence model.
3. [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
   Research synthesis for Web Bot Auth, HTTP Message Signatures, signed agents, verified bots, and the local policy and trust model Shuma needs for authenticated automated traffic.
4. [`2026-03-21-verified-identity-execution-readiness-refresh.md`](2026-03-21-verified-identity-execution-readiness-refresh.md)
   Readiness refresh reconciling the verified-identity planning chain with the updated roadmap and exposing the first execution-ready `WB-0.*` and `WB-1.*` slices.
5. [`2026-03-21-verified-identity-planning-refresh-post-implementation-review.md`](2026-03-21-verified-identity-planning-refresh-post-implementation-review.md)
   Post-implementation review for the verified-identity planning refresh tranche.
6. [`2026-03-21-wb-0-1-canonical-verified-identity-domain-post-implementation-review.md`](2026-03-21-wb-0-1-canonical-verified-identity-domain-post-implementation-review.md)
   Post-implementation review for the first verified-identity execution tranche, confirming the canonical provider-independent identity domain landed before config and provider wiring work.
7. [`2026-03-21-wb-0-2-verified-identity-config-placeholders-and-validation-post-implementation-review.md`](2026-03-21-wb-0-2-verified-identity-config-placeholders-and-validation-post-implementation-review.md)
   Post-implementation review for the verified-identity config tranche, confirming canonical defaults, admin/config/export parity, config-seed wiring, and dashboard Advanced JSON parity landed before request-path identity verification work.
8. [`2026-03-21-wb-1-1-verified-identity-provider-seam-post-implementation-review.md`](2026-03-21-wb-1-1-verified-identity-provider-seam-post-implementation-review.md)
   Post-implementation review for the verified-identity provider seam tranche, confirming trusted provider assertions now normalize through the shared provider registry into canonical identity verification results without changing routing yet.
9. [`2026-03-21-wb-1-2-verified-identity-observe-only-telemetry-post-implementation-review.md`](2026-03-21-wb-1-2-verified-identity-observe-only-telemetry-post-implementation-review.md)
   Post-implementation review for the verified-identity telemetry tranche, confirming observe-only request-path recording now populates monitoring summaries with identity, outcome, freshness, provenance, and scheme visibility without changing routing.
10. [`2026-03-21-wb-1-3-verified-identity-request-path-annotations-post-implementation-review.md`](2026-03-21-wb-1-3-verified-identity-request-path-annotations-post-implementation-review.md)
   Post-implementation review for the request-path annotation tranche, confirming verified identities now reach policy facts and request-outcome monitoring lanes without silently changing enforcement.
11. [`2026-03-21-wb-observe-only-tranche-review-and-shortfall-closeout.md`](2026-03-21-wb-observe-only-tranche-review-and-shortfall-closeout.md)
   Full review of the observe-only verified-identity tranche (`WB-0.1` through `WB-1.3`), recording the one shortfall found at the Prometheus metrics boundary and the immediate follow-up that closed it.
12. [`2026-03-21-wb-2-1-native-http-message-signature-readiness-review.md`](2026-03-21-wb-2-1-native-http-message-signature-readiness-review.md)
   Readiness review defining the smallest clean `WB-2.1` slice as the native verifier core with deterministic replay/freshness enforcement while leaving bounded remote directory discovery and caching for `WB-2.2`.
13. [`2026-03-21-wb-2-1-native-http-message-signature-post-implementation-review.md`](2026-03-21-wb-2-1-native-http-message-signature-post-implementation-review.md)
   Post-implementation review for the native verifier tranche, confirming the internal runtime now has deterministic HTTP Message Signatures verification with explicit external-directory deferral to `WB-2.2`, and recording the replay-state fail-closed follow-up completed during closeout review.
14. [`2026-03-21-wb-2-2-directory-discovery-cache-readiness-review.md`](2026-03-21-wb-2-2-directory-discovery-cache-readiness-review.md)
   Readiness review for `WB-2.2`, fixing the bounded external directory-discovery/cache contract around HTTPS-only outbound resolution, explicit stale/unavailable semantics, and deterministic cache-size limits before implementation.
15. [`2026-03-21-wb-2-2-directory-discovery-cache-post-implementation-review.md`](2026-03-21-wb-2-2-directory-discovery-cache-post-implementation-review.md)
   Post-implementation review for `WB-2.2`, confirming native external directory discovery/cache now lands with explicit stale/unavailable behavior, bounded cache recovery, and the immediate cache-index rebuild follow-up completed during closeout review.
16. [`2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-readiness-review.md`](2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-readiness-review.md)
   Readiness review for `WB-2.3`, freezing the gateway and edge trust contract around forwarded HTTPS context, `Signature*` pass-through, and stripping of Shuma-owned verified-identity assertion headers before implementation.
17. [`2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-post-implementation-review.md`](2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-post-implementation-review.md)
   Post-implementation review for `WB-2.3`, confirming the gateway/edge trust contract is now explicit in docs and regression-protected by focused native and gateway tests without expanding authorization scope.
4. [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
   Research synthesis for the machine-first Monitoring destination, bounded operator snapshot contract, and the first config-diff-only feedback loop that should precede the human Monitoring overhaul.
5. [`2026-03-20-operator-snapshot-foundation-post-implementation-review.md`](2026-03-20-operator-snapshot-foundation-post-implementation-review.md)
   Post-implementation review confirming the first `operator_snapshot_v1` foundation slice landed cleanly and tightening the no-write-on-read proof for the new admin read path.
6. [`2026-03-20-operator-snapshot-recent-changes-post-implementation-review.md`](2026-03-20-operator-snapshot-recent-changes-post-implementation-review.md)
   Post-implementation review confirming the bounded `recent_changes` ledger landed as a write-side compact summary rather than a read-time event-log scan.
7. [`2026-03-20-operator-snapshot-allowed-actions-post-implementation-review.md`](2026-03-20-operator-snapshot-allowed-actions-post-implementation-review.md)
   Post-implementation review confirming `allowed_actions_v1` landed as a conservative controller write contract and that `OPS-SNAPSHOT-1` is now complete.
8. [`2026-03-17-operator-decision-support-telemetry-audit.md`](2026-03-17-operator-decision-support-telemetry-audit.md)
   Repo-grounded audit of current telemetry collection, operator-useful decision-support signals, contributor-only diagnostics, and the highest-value monitoring gaps Shuma should close before the Monitoring overhaul.
9. [`2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
   State-of-the-art research synthesis for the telemetry Shuma should collect in the agentic era, grounded in current Cloudflare, Google, OpenAI, Anthropic, Web Bot Auth, and HTTP Message Signatures guidance on traffic classification, detection layering, operator analytics, and verified-agent handling.
10. [`2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
   Cost-aware gap analysis comparing Shuma's current telemetry collection to the desired operator-grade model, with explicit attention to hot-read budget, retained-footprint economics, and the minimum next telemetry tranche that improves operator decision support without regressing efficiency.
11. [`2026-03-19-controller-readiness-telemetry-foundation-review.md`](2026-03-19-controller-readiness-telemetry-foundation-review.md)
   Architecture review addendum for the newly landed telemetry foundation work, identifying the remaining controller-grade gaps that should be treated as first-order Stage 1 foundation work before the Monitoring overhaul or bounded inside-agent benchmarking loops.
12. [`2026-03-19-defence-funnel-origin-integrity-review.md`](2026-03-19-defence-funnel-origin-integrity-review.md)
   Post-implementation review for the first defence-funnel slice, tightening the contract so only live-safe family stages are populated today and recording the immediate follow-on need for origin-aware `not_a_bot`, `challenge`, and `pow` family telemetry.
13. [`2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md`](2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md)
   Closeout review confirming that the controller-grade backend telemetry foundation is complete and that the next work should move to the Monitoring-overhaul discussion and section-ownership planning rather than another telemetry architecture sweep.
14. [`2026-03-20-monitoring-diagnostics-ownership-post-implementation-review.md`](2026-03-20-monitoring-diagnostics-ownership-post-implementation-review.md)
   Post-implementation review confirming that the Monitoring/Diagnostics ownership split landed cleanly, that the legacy bounded monitoring surface now has a truthful Diagnostics home, and that the next work should move to the substantive Monitoring overhaul.
15. [`2026-03-20-adversary-evolution-loop-role-synthesis.md`](2026-03-20-adversary-evolution-loop-role-synthesis.md)
   Research synthesis clarifying the roles of deterministic oracle traffic, emergent Scrapling and frontier lanes, and the diagnosis agent in Shuma's future adaptive feedback loop.
16. [`2026-03-20-benchmark-suite-v1-research-synthesis.md`](2026-03-20-benchmark-suite-v1-research-synthesis.md)
   Research synthesis defining the first benchmark families Shuma should use to judge bot-cost asymmetry, human-friction control, adversary-sim effectiveness, and beneficial non-human posture for both instance tuning and later project evolution.
17. [`2026-03-20-benchmark-fleet-and-intelligence-enrichment-research-synthesis.md`](2026-03-20-benchmark-fleet-and-intelligence-enrichment-research-synthesis.md)
   Research synthesis defining how later fleet or central-intelligence evidence should enrich benchmark scenario selection, family priority, and bounded weighting without rewriting local benchmark truth.
18. [`2026-03-20-benchmark-suite-contract-post-implementation-review.md`](2026-03-20-benchmark-suite-contract-post-implementation-review.md)
   Post-implementation review confirming the static `benchmark_suite_v1` registry landed as a machine-first backend contract and that the next benchmark work should move to `benchmark_results_v1` rather than Monitoring UI work.
19. [`2026-03-20-benchmark-results-contract-post-implementation-review.md`](2026-03-20-benchmark-results-contract-post-implementation-review.md)
   Post-implementation review confirming the first `benchmark_results_v1` slice landed as a bounded machine-first backend contract over `operator_snapshot_v1` and that the next benchmark work should move to explicit escalation semantics.
20. [`2026-03-20-benchmark-escalation-boundary-post-implementation-review.md`](2026-03-20-benchmark-escalation-boundary-post-implementation-review.md)
   Post-implementation review confirming the benchmark-driven escalation boundary landed as a bounded machine-first contract over benchmark results plus `allowed_actions_v1`, and that the next benchmark work should move to snapshot and Monitoring projection.
21. [`2026-03-20-benchmark-results-snapshot-projection-post-implementation-review.md`](2026-03-20-benchmark-results-snapshot-projection-post-implementation-review.md)
   Post-implementation review confirming `benchmark_results_v1` is now projected directly into `operator_snapshot_v1`, that `/admin/benchmark-results` reuses the same materialized contract, and that Monitoring is discussion-ready from the backend side.
22. [`2026-03-20-benchmark-fleet-enrichment-contract-post-implementation-review.md`](2026-03-20-benchmark-fleet-enrichment-contract-post-implementation-review.md)
   Post-implementation review confirming the later fleet or central-intelligence benchmark-enrichment rules are now captured as a separate advisory contract and that the local benchmark-planning tranche is complete.
23. [`2026-03-20-mature-adversary-sim-evolution-roadmap-post-implementation-review.md`](2026-03-20-mature-adversary-sim-evolution-roadmap-post-implementation-review.md)
   Post-implementation review confirming the mature adversary-sim roadmap now treats deterministic traffic as oracle and memory, emergent lanes as primary adaptive inputs, and replay promotion as a named future step.
24. [`2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md`](2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md)
   Research synthesis arguing that a realistic adversary harness should start from minimal seeds and scope fences, then let traversal telemetry become the authoritative adversary-reachable surface map.
25. [`2026-03-20-minimal-seed-surface-discovery-post-implementation-review.md`](2026-03-20-minimal-seed-surface-discovery-post-implementation-review.md)
   Post-implementation review confirming the shared-host backlog and roadmap now prefer minimal operator-defined seeds plus traversal telemetry rather than a rich precomputed public-surface catalog.
26. [`2026-03-20-shared-host-scope-contract-post-implementation-review.md`](2026-03-20-shared-host-scope-contract-post-implementation-review.md)
   Post-implementation review confirming `SIM-SH-SURFACE-1-1` landed as a versioned fail-closed scope contract plus tooling validator, and that the next shared-host work should move to the seed contract rather than control-plane surface.
27. [`2026-03-20-shared-host-seed-contract-post-implementation-review.md`](2026-03-20-shared-host-seed-contract-post-implementation-review.md)
   Post-implementation review confirming `SIM-SH-SURFACE-1-2` landed as a minimal seed inventory plus bounded `robots.txt` hint ingestion, and that `SIM-SCR-LANE-1` is now execution-ready.

## 🐙 SIM2 Post-Implementation Shortfalls (2026-02-27)

1. [`2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md`](2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md)  
   Research synthesis for enforcing black-box attacker/control-plane separation by capability construction.
2. [`2026-02-27-sim2-shortfall-2-coverage-contract-governance.md`](2026-02-27-sim2-shortfall-2-coverage-contract-governance.md)  
   Coverage-contract governance research for eliminating drift between SIM2 plan commitments and executable gates.
3. [`2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md`](2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md)  
   Realism-focused research for making `traffic_model` semantics execution-effective and measurable.
4. [`2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md`](2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md)  
   Trust-boundary research for capability-authenticated simulation telemetry tagging across production-capable adversary-sim surfaces.
5. [`2026-02-27-sim2-architecture-shortfall-orchestration-capability.md`](2026-02-27-sim2-architecture-shortfall-orchestration-capability.md)  
   Architecture research for reducing centralized imperative orchestration and moving toward capability-by-construction.
6. [`2026-02-28-sim2-gcr-1-ui-toggle-blackbox-adversary-orchestration-research.md`](2026-02-28-sim2-gcr-1-ui-toggle-blackbox-adversary-orchestration-research.md)  
   Research synthesis for safe UI-toggle control-plane architecture for black-box LLM adversary lifecycle orchestration.
7. [`2026-02-28-sim2-gcr-3-ui-toggle-trust-boundary-controls-research.md`](2026-02-28-sim2-gcr-3-ui-toggle-trust-boundary-controls-research.md)  
   Research synthesis for endpoint trust boundaries on adversary toggle control (session/CSRF/replay/throttling/audit controls).
8. [`2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-research.md`](2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-research.md)  
   Research synthesis for capability-safe containerized frontier orchestration (runtime hardening, signed envelopes, bounded channels, fail-closed teardown).
9. [`2026-02-28-sim2-gcr-4-rust-realtime-monitoring-architecture-candidates.md`](2026-02-28-sim2-gcr-4-rust-realtime-monitoring-architecture-candidates.md)  
   Research synthesis for realtime monitoring delivery candidates in Rust/Spin (cursor polling vs SSE, ordering, backpressure, bounded cost).
10. [`2026-02-28-sim2-gcr-9-rust-realtime-benchmark-comparison.md`](2026-02-28-sim2-gcr-9-rust-realtime-benchmark-comparison.md)  
   Benchmark evidence comparing cursor polling and streaming candidate behavior under steady and burst envelopes, with quantitative threshold recommendations.
11. [`2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-research.md`](2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-research.md)  
   Retention-lifecycle research for deterministic purge, bucketed storage, and operator-visible retention health semantics.
12. [`2026-02-28-sim2-gcr-6-monitoring-cost-efficiency-patterns-research.md`](2026-02-28-sim2-gcr-6-monitoring-cost-efficiency-patterns-research.md)  
   Cost-governance research for monitoring pipelines (cardinality, rollups, sampling restrictions, payload/compression budgets, query controls).
13. [`2026-02-28-sim2-gcr-7-telemetry-artifact-security-privacy-controls-research.md`](2026-02-28-sim2-gcr-7-telemetry-artifact-security-privacy-controls-research.md)  
   Security/privacy control research for telemetry and adversary artifacts (classification, secret scrubbing, pseudonymization, retention tiers, incident hooks).
14. [`2026-02-28-sim2-gcr-10-adr-decision-capture-research.md`](2026-02-28-sim2-gcr-10-adr-decision-capture-research.md)  
   ADR-capture synthesis that formalizes selected SIM2 architecture decisions in ADRs `0007`/`0008`/`0009`.
15. [`2026-02-28-sim2-gcr-8-gc6-gc8-gc11-gc14-synthesis.md`](2026-02-28-sim2-gcr-8-gc6-gc8-gc11-gc14-synthesis.md)  
   Cross-track synthesis for `GC-6`/`GC-8`/`GC-11`/`GC-14` with unified quantitative thresholds and implementation sequencing.
