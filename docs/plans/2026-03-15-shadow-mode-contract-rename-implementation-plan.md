# Shadow Mode Contract Rename Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the pre-launch split shadow-toggle contract with a single clean `shadow_mode` contract and remove the redundant shadow-execution source abstraction.

**Architecture:** Keep the useful internal distinction between enforced execution and shadow execution, but make `shadow_mode` the only operator-facing term and config key. Collapse `ShadowSource` out of the runtime/event contract so monitoring and event rows keep only the durable truths: whether execution was enforced, what would have happened, and whether enforcement was applied.

**Tech Stack:** Rust runtime/admin API, Svelte dashboard, JS dashboard/domain tests, shell/Python test harness scripts, Markdown docs.

---

### Task 1: Record the rename and cleanup contract

**Files:**
- Modify: `docs/plans/2026-03-12-shadow-mode-telemetry-monitoring-truthfulness-plan.md`
- Modify: `docs/configuration.md`
- Modify: `docs/dashboard-tabs/game-loop.md`

**Step 1: Write the failing doc assertions**

Update dashboard/unit source-contract assertions so the old split shadow-toggle/source field shape is no longer expected in the operator-facing/dashboard contract.

**Step 2: Run test to verify it fails**

Run: `make test-dashboard-unit`
Expected: FAIL on the old dashboard contract expectations.

**Step 3: Write minimal implementation**

Update design and operator docs so they describe one operator-visible mode, `shadow_mode`, and no longer describe a separate shadow source field as part of the durable telemetry contract.

**Step 4: Run test to verify it passes**

Run: `make test-dashboard-unit`
Expected: the changed contract assertions pass or move to the next failing rename surface.

### Task 2: Rename the runtime/config contract

**Files:**
- Modify: `src/config/mod.rs`
- Modify: `config/defaults.env`
- Modify: `scripts/config_seed.sh`
- Modify: `src/admin/api.rs`
- Modify: `src/observability/hot_read_documents.rs`
- Modify: `src/observability/hot_read_projection.rs`
- Modify: `dashboard/src/lib/domain/config-schema.js`
- Modify: `dashboard/src/lib/domain/api-client.js`
- Modify: `dashboard/src/lib/runtime/dashboard-native-runtime.js`
- Modify: `dashboard/src/routes/+page.svelte`

**Step 1: Write the failing test**

Adjust focused config/admin/dashboard contract tests to expect `shadow_mode` as the sole operator/runtime toggle.

**Step 2: Run test to verify it fails**

Run: `make test-unit`
Expected: FAIL on config/admin/runtime tests that still emit or accept the old toggle contract.

**Step 3: Write minimal implementation**

Rename the persisted/admin/dashboard/runtime config key to `shadow_mode` everywhere without any compatibility alias.

**Step 4: Run test to verify it passes**

Run: `make test-unit`
Expected: focused runtime/admin/config tests for the rename pass.

### Task 3: Remove redundant shadow-source abstraction

**Files:**
- Modify: `src/runtime/effect_intents/intent_types.rs`
- Modify: `src/runtime/effect_intents/intent_executor.rs`
- Modify: `src/runtime/shadow_mode.rs` or renamed module files
- Modify: `src/runtime/request_flow.rs`
- Modify: `src/runtime/request_router.rs`
- Modify: `src/runtime/policy_pipeline.rs`
- Modify: `src/lib.rs`
- Modify: `src/admin/api.rs`
- Modify: `docs/observability.md`

**Step 1: Write the failing test**

Adjust event-log/telemetry tests to assert that `execution_mode`, `intended_action`, and `enforcement_applied` remain, while the redundant shadow source field is absent.

**Step 2: Run test to verify it fails**

Run: `make test-unit`
Expected: FAIL on event-log persistence tests still expecting the redundant source field.

**Step 3: Write minimal implementation**

Remove the dedicated source enum, simplify `ExecutionMode::Shadow`, and update event persistence/monitoring payloads to omit the redundant source field.

**Step 4: Run test to verify it passes**

Run: `make test-unit`
Expected: telemetry/runtime tests for shadow execution pass with the smaller contract.

### Task 4: Clean up dashboard monitoring semantics

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/GameLoopTab.svelte`
- Modify: `dashboard/src/lib/components/dashboard/monitoring-view-model.js`
- Modify: `dashboard/src/lib/components/dashboard/monitoring/ShadowSection.svelte`
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`

**Step 1: Write the failing test**

Add focused dashboard tests proving:
- there is no standalone `Shadow Mode` section,
- Monitoring charts/readouts do not silently imply a separate subsystem,
- operator copy uses `shadow mode` consistently.

**Step 2: Run test to verify it fails**

Run: `make test-dashboard-unit`
Expected: FAIL on old `ShadowSection` import/render expectations and outdated copy assertions.

**Step 3: Write minimal implementation**

Remove or fold the standalone shadow section into clearer monitoring affordances, keep shadow semantics in labeled event/trend displays, and align dashboard wording to `shadow_mode`.

**Step 4: Run test to verify it passes**

Run: `make test-dashboard-unit`
Expected: monitoring/dashboard contract assertions pass.

### Task 5: Update scripts, harnesses, and docs

**Files:**
- Modify: `scripts/tests/integration.sh`
- Modify: `scripts/tests/adversarial_simulation_runner.py`
- Modify: `scripts/tests/test_config_lifecycle.py`
- Modify: `e2e/seed-dashboard-data.js`
- Modify: `docs/api.md`
- Modify: `docs/dashboard.md`
- Modify: `docs/testing.md`
- Modify: `docs/quick-reference.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Write the failing test**

Adjust focused harness/docs source assertions to expect `shadow_mode` and no redundant shadow source field.

**Step 2: Run test to verify it fails**

Run: `make test-unit`
Expected: FAIL on harness/unit assertions still carrying the old toggle name.

**Step 3: Write minimal implementation**

Rename scripts/harness payloads and docs to `shadow_mode`, update operator explanations, and archive the work in TODO history.

**Step 4: Run test to verify it passes**

Run: `make test-unit`
Expected: unit-level harness and contract tests pass.

### Task 6: Verification and finish

**Files:**
- Modify: `.spin/last-full-test-pass.json` only if a full `make test` run is actually completed successfully

**Step 1: Run targeted verification**

Run:
- `make test-unit`
- `make test-dashboard-unit`
- `git diff --check`

Expected: all rename/contract changes pass focused verification; report unrelated pre-existing failures separately if they persist.

**Step 2: Decide whether full-suite verification is required now**

Check `.spin/last-full-test-pass.json` and worktree/HEAD state. Because this tranche changes runtime, scripts, and dashboard contracts broadly, a fresh full `make test` may ultimately be needed before commit/push unless the user explicitly chooses to batch further work first.

**Step 3: Update audit trail**

Prepend a dated entry to `todos/completed-todo-history.md` describing the rename, the telemetry cleanup, and the verification performed.
