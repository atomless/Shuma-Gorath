# Telemetry Retention Rebaseline Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Re-evaluate raw-event, hot-read-summary, and rollup retention windows now that compact event telemetry is live and measured, then implement only the retention changes that are justified by evidence.

**Architecture:** Keep the accepted ADR 0009 lifecycle model intact: automatic purge stays the governing mechanism, retention health remains operator-visible, and tier decisions are made from measured retained-byte pressure rather than intuition. The compact event schema is now one input to the decision, but the whole-system retained footprint must include raw rows, retention metadata, and hot-read documents before defaults move.

**Tech Stack:** Python live-evidence helpers, Rust config/retention modules, Makefile verification, shared-host and Fermyon live telemetry receipts

---

## Context and Assumptions

1. `TEL-EVT-1-5` is complete and the compact schema has live proof on both shared-host and Fermyon targets.
2. The latest shared-host live evidence shows raw eventlog values (`2411 B`) are smaller than hot-read documents (`17295 B`) and retention metadata (`2268 B`) on the low-volume sample.
3. Because that host is low traffic, the next tranche must gather a denser challenge-heavy sample before changing retention defaults for the whole product.
4. Pre-launch rules still apply: no compatibility shim layer, no second telemetry storage model, and no read-path retention cleanup regression.

## Task 1: Lock In Tier-Pressure Measurement

**Files:**
- Modify: `scripts/tests/telemetry_shared_host_evidence.py`
- Modify: `scripts/tests/telemetry_fermyon_edge_evidence.py`
- Test: `scripts/tests/test_telemetry_shared_host_evidence.py`
- Test: `scripts/tests/test_telemetry_fermyon_edge_evidence.py`
- Doc: `docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`

**Steps:**

1. Add or extend live-evidence fields so retained-byte pressure is captured in a reviewable shape for each telemetry tier.
2. Keep the shared-host helper authoritative for exact retained KV bytes and keep the Fermyon helper authoritative for live payload budgets and compact-row shape proof.
3. If a higher-volume challenge-heavy live sample is still missing, add an explicit evidence field or note that marks the current sample as low-volume so later reviewers do not over-generalize it.
4. Run:
   - `make test-telemetry-hot-read-evidence`
   - `make test-telemetry-hot-read-live-evidence`
5. Commit the evidence-tooling slice separately once green.

## Task 2: Decide Whether Defaults Should Move

**Files:**
- Modify: `config/defaults.env`
- Modify: `src/config/mod.rs`
- Modify: `src/config/tests.rs`
- Modify: `src/observability/retention.rs`
- Modify: `docs/configuration.md`
- Modify: `docs/observability.md`
- Doc: `docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`

**Steps:**

1. Compare current effective windows:
   - raw event evidence: `72h` enforced cap
   - monitoring summaries: `168h`
   - monitoring rollups: `720h`
2. Use the measured retained-byte pressure, live payload budgets, and ADR 0009 security/privacy constraints to decide one of two outcomes:
   - keep current defaults because the evidence is still too sparse or the privacy/security tradeoff still dominates, or
   - change one or more windows because the retained-footprint evidence clearly supports it.
3. Record the decision and rationale explicitly in docs, including why the compact schema did or did not justify a retention change yet.
4. Add config parsing/default tests for any changed retention variable semantics.
5. Commit the retention-decision slice separately once the rationale and tests are aligned.

## Task 3: Implement Retention Contract Changes Only If Justified

**Files:**
- Modify: `config/defaults.env`
- Modify: `scripts/config_seed.sh`
- Modify: `scripts/bootstrap/setup.sh`
- Modify: `Makefile`
- Modify: `src/config/mod.rs`
- Modify: `src/observability/retention.rs`
- Modify: `src/admin/api.rs`
- Test: `src/observability/retention.rs`
- Test: `src/config/tests.rs`
- Test: `src/admin/api.rs`

**Steps:**

1. If Task 2 concludes defaults should change, update the canonical env defaults first.
2. Preserve the existing lifecycle rules:
   - purge remains automatic,
   - retention health stays truthful,
   - no read-path cleanup returns,
   - eventlog high-risk governance stays explicit.
3. Keep bootstrap/setup/config-seed flows in sync with the new defaults.
4. Add regression coverage for:
   - parsed retention values,
   - effective eventlog cap behavior,
   - retention health payload truthfulness after the change.
5. Run:
   - `make test-telemetry-storage`
   - `make test-unit`
6. Commit this slice only if Task 2 justified a real contract change.

## Task 4: Re-Prove Live Budgets and Operational Truth

**Files:**
- Modify: `docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`
- Modify: `docs/testing.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Steps:**

1. Re-run the canonical live telemetry proof after any retention change:
   - `make test-telemetry-hot-read-live-evidence`
2. Confirm all of the following remain true:
   - shared-host bootstrap/delta budgets are green,
   - Fermyon bootstrap/delta budgets are green,
   - retention health remains healthy,
   - the compact recent-event shape is still live.
3. Update evidence notes and TODO history with the final result.
4. Commit the final proof/docs slice separately.

## Definition of Done

1. The repository has an explicit evidence-backed answer for whether the compact schema should change retention defaults now.
2. If defaults change, config/bootstrap/docs/tests all move together with no lifecycle drift.
3. If defaults do not change, that is also documented explicitly with measured reasoning.
4. Live shared-host and Fermyon evidence remain within the current telemetry budget envelope.
5. Automatic purge/default-on lifecycle governance remains the controlling model throughout.
