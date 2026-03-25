# MZ-T4 Maze Canonical Verification Wiring Post-Implementation Review

Date: 2026-03-24
Status: Completed
Parent plan: [`../plans/2026-03-24-mz-t4-maze-canonical-verification-wiring-plan.md`](../plans/2026-03-24-mz-t4-maze-canonical-verification-wiring-plan.md)

## What landed

- Added the focused wiring proof in:
  - [`../../scripts/tests/test_maze_verification_wiring.py`](../../scripts/tests/test_maze_verification_wiring.py)
  - [`../../Makefile`](../../Makefile) as `make test-maze-verification-wiring`
- Added the canonical aggregate maze gate in:
  - [`../../Makefile`](../../Makefile) as `make test-maze-verification-gate`
- Routed the umbrella suite through that canonical maze gate in:
  - [`../../Makefile`](../../Makefile) `test`
- Reused the same canonical maze gate in the release workflow instead of maintaining a divergent maze command list in:
  - [`../../.github/workflows/release-gate.yml`](../../.github/workflows/release-gate.yml)
- Updated docs in:
  - [`../../docs/testing.md`](../../docs/testing.md)
  - [`../../docs/maze.md`](../../docs/maze.md)
  - [`../../docs/plans/2026-02-25-maze-carry-forward-plan.md`](../../docs/plans/2026-02-25-maze-carry-forward-plan.md)

## Result

The maze carry-forward proof is now genuinely canonical: the benchmark, live traversal, live browser, and state-concurrency checks are exercised through one shared local and release-oriented gate instead of only as side targets.

## Verification

Passed:

- `make test-maze-verification-wiring`
- `make spin-wait-ready`
- `make test-maze-verification-gate`
- `git diff --check`

Additional evidence:

- `make test` was re-run because this tranche changed the canonical umbrella path. It failed on four unrelated existing baseline tests outside `MZ-T4`:
  - `admin::api::tests::recent_sim_run_history_normalizes_scrapling_profiles_and_aggregates_observed_categories`
  - `observability::benchmark_results::tests::escalation_hint_promotes_unaddressable_budget_breach_to_code_evolution_candidate`
  - `providers::external::tests::verified_identity_provider_returns_disabled_when_provider_path_is_off`
  - `observability::hot_read_projection::tests::counter_flush_refresh_operator_snapshot_document_stays_within_budget_and_separates_live_from_sim`

## Follow-up assessment

No further maze carry-forward work remains. The umbrella red tests exposed by the `make test` rerun should be handled separately if they are still present on the next active tranche.
