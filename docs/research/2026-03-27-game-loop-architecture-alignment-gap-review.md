Date: 2026-03-27
Status: Active review driver

Related context:

- [`2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`2026-03-27-game-loop-current-state-and-gap-review.md`](2026-03-27-game-loop-current-state-and-gap-review.md)
- [`2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md`](2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md)
- [`../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`../plans/2026-03-27-game-loop-board-state-refactor-plan.md`](../plans/2026-03-27-game-loop-board-state-refactor-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Re-check the current Game Loop architecture against the newer restriction-vs-recognition design and identify the concrete refactors, cleanup, and retirement work still required to align the codebase with that design.

# Executive Summary

The current runtime defense path is cleaner than the rest of the stack.

At runtime, Shuma still mostly decides from host-observable evidence:

1. [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs) derives `non_human_category` from verified identity or Shuma-side lane assignment.
2. It does not ingest Scrapling persona labels, fulfillment modes, or recent-run category intent as live defense truth.

That is good.

The architectural problem begins one layer later.

Snapshot assembly, benchmark scoring, escalation classification, reconcile, API adaptation, and dashboard projection still re-entangle three things that the new design says must be distinct:

1. defense-runtime truth,
2. restriction scoring,
3. recognition evaluation.

So the repo is now in a transitional state:

1. the doctrine is clearer than the implementation,
2. the Game Loop UI has become more honest,
3. but the underlying machine-first model is still too category-first and too family-first,
4. and several data contracts still flatten unknown or evaluation-only information into controller-looking truth.

# What The Code Shows Today

## 1. Runtime defense rail is comparatively clean

[`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs) still assigns category from:

1. verified identity category if available,
2. otherwise Shuma's own runtime lane classification.

That means the defense runtime is not currently cheating by reading simulator ground truth.

This part is broadly aligned with the new doctrine.

## 2. Snapshot non-human summary still mixes restriction evidence and recognition-eval evidence

[`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs) still builds one shared receipt set that combines:

1. live category rows or lane-derived receipts,
2. projected adversary-sim receipts from `sim_receipts_from_recent_runs(...)`.

Those projected sim receipts are explicitly marked degraded and based on `projected_recent_sim_run`, which is honest as far as it goes.

But [`../../src/observability/operator_snapshot_non_human.rs`](../../src/observability/operator_snapshot_non_human.rs) still exposes one combined `receipts` vector plus one shared readiness and coverage model.

So the snapshot layer still makes it easy for evaluation-only sim truth and restriction-grade observed truth to flow through the same summary surface.

That is the opposite of the clearer three-rail model.

## 3. Restriction scoring is still too category-first

[`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs) still seeds `default_category_postures()` with every canonical non-human category set to `blocked`.

[`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs) still turns those postures into the first-class benchmark family `non_human_category_posture`.

[`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs) still materializes that family alongside:

1. `scrapling_exploit_progress`,
2. `scrapling_surface_contract`,
3. `suspicious_origin_cost`,
4. and the other main benchmark families.

This means the repo still structurally treats per-category posture alignment as part of the main restriction scoreboard, even though the newer design says exact hostile category posture for undeclared hostile traffic is secondary unless Shuma can really infer it from shared-path evidence.

## 4. Benchmark -> escalation -> reconcile is still too monolithic

[`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs) still derives one main `escalation_hint` from the family set.

[`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs) still branches primarily on:

1. `benchmark.overall_status`,
2. `benchmark.escalation_hint.decision`,
3. and `benchmark.escalation_hint.candidate_action_families`.

`judge`, `diagnosis`, and `move_selection` are present in reconcile output, but they are still largely projections of the benchmark payload rather than independently owned controller inputs.

So the controller is still effectively taking one monolithic hint object as its oracle.

That architecture is too tightly coupled for the newer board-state-first loop.

## 5. Board-state locus contracts are still too weak

[`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs) defines `BenchmarkExploitLocus` with:

1. `attempt_count: u64`,
2. plain `cost_channel_ids`,
3. plain `repair_family_candidates`,
4. and no explicit materialization-status or provenance fields for those subparts.

[`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js) still adapts `attempt_count` with `Number(source.attempt_count || 0)`.

That means missing data can still collapse into zero-looking data at the adapter layer.

The UI has already been moving toward more honest wording, but the underlying contract is still too coarse and too easy to misread.

## 6. The dashboard is still projecting the old family model directly

[`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte) still directly binds to:

1. `findBenchmarkFamily('non_human_category_posture')`,
2. `benchmarkResults.escalation_hint.candidate_action_families`,
3. `benchmarkResults.escalation_hint.blockers`,
4. and fallback breach-locus sources pulled from escalation hint, evidence quality, or exploit-progress families.

So even though the copy is getting more honest, the tab is still largely a direct view over the legacy family-first benchmark shape.

# Architecture Misalignments That Must Be Fixed

## A. Recognition evaluation and restriction scoring need separate snapshot rails

The repo currently has one non-human summary carrying both:

1. current restriction-relevant observed evidence,
2. evaluation-only degraded sim projection.

That needs to split.

The recognition-evaluation rail should be allowed to compare:

1. simulator-known category intent,
2. Shuma's inferred non-humanness,
3. Shuma's inferred hostile category.

But the restriction rail should only consume:

1. Shuma-side confidence,
2. host cost,
3. board progression,
4. and human-friction guardrails.

## B. Category posture must stop being a primary restriction objective for undeclared hostile traffic

`category_postures` and `non_human_category_posture` are not safe as the main restriction target for undeclared hostile traffic in the current design.

They can remain:

1. a recognition-evaluation plane,
2. a verified or declared traffic plane,
3. or a high-confidence diagnostic plane.

But they should no longer sit in the repo as if they were the main tuning score for hostile traffic that Shuma can only classify progressively through the defense layers.

## C. The controller contract needs cleaner separation of concern

Right now the pipeline is still:

1. family scoring,
2. one escalation hint,
3. reconcile consumes that hint.

The newer design needs:

1. restriction judge,
2. recognition evaluation,
3. restriction diagnosis,
4. and move selection

to be explicit siblings rather than one of them secretly owning the others.

## D. Board-state loci need materialization truth, not only values

Every breach-locus field that can be missing must become explicit about whether it is:

1. measured,
2. derived,
3. not materialized,
4. or not applicable.

Without that, the Game Loop will continue to look more certain than it really is.

## E. Legacy category-first surfaces must be retired only after full-path replacement

There are several older surfaces that now conflict with the new architecture.

But they are not "dead code" yet.

They are still active across:

1. snapshot generation,
2. benchmark suite,
3. API payloads,
4. dashboard projection,
5. docs,
6. and tests.

So they must be retired deliberately, with replacement proof first, not by ad-hoc deletion.

# Retirement Candidates Once Replacements Exist

These are retirement candidates, not current dead-code claims.

## 1. `default_category_postures()` as a primary hostile-restriction target

Candidate for demotion or redesign after restriction scoring is re-centered.

Evidence:

- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)

## 2. `non_human_category_posture` as a first-class main restriction family

Candidate for relocation into recognition evaluation or secondary diagnostics once the replacement restriction families exist.

Evidence:

- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_suite.rs`](../../src/observability/benchmark_suite.rs)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

## 3. `sim_receipts_from_recent_runs(...)` as a shared input to restriction readiness

Candidate for confinement to the recognition-evaluation rail after the split.

Evidence:

- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/operator_snapshot_non_human.rs`](../../src/observability/operator_snapshot_non_human.rs)

## 4. `BenchmarkEscalationHint` as the controller's de facto oracle

Candidate for demotion once explicit restriction-judge, diagnosis, and move-selection contracts land.

Evidence:

- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)

## 5. Numeric-default breach-locus adapters

Candidate for removal once loci carry proper materialization truth.

Evidence:

- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)

# Recommended Refactor Order

1. settle Scrapling owned-surface exercise and dependency truth first,
2. split recognition-evaluation from restriction scoring in snapshot and benchmark assembly,
3. replace category-first restriction objectives with confidence-and-cost-first restriction scoring,
4. refactor controller contracts so reconcile stops depending on one monolithic escalation hint,
5. normalize breach-locus materialization and blocker typing,
6. and only then retire the legacy category-first surfaces from suite, API, dashboard, docs, and tests.

# Why This Matters

Without this refactor, the repo will keep suffering the same pattern:

1. runtime truth grows more nuanced,
2. board-state thinking improves,
3. but the benchmark/controller/UI layer keeps pulling everything back into an older category-first family model.

That would make the Game Loop noisier, more contradictory, and harder to trust exactly where an RSI loop most needs clean boundaries and crisp scores.

# Outcome

The repo now needs one explicit alignment tranche focused on:

1. rail separation,
2. restriction-objective redesign,
3. controller contract cleanup,
4. breach-locus contract cleanup,
5. and later retirement of replaced category-first surfaces.

Those findings are converted into execution-ready planning and TODOs in the linked plan and backlog updates.
