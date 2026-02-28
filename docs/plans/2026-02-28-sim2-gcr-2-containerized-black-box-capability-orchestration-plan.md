# SIM2-GCR-2 Plan: Containerized Black-Box Capability Orchestration

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-research.md`](../research/2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-research.md)

## Objective

Upgrade frontier actor execution into a capability-safe, black-box container orchestration model with enforceable runtime isolation, signed command envelopes, bounded channels, and deterministic fail-closed teardown.

## Non-goals

1. Building a generic external orchestration platform.
2. Granting frontier workers direct control-plane write capability.
3. Relaxing deterministic replay as release-blocking oracle.

## Architecture Decisions

1. Frontier workers run under a strict hardened runtime profile (rootless/non-root, `no_new_privileges`, capped capabilities, read-only filesystem, bounded resources).
2. Host issues signed, scoped capability envelopes for executable actions.
3. Command flow is one-way and bounded (host -> worker); worker output is evidence-only.
4. Deadline/heartbeat failures are fail-closed and trigger deterministic forced teardown.
5. Worker environment receives minimum scoped artifacts only; host control credentials are never injected.

## Delivery Phases

### Phase 1: Hardened Runtime Contract

1. Define runtime profile schema for frontier workers:
   1. user identity mode (non-root/rootless),
   2. privileges/capability allowlist,
   3. filesystem mode (`readonly_rootfs` + bounded writable scratch),
   4. namespace and mount restrictions,
   5. resource limits (CPU/memory/pids/runtime).
2. Add startup validation that rejects worker launch if profile invariants are violated.
3. Add telemetry labels for runtime profile compliance/violations.

Acceptance criteria:

1. Worker launch fails closed when hardening profile is not satisfied.
2. Privileged mode, sensitive host mounts, and daemon socket mounts are blocked.
3. Resource ceilings are explicit and test-covered.

### Phase 2: Signed Capability Envelopes

1. Define envelope schema and canonical serialization.
2. Sign envelopes on host with versioned key id and algorithm allowlist.
3. Validate signature, expiry, nonce, and action scope in worker before execution.
4. Reject and audit invalid/stale/replayed/out-of-scope envelopes.

Acceptance criteria:

1. Unsigned, malformed, stale, replayed, or out-of-scope envelopes are never executed.
2. Accepted envelopes are fully traceable (`run_id`, `step_id`, `key_id`).
3. Rotation-ready key versioning is built into schema and diagnostics.

### Phase 3: One-Way Bounded Command Channel

1. Implement bounded command queue from host to worker with deterministic backpressure behavior.
2. Restrict worker output path to append-only evidence/events schema.
3. Enforce policy that worker cannot call admin lifecycle/config mutation endpoints.

Acceptance criteria:

1. Command queue cannot grow unbounded under burst load.
2. Worker cannot mutate control-plane state through any direct API path.
3. Evidence path remains lineage-rich and side-effect constrained.

### Phase 4: Fail-Closed Teardown Semantics

1. Add hard run deadline, heartbeat timeout, and forced kill sequence.
2. Ensure process-tree cleanup and deterministic terminal state emission.
3. Add post-run cleanup lifecycle (artifacts/temp resources) with explicit TTL rules.

Acceptance criteria:

1. Deadline or heartbeat failure deterministically transitions run to failed terminal state.
2. No orphan worker processes remain after teardown.
3. Cleanup behavior is bounded, deterministic, and observable.

### Phase 5: Verification and Gate Wiring

1. Add tests for runtime isolation bypass attempts:
   1. privileged container flag attempt,
   2. disallowed mount attempt,
   3. daemon-socket mount attempt.
2. Add envelope negative tests:
   1. invalid signature,
   2. nonce replay,
   3. expiry,
   4. action-scope violation.
3. Add teardown tests:
   1. deadline expiry,
   2. heartbeat loss,
   3. forced kill failure path diagnostics.
4. Wire checks into canonical Makefile verification path.

Acceptance criteria:

1. Security regressions fail deterministically in CI with explicit diagnostics.
2. Frontier lane remains operational for valid workloads within budget.
3. No bypass path executes actions outside capability envelope + runtime profile constraints.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test-adversarial-fast` (with `make dev` running)
4. `make test`

## Rollback Plan

1. Keep hardened runtime and envelope enforcement behind phased rollout flag per lane until test matrix stabilizes.
2. On false positives, allow temporary narrowed relaxations with explicit audit marker and sunset date.
3. Never roll back restrictions that would expose host-control credentials or daemon socket access.

## Definition of Done

1. Frontier worker execution is capability-gated and runtime-hardened by construction.
2. Command and evidence channels preserve one-way authority boundaries.
3. Teardown is deterministic and fail-closed under timeout/crash/replay faults.
4. Verification proves isolation, replay resistance, and teardown correctness.
