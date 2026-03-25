# TEST-ENV-1 Rust Env-Isolation Post-Implementation Review

Date: 2026-03-24

## What landed

`TEST-ENV-1` now has both enforcement and the surgical code fix it was carrying:

1. [`scripts/tests/test_rust_env_isolation_contract.py`](../../scripts/tests/test_rust_env_isolation_contract.py)
   scans Rust test functions and fails when a test mutates process env without
   acquiring `lock_env()` before the first mutation.
2. [`Makefile`](../../Makefile) now exposes that proof as:
   - `make test-env-isolation-contract`
3. The known remaining offender in
   [`src/runtime/shadow_mode/tests.rs`](../../src/runtime/shadow_mode/tests.rs)
   now:
   - acquires `lock_env()`
   - restores both mutated env vars exactly

## Why this tranche mattered

The repo already treated `lock_env()` as the canonical guard for tests that
mutate process env, but the rule was still partly social. That left one known
test bypassing the mutex and left future drift to human vigilance.

This tranche converts that into an executable contract and closes the known gap.

## Verification

The tranche was verified with:

- `make test-env-isolation-contract`
- `make test-shadow-mode`
- `git diff --check`

## Assessment

The delivered shape is appropriately strict and local:

1. repo-wide enforcement now exists in a focused contract test rather than as a
   comment or tribal rule,
2. the touched shadow-mode test now participates in the same discipline as the
   rest of the repo,
3. the exact env restoration is cleaner than the pre-existing partial cleanup.

## Follow-on

Future Rust tests that intentionally mutate process env should satisfy this
contract directly inside the test body by taking `lock_env()` before the first
mutation. If a new safe pattern ever needs to exist, the contract test should be
updated deliberately rather than worked around piecemeal.
