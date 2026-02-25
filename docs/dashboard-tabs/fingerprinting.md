# Dashboard Tab: Fingerprinting

Route: `#fingerprinting`  
Component: [`dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`](../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte)

Purpose:

- Configure Akamai bot-signal influence on fingerprinting and inspect runtime fingerprint diagnostics.

Panels:

- `Akamai Bot Signal`:
  - enable toggle (`provider_backends.fingerprint_signal` internal/external),
  - influence mode selector (`edge_integration_mode`: `additive` or `authoritative`).
- `Diagnostics` (read-only):
  - runtime counters,
  - selected fingerprint mismatch counters,
  - scored fingerprint signals from `botness_signal_definitions`.

Behavior notes:

- When Akamai bot signals are disabled, influence mode is disabled in the UI.
- `authoritative` mode surfaces a warning because high-confidence edge outcomes can directly drive stronger actions.
