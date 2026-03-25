Date: 2026-03-24

# SIM-SCR-CHALLENGE-2D Coverage Closure And Gap Assignment Post-Implementation Review

## What landed

Shuma now compares recent Scrapling defense receipts against the frozen request-native owned-surface matrix instead of leaving that comparison implicit.

Specifically:

- `operator_snapshot_v1.adversary_sim` now carries `scrapling_owned_surface_coverage`,
- that summary compares recent Scrapling `observed_defense_keys` receipts against the canonical owned-surface matrix,
- it reports explicit per-surface `coverage_status`, `coverage_basis`, and `gap_assignment`,
- and the current missing owned surface is now machine-visible as:
  - `geo_ip_policy`
  - `gap_assignment=request_native_proxy_or_source_ip_diversification`

## Why this closes `SIM-SCR-CHALLENGE-2D`

`SIM-SCR-CHALLENGE-2D` was not “make every surface covered.” It was:

1. compare observed receipts against the owned-surface matrix,
2. make any mismatch fail visible,
3. and assign the remaining gap truthfully before reopening broader runtime changes.

That comparison is now first-class in the backend contract instead of living only in worker-local tests or human interpretation.

## Proof

Focused verification:

- `make test-adversarial-coverage-receipts`
- `git diff --check`

The proof now covers:

- the frozen owned-surface matrix,
- receipt-backed coverage comparison,
- the operator-snapshot projection of that comparison,
- and the explicit current `geo_ip_policy` gap assignment.

## Outcome

This tranche shows that the remaining uncovered owned surface is not currently a browser or stealth-runtime problem.

The present evidence points to:

1. request-native source-IP diversification,
2. proxy-backed request-native egress,
3. or another truthful public-network identity mechanism,

as the next necessary follow-on for `geo_ip_policy`.

So the next active path should be a request-native geo/IP coverage tranche, not automatic escalation to `SIM-SCR-CHALLENGE-2C`.
