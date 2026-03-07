# ADR 0010: Adversary Sim Autonomous Heartbeat and UI-Decoupled Generation

- Status: Proposed
- Date: 2026-03-01
- Owners: Shuma core maintainers
- Related:
  - [`0007-adversary-sim-toggle-command-controller.md`](0007-adversary-sim-toggle-command-controller.md)
  - [`0008-realtime-monitoring-cursor-sse-hybrid.md`](0008-realtime-monitoring-cursor-sse-hybrid.md)
  - [`../../todos/todo.md`](../../todos/todo.md) (`SIM-DEPLOY-2`)
  - [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md) (`SIM-LLM-1`)
  - [`../../todos/completed-todo-history.md`](../../todos/completed-todo-history.md) (`SIM-DEPLOY-1`)

## Context

Current adversary-sim traffic generation cadence is driven by dashboard runtime timers that call `POST /admin/adversary-sim/tick`. This creates coupling between:

1. attack generation and dashboard page lifecycle,
2. control-plane UI state updates and runtime generation cadence,
3. monitoring refresh behavior and perceived simulator liveness.

Observed outcomes include flaky/twitchy toggle UX, generation gaps when dashboard timers are interrupted, and architectural mismatch with the intended model where adversary traffic should exist independently of monitoring view refresh.

At the product level, adversary simulation is also being reconsidered as potential production value (operator-in-place red teaming after deployment), which is incompatible with browser-lifecycle-driven generation.

## Decision

1. Make adversary traffic generation heartbeat backend-owned and UI-independent.
2. Dashboard must not own or drive generation cadence; it is limited to:
   - control commands (`POST /admin/adversary-sim/control`),
   - status reads (`GET /admin/adversary-sim/status`),
   - monitoring rendering.
3. Introduce an autonomous adversary-sim supervisor path that:
   - reads control desired/actual state,
   - emits bounded tick execution on its own cadence,
   - enforces resource and failure guardrails,
   - remains active regardless of dashboard tab presence.
4. Keep monitoring freshness as backend source-of-truth; UI renders backend freshness and does not synthesize competing freshness ownership.
5. Keep deterministic lane as release-blocking regression oracle; LLM/containerized lane remains discovery/promotable corpus.
6. This ADR makes the adversary-sim runtime production-capable; ongoing production operating-envelope hardening lives in `SIM-DEPLOY-*`.

## Alternatives Considered

1. Dashboard Web Worker heartbeat
2. Service Worker / Shared Worker heartbeat
3. Continue dashboard-timer heartbeat with UX hardening only
4. Backend autonomous supervisor (selected)

## Consequences

### Positive

- Removes browser/tab lifecycle as generation dependency.
- Aligns architecture with real-traffic model (internal adversary should behave like independent actor).
- Reduces toggle/status flicker classes caused by multiple frontend writers.
- Establishes a viable path toward production operator red-teaming.

### Negative / Trade-offs

- Adds runtime operational complexity (supervisor lifecycle, health, and failure handling).
- Requires careful capability boundaries so supervisor cannot bypass existing trust controls.
- Requires incremental migration and temporary compatibility period.

## Security Impact

- Improves security posture by removing UI-driven generation side effects.
- Requires strict least-authority capability model for supervisor control surface.
- Requires explicit abuse controls and bounded operating envelopes for production use.

## Human Friction Impact

- Operators can close dashboard without halting active simulation.
- UI mental model is simpler: control and observe, not drive.
- Additional operational docs are required for supervisor health and diagnostics.

## Adversary Cost Placement

- Better supports realistic cost-imposition and defense-tuning behavior by making adversary generation independent of UI polling cadence.
- Preserves deterministic verification while allowing adaptive discovery to run as a real actor lane.

## Operational Impact

- Deploy:
  - add supervisor runtime path (local and deployment profiles) with explicit lifecycle management.
- Config:
  - add cadence/backoff/guardrail controls for supervisor-driven ticks.
  - maintain explicit lifecycle and resource guardrails for any production rollout.
- Monitoring/alerts:
  - add supervisor heartbeat health/lag diagnostics and failure taxonomy.
- Rollback:
  - disable supervisor generation path and force adversary sim to `off`; preserve monitoring visibility for historical telemetry.

## Resource Impact

- Bandwidth:
  - predictable and bounded by supervisor cadence and lane budget.
- CPU:
  - modest constant overhead while enabled; none when disabled.
- Memory:
  - small additional footprint for supervisor state/queueing.
- Energy/efficiency notes:
  - removes waste from browser-timer retries and allows tighter backoff semantics under failure.

## Verification

- Tests:
  - generation continues when dashboard tab is closed,
  - no dashboard tick endpoint calls are required for traffic generation,
  - toggle/control semantics remain idempotent and lease-safe,
  - monitoring freshness remains backend-authored and stable under transport fallback.
- Benchmarks (if relevant):
  - cadence accuracy and resource envelope under enabled runs.
- Docs updated:
  - ADR added; follow-up implementation docs required with rollout.

## Follow-ups

- Implement supervisor-driven generation path and remove dashboard-owned tick cadence.
- Remove frontend tick loop and related optimistic state races once backend heartbeat is live.
- `SIM-DEPLOY-1` decision is complete; continue `SIM-DEPLOY-2` as the production operating-envelope hardening tranche.
