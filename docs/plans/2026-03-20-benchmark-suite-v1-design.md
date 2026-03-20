# Benchmark Suite v1 Design

Date: 2026-03-20
Status: Proposed

Related context:

- [`../research/2026-03-20-benchmark-suite-v1-research-synthesis.md`](../research/2026-03-20-benchmark-suite-v1-research-synthesis.md)
- [`../research/2026-03-20-benchmark-fleet-and-intelligence-enrichment-research-synthesis.md`](../research/2026-03-20-benchmark-fleet-and-intelligence-enrichment-research-synthesis.md)
- [`../research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](../research/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](./2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md`](./2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](./2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

## Objectives

1. Give Shuma one explicit benchmark contract for deciding whether outcomes are improving along the bot-cost versus human-friction frontier.
2. Keep the benchmark model usable by both the instance tuning loop and the later project-evolution loop.
3. Make benchmark families machine-readable, bounded, and exactness-aware.
4. Prevent Monitoring from inventing a human-only success model ahead of the machine-first control contract.
5. Keep the first version small enough to align with the telemetry Shuma already has or has explicitly planned.

## Non-goals

1. Implementing benchmark execution in this tranche.
2. Defining every future benchmark Shuma may ever want.
3. Creating a benchmark family per defence module.
4. Treating raw event tails or dashboard charts as the benchmark contract.
5. Allowing benchmark results to directly authorize code changes without a later reviewed path.

## Core Design Decision

`benchmark_suite_v1` should start with four benchmark families:

1. `suspicious_origin_cost`
2. `likely_human_friction`
3. `representative_adversary_effectiveness`
4. `beneficial_non_human_posture`

Each family should define:

1. the question it answers,
2. the eligible population,
3. the metrics it requires,
4. the target or budget contract,
5. the comparison modes it supports,
6. and its capability gate.

This suite should be the measuring stick for both:

1. the later per-instance tune-confirm-repeat loop,
2. and the later benchmark-driven project-evolution loop.

## Top-Level `benchmark_suite_v1` Shape

Conceptually, the suite definition should look like:

```json
{
  "schema_version": "benchmark_suite_v1",
  "families": [
    {
      "id": "suspicious_origin_cost",
      "eligibility": { "...": "..." },
      "targets": { "...": "..." },
      "metrics": [{ "...": "..." }]
    }
  ],
  "comparison_modes": ["prior_window", "baseline", "candidate"],
  "decision_boundaries": { "...": "..." }
}
```

The definition should stay declarative and avoid code-path-specific trivia.

## Benchmark Family 1: `suspicious_origin_cost`

### Decision question

1. How much suspicious traffic is still consuming defended-site cost, and how much of that cost is being shifted back onto Shuma?

### Why this family exists

1. It measures Shuma's central cost-asymmetry goal.
2. It can be improved through config tuning, but persistent misses may also reveal code gaps.
3. It is meaningful for both live traffic and representative adversary scenarios.

### First-wave metric set

1. `suspicious_forwarded_request_rate`
2. `suspicious_forwarded_byte_rate`
3. `suspicious_short_circuit_rate`
4. `suspicious_locally_served_byte_share`

### Data requirements

1. live-vs-sim separation,
2. suspicious-lane or suspicious-family denominators,
3. forwarded-versus-local byte attribution,
4. outcome exactness and basis metadata.

### Notes

1. This family should remain live-safe by default, with adversary-sim evidence reported in a parallel comparison subject rather than mixed into live ingress.
2. Total suspicious counts alone are not enough; the family needs eligible-population ratios.

## Benchmark Family 2: `likely_human_friction`

### Decision question

1. How much friction or denial is Shuma imposing on likely-human or interactive traffic, and is that within the site's target budget?

### Why this family exists

1. It is the main counterbalance to bot-cost shaping.
2. It is the clearest reason for a future controller to loosen thresholds.
3. It gives project-evolution work a principled target beyond "blocked more bots."

### First-wave metric set

1. `likely_human_friction_rate`
2. `interactive_friction_rate`
3. `likely_human_hard_block_rate`
4. later, `likely_human_added_cost_proxy` when Shuma collects a truthful cost or latency proxy

### Data requirements

1. lane-aware denominators,
2. friction outcome classification,
3. hard-block classification,
4. exactness and basis metadata.

### Notes

1. The first version should prefer measured friction outcomes over inferred sentiment.
2. If a stronger latency proxy is not yet available, the contract should say so explicitly rather than pretending otherwise.

## Benchmark Family 3: `representative_adversary_effectiveness`

### Decision question

1. Against representative hostile scenarios, how effective is the current Shuma posture?

### Why this family exists

1. Live traffic alone is not enough to validate the arms race.
2. Adversary-sim is already a first-class evidence source.
3. This family provides the bridge from instance tuning toward code evolution.

### First-wave metric set

1. `scenario_goal_success_rate`
2. `scenario_origin_reach_rate`
3. `scenario_escalation_rate`
4. `scenario_regression_status`

### Data requirements

1. adversary-sim scenario families and run outcomes,
2. scenario-level objective mapping,
3. bounded recent-run or benchmark-result summaries,
4. explicit origin separation from live traffic.

### Notes

1. This family should compare named scenario families rather than free-form run narratives.
2. It should remain separate from live traffic summaries even when both inform the same tuning decision.

## Benchmark Family 4: `beneficial_non_human_posture`

### Decision question

1. Is Shuma treating beneficial or authenticated non-human traffic in line with the site's declared stance?

### Why this family exists

1. The agentic era requires Shuma to measure both exclusion and intended allowance.
2. Different sites will adopt different policy stances, so this family must be policy-aware.
3. Verified-identity work needs a prepared benchmark home rather than inventing one later.

### First-wave metric set

1. `allowed_as_intended_rate`
2. `friction_mismatch_rate`
3. `deny_mismatch_rate`
4. `coverage_status`

### Data requirements

1. local non-human stance,
2. declared or verified identity classes,
3. policy outcome classification,
4. capability gating while identity features are still landing.

### Notes

1. This family should return `not_applicable` or `not_yet_supported` when the required identity capability is absent.
2. A site with `deny_all_non_human` posture should still be benchmarkable here; success then means consistent denial with minimal unintended allowance.

## Common Family Rules

Every family must define:

1. eligible population,
2. numerator and denominator,
3. exactness and evidentiary basis,
4. target and tolerance band,
5. comparison modes,
6. and capability gates.

Every result should be able to say:

1. `inside_budget`
2. `near_limit`
3. `outside_budget`
4. `not_applicable`
5. `not_yet_supported`
6. `insufficient_evidence`

## `benchmark_results_v1` Direction

`benchmark_results_v1` should carry:

1. suite version,
2. subject kind,
3. baseline reference,
4. watch window,
5. per-family statuses and metric deltas,
6. overall improvement or regression flags,
7. and a bounded escalation hint.

Subject kinds should include:

1. `current_instance`
2. `prior_baseline`
3. `candidate_config`
4. `candidate_code`

The results contract should remain bounded and summary-oriented.

## Config-vs-Code Decision Boundary

The benchmark layer should support an explicit decision boundary:

1. `config_tuning_candidate`
   - benchmark miss appears addressable by the existing allowed action surface,
   - no missing capability is implied,
   - and the tradeoff frontier still looks navigable with current code.
2. `code_evolution_candidate`
   - repeated misses persist across windows, scenarios, or instances,
   - required signal or action surface is absent,
   - or the current capability set cannot reduce hostile cost without exceeding human-friction budgets.

This boundary must remain explicit and benchmark-driven rather than anecdotal.

## `escalation_hint` Contract Direction

The first machine-readable escalation contract should remain bounded and review-aware.

`benchmark_results_v1.escalation_hint` should carry:

1. `availability`
2. `decision`
3. `review_status`
4. `trigger_family_ids`
5. `candidate_action_families`
6. `blockers`
7. `note`

### First-wave semantics

1. `review_status` should remain `manual_review_required`.
2. `decision` should use only:
   - `config_tuning_candidate`
   - `observe_longer`
   - `code_evolution_candidate`
3. The first slice may remain `partial_support` when the decision is derived from the current watch window plus the current action surface but baseline history is not yet materialized.
4. `candidate_action_families` should be derived from the existing `allowed_actions_v1` family surface rather than from UI-local heuristics.
5. `blockers` should explain why a stronger action is not yet justified, for example:
   - no outside-budget benchmark family is currently observed,
   - evidence is still insufficient,
   - or no existing config/action family can plausibly address the active benchmark miss.

### First-wave decision rules

1. If no benchmark family is outside budget, the hint should remain `observe_longer`.
2. If an outside-budget family maps to an existing config/action family, the hint should become `config_tuning_candidate`, even when the relevant config family is manual-review or canary-gated.
3. If an outside-budget family has no matching config/action family or requires an absent capability, the hint should become `code_evolution_candidate`.
4. The first slice must not require baseline history to return a bounded escalation hint, but it must say explicitly when the decision is based on current-window evidence only.

## Relationship To `operator_snapshot_v1`

`benchmark_suite_v1` should not bypass `operator_snapshot_v1`.

The intended layering is:

1. telemetry and hot-read summaries,
2. `operator_snapshot_v1`,
3. `benchmark_results_v1`,
4. Monitoring and later agent/controller projections.

For the first projection tranche, `operator_snapshot_v1` should carry `benchmark_results_v1` directly rather than a second snapshot-local benchmark wrapper. The nested benchmark payload's `suite_version` is the local reference back to `benchmark_suite_v1`.

That keeps one semantic model for machine and human consumers.

## Relationship To Monitoring

The future Monitoring tab should render benchmark-family status and supporting snapshot sections.

It should not define its own alternative headline semantics for:

1. success,
2. improvement,
3. regression,
4. or budget breach.

## Relationship To Central Intelligence

Central intelligence should later enrich:

1. benchmark scenario selection,
2. benchmark weighting,
3. and cross-instance comparison context.

It should not be the transport for the benchmark contract itself, and it must not turn the Git repository into the fleet-intelligence data plane.

That later enrichment now has a dedicated contract in [`2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md`](./2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md). The important rule is that central intelligence may enrich benchmark emphasis, but it must not redefine local benchmark truth.
