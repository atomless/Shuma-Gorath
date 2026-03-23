# VID-TAX-2, VID-BOT-1, and VID-GUARD-1 Calibration And No-Harm Post-Implementation Review

Date: 2026-03-23
Status: Completed

Related plan:

- [`../plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](../plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)

# Delivered

The verified-identity calibration track now extends the earlier faithful category crosswalk into auditable machine-first alignment, conflict, and no-harm controller behavior.

## `VID-TAX-2`

1. Shuma now emits bounded verified-identity versus taxonomy alignment receipts in [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs).
2. Each receipt preserves verified-identity operator, stable identity, scheme, verified category, projected taxonomy category, alignment status, degradation reason, count, `end_user_controlled`, and evidence references.
3. The verified-identity snapshot section in [`../../src/observability/operator_snapshot_verified_identity.rs`](../../src/observability/operator_snapshot_verified_identity.rs) now carries a bounded `taxonomy_alignment` summary with aligned, fallback, misaligned, and insufficient-evidence counts.

## `VID-BOT-1`

1. The beneficial non-human benchmark family in [`../../src/observability/benchmark_beneficial_non_human.rs`](../../src/observability/benchmark_beneficial_non_human.rs) now surfaces explicit verified-identity calibration metrics:
   - `taxonomy_alignment_mismatch_rate`
   - `verified_botness_conflict_rate`
   - `user_triggered_agent_friction_mismatch_rate`
2. Those metrics stay bounded and return `insufficient_evidence` until the protected verified sample is large enough to justify controller use.
3. The benchmark note now cites the alignment-receipt count so later Monitoring can explain the calibration basis directly.

## `VID-GUARD-1`

1. Benchmark tuning eligibility in [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs) now adds explicit blockers when verified-identity alignment or conflict metrics show likely harm to tolerated or allowed verified traffic.
2. Reconcile in [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs) now carries those blockers through fail-closed `observe_longer` outcomes instead of tuning through the conflict.
3. The guardrail remains restrictive by default: it adds no implicit allow path and only blocks tuning when calibration evidence says the current posture is likely harming protected verified traffic.

# Verification

- `make test-verified-identity-alignment-receipts`
- `make test-verified-identity-botness-conflicts`
- `make test-verified-identity-guardrails`
- `make test-verified-identity-calibration-readiness`
- `git diff --check`

Focused proof now covers:

1. alignment receipt materialization and ordering,
2. verified-identity snapshot alignment summary projection,
3. explicit benchmark conflict metrics for protected verified traffic and user-triggered verified agents,
4. benchmark tuning blockers derived from those metrics,
5. and reconcile fail-closed behavior when those blockers are present.

# Review Result

No tranche-local shortfall remains open.

Residual note:

1. `src/config/runtime_env.rs::spin_variable_name` still emits the existing dead-code warning during focused Rust tests. That warning pre-dates this tranche and should remain handled under the separate build-hygiene cleanup path rather than reopening the verified-identity calibration work.
