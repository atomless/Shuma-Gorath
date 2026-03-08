# Durable Operator State And Clean/Reset Semantics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Separate durable operator state from disposable build/runtime scratch space so `make clean` is truthful and remote day-2 workflows survive local cleanup.

**Architecture:** Treat `.spin/` as ephemeral local runtime/test scratch only, and move durable operator artifacts into a separate gitignored project-local state directory. Narrow `make clean` to reproducible build/test artifacts, add an explicit destructive local-state reset target for `.spin`, and update setup/deploy/remote flows to use the durable state directory for remote receipts, provider setup receipts, and default surface-catalog outputs.

**Tech Stack:** Makefile orchestration, existing Python deploy helpers, gitignored local state, Linode setup/deploy scripts, dashboard/operator docs.

---

## Context

The current contract is inconsistent:

1. `.env.local` persists `SHUMA_ACTIVE_REMOTE=<name>` as durable local operator state.
2. The normalized remote receipt lives under `.spin/remotes/<name>.json`.
3. `make clean` currently removes the entire `.spin/` tree.
4. The setup flow also defaults the provider setup receipt and generated surface catalog into `.spin/`.

That means a normal local cleanup can silently invalidate later `make remote-*` day-2 operations while leaving `.env.local` pointing at a missing receipt.

## Decisions

1. `make clean` must clean only reproducible build/test artifacts.
2. Destructive local runtime/test state cleanup gets its own explicit target.
3. Durable operator state must not live under `.spin/`.
4. The durable operator state directory should be gitignored and repo-local.
5. Move together:
   - normalized remote receipts,
   - provider setup receipts,
   - default generated site surface catalogs used by deploy/update.
6. Keep `.spin/` for ephemeral runtime/test artifacts such as local SQLite KV, logs, lock files, deploy preflight reports, and verification receipts.
7. Do not add backward-compatibility shims unless they are required to complete the live proof cleanly.

## Task 1: Define the durable operator-state location and cleanup contract

**Files:**
- Modify: `Makefile`
- Modify: `.gitignore`
- Modify: `AGENTS.md`
- Modify: `docs/quick-reference.md`
- Modify: `docs/deployment.md`
- Modify: `README.md`

Steps:

1. Introduce a single canonical durable local state dir variable for operator artifacts.
2. Move `LINODE_SETUP_RECEIPT` and `REMOTE_RECEIPTS_DIR` defaults to that directory.
3. Narrow `make clean` so it no longer deletes `.spin/`.
4. Add a truthful destructive local-state reset target for `.spin/`.
5. Update docs/help text so operators understand the difference between build cleanup and local-state reset.

## Task 2: Move setup/deploy defaults for durable artifacts

**Files:**
- Modify: `scripts/deploy/linode_shared_host_setup.py`
- Modify: `scripts/deploy/remote_target.py`
- Modify: `scripts/deploy_linode_one_shot.sh`
- Modify: `scripts/prepare_linode_shared_host.py`
- Modify: `scripts/build_site_surface_catalog.py` (only if default/help text needs alignment)

Steps:

1. Move default remote-receipt output from `.spin/remotes/` to the new durable dir.
2. Move default Linode setup receipt path to the new durable dir.
3. Move default generated surface-catalog output from `.spin/` to the new durable dir.
4. Keep deploy preflight/smoke temp files and runtime scratch under `.spin/` or `/tmp` as appropriate.
5. Ensure provider setup/deploy still auto-activate the emitted remote after the path move.

## Task 3: Add failing tests for lifecycle truth

**Files:**
- Modify: `scripts/tests/test_prepare_linode_shared_host.py`
- Modify: `scripts/tests/test_deploy_linode_one_shot.py`
- Modify: `scripts/tests/test_remote_target.py`
- Create or modify: a focused cleanup-contract test under `scripts/tests/`
- Modify: `Makefile` if a focused test target is needed

Steps:

1. Add failing tests that assert the default durable artifact paths no longer point into `.spin/`.
2. Add a failing test that proves `make clean` no longer removes durable operator state.
3. Add a failing test that proves the destructive reset target does remove `.spin/`.
4. Verify red before implementation, then implement the minimal changes to make them pass.

## Task 4: Update plan/docs/backlog truth

**Files:**
- Modify: `docs/plans/2026-03-07-generic-ssh-remote-maintenance-layer-design.md`
- Modify: `docs/plans/2026-03-06-linode-shared-host-setup-skill-and-handoff-plan.md`
- Modify: `skills/prepare-shared-host-on-linode/SKILL.md`
- Modify: `skills/prepare-shared-host-on-linode/references/OPERATIONS.md`
- Modify: `skills/deploy-shuma-on-linode/SKILL.md`
- Modify: `skills/deploy-shuma-on-linode/references/OPERATIONS.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

Steps:

1. Update plans and skills so they no longer claim durable day-2 state lives under `.spin/`.
2. Record the clean/reset contract explicitly.
3. Add or update an execution-ready TODO tranche for the durable-state lifecycle fix if needed.
4. Archive it immediately once the tranche is complete.

## Task 5: Verify the full operator path

**Files:**
- No new product files unless verification exposes a real gap

Steps:

1. Run the relevant focused tests first.
2. Run full `make test`.
3. Rerun the live Linode happy path from scratch:
   - prepare Linode shared host,
   - deploy Shuma,
   - confirm the emitted durable state and auto-selected remote,
   - run cleanup commands including `make clean`,
   - confirm day-2 commands still work,
   - apply a temporary visible dashboard style change,
   - commit it,
   - run `make remote-update`,
   - open the remote dashboard and confirm the change is visible,
   - revert the temporary change cleanly afterward.
4. Review all recently completed TODOs and linked plan requirements immediately after completion.

## Verification

1. `make test-deploy-linode`
2. Focused cleanup-contract verification target if added
3. `make test`
4. Live Linode setup/deploy/day-2/update proof

## Non-Goals

1. Do not generalize beyond the current `ssh_systemd` day-2 backend.
2. Do not redesign the remote receipt schema unless required by the path move.
3. Do not add backward-compatibility fallback reads from legacy `.spin/` paths unless the live proof shows they are strictly necessary.
