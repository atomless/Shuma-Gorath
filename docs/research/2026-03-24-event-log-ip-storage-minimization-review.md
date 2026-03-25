Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../docs/privacy-gdpr-review.md`](../../docs/privacy-gdpr-review.md)
- [`https://eur-lex.europa.eu/eli/reg/2016/679/oj`](https://eur-lex.europa.eu/eli/reg/2016/679/oj)

# Event-Log IP Storage Minimization Review

## Question

What is the cleanest way to add optional storage-level event-log IP minimization for privacy-sensitive deployments without lying about forensic capability or widening into a general telemetry redesign?

## Current code-grounded state

Today Shuma stores event-log IPs raw at write time:

1. `log_event_with_execution_metadata(...)` persists `EventLogRecord` rows with `entry.ip` unchanged,
2. non-forensic views bucket the IP only on presentation through `pseudonymize_ip_identifier(...)`,
3. forensic mode returns the stored raw IP unchanged,
4. and the security/privacy payload still effectively assumes the event log remains a raw high-risk store with a pseudonymized-default read path.

This means the current privacy posture is a presentation-layer minimization control, not a storage-layer one.

## Why that is insufficient

For privacy-sensitive deployments, that is not enough:

1. event-log rows still retain raw IPs at rest even when operators only ever use pseudonymized admin views,
2. the current config surface offers no explicit deployer choice between raw investigation value and stronger storage minimization,
3. and the operator surface does not distinguish between "forensic break-glass is allowed" and "raw IPs still exist in storage".

That gap matters under data-minimization and privacy-by-design reasoning. The relevant GDPR principles are the data-minimization rule in Article 5(1)(c) and the privacy-by-design/default obligation in Article 25. Pseudonymization is also explicitly recognized in Article 4(5) and Article 32 as a risk-reduction technique, but it is not the same thing as masking or aggregation.

## Recommended contract

Add one env-only event-log IP storage mode with explicit operator truth:

1. `raw`
   - preserve current behavior,
   - raw IP remains available for forensic reads.
2. `masked`
   - persist the same coarse bucket form already used by non-forensic views,
   - retain locality and rough source grouping,
   - but raw forensic recovery is no longer possible for new rows.
3. `pseudonymized`
   - persist a stable keyed pseudonymous token derived from the IP and a trusted secret,
   - preserve per-identity correlation without retaining the raw IP,
   - but geographic/prefix locality is no longer preserved in stored rows.

## Important truthfulness requirement

The chosen mode must not live only as a current global setting. Event-log rows are immutable and the mode may change over time, so each persisted row should carry its write-time IP storage mode. Otherwise Shuma could not truthfully explain historical mixed-mode rows.

## Boundary recommendation

This should be:

1. env-only,
2. deployer/operator controlled,
3. outside the controller-tunable surface,
4. and surfaced as read-only runtime truth in admin/dashboard/runtime inventory.

It is not a tuning knob and must not become optimizer-mutable.

## Minimal implementation seam

The change can stay tightly scoped:

1. add an env-only storage-mode enum and validation in config,
2. apply the mode once in the event-log write path,
3. persist per-record mode metadata,
4. adjust presentation helpers so default and forensic views stay truthful for raw, masked, and pseudonymized rows,
5. surface the current write mode and raw-forensic availability in the security/privacy payload and runtime inventory.

No broader telemetry schema migration or monitoring redesign is required.

## Recommended proof

The tranche should prove:

1. raw mode preserves current storage behavior,
2. masked mode stores only coarse buckets and forensic mode cannot recover raw IPs from new rows,
3. pseudonymized mode stores stable keyed pseudonyms and forensic mode cannot recover raw IPs from new rows,
4. mixed historical rows remain understandable because each row carries its write-time mode,
5. admin/runtime surfaces expose the current deployment mode and its forensic limitation truthfully.

## Result

`SEC-GDPR-3` should land as a focused storage-minimization slice:

1. env-only `SHUMA_EVENT_LOG_IP_STORAGE_MODE`,
2. immutable-row mode annotation,
3. focused write/read tests,
4. runtime inventory + docs update,
5. explicit tradeoff documentation for `raw` vs `masked` vs `pseudonymized`.
