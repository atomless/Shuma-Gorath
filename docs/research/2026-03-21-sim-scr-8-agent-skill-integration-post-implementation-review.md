# SIM-SCR-8 Agent Skill Integration Post-Implementation Review

Date: 2026-03-21
Scope: `SIM-SCR-8-3`

## What Landed

The deploy skill surface now has an explicit Scrapling handoff layer instead of expecting the operator or future agents to reconstruct it ad hoc:

1. `skills/prepare-scrapling-for-deploy/SKILL.md` captures the minimal deploy-time Scrapling contract and the shared-host-first runtime boundary.
2. the shared-host Linode setup/deploy skills now point agents at that skill and explicitly say the common path must not ask for manual scope, seed, or `ADVERSARY_SIM_SCRAPLING_*` env artifacts,
3. the Fermyon/Akamai setup/deploy skills now say plainly that edge deploy does not make the full hosted Scrapling runtime operational.

The skill discovery surfaces in `README.md`, `docs/README.md`, and `docs/deployment.md` were updated as part of the same slice so the new skill is not hidden from future agents.

## Verification

This slice is docs-only (`*.md` only), so behavior tests were intentionally skipped.

Passed:

- `git diff --check`

## Review Findings

### 1. Discoverability gap was real and resolved

The first edit pass updated the skill files but left the new Scrapling deploy skill undiscoverable from the repo’s canonical skill lists.

Resolution:

- added the new skill to `README.md`,
- added it to `docs/README.md`,
- and added it to `docs/deployment.md`.

### 2. No remaining tranche-local shortfall inside `SIM-SCR-8-3`

The skill surface now consistently says:

1. shared-host is the supported full hosted runtime target,
2. the default seed is root-only,
3. gateway catalogs are not the runtime reachable-surface truth,
4. and edge/Fermyon remains a truthful gateway path rather than a first-class hosted worker target.

The remaining open work inside `SIM-SCR-8` is the broader operator/doc closeout in `SIM-SCR-8-4`.
