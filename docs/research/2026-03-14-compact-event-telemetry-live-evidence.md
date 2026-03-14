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
  - `86.77 ms`
  - `18698 B`
- `/admin/monitoring?bootstrap=1...` with gzip
  - `62.91 ms`
  - `3502 B`
  - `81.27%` compression reduction
- `/admin/monitoring/delta`
  - `64.15 ms`
  - `5483 B`

Result:

- bootstrap within budget: `true`
- delta within budget: `true`

Persisted-row evidence:

- fresh compact `js_verification` row: `146 B`
- legacy `js_verification` rows in the same retained sample: `259-260 B`
- reduction for the fresh compact challenge row: about `44%`
- current mixed retained-row sample average: `187.0 B`

Hot-read document evidence:

- bootstrap hot-read document: `13263 B`
- recent-events-tail hot-read document: `4032 B`

Retained-byte pressure on the live shared host:

- raw eventlog values: `2411 B` across `7` active event windows (`344.43 B` per active window)
- eventlog retention bucket indexes: `1939 B`
- monitoring retention bucket indexes: `1785 B`
- retention catalogs: `329 B`
- hot-read documents: `17295 B`
- total measured telemetry value footprint in the default store: `23759 B`

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
  - `173.46 ms`
  - `30379 B`
- `/admin/monitoring/delta`
  - `150.06 ms`
  - `19019 B`

Result:

- bootstrap within budget: `true`
- delta within budget: `true`

Direct live compact-row proof:

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

## Verification

- `make test-telemetry-hot-read-evidence`
- `make test-telemetry-hot-read-live-evidence`
