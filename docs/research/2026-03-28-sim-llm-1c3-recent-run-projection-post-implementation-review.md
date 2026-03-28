# SIM-LLM-1C3 Recent-Run Projection Post-Implementation Review

Date: 2026-03-28  
Status: implemented

## Scope

Close the remaining `SIM-LLM-1C3` gap by making the live `bot_red_team` runtime visible through the same recent-run and machine-first operator surfaces already used for Scrapling, without enabling the lane in controls or leaking simulator privilege into runtime behavior.

## What Landed

1. LLM runtime worker-result ingest now persists one additive recent-run receipt event:
   - it carries run identity,
   - lane and profile identity,
   - projected mode and category targets,
   - bounded runtime lineage,
   - action counts,
   - terminal failure truth,
   - and bounded per-action receipts.
2. Recent-run accumulation no longer treats receipt-backed history as Scrapling-only:
   - the shared recent-run path now recognizes LLM receipt events,
   - merges repeated LLM summaries within the bounded monitoring window,
   - and projects `bot_red_team` categories from receipt truth instead of guessing from profile name alone.
3. Hot-read and operator-snapshot recent-run rows now carry additive `llm_runtime_summary` truth:
   - generation source,
   - provider and model,
   - executed and failed action counts,
   - last response status,
   - failure class or terminal failure,
   - and bounded action receipts.
4. The existing `Recent Red Team Runs` UI now renders that additive truth without inventing a new LLM-only panel:
   - the row keeps the shared run table structure,
   - adds a `Runtime` column,
   - and shows provider-backed versus degraded lineage without implying the lane is operator-enabled.
5. Scrapling recent-run coverage remains intact:
   - the new field is optional,
   - and Scrapling rows still render with their existing owned-surface evidence unchanged.

## Why This Matters

Before this slice, the later LLM attacker could already:

1. generate bounded actions,
2. dispatch a runtime worker,
3. and ingest a typed result.

But after ingest, that actor still mostly disappeared from the recent-run and operator-snapshot evidence path. That made the repo understate what was already real and blocked truthful later combined-attacker proof. This slice closes that observability seam without widening the operator control surface prematurely.

## Verification Outcome

1. `make test-adversarial-llm-runtime-projection`
   - proves LLM runtime receipt persistence,
   - recent-run category and mode projection,
   - operator-snapshot preservation,
   - and rendered Red Team visibility.
2. `make test`
   - proves the canonical full local verification path still passes with the additive recent-run projection in place.
3. `git diff --check`
   - confirms the slice is formatting-clean before commit.

## Remaining Follow-On

1. `bot_red_team` is still intentionally disabled as an operator-selected control lane. This slice only closes observability truth for executed runtime work.
2. The next combined-attacker work should build on this shared recent-run path instead of inventing an LLM-only proof surface.
