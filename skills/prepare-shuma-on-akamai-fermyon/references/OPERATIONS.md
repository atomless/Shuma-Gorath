# Akamai + Fermyon Setup Operations

## Human Boundary

Do not offload normal setup work onto the operator.

The only hard external prerequisite is:

- the operator already has a Fermyon / Akamai account.

The only additional manual step that may still be required is token creation if `SPIN_AKA_ACCESS_TOKEN` is missing.

Use Fermyon's own language when blocked there:

- action: create a **Personal Access Token**
- repo env key: `SPIN_AKA_ACCESS_TOKEN`

## Canonical Helper

Use the Make target, not ad hoc script invocations:

```bash
make prepare-fermyon-akamai-edge PREPARE_FERMYON_ARGS="--upstream-origin https://origin.example.com --surface-catalog-path /abs/path/to/catalog.json --origin-lock-confirmed true --reserved-route-collision-check-passed true --admin-edge-rate-limits-confirmed true --admin-api-key-rotation-confirmed true"
```

Useful flags:

- `--fermyon-token <token>`
- `--no-store-token`
- `--account-id <id>`
- `--account-name <name>`
- `--app-name <name>`
- `--edge-hostname <hostname>`
- `--staging-hostname <hostname>`
- `--admin-ip <cidr>`
- `--surface-catalog-path <path>`
- `--docroot <path>`
- `--site-mode <mode>`
- `--catalog-output <path>`
- `--yes`

## Persistence Rules

The helper persists local secrets and setup state in the right places:

- `.env.local`
  - `SPIN_AKA_ACCESS_TOKEN`
  - `SHUMA_ADMIN_IP_ALLOWLIST`
  - `SHUMA_GATEWAY_UPSTREAM_ORIGIN`
  - `GATEWAY_SURFACE_CATALOG_PATH`
  - `SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME`
  - `SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE`
  - required Shuma runtime secrets
- `.shuma/fermyon-akamai-edge-setup.json`
  - app/account metadata
  - gateway posture and upstream origin
  - explicit setup attestations
  - surface-catalog path
  - downstream deploy receipt / rendered manifest paths

Never write the raw Fermyon token into the receipt.
`make clean` must not delete those `.shuma` artifacts; use `make reset-local-state` only when you intentionally want to wipe `.spin`.

## Receipt Semantics

The setup receipt is the deploy handoff artifact.

Expected shape:

- `schema=shuma.fermyon.akamai_edge_setup.v2`
- `mode=aka`
- `status=ready|blocked`
- `progress.last_completed_step`
- `progress.blocked_at_step`
- `progress.blocked_reason`
- `progress.next_operator_action`
- `spin.spin_version`
- `spin.aka_plugin_version`
- `fermyon.account_id`
- `fermyon.account_name`
- `fermyon.app_name`
- `gateway.deployment_profile=edge-fermyon`
- `gateway.upstream_origin`
- `gateway.origin_lock_confirmed`
- `gateway.reserved_route_collision_check_passed`
- `gateway.admin_edge_rate_limits_confirmed`
- `gateway.admin_api_key_rotation_confirmed`
- `gateway.surface_catalog_path`
- `artifacts.deploy_receipt_path`
- `artifacts.rendered_manifest_path`

If setup is blocked, the receipt must still be written so the next agent can resume from the recorded blocker instead of rediscovering it.

## Upstream-Origin Rule

`edge-fermyon` posture requires a real HTTPS upstream origin.

Do not continue with:

- `http://...`
- blank origins
- placeholder hostnames that do not exist yet

If the origin is not ready yet, stop after setup and leave the receipt unproven.

## Surface Catalog Rule

- a manual sitemap is not required,
- the initial catalog must still be explicit and frozen,
- use `--surface-catalog-path` when you already have the artifact,
- otherwise use `--docroot` so the helper can build it.

## Admin Allowlist Rule

Preferred default:

- detect the setup machine's current public IP,
- propose `<detected-ip>/32`,
- require confirmation unless `--yes` is explicitly set.

Do not silently assume the detected IP is stable.

## Known Failure Modes

### Missing token in non-interactive mode

Add `SPIN_AKA_ACCESS_TOKEN` to `.env.local` first or rerun interactively so the helper can prompt for it.

### `spin aka login` plugin panic

Symptoms:

- `thread 'main' panicked`
- `plugin/src/commands/login.rs`
- `index out of bounds`

Meaning:

- the upstream `aka` plugin token-login path is broken on this machine/token combination.

Response:

- in interactive mode, allow the helper to fall back to Fermyon device login,
- in non-interactive mode, do not treat the token as accepted,
- do not continue to deploy,
- capture the exact failure in the receipt/logs/docs and leave the edge proof backlog open.

### Device login says `User is not allow-listed!`

Meaning:

- browser authentication succeeded,
- but the account is not yet enabled for Fermyon Wasm Functions on Akamai.

Response:

- verify the Wasm Functions access request has been approved,
- verify the browser login is using the same identity that requested access,
- if the message persists, contact Fermyon support / Discord with the exact text,
- expect the helper to leave `.shuma/fermyon-akamai-edge-setup.json` in `status=blocked` form with `blocked_at_step=auth_validation`,
- do not mark `FERM-SKILL-3` complete until the provider allowlist is enabled.

### Missing or stale account targeting

If the operator knows the intended account, pass `--account-id` or `--account-name` explicitly instead of guessing.

### Non-HTTPS upstream origin

`edge-fermyon` posture must use HTTPS. Fix the upstream first instead of weakening the contract.
