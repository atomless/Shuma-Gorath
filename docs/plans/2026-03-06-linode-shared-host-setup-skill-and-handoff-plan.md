# Linode Shared-Host Setup Skill And Handoff Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a generic Linode shared-host setup precursor that turns a local site plus operator account details into the exact artifacts and inputs the Shuma Linode deploy path needs.

**Architecture:** Keep the setup skill generic and host-light. The setup flow should gather Linode/DNS/SSH/admin prerequisites, generate a deterministic surface-catalog artifact from the local site docroot without requiring a human-authored sitemap, and hand off to the existing Shuma deploy skill. The deploy path now supports prepared-instance handoff via `--existing-instance-id`; automated same-host origin staging remains an explicit follow-on item rather than hidden implied behavior.

**Tech Stack:** Repo-local skills, Markdown runbooks, Python deploy helpers, Makefile verification.

---

## Decisions Captured

1. The first acid test is `../dummy_static_site`, a simple multi-page static HTML docroot.
2. Setup and deploy skills should stay as ignorant as possible of host-site implementation details beyond what Shuma needs to defend the site.
3. A manually authored sitemap must not be a hard prerequisite for shared-host onboarding.
4. Deterministic deployment preflight and smoke still require an explicit frozen surface-catalog artifact.
5. The setup flow should generate that artifact automatically from local evidence sources, not ask the operator to invent it by hand.
6. The initial evidence order is:
   - local docroot inventory first,
   - `robots.txt` and `sitemap.xml` if present,
   - bounded live/shared-host discovery later,
   - telemetry and Scrapling as additive refinement later.
7. The only hard external prerequisite the setup flow should assume is that the operator already has a Linode account.
8. The setup flow should guide acquisition of the remaining inputs instead of treating them as unexplained prerequisites:
   - use Linode's Cloud Manager terminology (`API Tokens`, `Create a Personal Access Token`),
   - propose the setup machine's current public IP as the default `SHUMA_ADMIN_IP_ALLOWLIST` value, subject to operator confirmation,
   - treat the Linode public IP as sufficient for host-preparation-only steps,
   - require a real public domain/FQDN only before final canonical Shuma attach.
9. The Linode setup and deploy skills are agent capabilities, not human runbooks; the executable source of truth should be repo-local helpers and `make` targets, with the skill describing when to use them and what artifacts to expect.

## Scope For This Slice

1. Publish a new generic Linode shared-host setup skill.
2. Add a generic repo helper that compiles a site-surface catalog JSON from a local docroot and can be consumed by Linode setup rather than owned by it.
3. Update deploy/docs/backlog so the new setup skill is the documented precursor to Linode Shuma deployment.
4. Record the remaining same-host handoff gap explicitly.

## Non-Goals For This Slice

1. Do not claim that same-host Linode origin staging is fully automated end-to-end; only prepared-instance handoff is closed in-repo.
2. Do not complete the broader shared-host discovery tranche (`SIM-SH-SURFACE-1`) here.
3. Do not require sitemap authoring or Scrapling completion before initial shared-host setup.

## Implementation Tasks

### Task 1: Add the local docroot surface-catalog helper

**Files:**
- Create: `scripts/build_site_surface_catalog.py`
- Create: `scripts/site_surface_catalog.py`
- Create: `scripts/tests/test_build_site_surface_catalog.py`
- Modify: `Makefile`
- Modify: `scripts/README.md`

Steps:

1. Add a failing unit test covering static HTML docroot inventory.
2. Add a failing unit test covering PHP docroot index mapping and sitemap merge.
3. Implement the generic helper with deterministic sorting, hidden-file filtering, and path normalization compatible with existing gateway catalog helpers.
4. Add the helper test to `make test-deploy-linode`.
5. Update script docs.

### Task 2: Publish the setup skill

**Files:**
- Create: `skills/prepare-shared-host-on-linode/SKILL.md`
- Create: `skills/prepare-shared-host-on-linode/references/OPERATIONS.md`
- Modify: `README.md`
- Modify: `docs/README.md`
- Modify: `docs/deployment.md`
- Modify: `skills/deploy-shuma-on-linode/SKILL.md`
- Modify: `skills/deploy-shuma-on-linode/references/OPERATIONS.md`

Steps:

1. Define the skill trigger and output contract.
2. Document the generic site-surface catalog helper as the no-sitemap initial path consumed by Linode setup.
3. Make the deploy skill explicitly depend on the setup skill for operator preparation and artifact generation.
4. State current same-host handoff limits truthfully.

### Task 3: Capture backlog truth

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

Steps:

1. Archive the completed setup-skill/helper publication work.
2. Add an explicit active item for closing the same-host Linode handoff gap before the first `dummy_static_site` end-to-end proof.

## Verification

1. Run `make test-deploy-linode`.
2. Do not claim live Linode readiness beyond what the code/docs now truly support.
