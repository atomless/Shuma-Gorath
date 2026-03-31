# 🐙 <abbr title="Application Programming Interface">API</abbr> & Endpoints

## 🐙 Authentication

Admin endpoints support two auth modes:
- Bearer token (read/write): `Authorization: Bearer <SHUMA_API_KEY>`
- Bearer token (read-only, optional): `Authorization: Bearer <SHUMA_ADMIN_READONLY_API_KEY>`
- Session cookie: `POST /shuma/admin/login` as `application/x-www-form-urlencoded` with `password=<SHUMA_API_KEY>` (and optional `next=/shuma/dashboard/index.html`) sets a short-lived `HttpOnly` cookie and redirects with `303 See Other`

Write endpoints (`POST`, `PUT`, `PATCH`, `DELETE` on mutating admin routes) require read/write access.
Read-only bearer tokens can access non-mutating admin endpoints only.

If `SHUMA_ADMIN_IP_ALLOWLIST` is set, the client <abbr title="Internet Protocol">IP</abbr> must be in the allowlist.

For session-authenticated write requests (`POST`, `PUT`, `PATCH`, `DELETE`), include:
- `X-Shuma-CSRF: <csrf_token>` (returned by `/shuma/admin/session`)

If `SHUMA_FORWARDED_IP_SECRET` is configured, any request that relies on `X-Forwarded-For` must also include:
- `X-Shuma-Forwarded-Secret: <SHUMA_FORWARDED_IP_SECRET>`

If `SHUMA_ENFORCE_HTTPS=true`:
- requests without <abbr title="Hypertext Transfer Protocol Secure">HTTPS</abbr> context are rejected with `403 HTTPS required`
- forwarded proto headers are trusted only when `SHUMA_FORWARDED_IP_SECRET` validation succeeds

If `SHUMA_API_KEY` is missing, `/shuma/admin/*` endpoints are disabled. Placeholder/insecure <abbr title="Application Programming Interface">API</abbr> keys are rejected.

Failed admin auth attempts are rate-limited per <abbr title="Internet Protocol">IP</abbr> bucket (`SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE`, default `10`), but you should still enforce <abbr title="Content Delivery Network">CDN</abbr>/<abbr title="Web Application Firewall">WAF</abbr> rate limits for `POST /shuma/admin/login` and `/shuma/admin/*`.

If `SHUMA_HEALTH_SECRET` is configured, `/shuma/health` also requires:
- `X-Shuma-Health-Secret: <SHUMA_HEALTH_SECRET>`

## 🐙 Public Endpoints

- `GET /` - Main bot defence handler
- `GET /shuma/health` - Health check (exact loopback or trusted forwarded loopback)
- `GET /shuma/metrics` - Prometheus metrics (no auth)
- `GET /instaban` - Honeypot (triggers ban)
- `GET /pow` - <abbr title="Proof of Work">PoW</abbr> challenge seed (when enabled)
- `POST /pow/verify` - <abbr title="Proof of Work">PoW</abbr> verification (sets js_verified cookie)
- `POST /cdp-report` - Client automation reports (<abbr title="JavaScript Object Notation">JSON</abbr>)
- `POST /fingerprint-report` - External/edge fingerprint intake (currently Akamai-only adapter shape, with internal <abbr title="Chrome DevTools Protocol">CDP</abbr> fallback for non-Akamai/legacy payloads)
- `POST <maze_path_prefix>checkpoint` - Maze traversal checkpoint submission
- `POST <maze_path_prefix>issue-links` - Maze progressive hidden-link issuance (signed seed + checkpoint gated)
- `GET <maze_assets_prefix>/maze.<hash>.min.css` - Shared maze stylesheet asset (immutable cache)
- `GET <maze_assets_prefix>/maze.<hash>.min.js` - Shared maze runtime script asset (immutable cache)
- `GET <maze_assets_prefix>/maze-worker.<hash>.min.js` - Maze worker asset (expansion + micro-<abbr title="Proof of Work">PoW</abbr> off-main-thread)
- `GET /robots.txt` - robots.txt (configurable)
- `GET /shuma/dashboard/...` - Dashboard static assets
- `GET /challenge/puzzle` - Dev-only puzzle challenge page (`shadow_mode=true` in runtime config)
- `POST /challenge/puzzle` - Puzzle challenge answer submission

Maze route note:
- `<maze_path_prefix>` is an opaque, deployment-specific prefix derived from maze secret material (for example `/_/<segment>/`).

### 🐙 Challenge Submission Format

`POST /challenge/puzzle` expects:
- `seed` (signed challenge seed)
- `output` (base-3 string, length 16 for 4x4 grids)

Output encoding:
- `0` = empty
- `1` = black cell
- `2` = pink cell

### 🐙 Maze Progressive Link Issuance

`POST <maze_path_prefix>issue-links` expects <abbr title="JavaScript Object Notation">JSON</abbr> fields:

- `parent_token` (current page `mt` token)
- `flow_id`, `entropy_nonce`, `path_prefix`
- `seed`, `seed_sig`, `hidden_count`, `segment_len`
- optional `requested_hidden_count` (must be <= signed hidden count)
- optional `candidates` (worker-generated candidate metadata)

Behavior:

- request is binding-validated against parent token (`ip_bucket`, `ua_bucket`, path prefix),
- expansion seed signature is verified before issuing links,
- parent-token link issuance is single-use; replayed issue-link requests return `409`,
- checkpoint posture is enforced before deep hidden issuance,
- response returns `{"links":[...]}` with signed child `mt` tokens (and optional `pow_difficulty`).

### 🐙 Challenge Seed Lifecycle

- Seeds are short-lived and single-use.
- Any submit attempt consumes the seed, including incorrect attempts.
- Re-submitting a consumed or expired seed returns `403 Expired`.
- Invalid or tampered seed/token data returns `403 Forbidden. Please request a new challenge.`

Challenge submit responses:
- `200` - Correct answer (`Thank you! Challenge complete.`)
- `403` - Incorrect answer (`Incorrect.` + `Request new challenge.` link)
- `403` - Expired/replay (`Expired` + `Request new challenge.` link)
- `403` - Invalid token/signature/<abbr title="Internet Protocol">IP</abbr> binding (`Forbidden. Please request a new challenge.` + link)

### 🐙 <abbr title="JavaScript">JS</abbr> Verification and <abbr title="Proof of Work">PoW</abbr> Flow

Normal routing can enforce a <abbr title="JavaScript">JS</abbr> verification gate before full access:

1. If `js_required_enforced=true` and the request has no valid `js_verified` cookie, the server returns an inline <abbr title="JavaScript">JS</abbr> verification interstitial for the requested path.
2. That interstitial performs <abbr title="Chrome DevTools Protocol">CDP</abbr> reporting (`POST /cdp-report`) as telemetry.
3. If `SHUMA_POW_ENABLED=true`, the interstitial solves <abbr title="Proof of Work">PoW</abbr> and submits `POST /pow/verify`. The user-facing `Verifying...` copy is intentionally delayed so fast solves stay visually invisible; it appears only when the solve takes longer than the short threshold baked into the interstitial.
4. `/pow/verify` validates the proof and returns `Set-Cookie: js_verified=...`.
5. After a valid `js_verified` cookie is set, the page reloads and the original route is retried.

If `SHUMA_POW_ENABLED=false`:

- the same interstitial still runs, but it sets `js_verified` directly in browser <abbr title="JavaScript">JS</abbr> and reloads.
- this is lower-friction but weaker than server-verified <abbr title="Proof of Work">PoW</abbr> issuance.

If `js_required_enforced=false`:

- normal routing does not send visitors through the <abbr title="JavaScript">JS</abbr> verification interstitial.
- `/pow` and `/pow/verify` still exist, but they are not part of the default access gate.

### 🐙 Health Check Example

When `SHUMA_HEALTH_SECRET` is unset:

```bash
curl -H "X-Forwarded-For: 127.0.0.1" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/shuma/health
```

When `SHUMA_HEALTH_SECRET` is set, you must also send:

```bash
curl -H "X-Forwarded-For: 127.0.0.1" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  -H "X-Shuma-Health-Secret: $SHUMA_HEALTH_SECRET" \
  http://127.0.0.1:3000/shuma/health
```

When `SHUMA_DEBUG_HEADERS=true`, the health response includes:
- `X-KV-Status` (available/unavailable)
- `X-Shuma-Fail-Mode` (open/closed)

## 🐙 Admin Endpoints

- `GET /shuma/admin` - <abbr title="Application Programming Interface">API</abbr> help
- `POST /shuma/admin/login` - Native dashboard login form endpoint; accepts `application/x-www-form-urlencoded` `password=<SHUMA_API_KEY>` plus optional `next=...`, sets the admin session cookie, and redirects
- `GET /shuma/admin/session` - Current auth/session state
- `POST /shuma/admin/logout` - Clear admin session cookie
- `GET /shuma/admin/ban` - List active bans. Under strict external ban-store outage posture, this returns `503` instead of serving local-only fallback state when authoritative reads are unavailable.
- `POST /shuma/admin/ban` - Ban an <abbr title="Internet Protocol">IP</abbr> (<abbr title="JavaScript Object Notation">JSON</abbr> body: `{"ip":"x.x.x.x","duration":3600}`; reason is always `manual_ban`; `duration` is optional and defaults to `ban_durations.admin`). Under strict external outage posture, this returns `503` instead of claiming success when external sync fails.
- `POST /shuma/admin/unban?ip=x.x.x.x` - Unban an <abbr title="Internet Protocol">IP</abbr>. Under strict external outage posture, this returns `503` instead of claiming success when external sync fails.
- `GET /shuma/admin/analytics` - Ban/event statistics plus explicit ban-store availability markers (`ban_store_status`, `ban_store_message`).
- `GET /shuma/admin/events?hours=N` - Recent events + summary stats (simulation rows are included when present and tagged per row). Default view is pseudonymized; forensic raw view requires `forensic=1&forensic_ack=I_UNDERSTAND_FORENSIC`.
- `GET /shuma/admin/cdp/events?hours=N&limit=M` - <abbr title="Chrome DevTools Protocol">CDP</abbr>-only detections/auto-bans (time-windowed, limit configurable). Default view is pseudonymized; forensic raw view requires `forensic=1&forensic_ack=I_UNDERSTAND_FORENSIC`.
- `GET /shuma/admin/operator-snapshot` - Read-only machine-first `operator_snapshot_v1` contract for later controller loops and Monitoring projection work. Returns persisted `operator_objectives_v1` metadata including objective revision, category-aware `category_postures`, live-vs-shadow-vs-adversary-sim sections, the seeded canonical `non_human_traffic` taxonomy basis plus category-classification readiness, the decision chain from fingerprinting to categorization to cumulative abuse score to posture severity, the bounded `non_human_coverage_v1` summary showing which mapped categories are currently `covered`, `partial`, `stale`, `unavailable`, or explicit gaps, bounded live and adversary-sim category receipts, and adversary-sim recent-run evidence that now normalizes full-spectrum Scrapling mode telemetry into observed fulfillment modes, observed category ids, and owned-surface receipts for receipt-backed coverage. Live verified identity now projects faithfully through that same taxonomy path, so verified `search`, `training`, `preview`, `service_agent`, and `user_triggered_agent` traffic no longer collapses into only coarse beneficial buckets in the machine-first snapshot. The typed verified-identity summary now also includes:
  - bounded `taxonomy_alignment` receipts and counts so the operator surface can tell whether verified categories aligned cleanly with the canonical taxonomy, fell back through `other`, or currently lack corroborating live classification evidence,
  - and `effective_non_human_policy`, which exposes the resolved policy profile, objective revision, verified-identity override mode, and per-category authority rows that now replace the old split verified-identity stance model.
  Live and adversary-sim traffic sections now also carry bounded `forwarded_upstream_latency_ms_total` totals, and `budget_distance` now includes the host-impact proxy `suspicious_forwarded_latency_share` when current telemetry supports it. The snapshot still returns `allowed_actions_v1`, the canonical `recursive_improvement_game_contract_v1` surface naming immutable rules, legal move ring, safety gates, regression anchors, and the explicit evaluator scorecard partitioning over:
  - `optimization_targets`
  - `hard_guardrails`
  - `regression_inputs`
  - `diagnostic_contexts`
  - `comparison_contract`
  It also returns:
  - budget-distance rows,
  - a bounded `recent_changes` ledger with decision IDs, objective revision references, expected-impact summaries, watch-window status, and compact evidence references,
  - a bounded `episode_archive` with completed episode context, baseline scorecards, proposed moves, retain or rollback outcomes, benchmark deltas, hard-guardrail triggers, and conservative homeostasis summary state,
  - nested `benchmark_results_v1` including explicit `tuning_eligibility` blockers, the canonical `non_human_category_posture` family, and the effective `protected_evidence` summary used by the controller,
  - and a bounded `replay_promotion` summary that now remains replay-lineage provenance rather than the only protected-evidence basis.
  `operator_objectives_v1` remains the rule surface for the loop and must never be controller-mutable even though it is projected alongside the controller-facing config envelope. This endpoint does not repair or rebuild documents on read; if the hot-read document has not been materialized yet it returns `503` with `error=operator_snapshot_not_materialized`.
- `GET /shuma/admin/operator-objectives` - Read the persisted `operator_objectives_v1` contract that the operator snapshot and later reconcile loop use as the site-owned objective truth. If the site has not stored objectives yet, this endpoint seeds the conservative default profile and returns it.
- `POST /shuma/admin/operator-objectives` - Replace the persisted operator-objectives document. Accepts a bounded objective payload, including canonical `category_postures` rows keyed by seeded non-human category ids, validates it, persists a server-assigned revision, records a causal decision-ledger row plus recent-change summary, and refreshes the hot-read snapshot. This endpoint is disabled when `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false`.
- `POST /shuma/admin/oversight/reconcile` - Run one bounded oversight preview cycle over the already-materialized machine-first snapshot. Returns the reconcile result, config-validation outcome for any proposed patch, and an explicit `apply` block that tells the operator whether the candidate is merely refused, preview-eligible for canary apply, or blocked by missing evidence. The reconcile payload now carries explicit shortfall-attribution semantics:
  - `problem_class` such as `likely_human_friction_overspend`, `suspicious_forwarded_reach_overspend`, or `suspicious_forwarded_latency_overspend`
  - `guidance_status` such as `observe_longer`, `bounded_family_guidance`, `code_evolution_only`, or `exact_bounded_move`
  - `tractability` such as `not_actionable_yet`, `family_level_policy_choice`, `code_or_capability_gap`, or `exact_bounded_config_move`
  The preview still fails closed when the verified-identity calibration guardrails show likely harm to tolerated or allowed verified traffic, including degraded taxonomy alignment, verified-botness conflict pressure, or user-triggered-agent friction mismatch. This endpoint must remain preview-only: it never writes runtime config even when `rollout_guardrails.automated_apply_status=canary_only`, and if `operator_snapshot_v1` is missing it returns an explicit `insufficient_evidence` decision instead of mutating or repairing state on read.
- `GET /shuma/admin/oversight/history` - Read the bounded persisted `oversight_decision_ledger_v1` history used by the first closed config loop. Returns the same canonical `recursive_improvement_game_contract_v1` summary projected by `operator_snapshot_v1`, the same bounded `episode_archive` plus homeostasis summary used by later recursive-improvement memory, and recent preview/apply/watch/rollback decisions, including trigger source, benchmark context, explicit shortfall-attribution fields (`problem_class`, `guidance_status`, `tractability`), validation status, refusal reasons, compact evidence references, and the persisted `apply.stage` lineage (`eligible`, `canary_applied`, `watch_window_open`, `improved`, `refused`, or `rollback_applied`).
- `GET /shuma/admin/oversight/agent/status` - Read the bounded shared-host agent status projection for the first bounded canary-apply loop. Returns the execution-boundary contract, periodic and post-sim trigger semantics, the latest agent run, recent bounded run history, the latest linked oversight decision row, and the same bounded `episode_archive` plus homeostasis summary so later loop projections can consume one machine-first episode-memory surface instead of inventing a second local history model.
- `GET /shuma/admin/replay-promotion` - Read the bounded persisted `replay_promotion_v1` contract materialized from the adversarial promotion lane. Returns the persisted frontier metadata, hybrid governance status, discovery-quality metrics, summary counts, and bounded replay-candidate lineage rows. This endpoint remains replay provenance; the effective controller-facing protected-evidence summary is exposed separately through `benchmark_results_v1.protected_evidence`. If promotion lineage has not been materialized yet, this endpoint returns `503` with `error=replay_promotion_not_materialized`.
- `POST /shuma/admin/replay-promotion` - Materialize a bounded backend replay-promotion contract from the promotion triage lane output. Accepts the current `adversarial-promotion.v1` payload from `scripts/tests/adversarial_promote_candidates.py`, persists the bounded control-plane view, and refreshes `operator_snapshot_v1` so later reconcile and agent reads can consume replay lineage without parsing sidecar artifacts directly.
- `GET /shuma/admin/benchmark-suite` - Read-only machine-first `benchmark_suite_v1` registry. Returns the bounded benchmark family contract for suspicious-origin cost, likely-human friction, representative adversary effectiveness, beneficial non-human posture, and canonical `non_human_category_posture` alignment, along with supported comparison modes, subject kinds, and benchmark-driven escalation boundaries. This is a static backend-owned contract and does not depend on hot-read materialization.
- `GET /shuma/admin/benchmark-results` - Read-only machine-first `benchmark_results_v1` envelope. Returns the same bounded current-instance benchmark contract that is nested inside `operator_snapshot_v1`, including subject kind, watch window, baseline-reference availability, `improvement_status`, prior-window comparison metadata (`baseline_reference.subject_kind`, `baseline_reference.generated_at`, per-family `baseline_status` and `comparison_status`, per-metric `baseline_current`, `comparison_delta`, and `comparison_status`), exactness or capability metadata, the effective `protected_evidence` summary used by later controller loops, the bounded replay-lineage provenance summary `replay_promotion`, the current non-human classification readiness block, and the bounded `non_human_coverage_v1` summary used to decide whether mapped categories are covered well enough for tuning. That coverage summary now includes receipt-backed full-spectrum Scrapling category proof for `indexing_bot`, `ai_scraper_bot`, `automated_browser`, and `http_agent` when recent sim telemetry has actually observed those personas. The `suspicious_origin_cost` family now also carries the host-impact proxy `suspicious_forwarded_latency_share` plus the tracking metric `suspicious_average_forward_latency_ms`, and prior-window comparison treats both as lower-is-better instead of inventing a second cost family. The beneficial non-human benchmark family now also surfaces explicit verified-identity calibration metrics, including `taxonomy_alignment_mismatch_rate`, `verified_botness_conflict_rate`, and `user_triggered_agent_friction_mismatch_rate`, with `insufficient_evidence` behavior when the protected verified sample is too small. The endpoint returns the explicit `tuning_eligibility` status and blockers used later by canary-apply logic, including verified-identity no-harm guardrails when those metrics or the snapshot alignment summary show the controller is drifting against tolerated or allowed verified traffic. It also returns the per-category `non_human_category_posture` family keyed to the persisted operator posture rows and an explicit review-aware `escalation_hint` containing:
  - the current benchmark decision (`config_tuning_candidate`, `observe_longer`, or `code_evolution_candidate`)
  - `problem_class`, `guidance_status`, `tractability`, and `expected_direction`
  - trigger families and exact trigger metrics
  - matching config-action families plus bounded family risk guidance
  - blockers when evidence is not yet actionable
When non-human classification, protected evidence, mapped category coverage, or verified-identity guardrails are not yet ready, tuning eligibility is `blocked` and escalation fails closed to `observe_longer` with explicit blockers rather than silently proposing tuning. Strong live Scrapling runtime board evidence may now satisfy `protected_evidence` without replay-promotion materialization, but raw synthetic traffic and advisory frontier or LLM discovery remain ineligible. The current endpoint materializes prior-window comparison from the last snapshot; the same comparison contract is intended for later baseline and candidate subjects. This endpoint does not materialize snapshots on read; if the operator snapshot document is missing it returns `503` with `error=benchmark_results_snapshot_missing`.
- `GET /shuma/admin/monitoring?hours=N&limit=M` - Consolidated monitoring summaries plus dashboard-native detail payload for Traffic, Game Loop, and Diagnostics refreshes, including ban-store availability markers for `details.analytics` and `details.bans`. Default view is pseudonymized; forensic raw view requires `forensic=1&forensic_ack=I_UNDERSTAND_FORENSIC`.
- `GET /shuma/admin/monitoring/delta?after_cursor=...&limit=N&hours=M` - Cursor-ordered monitoring event deltas (`next_cursor`, `has_more`, `overflow`) with `ETag`/`If-None-Match` support and freshness/load-envelope metadata. Default view is pseudonymized; forensic raw view requires `forensic=1&forensic_ack=I_UNDERSTAND_FORENSIC`.
- `GET /shuma/admin/monitoring/stream?after_cursor=...&limit=N&hours=M` - One-shot <abbr title="Server-Sent Events">SSE</abbr> monitoring delta (`text/event-stream`) with `Last-Event-ID` resume using the same cursor namespace. Default view is pseudonymized; forensic raw view requires `forensic=1&forensic_ack=I_UNDERSTAND_FORENSIC`.
- `GET /shuma/admin/ip-bans/delta?after_cursor=...&limit=N&hours=M` - Cursor-ordered ban/unban deltas plus active-ban snapshot (`active_bans`) and explicit active-ban availability markers (`active_bans_status`, `active_bans_message`) with `ETag`/`If-None-Match` support and freshness/load-envelope metadata. Default view is pseudonymized; forensic raw view requires `forensic=1&forensic_ack=I_UNDERSTAND_FORENSIC`.
- `GET /shuma/admin/ip-bans/stream?after_cursor=...&limit=N&hours=M` - One-shot <abbr title="Server-Sent Events">SSE</abbr> IP-ban delta (`text/event-stream`) with `Last-Event-ID` resume using the same cursor namespace, including `active_bans`, `active_bans_status`, and `active_bans_message`. Default view is pseudonymized; forensic raw view requires `forensic=1&forensic_ack=I_UNDERSTAND_FORENSIC`.
- `GET /shuma/admin/ip-range/suggestions?hours=N&limit=M` - Suggested IP-range candidates with collateral-risk scoring
- `GET /shuma/admin/config` - Read configuration envelope
  - `config` contains writable persisted admin settings.
  - `runtime` contains read-only operational overlays such as `gateway_deployment_profile`, `akamai_edge_available`, `adversary_sim_available`, and the effective `adversary_sim_enabled` state used for dashboard rendering; `SHUMA_ADVERSARY_SIM_ENABLED` seeds only the initial desired state, and once lifecycle state exists the runtime overlay is projected from `ControlState.desired_enabled` rather than from a separate runtime override writer. Lifecycle writes still go through `POST /shuma/admin/adversary-sim/control`.
- `POST /shuma/admin/config` - Update configuration (partial <abbr title="JavaScript Object Notation">JSON</abbr>, disabled when `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false`)
- `POST /shuma/admin/config/validate` - Validate a config patch without persisting changes (returns `{ valid, issues[] }` with field/expected/received hints when invalid)
- `GET /shuma/admin/config/export` - Export non-secret runtime config as deploy-ready env key/value output

Controller mutability note:

- Admin writability is broader than controller eligibility. Shuma classifies writable config into `never`, `manual_only`, and `controller_tunable`, and the controller must never treat `operator_objectives_v1` or the hard security or trust-boundary config surface as legal moves.
- `POST /shuma/admin/adversary-sim/control` - Explicit adversary-sim lifecycle command submission (`{"enabled":true|false,"lane":"synthetic_traffic|scrapling_traffic|bot_red_team","reason":"optional"}` with `lane` optional), admin-auth + CSRF protected, strict same-origin/fetch-metadata checks, and required `Idempotency-Key` header
- `GET /shuma/admin/adversary-sim/status` - Adversary-sim lifecycle status read path, including desired vs actual state, active runtime-lane routing fields (`desired_lane`, `active_lane`, `lane_switch_seq`, `last_lane_switch_at`, `last_lane_switch_reason`), live `lane_diagnostics` counters for the selected lane, generation and lane-diagnostics `truth_basis` markers, bounded `persisted_event_evidence` when the status surface recovered lower-bound truth from persisted simulation-tagged monitoring events, and controller lease metadata. Legacy `active_lane_count` plus `lanes.{deterministic,containerized}` remain during the migration. This endpoint is read-only: it reports stale persisted state via `controller_reconciliation_required` and does not reconcile or persist state as part of the read.
- `POST /shuma/admin/adversary-sim/history/cleanup` - Explicitly clear retained telemetry history (`eventlog:v2:*`, `monitoring:v1:*`, and derived monitoring detail counters) without changing adversary-sim control state.
  - In `runtime-dev`: endpoint is available without extra cleanup acknowledgement.
  - In `runtime-prod`: endpoint requires header `X-Shuma-Telemetry-Cleanup-Ack: I_UNDERSTAND_TELEMETRY_CLEANUP`.
- `GET /shuma/admin/maze` - maze stats
- `GET /shuma/admin/maze/preview?path=<maze_entry_path>...` - Non-operational maze preview (admin-auth only; no live traversal token issuance)
- `GET /shuma/admin/maze/seeds` - Maze operator-seed source list and cached corpus snapshot
- `POST /shuma/admin/maze/seeds` - Upsert maze operator-seed sources
- `POST /shuma/admin/maze/seeds/refresh` - Trigger manual maze operator-corpus refresh
- `GET /shuma/admin/robots` - robots.txt config and preview
- `POST /shuma/admin/robots/preview` - robots.txt preview from an unsaved config patch (does not persist)
- `GET /shuma/admin/cdp` - <abbr title="Chrome DevTools Protocol">CDP</abbr> + fingerprint detection config and stats

Internal supervisor-only adversary-sim endpoints:

- `POST /shuma/internal/adversary-sim/beat` - Internal supervisor heartbeat endpoint that returns lane dispatch mode and, when `active_lane=scrapling_traffic`, the bounded Scrapling worker plan for the next beat.
- `POST /shuma/internal/adversary-sim/worker-result` - Internal supervisor write path for bounded Scrapling worker results. This path is internal-supervisor authenticated, rejects stale or off-state worker results with `409 stale_worker_result`, and never accepts dashboard/client traffic.

Hosted Scrapling worker deployment boundary:

- shared-host `ssh_systemd` deployments are the current supported full hosted worker path,
- Fermyon/Akamai edge deploys can still expose lifecycle and control-plane truth, but they do not imply a supported full hosted worker runtime,
- and the admin API does not accept a deploy-time surface catalog for lane routing; deploy-time scope/seed artifacts stay outside the API surface and traversal telemetry remains the authoritative reachable-surface map.

`GET /shuma/admin/session` includes `access` as `read_only`, `read_write`, or `none`.

Expensive admin read endpoints (`/shuma/admin/events`, `/shuma/admin/cdp/events`, `/shuma/admin/operator-snapshot`, `/shuma/admin/monitoring`, `/shuma/admin/monitoring/delta`, `/shuma/admin/monitoring/stream`, `/shuma/admin/ip-bans/delta`, `/shuma/admin/ip-bans/stream`, `/shuma/admin/ip-range/suggestions`, `/shuma/admin/ban` `GET`) are rate-limited to reduce <abbr title="Key-Value">KV</abbr>/<abbr title="Central Processing Unit">CPU</abbr> abuse amplification (`429` with `Retry-After: 60` when limited).

Simulation telemetry uses per-row metadata tags (`sim_run_id`, `sim_profile`, `sim_lane`, `is_simulation`) rather than read-time query toggles.
Deprecated simulation-namespace config keys are rejected on write (`sim_telemetry_namespace` and related unknown namespace-era fields).
Security/privacy controls are enforced by default:
- persistence classification contract (`public|internal|sensitive|secret-prohibited`) is applied before write,
- secret-like fields are scrubbed with explicit redaction markers,
- secret-canary matches are fail-closed (event is dropped, incident state emitted),
- non-forensic admin views pseudonymize sensitive identifiers by default.

Adversary-sim command contract (`adversary-sim-control.v1`) highlights:

- `POST /shuma/admin/adversary-sim/control` always returns an `operation_id` and `decision` (`accepted` or `replayed` on `200`).
- Exact idempotent retries (`Idempotency-Key` + same canonical payload, including lane when provided) replay the original operation and keep `operation_id` stable.
- Reusing an idempotency key with a different payload returns `409`.
- Production adversary-sim is a normal operating path in `runtime-prod`, not a gated exception. A truthful operating receipt keeps one off-state `GET /shuma/admin/adversary-sim/status` posture snapshot, one accepted ON `operation_id`, the no-impact proof from `make test-adversary-sim-runtime-surface` against the running target, and one accepted OFF `operation_id`.
- Control responses include requested and accepted lane metadata:
  - `requested_state.enabled`
  - `requested_state.lane` (`null` when omitted)
  - `accepted_state.desired_enabled`
  - `accepted_state.desired_lane`
  - `accepted_state.actual_phase`
  - `accepted_state.active_lane`
- Status responses include:
  - `desired_state` (`running|off`)
  - `actual_state` (`running|stopping|off`)
  - `desired_lane` (`synthetic_traffic|scrapling_traffic|bot_red_team`)
  - `active_lane` (`synthetic_traffic|scrapling_traffic|bot_red_team|null`)
  - `lane_switch_seq`
  - `last_lane_switch_at`
  - `last_lane_switch_reason`
  - `lane_diagnostics.lanes.<lane>.beat_attempts|beat_successes|beat_failures`
  - `lane_diagnostics.lanes.<lane>.generated_requests|blocked_requests|offsite_requests|response_bytes`
  - `lane_diagnostics.lanes.<lane>.response_status_count`
  - `lane_diagnostics.lanes.<lane>.last_generated_at|last_error`
  - `lane_diagnostics.request_failure_classes.<class>.count|last_seen_at`
  - `generation_active` (`true|false`; producer lifecycle only)
  - `historical_data_visible` (`true` when retained telemetry remains queryable regardless of producer state)
  - `history_retention.retention_hours`
  - `history_retention.retention_health.state`
  - `history_retention.retention_health.purge_lag_hours`
  - `history_retention.retention_health.pending_expired_buckets`
  - `history_retention.retention_health.last_error`
  - `history_retention.cleanup_endpoint`
  - `history_retention.cleanup_command`
  - `controller_reconciliation_required` (`true|false`)
  - `controller_lease` (owner/fencing metadata when held)

`POST /shuma/admin/adversary-sim/history/cleanup` response includes:
- `cleaned`
- `deleted_keys`
- `deleted_by_family`
- `retention_hours`
- `cleanup_command` (`make telemetry-clean`)

`GET /shuma/admin/maze/preview` is intentionally non-operational:
- links recurse only into `/shuma/admin/maze/preview`,
- live `mt` traversal tokens are not emitted,
- hidden covert-decoy tracking markers/links are not emitted,
- maze replay/checkpoint/budget/risk counters are not mutated.

### 🐙 Ban Responses

`GET /shuma/admin/ban` returns:
- `bans`
- `status`
- `message`

When `SHUMA_PROVIDER_BAN_STORE=external` is paired with strict outage posture (`SHUMA_BAN_STORE_OUTAGE_MODE=fail_open` or `fail_closed`) and the authoritative backend is unavailable, `GET /shuma/admin/ban` returns `503` with:
- `Ban store unavailable: strict outage posture requires authoritative backend access for ban-state reads`

`POST /shuma/admin/ban` returns:
- `{"status":"banned","ip":"x.x.x.x"}` on success

`POST /shuma/admin/unban?ip=x.x.x.x` returns:
- `{"status":"unbanned","ip":"x.x.x.x"}` on success

Under strict outage posture, manual ban and unban writes return `503` instead of claiming success when the external backend cannot be synchronized.

### 🐙 Analytics Response

`GET /shuma/admin/analytics` returns:
- `ban_count` (nullable when authoritative active-ban reads are unavailable)
- `ban_store_status` (`available|unavailable`)
- `ban_store_message`
- `shadow_mode`
- `fail_mode`

### 🐙 Admin Events Response

`GET /shuma/admin/events?hours=24` returns:
- `recent_events` (up to 100 events)
- `event_counts` (counts per event type)
- `top_ips` (top 10 IPs by event count)
- `unique_ips` (distinct <abbr title="Internet Protocol">IP</abbr> count)
- `security_mode` (`pseudonymized_default|forensic_raw`)
- `security_privacy` (classification/scrub/incidence/retention-tier state)

Each event row includes the canonical event fields plus simulation metadata when available:
- `sim_run_id` (optional)
- `sim_profile` (optional)
- `sim_lane` (optional)
- `is_simulation` (`true` for simulation-tagged rows)

For <abbr title="Chrome DevTools Protocol">CDP</abbr>-only operational views without the 100-row mixed-event cap, use:

`GET /shuma/admin/cdp/events?hours=24&limit=500` returns:
- `events` (<abbr title="Chrome DevTools Protocol">CDP</abbr> detection and <abbr title="Chrome DevTools Protocol">CDP</abbr> auto-ban events only, up to `limit`)
- `hours` (effective query window)
- `limit` (effective result cap)
- `total_matches` (number of matched <abbr title="Chrome DevTools Protocol">CDP</abbr> events before truncation)
- `counts.detections` (<abbr title="Chrome DevTools Protocol">CDP</abbr> detection event count in the window)
- `counts.auto_bans` (<abbr title="Chrome DevTools Protocol">CDP</abbr> auto-ban event count in the window)

### 🐙 Admin Monitoring Summary Response

`GET /shuma/admin/monitoring?hours=24&limit=10` returns:
- `summary.generated_at`
- `summary.hours`
- `summary.honeypot`:
- `total_hits`, `unique_crawlers`, `top_crawlers`, `top_paths`
- `summary.challenge`:
- `total_failures`, `unique_offenders`, `top_offenders`, `reasons`, `trend`
- `summary.pow`:
- `total_failures`, `total_successes`, `total_attempts`, `success_ratio`
- `unique_offenders`, `top_offenders`, `reasons`, `outcomes`, `trend`
- `summary.rate`:
- `total_violations`, `unique_offenders`, `top_offenders`, `top_paths`, `outcomes`
- `summary.geo`:
- `total_violations`, `actions`, `top_countries`
- `summary.request_outcomes.by_scope|by_lane|by_non_human_category` now also include bounded `forwarded_upstream_latency_ms_total` counters for forwarded traffic
- `freshness_slo` (`p50/p95/p99` visibility-delay targets plus lag/degraded thresholds)
- `load_envelope` (declared ingest/client/query budget envelope for realtime contract)
- `freshness` (`state=fresh|degraded|stale`, `lag_ms`, `last_event_ts`, slow-consumer lag taxonomy, transport)
- `retention_health`:
- `retention_hours`, `oldest_retained_ts`, `purge_lag_hours`, `pending_expired_buckets`
- `last_purge_success_ts`, `last_attempt_ts`, `last_purged_bucket`, `last_error`
- `state` (`healthy|degraded|stalled`), `guidance`, `bucket_schema`
- `details.cost_governance`:
- `cost_envelope_profiles.runtime_dev|runtime_prod` (`ingest_events_per_second`, `query_calls_per_second_per_client`, `payload_p95_kb`, `guarded_dimension_cardinality_cap_per_hour`, `compression_min_percent_for_payloads_over_64kb`)
- `guarded_dimension_cardinality_cap_per_hour`, `observed_guarded_dimension_cardinality_max`, `overflow_bucket_accounted`, `overflow_bucket_count`, `cardinality_pressure`
- `rollups` (`1m`, `5m`, `1h`, `raw_event_lineage_source`)
- `unsampleable_event_classes`, `unsampleable_event_drop_count`, `sampling_status`
- `sampling` (`eligible_low_risk_classes`, `sampled_count`, `unsampled_count`)
- `payload_budget` (`p95_max_kb`, `estimated_current_payload_kb`, `status`) and `payload_budget_status`
- `compression` (`status`, `negotiated`, `algorithm`, `input_bytes`, `output_bytes`, `reduction_percent`, `min_percent`)
- `query_budget` (`cost_units`, `cost_class`, `avg_req_per_sec_client_target`, `max_req_per_sec_client`, `status`, `estimated_bucket_count`, `estimated_keys_visited`, `response_event_rows`, `bucket_density`, `density_penalty_units`, `residual_scan_keys`) and `query_budget_status`
- `degraded_state` (`normal|degraded`) and `degraded_reasons`
- `read_surface` (`monitoring_buckets`, `monitoring_keys`, `rollup_buckets`, `rollup_keys`, `eventlog_buckets`, `eventlog_keys`, `detail_catalog_keys`, `residual_scan_keys`)
- `security_privacy`:
- `classification` (`version`, `field_classification_enforced`, schema mapping)
- `sanitization` (`secret_scrub_actions_total`, `secret_canary_leak_count`, `secret_canary_detected_total`)
- `access_control` (`view_mode`, pseudonymization coverage/required percent, forensic break-glass state)
- `retention_tiers` (`high_risk_raw_artifacts_hours`, `high_risk_raw_artifacts_max_hours`, `redacted_summary_hours`, `override_requested`, `override_audit_entry`)
- `incident_response` (`incident_hook_emitted`, `incident_hook_emitted_total`, workflow stage state, last violation/incident payload)

When payload size is greater than `64KB` and the client sends `Accept-Encoding: gzip`, the endpoint may return compressed JSON (`Content-Encoding: gzip`) with `Vary: Accept-Encoding`.
Cost-state response headers are also emitted:
- `X-Shuma-Monitoring-Cost-State`
- `X-Shuma-Monitoring-Query-Budget`
- `X-Shuma-Monitoring-Security-Mode` (`pseudonymized_default|forensic_raw`)

When authoritative active-ban reads are unavailable under strict outage posture:
- `details.analytics.ban_count` is `null`
- `details.analytics.ban_store_status` is `unavailable`
- `details.analytics.ban_store_message` explains the outage contract
- `details.bans.status` is `unavailable`
- `details.bans.message` explains the outage contract
- `details.maze.maze_auto_bans` is `null`

`GET /shuma/admin/monitoring/delta?after_cursor=<cursor>&limit=100&hours=24` returns:
- `cursor_contract` (version + ordering + overflow taxonomy)
- `freshness_slo` (`p50/p95/p99` visibility-delay targets plus lag/degraded thresholds)
- `load_envelope` (declared ingest/client/query budget envelope for realtime contract)
- `after_cursor` (echo)
- `window_end_cursor` (latest cursor currently visible in window; use to initialize tail-following)
- `next_cursor` (resume token)
- `has_more` (`true|false`)
- `overflow` (`none|limit_exceeded`)
- `events` (event rows with per-row `cursor`)
- `freshness` (`state=fresh|degraded|stale`, `lag_ms`, `last_event_ts`, slow-consumer lag taxonomy, transport)
- `stream_supported` and `stream_endpoint`

`GET /shuma/admin/ip-bans/delta?after_cursor=<cursor>&limit=100&hours=24` returns:
- same cursor fields as monitoring delta
- `events` filtered to `Ban` and `Unban`
- `active_bans` snapshot for current ban state reconciliation
- `active_bans_status` (`available|unavailable`)
- `active_bans_message`

`GET /shuma/admin/monitoring/stream?after_cursor=<cursor>&limit=100&hours=24` and `GET /shuma/admin/ip-bans/stream?...` return one <abbr title="Server-Sent Events">SSE</abbr> frame per request with:
- `event: monitoring_delta` or `event: ip_bans_delta`
- `id: <next_cursor>` (resume token for `Last-Event-ID`)
- `data: <JSON payload>` (same cursor/freshness contract as delta, plus `stream_contract`; for `ip_bans_delta` this also includes `active_bans_status` and `active_bans_message`)

The stream path is intentionally one-shot in this phase; the browser reconnect loop provides bounded fan-out/backpressure while preserving deterministic cursor ordering.
- `prometheus`:
- `endpoint` (`/shuma/metrics`), helper notes, and scrape examples for external platforms
- `details` (dashboard Monitoring-tab refresh contract):
- `retention_health`: same lifecycle contract as top-level (`state/guidance/lag/pending/error`)
- `analytics`: `ban_count`, `ban_store_status`, `ban_store_message`, `shadow_mode`, `fail_mode`
- `events`: `recent_events`, `event_counts`, `top_ips`, `unique_ips`, `recent_events_window` (`hours`, `requested_limit`, `applied_recent_event_cap`, `total_events_in_window`, `returned_events`, `has_more`, `continue_via`, `response_shaping_reason`)
- `cost_governance`: same contract described above
- `bans`: `bans`, `status`, `message`
- `maze`: `total_hits`, `unique_crawlers`, `maze_auto_bans`, `deepest_crawler`, `top_crawlers`
- `cdp`: `config`, `stats`, `fingerprint_stats`
- `cdp_events`: `events`, `hours`, `limit`, `total_matches`, `counts`

### 🐙 IP Range Suggestions Response

`GET /shuma/admin/ip-range/suggestions?hours=24&limit=20` returns:
- `generated_at` (unix seconds)
- `hours` (effective window, clamped to `1..720`)
- `summary`: `suggestions_total`, `low_risk`, `medium_risk`, `high_risk`
- `suggestions[]`:
- `cidr`, `ip_family`
- `bot_evidence_score`, `human_evidence_score`
- `collateral_risk`, `confidence`, `risk_band`
- `recommended_action` (`deny_temp`, `tarpit`, `logging-only`)
- `recommended_mode` (`enforce`, `logging-only`)
- `evidence_counts` (signal/event counters used in the score)
- `safer_alternatives` (narrower CIDR candidates when high-collateral parent suggestions are split)
- `guardrail_notes` (explanations for suppression/split/clamp behavior)

Compact persisted event rows now use sparse omission for semantically absent or implied fields:

- `ip`, `reason`, `outcome`, `outcome_code`, `botness_score`, `admin`, `taxonomy`, `sim_*`, and execution metadata fields are omitted when absent.
- `ts` and `event` remain explicit in stored rows.
- `is_simulation` is stored only for simulation-tagged rows (`true`); non-simulation rows omit it.

Raw monitoring event rows may include:

- `taxonomy.level`
- `taxonomy.action` when the action is not intentionally omitted as derivable from `taxonomy.level`
- `taxonomy.detection` when the detection is not intentionally omitted as derivable from `reason`
- `taxonomy.signals[]` when the active signal set is not intentionally omitted as derivable from `reason`
- `outcome_code`
- `botness_score`

Challenge-heavy rows intentionally compact redundant taxonomy facts:

- `js_verification` rows keep `taxonomy.level` and `outcome_code`, but omit `taxonomy.action`, `taxonomy.detection`, and `taxonomy.signals[]` because the canonical `reason` already implies those facts.
- `botness_gate_*` rows omit `taxonomy.action` and the matching `taxonomy.detection` when those are implied by `taxonomy.level` plus `reason`, while still preserving `taxonomy.signals[]` when the signal set carries event-specific analysis value.

Legacy rows may still carry canonical taxonomy metadata inside `outcome` as:

- `taxonomy[level=L* action=A* detection=D* signals=S_*...]`

This remains the same event-log plus hot-read document architecture; no parallel telemetry storage or query path is introduced.

### 🐙 <abbr title="Chrome DevTools Protocol">CDP</abbr> + Fingerprint Admin View

`GET /shuma/admin/cdp` returns:
- `config`:
  - `enabled`, `auto_ban`, `detection_threshold`
  - `probe_family`, `probe_rollout_percent`
  - `fingerprint_signal_enabled`
  - `fingerprint_state_ttl_seconds`, `fingerprint_flow_window_seconds`, `fingerprint_flow_violation_threshold`
  - `fingerprint_pseudonymize`
  - `fingerprint_entropy_budget`
  - `fingerprint_family_cap_header_runtime`, `fingerprint_family_cap_transport`, `fingerprint_family_cap_temporal`, `fingerprint_family_cap_persistence`, `fingerprint_family_cap_behavior`
- `stats`:
  - `total_detections`, `auto_bans`
- `fingerprint_stats`:
  - `events`
  - `ua_client_hint_mismatch`
  - `ua_transport_mismatch`
  - `temporal_transition`
  - `flow_violation`
  - `persistence_marker_missing`
  - `untrusted_transport_header`

### 🐙 Canonical Escalation IDs

Policy telemetry and event outcomes use four stable <abbr title="Identifier">ID</abbr> classes:

- `L*` escalation level IDs (`L0_ALLOW_CLEAN` .. `L11_DENY_HARD`)
- `A*` action IDs (`A_ALLOW`, `A_VERIFY_JS`, `A_CHALLENGE_STRONG`, `A_DENY_TEMP`, ...)
- `D*` detection IDs (stable detection taxonomy for matched paths/signals)
- `S_*` signal IDs (canonical signal taxonomy)

<abbr title="JavaScript">JS</abbr>/browser signal note:

- `S_JS_REQUIRED_MISSING` means the request did not include a valid `js_verified` marker while <abbr title="JavaScript">JS</abbr> enforcement is enabled (missing/expired/invalid marker).
- This signal can be used as botness evidence and can also be the direct trigger for `L4_VERIFY_JS`.

### 🐙 Config Export Response

`GET /shuma/admin/config/export` returns:
- `format` (`env`)
- `site_id`
- `generated_at` (unix seconds)
- `env` (non-secret `SHUMA_*` values as strings)
- `env_text` (newline-delimited `KEY=value` export)
- `excluded_secrets` (secret keys intentionally omitted, including Redis provider URLs)

### 🐙 Example: List Bans

```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  http://127.0.0.1:3000/shuma/admin/ban
```

Each ban entry includes:
- `ip`
- `reason`
- `banned_at` (unix seconds)
- `expires` (unix seconds)
- `fingerprint` (optional):
- `score` (0-10 or null)
- `signals` (array of triggering signal keys)
- `summary` (human-readable context)

### 🐙 Example: Ban an <abbr title="Internet Protocol">IP</abbr>

```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"1.2.3.4","duration":3600}' \
  http://127.0.0.1:3000/shuma/admin/ban
```

### 🐙 Example: Fetch Events

```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  http://127.0.0.1:3000/shuma/admin/events?hours=24
```

## 🐙 Botness Policy Fields (`/shuma/admin/config`)

The unified botness model uses weighted scored signals plus terminal hard-ban signals. Most fields below are writable via `POST /shuma/admin/config`; read-only runtime overlays are called out explicitly.

Core enforcement fields:
- `js_required_enforced` - enable/disable <abbr title="JavaScript">JS</abbr>-required enforcement
- `rate_limit` - per-minute request limit used for hard rate limiting and rate-pressure scoring, applied per source IP bucket (IPv4 /24, IPv6 /64)
- `honeypot_enabled` - enable/disable honeypot trap handling for configured trap paths
- `runtime.adversary_sim_enabled` - read-only effective adversary-sim desired state surfaced in config/status payloads for dashboard/runtime rendering; it uses `SHUMA_ADVERSARY_SIM_ENABLED` only as the initial seed, then projects from persisted lifecycle control state after the first control write. Change it via `POST /shuma/admin/adversary-sim/control`, not `POST /shuma/admin/config`
- `adversary_sim_duration_seconds` - adversary-sim run-window duration for backend autonomous supervisor generation (bounded `30..900`)
- `challenge_puzzle_enabled` - enable/disable challenge serving at challenge-tier routes (when disabled, challenge tier falls back to maze or block)
- `defence_modes.rate` / `defence_modes.geo` / `defence_modes.js` - per-module composability mode (`off`, `signal`, `enforce`, `both`)

Scored thresholds:
- `not_a_bot_risk_threshold` - score at/above which not-a-bot is served (when enabled)
- `challenge_puzzle_risk_threshold` - score at/above which challenge is served
- `botness_maze_threshold` - score at/above which requests are routed to maze

Not-a-Bot controls:
- `not_a_bot_enabled`
- `not_a_bot_pass_score`
- `not_a_bot_fail_score`
- `not_a_bot_nonce_ttl_seconds` - Verification Token Lifetime (seconds): how long the signed Not-a-Bot token remains valid after page load. If it expires before submit, verification fails.
- `not_a_bot_marker_ttl_seconds` - Pass Marker Lifetime (seconds): how long a successful Not-a-Bot pass is remembered for the same IP/UA bucket, so repeat requests can skip this step.
- `not_a_bot_attempt_limit_per_window`
- `not_a_bot_attempt_window_seconds`

Scored weights:
- `botness_weights.js_required`
- `botness_weights.geo_risk`
- `botness_weights.rate_medium`
- `botness_weights.rate_high`
- `botness_weights.maze_behavior`

Mutability:
- Runtime config mutation is controlled globally by `SHUMA_ADMIN_CONFIG_WRITE_ENABLED`.
- When `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false`, `POST /shuma/admin/config` returns `403`.
- `POST /shuma/admin/config/validate` runs the same server-side validators as `POST /shuma/admin/config` but does not write <abbr title="Key-Value">KV</abbr> state.

Effective-mode visibility:
- `defence_modes_effective` reports runtime-effective signal/action booleans per module.
- `defence_mode_warnings` reports mode conflicts (for example <abbr title="JavaScript">JS</abbr> mode overridden by `js_required_enforced=false`).
- Frontier readiness (read-only, env-derived):
  - `frontier_mode` (`disabled`, `single_provider_self_play`, `multi_provider_playoff`)
  - `frontier_provider_count`
  - `frontier_diversity_confidence` (`none`, `low`, `higher`)
  - `frontier_reduced_diversity_warning`
  - `frontier_providers` (provider/model/configured summary; no API keys)
- Enterprise state posture visibility:
  - `enterprise_multi_instance`
  - `enterprise_unsynced_state_exception_confirmed`
  - `enterprise_state_guardrail_warnings`
  - `enterprise_state_guardrail_error`
- Invalid `defence_modes` keys or invalid mode values are rejected by `POST /shuma/admin/config` with `400`.

Signal catalog:
- `botness_signal_definitions.scored_signals` lists weighted contributors.
- `botness_signal_definitions.terminal_signals` lists immediate actions that bypass scoring.

## 🐙 Akamai Bot Signal Fields (`/shuma/admin/config`)

- `provider_backends.fingerprint_signal`
  - `internal`: internal report path and Browser CDP Automation Detection pipeline.
  - `external`: edge adapter path (currently Akamai on `/fingerprint-report`, with internal fallback for non-Akamai/legacy payloads).
- `edge_integration_mode`
  - `off`: ignore Akamai outcomes.
  - `additive`: add bounded Akamai evidence into local fingerprint scoring.
  - `authoritative`: allow documented strong-signal short-circuit actions.

Terminology and architecture references:
- [`fingerprinting-terminology.md`](fingerprinting-terminology.md)
- [`fingerprinting-signal-planes.md`](fingerprinting-signal-planes.md)

## 🐙 Robots + <abbr title="Artificial Intelligence">AI</abbr> Policy Fields (`/shuma/admin/config`)

Robots serving controls:
- `robots_enabled`
- `robots_crawl_delay`

<abbr title="Artificial Intelligence">AI</abbr>-bot policy controls:
- `ai_policy_block_training`
- `ai_policy_block_search`
- `ai_policy_allow_search_engines`

## 🐙 <abbr title="Geolocation">GEO</abbr> Policy Fields (`/shuma/admin/config`)

- `geo_risk` - country list that contributes to cumulative botness scoring
- `geo_allow` - country list with explicit allow precedence (suppresses <abbr title="Geolocation">GEO</abbr> scoring)
- `geo_challenge` - country list that routes directly to challenge
- `geo_maze` - country list that routes directly to maze
- `geo_block` - country list that routes directly to block
- `geo_edge_headers_enabled` - enables/disables use of trusted edge country headers for GEO policy

Routing precedence for overlapping lists is:

- `geo_block` > `geo_maze` > `geo_challenge` > `geo_allow`

<abbr title="Geolocation">GEO</abbr> headers are only used when forwarded headers are trusted for the request:

- `SHUMA_FORWARDED_IP_SECRET` must be configured
- caller must provide matching `X-Shuma-Forwarded-Secret`
- `geo_edge_headers_enabled` must be `true`

## 🐙 <abbr title="Internet Protocol">IP</abbr>-Range Policy Fields (`/shuma/admin/config`)

Use this policy to match requests by <abbr title="Internet Protocol">IP</abbr> address/range and apply a configured action.

Mode:

- `ip_range_policy_mode`
  - `off`: do not run IP range policy.
  - `advisory`: evaluate and record outcomes only.
  - `enforce`: apply actions for matching rules.

Core fields:

- `ip_range_emergency_allowlist`
  - <abbr title="Classless Inter-Domain Routing">CIDR</abbr> ranges that bypass IP range actions.
  - Evaluated before custom rules.
- `ip_range_custom_rules`
  - Ordered custom rule objects (`id`, `enabled`, `cidrs`, `action`, optional `redirect_url`, optional `custom_message`).
  - First matching custom rule wins.

Valid actions for custom rules:

- `forbidden_403`
- `custom_message`
- `drop_connection`
- `redirect_308`
- `rate_limit`
- `honeypot`
- `maze`
- `tarpit`

Decision order:

- emergency allowlist -> custom rules (first match) -> default pipeline

Operational guidance:

- Full plain-English rollout/rollback runbook: [`docs/ip-range-policy-runbook.md`](ip-range-policy-runbook.md)

## 🐙 Maze Excellence Fields (`/shuma/admin/config`)

- `maze_rollout_phase` - staged enforcement (`instrument`, `advisory`, `enforce`)
- `maze_token_ttl_seconds`, `maze_token_max_depth`, `maze_token_branch_budget`, `maze_replay_ttl_seconds`
- `maze_entropy_window_seconds`, `maze_path_entropy_segment_len`
- `maze_client_expansion_enabled`, `maze_checkpoint_every_nodes`, `maze_checkpoint_every_ms`, `maze_step_ahead_max`, `maze_no_js_fallback_max_depth`
- `maze_micro_pow_enabled`, `maze_micro_pow_depth_start`, `maze_micro_pow_base_difficulty`
- `maze_max_concurrent_global`, `maze_max_concurrent_per_ip_bucket`, `maze_max_response_bytes`, `maze_max_response_duration_ms`
- `maze_server_visible_links`, `maze_max_links`, `maze_max_paragraphs`
- `maze_covert_decoys_enabled`
- `maze_seed_provider`, `maze_seed_refresh_interval_seconds`, `maze_seed_refresh_rate_limit_per_hour`, `maze_seed_refresh_max_sources`, `maze_seed_metadata_only`

`POST /shuma/admin/maze/seeds` payload shape:

- `sources`: array of source entries (`id`, `url`, optional `title`, optional `description`, optional `keywords`, optional `allow_seed_use`, optional `robots_allowed`, optional `body_excerpt`)

`POST /shuma/admin/maze/seeds/refresh` returns refresh status and source/corpus metadata.
