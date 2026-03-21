# SIM-SCR-8 Shared-Host Deploy Automation Post-Implementation Review

Date: 2026-03-21
Scope: `SIM-SCR-8-1` and `SIM-SCR-8-2`

## What Landed

The shared-host deploy path now makes the Scrapling runtime deployable from the same agent-facing baseline as the rest of the Linode workflow:

1. `scripts/deploy/scrapling_deploy_prep.py` and `scripts/prepare_scrapling_deploy.py` infer the scope fence, root-only seed inventory, runtime env mapping, and deployment receipt from the canonical public base URL.
2. `scripts/deploy_linode_one_shot.sh` now runs that helper automatically, uploads the generated scope and seed artifacts, writes `ADVERSARY_SIM_SCRAPLING_*` env values into the deployed overlay, and stores enough Scrapling metadata in the normalized remote receipt for later day-2 operations.
3. `scripts/deploy/remote_target.py` now treats `deploy.scrapling` as an optional normalized receipt extension and preserves the same scope/seed artifacts during `make remote-update`.

## Verification

Passed:

- `make test-adversarial-python-unit`
- `make test-scrapling-deploy-shared-host`
- `git diff --check`

## Review Findings

### 1. Original verification target was not truthful for this tranche

The first closeout pass showed that `make test-adversarial-python-unit` did not exercise the Linode deploy or remote-update receipt contracts at all, so it could not prove the new shared-host Scrapling wiring.

Resolution:

- added the focused `make test-scrapling-deploy-shared-host` target covering:
  - `scripts/tests/test_scrapling_deploy_prep.py`
  - `scripts/tests/test_remote_target.py`
  - `scripts/tests/test_deploy_linode_one_shot.py`

This closes the tranche-local verification gap.

### 2. No remaining tranche-local architecture shortfall

The resulting implementation matches the intended design:

1. the deploy-time seed remains root-only by default,
2. shared-host deploy now infers and carries the Scrapling runtime artifacts automatically,
3. and the normalized day-2 `ssh_systemd` receipt keeps that behavior intact through later updates.

The remaining open work inside `SIM-SCR-8` is higher-level operator and skill/documentation closeout in `SIM-SCR-8-3` and `SIM-SCR-8-4`.
