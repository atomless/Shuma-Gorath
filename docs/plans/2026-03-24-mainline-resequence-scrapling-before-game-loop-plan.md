# Mainline Resequence Scrapling Before Game Loop Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reorder the active backlog so attacker-faithful Scrapling becomes the immediate mainline, followed by the first working self-improving loop, before later LLM attacker/defender runtime work or secondary dashboard follow-ons.

**Architecture:** Keep the previously written Scrapling and game-loop contract work, but change execution order. Promote the Scrapling challenge-expansion work from a deferred conceptual lane into the next active implementation queue, add execution-ready checklist items for the malicious/receipt-backed expansion, and update the later LLM attacker contract so it remains host-root-only, black-box, and confined to the same public knowledge an outside attacker could obtain from the attacked host itself.

**Tech Stack:** Planning docs, active and blocked TODO backlog, adversary-sim roadmap, later LLM player-role plans.

---

## Guardrails

1. Do not widen this into “all Scrapling features”; the active queue should target only attacker-relevant capability for Scrapling-owned surfaces.
2. Do not reopen the later LLM runtime actors ahead of the first proven game-loop run over attacker-faithful Scrapling.
3. Keep the LLM attacker explicitly black-box: host-root only, no Shuma internals, and no Shuma repo or docs lookup.
4. Preserve the distinction between judge-side planning and player-side execution readiness.

## Task 1: Reorder The Active Backlog

**Files:**
- Modify: `todos/todo.md`
- Modify: `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`

**Work:**
1. Add a new top-priority active section for attacker-faithful Scrapling and first-game-loop readiness.
2. Add execution-ready checklist items for:
   - `SIM-SCR-CHALLENGE-2A` defense-surface matrix and success contract
   - `SIM-SCR-CHALLENGE-2B` malicious request-native Scrapling interactions
   - `SIM-SCR-CHALLENGE-2D` receipt-backed coverage closure and explicit remaining-gap assignment
   - `SIM-SCR-CHALLENGE-2C` browser or stealth Scrapling adoption where required only if `2D` proves request-native Scrapling is still insufficient for an owned surface
   - `RSI-GAME-MAINLINE-1A` local route-level first working self-improving loop proof over the truthful attacker basis
   - `RSI-GAME-MAINLINE-1B` stronger follow-on proof over the same contract after `1A`
3. Make clear that the previous dashboard cleanup follow-ons are no longer the immediate mainline.

**Acceptance criteria:**
1. The next active tranche is plainly Scrapling attacker-faithfulness, not UI cleanup.
2. The backlog shows the first game-loop execution following directly after Scrapling proof.

## Task 2: Update The Later LLM Attacker Contract

**Files:**
- Modify: `docs/plans/2026-03-24-llm-player-role-decomposition-plan.md`
- Modify: `docs/plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**
1. State explicitly that the later LLM attacker starts from only the host site's root entrypoint and category-fulfillment objective.
2. State explicitly that it is confined to the same public-knowledge position as an outside attacker and may only use host-derived public hints such as `robots.txt`, sitemap references, and traversal-visible pages.
3. State explicitly that it must know nothing about Shuma internals, routes, defenses, source code, or docs, and must not be allowed to search the web for them.
4. Keep the malicious-category priming explicit for categories where malicious behavior is the point.

**Acceptance criteria:**
1. The later LLM attacker contract is unmistakably black-box.
2. The plan no longer leaves room for Shuma-aware attacker priming.
3. The repo explicitly says the attacker is limited to outside-attacker public knowledge, not product-internal awareness.

## Task 3: Record The Resequence In The Indexes And Completion History

**Files:**
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/completed-todo-history.md`

**Work:**
1. Add the new resequencing review and plan to the indexes.
2. Record the rationale in the completion log so the backlog shift is auditable.

**Acceptance criteria:**
1. The new mainline is discoverable from the planning chain.
2. The audit trail explains why Scrapling moved ahead of the fuller game loop and later LLM runtime work.

## Recommended Implementation Order

The optimal order is:

1. `SIM-SCR-CHALLENGE-2A`
   - freeze the owned defense-surface matrix and define which surfaces Scrapling must touch, fail, or be able to pass
2. `SIM-SCR-CHALLENGE-2B`
   - implement malicious request-native Scrapling behavior for the owned surfaces that remain request-native
3. `SIM-SCR-CHALLENGE-2D`
   - prove receipt-backed coverage, including explicit remaining-gap assignment where Scrapling does not own a surface
4. `SIM-SCR-CHALLENGE-2C`
   - only if `2D` proves a remaining Scrapling-owned surface still needs browser or stealth Scrapling
5. `CTRL-SURFACE-1`
6. `CTRL-SURFACE-2`
7. `CTRL-SURFACE-3`
   - freeze the legal move ring before broadening the self-improving loop
8. `RSI-GAME-1A`
9. `RSI-GAME-1B`
10. `RSI-SCORE-1`
11. `RSI-GAME-1C`
   - complete the judge-side game contract and episode/archive machinery
12. `RSI-GAME-MAINLINE-1A`
   - prove the first working self-improving loop over the now-truthful Scrapling attacker basis through the real post-sim route path
13. `RSI-GAME-MAINLINE-1B`
   - extend that proof into the next strongest truthful operational harness
14. after that, take the next backend contract slices before returning to deferred dashboard cleanup:
   - `SIM-LLM-1A`
   - `SIM-LLM-1B`
15. only after that, return to deferred dashboard follow-ons and the remaining later LLM runtime work:
   - `MON-OVERHAUL-1C`
   - `DIAG-CLEANUP-1`
   - `SIM-LLM-1B..1C`
   - `OVR-AGENT-2A..2C`

Dashboard/operator-surface cleanup can wait because it does not change the truthfulness of the attacker side or the legality and judgment of the loop itself.

Current note:

1. `RSI-GAME-1A`, `RSI-GAME-1B`, `RSI-SCORE-1`, and `RSI-GAME-1C` are now landed.
2. `RSI-GAME-MAINLINE-1A` and `RSI-GAME-MAINLINE-1B` are now landed, so the first working game-loop proof lane is complete.
3. [`TEST-MAINLINE-1`](2026-03-25-testing-suite-structure-and-mainline-friction-plan.md) is now landed, so the active attacker-faithful Scrapling -> game-loop path has one obvious low-friction verification bundle.
4. `SIM-LLM-1A` and `SIM-LLM-1B` are now landed, so the later full attacker runtime remains the next backend track but stays explicitly blocked until intentionally reopened; deferred dashboard cleanup remains the active queue beneath that blocked runtime path.
