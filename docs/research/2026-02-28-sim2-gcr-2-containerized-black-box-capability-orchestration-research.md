# SIM2-GCR-2 Research: Containerized Black-Box Capability Orchestration

Date: 2026-02-28  
Status: Recommended architecture selected

## Objective

Select the safest and most operationally reliable orchestration model for containerized frontier adversary actors, with explicit controls for least-authority token handoff, signed command envelopes, bounded execution, one-way command channels, and fail-closed teardown.

## Repository Baseline (Current State)

1. SIM2 already has a frontier/adversary lane and black-box posture requirements in TODO/plan contracts.
2. Existing backlog includes action-grammar validation, egress allowlisting, and lineage (`SIM2-GC-8-*`) but does not yet codify full runtime hardening and teardown semantics at the container boundary.
3. Existing backlog does not yet explicitly require signed host<->worker envelopes with replay TTL semantics.
4. Existing backlog does not yet explicitly require bounded command-channel backpressure and deterministic process-tree teardown guarantees.

## Primary-Source Findings

1. Container runtime should operate with least privilege and least access; image/runtime integrity validation is explicitly recommended.
   Source: [NIST SP 800-190](https://doi.org/10.6028/NIST.SP.800-190)
2. Running mixed-sensitivity workloads on the same host kernel increases risk; segmentation by purpose/sensitivity is recommended.
   Source: [NIST SP 800-190](https://doi.org/10.6028/NIST.SP.800-190)
3. Insecure runtime configuration includes privileged mode and mounting sensitive host directories; both materially increase breakout impact.
   Source: [NIST SP 800-190](https://doi.org/10.6028/NIST.SP.800-190)
4. OCI runtime spec provides enforceable process/container hardening primitives: read-only rootfs, capability sets, `noNewPrivileges`, and `rlimits`.
   Source: [OCI Runtime Spec config.md](https://raw.githubusercontent.com/opencontainers/runtime-spec/main/config.md)
5. Docker confirms `no-new-privileges`, seccomp profiles, dropped capabilities, rootless operation, and read-only filesystem controls as first-class runtime controls.
   Sources:
   - [Docker `docker run` security options](https://docs.docker.com/reference/cli/docker/container/run)
   - [Docker seccomp docs](https://docs.docker.com/engine/security/seccomp/)
   - [Docker rootless mode](https://docs.docker.com/engine/security/rootless/)
   - [Docker read-only root filesystem behavior](https://docs.docker.com/reference/cli/docker/container/run/)
6. Docker explicitly warns that bind-mounting the Docker socket gives full host-daemon control; this is incompatible with black-box least authority.
   Source: [Docker `docker run` reference](https://docs.docker.com/reference/cli/docker/container/run/)
7. Resource constraints are mandatory to avoid host destabilization: CPU/memory bounds and OOM behavior must be explicit.
   Source: [Docker resource constraints](https://docs.docker.com/engine/containers/resource_constraints/)
8. Deterministic termination contracts are well-established in controller patterns: deadline expiry should terminate workers and mark failed.
   Source: [Kubernetes Jobs (`activeDeadlineSeconds`, `backoffLimit`)](https://v1-32.docs.kubernetes.io/docs/concepts/workloads/controllers/job/)
9. Post-run cleanup should be automated with explicit TTL semantics to avoid resource drift.
   Source: [Kubernetes TTL-after-finished](https://kubernetes.io/docs/concepts/workloads/controllers/job/)
10. Signed envelope protocols should use standard integrity formats and algorithm-agility constraints.
    Sources:
    - [RFC 7515 (JWS)](https://datatracker.ietf.org/doc/html/rfc7515)
    - [RFC 8725 (JWT BCP)](https://datatracker.ietf.org/doc/html/rfc8725)
11. Unbounded async channels risk memory exhaustion; bounded channels provide explicit backpressure.
    Sources:
    - [Tokio channels tutorial](https://tokio.rs/tokio/tutorial/channels)
    - [Tokio `unbounded_channel` docs](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.unbounded_channel.html)
12. Child-process lifecycle handling must explicitly kill/wait to avoid orphan/zombie ambiguity.
    Source: [Tokio `process::Child` docs](https://docs.rs/tokio/latest/tokio/process/struct.Child.html)

## Inferences for Shuma (Derived from Sources)

The following are direct design inferences from the sources above:

1. **One-way command channel** is required so the worker cannot invoke control-plane mutations (inference from least-privilege and daemon/socket risk guidance).
2. **Signed host-issued capability envelopes** should bind action scope, run id, expiry, and nonce to each executable step (inference from JWS integrity guidance + replay-resistance posture).
3. **Fail-closed teardown** must treat timeout, heartbeat loss, or envelope-verification failure as terminal run failure with forced kill and cleanup (inference from deadline-based controller semantics).

## Architecture Options

### Option A: Minimal sandbox + free-form worker output

Container gets broad runtime defaults and action text parsed permissively by host.

### Option B: Hardened runtime only (no signed capability envelopes)

Apply rootless/seccomp/cap limits/resource limits, but keep command handoff unsigned and loosely typed.

### Option C: Hardened runtime + signed capability envelopes + bounded one-way channel + fail-closed teardown (Recommended)

Host issues signed, short-lived capability envelopes to worker over bounded command channel; worker can only execute allowed grammar and emit append-only evidence; controller enforces hard deadline/kill semantics.

### Option D: External sandbox/orchestration platform dependency

Delegate all worker isolation and orchestration guarantees to external workload platform.

## Decision Matrix

| Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| A. Minimal sandbox | Lowest implementation effort | High breakout/replay/drift risk; weak trust boundary | Low initial, high incident risk | Weakest | Low |
| B. Hardened runtime only | Significant containment improvements | Unsigned/unscoped command surface remains exploitable | Medium | Moderate-strong | Low-medium |
| C. Hardened runtime + signed envelopes + bounded one-way channel (recommended) | End-to-end least-authority control, replay resistance, deterministic failure semantics, auditable lineage | Requires envelope schema, channel control, teardown tests | Medium | Strongest practical in current architecture | Medium |
| D. External sandbox platform | Potentially strongest isolation guarantees | High ops complexity, dependency lock-in, reduced local dev ergonomics | High | Strong | High |

## Recommendation

Adopt **Option C** and codify a capability-by-construction frontier execution contract.

Required contract controls:

1. **Runtime hardening baseline**
   1. Non-root/rootless execution.
   2. `no_new_privileges` enabled.
   3. Drop all capabilities by default; allowlist only explicit minimum set.
   4. Read-only root filesystem with explicit writable scratch mounts only.
   5. No privileged mode, no host PID/IPC/NET namespace joins, no sensitive host mounts, and no daemon socket mount.
2. **Signed capability envelope**
   1. Host signs each executable command envelope.
   2. Envelope fields include `run_id`, `step_id`, `allowed_action`, `target_scope`, `issued_at`, `expires_at`, `nonce`, `key_id`.
   3. Worker rejects invalid signature, stale expiry, nonce replay, or out-of-scope action.
3. **One-way bounded command channel**
   1. Host -> worker command path is bounded (backpressure-aware).
   2. Worker -> host path is append-only evidence/events, not control mutation.
   3. Worker cannot call admin control endpoints or mutate lifecycle state directly.
4. **Bounded execution and fail-closed teardown**
   1. Enforce hard CPU/memory/runtime/concurrency budgets.
   2. Use hard run deadline and explicit forced kill semantics.
   3. On teardown failure or heartbeat loss, mark run failed and emit terminal diagnostics.
5. **Secret and token handoff minimization**
   1. Pass only scoped short-lived capability artifacts required for current run.
   2. Never expose host control credentials, admin session state, or signing roots in worker environment.

## Security and Ops Implications

1. Strongly reduces blast radius from compromised frontier worker behavior.
2. Converts policy-by-convention to enforceable capability constraints at trust boundary.
3. Improves operational determinism under timeout/stall/crash conditions.
4. Adds moderate implementation complexity but lowers incident and debugging cost.

## Plan and TODO Impact

1. New plan doc: `docs/plans/2026-02-28-sim2-gcr-2-containerized-black-box-capability-orchestration-plan.md`.
2. Expand `SIM2-GC-8` with explicit tasks for runtime hardening profile, signed envelope semantics, bounded one-way channels, and fail-closed teardown.
3. Expand `SIM2-GC-11` with negative-path tests for isolation bypass, envelope replay/signature failure, and teardown determinism.
