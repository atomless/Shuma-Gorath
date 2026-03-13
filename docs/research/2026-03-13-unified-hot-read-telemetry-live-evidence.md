## Unified Hot-Read Telemetry Live Evidence

Date: 2026-03-13

## Scope

This note records the live proof that the `TEL-HOT-1` hot-read architecture now serves the same bounded telemetry bootstrap and delta shapes on both:

- shared-host / Linode
- Fermyon / Akamai edge

The goal was not only to make Fermyon fast enough. The goal was to prove one shared KV-backed telemetry architecture can keep host cost low and operator reads fast across both deployment classes without introducing:

- a Fermyon-only telemetry store,
- a SQLite split,
- a new external database requirement,
- new whole-keyspace scans in the hot path,
- or correctness dependence on non-atomic shared KV mutation.

## Evidence Receipts

- Shared-host: [/.spin/telemetry_shared_host_evidence.json](/Users/jamestindall/Projects/Shuma-Gorath/.spin/telemetry_shared_host_evidence.json)
- Fermyon edge: [/.spin/telemetry_fermyon_edge_evidence.json](/Users/jamestindall/Projects/Shuma-Gorath/.spin/telemetry_fermyon_edge_evidence.json)

Collection commands:

- `make telemetry-shared-host-evidence`
- `make telemetry-fermyon-edge-evidence`
- `make test-telemetry-hot-read-live-evidence`

## Live Results

### Shared-host / Linode

Environment:

- Remote: `dummy-static-site-fresh`
- Public base URL: `https://shuma.jamestindall.org`

Observed:

- `/admin/monitoring?bootstrap=1...`: `91.28 ms`
- `/admin/monitoring?bootstrap=1...` with gzip: `70.49 ms`
- `/admin/monitoring/delta`: `72.86 ms`
- `/admin/monitoring/stream`: `70.57 ms`

Budgets:

- bootstrap budget: `750 ms`
- delta budget: `250 ms`

Result:

- within budget for bootstrap: `true`
- within budget for delta: `true`

### Fermyon / Akamai edge

Environment:

- App URL: `https://79b823de-37b6-4a85-b3cc-16a40738c5a7.fwf.app`
- Account: `atomless`

Observed:

- `/admin/monitoring?bootstrap=1...`: `161.96 ms`
- `/admin/monitoring/delta`: `152.5 ms`

Budgets:

- bootstrap budget: `2000 ms`
- delta budget: `750 ms`

Result:

- within budget for bootstrap: `true`
- within budget for delta: `true`

## Architectural Assessment

The live result confirms the intended architecture:

1. Fermyon and Linode are both now using the same hot-read document family.
2. The hot operator path no longer needs request-time reconstruction of expensive telemetry summaries.
3. The performance win is coming from read-shape reduction, not a platform-specific shortcut.

The key implementation choices that made this possible were:

1. embedding the summary payload directly in the bounded bootstrap hot-read document,
2. serving the initial delta bootstrap from the bounded recent-events tail and security/privacy summary hot-read documents,
3. preferring the hot-read bootstrap path on edge even when the dashboard requests a larger generic limit,
4. keeping detailed drill-down on the existing bounded raw/bucket paths instead of creating a second storage model.

## Cost and Correctness Conclusions

### No Fermyon-only split required

The current result does not justify:

- a Fermyon-only telemetry subsystem,
- a SQLite-first divergence,
- or an external relational database.

The shared KV-backed hot-read architecture is currently sufficient.

### No secondary memoization justified

The hot-read documents are already fast enough on both targets that an extra in-memory memoization layer is not justified now.

Reason:

- it would add another freshness surface,
- it would complicate correctness under edge instance churn,
- and the current measured latencies are already well inside the stated budgets.

### No cold-tier compression justified

At-rest compression remains unjustified for the hot path.

Reason:

- the dominant problem was request-time read amplification, not stored-value size,
- transport gzip already provides strong payload reduction where it matters,
- and extra compression/decompression would add complexity without a demonstrated host-cost win.

## Remaining Caveat

The live Fermyon evidence was captured against the current deployed app and passed, but the deploy receipt at [/.shuma/fermyon-akamai-edge-deploy.json](/Users/jamestindall/Projects/Shuma-Gorath/.shuma/fermyon-akamai-edge-deploy.json) still shows an older `git_head`.

Reason:

- a later `make deploy-fermyon-akamai-edge` run updated the live app but then failed during a post-deploy adversary-sim smoke step before the helper rewrote the receipt.

This does not invalidate the telemetry evidence itself, because the live evidence commands exercised the deployed edge app directly and the measured endpoints passed. It does mean the Fermyon deploy-helper receipt update path still needs to remain exact when post-deploy smoke fails after publish.

## Verification

- `make test-telemetry-hot-read-bootstrap`
- `make test-telemetry-hot-read-evidence`
- `make test-deploy-fermyon`
- `make telemetry-shared-host-evidence`
- `make telemetry-fermyon-edge-evidence`
- `make test-telemetry-hot-read-live-evidence`
