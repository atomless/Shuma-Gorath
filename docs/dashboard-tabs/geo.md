# Dashboard Tab: GEO

Route: `#geo`  
Component: [`dashboard/src/lib/components/dashboard/GeoTab.svelte`](../../dashboard/src/lib/components/dashboard/GeoTab.svelte)

Purpose:

- Configure GEO scoring/routing and, when Akamai edge posture is available, trusted edge country-header ingestion.

Panels:

- `Trusted GEO Edge Header Signal` toggle (`geo_edge_headers_enabled`) when Akamai edge posture is available.
- `GEO Risk Based Scoring`:
  - enable/disable scoring path,
  - scoring countries list (`geo_risk`).
- `GEO Risk Based Routing`:
  - enable/disable routing path,
  - allow/challenge/maze/block country lists (`geo_allow`, `geo_challenge`, `geo_maze`, `geo_block`).

Validation:

- Country fields require valid ISO 3166-1 alpha-2 codes in comma-separated format.
- Scoring/routing toggles are persisted through `defence_modes.geo`.
- The current signal surface expects the upstream edge layer to map provider-native GEO data into `X-Geo-Country`; it is not yet a direct Akamai EdgeScape parser.
- Operator controls for trusted GEO edge-header ingestion are hidden unless the deployment reports `gateway_deployment_profile=edge-fermyon` (`akamai_edge_available=true` in `/admin/config`).
