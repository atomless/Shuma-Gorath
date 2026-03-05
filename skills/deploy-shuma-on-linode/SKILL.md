---
name: deploy-shuma-on-linode
description: Use when deploying this repository to a fresh Linode account or VM and you need one-command provisioning, bootstrap, and first production bring-up.
---

# Deploy Shuma-Gorath On Linode

## Overview

Use the repository-native one-shot deployment path:

- Provision Linode instance via Linode API.
- Bootstrap host baseline.
- Clone this repository on the server.
- Reuse existing runtime workflow (`make setup-runtime`, `make prod`, `make stop`) without introducing a parallel pipeline.

Production posture is gateway-only (`client -> shuma -> existing origin`). This path is for existing-site protection, not in-app front-door hosting.

## Mandatory Input Gate

Do not provision anything until all required inputs are known and validated.

Required:

- `LINODE_TOKEN`: Linode API token with Linodes read/write scope.
- `SHUMA_ADMIN_IP_ALLOWLIST`: trusted admin IP/CIDR list.
- `SHUMA_GATEWAY_UPSTREAM_ORIGIN`: existing origin in `scheme://host[:port]` form.
- `SHUMA_GATEWAY_DEPLOYMENT_PROFILE=shared-server`.
- `SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true`.
- `SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true` after clean preflight.
- `GATEWAY_SURFACE_CATALOG_PATH`: discovered origin public-surface catalog JSON.

Recommended:

- `--domain <fqdn>` for TLS via Caddy.
- `--profile <small|medium|large>` to pick a default host size.
- `--region`, `--label` to control host placement and naming.
- `--type` only when overriding profile defaults.

Preflight-only validation:

```bash
LINODE_TOKEN=<token> \
SHUMA_ADMIN_IP_ALLOWLIST=<ip-or-cidr-list> \
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--profile medium --region gb-lon --preflight-only"
```

Use this before first live run and whenever changing region/type/profile.

Gateway preflight is mandatory before cutover:

```bash
make deploy-env-validate
```

This enforces gateway contract alignment, reserved-route collision preflight, and production lock attestations.

## Deployment Profiles

Default profile mappings in this repository:

| Profile | Default Linode type |
| --- | --- |
| `small` | `g6-nanode-1` |
| `medium` | `g6-standard-1` |
| `large` | `g6-standard-2` |

If you need a different plan, override with `--type <linode-type>`.

## Canonical Command

Run from repository root:

```bash
LINODE_TOKEN=<token> \
SHUMA_ADMIN_IP_ALLOWLIST=<ip-or-cidr-list> \
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--domain shuma.example.com --region gb-lon --profile medium"
```

If you do not set `--domain`, deployment exposes Spin directly on `:3000` and sets `SHUMA_ENFORCE_HTTPS=false`.

## What The Automation Does

1. Creates a new Linode VM (Ubuntu image by default).
2. Waits for SSH readiness with your provided key.
3. Uploads generated `.env.local` runtime secrets and hardening values.
4. Runs server bootstrap using existing Makefile targets.
5. Installs/starts a `systemd` unit for persistent runtime.
6. Optionally configures Caddy reverse proxy for domain/TLS.
7. Enables firewall rules for SSH and serving ports.

## Gateway Cutover and Rollback

Cutover sequence:

1. Run `make deploy-env-validate`.
2. Run `make test-gateway-profile-shared-server` and `make smoke-gateway-mode`.
3. Switch edge/DNS path so client traffic reaches Shuma first.
4. Lock origin ingress so only the Shuma path can reach origin.

Rollback sequence:

1. Restore prior edge/DNS path directly to last known-good origin route.
2. Revert Shuma runtime/env to the last known-good bundle.
3. Reconfirm origin ingress lock and admin path protections after rollback.

## Verification

After deployment, run:

```bash
ssh -i <private-key> shuma@<server-ip> 'sudo systemctl status shuma-gorath --no-pager'
ssh -i <private-key> shuma@<server-ip> 'sudo journalctl -u shuma-gorath -n 200 --no-pager'
```

If `--domain` was used and TLS is not active yet, confirm DNS points to the new server IP and restart Caddy.

## Operations Reference

For troubleshooting and cleanup procedures, use:

- [references/OPERATIONS.md](references/OPERATIONS.md)
