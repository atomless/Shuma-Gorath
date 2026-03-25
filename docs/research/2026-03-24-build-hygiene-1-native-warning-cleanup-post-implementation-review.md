# BUILD-HYGIENE-1 Native Warning Cleanup Post-Implementation Review

Date: 2026-03-24

## What landed

`BUILD-HYGIENE-1` now removes the pre-existing native-test warning debt around
[`src/config/runtime_env.rs`](../../src/config/runtime_env.rs) and leaves a focused
Makefile proof for this build shape in [`Makefile`](../../Makefile):

1. `spin_variable_name` is now compiled only where it is actually used:
   - `cfg(test)`
   - `target_arch = "wasm32"`
2. `make test-native-build-warning-hygiene` now forces a fresh native compile and
   treats warnings as errors for the focused host test-build path.

## Root cause

`cargo test` builds a normal native host `lib` before running test harnesses. In
that artifact, `spin_variable_name` had no live caller because its only consumers
were behind `cfg(test)` or `wasm32`, so native test builds emitted a dead-code
warning even though runtime behavior was correct.

The fix was to narrow the helper's compile surface to the same environments that
actually use it, rather than suppressing the warning or widening host-native use
artificially.

## Verification

The tranche was verified with:

- `make test-native-build-warning-hygiene`
- `git diff --check`

## Assessment

The fix is appropriately small and behavior-preserving:

1. runtime env lookup semantics are unchanged,
2. test-only Spin-variable shims still work,
3. wasm32 Spin-variable lookup still works,
4. native host builds no longer normalize away this warning class.

## Follow-on

`BUILD-HYGIENE-1` is complete for the currently queued native warning debt. Any
future compiler warnings in canonical Makefile paths should add focused warning
hygiene proof rather than relying on contributors to notice them manually.
