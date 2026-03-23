# Host-COST-1 And Host-COST-2 Post-Implementation Review

Date: 2026-03-23
Status: Closed

Related context:

- [`2026-03-23-host-impact-cost-proxy-and-benchmark-review.md`](2026-03-23-host-impact-cost-proxy-and-benchmark-review.md)
- [`../plans/2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md`](../plans/2026-03-23-host-impact-cost-proxy-and-benchmark-implementation-plan.md)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
- [`../../src/observability/benchmark_results_families.rs`](../../src/observability/benchmark_results_families.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)

# Scope Reviewed

This closeout reviewed the delivered host-impact proxy track that was planned as `HOST-COST-1` and `HOST-COST-2`:

1. forwarded upstream latency capture in request outcomes and bounded monitoring summaries,
2. host-impact projection into `operator_snapshot_v1`,
3. host-impact benchmark materialization inside the existing `suspicious_origin_cost` family,
4. and reconcile consumption through the existing suspicious-origin pressure path.

# What Landed

1. Forwarded request outcomes now carry bounded forwarded upstream latency truth.
2. Monitoring scope, lane, and non-human-category summaries now accumulate `forwarded_upstream_latency_ms_total` alongside request and byte counters.
3. `operator_snapshot_v1` now projects total live and per-lane forwarded latency totals.
4. `operator_objectives_v1` now includes a budgetable `suspicious_forwarded_latency_share` objective.
5. `benchmark_results_v1` now materializes:
   - `suspicious_forwarded_latency_share`
   - `suspicious_average_forward_latency_ms`
6. Prior-window comparison treats both new metrics as `LowerIsBetter`.
7. Reconcile can now classify latency-share budget misses as suspicious-origin pressure without inventing a second controller family.

# Review Result

The delivered shape matches the plan's main architectural decisions:

1. no speculative CPU, memory, or billing model was introduced,
2. the new budget metric is ratio-shaped and bounded,
3. the benchmark extension stayed inside the existing suspicious-origin family,
4. and reconcile still uses one suspicious-origin pressure family instead of splitting into parallel cost controllers.

The implementation also stayed aligned with the repo-wide telemetry rule:

- the new proxy is derived from observed forwarded latency on real request outcomes,
- then summarized through the existing bounded hot-read path,
- rather than being synthesized by route weights or heuristics.

# Shortfalls Found

No tranche-local shortfall remains open.

One unrelated pre-existing warning still remains visible in focused Rust test output:

- `src/config/runtime_env.rs::spin_variable_name` is still reported as dead code under native test builds.

That warning predates this tranche and remains tracked separately as broader build hygiene rather than as a host-impact blocker.

# Verification

- `make test-host-impact-telemetry`
- `make test-host-impact-benchmark`
- `make test-oversight-host-impact`
- `git diff --check`

# Operational Note

This track improves the loop's cost truth, but it remains a proxy model:

- request share,
- byte share,
- and now forwarded latency share.

It is intentionally not yet a literal infrastructure billing or CPU accounting system, which keeps the first closed loop truthful and low-surprise while still moving materially closer to host-impact-aware tuning.
