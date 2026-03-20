# Machine-First Operator Snapshot and Feedback Loop Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the machine-first objective and operator-snapshot foundations that future Monitoring, Tuning, and scheduled controller work will consume.

**Architecture:** Keep the current telemetry foundation as the source of truth, add a bounded `operator_objectives_v1` and `operator_snapshot_v1` layer above it, treat the human Monitoring tab as a later projection over that backend contract instead of inventing UI-first semantics, and preserve a separate later benchmark-driven code-evolution loop so Shuma can evolve its own code as part of the arms race rather than only per-instance config.

**Tech Stack:** Rust hot-read documents and admin APIs, existing monitoring summary materialization, dashboard read-model consumers, Markdown planning and backlog docs.

---

## Phase 1: Objective and Budget Contract

### Task 1: Define `operator_objectives_v1`

**Files:**

- Create: `docs/plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`
- Modify later implementation targets in:
  - `src/config/mod.rs`
  - `src/admin/api.rs`
  - `dashboard/src/lib/domain/config-schema.js`

**Outcome:**

Define the minimal objective schema for:

1. likely-human friction budget,
2. suspicious-origin leakage or cost budget,
3. non-human posture stance,
4. adversary-sim benchmark expectations,
5. and later controller safety rails.

**Guardrails:**

1. Keep trust-boundary controls manual-only.
2. Do not let objectives silently imply policy changes.
3. Keep the first objective schema small and numeric where possible.

### Task 2: Define budget-distance semantics

**Files:**

- Modify later implementation targets in:
  - `src/observability/monitoring.rs`
  - `src/observability/hot_read_projection.rs`

**Outcome:**

Specify how Shuma computes:

1. current value,
2. target,
3. delta,
4. tolerance band,
5. and status (`inside`, `near_limit`, `outside`).

This is the minimum controller-facing layer that avoids forcing agents to infer meaning from raw counters.

## Phase 2: `operator_snapshot_v1` Foundation

### Task 3: Materialize a bounded `operator_snapshot_v1`

**Files:**

- Modify:
  - `src/observability/hot_read_contract.rs`
  - `src/observability/hot_read_documents.rs`
  - `src/observability/hot_read_projection.rs`
  - `src/observability/monitoring.rs`

**Outcome:**

Add a new bounded hot-read snapshot that reuses:

1. current monitoring summaries,
2. live-vs-sim origin separation,
3. existing exactness machinery,
4. and config/runtime posture summaries.

It must remain bounded and budget-safe, and it must not widen raw event tails.

### Task 4: Add snapshot sections in strict order of value

**Section order:**

1. metadata and exactness,
2. objective profile reference,
3. live traffic summary,
4. shadow summary,
5. adversary-sim summary,
6. recent changes summary,
7. budget distance,
8. allowed action envelope.

**Guardrails:**

1. No free-form narrative required for first version.
2. Prefer enums, scalars, ratios, and capped top-Ns.
3. Keep drill-down detail out of bootstrap unless proven necessary.

## Phase 3: Read Contract and Ledger Support

### Task 5: Expose snapshot through a dedicated admin read path

**Files:**

- Modify:
  - `src/admin/api.rs`
  - `docs/api.md`
  - `docs/dashboard.md`

**Outcome:**

Add a dedicated admin read endpoint for `operator_snapshot_v1`.

Requirements:

1. dedicated endpoint or clearly separated payload section,
2. explicit schema version,
3. explicit freshness and exactness metadata,
4. no raw event tail dependence,
5. no hidden write-on-read behavior.

### Task 6: Add recent-change ledger summary

**Files:**

- Modify:
  - hot-read projection and relevant config-write/audit paths

**Outcome:**

Add a bounded summary of recent meaningful config changes, sufficient for future controller context:

1. when it changed,
2. what family changed,
3. manual vs future automated source,
4. target objective if known,
5. and watch-window status if available.

This should stay summary-level; full audit detail belongs elsewhere.

## Phase 4: Allowed Action Surface

### Task 7: Define `allowed_actions_v1`

**Files:**

- Modify later implementation targets in:
  - `src/config/mod.rs`
  - `src/admin/api.rs`
  - docs and backlog references

**Outcome:**

Enumerate the safe config families a future scheduled controller may propose against.

Include:

1. allowed families,
2. manual-only families,
3. forbidden families,
4. canary-required families,
5. safe min/max envelopes.

This should not yet enable automation; it just defines the boundary.

## Phase 5: Monitoring Projection

### Task 8: Reframe `MON-OVERHAUL-1`

**Files:**

- Modify:
  - `todos/blocked-todo.md`
  - `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
  - later dashboard implementation docs

**Outcome:**

Make Monitoring explicitly a projection over `operator_snapshot_v1`.

The future Monitoring tab should answer:

1. Are we meeting targets?
2. Where is suspicious traffic still getting through?
3. Where are likely humans paying friction cost?
4. What would shadow mode have done?
5. How is adversary-sim performing?
6. What changed recently?

Do not let the Monitoring UI define semantics ahead of the snapshot.

## Phase 6: Scheduled Controller Planning Handoff

### Task 9: Use snapshot and action contracts as the gate for `OVR-AGENT-2`

**Files:**

- Modify:
  - `todos/blocked-todo.md`
  - future oversight planning docs

**Outcome:**

Make the scheduled controller planning depend on:

1. `operator_objectives_v1`,
2. `operator_snapshot_v1`,
3. `allowed_actions_v1`,
4. and the completed Tuning surface.

That ensures later controller design starts from truthful inputs and bounded outputs.

## Phase 7: Benchmark-Driven Project Evolution Handoff

### Task 10: Define the benchmark contract for code evolution

**Files:**

- Create later benchmark-design and project-evolution docs, beginning with:
  - `docs/research/2026-03-20-benchmark-suite-v1-research-synthesis.md`
  - `docs/plans/2026-03-20-benchmark-suite-v1-design.md`
  - `docs/plans/2026-03-20-benchmark-suite-v1-implementation-plan.md`
- Modify:
  - `todos/blocked-todo.md`
  - roadmap docs

**Outcome:**

Define:

1. `benchmark_suite_v1`,
2. `benchmark_results_v1`,
3. the separation between instance tuning and project evolution,
4. and the gate that determines when Shuma needs code changes rather than only config changes.

### Task 11: Keep code and PR generation as a later reviewed path

**Outcome:**

Make explicit that:

1. config tuning is the first automation loop,
2. code evolution is the second loop,
3. and code or PR generation must remain benchmark-driven and review-heavy rather than being bundled into the first operator loop.

## Verification Expectations

This planning tranche is docs-only.

When implementation begins, definition of done should include:

1. bounded hot-read proof,
2. exactness and freshness contract tests,
3. API contract tests for the snapshot endpoint,
4. proof that live and adversary-sim remain separate,
5. proof that Monitoring later renders from the same snapshot contract rather than inventing a parallel model.

## Commit Strategy

When implemented, keep atomic commits roughly in this order:

1. objective schema and exactness metadata,
2. snapshot materialization,
3. snapshot read endpoint and tests,
4. recent-change ledger summary,
5. allowed action contract,
6. Monitoring projection work,
7. later controller planning or dry-run loop work.
