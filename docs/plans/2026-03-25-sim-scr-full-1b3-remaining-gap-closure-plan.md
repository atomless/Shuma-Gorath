# Remaining Gap Closure Plan

**Goal:** Close `SIM-SCR-FULL-1B3` truthfully by freezing which upstream Scrapling powers are now explicit omissions or separate-lane questions after the first browser-backed owned-surface slice.

**Architecture:** Do not invent another behavior tranche if no still-ratified owned surface needs it. Preserve the current owned-surface implementation, update the planning chain to say the remaining upstream powers are explicit exclusions or separate-lane questions, and move the mainline to `SIM-SCR-FULL-1C`.

**Tech Stack:** `docs/research/2026-03-25-sim-scr-full-1b3-remaining-gap-closure-review.md`, `docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`, `docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`, `todos/todo.md`, `todos/completed-todo-history.md`, `todos/blocked-todo.md`.

---

## Task 1: Freeze the omission ledger

**Files:**
- Modify: `docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`
- Modify: `docs/plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**
1. Record that after `SIM-SCR-FULL-1B2B`, no further currently owned surface needs another browser or stealth implementation slice before `SIM-SCR-FULL-1C`.
2. State explicitly that the remaining upstream powers are:
   - proxy or origin-distribution support: temporary shared-host exclusion,
   - `real_chrome`, `cdp_url`, and explicit persistent-profile controls: not currently required by a ratified owned surface,
   - `solve_cloudflare`: not applicable to the current internal Shuma challenge pages,
   - browser-class surfaces like maze, JS verification, and browser-automation detection: separate-lane questions.

## Task 2: Close the backlog slice

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Modify: `docs/plans/README.md`
- Modify: `docs/research/README.md`
- Create: `docs/research/2026-03-25-sim-scr-full-1b3-remaining-gap-closure-post-implementation-review.md`

**Work:**
1. Move `SIM-SCR-FULL-1B3` to completed with the recorded omission-ledger rationale.
2. Leave `SIM-SCR-FULL-1C` as the next active mainline Scrapling slice.

## Verification

This tranche is docs-only. Verify with:

```bash
git diff --check
```

## Definition Of Done

This slice is complete when:

1. the repo no longer implies there is another hidden owned-surface behavior tranche between `1B2B` and `1C`,
2. the remaining upstream Scrapling powers are explicitly classified as omissions or separate-lane questions,
3. and `SIM-SCR-FULL-1C` is the clear next mainline Scrapling step.
