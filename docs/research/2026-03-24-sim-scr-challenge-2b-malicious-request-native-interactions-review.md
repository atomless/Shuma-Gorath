Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-plan.md`](../plans/2026-03-24-sim-scr-challenge-2b-malicious-request-native-interactions-plan.md)
- [`../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/tests/test_scrapling_worker.py`](../../scripts/tests/test_scrapling_worker.py)

# SIM-SCR-CHALLENGE-2B Review

## Goal

Turn the current request-native Scrapling personas into attacker-faithful workers for the owned request-native surfaces, rather than leaving challenge-abuse behavior stranded in the deterministic lane.

## Current Gap

The worker contract currently proves only:

1. bounded public traversal for `crawler`,
2. bounded pagination and detail scraping for `bulk_scraper`,
3. bounded method mix and redirect follow-up for `http_agent`.

That is still useful, but it does not satisfy the owned-surface contract frozen in [`scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs).

Specifically:

- `bulk_scraper` is expected to touch `not_a_bot_submit` and `puzzle_submit_or_escalation`,
- `http_agent` is expected to touch those surfaces plus `pow_verify_abuse` and `tarpit_progress_abuse`,
- all Scrapling request-native personas are expected to touch challenge-routing pressure rather than only ordinary public pages.

Today the exact malicious request shapes still live in [`adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs), while the Python Scrapling worker has no route-level contract for them.

## Cleanest Implementation Shape

The cleanest seam is the worker plan.

Rather than hard-coding Shuma paths ad hoc in Python, widen [`ScraplingWorkerPlan`](../../src/admin/adversary_sim_worker_plan.rs) so the Rust control plane gives the worker the exact owned request-native surface hints it is allowed to act on:

- per-mode `surface_targets`,
- a bounded `runtime_paths` bundle for the public challenge and abuse routes the worker may hit.

That keeps the Python worker honest:

- it does not need to discover or invent Shuma-specific route guesses,
- it only receives the bounded routes the plan says belong to its owned request-native remit,
- and the beat payload becomes the machine-readable contract that later coverage and receipt work can inspect.

## Recommended Persona Expansion

`crawler`
- keep public traversal as the dominant behavior,
- add a light challenge-routing touch using public request-native route hints only,
- do not widen into submit-abuse surfaces it does not own.

`bulk_scraper`
- keep catalog/detail traversal that should still pass some requests,
- add malicious request-native submits for:
  - `not_a_bot_submit`,
  - `puzzle_submit_or_escalation`.

`http_agent`
- keep the ordinary method mix that should still pass some requests,
- add malicious request-native submits for:
  - `not_a_bot_submit`,
  - `puzzle_submit_or_escalation`,
  - `pow_verify_abuse`,
  - `tarpit_progress_abuse`.

The malicious submits should remain attacker-faithful:

- invalid or replay-like tokens,
- invalid proofs,
- low-trust telemetry,
- direct abuse of public submit endpoints,
- no privileged headers,
- no internal Shuma secrets.

## What This Slice Should Not Do

`SIM-SCR-CHALLENGE-2B` should not:

- claim browser or stealth Scrapling coverage,
- add code for browser-class surfaces like maze or JS verification,
- add final receipt-backed coverage closure,
- or broaden the worker into a generic Shuma-aware internal client.

Those belong to `SIM-SCR-CHALLENGE-2C` and `SIM-SCR-CHALLENGE-2D`.

## Decision

`SIM-SCR-CHALLENGE-2B` should:

1. widen the Scrapling worker plan with mode-owned route and surface hints,
2. teach the real Python Scrapling personas to issue attacker-faithful malicious request-native attempts against those surfaces,
3. keep some ordinary success traffic in the same personas,
4. and prove the behavior with a new focused make target rather than only the broad Scrapling worker suite.
