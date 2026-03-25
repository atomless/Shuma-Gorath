Date: 2026-03-24

# RSI-GAME-MAINLINE-1 Live Identity Preflight Post-Implementation Review

## What landed

The live shared-host first-loop verifier now fails fast when the remote cannot truthfully prove Scrapling's attacker-faithful `geo_ip_policy` coverage.

Specifically:

- `scripts/tests/live_feedback_loop_remote.py` now preflights the remote transport environment for `ADVERSARY_SIM_SCRAPLING_PUBLIC_NETWORK_IDENTITIES`,
- it requires at least one bounded `http_proxy` identity with a usable proxy URL,
- and it stops before starting the live sim run when that attacker-faithful prerequisite is missing.

The verifier also records the configured identity summary in the success-path report so the live proof captures which public-network identity pool existed while the loop was being exercised.

## Why this follow-on was necessary

The earlier live proof had become truthful at the operator-snapshot and episode-lineage level, but it still burned a full run and only then failed with a late `partial` Scrapling coverage result when the remote had no configured public-network identities.

That was not precise enough.

At this stage the remaining live blocker is environmental, not architectural:

1. the request-native Scrapling lane already has the code paths to touch `geo_ip_policy` attacker-faithfully,
2. the focused local and unit proof paths are green,
3. but the active shared-host remote still lacks `ADVERSARY_SIM_SCRAPLING_PUBLIC_NETWORK_IDENTITIES`.

So the verifier needed to say that explicitly and immediately.

## Proof

Focused verification:

- `make test-live-feedback-loop-remote-unit`
- `make test-rsi-game-mainline`
- `make test-live-feedback-loop-remote ENV_LOCAL=/Users/jamestindall/Projects/Shuma-Gorath/.env.local REMOTE_RECEIPTS_DIR=/Users/jamestindall/Projects/Shuma-Gorath/.shuma/remotes REMOTE=dummy-static-site-fresh`
- `git diff --check`

The shared-host live proof now fails with the explicit prerequisite message:

- `Remote transport environment does not configure ADVERSARY_SIM_SCRAPLING_PUBLIC_NETWORK_IDENTITIES ...`

## Outcome

`RSI-GAME-MAINLINE-1` now has a sharper live-proof boundary:

1. local and unit proof continue to validate the bounded loop over attacker-faithful Scrapling receipts and episode lineage,
2. live proof now fails early when the remote cannot satisfy the Scrapling public-network identity prerequisite,
3. and the remaining blocker is clearly deployer configuration rather than hidden inside a late generic coverage failure.
