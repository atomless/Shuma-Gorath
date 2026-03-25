Date: 2026-03-24

# Build Hygiene Runtime Env Dead-Code Warning Post-Implementation Review

## What landed

`src/config/runtime_env.rs::spin_variable_name` is no longer compiled into native non-test builds where nothing can call it.

The helper is now gated to:

- test builds,
- and `wasm32` builds where Spin variable lookup actually exists.

## Why this follow-on mattered

Focused verification paths for the first working game-loop proof were still emitting the same stale dead-code warning on every Rust compile:

- `warning: function spin_variable_name is never used`

That warning had already been called out repeatedly as part of `BUILD-HYGIENE-1`, and it was diluting the signal from the canonical focused proof paths.

The fix did not need a new abstraction or a suppression.

The function simply had the wrong compile surface.

## Proof

Focused verification:

- `make test-rsi-game-mainline`
- `git diff --check`

The same target that previously emitted the warning now compiles cleanly.

## Outcome

This removes one real native-test warning source from the repo-wide build-hygiene backlog while keeping the helper available exactly where it is needed.

`BUILD-HYGIENE-1` remains open for any broader warning cleanup still required elsewhere, but the `runtime_env.rs::spin_variable_name` portion is now closed.
