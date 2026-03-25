# Scrapling supervisor worker-output surfacing post-implementation review

Date: 2026-03-24

## Delivered

- Updated [`scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs) so the host-side Scrapling supervisor now pipes bounded worker `stdout`/`stderr` and threads a compact summary into synthetic transport failures when the Python worker exits non-zero or produces no readable result file.
- Updated [`scripts/tests/test_adversary_sim_supervisor.py`](../../scripts/tests/test_adversary_sim_supervisor.py) so the focused supervisor contract gate now proves the failure path captures bounded worker stdio rather than discarding it.

## Why this tranche was necessary

- The live shared-host proof was still failing with `worker exited with status Some(1)`, but the supervisor intentionally nulled both worker `stdout` and `stderr`, which left the next root cause invisible.
- Manual host-side reproduction proved the Scrapling worker can execute successfully when given a valid plan and telemetry secret, so the immediate missing capability was truthful failure surfacing, not another speculative control-plane change.

## Verification

- `make test-adversary-sim-scrapling-worker`
- `git diff --check`

## Remaining gap

- This slice does not itself fix the underlying live shared-host Scrapling failure; it makes the next live run surface the real worker crash reason so the next tranche can be evidence-driven instead of guess-driven.
