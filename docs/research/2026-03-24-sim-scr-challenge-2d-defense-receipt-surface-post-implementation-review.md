# SIM-SCR-CHALLENGE-2D Defense Receipt Surface Post-Implementation Review

Date: 2026-03-24

## What landed

The recent-sim-run receipt surface now exposes exact observed defense keys, and those keys are aligned to the Scrapling owned-surface matrix names where possible.

Specifically:

- `MonitoringRecentSimRunSummary` now carries `observed_defense_keys`,
- `OperatorSnapshotRecentSimRun` now carries the same field,
- the recent-run aggregation path now emits sorted exact defense keys rather than only a coarse `defense_delta_count`,
- and the defense classifier now distinguishes:
  - `challenge_routing`
  - `challenge_puzzle`
  - `not_a_bot`
  - `proof_of_work`
  - plus aligned `rate_limit` and `geo_ip_policy` names

## Why this matters

`SIM-SCR-CHALLENGE-2D` needs receipt-backed proof that Scrapling touches the owned surfaces it is supposed to represent. Before this tranche, recent runs only exposed a defense-count summary and a coarse challenge bucket, which was not precise enough to compare against the owned-surface matrix.

This slice makes the telemetry receipt closer to the matrix:

- recent sim runs can now say which defenses were actually observed,
- those names are no longer forced through a single generic `challenge` bucket,
- and the operator snapshot now preserves the same receipt shape for later judge-side reasoning.

## Proof

Focused verification:

- `make test-adversarial-coverage-receipts`
- `git diff --check`

The added proof shows that recent-run summaries now preserve matrix-aligned defense names from observed telemetry.

## Remaining gap

This does not yet close `SIM-SCR-CHALLENGE-2D`.

Still remaining:

- compare the observed-defense receipt set against the owned-surface matrix explicitly,
- make any missing owned surfaces fail visible,
- and decide whether any remaining gap truly requires `SIM-SCR-CHALLENGE-2C` browser or stealth Scrapling rather than more request-native coverage or telemetry refinement.
