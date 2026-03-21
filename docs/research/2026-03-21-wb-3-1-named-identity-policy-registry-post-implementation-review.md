Date: 2026-03-21
Status: Complete

Related context:

- [`2026-03-21-wb-3-1-named-identity-policy-registry-readiness-review.md`](2026-03-21-wb-3-1-named-identity-policy-registry-readiness-review.md)
- [`../plans/2026-03-21-wb-3-1-named-identity-policy-registry-implementation-plan.md`](../plans/2026-03-21-wb-3-1-named-identity-policy-registry-implementation-plan.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)

# Scope reviewed

`WB-3.1` activates the local verified-identity authorization registry: named allow/observe/restrict/deny rules, category-default fallback for category-driven stances, restrictive top-level fallback, and runtime enforcement between the existing first and second policy tranches without yet introducing low-cost response shaping.

# Findings

1. The pure verified-identity policy evaluator landed in the existing domain seam:
   - named policy order is deterministic and first-match-wins,
   - path-prefix matching participates in rule selection,
   - category defaults only participate for `allow_verified_by_category` and `allow_verified_with_low_cost_profiles_only`,
   - and unresolved service-profile references fail closed to `denied` even though config validation already prevents that state.
2. Runtime enforcement now matches the planned stage order:
   - first-tranche coarse controls still execute first,
   - the new verified-identity policy stage runs before the later geo/botness/JS tranche,
   - explicit `allow` and non-`denied` `use_service_profile(...)` short-circuit as current-path allow,
   - `deny` and `use_service_profile(denied)` block immediately,
   - and `observe`/`restrict` continue into the later defence stages with distinct policy outcomes preserved.
3. The policy graph, plan builder, telemetry taxonomy, block-page reason, request-flow wiring, traffic classification, and monitoring policy-source normalization were all updated together, so the runtime does not rely on ad hoc side paths or misleading event labels.
4. Operator-facing truth is now explicit:
   - docs call out precedence and fallback order,
   - docs state that verified identity is not unconditional allow,
   - and docs make the current limitation explicit that service-profile selection is resolved now but lower-cost response shaping lands later.

# Review shortfall

1. One tranche-local shortfall was found during closeout review: the canonical plan-builder characterization snapshot and the shared traffic-classification regression set had not yet been extended to the new verified-identity decision family.
2. This did not change runtime behavior, but it left the repo's strongest parity artifact and one shared monitoring-classification seam without direct proof for the new policy source.

# Shortfall closure

1. Executed `WB-3.1-REVIEW-1` immediately:
   - extended `make test-verified-identity-policy` to cover traffic-classification tests,
   - updated the plan-builder characterization harness to evaluate the verified-identity tranche in the real stage order,
   - added snapshot cases for verified-identity allow/deny/observe/restrict,
   - and added targeted traffic-classification proof for `PolicyGraphVerifiedIdentityTranche`.
2. After that follow-up, no tranche-local `WB-3.1` shortfall remained open.

# Verification

- `make test-verified-identity-policy`
- `make test-runtime-preflight-unit`
- `make test-verified-identity-native`
- `make test-verified-identity-provider`
- `make test-verified-identity-proxy-trust`
- `git diff --check`

# Outcome

`WB-3.1` is complete. The implementation matches the reviewed plan, the one closeout gap was executed immediately as `WB-3.1-REVIEW-1`, and the next tranche remains `WB-3.2`.
