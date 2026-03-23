Date: 2026-03-23
Status: Proposed planning driver

Related context:

- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
- [`2026-03-21-verified-identity-execution-readiness-refresh.md`](2026-03-21-verified-identity-execution-readiness-refresh.md)
- [`2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md`](2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md)
- [`2026-03-23-scrapling-non-human-category-capability-review.md`](2026-03-23-scrapling-non-human-category-capability-review.md)
- [`../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/benchmark_beneficial_non_human.rs`](../../src/observability/benchmark_beneficial_non_human.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Decide how verified identity, including native Web Bot Auth, should calibrate Shuma's canonical non-human taxonomy, fingerprinting quality, botness interpretation, and closed-loop tuning safety before `MON-OVERHAUL-1`.

# Findings

1. Verified identity already enters the request path early and is treated as exact observed evidence rather than advisory metadata.
   - `RequestFlow` verifies identity before the main policy tranches and records a verified-identity lane into request outcomes.
   - The current runtime therefore already has a high-confidence non-human label source for the subset of traffic that opts into cryptographic or trusted-provider identity.

2. The current taxonomy crosswalk is too compressed to use verified identity as a serious calibration source.
   - `signed_agent` currently collapses into `agent_on_behalf_of_human`.
   - `verified_bot` currently collapses into `verified_beneficial_bot`.
   - This loses the distinctions already present in the verified-identity contract (`search`, `training`, `preview`, `service_agent`, `user_triggered_agent`) and prevents later tuning or monitoring from learning whether fingerprinting is correctly separating those categories.

3. The current non-human classification receipts are lane-derived rather than alignment-derived.
   - Shuma can currently emit a receipt saying a request landed in `verified_bot` or `signed_agent`.
   - It does not yet emit a first-class receipt saying whether the verified identity category aligned with the chosen taxonomy category, whether the mapping was a faithful crosswalk or a degraded fallback, or whether botness disagreed with the verified identity.

4. The current benchmark layer partially accounts for verified identity, but only at a coarse posture level.
   - `beneficial_non_human_posture` measures allow or deny mismatch and verification coverage ratio.
   - It does not currently measure verified-identity versus taxonomy alignment quality, nor verified-identity versus botness conflict pressure.
   - The non-human category posture family also consumes category receipts without distinguishing whether those receipts were strongly supported by verified identity or merely collapsed from it.

5. The current controller guardrails are weaker than they should be for high-confidence beneficial traffic.
   - Tuning eligibility already fails closed on non-human classification readiness, coverage gaps, and protected replay evidence.
   - It does not yet fail closed specifically when high-confidence verified traffic is being frictioned or blocked contrary to configured posture, or when botness is repeatedly conflicting with verified identity in a way that implies calibration drift.

6. Web Bot Auth is therefore important to the feedback loop, but in a bounded role.
   - It should act as a calibration anchor and harm-avoidance guardrail.
   - It must not become the sole classifier, because most hostile traffic will not adopt it and authenticated identity is not the same thing as authorization or benign intent.

# Decision

1. Insert a dedicated pre-`MON-OVERHAUL-1` verified-identity calibration track after the Scrapling request-native coverage slices and before the Monitoring redesign.

2. Execute that track as four ordered tranches:
   - `VID-TAX-1` faithful verified-identity category crosswalk into the canonical taxonomy,
   - `VID-TAX-2` explicit verified-identity versus taxonomy alignment receipts,
   - `VID-BOT-1` verified-identity versus botness conflict metrics,
   - `VID-GUARD-1` hard diagnosis and tuning guardrails for verified-identity friction mismatch.

3. Keep the taxonomy stable while deepening the crosswalk.
   - Do not add new taxonomy categories just to mirror every verified-identity category.
   - Instead, define a tested projection from verified-identity categories into the existing taxonomy, while preserving enough alignment metadata to show where the crosswalk is strong, weak, or lossy.

4. Treat verified identity as a controller safety boundary.
   - If Shuma is persistently challenging, restricting, or blocking high-confidence verified traffic in ways that violate configured posture, the controller must fail closed and refuse to recommend or apply more aggressive tuning until the mismatch is understood.

# Why This Is Safe

1. It strengthens the closed loop without broadening trust.
   - The plan does not grant verified traffic automatic allow.
   - It only improves the system's ability to say when its own categorization or tuning is drifting away from explicit operator intent.

2. It improves Monitoring by improving backend truth first.
   - `MON-OVERHAUL-1` will be able to project faithful verified-identity semantics instead of flattening them into two vague buckets.

3. It keeps the first closed loop conservative.
   - Verified identity becomes a bounded calibration and no-harm guardrail, not an unreviewed auto-permit mechanism.
