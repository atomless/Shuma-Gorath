# Agentic Recent-Run Coverage Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the `Coverage` column in `Recent Red Team Runs` truthful for `bot_red_team` rows by projecting whole-run Agentic surface coverage through the shared recent-run model.

**Architecture:** Reuse the existing LLM surface-observation helper instead of inventing a second Agentic-only parser. Add one additive recent-run coverage summary for Agentic rows, preserve Scrapling coverage unchanged, and keep monitoring deltas external-event-only.

**Tech Stack:** Rust hot-read/recent-run projection, Svelte dashboard rendering, Node dashboard unit tests, Playwright rendered proof.

---

### Task 1: Freeze the proof contract

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/todos/todo.md`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/docs/research/README.md`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/docs/plans/README.md`

**Step 1: Record the execution-ready TODO**

Add one atomic TODO item for Agentic recent-run coverage projection with explicit closure gates and proof commands.

**Step 2: Index the new research and plan notes**

Add this slice to the research and plan indexes so the planning chain remains discoverable.

### Task 2: Write the failing backend proof first

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/operator_snapshot.rs`

**Step 1: Extend the existing LLM recent-run Rust tests**

Add failing assertions that:

1. `recent_sim_run_history_projects_llm_runtime_receipts_and_categories` no longer expects empty coverage for Agentic rows when action-derived surface observations exist.
2. `snapshot_payload_projects_recent_run_llm_runtime_summary` proves the operator snapshot preserves the new Agentic coverage summary.

**Step 2: Run the focused Rust proof and verify red**

Run: `make test-adversarial-llm-runtime-projection`

Expected: the new LLM coverage assertions fail because the current recent-run/operator surfaces do not yet project Agentic coverage.

### Task 3: Write the failing dashboard proof first

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/e2e/dashboard.modules.unit.test.js`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/e2e/dashboard.smoke.spec.js`

**Step 1: Extend the shared Red Team pane contracts**

Add failing assertions that:

1. the monitoring view-model preserves an additive Agentic coverage summary,
2. and the rendered Red Team table shows a non-empty coverage cell for an Agentic row with observed surface evidence.

**Step 2: Run the focused dashboard proof and verify red**

Run: `make test-dashboard-red-team-pane`

Expected: the new Agentic coverage assertions fail because the table still only renders Scrapling coverage.

### Task 4: Implement the additive shared coverage summary

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/llm_surface_observation.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/hot_read_documents.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/operator_snapshot_live_traffic.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/hot_read_projection.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/dashboard/src/lib/components/dashboard/monitoring-view-model.js`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte`

**Step 1: Add a serializable Agentic coverage summary**

Model one additive summary for Agentic recent-run surface coverage that captures:

1. overall status,
2. observed/progress surface ids and counts,
3. labels,
4. and whole-run per-surface receipts.

**Step 2: Aggregate Agentic surface observations whole-run**

During recent-run accumulation, derive LLM surface observations per receipt event and merge them across the whole run instead of relying on `latest_action_receipts` from the merged runtime summary.

**Step 3: Project the new summary through hot-read and operator snapshot**

Keep Scrapling coverage unchanged and add the Agentic coverage summary as a separate optional field.

**Step 4: Render the shared Coverage column honestly**

Teach the dashboard row shaper and panel formatter to:

1. use Scrapling owned-surface coverage when present,
2. otherwise use the Agentic coverage summary,
3. and render Agentic coverage in a way that is honest about observed surface evidence rather than pretending Scrapling-style required-surface closure exists.

### Task 5: Verify, document, and archive

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/docs/dashboard-tabs/red-team.md`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/todos/todo.md`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/todos/completed-todo-history.md`

**Step 1: Run focused proof**

Run:

- `make test-adversarial-llm-runtime-projection`
- `make test-dashboard-red-team-pane`
- `make test-code-quality`

Expected: all pass.

**Step 2: Update operator docs**

Document that `Coverage` in `Recent Red Team Runs` now uses:

1. Scrapling owned-surface closure for `scrapling_traffic`,
2. and additive Agentic surface-observation coverage for `bot_red_team`.

**Step 3: Move the TODO to completed history**

Archive the completed TODO with the landed files and proof commands.
