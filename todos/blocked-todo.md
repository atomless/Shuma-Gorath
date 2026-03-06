# Blocked TODO Roadmap

Last updated: 2026-03-06

This file holds gated, contingent, or explicitly deferred work that is not execution-ready.
Move an item back into `todos/todo.md` only when its blocking condition is cleared.
Completed work lives in `todos/completed-todo-history.md`.
Security finding validity and closure status live in `todos/security-review.md`.

## P0 Blocked by Shared-Host Discovery and Runtime-Safety Gates

- [ ] SIM-SCR-LANE-1 Three-lane runtime migration and Scrapling integration.
  Blocker: do not start until `SIM-SH-SURFACE-1` is complete with real shared-host evidence per [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md).

- [ ] SIM-LLM-1 Full LLM-orchestrated, instruction-driven, containerized adversary lane as a first-class runtime actor.
  Blocker: keep blocked until shared-host discovery is complete, the runtime lane model is proven safe, and production-safe operating mode gates are approved.

- [ ] SIM-DEPLOY-2 Production-safe adversary-sim operating modes (explicit opt-in, spawn-on-enable lifecycle, strict rate/resource envelopes, kill switch, auditability, and no-impact guarantees for normal user traffic).
  Blocker: production availability remains gated by [`docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md`](../docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md).

## P1 Blocked by Roadmap Reprioritisation After Deployment Baseline

- [ ] SIM-BREACH-REPLAY-1 External breach to replayable attack pipeline.
  Blocker: defer until shared-host deployment readiness and discovery baseline are complete, then re-assess capture, retention, and replay governance against the deployed operating model.

## P1 Blocked by Enterprise Baseline Maturity

- [ ] DEP-ENT-6 Optional asynchronous mirror of high-confidence bans to Akamai Network Lists.
  Blocker: wait until `DEP-ENT-1..5` establish the authoritative enterprise distributed-state baseline.

- [ ] OUT-4 ADR for non-Redis external integrations (for example webhook notifications or cross-service sync) that defines the approved pattern in Spin (`allowed_outbound_hosts` expansion vs sidecar/bridge service).
  Blocker: wait until a concrete non-Redis integration target is approved.

- [ ] OUT-5 External transport design for non-stub `challenge_engine=external` and `maze_tarpit=external`.
  Blocker: wait until there is an approved external provider path after the baseline deployment work is complete.
