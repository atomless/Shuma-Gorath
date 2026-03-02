# SIM Prime Directive: Shared Deterministic Corpus + Out-of-Process Heartbeat

Date: 2026-03-02  
Status: Active implementation directive (supersedes ambiguous SIM execution assumptions)

Related:
- [`2026-03-02-sim-runtime-architecture-overview-and-gap-report.md`](./2026-03-02-sim-runtime-architecture-overview-and-gap-report.md)
- [`../adr/0010-adversary-sim-autonomous-heartbeat.md`](../adr/0010-adversary-sim-autonomous-heartbeat.md)
- [`../plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](../plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)

## Prime Directive

Build one coherent deterministic adversary program with:

1. One shared deterministic attack corpus used by both runtime and CI oracle lanes.
2. Two separate executors with separate responsibilities:
   - runtime executor (operator-facing generation),
   - CI oracle executor (scenario setup/gates/evidence).
3. One runtime heartbeat owner that is out-of-process and not request-lifecycle-driven.
4. Strict off-state inertness and ephemeral toggle semantics.

## Decision Summary

### 1) Keep the CI Python oracle as a separate system

The Python runner is a verification/oracle harness, not runtime generation control.  
It keeps setup/teardown, profile gates, realism checks, repeatability checks, and reporting artifacts.

Reference:
- [`../../scripts/tests/adversarial_simulation_runner.py`](../../scripts/tests/adversarial_simulation_runner.py)
- [`../../scripts/tests/adversarial_repeatability.py`](../../scripts/tests/adversarial_repeatability.py)

### 2) Converge deterministic attack content, not executors

Current duplication is primarily at the attack-stimulus definition layer (overlapping defense probes implemented twice).  
Convergence target is a canonical deterministic attack corpus with lane-specific profiles:

1. Runtime profile: attacker-plane only, bounded, operator-safe.
2. CI oracle profile: same attacker actions plus control-plane setup/gating/report logic.

### 3) Move runtime generation heartbeat to out-of-process ownership

Runtime generation must not depend on request throughput or dashboard behavior.  
Generation ownership moves to a host-side supervisor process that calls a dedicated internal beat endpoint on cadence.

### 4) Use Rust for the supervisor worker

Preferred supervisor implementation is a Rust binary:

1. low overhead and no extra runtime dependency,
2. deployment-friendly single artifact,
3. aligns with project performance and operational goals.

Python remains appropriate for CI oracle orchestration and contract/report tooling.

## Why This Split Is Correct

1. Runtime lane and CI oracle lane have different purposes and different invariants.
2. A shared corpus eliminates drift while preserving each lane's strengths.
3. Out-of-process heartbeat removes coupling to request loops and UI lifecycle.
4. Operators get reliable runtime behavior; CI keeps strong regression and evidence gates.

## Runtime Heartbeat Architecture (Target)

### A. Internal beat endpoint

Add a dedicated internal beat endpoint (example: `POST /internal/adversary-sim/beat`) that:

1. authenticates with explicit internal secret and trust-boundary checks,
2. reconciles control state and run window,
3. executes at most one bounded generation beat per call,
4. persists state and diagnostics,
5. never performs control-plane mutation beyond beat-owned counters/health.

### B. Control endpoint contract

`POST /admin/adversary-sim/control` only mutates desired control state and returns status; it does not execute generation ticks inline.

### C. Supervisor process behavior

Host-side supervisor process:

1. starts on toggle-on,
2. calls internal beat every configured interval (default 1s),
3. exits on toggle-off, run-window expiry, or server unreachability,
4. performs no generation work while simulation is off.

### D. Off-state inertness contract

When simulation is off:

1. no active heartbeat loop,
2. no active generator process,
3. no emitted simulation traffic,
4. next server start defaults to toggle off.

Historical telemetry is retained according to retention policy and is not wiped by stop/off transitions.

## Duplication Inventory and Convergence Plan

### Duplicated today

1. Overlapping deterministic attack stimuli in runtime Rust generator and Python runner drivers.
2. Separate evolution of scenario families (risk of taxonomy drift).

### Not duplicated (and should stay separate)

1. CI control-plane setup/reset logic.
2. Quantitative gate/report generation and repeatability checks.
3. Runtime control endpoint and operator lifecycle semantics.

### Convergence target

1. Shared deterministic attack corpus artifact consumed by both executors.
2. Executor-specific adapters:
   - runtime adapter executes runtime-safe attacker actions only,
   - CI adapter executes same attacker actions plus setup/gate/report workflows.
3. Contract tests that fail on corpus-version drift across executors.

## Deployment Notes Across Target Environments

1. Single host/VPS: transient Rust supervisor process via host launcher (`make dev` flow + production service manager guidance).
2. Container/Kubernetes: sidecar or dedicated worker using same Rust supervisor binary.
3. Edge/no local process control environments: external supervisor service calling the internal beat endpoint.

## Non-Negotiable Guarantees

1. Shared deterministic corpus is canonical for deterministic attacker actions.
2. CI oracle remains separate and focused on verification.
3. Runtime heartbeat is out-of-process and independent of request path and dashboard refresh.
4. Toggle remains ephemeral and defaults to off on restart.
5. Off-state consumes no simulation-generation resources.
6. Simulation traffic remains in same enforcement/telemetry path as external traffic (distinguished only by simulation metadata tags).

## Expected Payoff

1. Lower runtime/CI drift and stronger regression confidence.
2. Faster debugging through shared scenario semantics.
3. Cleaner architecture boundaries and fewer coupling regressions.
4. Better readiness for adding LLM/containerized lane on top of one runtime heartbeat model.
