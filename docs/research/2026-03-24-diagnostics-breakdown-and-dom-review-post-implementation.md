# Diagnostics Breakdown And DOM Review Post-Implementation

Date: 2026-03-24

## What changed

- Restored `Defense Breakdown` as a richer diagnostics surface by combining furniture-state facts with recent trigger, handling-mode, and source context instead of showing only sparse generic summary facts.
- Kept the de-shelled page ownership structure for `Traffic`, `Game Loop`, and `Diagnostics`, and removed the leftover unused `events` class from the shared [`SectionBlock`](../../dashboard/src/lib/components/dashboard/primitives/SectionBlock.svelte) primitive.
- Tightened rendered proof so the diagnostics smoke test explicitly refreshes the tab before asserting the richer breakdown content.

## Why

- The earlier repair fixed the missing-defense regression but over-corrected into a breakdown that read more like generic monitoring than subsystem diagnostics.
- The shared section primitive still carried stale class noise that added nothing to the rendered DOM and made DOM review harder.
- The diagnostics rendered proof was observing the bootstrap shell too early, which hid the real diagnostics content path.

## Result

- `Defense Breakdown` now keeps the full defense roster while surfacing more meaningful per-defense properties again.
- The three tabs continue to avoid redundant section-owner shells and `div.section` markup.
- The focused unit and rendered diagnostics proofs now exercise the intended richer content.

## Verification

- `make test-dashboard-diagnostics-pane`
- `make test-dashboard-traffic-pane`
- `make test-dashboard-game-loop-accountability`
- `make test-dashboard-tab-information-architecture`
- `git diff --check`
