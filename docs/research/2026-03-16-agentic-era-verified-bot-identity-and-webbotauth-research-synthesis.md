# Agentic-Era Verified Bot Identity and Web Bot Auth Research Synthesis

Date: 2026-03-16
Status: Active

## Scope

This research synthesis answers four Shuma questions:

1. What is the current standards and ecosystem reality around cryptographically authenticated bots and agents?
2. How should Shuma distinguish bot identity, bot reputation, and crawler preference signaling?
3. What operator controls become necessary once named bots and agents can authenticate themselves?
4. How should verified identity interact with Shuma's existing three-lane model of verified beneficial agents, declared crawlers/search bots, and unverified or suspicious automation?

## Current Shuma Baseline

Shuma already has several useful primitives:

1. advisory crawler policy and AI preference signaling in `src/crawler_policy/robots.rs`,
2. provider seams in `src/providers/contracts.rs`,
3. a verified-agent lane requirement in the March 15 agentic-era oversight research and design docs,
4. a broader agentic-era design direction that keeps the request path deterministic and Rust-owned.

The current gap is that Shuma does not yet have a dedicated bot-identity domain with:

1. a canonical verified-identity contract,
2. a local trust and authorization policy for named bots or agents,
3. monitoring and control surfaces for verified identities,
4. or a clean separation between authenticated identity and community or local reputation.

## External Findings

### 1. `webbotauth` is active now, and it is explicitly about authentication rather than reputation

The IETF `webbotauth` working group is active as of March 2026. Its charter scope is about:

1. cryptographically authenticating automated clients to origins,
2. discovering keys and related metadata,
3. publishing relevant metadata for automated clients,
4. and documenting privacy and security considerations.

The charter is also explicit that reputation systems are out of scope.

Shuma implication:

1. Verified bot identity and central intelligence must remain separate subsystems.
2. Shuma should not confuse "this identity authenticated successfully" with "this actor is trustworthy" or "this actor should be allowed."
3. The right split is:
   - identity/authentication,
   - local authorization policy,
   - optional external reputation/intelligence.
4. The operator must have an obvious, simple way to deny all non-human traffic, including successfully authenticated bots or agents, because many deployments will want cryptographic identity for recognition and reporting without offering any privileged access.

### 2. RFC 9421 gives the ecosystem a real cryptographic request-authentication base

RFC 9421 standardized HTTP Message Signatures in 2023. `webbotauth` and current platform support build on this rather than inventing a wholly separate signature model.

The current IETF architecture draft for web bot auth is already fairly concrete:

1. the `Signature-Agent` header identifies the automated client identity,
2. HTTP Message Signatures prove control over that identity,
3. anti-replay protections are required,
4. and reverse proxies need to preserve the relevant protocol elements instead of stripping them.

Shuma implication:

1. Shuma should treat cryptographic identity verification as a first-class request-path signal, not as a dashboard-only annotation.
2. The request-path verifier should be abstract enough to support multiple authentication schemes over time, but HTTP Message Signatures are the right first implementation target.
3. Replay protection, clock skew handling, and proxy/header preservation are core design requirements, not polish.

### 3. Discovery and registry are emerging pieces, and they are not the same as trust

The `webbotauth` registry draft defines a public list of URLs that point to well-known bot-auth resources, plus optional metadata such as:

1. RFC 9309 compliance posture,
2. trigger semantics,
3. rate-control expectations,
4. and related descriptive fields.

The Cloudflare agent-registry work makes the same operational point in vendor terms: origins need a way to discover public keys and policy metadata for named agents. The adjacent `Signature-Key` IETF draft is also worth watching because it explores direct key discovery hints for signatures.

Shuma implication:

1. Registry or directory data should be treated as discovery input, not automatic authorization.
2. Shuma needs a local trust policy layer that decides what to do with a named authenticated agent.
3. Shuma should watch `Signature-Key`, but it should not depend on that draft for the first tranche.

### 4. Major platforms already support verified bots and signed agents in production

This is no longer just standards work.

Cloudflare:

1. has documented signed agents as a first-class bot category,
2. uses HTTP Message Signatures or mTLS for signed-agent verification,
3. and now documents Web Bot Auth support as a verified-bot method.

Vercel:

1. exposes Verified Bots as an operator-facing feature,
2. provides verified bot attributes such as `verifiedBotName` and `verifiedBotCategory`,
3. and added Web Bot Auth support in 2025 so verification no longer depends only on static IP ranges or reverse DNS.

OpenAI:

1. documents that ChatGPT agent signs every outbound HTTP request,
2. publishes public keys in `/.well-known/http-message-signatures-directory`,
3. and states that ChatGPT agent traffic is controlled by an end user rather than being a training crawler.

Anthropic:

1. documents separate crawler identities such as `ClaudeBot`, `Claude-SearchBot`, and `Claude-User`,
2. reinforcing that there is not one monolithic "Anthropic bot" policy surface.

Google:

1. distinguishes user-triggered fetchers from standard crawlers,
2. and states that user-triggered fetchers generally ignore `robots.txt`.

Shuma implication:

1. Shuma should treat verified identity as a current interoperability requirement, not a speculative future feature.
2. Operator policy needs to be per identity and per category, not just per company.
3. `robots.txt` and AI preference signaling remain necessary, but they are not enough for user-triggered agents.

### 5. Authenticated automation still splits into meaningful traffic classes

The current ecosystem evidence reinforces the March 15 Shuma insight that "bot" is not one class.

Useful distinctions now visible in official vendor docs:

1. training crawlers,
2. search/indexing or retrieval bots,
3. preview or link-expansion bots,
4. user-triggered agents acting on behalf of a human,
5. and site-integrated assistants or service agents.

The IETF use-cases draft also reflects this, including use cases where origins want:

1. different service levels for different authenticated agents,
2. alternate content or summarization feeds,
3. or outright denial for some classes of automated access.

Shuma implication:

1. Verified identity must feed a policy lane, not just a yes/no allowlist.
2. The correct model is "authenticated agent, then policy," not "authenticated agent, therefore allowed."
3. Shuma should support per-identity actions such as:
   - allow,
   - allow on a lower-cost content profile,
   - restrict by scope or rate,
   - observe,
   - deny.
4. Shuma should also support a global operator stance such as "deny all non-human traffic by default," with per-identity exceptions only when the operator chooses to create them.

### 6. Low-friction beneficial-agent handling is not only about allowing access

The March 15 oversight research already captured the dual strategy:

1. increase attacker cost for suspicious or undeclared automation,
2. reduce cost and friction for beneficial authenticated agents.

The new verified-bot identity evidence sharpens that:

1. authenticated user-triggered agents may be conversion-positive traffic,
2. but they may also be costlier than search bots because they request user-specific browsing or retrieval work,
3. so the best operator outcome is often not "let them browse like a browser," but "give them cheaper structured access under clear policy."
4. some operators will still want to deny all non-human traffic, and verified identity should make that choice easier to express and audit, not harder.

Shuma implication:

1. Verified identity should eventually connect to operator-controlled low-cost content profiles, not only enforcement bypass.
2. Low-friction treatment must remain a policy choice, not a side effect of successful verification.
3. This is where Shuma's future agent-oriented content work belongs.

### 7. Legacy UA and IP signals are still useful, but they should become secondary identity aids

Cloudflare, Vercel, OpenAI, Anthropic, and Google all reinforce the same pattern:

1. user-agent strings still matter for declaration and policy communication,
2. IP ranges and reverse DNS still matter operationally,
3. but stronger identity is shifting toward cryptographic verification.

Shuma implication:

1. UA and IP rules should remain supporting evidence and fallback surfaces.
2. They should not remain Shuma's long-term primary "known bot" control model.
3. Static robots lists and CIDR allowlists should not be mistaken for a durable verified-identity architecture.

### 8. Verified identity introduces real trust-boundary and abuse questions

The standards and vendor docs surface several recurring concerns:

1. replay attacks,
2. key rotation and cache freshness,
3. directory poisoning or stale directories,
4. intermediaries stripping or mutating signature-relevant headers,
5. privacy and user correlation concerns for user-controlled agents,
6. and authenticated agents that still violate local policy or rate budgets.

Shuma implication:

1. Authenticated identity must be observable but not silently privileged.
2. Verified identities still need local rate budgets, scope rules, and downgrade paths when behavior breaches policy.
3. Trust-root and directory configuration should be operator-controlled and manual-only in early phases.

## Research-Backed Requirements

R1. Preserve Shuma's three-lane model:

- verified beneficial agents,
- declared crawlers/search bots,
- unverified or suspicious automation.

R2. Add a dedicated verified-bot identity lane driven by cryptographic identity or equivalent high-confidence provider verification.

R3. Keep identity/authentication separate from reputation/intelligence and from `robots.txt`/AI preference signaling.

R4. Support per-identity policy, not just per-company or per-user-agent policy.

R5. Treat authenticated identity as an input to authorization and service-level selection, not automatic allow.

R5a. Provide an obvious top-level operator stance for non-human traffic, including the ability to deny all non-human traffic regardless of successful authentication and then carve out explicit exceptions only where desired.

R6. Keep the request path deterministic and Rust-owned. Agents and LLMs must not participate in identity verification decisions on the hot path.

R7. Build identity verification through the provider contract path so:

1. local verification,
2. edge/provider-verified bots,
3. and future schemes such as mTLS

can normalize into one internal contract.

R8. Make replay protection, clock skew tolerance, directory freshness, and header-preservation expectations explicit.

R9. Reserve trust roots, identity allow/deny policy, and directory source selection as manual-only operator surfaces in early phases.

R10. Expose monitoring for:

1. verification outcomes,
2. identities seen,
3. policy actions taken,
4. replay rejects,
5. directory freshness,
6. and verified-agent success budgets.

R11. Keep central intelligence separate. Reputation may influence treatment, but it must not mint identity.

R12. Plan for low-cost content profiles for beneficial authenticated agents.

R13. Watch adjacent IETF work such as `Signature-Key`, but do not block first-tranche design on drafts that are not yet ecosystem-default.

## Shuma-Specific Conclusions

The evidence points to a five-part Shuma model.

### 1. Identity verification becomes its own domain

Shuma should add a dedicated bot-identity domain rather than burying verified agents inside robots or generic fingerprinting code.

### 2. Shuma needs a local authorization policy for authenticated agents

The core operator question is not only "is this really ChatGPT agent?" but also:

1. do we allow it,
2. at what rate,
3. on what paths,
4. with what content profile,
5. and what happens if it violates local behavior budgets?

### 3. Verified identity should reduce friction and reduce cost

The beneficial-agent lane should eventually map to cheaper responses and lower origin cost, not just bypass more defenses.

### 4. Reputation and verified identity must remain distinct

This aligns Shuma's verified-identity work with the separate March 16 central-intelligence work: reputation can bias treatment, but identity verification and central intelligence must not collapse into one feature.

### 5. Oversight should observe and tune budgets around verified agents, but not own trust roots

The future oversight controller should track verified-agent success and cost budgets. It should not be allowed to auto-mutate trust roots, directory sources, or named allow/deny policy without explicit operator action.

## Recommended Sequence

1. Write a dedicated Shuma design and phased implementation plan for verified-bot identity and web bot auth.
2. Add a canonical internal identity contract and telemetry surface.
3. Ship observe-only verification first.
4. Add local per-identity policy and provider normalization.
5. Add low-cost content/profile support for beneficial authenticated agents.
6. Wire verified-agent budgets into the oversight controller after the identity and policy surfaces are truthful.

## Source Links

- IETF Web Bot Auth working group charter: [https://datatracker.ietf.org/wg/webbotauth/about/](https://datatracker.ietf.org/wg/webbotauth/about/)
- RFC 9421, HTTP Message Signatures: [https://www.rfc-editor.org/rfc/rfc9421.html](https://www.rfc-editor.org/rfc/rfc9421.html)
- IETF draft, Web Bot Auth architecture: [https://datatracker.ietf.org/doc/html/draft-meunier-web-bot-auth-architecture-05](https://datatracker.ietf.org/doc/html/draft-meunier-web-bot-auth-architecture-05)
- IETF draft, Web Bot Auth registry: [https://datatracker.ietf.org/doc/html/draft-meunier-webbotauth-registry-01](https://datatracker.ietf.org/doc/html/draft-meunier-webbotauth-registry-01)
- IETF draft, Web Bot Auth use cases: [https://datatracker.ietf.org/doc/html/draft-nottingham-webbotauth-use-cases-01](https://datatracker.ietf.org/doc/html/draft-nottingham-webbotauth-use-cases-01)
- IETF draft, HTTP Signature-Key: [https://datatracker.ietf.org/doc/html/draft-hardt-httpbis-signature-key-02](https://datatracker.ietf.org/doc/html/draft-hardt-httpbis-signature-key-02)
- OpenAI, ChatGPT agent allowlisting: [https://help.openai.com/en/articles/11845367-chatgpt-agent-allowlisting/](https://help.openai.com/en/articles/11845367-chatgpt-agent-allowlisting/)
- OpenAI, Publishers and developers FAQ: [https://help.openai.com/en/articles/12627856-publishers-and-developers-faq/](https://help.openai.com/en/articles/12627856-publishers-and-developers-faq/)
- Anthropic, crawler controls for website owners: [https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler)
- Google, user-triggered fetchers: [https://developers.google.com/search/docs/crawling-indexing/google-user-triggered-fetchers](https://developers.google.com/search/docs/crawling-indexing/google-user-triggered-fetchers)
- Cloudflare docs, Signed agents: [https://developers.cloudflare.com/bots/concepts/bot/signed-agents/](https://developers.cloudflare.com/bots/concepts/bot/signed-agents/)
- Cloudflare docs, Signed agent policy: [https://developers.cloudflare.com/bots/concepts/bot/signed-agents/policy/](https://developers.cloudflare.com/bots/concepts/bot/signed-agents/policy/)
- Cloudflare docs, Web Bot Auth bot verification: [https://developers.cloudflare.com/bots/reference/bot-verification/web-bot-auth/](https://developers.cloudflare.com/bots/reference/bot-verification/web-bot-auth/)
- Cloudflare blog, Web Bot Auth: [https://blog.cloudflare.com/web-bot-auth/](https://blog.cloudflare.com/web-bot-auth/)
- Cloudflare blog, Signed agents: [https://blog.cloudflare.com/signed-agents/](https://blog.cloudflare.com/signed-agents/)
- Cloudflare blog, Agent registry: [https://blog.cloudflare.com/agent-registry/](https://blog.cloudflare.com/agent-registry/)
- Vercel docs, Verified Bots: [https://vercel.com/docs/botid/verified-bots](https://vercel.com/docs/botid/verified-bots)
- Vercel changelog, Web Bot Auth support: [https://vercel.com/changelog/vercels-bot-verification-now-supports-web-bot-auth](https://vercel.com/changelog/vercels-bot-verification-now-supports-web-bot-auth)
- Vercel changelog, verified bot directory: [https://vercel.com/changelog/vercel-botid-now-leverages-vercels-verified-bot-directory](https://vercel.com/changelog/vercel-botid-now-leverages-vercels-verified-bot-directory)
