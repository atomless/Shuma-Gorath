# ADR 0006: Functional-Core Policy Orchestration with Capability-Gated Effects

- Status: Accepted
- Date: 2026-02-27

## Context

SIM2 delivered broad capability and stronger adversarial realism, but request-path orchestration remained relatively centralized and imperative. Core decision logic, side effects, and trust-boundary-sensitive operations were mixed together, increasing regression and drift risk.

The architectural shortfall to address:

> Core orchestration remained centralized/imperative, and key trust-boundary controls were still policy-by-convention rather than capability-by-construction.

## Decision

Adopt a functional-core / imperative-shell request policy model for main request handling:

1. Functional `RequestFacts` normalization layer
   - `src/runtime/request_facts.rs`
   - Side-effect-free projection of request + precomputed runtime inputs into typed facts.
2. Functional `PolicyDecisionGraph` layer
   - `src/runtime/policy_graph.rs`
   - Ordered pure stages with typed decisions:
     - First tranche: `ip_range`, `honeypot`, `rate_limit`, `existing_ban`
     - Second tranche: `geo`, `botness`, `js/challenge`
3. Effect-intent executor layer
   - `src/runtime/effect_intents.rs`
   - Explicit plans for metrics, monitoring, event logs, bans, and response rendering.
4. Capability-gated privileged side effects
   - `src/runtime/capabilities.rs`
   - Request-path effect execution requires explicit capability tokens minted at the orchestration trust boundary.

`src/lib.rs` remains orchestration shell, delegating policy routing to runtime orchestration (`facts -> decisions -> effects -> response`) while preserving pipeline order and endpoint semantics (including `/pow` ordering).

## Trust-boundary model

Privileged writes (ban store, metrics, monitoring, event log) are executed only through the effect executor with typed capability tokens. Tokens are minted in request orchestration and are not implicitly available through raw request/context convention.

## Migration sequence

1. Publish architecture contract (this ADR).
2. Add characterization tests for representative policy outcomes.
3. Introduce `RequestFacts` + `PolicyDecisionGraph` pure stages.
4. Route request handling through `EffectIntent` executor and capability tokens.
5. Preserve legacy handlers as retained rollback seam during stabilization.

## Rollback strategy

If regressions appear, rollback can re-enable legacy policy handler functions in `src/runtime/policy_pipeline.rs` for affected stages while keeping new modules intact as non-disruptive scaffolding.

## Consequences

### Positive

- Decision logic is testable as pure functions.
- Side effects are centralized and auditable.
- Trust-sensitive writes are gated by explicit capabilities.
- Request orchestration complexity in `src/lib.rs` is reduced.

### Costs / trade-offs

- Temporary duplication while legacy handlers remain as rollback seam.
- Slightly larger runtime module surface area.

### Security and operations impact

- Improves trust-boundary integrity by construction.
- Improves incident triage due to explicit, typed effect plans.
- Keeps existing policy taxonomy and monitoring semantics stable.
