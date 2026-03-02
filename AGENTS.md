# AGENTS.md

This file provides instructions for coding agents working in this repository.

## Scope and precedence

1. Follow explicit user instructions first.
2. Follow this file next.
3. Follow canonical project policy docs:
   - `docs/project-principles.md`
   - `CONTRIBUTING.md`
   - `docs/adr/README.md`
4. If instructions conflict, preserve security, correctness, and principle alignment.

## Core operating goals

- Keep defense as frictionless as possible for humans and tolerated bots.
- Make malicious bot behavior progressively more expensive.
- Prefer asymmetric designs where attacker cost rises faster than defender cost.
- Prioritize resource efficiency (bandwidth, CPU, memory, energy).
- Keep architecture modular and platform-agnostic.
- Implementation pattern mandate (strict, repo-wide): when adding or modifying behavior in an area that already has similar implementations, agents MUST first identify and follow the dominant existing project pattern (module boundaries, control flow, naming, error handling, telemetry shape, and test style). Introducing a new local pattern is allowed only when the current pattern is demonstrably insufficient and the user explicitly approves that deviation.
- Reuse-first mandate (strict, repo-wide): agents MUST prefer extending shared modules/components/utilities over duplicating near-equivalent logic. If similar code exists, factor to a shared helper/component in the canonical location and consume it from feature code instead of copy/paste variants.
- Enforcement command: treat avoidable duplication, one-off abstractions, and unapproved pattern drift as release-blocking non-compliance that must be revised before merge.
- For user-facing help text and documentation, prefer clear and helpful explanations over brevity.
- For user-facing validation/help text, write constraints as explicit rules using “must”/“must not” phrasing and concrete examples; avoid compressed slash notation (for example, avoid phrasing like `no spaces/query/fragment`).
- Treat this repository as pre-launch by default: do not add backward-compatibility aliases/shims/migration layers unless the user explicitly requests them for the specific change.
- UI design language is controlled and must remain consistent: do not invent, introduce, or experiment with new visual language (new border styles, spacing systems, color semantics, component idioms, interaction patterns, or typography shifts) unless the user explicitly asks for a design change. Default behavior is strict reuse of the existing canonical dashboard styles/components.
- UI element implementation mandate (strict): whenever adding or modifying dashboard UI fields/inputs/selects/buttons/tables, agents MUST reuse existing shared components, classes, and design tokens. Agents MUST NOT add one-off styling or duplicate near-equivalent CSS where canonical styles already exist. If a required pattern is missing, agents must first extend the shared style/component surface in its canonical location and then consume that shared pattern from feature code.
- Enforcement command: treat shared-style reuse as a release-blocking requirement. Any UI change that bypasses canonical components/classes without an explicit user-approved design exception is non-compliant and must be revised before merge.

## Required workflow for non-trivial changes

1. Read relevant docs and touched modules before editing.
2. Check `docs/plans/` for active or recent design docs relevant to the area you are changing, and align implementation with those plans unless the user explicitly overrides them.
3. When acting on TODO items, achieve full situational awareness before implementation:
   - scan the full TODO backlog first (`todos/todo.md` and `todos/security-review.md`) to identify intersecting items,
   - identify and read relevant plan documents in `docs/plans/`,
   - synchronize TODO execution with those plans so work does not duplicate, contradict, or drift from planned architecture.
4. Make small, reviewable changes.
5. Add/update tests for behavior changes.
6. Update docs for behavior/config/ops changes.
7. Run verification through `Makefile` targets only:
   - for any newly created development branch/worktree, run `make setup` before `make dev`/`make test` so `.env.local`, local tooling, and seeded config are initialized for that workspace,
   - `make test` as the umbrella verification path (unit + integration + dashboard e2e),
   - `make test-unit`, `make test-integration`, and `make test-dashboard-e2e` for focused reruns,
   - `make build` for release build verification,
   - `make setup`/`make verify` when environment/bootstrap behavior is touched.
   Makefile targets are the single source of truth for setup/build/test/deploy workflows.
   Direct ad-hoc tool invocations (for example `cargo test`, `cargo build`, `corepack pnpm run ...`, `playwright test`, `node e2e/...`, `spin up`) are not the canonical path for normal contributor/agent workflow.
   If a required workflow is missing from `Makefile`, add/update the target first, then run it via `make` (do not bypass and "fix later").
   Keep docs/PR notes/user guidance aligned to `make` commands so contributors follow one documented path.
   Keep `make dev` watch inputs scoped to source-of-truth files and explicitly exclude generated dashboard artifacts (for example `dashboard/.svelte-kit/**` and `dashboard/.vite/**`) to avoid self-triggered restart loops while preserving live reload for `dashboard/src/**`, `dashboard/static/**`, and `dashboard/style.css` edits.
   For `make test`, integration and dashboard e2e tests are mandatory and must not be skipped: start Spin first with `make dev` (separate terminal/session), then run `make test`.
   Exception: if a change is documentation-only (`*.md` and no behavior/config/runtime code changes), do not run tests; document that verification was intentionally skipped because the slice is docs-only.
   Non-negotiable anti-churn directive:
   - after every successful full-suite run (`make test`), agents MUST record a local verification receipt at `.spin/last-full-test-pass.json` with, at minimum, UTC timestamp, command, `git rev-parse HEAD`, and `git status --porcelain` output fingerprint;
   - before re-running full-suite verification, agents MUST compare the current `HEAD` and worktree fingerprint to the latest receipt;
   - if both match (no code/config/test changes since the last full pass), agents MUST NOT rerun `make test`; they MUST reuse the existing passing receipt and proceed with commit/push;
   - when state differs from the latest receipt, agents MUST run the minimum relevant verification first, then refresh `.spin/last-full-test-pass.json` only after a new successful `make test`.
8. Before reporting completion, confirm relevant CI status (or state explicitly that CI is pending/unverified).
9. Commit/push in atomic slices by default:
   - one logical change per commit,
   - avoid mixing unrelated refactors and feature/bug work in the same commit,
   - run relevant Makefile verification before each commit,
   - push after each validated atomic commit unless the user explicitly asks for batching,
   - after changes are merged, clean up merged branches as housekeeping (delete merged local topic branches and merged remote topic branches), while preserving protected branches such as `main`.
10. Document security, operational, and resource implications.
11. TODO housekeeping is immediate, not batched:
   - when any TODO checklist item is completed, move it from `todos/todo.md` to `todos/completed-todo-history.md` at the point of completion,
   - prepend new completion entries to the top of `todos/completed-todo-history.md`,
   - include the completion date for the moved entries,
   - preserve the original TODO section title(s) as headings in the archive entry.
12. For any new `SHUMA_*` variable, follow the single-source-of-truth lifecycle:
   - define/update canonical default in `config/defaults.env` first and classify it as env-only or KV-tunable,
   - wire seeding/bootstrap paths so `make config-seed`/`make setup` produce a correct local baseline (at minimum update `scripts/config_seed.sh`, `scripts/bootstrap/setup.sh`, and `Makefile` env wiring/help as applicable),
   - keep dev-only overrides intentional for local manual config/monitoring/tuning workflows (do not silently broaden them),
   - ensure tests leave no strange state behind (restore env mutations and reset runtime config they toggle),
   - ensure production-start defaults remain secure-by-default (no debug/unsafe defaults enabled by default).
13. Keep admin-writable config and Dashboard Advanced JSON in strict parity:
   - every KV-editable key accepted by `POST /admin/config` must appear in `dashboard/src/lib/domain/config-schema.js` Advanced JSON paths,
   - env-only keys remain excluded,
   - maintain/update parity tests so drift fails fast.

## Security and abuse posture

- Default to secure behavior and explicit hardening guidance.
- Do not weaken auth, trust-boundary checks, or monitoring visibility.
- Prefer low-cost passive signals before expensive interactive friction for likely humans.

## Architecture and boundaries

- Respect module boundaries documented in `docs/module-boundaries.md`.
- If internal feature work touches a capability covered by `src/providers/contracts.rs`, route the change through the provider interface and registry path rather than adding new direct module calls.
- Use ADRs (`docs/adr/0000-template.md`) for cross-cutting architecture/security/ops decisions.
- Avoid compatibility shims in pre-launch work; if an exception is explicitly requested, keep it tightly scoped, temporary, and documented with removal criteria.
- Use descriptive Rust module/file naming: prefer clear, responsibility-revealing `snake_case` names (for example `request_validation.rs`, `browser_user_agent.rs`) over vague names.
- Prefer explicit module files (`foo.rs`) over opaque `mod.rs` for new work when practical; keep directory + filename understandable without opening the file.
- Keep `.env.local` entries in unquoted `KEY=value` form for consistency. The setup script normalizes quoted values, so avoid introducing new quoted scalars unless a value truly needs shell quoting semantics.

## Pull request expectations

Ensure PR descriptions address:

- human visitor impact (friction/latency/challenge rate),
- adversary cost asymmetry and cost placement,
- resource impact,
- monitoring impact,
- rollback plan for risky changes.

Use `.github/pull_request_template.md`.

## Notes for agent tooling

- `AGENTS.md` is a convention, not a universal standard across all tools.
- If your tooling does not auto-read this file, treat `CONTRIBUTING.md` and `docs/project-principles.md` as mandatory equivalents.
