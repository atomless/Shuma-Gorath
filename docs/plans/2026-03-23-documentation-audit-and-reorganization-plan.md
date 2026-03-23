# Documentation Audit And Reorganization Implementation Plan

Historical execution note: this plan describes the first cleanup slice, which briefly used explicit `archive/outdated` buckets as an intermediate step. The follow-on flattening tranche later removed those nested directories once the top-level indexes were strong enough; see [`2026-03-23-archive-directory-flattening-plan.md`](2026-03-23-archive-directory-flattening-plan.md).

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Re-center the docs on the shared-host-first mainline, archive clearly outdated deferred-edge docs, and add enough information architecture that readers can find current topics without wading through historical clutter.

**Architecture:** Treat documentation as a product surface with a clear split between current truth, deferred work, and archived history. Keep active operator and contributor entry points short and curated. This first slice used explicit `archive/outdated` buckets as an intermediate step before the later flat-layout cleanup.

**Tech Stack:** Markdown docs, repo-local indexes, archive directories, TODO history

---

### Task 1: Record The Audit And Execution Boundaries

**Files:**
- Create: `docs/research/2026-03-23-documentation-audit-and-information-architecture-review.md`
- Create: `docs/plans/2026-03-23-documentation-audit-and-reorganization-plan.md`

**Steps:**
1. Capture the documentation findings: mixed current/deferred entry docs, defunct Fermyon proof chain, missing plan index, and missing `archive/outdated` distinction.
2. Freeze the first cleanup slice around entry-doc cleanup plus deferred-edge archival, not a whole-repo historical purge.

### Task 2: Add Better Front Doors For The Docs Tree

**Files:**
- Create: `docs/plans/README.md`
- Create: `docs/deferred-edge-gateway.md`
- Modify: `docs/index.md`
- Modify: `docs/research/README.md`

**Steps:**
1. Add a plans index that groups current mainline work by topic instead of date only.
2. Add one deferred-edge explainer that tells the truth: edge is later gateway-only work, not the current closed-loop target.
3. Rewrite the top-level docs index so it prioritizes current posture, current operating docs, topic indexes, and archives.
4. Rewrite the research index intro so it distinguishes active drivers from historical material and points to archives explicitly.

### Task 3: Create Explicit Outdated Archives And Move Defunct Deferred-Edge Docs

**Files:**
- Create: `docs/plans/archive/outdated/README.md`
- Create: `docs/research/archive/outdated/README.md`
- Move: `docs/research/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md`
- Move: `docs/research/2026-03-12-fermyon-akamai-edge-live-proof.md`
- Move: `docs/research/2026-03-14-fermyon-edge-signal-and-blank-slate-live-proof.md`
- Move: `docs/plans/2026-03-09-fermyon-akamai-edge-baseline-prerequisite-plan.md`
- Move: `docs/plans/2026-03-10-fermyon-akamai-edge-skill-implementation-plan.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`

**Steps:**
1. Create explicit outdated archive buckets under plans and research.
2. Move the deferred-edge blocker/proof/prerequisite docs into those buckets.
3. Update the archive READMEs so contributors understand when “archive” versus “archive/outdated” is appropriate.

### Task 4: Make Entry Docs Truthful About The Current Mainline

**Files:**
- Modify: `docs/deployment.md`
- Modify: `docs/quick-reference.md`
- Modify: `docs/testing.md`

**Steps:**
1. Update deployment docs so the current design record is the shared-host-first plan, not the older partially superseded deployment plan.
2. Move edge/Fermyon commands and proofs out of the default command flow and into clearly labeled deferred-edge sections.
3. Keep the commands documented, but explicitly mark them as non-mainline and not part of the current closed-loop rollout path.

### Task 5: Repair The Link Graph And Record The Cleanup

**Files:**
- Modify: any docs/backlog/history files that still point at the moved deferred-edge docs
- Modify: `todos/completed-todo-history.md`

**Steps:**
1. Update references from active docs, relevant roadmap/backlog files, and completion history so the moved docs still have a discoverable trail.
2. Add a completion record explaining why the docs were reorganized and what was archived.
3. Verify with `git diff --check`.
