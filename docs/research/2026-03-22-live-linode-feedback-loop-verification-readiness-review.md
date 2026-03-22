# Live Linode Feedback-Loop Verification Readiness Review

Date: 2026-03-22
Status: ready

## Goal

Refresh the proof boundary for the newly landed shared-host recommend-only feedback loop before `MON-OVERHAUL-1`, then execute a truthful live verification against the canonical Linode shared-host deployment path.

## Current evidence state

The repository already has:

1. a historical shared-host Linode live proof for the general deployment path in [`2026-03-06-linode-shared-host-live-proof.md`](./2026-03-06-linode-shared-host-live-proof.md),
2. a current shared-host remote receipt at [`../../.shuma/remotes/dummy-static-site-fresh.json`](../../.shuma/remotes/dummy-static-site-fresh.json),
3. and local focused verification for `OVR-RECON-1` and `OVR-AGENT-1` in [`2026-03-21-ovr-recon-1-recommend-only-reconcile-post-implementation-review.md`](./2026-03-21-ovr-recon-1-recommend-only-reconcile-post-implementation-review.md) and [`2026-03-21-ovr-agent-1-shared-host-agent-loop-post-implementation-review.md`](./2026-03-21-ovr-agent-1-shared-host-agent-loop-post-implementation-review.md).

What is missing is an exact live proof that the recent feedback-loop work is actually running on the Linode shared-host target. The current remote receipts still point at older deployed commits:

1. [`../../.shuma/remotes/dummy-static-site-fresh.json`](../../.shuma/remotes/dummy-static-site-fresh.json) records `last_deployed_commit` `35332bec4841b41a75b5b3ffd4f25275e25fec1a` from `2026-03-14T10:14:26.909827Z`.
2. [`../../.shuma/remotes/dummy-static-site-prod.json`](../../.shuma/remotes/dummy-static-site-prod.json) records `last_deployed_commit` `61c32ded1f9cb447e255e166760233ae5e878cd6` from `2026-03-08T19:46:18.267524Z`.

Those receipts predate `OVR-RECON-1` and `OVR-AGENT-1`, so they cannot be used as proof that the current shared-host feedback loop is live.

## Verification contract required now

The live proof needs to establish all of the following on the Linode target:

1. the selected shared-host remote is updated to the current committed `HEAD`,
2. the running service still satisfies the existing shared-host deploy and runtime-surface contract,
3. the deployed runtime exposes `GET /admin/oversight/agent/status` truthfully on the public admin surface,
4. the shared-host-only internal agent trigger contract can execute a bounded run on the host,
5. a completed adversary-sim run causes the same feedback-loop machinery to record a post-sim agent run,
6. and the resulting latest or recent agent-run state is durably inspectable.

## Gaps in current tooling

The repository currently has:

1. focused local controller gates such as `make test-oversight-agent` and `make test-oversight-post-sim-trigger`,
2. a local running-target gate in `make test-adversary-sim-runtime-surface`,
3. and live remote-smoke patterns such as `make test-remote-edge-signal-smoke` and `make test-dashboard-e2e-external`.

It does not yet have one focused `make` target that proves the current shared-host feedback loop on a live ssh-managed remote. Executing that proof through ad hoc SSH and curl commands would violate the repo's Makefile-first verification workflow and would leave no reusable evidence path for later regressions.

## Recommended tranche

Add one truthful live shared-host feedback-loop gate that:

1. selects the active normalized ssh-managed remote receipt by default,
2. can use SSH loopback transport for internal-only calls on the host,
3. verifies public admin status plus internal supervisor-triggered execution,
4. drives one short adversary-sim run through the public admin control path,
5. waits for a linked post-sim agent run to appear in the status projection,
6. writes a bounded local JSON receipt under `.spin/`,
7. and is then used immediately after `make remote-update` against the selected Linode target.

## Assumptions for execution

1. The active live target is the normalized shared-host Linode remote selected through `.env.local` unless explicitly overridden.
2. The current shared-host target remains the correct proof environment for the first recommend-only agent loop; edge/Fermyon remains out of scope for this tranche.
3. A short live adversary-sim run is acceptable on that target because the production operating contract already treats adversary-sim as a supported operator lane, with no-impact verification required before broader rollout.
