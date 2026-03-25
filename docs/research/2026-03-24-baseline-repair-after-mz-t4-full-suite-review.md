## Why this review exists

`MZ-T4` correctly wired the maze verification gate into the canonical test path, but the follow-on `make test` run exposed four pre-existing baseline failures outside the maze slice. The mainline is therefore not in a truthful execution-ready state for the next scheduled Scrapling and game-loop tranches.

This review captures the actual failure causes before repair so the next slice stays narrow and auditable.

## Current failure set

### 1. Benchmark escalation-hint expectation drift

Focused repro:

- `make test-benchmark-results-contract`

Observed failure:

- `observability::benchmark_results::tests::escalation_hint_promotes_unaddressable_budget_breach_to_code_evolution_candidate`

Evidence:

- The test expects `no_matching_config_surface`.
- [`src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs) now classifies `beneficial_non_human_posture` directly as `code_evolution_candidate`.
- The `no_matching_config_surface` blocker is only emitted when the classification is still `config_tuning_candidate` but no legal controller family remains.

Conclusion:

- This is test expectation drift after the landed move-selection policy and controller-boundary work, not a fresh implementation regression.

### 2. Verified-identity provider expectation drift

Focused repro:

- `make test-verified-identity-provider`

Observed failure:

- `providers::external::tests::verified_identity_provider_returns_disabled_when_provider_path_is_off`

Evidence:

- [`src/providers/external.rs`](../../src/providers/external.rs) returns `Disabled` only when `cfg.verified_identity.enabled` is false.
- With verified identity now enabled by default, the external provider returns `NotAttempted` when provider assertions are off or the edge path is off.

Conclusion:

- This is test expectation drift after the default verified-identity enablement flip, not a fresh provider regression.

### 3. Operator-snapshot hot-read budget breach

Focused repro:

- `make test-telemetry-hot-read-projection`

Observed failure:

- `observability::hot_read_projection::tests::counter_flush_refresh_operator_snapshot_document_stays_within_budget_and_separates_live_from_sim`
- Serialized operator snapshot size: `53136`
- Current max contract size: `40960`

Evidence:

- [`src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs) now carries a larger machine-first surface, including:
  - `allowed_actions`
  - `game_contract`
  - `episode_archive`
  - `benchmark_results`
  - expanded non-human and verified-identity projections
- [`src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs) still caps the operator snapshot hot-read document at `40 * 1024`.

Conclusion:

- This is a real contract decision point. Either the operator snapshot must be trimmed back under the existing cap, or the cap must be rebaselined to match the now-landed machine-first snapshot scope.

### 4. Scrapling owned-surface coverage failure only under the broad unit suite

Focused repro:

- `make test-adversary-sim-scrapling-coverage-receipts`

Current focused result:

- Passes.

Original failure observed in the broad suite:

- `admin::api::tests::recent_sim_run_history_normalizes_scrapling_profiles_and_aggregates_observed_categories`
- Expected `owned_surface_coverage.overall_status == "covered"`
- Saw `unavailable`

Evidence:

- The exact focused gate passes, including the owned-surface receipts and snapshot projection.
- The failure therefore appears to depend on broad-suite execution context rather than the isolated Scrapling coverage contract itself.

Current best hypothesis:

- This is likely order-sensitive or shared-state-sensitive behavior inside the wider unit run, not a simple local contract break in the Scrapling owned-surface logic.

## Repair direction

The minimal truthful repair tranche is:

1. update the two stale expectations to match the already-landed contracts,
2. make an explicit operator-snapshot hot-read budget decision and prove it through the focused hot-read gate,
3. rerun the broad unit and full test paths to confirm whether the Scrapling failure remains,
4. only if it remains, continue into targeted shared-state or order-dependence debugging for that path.

## Why this comes before the next scheduled mainline tranche

The next scheduled work is attacker-faithful Scrapling plus the game-loop mainline. Both depend on a trustworthy green baseline. Continuing from a knowingly red `make test` would create plan-to-implementation drift and make later failures much harder to attribute.
