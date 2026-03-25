Date: 2026-03-24

# TEST-ENV-1 Shadow-Mode Lock Env Post-Implementation Review

## What landed

The remaining explicitly called-out shadow-mode env-mutation test now uses the repo's canonical env-isolation guard.

In `src/runtime/shadow_mode/tests.rs`:

- `shadow_passthrough_requires_native_forwarding_capability_on_host_runtime` now holds `crate::test_support::lock_env()`,
- and it restores both `SHUMA_GATEWAY_NATIVE_TEST_MODE` and `SHUMA_GATEWAY_UPSTREAM_ORIGIN` to their prior values after the assertion path completes.

## Why this follow-on mattered

The active backlog for `TEST-ENV-1` explicitly called out this file as the known remaining offender.

That meant the fix here needed to be exact:

1. use the established `lock_env()` discipline,
2. restore all mutated vars, not only one of them,
3. and verify through the existing focused shadow-mode proof path.

## Proof

Focused verification:

- `make test-shadow-mode`
- `git diff --check`

## Outcome

The shadow-mode offender is fixed.

`TEST-ENV-1` remains open as a broader repo-wide env-isolation discipline item until the rest of the env-mutation surface is re-audited explicitly, but this named offender is now closed.
