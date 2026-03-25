Date: 2026-03-24
Status: Implemented locally, live redeploy verification pending clean committed follow-on build

Related context:

- [`2026-03-24-rsi-game-mainline-first-working-loop-review.md`](2026-03-24-rsi-game-mainline-first-working-loop-review.md)
- [`../../Makefile`](../../Makefile)
- [`../../scripts/deploy/remote_target.py`](../../scripts/deploy/remote_target.py)
- [`../../scripts/tests/test_prod_start_spin_manifest.py`](../../scripts/tests/test_prod_start_spin_manifest.py)
- [`../../scripts/tests/test_remote_target.py`](../../scripts/tests/test_remote_target.py)
- [`../../docs/testing.md`](../../docs/testing.md)

# Prod-start supervisor env handoff and remote receipt backfill

## Question

Why was the shared-host live Scrapling loop still failing after the remote `.env.local` and Scrapling files were present?

## Conclusion

Two operational gaps were hiding under the same symptom:

1. older normalized remote receipts could still be missing `deploy.scrapling`, so day-2 `remote-update` had no canonical scope/seed metadata to copy forward, and
2. `make prod-start` passed the Scrapling paths and sim-tag secret only through Spin `--env` flags, which do not reach the host-side `run_with_oversight_supervisor.sh` / `run_with_adversary_sim_supervisor.sh` wrapper chain.

That meant the deployed service could look correctly configured on disk while the actual adversary-sim supervisor process still lacked:

- `SHUMA_SIM_TELEMETRY_SECRET`
- `ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH`
- `ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH`
- `ADVERSARY_SIM_SCRAPLING_CRAWLDIR`

## Delivered

1. `scripts/deploy/remote_target.py` now backfills missing `deploy.scrapling` metadata from the canonical Scrapling deploy-prep helper before running a day-2 update, so older remote receipts can still copy the right scope and seed artifacts.
2. `Makefile` now normalizes the host-side Scrapling env keys and threads them through every wrapper-based startup path (`dev`, `dev-closed`, `run`, `run-prebuilt`, `prod-start`) via one shared `SUPERVISOR_HOST_ENV` surface.
3. `scripts/tests/test_prod_start_spin_manifest.py` now proves the exact failure mode: when the Scrapling paths and sim-tag secret are present only in `ENV_LOCAL`, `make prod-start` must still export them into the wrapper child environment.
4. `Makefile` now exposes `make test-prod-start-contract` as the focused verification path for that contract.
5. `scripts/supervisor/adversary_sim_supervisor.rs` now keeps the beat plan's `fulfillment_mode` when it synthesizes a failure payload for a non-zero Scrapling worker exit, and `scripts/tests/test_adversary_sim_supervisor.py` now proves that the host-side supervisor keeps that required field.

## Verification

- `make test-prod-start-contract`
- `make test-remote-target-contract`
- `make test-scrapling-deploy-shared-host`
- `make test-rsi-game-mainline`
- `make test-adversary-sim-scrapling-worker`
- `git diff --check`

## Follow-up truth

The first remote rerun after this fix was not valid evidence because `remote-update` warned that the local worktree was dirty and therefore archived only committed `HEAD`. The next required step is:

1. commit this tranche and the `fulfillment_mode` follow-on,
2. redeploy from a clean worktree,
3. rerun `make test-live-feedback-loop-remote`,
4. then confirm live Scrapling receipts and post-sim Game Loop lineage on the actual fixed build.
