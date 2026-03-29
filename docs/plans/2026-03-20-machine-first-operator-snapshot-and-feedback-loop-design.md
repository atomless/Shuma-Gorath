# Machine-First Operator Snapshot and Feedback Loop Design

Date: 2026-03-20
Status: Proposed

Related context:

- [`../research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](../research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`../research/2026-03-20-benchmark-suite-v1-research-synthesis.md`](../research/2026-03-20-benchmark-suite-v1-research-synthesis.md)
- [`../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md`](../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md)
- [`../research/2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md`](../research/2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md)
- [`../research/2026-03-20-monitoring-diagnostics-ownership-post-implementation-review.md`](../research/2026-03-20-monitoring-diagnostics-ownership-post-implementation-review.md)
- [`2026-03-15-agentic-era-oversight-design.md`](./2026-03-15-agentic-era-oversight-design.md)
- [`2026-03-20-benchmark-suite-v1-design.md`](./2026-03-20-benchmark-suite-v1-design.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](./2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

## Objectives

1. Give Shuma one machine-readable operator contract that can later feed both a scheduled frontier-model controller and a thin human Game Loop tab.
2. Let Shuma optimize toward explicit bot-exclusion, suspicious-cost, and human-friction budgets instead of ad hoc dashboard interpretation.
3. Keep request-path behavior deterministic and Rust-owned.
4. Keep Diagnostics as the place for raw subsystem inspection, transport detail, and bounded drill-downs.
5. Make the first future agent loop config-diff-only, not code-change-first.
6. Prepare a second, later loop where Shuma's own code evolution is informed by explicit benchmark evidence rather than intuition.
7. Keep the operator and controller loops anchored to observed telemetry as the authoritative map of what the adversary actually reached and what Shuma actually did.

## Non-goals

1. Building the scheduled LLM controller in this tranche.
2. Replacing the existing bounded monitoring summary with raw event feeds for controller use.
3. Running any model in the request path.
4. Allowing the first automated loop to change trust roots, identity allowlists, provider backends, or code.
5. Designing the final human Monitoring layout before the machine contract exists.

## Core Design Decision

Shuma should treat the long-term Monitoring destination as a backend contract, not as a human dashboard.

The primary artifact becomes:

1. `operator_snapshot_v1`

That snapshot is:

1. bounded,
2. machine-readable,
3. exactness-tagged,
4. windowed,
5. and structured around operator objectives and budget distance.

The future human Game Loop tab is then a projection over selected `operator_snapshot_v1` sections.

## Two Feedback Loops

Shuma should explicitly model two related but distinct feedback loops.

### 1. Instance tuning loop

Scope:

1. one Shuma instance or site,
2. one bounded observation window,
3. one bounded config-diff action surface.

Purpose:

1. tighten or loosen thresholds,
2. enable or disable existing defences,
3. adjust routing,
4. and confirm or roll back the result.

Inputs:

1. `operator_objectives_v1`,
2. `operator_snapshot_v1`,
3. `allowed_actions_v1`,
4. adversary-sim evidence,
5. and later central-intelligence enrichment.

Telemetry rule:

1. the loop must prefer observed telemetry and bounded summaries over speculative surface inventories,
2. and any replay-promotion candidate or adversary-surface understanding carried into this loop must be derived from what the telemetry actually observed.

### 2. Project evolution loop

Scope:

1. many instances over time,
2. fleet-level evidence plus benchmark suites,
3. code and design evolution rather than only config changes.

Purpose:

1. detect when the current Shuma codebase is no longer winning the cost asymmetry race,
2. identify missing capabilities or weak defences,
3. propose code changes,
4. and later allow reviewed PR creation only when benchmark evidence says the code itself needs to evolve.

The first loop should arrive much earlier. The second loop should remain more review-heavy and explicitly benchmark-driven.

## Required Companion Contracts

### 1. `operator_objectives_v1`

This defines what "good" means for a Shuma instance.

It should include:

1. objective window length and compliance semantics,
2. human-friction budgets,
3. suspicious-origin cost or leakage budgets,
4. allowed non-human posture,
5. adversary-sim benchmark targets,
6. and rollout guardrails for future automated tuning.

It should be explicit that a site may choose different non-human stances, for example:

1. `deny_all_non_human`,
2. `allow_only_named_verified_identities`,
3. `allow_verified_by_category`,
4. `allow_verified_with_low_cost_representation_only`.

### 2. `operator_snapshot_v1`

This is the canonical current-state document.

It should include:

1. metadata and freshness,
2. exactness and evidentiary basis,
3. current objective profile reference,
4. live traffic summaries,
5. shadow assertions,
6. adversary-sim summaries,
7. recent change summaries,
8. runtime/config posture summaries,
9. budget-distance summaries,
10. and action-surface metadata.

### 3. `allowed_actions_v1`

This is the bounded surface a future scheduled controller may propose against.

It should enumerate:

1. allowed config families,
2. allowed keys or key groups,
3. minimum and maximum safe ranges,
4. canary-required families,
5. manual-only families,
6. and permanently forbidden families.

### 4. `benchmark_suite_v1`

This is the project-evolution contract for judging Shuma itself.

It should define benchmark families such as:

1. suspicious-origin cost to defended site,
2. local friction cost imposed on likely-human traffic,
3. success or leakage rate for representative hostile traffic,
4. success rate for beneficial authenticated or declared automation according to local stance,
5. adversary-sim benchmark outcomes by lane and scenario family,
6. and later central-intelligence-informed challenge sets.

### 5. `benchmark_results_v1`

This is the bounded benchmark output that compares:

1. current code and config posture,
2. prior baseline,
3. and candidate code changes where applicable.

It should make code evolution measurable instead of anecdotal.

## Proposed `operator_snapshot_v1` Shape

The exact JSON can evolve, but the contract should look like this conceptually:

```json
{
  "schema_version": "operator_snapshot_v1",
  "window": {
    "start": "...",
    "end": "...",
    "duration_seconds": 3600
  },
  "freshness": { "...": "..." },
  "exactness": { "...": "..." },
  "objectives": { "...": "..." },
  "live_traffic": { "...": "..." },
  "shadow_mode": { "...": "..." },
  "adversary_sim": { "...": "..." },
  "verified_identity": { "...": "..." },
  "recent_changes": { "...": "..." },
  "budget_distance": { "...": "..." },
  "allowed_actions": { "...": "..." }
}
```

### Metadata

Must include:

1. schema version,
2. generation time,
3. window bounds,
4. source document freshness,
5. and exactness basis for each major section.

This lets future agents know whether they are looking at:

1. exact observed counts,
2. bounded rollups,
3. derived runtime classifications,
4. or best-effort drill-downs.

### Live traffic section

Must answer:

1. what lanes live traffic occupied,
2. what outcomes each lane saw,
3. what bytes or cost reached origin,
4. what bytes Shuma served locally,
5. what friction likely-human traffic experienced,
6. and where suspicious traffic still leaked through.

This section should be live-only and explicitly exclude adversary-sim origin.

### Shadow section

Must answer:

1. whether Shuma is in shadow mode,
2. what actions would have been enforced,
3. and where shadow evidence suggests tightening or loosening.

It must not pretend to be a direct paired counterfactual for the same requests under both modes.

### Adversary-sim section

Must answer:

1. what benchmark traffic ran,
2. what outcomes it received,
3. what gaps it exposed,
4. and whether the current config is holding up against known representative scenarios.

This section must be first-class, but separate from live operator ingress.
Its reachable-surface understanding and replay-promotion candidates must be derived from observed traversal telemetry rather than from an independently maintained public-surface catalog.

### Recent changes section

Must answer:

1. what changed recently,
2. whether the change was manual or future-controller initiated,
3. what objective or reason it targeted,
4. and whether enough watch-window evidence has accumulated yet to judge it.

### Budget distance section

This is the most important controller-facing layer.

It should compute, for each budgeted objective:

1. current observed value,
2. target,
3. distance from target,
4. trend direction when safely available,
5. and whether the instance is inside, near, or outside tolerance.

This should let a future controller reason over bounded numeric deltas rather than interpreting prose.

### Benchmark section

`operator_snapshot_v1` itself does not need to contain the entire project benchmark history, but it should contain enough local benchmark summary to feed the instance loop and enough references to support later project-evolution aggregation.

The first clean projection should carry `benchmark_results_v1` directly rather than inventing a second snapshot-local benchmark summary type.

That means the contract should reserve room for:

1. a nested `benchmark_results` section whose internal `suite_version` points at the static `benchmark_suite_v1` registry,
2. the current family statuses, metric deltas, improvement status, and escalation hint already defined by the benchmark contract,
3. recent adversary-sim benchmark deltas,
4. local objective-compliance trend summaries,
5. and references that a later fleet or project benchmark layer can aggregate.

The benchmark section should therefore answer:

1. what benchmark family statuses currently look like,
2. whether the current watch window suggests `observe_longer`, `config_tuning_candidate`, or `code_evolution_candidate`,
3. and which benchmark contract version later Monitoring should project without reinterpretation.

Later fleet or central-intelligence inputs should not rewrite that benchmark section. They should arrive through a separate advisory enrichment layer, as captured in [`2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md`](./2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md), so that local snapshot truth and later shared emphasis stay separate.

## Human Game Loop Implication

The human Game Loop tab should become a thin projection of `operator_snapshot_v1`.

That means:

1. headline sections are driven by snapshot sections,
2. terminology matches the snapshot contract,
3. live, shadow, and adversary-sim remain explicitly separated,
4. and the tab does not introduce a second semantic model.

The tab can be opinionated about layout, but not about truth.

## Diagnostics Implication

Diagnostics remains the home for:

1. subsystem drill-downs,
2. raw recent-event tails,
3. transport or freshness troubleshooting,
4. detailed top offenders,
5. and anything contributor-oriented or not required by the primary operator loop.

Diagnostics should not be the source of truth for future controller inputs.

## Base Feedback Loop

The first target loop should be:

1. load `operator_objectives_v1`,
2. materialize `operator_snapshot_v1`,
3. compare `budget_distance`,
4. propose one bounded config diff from `allowed_actions_v1`,
5. validate that diff,
6. apply it only in permitted families,
7. let live traffic and adversary-sim run,
8. observe the next window,
9. confirm or roll back.

That loop deliberately excludes:

1. code changes,
2. PR creation,
3. trust-root changes,
4. identity allowlist changes,
5. and provider-backend changes.

## Project Evolution Loop

The later loop should look different:

1. collect benchmark results across adversary-sim, selected live outcomes, and later central intelligence,
2. compare those results against `benchmark_suite_v1`,
3. determine whether config-only tuning is insufficient,
4. identify missing or weak defence behavior in the codebase,
5. generate reviewed implementation suggestions,
6. and only later, when explicitly enabled, draft PRs that are judged by the same benchmark suite.

This loop should never be the first automation layer Shuma relies on.

It should be gated behind:

1. benchmark maturity,
2. clear regression criteria,
3. human review,
4. and reproducible before-versus-after evidence.

## Design Consequences For Roadmap Sequencing

The next implementation work should therefore be:

1. `OPS-SNAPSHOT-1` machine-first operator snapshot foundation,
2. benchmark-contract planning for code-evolution criteria,
3. then `MON-OVERHAUL-1` as a thin projection over that foundation,
4. then any future broader operator-facing Tuning work, if reopened through fresh planning, aligned to the same objective and action model,
5. then later `OVR-AGENT-2` for the scheduled recommend-or-apply loop,
6. and only after that the separate code-evolution and PR-generation path.

Status update (2026-03-21):

1. The later feedback-loop closure review and agent-first sequencing review refine this order.
2. After benchmark, objective, decision-evidence, and reconcile truth are complete, the first shared-host agent tweaker loop should now land before `MON-OVERHAUL-1`.
3. Monitoring and Tuning remain projections over the same machine-first contracts, but should project the semantics proven by that first backend loop rather than guess them in advance.
4. See [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md) and [`../research/2026-03-21-agent-first-feedback-loop-sequencing-review.md`](../research/2026-03-21-agent-first-feedback-loop-sequencing-review.md).

## Recommended Acceptance Standard

This design should be considered ready to build against when Shuma has:

1. one objective contract,
2. one operator snapshot contract,
3. one allowed action contract,
4. one recent-change ledger summary,
5. and a roadmap that treats Monitoring UI as a consumer of those contracts rather than as their origin.
