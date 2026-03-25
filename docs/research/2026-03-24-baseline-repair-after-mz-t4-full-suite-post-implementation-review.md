# Baseline Repair After MZ-T4 Full-Suite Post-Implementation Review

## What landed

`BASELINE-REPAIR-1` is now closed.

The tranche repaired the four baseline failures exposed by the first canonical `make test` run after `MZ-T4`:

1. stale escalation-hint expectations now match the landed move-selection contract,
2. verified-identity provider expectations now match the post-default-enable semantics,
3. the operator-snapshot hot-read budget is explicitly rebaselined to fit the current machine-first snapshot surface,
4. the broad-suite-only Scrapling/runtime drift is repaired so the running-target gate now proves real recent-run owned-surface closure instead of timing out on stale local state.

## What changed

### Contract-alignment repairs

- [`src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs) now asserts the current `code_evolution_only` / `code_or_capability_gap` semantics for unaddressable beneficial-non-human posture misses.
- [`src/providers/external.rs`](../../src/providers/external.rs) now asserts `NotAttempted` when verified identity is globally enabled but provider assertions remain unavailable.

### Hot-read budget truth

- [`src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs) raises `HOT_READ_OPERATOR_SNAPSHOT_MAX_BYTES` from `40 * 1024` to `60 * 1024`.
- [`docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](../../docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md) now states that the operator snapshot legitimately carries a broader machine-first control-loop surface and must be kept explicitly bounded rather than silently trimmed.

### Scrapling/runtime baseline repair

- [`Makefile`](../../Makefile) now prepares local Scrapling scope/seed artifacts before local Spin runs, exports the sim-telemetry secret to the host supervisor, and adds a focused supervisor unit target.
- [`scripts/deploy/scrapling_deploy_prep.py`](../../scripts/deploy/scrapling_deploy_prep.py) now supports controlled local HTTP scope/seed generation for loopback-hosted dev/test runs.
- [`scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs) now parses chunked HTTP beat responses correctly, preserves `fulfillment_mode` in failure results, and captures worker stderr for actionable failure receipts.
- [`scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py) now preserves distinct surface receipt outcomes per surface, records truthful request method/path metadata, and reorders the `http_agent` request sequence so the live runtime budget reaches PoW and tarpit abuse surfaces before lower-value CRUD traffic.
- [`scripts/tests/adversary_runtime_toggle_surface_gate.py`](../../scripts/tests/adversary_runtime_toggle_surface_gate.py) now:
  - clears loopback bans before the run,
  - reasserts PoW and puzzle on,
  - keeps enough rate-limit headroom to allow public-path pass receipts,
  - waits for `operator_snapshot_v1.adversary_sim.recent_runs[*].owned_surface_coverage.overall_status == "covered"`,
  - and still proves live-only monitoring summary no-impact.
- [`src/admin/api.rs`](../../src/admin/api.rs) now lets receipt-only recent sim-run events participate in recent-run aggregation, while still keeping `monitoring_event_count` limited to true monitoring events. That keeps owned-surface coverage truthful in operator snapshot without inflating monitoring counts.

## Verification

Passed:

- `make test-adversary-sim-scrapling-worker`
- `make test-adversary-sim-scrapling-coverage-receipts`
- `make test-adversary-sim-runtime-surface-unit`
- `make test-adversary-sim-runtime-surface`
- `make test`
- `git diff --check`

The full suite completed green, including:

- Rust unit + integration layers,
- the repaired runtime-toggle Scrapling gate,
- adversarial fast matrix,
- dashboard/browser end-to-end coverage.

## Follow-on note

This tranche restores a truthful green baseline; it does not yet widen Scrapling beyond the currently owned request-native surface. The next mainline attacker-faithful work should build from this repaired baseline rather than re-litigate these same harness failures.
