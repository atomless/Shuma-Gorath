# 🐙 Edge Fingerprint Adapter Guide

This guide explains how to add an edge fingerprint adapter to Shuma-Gorath.

Current supported edge fingerprint providers:
- `["Akamai"]`

Shuma-Gorath is intentionally built around a normalized internal signal shape so new provider adapters can be added without rewriting policy/routing logic.

## 🐙 What "edge fingerprint provider" means

Edge fingerprint providers send bot-risk outcomes from edge infrastructure into Shuma-Gorath. Those outcomes are normalized and then fed into the same internal scoring/routing logic used by native signals.

The edge provider path is selected by:
- `provider_backends.fingerprint_signal=external`
- `edge_integration_mode=off|additive|authoritative`

The active external report endpoint is:
- `POST /fingerprint-report`

## 🐙 Required normalized shape

Each provider-specific adapter must normalize its payload into this internal shape (from `src/providers/external.rs`):

```rust
struct NormalizedFingerprintSignal {
    confidence: f32,   // 0.0..=10.0
    hard_signal: bool, // strong evidence flag
    checks: Vec<String>, // sanitized, bounded signal identifiers
    summary: String,   // sanitized operator-facing summary
}
```

That normalized signal is then mapped into `CdpReport`:

```rust
CdpReport {
    cdp_detected: normalized.hard_signal || normalized.confidence >= 4.0,
    score: (normalized.confidence / 2.0).clamp(0.0, 5.0),
    checks: normalized.checks.clone(),
}
```

## 🐙 Adapter implementation checklist

1. Add a provider payload struct and parser.
2. Validate ranges and required fields.
3. Sanitize all free-form identifiers (for example check names/tags) using existing request-validation helpers.
4. Bound list cardinality to keep storage/metrics/log cost predictable.
5. Normalize to `NormalizedFingerprintSignal`.
6. Reuse existing CDP tiering (`classify_cdp_tier`) and policy taxonomy transitions.
7. Preserve mode behavior:
   - `off`: ignore edge outcomes.
   - `additive`: add bounded edge evidence into local scoring.
   - `authoritative`: allow configured strong-signal short-circuit actions.
8. Keep fallback behavior explicit:
   - unknown/non-matching payloads should downgrade safely (for example internal handler fallback where applicable),
   - malformed matching payloads should return clear validation errors.

## 🐙 Where to extend code

Primary files:
- `src/providers/contracts.rs` (`FingerprintSignalProvider` contract)
- `src/providers/registry.rs` (provider selection and implementation labels)
- `src/providers/external.rs` (provider payload parsing + normalization + edge behavior)
- `src/lib.rs` (active report path dispatch via selected provider)

## 🐙 Test expectations for a new adapter

Add/update tests to prove:
- valid provider payloads normalize correctly,
- invalid score/shape values are rejected,
- unknown payloads follow safe fallback behavior,
- additive/authoritative mode gates are respected,
- auto-ban is only triggered under intended high-confidence conditions.

## 🐙 Operator documentation updates

When adding a new provider adapter:
- keep operator terminology aligned with [`fingerprinting-terminology.md`](fingerprinting-terminology.md),
- update signal-plane behavior docs in [`fingerprinting-signal-planes.md`](fingerprinting-signal-planes.md),
- update [`configuration.md`](configuration.md) provider matrix,
- update [`api.md`](api.md) endpoint behavior notes,
- update observability documentation for provider implementation labels and expected metrics behavior.
