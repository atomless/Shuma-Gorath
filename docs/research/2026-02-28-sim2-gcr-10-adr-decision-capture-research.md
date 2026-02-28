# SIM2-GCR-10 Research: ADR-Backed Decision Capture for SIM2 Gap Closure

Date: 2026-02-28  
Status: ADR decision set accepted

## Objective

Codify selected SIM2 gap-closure architecture decisions into ADR artifacts for:

1. UI-toggle-driven black-box adversary orchestration,
2. realtime monitoring delivery architecture,
3. telemetry retention/cost/security lifecycle governance.

## Inputs

Research tracks consolidated:

1. `SIM2-GCR-1` + `SIM2-GCR-3` + `SIM2-GCR-2` (control-plane trust boundaries and capability-safe orchestration)
2. `SIM2-GCR-4` + `SIM2-GCR-9` (realtime delivery architecture + benchmark evidence)
3. `SIM2-GCR-5` + `SIM2-GCR-6` + `SIM2-GCR-7` (retention, cost, and security/privacy lifecycle controls)

## ADR Capture Options

### Option A: No new ADRs (keep decisions in research/todo only)

### Option B: Single monolithic ADR for all SIM2 gap-closure topics

### Option C: Three scoped ADRs aligned to decision domains (Recommended)

## Decision Matrix

| Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| A. No new ADRs | No additional docs overhead | High drift risk; weak enforceability; decision ambiguity | Low | Weak | Low |
| B. One monolithic ADR | Single reference file | Hard to maintain, poor ownership boundaries, noisy supersession path | Medium | Moderate | Medium-high |
| C. Three scoped ADRs (recommended) | Clear ownership and lifecycle per domain; easier drift detection; explicit trust-boundary lock-in | Slightly more docs surface | Medium | Strong | Medium |

## Selected ADR Set

1. [`docs/adr/0007-adversary-sim-toggle-command-controller.md`](../adr/0007-adversary-sim-toggle-command-controller.md)
   - Locks command-controller semantics, trust-boundary controls, and capability-safe worker orchestration constraints.
2. [`docs/adr/0008-realtime-monitoring-cursor-sse-hybrid.md`](../adr/0008-realtime-monitoring-cursor-sse-hybrid.md)
   - Locks cursor-delta + optional SSE hybrid model and benchmark-derived thresholds.
3. [`docs/adr/0009-telemetry-lifecycle-retention-cost-security.md`](../adr/0009-telemetry-lifecycle-retention-cost-security.md)
   - Locks unified lifecycle governance for retention determinism, cost controls, and security/privacy protections.

## Why This Set

1. Matches SIM2 shortfall boundaries directly and avoids cross-domain ambiguity.
2. Preserves modular evolution (one ADR can be superseded without destabilizing unrelated domains).
3. Creates explicit trust-boundary and SLO contracts before high-risk implementation slices.

## Plan and TODO Impact

1. New plan doc: `docs/plans/2026-02-28-sim2-gcr-10-adr-decision-capture-plan.md`.
2. Update SIM2 GC implementation sections with explicit ADR references for conformance checks.
3. Add verification item requiring ADR-conformance checks in `SIM2-GC-11` CI diagnostics.
