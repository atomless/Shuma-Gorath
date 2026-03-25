# TEST-HYGIENE-6B Supervisor Wrapper Contract Reclassification Post-Implementation Review

Date: 2026-03-24
Status: Partially closed

Related context:

- [`2026-03-23-testing-surface-audit.md`](2026-03-23-testing-surface-audit.md)
- [`../plans/2026-03-23-testing-surface-rationalization-plan.md`](../plans/2026-03-23-testing-surface-rationalization-plan.md)
- [`../../Makefile`](../../Makefile)
- [`../../scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py)
- [`../testing.md`](../testing.md)

## Scope Reviewed

This slice closed the first concrete `TEST-HYGIENE-6B` target:

1. stop hiding supervisor wrapper source archaeology inside feature-behavior targets,
2. move the retained shell-shape proof into an explicit contract lane,
3. keep the focused feature gates proving feature behavior rather than wrapper file composition.

## What Landed

1. Added a dedicated contract target, `make test-supervisor-wrapper-contracts`, that now owns:
   - [`scripts/tests/test_adversary_sim_supervisor.py`](../../scripts/tests/test_adversary_sim_supervisor.py)
   - [`scripts/tests/test_oversight_supervisor.py`](../../scripts/tests/test_oversight_supervisor.py)
2. Removed those wrapper archaeology tests from feature-behavior targets that previously carried them:
   - `make test-oversight-post-sim-trigger`
   - `make test-adversary-sim-scrapling-worker`
3. Tightened the Makefile selector contract test in:
   - [`../../scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py)
   so it now proves:
   - the new contract lane owns the wrapper archaeology,
   - and the feature gates no longer quietly depend on it.
4. Updated [`../testing.md`](../testing.md) so the documented blast radius matches the real target ownership.

## Review Result

This is the correct first cut for `TEST-HYGIENE-6B`:

1. wrapper source-shape proof still exists where it is genuinely useful,
2. the proof now lives in an explicitly named wiring or contract lane,
3. and the behavior-oriented targets no longer overclaim by including hidden shell archaeology.

## Residual Follow-On

`TEST-HYGIENE-6B` remains open for the rest of the tranche:

1. integration cleanup shell-shape checks still need the same audit and reclassification treatment,
2. and any other remaining shell-wrapper archaeology outside explicit contract lanes should be moved the same way.

## Verification

- `make test-supervisor-wrapper-contracts`
- `make test-oversight-post-sim-trigger`
- `make test-adversary-sim-scrapling-worker`
- `git diff --check`
