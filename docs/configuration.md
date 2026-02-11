# 🐙 Configuration

Shuma-Gorath uses a single runtime model:

- Tunables are **KV-backed only** (`config:<site_id>`, default site is `config:default`).
- Env vars are for **secrets and runtime guardrails only**.
- `config/defaults.env` is the canonical defaults source used by setup/seed tooling.

There is no runtime `SHUMA_CONFIG_USE_KV` mode switch.

## 🐙 Startup Model

`make setup` performs three key actions:

1. Creates `.env.local` from `config/defaults.env` if missing.
2. Generates local dev secrets in `.env.local` (for example `SHUMA_API_KEY`, `SHUMA_JS_SECRET`, `SHUMA_FORWARDED_IP_SECRET`).
3. Runs `make config-seed`, which writes tunable defaults into KV **only when `config:default` is missing**.

At runtime:

- Tunables are loaded from KV (`config:default`).
- Env-only keys are read from process env.
- If KV config is missing/invalid, requests that require config fail with `500 Configuration unavailable`.

## 🐙 Env-Only Keys

Canonical template:

- `/.env.full.example`

Supported env-only keys:

- `SHUMA_API_KEY` - admin login key for dashboard/API
- `SHUMA_JS_SECRET` - signs `js_verified` cookie
- `SHUMA_POW_SECRET` - optional dedicated PoW seed-signing secret
- `SHUMA_CHALLENGE_SECRET` - optional dedicated challenge seed-signing secret
- `SHUMA_FORWARDED_IP_SECRET` - trust gate for forwarded client IP/proto headers
- `SHUMA_ADMIN_IP_ALLOWLIST` - optional IP/CIDR allowlist for `/admin/*`
- `SHUMA_EVENT_LOG_RETENTION_HOURS` - event retention window
- `SHUMA_ADMIN_CONFIG_WRITE_ENABLED` - allow/deny admin config writes to KV
- `SHUMA_KV_STORE_FAIL_OPEN` - fail-open vs fail-closed behavior when KV is unavailable
- `SHUMA_ENFORCE_HTTPS` - reject non-HTTPS requests when true
- `SHUMA_DEBUG_HEADERS` - expose internal debug headers (dev only)
- `SHUMA_POW_CONFIG_MUTABLE` - runtime editability for PoW settings
- `SHUMA_CHALLENGE_CONFIG_MUTABLE` - runtime editability for challenge settings
- `SHUMA_BOTNESS_CONFIG_MUTABLE` - runtime editability for botness/challenge threshold settings

`make env-help` prints this list locally.

## 🐙 Tunables (KV-Backed)

These values are seeded from `config/defaults.env` into `config:default` and loaded from KV at runtime:

- Core flow: `test_mode`, `js_required_enforced`
- PoW: `pow_enabled`, `pow_difficulty`, `pow_ttl_seconds`
- Challenge/botness: `challenge_transform_count`, `challenge_risk_threshold`, `botness_maze_threshold`, `botness_weights.*`
- Ban durations: `ban_duration*`, `ban_durations.*`
- Rate/maze: `rate_limit`, `maze_enabled`, `maze_auto_ban`, `maze_auto_ban_threshold`
- GEO routing/scoring lists: `geo_risk`, `geo_allow`, `geo_challenge`, `geo_maze`, `geo_block`
- Browser/version lists: `browser_block`, `browser_whitelist`
- Whitelists: `whitelist`, `path_whitelist`
- robots policy: `robots_enabled`, `robots_block_ai_training`, `robots_block_ai_search`, `robots_allow_search_engines`, `robots_crawl_delay`
- CDP: `cdp_detection_enabled`, `cdp_auto_ban`, `cdp_detection_threshold`

## 🐙 Admin Config Writes

- `GET /admin/config` reads effective KV-backed config.
- `POST /admin/config` writes to KV only when `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=true`.
- Writes persist across restarts because they are stored in KV.

## 🐙 JS Verification + PoW

- `js_required_enforced=true` routes visitors without a valid `js_verified` cookie to JavaScript verification.
- `pow_enabled=true` makes that verification flow include server-verified PoW before cookie issuance.
- `js_required_enforced=false` bypasses this verification route for normal requests.

## 🐙 GEO Trust Boundary

GEO signals are only trusted when forwarded-header trust is established:

- `SHUMA_FORWARDED_IP_SECRET` is configured, and
- request includes matching `X-Shuma-Forwarded-Secret`.

Without trust, GEO routing/scoring is skipped.
