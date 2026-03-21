Date: 2026-03-21
Status: Proposed (direction update)

Related context:

- [`../research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](../research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)
- [`../adversarial-operator-guide.md`](../adversarial-operator-guide.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Re-center Shuma's pre-launch architecture on a shared-host-first control plane so the first adaptive feedback loop can ship without forcing Fermyon to host the dashboard, adversary workers, or later scheduled diagnosis/recommend/apply agents.

# Core Decisions

1. Shared-host is the primary full-runtime target for pre-launch.
2. Edge is a later gateway-only posture, not the near-term home of the full adaptive loop.
3. The shared-host control plane should own the dashboard, admin API, runtime policy composition, adversary heartbeat ownership, Scrapling worker execution, benchmark materialization, and later scheduled diagnosis agents.
4. Edge posture, when revisited, should stay thin: request-time forwarding, trusted signal/header normalization, emergency bounded controls, and telemetry emission back to the shared-host control plane.
5. Adversary discovery keeps the existing realism rule: minimal starting knowledge, then traversal telemetry becomes the reachable-surface map.

# Target Architecture

## Shared-Host Control Plane

This is the mainline hosted Shuma shape:

1. Spin-hosted runtime and dashboard
2. host-side adversary supervisor heartbeat
3. Scrapling worker runtime
4. machine-first operator snapshot and benchmark surfaces
5. later recommend-only diagnosis/reconfigurer agents

This is where the first closed loop should become operational.

## Deferred Edge Gateway Plane

This is the long-term edge posture to revisit later:

1. request-time gateway behavior
2. trusted edge signal ingestion
3. signed forwarding or origin-auth enforcement
4. bounded telemetry emission back to the shared-host control plane
5. optional edge-local emergency controls that do not try to host the full Shuma operating loop

This path must not be allowed to drag the pre-launch sequence away from the shared-host-first loop.

# Immediate Cleanup Tranche

This direction update requires one immediate documentation and backlog tranche:

1. write the architecture review that captures why Fermyon is being shelved as a near-term full-runtime target,
2. update the roadmap so edge distributed-state work is no longer on the main pre-launch critical path,
3. move edge/Fermyon follow-on items out of `todos/todo.md` into `todos/blocked-todo.md`,
4. refresh public docs and agent skills so they no longer present Fermyon as the current primary runtime story,
5. keep historical edge helper docs, but relabel them as deferred gateway-only/historical support rather than the next default destination.

# Deferred Follow-On Architecture Work

After launch-critical shared-host work is in place, revisit the edge story in two steps:

1. `EDGE-GW-ARCH-1`
   - plan the thin edge-gateway plus shared-host control-plane split,
   - define exact state ownership,
   - define what deployment-local sync correctness still matters once the edge is not the full runtime,
   - define the day-2 operations model for that split architecture.
2. `EDGE-GW-ARCH-2`
   - remove or generalize vendor-shaped runtime assumptions,
   - rename or replace `edge-fermyon`-specific semantics where they leak into core architecture,
   - keep provider-specific adapters behind the later edge architecture rather than inside the main runtime model.

# Sequencing Consequence

The active pre-launch mainline should now optimize for:

1. machine-first monitoring and benchmark truth,
2. verified bot identity,
3. shared-host adversary-sim maturity,
4. monitoring/tuning projection over those machine-first contracts,
5. sim retention and privacy gates,
6. and the later scheduled diagnosis/recommend/apply loop.

Enterprise edge distribution and Akamai-edge control expansion remain real long-term work, but they are no longer the next execution-ready slices.

# Exit Criteria For This Direction Update

This plan is satisfied when:

1. the roadmap no longer presents Fermyon/edge distributed-state work as the next mainline pre-launch blocker,
2. the active TODO queue no longer treats the edge-specific follow-on work as execution-ready,
3. public docs and edge skills describe Fermyon as a deferred gateway-only path,
4. and the remaining code-level vendor-shaped assumptions are captured explicitly as later cleanup rather than hidden drift.
