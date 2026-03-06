# Linode Deploy Operations

## Preflight Checklist

Run this before provisioning.

If the setup helper already ran, `.env.local` may already contain `LINODE_TOKEN`, `SHUMA_ADMIN_IP_ALLOWLIST`, and `GATEWAY_SURFACE_CATALOG_PATH`, and `.spin/linode-shared-host-setup.json` may already contain the instance id plus SSH key paths. Reuse those artifacts instead of re-asking the operator.

Fresh-host preflight:

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

For a prepared same-host Linode instance, run:

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

Run gateway contract guardrails before production cutover:

```bash
make deploy-env-validate
make test-gateway-profile-shared-server
make smoke-gateway-mode
```

If you are starting from a local site rather than an already-running upstream, first run the setup precursor:

- [`../../prepare-shared-host-on-linode/SKILL.md`](../../prepare-shared-host-on-linode/SKILL.md)

Use it to generate `GATEWAY_SURFACE_CATALOG_PATH` from the local docroot and to decide whether the upstream will be external or same-host internal (`http://127.0.0.1:8080`).

The preflight verifies:

- local `make deploy-env-validate`,
- local rendered Spin manifest alignment via `scripts/deploy/render_gateway_spin_manifest.py`,
- Linode token can query API,
- fresh-create mode: region slug, instance type, and image lookup,
- prepared-host mode: the named existing Linode instance exists, is running, and has an IPv4 address.

Gateway guardrails additionally verify:

- upstream origin contract (`SHUMA_GATEWAY_UPSTREAM_ORIGIN`, profile, TLS posture),
- reserved-route collision preflight (`GATEWAY_SURFACE_CATALOG_PATH`),
- origin-lock attestation (`SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true`).
- admin edge-limit attestation (`SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`),
- admin API-key rotation attestation (`SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`).

Post-deploy smoke additionally verifies:

- `/health`, `/metrics`, and `/admin/config` remain Shuma-owned local routes,
- one non-reserved public path matches the direct upstream origin response,
- challenge rendering still responds on the public gateway path.

If the auto-selected public path is too dynamic, rerun smoke with:

```bash
SHUMA_SMOKE_FORWARD_PATH=/stable/public/path GATEWAY_SURFACE_CATALOG_PATH=/abs/path/to/catalog.json make smoke-single-host
```

## Common Issues

### Linode API auth fails

Symptoms:

- script exits with HTTP `401` or `403` during preflight/create.

Checks:

```bash
echo "$LINODE_TOKEN" | wc -c
```

- confirm token has Linodes read/write scope.
- confirm token has not been revoked.

### Region or type validation fails

Symptoms:

- preflight fails before any instance is created on the fresh-create path.

Fix:

- choose a valid region/type for your account.
- rerun with `--region` and/or `--type`.

### Existing instance validation fails

Symptoms:

- preflight fails on `--existing-instance-id` before SSH/bootstrap starts.

Fix:

- confirm the Linode id is correct,
- confirm the instance is running,
- confirm it has a reachable IPv4 address,
- do not combine `--existing-instance-id` with fresh-create flags such as `--profile`, `--region`, `--type`, `--image`, or `--destroy-on-failure`.

### SSH never becomes ready

Symptoms:

- instance created but script times out waiting for SSH.

Checks:

```bash
ssh -i <private-key> shuma@<instance-ip>
```

- confirm local private key matches uploaded public key.
- verify Linode networking/firewall allows SSH.

### Service fails after deploy

Checks:

```bash
ssh -i <private-key> shuma@<instance-ip> 'sudo systemctl status shuma-gorath --no-pager'
ssh -i <private-key> shuma@<instance-ip> 'sudo journalctl -u shuma-gorath -n 200 --no-pager'
```

Potential causes:

- local bundle was built from committed `HEAD` only and local uncommitted changes were not shipped.
- insufficient instance resources for build/start.
- `.env.local` values need adjustment for your environment.
- gateway upstream origin contract misalignment.
- rendered runtime manifest missing or stale (`SHUMA_SPIN_MANIFEST=/opt/shuma-gorath/spin.gateway.toml`).
- auto-selected smoke forward path is too dynamic; rerun with `SHUMA_SMOKE_FORWARD_PATH` set to a stable public asset or page.

### Gateway preflight fails

Symptoms:

- `make deploy-env-validate` fails with gateway contract or collision errors.

Fix:

- correct `SHUMA_GATEWAY_*` env values,
- run reserved-route collision preflight using an updated surface catalog,
- do not cut over traffic until guardrail checks pass.

### Dirty local worktree warning

Symptoms:

- deploy output warns that the local worktree is dirty.

Meaning:

- the VM receives the committed local `HEAD` only, not your uncommitted edits.

Fix:

- commit the exact state you intend to deploy, then rerun the Linode path.

### TLS/Caddy not serving

Checks:

```bash
ssh -i <private-key> shuma@<instance-ip> 'sudo systemctl status caddy --no-pager'
```

- verify DNS A/AAAA points to the Linode public IP.
- restart Caddy after DNS propagation.

## Cutover and Rollback

Cutover:

1. Confirm `make deploy-env-validate` and gateway tests are green.
2. Route public traffic to Shuma.
3. Lock origin ingress to Shuma-only path.

Rollback:

1. Restore previous DNS/edge route.
2. Revert to last known-good deployment bundle.
3. Recheck origin lock and admin protections.

## Cleanup

Use one of these cleanup paths:

1. During failures, run with `--destroy-on-failure` so failed creates are auto-removed.
2. Manual remove:

```bash
curl -X DELETE \
  -H "Authorization: Bearer <LINODE_TOKEN>" \
  "https://api.linode.com/v4/linode/instances/<INSTANCE_ID>"
```
