# SIM2 Architecture Shortfall Research: Orchestration Centralization and Capability-by-Construction

Date: 2026-02-27  
Status: Completed

## Shortfall Statement

SIM2 delivered broad capability, but core orchestration remains concentrated in imperative request-path flows with trust-boundary guarantees frequently enforced by policy checks rather than type/capability construction. This creates regression risk as behavior grows.

## Current-State Evidence

1. Core request orchestration remains large and heavily conditional.  
   Evidence: `src/lib.rs` (~1194 lines), with route/policy sequencing in `src/lib.rs:960-1160`.
2. Policy pipeline itself is large and imperative, mixing decision sequencing with side-effect behavior.  
   Evidence: `src/runtime/policy_pipeline.rs` (~1293 lines).
3. Provider contracts exist, but important trust-boundary semantics are still header/config conventions rather than capability tokens/types.  
   Evidence: `src/providers/contracts.rs`, `src/runtime/sim_telemetry.rs:40-53`, `scripts/tests/adversarial_simulation_runner.py:1517-1535`.

## Research Findings

1. Hexagonal architecture emphasizes keeping core logic independent of external mechanisms and interface details.  
   Source: Alistair Cockburn, Hexagonal architecture  
   <https://alistair.cockburn.us/hexagonal-architecture>
2. Zero-trust architecture formalizes policy decision point and policy enforcement point separation to reduce implicit trust coupling.  
   Source: NIST SP 800-207  
   <https://csrc.nist.gov/pubs/sp/800/207/final>
3. Capability models reduce ambient authority and make privilege explicit in the object/function boundary.  
   Source: Capsicum practical capabilities  
   <https://www.usenix.org/conference/usenixsecurity10/capsicum-practical-capabilities-unix>
4. Policy-as-code supports decoupled decision logic with testable declarative contracts.  
   Source: OPA policy language and testing model  
   <https://www.openpolicyagent.org/docs/policy-language>

## Addressing Options

1. Incremental cleanup only: continue modular refactors without new orchestration model.
2. Introduce functional-core/imperative-shell architecture for request decisions, with explicit typed capabilities for privileged actions.
3. Large rewrite into a new policy engine runtime.

## Recommended Direction

Adopt option 2 in staged migration slices.

Target architecture pattern:

1. `RequestFacts` extraction phase:
   - pure data normalization from request + config + provider availability.
2. `PolicyProgram` decision phase:
   - pure functions produce deterministic decision graph and effect intents.
3. `EffectExecutor` phase:
   - side effects (KV writes, bans, metrics, events) executed from explicit intents.
4. Capability-typed boundary:
   - privileged operations require capability tokens/objects that cannot be created in attacker request paths.

## Proposed Migration Principles

1. Preserve behavior with characterization tests before each extraction.
2. Move one policy stage at a time into pure decision modules.
3. Enforce capability possession at compile-time where possible (Rust typed wrappers) and at runtime otherwise.
4. Keep provider boundary usage explicit and centralized to avoid bypass.

## Success Signals

1. Request-path decision logic is mostly pure/data-driven and testable without side effects.
2. Privileged actions become impossible without explicit capability objects.
3. Orchestration sequencing changes are validated through deterministic policy-graph tests.

## Source Links

1. Hexagonal architecture: <https://alistair.cockburn.us/hexagonal-architecture>
2. NIST SP 800-207 (Zero Trust): <https://csrc.nist.gov/pubs/sp/800/207/final>
3. Capsicum capabilities: <https://www.usenix.org/conference/usenixsecurity10/capsicum-practical-capabilities-unix>
4. OPA policy language: <https://www.openpolicyagent.org/docs/policy-language>
