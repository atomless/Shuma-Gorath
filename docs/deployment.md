# 🐙 Deployment & Configuration

Shuma-Gorath is designed to run on Spin (local or cloud). Use the Makefile paths as the official workflow.

Shuma-Gorath is intended to complement enterprise bot defenses (for example Akamai Bot Manager), but can run standalone.
Akamai-specific operator controls belong only to the Akamai edge posture (`SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon`). Shared-server and other non-edge deployments keep the baseline and generic trusted-header surfaces, but must not present themselves as Akamai-edge integrations.

The Fermyon / Akamai edge path is now a deferred gateway-only posture. Keep it truthful and supportable for later edge experiments, but do not treat it as the current pre-launch control-plane, hosted Scrapling, or scheduled-agent target.

For the current deployment-track design record, see:

- [`docs/plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md`](plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md)
- [`docs/deferred-edge-gateway.md`](deferred-edge-gateway.md)

## 🐙 Runtime Configuration Model

- Admin-editable runtime settings are loaded from <abbr title="Key-Value">KV</abbr> (`config:default`).
- Environment-only variables are secrets/guardrails and are read from process env.
- `make setup` and `make setup-runtime` seed <abbr title="Key-Value">KV</abbr>-backed admin-editable settings from `config/defaults.env` using `make config-seed`.
- Normal runtime/deploy verification paths are read-only with respect to persisted <abbr title="Key-Value">KV</abbr> config and use `make config-verify`; if that check reports missing, stale, or invalid config, run `make config-seed` explicitly before starting or deploying.
- Runtime config is process-cached for a short <abbr title="Time To Live">TTL</abbr> (2 seconds) to reduce hot-path <abbr title="Key-Value">KV</abbr> reads.
- `POST /shuma/admin/config` invalidates cache on the handling instance; other instances converge on their <abbr title="Time To Live">TTL</abbr> window.
- `GET /shuma/admin/config/export` provides a non-secret `KEY=value` handoff snapshot for immutable redeploy workflows.

For the canonical explanation of these two configuration classes, see:
[`docs/configuration.md#configuration-sources-admin-editable-runtime-settings-vs-environment-only-variables`](configuration.md#configuration-sources-admin-editable-runtime-settings-vs-environment-only-variables).

If <abbr title="Key-Value">KV</abbr> config is missing/invalid at runtime, config-dependent request handling fails with `500 Configuration unavailable`.

## 🐙 Release Gate Contract

Release cut/deploy lanes must enforce the same adversarial policy as protected CI lanes:

1. Blocking deterministic oracle:
   - `make test-adversarial-coverage`
   - `make test-adversarial-promote-candidates` (blocks only when deterministic replay confirms regression candidates)
2. Advisory frontier telemetry:
   - `make test-adversarial-frontier-attempt`
3. Frontier unavailability threshold policy:
   - `make test-frontier-unavailability-policy` (with `FRONTIER_POLICY_ENABLE_GITHUB=1` in CI) updates degraded streak tracking and opens/assigns refresh action when threshold is crossed.
   - If repository Issues are disabled, the policy must degrade to artifact-only mode and must not fail the deploy lane solely because GitHub issue creation is unavailable.

Stochastic single-run frontier anomalies must not block release directly; only deterministic confirmed regressions are release blockers.

## 🐙 Simulation Data-Plane Separation Contract

Adversary simulation telemetry is isolated by authenticated tagging and explicit surface guardrails rather than by forbidding `runtime-prod`:

1. Simulation-tagged request telemetry writes to canonical event/monitoring stores:
   - event log: `eventlog:v2:*`
   - monitoring counters: `monitoring:v1:*`
2. Simulation rows remain identifiable via metadata (`sim_run_id`, `sim_profile`, `sim_lane`, `is_simulation`).
3. Production-capable control surface:
   - `SHUMA_ADVERSARY_SIM_AVAILABLE` defaults to `true` in both runtime classes so deployed operators can use adversary-sim controls in production.
   - Traffic generation remains off until an operator enables it through `POST /shuma/admin/adversary-sim/control` (or the dashboard `Red Team` toggle). `SHUMA_ADVERSARY_SIM_ENABLED` seeds only the initial desired state.
   - `GET /shuma/admin/adversary-sim/status` now makes the default production posture explicit: `gateway_deployment_profile`, `guardrails.surface_available_by_default=true`, `guardrails.generation_default=off_until_explicit_enable`, `guardrails.generation_requires_explicit_enable=true`, and deployment-profile-specific supervisor cadence/trigger fields (`deployment_profile`, `trigger_surface`, `cadence_seconds`, `cron_schedule` when edge cron applies).
   - Shared-host startup paths now run through `scripts/run_with_oversight_supervisor.sh`, which chains the existing adversary-sim supervisor wrapper and adds bounded periodic `POST /shuma/internal/oversight/agent/run` calls so the first canary-apply feedback loop stays off the request path while sharing the same trusted-forwarding contract as the host-side sim supervisor.
   - Treat that surface as a normal production operating lane, not as a dev-only exception. A deployment receipt should capture one status read while off, an explicit ON operation, the no-impact proof from `make test-adversary-sim-runtime-surface` against the running target, the shared-host feedback-loop proof from `make test-live-feedback-loop-remote` when the bounded apply loop is in scope, and the explicit OFF operation used as the kill switch.
   - Deployments that must hide the surface entirely may set `SHUMA_ADVERSARY_SIM_AVAILABLE=false`.

This separation does not require different admin API keys between dev/prod; isolation is enforced by authenticated simulation metadata, operator-controlled lifecycle state, and deployment environment boundaries.

## 🐙 Setup Path Selection

Pick one setup flow and stick to it for that machine:

- Runtime-only single-host operator path (production/minimal):
  - `make setup-runtime`
  - `make verify-runtime`
- Full contributor/dev path (includes dashboard/e2e toolchain):
  - `make setup`
  - `make verify`

Both setup flows now also provision the repo-owned Scrapling worker runtime at `.venv-scrapling` with the pinned `scrapling[fetchers]` dependency set used by the real `scrapling_traffic` lane and its focused verification gates.

`make setup` remains the full contributor workflow. `make setup-runtime` intentionally skips Node/pnpm/Playwright, but it does not skip the Scrapling worker runtime.

## 🐙 One-Command Linode Provision + Deploy

If you are starting from a local site plus a Linode account, prepare the shared-host receipt first:

```bash
make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /abs/path/to/site"
```

That helper is agent-oriented. It can:

- capture or validate the Linode Personal Access Token and persist it to gitignored `.env.local`,
- propose and persist `SHUMA_ADMIN_IP_ALLOWLIST`,
- generate `GATEWAY_SURFACE_CATALOG_PATH`,
- create or inspect the Linode instance,
- write `.shuma/linode-shared-host-setup.json`,
- emit a normalized day-2 target receipt under `.shuma/remotes/<name>.json`.

After that, `make deploy-linode-one-shot` can reuse the persisted `.env.local` state plus the SSH key paths stored in `.shuma/linode-shared-host-setup.json` instead of asking the operator for those inputs again.

For a fresh Linode account/host bootstrap, you can provision and deploy from this repository in one command:

```bash
LINODE_TOKEN=<token> \
SHUMA_ADMIN_IP_ALLOWLIST=<trusted-ip-or-cidr> \
SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://origin.example.com \
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true \
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true \
SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true \
SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true \
GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json \
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--domain shuma.example.com --region gb-lon --type g6-standard-1"
```

For an interactive local run where success should end with the hosted dashboard opening on the operator machine, add `--open-dashboard`:

```bash
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--domain shuma.example.com --existing-instance-id 123456 --open-dashboard"
```

Requirements:

- run from a cloned `Shuma-Gorath` repository
- local SSH keypair available (default public key lookup: `~/.ssh/id_ed25519.pub`, fallback `~/.ssh/id_rsa.pub`)
- Linode Personal Access Token (exported locally as `LINODE_TOKEN`) with Linodes read/write scope
- domain/TLS is mandatory for the canonical production path
- local `make deploy-env-validate` must pass before provisioning

This workflow runs local production preflight, builds an exact local git `HEAD` release bundle, provisions the VM, bootstraps runtime dependencies on the server, validates remote single-host posture with `make deploy-self-hosted-minimal`, runs `make smoke-single-host` (including forwarded public-path parity against the configured upstream origin plus reserved-route/shuma/admin checks), installs a `systemd` unit that starts the already-prepared runtime with `make prod-start`, and prints the final dashboard URL. When `--open-dashboard` is set, it also opens `/shuma/dashboard` locally after success.
Telemetry-read responsiveness is part of the deploy acceptance contract, not a secondary polish check. After deploy:

- shared-host / Linode acceptance should include `make telemetry-shared-host-evidence`,
- Fermyon / Akamai-edge acceptance should include `make telemetry-fermyon-edge-evidence`,
- cross-target hot-read work should use `make test-telemetry-hot-read-live-evidence` as the canonical live proof.

For shared-host gateway deployments, the canonical path also renders a deployment-specific Spin manifest from [`spin.toml`](../spin.toml) so the runtime keeps the repo template deny-by-default while the deployed host gets the exact upstream allowlist it needs.
For admin-route smoke checks, `make smoke-single-host` derives an allowlisted forwarded IP from `SHUMA_ADMIN_IP_ALLOWLIST` by default. Override it with `SHUMA_SMOKE_ADMIN_FORWARDED_IP` when the first allowlist entry is not the right trusted operator IP for the check.
On successful deploy, the Linode path also refreshes `.shuma/remotes/<name>.json` so later `make remote-*` day-2 operations can use the provider-agnostic `ssh_systemd` contract instead of rerunning provider-specific setup.

If you already have a prepared Linode instance with a same-host origin listening on a local-only upstream such as `http://127.0.0.1:8080`, attach Shuma without reprovisioning by using `--existing-instance-id`:

```bash
LINODE_TOKEN=<token> \
SHUMA_ADMIN_IP_ALLOWLIST=<trusted-ip-or-cidr> \
SHUMA_GATEWAY_UPSTREAM_ORIGIN=http://127.0.0.1:8080 \
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true \
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true \
SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true \
SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true \
GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json \
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--existing-instance-id 123456 --domain shuma.example.com"
```

Related repo-local deployment skills:

- [`../skills/prepare-scrapling-for-deploy/SKILL.md`](../skills/prepare-scrapling-for-deploy/SKILL.md)
- [`../skills/prepare-shared-host-on-linode/SKILL.md`](../skills/prepare-shared-host-on-linode/SKILL.md)
- [`../skills/deploy-shuma-on-linode/SKILL.md`](../skills/deploy-shuma-on-linode/SKILL.md)

Deferred edge-gateway-only skills:

- [`../skills/prepare-shuma-on-akamai-fermyon/SKILL.md`](../skills/prepare-shuma-on-akamai-fermyon/SKILL.md)
- [`../skills/deploy-shuma-on-akamai-fermyon/SKILL.md`](../skills/deploy-shuma-on-akamai-fermyon/SKILL.md)

## 🐙 Generic SSH Remote Day-2 Operations

Once setup or deploy succeeds, the emitted `.shuma/remotes/<name>.json` receipt is auto-selected into `.env.local`, so routine operations can move straight to the provider-agnostic remote layer:

```bash
make remote-update
make remote-status
make remote-logs
make remote-start
make remote-stop
make remote-open-dashboard
```

Rules:

- `.env.local` keeps only `SHUMA_ACTIVE_REMOTE=<name>` for remote selection, and successful setup/deploy now updates it automatically.
- structured remote target state lives in `.shuma/remotes/<name>.json`.
- the current generic backend contract is `ssh_systemd` only.
- `remote-update` now means: ship the exact committed local `HEAD`, preserve remote `.env.local` and `.spin`, validate and restart on the remote host, run a remote loopback `/shuma/health` check plus public-route smoke against the public base URL, refresh local receipt metadata, and attempt rollback if smoke fails. If an older host is missing smoke-critical secrets in local `.env.local`, the helper hydrates those values from the remote `.env.local` first and persists them locally.
- `remote-update` must use the shipped prebuilt release bundle on the host. It must not rebuild dashboard or runtime artifacts remotely, because day-2 updates must stay tied to the exact committed local bundle rather than to ad hoc remote build tooling.
- `make remote-use REMOTE=<name>` remains the manual switch command when you want to change the active target later.
- `make clean` must not remove `.shuma`; use `make reset-local-state` only when you intentionally want to wipe local `.spin` runtime/test state.

## 🐙 10-Minute `self_hosted_minimal` Runbook (Start + Health + Rollback)

This is the fastest secure baseline for a single VM/shared host.

1. Bootstrap runtime dependencies:

```bash
make setup-runtime
make verify-runtime
```

2. Set production env-only values in your secret manager or `.env.local` (minimum: `SHUMA_API_KEY`, `SHUMA_JS_SECRET`, `SHUMA_ADMIN_IP_ALLOWLIST`, `SHUMA_HEALTH_SECRET`, `SHUMA_DEBUG_HEADERS=false`, `SHUMA_ENFORCE_HTTPS=true`, `SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`, `SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`). `SHUMA_ADMIN_CONFIG_WRITE_ENABLED` now defaults to `true`; set it to `false` only when you explicitly want read-only admin config for that deployment.

3. Build + validate single-host deployment posture:

```bash
make deploy-self-hosted-minimal
```

4. Start service:

```bash
make prod
```

5. Verify service health and baseline functionality:

```bash
GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json make smoke-single-host
```

If the auto-selected public path is too dynamic for exact body parity, rerun with a stable path override:

```bash
GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json \
SHUMA_SMOKE_FORWARD_PATH=/public/stable-page-or-asset \
make smoke-single-host
```

The auto-selector prefers obvious static assets before HTML pages so the default shared-host parity probe stays as close as possible to a bypass-safe public path.

If the admin allowlist contains multiple candidate ranges and the first entry is not the operator IP you want to simulate during smoke, override the admin-route forwarded IP explicitly:

```bash
GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json \
SHUMA_SMOKE_ADMIN_FORWARDED_IP=203.0.113.8 \
make smoke-single-host
```

6. Verify the production adversary-sim operating receipt:

   - From the trusted operator path, read `GET /shuma/admin/adversary-sim/status` (or the dashboard `Red Team` status card) and confirm:
     - `gateway_deployment_profile` matches the deployed posture,
     - `guardrails.surface_available_by_default=true`,
     - `guardrails.generation_default=off_until_explicit_enable`,
     - `guardrails.generation_requires_explicit_enable=true`.
   - Enable adversary-sim once through `POST /shuma/admin/adversary-sim/control` or the dashboard toggle and keep the returned `operation_id` as the ON receipt.
   - On the running host or an equivalent prod-like target, run:

```bash
make test-adversary-sim-runtime-surface
```

   - Record that the gate proved both deterministic adversary-sim defense-surface coverage and live-summary no-impact while the run was active.
   - Disable adversary-sim through the same control path, keep the OFF `operation_id` as the production kill-switch receipt, and use `POST /shuma/admin/adversary-sim/history/cleanup` only when you intentionally need retained telemetry reset.

6. Verify the first shared-host bounded feedback loop when that loop is part of the release target:

```bash
make test-live-feedback-loop-remote
```

   - Record that the live proof confirmed the deployed service is running through `scripts/run_with_oversight_supervisor.sh`, that `GET /shuma/admin/operator-snapshot` and `GET /shuma/admin/oversight/agent/status` are available, that one internal periodic agent run was recorded with explicit `execution.apply.stage` truth, and that one completed adversary-sim run produced a linked `post_adversary_sim` agent run with matching apply-stage lineage in the status/history projection.

7. Rollback quickly if smoke or adversary-sim verification fails:

```bash
make stop
# restore previous known-good artifact/config via your normal deploy mechanism
make prod
make smoke-single-host
```

For immutable deployments, rollback is: redeploy previous release artifact + previous config export (`GET /shuma/admin/config/export` snapshot), then rerun `make smoke-single-host` with `GATEWAY_SURFACE_CATALOG_PATH` (and `SHUMA_SMOKE_FORWARD_PATH` when needed).

## 🐙 Deployment Personas (Provider Scope)

Use one of these operating profiles as your baseline:

| Persona | Who it is for | Provider posture | Edge mode posture | Default recommendation |
| --- | --- | --- | --- | --- |
| `self_hosted_minimal` | Small/self-hosted deployments without managed edge bot tooling | All `provider_backends.*=internal` | `off` | Recommended default for all new installs |
| `enterprise_akamai` (additive) | Enterprise deployments with Akamai edge/Bot Manager telemetry | Start internal, then selectively enable external per capability after validation | `additive` | First enterprise cutover stage |
| `enterprise_akamai` (authoritative) | Mature enterprise deployments with validated external adapters and explicit rollback drills | External only for capabilities with proven parity/SLOs | `authoritative` | Optional, advanced stage only |

Current implementation note:

- `fingerprint_signal=external` now uses an Akamai-first adapter (`/fingerprint-report`) that maps edge/Bot Manager-style outcomes into normalized fingerprint/<abbr title="Chrome DevTools Protocol">CDP</abbr>-tier signals; non-Akamai/legacy payloads are explicitly downgraded to the internal <abbr title="Chrome DevTools Protocol">CDP</abbr> handler path.
- `rate_limiter=external` uses a Redis-backed distributed adapter when `SHUMA_RATE_LIMITER_REDIS_URL` is configured.
  On backend degradation it applies route-class outage posture:
  - main traffic: `SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN` (default `fallback_internal`)
  - admin auth: `SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH` (default `fail_closed`)
- `ban_store=external` uses a Redis-backed distributed adapter when `SHUMA_BAN_STORE_REDIS_URL` is configured.
  On backend degradation or misconfiguration it applies `SHUMA_BAN_STORE_OUTAGE_MODE`:
  - `fallback_internal` (default) permits local fallback reads and deferred local writes.
  - `fail_open` and `fail_closed` do not accept local-only fallback state; admin/operator reads surface unavailability and strict manual writes return `503`.
  - authoritative enterprise multi-instance ban sync requires `SHUMA_BAN_STORE_OUTAGE_MODE=fail_closed`.
- `challenge_engine` and `maze_tarpit` still use explicit unsupported external adapters with safe internal fallback semantics.
- Keep production deployments on internal providers unless you are explicitly exercising a staged integration plan.

### Profile Gate For Distributed State Risk

- `self_hosted_minimal`:
  - local state is acceptable for single-instance operation.
  - distributed sync is not required for baseline correctness.
- `enterprise_akamai`:
  - multi-instance deployments must treat distributed state as a critical-path control.
  - keep rollout in additive posture until rate-limiter atomicity and ban-sync semantics are validated.
  - authoritative ban sync requires `SHUMA_PROVIDER_BAN_STORE=external`, a working `SHUMA_BAN_STORE_REDIS_URL`, and `SHUMA_BAN_STORE_OUTAGE_MODE=fail_closed`.
  - runtime now hard-fails (`503`) when enterprise multi-instance posture is unsafe (for example authoritative mode with local-only rate/ban state).
- One codebase policy:
  - keep one shared policy engine; profile differences should be state backend and precedence choices, not separate policy logic.

### Profile-First Deployment Wrappers

Use wrappers so every path starts from one baseline and layers profile-specific checks:

- Shared baseline:
  - `make deploy-profile-baseline` (config verification + dashboard/runtime build)
- Single-host profile:
  - `make deploy-self-hosted-minimal`
- Enterprise overlay:
  - `make deploy-enterprise-akamai`

## 🐙 Required Env-Only Keys

Set these in your deployment secret/config system:

- `SHUMA_API_KEY`
- `SHUMA_ADMIN_READONLY_API_KEY` (optional; recommended when operators/automation need read-only admin <abbr title="Application Programming Interface">API</abbr> access)
- `SHUMA_JS_SECRET`
- `SHUMA_FORWARDED_IP_SECRET` (required when trusting forwarded headers)
- `SHUMA_HEALTH_SECRET` (recommended; required if you want header-authenticated `/shuma/health`)
- `SHUMA_ADMIN_CONFIG_WRITE_ENABLED`
- `SHUMA_KV_STORE_FAIL_OPEN`
- `SHUMA_ENFORCE_HTTPS`
- `SHUMA_DEBUG_HEADERS`
- `SHUMA_ENTERPRISE_MULTI_INSTANCE` (optional; required for enterprise multi-instance guardrail posture)
- `SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED` (optional; temporary additive/off exception attestation only)
- `SHUMA_RATE_LIMITER_REDIS_URL` (optional generally; required when enterprise multi-instance uses `SHUMA_PROVIDER_RATE_LIMITER=external`)
- `SHUMA_BAN_STORE_REDIS_URL` (optional generally; required when enterprise multi-instance uses `SHUMA_PROVIDER_BAN_STORE=external`)
- `SHUMA_BAN_STORE_OUTAGE_MODE` (optional generally; must be `fail_closed` when enterprise multi-instance uses authoritative external ban sync)
- `SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN` (optional; `fallback_internal|fail_open|fail_closed`)
- `SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH` (optional; `fallback_internal|fail_open|fail_closed`)
- `SHUMA_GATEWAY_UPSTREAM_ORIGIN` (required for `runtime-prod`; `scheme://host[:port]`)
- `SHUMA_GATEWAY_DEPLOYMENT_PROFILE` (`shared-server|edge-fermyon`)
- `SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED` (required `true` for `runtime-prod`)
- `SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED` (required `true` for `runtime-prod`)
- `SHUMA_GATEWAY_TLS_STRICT` (must be `true` in production)

For the full env-only list and per-variable behavior, use [`docs/configuration.md`](configuration.md).
Template source: run `make setup` or `make setup-runtime` and use `.env.local` (gitignored) as your env-only override baseline.

## 🐙 Gateway Outbound Contract (Spin/Fermyon)

Production gateway posture is explicit and fail-closed:

1. `SHUMA_GATEWAY_UPSTREAM_ORIGIN` must use explicit `scheme://host[:port]`.
2. The effective Spin manifest used for deployment must include that exact upstream origin in `component.bot-defence.allowed_outbound_hosts`.
   For the canonical shared-host path, `scripts/deploy/render_gateway_spin_manifest.py` renders a deployment-specific manifest and `SHUMA_SPIN_MANIFEST` points runtime/preflight at that rendered file.
3. Wildcard outbound hosts are not allowed for `runtime-prod`.

Spin limitation reminder:

- Outbound `Host` header cannot be manually overridden by component code, so upstream authority must be accepted as canonical transport authority.
- Preserve original public host context through regenerated forwarded/provenance headers rather than outbound `Host` rewrites.

Outbound pressure governance:

Use Spin runtime config to bound gateway-origin pressure in production.
Add a runtime config file (for example `runtime-config.toml`) with explicit outbound budget values, then start Spin with that file.
Repository baseline example: [`runtime-config.toml.example`](../runtime-config.toml.example).

```toml
[outbound_http]
connection_pooling = true
max_concurrent_requests = 32
```

Tune these values to match origin capacity and expected concurrency envelope.

Gateway v1 protocol support matrix:

| Capability | Gateway v1 status | Behavior |
| --- | --- | --- |
| HTTP request forwarding (`GET`, `HEAD`, `POST`, `PUT`, `PATCH`, `DELETE`, `OPTIONS`) | Supported | Allow-path requests are forwarded to configured upstream origin. |
| Control-plane and enforcement-local routes (`/shuma/admin`, `/shuma/internal`, `/shuma/health`, `/shuma/metrics`, challenge/maze/tarpit internals) | Supported (local ownership) | Always served locally by Shuma; never proxied upstream. |
| Forwarded/provenance headers | Supported (proxy-owned) | Client-supplied `Forwarded`/`X-Forwarded-*` is stripped and regenerated from trusted runtime context only. |
| Redirect handling (`Location`) | Supported with confinement | Relative redirects pass; absolute/scheme-relative redirects must stay in configured upstream authority or are denied fail-closed. |
| Cookie handling (`Set-Cookie`) | Supported with deterministic rewrite | Upstream-domain cookies are rewritten to public host domain when valid; foreign-domain cookies are dropped. |
| Request/response body size | Supported with bounds | Request body capped at 1 MiB and response body capped at 2 MiB; overflow is denied fail-closed. |
| Loop prevention | Supported | Startup authority-collision guard + runtime hop marker budget (`SHUMA_GATEWAY_LOOP_MAX_HOPS`). |
| Upstream HTTP 4xx/5xx pass-through | Supported | Returned to client as upstream outcomes (not transport failures). |
| WebSocket/HTTP upgrade/`CONNECT` tunneling | Unsupported (explicit fail-fast) | Denied with `policy_denied` transport class. |
| Unbounded streaming/trailers/raw tunnel passthrough | Unsupported in v1 | Gateway enforces bounded request/response envelope. |

Gateway onboarding with shared-host discovery outputs:

1. Produce or obtain an origin public-surface catalog artifact from shared-host discovery (`robots.txt`, `sitemap.xml`, bounded crawl outputs).
   - If the site already exists locally as a docroot and no sitemap has been authored, generate the initial artifact with:
     ```bash
     python3 scripts/build_site_surface_catalog.py \
       --docroot /abs/path/to/site/docroot \
       --mode static-html-docroot \
       --output /abs/path/to/.shuma/catalogs/site.surface-catalog.json
     ```

   - This helper inventories the docroot first, then merges local sitemap evidence when present. A human-authored sitemap is not required for the initial deterministic artifact.
2. Run reserved-route collision preflight against that artifact by setting:
   - `GATEWAY_SURFACE_CATALOG_PATH=<catalog-json-path>`
   - `SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true` only after clean preflight.
3. Fix all collisions before cutover (no exceptions for Shuma/Spin-owned route namespaces).
4. Use the same catalog during initial gateway tuning:
   - verify public paths expected by catalog are forwarded,
   - verify sensitive `/shuma/admin/*` and `/shuma/internal/*` routes remain local and non-forwarded,
   - tighten allowlists and origin lock before enabling strict enforcement.

Shared-host same-box note:

- `deploy_linode_one_shot.sh` can now either provision a fresh VM or attach to a prepared Linode instance with `--existing-instance-id`,
- the new setup skill can prepare same-host handoff inputs such as `http://127.0.0.1:8080`,
- same-host origin staging/host preparation automation remains an explicit follow-on item and must not be implied complete until proven.

Gateway cutover and rollback (operator runbook summary):

1. Pre-cutover:
   - validate env guardrails with `make deploy-env-validate`,
   - run gateway profile checks (`make test-gateway-profile-shared-server` and/or `make test-gateway-profile-edge`),
   - run `make smoke-gateway-mode`,
   - run wasm trust-path hardening matrix `make test-gateway-wasm-tls-harness`,
   - optionally run active direct-origin bypass probe:
     - `GATEWAY_PROBE_GATEWAY_URL=https://<public-shuma-host>`
     - `GATEWAY_PROBE_ORIGIN_URL=https://<direct-origin-host>`
     - `make test-gateway-origin-bypass-probe`
     - set `GATEWAY_PROBE_FAIL_ON_INCONCLUSIVE=1` when your environment is expected to permit direct-origin probing and you want inconclusive outcomes to fail.
2. Cutover:
   - route public traffic to Shuma,
   - lock origin ingress to Shuma-only path,
   - if using signed-header origin auth, rotate credentials with overlap-safe rollout and confirm origin rejects stale credentials.
3. Rollback:
   - restore previous edge/DNS route to prior origin path,
   - revert Shuma deployment bundle/env to last known-good release,
   - rotate/disable temporary origin-auth credentials used during failed cutover.

## 🐙 Security Baseline

- Keep `SHUMA_DEBUG_HEADERS=false` in production.
- Keep `SHUMA_ENFORCE_HTTPS=true` in production.
- Keep `SHUMA_ADMIN_CONFIG_WRITE_ENABLED=true` for the normal production posture when you want live operational tuning through the dashboard/shuma/admin <abbr title="Application Programming Interface">API</abbr>. Set it to `false` only when you intentionally want read-only admin config.
- Generate a strong `SHUMA_API_KEY` with `make api-key-generate` (or rotate with `make api-key-rotate`).
- Set `SHUMA_HEALTH_SECRET` and require `X-Shuma-Health-Secret` for `/shuma/health`.
- Restrict `/shuma/admin/*` with `SHUMA_ADMIN_IP_ALLOWLIST` and upstream network controls.
- Apply <abbr title="Content Delivery Network">CDN</abbr>/<abbr title="Web Application Firewall">WAF</abbr> rate limits to `POST /shuma/admin/login` and all `/shuma/admin/*`.

Validation helper before deploy:

```bash
make deploy-env-validate
```

`make deploy-env-validate` enforces:

- `SHUMA_DEBUG_HEADERS=false`
- non-empty and non-overbroad `SHUMA_ADMIN_IP_ALLOWLIST` (rejects wildcard and global-range entries)
- explicit operator attestation that admin edge limits are configured:
  `SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`
- explicit operator attestation that admin <abbr title="Application Programming Interface">API</abbr> key rotation is complete for the deployment cadence:
  `SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`
- enterprise multi-instance state guardrail:
  - when `SHUMA_ENTERPRISE_MULTI_INSTANCE=true`, validate `SHUMA_EDGE_INTEGRATION_MODE` and provider backend values,
  - require `SHUMA_RATE_LIMITER_REDIS_URL` (`redis://...` or `rediss://...`) when `SHUMA_PROVIDER_RATE_LIMITER=external`,
  - require `SHUMA_BAN_STORE_REDIS_URL` (`redis://...` or `rediss://...`) when `SHUMA_PROVIDER_BAN_STORE=external`,
  - block local-only rate/ban state in authoritative mode,
  - require `SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true` for temporary additive/off exceptions when distributed state is not yet enabled.
- gateway outbound contract guardrail (`runtime-prod`):
  - `SHUMA_GATEWAY_UPSTREAM_ORIGIN` must be valid and explicit,
  - upstream origin must be present in `component.bot-defence.allowed_outbound_hosts` in the effective manifest selected by `SHUMA_SPIN_MANIFEST` (or `spin.toml` when `SHUMA_SPIN_MANIFEST` is unset),
  - wildcard outbound entries are rejected,
  - `edge-fermyon` profile rejects variable-templated `allowed_outbound_hosts` entries.
- reserved-route collision preflight guardrail (`runtime-prod`):
  - `GATEWAY_SURFACE_CATALOG_PATH` must point to the discovered origin surface catalog JSON,
  - collisions against Shuma/Spin reserved namespaces fail deployment and write a deterministic local deploy receipt (default: `.spin/deploy/gateway_reserved_route_collision_report.json`),
  - if overriding report location, set `GATEWAY_ROUTE_COLLISION_REPORT_PATH`,
  - after a clean run, `SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED` must be `true`.

### 🐙 Admin Surface Pre-Deploy Checklist

Run this checklist for every production deployment:

1. Admin exposure:
   - Confirm `/shuma/admin/*` is reachable only via trusted ingress (<abbr title="Content Delivery Network">CDN</abbr>/<abbr title="Web Application Firewall">WAF</abbr>/<abbr title="Virtual Private Network">VPN</abbr> path), not open origin exposure.
2. Admin allowlist:
   - Confirm `SHUMA_ADMIN_IP_ALLOWLIST` contains only trusted operator/<abbr title="Virtual Private Network">VPN</abbr> IPs or CIDRs.
   - Confirm no wildcard/global ranges are present.
3. Login and admin edge rate limits:
   - Confirm edge/<abbr title="Content Delivery Network">CDN</abbr> policy exists for `POST /shuma/admin/login` (strict threshold).
   - Confirm edge/<abbr title="Content Delivery Network">CDN</abbr> policy exists for `/shuma/admin/*` (moderate threshold).
   - Set `SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true` in deploy-time environment after verification.
4. App-side auth failure limiter:
   - Confirm `SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE` is set to a conservative value for the environment.
5. <abbr title="Application Programming Interface">API</abbr> key rotation cadence:
   - Rotate `SHUMA_API_KEY` on a regular cadence (recommended 90 days) using `make gen-admin-api-key` / `make api-key-rotate`.
   - Set `SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true` in deploy-time environment after rotation verification.
6. Enterprise multi-instance state posture:
   - For multi-instance enterprise deployments, set `SHUMA_ENTERPRISE_MULTI_INSTANCE=true`.
   - Prefer distributed state backends for both:
     - `SHUMA_PROVIDER_RATE_LIMITER=external`
     - `SHUMA_PROVIDER_BAN_STORE=external`
   - When using `SHUMA_PROVIDER_RATE_LIMITER=external`, set `SHUMA_RATE_LIMITER_REDIS_URL` to a reachable Redis endpoint.
   - When using `SHUMA_PROVIDER_BAN_STORE=external`, set `SHUMA_BAN_STORE_REDIS_URL` to a reachable Redis endpoint.
   - Set explicit outage posture for degraded external rate-limiter behavior:
     - `SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN`
     - `SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH`
   - Do not run local-only rate/ban state with `SHUMA_EDGE_INTEGRATION_MODE=authoritative`.
   - If you must run temporary additive/off posture without distributed state, set `SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true` and track a time-bounded remediation plan.

## 🐙 External Provider Rollout & Rollback Runbook

This runbook is required before enabling any external provider in non-dev environments.

### 1. Prerequisites (Do Not Skip)

- Record a baseline while fully internal (`provider_backends.*=internal`, `edge_integration_mode=off`) for at least one representative traffic window.
- Ensure dashboards include:
  - `bot_defence_provider_implementation_effective_total`
  - `bot_defence_botness_signal_state_total`
  - `bot_defence_edge_integration_mode_total`
  - challenge/block rates and p95 latency from your platform metrics.
- Ensure operators can quickly apply config changes (immutable redeploy or controlled `POST /shuma/admin/config` workflow).
- Ensure rollback authority and on-call ownership are assigned before cutover.

### 2. Staged Cutover Sequence

1. Internal baseline (required):
   - Keep all providers `internal`.
   - Keep `edge_integration_mode=off`.
   - Confirm stable baseline metrics and normal challenge/block behavior.
2. Additive stage (first external stage):
   - Enable one capability at a time, beginning with `fingerprint_signal`.
   - Set `edge_integration_mode=additive`.
   - Keep all other providers internal during this stage.
   - Soak in staging, then production, and confirm expected metrics/outcomes before expanding scope.
3. Authoritative stage (optional):
   - Enter only after additive stage shows stable behavior and clear operational benefit.
   - Set `edge_integration_mode=authoritative` only for capabilities with explicit authoritative semantics and rollback confidence.
   - Maintain safety-critical local controls and admin protections regardless of edge mode.

### 3. Success Gates Per Stage

- Provider selection gate:
  - `bot_defence_provider_implementation_effective_total` shows expected capability/backend/implementation labels.
- Edge mode gate:
  - `bot_defence_edge_integration_mode_total` confirms requested mode (`off`, `additive`, `authoritative`).
- Signal health gate:
  - `bot_defence_botness_signal_state_total{state="unavailable"}` does not spike unexpectedly for enabled external signal paths.
- Outcome gate:
  - challenge/block rates remain within expected variance versus baseline.
  - no unexplained increase in user-facing friction or false positives.

### 4. Rollback Triggers

Trigger immediate rollback when any of the following occurs:

- sustained increase in `state="unavailable"` for an enabled external signal provider,
- sudden challenge/block rate jump not explained by traffic or attack context,
- operational instability (timeouts/errors) attributable to external integration,
- operator confidence loss in explainability of decisions/outcomes.

### 5. Rollback Procedure (Immediate)

1. Set affected `provider_backends` capability back to `internal`.
2. Set `edge_integration_mode=off`.
3. Redeploy/reload config via your standard production change path.
4. Verify post-rollback metrics:
   - provider implementation metric returns to `implementation="internal"` for affected capability,
   - edge integration metric reflects `mode="off"`,
   - challenge/block behavior returns toward baseline.
5. Capture incident notes and defer re-enable until root cause and safeguards are documented.

## 🐙 <abbr title="Content Delivery Network">CDN</abbr>/<abbr title="Web Application Firewall">WAF</abbr> Rate Limits (Cloudflare + Akamai)

Treat this as first-layer abuse control. Keep app-level auth and rate-limiting logic enabled as a second layer.

Recommended baseline policies:

- `POST /shuma/admin/login`: strict limit (start around `5 requests/minute/IP`, burst up to `10`).
- All other `/shuma/admin/*`: moderate limit (start around `60 requests/minute/IP`).
- Exempt trusted operator and monitoring source IPs/CIDRs to avoid self-lockout.

### 🐙 Cloudflare

Use <abbr title="Web Application Firewall">WAF</abbr> Rate Limiting rules with client <abbr title="Internet Protocol">IP</abbr> as the key characteristic.

Suggested rules:

1. Login endpoint:
   - Match expression: `http.request.method eq "POST" and http.request.uri.path eq "/shuma/admin/login"`
   - Initial action: `Managed Challenge` (or `Block` for <abbr title="Application Programming Interface">API</abbr>-only admin workflows)
2. Admin surface:
   - Match expression: `starts_with(http.request.uri.path, "/shuma/admin/") and http.request.uri.path ne "/shuma/admin/login"`
   - Initial action: `Managed Challenge` or `Block` based on your operator <abbr title="User Experience">UX</abbr> requirements

Operational notes:

- Start in monitor/challenge mode, review false positives, then tighten.
- Ensure Cloudflare uses the real client <abbr title="Internet Protocol">IP</abbr> signal from your edge chain.
- Keep `/shuma/admin/*` route protections in place even after app-level distributed limiter work.

### 🐙 Akamai

Use App & <abbr title="Application Programming Interface">API</abbr> Protector rate controls/rate policies keyed by client <abbr title="Internet Protocol">IP</abbr>.

Suggested policies:

1. Login endpoint policy:
   - Match target: path `/shuma/admin/login` + method `POST`
   - Threshold: strict (same baseline as above; tune with observed traffic)
2. Admin surface policy:
   - Match target: path prefix `/shuma/admin/` excluding login
   - Threshold: moderate (same baseline as above; tune with observed traffic)

Operational notes:

- Roll out in alert/monitor mode first, then enforce deny/challenge actions.
- Confirm client <abbr title="Internet Protocol">IP</abbr> restoration (`True-Client-IP`/equivalent) so limits key on users, not intermediate proxies.
- Keep these policies as a permanent first layer; they are not throwaway once distributed app-level limiting is added.

## 🐙 Forwarded Header Trust

When `SHUMA_FORWARDED_IP_SECRET` is set, forwarded client/proto headers are trusted only if request includes:

```http
X-Shuma-Forwarded-Secret: <same secret>
```

Configure your <abbr title="Content Delivery Network">CDN</abbr>/reverse proxy to inject this header.
Also sanitize incoming `X-Forwarded-For` / `X-Real-IP` from untrusted clients and overwrite with edge-observed values.

## 🐙 Health Endpoint Hardening

- `/shuma/health` allows loopback IPs only (`127.0.0.1`, `::1`) after trusted forwarded-<abbr title="Internet Protocol">IP</abbr> extraction.
- For defense in depth, set `SHUMA_HEALTH_SECRET` and require monitors/proxies to send:

```http
X-Shuma-Health-Secret: <same secret>
```

- Strip `X-Shuma-Health-Secret` from public inbound traffic at your edge and only inject it from trusted monitoring/proxy paths.

## 🐙 Fail-Open vs Fail-Closed

`SHUMA_KV_STORE_FAIL_OPEN` controls behavior when <abbr title="Key-Value">KV</abbr> is unavailable:

- `true`: allow requests through (reduced protection)
- `false`: block with server error (stricter posture)

Choose deliberately for your production risk posture.

## 🐙 Outbound Policy

Outbound <abbr title="Hypertext Transfer Protocol">HTTP</abbr>(S) is disabled by default:

```toml
allowed_outbound_hosts = []
```

Only add explicit hosts if a new feature requires outbound calls.
For the canonical shared-host gateway deployment path, keep [`spin.toml`](../spin.toml) as the deny-by-default template and render a deployment-specific manifest with `scripts/deploy/render_gateway_spin_manifest.py`.

## 🐙 Fermyon / Akamai Edge (Deferred Gateway-Only Track)

Use this helper path only when you are explicitly working on the deferred Akamai-edge gateway posture:

```bash
make prepare-fermyon-akamai-edge PREPARE_FERMYON_ARGS="--upstream-origin https://origin.example.com --surface-catalog-path /abs/path/to/catalog.json --origin-lock-confirmed true --reserved-route-collision-check-passed true --admin-edge-rate-limits-confirmed true --admin-api-key-rotation-confirmed true"
make deploy-fermyon-akamai-edge
```

What the helpers do:

- persist `SPIN_AKA_ACCESS_TOKEN` in `.env.local`,
- write `.shuma/fermyon-akamai-edge-setup.json`,
- render a deployment-specific Spin manifest,
- run enterprise edge preflight through the canonical Make targets,
- provision a managed five-job adversary-sim edge cron set,
- bootstrap edge config if the KV is still empty,
- verify adversary-sim generation with both an immediate primed tick and a later cron-driven follow-up,
- run the external live dashboard smoke so edge proof includes:
  - dashboard readiness,
  - Shadow Mode UI convergence,
  - Adversary Sim UI convergence,
  - and monitoring visibility of a fresh simulation event,
- write `.shuma/fermyon-akamai-edge-deploy.json` after a successful `spin aka deploy`.

Current honest boundary:

- this path is `spin aka` only,
- it does not participate in `SHUMA_ACTIVE_REMOTE` or `make remote-*`,
- it is not the current pre-launch home of the dashboard, Scrapling worker, or later diagnosis/recommend/apply agents,
- if `spin aka` PAT login panics, the helper falls back to device login in interactive sessions,
- if device login still ends with `User is not allow-listed!`, treat that as a real provider-access blocker and expect the setup receipt to remain in `status=blocked` form until provider approval is active.

Historical proof and blockers for this deferred path are recorded in:

- [`deferred-edge-gateway.md`](deferred-edge-gateway.md)
- [`research/README.md`](research/README.md)

Example variable wiring for the rendered edge manifest:

```toml
[variables]
shuma_api_key = { default = "" }
shuma_js_secret = { default = "" }
shuma_forwarded_ip_secret = { default = "" }

[component.bot-defence.variables]
shuma_api_key = "{{ shuma_api_key }}"
shuma_js_secret = "{{ shuma_js_secret }}"
shuma_forwarded_ip_secret = "{{ shuma_forwarded_ip_secret }}"
```

### 🐙 Edge-Chain Placement Note (Fermyon)

- Preferred: place Shuma as close to first-hop edge traffic as possible so it can evaluate full request flow and apply low-latency policy.
- If an upstream enterprise edge provider (for example Akamai Bot Manager) sits in front of Shuma, strong bots may be blocked before Shuma sees those requests.
  This is expected and reduces Shuma-visible request-sequence coverage.
- Treat external fingerprinting and Shuma sequence/behavior signals as complementary:
  - external fingerprinting answers identity confidence ("what this client is likely to be"),
  - Shuma request-sequence/timing answers behavioral consistency ("how this client behaves over flow steps").
- In `authoritative` edge mode, strong external fingerprint outcomes may short-circuit into immediate auto-ban when `cdp_auto_ban=true`; keep Shuma policy telemetry enabled so residual/gray traffic remains observable.

## 🐙 Local Dev

`make setup` creates `.env.local`, generates dev secrets, and seeds <abbr title="Key-Value">KV</abbr> defaults.

```bash
make setup
make dev
make api-key-show
```

`make dev` enables local dashboard operation with local-write defaults (`WRITE=true`). Use `DEV_ADMIN_CONFIG_WRITE_ENABLED=false` to simulate an operator-disabled read-only admin-config deployment.
Use `make dev-prod` to keep local watch-mode ergonomics while forcing production runtime posture (`runtime-prod`, `DEBUG_HEADERS=false`) with adversary-sim availability following `SHUMA_ADVERSARY_SIM_AVAILABLE` and admin writes still enabled for local config tuning and persistence checks. This target now runs in explicit local-direct mode, which allows localhost-only `runtime-prod` startup without `SHUMA_GATEWAY_UPSTREAM_ORIGIN`; it is for prod-like local observation only and does not satisfy deployment guardrails.
