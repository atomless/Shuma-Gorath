# SIM-REALISM-1 Lane Traffic Realism And Cadence Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make Scrapling and Agentic Traffic emit representative crawl or scrape pressure for the attacker categories they claim to fulfill, without widening their knowledge boundary or leaking simulator truth into Shuma's defences.

**Architecture:** Reuse the existing adversary-sim heartbeat, lane-plan, supervisor, and worker seams. Add one explicit realism profile contract per lane or mode, then route that contract through the current Rust plan generation and Python execution paths. Preserve the current public-discovery-only Scrapling model and the current black-box Agentic model. Prove realism from runtime receipts and rendered observer truth, not from declared labels.

**Tech Stack:** Rust adversary-sim runtime and internal API, Rust supervisor transport, Python Scrapling worker, Python LLM runtime worker and adversarial container runner, existing operator-snapshot and oversight observer surfaces, Makefile verification, backlog and docs closeout.

**Status note:** `SIM-REALISM-1A`, `SIM-REALISM-1B`, and `SIM-REALISM-1C` are now landed. The later field-grounded follow-on plan is [`2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](2026-03-30-adversary-lane-wild-traffic-gap-plan.md), which keeps representative mixed-attacker claims blocked until the second realism chain lands as well.

---

## Guardrails

1. Do not make lanes "realistic" by feeding them internal route catalogs, simulator-only host hints, or Shuma-specific knowledge.
2. Do not collapse all adversary traffic into one harsher default cadence; realism must stay profile-specific.
3. Do not let simulator labels or profile ids enter defence truth, classification truth, or tuning truth.
4. Do not claim mixed-attacker or tuning representativeness from mere lane presence; completion requires runtime proof of emitted behavior.

## Task 1: Refresh The Backlog And Sequencing Truth

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`

**Work:**
1. Add the `SIM-REALISM-1A..1D` sequence as the next adversary-sim realism chain.
2. Make the backlog explicit that mixed-attacker proof and later tuning-quality claims are not yet representative while this chain is open.
3. Thread the new realism dependency into the mature adversary-sim roadmap and active sequencing notes.

**Acceptance criteria:**
1. The active backlog names the realism chain explicitly instead of treating cadence realism as implicit.
2. The sequencing docs no longer imply that `SIM-LLM-1C3` plus lane visibility alone are enough for representative mixed-attacker proof.

## Task 2: `SIM-REALISM-1A` Freeze The Executable Realism Profile Contract

**Files:**
- Later code targets: `src/admin/adversary_sim_worker_plan.rs`, `src/admin/adversary_sim_lane_runtime.rs`, `src/admin/adversary_sim_llm_lane.rs`
- Later contract or fixture targets: `scripts/tests/adversarial/`, focused Rust and Python contract tests
- Modify when implementation starts: `Makefile`, `docs/adversarial-operator-guide.md`, `docs/testing.md`

**Work:**
1. Define a versioned profile contract that captures, per lane or mode:
   - request or action budget,
   - burst size,
   - intra-burst jitter,
   - dwell or pause windows,
   - identity rotation cadence,
   - JavaScript or browser propensity,
   - retry ceilings,
   - and runtime receipt fields that prove actual emitted behavior.
2. Keep the profile contract separate from category labels so the lane can stay truthful even if category ownership evolves later.
3. Wire the contract so both Rust plan generation and Python execution can consume one canonical shape.

**Acceptance criteria:**
1. There is one canonical realism contract shared across the lane planner and the lane worker paths.
2. The contract proves how emitted behavior will be measured, not only how it is requested.
3. No contract field widens host knowledge beyond root URL, allowed scope, public hint documents, and response-derived discovery.

**Proof:**
1. Add and pass `make test-adversarial-lane-realism-contract`.
2. Keep `make test-adversarial-lane-contract` green.

## Task 3: `SIM-REALISM-1B` Implement Scrapling Profile-Driven Pacing And Identity Behavior

**Files:**
- Later code targets: `src/admin/adversary_sim.rs`, `src/admin/adversary_sim_lane_runtime.rs`, `src/admin/adversary_sim_worker_plan.rs`, `scripts/supervisor/scrapling_worker.py`
- Later proof targets: focused Rust tests, focused Scrapling worker tests, `Makefile`
- Modify when implementation starts: `docs/adversarial-operator-guide.md`, `docs/testing.md`

**Work:**
1. Replace the current flat per-tick pacing with per-mode profiles for:
   - `crawler`,
   - `bulk_scraper`,
   - `browser_automation`,
   - `stealth_browser`,
   - and `http_agent`.
2. Make those profiles drive:
   - request or navigation counts,
   - intra-request jitter,
   - pause or dwell windows,
   - session continuity,
   - and identity rotation when proxy support exists.
3. Preserve the current public-discovery-only boundary and do not reintroduce route choreography.
4. Emit runtime receipts proving the actual pacing and session shape used.

**Acceptance criteria:**
1. Different Scrapling personas materially differ in emitted request or session cadence rather than sharing one generic burst shape.
2. Runtime receipts prove actual request counts, pauses or dwell, and identity or session grouping.
3. Scrapling still starts only from allowed public seed knowledge and response-derived traversal.

**Proof:**
1. Add and pass `make test-adversary-sim-scrapling-realism`.
2. Keep `make test-adversary-sim-scrapling-worker` green.
3. Keep `make test-adversary-sim-scrapling-category-fit` green.
4. Keep `make test-adversary-sim-scrapling-coverage-receipts` green.

## Task 4: `SIM-REALISM-1C` Implement Agentic Request-Mode Pacing And Focused Burst Structure

**Files:**
- Later code targets: `src/admin/adversary_sim_llm_lane.rs`, `scripts/supervisor/llm_runtime_worker.py`, `scripts/tests/adversarial_runner/llm_fulfillment.py`, `scripts/tests/adversarial_container/worker.py`
- Later proof targets: focused LLM runtime tests, `Makefile`
- Modify when implementation starts: `docs/adversarial-operator-guide.md`, `docs/testing.md`

**Work:**
1. Replace the current request-mode one-shape execution with profile-driven:
   - micro-burst ceilings,
   - pause windows,
   - focused-page-set behavior,
   - and response-aware continuation rules.
2. Preserve the black-box root-only and public-hint-only knowledge boundary.
3. Record runtime receipts that make the actual burst or pause structure observable.
4. Keep degraded fallback explicit when provider-backed generation is unavailable, but make the degraded path still respect the realism contract.

**Acceptance criteria:**
1. Request-mode no longer degrades into an unrealistically uniform sequential action stream.
2. The runtime can prove focused micro-burst behavior and pause windows from receipts.
3. The worker still remains same-origin, bounded, and Shuma-blind.

**Proof:**
1. Add and pass `make test-adversarial-llm-realism`.
2. Keep `make test-adversarial-llm-fit` green.
3. Keep `make test-adversarial-llm-runtime-dispatch` green.
4. Keep `make test-adversarial-llm-runtime-projection` green.

## Task 5: `SIM-REALISM-1D` Replace `browser_mode_not_supported` With A Real Agentic Browser Session Lane

Status: Landed on 2026-03-30.

**Files:**
- Later code targets: `src/admin/adversary_sim_llm_lane.rs`, `scripts/supervisor/llm_runtime_worker.py`, `scripts/tests/adversarial_runner/llm_fulfillment.py`, browser execution path under `scripts/tests/adversarial_container/`
- Later proof targets: focused browser-mode runtime tests, `Makefile`, observer rendering proof if surfaced materially changes
- Modify when implementation starts: `docs/adversarial-operator-guide.md`, `docs/testing.md`, relevant dashboard docs if operator-facing truth changes

**Work:**
1. Implement real bounded browser-mode execution instead of the current unsupported receipt.
2. Make browser-mode behave like a focused delegated session:
   - stable session identity,
   - small action count,
   - meaningful dwell,
   - narrow goal-driven traversal,
   - and no broad crawler-style traversal.
3. Project browser-mode runtime receipts into recent-run and observer surfaces truthfully.

**Acceptance criteria:**
1. Agentic browser mode emits real black-box session traffic.
2. Runtime receipts prove dwell, top-level action count, and stable session shape.
3. Mixed-attacker surfaces can distinguish real browser-mode execution from request-mode-only runs.

**Proof:**
1. Add and pass `make test-adversarial-llm-browser-runtime`.
2. Keep `make test-adversarial-llm-runtime-dispatch` green.
3. Keep `make test-adversarial-llm-runtime-projection` green.
4. Keep `make test-admin-machine-contracts` green if read models change.
5. Keep `make test-dashboard-game-loop-accountability` green if observer surfaces change.

## Recommended Implementation Order

1. backlog and sequencing truth refresh
2. `SIM-REALISM-1A`
3. `SIM-REALISM-1B`
4. `SIM-REALISM-1C`
5. `SIM-REALISM-1D`
6. only then reopen representative mixed-attacker proof claims or later tuning-quality claims that depend on those lanes
