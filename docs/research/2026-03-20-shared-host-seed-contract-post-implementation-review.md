# Shared-Host Seed Contract Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-shared-host-seed-contract.md`](../plans/2026-03-20-shared-host-seed-contract.md)
- [`../plans/2026-03-20-shared-host-scope-fence-contract.md`](../plans/2026-03-20-shared-host-scope-fence-contract.md)
- [`2026-03-20-shared-host-scope-contract-post-implementation-review.md`](./2026-03-20-shared-host-scope-contract-post-implementation-review.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`../plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](../plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)

## Review Goal

Confirm that `SIM-SH-SURFACE-1-2` landed as the minimal seed contract the updated shared-host roadmap required:

1. one required primary start URL,
2. optional manual extra seeds,
3. bounded `robots.txt` hint ingestion,
4. one reused scope validator,
5. and no drift back into a catalog-first surface artifact.

## What Was Intended

This tranche was meant to make the next shared-host gate executable without overreaching:

1. build a minimal seed inventory,
2. keep `robots.txt` limited to hint documents,
3. preserve provenance and structured rejections,
4. and open the path to `SIM-SCR-LANE-1` without inventing a second discovery subsystem.

## What Landed

1. A versioned shared-host seed contract now defines stable source labels, output sections, and rejection reasons.
2. A shared seed-inventory module and CLI now build the minimal artifact from a scope descriptor, primary start URL, optional extra seeds, and optional `robots.txt` ingestion.
3. The first operator workflow is now explicit and Makefile-backed through `make build-shared-host-seed-inventory`.
4. `robots.txt` ingestion is intentionally bounded to `Sitemap:` hint URLs, which are emitted separately from accepted crawl start URLs.
5. A focused `make test-shared-host-seed-contract` gate and broader adversarial Python-unit integration now keep the contract honest.
6. The active backlog now reflects that the shared-host scope-and-seed gate is complete and that `SIM-SCR-LANE-1` is unblocked.

## Shortfall Found During Review

One real shortfall surfaced during live verification:

1. the new CLI path initially imported correctly under unit-test module execution but failed when invoked as a standalone script.

That was corrected before closeout by making the script bootstrap its repository import path the same way other repo-local tooling does.

## Architectural Assessment

### 1. The seed artifact stays honest

The implementation does not claim to know the reachable public surface. It only records accepted start URLs, hint documents, and rejections.

### 2. `robots.txt` stayed in the right role

The first seed contract treats robots-derived sitemap references as bounded hints rather than as authoritative discovery truth.

### 3. The next runtime tranche now has its real prerequisite

With both shared-host gate slices complete, the next meaningful work is the runtime-lane tranche itself rather than more discovery-plumbing discussion.

## Result

Treat `SIM-SH-SURFACE-1-2` as complete.

`SIM-SCR-LANE-1` is now execution-ready and should be the next optimal tranche.
