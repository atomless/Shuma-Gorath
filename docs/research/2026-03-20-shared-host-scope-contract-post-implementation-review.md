# Shared-Host Scope Contract Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-shared-host-scope-fence-contract.md`](../plans/2026-03-20-shared-host-scope-fence-contract.md)
- [`../plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](../plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)

## Review Goal

Confirm that `SIM-SH-SURFACE-1-1` landed as the narrow executable contract the updated shared-host roadmap called for:

1. one versioned descriptor contract,
2. one fail-closed validator,
3. one stable rejection taxonomy,
4. and no speculative admin or KV control surface.

## What Was Intended

This tranche was meant to convert the new shared-host discovery direction into real executable code before any seed workflow or Scrapling runtime work started.

The key design correction was scope:

1. keep the first implementation in pre-lane tooling,
2. make it versioned and testable,
3. and avoid adding dashboard, status, or persisted config surface before a runtime consumer exists.

## What Landed

1. A versioned shared-host contract file now defines the descriptor fields, default safety posture, baseline denied prefixes, and stable rejection reasons.
2. A shared Python validator now normalizes descriptor payloads, enforces the fail-closed URL gate, and revalidates redirects through the same scope contract.
3. A focused contract checker and `make test-shared-host-scope-contract` target now fail fast on contract drift.
4. The canonical adversarial Python unit target now includes this contract surface, so `make test` covers it through the existing Python verification path.
5. The backlog and Scrapling plan no longer imply that the first scope slice should begin with admin-writable KV or dashboard work.

## Shortfall Found During Review

One real shortfall surfaced during the tranche review:

1. the first validator draft would have allowed non-HTTP schemes if `require_https=false`, which violated the intended fail-closed posture.

That gap was fixed immediately before closeout:

1. non-HTTP schemes now reject as `malformed_url`,
2. malformed authorities with invalid ports now reject cleanly,
3. and regression tests cover both cases.

## Architectural Assessment

### 1. The first scope gate is now minimal for the right reasons

The repo now has the narrow scope contract it actually needs for the next seed tranche, without dragging in premature control-plane surface.

### 2. Deployment catalog tooling stays in its correct lane

The implementation does not blur gateway onboarding catalogs into the emergent adversary surface map.

### 3. The next seed tranche has a clean dependency

`SIM-SH-SURFACE-1-2` can now reuse one validator and one rejection taxonomy instead of inventing its own local scope rules.

## Result

Treat `SIM-SH-SURFACE-1-1` as complete.

The next optimal tranche is `SIM-SH-SURFACE-1-2`: minimal primary URL plus optional `robots.txt` and small extra-seed intake, all normalized through this shared scope contract.
