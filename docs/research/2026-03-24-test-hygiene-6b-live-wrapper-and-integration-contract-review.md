# TEST-HYGIENE-6B Review

Date: 2026-03-24

## Goal

Finish the non-dashboard portion of `TEST-HYGIENE-6B` by removing the remaining shell-wrapper archaeology from feature-behavior proof lanes, while keeping the useful contract checks that still matter.

## Current State

The repo already has one good explicit wrapper-contract lane:

- [`scripts/tests/test_oversight_supervisor.py`](../../scripts/tests/test_oversight_supervisor.py)

That file is honest about what it proves: shell-wrapper contract shape for `scripts/run_with_oversight_supervisor.sh`.

Two remaining seams still need cleanup:

1. [`scripts/tests/test_live_feedback_loop_remote.py`](../../scripts/tests/test_live_feedback_loop_remote.py)
   - mixes true feature-behavior proof for the live feedback-loop verifier with remote service wrapper and process-tree contract checks.
   - the wrapper checks are useful, but they are contract proof, not loop-behavior proof.

2. [`scripts/tests/test_integration_cleanup.py`](../../scripts/tests/test_integration_cleanup.py)
   - is already an honest shell-cleanup contract test in spirit,
   - but its `make` surface is still named [`test-integration-script-unit`](../../Makefile), which is less explicit than it should be for retained source-shape proof.

## Recommended Shape

1. Split the live feedback-loop local verifier tests into:
   - behavior proof: periodic trigger, post-sim linkage, completion-truth failure cases,
   - contract proof: remote service wrapper/process-tree requirements.

2. Keep the live operational proof itself unchanged:
   - [`make test-live-feedback-loop-remote`](../../Makefile)
   - the live tool should still verify the wrapper contract on the real host.

3. Make retained shell-shape cleanup proof explicit in the `make` surface:
   - replace `test-integration-script-unit` with a name that says `cleanup contract`.

## Why This Is The Right Slice

- It removes the most obvious remaining feature/contract mixing without turning `TEST-HYGIENE-6B` into a larger test-system redesign.
- It preserves the valuable wrapper and cleanup guards instead of deleting them.
- It improves target truthfulness without reopening the broader `TEST-HYGIENE-6C` selector-microtest lane.

## Expected Verification

- focused live-feedback-loop behavior target
- focused live-feedback-loop contract target
- focused integration cleanup contract target
- `git diff --check`
