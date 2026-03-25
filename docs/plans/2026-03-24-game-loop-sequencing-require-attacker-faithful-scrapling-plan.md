# Game Loop Sequencing Require Attacker-Faithful Scrapling Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make attacker-faithful Scrapling coverage an explicit prerequisite for the fuller attacker/defender game loop without blocking judge-side game-contract work.

**Architecture:** Preserve the current split between machine-first judge planning and later player-side runtime phases. Tighten the backlog and sequencing docs so the fuller game loop stays blocked until Scrapling-owned defense surfaces are covered by attacker-faithful Scrapling behavior with receipt-backed verification, and until any remaining uncovered surfaces are explicitly assigned to another lane.

**Tech Stack:** Planning docs, blocked backlog, recursive-improvement methodology docs, mature adversary-sim roadmap.

---

## Guardrails

1. Do not reinterpret this prerequisite as "adopt every upstream Scrapling feature."
2. Do not block judge-side contract planning on Scrapling completion.
3. Do block fuller player-side game-loop execution on attacker-faithful Scrapling coverage for Scrapling-owned surfaces.
4. Keep the prerequisite explicit about receipt-backed verification and explicit lane assignment for remaining gaps.

## Task 1: Sequence The Main Loop-Closure Plan

**Files:**
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`

**Work:**
1. State explicitly that `SIM-SCR-CHALLENGE-1` is a prerequisite for the fuller attacker/defender game loop.
2. State explicitly that browser or stealth Scrapling becomes a prerequisite only where the defense-surface matrix shows it is needed for Scrapling-owned surfaces after any request-native public-network identity follow-ons have been exhausted.
3. Keep judge-side `RSI-GAME-*`, `RSI-SCORE-1`, `RSI-PROTO-1`, and `RSI-EVAL-1` planning unblocked.

**Acceptance criteria:**
1. The planning chain no longer treats the existence of a Scrapling lane as sufficient attacker readiness.
2. The sequencing language distinguishes judge work from player-side execution readiness.

## Task 2: Tighten Blocked Player-Side Backlog Gates

**Files:**
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-24-llm-player-role-decomposition-plan.md`
- Modify: `docs/plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`

**Work:**
1. Add `SIM-SCR-CHALLENGE-1` as an explicit blocker for the later attacker/defender runtime tracks.
2. Add conditional `SIM-SCR-BROWSER-1` language where the owned-surface matrix requires it.
3. Make the blocked wording require that any remaining uncovered surfaces be explicitly assigned to the correct follow-on rather than silently accepted, including request-native source-IP or proxy diversification when that is the real missing capability.

**Acceptance criteria:**
1. The player-side backlog now encodes the Scrapling prerequisite explicitly.
2. The game-loop and player-role plans remain consistent with the backlog wording.

## Task 3: Record The Principle In The Research/Plan Indexes And Completion History

**Files:**
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/completed-todo-history.md`

**Work:**
1. Add the new sequencing review and plan to the indexes.
2. Record the planning completion with the exact prerequisite wording.

**Acceptance criteria:**
1. The new sequencing rule is discoverable from the active planning chain.
2. The audit trail explains why the fuller game loop remains blocked on attacker-faithful Scrapling coverage.
