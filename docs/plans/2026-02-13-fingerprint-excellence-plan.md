# Fingerprint Excellence Plan

Date: 2026-02-13
Status: Implemented (Phases 1-4 completed; Finch comparison spike pending)

## Context

Fingerprinting currently relies on a limited set of in-app signals and an external provider stub path. This leaves high-value edge transport signals underused and limits resistance to evasive bots that keep changing browser traits over time.

## Goals

- Build a normalized, provenance-aware fingerprint signal model.
- Detect cross-attribute inconsistencies and temporal drift.
- Keep false positives low by combining weak signals instead of over-trusting one source.
- Support Akamai-first edge signal ingestion without losing self-hosted portability.

## Non-goals

- Universal multi-vendor feature parity across all edge providers.
- Replacing Shuma botness policy orchestration with external policy engines.

## State-of-the-art Signals

1. Fingerprint stability and entropy are measurable but drift over time.
2. Detector surfaces are fingerprintable; static detector behavior is eventually evaded.
3. Inconsistency detection (rather than static fingerprint matching) is effective against evasive automation.
4. Path and timing provenance improve classifier quality when fused with browser/network signals.

## Internal vs Akamai Ownership

- `self_hosted_minimal`:
  - Internal signals as primary.
  - No hard dependency on external transport telemetry.
- `enterprise_akamai`:
  - Akamai transport/bot telemetry as primary source for network-layer attributes.
  - Shuma keeps normalization, weighting, correlation, and final routing decisions.

## Proposed Architecture

### A. Normalized fingerprint schema

- Define canonical attributes with provenance and confidence:
  - browser/runtime,
  - header/client-hint consistency,
  - transport identity (JA3/JA4-class inputs),
  - sequence/timing features.
- Include `source=internal|akamai` and `availability=active|disabled|unavailable`.

### B. Consistency engine

- Add explicit consistency checks across attributes.
- Score mismatch classes rather than simple boolean fails.
- Separate hard-fail patterns from soft-suspicion patterns.

### C. Temporal and session coherence

- Track short-window coherence for suspect traffic.
- Detect impossible fingerprint transitions within bounded time.
- Keep memory and retention bounded with TTL windows.

### D. Provider integration

- Replace external fingerprint stub with Akamai-first mapping adapter.
- Keep strict fallback to internal signal paths when external data is absent.

### E. Safety and explainability

- Log reasons with stable detection IDs.
- Keep operator-visible attribution for each score contribution.

## Rollout Strategy

1. Instrument schema and provenance without changing enforcement.
2. Enable consistency scoring in advisory mode.
3. Add policy thresholds for challenge/maze routing.
4. Enable authoritative edge precedence only after drift and false-positive baselines are stable.

## Implementation Status (2026-02-16)

- Completed:
  - normalized fingerprint schema with provenance/confidence + family caps/budgeting,
  - internal inconsistency + temporal/flow coherence detection IDs and telemetry,
  - trusted transport-header ingestion with untrusted-header detection paths,
  - CDP probe-family rotation (`v1`/`v2`/`split`) and staged rollout control,
  - persistence-abuse marker and low-friction micro-signal collection in JS challenge context,
  - operatorization via `/admin/cdp` config/stat surfaces and dashboard fingerprint cards,
  - regression coverage for mismatch/temporal/flow/probe-family paths.
- Pending:
  - Finch comparison spike (`FP-R Finch`) remains open as an evaluation-only follow-up.

## Structured Implementation TODOs

1. FP-1: Finalize normalized fingerprint schema and versioning.
2. FP-2: Implement Akamai edge outcome mapping into normalized fields.
3. FP-3: Add consistency rules for UA/client-hint/transport mismatches.
4. FP-4: Add temporal coherence windows and bounded state retention.
5. FP-5: Add stable detection IDs for mismatch classes.
6. FP-6: Extend botness weighting with provenance-aware confidence.
7. FP-7: Add dashboard/operator views for fingerprint attribution.
8. FP-8: Add replay and evasive-bot simulation tests.
9. FP-9: Add advisory vs authoritative integration tests.
10. FP-10: Publish rollback criteria and runbook for edge-authoritative mode.

## Research Addendum (2026-02-16)

The `R-FP-01`..`R-FP-09` paper tranche is now completed in:
`docs/research/2026-02-16-fingerprinting-research-synthesis.md`.

Execution-priority deltas from that synthesis:

1. Move inconsistency and temporal coherence ahead of static fingerprint enrichment.
   - Prioritize `FP-3`, `FP-4`, `FP-5`, and `FP-6` before broadening feature intake.
2. Treat detector-surface minimization as a first-class requirement.
   - Add versioned probe rotation and detector-fingerprinting regression checks under `FP-8`.
3. Add explicit feature-family entropy budgeting.
   - Extend `FP-1` schema with family-level caps and confidence dampening to reduce brittle over-weighting.
4. Keep challenge-bound markers ephemeral.
   - Any Picasso-style device-class marker must be short-lived, signed, replay-bound, and non-identity-forming.
5. Add privacy and retention controls as implementation gates.
   - No new fingerprint family should ship without TTL bounds, pseudonymization path, and operator documentation.

Implementation note:
- When external edge fingerprints are present (`enterprise_akamai`), they remain high-value corroboration inputs.
- Final policy composition and enforcement remain Shuma-internal.

## Enterprise Offering Snapshot (Akamai and Cloudflare)

- Akamai:
  - Bot Manager provides edge-side bot scoring, behavioral analysis, and browser fingerprinting, with threshold-based response segments (`cautious`, `strict`, `aggressive`) that can drive challenge/deny decisions.
  - Akamai bot controls are API-addressable and designed for edge-first enforcement and reporting.
- Cloudflare:
  - Bot Management exposes a request-level bot score (`1..99`) and rule-consumable bot fields (`verified_bot`, `static_resource`, `detection_ids`, JA3/JA4) for custom rule and Worker logic.
  - Detection IDs give stable heuristic identifiers that map well to Shuma detection-taxonomy goals.
- Planning implication:
  - Keep Shuma as the normalization/correlation owner while ingesting Akamai or Cloudflare edge outcomes as provenance-tagged signals.
  - Prioritize adapter support for edge attributes that Shuma cannot observe natively (for example JA3/JA4 and provider detection IDs).

## Source References

- https://link.springer.com/chapter/10.1007/978-3-642-14527-8_1
- https://doi.org/10.1109/SP.2018.00008
- https://doi.org/10.1007/978-3-030-29962-0_28
- https://arxiv.org/abs/2406.07647
- https://cybersecurity.springeropen.com/articles/10.1186/s42400-019-0023-1
- https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- https://www.akamai.com/products/bot-manager
- https://developers.cloudflare.com/bots/reference/bot-management-variables/
- https://developers.cloudflare.com/bots/additional-configurations/detection-ids/
- https://developers.cloudflare.com/bots/additional-configurations/ja3-ja4-fingerprint/
