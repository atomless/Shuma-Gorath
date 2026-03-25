Date: 2026-03-25
Status: Proposed

Related context:

- [`2026-03-23-testing-surface-audit.md`](2026-03-23-testing-surface-audit.md)
- [`../plans/2026-03-23-testing-surface-rationalization-plan.md`](../plans/2026-03-23-testing-surface-rationalization-plan.md)
- [`../plans/2026-03-25-testing-suite-structure-and-mainline-friction-plan.md`](../plans/2026-03-25-testing-suite-structure-and-mainline-friction-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# TEST-HYGIENE-2 Worktree-Clean Readiness Review

## Question

What is the smallest truthful slice that stops routine verification from rewriting tracked adversarial and SIM2 artifacts?

## Conclusion

The next slice should stay narrow:

1. keep tracked JSON under `scripts/tests/adversarial/` as read-only contracts and fixture inputs,
2. move generated adversarial and SIM2 output paths used by `make` targets under `.spin/adversarial/`,
3. and update the focused Makefile contract tests and docs so the new artifact location is explicit.

## Why this is the right next step

The earlier testing audit already identified the main problem:

1. `make test` and several focused adversarial/SIM2 targets still write generated reports into tracked paths under `scripts/tests/adversarial/`,
2. those writes weaken `git diff` as a proof surface,
3. and they create avoidable contributor noise even when behavior is correct.

The current game-loop and active mainline proof path is now landed, so reducing that verification churn is a high-leverage follow-on.

## Existing seams make this a focused tranche

The fix can stay mostly in `Makefile` and docs:

1. the writing behavior is primarily introduced by `make` targets, not by immutable contract fixtures,
2. the repo already treats `.spin/` as disposable local runtime and test state,
3. and the scripts already accept explicit output paths for most generated artifacts.

So the tranche can preserve script defaults and tracked fixtures while making routine `make` workflows worktree-clean.

## Decision

Treat `TEST-HYGIENE-2` as the next active unblocked tranche.

Scope it to:

1. generated adversarial and SIM2 report outputs,
2. focused `make` target wiring proof,
3. and operator/testing docs that describe where those artifacts now land.
