# ADR 0004: Tarpit v2 Progression Contract

- Status: Proposed
- Date: 2026-02-23
- Owners: @jamestindall
- Related:
  - [`docs/plans/2026-02-22-http-tarpit-cost-shift-implementation-plan.md`](../plans/2026-02-22-http-tarpit-cost-shift-implementation-plan.md)
  - [`docs/plans/2026-02-23-tarpit-v2-progression-envelope.md`](../plans/2026-02-23-tarpit-v2-progression-envelope.md)
  - `todos/todo.md` (`TAH-1`..`TAH-18`)

## Context

Current tarpit behavior is still `maze_only` or `maze_plus_drip` with bounded concurrency, but the `maze_plus_drip` path still builds a single hidden filler payload in one response body. That does not satisfy the intended v2 asymmetry model:

- small initial server response,
- iterative work-gated progression,
- strict token-chain continuity,
- deterministic budget/fallback behavior,
- explicit replay/order/binding rejection reasons.

We also need a clear and stable operator config policy for UI exposure while this work lands.

## Decision

Adopt a tarpit v2 contract with the following invariants.

1. Response model:
   - Tarpit entry returns a small bootstrap response.
   - All costly progression happens through a dedicated progression endpoint.
   - Each step emits only a bounded chunk after proof verification.
2. Proof-gated progression:
   - Every step requires a server-verifiable work proof.
   - Default proof is hashcash-style leading-zero work with bounded difficulty.
   - Work difficulty is adaptive only within configured lower/upper bounds.
3. Token-chain continuity:
   - Every progression token is signed, short-lived, single-use, and bound to IP bucket and UA bucket.
   - Step `N+1` must reference a valid parent digest from step `N`.
   - Out-of-order, stale, replayed, or binding-mismatched submissions are rejected with explicit reason codes.
4. Budget envelope:
   - Tarpit step admission and emission are budget-checked (global, per-bucket, per-flow bytes/time).
   - Budget exhaustion uses deterministic fallback policy (`maze` or `block`) and records explicit outcomes.
   - Persistent tarpit behavior can escalate to short-ban, then block, per existing abuse policy.
5. Config exposure policy:
   - Main config pane exposes only `tarpit_enabled`.
   - Remaining tarpit runtime controls are Advanced JSON.
   - Absolute safety ceilings remain env-only where applicable.

## Alternatives Considered

1. Keep single-response `maze_plus_drip` filler model.
2. Make tarpit fully standalone and independent from maze primitives.
3. Expose many tarpit tuning knobs in the main UI.

## Consequences

### Positive

- Stronger attacker cost asymmetry with tighter host-cost controls.
- Clear progression semantics for tests, monitoring, and incident triage.
- Lower operator error risk from keeping advanced knobs out of the main pane.

### Negative / Trade-offs

- More moving parts (endpoint, token lifecycle, proof validation, budget accounting).
- More state keys and reason-code surfaces to keep consistent across runtime and docs.

## Security Impact

- Improves replay/order/binding integrity via explicit signed progression envelopes.
- Reduces chance of accidental unbounded tarpit behavior through per-step gate checks.
- Requires careful implementation of single-use token semantics and authoritative reason labeling.

## Human Friction Impact

- No additional friction for likely-human paths because tarpit remains limited to confirmed attack paths.
- Legitimate crawler/sensitive-path bypass behavior must remain explicit and documented.

## Adversary Cost Placement

- Moves cost from one large server-generated payload to repeated proof-validated progression work.
- Forces bots to perform iterative verified work and sustained retrieval to continue.

## Operational Impact

- Deploy: no external dependency required for internal-first runtime.
- Config: main pane remains simple (`tarpit_enabled` only); advanced tuning in JSON.
- Monitoring/alerts: add progression admissions/denials, proof outcomes, chain violations, and budget outcomes.
- Rollback: disable tarpit (`tarpit_enabled=false`) or force deterministic fallback policy.

## Resource Impact

- Bandwidth: bounded and explicitly budgeted per flow/window.
- CPU: predictable per-step verification overhead; bounded by difficulty and admission controls.
- Memory: lightweight progression state with TTL lifecycle and replay keys.
- Energy/efficiency notes: iterative small-step model avoids large one-shot body generation.

## Verification

- Tests:
  - progression token schema + replay/order/binding validation,
  - proof verification behavior and reason-code mapping,
  - budget admission/exhaustion deterministic fallback behavior.
- Benchmarks (if relevant):
  - step verification overhead and bounded emission checks under burst traffic.
- Docs updated:
  - [`docs/plans/2026-02-23-tarpit-v2-progression-envelope.md`](../plans/2026-02-23-tarpit-v2-progression-envelope.md)
  - [`docs/dashboard.md`](../dashboard.md)
  - `todos/todo.md`

## Follow-ups

- Implement `TAH-3`..`TAH-18` in atomic slices under this contract.
