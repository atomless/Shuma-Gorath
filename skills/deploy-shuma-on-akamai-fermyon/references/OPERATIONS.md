# Akamai + Fermyon Deploy Operations

Maturity note:

- this is now a live-proven deploy-path operations guide for the `spin aka` path;
- the Akamai edge baseline is proven, but provider-specific gotchas remain important operational knowledge;
- shared day-2 verbs are still a later design slice because this path does not participate in `ssh_systemd` remote management.

## Preflight Checklist

Run before any deploy:

```bash
spin --version
spin aka --help >/dev/null
spin aka login --help >/dev/null
spin aka deploy --help >/dev/null
test -f .shuma/fermyon-akamai-edge-setup.json
```

The deploy helper itself performs the enterprise guardrails and writes the deploy receipt.

## Deploy Execution

Use the Make target, not raw ad hoc `spin aka` invocations:

```bash
make deploy-fermyon-akamai-edge
```

Optional preflight-only run:

```bash
make deploy-fermyon-akamai-edge DEPLOY_FERMYON_ARGS="--preflight-only"
```

## Required Akamai Staging Property Gate

When Akamai Property Manager is part of the path, production activation must only happen after staging verification:

1. Activate the updated property version on staging.
2. Run functional checks against staging host/path coverage (health/admin/public routes as scoped).
3. Verify expected edge behavior and headers for routed traffic.
4. Activate to production only after staging checks pass.

Minimal staging smoke shape:

```bash
curl -sS -D- -o /dev/null "https://<staging-host>/health"
curl -sS -D- -o /dev/null "https://<staging-host>/dashboard/index.html"
```

## Common Issues

### Enterprise flag/mode mismatch

Symptoms:

- `SHUMA_ENTERPRISE_MULTI_INSTANCE must be true for deploy-enterprise-akamai`
- `SHUMA_EDGE_INTEGRATION_MODE must be additive or authoritative`

Fix:

- set `SHUMA_ENTERPRISE_MULTI_INSTANCE=true`
- set `SHUMA_EDGE_INTEGRATION_MODE=additive` (first rollout stage)

### Distributed-state guardrail failures

Symptoms:

- deployment validation blocks because Redis URLs are missing for external providers.
- authoritative mode blocked with local-only rate/ban state.

Fix:

- set `SHUMA_PROVIDER_RATE_LIMITER=external` and `SHUMA_PROVIDER_BAN_STORE=external`
- set `SHUMA_RATE_LIMITER_REDIS_URL` and `SHUMA_BAN_STORE_REDIS_URL`
- keep authoritative mode disabled until distributed state is proven

### Gateway guardrail failures

Symptoms:

- `make deploy-env-validate` fails with upstream/outbound/collision errors.

Fix:

- verify edge gateway contract values (`SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon`, HTTPS upstream, signed-header origin auth),
- ensure upstream origin authority exists in `spin.toml` `allowed_outbound_hosts`,
- run reserved-route preflight with valid `GATEWAY_SURFACE_CATALOG_PATH`,
- keep `SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true` only when origin lock is actually enforced.

### Akamai/Wasm Functions deploy auth failure

Symptoms:

- `spin aka deploy` fails with auth/permission error.

Fix:

```bash
make prepare-fermyon-akamai-edge PREPARE_FERMYON_ARGS="..."
make deploy-fermyon-akamai-edge
```

If `spin aka login` panics with `plugin/src/commands/login.rs` and `index out of bounds`, treat it as an upstream plugin blocker and stop. Do not fabricate a successful deploy receipt.
In interactive runs, prefer the helper's device-login fallback over raw PAT login retries.

If the helper falls back to device login and the browser finishes with `User is not allow-listed!`, stop there too. The account still is not enabled for Wasm Functions on Akamai, and the setup receipt should remain in `status=blocked` form until access is granted and setup is rerun.

### Runtime starts but immediately panics on empty-string env vars

Symptoms:

- edge app deploys, but `/index.html` and `/admin/config` return `500`
- `spin aka logs` shows panics such as `Invalid integer env var SHUMA_MONITORING_RETENTION_HOURS=`

Fix:

- ensure the deploy helper is loading `config/defaults.env` as well as `.env.local`
- redeploy so defaulted runtime variables are passed explicitly as Spin variables instead of arriving as empty-string manifest defaults

### Fresh edge app returns `500 Configuration unavailable`

Symptoms:

- public and admin routes fail immediately after first deploy
- authenticated `GET /admin/config` returns missing-config `500`

Fix:

- let the deploy helper seed config through `POST /admin/config/bootstrap`
- do not post full seeded config to `POST /admin/config`; that endpoint is the patch surface, not the bootstrap surface

### Akamai staging gate failure

Symptoms:

- staging host does not return expected status/header behavior.
- edge rule pathing is inconsistent across expected routes.

Fix:

- do not promote to production.
- correct Akamai property/config and repeat staging verification.
- promote only when staging checks are clean.

### Origin-auth credential rotation failure

Symptoms:

- origin rejects requests after rotation or accepts stale credential unexpectedly.

Fix:

1. Re-enable overlap-safe old+new acceptance temporarily at origin.
2. Verify Shuma injects expected signed header name/value.
3. Re-run staging verification.
4. Remove stale credential only after confirming new credential path is stable.

### No expected edge signal effects

Checks:

- verify Akamai edge is forwarding the expected headers/payloads.
- verify Shuma trusts forwarded headers via matching `X-Shuma-Forwarded-Secret` and `SHUMA_FORWARDED_IP_SECRET`.
- verify runtime mode and provider posture match intended rollout stage.

## Rollback

Preferred fast rollback (safe posture):

1. set `SHUMA_EDGE_INTEGRATION_MODE=additive` (or `off` if needed).
2. if needed, revert distributed providers to internal:
- `SHUMA_PROVIDER_RATE_LIMITER=internal`
- `SHUMA_PROVIDER_BAN_STORE=internal`
3. redeploy:

```bash
make deploy-fermyon-akamai-edge
```

Important:

- authoritative mode with local-only rate/ban state is blocked by design.
- temporary unsynced exceptions in enterprise mode should be explicit, time-bounded, and removed once distributed backends are ready.
- if a production issue is edge-routing/property related, revert Akamai property to last known-good version and re-verify staging before re-promoting.
- if rollback includes gateway credential rotation issues, revoke temporary/new origin-auth credentials and restore last known-good signed-header key pair before retrying cutover.
