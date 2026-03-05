# Akamai + Fermyon Deploy Operations

## Preflight Checklist

Run before any deploy:

```bash
export SHUMA_FERMYON_DEPLOY_MODE=aka   # or cloud

spin --version
case "${SHUMA_FERMYON_DEPLOY_MODE:-}" in
  aka)
    spin aka --help >/dev/null
    spin aka login --help >/dev/null
    spin aka deploy --help >/dev/null
    ;;
  cloud)
    spin cloud --help >/dev/null
    spin cloud login --help >/dev/null
    spin cloud deploy --help >/dev/null
    ;;
  *)
    echo "SHUMA_FERMYON_DEPLOY_MODE must be aka or cloud" >&2
    exit 1
    ;;
esac

make deploy-enterprise-akamai
```

This checks enterprise posture and fails fast when guardrails are not satisfied.

## Deploy Execution

Use exactly one command family per run:

```bash
if [ "${SHUMA_FERMYON_DEPLOY_MODE}" = "aka" ]; then
  spin aka login
  spin aka deploy
else
  spin cloud login
  make deploy
fi
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

### Mode mismatch / wrong command family

Symptoms:

- `spin aka ...` fails in a cloud-only setup.
- `spin cloud ...` fails in an Akamai/Wasm Functions setup.

Fix:

- set `SHUMA_FERMYON_DEPLOY_MODE` to exactly one value: `aka` or `cloud`.
- run only the matching command family for that deploy run.

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

### Cloud deploy auth failure

Symptoms:

- `make deploy` fails at `spin cloud deploy` with auth/permission error.

Fix:

```bash
spin cloud login
make deploy
```

### Akamai/Wasm Functions deploy auth failure

Symptoms:

- `spin aka deploy` fails with auth/permission error.

Fix:

```bash
spin aka login
spin aka deploy
```

### Akamai staging gate failure

Symptoms:

- staging host does not return expected status/header behavior.
- edge rule pathing is inconsistent across expected routes.

Fix:

- do not promote to production.
- correct Akamai property/config and repeat staging verification.
- promote only when staging checks are clean.

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
make deploy
```

Important:

- authoritative mode with local-only rate/ban state is blocked by design.
- temporary unsynced exceptions in enterprise mode should be explicit, time-bounded, and removed once distributed backends are ready.
- if a production issue is edge-routing/property related, revert Akamai property to last known-good version and re-verify staging before re-promoting.
