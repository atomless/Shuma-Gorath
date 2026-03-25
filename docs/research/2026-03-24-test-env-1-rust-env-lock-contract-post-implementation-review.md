Date: 2026-03-24

# TEST-ENV-1 Rust Env-Lock Contract Post-Implementation Review

## What landed

`TEST-ENV-1` is now enforced by an explicit static contract lane instead of relying on manual re-audits.

The new lane is:

- `make test-rust-env-lock-contract`

It is implemented by:

- `scripts/tests/test_rust_env_lock_contract.py`

and now also runs inside the canonical:

- `make test`

umbrella.

## What the contract proves

The static contract now scans Rust tests under:

- `src/**/*.rs`
- `tests/**/*.rs`

and fails if a test mutates process environment through:

- `std::env::set_var`
- `std::env::remove_var`
- `env::set_var`
- `env::remove_var`
- `clear_env(...)`
- `clear_gateway_env(...)`
- `set_gateway_env_baseline(...)`

without also using one of the approved isolation patterns:

- `crate::test_support::lock_env()`
- local `lock_env()`
- `with_runtime_env(...)`
- `with_runtime_env_overrides(...)`

## Why this mattered

The earlier `TEST-ENV-1` follow-on closed the specifically named shadow-mode offender, but the backlog item remained open because there was still no repo-wide guard preventing future drift.

This slice closes that broader gap by making the discipline executable:

1. Rust env-mutating tests must hold the env mutex or use an approved wrapper that does.
2. The rule is now part of a named Makefile contract target.
3. The canonical full suite now runs that contract automatically.

## Audit outcome

The dry-run audit found no new env-mutation offenders once helper-backed and wrapper-backed patterns were accounted for.

That means this tranche is about codifying and preserving the discipline, not cleaning up another broken file.

## Proof

Focused verification:

- `make test-rust-env-lock-contract`
- `make test-shadow-mode`
- `git diff --check`

## Outcome

`TEST-ENV-1` is now closed.

The repo has an explicit, repeatable guard for Rust test env isolation, and the previously fixed shadow-mode case remains covered by the focused shadow-mode gate.
