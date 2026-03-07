---
name: prepare-shared-host-on-linode
description: Use when an agent needs to turn a local site plus an existing Linode account into a deploy-ready shared-host receipt ahead of Shuma cutover.
---

# Prepare A Shared Host On Linode

## Overview

This skill is agent-facing. It is not a human checklist.

Use it before [`../deploy-shuma-on-linode/SKILL.md`](../deploy-shuma-on-linode/SKILL.md) when you start from:

- a local site on disk,
- an operator who already has a Linode account,
- and no ready-made shared-host deployment receipt yet.

The human boundary is intentionally small:

- the operator must already have a Linode account,
- if `LINODE_TOKEN` is not already present in `.env.local`, the operator must create a Linode **Personal Access Token** in Cloud Manager and either:
  - paste it into the helper prompt, or
  - add it to gitignored `.env.local`.

Everything else should be agent work.

Live proof reference:

- [`../../docs/research/2026-03-06-linode-shared-host-live-proof.md`](../../docs/research/2026-03-06-linode-shared-host-live-proof.md)

## Agent Contract

Run the canonical helper instead of narrating manual steps:

```bash
make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /abs/path/to/site --site-mode static-html-docroot"
```

What the helper does:

- validates or captures `LINODE_TOKEN`,
- stores local secrets and handoff env values in `.env.local`,
- detects the current public IP and proposes `<ip>/32` for `SHUMA_ADMIN_IP_ALLOWLIST`,
- ensures a dedicated SSH keypair exists,
- creates a fresh Linode instance or inspects an existing one,
- builds `GATEWAY_SURFACE_CATALOG_PATH` from the local docroot,
- writes a provider setup receipt to `.spin/linode-shared-host-setup.json`,
- writes a normalized day-2 remote receipt to `.spin/remotes/<name>.json`.

What the helper must not do:

- it must not store the raw Linode token in the receipt,
- it must not pretend same-host origin staging is complete if `SHUMA_GATEWAY_UPSTREAM_ORIGIN` is not yet real,
- it must not require a human-authored sitemap.

## Output Contract

After a successful run, expect:

- `.env.local` contains:
  - `LINODE_TOKEN`
  - `SHUMA_ADMIN_IP_ALLOWLIST`
  - `GATEWAY_SURFACE_CATALOG_PATH`
- `.spin/linode-shared-host-setup.json` contains:
  - Linode instance id
  - Linode public IPv4
  - SSH key paths
  - catalog path
  - setup mode (`fresh-instance` or `existing-instance`)
- `.spin/remotes/<name>.json` contains the normalized `ssh_systemd` contract for later `make remote-*` day-2 operations.

Treat the provider setup receipt as the deploy handoff artifact, and the normalized remote receipt as the future day-2 operations artifact.

## Canonical Usage

Fresh-instance setup using the first static acid test:

```bash
make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /Users/jamestindall/Projects/dummy_static_site --site-mode static-html-docroot"
```

If you want the day-2 remote to use a stable friendly name instead of the Linode label, set it explicitly:

```bash
make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /abs/path/to/site --remote-name blog-prod"
```

Prepared-instance attach:

```bash
make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /abs/path/to/site --existing-instance-id 123456 --admin-ip 203.0.113.10/32"
```

Non-interactive acceptance of the detected admin IP:

```bash
make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /abs/path/to/site --yes"
```

## Handoff Rule

Stop after setup if the origin is not yet real.

Only hand off to the deploy skill when all of these are true:

- the receipt exists,
- the Linode instance is known,
- the upstream origin contract is real,
- the final public domain/FQDN is known for canonical Shuma attach.

The live `dummy_static_site` proof confirmed this boundary is correct:

- setup creates the receipt and catalog,
- deploy attaches to that receipt,
- origin staging remains a separate site-specific step and must already be complete before Shuma attach.

## Operations Reference

Use [`references/OPERATIONS.md`](references/OPERATIONS.md) for receipt semantics, helper flags, and common failure modes.
