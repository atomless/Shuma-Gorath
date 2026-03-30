# Verified Identity Health Status Ownership Review

Date: 2026-03-30

## Question

Where should the read-only `Verified Identity Health` summary live in the dashboard now that `Verification` is being kept focused on editable verification controls?

## Current state

- `VerificationTab.svelte` currently mixes two concerns:
  - editable verified-identity mechanics (`verified_identity.*` config),
  - and a read-only operator-snapshot summary (`Verified Identity Health`).
- `StatusTab.svelte` already owns read-only operator and dashboard health surfaces such as:
  - dashboard connectivity,
  - telemetry delivery freshness,
  - retention health,
  - runtime performance telemetry.
- `refreshVerificationTab` currently fetches `operatorSnapshot` only to support the read-only health summary.
- `refreshStatusTab` does not currently fetch `operatorSnapshot`.

## Why the current ownership is wrong

`Verified Identity Health` is not a verification control. It is a bounded runtime health summary derived from `operatorSnapshot`, alongside other read-only operational summaries. Keeping it in `Verification` makes the tab mix:

1. editable control-plane posture, and
2. read-only operational health.

That is the same sort of ownership mixing we have been cleaning out elsewhere in the dashboard.

## Recommended ownership

- `Verification`
  - editable verification-source and verification-mechanic controls only.
- `Status`
  - read-only `Verified Identity Health` summary at the bottom of the tab.

## Data-path implication

To make that ownership truthful:

- `StatusTab` must receive `operatorSnapshot`,
- `refreshStatusTab` must fetch/update `operatorSnapshot`,
- `refreshVerificationTab` must stop fetching `operatorSnapshot`,
- route wiring and tests must follow the new ownership exactly.

## Acceptance criteria

1. `Verification` no longer renders `Verified Identity Health`.
2. `Status` renders the existing summary at the bottom of the tab.
3. `Status` fetches `operatorSnapshot` through its own refresh path.
4. `Verification` no longer depends on `operatorSnapshot`.
5. Docs, tests, and focused `make` targets describe the new ownership truthfully.
