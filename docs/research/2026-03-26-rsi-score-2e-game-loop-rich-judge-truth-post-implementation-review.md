# RSI-SCORE-2E Game Loop Rich Judge Truth Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js)
- [`../../e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js)
- [`../../Makefile`](../../Makefile)
- [`../../docs/dashboard-tabs/game-loop.md`](../../docs/dashboard-tabs/game-loop.md)
- [`../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Scope Reviewed

This closeout reviewed `RSI-SCORE-2E`: project the richer judge truth in `Game Loop` so the tab no longer reads like one blended attacker-success score and instead makes guardrails, exploit progress, evidence quality, urgency, breach loci, and move or escalation state visible at a glance.

# What Landed

1. The dashboard API adapters now preserve:
   - benchmark urgency,
   - evidence-quality details,
   - named breach loci,
   - exploit loci on benchmark families,
   - and archive-level homeostasis break and restart-baseline lineage.
2. `Game Loop` now surfaces separate top-level cards for:
   - exploit progress,
   - evidence quality,
   - urgency,
   - move outcome,
   alongside the existing loop status cards.
3. The `Recent Loop Progress` section now exposes homeostasis-break state, named break reasons, and restart-baseline lineage instead of leaving them implicit.
4. `What The Loop Decided` now distinguishes:
   - judge state,
   - evidence quality,
   - diagnosis confidence,
   - move or escalation outcome,
   - config-ring status,
   - code-evolution status,
   - and named breach loci.
5. `Where The Pressure Sits` now includes a first-class `Exploit Progress` panel so category posture achievement no longer has to stand in for terrain-local attacker advance.
6. The focused `make test-dashboard-game-loop-accountability` target now covers the richer source contract, adapter path, and rendered smoke for these judge planes.

# Acceptance Review

`RSI-SCORE-2E` required the Game Loop tab to stop reading like one blended attacker-success score, expose guardrails/exploit progress/evidence quality/urgency/move state distinctly, surface named breach loci plus homeostasis-break reasons, and preserve the bounded corroboration role of compact Scrapling evidence.

Those criteria are now satisfied:

1. operators can now distinguish guardrails, exploit progress, evidence quality, urgency, and move outcome at a glance from separate cards and rows;
2. named breach loci and selected repair surfaces are rendered explicitly instead of being left implicit in aggregate pressure language;
3. the Game Loop tab keeps category posture achievement separate from the compact Scrapling surface-contract corroboration and from exploit-progress scoring;
4. rendered proof now covers the richer judge wording and localized repair state, not only payload adaptation;
5. and the repo has the required focused proof through:
   - `make test-dashboard-game-loop-accountability`

The key behavioral correction is this:

the Game Loop tab no longer asks operators to mentally reconstruct whether the loop is blocked by weak evidence, urgent exploit regression, config exhaustion, or a real bounded tuning opportunity.

It now names those states directly.

# Shortfalls Found

This slice does not itself prove the strict `human_only_private` loop is operational over repeated retained improvement cycles.

The following planned work remains open:

1. `RSI-GAME-HO-1A` strict-loop runtime proof against `human_only_private`;
2. `RSI-GAME-HO-1B` repeated retained-versus-rolled-back cycle proof;
3. `RSI-GAME-HO-1C` explicit unlock-condition proof for measured retained improvement.

So this tranche makes the operator-facing scoring planes trustworthy, but it does not yet prove the strict loop is achieving repeated improvement.

# Verification

- `make test-dashboard-game-loop-accountability`
- `git diff --check`
