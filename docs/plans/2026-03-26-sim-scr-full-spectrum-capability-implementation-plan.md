# Scrapling Full-Spectrum Capability Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expand the Scrapling adversary-sim lane from the current request-native baseline into the full attacker-relevant capability now ratified for Shuma, including browser automation, stealth automation, truthful `automated_browser` coverage, and receipt-backed browser-surface proof.

**Architecture:** Preserve the already-landed request-native personas, then add two bounded browser personas to the same Scrapling lane: a dynamic browser mode and a stealth browser mode. Use those browser personas to attack Shuma's current browser-relevant surfaces directly, move `automated_browser` into Scrapling ownership, and keep every new claim receipt-backed at the worker, recent-run, snapshot, and contract layers. Treat proxy support as an immediate in-scope follow-on inside `SIM-SCR-FULL-1B`, not a hidden omission.

**Tech Stack:** Rust adversary-sim control plane, Python Scrapling worker, repo-owned Scrapling runtime bootstrap, Playwright or Patchright-backed Scrapling browser sessions, canonical non-human coverage contracts, focused Makefile verification.

---

## Guardrails

1. Do not over-claim `browser_agent` or `agent_on_behalf_of_human`; only `automated_browser` moves in this tranche.
2. Do not claim Cloudflare solving as Shuma defense coverage.
3. Do not use a misleading request-native-only verification target to prove browser or stealth work.
4. Do not widen browser surfaces without worker receipts and recent-run projection proof.
5. Keep request-native personas intact while adding browser or stealth personas.
6. Keep `cdp_report_ingestion` out of Scrapling ownership; attackers should trigger Shuma's detection logic, not manually self-report that route.

### Task 1: Freeze The New Contract In Tests

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/non_human_lane_fulfillment.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/scrapling_owned_surface.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/adversary_sim_lane_runtime.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/tests/test_adversarial_coverage_contract.py`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/tests/adversarial/coverage_contract.v2.json`

**Step 1: Write the failing Rust and contract tests**

- Add test expectations for:
  - Scrapling owning `automated_browser`,
  - new fulfillment modes `browser_automation` and `stealth_browser`,
  - browser surfaces moving into Scrapling ownership,
  - and the runtime mode cycle widening beyond the old three request-native personas.

**Step 2: Run the focused tests to verify they fail**

Run:

```bash
make test-adversary-sim-scrapling-category-fit
make test-adversary-sim-scrapling-owned-surface-contract
make test-adversarial-coverage-contract
```

Expected: failures showing the current request-native-only mappings are now stale.

**Step 3: Write the minimal Rust and contract changes**

- Reassign `automated_browser` to `scrapling_traffic`.
- Add new Scrapling fulfillment modes and surface targets.
- Move `maze_navigation`, `js_verification_execution`, and `browser_automation_detection` into Scrapling ownership.
- Update the coverage contract JSON to match.

**Step 4: Run the focused tests to verify they pass**

Run:

```bash
make test-adversary-sim-scrapling-category-fit
make test-adversary-sim-scrapling-owned-surface-contract
make test-adversarial-coverage-contract
```

### Task 2: Add Failing Worker Tests For Browser And Stealth Personas

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/tests/test_scrapling_worker.py`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/Makefile`

**Step 1: Write the failing worker tests**

- Add a `browser_automation` test proving:
  - public traversal still occurs,
  - JS verification execution is observed,
  - maze navigation is observed,
  - and category or mode metadata is tagged as browser automation.
- Add a `stealth_browser` test proving:
  - the stealth persona executes browser automation,
  - browser-automation detection is observed as a pass or fail outcome,
  - and the worker emits signed simulation headers under the new mode.
- Add a focused Make target whose name truthfully covers browser or stealth capability instead of reusing the request-native-only target name.

**Step 2: Run the focused worker tests to verify they fail**

Run:

```bash
make test-adversary-sim-scrapling-worker
```

Expected: failures because the worker currently rejects the new modes and lacks the browser session paths.

**Step 3: Commit point**

Do not commit yet; continue directly into the minimal implementation while the red state is understood.

### Task 3: Implement Browser And Stealth Worker Paths

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/supervisor/scrapling_worker.py`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/bootstrap/scrapling_runtime.sh`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/adversary_sim_worker_plan.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/adversary_sim_lane_runtime.rs`

**Step 1: Write the minimal implementation**

- Import `DynamicSession` and `StealthySession`.
- Extend the worker plan validation to accept the new modes.
- Add browser-session runtime paths needed for:
  - `/pow`,
  - and a generated maze entry path.
- Add dynamic-browser execution that:
  - traverses public pages,
  - executes JS verification,
  - and progresses through the maze.
- Add stealth-browser execution that:
  - exercises the same browser path with stealth settings,
  - and evaluates Shuma's browser-detection probe.
- Expand bootstrap readiness so the repo-owned runtime proves the browser fetchers it now depends on.

**Step 2: Run the focused worker tests to verify they pass**

Run:

```bash
make test-adversary-sim-scrapling-worker
```

### Task 4: Prove Recent-Run, Snapshot, And Coverage Projection

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/operator_snapshot_non_human.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/observability/non_human_coverage.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/docs/adversarial-operator-guide.md`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/docs/testing.md`

**Step 1: Add failing projection tests if needed**

- Ensure the recent-run and snapshot tests prove browser-mode category projection and owned-surface coverage, not only the old request-native trio.

**Step 2: Run the focused projection tests to verify they fail**

Run:

```bash
make test-adversarial-coverage-receipts
```

Expected: failures until the browser-mode receipts and coverage expectations are threaded through.

**Step 3: Write the minimal projection and doc changes**

- Keep recent-run normalization truthful for mixed request-native and browser Scrapling runs.
- Update operator docs to stop describing `automated_browser` as outside Scrapling ownership.

**Step 4: Run the focused projection tests to verify they pass**

Run:

```bash
make test-adversarial-coverage-receipts
make test-adversary-sim-scrapling-coverage-receipts
```

### Task 5: Immediate Proxy Follow-On Inside `SIM-SCR-FULL-1B`

**Files:**
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/supervisor/scrapling_worker.py`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/adversary_sim_worker_plan.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/src/admin/adversary_sim_lane_runtime.rs`
- Modify: `/Users/jamestindall/Projects/Shuma-Gorath/scripts/tests/test_scrapling_worker.py`

**Step 1: Write the failing tests**

- Add a focused worker-plan and worker-execution contract for optional browser or request proxy settings.

**Step 2: Run the focused tests to verify they fail**

Run:

```bash
make test-adversary-sim-scrapling-worker
```

**Step 3: Implement the minimal proxy plumbing**

- Add explicit optional proxy inputs to the worker plan.
- Thread them into request-native and browser sessions.
- Keep proof truthful: support and receipt it locally, and avoid claiming distributed-origin effectiveness until a live proxy-backed proof ring exists.

**Step 4: Run the focused tests to verify they pass**

Run:

```bash
make test-adversary-sim-scrapling-worker
```

## Final Verification

Run:

```bash
make test-adversary-sim-scrapling-category-fit
make test-adversary-sim-scrapling-owned-surface-contract
make test-adversary-sim-scrapling-worker
make test-adversary-sim-scrapling-coverage-receipts
make test-adversarial-coverage-receipts
```

Expected: all focused Scrapling capability, worker, and coverage gates pass without relying on the legacy request-native-only framing.
