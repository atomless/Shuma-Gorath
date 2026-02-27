# SIM2 Plan 3: Traffic-Model Execution Realism

Date: 2026-02-27  
Status: Proposed

Reference research:

- [`docs/research/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md`](../research/2026-02-27-sim2-shortfall-3-traffic-model-execution-realism.md)

## Objective

Make manifest `traffic_model` fields execution-effective so realism gates measure actual behavior rather than metadata declarations.

## Non-goals

1. Full migration to external load-testing frameworks.
2. Unbounded concurrency or non-deterministic traffic schedules.

## Architecture Decisions

1. Introduce deterministic traffic execution layer used by all scenario drivers.
2. Implement `traffic_model` semantics for think-time, retry strategy, and cookie behavior.
3. Preserve deterministic outcomes by seeding timing/retry logic from scenario seed.
4. Extend reports with runtime evidence proving model application.

## Delivery Phases

### Phase 1: Deterministic Execution Policy Layer

1. Add execution policy object derived from scenario `traffic_model`.
2. Wrap driver request dispatches with shared policy hooks.

Acceptance criteria:

1. Every driver invocation passes through policy layer.
2. Policy layer can run in strict deterministic mode.

### Phase 2: Think-Time and Retry Semantics

1. Implement bounded deterministic think-time scheduling.
2. Implement retry modes:
   - `single_attempt`
   - `bounded_backoff`
   - `retry_storm` (bounded and explicit)

Acceptance criteria:

1. Retry/sleep behavior is visible in report evidence.
2. Runtime remains within profile max-runtime guardrails.

### Phase 3: Cookie Behavior Semantics

1. Implement stateful cookie jar mode.
2. Implement stateless and reset-per-request modes.

Acceptance criteria:

1. Cookie modes measurably affect request behavior where scenarios depend on session continuity.
2. Cookie behavior is deterministic and test-covered.

### Phase 4: Persona/Cohort Realism Evidence

1. Add per-persona execution metrics (effective wait ranges, retry counts, cookie mode usage).
2. Extend coverage/quantitative checks to assert key realism envelopes.

Acceptance criteria:

1. `full_coverage` report proves persona-specific runtime behavior.
2. Realism regressions fail with actionable diagnostics.

## Verification Strategy

1. `make test-adversarial-manifest`
2. `make test-adversarial-fast`
3. `make test-adversarial-coverage`
4. `make test` (with `make dev` running)

## Operational and Security Notes

1. Runtime remains bounded and deterministic by profile budgets.
2. Improves confidence that adversarial gates represent realistic traffic behavior.

## Definition of Done

1. `traffic_model` controls directly affect runtime behavior.
2. Reports include concrete runtime evidence for pacing/retry/cookie semantics.
3. Deterministic reproducibility remains intact.
