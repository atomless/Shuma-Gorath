# RSI-SCORE-2B Evidence-Quality And Diagnosis-Confidence Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`../../src/observability/benchmark_scrapling_evidence_quality.rs`](../../src/observability/benchmark_scrapling_evidence_quality.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/observability/benchmark_comparison.rs`](../../src/observability/benchmark_comparison.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Scope Reviewed

This closeout reviewed `RSI-SCORE-2B`: add explicit exploit-evidence quality and diagnosis-confidence gates so the loop can distinguish weak, poorly localized attacker evidence from controller-grade proof that is strong enough to justify a bounded config move.

# What Landed

1. Shuma now materializes a first-class exploit-evidence quality assessment in `benchmark_results_v1`.
2. That assessment records whether the latest Scrapling exploit evidence is:
   - category-native or projected,
   - sufficiently sampled,
   - fresh,
   - multi-persona or single-persona,
   - reproduced across recent runs,
   - and localized to named breach loci.
3. `BenchmarkEscalationHint` now carries:
   - explicit `evidence_quality`,
   - explicit diagnosis-confidence state,
   - and the ranked `breach_loci` preserved alongside the recommendation surface.
4. Tuning eligibility now fails closed when `scrapling_exploit_progress` is outside budget but the exploit evidence remains low confidence or otherwise too weak to justify a fine-grained move.
5. Reconcile-side proof now freezes that contract: low-confidence exploit evidence yields `observe_longer` instead of silently behaving like actionable tuning input.
6. A focused make path, `make test-rsi-score-evidence-quality`, now proves the new gate end to end.

# Acceptance Review

`RSI-SCORE-2B` required explicit evidence-quality outputs, explicit diagnosis-confidence states, fail-closed tuning behavior, preserved breach loci, and focused proof.

Those criteria are now satisfied:

1. benchmark and reconcile contracts expose explicit evidence-quality and diagnosis-confidence outputs;
2. low-confidence exploit evidence is machine-visibly distinguishable from high-confidence exploit evidence;
3. tuning eligibility and reconcile behavior fail closed when those gates are not satisfied;
4. diagnosis output preserves the named breach loci under consideration;
5. and the repo now has the required focused proof surface through:
   - `make test-rsi-score-evidence-quality`
   - `make test-benchmark-results-contract`
   - `make test-oversight-reconcile`

The key behavioral correction is this:

`scrapling_exploit_progress` can now read `outside_budget` while the loop still refuses to tune because the exploit evidence is not yet strong enough for a bounded move.

That is intentional. It prevents the controller from turning a single weak or poorly reproduced attacker glimpse into a scattershot config change.

# Shortfalls Found

This slice does not yet tell the loop how urgent an exploit regression is or when a sudden new bypass should break homeostasis immediately.

The following planned work remains open:

1. `RSI-SCORE-2C` urgency and event-triggered homeostasis break;
2. `RSI-SCORE-2D` sharper judge/diagnoser/move-selector separation plus config-ring exhaustion;
3. `RSI-SCORE-2E` Game Loop projection of the richer judge truth.

So this tranche makes exploit evidence actionability explicit, but it does not yet make the loop react differently to a slow burn versus a sudden fresh bypass.

# Verification

- `make test-rsi-score-evidence-quality`
- `make test-benchmark-results-contract`
- `make test-oversight-reconcile`
- `git diff --check`
