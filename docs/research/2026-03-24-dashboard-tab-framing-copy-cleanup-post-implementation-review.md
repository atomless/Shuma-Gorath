# 2026-03-24 Dashboard tab framing-copy cleanup post-implementation review

## Scope

This slice removed superfluous framing titles and explanatory copy from the `Traffic`, `Game Loop`, and `Diagnostics` tabs so those tabs reuse the existing dashboard section language without redundant hero-style intros.

## Delivered

- Removed the top framing panes from:
  - `Traffic`
  - `Game Loop`
  - `Diagnostics`
- Removed the `Traffic Overview` section title and subtitle while preserving the overview cards and charts.
- Removed the `Traffic Telemetry Health` heading and subtitle, and moved the freshness/read-path strip to the bottom of the `Traffic` tab.
- Removed the `Current Status` section heading and its explanatory sentence from `Game Loop` while preserving the status cards and runtime rows.
- Made `SectionBlock` suppress empty headings so shared section primitives can be reused without inventing a one-off variant.

## Why this was the right change

- The tab tops had drifted into bespoke framing language even though the rest of the dashboard already has a stable section pattern.
- The removed copy did not add decision-support value; it mostly repeated what the tab name and the visible content already communicated.
- Moving the traffic freshness/read-path strip to the bottom keeps the live traffic picture primary while still preserving lightweight telemetry-health proof.

## Verification

- `make test-dashboard-tab-information-architecture`
- `make test-dashboard-game-loop-accountability`
- `make test-dashboard-traffic-pane`
- `git diff --check`

## Follow-up

- No additional follow-up is required for this slice.
