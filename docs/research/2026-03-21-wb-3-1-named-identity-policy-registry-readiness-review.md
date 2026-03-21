Date: 2026-03-21
Status: Ready for execution

Related context:

- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
- [`2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-post-implementation-review.md`](2026-03-21-wb-2-3-proxy-and-edge-trust-semantics-post-implementation-review.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)

# Scope reviewed

`WB-3.1` is the first local authorization-policy tranche after verified identity verification and trust semantics are in place. Its job is to activate a named identity policy registry that can make explicit allow/observe/restrict/deny decisions for authenticated non-human traffic without yet bundling downgrade/violation handling or low-cost content rendering.

# Findings

1. The config and domain schema for `WB-3.1` already exists:
   - `verified_identity.non_human_traffic_stance`
   - `verified_identity.named_policies`
   - `verified_identity.category_defaults`
   - `verified_identity.service_profiles`
   are already present in `src/config/mod.rs`, validated, exposed through admin config, and documented in `docs/configuration.md`.
2. `src/bot_identity/policy.rs` currently defines the policy vocabulary but not the runtime evaluator:
   - matcher fields,
   - action variants,
   - service-profile bindings,
   - and the `IdentityPolicyOutcome` enum already exist.
3. The runtime already carries verified identity through the request path:
   - `src/runtime/request_flow.rs` resolves verified identity before policy evaluation,
   - `src/runtime/request_facts.rs` already stores `verified_identity`,
   - `src/runtime/policy_pipeline.rs` already has a low-cost early stage before botness/JS/fingerprint work.
   That means `WB-3.1` can stay inside the existing policy-graph path instead of adding a parallel allowlist system.
4. The cleanest precedence model is explicit and minimal:
   - named policies, in listed order, first match wins;
   - then category defaults only for category-driven top-level stances;
   - then a top-level fallback stance outcome.
5. The top-level fallback stance should stay restrictive:
   - `deny_all_non_human` and `allow_only_explicit_verified_identities` both fall back to deny when no named policy matches,
   - `allow_verified_by_category` and `allow_verified_with_low_cost_profiles_only` may consult category defaults before the final deny fallback.
6. `WB-3.1` should not overclaim service-profile behavior. The config and action shape already allow `use_service_profile`, but actual lower-cost response shaping belongs to `WB-4.1`. For this tranche, the registry should resolve the selected profile and make it observable in logs/tests, while authorization behavior stays in the current browser-like request path.
7. `Observe` and `Restrict` should become real policy outcomes now, but they do not need a second routing model yet:
   - both should continue through the normal later defence stages instead of short-circuiting to explicit allow,
   - with their distinct policy outcome preserved so `WB-3.2` can later add downgrade/violation behavior without redefining the registry.
8. The early-stage ordering matters:
   - first-tranche coarse controls (IP-range policy, honeypot, rate-limit, existing ban) should still execute before verified-identity authorization,
   - verified-identity policy should execute before the later botness/geo/JS tranche so explicit allowed identities can skip unnecessary later work.
9. Host/site scope is mentioned in the higher-level March design, but the active `WB-3.1` implementation plan and current runtime do not need it yet:
   - current request handling uses the existing `site_id` path with a single runtime site,
   - acceptance criteria require optional path scope, not host/site scope.
   Adding host/site scope now would broaden the tranche without current product value.

# Decisions

1. `WB-3.1` will add a pure policy evaluator in the existing `src/bot_identity/policy.rs` domain rather than inventing a second runtime-only matcher.
2. Explicit precedence will be:
   - named policies in configured order,
   - category defaults only for `allow_verified_by_category` and `allow_verified_with_low_cost_profiles_only`,
   - final deny fallback from the top-level stance when no earlier rule produces a match.
3. Runtime behavior in this tranche will be:
   - `deny` and `use_service_profile(denied)` => explicit block,
   - `allow` and non-denied `use_service_profile(...)` => explicit allow in the current request path,
   - `observe` and `restrict` => continue into the existing later defence stages.
4. Service-profile selections other than `denied` will be resolved and recorded, but they will not change response shape yet. That limitation must be documented explicitly so the current stance/action names remain honest before `WB-4.1`.
5. `WB-3.1` will add a dedicated verified-identity policy stage between the existing first and second policy tranches rather than folding it into botness or adding ad hoc request-flow branching.
6. The tranche will add focused tests and a focused `make` target for:
   - pure policy precedence,
   - runtime policy graph/plan mapping,
   - and request-path ordering around explicit allow versus deny fallback.

# Why this is the right next slice

1. It activates the product value promised by verified identity: exact restriction and explicit named exceptions for authenticated non-human traffic.
2. It reuses the already-landed config/domain/request-path seams instead of creating a one-off allowlist path.
3. It keeps authentication, authorization, and future low-cost delivery separate, which matches both the repo rules and the earlier research.
4. It reduces later churn because `WB-3.2` and `WB-4.1` can build on a real registry and explicit precedence instead of retrofitting policy after the fact.
