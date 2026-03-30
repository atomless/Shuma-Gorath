# Fingerprinting Tab Temporary Rehome Plan

**Goal:** Remove the fragmented `Fingerprinting` dashboard tab by moving Akamai bot-signal source posture into `Verification` and temporarily moving the read-only botness signal inventory into `Tuning`.

**Architecture:** Rehome the two existing panels into their nearest truthful destination tabs, then delete the standalone tab from the dashboard tab registry, route wiring, focused tests, and tab docs. This slice does not change backend scoring or mutability semantics; it only cleans up dashboard ownership and presentation until the broader `TUNE-SURFACE-2*` work lands.

**Tech Stack:** Svelte dashboard tabs, dashboard route/runtime state, focused Node unit tests, focused Playwright smoke checks, Markdown docs.

---

## Acceptance Criteria

1. `#fingerprinting` is removed from the dashboard tab registry and route/controller/store wiring.
2. `Akamai Bot Signal` is rendered at the top of `Verification`.
3. The read-only botness signal inventory is rendered in `Tuning`.
4. `FingerprintingTab.svelte` and tab-specific docs/tests are removed or rewritten so no dead ownership path remains.
5. Focused verification passes:
   - `make test-dashboard-tab-information-architecture`
   - `make test-dashboard-policy-pane-ownership`
   - `make test-dashboard-verified-identity-pane`
   - `make dashboard-build`

## Task 1: Write failing dashboard-contract tests first

Files:
- Modify: `e2e/dashboard.modules.unit.test.js`
- Modify: `e2e/dashboard.smoke.spec.js`
- Modify if needed: `Makefile`

Steps:
1. Update unit tests to expect no `fingerprinting` tab in the canonical tab registry.
2. Update pane-ownership assertions so:
   - `VerificationTab.svelte` owns the Akamai bot-signal controls,
   - `TuningTab.svelte` owns the read-only botness signal list,
   - and `FingerprintingTab.svelte` is no longer part of the live surface contract.
3. Update smoke tests to stop navigating to `#fingerprinting` and instead prove the moved controls in `Verification` and `Tuning`.
4. Run the smallest focused `make` target(s) and confirm the tests fail for the expected reasons before implementation.

## Task 2: Rehome the Akamai panel into `Verification`

Files:
- Modify: `dashboard/src/lib/components/dashboard/VerificationTab.svelte`

Steps:
1. Import the existing config-runtime and edge-mode helpers needed for the Akamai controls.
2. Add the Akamai panel at the top of the tab using existing config-panel styling patterns.
3. Preserve current config payload semantics and warnings.
4. Keep the verification tab save-flow truthful if the Akamai section has unsaved changes.

## Task 3: Rehome the read-only signal inventory into `Tuning`

Files:
- Modify: `dashboard/src/lib/components/dashboard/TuningTab.svelte`

Steps:
1. Read `botness_signal_definitions` from the runtime snapshot.
2. Render the runtime scoring-definition list below the current editable botness controls.
3. Keep the section read-only and outside the dirty/save calculation.
4. Preserve the current tuning save semantics for the editable fields.

## Task 4: Retire the `Fingerprinting` tab end to end

Files:
- Modify: `dashboard/src/routes/+page.svelte`
- Modify: `dashboard/src/lib/domain/dashboard-state.js`
- Modify: `dashboard/src/lib/runtime/dashboard-route-controller.js`
- Modify: `dashboard/src/lib/runtime/dashboard-native-runtime.js`
- Modify: `dashboard/src/lib/state/dashboard-store.js`
- Delete: `dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`

Steps:
1. Remove the tab key from canonical tab arrays and loading-message maps.
2. Remove the lazy-load and render path from the route.
3. Remove any no-longer-needed refresh activation wiring for the old tab.
4. Delete the tab component once no live references remain.

## Task 5: Update docs and completion records

Files:
- Modify: `docs/dashboard.md`
- Modify: `docs/dashboard-tabs/tuning.md`
- Modify: `docs/dashboard-tabs/verification.md`
- Delete or rewrite: `docs/dashboard-tabs/fingerprinting.md`
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/completed-todo-history.md`

Steps:
1. Remove `#fingerprinting` from dashboard tab-route docs.
2. Document the temporary scoring-definition section in `Tuning`.
3. Document Akamai bot-signal posture at the top of `Verification`.
4. Leave a completed-history record explaining the temporary nature of the rehome.

## Task 6: Run focused verification and only then report completion

Steps:
1. Run `make test-dashboard-tab-information-architecture`.
2. Run `make test-dashboard-policy-pane-ownership`.
3. Run `make test-dashboard-verified-identity-pane`.
4. Run `make dashboard-build`.
5. If any target fails, fix the contract and rerun before claiming the slice is done.
