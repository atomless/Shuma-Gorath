# Open Backlog And Plan Reference-Stance Alignment Review

Date: 2026-03-24
Status: Completed audit

Related context:

- [`2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)
- [`../../todos/security-review.md`](../../todos/security-review.md)

# Purpose

Audit every still-open backlog item and the main plan documents they depend on, then decide which parts of the planning chain need updating so the repo consistently reflects the new recursive-improvement methodology:

1. operator-facing product stances stay distinct from the development reference stance,
2. later controller phases use run-to-homeostasis episodes rather than one-shot logic,
3. and the strict reference stance remains the later regression anchor for code evolution.

# Audit Surface

Reviewed open backlog files:

1. [`../../todos/todo.md`](../../todos/todo.md)
2. [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)
3. [`../../todos/security-review.md`](../../todos/security-review.md)

Reviewed still-relevant planning docs linked from those open items:

1. [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
2. [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
3. [`../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md`](../plans/2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md)
4. [`../plans/2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md`](../plans/2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md)
5. [`../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)
6. [`../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)

# Findings

## 1. Most open items do not need methodology edits

The majority of still-open backlog items are orthogonal to the new recursive-improvement goal.

That includes:

1. privacy follow-up items such as `SEC-GDPR-2..4`,
2. hardening and coverage items such as `TEST-HYGIENE-*`, `MZ-T*`, `TAH-*`, and CI hygiene,
3. edge-gateway and enterprise deferred items,
4. and security-review findings that remain tracked under enterprise/distributed-state or privacy follow-ons.

These items stay valid as written because they are:

1. implementation hygiene,
2. product hardening,
3. or deferred deployment architecture,
4. rather than recursive-improvement methodology design.

## 2. The backlog itself was already mostly aligned

The open backlog was already in relatively good shape after the 2026-03-24 methodology write-up:

1. `OVR-AGENT-2` already points at the new reference-stance and run-to-homeostasis notes,
2. `OVR-CODE-1` already points at the strict reference-stance regression-anchor rule,
3. and `RSI-METH-1` already exists as the dedicated blocked later methodology item.

The main backlog gap was not missing blocked later work.
It was that some currently active or future-facing UI and roadmap docs still risked blurring:

1. operator product stances,
2. the later development reference stance,
3. current Monitoring or Tuning ownership,
4. and later recursive-training behavior.

## 3. Monitoring and Tuning needed a cleaner distinction from later recursive-training methodology

The Monitoring and Tuning plans were already directionally right, but they needed one sharper statement:

1. `Monitoring` should remain the accountability surface for the current operator-selected product stance,
2. `Tuning` should remain the editor for that operator-selected stance,
3. and the later `Human-only / private` development reference stance belongs to later controller methodology rather than to the initial operator-facing Monitoring or Tuning contract.

Without that clarification, the same word "stance" could wrongly imply:

1. a user-selected live site posture,
2. and a later internal development/training posture,
3. as though they were the same object.

They are not.

## 4. Roadmap docs needed the same separation

The broader roadmap and mature adversary-sim roadmap were not wrong, but they were still one level too generic about later controller evolution.

They needed to say more explicitly that:

1. later recursive-improvement phases begin from a strict development reference stance,
2. later relaxed preset sweeps come afterwards,
3. and this does not reclassify operator-facing presets as training contracts.

## 5. No open security-review finding required methodology changes

The open entries in [`../../todos/security-review.md`](../../todos/security-review.md) remain accurate and do not need rewriting for the new goal.

They are about:

1. enterprise multi-instance ban correctness,
2. logging structure,
3. fingerprint cleanup,
4. and storage-level IP minimization.

Those remain valid regardless of recursive-improvement methodology.

# Decisions

1. Keep the active near-term execution queue intact; do not re-sequence unrelated hygiene, privacy, or deployment work in response to the new methodology.
2. Clarify in active and blocked backlog text that Monitoring and Tuning are product-facing stance surfaces, not later recursive-training surfaces.
3. Clarify in the main roadmap and linked UI plans that operator-facing stance presets are product presets, even if one of them shares the name `Human-only / private` with the later development reference stance.
4. Clarify in the mature adversary-sim and later apply-loop planning docs that later controller evolution should adopt:
   - strict reference stance first,
   - run-to-homeostasis episode control,
   - then later relaxed preset sweeps,
   - and finally strict-stance regression anchoring for code evolution.

# Result

After this audit, the repo should tell one consistent story:

1. present and near-term work remains product- and operator-facing,
2. later recursive-improvement work remains explicitly blocked,
3. and when those later phases reopen, they do so under the stricter reference-stance and homeostasis methodology rather than by silently reinterpreting current Monitoring or Tuning work.
