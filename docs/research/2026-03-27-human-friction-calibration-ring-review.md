Date: 2026-03-27
Status: Research review

Related context:

- [`2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`2026-03-27-game-loop-current-state-and-gap-review.md`](2026-03-27-game-loop-current-state-and-gap-review.md)
- [`../plans/2026-03-27-game-loop-board-state-refactor-plan.md`](../plans/2026-03-27-game-loop-board-state-refactor-plan.md)
- [`../plans/2026-03-19-monitoring-human-friction-denominator-plan.md`](../plans/2026-03-19-monitoring-human-friction-denominator-plan.md)
- [Google: Getting Started With Measuring Web Vitals](https://web.dev/articles/vitals-measurement-getting-started)
- [Google: Site Speed And Business Metrics](https://web.dev/articles/site-speed-and-business-metrics)
- [Google SRE Workbook: Alerting On SLOs](https://sre.google/workbook/alerting-on-slos/)

# Purpose

Define how Shuma should eventually measure human friction in a way that is:

1. based on real human evidence,
2. separable from adversary-sim truth,
3. actionable for config or code decisions,
4. and compatible with the Game Loop's board-state doctrine.

# Findings

## 1. Human friction must come from field evidence, not lab-only or sim-only evidence

Google's Web Vitals guidance explicitly distinguishes field measurement from synthetic or lab-only measurement and recommends measuring user experience in the field with real-user data collection.

For Shuma that means:

1. adversary-sim traffic cannot count as human-friction evidence,
2. local browser smoke tests are useful but not sufficient,
3. and the real calibration ring must use human-operated traversals or a clearly human-operated test cohort.

## 2. Human friction must include task success, not only latency

Speed alone is not the whole story.
Google's guidance on correlating site speed and business metrics recommends tying performance data to real outcome measures such as conversion or abandonment rather than treating timing metrics as the only user-impact signal.

For Shuma the parallel is:

1. measure whether the human reached the intended content,
2. whether the visit required extra retries or challenge steps,
3. and whether the visit was abandoned after friction was added.

So the human ring must treat "human got through" as a first-class outcome, not just "page became interactive within threshold."

## 3. Human friction needs its own budget semantics and burn-rate logic

The SRE Workbook's guidance on alerting from SLO burn rates is directly relevant here.
Shuma should not react to every single human-friction spike as if it were a stable trend, but it also should not let a sharp regression hide inside long-window averages.

So the human-friction ring should eventually support:

1. short-window harm detection,
2. longer-window budget consumption,
3. and change-aware comparison against a retained baseline.

## 4. Human friction should be measured at the journey level and the defense-surface level

The board-state doctrine means the host site is the board and Shuma's defenses are the pieces.
So human-friction evidence must preserve:

1. the journey attempted,
2. the defense surface encountered,
3. the friction imposed there,
4. and whether the user still reached the intended board position.

That means route-local and defense-local evidence matters more than one global "human friction score."

# Recommended Human-Friction Ring

## A. Keep it separate from the strict non-human exclusion proof

The strict adversary loop should continue to ask:

1. how much non-human traffic leaked,
2. where it breached,
3. and what config or code closes that gap.

The human ring should ask a different question:

1. given the currently defended config,
2. what burden does that defense place on actual humans,
3. and is that burden still inside the tolerated budget?

## B. Measure a small set of human-operational outcomes

Recommended core measurements:

1. `content_reached`
   - did the human reach the intended page or content state?
2. `challenge_burden`
   - how many challenges, puzzles, or verification steps were imposed?
3. `extra_latency_ms`
   - how much additional delay was added beyond the retained baseline?
4. `retry_count`
   - how many extra attempts were required?
5. `abandonment`
   - did the human leave before reaching the intended content?
6. `route_or_surface_id`
   - which board location and defense surface imposed the friction?

Optional browser-user metrics to attach where possible:

1. CWV or nearby page metrics such as LCP and INP,
2. response timing for key route transitions,
3. and route-to-route dwell or completion timing.

## C. Tag human-friction evidence to config and code lineage

Every human-calibration run should record:

1. the active config revision,
2. the retained baseline or last accepted episode,
3. the relevant defense families touched,
4. and the journey attempted.

Without that, human-friction evidence cannot help the Game Loop choose between:

1. keeping a strict successful anti-bot config,
2. relaxing one defense family slightly,
3. or escalating to a code change that reduces human burden without reopening the attacker breach.

## D. Prefer explicit human journeys over generic browsing

The ring should start with a small canonical human journey set:

1. land on public content,
2. navigate to detail content,
3. submit one safe human interaction,
4. recover from one challenge,
5. and verify the final content was reached.

That will be more actionable than vague "browse around the site and see if it feels okay" testing.

# Research Conclusion

The correct human-friction ring for Shuma is:

1. real-human and field-derived,
2. journey-local and defense-local,
3. tied to config and episode lineage,
4. measured with both success and burden signals,
5. and budgeted with burn-rate style alerting rather than one flat score.

The current repo has enough telemetry foundation to plan that ring cleanly, but not enough yet to claim it is operational.
