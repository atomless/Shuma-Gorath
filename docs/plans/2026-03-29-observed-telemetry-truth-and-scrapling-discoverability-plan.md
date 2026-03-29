# Observed Telemetry Truth And Scrapling Discoverability Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an explicit release-blocking ban on presentation-side telemetry fabrication and remove Scrapling’s out-of-band route choreography so the attacker operates from discoverable public host knowledge only.

**Architecture:** Preserve the existing Game Loop, judge, and recent-run observer mechanics while tightening the repo contract in two places. First, freeze the no-fabrication rule in policy and docs. Second, simplify the Scrapling worker plan by removing `runtime_paths`, then drive bulk-scraper, browser, and HTTP-agent behavior from root URL, optional `robots.txt` hints, traversal-discovered links, and forms or redirects observed during the run. Preserve simulation-tag attribution, but do not let it become defense truth.

**Tech Stack:** AGENTS policy docs, Rust adversary-sim worker-plan and beat payload contracts, Python Scrapling worker, dashboard rendered-proof fixtures, focused Rust/Python/dashboard tests, docs and TODO bookkeeping.

---

## Acceptance Criteria

1. `AGENTS.md` contains a non-negotiable directive that presentation layers, adapters, summaries, and docs must never invent, backfill, merge, or stitch execution claims beyond what authoritative telemetry actually recorded.
2. `ScraplingWorkerPlan` no longer contains `runtime_paths`, and internal beat payload tests prove the field is absent.
3. Scrapling request emission no longer includes persona/category strings such as `scrapling-bulk-scraper`, `scrapling-http-agent`, or similar convenience markers in public request paths or bodies.
4. Scrapling personas no longer call synthetic convenience routes like `/agent/ping`, `/agent/submit`, `/agent/update`, or `/agent/redirect`.
5. Scrapling reaches any challenge, abuse, browser, or maze surfaces only from discoverable public content observed during the run: root URL, optional `robots.txt`/sitemap hints, traversal-visible links, or forms and redirects encountered in those pages.
6. Focused proof exists through:
   - `make test-adversary-sim-scrapling-worker`
   - `make test-admin-machine-contracts`
   - `make test-dashboard-game-loop-accountability`
   - `make test-adversarial-llm-fit`

## Task 1: Freeze the new truth and discoverability contract in failing tests

**Files:**
- Modify: `scripts/tests/test_scrapling_worker.py`
- Modify: `src/admin/api.rs`
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`
- Modify: `scripts/tests/test_llm_fulfillment.py`

**Step 1: Write the failing tests**

Add focused assertions proving:

1. Scrapling worker plans can execute without `runtime_paths`.
2. Admin beat payloads no longer expose `runtime_paths`.
3. Bulk-scraper and HTTP-agent requests do not contain `scrapling-*` persona query values.
4. HTTP-agent persona no longer uses `/agent/*` helper routes.
5. Dashboard rendered proof no longer shows the old `scrapling-*` sample request path in `Defences In This Round`.
6. The bounded LLM contract still exposes only host-root/public-hint discoverability inputs.

**Step 2: Run the focused tests to verify RED**

Run:

1. `make test-adversary-sim-scrapling-worker`
2. `make test-admin-machine-contracts`
3. `make test-dashboard-game-loop-accountability`
4. `make test-adversarial-llm-fit`

Expected: the new assertions fail because the current worker still depends on `runtime_paths`, emits persona-coded public paths, and the beat payload still serializes the out-of-band route map.

## Task 2: Remove out-of-band Scrapling route knowledge

**Files:**
- Modify: `src/admin/adversary_sim_worker_plan.rs`
- Modify: `src/admin/adversary_sim.rs`
- Modify: `src/admin/adversary_sim_lane_runtime.rs`
- Modify: `scripts/supervisor/scrapling_worker.py`

**Step 1: Delete `runtime_paths` from the worker-plan contract**

Remove the Rust plan struct and plan builder wiring that hand direct route paths to the worker.

**Step 2: Replace route-map dependence with discoverable inputs**

Update the worker so it uses:

1. accepted start URLs from the seed inventory,
2. optional bounded `robots.txt` / sitemap hints when present,
3. traversal-visible links,
4. and forms or redirects observed during the run.

**Step 3: Remove convenience request content**

Delete persona-coded query strings and helper-route traffic. The worker may still choose attacker-faithful invalid submissions, but only after discovering the relevant surface from public content.

**Step 4: Keep surface receipts fail-closed**

When a surface is not discovered in a run, leave the receipt absent rather than recreating it from plan intent.

**Step 5: Verify focused tests**

Run:

1. `make test-adversary-sim-scrapling-worker`
2. `make test-admin-machine-contracts`

Expected: the worker contract is now route-map-free and the emitted request evidence is attacker-faithful.

## Task 3: Prove the rendered observer surface stays truthful

**Files:**
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`
- Modify: `docs/dashboard-tabs/game-loop.md`

**Step 1: Update the Game Loop fixture evidence**

Replace the old category-coded sample path fixture with a discoverable public path sample.

**Step 2: Add rendered proof**

Assert that the Game Loop defence-cast sample request path does not contain the old convenience markers and still renders the surface summary correctly.

**Step 3: Verify focused dashboard proof**

Run: `make test-dashboard-game-loop-accountability`

Expected: the UI now shows truthful surface receipts without exposing simulator convenience strings.

## Task 4: Harden docs and close the tranche

**Files:**
- Modify: `AGENTS.md`
- Modify: `docs/adversarial-operator-guide.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Add the non-negotiable AGENTS directive**

Make the no-fabrication rule explicit and release-blocking.

**Step 2: Update the operator guide**

Document that Scrapling now operates from discoverable public host inputs only and that simulation tags remain attribution-only rather than defense truth.

**Step 3: Update indexes and TODO history**

Add the new research and plan docs to the indexes and record the delivered tranche with exact proof commands.

**Step 4: Final focused verification**

Run:

1. `make test-adversary-sim-scrapling-worker`
2. `make test-admin-machine-contracts`
3. `make test-dashboard-game-loop-accountability`
4. `make test-adversarial-llm-fit`

Expected: the repo policy, worker contract, and rendered observer proof all align with the new truth boundary.
