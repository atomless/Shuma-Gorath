# Minimal Seed And Telemetry Surface Discovery Design

Date: 2026-03-20
Status: Proposed

Related context:

- [`../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md`](../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](./2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](./2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)

## Objectives

1. Replace the older catalog-first discovery model with a minimal and higher-fidelity model.
2. Keep only the shared-host work that is truly necessary for safe emergent crawling.
3. Make the reachable public surface emerge from adversary telemetry rather than from a heavyweight precomputed catalog.
4. Preserve a path for later deterministic replay promotion from observed telemetry traces.

## Non-goals

1. Building a large pre-run public-surface catalog.
2. Requiring `sitemap.xml` or `robots.txt` as authoritative discovery truth.
3. Treating shared-host discovery as a standalone product in front of the adversary loop.
4. Requiring full real-host discovery evidence before the first useful emergent lane can run.

## Core Design Decision

Shuma should model shared-host setup for emergent lanes as:

1. `scope_fence`
2. `minimal_seed_contract`
3. `observed_reachable_surface`

and not as:

1. `inventory_builder`
2. `catalog_compiler`
3. `authoritative_public_surface_map`

Guiding rule:

1. telemetry is the map.
2. If a route or exploit path never appears in the observed traversal telemetry for the active loop, Shuma should not treat it as part of the active adversary-reachable surface unless a narrower safety contract explicitly requires it.

## 1. Scope Fence

This remains mandatory.

Shuma must define:

1. allowed hosts
2. denied path prefixes
3. HTTPS requirement
4. redirect revalidation
5. no IP-literal targets

This is safety, not discovery sophistication.

## 2. Minimal Seed Contract

The operator should provide the smallest seed set needed to start realistic crawling.

Recommended first version:

1. required primary public start URL
2. optional `robots.txt` fetch and parsing
3. optional small explicit extra seed list

The operator should not need to provide or approve a full discovered public-surface catalog before the emergent lane can run.

## 3. Observed Reachable Surface

The adversary harness should discover the reachable surface by traversal.

What Shuma should record:

1. which public URLs were reached,
2. how they were reached,
3. which paths were rejected by scope policy,
4. and which traversal paths led to exploit or benchmark-relevant outcomes.

This observed traversal telemetry is the map.

## Relationship To Scrapling Lane

The Scrapling lane should start from the minimal seed contract and follow links under the scope fence.

It should not depend on a precomputed catalog to behave realistically.

If later operator workflows want exports, those should be exports of observed traversal telemetry, not a parallel authoritative catalog product.

## Relationship To Deterministic Replay Memory

Later deterministic replay promotion should consume:

1. observed traversal traces,
2. exploit-relevant request sequences,
3. and promoted replay candidates,

not a large prebuilt route catalog.

That keeps deterministic memory anchored to real discovered behavior.

## Backlog Consequence

The shared-host backlog should shrink to:

1. fail-closed scope contract
2. minimal seed contract

Everything else should move either:

1. into emergent-lane telemetry work,
2. or into later optional export or curation tooling only if proven necessary.
