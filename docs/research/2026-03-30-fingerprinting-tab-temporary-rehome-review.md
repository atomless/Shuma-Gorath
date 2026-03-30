Date: 2026-03-30
Status: Active

# Objective

Define the smallest truthful temporary dashboard change that:

1. removes the now-fragmented `Fingerprinting` tab,
2. keeps Akamai edge bot-signal source posture visible and editable,
3. keeps the read-only botness signal inventory visible for operator context,
4. and avoids leaving a nearly empty or semantically misleading tab behind while the broader `TUNE-SURFACE-2*` chain is still pending.

# Findings

## 1. The current `Fingerprinting` tab has split semantics

The current tab mixes two different jobs:

1. `Akamai Bot Signal`, which edits provider/source posture through:
   - `provider_backends.fingerprint_signal`
   - `edge_integration_mode`
2. `Botness Scoring Signals`, which is a read-only runtime projection from `botness_signal_definitions`.

That means the tab is neither a pure tuning surface nor a pure source-of-truth explanation tab.

## 2. Once editable fingerprint sensitivity controls move toward `Tuning`, the remaining Akamai controls fit `Verification`

The Akamai controls are not part of the bounded loop-tunable ring. They are provider-trust and source-posture settings. Those semantics fit naturally beside:

1. `JS Required`,
2. internal `CDP` automation verification,
3. `PoW`,
4. challenge controls,
5. and verified identity.

That makes `Verification` the cleanest surviving home for Akamai bot-signal source posture once the standalone `Fingerprinting` tab is removed.

## 3. The read-only botness signal list is acceptable in `Tuning` only as a temporary bridge

The desired product direction is still that `Tuning` should be editable cost-shaping controls only. But as a temporary measure, keeping the read-only signal list there is less misleading than leaving it stranded in a dedicated tab after the Akamai section moves away.

This temporary bridge is acceptable only if:

1. the section is clearly framed as the current runtime scoring definition,
2. it does not imply those controls are already editable there,
3. and the broader `TUNE-SURFACE-2A` / `2B` chain remains the follow-on that replaces the bridge with a cleaner final ownership model.

## 4. The tab itself should be retired in this temporary slice

Once both existing sections are rehomed, leaving `#fingerprinting` in the canonical tab registry would create:

1. dead route/controller/store state,
2. stale docs,
3. stale smoke flows,
4. and a misleading information architecture.

So the smallest clean slice is not "hide most of the tab." It is:

1. move `Akamai Bot Signal` to the top of `Verification`,
2. move the read-only botness signal inventory into `Tuning`,
3. remove the `Fingerprinting` tab route and component,
4. and delete or rewrite the corresponding docs and tests.

# Recommended Temporary Contract

1. `Verification` owns Akamai bot-signal source posture at the top of the tab.
2. `Tuning` owns current editable botness controls plus the temporary read-only runtime scoring-definition panel.
3. `Fingerprinting` ceases to be a dashboard tab.
4. Fingerprinting terminology and architecture docs remain valid, but dashboard tab docs and tests must stop treating `#fingerprinting` as a live route.

# Acceptance Criteria

1. The dashboard tab registry and route no longer expose `#fingerprinting`.
2. `Verification` renders the Akamai bot-signal panel before the rest of the verification controls.
3. `Tuning` renders the read-only botness signal inventory.
4. No dashboard doc or focused test still treats `Fingerprinting` as a live tab.
5. The slice is proved through focused dashboard unit/e2e targets rather than broad unrelated suite churn.
