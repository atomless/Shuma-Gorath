Date: 2026-03-27
Status: Proposed current-state review

Related context:

- [`2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`2026-03-27-game-loop-category-posture-scoring-audit.md`](2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`2026-03-26-game-loop-scoring-and-diagnoser-audit.md`](2026-03-26-game-loop-scoring-and-diagnoser-audit.md)
- [`2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md`](2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md)
- [`../plans/2026-03-27-game-loop-board-state-refactor-plan.md`](../plans/2026-03-27-game-loop-board-state-refactor-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/benchmark_scrapling_exploit_progress.rs`](../../src/observability/benchmark_scrapling_exploit_progress.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

# Purpose

Explain, in simple terms, how the Game Loop works today, how its scoring actually works, and how far it still is from a version that can truly self-improve Shuma's defenses against both external and internally generated non-human traffic.

# Executive Summary

In plain English, the current Game Loop is:

1. a machine-first judge over recent telemetry,
2. plus a bounded config recommender and canary loop,
3. plus a dashboard projection of those judge outputs.

That means it is no longer just a static dashboard.
It can already:

1. score recent traffic against declared targets,
2. compare the current window with a prior window,
3. decide whether tuning is blocked, should observe longer, should recommend a bounded config move, or should refer the problem outward,
4. and keep retain or rollback episode history when config moves are tested.

But it is not yet a fully trustworthy self-improving defense loop.

The main reason is simple:
the best attacker-side signals are now visible, but they still do not yet drive precise shared-path category truth, precise host-cost attribution, or precise config-vs-code decisions as cleanly as they need to.

So today the loop is stronger as:

1. a judge,
2. a bounded config experiment harness,
3. and a proof surface,

than it is as:

1. a fully self-improving defender.

# How The Game Loop Functions Right Now

## 1. Shuma starts from declared targets

`operator_snapshot_objectives.rs` defines the current policy profile, budgets, category postures, scorecard, rollback inputs, and homeostasis inputs.

Under strict `human_only_private`, the important ideas are:

1. likely-human friction is still a first-class objective,
2. suspicious forwarded reach, bytes, and latency are explicit cost budgets,
3. Scrapling exploit progress is an explicit optimization target,
4. category posture is also an explicit optimization target,
5. and scalarization is explicitly forbidden.

So the judge is not meant to collapse everything into one number.

## 2. Shuma turns recent telemetry into benchmark families

`benchmark_results.rs` builds one machine-first payload from current snapshot sections.

Right now the main family groups are:

1. `likely_human_friction`
2. `suspicious_origin_cost`
3. `scrapling_exploit_progress`
4. `scrapling_surface_contract`
5. `representative_adversary_effectiveness`
6. `beneficial_non_human_posture`
7. `non_human_category_posture`

That payload also includes:

1. tuning eligibility,
2. an escalation hint,
3. urgency,
4. comparison against a prior window,
5. and readiness or coverage information.

## 3. Shuma compares the current window with the prior window

`benchmark_comparison.rs` and `benchmark_results_comparison.rs` compare the latest benchmark subject against a prior reference.

That is how the loop gets:

1. `overall_status`
2. `improvement_status`
3. per-family comparison status
4. and later escalation guidance.

## 4. Shuma decides whether action is even allowed

`benchmark_results.rs` computes `tuning_eligibility`.

That step can block tuning if, for example:

1. category-aware evidence is not ready,
2. replay protection evidence is not eligible,
3. Scrapling surface evidence is incomplete,
4. exploit evidence quality is too weak,
5. or verified-identity guardrails are unhappy.

So outside-budget pressure alone is not enough to justify a move.

## 5. Shuma classifies the current problem

`benchmark_results_comparison.rs` chooses a primary outside-budget family and maps it to a problem class and a decision shape.

This is one of the most important current truths:

1. `likely_human_friction` and `suspicious_origin_cost` can become `config_tuning_candidate`
2. `scrapling_exploit_progress`, `scrapling_surface_contract`, and `non_human_category_posture` currently become `code_evolution_candidate`

That means the loop already knows those attacker-side gaps matter, but it does not yet treat them as clean bounded config-tuning problems.

## 6. Oversight reconcile either recommends a bounded patch or refuses

`oversight_reconcile.rs` takes the benchmark payload and returns one of a few high-level outcomes:

1. refuse because evidence is stale or contradictory
2. `within_budget`
3. `observe_longer`
4. `code_evolution_referral`
5. `config_ring_exhausted`
6. or `recommend_patch`

If it reaches `recommend_patch`, it also preserves:

1. the judge state,
2. the diagnosis,
3. the selected repair surface,
4. and the ranked bounded candidates.

## 7. The dashboard projects that machine truth

`GameLoopTab.svelte` currently renders:

1. current status cards
2. budget usage
3. where the pressure sits
4. tuning and escalation context
5. trust and blockers

So the tab is a projection of the machine-first payload, not the scoring engine itself.

# How The Scoring Works Right Now

## 1. Budget-style scoring

Some families are straightforward budgets.

Examples:

1. `likely_human_friction_rate`
2. `suspicious_forwarded_request_rate`
3. `suspicious_forwarded_byte_rate`
4. `suspicious_forwarded_latency_share`

These are compared against targets and marked:

1. `inside_budget`
2. `near_limit`
3. `outside_budget`
4. or `insufficient_evidence`

Under strict `human_only_private`, suspicious-origin cost uses adversary-sim scope as the tracked suspicious lane.

That is why the loop can honestly say:

1. suspicious forwarded leakage is zero,

while also saying:

2. Scrapling still made terrain-local exploit progress.

Those are different truths.

## 2. Exploit-progress scoring

`scrapling_exploit_progress` is built from the latest Scrapling owned-surface receipts.

It measures three zero-target metrics:

1. `scrapling_breach_surface_rate`
2. `scrapling_deepest_breach_stage_ratio`
3. `scrapling_pass_surface_success_rate`

Any positive value means Scrapling made some successful progress where the strict posture wants none.

This is the closest thing today to "how far the invader advanced on the board."

## 3. Surface-contract scoring

The surface-contract family is slightly different again.

It asks:

1. did Scrapling satisfy the contract for the surfaces it was required to exercise?

That is not the same as:

1. was Shuma generally breached?

Some surfaces are meant to pass sometimes, some are meant to fail, and some are mixed.

So this is a contract-truth plane, not a simple attacker-win percentage.

## 4. Category-posture scoring

`benchmark_non_human_categories.rs` computes per-category posture alignment from category receipts.

For a blocked category, the current score is:

`short_circuited_requests / total_requests`

So mathematically it can produce real partial values like:

1. `0.25`
2. `0.63`
3. `0.87`

The problem is not the formula.
The problem is the input truth.

Today, exact non-verified category receipts are weak.
`traffic_classification.rs` mostly maps generic suspicious non-verified traffic to `SuspiciousAutomation`, which crosswalks only to `unknown_non_human`.
And `non_human_classification.rs` still has a fallback that projects recent sim-run category presence as degraded `projected_recent_sim_run` receipts.

So the current category posture surface is often not wrong mathematically.
It is underpowered evidentially.

## 5. Overall status and escalation priority

`benchmark_results_comparison.rs` treats the system as:

1. `outside_budget` if any family is outside budget,
2. else `near_limit` if any family is near limit,
3. else `inside_budget` if at least one family is inside budget.

When multiple families are outside budget, there is a fixed priority order:

1. `likely_human_friction`
2. `scrapling_exploit_progress`
3. `scrapling_surface_contract`
4. `suspicious_origin_cost`
5. `beneficial_non_human_posture`
6. `non_human_category_posture`
7. `representative_adversary_effectiveness`

That priority matters because only the chosen primary family drives the headline escalation hint.

# What The Loop Already Does Well

## 1. It has a real judge

This is no longer just an operator intuition loop.
There is a machine-first scorecard, comparison model, urgency surface, and bounded reconcile contract.

## 2. It has a real bounded config loop

The repo can already:

1. recommend a bounded patch,
2. apply it in a controlled way,
3. archive the episode,
4. retain or roll back,
5. and mark the config ring as exhausted after repeated failed rolled-back moves.

## 3. It already distinguishes some important planes

The system already has separate concepts for:

1. human-friction guardrails,
2. suspicious-origin leakage,
3. Scrapling exploit progress,
4. surface-contract satisfaction,
5. evidence quality,
6. urgency,
7. and code referral.

That is a strong base.

# Where It Is Still Too Far Off

## 1. The UI is still noisier and more confusing than the machine truth

The Game Loop page still makes it too easy to read different planes as one story.

The biggest examples are:

1. origin leakage can look like total defensive success,
2. category posture can look like exact measured performance when it is actually unscored or degraded,
3. and the tab does not yet make "no config move was actually applied here" prominent enough.

## 2. Category posture is still not trustworthy enough

This is the nearest-term truth gap.

The current system can score categories correctly only if it has exact category receipts.
For non-verified Scrapling traffic, that exact Shuma-side inference is still weak.

So today:

1. the category posture section can be useful as a target shape,
2. but it is not yet a strong basis for fine-grained self-improving defense.

## 3. Exploit progress is visible, but it still does not cleanly drive config adaptation

Right now the scoring system correctly treats `scrapling_exploit_progress` as a serious problem.
But the escalation policy still classifies it as `code_evolution_candidate`, not a bounded config-tuning family.

That means the loop can say:

1. "the attacker got through here,"

but it often cannot yet say:

2. "therefore this exact small config move is the next best repair."

## 4. Host-cost attribution is not local enough yet

The loop can name breach loci, but it still needs richer, more local answers to:

1. what exactly was consumed past the expected stop point,
2. which defense surface is accountable,
3. and what smallest repair changes that local board state.

Without that, tuning still risks being too family-level and not local enough.

## 5. Human friction is still mostly a guardrail, not a real calibration ring

Today the loop treats likely-human friction as a budget and guardrail, which is useful.

But the later question you care about is stronger:

1. once Shuma finds a hard strict config, what actual burden does that impose on real humans?

That later ring is not implemented yet as a real traversal-calibration loop.

## 6. Code evolution is still only a referral, not a working second ring

Today the loop can say:

1. "this is a code gap,"

but it cannot yet:

2. hand that gap to a frontier LLM in a bounded, judge-controlled code-suggestion flow and then verify those code changes the same way it verifies config changes.

That is still planned, not implemented.

# Simple Bottom Line

If we strip the language right back:

## What the Game Loop is today

It is a real machine referee plus a bounded config experiment loop.

## What it is not yet

It is not yet a truly self-improving defense game that can:

1. observe a new attacker breach,
2. localize the exact weak defense surface,
3. choose the smallest effective bounded config move,
4. learn reliably from failed moves,
5. escalate cleanly to code change when config is exhausted,
6. and then verify the new code path under the same independent judge.

## How far off it is

Closer on plumbing than on final intelligence.

The judge, episode memory, rollback plumbing, and basic move-selection architecture are already real.
The main missing pieces are the ones that turn that machinery into trustworthy adaptation:

1. shared-path truthful category inference,
2. cleaner board-state scoring and UI projection,
3. breach-local host-cost attribution,
4. stronger config actionability from exploit progress,
5. real human-friction calibration,
6. and the later frontier-LLM code-evolution ring.

# Recommended Immediate Reading Of The Current State

The right short reading today is:

1. the loop can already judge and archive,
2. it can already canary and roll back bounded config changes,
3. it can already see meaningful attacker progress,
4. but it still cannot yet be trusted to fine-tune itself from all the attacker evidence that matters most.

That is exactly why the immediate work remains:

1. fix category-posture truth,
2. complete the board-state refactor,
3. and only then reopen the stronger code-evolution and human-friction rings.
