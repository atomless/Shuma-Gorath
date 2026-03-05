---
name: deploy-shuma-on-akamai-fermyon
description: Use when deploying this repository to Fermyon (`spin aka` or `spin cloud`) with enterprise Akamai edge posture and you need a staged, guardrail-first rollout.
---

# Deploy Shuma-Gorath On Akamai + Fermyon

## Overview

Use this skill for enterprise edge rollout where Akamai and Shuma operate as complementary layers.

This workflow stays Makefile-first:

- validate enterprise posture with existing guardrails,
- select an explicit deployment command family (`aka` or `cloud`) before deploy,
- deploy via the selected Spin command family,
- preserve staged additive-to-authoritative rollout discipline.

Production posture is gateway-only (`client -> shuma -> existing origin`) with edge profile guardrails.

## Mandatory Mode Gate (Required)

You must choose exactly one deployment mode before any deploy action:

- `SHUMA_FERMYON_DEPLOY_MODE=aka` uses Fermyon Wasm Functions on Akamai (`spin aka ...`)
- `SHUMA_FERMYON_DEPLOY_MODE=cloud` uses Fermyon Cloud (`spin cloud ...`)

Do not deploy if this is unset or ambiguous.

Run mode preflight from repository root:

```bash
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
    echo "SHUMA_FERMYON_DEPLOY_MODE must be exactly 'aka' or 'cloud'." >&2
    exit 1
    ;;
esac
```

## Mandatory Input Gate

Do not deploy until these are explicitly set in environment or `.env.local`.

Required baseline secrets/hardening:

- `SHUMA_API_KEY`
- `SHUMA_JS_SECRET`
- `SHUMA_FORWARDED_IP_SECRET`
- `SHUMA_HEALTH_SECRET`
- `SHUMA_ADMIN_IP_ALLOWLIST`
- `SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`
- `SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`

Required enterprise posture:

- `SHUMA_ENTERPRISE_MULTI_INSTANCE=true`
- `SHUMA_EDGE_INTEGRATION_MODE=additive` for first rollout stage

Required gateway posture (edge/Fermyon):

- `SHUMA_GATEWAY_DEPLOYMENT_PROFILE=edge-fermyon`
- `SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://<origin-host[:port]>`
- `SHUMA_GATEWAY_TLS_STRICT=true`
- `SHUMA_GATEWAY_ORIGIN_AUTH_MODE=signed_header`
- `SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME=<token-safe-header-name>`
- `SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE=<secret-value>`
- `SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true`
- `SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true`
- `GATEWAY_SURFACE_CATALOG_PATH=<catalog-json-path>`

Recommended distributed-state posture for enterprise reliability:

- `SHUMA_PROVIDER_RATE_LIMITER=external`
- `SHUMA_PROVIDER_BAN_STORE=external`
- `SHUMA_RATE_LIMITER_REDIS_URL=redis://...` or `rediss://...`
- `SHUMA_BAN_STORE_REDIS_URL=redis://...` or `rediss://...`
- `SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN=fallback_internal|fail_open|fail_closed`
- `SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH=fallback_internal|fail_open|fail_closed`

## Canonical Command Path

Run from repository root:

```bash
# 1) Enterprise posture validation + baseline build (always)
make deploy-enterprise-akamai
make deploy-env-validate
make test-gateway-profile-edge
make smoke-gateway-mode

# 2) Deploy using selected mode
if [ "${SHUMA_FERMYON_DEPLOY_MODE}" = "aka" ]; then
  spin aka login
  spin aka deploy
else
  spin cloud login
  make deploy
fi
```

Notes:

- `make deploy-enterprise-akamai` enforces enterprise-mode guardrails before any deploy command family.
- `make deploy-env-validate` enforces gateway contract + outbound alignment + reserved-route preflight contract.
- `make deploy` reruns API/deployment validation before `spin cloud deploy`.
- `aka` mode uses the same pre-deploy guardrails by running `make deploy-enterprise-akamai` first, then `spin aka deploy`.

## Mandatory Akamai Staging Gate (Before Production)

When traffic is fronted by Akamai Property Manager, production activation must follow this order:

1. Activate the updated property version on Akamai staging.
2. Run staging verification for the Shuma-routed paths and expected headers/behavior.
3. Promote to production only after staging checks pass.

Do not activate production first. This gate is mandatory for every edge rule change and every deploy that changes routed behavior.

Signed-header origin-auth lifecycle requirement:

1. Rotate with overlap-safe rollout (new + old accepted briefly).
2. Flip Shuma injection to new credential.
3. Remove old credential at origin.
4. Confirm stale credential path fails closed before production activation.

## Rollout Stages

1. Stage A: additive (recommended first)
- `SHUMA_EDGE_INTEGRATION_MODE=additive`
- validate behavior and telemetry before stronger posture.

2. Stage B: authoritative (optional, advanced)
- `SHUMA_EDGE_INTEGRATION_MODE=authoritative`
- only after additive stage is stable and distributed-state posture is proven.

## Operations Reference

For troubleshooting, rollback, and failure-mode handling:

- [references/OPERATIONS.md](references/OPERATIONS.md)

## External References

- Fermyon Wasm Functions deploy docs: https://developer.fermyon.com/wasm-functions/deploy
- Fermyon `spin aka` command reference: https://developer.fermyon.com/wasm-functions/aka-command-reference
- Fermyon `spin cloud` command reference: https://developer.fermyon.com/cloud/cloud-command-reference
- Akamai staging test guidance: https://techdocs.akamai.com/ion/docs/test-your-ion-property
