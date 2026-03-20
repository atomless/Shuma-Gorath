# Benchmark Suite v1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Define the first benchmark contracts Shuma will use to judge config tuning, later Monitoring projections, and the later project-evolution loop.

**Architecture:** Reuse the machine-first operator snapshot work as the source of benchmark inputs, keep benchmark contracts bounded and schema-versioned, and separate the per-instance tuning loop from the later benchmark-driven code-evolution loop.

**Tech Stack:** Markdown planning docs now; later Rust hot-read documents and admin APIs, existing monitoring summaries, adversary-sim summaries, and bounded dashboard projections.

---

## Phase 1: Benchmark Family Contract

### Task 1: Define `benchmark_suite_v1`

**Files:**

- Create:
  - `docs/research/2026-03-20-benchmark-suite-v1-research-synthesis.md`
  - `docs/plans/2026-03-20-benchmark-suite-v1-design.md`

**Outcome:**

Write the first benchmark-family registry covering:

1. suspicious origin cost,
2. likely-human friction,
3. representative adversary effectiveness,
4. beneficial non-human posture.

**Guardrails:**

1. Keep the family count intentionally small.
2. Define eligible populations and targets explicitly.
3. Do not let Monitoring invent different success criteria first.

## Phase 2: Results Envelope

### Task 2: Define `benchmark_results_v1`

**Files:**

- Modify later implementation targets in:
  - `src/observability/hot_read_contract.rs`
  - `src/observability/hot_read_documents.rs`
  - `src/observability/hot_read_projection.rs`
  - `src/admin/api.rs`

**Outcome:**

Define the bounded result envelope for:

1. subject kind,
2. baseline reference,
3. watch window,
4. per-family metric deltas,
5. improvement or regression status,
6. capability-gate and exactness metadata.

**Guardrails:**

1. Keep the result contract bounded and summary-level.
2. Do not depend on raw event tails.
3. Keep subject comparison semantics explicit.

## Phase 3: Config-Vs-Code Decision Boundary

### Task 3: Define the escalation rule

**Files:**

- Modify:
  - `docs/plans/2026-03-20-benchmark-suite-v1-design.md`
  - `todos/todo.md`
  - `todos/blocked-todo.md`

**Outcome:**

Define the benchmark-driven boundary between:

1. config tuning is enough,
2. more observation is needed,
3. and the codebase itself needs to evolve.

**Guardrails:**

1. Keep code evolution behind a later reviewed path.
2. Do not allow benchmark misses alone to authorize code generation.
3. Ensure the boundary is grounded in repeated benchmark evidence, not one instance or one run.

## Phase 4: Snapshot And Monitoring Alignment

### Task 4: Keep benchmark work aligned with `operator_snapshot_v1`

**Files:**

- Modify:
  - `docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`
  - `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
  - `todos/blocked-todo.md`

**Outcome:**

Make it explicit that:

1. `operator_snapshot_v1` is the benchmark input contract,
2. Monitoring later projects benchmark and snapshot sections rather than inventing its own semantic model,
3. and the scheduled controller and later project-evolution loop both depend on the same benchmark contract.

## Phase 5: Fleet And Intelligence Enrichment

### Task 5: Define later enrichment rules

**Files:**

- Modify:
  - `docs/plans/2026-03-20-benchmark-suite-v1-design.md`
  - `docs/plans/2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md`
  - later central-intelligence planning docs

**Outcome:**

Define how later fleet or central-intelligence evidence may enrich:

1. scenario selection,
2. benchmark weighting,
3. and benchmark priority,

without making the Git repository the transport or source of truth for shared benchmark data.

This task is now captured by:

1. `docs/research/2026-03-20-benchmark-fleet-and-intelligence-enrichment-research-synthesis.md`
2. `docs/plans/2026-03-20-benchmark-fleet-and-intelligence-enrichment-contract.md`

## Verification Expectations

This planning tranche is docs-only.

When implementation begins, definition of done should include:

1. bounded benchmark-result materialization,
2. exactness and capability-gate tests,
3. baseline-versus-current comparison proof,
4. proof that Monitoring renders from the same benchmark semantics,
5. and proof that config-vs-code escalation stays benchmark-driven rather than ad hoc.

## Commit Strategy

1. Keep the benchmark research, design, roadmap, and TODO updates in one atomic docs-only commit.
2. Start benchmark implementation later in separate commits once `OPS-SNAPSHOT-1` produces the required input surfaces.
