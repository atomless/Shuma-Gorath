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
   - `src/runtime/effect_intents/`
   - Explicit plans for metrics, monitoring, event logs, bans, and response rendering.
4. Capability-gated privileged side effects
   - `src/runtime/capabilities.rs`
   - Request-path effect execution requires explicit capability tokens minted at the orchestration trust boundary.

`src/lib.rs` remains a thin trust-boundary shell, delegating policy routing to `src/runtime/request_flow.rs` (`facts -> decisions -> effects -> response`) while preserving pipeline order and endpoint semantics (including `/pow` ordering).

## Trust-boundary model

Privileged writes (ban store, metrics, monitoring, event log) are executed only through the effect executor with typed capability tokens. Tokens are minted in request orchestration and are not implicitly available through raw request/context convention.

## Migration sequence

1. Publish architecture contract (this ADR).
2. Add characterization tests for representative policy outcomes.
3. Introduce `RequestFacts` + `PolicyDecisionGraph` pure stages.
4. Route request handling through `EffectIntent` executor and capability tokens.
5. Remove legacy rollback seams after parity and architecture-guard coverage are in place.

## Rollback strategy

If regressions appear, rollback is handled by reverting staged orchestration commits while preserving the module contracts (`request_flow`, `request_facts`, `policy_graph`, `effect_intents`, `capabilities`) and re-running characterization parity tests before re-apply.

## Consequences

### Positive

- Decision logic is testable as pure functions.
- Side effects are centralized and auditable.
- Trust-sensitive writes are gated by explicit capabilities.
- Request orchestration complexity in `src/lib.rs` is reduced.

### Costs / trade-offs

- Slightly larger runtime module surface area.
- More explicit capability and intent plumbing in orchestration code.

### Security and operations impact

- Improves trust-boundary integrity by construction.
- Improves incident triage due to explicit, typed effect plans.
- Keeps existing policy taxonomy and monitoring semantics stable.
- Requires disciplined module-boundary updates whenever orchestration ownership shifts.
