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
- [`../skills/deploy-shuma-on-linode/SKILL.md`](../skills/deploy-shuma-on-linode/SKILL.md) - Repo-local agent skill for one-command Linode provisioning + deployment
- [`../skills/deploy-shuma-on-akamai-fermyon/SKILL.md`](../skills/deploy-shuma-on-akamai-fermyon/SKILL.md) - Repo-local agent skill for staged enterprise Akamai/Fermyon edge deployment
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
- [`../todos/security-review.md`](../todos/security-review.md) - Security audit notes / backlog
