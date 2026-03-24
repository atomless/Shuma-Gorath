# Path To Closed-Loop LLM Adversary And Diagnosis Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn the current live recommend-only Scrapling feedback loop into a genuinely closed config loop, then use that proven loop as the foundation for a later LLM-backed diagnosis/config harness and only after that a benchmark-driven code-evolution harness.

**Architecture:** Keep the current machine-first snapshot, benchmark, replay-promotion, and reconcile contracts as the core control plane. Add the missing category, classifier, and adversary-lane representativeness gates first, close the loop at config tuning before code changes, and keep the LLM adversary lane and LLM diagnosis lane as separate bounded roles. Use one capability-safe container boundary and a pluggable model-backend contract for bounded LLM adversary modes, with frontier-backed execution as the initial reference path for the highest-capability categories and optional later local-model backends only if evals prove parity.

LLM attacker black-box note:

1. the later LLM attacker must start from the host site's root entrypoint and category objective only,
2. it must know nothing about Shuma internals, routes, or defenses,
3. and it should be primed to fulfill the remaining non-human categories and to behave maliciously where those categories imply malicious behavior.

**Tech Stack:** Rust control plane, Python adversarial runner/orchestration, Scrapling worker, capability-safe containerized adversary actor runtime, external frontier LLM APIs, optional later local model backends, Makefile verification.

---

### Task 1: `TRAFFIC-TAX-1`

**Files:**
- Modify: `docs/plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`
- Modify: `todos/todo.md`
- Later code targets: `src/observability/`, `src/runtime/`, `src/admin/`, dashboard objective/tuning surfaces

**Step 1: Define the canonical non-human taxonomy contract**

Capture:

1. stable machine ids,
2. stable human-facing labels and descriptions,
3. unknown/mixed/insufficient-evidence states,
4. category-to-operator-posture intent compatibility.

**Step 2: Add the backend-contract ownership note**

State that the taxonomy is:

1. machine-first,
2. operator-facing,
3. and the only taxonomy later objective and tuning surfaces should project.

**Step 3: Verification**

Run: `git diff --check`
Expected: PASS

**Step 4: Commit**

```bash
git add docs/plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md todos/todo.md
git commit -m "docs: define canonical non-human taxonomy contract"
```

### Task 2: `TRAFFIC-TAX-2`

**Files:**
- Modify: `docs/plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`
- Modify: `docs/plans/2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md`
- Later code targets: `src/runtime/traffic_classification.rs`, `src/observability/`, `src/admin/oversight_*`, snapshot/benchmark contracts

**Step 1: Define the classifier and abuse-score contract**

Capture:

1. signal and fingerprint evidence,
2. category assignment,
3. confidence or exactness,
4. stale and degraded states,
5. cumulative abuse score semantics,
6. failure-closed tuning behavior.

**Step 2: Keep the adaptive layer constrained**

State explicitly that:

1. categorization quality should evolve first,
2. taxonomy breadth should not churn as part of the first loop,
3. and later severity decisions remain bounded by category posture.

**Step 3: Verification**

Run: `git diff --check`
Expected: PASS

**Step 4: Commit**

```bash
git add docs/plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md docs/plans/2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md
git commit -m "docs: define traffic classification and abuse-score contract"
```

### Task 3: `SIM-LLM-FIT-1`

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Later code targets: bounded LLM adversary runtime modules, container orchestration helpers, adversarial runner helpers

**Step 1: Add the bounded LLM category-fulfillment tranche**

Define a new active tranche for:

1. minimum LLM-backed browser or request modes,
2. category-fulfillment scope only,
3. pluggable model-backend contract,
4. frontier-backed reference execution for high-capability categories,
5. optional later local-model backend only if evals prove parity.

**Step 2: Keep `SIM-LLM-1` blocked as the larger later actor**

Reframe the existing blocked item as:

1. the full first-class runtime actor,
2. later than the bounded fulfillment tranche,
3. and not required before the first closed config loop.

**Step 3: Verification**

Run: `git diff --check`
Expected: PASS

**Step 4: Commit**

```bash
git add todos/todo.md todos/blocked-todo.md docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md
git commit -m "docs: add bounded llm adversary fulfillment tranche"
```

### Task 4: `SIM-FULFILL-1` And `SIM-COVER-1`

**Files:**
- Modify: `docs/plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`
- Modify: `todos/todo.md`
- Later code targets: Scrapling lane controls, bounded LLM adversary modes, coverage receipts, replay-promotion visibility

**Step 1: Freeze the category-to-lane matrix**

For each category, define:

1. Scrapling mode if sufficient,
2. bounded LLM browser mode if needed,
3. bounded LLM request mode if needed,
4. unresolved gap if neither exists yet.

**Step 2: Freeze the coverage proof contract**

Require:

1. receipts by category,
2. partial and stale coverage states,
3. replay-promoted LLM lineage treatment,
4. no lane-local labels as tuning truth.

**Step 3: Verification**

Run: `git diff --check`
Expected: PASS

**Step 4: Commit**

```bash
git add docs/plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md todos/todo.md
git commit -m "docs: freeze llm and scrapling fulfillment and coverage contract"
```

### Task 5: `SIM-PROTECTED-1`, `OPS-OBJECTIVES-3`, And `OPS-BENCH-3`

**Files:**
- Modify: `docs/plans/2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md`
- Modify: `docs/plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`

**Step 1: Freeze evidence admission**

Define:

1. synthetic exclusion,
2. replay-promoted LLM lineage as protected,
3. raw LLM discoveries as advisory,
4. category-aware objective and benchmark blockers.

**Step 2: Freeze the operator-facing posture model**

Keep:

1. `allowed`,
2. `tolerated`,
3. `cost_reduced`,
4. `restricted`,
5. `blocked`,

as the controller and later tuning-surface posture scale.

**Step 3: Verification**

Run: `git diff --check`
Expected: PASS

**Step 4: Commit**

```bash
git add docs/plans/2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md docs/plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md todos/todo.md todos/blocked-todo.md
git commit -m "docs: freeze protected llm evidence and category posture gates"
```

### Task 6: `OVR-APPLY-1`

**Files:**
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `todos/blocked-todo.md`
- Later code targets: `src/admin/oversight_*`, config validation/apply seams, benchmark/watch-window contracts

**Step 1: Freeze the first closed config loop contract**

Require:

1. one bounded config family at a time,
2. protected-evidence prerequisite checks,
3. watch-window comparison,
4. rollback,
5. durable decision lineage.

**Step 2: Keep code evolution out**

State explicitly that:

1. this phase closes the config loop only,
2. and code change loops remain blocked until after live proof.

**Step 3: Verification**

Run: `git diff --check`
Expected: PASS

**Step 4: Commit**

```bash
git add docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md todos/blocked-todo.md
git commit -m "docs: freeze first closed config loop boundary"
```

### Task 7: `OVR-AGENT-2`

**Files:**
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Later code targets: later hosted or scheduled diagnosis agent modules and contracts

**Step 1: Reframe `OVR-AGENT-2` as the later LLM diagnosis/config harness**

Define it as:

1. the later LLM-backed diagnosis/config system,
2. over proven machine-first contracts and protected evidence,
3. after the first closed config loop.

**Step 2: Verification**

Run: `git diff --check`
Expected: PASS

**Step 3: Commit**

```bash
git add todos/blocked-todo.md docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md
git commit -m "docs: reframe later llm diagnosis harness"
```

### Task 8: `OVR-CODE-1`

**Files:**
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Later code targets: later reviewed code-change planning and PR-generation surfaces

**Step 1: Keep the code loop explicitly separate**

Define it as:

1. later benchmark-driven code evolution,
2. explicitly after the closed config loop and later diagnosis harness,
3. never conflated with config tuning.

**Step 2: Verification**

Run: `git diff --check`
Expected: PASS

**Step 3: Commit**

```bash
git add todos/blocked-todo.md docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md
git commit -m "docs: keep code evolution behind closed config loop"
```

### Task 9: Roadmap And Backlog Sync

**Files:**
- Modify: `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
- Modify: `docs/research/README.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Sync the canonical planning chain**

Update the roadmap and research index to say:

1. current state is a live recommend-only Scrapling loop,
2. next closed-loop path runs through taxonomy, classifier, bounded LLM fulfillment, coverage, protected evidence, and closed config tuning,
3. later LLM diagnosis and code loops remain downstream.

**Step 2: Record the planning tranche**

Add a completion-history note with the research note, plan doc, roadmap/backlog sync, and `git diff --check`.

**Step 3: Verification**

Run: `git diff --check`
Expected: PASS

**Step 4: Commit**

```bash
git add docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md docs/research/README.md todos/completed-todo-history.md
git commit -m "docs: capture path to closed llm adversary and diagnosis loops"
```
