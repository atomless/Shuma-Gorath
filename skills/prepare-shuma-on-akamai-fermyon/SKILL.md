---
name: prepare-shuma-on-akamai-fermyon
description: Use when an agent needs to turn an existing Fermyon/Akamai edge account plus a real upstream origin into a deploy-ready Akamai-edge handoff receipt for Shuma.
---

# Prepare Shuma On Akamai + Fermyon

## Overview

This skill is agent-facing. It is not a human checklist.

Use it before [`../deploy-shuma-on-akamai-fermyon/SKILL.md`](../deploy-shuma-on-akamai-fermyon/SKILL.md) when you need the Akamai-edge-only setup half of the Fermyon baseline.

If the operator is asking for the full hosted Scrapling adversary runtime, stop and route that work to the shared-host path instead:

- [`../prepare-scrapling-for-deploy/SKILL.md`](../prepare-scrapling-for-deploy/SKILL.md)
- [`../deploy-shuma-on-linode/SKILL.md`](../deploy-shuma-on-linode/SKILL.md)

This tranche is intentionally narrow:

- target `spin aka` only,
- keep `SHUMA_ACTIVE_REMOTE` untouched,
- write durable Akamai/Fermyon receipts under `.shuma/`,
- do not pretend plain Fermyon Cloud is part of this path.
- do not pretend this path makes the full Scrapling worker runtime operational.

The operator boundary is intentionally small:

- the operator already has a Fermyon / Akamai account,
- if `SPIN_AKA_ACCESS_TOKEN` is not already present in `.env.local`, the operator must create a Fermyon **Personal Access Token** and either:
  - paste it into the helper prompt, or
  - add it to gitignored `.env.local`.

Everything else should be agent work.

## Agent Contract

Run the canonical helper instead of narrating manual steps:

```bash
make prepare-fermyon-akamai-edge PREPARE_FERMYON_ARGS="--upstream-origin https://origin.example.com --surface-catalog-path /abs/path/to/catalog.json --origin-lock-confirmed true --reserved-route-collision-check-passed true --admin-edge-rate-limits-confirmed true --admin-api-key-rotation-confirmed true"
```

What the helper does:

- validates or captures `SPIN_AKA_ACCESS_TOKEN`,
- stores local secrets and handoff env values in `.env.local`,
- ensures `spin` and the `aka` plugin are available,
- attempts non-interactive `spin aka` authentication,
- if the known token-login plugin panic occurs and the session is interactive, falls back to Fermyon's device-login flow,
- captures account metadata when the plugin can authenticate,
- captures or generates deploy-ready Shuma secrets and origin-auth inputs,
- captures `SHUMA_GATEWAY_UPSTREAM_ORIGIN`,
- captures or generates `GATEWAY_SURFACE_CATALOG_PATH`,
- writes a provider setup receipt to `.shuma/fermyon-akamai-edge-setup.json`.

For Scrapling/runtime questions, the agent should keep the boundary truthful:

- gateway catalogs remain gateway artifacts,
- the default Scrapling seed remains the normalized public root URL,
- shared-host is the current supported full runtime target.

What the helper must not do:

- it must not store the raw Fermyon token in the receipt,
- it must not claim success if `spin aka login` panics or fails,
- it must not overload the SSH `remote-*` contract,
- it must not accept a non-HTTPS upstream origin for `edge-fermyon` posture,
- it must not imply that edge setup alone satisfies the hosted Scrapling runtime contract.

## Output Contract

After a successful run, expect:

- `.env.local` contains:
  - `SPIN_AKA_ACCESS_TOKEN`
  - `SHUMA_ADMIN_IP_ALLOWLIST`
  - `SHUMA_GATEWAY_UPSTREAM_ORIGIN`
  - `GATEWAY_SURFACE_CATALOG_PATH`
  - `SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME`
  - `SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE`
  - required Shuma runtime secrets such as `SHUMA_API_KEY`, `SHUMA_JS_SECRET`, `SHUMA_FORWARDED_IP_SECRET`, `SHUMA_HEALTH_SECRET`, `SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET`, and `SHUMA_SIM_TELEMETRY_SECRET`
- `.shuma/fermyon-akamai-edge-setup.json` contains:
  - `status=ready` or `status=blocked`
  - resumable progress metadata (`last_completed_step`, `blocked_at_step`, `blocked_reason`, `next_operator_action`)
  - app/account targeting metadata
  - upstream origin and gateway posture
  - explicit guardrail attestations
  - surface-catalog path
  - deploy receipt and rendered manifest paths

This path does **not** create or select an SSH remote receipt. Fermyon/Akamai edge remains outside the current `ssh_systemd` day-2 layer.

## Canonical Usage

Using an existing frozen catalog:

```bash
make prepare-fermyon-akamai-edge PREPARE_FERMYON_ARGS="--upstream-origin https://origin.example.com --surface-catalog-path /abs/path/to/catalog.json --origin-lock-confirmed true --reserved-route-collision-check-passed true --admin-edge-rate-limits-confirmed true --admin-api-key-rotation-confirmed true"
```

Using a local docroot to generate the catalog:

```bash
make prepare-fermyon-akamai-edge PREPARE_FERMYON_ARGS="--upstream-origin https://origin.example.com --docroot /abs/path/to/site --site-mode static-html-docroot --origin-lock-confirmed true --reserved-route-collision-check-passed true --admin-edge-rate-limits-confirmed true --admin-api-key-rotation-confirmed true"
```

Optional explicit targeting:

```bash
make prepare-fermyon-akamai-edge PREPARE_FERMYON_ARGS="--upstream-origin https://origin.example.com --surface-catalog-path /abs/path/to/catalog.json --account-id acc_123 --app-name shuma-edge-prod"
```

## Honest Boundary

Stop after setup if either of these is still false:

- the upstream origin is not yet real and reachable over HTTPS,
- `spin aka login` cannot authenticate cleanly on this machine.

If the helper reports the known upstream plugin panic:

- do not fabricate account metadata,
- use the interactive device-login fallback instead of pretending PAT login succeeded,
- keep the failure visible in the setup receipt and operator notes.

If device login completes browser auth but returns `User is not allow-listed!`:

- treat that as a provider-access blocker, not as a repo bug,
- expect the setup receipt to be left behind in `status=blocked` form with the exact blocker and rerun instruction,
- wait for Wasm Functions access approval or support intervention,
- then rerun the helper.

The live edge baseline is now proven, so a successful rerun after provider approval should continue cleanly into `make deploy-fermyon-akamai-edge`.

## Operations Reference

Use [`references/OPERATIONS.md`](references/OPERATIONS.md) for receipt semantics, helper flags, and common failure modes.
