# Event-Log IP Storage Minimization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an optional storage-level event-log IP minimization mode so privacy-sensitive deployments can choose raw, masked, or keyed-pseudonymized event-log IP persistence without breaking operator truthfulness.

**Architecture:** Keep the change local to env-only config parsing plus the canonical event-log write/read helpers in `src/admin/api.rs`. Do not redesign monitoring retention or the broader telemetry architecture.

**Tech Stack:** Rust config and admin/event-log modules, focused unit tests, one focused Make target, configuration/privacy docs, and runtime inventory metadata.

---

## Guardrails

1. Do not make this controller-tunable.
2. Do not widen this into a full telemetry anonymization program.
3. Do not silently change current behavior; default must remain `raw`.
4. Do not lie about forensic raw availability once storage minimization is enabled.
5. Preserve existing event-log query paths and immutable-row model.

## Task 1: Add failing storage-mode contract tests first

**Files:**
- Modify: `src/config/tests.rs`
- Modify: `src/admin/api.rs`
- Modify: `Makefile`

**Work:**
1. Add focused failing tests for:
   - env validation/parsing of the new storage mode,
   - masked storage persisting bucketed IPs and exposing row mode,
   - pseudonymized storage persisting stable keyed pseudonyms and exposing row mode,
   - security/privacy payload surfacing the current storage mode and whether raw forensic recovery is available for new rows.
2. Add a narrow Make target for the new proof.
3. Run the target and confirm the failure is the missing storage-mode behavior.

**Acceptance criteria:**
1. Tests fail against the current raw-only implementation.
2. The desired storage contract is machine-checked before implementation.

## Task 2: Implement env-only storage mode and row metadata

**Files:**
- Modify: `config/defaults.env`
- Modify: `src/config/mod.rs`
- Modify: `src/admin/api.rs`

**Work:**
1. Add env-only `SHUMA_EVENT_LOG_IP_STORAGE_MODE` with supported values:
   - `raw`
   - `masked`
   - `pseudonymized`
2. Validate it in env boot/verification and expose a canonical accessor.
3. Apply the selected mode during event-log persistence.
4. Persist per-row mode metadata so mixed historical rows remain understandable.
5. Keep default non-forensic and forensic presentations truthful for rows stored under each mode.

**Acceptance criteria:**
1. Default `raw` mode preserves current behavior.
2. Masked/pseudonymized modes minimize newly written rows at rest.
3. Each stored row carries enough mode truth for later presentation and audit.

## Task 3: Surface operator truth and close docs/backlog

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `dashboard/static/assets/status-var-meanings.json`
- Modify: `docs/configuration.md`
- Modify: `docs/privacy-gdpr-review.md`
- Modify: `docs/testing.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/security-review.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-24-sec-gdpr-3-event-log-ip-storage-minimization-post-implementation-review.md`

**Work:**
1. Surface the current storage mode and forensic raw availability truth in security/privacy payloads and runtime inventory.
2. Document the `raw` vs `masked` vs `pseudonymized` tradeoff clearly.
3. Move `SEC-GDPR-3` to completed history and update the security-review tracker.
4. Add the post-implementation review and index references.

**Acceptance criteria:**
1. The operator/deployer can see the active storage mode and its forensic consequence.
2. The docs explain the storage tradeoffs explicitly.
3. The backlog and security tracker show `SEC-GDPR-3` as closed.

## Verification

1. `make test-event-log-ip-storage-mode`
2. `git diff --check`

## Exit Criteria

This tranche is complete when:

1. event-log IP storage mode is env-configurable and defaults to `raw`,
2. masked and pseudonymized modes minimize new rows at rest,
3. row-level mode truth preserves historical auditability,
4. operator surfaces expose the storage-mode and forensic limitation truthfully,
5. and the docs/backlog reflect the delivered contract.
