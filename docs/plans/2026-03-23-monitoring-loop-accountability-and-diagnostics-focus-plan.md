# Monitoring Loop Accountability And Diagnostics Focus Plan

Date: 2026-03-23

## Goal

Redefine `MON-OVERHAUL-1` so Monitoring becomes the human-readable accountability surface for the closed loop, while Diagnostics becomes a more explicitly diagnostics-first deep-inspection surface.

## Principles

1. Monitoring must project machine-first truth, not invent a parallel human-only interpretation layer.
2. Monitoring should show observed outcomes and loop judgment separately.
3. Diagnostics should keep subsystem and forensic depth without dominating the operator narrative.
4. Tuning and posture editing stay out of Monitoring and remain later `TUNE-SURFACE-1` work.
5. Monitoring should describe accountability against the current operator-selected product stance; the later development reference stance belongs to later recursive-improvement methodology, not to the first Monitoring contract.

## Execution Slices

### MON-OVERHAUL-1A: Information Architecture And Ownership Reset

Objective:

Make the page contracts explicit before wiring more data into them.

Scope:

1. rewrite Monitoring copy and layout around loop-accountability questions:
   - loop verdict,
   - outcome frontier,
   - change judgment,
   - category breakdown,
   - trust and actionability.
2. tighten Diagnostics language and sectioning so it reads as diagnostics-first rather than a leftover Monitoring clone.
3. update Monitoring and Diagnostics docs to reflect the sharper ownership contract.
4. add focused rendered tests that prove the new tab framing and section ownership.
5. ensure the page language stays product-facing: it should explain performance against the current live stance, not imply that later reference-stance training episodes are the same thing.

Verification:

1. focused dashboard unit target for section ownership and IA,
2. focused Playwright smoke for the rendered Monitoring and Diagnostics identity,
3. `git diff --check`.

### MON-OVERHAUL-1B: Loop Verdict, Outcome Frontier, And Change Judgment Projection

Objective:

Project the highest-value machine-first contracts into Monitoring without dropping into subsystem detail.

Scope:

1. project top-level benchmark family status from `benchmark_results_v1`,
2. project prior-window comparison and trend direction,
3. project bounded controller status:
   - latest recommendation,
   - latest apply stage,
   - last rollback or retain result,
   - refusal or blocker reasons,
4. keep live, shadow, and adversary-sim semantics clearly separated.

Verification:

1. backend/admin payload contract proof if required,
2. focused dashboard unit/store tests,
3. focused Playwright rendered proof.

### MON-OVERHAUL-1C: Category Breakdown And Trust Surface

Objective:

Show where remaining non-human problems sit and how trustworthy the loop's conclusion is.

Scope:

1. project taxonomy rows with posture, observed share, leakage or cost, friction spillover, and evidence quality,
2. project verified-identity and beneficial-non-human guardrail state,
3. project tuning eligibility, protected-evidence readiness, and coverage blockers,
4. keep raw subsystem detail in Diagnostics rather than leaking it back into Monitoring.
5. if later recursive-improvement reference-stance runs are ever shown here, label them explicitly as separate evaluation context rather than the live product stance.

Verification:

1. focused dashboard unit and rendered proof for category tables and trust blockers,
2. `git diff --check`.

## Sequencing

1. execute `MON-OVERHAUL-1A` first,
2. then `TEST-HYGIENE-6` and the remaining dashboard archaeology cleanup against the settled Monitoring or Diagnostics contracts,
3. then `MON-OVERHAUL-1B`,
4. then `MON-OVERHAUL-1C`,
5. then later `TUNE-SURFACE-1`.

## Notes

1. Do not let Monitoring become a pure self-report from the controller. It must show observed outcome truth and controller judgment as distinct surfaces.
2. Do not pull raw event feeds, Prometheus helper internals, or deep subsystem telemetry into the top Monitoring flow.
3. If a surface is primarily about "what happened inside one subsystem?", it belongs in Diagnostics unless it is directly part of the loop-accountability narrative.
