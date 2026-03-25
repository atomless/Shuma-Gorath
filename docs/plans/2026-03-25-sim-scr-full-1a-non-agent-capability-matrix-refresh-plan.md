Date: 2026-03-25
Status: Proposed

Related context:

- [`../research/2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-review.md`](../research/2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-review.md)
- [`2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`2026-03-25-scrapling-full-attacker-capability-principle-plan.md`](2026-03-25-scrapling-full-attacker-capability-principle-plan.md)
- [`2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Goal

Refresh the canonical Scrapling capability matrix so the next implementation slice (`SIM-SCR-FULL-1B`) is driven by the full attacker-relevant non-agent remit rather than the older request-native-bounded matrix.

# Guardrails

1. Do not claim that upstream browser or stealth docs prove Shuma behavior today.
2. Do not collapse the matrix into “adopt every upstream feature” without surface ownership discipline.
3. Do not keep dynamic or stealth Scrapling implicitly assigned away when they are needed for Scrapling-owned surfaces.
4. Do not let browser or stealth capability adoption silently widen taxonomy ownership into `automated_browser`, `browser_agent`, or `agent_on_behalf_of_human`.
5. Keep any explicit omission recorded with a reason and reconsideration trigger.

# Tasks

## Task 1: Freeze the refreshed full-power matrix

**Files:**
- Create: `docs/research/2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-review.md`
- Modify: `docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`
- Modify: `docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`

**Work:**
1. Record that the earlier `SIM-SCR-CAP-1` matrix was a request-native baseline, not the final maturity target.
2. Freeze the refreshed capability families for:
   - request-native fetchers,
   - dynamic browser fetchers,
   - stealth browser fetchers,
   - solver or bypass-style challenge capability,
   - and explicit temporary exclusions.
3. State clearly that dynamic or stealth capability can belong to Scrapling-owned surfaces without automatically claiming later browser-agent category ownership.

**Acceptance criteria:**
1. The repo now has one current matrix for full-power Scrapling.
2. The mainline plan no longer implies browser or stealth Scrapling is assigned away by default.

## Task 2: Sync the active and blocked backlog

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `docs/plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`

**Work:**
1. Close `SIM-SCR-FULL-1A` once the refreshed matrix is written.
2. Keep `SIM-SCR-FULL-1B` as the next active coding slice.
3. Reframe `SIM-SCR-CHALLENGE-2C` so it depends on the refreshed matrix outcome rather than the older request-native-bounded one.
4. Reframe `SIM-SCR-BROWSER-1` so it stays about truthful `automated_browser` category ownership, not about avoiding dynamic or stealth capability inside the current Scrapling-owned surface remit.

**Acceptance criteria:**
1. The next active implementation step is clear.
2. The blocked backlog no longer preserves the older “browser-runtime assigned away by default” wording.

## Task 3: Update indexes and audit trail

**Files:**
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`
- Modify: `todos/completed-todo-history.md`
- Create: `docs/research/2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-post-implementation-review.md`

**Work:**
1. Add the new review and plan to the indexes.
2. Record the tranche completion and why it superseded the older narrower matrix.
3. Capture a short post-implementation review for the docs-only slice.

**Acceptance criteria:**
1. The new matrix is discoverable from the planning chain.
2. The audit trail shows exactly when the repo moved from a request-native-bounded matrix to the fuller non-agent remit.

# Definition Of Done

This tranche is complete when:

1. the refreshed Scrapling matrix is written and discoverable,
2. `SIM-SCR-FULL-1B` is the explicit next active slice,
3. stale wording that assigns dynamic or stealth Scrapling away by default is removed from the current mainline chain,
4. and the audit trail records why the older matrix is now only historical context.
