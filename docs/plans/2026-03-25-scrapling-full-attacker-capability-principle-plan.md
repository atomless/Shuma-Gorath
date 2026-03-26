# Scrapling Full Attacker-Capability Principle Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reframe the Scrapling planning chain so Shuma uses attacker-relevant upstream Scrapling capability wherever it materially increases adversary power against Shuma defenses, with any non-adoption overtly justified rather than passively omitted.

**Architecture:** Preserve the already-landed truthful request-native Scrapling baseline, but stop treating that baseline as the default outer limit of Scrapling maturity. Re-run the capability matrix under the stronger full-spectrum mandate: if an upstream Scrapling capability materially increases adversary power against Shuma defenses, it belongs in the active Scrapling maturity work unless an explicit exclusion record proves otherwise. Keep category-ownership questions such as `automated_browser` separate where helpful, but do not let taxonomy purity suppress real attacker capability.

**Tech Stack:** Planning docs, TODO backlog, mature adversary-sim roadmap, later LLM attacker sequencing, upstream Scrapling docs.

---

## Guardrails

1. Do not rewrite the principle as "adopt every upstream Scrapling feature" with no filtering.
2. Do require explicit adoption or explicit exclusion for every attacker-relevant upstream capability that maps to Scrapling-owned surfaces.
3. Do not let browser or stealth Scrapling adoption silently broaden category ownership beyond the already-ratified boundaries, but also do not use category-ownership purity as a reason to suppress attacker-relevant capability.
4. Do not treat upstream docs or marketing claims as proof of Shuma behavior.
5. Keep the later LLM attacker runtime blocked until the Scrapling capability matrix and omission ledger are settled.
6. Do not assign away attacker-relevant Scrapling power merely because it is browser-class if it materially increases attacks on Shuma-owned surfaces.

## Task 1: Add The New Governing Principle To Research And Plans

**Files:**
- Modify: `docs/research/2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md`
- Modify: `docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`
- Modify: `docs/plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md`
- Modify: `docs/plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`

**Work:**
1. Replace the conservative "only reopen if a later gap proves it" framing with the stronger full-spectrum principle: attacker-relevant upstream Scrapling capability should be adopted whenever it materially increases adversary power against Shuma defenses.
2. Preserve the distinction between:
   - Scrapling-owned surfaces,
   - later broader browser or stealth capability adoption,
   - and separate `automated_browser` category ownership.
3. State explicitly that every omitted attacker-relevant capability must have a recorded reason showing why it does not increase effective attack power, is covered elsewhere with proof, or is unsafe or untruthful to claim yet.

**Acceptance criteria:**
1. The active planning chain no longer treats broader Scrapling capability as a merely reluctant contingency.
2. The docs now distinguish default adoption for owned surfaces from explicit exclusions and separate category ownership questions.

## Task 2: Manifest The Principle As Real TODOs

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`

**Work:**
1. Add an active TODO tranche for the upstream attacker-capability matrix and explicit omission ledger.
2. Reframe `SIM-SCR-CHALLENGE-2C` so it no longer acts as the default parking lot for broader Scrapling power; attacker-relevant browser or stealth capability that materially increases attacks on currently owned surfaces belongs in the active full-spectrum Scrapling tranche instead.
3. Keep `SIM-SCR-BROWSER-1` separate only for the later truthful `automated_browser` question, not as a reason to defer active attacker power against current Shuma defenses.
4. Update `SIM-LLM-1` and `SIM-LLM-1C` blockers so the later full attacker runtime remains downstream of the settled Scrapling capability matrix and any resulting adoption or exclusion ledger.

**Acceptance criteria:**
1. The backlog contains one active full-spectrum Scrapling capability-expansion driver rather than only conditional blocked follow-ons.
2. The later LLM attacker runtime is still correctly downstream of mature Scrapling attacker-faithfulness.

## Task 3: Sync The Broader Roadmap

**Files:**
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`

**Work:**
1. Update the mature adversary roadmap so it says Scrapling should track and adopt attacker-relevant upstream capability for the surfaces it owns, not merely watch it passively.
2. Update the main feedback-loop sequencing note so the current request-native proof is treated as a baseline, not the end state of Scrapling maturity.

**Acceptance criteria:**
1. The roadmap now treats Scrapling as a maintained attacker-capability lane.
2. The main sequencing docs stay aligned with the new backlog wording.

## Task 4: Update Indexes And Audit Trail

**Files:**
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/completed-todo-history.md`

**Work:**
1. Add the new review and plan to the indexes.
2. Record the planning rewrite so the audit trail shows when the repo moved from conditional wider Scrapling adoption to default attacker-relevant capability adoption with explicit exclusions.

**Acceptance criteria:**
1. The new governing principle is discoverable from the doc indexes.
2. The TODO history explains why the planning chain changed.
