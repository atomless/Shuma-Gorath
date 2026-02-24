# Tarpit v2 Progression Envelope Spec

Date: 2026-02-23  
Status: Proposed

Related:

- [`docs/adr/0004-tarpit-v2-progression-contract.md`](../adr/0004-tarpit-v2-progression-contract.md)
- `todos/todo.md` (`TAH-2`, `TAH-3`, `TAH-4`, `TAH-7`, `TAH-9`, `TAH-11`)

## Objective

Define the signed progression token schema and validation rules for tarpit v2 so runtime, tests, and monitoring use one contract.

## Envelope Schema

Format:

- Signed token string (`token.payload.signature` style, implementation-specific encoding).
- Payload is a structured object with these required fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `version` | `u8` | Token schema version (`1` for initial rollout). |
| `operation_id` | `string` | Single-use step operation identifier. |
| `flow_id` | `string` | Stable identifier for one tarpit flow. |
| `step` | `u16` | Monotonic progression index (entry token starts at `0`). |
| `parent_digest` | `string` | Digest of previous accepted step (`entry` sentinel at step `0`). |
| `ip_bucket` | `string` | Source IP bucket binding at issuance time. |
| `ua_bucket` | `string` | User-Agent bucket binding at issuance time. |
| `path_class` | `string` | Expected route class (`tarpit_progress`). |
| `issued_at` | `u64` | Unix timestamp seconds at issue time. |
| `expires_at` | `u64` | Unix timestamp seconds at expiry time. |
| `difficulty` | `u8` | Hashcash difficulty for this step (bounded by policy). |
| `work_alg` | `string` | Work algorithm id (`hashcash_sha256_v1` initially). |
| `max_chunk_bytes` | `u32` | Upper bound for bytes emitted for this step. |
| `flow_bytes_emitted` | `u64` | Cumulative emitted bytes before this step. |
| `flow_started_at` | `u64` | Flow start timestamp (seconds) for duration budgeting. |

Optional fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `hint` | `string` | Optional operator/debug-safe hint for monitoring correlation. |
| `policy_epoch` | `u32` | Optional policy revision marker for rollout diagnostics. |

## Validation Rules

Validation order is normative.

1. Signature and structure
   - Reject malformed token or signature mismatch.
   - Reject unsupported `version`.
2. Temporal validity
   - Reject if `now > expires_at`.
   - Reject if `issued_at > expires_at`.
   - Reject if token age exceeds configured replay envelope.
3. Binding checks
   - Current request IP bucket must equal `ip_bucket`.
   - Current request UA bucket must equal `ua_bucket`.
   - Request path class must equal `path_class`.
4. Ordering checks
   - Expected step for flow must equal `step`.
   - `parent_digest` must match a previously accepted chain marker for the flow.
5. Replay checks
   - `operation_id` must be single-use for the flow.
   - Reuse of already-seen `operation_id` is replay and rejected.
6. Proof checks
   - Verify proof according to `work_alg` and `difficulty`.
   - Reject missing/invalid proof.
7. Budget checks
   - Verify flow/global/per-bucket budgets before emitting next chunk.
   - If budget exhausted, emit deterministic fallback outcome.

## Rejection Reason Codes (Normative)

These reason keys should be emitted consistently in event logs and metrics labels.

- `tarpit_progress_malformed`
- `tarpit_progress_signature_mismatch`
- `tarpit_progress_invalid_version`
- `tarpit_progress_expired`
- `tarpit_progress_invalid_window`
- `tarpit_progress_binding_ip_mismatch`
- `tarpit_progress_binding_ua_mismatch`
- `tarpit_progress_path_mismatch`
- `tarpit_progress_step_out_of_order`
- `tarpit_progress_parent_chain_missing`
- `tarpit_progress_replay`
- `tarpit_progress_invalid_proof`
- `tarpit_progress_budget_exhausted`

## Success Outcome Contract

When progression succeeds:

1. Mark `operation_id` as seen (single-use replay key with TTL).
2. Record chain marker keyed by `(flow_id, digest(flow_id:operation_id))`.
3. Emit bounded content chunk (`<= max_chunk_bytes` and within flow budget).
4. Issue next signed token with:
   - `step + 1`,
   - `parent_digest = digest(flow_id:operation_id)`,
   - updated `flow_bytes_emitted`,
   - bounded/adaptive `difficulty` (within configured min/max).

## State Keys (Initial Naming Contract)

These names define the intended semantics; exact storage helpers may evolve.

- `tarpit:progress:seen:<flow_id>:<operation_id>`
- `tarpit:progress:chain:<flow_id>:<digest>`
- `tarpit:progress:step:<flow_id>`
- `tarpit:budget:egress:global:<site_id>:<window_id>`
- `tarpit:budget:egress:bucket:<site_id>:<ip_bucket>:<window_id>`
- `tarpit:budget:egress:flow:<flow_id>`

## Config Classification

Main pane:

- `tarpit_enabled` only.

Advanced JSON:

- all progression difficulty, budget, and fallback tuning knobs.

Env-only:

- absolute safety ceilings/guardrails where policy requires non-runtime mutability.
