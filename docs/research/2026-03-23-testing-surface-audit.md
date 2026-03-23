# Testing Surface Audit

Date: 2026-03-23

## Findings

### [P1] The dashboard "unit" lane still over-relies on source archaeology instead of operator-visible behavior

The largest concentration of low-signal proof remains in [`e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js). A substantial portion of that file reads source files directly with `fs.readFileSync(...)` and then asserts for string presence or absence instead of exercising the rendered tab, store, or runtime behavior. Representative examples:

- [`e2e/dashboard.modules.unit.test.js:4786`](../../e2e/dashboard.modules.unit.test.js#L4786) proves panel ownership by reading multiple Svelte source files and regex-matching snippets.
- [`e2e/dashboard.modules.unit.test.js:5844`](../../e2e/dashboard.modules.unit.test.js#L5844) proves dashboard runtime responsibilities by matching strings in the runtime module instead of exercising the runtime.
- [`e2e/dashboard.modules.unit.test.js:5865`](../../e2e/dashboard.modules.unit.test.js#L5865) proves refresh behavior by regexing the refresh module rather than driving a render or state transition.

These checks are useful only as narrow source-contract or migration guards. They are not good proofs of current operator-visible behavior, and they create false confidence when they sit alongside stronger rendered checks. The repo already has better proof patterns in [`e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js#L1052), [`e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js#L1989), and [`e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js#L3539), which drive the real UI through Playwright.

### [P1] Several feature-focused targets include tests that only verify Makefile wiring or script text, not feature behavior

There is a second layer of archaeology outside the dashboard suite: small Python tests that validate target composition or shell-script text and are then included in feature-oriented `make` targets.

Representative examples:

- [`scripts/tests/test_verified_identity_make_targets.py`](../../scripts/tests/test_verified_identity_make_targets.py)
- [`scripts/tests/test_host_impact_make_targets.py`](../../scripts/tests/test_host_impact_make_targets.py)
- [`scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py)
- [`scripts/tests/test_integration_cleanup.py`](../../scripts/tests/test_integration_cleanup.py)
- [`scripts/tests/test_oversight_supervisor.py`](../../scripts/tests/test_oversight_supervisor.py)
- [`scripts/tests/test_adversary_sim_supervisor.py`](../../scripts/tests/test_adversary_sim_supervisor.py)

These tests are not valueless. Some of them are valid target-truthfulness or wrapper-contract guards. The problem is tiering and naming: when they are bundled into domain proof targets, it becomes too easy to read "this feature is tested" when what actually happened was "the Makefile still points at the expected selectors" or "the shell script still contains the expected header strings."

### [P2] Test target truth-in-naming and scope descriptions have drifted

The clearest concrete example is [`Makefile:799`](../../Makefile#L799), where `test-integration` still claims `21 scenarios`, while [`scripts/tests/integration.sh:12`](../../scripts/tests/integration.sh#L12) explicitly enumerates `28` scenarios. This is small, but it is exactly the kind of drift that makes the suite feel daunting and hard to trust.

The same tiering problem exists at the suite level: [`Makefile:560`](../../Makefile#L560) defines `test-live-feedback-loop-remote` as the live shared-host proof, but [`Makefile:570`](../../Makefile#L570) defines `make test` without that live operational tier. That split is reasonable, but it is not explained as clearly as it should be, so current operational proof and pre-merge proof are easy to conflate.

### [P2] The current operational shared-host path is tested, but mostly outside the routine pre-merge suite

Current live operational functionality is not untested. It is covered by explicit live and remote targets such as:

- [`Makefile:560`](../../Makefile#L560) `test-live-feedback-loop-remote`
- [`Makefile:701`](../../Makefile#L701) `test-deploy-linode`
- [`Makefile:730`](../../Makefile#L730) `test-remote-target-contract`

Those are backed by meaningful helper tests such as [`scripts/tests/test_live_feedback_loop_remote.py`](../../scripts/tests/test_live_feedback_loop_remote.py), [`scripts/tests/test_remote_target.py`](../../scripts/tests/test_remote_target.py), and focused deploy/helper subprocess tests.

The issue is not absence of proof. The issue is that the suite currently lacks a crisp, documented statement of test tiers:

1. static and source-contract guards,
2. local behavior tests,
3. Spin runtime integration tests,
4. rendered UI tests,
5. live operational proofs.

Without that split, the repo makes it harder than necessary to answer "what currently proves local correctness?" versus "what currently proves live shared-host operation?"

### [P3] Full-suite verification still creates routine artifact churn that weakens signal

This is already captured as [`TEST-HYGIENE-2`](../../todos/todo.md#L153), and the audit confirms it remains real. The adversarial and SIM2 targets still emit tracked JSON artifacts under `scripts/tests/adversarial/`, which means routine verification can rewrite committed files. That is not a correctness failure, but it degrades the usefulness of `git diff` as a proof surface and makes the suite feel noisier than it should.

## Layer-By-Layer Assessment

### Rust unit and crate-level behavior tests

This is currently the strongest part of the suite.

Representative examples:

- [`src/runtime/request_outcome.rs:260`](../../src/runtime/request_outcome.rs#L260) asserts real request-outcome state, not implementation strings.
- [`src/admin/oversight_reconcile.rs:652`](../../src/admin/oversight_reconcile.rs#L652) exercises reconcile pressure selection against snapshot data.
- [`src/observability/benchmark_results.rs:861`](../../src/observability/benchmark_results.rs#L861) proves verified-identity guardrail behavior through the benchmark contract.
- [`tests/routing_order_integration.rs:90`](../../tests/routing_order_integration.rs#L90) and nearby tests hit the public request path end to end through `handle_bot_defence_impl`.

Assessment:

- Meaningful: yes.
- Tests current operational functionality: generally yes, for local/runtime behavior.
- Optimal: mostly yes. The dominant pattern is behavior-first, colocated, and explicit.

### Spin integration shell harness

[`scripts/tests/integration.sh`](../../scripts/tests/integration.sh) remains meaningful because it drives a live Spin runtime over HTTP, exercises enforcement paths, config mutation, challenge flows, GEO routes, metrics, CDP, external fingerprint precedence, and tarpit behaviors.

Assessment:

- Meaningful: yes.
- Tests current operational functionality: yes, for local Spin runtime behavior.
- Optimal: not fully. The single large bash harness is harder to evolve and its cleanup proof is currently guarded by source-text tests in [`scripts/tests/test_integration_cleanup.py`](../../scripts/tests/test_integration_cleanup.py) rather than a smaller executable harness.

### Gateway and deploy helper tests

This layer is stronger than it first appears.

Representative examples:

- [`scripts/tests/test_validate_gateway_contract.py`](../../scripts/tests/test_validate_gateway_contract.py) executes the gateway contract validator in subprocesses with temporary manifests.
- [`scripts/tests/test_remote_target.py`](../../scripts/tests/test_remote_target.py) exercises remote-target commands with temporary receipts and mocked subprocess calls.
- [`scripts/tests/test_live_feedback_loop_remote.py`](../../scripts/tests/test_live_feedback_loop_remote.py) uses a fake remote class to drive the live-proof verifier logic and report shaping.

Assessment:

- Meaningful: yes.
- Tests current operational functionality: yes, especially helper and remote orchestration semantics.
- Optimal: mostly yes, with some source-based shell checks still mixed in around wrappers.

### Adversarial and SIM2 tests

This layer is mixed.

Strong examples:

- [`scripts/tests/test_adversarial_simulation_runner.py`](../../scripts/tests/test_adversarial_simulation_runner.py) exercises runner behavior and request-contract shaping.
- [`scripts/tests/test_scrapling_worker.py`](../../scripts/tests/test_scrapling_worker.py) drives the real Scrapling worker against a recording HTTP server.
- [`scripts/tests/test_adversarial_lane_contract.py`](../../scripts/tests/test_adversarial_lane_contract.py) validates real request-contract behavior, not just file shape.

Weaker examples in the same neighborhood:

- [`scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py) only proves Makefile selector composition.

Assessment:

- Meaningful: yes, overall.
- Tests current operational functionality: yes, but intermixed with wiring-only checks.
- Optimal: not yet. This layer needs cleaner separation between behavior proof and target-truthfulness proof.

### Dashboard tests

This layer is split between very strong rendered tests and weaker archaeology tests.

Strong examples:

- [`e2e/dashboard.smoke.spec.js:1052`](../../e2e/dashboard.smoke.spec.js#L1052) proves the logged-out dashboard auth gate through real browser behavior.
- [`e2e/dashboard.smoke.spec.js:1989`](../../e2e/dashboard.smoke.spec.js#L1989) proves verified-identity surfacing end to end.
- [`e2e/dashboard.smoke.spec.js:3539`](../../e2e/dashboard.smoke.spec.js#L3539) proves Policy-tab save behavior end to end.

Weaker examples:

- much of [`e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js), especially tests that prove source composition by regexing Svelte or JS files.

Assessment:

- Meaningful: mixed.
- Tests current operational functionality: Playwright yes, Node unit only partially.
- Optimal: no. This is still the biggest cleanup opportunity.

### Live remote proofs

Targets like [`test-live-feedback-loop-remote`](../../Makefile#L560) are highly meaningful because they test the current shared-host operational path that matters most to the project right now.

Assessment:

- Meaningful: yes.
- Tests current operational functionality: yes, directly.
- Optimal: mostly yes as a separate tier, but insufficiently distinguished from routine pre-merge verification in docs/help text.

## Recommended Sequence

1. Define explicit test tiers and truthful target scope in the Makefile and docs.
2. Continue the dashboard archaeology replacement work, but broaden it beyond dashboard files to wrapper/source-selector tests where behavior harnesses are feasible.
3. Keep the strong rendered Playwright and live remote proof lanes; do not collapse everything into one umbrella suite.
4. Eliminate routine generated-artifact churn from `make test` so the suite becomes easier to trust operationally.

## Conclusion

The test surface is not weak overall. In fact, the repo already has several very strong behavior-first layers: Rust domain tests, request-path integration tests, gateway/deploy helper subprocess tests, Scrapling worker tests, Playwright dashboard smoke tests, and explicit live shared-host proofs.

The main debt is architectural, not numerical:

- too many source-shape checks are living inside feature proof lanes,
- target and tier boundaries are not explained crisply enough,
- and full-suite verification still creates too much routine churn.

That means the next improvement is not "add more tests everywhere." It is to make the existing test tiers more truthful, better separated, and easier to read.
