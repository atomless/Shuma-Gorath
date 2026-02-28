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

## 🐙 SIM2 Post-Implementation Shortfalls (2026-02-27)

1. [`2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md`](2026-02-27-sim2-shortfall-1-black-box-capability-enforcement.md)  
   Research synthesis for enforcing black-box attacker/control-plane separation by capability construction.
2. [`2026-02-27-sim2-shortfall-2-coverage-contract-governance.md`](2026-02-27-sim2-shortfall-2-coverage-contract-governance.md)  
   Coverage-contract governance research for eliminating drift between SIM2 plan commitments and executable gates.
3. [`2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md`](2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md)  
   Realism-focused research for making `traffic_model` semantics execution-effective and measurable.
4. [`2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md`](2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md)  
   Trust-boundary research for capability-authenticated simulation telemetry tagging in runtime-dev.
5. [`2026-02-27-sim2-architecture-shortfall-orchestration-capability.md`](2026-02-27-sim2-architecture-shortfall-orchestration-capability.md)  
   Architecture research for reducing centralized imperative orchestration and moving toward capability-by-construction.
6. [`2026-02-28-sim2-gcr-1-ui-toggle-blackbox-adversary-orchestration-research.md`](2026-02-28-sim2-gcr-1-ui-toggle-blackbox-adversary-orchestration-research.md)  
   Research synthesis for safe UI-toggle control-plane architecture for black-box LLM adversary lifecycle orchestration.
7. [`2026-02-28-sim2-gcr-3-ui-toggle-trust-boundary-controls-research.md`](2026-02-28-sim2-gcr-3-ui-toggle-trust-boundary-controls-research.md)  
   Research synthesis for endpoint trust boundaries on adversary toggle control (session/CSRF/replay/throttling/audit controls).
8. [`2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-research.md`](2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-research.md)  
   Research synthesis for capability-safe containerized frontier orchestration (runtime hardening, signed envelopes, bounded channels, fail-closed teardown).
