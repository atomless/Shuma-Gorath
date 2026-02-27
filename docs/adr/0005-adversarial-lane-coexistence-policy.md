# ADR 0005: Deterministic and Containerized Adversary Lane Coexistence

- Status: Proposed
- Date: 2026-02-27
- Owners: @jamestindall
- Related:
  - [`docs/testing.md`](../testing.md)
  - [`docs/adversarial-operator-guide.md`](../adversarial-operator-guide.md)
  - [`docs/adr/adversarial-lane-parity-signoff-checklist.md`](adversarial-lane-parity-signoff-checklist.md)
  - `todos/todo.md` (`SIM-V2-15`)

## Context

Shuma now has two black-box adversarial execution families:

1. Deterministic manifest-driven lanes (`test-adversarial-smoke`, `test-adversarial-abuse`, `test-adversarial-akamai`, `test-adversarial-coverage`).
2. Containerized black-box lanes (`test-adversarial-container-isolation`, `test-adversarial-container-blackbox`).

Deterministic lanes are currently the only reproducible merge/release oracle. Containerized runs provide complementary realism and isolation validation, but are more expensive and currently less deterministic for protected-lane enforcement.

We need an explicit policy that avoids accidental replacement drift.

## Decision

Adopt an explicit coexistence policy:

1. Deterministic lanes remain canonical mandatory merge/release gates until explicit parity sign-off is approved.
2. Containerized black-box lanes remain complementary and run in scheduled/manual soak lanes in this phase.
3. Frontier discovery stays advisory/adaptive; deterministic replay confirmation remains the blocking regression oracle.
4. No deterministic-lane demotion/removal may occur without:
   - completed parity-signoff checklist,
   - explicit owner approval,
   - ADR/TODO update documenting the migration contract.

Required parity-signoff criteria before any replacement discussion:

1. Category parity: `100%` of mandatory defense categories/gates covered by the candidate lane set.
2. Reproducibility parity: promoted frontier regressions deterministic replay confirmation rate `>= 95%` over rolling window of 20 findings.
3. Stability parity: 10 consecutive protected-lane runs without flaky nondeterministic failures.
4. Cost parity: protected-lane runtime increase `<= 20%` versus deterministic-only baseline (unless explicitly approved).

## Alternatives Considered

1. Replace deterministic lanes immediately with container lanes.
2. Run container lanes as mandatory protected-lane blockers before parity evidence.
3. Keep policy implicit in code comments and workflow YAML only.

## Consequences

### Positive

- Preserves deterministic release confidence while gaining complementary realism coverage.
- Prevents accidental gate replacement without measurable evidence.
- Keeps CI runtime/cost predictable in protected lanes.

### Negative / Trade-offs

- Two lane families must be maintained during coexistence.
- Additional documentation/process overhead for parity sign-off.

## Security Impact

- Maintains reproducible blocking gates for confirmed regressions.
- Preserves black-box-only posture across mandatory and complementary lanes.
- Reduces risk of releasing on stochastic-only findings.

## Human Friction Impact

- No additional user-facing friction; this is CI/release governance.
- Operator burden increases slightly for parity-signoff evidence collection.

## Adversary Cost Placement

- Frontier/container lanes continue to discover adaptive attack candidates.
- Deterministic lane ensures confirmed regressions become hard release blockers.

## Operational Impact

- Protected lanes: deterministic gates + advisory frontier attempt and threshold policy.
- Scheduled/manual lanes: deep soak plus container isolation/black-box runs.
- Any lane migration requires explicit signed-off policy update.

## Resource Impact

- Protected-lane runtime remains bounded by deterministic gates.
- Container lane cost is isolated to scheduled/manual workflows during coexistence.

## Verification

- CI/release workflows must continue enforcing deterministic blockers.
- Make target help text must mark container lanes as complementary scheduled/manual lanes.
- Operator/testing docs must include capability mapping and parity-signoff process.

## Follow-ups

1. Keep parity-signoff checklist current as metrics/lanes evolve.
2. Revisit this ADR only when sign-off evidence is complete and owner-approved.
