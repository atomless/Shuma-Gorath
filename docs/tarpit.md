# 🐙 Tarpit

Shuma-Gorath tarpit is an attack-escalation control, not a normal unsolved-user path.
Normal unsolved challenge attempts go to maze (or block if maze is disabled). Tarpit is reserved for confirmed challenge attacks and selected policy actions.

## 🐙 What Tarpit Is For

- Raise malicious-automation cost with repeated proof-gated progression.
- Keep host cost bounded with strict concurrency and egress budgets.
- Escalate persistent attacking sources to short-ban, then block.

## 🐙 Why This Is A Differentiator

- Built from a dedicated tarpit research track and ecosystem re-review, not from a generic delay-loop pattern.
- Keeps the operational discipline seen in strong enterprise/operator examples (strict budgets, explicit fallback, observability-first posture).
- Extends that baseline with Shuma-specific progression chaining, proof gating, and app-level routing control.

For the cross-capability comparison of enterprise baseline vs Shuma advancement, see [`value-proposition.md`](value-proposition.md).

## 🐙 When Tarpit Is Served

Tarpit handling is attempted only when both are enabled:

- `maze_enabled=true`
- `tarpit_enabled=true`

Tarpit-capable entry paths:

- Challenge attack outcomes (for example replay/tamper/sequence abuse).
- Not-a-Bot attack outcomes (for example replay/invalid-seed/binding/attempt-window abuse).
- IP-range policy rules with `action=tarpit`.

If tarpit cannot be served, behavior is deterministic:

- Challenge attack path: short ban (600s) + block response.
- IP-range tarpit action: fallback to maze (if enabled) otherwise block.

Crawler/safety bypass:

- Requests for `/robots.txt`, `/sitemap.xml`, `/health` and known indexer user agents bypass tarpit and are routed to maze fallback instead.

## 🐙 Progression Flow (Implementation)

1. **Entry attempt**
   - Runtime increments a per-IP-bucket tarpit persistence counter.
   - Escalation thresholds:
     - persistence `>= 5`: short ban (600s)
     - persistence `>= 10`: block
2. **Entry budget check**
   - Enforces active-concurrency caps:
     - global (`tarpit_max_concurrent_global`)
     - per IP bucket (`tarpit_max_concurrent_per_ip_bucket`)
   - If saturated, apply `tarpit_fallback_action` (`maze` or `block`).
3. **Bootstrap response**
   - Returns small HTML, not a long-lived drip stream.
   - Includes `window.__shumaTarpit` with:
     - signed progression token
     - `/tarpit/progress` endpoint
     - initial hashcash difficulty
4. **Progress step (`POST /tarpit/progress`)**
   - Client submits JSON payload: `token` + `nonce`.
   - Server validates:
     - token signature/version/path class/window
     - IP bucket and user-agent bucket binding
     - step order
     - parent-chain continuity
     - replay marker
     - hashcash proof validity
5. **Budget and chunk control**
   - Before emitting a chunk, server enforces:
     - per-flow max duration (`tarpit_egress_per_flow_max_duration_seconds`)
     - per-flow max bytes (`tarpit_egress_per_flow_max_bytes`)
     - windowed global bytes (`tarpit_egress_global_bytes_per_window`)
     - windowed per-bucket bytes (`tarpit_egress_per_ip_bucket_bytes_per_window`)
   - Chunk size is bounded and jittered (`tarpit_step_chunk_base_bytes`, `tarpit_step_chunk_max_bytes`, `tarpit_step_jitter_percent`), with optional shard rotation.
6. **State commit and next token**
   - Writes replay/chain markers, increments step and egress counters.
   - Issues next signed token with bounded adaptive difficulty (if enabled).

Budget exhaustion during progression returns deterministic fallback (`maze` or `block`).

## 🐙 Why This Is Not Slow-Drip

Shuma intentionally avoids classic slow-drip tarpit streams as the primary mode because they hold server connections longer and can shift too much residency cost to the host under heavy abuse.

Current design favors bounded-cost operation:

- short entry response,
- iterative proof-gated progression,
- strict replay/order/binding checks,
- hard budget caps and deterministic fallback.

This keeps operational cost control explicit while still increasing bot-side work.

## 🐙 Operator Configuration Surface

Main config pane:

- `tarpit_enabled` (single user-facing tarpit toggle)

Advanced JSON config (runtime controls):

- progression token windows (`tarpit_progress_token_ttl_seconds`, `tarpit_progress_replay_ttl_seconds`)
- hashcash difficulty policy (`tarpit_hashcash_min_difficulty`, `tarpit_hashcash_max_difficulty`, `tarpit_hashcash_base_difficulty`, `tarpit_hashcash_adaptive`)
- chunk shaping (`tarpit_step_chunk_base_bytes`, `tarpit_step_chunk_max_bytes`, `tarpit_step_jitter_percent`, `tarpit_shard_rotation_enabled`)
- egress budgets (`tarpit_egress_*`)
- active concurrency budgets (`tarpit_max_concurrent_*`)
- saturated-path fallback (`tarpit_fallback_action`)

For full variable definitions and ranges, see [`configuration.md`](configuration.md).

## 🐙 Monitoring and Operator Visibility

Prometheus metrics include:

- `bot_defence_tarpit_activations_total{mode=...}`
- `bot_defence_tarpit_progress_outcomes_total{outcome=...}`
- `bot_defence_tarpit_proof_outcomes_total{outcome="required|passed|failed"}`
- `bot_defence_tarpit_chain_violations_total{reason="step_out_of_order|parent_chain_missing|replay"}`
- `bot_defence_tarpit_budget_outcomes_total{outcome=...}`
- `bot_defence_tarpit_budget_exhaustion_reasons_total{reason=...}`
- `bot_defence_tarpit_escalation_outcomes_total{outcome=...}`
- `bot_defence_tarpit_duration_buckets_total{bucket=...}`
- `bot_defence_tarpit_bytes_buckets_total{bucket=...}`

Admin monitoring payload (`GET /admin/monitoring`) also includes tarpit runtime counters and config snapshots under `details.tarpit`, including:

- progression admissions and denials,
- proof outcomes,
- chain-violation totals and reasons,
- budget exhaustion reasons plus fallback actions,
- duration and bytes buckets,
- and capped persistence offender buckets.

Preview surface:

- `GET /admin/tarpit/preview` renders a non-operational tarpit preview for operators.

## 🐙 Related Docs

- `configuration.md` (config classes + variable catalog)
- `maze.md` (maze flow and rollout)
- `research/README.md` (research index)
