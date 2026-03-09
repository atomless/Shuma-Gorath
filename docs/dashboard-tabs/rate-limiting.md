# Dashboard Tab: Rate Limiting

Route: `#rate-limiting`  
Component: [`dashboard/src/lib/components/dashboard/RateLimitingTab.svelte`](../../dashboard/src/lib/components/dashboard/RateLimitingTab.svelte)

Purpose:

- Configure local rate limiting and, when Akamai edge posture is available, the external distributed rate-limiter backend toggle.

Panels:

- `External Rate Limiter Backend` toggle when Akamai edge posture is available:
  - maps to `provider_backends.rate_limiter` (`internal`/`external`).
- `Rate Limiting`:
  - enable/disable enforcement (stored in `defence_modes.rate`),
  - requests-per-minute threshold (`rate_limit`, valid range `1..1,000,000`).

Behavior notes:

- Threshold is per IP bucket (`IPv4 /24`, `IPv6 /64`), not per single host IP.
- Disabling enforcement keeps scoring signal path active and shows an in-panel warning.
- The current backend toggle is infrastructure selection, not direct Akamai rate-signal ingestion. Future Akamai-specific rate augmentation is separate backlog work.
- Operator controls for the external rate backend are hidden unless the deployment reports `gateway_deployment_profile=edge-fermyon` (`akamai_edge_available=true` in `/admin/config`).
