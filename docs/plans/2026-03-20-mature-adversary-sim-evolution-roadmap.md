# Mature Adversary-Sim Evolution Roadmap

Date: 2026-03-20
Status: Proposed

Related context:

- [`../research/2026-03-20-adversary-evolution-loop-role-synthesis.md`](../research/2026-03-20-adversary-evolution-loop-role-synthesis.md)
- [`../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md`](../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md)
- [`2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](./2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](./2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](./2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`2026-03-20-benchmark-suite-v1-design.md`](./2026-03-20-benchmark-suite-v1-design.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](./2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

## Objectives

1. Recast adversary sim as the adversary side of a real adaptive feedback loop.
2. Make emergent lanes the primary discovery inputs for diagnosis and tuning.
3. Keep deterministic scenarios as the oracle, comparator, and curated memory of known exploits.
4. Define how emergent findings later become deterministic replay cases and benchmark fixtures.

## Non-goals

1. Implementing Scrapling or frontier-agent lanes in this note.
2. Defining the full later controller runtime in this note.
3. Replacing benchmark contracts with sim-specific semantics.
4. Allowing direct autonomous mutation of the deterministic corpus in the first loop.

## Core Design Decision

Shuma should treat the adversary system as four cooperating roles:

1. `deterministic_oracle`
2. `scrapling_emergent`
3. `frontier_agent`
4. `diagnosis_and_tuning_agent`

These roles do not have equal jobs.

### 1. `deterministic_oracle`

Purpose:

1. release-blocking regression authority,
2. stable before/after comparator,
3. curated memory of known exploit shapes.

It should not be treated as the primary adaptive diagnosis lane.

### 2. `scrapling_emergent`

Purpose:

1. first real adaptive discovery lane,
2. lower-cost crawling, scraping, and request-native pressure,
3. continuous surface discovery and exploit surfacing under bounded scope.

This should be the first primary feedback lane.

Near-term ownership note:

1. Scrapling should be treated as the first truthful lane for crawler and request-native non-human categories.
2. Shuma should continuously track and adopt attacker-relevant upstream Scrapling capability for the defense surfaces Scrapling owns, while keeping every omission explicit and justified.
3. Browser-like Scrapling automation may become useful later, but it should not be merged into the same truth contract until the shared-host runtime, deploy envelope, and coverage receipts are widened and proven.
4. A separate intermediate follow-on is now required: Shuma should freeze an explicit capability matrix and omission ledger for the defense surfaces Scrapling ought to exercise, rather than assuming every missing interaction implies browser-runtime adoption or silently leaving capabilities unused.

### 3. `frontier_agent`

Purpose:

1. later high-capability agentic adversary lane,
2. multi-step exploit attempts,
3. strategic pressure beyond crawler-style discovery.

This should arrive after the Scrapling lane is already producing benchmarkable signal.

### 4. `diagnosis_and_tuning_agent`

Purpose:

1. read `operator_snapshot_v1` and nested `benchmark_results_v1`,
2. propose config changes,
3. and propose deterministic replay candidates from emergent findings.

The first version should be recommend-only.

Later recursive-improvement methodology note:

1. when this role grows beyond the first recommend-only bounded loop, it should begin from `Human-only / private` as a development reference stance,
2. continue bounded episodes while target-not-met and progress-not-flat are both true,
3. and only later broaden into relaxed preset sweeps over operator product stances.

## Promotion Pipeline: Discovery To Memory

This is the most important roadmap addition.

The mature loop should be:

1. emergent lane finds a meaningful exploit pattern,
2. benchmarkable telemetry records its effect,
3. diagnosis agent proposes a config change,
4. diagnosis agent also proposes a deterministic replay candidate,
5. that replay candidate is reduced to a stable representative sequence,
6. and after review it becomes part of the deterministic oracle corpus.

That means deterministic traffic evolves, but through reviewed promotion rather than uncontrolled mutation.

## Shared-Host Discovery Reframing

The older Scrapling plan treated shared-host discovery as the first full gate before emergent-lane execution.

This roadmap narrows that.

Shuma still needs:

1. a fail-closed scope contract,
2. a minimal operator-defined seed contract,
3. and operator-visible scope diagnostics and rejection evidence.

The recommended minimal seed contract is:

1. one required primary public start URL,
2. optional `robots.txt` fetch and parsing,
3. and an optional small explicit extra seed list.

The observed reachable surface should then emerge from traversal telemetry. Shuma should not require a rich precomputed public-surface catalog before Scrapling can become the first adaptive lane, and it should not treat sitemap-derived or precompiled inventory as the adversary's primary knowledge model.

Guiding rule:

1. telemetry is the map.
2. If a page or exploit path never appears in the observed traversal telemetry for the active adversary loop, it does not belong in that loop's working surface map unless a narrower safety contract explicitly requires it.

## Recommended First Closed Loop

The first real evolutionary loop should be:

1. `SIM-DEPLOY-2` production operating envelope,
2. minimal `SIM-SH-SURFACE-1` scope fence and seed contract,
3. `SIM-SCR-LANE-1` Scrapling emergent lane,
4. benchmarkable telemetry via existing snapshot and benchmark contracts,
5. recommend-only diagnosis/tuning harness,
6. reviewed config change,
7. deterministic replay candidate promotion when the emergent finding is stable and important.

This should happen before:

1. full frontier-agent lane,
2. later bounded auto-apply,
3. and code-evolution PR generation.

## Sequencing Consequences

### 1. Deterministic lane stays important

It remains:

1. regression oracle,
2. comparator,
3. and curated memory.

It does not remain the primary adaptive feedback lane.

### 2. Scrapling moves earlier in importance

Scrapling should be treated as the first primary adaptive lane, not just a later optional crawler variant.

That does not mean the current worker already captures all of Scrapling's upstream capability. The widened question now has two parts:

1. which Shuma defense surfaces request-native Scrapling should exercise,
2. and which later capabilities truly require Scrapling browser or stealth runtime.

Attacker-faithfulness principle:

1. adversary lanes must behave the way real attackers would behave for the surfaces they claim to represent,
2. not as neutered half-sims that merely prove a tool can make requests,
3. while still remaining inside Shuma's scope, safety, and evidentiary boundaries.

Upstream capability watch rule:

1. because Scrapling is an external rapidly evolving attacker-grade tool, Shuma should keep a standing watch on new Scrapling releases and official docs,
2. and when upstream adds attacker-relevant capability, Shuma should explicitly adopt it for Scrapling-owned surfaces or record a clear omission reason rather than letting the lane silently drift behind the real attacker capability frontier.

### 3. Frontier lane remains later, not first

The frontier-agent lane is valuable, but it is noisier and costlier. It should deepen the loop after the Scrapling lane and benchmark contracts are already trustworthy.

### 4. Monitoring UI is not the first blocker for the analysis harness

The first diagnosis harness should consume machine-first contracts directly. Monitoring remains the human projection over those contracts, not the prerequisite for machine analysis.

### 5. Product stances and the development reference stance are not the same thing

The later recursive-improvement program may begin from a strict development reference stance.

That does not make the operator-facing stance presets into training controls.

The clean division is:

1. operator presets define live site intent,
2. the later development reference stance defines the first recursive-improvement environment,
3. and later relaxed preset sweeps broaden that environment only after the strict reference loop has stabilized.

### 6. Representativeness must be judged against a canonical non-human taxonomy

The next maturity step is not just "Scrapling runs" or "frontier-agent runs."

Before Shuma treats emergent lanes as autonomous tuning evidence, it should:

1. define the non-human traffic categories it intends to simulate and defend against,
2. build a classifier that can map both simulated and observed traffic into those categories,
3. implement lane behaviors designed to fulfill those categories,
4. and only then judge Scrapling plus frontier-agent coverage jointly against that taxonomy.

This means the representativeness contract is partly owned by the lanes, but the taxonomy they are judged against is owned by Shuma's canonical traffic model, not by either lane in isolation.

In the near term, the thing that should evolve is the fingerprinting and classification quality inside that taxonomy.

Only later, if important non-human traffic persistently falls outside the existing categories, should Shuma consider a governed taxonomy-expansion path.

### 7. Representativeness must be profile-backed, not just lane-backed

The next realism gap is no longer whether the lanes own the right categories. It is whether they emit behavior that is characteristic of those categories.

The adopted execution order on this roadmap is therefore:

1. `SIM-REALISM-1A` profile contract,
2. `SIM-REALISM-1B` Scrapling pacing realism,
3. `ROUTE-NS-1A..1F` route-namespace correction and root-hosted generated public-content site,
4. `SIM-REALISM-1C` Agentic request-mode realism,
5. `SIM-REALISM-1D` real Agentic browser-mode execution,
6. `SIM-REALISM-2A` pressure-envelope realism,
7. `SIM-REALISM-2B` identity-envelope realism,
8. `SIM-REALISM-2C` header and transport-envelope realism,
9. `SIM-REALISM-2D` browser secondary-traffic realism,
10. `SIM-REALISM-2E` long-horizon dormancy and recurrence realism,
11. `SIM-REALISM-2F` per-persona exploration-envelope realism,
12. `SIM-REALISM-2G` traversal-frontier receipts and observer truth,
13. `SIM-REALISM-2H` richer root-host public discoverability without choreography,
14. `SIM-REALISM-2I` trusted-ingress client-IP realism,
15. `SIM-REALISM-2J` explicit degraded-identity wording and receipts,
16. `SIM-REALISM-3A` overlapping multi-lane concurrency realism and explicit Scrapling plus Agentic parallel-mode control,
17. `SIM-REALISM-3B` richer agentic action capability and degraded-fallback realism,
18. `SIM-REALISM-3C` true long-window dormancy and return realism,
19. `SIM-REALISM-3D` deeper transport and network fingerprint realism,
20. `SIM-REALISM-3E` explicit representativeness infrastructure gating.

At the current point in that order, `SIM-REALISM-3E` is now landed as well, so the adversary-realism implementation chain is complete. No further Game Loop or Tuning execution should proceed until the landed representativeness readiness gate reports representative hostile-lane backing in the target environment, because later loop and tuning work would otherwise optimize against attacker traffic that is still only partially or degradedly representative there even though the code path now fails closed truthfully.

Before Shuma should describe Scrapling or Agentic Traffic as representative tuning evidence, it should:

1. freeze explicit per-lane or per-mode realism profiles for cadence, burst shape, dwell, identity rotation, JavaScript or browser propensity, and retry ceilings,
2. land the richer generated public-content site so later realism work runs against a substantially deeper and more realistic public terrain,
3. prove those profiles through runtime receipts rather than by plan metadata alone,
4. replace placeholder agentic browser-mode receipts with real bounded browser-session execution,
5. unclip the current pressure envelope so request-native lanes can actually reach field-grounded burst shapes,
6. add truthful identity, geo, and session realism instead of synthetic local session churn,
7. add coherent header and transport envelopes,
8. model background browser traffic plus longer-horizon recurrence,
9. replace flat discovery caps with per-persona exploration envelopes and traversal-frontier receipts,
10. make the protected host root substantially richer and more publicly discoverable without hidden route hints,
11. restore realistic client-IP posture through trusted ingress rather than local process churn or privileged worker headers,
12. prove bounded overlapping Scrapling plus Agentic pressure instead of only serialized one-lane execution,
13. expand the agentic lane beyond polite GET-only or trivial click-through degraded behavior,
14. model hours-to-days dormancy or return behavior rather than only short-gap re-entry,
15. deepen transport realism or explicitly fail closed to degraded claims where the stack cannot support it,
16. and make representativeness itself an explicit infrastructure gate rather than a hand-waved assumption.

## Acceptance Standard For This Roadmap

This roadmap should be considered adopted when the backlog and sequence make these things explicit:

1. emergent lanes are primary discovery inputs,
2. deterministic oracle is comparator and memory,
3. shared-host work is a minimal safety gate rather than a full first-product loop,
4. and reviewed promotion from emergent exploit to deterministic scenario is a named future step.
5. traversal telemetry is the authoritative adversary-reachable surface map, while any later export or curation tooling remains secondary and derived.
6. lane presence alone is not treated as proof of representative adversary pressure; cadence, identity, transport, and recurrence realism must all be explicit and receipt-backed.
