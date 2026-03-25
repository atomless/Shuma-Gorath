# SIM-SCR-CAP-1 Capability Matrix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Freeze the attacker-relevant upstream Scrapling capability matrix and explicit omission ledger for Shuma's current Scrapling-owned surfaces, then promote the real next implementation slice that falls out of that matrix.

**Architecture:** Treat current request-native Scrapling as a truthful baseline, not the maturity target. Separate request-native fidelity gaps from browser-class capabilities. Keep browser or stealth Scrapling explicit and assigned rather than silently omitted, but do not blur it into the current request-native taxonomy ownership.

**Tech Stack:** Planning docs, active and blocked TODO backlog, mature adversary-sim roadmap, feedback-loop sequencing docs, official Scrapling documentation.

---

## Guardrails

1. Do not collapse the matrix into “adopt every Scrapling feature.”
2. Do not keep request-native omissions vague once the matrix is frozen.
3. Do not quietly expand Scrapling category ownership into `automated_browser`, `browser_agent`, or `agent_on_behalf_of_human`.
4. Do not treat upstream documentation as proof of Shuma behavior.
5. Do require every attacker-relevant capability to end up as one of:
   - adopt now,
   - keep adopted,
   - assign elsewhere,
   - explicitly exclude for now.

## Task 1: Freeze The Canonical Matrix

**Files:**
- Create: `docs/research/2026-03-25-sim-scr-cap-1-upstream-capability-matrix-review.md`
- Modify: `docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`

**Work:**
1. Capture the current upstream capability families against current Shuma ownership.
2. Make the omission ledger explicit instead of implied.
3. Record the distinction between:
   - current request-native ownership,
   - separate browser-class capability,
   - and explicit exclusions.

**Acceptance criteria:**
1. The repo has one canonical matrix stating what Scrapling can do upstream and what Shuma will do with each capability.
2. There are no remaining implicit “maybe later” omissions for attacker-relevant capability.

## Task 2: Promote The Real Next Implementation Slice

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`

**Work:**
1. Close `SIM-SCR-CAP-1` once the matrix is frozen.
2. Add the next active follow-on:
   - `SIM-SCR-RN-1` request-native attacker-fidelity uplift
3. Narrow `SIM-SCR-CHALLENGE-2C` so it reopens only if:
   - Shuma later ratifies browser-class Scrapling-owned surfaces, or
   - the request-native uplift still proves insufficient for a currently owned surface
4. Keep `SIM-SCR-BROWSER-1` as the later `automated_browser` question.
5. Update the later LLM attacker blockers so they now wait on the request-native fidelity uplift rather than the just-completed matrix freeze.

**Acceptance criteria:**
1. The next coding task is explicit and concrete.
2. Browser or stealth Scrapling remains explicit, but no longer vague.

## Task 3: Sync The Indexes And Audit Trail

**Files:**
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/completed-todo-history.md`

**Work:**
1. Add the new review and plan to the indexes.
2. Record the matrix freeze and the new next-step consequence in the completion history.

**Acceptance criteria:**
1. The matrix is discoverable from the planning chain.
2. The audit trail explains why request-native fidelity became the next mainline instead of a vague wider-browser branch.
