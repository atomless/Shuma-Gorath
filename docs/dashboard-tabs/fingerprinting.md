# Dashboard Tab: Fingerprinting

Route: `#fingerprinting`  
Component: [`dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`](../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte)

Purpose:

- Configure Akamai-edge bot-signal influence on fingerprinting and inspect fingerprint-related botness scoring definitions.

Panels:

- `Akamai Bot Signal`:
  - enable toggle (`provider_backends.fingerprint_signal` internal/external),
  - influence mode selector (`edge_integration_mode`: `additive` or `authoritative`),
  - current additive scored contribution (`fp_akamai_edge_additive`) when Akamai edge posture controls are available.
- `Botness Scoring Signals` (read-only):
  - additive "botness" fingerprinting signals from `botness_signal_definitions`, including fingerprint-specific signals and passive corroboration inputs such as JS verification, browser policy match, GEO risk, and rate pressure.
  - excludes the dedicated Akamai additive edge contribution, which is surfaced in the `Akamai Bot Signal` pane instead.

Behavior notes:

- Akamai bot-signal controls are available only when the deployment reports `gateway_deployment_profile=edge-fermyon` (`akamai_edge_available=true` in `/admin/config`). Non-edge deployments hide the controls and show an availability note instead.
- When Akamai bot signals are disabled, influence mode is disabled in the UI.
- `authoritative` mode surfaces a warning because high-confidence edge outcomes can directly drive stronger actions.
