# Benchmark Fleet And Intelligence Enrichment Contract

Date: 2026-03-20
Status: Proposed

Related context:

- [`../research/2026-03-20-benchmark-fleet-and-intelligence-enrichment-research-synthesis.md`](../research/2026-03-20-benchmark-fleet-and-intelligence-enrichment-research-synthesis.md)
- [`2026-03-20-benchmark-suite-v1-design.md`](./2026-03-20-benchmark-suite-v1-design.md)
- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](./2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](./2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](./2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

## Objectives

1. Define how later fleet-level or central-intelligence evidence may enrich Shuma's benchmark system.
2. Keep local benchmark truth anchored to `benchmark_suite_v1`, `benchmark_results_v1`, and `operator_snapshot_v1`.
3. Prevent shared intelligence from silently changing local success semantics or controller authority.
4. Keep the later project-evolution loop benchmark-driven without turning the Git repository into a live shared-intelligence transport.

## Non-goals

1. Implementing fleet or shared benchmark transport in this tranche.
2. Defining the later central-intelligence service architecture in full.
3. Changing the current local `benchmark_suite_v1` or `benchmark_results_v1` semantics.
4. Allowing external evidence to mutate local benchmark statuses directly.
5. Letting community or fleet evidence bypass human-friction or local-policy safety boundaries.

## Core Design Decision

Later fleet or central-intelligence evidence should enter the benchmark system through a separate optional advisory contract, `benchmark_enrichment_v1`, not by mutating:

1. `benchmark_suite_v1`,
2. `benchmark_results_v1`,
3. or `operator_snapshot_v1`.

That enrichment contract should be allowed to influence only:

1. benchmark scenario selection,
2. benchmark family priority,
3. and bounded family weight bias.

It must not change:

1. the local benchmark family definitions,
2. local family statuses,
3. local budget-distance calculations,
4. or local escalation results on its own.

## Why A Separate Contract Is Necessary

The benchmark layer now has three clean roles:

1. `benchmark_suite_v1` defines what success means.
2. `benchmark_results_v1` reports how the current instance is doing.
3. `operator_snapshot_v1` carries those results into later Monitoring and controller loops.

If fleet or shared intelligence were written directly into those contracts, Shuma would lose the distinction between:

1. local observed truth,
2. static benchmark semantics,
3. and advisory shared context.

That distinction must survive into the later controller and code-evolution loops.

## Proposed `benchmark_enrichment_v1` Shape

Conceptually, the later enrichment contract should look like:

```json
{
  "schema_version": "benchmark_enrichment_v1",
  "generated_at": 0,
  "applicability_scope": "fleet_local",
  "scenario_selection_hints": [],
  "family_priority_hints": [],
  "family_weight_biases": [],
  "sources": [],
  "review_posture": {
    "status": "advisory_only",
    "requires_human_review": true
  }
}
```

This contract is intentionally separate from the local benchmark contracts.

## Enrichment Dimension 1: Scenario Selection

`scenario_selection_hints` should let later fleet or intelligence inputs say:

1. a scenario family is rising in prevalence,
2. a new scenario family should be added to the benchmark set,
3. or a specific adversary pattern should be exercised more frequently.

Each hint should be bounded and typed, for example:

1. `scenario_family`
2. `reason`
3. `priority`
4. `confidence_class`
5. `freshness_ts`
6. `source_ids`

This is a hint to benchmark scheduling or selection, not a statement that the local instance has already failed that benchmark.

## Enrichment Dimension 2: Family Priority

`family_priority_hints` should let later shared evidence raise or lower review salience for benchmark families.

Examples:

1. raise `representative_adversary_effectiveness` because a fleet-local adversary family is shifting rapidly,
2. raise `beneficial_non_human_posture` because verified-agent traffic is becoming operationally important,
3. or raise `suspicious_origin_cost` because a specific suspicious class is growing across the fleet.

This should affect:

1. review ordering,
2. alerting emphasis,
3. and later controller attention or run cadence,

but not benchmark truth.

## Enrichment Dimension 3: Bounded Weight Bias

`family_weight_biases` should allow small bounded weight adjustments for later aggregate judgment.

Allowed behavior:

1. increase emphasis on a family because multiple fresh sources agree it matters,
2. apply capped positive or negative bias within a fixed range,
3. expose the original base weight and the effective adjusted weight.

Forbidden behavior:

1. unbounded or hidden reweighting,
2. changing local per-family status,
3. reducing safety-critical families to zero influence,
4. or letting external evidence directly emit a code-evolution judgment.

## Safety And Governance Rules

### 1. Preserve local safety vetoes

No enrichment input may suppress or down-rank a local hard blocker arising from:

1. `likely_human_friction` outside budget,
2. local policy mismatch in `beneficial_non_human_posture`,
3. or later critical verified-identity safety conditions.

### 2. Keep local stance authoritative

Fleet or shared evidence may suggest attention, but it must not override:

1. local non-human policy stance,
2. local operator objectives,
3. or local allowed-actions boundaries.

### 3. Require source metadata

Every enrichment record must carry:

1. `source_id`
2. `source_scope`
3. `confidence_class`
4. `freshness_ts`
5. `expires_at`
6. `review_required`
7. `evidence_basis`

### 4. Keep Git out of the live transport path

The Git repository may store:

1. static benchmark definitions,
2. design notes,
3. and implementation plans.

It must not be the live transport or system of record for:

1. dynamic fleet benchmark weighting,
2. shared scenario urgency,
3. or real-time intelligence-derived benchmark bias.

## Relationship To Central Intelligence

Central intelligence should later be one source of `benchmark_enrichment_v1`, not the benchmark system itself.

This means:

1. advisory reputation feeds may raise scenario or family priority,
2. curated high-confidence feeds may justify stronger bounded weight bias,
3. but neither class should rewrite local benchmark results or local escalation outcomes.

## Relationship To Monitoring

This contract is not a prerequisite for the Monitoring-overhaul design discussion.

Monitoring should first project:

1. `operator_snapshot_v1`
2. and nested `benchmark_results_v1`

before it later grows any projection of fleet or intelligence emphasis.

If later Monitoring shows enrichment, it should show it as:

1. emphasis metadata,
2. not as rewritten benchmark truth.

## Relationship To Later Controller And Code-Evolution Loops

The later controller loop may consume enrichment to decide:

1. which benchmark families deserve more attention,
2. which scenario families to schedule,
3. and how to rank multiple benchmark concerns.

The later code-evolution loop may consume enrichment to decide:

1. which benchmark families indicate strategic weakness,
2. and which attack patterns deserve code-level response first.

Neither loop should accept enrichment as a direct substitute for local benchmark evidence.

## Resulting Sequencing Rule

This contract should be treated as the completion of the local benchmark-planning tranche.

After this:

1. Monitoring discussion and design can proceed from the current local machine-first contracts.
2. Later central-intelligence architecture should implement this enrichment shape through a real data plane.
3. Later controller and code-evolution planning should consume this contract rather than inventing a new shared-benchmark overlay.
