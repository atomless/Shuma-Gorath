Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-plan.md`](../plans/2026-03-24-scrapling-owned-defense-surface-matrix-and-success-contract-plan.md)
- [`../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/tests/test_scrapling_worker.py`](../../scripts/tests/test_scrapling_worker.py)

# Scrapling Owned Defense-Surface Matrix And Success Contract Review

## Question

Before widening Scrapling further, what exact defense surfaces should the Scrapling lane own, and what counts as faithful success on each one?

## Current code-grounded state

Today Shuma proves Scrapling only as a bounded request-native lane with three personas:

1. `crawler`,
2. `bulk_scraper`,
3. `http_agent`.

That contract is explicit in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py) and [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs).

The deterministic sim lane, not Scrapling, currently owns the explicit abuse attempts for:

1. `not_a_bot` fail and escalate,
2. puzzle escalation,
3. PoW verify abuse,
4. tarpit progress abuse,
5. and several geo or route-pressure actions.

That split is visible in [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs).

So the missing contract is not category ownership alone. It is the defense-surface ownership matrix that says which surfaces Scrapling must actually encounter as a faithful hostile request-native attacker.

## Decision

Shuma should freeze a separate Scrapling-owned defense-surface matrix.

This matrix should be machine-readable and distinct from the category-fulfillment matrix.

Each owned surface needs four answers:

1. does Scrapling own this surface,
2. is request-native Scrapling enough or is browser or stealth required,
3. must Scrapling merely touch it, fail against it, or be able to pass it,
4. and if pass-capable is expected, is that because an outside attacker could plausibly do so with only public host knowledge.

## Recommended owned request-native surfaces

The first truthful Scrapling-owned request-native surface set should be:

1. `honeypot`
2. `rate_limit`
3. `geo_ip_policy`
4. `challenge_routing`
5. `not_a_bot`
6. `challenge_puzzle`
7. `proof_of_work`

These are the surfaces a malicious request-native crawler, scraper, or HTTP agent should be expected to encounter or pressure if Scrapling is to be treated as attacker-faithful for the surfaces it owns.

## Success semantics by surface

The matrix should distinguish several success contracts.

### `must_touch`

The attacker must be shown to reach or pressure the surface, but passing it is not required.

This fits:

1. `rate_limit`
2. `geo_ip_policy`
3. `challenge_routing`

### `must_fail_or_escalate`

The attacker must reach the surface and produce the hostile fail or escalation behavior a real malicious request-native attacker would produce, but it is not yet required to solve it.

This fits:

1. `honeypot`
2. `not_a_bot`
3. `challenge_puzzle`
4. `proof_of_work`

### `must_pass_when_publicly_solved`

This should remain a later conditional contract, not an immediate promise. If a future request-native or stealth Scrapling path can black-box solve a surface using only public host knowledge, then passing it becomes part of the owned-surface truth for that surface. Until then, the surface should remain explicitly fail or escalate only.

## Out-of-scope or differently-owned surfaces

The first matrix should explicitly keep these outside request-native Scrapling ownership:

1. `javascript_verification`
2. `cdp_detection`
3. `maze`
4. browser-only fingerprint or browser-automation surfaces

Those may later move into a browser or stealth Scrapling follow-on, but they should not be quietly implied by the request-native Scrapling lane.

## Why this contract is needed

Without it, Shuma cannot truthfully answer:

1. whether Scrapling is exercising the defenses it claims to represent,
2. whether a missing interaction is a real bug or simply not in Scrapling scope,
3. or whether browser or stealth Scrapling is genuinely required.

That makes later claims about attacker-faithfulness too hand-wavy.

## Result

`SIM-SCR-CHALLENGE-2A` should freeze:

1. the first owned request-native defense-surface matrix,
2. the success contract for each surface,
3. and the explicit separation between request-native owned surfaces and later browser or stealth expansion.
