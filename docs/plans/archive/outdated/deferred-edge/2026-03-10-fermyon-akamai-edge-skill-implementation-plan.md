# Fermyon / Akamai Edge Skill Implementation Plan

**Date:** 2026-03-10  
**Status:** Complete; implementation and live proof complete  
**Owners:** Codex / project maintainers

## Goal

Implement the Fermyon / Akamai edge baseline as agent-facing setup and deploy skills, using the same durable local-state pattern as the Linode path while keeping the contract truthful about what is and is not shared with `ssh_systemd` remotes.

This tranche is **Akamai-edge-only**:

- setup and deploy target `spin aka`,
- plain `spin cloud` remains out of scope for this tranche,
- the result should unblock the future Akamai-edge-only operator work once a real proof exists.

## Decisions

1. Do not overload `SHUMA_ACTIVE_REMOTE`.
   - `SHUMA_ACTIVE_REMOTE` remains the selector for normalized `ssh_systemd` receipts under `.shuma/remotes/<name>.json`.
   - Fermyon / Akamai edge gets its own durable receipts and commands.

2. Keep provider-specific setup/deploy separate from future day-2 target verbs.
   - This tranche implements:
     - `make prepare-fermyon-akamai-edge`
     - `make deploy-fermyon-akamai-edge`
   - Shared day-2 verbs across `ssh_systemd` and edge backends are a later design slice after live proof.

3. Store provider setup state under `.shuma/`.
   - Setup receipt: `.shuma/fermyon-akamai-edge-setup.json`
   - Deploy/proof receipt: `.shuma/fermyon-akamai-edge-deploy.json`
   - No raw secrets or personal access tokens may be written into receipts.

4. Persist local operator secrets in `.env.local`.
   - Fermyon PAT is stored under the official CLI env name `SPIN_AKA_ACCESS_TOKEN`.
   - Existing Shuma env-only secrets continue to live in `.env.local`.

5. Setup and deploy must be agent-executable, not human runbooks.
   - Skills point the agent at helpers and receipts.
   - Helpers own plugin/version detection, non-interactive login, preflight, and artifact writing.

## Required Capabilities

### Setup helper

The setup helper must:

- resolve and optionally persist the Fermyon personal access token,
- ensure the `aka` Spin plugin is present,
- validate `spin` + `spin aka` availability,
- attempt non-interactive `spin aka` authentication,
- capture account/workspace info if login succeeds,
- capture or generate deploy-ready Shuma secrets and gateway origin-auth inputs,
- capture `SHUMA_GATEWAY_UPSTREAM_ORIGIN`,
- capture or generate `GATEWAY_SURFACE_CATALOG_PATH`,
- write the durable setup receipt.

### Deploy helper

The deploy helper must:

- consume the setup receipt and `.env.local`,
- render a deployment-specific Spin manifest with the explicit upstream allowlist,
- run:
  - `make deploy-enterprise-akamai`
  - `make deploy-env-validate`
  - `make test-gateway-profile-edge`
  - `make smoke-gateway-mode`
- run `spin aka deploy` with explicit app/account targeting,
- write/update the deploy receipt with deployed app metadata and commit SHA.

## Honest Boundaries

1. This tranche must not pretend Fermyon edge is part of the current `ssh_systemd` remote layer.
2. This tranche must not claim success if `spin aka` token login is broken upstream.
3. If the current `aka` plugin/token path fails, the helper must surface the exact failure clearly and the backlog must remain open for real proof.

## Expected Deliverables

1. New setup helper and Make target.
2. Refactored deploy helper and Make target.
3. New setup skill plus refactored deploy skill.
4. Focused unit tests for:
   - token resolution/persistence
   - plugin detection
   - receipt writing
   - deploy command shaping
5. Documentation updates in deployment and quick-reference docs.
6. TODO/archive updates that reflect the actual verified maturity boundary.

## Proof Standard

To close `FERM-SKILL-1..3`, the project needs:

1. a successful setup receipt written by the agent-facing helper,
2. a successful `spin aka deploy` against a real account/app,
3. captured gotchas folded back into the skills,
4. backlog/archive updates reflecting the real proof status.

## Current Proof Status

- `FERM-SKILL-1` is complete.
- `FERM-SKILL-2` is complete.
- `FERM-SKILL-3` is complete.

Reference:

- [`../../../../research/archive/outdated/deferred-edge/2026-03-12-fermyon-akamai-edge-live-proof.md`](../../../../research/archive/outdated/deferred-edge/2026-03-12-fermyon-akamai-edge-live-proof.md)
- [`../../../../research/archive/outdated/deferred-edge/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md`](../../../../research/archive/outdated/deferred-edge/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md)
