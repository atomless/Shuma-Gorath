Date: 2026-03-24

# SIM-SCR-GEO-1 Public-Network Identity Post-Implementation Review

## What landed

Shuma's request-native Scrapling lane can now touch its remaining owned `geo_ip_policy` surface without violating attacker-faithfulness.

Specifically:

- the worker-plan contract now carries an optional bounded `public_network_identity`,
- the shared-host runtime can select that identity only from env-configured public-network identities,
- the Scrapling worker resolves it into proxy-backed request-native egress,
- `http_agent` ticks now attempt `geo_ip_policy` under that public identity,
- and the result surface records both the identity used and per-surface identity receipts.

The new diagnostics truth is visible through:

- `lane_diagnostics.lanes.<lane>.last_public_network_identity`
- `lane_diagnostics.lanes.<lane>.last_surface_identity_receipts`

## Why this closes `SIM-SCR-GEO-1`

`SIM-SCR-GEO-1` was not about "more Scrapling features" in the abstract.

It was specifically about closing the one remaining request-native owned-surface gap without cheating.

That gap is now closed because:

1. `geo_ip_policy` is touched through request-native public-network identity diversity,
2. the worker does not send trusted Shuma-only geo headers itself,
3. the interaction is proved through a proxy-backed hostile path,
4. and the receipt surface now records which identity class and identity id were used.

## Proof

Focused verification:

- `make test-adversary-sim-scrapling-worker`
- `make test-adversarial-coverage-receipts`
- `git diff --check`

The worker proof now covers:

- bounded public-network identity selection,
- truthful proxy-backed `geo_ip_policy` interaction,
- absence of privileged geo headers on the attacker side,
- and persisted identity receipts in the backend diagnostics path.

## Outcome

The request-native Scrapling-owned surface matrix is now covered truthfully enough to move the mainline forward.

That means:

1. `SIM-SCR-CHALLENGE-2C` remains conditional and blocked rather than active,
2. the immediate next mainline is now `RSI-GAME-1A`,
3. and deferred dashboard follow-ons remain behind the first judge-side game-contract slices and the first explicit self-improving loop run.
