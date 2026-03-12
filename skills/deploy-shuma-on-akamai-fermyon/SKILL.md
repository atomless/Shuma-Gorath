---
name: deploy-shuma-on-akamai-fermyon
description: Use when an agent needs to deploy this repository to Fermyon Wasm Functions on Akamai from a prepared Akamai-edge setup receipt.
---

# Deploy Shuma-Gorath On Akamai + Fermyon

## Overview

Use this skill for the deploy-side half of the Akamai-edge-only Fermyon baseline.

This skill is agent-facing. It is not a human checklist.

Use it only after [`../prepare-shuma-on-akamai-fermyon/SKILL.md`](../prepare-shuma-on-akamai-fermyon/SKILL.md) has produced a setup receipt.

Current maturity boundary:

- this skill targets `spin aka` only,
- plain `spin cloud` is out of scope for this tranche,
- the Akamai edge baseline is now live-proven, so follow-on Akamai Rate/GEO work may proceed from this posture.

Production posture is gateway-only (`client -> shuma -> existing origin`) with `edge-fermyon` guardrails.

## Mandatory Input Gate

Do not deploy until the setup receipt exists and these are already real in `.env.local` or the receipt:

Required baseline secrets/hardening:

- `SHUMA_API_KEY`
- `SHUMA_JS_SECRET`
- `SHUMA_FORWARDED_IP_SECRET`
- `SHUMA_HEALTH_SECRET`
- `SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET`
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

Required Akamai/Fermyon auth:

- `SPIN_AKA_ACCESS_TOKEN`
- working `spin aka login`

## Agent Contract

Run the canonical helper instead of narrating raw `spin aka` steps:

```bash
make deploy-fermyon-akamai-edge
```

What the helper does:

- loads `.shuma/fermyon-akamai-edge-setup.json`,
- renders a deployment-specific Spin manifest,
- validates enterprise edge posture through the canonical Make targets,
- attempts to reuse an existing `spin aka` session,
- otherwise attempts non-interactive `spin aka` login and falls back to device login when the known token-login panic occurs in an interactive session,
- runs `spin aka deploy` with explicit account/app targeting,
- provisions a managed five-job adversary-sim cron set so the effective edge cadence is one beat per minute while each individual job still respects Fermyon's five-minute minimum,
- bootstraps config when the edge KV is still empty,
- proves adversary-sim generation live by requiring an immediate primed tick and a later cron-driven follow-up tick,
- writes `.shuma/fermyon-akamai-edge-deploy.json`.

What the helper must not do:

- it must not pretend plain `spin cloud` is part of this path,
- it must not continue after a `spin aka login` panic,
- it must not overload the SSH `remote-*` contract.

## Canonical Command Path

Run from repository root:

```bash
make deploy-fermyon-akamai-edge
```

Notes:

- the helper already runs:
  - `make deploy-enterprise-akamai`
  - `make test-gateway-profile-edge`
  - `make smoke-gateway-mode`
- `make deploy-enterprise-akamai` already includes `make deploy-env-validate`.
- use `DEPLOY_FERMYON_ARGS="--preflight-only"` when you want to stop after guardrails and auth validation.

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

## Honest Boundary

Stop and treat the current run as unproven if either of these is true:

- `spin aka login` fails or panics,
- no real Akamai/Fermyon deploy receipt is written.

If the helper reports the known upstream plugin panic, treat that as an upstream CLI defect and fall back to device login in interactive sessions instead of pretending PAT login worked.
If browser auth succeeds but Fermyon returns `User is not allow-listed!`, treat that as a provider-access blocker, expect the setup receipt to remain in `status=blocked` form, and stop.

## Operations Reference

For troubleshooting, rollback, and failure-mode handling:

- [references/OPERATIONS.md](references/OPERATIONS.md)

## External References

- Fermyon Wasm Functions deploy docs: https://developer.fermyon.com/wasm-functions/deploy
- Fermyon `spin aka` command reference: https://developer.fermyon.com/wasm-functions/aka-command-reference
- Akamai staging test guidance: https://techdocs.akamai.com/ion/docs/test-your-ion-property
