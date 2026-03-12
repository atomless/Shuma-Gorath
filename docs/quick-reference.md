# 🐙 Quick Reference - <abbr title="WebAssembly">WASM</abbr> Bot Defence

For full documentation, see [`docs/index.md`](index.md).

## 🐙 Common Commands

### 🐙 Setup
```bash
# Fastest full local contributor path
make setup          # Installs full toolchain, creates .env.local, and seeds KV config
make dev            # Starts the local dev server

# Recommended follow-up verification for the full path
make verify         # Verifies full setup (tooling + build path + read-only KV config check)

# If you only want the runtime and not the dashboard/e2e toolchain
make setup-runtime  # Runtime-only setup (Rust/wasm/Spin + env/bootstrap + KV seed)
make verify-runtime # Runtime-only verification (no Node/pnpm/Playwright, includes read-only KV config check)

# Config lifecycle helpers (only when needed)
make config-verify  # Read-only KV config lifecycle check (missing/stale/invalid)
make config-seed    # Explicit KV config seed/backfill/repair
```

### 🐙 Build & Run: Local
```bash
make dev            # Build and run with file watching (auto-rebuild)
make dev-prod       # Build/run with watching in runtime-prod local-direct posture (admin writes enabled)
make run            # Build once and run (no watching)
make build-runtime  # Build runtime/deploy release artifact (no dashboard budget gate)
make build-full-dev # Build release artifact with dashboard budget reporting (set SHUMA_DASHBOARD_BUNDLE_BUDGET_ENFORCE=1 for hard-fail)
make build          # Alias of make build-runtime
make prod           # Build for production and start server
make smoke-single-host # Post-deploy smoke: health/admin/metrics/challenge + forwarded public-path parity when gateway inputs are present
make stop           # Stop running Spin server
make status         # Check if server is running
make clean          # Clean build artifacts
make reset-local-state # Wipe local .spin runtime/test state while preserving durable .shuma operator state
```

### 🐙 Build & Run: Remote
```bash
make prepare-linode-shared-host # Agent-oriented Linode shared-host setup + receipt generation
make deploy-linode-one-shot # Provision Linode VM + deploy runtime in one command
make remote-update  # Upload exact committed HEAD to the active ssh_systemd remote, restart, smoke, refresh receipt
make remote-status  # Show systemd status for the active ssh_systemd remote
make remote-logs    # Show recent journal logs for the active ssh_systemd remote
make remote-start   # Start the active ssh_systemd remote service
make remote-stop    # Stop the active ssh_systemd remote service
make remote-open-dashboard # Open the hosted dashboard for the active ssh_systemd remote
make telemetry-shared-host-evidence # Capture live telemetry storage/query evidence for the active ssh_systemd remote
```

## 🐙 Runtime and Deployment Posture Matrix

| Posture | `SHUMA_RUNTIME_ENV` | `SHUMA_DEBUG_HEADERS` | `SHUMA_ADMIN_IP_ALLOWLIST` | `SHUMA_ENFORCE_HTTPS` | `SHUMA_GATEWAY_UPSTREAM_ORIGIN` | `SHUMA_LOCAL_PROD_DIRECT_MODE` |
| --- | --- | --- | --- | --- | --- | --- |
| `make dev` | `runtime-dev` | `true` | empty by default | `false` by default | not required | `false` (normally) |
| `make dev-prod` | `runtime-prod` | `false` | empty by default | `false` by default | not required (local-direct) | `true` |
| deployed production | `runtime-prod` | `false` | required and must be narrow | `true` | required | `false` |

Notes:
- `make dev` is intentionally permissive for local debugging and dashboard/operator iteration.
- `make dev-prod` is a localhost-only prod-like posture for observing `runtime-prod` behavior without a real gateway upstream. It is not a deployment substitute.
- deployed production adds deployment guardrails beyond the table above: non-overbroad admin allowlisting, edge rate-limit attestation, API-key rotation attestation, gateway origin lock confirmation, reserved-route collision proof, and strict gateway TLS posture.
- `SHUMA_ADMIN_CONFIG_WRITE_ENABLED` now defaults to `true` in all three postures; disable it only when you explicitly want a read-only admin config surface.
- `SHUMA_ADVERSARY_SIM_AVAILABLE` now defaults to `true` in all three postures; set it to `false` only when a deployment must hide adversary-sim surfaces entirely.

### 🐙 Testing
```bash
# All tests (recommended)
make test                  # Full suite: unit + integration + dashboard e2e (requires existing make dev server)

# Unit tests only (native Rust, NO Spin required)
make test-unit             # Run all unit tests
make test-gateway-harness  # Gateway fixture/failure harness + deploy guardrail parser tests

# Dashboard module unit tests only (NO Spin required)
make test-dashboard-unit   # Run dashboard JS/module contract tests

# Integration tests only (Spin environment required)
make dev                   # In terminal 1
make test-integration      # In terminal 2
make test-gateway-profile-shared-server # Shared-server gateway verification
make test-gateway-profile-edge          # Edge/Fermyon gateway verification
make smoke-gateway-mode                 # Fast gateway smoke checks
make test-adversary-sim-lifecycle       # Focused adversary-sim lifecycle regression gate
make test-adversary-sim-runtime-surface # Runtime-toggle defense-surface telemetry gate

# Live adversarial detection drill (Spin environment required)
make dev                   # In terminal 1
make test-adversarial-live # In terminal 2 (Ctrl+C to stop)

# Live remote trusted-edge signal proof (active ssh_systemd remote required)
make test-remote-edge-signal-smoke # Proves live fingerprint-report additive/authoritative + trusted GEO challenge/maze/block

# Dashboard e2e smoke tests only (Spin environment required)
make dev                   # In terminal 1
make test-dashboard-e2e    # In terminal 2
```
**Important:** Unit tests run in native Rust. Integration and dashboard e2e tests MUST run against a running Spin server; test targets do not start Spin.
`make clean` removes reproducible build/test artifacts only. Use `make reset-local-state` when you intentionally want to wipe `.spin` runtime/test state without deleting durable operator artifacts under `.shuma`.

## 🐙 <abbr title="Application Programming Interface">API</abbr> Endpoints

### 🐙 Public Endpoints
- `GET /` - Main bot defence (may show block page, <abbr title="JavaScript">JS</abbr> challenge, or pass through)
- `GET /health` - Health check (exact loopback or trusted forwarded loopback)
- `GET /instaban` - Honeypot (triggers ban)
- `GET /metrics` - Prometheus metrics
- `GET /robots.txt` - robots.txt (configurable)
- `GET /pow` - <abbr title="Proof of Work">PoW</abbr> seed (when enabled)
- `POST /pow/verify` - <abbr title="Proof of Work">PoW</abbr> verification
- `POST /cdp-report` - <abbr title="Chrome DevTools Protocol">CDP</abbr> automation report intake
- `POST /fingerprint-report` - External/edge fingerprint intake (currently Akamai-only mapping)
- `POST /challenge/puzzle` - Submit puzzle challenge answer (if challenge is served)

### 🐙 Admin <abbr title="Application Programming Interface">API</abbr>
- Supports read/write bearer auth (`SHUMA_API_KEY`), optional read-only bearer auth (`SHUMA_ADMIN_READONLY_API_KEY`), and same-origin admin sessions from `/admin/login`.
- Mutating session-authenticated calls also require `X-Shuma-CSRF`.
- If `SHUMA_ADMIN_IP_ALLOWLIST` is set, the client <abbr title="Internet Protocol">IP</abbr> must be allowlisted.

- `POST /admin/login` - Native dashboard login form endpoint (`application/x-www-form-urlencoded` `password=<SHUMA_API_KEY>`, optional `next=...`) that sets the admin session cookie and redirects
- `GET /admin/session` - Current auth/session state
- `POST /admin/logout` - Clear the admin session cookie
- `GET /admin/ban` - List all bans
- `POST /admin/ban` - Manually ban <abbr title="Internet Protocol">IP</abbr> (<abbr title="JavaScript Object Notation">JSON</abbr>: `{"ip":"x.x.x.x","duration":3600}`; reason is always `manual_ban`)
- `POST /admin/unban?ip=x.x.x.x` - Unban an <abbr title="Internet Protocol">IP</abbr>
- `GET /admin/analytics` - Get ban statistics
- `GET /admin/events?hours=24` - Get recent events
- `GET /admin/monitoring?hours=24&limit=10` - Get consolidated monitoring summaries + detail payload (`analytics`, `events`, `bans`, `maze`, `cdp`, `cdp_events`) for dashboard Monitoring refresh
- `GET /admin/monitoring/delta?...` - Cursor-ordered monitoring deltas
- `GET /admin/monitoring/stream?...` - One-shot monitoring SSE delta
- `GET /admin/ip-bans/delta?...` - Cursor-ordered ban/unban deltas
- `GET /admin/ip-bans/stream?...` - One-shot IP-ban SSE delta
- Expensive admin reads (`/admin/events`, `/admin/cdp/events`, `/admin/monitoring`, `/admin/ban` `GET`) are per-<abbr title="Internet Protocol">IP</abbr> rate-limited and return `429` + `Retry-After: 60` when limited.
- `GET /admin/config` - Get current configuration
- `POST /admin/config` - Update configuration (test_mode, ban_durations, robots serving, <abbr title="Artificial Intelligence">AI</abbr> bot policy, <abbr title="Chrome DevTools Protocol">CDP</abbr>, etc.)
- `POST /admin/config/validate` - Validate a config patch without persisting it
- `GET /admin/config/export` - Export non-secret runtime config for immutable redeploy handoff
  - Redis provider URLs are treated as secrets and excluded from this export.
- `POST /admin/adversary-sim/control` - Submit adversary-sim ON/OFF command
- `GET /admin/adversary-sim/status` - Read lifecycle state and diagnostics
- `POST /admin/adversary-sim/history/cleanup` - Explicitly clear retained simulation telemetry history
- `GET /admin/maze` - maze statistics
- `GET /admin/maze/preview?path=<maze_entry_path>...` - non-operational maze preview surface
- `GET /admin/robots` - robots.txt configuration and preview
- `POST /admin/robots/preview` - robots.txt preview from unsaved toggles/patch (no persistence)
- `GET /admin/cdp` - <abbr title="Chrome DevTools Protocol">CDP</abbr> + fingerprint detection configuration and stats
- `GET /admin` - <abbr title="Application Programming Interface">API</abbr> help

## 🐙 Configuration

### 🐙 <abbr title="Application Programming Interface">API</abbr> Key
Set in `spin.toml` or environment:
```toml
[component.bot-defence]
environment = { SHUMA_API_KEY = "your-secret-key-here", SHUMA_JS_SECRET = "your-js-secret-here", SHUMA_EVENT_LOG_RETENTION_HOURS = "168", SHUMA_HEALTH_SECRET = "your-health-secret-here", SHUMA_ADMIN_IP_ALLOWLIST = "203.0.113.0/24,198.51.100.10" }
```

`SHUMA_JS_SECRET` is used to sign the `js_verified` cookie for the <abbr title="JavaScript">JS</abbr> challenge.
`SHUMA_FORWARDED_IP_SECRET` is optional and is used to trust `X-Forwarded-For` from your proxy/<abbr title="Content Delivery Network">CDN</abbr> (it must also send `X-Shuma-Forwarded-Secret`). If you set it, include that header in integration tests.
`SHUMA_HEALTH_SECRET` is optional and, when set, `/health` also requires `X-Shuma-Health-Secret`.
`SHUMA_EVENT_LOG_RETENTION_HOURS` requests raw event retention, but high-risk raw operator views are capped to `72h`.
`SHUMA_MONITORING_RETENTION_HOURS` controls how long hourly monitoring counters and bucket indexes are kept.
`SHUMA_MONITORING_ROLLUP_RETENTION_HOURS` controls how long derived daily monitoring rollups are kept for longer-window summary reads.
`SHUMA_ADMIN_IP_ALLOWLIST` limits admin <abbr title="Application Programming Interface">API</abbr> access to specific IPs/CIDRs (comma-separated).
`SHUMA_KV_STORE_FAIL_OPEN` controls fail-open/closed behavior when the <abbr title="Key-Value">KV</abbr> store is unavailable (`true`=open, `false`=closed).
`SHUMA_POW_ENABLED` enables proof-of-work before <abbr title="JavaScript">JS</abbr> verification (default: true in dev).
`SHUMA_POW_DIFFICULTY` sets the leading-zero bit target (default: 15).
`SHUMA_POW_TTL_SECONDS` controls <abbr title="Proof of Work">PoW</abbr> seed expiry (default: 90).
`SHUMA_POW_SECRET` optionally overrides the <abbr title="Proof of Work">PoW</abbr> signing secret (falls back to `SHUMA_JS_SECRET`).
`SHUMA_MAZE_PREVIEW_SECRET` optionally sets a dedicated secret for `/admin/maze/preview` entropy/signing isolation.
`SHUMA_ADMIN_CONFIG_WRITE_ENABLED` controls whether admin config updates are allowed (default: true; set `false` only when you explicitly want read-only admin config).

### 🐙 Forwarded <abbr title="Internet Protocol">IP</abbr> Secret (Deployment)
Local dev (Makefile): `make dev` sets a dev-only default and passes it to Spin. Override as needed:
```bash
make dev SHUMA_FORWARDED_IP_SECRET="your-dev-secret"
```

Fermyon / Akamai edge (agent path):
1. Define an application variable in `spin.toml`.
2. Map it into the component environment.
3. Prepare the Akamai-edge setup receipt.
4. Deploy through the Akamai-edge helper.

Example `spin.toml` wiring (no secret committed):
```toml
[variables]
forwarded_ip_secret = { default = "" }

[component.bot-defence]
environment = { SHUMA_FORWARDED_IP_SECRET = "{{ forwarded_ip_secret }}" }
```

Other deploy targets:
- Set `SHUMA_FORWARDED_IP_SECRET` as an environment variable in your platform's secrets/config (Kubernetes, Docker, systemd, etc.).
- Ensure your proxy/<abbr title="Content Delivery Network">CDN</abbr> sends `X-Shuma-Forwarded-Secret` with the same value on each request.

Canonical commands:

```bash
make prepare-fermyon-akamai-edge PREPARE_FERMYON_ARGS="--upstream-origin https://origin.example.com --surface-catalog-path /abs/path/to/catalog.json --origin-lock-confirmed true --reserved-route-collision-check-passed true --admin-edge-rate-limits-confirmed true --admin-api-key-rotation-confirmed true"
make deploy-fermyon-akamai-edge
```

If PAT login panics, the helper falls back to Fermyon device login in interactive sessions. If browser auth then says `User is not allow-listed!`, provider access is still pending, the setup receipt is left behind in `status=blocked` form, and rerunning setup after provider approval resumes cleanly.
The deploy helper also provisions the managed five-job adversary-sim edge cron set and verifies both the immediate primed tick and a later cron-driven follow-up tick before treating the deploy as proven.

For more deployment detail, see [`docs/deployment.md`](deployment.md).

### 🐙 Test Mode
Enable for safe production testing (logs but doesn't block):

**Via Dashboard:** Use the Test Mode toggle in Admin Controls

**Via <abbr title="Application Programming Interface">API</abbr>:**
```bash
# Enable test mode
curl -X POST -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"test_mode": true}' \
  http://127.0.0.1:3000/admin/config

# Check current status
curl -H "Authorization: Bearer YOUR_API_KEY" \
  http://127.0.0.1:3000/admin/config
```

Test mode is a <abbr title="Key-Value">KV</abbr>-backed admin-editable runtime setting; use dashboard or `POST /admin/config` to change it.

### 🐙 Default Config
Defaults are defined in `config/defaults.env` and seeded into <abbr title="Key-Value">KV</abbr> for admin-editable runtime settings:
- **Ban duration**: 21600 seconds (6 hours)
- **Rate limit**: 80 requests/minute
- **Honeypots**: `/instaban`
- **Browser blocks**: Chrome <120, Firefox <115, Safari <15

Full configuration reference (including configuration-class explanation): [`docs/configuration.md`](configuration.md).

## 🐙 Dashboard

1. Open `http://127.0.0.1:3000/dashboard/login.html` in browser
2. Enter the key from `make api-key-show` (local dev) or the deployed `SHUMA_API_KEY`
3. Dashboard login submits a native form to `/admin/login`, which sets the same-origin admin session cookie and redirects into the dashboard
4. After login, use `http://127.0.0.1:3000/dashboard/index.html` (or `/dashboard`) for the full tabbed UI

## 🐙 Common Tasks

### 🐙 Ban an <abbr title="Internet Protocol">IP</abbr> manually
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"1.2.3.4","reason":"spam","duration":3600}' \
  http://127.0.0.1:3000/admin/ban
```

### 🐙 Unban an <abbr title="Internet Protocol">IP</abbr>
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
```

### 🐙 View recent events
```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/events?hours=24" | jq
```

### 🐙 Export runtime config for immutable redeploy handoff
```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/config/export" | jq -r '.env_text'
```

### 🐙 Test honeypot
If `SHUMA_FORWARDED_IP_SECRET` is set, include the matching header:
```bash
curl -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/instaban
# Subsequent requests from 1.2.3.4 will be blocked
```

## 🐙 Troubleshooting

### 🐙 Build Errors
- If switching targets and you see build issues, run `make clean`
- Ensure dependencies are installed: `make setup` then `make verify`

### 🐙 Port Already in Use
- Use `make stop` then `make dev`

### 🐙 Tests Failing
- Use Makefile targets (`make test`, `make test-unit`, `make test-dashboard-unit`, `make test-integration`, `make test-dashboard-e2e`)
- `make test` waits for existing Spin readiness (`/health`) and requires the running server to report `runtime-dev`
- `make dev-prod` is for prod-like localhost observation; stop it and restart with `make dev` before `make test`
- If startup is slow, increase wait timeout: `make test SPIN_READY_TIMEOUT_SECONDS=180`
- Check logs with `make logs`

### 🐙 Dashboard Not Loading
- Ensure Spin is running: `make status`
- Open `http://127.0.0.1:3000/dashboard/index.html`
- Confirm <abbr title="Application Programming Interface">API</abbr> key and check logs: `make logs`

## 🐙 Project Structure
```
src/
├── lib.rs                 # Main orchestration entrypoint
├── admin/                 # Admin API + auth/session flow
├── challenge/             # Puzzle + challenge flows
├── config/                # Runtime config loading/defaults
├── enforcement/           # Ban/block/rate/honeypot actions
├── maze/                  # Maze barrier logic
├── observability/         # Metrics/export
├── providers/             # Provider contracts + registry + internal adapters
├── runtime/               # Request router/policy pipeline/test-mode helpers
└── signals/               # Browser/CDP/GEO/IP/JS/allowlist signals

dashboard/                 # Web dashboard UI
scripts/tests/integration.sh # Spin integration scenarios
```

## 🐙 Security Notes

- **Never commit <abbr title="Application Programming Interface">API</abbr> keys** - Use environment variables
- **Rotate keys regularly** - Change SHUMA_API_KEY in production
- **Use <abbr title="Hypertext Transfer Protocol Secure">HTTPS</abbr> in production** - <abbr title="Transport Layer Security">TLS</abbr> required for <abbr title="Application Programming Interface">API</abbr> key security
- **Restrict admin access** - Use <abbr title="Internet Protocol">IP</abbr> allowlist or <abbr title="Virtual Private Network">VPN</abbr>
- **Monitor event logs** - Review admin actions regularly

## 🐙 Next Steps

1. **Shared-host Deploy**: Use `make prepare-linode-shared-host` and `make deploy-linode-one-shot` for the canonical Linode/shared-host path
2. **Routine Remote Ops**: Use `make remote-update`, `make remote-status`, and `make remote-open-dashboard` once the first SSH-managed remote is deployed
3. **Custom Config**: Update config in <abbr title="Key-Value">KV</abbr> store for your needs
4. **Monitor and Tune**: Use dashboard monitoring plus test mode/adversary sim to validate before tightening enforcement
5. **Enterprise/Egress Work**: Use [`docs/deployment.md`](deployment.md) for the staged enterprise/Akamai/Fermyon posture and gateway guardrails
