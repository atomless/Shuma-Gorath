# 2026-04-01 Code-Quality Gate And Deep Audit Review

## Question

What is the best Rust and SvelteKit static-analysis stack for dead code, unused functions or imports, and broken imports, and how should Shuma enforce that stack truthfully today?

## Findings

1. Rust's first trustworthy dead-code and unused-import baseline is still the compiler itself.
   - `rustc` warn-by-default lints already cover `dead_code`, `unused_imports`, `unused_variables`, and related signal.
   - Shuma's existing `make test-native-build-warning-hygiene` target is therefore the cleanest currently-passable Rust gate.

2. `cargo clippy` is the strongest first-party Rust follow-on, but it is broader than dead-code hygiene alone.
   - Strict `cargo clippy --all-targets --all-features -- -D warnings` currently surfaces a large body of existing repo debt, not just a few newly introduced dead-code issues.
   - That makes it valuable as a deeper audit lane, but not yet honest as a universal tranche-closing gate.

3. On the dashboard side, the current official Svelte static check is intentionally narrow.
   - [`../../package.json`](../../package.json) currently runs `svelte-check --no-tsconfig --diagnostic-sources "svelte,css"`.
   - That catches Svelte-template and CSS diagnostics, but it intentionally leaves JS-program analysis turned off even though [`../../dashboard/jsconfig.json`](../../dashboard/jsconfig.json) exists.

4. JS-aware `svelte-check` is the strongest no-new-dependency step Shuma can add immediately for the dashboard semantic audit lane.
   - A local baseline run of `svelte-check --diagnostic-sources "js,svelte"` currently reports 68 errors across 7 files.
   - That proves the audit is already useful, but also proves it is not yet truthful to present it as a green mandatory completion gate.

5. The best eventual stack is layered, not singular.
   - Rust baseline: compiler warnings as errors
   - Rust broader audit: Clippy
   - Rust dependency audit: `cargo-udeps` as the preferred unused-dependency sweep, with `cargo-machete` as a faster but less exact secondary pass
   - Dashboard baseline: current `svelte-check`
   - Dashboard broader audit: JS-aware `svelte-check`
   - Dashboard import/dead-file audit: ESLint import rules and `knip`

## Current Repo Baseline

### Healthy enough to block today

- `make test-native-build-warning-hygiene`
- `make test-dashboard-svelte-check`

These are already aligned with currently passing repo behavior and catch a real subset of dead-code and import drift.

### Valuable, but not yet honest as tranche-closing blockers

- strict `cargo clippy --all-targets --all-features -- -D warnings`
- JS-aware `svelte-check --diagnostic-sources "js,svelte"`

The baseline evidence gathered for this review showed:

- strict Clippy currently reports 148 diagnostics,
- JS-aware dashboard semantic checking currently reports 68 errors in 7 files.

That is genuine quality debt, not a reason to hide the tools. It is a reason to keep the names and enforcement honest.

## Recommendation

Adopt a two-tier model immediately:

1. `make test-code-quality`
   - mandatory for every non-doc tranche before it may be called complete
   - must remain truthful and passable today
   - should aggregate:
     - the repo wiring/policy contract,
     - focused Rust warning hygiene,
     - current dashboard static diagnostics

2. `make audit-code-quality-deep`
   - broader semantic audit lane
   - should run:
     - strict Clippy
     - JS-aware `svelte-check`
   - should be documented as deeper than the mandatory gate until the existing debt is burned down

3. Capture the follow-on explicitly.
   - Shuma should keep a blocked follow-on to graduate the deep audit into the mandatory gate once the known baseline debt is removed or intentionally re-scoped.

## Why This Is The Cleanest Fit For Shuma

- It satisfies the user's requirement that code-quality analysis become a compulsory part of tranche completion.
- It does so without pretending the repo is already clean enough for the stronger audits to be green.
- It preserves the repo's truth-in-naming rule for `make` targets and completion claims.
- It keeps the canonical contributor workflow inside the `Makefile`, CI, and PR checklist instead of making quality hygiene a side ritual.

## Sources

- Rust warn-by-default lints: <https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html>
- Clippy: <https://doc.rust-lang.org/clippy/>
- cargo-udeps: <https://github.com/est31/cargo-udeps>
- cargo-machete: <https://github.com/bnjbvr/cargo-machete>
- Svelte CLI `sv check` / `svelte-check`: <https://svelte.dev/docs/cli/sv-check>
- TypeScript `noUnusedLocals`: <https://www.typescriptlang.org/tsconfig/#noUnusedLocals>
- TypeScript `noUnusedParameters`: <https://www.typescriptlang.org/tsconfig/#noUnusedParameters>
- `eslint-plugin-svelte`: <https://github.com/sveltejs/eslint-plugin-svelte>
- `eslint-plugin-import`: <https://github.com/import-js/eslint-plugin-import>
- Knip: <https://knip.dev/>
