# Dashboard Scrapling Evidence Gap Review

Date: 2026-03-25
Status: Active

Related context:

- [`../plans/2026-03-25-dashboard-scrapling-evidence-surfacing-plan.md`](../plans/2026-03-25-dashboard-scrapling-evidence-surfacing-plan.md)
- [`../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`](../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md)
- [`../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`../dashboard-tabs/red-team.md`](../dashboard-tabs/red-team.md)
- [`../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte`](../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)

# Objective

Determine whether the current dashboard gives a human operator enough evidence to verify and understand Scrapling adversarial traffic and attacks.

# Findings

## 1. Backend evidence is materially richer than the dashboard projection

Recent sim-run summaries already carry:

- `observed_fulfillment_modes`
- `observed_category_ids`
- `owned_surface_coverage`

That contract is explicit in:

- [`src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`src/admin/api.rs`](../../src/admin/api.rs)

The owned-surface coverage summary itself is rich enough to support operator-facing proof:

- overall coverage status
- required, satisfied, and blocking surface ids
- per-surface receipts with success contract, coverage status, attempt count, sample method, sample path, and sample response status

## 2. Red Team currently proves lifecycle truth more than attack truth

The current Red Team tab is valuable, but it mostly answers:

- is the sim on or off?
- which lane is active?
- are counters direct or recovered?
- were there recent runs?

It does not yet clearly answer:

- which Scrapling personas actually ran?
- which non-human categories were observed?
- whether the owned request-native defense surfaces were covered
- which specific attack surfaces were attempted and what happened

## 3. The dashboard view-model path drops the strongest evidence

The most important dashboard projection gap is not the backend payload. It is the view-model and component layer:

- recent sim-run summaries are passed through mostly raw
- `AdversaryRunPanel` only shows compact run history fields
- `deriveAdversaryRunRowsFromSummaries()` does not preserve modes, categories, or owned-surface coverage

This means the data exists but is not being shaped into operator-facing evidence.

## 4. Game Loop should corroborate, not become the forensic surface

Game Loop should remain the accountability surface for the self-improving loop, not turn into Red Team forensics.

It should show a compact corroborating signal such as:

- latest Scrapling evidence coverage status
- latest observed categories
- required vs satisfied owned-surface counts

But detailed attack receipts belong in Red Team.

# Conclusion

The concern is valid. Shuma currently has meaningful Scrapling proof in backend and hot-read contracts, but the dashboard under-surfaces it. The clean fix is:

1. make `Red Team` the primary operator surface for receipt-backed Scrapling attack evidence,
2. add only a compact corroborating roll-up to `Game Loop`,
3. reuse existing dashboard section, table, and metric patterns rather than inventing a new UI language.
