# Blocked TODO Roadmap

Last updated: 2026-03-19

This file holds gated, contingent, or explicitly deferred work that is not execution-ready.
Move an item back into `todos/todo.md` only when its blocking condition is cleared.
Completed work lives in `todos/completed-todo-history.md`.
Security finding validity and closure status live in `todos/security-review.md`.

## P0 Blocked by Shared-Host Discovery and Runtime-Safety Gates

- [ ] SIM-SCR-LANE-1 Three-lane runtime migration and Scrapling integration.
  Blocker: do not start until `SIM-SH-SURFACE-1` is complete with real shared-host evidence per [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md).

- [ ] SIM-LLM-1 Full LLM-orchestrated, instruction-driven, containerized adversary lane as a first-class runtime actor.
  Blocker: keep blocked until shared-host discovery is complete, the runtime lane model is proven safe, and the active `SIM-DEPLOY-2` operating-envelope tranche establishes acceptable production bounds.

## P1 Blocked by Roadmap Reprioritisation After Deployment Baseline

- [ ] SIM-BREACH-REPLAY-1 External breach to replayable attack pipeline.
  Blocker: defer until shared-host deployment readiness and discovery baseline are complete, then re-assess capture, retention, and replay governance against the deployed operating model.

## P1 Deferred Pre-Launch Roadmap Captures

Reference context:
- [`docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`docs/research/2026-03-17-operator-decision-support-telemetry-audit.md`](../docs/research/2026-03-17-operator-decision-support-telemetry-audit.md)
- [`docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](../docs/plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`docs/plans/2026-03-15-agentic-era-oversight-design.md`](../docs/plans/2026-03-15-agentic-era-oversight-design.md)
- [`docs/plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](../docs/plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)

- [ ] SIM-MAT-1 Mature adversary-sim completion roadmap.
  Blocker: do not expand beyond the current deterministic baseline and already-captured shared-host/Scrapling/LLM gates until there is one explicit end-state roadmap that ties deterministic, Scrapling, and containerized frontier lanes into the future tuning and oversight loop.

- [ ] TUNE-SURFACE-1 Complete the Tuning tab and related config-control surfaces as the full operator contract for route, defence, ban, recidive, and intelligence thresholds.
  Blocker: defer execution until the operator monitoring overhaul defines which knobs are actionable, which remain diagnostic, and which are safe for future controller tuning.

- [ ] MON-OVERHAUL-1 Redesign Monitoring as an operator decision surface with explicit enforced-vs-shadow separation and clear attacker-effectiveness vs human-friction visibility.
  Blocker: keep the UI/surface redesign blocked until the active `MON-TEL-1` telemetry-foundation tranche lands and the remaining section-ownership plan is written. The prerequisite lane/outcome/exactness/bootstrap decisions from `2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md` and its linked design contracts are now settled and implemented; the remaining blocker is the controller-grade summary foundation identified in [`../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md): forwarded-versus-local byte attribution, bounded `response_kind` or `policy_source` or `route_action_family` summaries, and closure of the remaining control-path or fail-path outcome gaps.

- [ ] SIM-RET-1 Define a dedicated retention and disposal model for adversary-sim telemetry distinct from real-traffic telemetry.
  Blocker: defer execution until mature adversary-sim lane planning settles the expected telemetry classes, retention value horizon, and audit residue needed after tune-confirm-act loops.

- [ ] CTI-ARCH-1 Plan central-intelligence storage and service architecture, including source-trust model, freshness, governance, and whether Shuma uses a standalone service, managed provider, or other shared data plane.
  Blocker: defer execution until the current local recidive/jitter/intelligence design is ready to be broken into service/API/storage contracts; do not treat the Git repository itself as the default shared-intelligence transport.

- [ ] OVR-AGENT-2 Plan the scheduled agent analyzer/recommender/reconfigurer workflow, including model/runtime choice, config-vs-code scope, and whether PR/code-change suggestions are part of the same system or a separate reviewed path.
  Blocker: defer execution until monitoring, tuning, sim-evidence, and central-intelligence contracts are mature enough that the agent loop can be planned against truthful inputs and bounded outputs. In particular, do not start bounded benchmark/controller design until the controller-grade telemetry foundation gaps from [`../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../docs/research/2026-03-19-controller-readiness-telemetry-foundation-review.md) are closed, because the future agent loop needs benchmark-grade byte, outcome, and route-policy summaries rather than only operator charts.

## P1 Blocked by Enterprise Baseline Maturity

- [ ] DEP-ENT-6 Optional asynchronous mirror of high-confidence bans to Akamai Network Lists.
  Blocker: wait until `DEP-ENT-1..5` establish the authoritative enterprise distributed-state baseline.

- [ ] OUT-4 ADR for non-Redis external integrations (for example webhook notifications or cross-service sync) that defines the approved pattern in Spin (`allowed_outbound_hosts` expansion vs sidecar/bridge service).
  Blocker: wait until a concrete non-Redis integration target is approved.

- [ ] OUT-5 External transport design for non-stub `challenge_engine=external` and `maze_tarpit=external`.
  Blocker: wait until there is an approved external provider path after the baseline deployment work is complete.
