# SIM2 Plan 4: Simulation Telemetry Authenticity Hardening

Date: 2026-02-27  
Status: Proposed

Reference research:

- [`docs/research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md`](../research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md)

## Objective

Ensure simulation data-plane tagging in runtime-dev is capability-authenticated, not header-by-convention.

## Non-goals

1. Enabling simulation tagging in production.
2. Replacing current monitoring/event storage architecture.

## Architecture Decisions

1. Add signed simulation-metadata contract using HMAC and short freshness window.
2. Keep existing run/profile/lane headers, but require signature and timestamp/nonce headers.
3. Ignore invalid signatures and treat requests as non-simulation traffic.
4. Emit explicit observability for invalid simulation tag attempts.

## Delivery Phases

### Phase 1: Signed Metadata Contract

1. Define `sim-tag.v1` signature schema (canonical string format, HMAC algorithm, required headers).
2. Add env-only signing secret (`SHUMA_SIM_TELEMETRY_SECRET`) with dev/test bootstrap guidance.

Acceptance criteria:

1. Contract is documented and validated in unit tests.
2. Missing signing secret in dev/test produces explicit operator warning behavior.

### Phase 2: Runtime Verification in `sim_telemetry`

1. Verify signature, timestamp skew, and nonce replay window before activating simulation context.
2. Preserve current `runtime-dev` and `adversary_sim_available` guards.

Acceptance criteria:

1. Unsigned/invalid/stale requests never activate simulation context.
2. Valid signed requests preserve current simulation partition behavior.

### Phase 3: Runner/Container Signer Integration

1. Update deterministic and container adversary workers to emit signed metadata.
2. Keep attacker/control lane semantics unchanged.

Acceptance criteria:

1. Existing adversarial profiles still run end-to-end under signed metadata.
2. No additional privileged headers become available to attacker lane.

### Phase 4: Observability and Docs

1. Add counters/events for invalid simulation tags.
2. Update operator docs for troubleshooting signature failures.

Acceptance criteria:

1. Simulation-tag failures are diagnosable from monitoring and event stream.
2. Documentation includes key-rotation and local setup path.

## Verification Strategy

1. `make test-unit`
2. `make test-integration`
3. `make test-adversarial-fast`
4. `make test-adversarial-coverage`
5. `make test` (with `make dev` running)

## Operational and Security Notes

1. Prevents simulation data-plane spoofing in runtime-dev.
2. Preserves production fail-closed behavior.

## Definition of Done

1. Simulation tagging requires valid capability signature.
2. Unsigned headers cannot alter telemetry partitioning.
3. Adversarial profiles remain green with signed metadata enabled.
