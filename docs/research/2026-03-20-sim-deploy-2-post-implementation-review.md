# SIM-DEPLOY-2 Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-sim-deploy-2-production-operating-envelope-implementation-plan.md`](../plans/2026-03-20-sim-deploy-2-production-operating-envelope-implementation-plan.md)
- [`2026-03-20-sim-deploy-2-readiness-review.md`](./2026-03-20-sim-deploy-2-readiness-review.md)
- [`2026-03-02-adversary-toggle-incident-report-and-lifecycle-invariants.md`](./2026-03-02-adversary-toggle-incident-report-and-lifecycle-invariants.md)
- [`../../docs/adversarial-operator-guide.md`](../../docs/adversarial-operator-guide.md)
- [`../../docs/deployment.md`](../../docs/deployment.md)
- [`../../docs/api.md`](../../docs/api.md)
- [`../../docs/configuration.md`](../../docs/configuration.md)

## Review Goal

Review the full `SIM-DEPLOY-2` tranche against the 2026-03-20 readiness review and implementation plan, confirm that production adversary-sim now has a truthful operating envelope, and record any follow-on shortfall before work moves to the next prerequisite for Scrapling.

## What Was Intended

This tranche was meant to land five things in order:

1. a truthful focused lifecycle verification gate,
2. one backend desired-state authority for adversary-sim lifecycle control,
3. an explicit production-default posture contract,
4. explicit kill-switch and no-impact proof,
5. and deployment/operator evidence guidance that treats production adversary-sim as a normal operating path instead of a gated exception.

## What Landed

1. `make test-adversary-sim-lifecycle` now exercises the real stale-state and ownership cases rather than historical zero-match selectors.
2. Adversary-sim desired state now projects from one backend lifecycle authority after initial seed, removing the separate runtime enabled-writer path.
3. The status payload now makes the production posture explicit through deployment-profile and guardrail fields instead of leaving it implicit in defaults and supervisor code.
4. The runtime-surface gate now proves both sim-tagged defense-surface coverage and live-summary no-impact while adversary-sim runs.
5. Deployment, configuration, API, and operator docs now describe the production receipt explicitly: status while off, explicit ON/OFF control operations, runtime-surface no-impact proof, and cleanup only when intentionally requested.

## Architectural Assessment

### 1. The lifecycle contract is now materially simpler

This was the highest-risk architectural gap from the readiness review, and it is now closed.

The old split between runtime override state and persisted lifecycle state could let:

1. status and control disagree,
2. restart or expiry paths see stale intent,
3. and runtime overlays drift from backend ownership.

That split is now gone from the production lifecycle contract.

### 2. Production posture is explicit without creating a second model

The tranche succeeded in making the production posture visible without inventing new operating semantics.

Operators now see:

1. surface availability by default,
2. generation off until explicit enable,
3. bounded guardrails,
4. and deployment-profile-specific supervisor posture

through the same status contract that already owns lifecycle truth.

### 3. The final documentation gap is now closed

The remaining gap before this slice was not runtime behavior. It was operator proof.

Before this closeout, the code and focused gates already supported production adversary-sim, but the deploy/runbook guidance still left too much of the production receipt implicit.

That gap is now closed with explicit instructions to capture:

1. the off-state posture,
2. the ON operation receipt,
3. the live no-impact proof,
4. and the OFF kill-switch receipt.

## Shortfalls Found During Review

One real shortfall and one follow-on proof gap were found during the live closeout run, and both were corrected before closeout:

1. The first live rerun of `make test-adversary-sim-runtime-surface` showed that `rate` and `geo` monitoring summaries were still origin-blended even after `challenge` and `pow` had become live-only. That violated the no-impact claim.
2. Fixing that exposed a second, smaller harness gap: the runtime-surface gate's `rate` coverage proof had been relying on the now-invalid live summary path instead of forcing an actual rate-limit signal during the runtime-surface profile.

Both were resolved immediately:

1. `rate` and `geo` monitoring counters now follow the same origin-aware live-vs-sim separation model as `challenge` and `pow`.
2. The runtime-surface harness now lowers `rate_limit` in the runtime-surface profile so the live gate proves a real rate-limiter signal instead of inheriting a weaker honeypot-side proxy.

No further architectural blocker remains.

`SIM-DEPLOY-2` can now be treated as complete.

The next prerequisite work should move to the shared-host minimal scope-and-seed gate, because the mature adversary-sim roadmap now treats that plus `SIM-DEPLOY-2` as the entry condition for Scrapling.

## Result

Treat `SIM-DEPLOY-2` as closed.

The production adversary-sim operating envelope now has:

1. truthful focused verification,
2. one backend desired-state authority,
3. explicit production posture and kill-switch semantics,
4. no-impact proof for live summaries while simulation runs,
5. and operator/deployment guidance that treats the lane as first-class production behavior.
