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
- writes a receipt to `.spin/linode-shared-host-setup.json`.

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

Treat the receipt as the handoff artifact for the deploy skill.

## Canonical Usage

Fresh-instance setup using the first static acid test:

```bash
make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /Users/jamestindall/Projects/dummy_static_site --site-mode static-html-docroot"
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

## Operations Reference

Use [`references/OPERATIONS.md`](references/OPERATIONS.md) for receipt semantics, helper flags, and common failure modes.
