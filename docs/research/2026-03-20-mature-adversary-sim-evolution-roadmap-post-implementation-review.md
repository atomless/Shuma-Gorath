# Mature Adversary-Sim Evolution Roadmap Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`2026-03-20-adversary-evolution-loop-role-synthesis.md`](./2026-03-20-adversary-evolution-loop-role-synthesis.md)

## Review Goal

Confirm that the mature adversary-sim roadmap now reflects the intended role split:

1. deterministic lane as oracle and comparator,
2. emergent lanes as primary discovery inputs,
3. shared-host discovery as a minimal safety and seed gate,
4. and reviewed promotion from emergent finding to deterministic memory.

## What Was Intended

This slice was meant to do four things:

1. stop treating deterministic sim as the primary adaptive feedback lane,
2. promote Scrapling and later frontier-agent lanes to primary adaptive roles,
3. narrow the shared-host discovery gate,
4. and make replay promotion into deterministic memory a roadmap concept rather than an unspoken future idea.

## What Landed

1. A dedicated research synthesis now captures the role split between deterministic, emergent, and diagnosis lanes in `docs/research/2026-03-20-adversary-evolution-loop-role-synthesis.md`.
2. A dedicated mature-sim roadmap now captures the new sequencing and promotion model in `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`.
3. The pre-launch roadmap, Scrapling plan, and active or blocked backlog now reflect that shift instead of keeping the older "full shared-host discovery first" framing as the main gating story.
4. The roadmap now clearly treats reviewed promotion from emergent exploit to deterministic scenario as part of Shuma's long-term adaptive memory.

## Architectural Assessment

### 1. Deterministic traffic now has the right role

The roadmap now uses deterministic traffic as stable memory and comparison, which fits both benchmark logic and regression needs better than using it as the main adaptive input.

### 2. Emergent lanes now have the right importance

Scrapling and later frontier-agent traffic are now clearly positioned as the main discovery inputs for the feedback loop, which is much closer to the actual system goal.

### 3. The shared-host gate is more proportionate

The updated backlog no longer overstates the need for a full discovery artifact workflow before useful emergent-lane work can begin. Scope safety remains mandatory, but it is no longer confused with the whole adaptive loop.

## Shortfalls Found During Review

No new blocker was found.

The main remaining work is implementation:

1. bring the Scrapling lane forward behind the narrower scope gate,
2. and later split the analysis harness from the full auto-apply controller path.

## Result

Treat the mature adversary-sim roadmap capture as complete.

The roadmap now more accurately reflects the evolutionary system Shuma is trying to become, and later implementation can build from that without reopening the same deterministic-versus-emergent ambiguity.
