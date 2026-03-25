# MZ-T4 Maze Canonical Verification Wiring Review

Date: 2026-03-24
Status: Accepted

## Why this tranche exists

`MZ-T1`, `MZ-T2`, and `MZ-T3` added the missing maze proof surfaces:

- live opaque traversal,
- live browser/session traversal,
- native burst/concurrency state proof.

Those gates currently exist only as focused side targets in [`Makefile`](../../Makefile). The canonical umbrella path still runs only the older maze asymmetry benchmark during `make test`, and the main pre-merge CI workflow relies on that umbrella target:

- [`Makefile`](../../Makefile) `test`
- [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)

That means the repo can now claim stronger maze proof than the canonical merge gate actually enforces.

## Existing truth we should reuse

- [`Makefile`](../../Makefile) already has the focused maze targets we need:
  - `test-maze-benchmark`
  - `test-maze-live-traversal-contract`
  - `test-maze-live-browser-contract`
  - `test-maze-state-concurrency-contract`
- [`docs/plans/2026-02-25-maze-carry-forward-plan.md`](../plans/2026-02-25-maze-carry-forward-plan.md) already says `MZ-T4` is specifically about wiring the new coverage into canonical Makefile and CI gates.
- [`scripts/tests/playwright_runtime.py`](../../scripts/tests/playwright_runtime.py) and the existing dashboard/browser paths already provide a repo-local Chromium bootstrap path, so the live browser gate can remain in canonical verification without inventing a separate browser install story.

## Review conclusion

The cleanest `MZ-T4` slice is:

1. introduce one truthful canonical maze verification aggregate target,
2. route `make test` through that target instead of only the legacy benchmark gate,
3. keep CI truth aligned by ensuring the canonical pre-merge and release-oriented workflows exercise the same gate,
4. add a focused wiring proof so later drift fails fast.

## Guardrails

- Do not duplicate the maze command list in multiple Makefile or workflow locations if one canonical target can own it.
- Keep the canonical target truthful about scope: benchmark + live traversal + live browser + native state concurrency. It is not a remote or long-horizon soak harness.
- Do not widen this tranche into new maze behavior work; it is verification-routing closure only.
