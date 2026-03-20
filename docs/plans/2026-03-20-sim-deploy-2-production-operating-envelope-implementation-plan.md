# SIM-DEPLOY-2 Production Operating Envelope Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Harden Shuma's production adversary-sim operating envelope so the runtime lifecycle is single-authority, the focused lifecycle gate is truthful, and operators have explicit kill-switch, posture, and no-impact proof for production use.

**Architecture:** Keep the existing backend-owned heartbeat and control-plane architecture, but remove the remaining desired-state split, prove the lifecycle gate against the real stale-state cases, and then layer explicit production posture, kill-switch, and no-impact verification on top of that settled lifecycle contract.

**Tech Stack:** Rust admin/runtime lifecycle code, existing Makefile verification targets, Python runtime-surface harnesses, compact monitoring and hot-read summaries, and operator/deployment documentation.

---

## Phase 1: Verification Truthfulness

### Task 1: Refresh the lifecycle `make` gate so it runs the real stale-state cases

**Files:**

- Modify:
  - `Makefile`
- Test:
  - `src/admin/api.rs`
  - `scripts/tests/test_adversary_sim_supervisor.py`

**Step 1: Reproduce the current mismatch**

Run:

```bash
make test-adversary-sim-lifecycle
```

Expected:

1. the target passes,
2. but at least two `cargo test <selector>` invocations run zero tests because the selectors no longer match the renamed stale-state tests.

**Step 2: Point the target at the current read-only status contract tests**

Update the Rust selectors so the target executes the current stale-running and prior-process ownership tests rather than the historical names.

**Step 3: Re-run the focused lifecycle gate**

Run:

```bash
make test-adversary-sim-lifecycle
```

Expected:

1. the stale-running reconciliation case executes,
2. the previous-process ownership case executes,
3. the internal beat diagnostics case executes,
4. and the Python supervisor checks still pass.

**Step 4: Commit**

```bash
git add Makefile
git commit -m "test: make adversary sim lifecycle gate truthful"
git push -u origin codex/sim-deploy-2
```

## Phase 2: Desired-State Unification

### Task 2: Collapse adversary-sim desired state to one backend source of truth

**Files:**

- Modify:
  - `src/admin/api.rs`
  - `src/admin/adversary_sim.rs`
  - `src/config/mod.rs`
  - `docs/api.md`
  - `docs/testing.md`
  - `docs/adversarial-operator-guide.md`
- Test:
  - `src/admin/api.rs`
  - `src/config/tests.rs`

**Step 1: Write a failing lifecycle test for dual-authority removal**

Add or update focused Rust coverage to prove:

1. the control path can enable and disable adversary sim without any runtime override helper,
2. status reports the same desired state that control writes,
3. and restart or expiry behavior does not depend on a second ephemeral enabled flag.

**Step 2: Run the focused failing test**

Run the smallest relevant `cargo test <selector>` command through the existing `make` surface if a focused target exists; otherwise use the focused Rust selector while keeping the later tranche verification on `make`.

Expected:

1. failure due to the old runtime-override path still being required,
2. or failure because the status/control contract still reads from two desired-state sources.

**Step 3: Remove the runtime override path from adversary-sim lifecycle ownership**

Implement the single-authority model. Keep:

1. env and seeded config as initial posture only,
2. `ControlState` as lifecycle ownership,
3. and `GET /admin/config` runtime overlays as a projection of that one backend authority rather than a separate writer path.

**Step 4: Re-run focused lifecycle tests**

Run:

```bash
make test-adversary-sim-lifecycle
```

Expected:

1. control, status, and beat all agree on desired state,
2. stale-state diagnostics still surface via `controller_reconciliation_required`,
3. and no hidden write-on-read behavior returns.

**Step 5: Commit**

```bash
git add src/admin/api.rs src/admin/adversary_sim.rs src/config/mod.rs docs/api.md docs/testing.md docs/adversarial-operator-guide.md
git commit -m "refactor: unify adversary sim desired state"
git push
```

## Phase 3: Production Posture Contract

### Task 3: Define and surface the production-default runtime lane and resource posture

**Files:**

- Modify:
  - `src/admin/adversary_sim.rs`
  - `docs/adversarial-operator-guide.md`
  - `docs/configuration.md`
  - `docs/deployment.md`
  - `todos/todo.md`
- Test:
  - `src/admin/adversary_sim.rs`

**Step 1: Write a failing status-payload test for explicit production posture reporting**

Add focused coverage for the exact posture Shuma wants operators to treat as default in production:

1. surface available by default,
2. generation off until explicit enable,
3. hard resource and queue posture bounded,
4. deployment-profile-specific heartbeat posture visible.

**Step 2: Run the focused failing test**

Expected:

1. missing or incomplete posture fields,
2. or docs/status not yet explicit enough for operator use.

**Step 3: Implement the explicit operating-envelope surface**

Reuse the existing status payload and guardrail structures. Do not invent a second posture model. Extend the current payload and docs so production posture is explicit and stable.

**Step 4: Re-run lifecycle and posture checks**

Run:

```bash
make test-adversary-sim-lifecycle
```

Expected:

1. lifecycle gate still passes,
2. posture coverage is now explicit and bounded.

**Step 5: Commit**

```bash
git add src/admin/adversary_sim.rs docs/adversarial-operator-guide.md docs/configuration.md docs/deployment.md todos/todo.md
git commit -m "docs: codify production adversary sim posture"
git push
```

## Phase 4: Kill Switch And No-Impact Proof

### Task 4: Add explicit production kill-switch proof and no-impact verification

**Files:**

- Modify:
  - `Makefile`
  - `scripts/tests/adversary_runtime_toggle_surface_gate.py`
  - `docs/testing.md`
  - `docs/adversarial-operator-guide.md`
- Test:
  - `scripts/tests/test_adversary_runtime_toggle_surface_gate.py`

**Step 1: Write a failing test for the no-impact extension**

Add unit coverage proving the runtime-surface gate can separately:

1. validate simulation-category coverage,
2. and validate that live-only or likely-human summaries remain clean while adversary-sim runs.

**Step 2: Run the focused failing Python test**

Run:

```bash
python3 -m unittest scripts/tests/test_adversary_runtime_toggle_surface_gate.py
```

Expected:

1. failure because the harness only proves simulation-tagged category coverage today.

**Step 3: Extend the runtime-surface gate and Makefile**

Implement the smallest truthful verification extension that:

1. keeps the current deterministic surface proof,
2. adds no-impact assertions against live-only summary paths,
3. and documents the kill-switch path operators should use in production.

If a new focused `make` target is needed, name it truthfully.

**Step 4: Run the smallest relevant verification**

Run:

```bash
python3 -m unittest scripts/tests/test_adversary_runtime_toggle_surface_gate.py
make test-adversary-sim-lifecycle
```

If a live server is running, also run:

```bash
make test-adversary-sim-runtime-surface
```

**Step 5: Commit**

```bash
git add Makefile scripts/tests/adversary_runtime_toggle_surface_gate.py scripts/tests/test_adversary_runtime_toggle_surface_gate.py docs/testing.md docs/adversarial-operator-guide.md
git commit -m "test: prove adversary sim kill switch and no-impact contract"
git push
```

## Phase 5: Deployment And Evidence Closure

### Task 5: Treat production adversary-sim as a first-class operating path in docs and receipts

**Files:**

- Modify:
  - `docs/deployment.md`
  - `docs/adversarial-operator-guide.md`
  - `docs/api.md`
  - `docs/configuration.md`
  - `todos/todo.md`
  - `todos/completed-todo-history.md`

**Step 1: Update operator and deployment guidance**

Document:

1. production-default availability,
2. explicit operator enablement semantics,
3. kill-switch and rollback usage,
4. no-impact evidence expectations,
5. and the fact that production adversary-sim is no longer a gated exception.

**Step 2: Update TODO history and remaining backlog**

Move completed `SIM-DEPLOY-2-*` checklist items into `todos/completed-todo-history.md` immediately after each slice lands, and add any shortfall follow-ons discovered during post-implementation review.

**Step 3: Run final tranche verification**

Run the smallest truthful paths first, then the full canonical suite if state has changed enough to require it under the repo receipt rules.

At minimum:

```bash
make test-adversary-sim-lifecycle
```

If a live server is available:

```bash
make test-adversary-sim-runtime-surface
```

**Step 4: Commit**

```bash
git add docs/deployment.md docs/adversarial-operator-guide.md docs/api.md docs/configuration.md todos/todo.md todos/completed-todo-history.md
git commit -m "docs: close sim deploy production operating envelope"
git push
```

## Review Loop Expectations

After every tranche:

1. compare implementation against this plan and the readiness review,
2. write any shortfall immediately into `todos/todo.md` or `todos/blocked-todo.md`,
3. execute the shortfall task before moving to the next planned tranche when it is execution-ready,
4. and move completed checklist items into `todos/completed-todo-history.md` with the completion date and evidence.

## Verification Expectations

Definition of done for the full `SIM-DEPLOY-2` tranche requires:

1. truthful focused lifecycle verification,
2. one backend desired-state authority,
3. explicit production posture and kill-switch guidance,
4. no-impact proof that live operator summaries remain separate from adversary-sim evidence while simulation runs,
5. and docs that treat production adversary-sim as a normal operating path.
