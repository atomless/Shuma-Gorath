Date: 2026-03-26
Status: Completed

# Project Policy Acceptance-Criteria And Proof Governance Post-Implementation Review

## What landed

This docs-only tranche codified the acceptance-and-proof discipline into Shuma's three canonical policy surfaces:

1. [`../../docs/project-principles.md`](../../docs/project-principles.md) now requires explicit acceptance criteria, measurable proof surfaces, and truthful completion claims for non-trivial work.
2. [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md) now tells contributors what must appear in plans, TODOs, and completion notes before work may be treated as done.
3. [`../../AGENTS.md`](../../AGENTS.md) now requires agents to define acceptance criteria during the planning chain, write TODOs with closure evidence, and refuse completion claims when proof is missing, contradictory, or flaky.

## Why this matters

The repo already had strong verification language, but it was too easy for planning completion and baseline capability to be described too closely to delivered feature closure. This tranche makes the project-wide rule explicit and discoverable.

## Remaining work

This policy tranche does not itself satisfy the active `VERIFY-GATE-1` implementation work. It provides the durable governance layer that should guide that next work.

## Verification

This slice was docs-only.

Evidence:

- `git diff --check`
