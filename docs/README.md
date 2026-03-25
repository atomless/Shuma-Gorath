# 🐙 Documentation Index

Shuma is currently a shared-host-first control plane with a live-proven bounded feedback loop on Linode. Use the docs below as the current source of truth; deferred edge and historical material is still preserved, but it is now surfaced through curated indexes in the flat dated `plans/` and `research/` directories instead of nested archive folders.

## Start Here

- [`project-principles.md`](project-principles.md) - Project goals and decision rubric
- [`current-system-architecture.md`](current-system-architecture.md) - Current landed architecture and closed-loop shape
- [`quick-reference.md`](quick-reference.md) - Common commands and API cheat sheet
- [`deployment.md`](deployment.md) - Current deployment and remote operations guidance
- [`testing.md`](testing.md) - Canonical Makefile-only verification paths
- [`configuration.md`](configuration.md) - Runtime configuration reference
- [`privacy-cookie-disclosure-template.md`](privacy-cookie-disclosure-template.md) - Deployer-ready privacy and cookie notice starting point
- [`dashboard.md`](dashboard.md) - Dashboard entry doc
- [`dashboard-tabs/README.md`](dashboard-tabs/README.md) - Per-tab operator docs

## Current Mainline

- [`plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md`](plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md) - Shared-host-first direction update
- [`plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md) - Mainline sequencing through the first closed loop
- [`research/2026-03-22-live-linode-feedback-loop-proof.md`](research/2026-03-22-live-linode-feedback-loop-proof.md) - Live proof of the current shared-host loop
- [`research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md`](research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md) - Latest closeout on adversary-sim truth before the Monitoring overhaul

## Topic Indexes

- [`plans/README.md`](plans/README.md) - Curated planning index by topic
- [`research/README.md`](research/README.md) - Curated research index by topic
- [`adr/README.md`](adr/README.md) - Architecture decision records
- [`module-boundaries.md`](module-boundaries.md) - Domain boundaries and split guidance

## Product And Operator References

- [`api.md`](api.md) - Admin and public API reference
- [`security-hardening.md`](security-hardening.md) - Deployment hardening checklist
- [`observability.md`](observability.md) - Observability and Prometheus guidance
- [`bot-defence.md`](bot-defence.md) - Layered defence model
- [`challenge-verification.md`](challenge-verification.md) - Human verification strategy
- [`maze.md`](maze.md) - Maze crawler deception
- [`tarpit.md`](tarpit.md) - Tarpit behavior and bounded-cost progression
- [`fingerprinting-terminology.md`](fingerprinting-terminology.md) - Canonical fingerprinting terms
- [`fingerprinting-signal-planes.md`](fingerprinting-signal-planes.md) - Signal-plane and trust-boundary model
- [`fingerprint-edge-adapter-guide.md`](fingerprint-edge-adapter-guide.md) - Edge fingerprint adapter extension guide
- [`adversarial-operator-guide.md`](adversarial-operator-guide.md) - Red Team/operator guidance
- [`sim2-real-adversary-traffic-contract.md`](sim2-real-adversary-traffic-contract.md) - Real-traffic adversarial contract
- [`frontier-data-governance.md`](frontier-data-governance.md) - Frontier payload governance

## Deferred And Historical

- [`deferred-edge-gateway.md`](deferred-edge-gateway.md) - Truthful current status of the later edge gateway track
- [`plans/README.md`](plans/README.md) - Current planning index plus historical baselines and outdated deferred-edge notes
- [`research/README.md`](research/README.md) - Current research index plus historical baselines and outdated deferred-edge notes

## Repo Workflow And Backlog

- [`../CONTRIBUTING.md`](../CONTRIBUTING.md)
- [`../AGENTS.md`](../AGENTS.md)
- [`../todos/todo.md`](../todos/todo.md)
- [`../todos/blocked-todo.md`](../todos/blocked-todo.md)
- [`../todos/security-review.md`](../todos/security-review.md)
