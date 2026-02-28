# ADR 0009: Telemetry Lifecycle Governance for Retention, Cost, and Security/Privacy

- Status: Accepted
- Date: 2026-02-28
- Owners: Shuma core maintainers
- Related:
  - `docs/research/2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-research.md`
  - `docs/research/2026-02-28-sim2-gcr-6-monitoring-cost-efficiency-patterns-research.md`
  - `docs/research/2026-02-28-sim2-gcr-7-telemetry-artifact-security-privacy-controls-research.md`

## Context

SIM2 requires high-volume telemetry and adversary artifacts to evolve defenses rapidly. Without explicit lifecycle governance, we risk read-path retention overhead, cost/cardinality drift, and sensitive-data leakage. These concerns are coupled and must be governed as one policy system.

## Decision

Adopt unified telemetry lifecycle governance with three enforced planes:

1. **Retention determinism**
   - bucketed telemetry storage/index,
   - background purge worker with watermark,
   - operator-visible retention health and lag,
   - no opportunistic read-path keyspace cleanup.
2. **Cost governance**
   - cardinality caps with overflow buckets,
   - rollup windows for default dashboards,
   - unsampleable security-event protections,
   - payload/compression/query-budget thresholds.
3. **Security/privacy governance**
   - field-classification enforcement at persistence boundaries,
   - secret scrubbing + canary leak checks,
   - default pseudonymization in non-forensic views,
   - sensitivity-tiered artifact retention and incident hooks.

## Alternatives Considered

1. Keep ad-hoc controls in each subsystem.
2. TTL-only retention + sampling-first cost strategy.
3. Externalize lifecycle concerns to third-party data platform.

## Consequences

### Positive

- Telemetry lifecycle becomes auditable and deterministic.
- Resource usage is bounded with explicit thresholds.
- Sensitive-data exposure risk is reduced by construction.
- Operator clarity improves through explicit health/degraded states.

### Negative / Trade-offs

- Broader implementation surface area and migration complexity.
- More policy contracts to maintain and test.

## Security Impact

- Stronger protection against secret/PII leakage in logs and artifacts.
- Better incident detect/contain workflows through explicit hooks.
- Reduced over-retention risk for sensitive artifacts.

## Human Friction Impact

- Better monitoring clarity for operators.
- Additional explicit workflows for forensic break-glass and retention overrides.

## Adversary Cost Placement

- Improves defender operational tempo and confidence.
- Preserves adversary-pressure telemetry without over-collecting risky data.

## Operational Impact

- Deploy: add purge worker, rollup/cost controls, classification enforcement modules.
- Config: define retention windows, budget thresholds, pseudonymization defaults, incident hook settings.
- Monitoring/alerts: retention lag, cost pressure, leak violations, incident state.
- Rollback: per-plane rollback possible (for example disable rollup path) while preserving security-critical protections.

## Resource Impact

- Bandwidth: reduced via payload budgeting/compression.
- CPU: reduced query recomputation via rollups; modest overhead for classification/scrubbing.
- Memory: bounded buffers and cardinality controls.
- Energy/efficiency notes: lifecycle governance shifts cost from uncontrolled runtime drift to controlled background/rollup operations.

## Verification

- Tests:
  - retention bucket/purge/watermark determinism,
  - cost-threshold regressions,
  - security/privacy classification and canary leak gates.
- Benchmarks (if relevant): realtime + cost benchmark artifacts and threshold enforcement.
- Docs updated: yes (research/plans/todos/ADR set).

## Follow-ups

- Implement `SIM2-GC-15`, `SIM2-GC-16`, `SIM2-GC-17` and linked `SIM2-GC-11` regression slices.
- Reassess thresholds post-implementation with empirical production/dev benchmarks.
