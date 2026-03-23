Date: 2026-03-23
Status: Proposed

Related context:

- [`../research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md`](../research/2026-03-23-verified-identity-taxonomy-calibration-and-tuning-guardrails-review.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`](2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md)
- [`2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md`](2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md)
- [`2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)
- [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/operator_snapshot_non_human.rs`](../../src/observability/operator_snapshot_non_human.rs)
- [`../../src/observability/operator_snapshot_verified_identity.rs`](../../src/observability/operator_snapshot_verified_identity.rs)
- [`../../src/observability/benchmark_beneficial_non_human.rs`](../../src/observability/benchmark_beneficial_non_human.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# Goal

Deepen Shuma's verified-identity implementation so Web Bot Auth and other high-confidence verified identities act as:

1. a taxonomy-calibration input,
2. a fingerprinting and botness conflict detector,
3. and a hard no-harm guardrail for the first closed tuning loop.

# Scheduling Position

This track should execute:

1. after `SIM-SCR-FIT-1`, `SIM-SCR-FIT-2`, and `SIM-SCR-COVER-2`,
2. before `MON-OVERHAUL-1`,
3. before `TUNE-SURFACE-1`,
4. and before any broader `OVR-AGENT-2` expansion.

Reason:

1. Monitoring should project the settled meaning of verified beneficial traffic, not the current flattened approximation.
2. The closed loop should not continue tuning against botness and category pressure without a hard guardrail for verified-identity harm.

# Task 1: `VID-TAX-1` Faithful Verified-Identity Category Crosswalk

Define and implement an explicit, tested projection from verified-identity categories into the canonical non-human taxonomy.

Must include:

1. a deterministic crosswalk from `IdentityCategory` into the existing non-human taxonomy,
2. stronger mappings for the existing categories:
   - `search` -> `indexing_bot`,
   - `training` -> `ai_scraper_bot`,
   - `user_triggered_agent` -> `agent_on_behalf_of_human`,
   - `preview` -> `http_agent`,
   - `service_agent` -> `http_agent`,
3. an explicit fallback rule for `other` that does not silently pretend to be a more specific class,
4. preservation of verified-identity scheme, operator, provenance, and end-user-controlled semantics so later receipts can explain why a mapping happened,
5. no taxonomy expansion in this tranche.

Acceptance:

1. verified traffic no longer collapses almost entirely into `verified_beneficial_bot` and `agent_on_behalf_of_human`,
2. the runtime exposes a tested crosswalk rather than an implicit lane shortcut,
3. and the mapping is strong enough that later Monitoring can explain why a verified request counted as crawler, scraper, direct agent, or user-triggered agent traffic.

# Task 2: `VID-TAX-2` Verified-Identity Versus Taxonomy Alignment Receipts

Add explicit receipts that compare verified identity to the chosen taxonomy category.

Must include:

1. a machine-readable alignment receipt per relevant verified request or summarized row,
2. fields that preserve:
   - verified identity scheme,
   - verified identity category,
   - projected taxonomy category,
   - alignment status,
   - degradation or fallback reason,
   - evidence references,
3. snapshot-level visibility for top aligned and misaligned cases,
4. bounded summary counts so the operator snapshot and later Monitoring can show whether taxonomy calibration is healthy.

Acceptance:

1. Shuma can say not only "this was verified" but also "this verified identity aligned or did not align with the taxonomy projection",
2. later benchmark and tuning logic can block on repeated misalignment,
3. and the operator-facing Monitoring redesign can render these truths directly instead of inferring them later.

# Task 3: `VID-BOT-1` Verified-Identity Versus Botness Conflict Metrics

Add benchmark and snapshot metrics that expose when botness and verified identity disagree in important ways.

Must include:

1. a bounded conflict metric family for high-confidence verified traffic that is simultaneously classified or pressured as suspicious automation,
2. explicit conflict views for at least:
   - verified traffic with high botness pressure,
   - verified traffic that was short-circuited despite posture suggesting allow or tolerate,
   - user-triggered agent traffic that experiences human-like friction mismatch,
3. machine-first metrics surfaced through benchmark or snapshot payloads rather than only logs,
4. explicit `insufficient_evidence` behavior when verified sample size is too small.

Acceptance:

1. the system can quantify when fingerprinting and botness are drifting away from verified ground truth,
2. the beneficial non-human benchmark family no longer hides those conflicts inside coarse totals,
3. and later Monitoring can show these mismatches as first-class operator diagnostics.

# Task 4: `VID-GUARD-1` Hard Verified-Identity Guardrails In Diagnosis

Make diagnosis and bounded tuning fail closed when verified-identity mismatch indicates likely harm to tolerated or allowed verified traffic.

Must include:

1. new benchmark or reconcile blockers for:
   - verified-identity friction mismatch,
   - unresolved verified-identity versus botness conflict pressure,
   - degraded verified-identity taxonomy alignment,
2. escalation semantics that return `observe_longer` or equivalent fail-closed outcomes instead of recommending stronger tuning when those blockers are present,
3. explicit distinction between:
   - "verified but operator wants it restricted or blocked" and
   - "verified and operator intended it to be tolerated or allowed, but the system is harming it anyway",
4. no new auto-allow behavior.

Acceptance:

1. the controller will not recommend or apply changes that intensify harm against configured tolerated or allowed verified traffic,
2. diagnosis becomes more trustworthy because it treats verified-identity mismatches as evidence of calibration drift,
3. and the closed loop remains restrictive by default without being blind to beneficial traffic regressions.

# Verification Expectations

When these tranches execute, add focused Makefile-backed verification for:

1. verified-identity taxonomy crosswalk behavior,
2. alignment receipt materialization,
3. verified-identity versus botness conflict metrics,
4. oversight reconcile fail-closed behavior under verified-identity mismatch.

# Exit Criteria

This plan is complete when:

1. verified identity projects faithfully into the canonical taxonomy,
2. alignment receipts make that projection auditable,
3. benchmark and snapshot contracts quantify verified-identity versus botness conflicts,
4. and the controller refuses to tune through verified-identity friction mismatch.
