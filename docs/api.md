# 🐙 <abbr title="Application Programming Interface">API</abbr> & Endpoints

## 🐙 Authentication

Admin endpoints support two auth modes:
- Bearer token (read/write): `Authorization: Bearer <SHUMA_API_KEY>`
- Bearer token (read-only, optional): `Authorization: Bearer <SHUMA_ADMIN_READONLY_API_KEY>`
- Session cookie: `POST /admin/login` with `{"api_key":"<SHUMA_API_KEY>"}` sets a short-lived `HttpOnly` cookie

Write endpoints (`POST`, `PUT`, `PATCH`, `DELETE` on mutating admin routes) require read/write access.
Read-only bearer tokens can access non-mutating admin endpoints only.

If `SHUMA_ADMIN_IP_ALLOWLIST` is set, the client <abbr title="Internet Protocol">IP</abbr> must be in the allowlist.

For session-authenticated write requests (`POST`, `PUT`, `PATCH`, `DELETE`), include:
- `X-Shuma-CSRF: <csrf_token>` (returned by `/admin/login` and `/admin/session`)

If `SHUMA_FORWARDED_IP_SECRET` is configured, any request that relies on `X-Forwarded-For` must also include:
- `X-Shuma-Forwarded-Secret: <SHUMA_FORWARDED_IP_SECRET>`

If `SHUMA_ENFORCE_HTTPS=true`:
- requests without <abbr title="Hypertext Transfer Protocol Secure">HTTPS</abbr> context are rejected with `403 HTTPS required`
- forwarded proto headers are trusted only when `SHUMA_FORWARDED_IP_SECRET` validation succeeds

If `SHUMA_API_KEY` is missing, `/admin/*` endpoints are disabled. Placeholder/insecure <abbr title="Application Programming Interface">API</abbr> keys are rejected.

Failed admin auth attempts are rate-limited per <abbr title="Internet Protocol">IP</abbr> bucket (`SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE`, default `10`), but you should still enforce <abbr title="Content Delivery Network">CDN</abbr>/<abbr title="Web Application Firewall">WAF</abbr> rate limits for `POST /admin/login` and `/admin/*`.

If `SHUMA_HEALTH_SECRET` is configured, `/health` also requires:
- `X-Shuma-Health-Secret: <SHUMA_HEALTH_SECRET>`

## 🐙 Public Endpoints

- `GET /` - Main bot defence handler
- `GET /health` - Health check (loopback only)
- `GET /metrics` - Prometheus metrics (no auth)
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
- `GET /dashboard/...` - Dashboard static assets
- `GET /challenge/puzzle` - Dev-only puzzle challenge page (`test_mode=true` in runtime config)
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
3. If `SHUMA_POW_ENABLED=true`, the interstitial solves <abbr title="Proof of Work">PoW</abbr> and submits `POST /pow/verify`.
4. `/pow/verify` validates the proof and returns `Set-Cookie: js_verified=...`.
5. After a valid `js_verified` cookie is set, the page reloads and the original route is retried.

If `SHUMA_POW_ENABLED=false`:

- the same interstitial still runs, but it sets `js_verified` directly in browser <abbr title="JavaScript">JS</abbr> and reloads.
- this is lower-friction but weaker than server-verified <abbr title="Proof of Work">PoW</abbr> issuance.

If `js_required_enforced=false`:

- normal routing does not send visitors through the <abbr title="JavaScript">JS</abbr> verification interstitial.
- `/pow` and `/pow/verify` still exist, but they are not part of the default access gate.

### 🐙 Health Check Example

```bash
curl -H "X-Forwarded-For: 127.0.0.1" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/health
```

When `SHUMA_DEBUG_HEADERS=true`, the health response includes:
- `X-KV-Status` (available/unavailable)
- `X-Shuma-Fail-Mode` (open/closed)

## 🐙 Admin Endpoints

- `GET /admin` - <abbr title="Application Programming Interface">API</abbr> help
- `POST /admin/login` - Exchange <abbr title="Application Programming Interface">API</abbr> key for short-lived admin session cookie
- `GET /admin/session` - Current auth/session state
- `POST /admin/logout` - Clear admin session cookie
- `GET /admin/ban` - List active bans
- `POST /admin/ban` - Ban an <abbr title="Internet Protocol">IP</abbr> (<abbr title="JavaScript Object Notation">JSON</abbr> body: `{"ip":"x.x.x.x","duration":3600}`; reason is always `manual_ban`)
- `POST /admin/unban?ip=x.x.x.x` - Unban an <abbr title="Internet Protocol">IP</abbr>
- `GET /admin/analytics` - Ban/event statistics
- `GET /admin/events?hours=N` - Recent events + summary stats
- `GET /admin/cdp/events?hours=N&limit=M` - <abbr title="Chrome DevTools Protocol">CDP</abbr>-only detections/auto-bans (time-windowed, limit configurable)
- `GET /admin/monitoring?hours=N&limit=M` - Consolidated monitoring summaries plus dashboard-native detail payload for Monitoring tab refreshes
- `GET /admin/ip-range/suggestions?hours=N&limit=M` - Suggested IP-range candidates with collateral-risk scoring
- `GET /admin/config` - Read configuration
- `POST /admin/config` - Update configuration (partial <abbr title="JavaScript Object Notation">JSON</abbr>, disabled when `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false`)
- `POST /admin/config/validate` - Validate a config patch without persisting changes (returns `{ valid, issues[] }` with field/expected/received hints when invalid)
- `GET /admin/config/export` - Export non-secret runtime config as deploy-ready env key/value output
- `GET /admin/maze` - maze stats
- `GET /admin/maze/preview?path=<maze_entry_path>...` - Non-operational maze preview (admin-auth only; no live traversal token issuance)
- `GET /admin/maze/seeds` - Maze operator-seed source list and cached corpus snapshot
- `POST /admin/maze/seeds` - Upsert maze operator-seed sources
- `POST /admin/maze/seeds/refresh` - Trigger manual maze operator-corpus refresh
- `GET /admin/robots` - robots.txt config and preview
- `POST /admin/robots/preview` - robots.txt preview from an unsaved config patch (does not persist)
- `GET /admin/cdp` - <abbr title="Chrome DevTools Protocol">CDP</abbr> + fingerprint detection config and stats

`GET /admin/session` includes `access` as `read_only`, `read_write`, or `none`.

Expensive admin read endpoints (`/admin/events`, `/admin/cdp/events`, `/admin/monitoring`, `/admin/ip-range/suggestions`, `/admin/ban` `GET`) are rate-limited to reduce <abbr title="Key-Value">KV</abbr>/<abbr title="Central Processing Unit">CPU</abbr> abuse amplification (`429` with `Retry-After: 60` when limited).

`GET /admin/maze/preview` is intentionally non-operational:
- links recurse only into `/admin/maze/preview`,
- live `mt` traversal tokens are not emitted,
- hidden covert-decoy tracking markers/links are not emitted,
- maze replay/checkpoint/budget/risk counters are not mutated.

### 🐙 Analytics Response

`GET /admin/analytics` returns:
- `ban_count`
- `test_mode`
- `fail_mode`

### 🐙 Admin Events Response

`GET /admin/events?hours=24` returns:
- `recent_events` (up to 100 events)
- `event_counts` (counts per event type)
- `top_ips` (top 10 IPs by event count)
- `unique_ips` (distinct <abbr title="Internet Protocol">IP</abbr> count)

For <abbr title="Chrome DevTools Protocol">CDP</abbr>-only operational views without the 100-row mixed-event cap, use:

`GET /admin/cdp/events?hours=24&limit=500` returns:
- `events` (<abbr title="Chrome DevTools Protocol">CDP</abbr> detection and <abbr title="Chrome DevTools Protocol">CDP</abbr> auto-ban events only, up to `limit`)
- `hours` (effective query window)
- `limit` (effective result cap)
- `total_matches` (number of matched <abbr title="Chrome DevTools Protocol">CDP</abbr> events before truncation)
- `counts.detections` (<abbr title="Chrome DevTools Protocol">CDP</abbr> detection event count in the window)
- `counts.auto_bans` (<abbr title="Chrome DevTools Protocol">CDP</abbr> auto-ban event count in the window)

### 🐙 Admin Monitoring Summary Response

`GET /admin/monitoring?hours=24&limit=10` returns:
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
- `prometheus`:
- `endpoint` (`/metrics`), helper notes, and scrape examples for external platforms
- `details` (dashboard Monitoring-tab refresh contract):
- `analytics`: `ban_count`, `test_mode`, `fail_mode`
- `events`: `recent_events`, `event_counts`, `top_ips`, `unique_ips`
- `bans`: `bans`
- `maze`: `total_hits`, `unique_crawlers`, `maze_auto_bans`, `deepest_crawler`, `top_crawlers`
- `cdp`: `config`, `stats`, `fingerprint_stats`
- `cdp_events`: `events`, `hours`, `limit`, `total_matches`, `counts`

### 🐙 IP Range Suggestions Response

`GET /admin/ip-range/suggestions?hours=24&limit=20` returns:
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

Event `outcome` values may include canonical taxonomy metadata:

- `taxonomy[level=L* action=A* detection=D* signals=S_*...]`

This uses the same public ladder documented in [`/docs/bot-defence.md`](bot-defence.md) (`Escalation Ladder (L0-L11)`).

### 🐙 <abbr title="Chrome DevTools Protocol">CDP</abbr> + Fingerprint Admin View

`GET /admin/cdp` returns:
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

`GET /admin/config/export` returns:
- `format` (`env`)
- `site_id`
- `generated_at` (unix seconds)
- `env` (non-secret `SHUMA_*` values as strings)
- `env_text` (newline-delimited `KEY=value` export)
- `excluded_secrets` (secret keys intentionally omitted, including Redis provider URLs)

### 🐙 Example: List Bans

```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  http://127.0.0.1:3000/admin/ban
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
  http://127.0.0.1:3000/admin/ban
```

### 🐙 Example: Fetch Events

```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  http://127.0.0.1:3000/admin/events?hours=24
```

## 🐙 Botness Policy Fields (`/admin/config`)

The unified botness model uses weighted scored signals plus terminal hard-ban signals.

Core enforcement fields:
- `js_required_enforced` - enable/disable <abbr title="JavaScript">JS</abbr>-required enforcement
- `rate_limit` - per-minute request limit used for hard rate limiting and rate-pressure scoring, applied per source IP bucket (IPv4 /24, IPv6 /64)
- `honeypot_enabled` - enable/disable honeypot trap handling for configured trap paths
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
- When `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false`, `POST /admin/config` returns `403`.
- `POST /admin/config/validate` runs the same server-side validators as `POST /admin/config` but does not write <abbr title="Key-Value">KV</abbr> state.

Effective-mode visibility:
- `defence_modes_effective` reports runtime-effective signal/action booleans per module.
- `defence_mode_warnings` reports mode conflicts (for example <abbr title="JavaScript">JS</abbr> mode overridden by `js_required_enforced=false`).
- Enterprise state posture visibility:
  - `enterprise_multi_instance`
  - `enterprise_unsynced_state_exception_confirmed`
  - `enterprise_state_guardrail_warnings`
  - `enterprise_state_guardrail_error`
- Invalid `defence_modes` keys or invalid mode values are rejected by `POST /admin/config` with `400`.

Signal catalog:
- `botness_signal_definitions.scored_signals` lists weighted contributors.
- `botness_signal_definitions.terminal_signals` lists immediate actions that bypass scoring.

## 🐙 Akamai Bot Signal Fields (`/admin/config`)

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

## 🐙 Robots + <abbr title="Artificial Intelligence">AI</abbr> Policy Fields (`/admin/config`)

Robots serving controls:
- `robots_enabled`
- `robots_crawl_delay`

<abbr title="Artificial Intelligence">AI</abbr>-bot policy controls (first-class keys):
- `ai_policy_block_training`
- `ai_policy_block_search`
- `ai_policy_allow_search_engines`

Legacy compatibility mirrors (still returned and accepted):
- `robots_block_ai_training`
- `robots_block_ai_search`
- `robots_allow_search_engines`

## 🐙 <abbr title="Geolocation">GEO</abbr> Policy Fields (`/admin/config`)

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

## 🐙 <abbr title="Internet Protocol">IP</abbr>-Range Policy Fields (`/admin/config`)

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

## 🐙 Maze Excellence Fields (`/admin/config`)

- `maze_rollout_phase` - staged enforcement (`instrument`, `advisory`, `enforce`)
- `maze_token_ttl_seconds`, `maze_token_max_depth`, `maze_token_branch_budget`, `maze_replay_ttl_seconds`
- `maze_entropy_window_seconds`, `maze_path_entropy_segment_len`
- `maze_client_expansion_enabled`, `maze_checkpoint_every_nodes`, `maze_checkpoint_every_ms`, `maze_step_ahead_max`, `maze_no_js_fallback_max_depth`
- `maze_micro_pow_enabled`, `maze_micro_pow_depth_start`, `maze_micro_pow_base_difficulty`
- `maze_max_concurrent_global`, `maze_max_concurrent_per_ip_bucket`, `maze_max_response_bytes`, `maze_max_response_duration_ms`
- `maze_server_visible_links`, `maze_max_links`, `maze_max_paragraphs`
- `maze_covert_decoys_enabled`
- `maze_seed_provider`, `maze_seed_refresh_interval_seconds`, `maze_seed_refresh_rate_limit_per_hour`, `maze_seed_refresh_max_sources`, `maze_seed_metadata_only`

`POST /admin/maze/seeds` payload shape:

- `sources`: array of source entries (`id`, `url`, optional `title`, optional `description`, optional `keywords`, optional `allow_seed_use`, optional `robots_allowed`, optional `body_excerpt`)

`POST /admin/maze/seeds/refresh` returns refresh status and source/corpus metadata.
