Date: 2026-03-25
Status: Proposed planning driver

Related context:

- [`2026-03-25-sim-scr-full-1b-browser-session-foundation-review.md`](2026-03-25-sim-scr-full-1b-browser-session-foundation-review.md)
- [`../plans/2026-03-25-sim-scr-full-1b-browser-session-foundation-plan.md`](../plans/2026-03-25-sim-scr-full-1b-browser-session-foundation-plan.md)
- [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)
- [`../../scripts/bootstrap/setup-runtime.sh`](../../scripts/bootstrap/setup-runtime.sh)
- [Scrapling dynamic fetchers docs](https://scrapling.readthedocs.io/en/latest/fetching/dynamic.html)
- [Scrapling stealth fetchers docs](https://scrapling.readthedocs.io/en/latest/fetching/stealthy.html)

# Question

What is the real next blocker after `SIM-SCR-FULL-1B1`, before Shuma can truthfully implement browser-driven Scrapling challenge interactions?

# Findings

## 1. Browser sessions are importable but not executable in the current repo-owned runtime

An empirical local probe against the current `.venv-scrapling` showed:

1. `DynamicSession` imports,
2. `StealthySession` imports,
3. but both fail immediately on `fetch(...)` because the Playwright Chromium executable path does not exist in the local Playwright cache.

The error explicitly instructs the operator to run `playwright install`.

## 2. The runtime already has the Playwright CLI, but the browser binary is not provisioned

The repo-owned Scrapling venv already contains:

1. the `playwright` module,
2. and the `playwright` CLI,

but [`scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh) currently only:

1. creates the venv,
2. installs the Python package,
3. and checks Python imports.

It does not provision browser binaries or fail closed on missing browser executables.

## 3. That makes browser-driven challenge work premature

Until the runtime can actually execute browser sessions, any next slice that claims dynamic or stealth browser-driven challenge interaction would be non-truthful.

So the next honest move is:

1. provision the Playwright browser binary in the repo-owned Scrapling runtime,
2. extend readiness checks so missing browser executables fail closed,
3. and only then move to real browser-driven challenge interactions.

# Result

The next atomic slice should be:

1. `SIM-SCR-FULL-1B2A` browser-runtime provisioning and readiness

and only after that:

2. `SIM-SCR-FULL-1B2B` first browser-driven challenge or bypass interactions.
