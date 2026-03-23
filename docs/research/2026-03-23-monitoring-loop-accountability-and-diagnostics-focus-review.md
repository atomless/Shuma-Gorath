# Monitoring As Loop Accountability And Diagnostics Focus Review

Date: 2026-03-23

## Context

Shuma's first bounded closed config loop is now live-proven, and the machine-first contracts behind that loop are materially stronger than they were when the Monitoring and Diagnostics ownership split was first captured.

That creates a sharper question for `MON-OVERHAUL-1`:

1. what should the human Monitoring page primarily help an operator understand?

The tempting answer is "what should the operator tune manually?" But that is not the most truthful framing for Shuma's current architecture. The system already has a bounded controller loop, explicit benchmark families, tuning eligibility gates, protected-evidence checks, verified-identity guardrails, category posture, and canary apply or rollback semantics.

So the higher-value Monitoring surface is not a manual tuning cockpit first. It is a human-readable accountability and interpretation surface for the closed loop itself.

## Decision

`Monitoring` should be treated primarily as the human projection of loop effectiveness and loop judgment, not as the primary place where the operator decides which knob to turn next.

It should answer:

1. are the defences reducing unwanted non-human cost?
2. what friction or harm did that impose on likely-human or tolerated traffic?
3. what did the loop conclude and do?
4. did the most recent change improve things?
5. where does remaining unwanted non-human traffic still sit in the taxonomy?
6. how trustworthy is the current conclusion?

This is different from `Diagnostics`, which should become even more explicitly diagnostics-oriented than before.

## Monitoring Versus Diagnostics

### Monitoring

Monitoring should lead with loop-accountability questions:

1. current outcome vs budget,
2. current window vs prior window,
3. suspicious-origin cost vs likely-human friction frontier,
4. beneficial or verified non-human guardrail impact,
5. last recommendation or apply decision,
6. watch-window or rollback result,
7. per-category performance and posture,
8. trust or eligibility blockers.

### Diagnostics

Diagnostics should answer subsystem, transport, and forensic questions:

1. raw event tails,
2. subsystem counters,
3. deep challenge, maze, tarpit, CDP, GEO, and IP-range detail,
4. bounded feed mechanics,
5. telemetry freshness or overflow mechanics,
6. contributor and incident-debug inspection.

This means the transition note from 2026-03-20 remains directionally correct, but Diagnostics should now be made more intentionally diagnostics-first, not merely "the old Monitoring tab moved elsewhere."

## Recommended Monitoring Page Shape

### 1. Loop Verdict

Top-of-page summary:

1. improving, stable, or regressing,
2. inside budget, near limit, or outside budget,
3. active evaluation window,
4. last controller action or no-change outcome.

### 2. Outcome Frontier

The primary comparison surface should be:

1. suspicious non-human cost,
2. likely-human friction,
3. beneficial or verified non-human posture.

This should prefer explicit budget consumption and prior-window comparison over a single blended "score."

### 3. Change Judgment

Monitoring should show:

1. last recommendation,
2. whether a canary was applied,
3. watch-window result,
4. rollback or retain outcome,
5. refusal or blocker reasons when no action was taken.

### 4. Category Breakdown

Monitoring should then show the non-human taxonomy by row:

1. category,
2. operator posture,
3. observed share,
4. leakage or origin reach,
5. cost proxy,
6. friction spillover,
7. adversary-sim effectiveness where covered,
8. evidence quality.

### 5. Trust And Actionability

Monitoring should end the main summary with the reasons the loop can or cannot act confidently:

1. tuning eligible or blocked,
2. category coverage status,
3. protected-evidence status,
4. verified-identity guardrail state,
5. freshness and exactness caveats.

## Why This Is Better

1. It matches the actual architecture: Shuma already has a controller loop and benchmark contract.
2. It keeps manual operator tuning and posture editing in `Tuning`, where they belong.
3. It makes Monitoring a place where the loop proves itself instead of narrating subsystem internals.
4. It makes Diagnostics more useful for contributor and incident-debug work instead of leaving it as a vaguely old Monitoring clone.

## Consequence For `MON-OVERHAUL-1`

`MON-OVERHAUL-1` should be split into at least two slices:

1. a first information-architecture slice that makes Monitoring unmistakably loop-accountability-first and Diagnostics explicitly diagnostics-first,
2. a second data-projection slice that wires the benchmark, snapshot, controller, and category surfaces into that new layout.
