# Monitoring Loop Accountability And Diagnostics Focus Plan

Date: 2026-03-23

## Goal

Redefine the operator-facing tab ownership so:

1. Monitoring becomes the human-readable accountability surface for the closed loop,
2. Traffic becomes the live and recent traffic visibility surface,
3. and Diagnostics becomes a more explicitly diagnostics-first and furniture-operational deep-inspection surface.

## Principles

1. Monitoring must project machine-first truth, not invent a parallel human-only interpretation layer.
2. Monitoring should show observed outcomes and loop judgment separately.
3. Traffic visibility should not be forced into Monitoring merely because it uses charts.
4. Diagnostics should keep subsystem and forensic depth without dominating the operator narrative.
5. Tuning and posture editing stay out of Monitoring and remain separate future operator-surface work that would need fresh planning if reopened.
6. Monitoring should describe accountability against the current operator-selected product stance; the later development reference stance belongs to later recursive-improvement methodology, not to the first Monitoring contract.
7. Monitoring should show bounded progress over recent completed loops against benchmark families, not only the latest loop outcome.
8. Monitoring is the human-readable projection of the loop's independent judge or evaluator, not one of the players in the later attacker-defender game.
9. Reuse-first still applies, but traffic-oriented aggregate chart and view-model surfaces should move to `Traffic` unless they directly serve loop accountability.
10. Monitoring should distinguish real numeric objective budgets from per-category posture outcomes; both may use target-vs-current presentation, but only the former should be framed as budget usage.

## Execution Slices

### MON-OVERHAUL-1A: Information Architecture And Ownership Reset

Objective:

Make the page contracts explicit before wiring more data into them.

Scope:

1. rewrite Monitoring copy and layout around loop-accountability questions:
   - loop verdict,
   - recent loop progress,
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
3. project bounded recent multi-loop benchmark progress over the most recent completed loops rather than only the latest result,
4. project recent controller action history so recommendation, apply, retain, rollback, and refusal patterns are visible alongside benchmark movement,
5. keep the progress surface benchmark-family-oriented rather than collapsing it into one synthetic score,
6. project bounded controller status:
   - latest recommendation,
   - latest apply stage,
   - last rollback or retain result,
   - refusal or blocker reasons,
7. keep live, shadow, and adversary-sim semantics clearly separated,
8. do not pull traffic-oriented aggregate charts into Monitoring unless they directly serve loop-accountability truth.

### TRAFFIC-TAB-1: Dedicated Traffic Visibility Surface

Objective:

Introduce a first-class `Traffic` tab so live and recent traffic visibility stops competing with both Monitoring and Diagnostics ownership.

Scope:

1. move the current traffic-oriented Diagnostics sections into `Traffic`,
2. keep the reused components truthful to traffic visibility rather than loop-accountability or subsystem furniture proof,
3. add manual refresh plus bounded auto-refresh for this live traffic view,
4. keep the tab distinct from both Monitoring and Diagnostics in copy and ownership.

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
6. where the machine-first contract already supports it, category surfaces may show bounded recent trend, but Monitoring must remain a bounded accountability view rather than a full history browser.
7. keep the current high-level overall top line, but make real numeric budgets more explicit with target-vs-current visual budget usage rather than relying on wording like `inside_budget` as the primary signal.
8. express taxonomy categories as `Category target achievement` or equivalent target-vs-achieved language, not as fake per-category configured budgets.

Verification:

1. focused dashboard unit and rendered proof for category tables and trust blockers,
2. `git diff --check`.

### DIAG-CLEANUP-1: Diagnostics Ownership Cleanup After Monitoring Reuse

Objective:

Remove the remaining traffic-facing and aggregate leftovers from Diagnostics only after `Traffic` has claimed the traffic visibility surface and Monitoring has kept only the loop-accountability views it genuinely needs.

Scope:

1. remove or demote transitional aggregate and traffic-facing sections that no longer belong in Diagnostics once `Traffic` owns traffic visibility and Monitoring owns loop accountability,
2. retain the clearly diagnostics-first sections:
   - defense-specific diagnostics,
   - telemetry diagnostics,
   - external monitoring helper/export material,
3. clean up redundant helper or view-model code that only existed to support the transitional mixed-ownership shape,
4. keep the cleanup narrow and ownership-focused rather than turning it into a second redesign.

Verification:

1. focused dashboard unit and rendered proof for retained Diagnostics section ownership,
2. focused rendered proof that the removed traffic-facing sections now appear only in `Traffic`, while any remaining loop-accountability surfaces appear only in Monitoring,
3. `git diff --check`.

## Sequencing

1. execute `MON-OVERHAUL-1A` first,
2. then `MON-OVERHAUL-1B` so the loop-accountability surface becomes real,
3. then execute `TRAFFIC-TAB-1` so the traffic visibility surface gets its own truthful home,
4. then execute `DIAG-CLEANUP-1`,
5. then execute `MON-OVERHAUL-1C`,
6. then later operator-facing Tuning work, if reopened through fresh planning.

## Notes

1. Do not let Monitoring become a pure self-report from the controller. It must show observed outcome truth and controller judgment as distinct surfaces.
2. Do not pull raw event feeds, Prometheus helper internals, or deep subsystem telemetry into the top Monitoring flow.
3. If a surface is primarily about "what happened inside one subsystem?", it belongs in Diagnostics unless it is directly part of the loop-accountability narrative.
4. Do not delete shared traffic or aggregate diagnostics helpers just to make Diagnostics look cleaner before `Traffic` and Monitoring have had one clean chance to adopt the pieces they genuinely need.
