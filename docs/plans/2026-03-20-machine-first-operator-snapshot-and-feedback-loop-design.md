# Machine-First Operator Snapshot and Feedback Loop Design

Date: 2026-03-20
Status: Proposed

Related context:

- [`../research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](../research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`../research/2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md`](../research/2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md)
- [`../research/2026-03-20-monitoring-diagnostics-ownership-post-implementation-review.md`](../research/2026-03-20-monitoring-diagnostics-ownership-post-implementation-review.md)
- [`2026-03-15-agentic-era-oversight-design.md`](./2026-03-15-agentic-era-oversight-design.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](./2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

## Objectives

1. Give Shuma one machine-readable operator contract that can later feed both a scheduled frontier-model controller and a thin human Monitoring tab.
2. Let Shuma optimize toward explicit bot-exclusion, suspicious-cost, and human-friction budgets instead of ad hoc dashboard interpretation.
3. Keep request-path behavior deterministic and Rust-owned.
4. Keep Diagnostics as the place for raw subsystem inspection, transport detail, and bounded drill-downs.
5. Make the first future agent loop config-diff-only, not code-change-first.

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

The future human Monitoring tab is then a projection over selected `operator_snapshot_v1` sections.

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

## Human Monitoring Implication

The human Monitoring tab should become a thin projection of `operator_snapshot_v1`.

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

## Design Consequences For Roadmap Sequencing

The next implementation work should therefore be:

1. `OPS-SNAPSHOT-1` machine-first operator snapshot foundation,
2. then `MON-OVERHAUL-1` as a thin projection over that foundation,
3. then `TUNE-SURFACE-1` aligned to the same objective and action model,
4. then later `OVR-AGENT-2` for the scheduled recommend-or-apply loop.

## Recommended Acceptance Standard

This design should be considered ready to build against when Shuma has:

1. one objective contract,
2. one operator snapshot contract,
3. one allowed action contract,
4. one recent-change ledger summary,
5. and a roadmap that treats Monitoring UI as a consumer of those contracts rather than as their origin.
