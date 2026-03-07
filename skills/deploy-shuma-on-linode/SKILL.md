---
name: deploy-shuma-on-linode
description: Use when deploying this repository to either a fresh Linode VM or a prepared Linode instance and you need the canonical Shuma bootstrap and production bring-up path.
---

# Deploy Shuma-Gorath On Linode

## Overview

Use the repository-native one-shot deployment path:

- Run local production preflight (`make deploy-env-validate`).
- Render a deployment-specific Spin manifest from the repo template so the live host gets the exact gateway upstream allowlist.
- Build an exact release bundle from the local checked-out git `HEAD`.
- Provision a new Linode instance via Linode API or attach to a prepared Linode instance via `--existing-instance-id`.
- Bootstrap host baseline.
- Upload the release bundle and bootstrap from that bundle on the server.
- Reuse existing runtime workflow (`make setup-runtime`, `make deploy-self-hosted-minimal`, `make smoke-single-host`, `make prod-start`, `make stop`) without introducing a parallel pipeline.
  `make smoke-single-host` now includes forwarded public-path parity against the upstream origin plus reserved-route/admin checks.
- In interactive local runs, treat success as the hosted dashboard loading on the operator machine. Use `--open-dashboard` for that finish line.

Production posture is gateway-only (`client -> shuma -> existing origin`). This path is for existing-site protection, not in-app front-door hosting.

If you are starting from a local site plus a Linode account rather than an already-prepared upstream, use [`../prepare-shared-host-on-linode/SKILL.md`](../prepare-shared-host-on-linode/SKILL.md) first. That setup skill is agent-facing: it captures or validates `LINODE_TOKEN`, proposes `SHUMA_ADMIN_IP_ALLOWLIST`, generates `GATEWAY_SURFACE_CATALOG_PATH`, writes `.spin/linode-shared-host-setup.json`, emits the normalized day-2 remote receipt under `.spin/remotes/<name>.json`, and auto-selects that remote in `.env.local`.

Live proof reference:

- [`../../docs/research/2026-03-06-linode-shared-host-live-proof.md`](../../docs/research/2026-03-06-linode-shared-host-live-proof.md)

## Mandatory Input Gate

Do not provision anything until all required inputs are known and validated.

Required:

- `LINODE_TOKEN`: Linode API token with Linodes read/write scope.
- `SHUMA_ADMIN_IP_ALLOWLIST`: trusted admin IP/CIDR list.
- `SHUMA_GATEWAY_UPSTREAM_ORIGIN`: existing origin in `scheme://host[:port]` form.
- `SHUMA_GATEWAY_DEPLOYMENT_PROFILE=shared-server`.
- `SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true`.
- `SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true` after clean preflight.
- `SHUMA_GATEWAY_TLS_STRICT=true`.
- `SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`.
- `SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`.
- `GATEWAY_SURFACE_CATALOG_PATH`: discovered origin public-surface catalog JSON.
- `--domain <fqdn>`: canonical production path requires domain/TLS from the start.

These may already be satisfied by the setup helper:

- `LINODE_TOKEN` from `.env.local`
- `SHUMA_ADMIN_IP_ALLOWLIST` from `.env.local`
- `GATEWAY_SURFACE_CATALOG_PATH` from `.env.local`
- `--existing-instance-id <linode-id>` from `.spin/linode-shared-host-setup.json`

Prepared same-host rule:

- if you intend to use a same-host internal origin such as `http://127.0.0.1:8080`, the origin service must already be real before calling this path,
- once that prepared Linode host exists, use `--existing-instance-id <linode-id>` so Shuma attaches to it without reprovisioning drift.
- do not pretend the setup skill staged the origin for you; the Shuma attach path starts only once the upstream service is already live.

Recommended:

- `--profile <small|medium|large>` to pick a default host size.
- `--region`, `--label` to control host placement and naming.
- `--type` only when overriding profile defaults.

Preflight-only validation:

```bash
LINODE_TOKEN=<token> \
SHUMA_ADMIN_IP_ALLOWLIST=<ip-or-cidr-list> \
SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://origin.example.com \
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true \
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true \
SHUMA_GATEWAY_TLS_STRICT=true \
SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true \
SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true \
GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json \
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--profile medium --region gb-lon --preflight-only"
```

Use this before first live run and whenever changing region/type/profile. If the setup helper already populated `.env.local`, you only need to supply the still-missing upstream/domain/attestation inputs.

Prepared same-host preflight:

```bash
LINODE_TOKEN=<token> \
SHUMA_ADMIN_IP_ALLOWLIST=<ip-or-cidr-list> \
SHUMA_GATEWAY_UPSTREAM_ORIGIN=http://127.0.0.1:8080 \
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true \
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true \
SHUMA_GATEWAY_TLS_STRICT=true \
SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true \
SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true \
GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json \
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--existing-instance-id 123456 --domain shuma.example.com --preflight-only"
```

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
SHUMA_GATEWAY_UPSTREAM_ORIGIN=https://origin.example.com \
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true \
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true \
SHUMA_GATEWAY_TLS_STRICT=true \
SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true \
SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true \
GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json \
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--domain shuma.example.com --region gb-lon --profile medium"
```

Prepared same-host handoff:

```bash
LINODE_TOKEN=<token> \
SHUMA_ADMIN_IP_ALLOWLIST=<ip-or-cidr-list> \
SHUMA_GATEWAY_UPSTREAM_ORIGIN=http://127.0.0.1:8080 \
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true \
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true \
SHUMA_GATEWAY_TLS_STRICT=true \
SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true \
SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true \
GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json \
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--existing-instance-id 123456 --domain shuma.example.com"
```

Interactive finish line:

```bash
make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--existing-instance-id 123456 --domain shuma.example.com --open-dashboard"
```

If you want the normalized day-2 remote receipt to use a stable friendly name instead of the domain-derived default, add `--remote-name <name>` to the deploy args. Otherwise the successful deploy will auto-select the default name it derived locally.

## Live-Proven Same-Host Pattern

The 2026-03-06 live proof used this pattern successfully:

- build the receipt with `make prepare-linode-shared-host`,
- make the origin real on the prepared host at `http://127.0.0.1:8080`,
- attach with `--existing-instance-id`,
- use a TLS-capable FQDN from the start,
- let `make smoke-single-host` derive the admin-route forwarded IP from `SHUMA_ADMIN_IP_ALLOWLIST`,
- let the auto-selected parity probe prefer a static asset path unless `SHUMA_SMOKE_FORWARD_PATH` must be overridden explicitly.

If the origin ever logs paths that start with `/http://...`, the host is running a pre-`05a0376` build and must be redeployed.

## What The Automation Does

1. Runs local `make deploy-env-validate`.
   It does this against a rendered Spin manifest rather than mutating the repo `spin.toml`.
2. Builds a deployment bundle from the exact local git `HEAD`.
3. Creates a new Linode VM (Ubuntu image by default) or validates the prepared existing instance you named with `--existing-instance-id`.
4. Waits for SSH readiness with your provided private key.
5. Uploads generated `.env.local`, the release bundle, the reserved-route surface catalog, and bootstrap scripts.
6. Runs server bootstrap using existing Makefile targets.
7. Installs/starts a `systemd` unit for persistent runtime.
   The runtime uses `SHUMA_SPIN_MANIFEST=/opt/shuma-gorath/spin.gateway.toml`.
   The smoke run also derives a public forward-probe path from `GATEWAY_SURFACE_CATALOG_PATH`; if that path is too dynamic, rerun with `SHUMA_SMOKE_FORWARD_PATH=/stable/public/path`.
8. Configures Caddy reverse proxy for domain/TLS.
9. Enables firewall rules for SSH and serving ports.
10. Writes or refreshes `.spin/remotes/<name>.json` and auto-selects it in `.env.local` so generic `make remote-*` day-2 operations can take over from provider-specific deploy plumbing.

## Day-2 Handoff

After first successful deploy, routine operations should use the generic remote layer instead of rerunning Linode-specific setup logic:

```bash
make remote-update
make remote-status
make remote-logs
make remote-start
make remote-stop
make remote-open-dashboard
```

The successful deploy already selected the emitted remote locally. Use `make remote-use REMOTE=<name>` later only when you want to switch targets. `remote-update` now ships the exact committed local `HEAD`, preserves the remote `.env.local` and `.spin`, restarts the service, runs smoke, refreshes the receipt metadata, and attempts rollback if smoke fails.

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
ssh -i <private-key> shuma@<server-ip> 'cat /opt/shuma-gorath/.shuma-release.json'
```

If `--domain` was used and TLS is not active yet, confirm DNS points to the new server IP and restart Caddy.

## Operations Reference

For troubleshooting and cleanup procedures, use:

- [references/OPERATIONS.md](references/OPERATIONS.md)
