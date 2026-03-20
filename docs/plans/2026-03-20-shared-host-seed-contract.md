# Shared-Host Seed Contract

Date: 2026-03-20
Status: Active implementation plan

Related context:

- [`2026-03-20-shared-host-scope-fence-contract.md`](./2026-03-20-shared-host-scope-fence-contract.md)
- [`../research/2026-03-20-shared-host-scope-contract-post-implementation-review.md`](../research/2026-03-20-shared-host-scope-contract-post-implementation-review.md)
- [`2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](./2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](./2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md`](../research/2026-03-20-telemetry-as-map-adversary-surface-discovery-synthesis.md)
- [`../../scripts/tests/shared_host_scope.py`](../../scripts/tests/shared_host_scope.py)
- [`../../scripts/site_surface_catalog.py`](../../scripts/site_surface_catalog.py)

## Purpose

Define the first executable contract for `SIM-SH-SURFACE-1-2`:

1. one required primary public start URL,
2. optional `robots.txt` ingestion,
3. optional small explicit extra seed list,
4. shared-host scope enforcement through the completed `SIM-SH-SURFACE-1-1` validator,
5. and one truthful artifact that captures accepted inputs, hint documents, provenance, and rejection reasons without pretending to be a public-surface catalog.

## Why This Slice Needs Its Own Narrow Plan

The next seed slice is small, but it still has one architectural fork that matters:

1. should `robots.txt` become the beginning of a richer public-surface inventory system,
2. or should it remain a bounded hint input inside the minimal seed contract.

The current roadmap and research make the correct answer clear:

1. `robots.txt` may provide cheap hints,
2. but it must not become authoritative discovery truth,
3. and this tranche must not recreate the old catalog-first model under a new artifact name.

## Core Decision

`SIM-SH-SURFACE-1-2` should produce a minimal shared-host seed inventory, not a public-surface catalog.

That inventory should contain:

1. accepted crawl start URLs,
2. accepted hint documents discovered from `robots.txt`,
3. rejected inputs with stable reasons,
4. and source provenance for every accepted URL.

The observed reachable surface still belongs to later traversal telemetry, not to this artifact.

## Seed Contract Shape

Recommended output shape:

```json
{
  "schema_version": "shared-host-seed-contract.v1",
  "primary_start_url": "https://example.com/",
  "accepted_start_urls": [
    {
      "url": "https://example.com/",
      "sources": ["primary_start_url"]
    },
    {
      "url": "https://example.com/pricing",
      "sources": ["manual_extra_seed"]
    }
  ],
  "accepted_hint_documents": [
    {
      "url": "https://example.com/sitemap.xml",
      "sources": ["robots"]
    }
  ],
  "rejected_inputs": [
    {
      "source": "manual_extra_seed",
      "raw_value": "http://evil.example.net/",
      "reason": "host_not_allowed"
    }
  ]
}
```

## Source Types

The first contract should use exactly these source labels:

1. `primary_start_url`
2. `robots`
3. `manual_extra_seed`

These should be stable contract values, not tool-local wording.

## Input Rules

### 1. Primary start URL is required

Rules:

1. exactly one primary start URL must be provided,
2. it must pass the shared-host scope validator,
3. it becomes the default first crawl start URL,
4. and it must always appear in `accepted_start_urls` with `sources=["primary_start_url"]` when valid.

### 2. Manual extra seed list is optional and intentionally small

Rules:

1. each manual extra seed must pass the same shared-host scope validator,
2. duplicates must merge deterministically by URL with combined provenance,
3. and the first contract must not support unbounded bulk import.

### 3. `robots.txt` is optional and hint-only

Rules:

1. the first contract may fetch or ingest `robots.txt`,
2. it should parse only the URL-bearing `Sitemap:` directives in the first slice,
3. and those accepted sitemap URLs belong in `accepted_hint_documents`, not `accepted_start_urls`, unless a later reviewed design explicitly promotes them.

This is the most important narrowing rule in the tranche.

`robots.txt` may point at useful hint documents, but it does not become the adversary's authoritative site map and it does not justify a precomputed reachable-surface artifact.

## Reuse Rule

This slice must reuse [`../../scripts/tests/shared_host_scope.py`](../../scripts/tests/shared_host_scope.py) directly.

It must not:

1. invent a second URL validator,
2. invent a second rejection taxonomy for scope failures,
3. or special-case redirects differently from the shared-host scope contract.

## Rejection Handling

`rejected_inputs` should capture:

1. source label,
2. raw value,
3. stable reason code,
4. and optional detail text only when needed for operator diagnosis.

For scope failures, the reason code should be passed through from the shared-host scope validator.

For `robots.txt` ingestion failures, add a small seed-contract-specific layer only where necessary, for example:

1. `robots_fetch_failed`
2. `robots_parse_failed`

Do not multiply seed-specific error taxonomies beyond what is needed.

## Explicit Boundary: Hint Documents Are Not The Surface Map

This slice should not emit:

1. a claimed list of all public pages,
2. a sitemap-expanded route inventory,
3. or any artifact named or documented as the authoritative public surface.

The artifact is only:

1. the operator-supplied start set,
2. plus bounded hint documents,
3. under a fail-closed scope fence.

The actual reachable surface still emerges later from traversal telemetry.

## First Operator Workflow

The first operator-facing surface should be tooling-first, not admin API first.

Recommended shape:

1. one CLI or script that accepts:
   - a shared-host scope descriptor path,
   - one required primary public start URL,
   - zero or more extra seed URLs,
   - and optional `robots.txt` ingestion,
2. one artifact output path,
3. and one focused `make` target for verification.

Recommended first artifact path:

1. `scripts/tests/adversarial/shared_host_seed_inventory.json`

Recommended first verification target:

1. `make test-shared-host-seed-contract`

## What This Slice Must Not Do

This tranche must not:

1. compile a full route inventory,
2. merge sitemap contents into a claimed public surface map,
3. add dashboard or KV surface,
4. or blur deploy-time gateway catalog tooling into adversary-seed truth.

## Acceptance Criteria

`SIM-SH-SURFACE-1-2` should be considered complete when:

1. the required primary start URL is enforced,
2. optional extra seeds and optional `robots.txt` hints are normalized through the shared-host scope validator,
3. accepted URLs carry merged provenance,
4. rejected inputs carry stable reasons,
5. hint documents remain distinct from crawl start URLs,
6. and the resulting artifact is explicitly documented as a minimal seed inventory rather than a public-surface catalog.

## Verification Plan

Add and use:

1. `make test-shared-host-seed-contract`

That target should prove:

1. primary URL enforcement,
2. deterministic provenance merge,
3. scope rejection passthrough,
4. bounded `robots.txt` parsing behavior,
5. and the separation between accepted start URLs and accepted hint documents.

## Outcome

When this plan is executed, Shuma will have the smallest realistic operator seed contract it needs for the first emergent lane:

1. one valid start URL,
2. a few optional extra entries,
3. cheap `robots.txt` hints,
4. and no regression back into catalog-first discovery.
