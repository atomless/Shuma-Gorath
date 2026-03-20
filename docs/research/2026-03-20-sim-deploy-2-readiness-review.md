Date: 2026-03-20
Status: Active readiness review

Related context:

- [`2026-03-03-adversary-sim-production-availability-decision-criteria.md`](./2026-03-03-adversary-sim-production-availability-decision-criteria.md)
- [`2026-03-02-adversary-toggle-incident-report-and-lifecycle-invariants.md`](./2026-03-02-adversary-toggle-incident-report-and-lifecycle-invariants.md)
- [`../plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](../plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
- [`../../Makefile`](../../Makefile)

# Purpose

Assess whether `SIM-DEPLOY-2` is execution-ready relative to:

1. the current runtime implementation,
2. the active backlog and roadmap ordering,
3. and the longer-horizon mature adversary-sim plan.

This note is not another production-availability decision. Production availability is already settled. The question here is whether Shuma can start the production operating-envelope hardening tranche now, and in what order the tranche should run.

# Current Architectural Position

## 1. The production-capable runtime substrate is already landed

Shuma already has the main substrate that `SIM-DEPLOY-2` is supposed to harden:

1. backend-owned heartbeat and generation lifecycle,
2. read-only status path,
3. explicit control command path,
4. supervisor diagnostics,
5. and production-visible availability by default.

That means `SIM-DEPLOY-2` is not blocked on a missing architectural foundation. It is blocked only on closing operating-envelope gaps and contract drift risks.

## 2. Telemetry separation is no longer the blocker

The monitoring and hot-read work now keeps adversary-sim evidence separate from live operator ingress rather than silently blending it into live-only summaries.

That matters because `SIM-DEPLOY-2-2` needs credible no-impact verification for normal user traffic under operator use. Shuma now has the telemetry shape needed to prove that without inventing a second semantic model first.

## 3. Shared-host scope and Scrapling remain downstream

The mature-sim roadmap is now explicit that:

1. `SIM-DEPLOY-2` and the minimal shared-host scope-and-seed gate are the prerequisites for Scrapling,
2. not the other way around,
3. and neither a precomputed public-surface catalog nor later export tooling is a prerequisite to start `SIM-DEPLOY-2`.

So `SIM-SH-SURFACE-1` is adjacent work, not a blocker against starting this tranche.

# Readiness Findings

## 1. `SIM-DEPLOY-2` is startable now

No missing architectural blocker remains between the current codebase and the first `SIM-DEPLOY-2` execution slice.

The runtime already exposes:

1. lifecycle diagnostics,
2. hard guardrail constants,
3. explicit off-state inertness diagnostics,
4. internal supervisor beat semantics,
5. and production-default surface availability.

That is enough to begin hardening work immediately.

## 2. There is one execution prerequisite that should be cleared first

The focused lifecycle verification target is currently weaker than its name implies.

`make test-adversary-sim-lifecycle` still references older stale-state selector names, but the current Rust tests were renamed when the status read path became read-only. The target still passes because the remaining selectors and Python checks are healthy, but the stale-state and prior-process cases are not actually being exercised by the `make` target today.

This is not a blocker against `SIM-DEPLOY-2` in principle, but it should be the first execution slice because later `SIM-DEPLOY-2` acceptance should not rely on a truth-in-name mismatch in the focused lifecycle gate.

## 3. The largest remaining architectural risk is still dual desired-state authority

Shuma still carries two desired-state mechanisms:

1. the runtime adversary-sim enabled override,
2. and `ControlState.desired_enabled`.

Status, beat, and control handlers currently reconcile across those paths. That works, but it keeps the exact class of drift that earlier lifecycle incidents identified:

1. stale-enabled illusion,
2. control/status disagreement,
3. and asymmetric reconcile behavior.

This should be the first substantial code tranche after the verification-target cleanup.

# Sequencing Assessment

## Recommended execution order

1. `SIM-DEPLOY-2-0` Refresh lifecycle verification target truthfulness.
2. `SIM-DEPLOY-2-5` Collapse desired state to one backend source of truth.
3. `SIM-DEPLOY-2-1` Codify the production-default runtime lane and resource posture.
4. `SIM-DEPLOY-2-2` Add explicit production kill-switch proof, diagnostics closure, and no-impact verification.
5. `SIM-DEPLOY-2-3` Refresh deployment and operator docs plus evidence receipts.

## Why this is the right order

### First: verification truthfulness

The tranche needs a trustworthy fast gate before hardening claims depend on it.

### Second: desired-state unification

This is the highest-leverage risk reducer because it simplifies:

1. control semantics,
2. status semantics,
3. beat semantics,
4. and later kill-switch reasoning.

### Third: posture codification

Once there is one source of truth, the production-default posture can be documented and surfaced without ambiguity.

### Fourth: no-impact proof

No-impact verification should be built against the settled lifecycle contract, not while desired-state semantics are still in flux.

### Fifth: deployment/operator collateral

Docs and evidence receipts should describe the final contract, not an intermediate one.

# Non-Blockers And Boundaries

## 1. Verified identity is not a blocker for `SIM-DEPLOY-2`

Verified identity is a blocker for later mature adversary-sim expansion, especially when Shuma wants to model verified beneficial agents and spoofed signed-agent traffic. It is not a blocker for the production operating-envelope tranche itself.

## 2. `SIM-DEPLOY-2` is a local-runtime hardening tranche, not a lane-expansion tranche

The first slices should stay scoped to:

1. lifecycle truth,
2. runtime posture,
3. kill-switch and diagnostic exactness,
4. and operator-proof.

They should not bundle in:

1. Scrapling lane work,
2. lane-selector migration,
3. or identity-lane design.

# Evidence Expectations For Completion

`SIM-DEPLOY-2` should not be treated as complete until it has:

1. a truthful focused lifecycle `make` gate,
2. one desired-state authority in the backend lifecycle path,
3. explicit production posture and kill-switch docs,
4. proof that live operator traffic summaries remain separate from adversary-sim evidence while adversary-sim is active,
5. and deployment/operator guidance that treats adversary-sim as a first-class production operating path rather than a dev exception.

# Outcome

Treat `SIM-DEPLOY-2` as execution-ready now.

There is no roadmap blocker left in front of it.

The main prerequisite is local to the tranche itself:

1. fix the lifecycle gate truthfulness first,
2. then remove the remaining dual desired-state architecture,
3. then harden and document the production operating envelope on top of that cleaner lifecycle base.
