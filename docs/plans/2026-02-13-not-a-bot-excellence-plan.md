# Not-a-Bot Checkbox Excellence Plan

Date: 2026-02-13  
Last revised: 2026-03-06  
Status: Implemented (core tranche complete; carry-forward hardening remains open)

## Context

Not-a-bot (`/challenge/not-a-bot-checkbox`) is the intended low-friction checkpoint between passive signals and puzzle/maze escalation. The module existed only as a placeholder.

## Goals

- Add a strong lightweight challenge for medium-uncertainty traffic.
- Keep verifier cost bounded and deterministic under attack.
- Make outcome routing explainable: `pass`, `escalate_puzzle`, `maze_or_block`.
- Preserve accessibility with equivalent-strength keyboard/touch completion semantics.

## Non-goals

- Third-party CAPTCHA dependency in the default path.
- Stateful long-term user profiling.
- Replacing puzzle or maze as high-certainty controls.

## Research-derived constraints

1. Server-side token verification must be short-lived and single-use.
2. Checkbox success cannot be binary authority; it must be scored with corroborating signals.
3. Responses must be oracle-resistant; detailed failure reasons stay internal.
4. Equivalent accessibility path is mandatory for production quality.
5. Monitoring/output dimensions must stay low-cardinality and bounded for cost control.
6. Optional PAT-like signals are additive only and cannot be sole allow authority.
7. Interaction must be single-step (checkbox-like activation progresses immediately; no secondary submit button).
8. Accessibility paths are neutral-to-positive evidence only; never penalize assistive usage patterns directly.

Reference synthesis: [`docs/research/2026-02-19-not-a-bot-challenge-research-synthesis.md`](../research/2026-02-19-not-a-bot-challenge-research-synthesis.md).

## Internal vs Akamai ownership

- `self_hosted_minimal`:
  - Internal not-a-bot endpoint, scoring, and replay controls.
- `enterprise_akamai`:
  - Edge risk signals can tune trigger thresholds.
  - Shuma remains authoritative for nonce semantics and outcome routing.

## Consolidated Implementation Contract

### Purpose and scope

- Provide a low-friction step-up challenge for medium-uncertainty traffic before escalating to the stronger puzzle challenge.
- Keep not-a-bot a scored verification checkpoint, not a boolean checkbox gate.
- Keep very-low-certainty traffic in managed or invisible mode (passive signals plus JS verification/PoW only when required) rather than forcing this interactive step.
- Keep the default path edge-local and same-origin, with deterministic server-side verification and routing.

Out of scope:

- Cross-site reputation graph.
- Third-party CAPTCHA or identity-provider dependency in the default path.

### Routing placement

Runtime intent order for the core ladder is:

1. Hard blocks, bans, and explicit policy routes.
2. Maze route for high-confidence automation.
3. Not-a-Bot for medium-uncertainty traffic.
4. Puzzle challenge for stronger verification when Not-a-Bot is inconclusive.
5. Allow.

### Endpoint and rendering contract

- `POST /challenge/not-a-bot-checkbox` validates the signed seed plus compact telemetry summary and produces `pass`, `escalate_puzzle`, or `maze_or_block` routing.
- Live traffic is normally served the Not-a-Bot page through policy-driven response rendering, not by direct operator navigation to the route.
- Direct `GET /challenge/not-a-bot-checkbox` exists as a test-mode preview and verification path; outside `test_mode` it returns `404`.
- Interaction is one-step only: checkbox-like activation progresses immediately and there is no secondary Continue button.

### Signed nonce and continuity marker model

The signed seed is stronger than the original draft spec and now includes:

- `operation_id`
- `flow_id`
- `step_id`
- `step_index`
- `issued_at`
- `expires_at`
- `token_version`
- `ip_bucket`
- `ua_bucket`
- `path_class`
- `return_to`

Server validation enforces:

1. Signature validity.
2. Expiry.
3. Request binding (`ip_bucket`, `ua_bucket`, submit path class).
4. Single-use replay protection for the signed operation.
5. Ordering-window and timing-primitives validation through the operation-envelope contract.
6. Attempt caps per IP bucket and short window.

On `pass`, the runtime issues a short-lived same-origin continuity marker bound to IP and UA buckets.

### Compact telemetry contract

Current implementation accepts the following compact typed summary fields:

- `has_pointer`
- `pointer_move_count`
- `pointer_path_length`
- `pointer_direction_changes`
- `down_up_ms`
- `focus_changes`
- `visibility_changes`
- `interaction_elapsed_ms`
- `keyboard_used`
- `touch_used`
- `activation_method`
- `activation_trusted`
- `activation_count`
- `control_focused`

Validation rules:

- reject malformed payloads,
- reject invalid ranges and invalid activation enum values,
- ignore unexpected extra fields,
- never ingest raw event streams.

Implementation note:

- The earlier draft spec included a client-reported `events_order_valid` field.
- Current code-truth no longer trusts a client-supplied ordering flag; ordering and timing integrity are enforced server-side via the signed operation-envelope checks instead.

### Scoring and outcomes

- Submit scoring remains bounded to `0..10`.
- Hard-fail conditions bypass weighted scoring and route to `maze_or_block`.
- Request-path context signals already determine whether traffic enters the Not-a-Bot band; they are not re-weighted inside the submit scorer today.
- Current score inputs are timing plausibility, motion plausibility, modality plausibility, activation semantics, focus/visibility stability, and trusted activation.
- Accessibility-neutral policy remains mandatory: keyboard-only and touch-first flows must stay pass-capable without requiring pointer motion.
- Outcome thresholds remain:
  - `pass` when score reaches `not_a_bot_pass_score`,
  - `escalate_puzzle` when score reaches `not_a_bot_fail_score` but not the pass threshold,
  - `maze_or_block` otherwise or on hard-fail conditions.

Current carry-forward note:

- Stronger corroboration gates and short-lived server-side continuity modifiers remain deferred to the Not-a-Bot hardening sprint rather than the completed core tranche.

### Runtime controls and monitoring parity

Operational controls:

- `not_a_bot_enabled`
- `not_a_bot_risk_threshold`
- `not_a_bot_pass_score`
- `not_a_bot_fail_score`
- `not_a_bot_nonce_ttl_seconds`
- `not_a_bot_marker_ttl_seconds`
- `not_a_bot_attempt_limit_per_window`
- `not_a_bot_attempt_window_seconds`

Monitoring parity:

- counters for `served`, `pass`, `escalate`, `fail`, and `replay`,
- solve-latency buckets,
- abandonment estimate (`served - submitted`),
- dashboard visibility without unbounded payload cardinality.

### Security and privacy notes

- Treat telemetry as anti-abuse signal only; avoid persistent user profiling.
- Never expose scoring internals in client responses.
- Keep external failure responses generic.
- Keep debug detail behind explicit dev-only or test-only paths.
- Keep telemetry schema compact and fixed to control serialization and storage cost.
- Optional PAT/private-attestation style signals, if added, are additive evidence only and must never be sole allow authority.

## Proposed architecture

### A. Signed nonce lifecycle

- Short-lived signed payload with operation id, flow step fields, and expiry.
- Bind to IP bucket + UA bucket + expected submit path class.
- Enforce single-use replay marker with bounded TTL.

### B. Compact telemetry contract

- Strict typed/ranged summary fields only.
- Reject malformed payloads; ignore no untrusted dynamic extras.
- No raw high-cardinality event-stream ingestion.

### C. Deterministic scoring and outcomes

- Score normalized to `0..10`.
- Hard-fail conditions bypass weighted score.
- Outcome routing:
  - `pass` -> short-lived continuity marker,
  - `escalate_puzzle` -> stronger challenge,
  - `maze_or_block` -> policy-driven high-cost path.

### D. Abuse controls

- Per-IP-bucket attempt cap in short windows.
- Replay rejection with explicit metric labels.
- Generic external failure responses.

### E. Escalation placement

- Trigger not-a-bot below puzzle threshold.
- Trigger puzzle above not-a-bot threshold.
- Trigger maze above maze threshold or on repeated high-confidence abuse.

### F. Runtime controls + monitoring parity

- Runtime controls for not-a-bot route threshold, nonce/marker TTL, and attempt caps.
- Monitoring parity for not-a-bot:
  - outcomes (`served`, `pass`, `escalate`, `fail`, `replay`),
  - solve-latency buckets,
  - abandonment estimate (`served - submitted`).
- Dashboard exposure without introducing unbounded payload cardinality.

## Ordered implementation sequence

- [x] NAB-1: Implement GET/POST not-a-bot endpoints and signed nonce parse/verify.
- [x] NAB-2: Add strict telemetry schema validation and bounded scoring (`0..10`).
- [x] NAB-3: Implement continuity marker token issuance and verification on pass.
- [x] NAB-4: Add attempt caps, cooldown, replay tracking, and generic failure responses.
- [x] NAB-5: Wire policy routing for lower botness certainty -> not-a-bot -> puzzle escalation.
- [x] NAB-6: Add runtime controls for routing threshold, nonce/marker TTL, and attempt caps.
- [x] NAB-7: Add monitoring + dashboard parity (`served`, `pass`, `escalate`, `fail`, `replay`, solve latency, abandonment estimate).
- [x] NAB-8: Add lifecycle tests for success + all failure classes (unit + integration + dedicated browser e2e).
- [x] NAB-9: Add operator docs and threshold tuning guidance.
- [ ] NAB-10: Evaluate optional PAT-like attestation adapter as additive low-friction signal (non-blocking).
- [x] NAB-11: Align UI with state-of-the-art one-step interaction (`role=checkbox` control, auto-submit on activation, remove separate Continue action).
- [x] NAB-12: Formalize and document very-low-certainty invisible flow mapping (passive signals + JS/PoW path) so not-a-bot remains medium-certainty only.
- [x] NAB-13: Update scoring semantics for activation-modality evidence while preserving equivalent-strength keyboard/touch pass paths.

## Carry-forward work

- PAT-like additive attestation evaluation remains open and is tracked in [`todos/todo.md`](../../todos/todo.md) as `NAB-12`.
- Additional corroboration and continuity hardening remains open in [`docs/plans/2026-02-21-not-a-bot-hardening-sprint.md`](2026-02-21-not-a-bot-hardening-sprint.md) and is tracked in [`todos/todo.md`](../../todos/todo.md) as `NAB-13`.

## Source references

- [`docs/research/2026-02-19-not-a-bot-challenge-research-synthesis.md`](../research/2026-02-19-not-a-bot-challenge-research-synthesis.md)
- https://www.akamai.com/products/bot-manager
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- https://developers.cloudflare.com/cloudflare-challenges/challenge-types/challenge-pages/
- https://developers.cloudflare.com/turnstile/get-started/server-side-validation/
