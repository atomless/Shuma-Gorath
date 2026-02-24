# HTTP Tarpit Cost-Shift Implementation Plan

Date: 2026-02-22  
Status: Proposed

Reference research:

- [`docs/research/2026-02-22-http-tarpit-cost-shift-research-synthesis.md`](../research/2026-02-22-http-tarpit-cost-shift-research-synthesis.md)
- [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../research/2026-02-14-maze-tarpit-research-synthesis.md)

## Objective

Ship a Shuma-native tarpit implementation that:

1. raises attacker time/bandwidth cost,
2. keeps defender CPU/memory/connection cost bounded,
3. reuses shared maze primitives (token, budget, fallback, observability),
4. supports deterministic rollback and safe failure behavior.

## Non-goals

- Unbounded hold-open streams.
- Tarpit activation for uncertain or likely-human traffic.
- New tarpit-only token/budget state systems that diverge from maze behavior.

## Architecture Decisions

1. **Internal-first runtime**
   - Implement in Shuma Rust/Spin core; do not add third-party runtime dependencies.
2. **Shared primitive reuse**
   - Consume maze token and budget primitives (`TP-C1`, `TP-C2` constraints).
3. **Mode model**
   - `maze_only` baseline mode first.
   - `maze_plus_drip` second, only after guardrails and telemetry are in place.
4. **Deterministic fallback**
   - On tarpit budget saturation, fallback is explicit (`maze` or `block`) and observable.
5. **High-confidence gating**
   - Keep tarpit tied to abuse-grade paths; preserve low-friction handling for humans.

## Delivery Phases

## Phase 1: Internal `maze_only` Tarpit Availability

Purpose:

- Ensure tarpit is actually available for abuse paths before adding drip complexity.

Scope:

- Implement `maybe_handle_tarpit` for internal provider using maze-backed response path.
- Preserve short-ban fallback when tarpit is unavailable (maze disabled or provider unavailable).
- Add explicit admin/event labels for tarpit activation and outcome.

Acceptance criteria:

- Challenge abuse route no longer short-bans by default when internal maze/tarpit path is active.
- Deterministic fallback behavior remains unchanged when tarpit cannot run.

## Phase 2: Config Surface and Guardrails

Purpose:

- Make tarpit behavior operator-controlled while secure-by-default.

Scope:

- Add tarpit config set (<abbr title="Key-Value">KV</abbr>-backed admin-editable unless env-only by policy):
  - `tarpit_enabled`
  - single progressive tarpit behavior (no mode switch)
  - `tarpit_bytes_per_second`
  - `tarpit_stream_timeout_seconds`
  - `tarpit_max_concurrent_global`
  - `tarpit_max_concurrent_per_ip_bucket`
  - `tarpit_fallback_action`
- Wire defaults/env lifecycle (defaults file, seed/bootstrap, docs) per project config lifecycle rules.
- Expose all KV-editable values in Advanced JSON config.

Acceptance criteria:

- Defaults are secure and bounded.
- Invalid values clamp safely and predictably.
- Config appears consistently in admin API and dashboard Advanced JSON.

## Phase 3: Progressive Bounded Stream Mode

Purpose:

- Add true cost-imposition stream behavior with bounded host impact.

Scope:

- Add bounded drip response engine:
  - fixed byte pacing,
  - hard timeout,
  - hard per-response byte cap.
- Enforce shared budget governor (global + per-bucket).
- Apply deterministic fallback on budget exhaustion.

Acceptance criteria:

- No unbounded streams.
- Budget saturation triggers documented fallback.
- Runtime latency and resource impact remain within thresholds.

## Phase 4: Escalation, Distributed State, and Operability

Purpose:

- Harden behavior under persistent abuse and multi-instance deployments.

Scope:

- Escalate repeat tarpit persistence to short-ban/block with false-positive guardrails.
- Integrate tarpit counters with enterprise distributed state work.
- Add tarpit metrics/admin panels:
  - activated,
  - active streams,
  - bytes sent,
  - durations,
  - budget saturation,
  - fallback and escalation outcomes.

Acceptance criteria:

- Operators can tune using concrete telemetry.
- Enterprise mode avoids silent counter divergence.

## Verification Strategy

1. Unit tests:
   - mode selection,
   - budget admission/rejection,
   - deterministic fallback mapping,
   - escalation threshold behavior.
2. Integration tests:
   - abuse path to tarpit path,
   - budget exhaustion fallback,
   - replay/tamper handling.
3. Dashboard/e2e:
   - config propagation to preview/served behavior where applicable.
4. Makefile workflow:
   - `make test-unit` for iteration,
   - `make test` (with Spin runtime active via `make dev`) before completion claims.

## Rollout Guardrails

Rollout should pause/rollback if sustained:

- tarpit saturation beyond defined percentage of eligible requests,
- protected-route latency regression beyond threshold,
- human-success metrics degrade beyond threshold,
- unexpected rise in non-2xx/3xx on protected routes.

## Security and Resource Notes

- Security:
  - Keep tarpit limited to abuse-grade signals to reduce false positives.
  - Maintain explicit reason/outcome labeling for post-incident audits.
- Resource:
  - Hard caps are mandatory to avoid self-DoS.
  - Prefer low-memory state and short TTLs for in-flight tracking.

## TODO Mapping

This plan maps to `todos/todo.md` items:

- `TP-C1`, `TP-C2` (shared primitive reuse constraints),
- `TP-0` through `TP-9` (new phased tarpit implementation slices).
