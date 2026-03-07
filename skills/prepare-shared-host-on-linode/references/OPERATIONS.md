# Shared-Host Setup Operations

## Human Boundary

Do not offload normal setup work onto the operator.

The only hard external prerequisite is:

- the operator already has a Linode account.

The only additional manual step that may still be required is token creation in Cloud Manager if `LINODE_TOKEN` is missing.

Use Linode's own language when blocked there:

- page: `API Tokens`
- action: `Create a Personal Access Token`
- repo env key: `LINODE_TOKEN`

## Canonical Helper

Use the Make target, not ad hoc script invocations:

```bash
make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /abs/path/to/site"
```

Useful flags:

- `--site-mode static-html-docroot`
- `--site-mode php-docroot`
- `--existing-instance-id <id>`
- `--remote-name <name>`
- `--admin-ip <cidr>`
- `--yes`
- `--label <linode-label>`
- `--profile <small|medium|large>`
- `--region <linode-region>`
- `--type <linode-plan>`
- `--image <linode-image>`

## Persistence Rules

The helper persists local secrets and setup state in the right places:

- `.env.local`
  - `LINODE_TOKEN`
  - `SHUMA_ADMIN_IP_ALLOWLIST`
  - `GATEWAY_SURFACE_CATALOG_PATH`
- `.spin/linode-shared-host-setup.json`
  - instance id
  - public IPv4
  - SSH key paths
  - catalog path
  - setup mode
- `.spin/remotes/<name>.json`
  - normalized `ssh_systemd` day-2 contract
  - provider kind
  - public base URL
  - service name / SSH transport

Never write the raw Linode token into the receipt.

## Receipt Semantics

The receipt is the deploy handoff artifact.

The normalized remote receipt is the day-2 maintenance artifact for:

- `make remote-use REMOTE=<name>`
- `make remote-status`
- `make remote-logs`
- `make remote-start`
- `make remote-stop`
- `make remote-open-dashboard`

Expected shape:

- `mode`
  - `fresh-instance`
  - `existing-instance`
- `linode.instance_id`
- `linode.public_ipv4`
- `ssh.private_key_path`
- `ssh.public_key_path`
- `catalog_path`
- `admin_allowlist`

If the receipt is missing any of those, treat setup as incomplete.

The normalized remote receipt should additionally contain:

- `identity.backend_kind=ssh_systemd`
- `runtime.service_name=shuma-gorath`
- `runtime.public_base_url`
- `ssh.host`
- `ssh.private_key_path`

## Admin Allowlist Rule

Preferred default:

- detect the setup machine's current public IP,
- propose `<detected-ip>/32`,
- require confirmation unless `--yes` is explicitly set.

Do not silently assume the detected IP is stable.

## Domain Rule

- host-preparation-only steps do not require a domain,
- final canonical Shuma attach still requires `--domain` and TLS.

Use the Linode public IP for SSH access and origin staging before the final attach step.

## Surface Catalog Rule

- a manual sitemap is not required,
- the initial catalog must still be explicit and frozen,
- use local docroot evidence first,
- treat telemetry and Scrapling as later additive discovery.

Static example:

```bash
make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /Users/jamestindall/Projects/dummy_static_site --site-mode static-html-docroot"
```

## Same-Host Truth

This setup flow can:

- create or inspect the Linode host,
- persist deploy-local inputs,
- generate the surface catalog,
- hand off a prepared Linode instance to `deploy-shuma-on-linode`.

It does not yet prove:

- the protected origin service is staged and reachable at the final upstream contract.

If the origin is not real yet, stop after setup and do not claim deployment readiness.

Live proof note:

- the `dummy_static_site` same-host proof used this exact boundary successfully,
- the setup receipt handed off cleanly into `deploy-shuma-on-linode`,
- the origin itself was staged separately and stayed outside current Shuma setup-skill ownership.

If a future agent wants to repeat the static acid test, reuse the same pattern:

- local docroot -> `make prepare-linode-shared-host`,
- prepared Linode instance + same-host origin at `127.0.0.1:8080`,
- final attach through `deploy-shuma-on-linode`.

## Common Failure Modes

### Missing token in non-interactive mode

Add `LINODE_TOKEN` to `.env.local` first or rerun interactively so the helper can prompt for it.

### Wrong docroot

For PHP sites, point at the served docroot such as `public_html`, not the repository root.

### Over-trusting the detected admin IP

If the operator uses VPN, office egress, or unstable residential IPs, require an explicit replacement CIDR.

### Treating receipt creation as proof of deployability

The receipt proves host/setup readiness only. It does not prove the upstream origin is live.
