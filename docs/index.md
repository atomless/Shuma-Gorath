# 🐙 Documentation Index

Use the Makefile as the official workflow. These docs are the source of truth for how to build, run, test, and deploy Shuma-Gorath.

Before running any workflow in these docs, clone the repository locally:

```bash
git clone https://github.com/atomless/Shuma-Gorath.git
cd Shuma-Gorath
```

## 🐙 Core Docs

- [`project-principles.md`](project-principles.md) - Project goals, principles, and decision rubric
- [`adr/README.md`](adr/README.md) - <abbr title="Architecture Decision Record">ADR</abbr> process and template
- [`../CONTRIBUTING.md`](../CONTRIBUTING.md) - Contribution and <abbr title="Pull Request">PR</abbr> standards
- [`../AGENTS.md`](../AGENTS.md) - Instructions for coding agents working in this repository
- [`quick-reference.md`](quick-reference.md) - Command and <abbr title="Application Programming Interface">API</abbr> cheat sheet
- [`testing.md`](testing.md) - Testing guide (Makefile-only)
- [`dashboard.md`](dashboard.md) - Dashboard and admin <abbr title="User Interface">UI</abbr>
- [`dashboard-tabs/README.md`](dashboard-tabs/README.md) - Per-tab dashboard operator docs
- [`deployment.md`](deployment.md) - Production/deploy configuration
- [`../skills/prepare-shared-host-on-linode/SKILL.md`](../skills/prepare-shared-host-on-linode/SKILL.md) - Repo-local agent skill for shared-host Linode setup and deploy handoff preparation
- [`../skills/deploy-shuma-on-linode/SKILL.md`](../skills/deploy-shuma-on-linode/SKILL.md) - Repo-local agent skill for one-command Linode provisioning + deployment
- [`../skills/prepare-shuma-on-akamai-fermyon/SKILL.md`](../skills/prepare-shuma-on-akamai-fermyon/SKILL.md) - Repo-local agent skill for Akamai-edge-only Fermyon setup and deploy handoff preparation
- [`../skills/deploy-shuma-on-akamai-fermyon/SKILL.md`](../skills/deploy-shuma-on-akamai-fermyon/SKILL.md) - Repo-local agent skill for Akamai-edge-only Fermyon deploy execution from a prepared setup receipt; the edge baseline is now live-proven
- [`research/2026-03-12-fermyon-akamai-edge-live-proof.md`](research/2026-03-12-fermyon-akamai-edge-live-proof.md) - Completed live Fermyon/Akamai edge proof and the verified happy path/gotchas
- [`research/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md`](research/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md) - Historical first-pass blocker note for the Fermyon/Akamai edge proof
- [`research/2026-03-06-linode-shared-host-live-proof.md`](research/2026-03-06-linode-shared-host-live-proof.md) - First live proof of the shared-host Linode setup + deploy path
- [`plans/2026-03-07-generic-ssh-remote-maintenance-layer-design.md`](plans/2026-03-07-generic-ssh-remote-maintenance-layer-design.md) - Provider-agnostic `ssh_systemd` day-2 remote operations contract
- [`research/2026-03-15-agentic-era-oversight-research-synthesis.md`](research/2026-03-15-agentic-era-oversight-research-synthesis.md) - Research synthesis for Shuma's agentic-era operating model, verified-agent identity, and bounded autonomous oversight
- [`plans/2026-03-15-agentic-era-oversight-design.md`](plans/2026-03-15-agentic-era-oversight-design.md) - Proposed design for a backend-owned oversight plane, control contract, and budget schema
- [`plans/2026-03-15-agentic-era-oversight-implementation-plan.md`](plans/2026-03-15-agentic-era-oversight-implementation-plan.md) - Phased implementation plan for oversight snapshots, reconcile control, scheduler adapters, and bounded autonomous apply
- [`research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md) - Research synthesis for ban jitter, local repeat-offender escalation, and central intelligence as coordinated agentic-era cost-shaping features
- [`plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md) - Proposed coordinated design for banded ban jitter, local recidive memory, shared intelligence, and oversight-controller tuning
- [`plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-implementation-plan.md`](plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-implementation-plan.md) - Phased implementation plan for the jitter, recidive, intelligence, telemetry, and controller-integration tranches
- [`research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md) - Research synthesis for Web Bot Auth, signed agents, verified bots, and the verified-identity lane Shuma needs for authenticated automated traffic
- [`research/2026-03-17-operator-decision-support-telemetry-audit.md`](research/2026-03-17-operator-decision-support-telemetry-audit.md) - Repo-grounded telemetry audit for the Monitoring overhaul, identifying what operators can already act on, what remains contributor diagnostics, and which telemetry families Shuma still lacks
- [`research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md) - State-of-the-art telemetry research for the Monitoring overhaul, grounded in current Cloudflare, Google, OpenAI, Anthropic, Web Bot Auth, and HTTP Message Signatures guidance for classifying and differentiating human, crawler, assistant, verified-bot, and signed-agent traffic
- [`research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md) - Gap analysis comparing current telemetry collection with the desired operator-grade model under Shuma's hot-read, retention, and bounded-summary cost constraints
- [`research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](research/2026-03-19-controller-readiness-telemetry-foundation-review.md) - Architecture review addendum identifying the remaining controller-grade telemetry foundations Shuma still needs before the Monitoring overhaul and future bounded inside-agent benchmark loops
- [`research/2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md`](research/2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md) - Closeout review confirming the backend monitoring-telemetry foundation is complete and that the next step is the Monitoring-overhaul discussion and section-ownership plan
- [`research/2026-03-20-monitoring-diagnostics-ownership-post-implementation-review.md`](research/2026-03-20-monitoring-diagnostics-ownership-post-implementation-review.md) - Post-implementation review confirming the Monitoring/Diagnostics ownership split landed cleanly and that the next work should move to the substantive Monitoring overhaul
- [`plans/2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md`](plans/2026-03-20-monitoring-and-diagnostics-tab-ownership-plan.md) - UI ownership plan that moves the legacy subsystem-by-subsystem Monitoring surface into a new Diagnostics tab and reserves Monitoring as a clean slate for the operator decision surface overhaul
- [`research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md) - Research synthesis for a machine-first Monitoring destination where future scheduled controllers and later human Monitoring both consume one bounded operator snapshot contract
- [`plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md) - Proposed design for `operator_objectives_v1`, `operator_snapshot_v1`, and `allowed_actions_v1` as the control contract that should precede the human Monitoring overhaul
- [`plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-implementation-plan.md`](plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-implementation-plan.md) - Phased implementation plan for the operator-objective contract, machine-first snapshot materialization, bounded action metadata, and later Monitoring projection
- [`research/2026-03-20-benchmark-suite-v1-research-synthesis.md`](research/2026-03-20-benchmark-suite-v1-research-synthesis.md) - Research synthesis for the first benchmark families Shuma should use to measure suspicious-origin cost, likely-human friction, representative adversary effectiveness, and beneficial non-human posture
- [`plans/2026-03-20-benchmark-suite-v1-design.md`](plans/2026-03-20-benchmark-suite-v1-design.md) - Proposed design for `benchmark_suite_v1` and `benchmark_results_v1`, keeping the instance tuning loop and later project-evolution loop tied to one benchmark contract
- [`plans/2026-03-20-benchmark-suite-v1-implementation-plan.md`](plans/2026-03-20-benchmark-suite-v1-implementation-plan.md) - Phased implementation plan for benchmark-family definition, result-envelope design, config-vs-code escalation rules, and later snapshot/Monitoring alignment
- [`research/2026-03-19-defence-funnel-origin-integrity-review.md`](research/2026-03-19-defence-funnel-origin-integrity-review.md) - Post-implementation review tightening the defence-funnel contract so first-wave stages stay live-safe and recording the immediate follow-on need for origin-aware family counters
- [`plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md) - Prerequisite architecture note for the Monitoring telemetry foundation tranche, defining the lane, outcome-emission, exactness, and bootstrap-ownership decisions that should be settled before implementation
- [`plans/2026-03-18-monitoring-traffic-lane-and-denominator-contract.md`](plans/2026-03-18-monitoring-traffic-lane-and-denominator-contract.md) - Concrete design for Monitoring's canonical traffic-lane vocabulary, denominator boundary, route-family grouping, and the rule that operators tune underlying policy knobs rather than a second analytics-only lane score
- [`plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md) - Concrete design for one authoritative request-outcome telemetry hook, with request-flow-owned finalization, shared rendered-outcome taxonomy, and explicit shadow-mode and byte-accounting semantics
- [`plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md`](plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md) - Follow-on Stage 1 execution plan for the remaining controller-grade telemetry foundations: outcome-attributed bytes, richer bounded backend summaries, and terminal-path coverage closure before the Monitoring overhaul
- [`plans/2026-03-19-monitoring-human-friction-denominator-plan.md`](plans/2026-03-19-monitoring-human-friction-denominator-plan.md) - Narrow implementation plan for `MON-TEL-1-3`, defining the minimum lane-aware denominator and human-friction rate contract needed before the Monitoring UI overhaul
- [`plans/2026-03-19-monitoring-defence-funnel-plan.md`](plans/2026-03-19-monitoring-defence-funnel-plan.md) - Narrow implementation plan for `MON-TEL-1-4`, defining the first normalized defence-effectiveness funnel rows, supported family set, and honest use of `null` for unavailable stages
- [`plans/2026-03-19-monitoring-origin-aware-followup-telemetry-plan.md`](plans/2026-03-19-monitoring-origin-aware-followup-telemetry-plan.md) - Narrow implementation plan for `MON-TEL-1-5D`, separating legacy family follow-up telemetry by origin so live-only operator summaries and richer funnel stages become trustworthy before the Monitoring overhaul
- [`plans/2026-03-18-monitoring-operator-summary-exactness-contract.md`](plans/2026-03-18-monitoring-operator-summary-exactness-contract.md) - Concrete design for operator-summary truth metadata, separating exactness from evidentiary basis so Monitoring can expose exact, derived, and best-effort summaries honestly
- [`plans/2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md`](plans/2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md) - Concrete design for which Monitoring summaries belong in bootstrap, which belong in supporting hot-read documents, and which must remain diagnostics-only to preserve hot-read budget discipline
- [`plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md) - Proposed design for Shuma's verified bot and agent identity subsystem, local authorization policy, and low-cost authenticated-agent lane
- [`plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md) - Phased implementation plan for identity verification, named identity policy, monitoring, and oversight integration
- [`plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md) - Forward roadmap capture for still-missing pre-launch planning tracks and their recommended sequencing
- [`api.md`](api.md) - <abbr title="Application Programming Interface">API</abbr> usage and endpoint details
- [`configuration.md`](configuration.md) - Runtime configuration reference
- [`fingerprinting-terminology.md`](fingerprinting-terminology.md) - Canonical fingerprinting/JS verification terminology map
- [`fingerprinting-signal-planes.md`](fingerprinting-signal-planes.md) - Signal-plane architecture, trust boundaries, and Akamai add-vs-replace behavior
- [`fingerprint-edge-adapter-guide.md`](fingerprint-edge-adapter-guide.md) - How to add a new external edge fingerprint provider adapter
- [`security-hardening.md`](security-hardening.md) - Deployment security checklist
- [`privacy-gdpr-review.md`](privacy-gdpr-review.md) - <abbr title="General Data Protection Regulation">GDPR</abbr>/privacy and cookie/storage compliance review for telemetry and logging
- [`observability.md`](observability.md) - Prometheus/Grafana integration
- [`monitoring-prometheus-parity-audit.md`](monitoring-prometheus-parity-audit.md) - Monitoring widget-to-Prometheus parity matrix and prioritized missing-export plan
- [`bot-defence.md`](bot-defence.md) - Shuma-Gorath layered defence strategy with managed edge bot protection
- [`value-proposition.md`](value-proposition.md) - Research-backed value map and cost-asymmetry positioning by capability
- [`features.md`](features.md) - Feature list and roadmap
- [`challenge-verification.md`](challenge-verification.md) - Human verification strategy
- [`maze.md`](maze.md) - maze crawler deception
- [`tarpit.md`](tarpit.md) - tarpit activation, progression, and bounded-cost behavior
- [`research/README.md`](research/README.md) - Research index (including tarpit research collection and latest re-review addendum)
- [`module-boundaries.md`](module-boundaries.md) - Domain boundary contracts and split prep
- [`plans/2026-02-13-provider-externalization-design.md`](plans/2026-02-13-provider-externalization-design.md) - Provider externalization strategy (self-hosted-first, Akamai-integrated)
- [`plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md) - Deployment-track and adversarial-traffic simulation execution plan
- [`plans/2026-03-05-gateway-first-existing-site-deployment-plan.md`](plans/2026-03-05-gateway-first-existing-site-deployment-plan.md) - Gateway-only plan for existing-site adoption with one policy core and secure upstream integration
- [`research/2026-03-05-gateway-only-spin-architecture-research-synthesis.md`](research/2026-03-05-gateway-only-spin-architecture-research-synthesis.md) - Two-pass gateway architecture synthesis (Spin constraints + codebase impact map)
- [`research/2026-03-05-gateway-first-tranche-conformance-review.md`](research/2026-03-05-gateway-first-tranche-conformance-review.md) - DEP-GW-1 completion conformance review and evidence matrix
- [`research/2026-03-05-gateway-first-post-tranche-cleanup-review.md`](research/2026-03-05-gateway-first-post-tranche-cleanup-review.md) - Post-tranche cleanup findings and gateway follow-on architecture work
- [`plans/2026-02-25-adversarial-simulation-fast-smoke-design.md`](plans/2026-02-25-adversarial-simulation-fast-smoke-design.md) - Initial executable manifest + runner + mandatory fast-smoke gate design
- [`plans/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement-plan.md`](plans/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement-plan.md) - SIM2 black-box capability hardening plan
- [`plans/2026-02-27-sim2-shortfall-2-coverage-contract-governance-plan.md`](plans/2026-02-27-sim2-shortfall-2-coverage-contract-governance-plan.md) - SIM2 coverage-contract governance plan
- [`plans/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism-plan.md`](plans/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism-plan.md) - SIM2 traffic-model realism execution plan
- [`plans/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity-plan.md`](plans/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity-plan.md) - SIM2 simulation telemetry authenticity hardening plan
- [`plans/2026-02-27-sim2-orchestration-capability-architecture-plan.md`](plans/2026-02-27-sim2-orchestration-capability-architecture-plan.md) - SIM2 architecture uplift plan for functional orchestration and capability boundaries
- [`adr/0005-adversarial-lane-coexistence-policy.md`](adr/0005-adversarial-lane-coexistence-policy.md) - Deterministic vs containerized adversarial lane coexistence contract
- [`adr/adversarial-lane-parity-signoff-checklist.md`](adr/adversarial-lane-parity-signoff-checklist.md) - Required checklist template before any deterministic-lane demotion/replacement
- [`research/2026-02-25-llm-adversarial-testing-research-synthesis.md`](research/2026-02-25-llm-adversarial-testing-research-synthesis.md) - LLM-driven adversarial testing research and Shuma implications
- [`research/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md`](research/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md) - Research for black-box capability enforcement
- [`research/2026-02-27-sim2-shortfall-2-coverage-contract-governance.md`](research/2026-02-27-sim2-shortfall-2-coverage-contract-governance.md) - Research for coverage-contract governance hardening
- [`research/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md`](research/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md) - Research for execution-effective traffic realism
- [`research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md`](research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md) - Research for simulation telemetry authenticity controls
- [`research/2026-02-27-sim2-architecture-shortfall-orchestration-capability.md`](research/2026-02-27-sim2-architecture-shortfall-orchestration-capability.md) - Research for orchestration/capability architecture uplift
- [`adversarial-operator-guide.md`](adversarial-operator-guide.md) - Operator triage/tuning guidance for adversarial simulation failures and dashboard orchestration lifecycle
- [`sim2-real-adversary-traffic-contract.md`](sim2-real-adversary-traffic-contract.md) - Architecture contract and evidence schema for valid real-traffic adversarial runs
- [`frontier-data-governance.md`](frontier-data-governance.md) - Frontier payload allowlist, redaction pipeline, and retention matrix
- [`../todos/todo.md`](../todos/todo.md) - Active execution-ready backlog
- [`../todos/blocked-todo.md`](../todos/blocked-todo.md) - Blocked and contingent backlog
- [`../todos/security-review.md`](../todos/security-review.md) - Security audit notes / backlog
