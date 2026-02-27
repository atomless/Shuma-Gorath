# SIM2 Architecture Uplift Plan: Functional Orchestration and Capability-by-Construction

Date: 2026-02-27  
Status: Proposed

Reference research:

- [`docs/research/2026-02-27-sim2-architecture-shortfall-orchestration-capability.md`](../research/2026-02-27-sim2-architecture-shortfall-orchestration-capability.md)
- [`docs/module-boundaries.md`](../module-boundaries.md)

## Objective

Address the identified architectural shortfall:

> SIM2 delivered substantial capability, but core orchestration remains fairly centralized and imperative, with key trust-boundary controls still policy-by-convention rather than capability-by-construction.

The target is a staged migration to a functional-core orchestration model with explicit capability boundaries.

## Non-goals

1. Big-bang rewrite of request path.
2. Replacing provider registry model.
3. Behavior changes without characterization parity tests.

## Architecture Decisions

1. Introduce a pure decision core:
   - `RequestFacts` (normalized inputs)
   - `PolicyDecisionGraph` (ordered pure decision steps)
   - `EffectIntents` (side-effect declarations)
2. Keep imperative shell minimal:
   - gathers runtime inputs,
   - executes effect intents,
   - renders final response.
3. Enforce privileged operations via explicit capability objects:
   - trust-boundary material cannot be implicitly available from raw request headers.
4. Migrate stage-by-stage with characterization tests.

## Delivery Phases

### Phase 1: Architecture Contract and ADR

1. Publish ADR defining:
   - functional-core boundaries,
   - capability object model,
   - migration sequencing and rollback strategy.
2. Define file/module map for new orchestration core.

Acceptance criteria:

1. ADR accepted with explicit trust-boundary and policy-order guarantees.
2. Module map aligns with `docs/module-boundaries.md` dependency direction.

### Phase 2: RequestFacts and Characterization Harness

1. Extract request/config/provider signal normalization into pure `RequestFacts` builders.
2. Add characterization tests that snapshot current decisions for representative request matrix.

Acceptance criteria:

1. `RequestFacts` generation is side-effect free and unit-tested.
2. Characterization suite can detect behavioral drift before/after refactors.

### Phase 3: PolicyDecisionGraph Extraction

1. Move selected pipeline stages into pure decision functions returning typed outcomes.
2. Keep side effects out of decision functions.

Acceptance criteria:

1. Stage decisions are testable without KV/network dependencies.
2. Decision ordering remains explicit and deterministic.

### Phase 4: EffectIntent Executor

1. Introduce effect-intent execution layer for bans, metrics, monitoring, event logs, and provider invocations.
2. Ensure each decision stage emits intent payloads rather than writing side effects directly.

Acceptance criteria:

1. Side effects are centralized and auditable.
2. Existing observability/event semantics remain behavior-compatible.

### Phase 5: Capability-by-Construction Trust Boundaries

1. Replace convention-only trust checks with typed capabilities for privileged operations.
2. Ensure orchestration paths cannot execute privileged effects without capability possession.

Acceptance criteria:

1. Compile-time or explicit runtime capability checks gate privileged operations.
2. Trust-boundary regressions are covered by dedicated tests.

### Phase 6: Progressive Migration Completion

1. Continue stage extraction until `src/lib.rs` is thin orchestration shell.
2. Keep compatibility with current provider contracts and make-target workflows.

Acceptance criteria:

1. `src/lib.rs` primarily wires facts -> decisions -> effects.
2. `make test` remains green throughout staged migration.

## Verification Strategy

1. `make test-unit`
2. `make test-integration`
3. `make test-dashboard-e2e`
4. `make test` (with `make dev` running)
5. `make build`

## Operational and Security Notes

1. Security improves as privileged operations become explicit capabilities.
2. Maintenance improves through pure-stage unit testing and reduced orchestration coupling.
3. Rollback remains feasible because migration is incremental and stage-scoped.

## Definition of Done

1. Core decision pipeline is predominantly functional and side-effect free.
2. Privileged trust-boundary operations require explicit capabilities.
3. Orchestration complexity is reduced without behavior regressions.
