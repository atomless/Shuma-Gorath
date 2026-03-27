Date: 2026-03-27
Status: Proposed, execution blocked

Related context:

- [`../research/2026-03-27-human-friction-calibration-ring-review.md`](../research/2026-03-27-human-friction-calibration-ring-review.md)
- [`../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`../research/2026-03-27-game-loop-current-state-and-gap-review.md`](../research/2026-03-27-game-loop-current-state-and-gap-review.md)
- [`2026-03-27-game-loop-board-state-refactor-plan.md`](2026-03-27-game-loop-board-state-refactor-plan.md)
- [`2026-03-19-monitoring-human-friction-denominator-plan.md`](2026-03-19-monitoring-human-friction-denominator-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Plan the later human-friction calibration ring so Shuma can measure the burden imposed on real humans by a defense configuration without weakening or polluting the strict adversary-sim proof ring.

# Core Decisions

1. Human-friction evidence must come from real human traversal or an explicitly human-operated calibration workflow.
2. Adversary-sim traffic must never count as human-friction evidence.
3. The ring must preserve journey success and defense burden separately instead of collapsing them into one scalar.
4. Human-friction comparison must be tied to config revision and episode lineage.
5. The ring stays blocked from execution until the strict Scrapling loop is trusted enough that added human measurement will calibrate a real defense rather than a still-confused board state.

# Planned Shape

## `HUM-FRIC-1A`

### Human-operated journey contract

Define the canonical journeys and success semantics.

Acceptance criteria:

1. the repo names a small bounded human journey set,
2. each journey defines what counts as `content_reached`,
3. and each journey defines which route or defense surfaces may legitimately add friction.

## `HUM-FRIC-1B`

### Host-side friction event and denominator contract

Define the telemetry needed to measure burden on those journeys.

Acceptance criteria:

1. telemetry distinguishes human-operated calibration traffic from adversary-sim traffic,
2. the ring records challenge burden, extra latency, retry count, and abandonment or success,
3. and each event carries config revision plus journey context.

## `HUM-FRIC-1C`

### Budget and comparison contract

Define how the ring will decide whether current human burden is acceptable.

Acceptance criteria:

1. the ring names short-window and long-window human-friction budgets,
2. it compares current behavior against a retained baseline,
3. and it supports burn-rate style alerting or escalation semantics for sharp regressions.

## `HUM-FRIC-1D`

### Operator projection contract

Define how the later human ring should appear in Game Loop or nearby operator views without being confused with the strict adversary score.

Acceptance criteria:

1. the operator surface keeps human-friction calibration separate from adversary breach scoring,
2. journey success and burden remain distinct,
3. and the projection names the config revision and journey under test.

# Blocked Execution Rule

Execution must remain blocked until:

1. the strict Scrapling game loop is trusted as a real board-state loop,
2. the attacker picture is strong enough that human calibration is tuning a real defended state rather than compensating for scoring confusion,
3. and the operator workflow for running human traversals is explicit and safe.

# Definition Of Done For This Planning Slice

This planning slice is complete when:

1. the repo has one canonical human-friction calibration-ring plan,
2. the later execution work is explicitly blocked rather than vaguely deferred,
3. adversary-sim contamination of human-friction evidence is forbidden in the written contract,
4. and the later ring is described as calibration over a defended state, not as a way to weaken the strict adversary loop.
