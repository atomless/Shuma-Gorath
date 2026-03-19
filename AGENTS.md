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

Planning-first workflow is mandatory unless the user explicitly stipulates a different sequence for the specific task.

1. Read relevant docs and touched modules before editing.
2. For any non-trivial feature, architectural change, telemetry/control-plane change, or multi-step tranche, establish and preserve the full planning chain in order:
   - perform or refresh the necessary research first and capture it in `docs/research/` before implementation,
   - convert that research into one or more plan/design docs in `docs/plans/`,
   - update roadmap or sequencing docs when the work changes execution order or dependencies,
   - manifest execution as atomic checklist TODOs in `todos/todo.md` or `todos/blocked-todo.md`,
   - execute one atomic tranche at a time, not a blended mega-change,
   - move completed TODOs immediately to the top of `todos/completed-todo-history.md` with the completion date,
   - conduct a post-implementation review after every tranche,
   - if that review finds shortfalls between research, plan, implementation, or the excellence demanded by the project, write them up as further TODOs and carry them out immediately,
   - conduct one final review before moving to the next planned tranche or the next round of research.
3. Check `docs/plans/` for active or recent design docs relevant to the area you are changing, and align implementation with those plans unless the user explicitly overrides them.
4. When acting on TODO items, achieve full situational awareness before implementation:
   - scan the full TODO backlog first (`todos/todo.md`, `todos/blocked-todo.md`, and `todos/security-review.md`) to identify intersecting items,
   - identify and read relevant plan documents in `docs/plans/`,
   - synchronize TODO execution with those plans so work does not duplicate, contradict, or drift from planned architecture.
5. Make small, reviewable changes.
6. Add/update tests for behavior changes.
7. Update docs for behavior/config/ops changes.
8. Run verification through `Makefile` targets only:
   - for any newly created development branch/worktree, run `make setup` before `make dev`/`make test` so `.env.local`, local tooling, and seeded config are initialized for that workspace,
   - `make test` as the umbrella verification path (unit + integration + dashboard e2e),
   - `make test-unit`, `make test-integration`, and `make test-dashboard-e2e` for focused reruns,
   - `make build` for release build verification,
   - `make setup`/`make verify` when environment/bootstrap behavior is touched.
   Makefile targets are the single source of truth for setup/build/test/deploy workflows.
   Direct ad-hoc tool invocations (for example `cargo test`, `cargo build`, `corepack pnpm run ...`, `playwright test`, `node e2e/...`, `spin up`) are not the canonical path for normal contributor/agent workflow.
   If a required workflow is missing from `Makefile`, add/update the target first, then run it via `make` (do not bypass and "fix later").
   Keep docs/PR notes/user guidance aligned to `make` commands so contributors follow one documented path.
   For dashboard/browser verification, agents MUST choose the smallest relevant `make` path that proves the changed behavior. Do not default to broad dashboard e2e coverage when a focused `make test-dashboard-e2e PLAYWRIGHT_ARGS='--grep "..."'` run or an existing narrower `make` target can prove the exact rendered contract.
   If the current `Makefile` does not expose a focused path for the affected dashboard behavior, add or refine the target first, then verify through that target instead of falling back to unrelated suite-wide churn.
   Keep `make dev` watch inputs scoped to source-of-truth files and explicitly exclude generated dashboard artifacts (for example `dashboard/.svelte-kit/**` and `dashboard/.vite/**`) to avoid self-triggered restart loops while preserving live reload for `dashboard/src/**`, `dashboard/static/**`, and `dashboard/style.css` edits.
   For `make test`, integration and dashboard e2e tests are mandatory and must not be skipped: start Spin first with `make dev` (separate terminal/session), then run `make test`.
   Exception: if a change is documentation-only (`*.md` and no behavior/config/runtime code changes), do not run tests; document that verification was intentionally skipped because the slice is docs-only.
   Non-negotiable anti-churn directive:
   - after every successful full-suite run (`make test`), agents MUST record a local verification receipt at `.spin/last-full-test-pass.json` with, at minimum, UTC timestamp, command, `git rev-parse HEAD`, and `git status --porcelain` output fingerprint;
   - before re-running full-suite verification, agents MUST compare the current `HEAD` and worktree fingerprint to the latest receipt;
   - if both match (no code/config/test changes since the last full pass), agents MUST NOT rerun `make test`; they MUST reuse the existing passing receipt and proceed with commit/push;
   - when state differs from the latest receipt, agents MUST run the minimum relevant verification first, then refresh `.spin/last-full-test-pass.json` only after a new successful `make test`.
9. Before reporting completion, confirm relevant CI status (or state explicitly that CI is pending/unverified).
10. Commit/push in atomic slices by default:
   - one logical change per commit,
   - avoid mixing unrelated refactors and feature/bug work in the same commit,
   - run relevant Makefile verification before each commit,
   - push after each validated atomic commit unless the user explicitly asks for batching,
   - after changes are merged, clean up merged branches as housekeeping (delete merged local topic branches and merged remote topic branches), while preserving protected branches such as `main`.
11. Document security, operational, and resource implications.
12. TODO housekeeping is immediate, not batched:
   - when any TODO checklist item is completed, move it from `todos/todo.md` to `todos/completed-todo-history.md` at the point of completion,
   - when a TODO becomes explicitly blocked or contingent rather than execution-ready, move it from `todos/todo.md` to `todos/blocked-todo.md` with a short blocking condition,
   - when code or behavior work is completed without a pre-written TODO entry, agents MUST still add a dated completion record to `todos/completed-todo-history.md` describing the work delivered, why it was done, and the main evidence/verification, so every lasting change leaves an auditable paper trail,
   - prepend new completion entries to the top of `todos/completed-todo-history.md`,
   - include the completion date for the moved entries,
   - preserve the original TODO section title(s) as headings in the archive entry.
   - immediately after completing any TODO tranche, review the recently completed TODOs and their linked plan requirements as a consistency check,
   - confirm the delivered result actually met the planned requirements and did so with the architectural, security, and operational excellence demanded by this project; if it did not, continue the work or reopen the TODO instead of reporting completion.
13. For any new `SHUMA_*` variable, follow the single-source-of-truth lifecycle:
   - define/update canonical default in `config/defaults.env` first and classify it as env-only or KV-tunable,
   - wire seeding/bootstrap paths so `make config-seed`/`make setup` produce a correct local baseline (at minimum update `scripts/config_seed.sh`, `scripts/bootstrap/setup.sh`, and `Makefile` env wiring/help as applicable),
   - keep dev-only overrides intentional for local manual config/monitoring/tuning workflows (do not silently broaden them),
   - ensure tests leave no strange state behind (restore env mutations and reset runtime config they toggle),
   - ensure production-start defaults remain secure-by-default (no debug/unsafe defaults enabled by default).
14. Keep admin-writable config and Dashboard Advanced JSON in strict parity:
   - every KV-editable key accepted by `POST /admin/config` must appear in `dashboard/src/lib/domain/config-schema.js` Advanced JSON paths,
   - env-only keys remain excluded,
   - maintain/update parity tests so drift fails fast.
15. Non-negotiable whole-system evidence rule for usage/coverage claims:
   - agents MUST NOT claim that code/endpoint/functionality is "unused", "dead", "not exercised", or "only used by X" from partial inspection;
   - before making any such claim, agents MUST trace and verify the full execution surface end-to-end: runtime call sites, dashboard/client call paths, test suites (unit/integration/e2e/adversarial), `Makefile` target wiring, package scripts, and CI workflow execution paths;
   - agents MUST cite concrete evidence (`file:line` references and the exact verification commands/targets used) for every usage/non-usage assertion;
   - if full-path verification is incomplete, agents MUST explicitly state uncertainty and continue investigation instead of making definitive claims;
   - treat shallow usage assessments as release-blocking process failure.
16. Non-negotiable completion proof for large tranches/refactors (release-blocking):
   - for any large feature tranche, cross-cutting refactor, or architecture migration, agents MUST NOT claim "working", "complete", or "done" without end-to-end proof across runtime, CI, and UI surfaces;
   - Definition of Done for such work MUST include explicit acceptance checks proving expected traffic/data flow and shape at every boundary: generation/emission, persistence/telemetry, API read paths, and dashboard rendering/refresh behavior;
   - for dashboard telemetry or hot-read changes specifically, proof MUST cover the full data path end-to-end: backend emission/materialization, admin API payload shape, dashboard API-client normalization/adaptation, runtime/store merge, and the rendered tab or panel DOM that operators actually use;
   - unit or source-contract coverage for only one boundary is insufficient for dashboard telemetry changes. Agents MUST add or update tests at the boundary where the regression could be hidden, and MUST include at least one rendered proof when the user-facing dashboard output changes;
   - for adversary-simulation work specifically, completion MUST prove (1) traffic is generated, (2) traffic is persisted/observable in monitoring APIs, (3) traffic is rendered in dashboard sections expected by spec, and (4) CI/runtime gates assert those outcomes;
   - when any required proof is missing or flaky, agents MUST report the slice as incomplete, continue debugging, and must not present status as complete.
17. Non-negotiable Make target truth-in-naming rule (release-blocking):
   - agents MUST NOT add, rename, or document any `make` target whose name implies behavior/scope/isolation that the implementation does not actually guarantee;
   - before claiming a new `make` command is complete, agents MUST verify and document its real blast radius and data scope (for example runtime-specific vs shared keyspace, dev-only vs prod-only, destructive vs non-destructive);
   - when architecture constraints prevent strict semantics implied by a target name, agents MUST either (a) choose an accurate name, or (b) explicitly call out the limitation in help/docs and completion notes before merge;
   - ambiguous or misleading command naming/claims are process failures and must be corrected before completion.
18. Non-negotiable scope-lock and critical-state change gate (release-blocking):
   - for local issues (for example chart tick density, label rendering, or component-scoped UI behavior), agents MUST keep the first remediation slice scoped to the local module and MUST NOT modify global runtime/auth/connection/polling state flows in that same slice;
   - before changing any cross-cutting dashboard state path (connection state derivation, auth/session lifecycle, polling scheduler, route controller cancellation semantics, or global body/root class state), agents MUST present a file:line causal chain proving the local issue originates in that path and MUST receive explicit user signoff;
   - agents MUST NOT combine a local bugfix and a critical-state architecture change in one patch series without explicit approval for the architecture change;
   - for connection/auth/polling state changes, agents MUST add targeted regression tests that prove stability under cancellation/retry overlap (for example no connected/disconnected oscillation from client-side abort churn);
   - if investigation shows uncertainty about root cause, agents MUST instrument and gather evidence first, then propose the smallest change that addresses the proven cause.
19. Non-negotiable user no-touch and exact-restore contract (release-blocking):
   - when the user says not to touch specific files/regions/behaviors, agents MUST treat those areas as frozen and MUST NOT edit them for any reason unless the user later gives explicit permission;
   - if an agent violates a no-touch instruction or is asked to revert, the agent MUST restore the exact prior code (byte-for-byte where feasible) from authoritative evidence (`git` history/reflog/stash/recoverable objects), not an approximation;
   - agents MUST NEVER ad-lib, synthesize, or "best-guess" replacement code while claiming it is a restoration of prior code;
   - if exact prior content cannot be proven/recovered, the agent MUST stop, state that clearly, and request the missing source (commit hash/snippet/backup) before making further edits in that area;
   - completion claims for a revert are invalid unless the response includes concrete evidence of exact restoration (source reference and diff confirmation).

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
