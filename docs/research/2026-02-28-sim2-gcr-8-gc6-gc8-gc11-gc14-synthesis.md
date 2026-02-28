# SIM2-GCR-8 Research Synthesis: GC-6, GC-8, GC-11, GC-14

Date: 2026-02-28  
Status: Quantitative implementation synthesis finalized

## Objective

Synthesize prior SIM2 GCR findings into implementation-ready contracts for:

1. `SIM2-GC-6` realtime monitoring delivery,
2. `SIM2-GC-8` frontier actor execution,
3. `SIM2-GC-11` verification and gate enforcement,
4. `SIM2-GC-14` hybrid deterministic/emergent governance.

## Source Basis

This synthesis consolidates:

1. `SIM2-GCR-1`/`3`/`2` trust-boundary and orchestration findings.
2. `SIM2-GCR-4` architecture-candidate analysis.
3. `SIM2-GCR-9` benchmark evidence (`cursor polling` vs `SSE`).
4. `SIM2-GCR-5` retention lifecycle findings.
5. `SIM2-GCR-6` cost-governance findings.
6. `SIM2-GCR-7` telemetry/artifact security/privacy findings.
7. ADR set `0007`, `0008`, `0009`.

## Consolidated Quantitative Contract

| Domain | Quantitative Thresholds |
|---|---|
| `GC-6` realtime freshness | Under envelope (`>=1000 events/s`, `>=5 active clients`): `p95 <= 300ms`, `p99 <= 500ms`; non-degraded overflow/drop `==0`; streaming-enabled query budget `<=1 req/sec/client` average |
| `GC-8` frontier actor controls | Executed-action lineage completeness `==100%`; policy-violation execution `==0`; kill-switch stop latency `p95 <= 10s`; degraded outage state surfaced within one refresh/stream cycle |
| `GC-11` verification | CI must fail on threshold breaches (`freshness`, `overflow/drop`, `query budget`, `lineage completeness`, `secret canary`, `ADR drift`) with scenario-specific diagnostics |
| `GC-14` hybrid governance | Promotion requires deterministic confirmation rate `>=95%`; false-discovery rate `<=20%`; owner review SLA `<=48h` from emergent candidate triage to disposition |

## Decision Matrix (Implementation Sequencing)

| Sequence Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| Parallel all four domains | Fast calendar throughput | High integration collision risk | High | Moderate (harder to isolate regressions) | High |
| Realtime first, governance last | Immediate freshness gains | Promotion/verification drift risk if governance lags | Medium | Moderate | Medium |
| Verification-first then implementation | Strong guardrails upfront | Delays user-visible improvement | Medium | Strong | Medium |
| Layered dependency sequence (recommended): `GC-6 -> GC-8 -> GC-11 hardening -> GC-14 governance` | Balances user-visible progress with test/governance lock-in | Requires disciplined slice boundaries | Medium | Strong | Medium-low |

## Recommendation

Use layered dependency sequence:

1. Deliver `GC-6` realtime foundations and thresholds.
2. Deliver `GC-8` frontier execution constraints and lineage guarantees.
3. Harden `GC-11` matrix and threshold gates across new behaviors.
4. Finalize `GC-14` promotion/governance thresholds and runbook semantics.

## Synthesis Outcomes Applied to TODOs

1. `GC-6` now contains explicit freshness and query-budget thresholds.
2. `GC-8` and `GC-14` are upgraded with explicit numeric acceptance targets.
3. `GC-11` is expanded with threshold-specific regression coverage and ADR-conformance checks.
4. Lifecycle support sections (`GC-15`/`GC-16`/`GC-17`) provide retention/cost/security foundations that prevent realtime work from regressing into uncontrolled operational risk.

## Plan and Execution Hand-off

1. Implementation plan published at `docs/plans/2026-02-28-sim2-gcr-8-gc6-gc8-gc11-gc14-implementation-plan.md`.
2. Remaining research-track checklist can be closed once TODO deltas are committed.
3. High-risk implementation slices should explicitly cite ADR alignment (`0007`/`0008`/`0009`).
