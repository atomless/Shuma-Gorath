# WB-3.1 Named Identity Policy Registry Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Activate the verified-identity policy registry so authenticated non-human traffic can be explicitly denied, allowed, observed, or constrained by named local policy.

**Architecture:** Reuse the existing verified-identity domain and runtime policy graph. Add a pure evaluator to `src/bot_identity/policy.rs`, thread it through a dedicated policy stage between the existing first and second tranches, and map the resulting outcomes through the current effect-intent/response pipeline. Keep service-profile selection resolved-but-not-rendered until `WB-4.1`.

**Tech Stack:** Rust, existing verified-identity config/domain types, runtime policy graph/effect-intent pipeline, Makefile-focused regression targets, Markdown operator docs.

---

### Task 1: Capture the focused policy-regression gate

**Files:**
- Modify: `todos/todo.md`
- Modify: `Makefile`
- Test: `src/bot_identity/policy.rs`
- Test: `src/runtime/policy_graph.rs`

**Step 1: Write the failing tests**

Add tests that prove:
- named policy order is explicit and first-match-wins,
- path prefixes participate in matching,
- `deny_all_non_human` and `allow_only_explicit_verified_identities` deny by default when no named rule matches,
- category defaults only participate for the category-driven top-level stances,
- service-profile actions resolve to the configured binding.

**Step 2: Run test to verify it fails**

Run: `make test-verified-identity-policy`
Expected: FAIL because the focused target does not exist yet and the pure evaluator is not implemented.

**Step 3: Add the focused make target**

Add `test-verified-identity-policy` that runs only the verified-identity policy-registry and runtime mapping coverage relevant to `WB-3.1`.

**Step 4: Run test to verify the failing tests are wired**

Run: `make test-verified-identity-policy`
Expected: FAIL on the new policy-registry assertions, not on missing target wiring.

### Task 2: Implement the pure registry evaluator

**Files:**
- Modify: `src/bot_identity/policy.rs`
- Test: `src/bot_identity/policy.rs`

**Step 1: Write the next failing test**

Add coverage proving that:
- a named policy can override a deny-all fallback with explicit allow,
- category defaults can supply a profile-backed outcome,
- and `observe`/`restrict` remain distinct outcomes even though they do not yet change response shape.

**Step 2: Run test to verify it fails**

Run: `make test-verified-identity-policy`
Expected: FAIL because the evaluator and resolution metadata do not exist yet.

**Step 3: Write minimal implementation**

Implement:
- matcher evaluation,
- profile binding lookup,
- explicit resolution source metadata,
- and deterministic precedence/fallback semantics.

Keep host/site scope out of this tranche.

**Step 4: Run test to verify it passes**

Run: `make test-verified-identity-policy`
Expected: PASS for the pure evaluator layer.

### Task 3: Wire the registry into the runtime policy graph

**Files:**
- Modify: `src/runtime/policy_graph.rs`
- Modify: `src/runtime/policy_pipeline.rs`
- Modify: `src/runtime/effect_intents/plan_builder.rs`
- Modify: `src/runtime/policy_taxonomy.rs`
- Modify: `src/enforcement/block_page.rs`
- Test: `src/runtime/policy_graph.rs`
- Test: `src/runtime/effect_intents/plan_builder.rs`

**Step 1: Write the failing tests**

Add tests that prove:
- verified-identity policy runs after the existing coarse first tranche and before the current second tranche,
- deny fallback blocks verified identities when the restrictive top-level stance says so,
- named allow short-circuits before the later botness/geo/JS stage,
- observe/restrict continue through the normal second tranche,
- and plan/taxonomy mapping is truthful for the new policy decisions.

**Step 2: Run test to verify it fails**

Run: `make test-verified-identity-policy`
Expected: FAIL because the runtime policy graph has no verified-identity authorization stage yet.

**Step 3: Write minimal implementation**

Add a dedicated verified-identity policy stage that:
- blocks on deny,
- forward-allows on explicit allow or non-denied service-profile selection,
- continues on observe/restrict,
- records truthful policy taxonomy/logging,
- and keeps service-profile selection resolved but not yet response-shaping.

**Step 4: Run test to verify it passes**

Run: `make test-verified-identity-policy`
Expected: PASS.

### Task 4: Make the current semantics operator-visible

**Files:**
- Modify: `docs/configuration.md`
- Modify: `docs/security-hardening.md`
- Modify: `docs/research/README.md`

**Step 1: Write the failing/coverage-extending checks**

If additional focused tests are needed to keep docs honest, extend `test-verified-identity-policy` before editing docs.

**Step 2: Run the focused gate**

Run: `make test-verified-identity-policy`
Expected: PASS before the doc pass.

**Step 3: Write minimal docs**

Document:
- precedence and fallback behavior,
- the difference between explicit allow versus observe/restrict,
- and the current `WB-3.1` limitation that service-profile selection is resolved now but lower-cost response shaping lands in `WB-4.1`.

**Step 4: Run the focused gate again**

Run: `make test-verified-identity-policy`
Expected: PASS.

### Task 5: Close the tranche honestly

**Files:**
- Add: `docs/research/2026-03-21-wb-3-1-named-identity-policy-registry-post-implementation-review.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Step 1: Run tranche verification**

Run:
- `make test-verified-identity-policy`
- `make test-verified-identity-native`
- `make test-verified-identity-provider`
- `make test-verified-identity-proxy-trust`
- `git diff --check`

Expected: PASS.

**Step 2: Perform code and architecture review**

Compare the implementation against:
- the `WB-3.1` acceptance criteria,
- the readiness review,
- the verified-identity design/implementation plan,
- and the current truth-in-naming rule around service-profile behavior.

**Step 3: Write minimal follow-up if review finds a shortfall**

If the review finds a tranche-local gap, record it as a new TODO and execute it immediately before claiming completion.

**Step 4: Record completion**

Move the `WB-3.1` TODO to `todos/completed-todo-history.md`, write the post-implementation review doc, commit, and push.
