# Minimal Seed Surface Discovery Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](../plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md`](./2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md)

## Review Goal

Confirm that Shuma's shared-host planning now reflects the stricter fidelity-first stance:

1. minimal operator-defined seeds,
2. strict scope fence,
3. observed traversal telemetry as the map,
4. and no default requirement for a precomputed public-surface catalog.

## What Was Intended

This slice was meant to:

1. remove the heavy default catalog-first posture from the shared-host planning thread,
2. keep only the minimal safety and seed concepts,
3. and make later replay promotion depend on telemetry traces rather than a prebuilt route inventory.

## What Landed

1. A dedicated research synthesis now grounds the change in crawler-discovery reality and existing project direction.
2. A dedicated design note now defines the new model as `scope_fence + minimal_seed_contract + observed_reachable_surface`.
3. The roadmap, Scrapling plan, and backlog wording now all point at this lighter and more realistic posture.

## Architectural Assessment

### 1. Better adversary fidelity

The plan no longer presumes that the adversary starts with an authoritative site map. That is a cleaner fit for real crawler or agent behavior.

### 2. Less likely wasted work

The repo is no longer implicitly committed to building a large discovery-and-catalog subsystem before proving it is needed.

### 3. Cleaner relationship to deterministic memory

Deterministic replay promotion is now conceptually tied to observed traversal traces rather than to a precomputed catalog, which better fits the mature adversary evolution roadmap.

## Result

Treat this planning correction as complete.

The shared-host discovery concept is now reduced to the minimal role it needs to play in Shuma's future adaptive loop.
