# Scrapling Deploy Skill Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make shared-host Scrapling deployment an agent-facing, receipt-backed workflow that infers safe defaults, wires runtime artifacts automatically, keeps the adversary seed minimal by default, and keeps Fermyon truthful as a gateway target rather than a first-class full adversary-sim runtime target.

**Architecture:** Introduce one shared Scrapling deploy-preflight helper and durable receipt under `.shuma/`, reuse the settled shared-host scope and seed contracts, integrate the shared-host deploy and remote-update flows so the required artifacts and `ADVERSARY_SIM_SCRAPLING_*` env values are carried automatically, and add one dedicated agent-facing skill that existing shared-host deploy skills depend on.

**Tech Stack:** Python helper scripts, shell deploy wrappers, Makefile targets, repo-local skills, JSON receipts, unittest-based deploy helper tests.

---

### Task 1: Planning Chain And Backlog Refresh

**Files:**
- Create: `docs/research/2026-03-21-scrapling-deploy-skill-integration-research.md`
- Create: `docs/plans/2026-03-21-scrapling-deploy-skill-integration-plan.md`
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`

**Step 1: Capture the architecture note**

Write the research note summarizing:

1. why catalog-first deploy guidance must not be reused for Scrapling runtime,
2. why shared-host can automate Scrapling fully today,
3. why default seeds must stay minimal and root-only,
4. and why full adversary-sim runtime support should stay shared-host-first for now.

**Step 2: Capture the implementation plan**

Write this implementation plan with the shared helper, shared-host integration, skill integration, and edge de-scope slices.

**Step 3: Update TODOs**

Split `SIM-SCR-8` into execution-ready sub-slices:

1. shared helper + receipt,
2. shared-host deploy integration,
3. skill integration,
4. edge runtime de-scope and truthful docs.

Add a blocked follow-on only if later edge external-supervisor productization needs to return.

**Step 4: Commit**

```bash
git add docs/research/2026-03-21-scrapling-deploy-skill-integration-research.md docs/plans/2026-03-21-scrapling-deploy-skill-integration-plan.md todos/todo.md todos/blocked-todo.md
git commit -m "docs: plan scrapling deploy skill integration"
```

### Task 2: Shared Scrapling Preflight Helper

**Files:**
- Create: `scripts/deploy/scrapling_deploy_prep.py`
- Create: `scripts/prepare_scrapling_deploy.py`
- Modify: `Makefile`
- Test: `scripts/tests/test_scrapling_deploy_prep.py`

**Step 1: Write the failing tests**

Add focused tests that prove the helper:

1. derives the allowed host and primary start URL from a public base URL,
2. writes a valid scope descriptor using the settled contract,
3. writes a valid minimal seed inventory using only the normalized public root URL by default,
4. writes a durable receipt with:
   - local artifact paths,
   - remote artifact paths,
   - `ADVERSARY_SIM_SCRAPLING_*` env mappings,
   - runtime mode,
   - egress guidance,
   - verification commands,
5. and does not pull in gateway-catalog paths as default seed material.

**Step 2: Run the tests to watch them fail**

Run:

```bash
make test-adversarial-python-unit
```

Expected:

1. the new helper tests fail because the module and CLI do not exist yet.

**Step 3: Implement the minimal helper**

Add the helper so it:

1. reuses `scripts/tests/shared_host_scope.py`,
2. reuses `scripts/tests/shared_host_seed_inventory.py`,
3. defaults to root-only seed generation with no deploy-time `robots.txt` fetch,
4. writes outputs under `.shuma/scrapling/` by default,
5. and emits a deterministic receipt the shared-host deploy adapters can consume.

Add a thin CLI entrypoint and a focused `make prepare-scrapling-deploy` target.

**Step 4: Run the focused tests**

Run:

```bash
make test-adversarial-python-unit
```

Expected:

1. the new helper tests pass,
2. existing deploy helper unit tests stay green.

**Step 5: Commit**

```bash
git add scripts/deploy/scrapling_deploy_prep.py scripts/prepare_scrapling_deploy.py scripts/tests/test_scrapling_deploy_prep.py Makefile
git commit -m "feat: add scrapling deploy preflight helper"
```

### Task 3: Shared-Host Deploy And Remote-Update Integration

**Files:**
- Modify: `scripts/deploy_linode_one_shot.sh`
- Modify: `scripts/deploy/remote_target.py`
- Test: `scripts/tests/test_deploy_linode_one_shot.py`
- Test: `scripts/tests/test_remote_target.py`

**Step 1: Write the failing tests**

Add tests that prove:

1. shared-host deploy invokes the Scrapling prep helper with the canonical public base URL,
2. shared-host deploy uploads scope and seed artifacts to the host,
3. shared-host deploy persists the remote `ADVERSARY_SIM_SCRAPLING_*` env values into the overlay env file,
4. the normalized remote receipt keeps enough Scrapling metadata for day-2 updates,
5. and `make remote-update` uploads the same scope and seed artifacts on later updates.

**Step 2: Run the tests to watch them fail**

Run:

```bash
make test-adversarial-python-unit
```

Expected:

1. the new shared-host deploy and remote-target assertions fail.

**Step 3: Implement the minimal wiring**

Update the shared-host deploy path so it:

1. calls the shared helper,
2. copies the scope and seed files to remote deterministic paths,
3. writes the required env overlay values,
4. extends the remote receipt with optional Scrapling metadata,
5. and teaches remote update to preserve that behavior.

Keep the old `GATEWAY_SURFACE_CATALOG_PATH` behavior for gateway preflight and smoke only.

**Step 4: Run the focused tests**

Run:

```bash
make test-adversarial-python-unit
```

Expected:

1. deploy helper tests pass,
2. remote-target tests pass,
3. shared helper tests stay green.

**Step 5: Commit**

```bash
git add scripts/deploy_linode_one_shot.sh scripts/deploy/remote_target.py scripts/tests/test_deploy_linode_one_shot.py scripts/tests/test_remote_target.py
git commit -m "feat: wire scrapling artifacts into shared-host deploy"
```

### Task 4: Agent Skill Integration

**Files:**
- Create: `skills/prepare-scrapling-for-deploy/SKILL.md`
- Modify: `skills/prepare-shared-host-on-linode/SKILL.md`
- Modify: `skills/deploy-shuma-on-linode/SKILL.md`
- Modify: `skills/prepare-shuma-on-akamai-fermyon/SKILL.md`
- Modify: `skills/deploy-shuma-on-akamai-fermyon/SKILL.md`

**Step 1: Write the failing tests**

For this task, the “tests” are pressure scenarios against the skill docs:

1. agent starting from shared-host deploy should learn that Scrapling artifacts are inferred and generated automatically,
2. the default seed must remain just the normalized public root URL,
3. agent starting from Fermyon deploy should learn that Fermyon remains a gateway target and not a first-class full Scrapling runtime target,
4. skill wording must make telemetry-the-map explicit and must not send the agent down a catalog-first runtime workflow.

**Step 2: Implement the skills**

Add one dedicated agent-facing skill and update the existing deploy skills so they:

1. depend on it for shared-host Scrapling activation,
2. keep the operator boundary irreducible,
3. explain the root-only default seed contract,
4. and stop treating catalog build as a Scrapling runtime prerequisite.

For Fermyon, update the skills to state the truthful boundary rather than extending the current runtime contract.

**Step 3: Commit**

```bash
git add skills/prepare-scrapling-for-deploy/SKILL.md skills/prepare-shared-host-on-linode/SKILL.md skills/deploy-shuma-on-linode/SKILL.md skills/prepare-shuma-on-akamai-fermyon/SKILL.md skills/deploy-shuma-on-akamai-fermyon/SKILL.md
git commit -m "docs: add scrapling deploy agent skill"
```

### Task 5: Truthful Edge De-Scope And Closeout Docs

**Files:**
- Modify: `docs/adversarial-operator-guide.md`
- Modify: `docs/testing.md`
- Modify: `docs/api.md`
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `todos/completed-todo-history.md`
- Create: `docs/research/2026-03-21-scrapling-deploy-skill-integration-post-implementation-review.md`

**Step 1: Update the docs**

Document that:

1. shared-host deploy can make Scrapling operational end to end,
2. the default deploy-time adversary seed is just the public root URL,
3. deploy-time gateway catalogs remain gateway evidence only,
4. Fermyon remains a gateway/edge path but not the current primary full adversary-sim runtime target.

**Step 2: Run focused verification**

Run:

```bash
make test-adversarial-python-unit
git diff --check
```

Expected:

1. deploy-helper tests pass,
2. no diff hygiene issues remain.

**Step 3: Review and archive**

1. Write the post-implementation review.
2. Move the completed TODO slice into `todos/completed-todo-history.md`.
3. If the review finds shortfalls, add immediate follow-up TODOs before claiming completion.

**Step 4: Commit**

```bash
git add docs/adversarial-operator-guide.md docs/testing.md docs/api.md docs/research/2026-03-21-scrapling-deploy-skill-integration-post-implementation-review.md todos/todo.md todos/blocked-todo.md todos/completed-todo-history.md
git commit -m "docs: close scrapling deploy skill integration"
```
