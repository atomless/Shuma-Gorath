# Compact Event Telemetry Live Evidence

Date: 2026-03-14

## Scope

This note records the live proof for `TEL-EVT-1-5` after the compact event schema landed.

The goal was to prove all of the following at once:

1. compact persisted challenge rows are materially smaller on a real deployment,
2. hot-read bootstrap and delta payloads stay within the established `TEL-HOT` live budget envelope,
3. the compact schema is live on both shared-host and Fermyon edge targets,
4. the retained-byte footprint is measured honestly enough to drive the next retention/lifecycle reassessment.

## Evidence Receipts

- Shared-host: [`../../.spin/telemetry_shared_host_evidence.json`](../../.spin/telemetry_shared_host_evidence.json)
- Fermyon edge: [`../../.spin/telemetry_fermyon_edge_evidence.json`](../../.spin/telemetry_fermyon_edge_evidence.json)

Collection commands:

- `make test-telemetry-hot-read-evidence`
- `make test-telemetry-hot-read-live-evidence`

## Shared-Host Results

Environment:

- Remote: `dummy-static-site-fresh`
- Public base URL: `https://shuma.jamestindall.org`

Budgets:

- bootstrap budget: `750 ms`
- delta budget: `250 ms`

Observed:

- `/admin/monitoring?bootstrap=1...`
  - `94.67 ms`
  - `21309 B`
- `/admin/monitoring?bootstrap=1...` with gzip
  - `70.10 ms`
  - `3567 B`
  - `83.26%` compression reduction
- `/admin/monitoring/delta`
  - `62.32 ms`
  - `9514 B`

Result:

- bootstrap within budget: `true`
- delta within budget: `true`

Persisted-row evidence:

- current challenge-heavy recent-event sample: `27` rows total, `22` `js_verification`, `19` compact `js_verification`
- newest retained-row sample is now entirely compact and uniform:
  - `10/10` sampled recent rows are compact `js_verification` rows
  - each sampled row is `146 B`
- retained legacy `js_verification` rows from the earlier live sample were `259-260 B`
- reduction for the compact challenge row versus the retained legacy row shape remains about `44%`

Hot-read document evidence:

- bootstrap hot-read document: `15875 B`
- recent-events-tail hot-read document: `10676 B`

Retained-byte pressure on the live shared host:

- raw eventlog values: `5039 B` across `7` active event windows (`719.86 B` per active window)
- eventlog retention bucket indexes: `2822 B`
- monitoring retention bucket indexes: `1785 B`
- retention catalogs: `329 B`
- hot-read documents: `26551 B`
- total measured telemetry value footprint in the default store: `36526 B`

Interpretation:

1. the compact event contract clearly reduces challenge-row storage weight on the live host,
2. the current low-volume shared-host footprint is now dominated by hot-read documents and retention metadata rather than raw event rows,
3. the retention rebaseline must therefore evaluate tier pressure as a whole system problem, not just raw row byte savings.

## Fermyon Edge Results

Environment:

- App URL: `https://79b823de-37b6-4a85-b3cc-16a40738c5a7.fwf.app`
- Account: `atomless`

Budgets:

- bootstrap budget: `2000 ms`
- delta budget: `750 ms`

Observed:

- `/admin/monitoring?bootstrap=1...`
  - `165.81 ms`
  - `27347 B`
- `/admin/monitoring/delta`
  - `672.93 ms`
  - `15987 B`

Result:

- bootstrap within budget: `true`
- delta within budget: `true`

Direct live compact-row proof:

- the current Fermyon recent-event sample is challenge-heavy and compact:
  - `40` rows total
  - `29` challenge rows
  - `23` `js_verification` rows
  - `23/23` `js_verification` rows in the sample use the compact shape
- a fresh Fermyon `js_verification` recent-event row is now:

```json
{
  "event": "Challenge",
  "ip": "88.215.1.0",
  "outcome_code": "required",
  "reason": "js_verification",
  "taxonomy": {
    "level": "L4_VERIFY_JS"
  },
  "ts": 1773483872
}
```

That row shape confirms the edge deployment is serving the compact contract directly:

1. no blended verbose `outcome` narrative,
2. no `is_simulation: false`,
3. no redundant `taxonomy.action`,
4. no redundant `taxonomy.detection`,
5. no fixed `taxonomy.signals` bundle for `js_verification`.

## Caveat

The Fermyon live-evidence receipt still reports deploy receipt head `5d30b0be9abcc281f550e1cf03d6e09f854dbdf2`.

That receipt lag remains a deploy-helper caveat, not a telemetry-proof failure:

1. live bootstrap and delta requests are budget-green,
2. direct live recent-event queries show the compact row shape on edge,
3. the stale receipt head is caused by the post-publish adversary-sim smoke failure that still prevents the helper from rewriting the final receipt.

## Conclusion

`TEL-EVT-1-5` is complete.

The compact schema now has live proof that:

1. challenge-heavy persisted rows are materially smaller,
2. hot-read payload budgets remain green on both shared-host and Fermyon,
3. analysis/dashboard usability is preserved through the compact raw shape,
4. the next retention tranche should focus on measured tier balance, not assume raw-row compaction alone justifies longer raw retention.

## Retention Decision

The current retention defaults stay unchanged for now:

1. high-risk raw event evidence remains effectively capped at `72h`,
2. monitoring summaries remain at `168h`,
3. monitoring rollups remain at `720h`.

Reason:

1. compact rows are now clearly and materially smaller in live challenge-heavy samples,
2. but the retained-byte footprint on the shared-host sample is still dominated by hot-read documents and retention metadata rather than raw event rows,
3. so extending raw retention now would trade away the existing privacy/security posture without evidence that raw rows are still the dominant cost driver,
4. and shrinking the summary or rollup windows is not justified because live bootstrap and delta budgets remain comfortably green on both targets.

## Verification

- `make test-telemetry-hot-read-evidence`
- `make test-telemetry-hot-read-live-evidence`
