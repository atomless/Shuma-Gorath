Date: 2026-03-23
Status: Research review

Related context:

- [`../README.md`](../README.md)
- [`../deployment.md`](../deployment.md)
- [`../quick-reference.md`](../quick-reference.md)
- [`../testing.md`](../testing.md)
- [`../current-system-architecture.md`](../current-system-architecture.md)
- [`../plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md`](../plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md)
- [`2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Historical Note

This review drove the first cleanup slice, which initially introduced explicit `archive/outdated` buckets for deferred-edge material. The same-day follow-on flattening tranche later removed the nested archive directories and kept the same semantic split in the top-level indexes instead. See [`2026-03-23-archive-directory-flattening-review.md`](2026-03-23-archive-directory-flattening-review.md).

# Purpose

Audit the current docs tree after the shared-host-first closed-loop work landed, identify where documentation structure and claims have drifted away from the actual project posture, and define the first cleanup slice that makes the docs easier to navigate before `MON-OVERHAUL-1`.

# Findings

## 1. The entry docs mix current truth with deferred or historical material

`docs/README.md`, `docs/deployment.md`, `docs/quick-reference.md`, and `docs/testing.md` all still surface the Fermyon/Akamai path and its live-proof commands close to the main shared-host/Linode workflow. That makes the current pre-launch posture harder to read even though the roadmap already says the real adaptive loop is shared-host-first and that the edge path is deferred.

## 2. A clearly defunct deferred-edge proof chain is still sitting in active locations

These documents are no longer the right active entry points:

1. `docs/research/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md`
2. `docs/research/2026-03-12-fermyon-akamai-edge-live-proof.md`
3. `docs/research/2026-03-14-fermyon-edge-signal-and-blank-slate-live-proof.md`
4. `docs/plans/2026-03-09-fermyon-akamai-edge-baseline-prerequisite-plan.md`
5. `docs/plans/2026-03-10-fermyon-akamai-edge-skill-implementation-plan.md`

They still matter as historical receipts for a later deferred edge track, but they now clutter the active docs tree and overstate the relevance of a path that does not host the current closed feedback loop.

## 3. The research and plan folders have outgrown their indexes

`docs/research/README.md` and the lack of a parallel `docs/plans/README.md` make it hard to answer simple questions like:

1. which docs define the current mainline,
2. which docs are historical tranche evidence,
3. which docs describe deferred edge work,
4. and where to start for a given topic.

## 4. Archive rules exist, but they do not yet distinguish “delivered historical” from “outdated for the current posture”

`docs/plans/` and `docs/research/` exist, but the repository does not yet have a clear home for docs that are not merely delivered, but actively misleading if they remain in the active tree.

# Implications

The documentation problem is now an information-architecture problem, not just a stale-link problem.

The first cleanup slice should:

1. re-center all entry docs on the shared-host-first, live-proven control loop,
2. move the clearly defunct deferred-edge proof chain into explicit `archive/outdated` locations,
3. create topic indexes so readers can navigate by current concern instead of by date-only filenames,
4. and keep the later edge path available, but clearly marked as deferred gateway-only work.

# Recommended First Slice

1. Add a concise `docs/plans/README.md` so the plan folder has a navigable front door.
2. Add one dedicated `docs/deferred-edge-gateway.md` explainer that tells the truth about the later edge posture and points to archived historical material.
3. Create `docs/research/archive/outdated/` and `docs/plans/archive/outdated/` as explicit homes for defunct deferred-edge docs.
4. Rewrite `docs/README.md`, `docs/deployment.md`, `docs/quick-reference.md`, and `docs/testing.md` so current shared-host-first workflows come first and deferred edge material is grouped separately.
5. Update link targets so the active roadmap and backlogs point to the new archive locations where needed.

# Non-Goals For This First Slice

1. Do not archive every post-implementation review in one pass. Many are historical but still useful evidence in the current planning chain.
2. Do not rename runtime or config concepts in code to remove `edge-fermyon`; that is later architecture work already captured in the roadmap.
3. Do not delete later edge skills or commands; keep them, but move them out of the default docs path.
