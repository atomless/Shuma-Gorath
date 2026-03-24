# Compact Event Telemetry Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use systematic-debugging for telemetry-shape regressions and verification-before-completion before claiming completion.

**Goal:** Reduce persisted event-log row size and hot-read payload weight by replacing verbose blended outcome strings with a compact canonical event schema, making rows sparse where semantically safe, and correcting the dashboard raw-feed truthfulness contract.

**Architecture:** Keep one shared telemetry architecture and explicitly preserve the completed `TEL-HOT` hot-read design. The event log remains the source of truth, but the canonical persisted row becomes compact and machine-oriented. Human-readable display text moves to the dashboard/read path. No Fermyon-only path, no SQLite split, no external database, no reference-dictionary indirection, and no regression in the live Linode/Fermyon hot-read budget contract.

**Tech Stack:** Rust, Spin KV, Svelte dashboard, Makefile verification, live shared-host and Fermyon telemetry evidence helpers

---

## Scope

This plan covers:

1. persisted event-row compaction,
2. dashboard raw/rendered feed truthfulness,
3. verification that hot-read telemetry remains fast on Linode and Fermyon after the schema change.

This plan must preserve the `TEL-HOT` tranche outcomes:

1. one shared KV-backed hot-read architecture across Linode and Fermyon,
2. no new whole-keyspace scans in the hot path,
3. no new storage/query path that bypasses existing hot-read documents,
4. no new correctness dependence on unsupported multi-writer KV mutation,
5. the current live budget envelope proven by `make test-telemetry-hot-read-live-evidence`.

This plan does **not** introduce:

1. a separate cold archive system,
2. a new external database,
3. a global dictionary-based rehydration scheme,
4. a Fermyon-only telemetry variant,
5. a second parallel telemetry architecture outside the existing hot-read document model.

## Design Decisions

### 1. Persist compact machine fields, not verbose narrative strings

The canonical event row should store:

1. event type,
2. timestamp,
3. optional actor fields only when present,
4. simulation and execution metadata,
5. compact reason/action/taxonomy fields,
6. only active or non-default signal facts,
7. optional compact scoring/banding fields when meaningful.

It should **not** store:

1. display labels,
2. duplicate human-readable and machine-readable variants,
3. default-zero or default-disabled signal-state matrices,
4. repetitive provider/mode matrices on every event when inferable from runtime/config context.

### 2. Make rows sparse

Optional fields in the persisted event record must be omitted when absent, not serialized as `null`, unless explicit null-vs-missing semantics are required.

### 3. Keep events self-describing enough for export

Do not use a global shared reference object or dictionary that is required to interpret events. Compactness should come from:

1. implicit defaults,
2. omission of absent/default values,
3. structured compact fields,

not from external hydration dependencies.

### 4. Make the dashboard honest about “raw”

The dashboard must not present display-normalized events as if they were raw persisted rows.

Acceptable end states:

1. a true raw feed backed by persisted event rows only, plus a separate rendered/operator-friendly feed, or
2. a renamed rendered feed that no longer claims to be raw.

### 5. Preserve `TEL-HOT` live budget guarantees

Every implementation step in this tranche must preserve the current hot-read guarantees:

1. bootstrap and delta must continue to flow through the existing bounded hot-read documents,
2. compaction must not reintroduce expensive reconstruction into `/admin/monitoring?bootstrap=1...` or initial delta reads,
3. any additional shaping or hydration must be cheap enough that the current Fermyon and shared-host evidence budgets remain green.

### 6. Optimize value shape and sparsity, not schema legibility

This tranche must not pursue byte reduction through:

1. cryptic key minification,
2. shared reference/dictionary hydration,
3. duplicate raw + rendered heavyweight payloads,
4. full provider/mode/default signal matrices stored on every event by default.

The main savings must come from:

1. sparse omission of semantically safe absent fields,
2. compact structured machine fields,
3. removal of duplicated narrative/machine payloads,
4. keeping verbose expansion bounded to explicit forensic/debug surfaces only when justified.

## Task Breakdown

### Task 1: Define the compact persisted event schema

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/runtime/policy_taxonomy.rs`
- Test: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`

Steps:

1. Inventory which current event fields are canonical vs display-derived.
2. Define the compact persisted schema shape.
3. Add tests that assert:
   - sparse omission of absent fields,
   - compact structured taxonomy fields,
   - no duplicate display-only fields.
4. Explicitly prove the new schema does not require a parallel read path outside the existing `TEL-HOT` hot-read architecture.

### Task 2: Replace verbose challenge outcome encoding

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/runtime/effect_intents/intent_executor.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/runtime/policy_taxonomy.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`
- Test: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`

Steps:

1. Stop using long free-text `outcome` strings as the canonical challenge/botness event payload.
2. Persist compact machine fields instead.
3. Keep any rich narrative only in a bounded optional debug/forensic surface if still justified.
4. Add tests proving the persisted row is materially smaller and still analyzable.
5. Ensure the compact encoding can be rendered through the existing hot-read bootstrap/delta path without reintroducing expensive on-demand reconstruction.
6. Avoid storing provider/mode/default-disabled state matrices in canonical rows unless the event carries a non-default deviation that is genuinely event-specific.

### Task 3: Rework the dashboard feed contract

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/dashboard/src/lib/components/dashboard/GameLoopTab.svelte`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/dashboard/src/lib/components/dashboard/monitoring-view-model.js`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/dashboard/src/lib/components/dashboard/monitoring/RecentEventsTable.svelte`
- Test: `/Users/jamestindall/Projects/Shuma-Gorath/e2e/dashboard.modules.unit.test.js`

Steps:

1. Decide and implement either:
   - true raw feed plus rendered feed, or
   - rendered feed with honest naming.
2. Derive human-readable outcome/execution labels from compact canonical fields.
3. Ensure the table and filters still behave correctly.
4. Keep any display-only hydration in the dashboard/read path lightweight enough that it does not undermine the `TEL-HOT` latency gains.
5. Avoid duplicating fat event objects for “raw” and “rendered” views when one compact persisted record plus a lightweight derived view will do.

### Task 4: Ensure hot-read documents benefit from the new compact rows

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/hot_read_documents.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/hot_read_projection.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`
- Test: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`

Steps:

1. Confirm the embedded recent-event rows in hot-read documents now carry the compact shape.
2. Preserve existing bounded hot-read behavior and cursor semantics.
3. Add regression tests that the compact event rows still serve bootstrap/delta correctly.
4. Prove no new whole-keyspace scans or alternate hot-read storage paths were introduced.

### Task 5: Re-prove live performance and storage impact

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/tests/telemetry_shared_host_evidence.py`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/tests/telemetry_fermyon_edge_evidence.py`
- Modify docs/evidence notes as needed

Steps:

1. Extend evidence helpers to record representative event-row byte sizes before/after where practical.
2. Extend evidence helpers to record recent-events-tail document bytes and bootstrap payload bytes before/after where practical.
3. Re-run:
   - `make test-telemetry-hot-read-bootstrap`
   - `make test-telemetry-hot-read-evidence`
   - `make telemetry-shared-host-evidence`
   - `make telemetry-fermyon-edge-evidence`
4. Confirm no regression in hot-read budgets and document the size win across rows, recent-events-tail documents, and bootstrap payloads.
5. Treat any Linode or Fermyon hot-read budget regression as a release-blocking failure for this tranche.
6. Treat a representative challenge-heavy sample that fails to achieve a material byte reduction as a review gate; default expectation is at least a 25% reduction unless a documented review concludes further reduction would materially harm truthfulness or queryability.

## Verification

Minimum required:

1. focused backend tests for compact event schema and hot-read compatibility,
2. dashboard unit coverage for raw/rendered feed truthfulness,
3. live shared-host telemetry evidence,
4. live Fermyon telemetry evidence,
5. explicit confirmation that the current `TEL-HOT` budget envelope remains green.

## Definition of Done

1. Persisted event rows are smaller and sparse where semantically safe.
2. Verbose blended outcome narratives are no longer the canonical stored event shape.
3. Dashboard no longer mislabels display-normalized telemetry as raw.
4. Hot-read bootstrap/delta budgets remain within current targets on both Fermyon and Linode.
5. Recent-events-tail hot-read documents and bootstrap payloads are also materially smaller, not just isolated persisted rows.
6. No new storage backend, no Fermyon-only path, no external dictionary hydration scheme, and no schema-minification trickery has been introduced.
7. The tranche is demonstrably an optimization within the existing `TEL-HOT` architecture, not a parallel telemetry system.
