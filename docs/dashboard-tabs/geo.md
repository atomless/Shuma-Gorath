# Dashboard Tab: GEO

Route: `#geo`  
Component: [`dashboard/src/lib/components/dashboard/GeoTab.svelte`](../../dashboard/src/lib/components/dashboard/GeoTab.svelte)

Purpose:

- Configure GEO scoring/routing and Akamai GEO signal ingestion.

Panels:

- `Akamai GEO Signal` toggle (`geo_edge_headers_enabled`).
- `GEO Risk Based Scoring`:
  - enable/disable scoring path,
  - scoring countries list (`geo_risk`).
- `GEO Risk Based Routing`:
  - enable/disable routing path,
  - allow/challenge/maze/block country lists (`geo_allow`, `geo_challenge`, `geo_maze`, `geo_block`).

Validation:

- Country fields require valid ISO 3166-1 alpha-2 codes in comma-separated format.
- Scoring/routing toggles are persisted through `defence_modes.geo`.
