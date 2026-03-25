Date: 2026-03-24
Status: Proposed

Related context:

- [`2026-03-24-sim-scr-challenge-2d-coverage-closure-and-gap-assignment-post-implementation-review.md`](2026-03-24-sim-scr-challenge-2d-coverage-closure-and-gap-assignment-post-implementation-review.md)
- [`2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-review.md`](2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-review.md)
- [`../plans/2026-03-24-scrapling-geo-ip-policy-source-diversification-plan.md`](../plans/2026-03-24-scrapling-geo-ip-policy-source-diversification-plan.md)

# Scrapling Geo/IP Policy Source Diversification Review

## Question

How should Shuma close the remaining Scrapling-owned `geo_ip_policy` gap without violating attacker-faithfulness?

## Current code-grounded state

Recent request-native Scrapling proof now shows six of seven owned surfaces directly:

1. `challenge_routing`
2. `rate_limit`
3. `honeypot`
4. `not_a_bot`
5. `challenge_puzzle`
6. `proof_of_work`

The remaining missing surface is `geo_ip_policy`.

That gap is now explicit in the operator snapshot and focused coverage proof.

## Key judgment

This is not primarily a browser-runtime problem.

Browser or stealth Scrapling may change HTML or challenge interaction behavior, but it does not by itself provide the public-network identity diversity needed to trigger geo or IP policy honestly.

So widening Scrapling into browser mode first would be the wrong next move for this gap.

## Attacker-faithful options

The truthful request-native options are:

1. bounded proxy-backed egress,
2. bounded source-IP diversification across controlled public identities,
3. or another public-network identity mechanism that preserves the outside-attacker knowledge boundary.

What is not acceptable:

1. trusted internal geo headers,
2. Shuma-only privileged routing hints,
3. or any fake geo proof that an outside attacker could not actually use.

## Result

The next active follow-on should be a request-native source-diversification tranche for `geo_ip_policy`.

`SIM-SCR-CHALLENGE-2C` should remain conditional and later:

1. only if an owned surface truly needs browser or stealth behavior,
2. not as a substitute for public-network identity diversity.
