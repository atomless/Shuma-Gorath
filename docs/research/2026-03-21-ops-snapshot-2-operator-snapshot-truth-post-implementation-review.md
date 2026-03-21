# OPS-SNAPSHOT-2 Post-Implementation Review

Date: 2026-03-21
Plan reference: `docs/plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`
Task: `OPS-SNAPSHOT-2`

## Scope Delivered

`OPS-SNAPSHOT-2` completed the planned snapshot-truth tranche by replacing backend-default and placeholder state with persisted objective, decision, and verified-identity contracts:

- `src/observability/operator_snapshot_objectives.rs`
  - defines the persisted `operator_objectives_v1` contract, typed adversary-sim expectations, rollout guardrails, and validation rules
- `src/observability/operator_objectives_store.rs`
  - persists and seeds site-owned objective profiles
- `src/admin/operator_objectives_api.rs`
  - exposes the primary-state `GET` and `POST /admin/operator-objectives` surface
- `src/observability/decision_ledger.rs`
  - persists bounded decision records, expected-impact summaries, and compact evidence references
- `src/admin/recent_changes_ledger.rs`
  - joins recent-change rows with decision IDs, objective revisions, evidence references, and per-decision watch-window semantics
- `src/observability/operator_snapshot_verified_identity.rs`
  - replaces the placeholder with a typed verified-identity summary derived from current policy state and observed telemetry
- `src/observability/operator_snapshot.rs`
  - loads persisted objectives, materializes the typed verified-identity section, and threads decision-evidence-enriched recent-change rows into `operator_snapshot_v1`
- `src/observability/hot_read_contract.rs`
  - updates the machine-first hot-read contract to reflect persisted objectives, decision-evidence summaries, and typed verified-identity projection
- `src/admin/api.rs`
  - routes the new operator-objectives endpoint, updates write access rules, and makes manual config writes emit decision-ledger records

## Plan Conformance Review

### 1. Persisted writable `operator_objectives_v1`

Delivered. Objectives are now a persisted site-owned contract with server-assigned revision metadata rather than a backend-default-only snapshot field.

### 2. Objective revision or reference in the snapshot

Delivered. `operator_snapshot_v1` now carries the persisted objective revision inside the objectives contract, and decision-linked recent-change rows also reference the objective revision active at change time.

### 3. Typed verified-identity summary

Delivered. The snapshot no longer exposes a placeholder note. It now carries a bounded typed summary with enablement, stance, policy counts, verification totals, and compact policy-tranche telemetry.

### 4. Causal decision and watch evidence

Delivered. `recent_changes` is still the bounded summary section, but it now includes decision IDs, decision kind or status, expected-impact summary, evidence references, objective revision, and watch-window status derived from a durable decision ledger.

### 5. Durable evidence references for later reconcile reasoning

Delivered. Manual config changes and operator-objectives updates now record bounded decision rows with durable evidence references that later reconcile work can reuse instead of reconstructing purely from prose or raw config diffs.

## Verification Evidence

The tranche was verified with the focused snapshot, objective, and hot-read gates required by the plan:

- `make test-operator-objectives-contract`
- `make test-operator-snapshot-foundation`
- `make test-telemetry-hot-read-contract`
- `make test-telemetry-hot-read-projection`
- `git diff --check`

Focused regression proof for the new primary-state and decision-evidence path lives in:

- `src/admin/operator_objectives_api.rs`
- `src/observability/operator_objectives_store.rs`
- `src/observability/decision_ledger.rs`
- `src/observability/operator_snapshot.rs`
- `src/admin/recent_changes_ledger.rs`

## Architectural Result

The snapshot is now materially closer to the recommend-only reconcile loop the plan wants:

- one persisted objective contract now defines the site-owned target function,
- one typed verified-identity section now exposes policy-relevant non-human posture truth,
- and one bounded decision-evidence chain now links changes to objective revision and watch context.

That means later `OVR-RECON-1` and `OVR-AGENT-1` work can consume one coherent machine-first contract chain rather than inferring operator intent from backend defaults and placeholder text.

## Shortfall Check

One tranche-local shortfall was found during verification: the first typed verified-identity and objective revision implementation pushed `operator_snapshot_v1` slightly over the existing 32 KiB hot-read budget and left one stale hot-read projection expectation behind the old backend-default profile ID.

That shortfall was closed immediately as `OPS-SNAPSHOT-2-REVIEW-1` by trimming zero-value verified-identity serialization, updating the stale hot-read expectation, and rerunning the focused projection gates.

No `OPS-SNAPSHOT-2` shortfall remains open before proceeding to `ADV-PROMO-1`.
