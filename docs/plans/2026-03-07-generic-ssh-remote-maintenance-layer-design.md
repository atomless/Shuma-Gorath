# Generic SSH Remote Maintenance Layer Design

**Goal:** Add a provider-agnostic day-2 remote maintenance layer for Shuma deployments that already satisfy a normalized SSH + `systemd` host contract.

**Architecture:** Keep provider-specific setup and first deploy flows responsible for creating a remote that satisfies the contract, then hand off to a generic maintenance layer that reads a normalized gitignored receipt under `.spin/remotes/<name>.json`. Keep `.env.local` limited to selecting the active remote target (`SHUMA_ACTIVE_REMOTE=<name>`) plus normal env-only secrets. Do not pretend non-SSH platforms share identical lifecycle semantics in this tranche.

**Tech Stack:** Makefile orchestration, Python helper/receipt management, SSH, `systemd`, existing release-bundle shipping path, gitignored local state under `.spin/`.

---

## Context

The repository already has the correct foundational pattern for remote operations:

1. local operator state is read from `.env.local`,
2. provider-specific setup writes a machine-readable receipt,
3. deployment normalizes the remote runtime onto a `systemd` service,
4. the runtime is already started and stopped through canonical Make targets on the host.

Today, that pattern is Linode-specific:

- `.env.local` is the local operator env source,
- `.spin/linode-shared-host-setup.json` stores provider-specific handoff data,
- Linode deploy installs a `systemd` unit with:
  - `ExecStart=/usr/bin/make prod-start`
  - `ExecStop=/usr/bin/make stop`

This design generalizes the day-2 operations layer without generalizing provider setup prematurely.

## Decisions

1. The first generic backend is `ssh_systemd`.
2. `.env.local` stores only the active remote selector:
   - `SHUMA_ACTIVE_REMOTE=<name>`
3. Structured remote target state lives in gitignored receipts:
   - `.spin/remotes/<name>.json`
4. Provider-specific setup/deploy writers must emit the same normalized receipt schema, with optional provider extension fields.
5. Successful provider-specific setup/deploy writers should also auto-activate the emitted receipt locally by updating `SHUMA_ACTIVE_REMOTE`, while preserving `make remote-use REMOTE=<name>` as the explicit switch command.
6. Generic maintenance commands must consume the normalized receipt, not provider-specific files.
7. The canonical day-2 command set is:
   - `make remote-use REMOTE=<name>`
   - `make remote-update`
   - `make remote-start`
   - `make remote-stop`
   - `make remote-status`
   - `make remote-logs`
   - `make remote-open-dashboard`
8. `make remote-update` must mean:
   - build the exact local committed `HEAD` release bundle,
   - upload/install it on the selected remote,
   - restart the remote service,
   - run smoke against that remote,
   - update receipt metadata.
9. `remote-update` must not imply syncing arbitrary uncommitted worktree state.
10. Do not add ambiguous generic names such as `make dev-remote` or `make dev-prod-remote` in this tranche.
11. Fermyon and other non-SSH backends are explicitly out of scope for the first generic maintenance layer.

## Problem Statement

Once a remote deployment succeeds, operators need a low-friction way to:

- select that remote for ongoing use,
- redeploy local code changes,
- start and stop Shuma on the host,
- inspect runtime status and logs,
- reopen the hosted dashboard.

Today, those capabilities are tied to provider-specific setup/deploy flows rather than exposed as a stable provider-agnostic day-2 surface. That creates unnecessary friction once the host already exists and the provider-specific provisioning work is complete.

## Normalized Remote Receipt Contract

The normalized receipt lives at:

- `.spin/remotes/<name>.json`

Required top-level shape:

```json
{
  "schema": "shuma.remote_target.v1",
  "identity": {
    "name": "blog-prod",
    "backend_kind": "ssh_systemd",
    "provider_kind": "linode"
  },
  "ssh": {
    "host": "203.0.113.10",
    "port": 22,
    "user": "shuma",
    "private_key_path": "/Users/example/.ssh/shuma-linode"
  },
  "runtime": {
    "app_dir": "/opt/shuma-gorath",
    "service_name": "shuma-gorath",
    "public_base_url": "https://blog.example.com"
  },
  "deploy": {
    "spin_manifest_path": "/opt/shuma-gorath/spin.gateway.toml",
    "surface_catalog_path": "/opt/shuma-gorath/site.surface-catalog.json",
    "smoke_path": "/health"
  },
  "metadata": {
    "last_deployed_commit": "",
    "last_deployed_at_utc": ""
  },
  "provider": {
    "linode": {
      "instance_id": 123456
    }
  }
}
```

Rules:

- generic remote-maintenance code may read only the normalized contract plus backend-kind-specific fields it owns,
- provider-specific fields must live under a provider extension block,
- raw provider tokens must never be persisted in the receipt,
- secret env values remain in `.env.local` or other local secret sources, not in receipts.

## Active Remote Selection

The active remote selector is the only remote-target key that belongs in `.env.local`:

```dotenv
SHUMA_ACTIVE_REMOTE=blog-prod
```

`make remote-use REMOTE=<name>` should:

1. validate that `.spin/remotes/<name>.json` exists,
2. validate the receipt schema/version,
3. upsert `SHUMA_ACTIVE_REMOTE=<name>` into `.env.local`,
4. print the selected target summary.

This keeps operator selection friction low while avoiding structured state drift inside `.env.local`.

Successful provider setup/deploy flows should also call that same activation path once they have written a valid normalized receipt, so the happy path leaves day-2 operations ready with no extra manual step.

## Generic Command Semantics

### `make remote-update`

Truthful meaning:

1. resolve the active remote receipt,
2. require `backend_kind=ssh_systemd`,
3. build the exact local committed `HEAD` release bundle,
4. upload bundle and any required deployment artifacts,
5. install/update the remote runtime payload,
6. restart the `systemd` service,
7. run smoke against the remote public base URL,
8. update `metadata.last_deployed_commit` and `metadata.last_deployed_at_utc`.

Non-goals:

- do not sync uncommitted local worktree state,
- do not reprovision provider infrastructure,
- do not mutate provider account configuration.

### `make remote-start`

Run the normalized service start operation for the active `ssh_systemd` target:

- `sudo systemctl start <service_name>`

### `make remote-stop`

Run the normalized service stop operation for the active `ssh_systemd` target:

- `sudo systemctl stop <service_name>`

### `make remote-status`

Show a concise runtime summary:

- `sudo systemctl status <service_name> --no-pager`

### `make remote-logs`

Show recent logs:

- `sudo journalctl -u <service_name> -n 200 --no-pager`

### `make remote-open-dashboard`

Open:

- `<public_base_url>/dashboard`

on the operator machine.

## Backend Dispatch

The generic helper must dispatch by `identity.backend_kind`.

Initial supported backend:

- `ssh_systemd`

Out of scope in this tranche:

- `fermyon_cloud`
- `ssh_non_systemd`
- container-orchestrated targets
- multiple simultaneous active remotes

Reason:

- `start` / `stop` / `logs` semantics are honest and stable for `ssh_systemd`,
- pretending those same commands mean the same thing on Fermyon would violate the repository’s truth-in-naming rule.

## Linode Integration

The existing Linode setup/deploy path should become one provider-specific writer of the generic receipt:

1. Linode setup may continue to write its provider receipt.
2. Linode deploy should also emit `.spin/remotes/<name>.json` in normalized form.
3. The normalized receipt should be the source of truth for day-2 commands.
4. Linode-specific skills remain responsible for setup/provisioning, not for all future remote maintenance.

This preserves the current provider-specific deployment work while removing provider lock-in from routine operations.

## Security and State Handling

1. Do not persist provider tokens in remote receipts.
2. Do not copy all of `.env.local` to remote-maintenance receipts.
3. Keep SSH key paths local-only.
4. Treat the receipt as operational metadata, not as a secret store.
5. Require explicit schema validation before acting on a receipt.
6. Keep remote-update immutable with respect to source content:
   - exact committed `HEAD`,
   - no implicit dirty-worktree sync,
   - no hidden Git fetch/clone on the server.

## Verification Expectations

Implementation of this design must prove:

1. `make remote-use REMOTE=<name>` updates `.env.local` and fails cleanly for missing/invalid receipts.
2. provider-specific writers can emit the normalized receipt without losing current deployment capabilities.
3. `make remote-update` uses the exact committed local `HEAD`, not uncommitted content.
4. `make remote-start`, `make remote-stop`, `make remote-status`, and `make remote-logs` operate solely from normalized `ssh_systemd` receipt data.
5. `make remote-open-dashboard` opens the selected remote’s hosted dashboard URL locally.
6. docs and skills describe the boundary truthfully:
   - provider-specific setup/deploy creates the target,
   - generic remote maintenance operates it afterward.

## Non-Goals

1. Do not solve multi-remote fleet orchestration.
2. Do not add concurrent active remote support.
3. Do not support every hosting provider in the first slice.
4. Do not unify non-SSH lifecycle semantics under misleading generic target names.
5. Do not replace provider-specific deploy/setup skills; only separate them from day-2 maintenance.

## Recommended First Implementation Slice

1. Add the normalized remote receipt schema and loader.
2. Add `make remote-use`.
3. Add `ssh_systemd` backend helper for:
   - `remote-status`
   - `remote-logs`
   - `remote-open-dashboard`
4. Update Linode deploy to emit the normalized receipt.
5. Add `remote-start` and `remote-stop`.
6. Add `remote-update` last, once receipt loading and SSH lifecycle control are already stable.
