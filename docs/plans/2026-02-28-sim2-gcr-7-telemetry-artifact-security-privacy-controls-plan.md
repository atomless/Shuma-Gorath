# SIM2-GCR-7 Plan: Telemetry and Artifact Security/Privacy Controls

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-7-telemetry-artifact-security-privacy-controls-research.md`](../research/2026-02-28-sim2-gcr-7-telemetry-artifact-security-privacy-controls-research.md)

## Objective

Enforce security/privacy-by-construction across telemetry and adversary artifacts without degrading operational observability.

## Non-goals

1. Replacing all telemetry schemas at once.
2. Introducing opaque black-box DLP dependencies as primary control.
3. Removing operator forensic capability entirely.

## Architecture Decisions

1. Field-classification policy is enforced before persistence.
2. Secret-scrubbing and canary-leak detection are mandatory for artifact paths.
3. Pseudonymization defaults apply to non-forensic telemetry views.
4. Retention tiers are sensitivity-driven with strict default ceilings.
5. Security policy violations emit incident-response events and operator workflows.

## Delivery Phases

### Phase 1: Field Classification Contract

1. Define telemetry/artifact field classes and allowed persistence contexts.
2. Add schema-level validation for prohibited/secret classes.
3. Add explicit reason taxonomy for classification failures.

Acceptance criteria:

1. Every persisted telemetry/artifact field is class-tagged.
2. Prohibited classes cannot be persisted.
3. Classification failures are observable and test-covered.

### Phase 2: Secret Scrubbing and Canary Detection

1. Add scrubber for high-risk secret patterns in event/artifact payloads.
2. Add canary markers and fail-closed validation in frontier artifact pipeline.
3. Emit structured security events for scrub/drop actions.

Acceptance criteria:

1. Secret canary leakage to persisted artifacts is zero in tests.
2. Scrubber actions preserve operational diagnostics without raw secret exposure.
3. Violation telemetry is actionable and correlated by run/operation id.

### Phase 3: Pseudonymization and Access Modes

1. Expand pseudonymization controls beyond fingerprint state to monitoring/event surfaces where feasible.
2. Provide explicit forensic access mode with operator acknowledgement and audit trail.
3. Ensure default UI/API views remain pseudonymized.

Acceptance criteria:

1. Sensitive identifiers are pseudonymized by default in non-forensic views.
2. Forensic raw access is explicit, auditable, and bounded.
3. Mode transitions are reflected in operator-visible state.

### Phase 4: Retention Tiers and Incident Hooks

1. Define retention tiers by sensitivity (raw high-risk, redacted summary, operational metrics).
2. Enforce default high-risk artifact retention ceiling (`<=72h`).
3. Wire leak/policy-violation hooks into incident workflow events.

Acceptance criteria:

1. Retention tiers are enforced and visible in status/monitoring payloads.
2. High-risk artifact retention exceeds default ceiling only via explicit override with audit entry.
3. Incident hooks trigger deterministic containment workflow outputs.

### Phase 5: Verification and CI Enforcement

1. Add regression suite for classification, scrubbing, canary detection, pseudonymization, and retention tiers.
2. Add CI diagnostics artifacts for policy-violation evidence.
3. Wire checks into canonical Makefile test surface.

Acceptance criteria:

1. Security/privacy regressions fail deterministically in CI.
2. CI artifacts identify exact policy class violated.
3. Controls remain compatible with realtime monitoring and adversary workflows.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test`

## Rollback Plan

1. If pseudonymization migration causes operational breakage, keep classification/scrubbing active and roll back only view-level transformation.
2. If canary detection is noisy, tune signatures but keep fail-closed secret-prohibited classes enforced.
3. Preserve incident event emission in all rollback modes.

## Definition of Done

1. Telemetry and artifact persistence paths are classification-enforced.
2. Secret leakage prevention is fail-closed and test-backed.
3. Pseudonymization and retention tiers are default-on and operator-visible.
4. Incident hooks provide deterministic containment and auditability.
