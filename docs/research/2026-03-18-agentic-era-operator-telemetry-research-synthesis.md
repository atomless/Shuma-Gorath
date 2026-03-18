Date: 2026-03-18
Status: Active research

Related Shuma context:

- [`2026-03-17-operator-decision-support-telemetry-audit.md`](./2026-03-17-operator-decision-support-telemetry-audit.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)
- [`../observability.md`](../observability.md)

Primary external sources:

- [Cloudflare Bot Analytics](https://developers.cloudflare.com/bots/bot-analytics/)
- [Cloudflare Bot scores](https://developers.cloudflare.com/bots/concepts/bot-score/)
- [Cloudflare Bot detection engines](https://developers.cloudflare.com/bots/concepts/bot-detection-engines/)
- [Cloudflare Verified bots](https://developers.cloudflare.com/bots/concepts/bot/verified-bots/)
- [Cloudflare Signed agents](https://developers.cloudflare.com/bots/concepts/bot/signed-agents/)
- [Cloudflare Web Bot Auth](https://developers.cloudflare.com/bots/reference/bot-verification/web-bot-auth/)
- [Cloudflare Signals Intelligence](https://developers.cloudflare.com/bots/additional-configurations/ja3-ja4-fingerprint/signals-intelligence/)
- [Cloudflare Detection IDs](https://developers.cloudflare.com/bots/additional-configurations/detection-ids/)
- [Cloudflare Sequence rules](https://developers.cloudflare.com/bots/additional-configurations/sequence-rules/)
- [Cloudflare AI Crawl Control](https://developers.cloudflare.com/ai-crawl-control/)
- [Cloudflare AI Crawl Control bot reference](https://developers.cloudflare.com/ai-crawl-control/reference/bots/)
- [Cloudflare Turnstile Analytics](https://developers.cloudflare.com/turnstile/turnstile-analytics/)
- [Cloudflare Turnstile challenge outcomes](https://developers.cloudflare.com/turnstile/turnstile-analytics/challenge-outcomes/)
- [Cloudflare Bot Feedback Loop](https://developers.cloudflare.com/bots/concepts/feedback-loop/)
- [Cloudflare blog: Forget IPs: using cryptography to verify bot and agent traffic](https://blog.cloudflare.com/web-bot-auth/)
- [Cloudflare blog: Introducing Markdown for Agents](https://blog.cloudflare.com/markdown-for-agents/)
- [Cloudflare docs: AI Labyrinth](https://developers.cloudflare.com/bots/additional-configurations/ai-labyrinth/)
- [Google overview of crawlers and fetchers](https://developers.google.com/crawling/docs/crawlers-fetchers/overview-google-crawlers)
- [Google common crawlers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-common-crawlers)
- [Google special-case crawlers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-special-case-crawlers)
- [Google user-triggered fetchers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-user-triggered-fetchers)
- [Google verification of crawler/fetcher requests](https://developers.google.com/crawling/docs/crawlers-fetchers/verify-google-requests)
- [Google reduce crawl rate](https://developers.google.com/crawling/docs/crawlers-fetchers/reduce-crawl-rate)
- [Google reCAPTCHA v3](https://developers.google.com/recaptcha/docs/v3)
- [OpenAI crawler overview](https://platform.openai.com/docs/bots/)
- [OpenAI ChatGPT agent allowlisting](https://help.openai.com/en/articles/11845367)
- [Anthropic crawler controls](https://support.anthropic.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler)
- [IETF Web Bot Auth working group](https://datatracker.ietf.org/wg/webbotauth/about/)
- [RFC 9421: HTTP Message Signatures](https://www.rfc-editor.org/rfc/rfc9421.html)

# Purpose

Identify the telemetry Shuma should collect now so that the Monitoring overhaul, tuning surface, verified-identity work, and later oversight controller are aligned with current state of the art in bot and agent defence.

This research focuses on:

1. how modern defenders and platforms classify non-human traffic,
2. how they distinguish beneficial automation, hostile automation, and likely-human traffic,
3. what analytics they expose to operators,
4. and which telemetry Shuma should adopt or emulate without abandoning its bounded, Rust-owned, hot-read architecture.

# Executive Summary

Three conclusions are now very clear.

## 1. "Bot" is not one class anymore

Current industry practice has decisively moved away from a single "bot versus human" distinction.

Google now explicitly distinguishes:

1. common crawlers,
2. special-case crawlers,
3. user-triggered fetchers.

OpenAI distinguishes:

1. training crawlers,
2. search crawlers,
3. user-triggered web fetchers.

Anthropic similarly distinguishes:

1. training crawler traffic,
2. search crawler traffic,
3. user-triggered assistant fetches.

Cloudflare's bot and agent directory now distinguishes at least:

1. verified bots,
2. signed agents,
3. AI crawlers,
4. AI assistants,
5. AI search,
6. search engines,
7. aggregators.

Shuma therefore should not build the next telemetry model around a single undifferentiated `botness` bucket.

## 2. State-of-the-art differentiation is layered, not singular

The best current practice is not "pick the best signal." It is layered classification:

1. positive identity when available,
2. provider or directory verification when identity is declared,
3. browser and fingerprint evidence,
4. behavioral or sequence evidence,
5. challenge/friction outcomes,
6. operator feedback and false-positive correction.

Cloudflare's public bot model makes this very visible through:

1. heuristics,
2. JavaScript detections,
3. machine learning,
4. anomaly detection,
5. global JA4 Signals Intelligence,
6. detection IDs,
7. sequence rules,
8. feedback loops.

Google's crawler guidance reinforces the same lesson from the "good bot" side:

1. user-agent strings are not enough,
2. verify with DNS and published IP ranges,
3. classify fetchers by purpose,
4. and do not assume user-triggered fetchers obey `robots.txt`.

## 3. Operator-grade telemetry is contextual, not just volumetric

Cloudflare Turnstile Analytics and Google reCAPTCHA v3 are especially important here. They do not just expose raw request counts. They expose:

1. action context,
2. score or outcome context,
3. solve and fail rates,
4. dimensions like country, ASN, hostname, browser, and user agent,
5. and operator tuning guidance tied to those outcomes.

That means Shuma should not stop at "how many rate-limit events happened" or "how many bans were issued." It should expose:

1. what class of traffic was involved,
2. what defence was applied,
3. what happened next,
4. how much friction likely-humans saw,
5. and what cost suspicious traffic still imposed on the host.

# Research Findings

## A. Fine-grained categorization of non-human traffic is now mandatory

### Google

Google's official crawler guidance is unusually explicit and useful.

Google separates automated traffic into:

1. common crawlers, which automatically obey `robots.txt`,
2. special-case crawlers, which may ignore the global `*` robots rule where there is product-specific agreement,
3. user-triggered fetchers, which generally ignore `robots.txt` because the fetch was initiated by a user.

Google also publishes:

1. distinct IP JSON files for these classes,
2. reverse-DNS masks per class,
3. verification steps for matching observed traffic to those classes.

Implication for Shuma:

1. Shuma needs a first-class telemetry distinction between autonomous crawling, product-specific crawling, and user-triggered fetches.
2. A "good bot" lane is too coarse by itself.
3. `robots.txt` compliance telemetry matters for crawlers, but user-triggered assistant/fetcher traffic must be treated as a separate class.

### Cloudflare

Cloudflare's current taxonomy goes even further:

1. verified bots,
2. signed agents,
3. AI crawlers,
4. AI assistants,
5. AI search,
6. search engines,
7. aggregators.

This is important because it reflects a real operational difference:

1. training crawlers,
2. search indexers,
3. user-driven assistants,
4. and cryptographically verified signed agents

should not all be measured or treated the same way.

### OpenAI and Anthropic

OpenAI and Anthropic now independently expose this same functional split:

1. training bot,
2. search bot,
3. user-triggered assistant fetcher.

Implication for Shuma:

1. the telemetry model should expect this taxonomy to persist and expand,
2. and should be designed now so new verified-agent categories can be inserted cleanly later.

## B. Identity and behavior are separate dimensions

The IETF `webbotauth` work is explicit that reputation and intent vocabulary are out of scope. It is about authenticating the automated client, not deciding whether it is benign.

Cloudflare's current model aligns with this:

1. verified bots and signed agents are identity classes,
2. bot score and detection engines are behavior classes,
3. and local WAF rules still decide what to allow or block.

OpenAI's ChatGPT agent allowlisting guidance shows the same pattern:

1. requests are signed using RFC 9421,
2. intermediaries can verify the signature or trust a directory-backed platform verdict,
3. but the site still decides local policy.

Implication for Shuma:

1. telemetry must keep `identity/authentication`, `behavioral suspicion`, and `local authorization decision` separate.
2. If these are collapsed into one score, the operator loses crucial meaning.
3. successful authentication must not be treated as implicit access, because some operators will want to block all non-human traffic while still understanding exactly which bots or agents are presenting themselves.

Recommended separate telemetry fields:

1. claimed identity,
2. verification method,
3. verification result,
4. behavioral lane,
5. suspicion score or band,
6. local policy decision,
7. policy source, such as top-level non-human stance versus named identity exception.

## C. State of the art human-vs-automation differentiation is multi-engine

Cloudflare's public bot stack currently exposes the most useful public decomposition:

1. heuristics,
2. JavaScript detections,
3. machine learning,
4. anomaly detection,
5. JA4 global Signals Intelligence,
6. detection IDs,
7. sequence rules,
8. bot feedback loop for false positives and false negatives.

Google's reCAPTCHA v3 also reinforces several telemetry design lessons:

1. use action-scoped risk analysis rather than one site-wide score,
2. score every interaction or action context,
3. observe the traffic first,
4. then tune thresholds based on real traffic and outcomes.

Implication for Shuma:

1. telemetry should record not only the final route decision, but also the contributing signal families.
2. Those signal families should be grouped into stable buckets, for example:
   1. identity,
   2. browser integrity,
   3. fingerprint,
   4. sequence or timing,
   5. rate pressure,
   6. geo or policy posture,
   7. threat-intelligence or central-intelligence input,
   8. challenge/friction outcome evidence.

## D. Operator analytics are dimension-rich and feedback-aware

Cloudflare Bot Analytics, AI Crawl Control, and Turnstile Analytics all emphasize the same pattern:

1. traffic type or bot-score segmentation,
2. detection-source segmentation,
3. top attributes such as IP, UA, ASN, country, hostname, browser,
4. challenge outcomes,
5. solve rates,
6. action-specific analytics,
7. false-positive and false-negative feedback loops.

Important operator patterns that show up repeatedly:

1. segment traffic by type,
2. segment by score source or detection source,
3. expose top offenders and top affected attributes,
4. track solve rates and failures by dimension,
5. explicitly support misclassification feedback.

Implication for Shuma:

1. Monitoring should support dimensioned drill-down on operator-useful dimensions, not just raw event tables.
2. The later oversight controller should consume summary views that already expose those dimensions in bounded form.

## E. Cost and crawl etiquette are first-class telemetry concerns

Google explicitly documents that crawl rate can create critical load or unwanted cost and recommends checking recent access logs to understand the source of traffic and whether the site structure is provoking crawl spikes.

Cloudflare AI Crawl Control and AI Labyrinth make the same economic framing explicit:

1. monitor crawler activity and request patterns,
2. track robots compliance,
3. create per-crawler policies,
4. shift unauthorized AI crawlers into low-value or deceptive paths,
5. and treat misbehaving deep traversal as a bot-identification signal.

Cloudflare's Markdown for Agents goes further and shows that agent-aware telemetry can include:

1. negotiated content format,
2. content-type distribution,
3. markdown token count,
4. content-signal policy exposure.

Implication for Shuma:

1. telemetry should explicitly represent host cost and content-shape decisions, not just binary allow/block outcomes.
2. Agentic-era monitoring should track whether suspicious traffic is being served expensive HTML/JS, cheaper markdown/text, maze content, or tarpit content.

# Recommended Traffic Taxonomy For Shuma Telemetry

Shuma should adopt a minimum operator taxonomy like this:

1. likely human,
2. unknown interactive client,
3. suspicious automation,
4. declared crawler or search bot,
5. user-triggered assistant or fetcher,
6. verified bot,
7. signed agent,
8. adversary-sim traffic.

Notes:

1. `verified bot` and `signed agent` are identity-bearing subtypes, not mere score bands.
2. `declared crawler` is not automatically trustworthy.
3. `user-triggered assistant/fetcher` should be separate because `robots.txt` semantics and operator expectations differ.
4. `adversary-sim` must remain a separate telemetry class with different retention policy.

# Recommended Telemetry Families Shuma Should Collect

## 1. Request classification and identity telemetry

For each request or bounded aggregate, Shuma should be able to expose:

1. traffic lane,
2. claimed identity class,
3. verification method,
4. verification result,
5. confidence band,
6. signal families that contributed to classification,
7. whether the lane is exact, derived, or estimated,
8. whether the final treatment came from a global non-human policy stance, category default, or named identity rule.

This is the foundation for every later operator summary.

## 2. Action-context telemetry

Inspired by reCAPTCHA v3 and Turnstile Analytics, Shuma should record per protected action or route family:

1. route or action family,
2. decision path,
3. friction issued,
4. solve/pass/fail/escalate/deny outcome,
5. execution mode (`enforced` or `shadow`),
6. whether the request was forwarded or short-circuited.

This is much more useful than one global site-wide score.

## 3. Human-friction telemetry

Shuma should collect, per friction family and per relevant route family:

1. likely-human friction issuance rate,
2. solve/pass rate,
3. fail rate,
4. abandonment rate,
5. p50/p95 solve latency,
6. interactive versus non-interactive success where meaningful,
7. challenge escalation rate after initial low-friction paths.

This is one of the most important areas because Shuma's stated goal is minimum human burden.

## 4. Suspicious-traffic cost telemetry

Shuma should collect bounded summaries for suspicious or likely-automated traffic:

1. requests received,
2. requests forwarded,
3. bytes served,
4. approximate origin work or origin-proxy cost,
5. tarpit or maze bytes served,
6. low-cost content served instead of full HTML/JS,
7. observed crawl rate or burst pressure.

This should be segmented by traffic lane, because a verified agent requesting markdown is very different from an unverified crawler burning through rendered pages.

## 5. Defense-effectiveness telemetry

For each defence family, Shuma should summarize:

1. candidates seen,
2. triggers,
3. passes,
4. failures,
5. escalations,
6. denials,
7. bans,
8. repeat offenders,
9. suspicious traffic still forwarded after the defence fired,
10. likely-human traffic affected by the defence.

This creates the operator decision surface Shuma is currently missing.

## 6. Dimensioned operator analytics

Borrowing directly from Cloudflare Bot Analytics and Turnstile Analytics, Shuma should offer bounded drill-down by:

1. country,
2. ASN,
3. hostname,
4. path or route family,
5. user agent family,
6. identity class,
7. verification source,
8. browser family,
9. fingerprint family,
10. attack or detection family.

These should be bounded hot-read dimensions, not unbounded raw tails.

## 7. Misclassification and tuning telemetry

Cloudflare's Bot Feedback Loop is a strong reminder that operator feedback is part of the telemetry system.

Shuma should eventually support telemetry for:

1. suspected false positives,
2. suspected false negatives,
3. manual operator feedback on challenge or ban quality,
4. later future-controller recommendations based on those rates.

Pre-launch, this can remain a future surface, but the telemetry model should reserve a place for it.

## 8. Verified-agent and crawler-etiquette telemetry

For agentic-era readiness, Shuma should collect:

1. robots compliance state for declared crawlers,
2. content-signal compatibility when Shuma later adopts content-use signaling,
3. content format served (`html`, `markdown`, `text`, maze, tarpit),
4. token or byte weight where Shuma can estimate it cheaply,
5. directory freshness and verification freshness for signed traffic,
6. replay, expired-signature, and signature-failure telemetry for verified identity.

This should also make operator intent visible: when a verified or signed identity is denied, Monitoring should make it clear whether that denial came from a deliberate "block all non-human traffic" posture or from a more specific identity/category rule.

# Best-Known Differentiation Strategy For Shuma

The research strongly suggests a five-layer differentiation strategy.

## Layer 1: Positive identity

Use when available:

1. Web Bot Auth,
2. HTTP Message Signatures,
3. directory-backed signed-agent identity,
4. provider-verified bot directories.

This is the cleanest signal for declared beneficial automation.

## Layer 2: Provider verification

For known crawlers and fetchers:

1. published IP ranges,
2. reverse DNS and forward DNS validation,
3. known user-agent tokens,
4. provider-specific detection IDs where available.

This is especially relevant for Google-style common, special-case, and user-triggered fetchers.

## Layer 3: Browser and fingerprint evidence

For undeclared or suspicious traffic:

1. JS or browser execution evidence,
2. fingerprint consistency,
3. JA3 or JA4 style transport fingerprints,
4. impossible transitions and missing persistence markers,
5. header-order or protocol-shape anomalies.

## Layer 4: Behavioral evidence

Use:

1. rate pressure,
2. sequence anomalies,
3. traversal depth,
4. honeypot hits,
5. deep-labyrinth traversal,
6. repeated re-entry after friction or ban.

## Layer 5: Challenge and outcome evidence

For uncertain interactive traffic:

1. low-friction challenge results,
2. higher-friction challenge escalation,
3. solve latency,
4. abandonment,
5. replay or tamper outcomes.

This layered model matches public Cloudflare practice more closely than any single-score design.

# What Shuma Should Not Do

## 1. Do not keep one undifferentiated "bot" bucket

That is already behind industry practice and will age badly as signed agents and user-triggered fetchers grow.

## 2. Do not rely on `User-Agent` and IP alone

Both Cloudflare and Google now make clear that:

1. user-agent strings can be spoofed,
2. IP ranges can be brittle or shared,
3. and positive cryptographic identity is becoming the preferred path for desirable automation.

## 3. Do not collapse identity, suspicion, and authorization into one score

These are different concepts and should be different telemetry dimensions.

## 4. Do not make raw event tails the main operator evidence surface

State-of-the-art operator analytics are bounded, summarized, and dimensioned. Raw rows are for drill-down and diagnostics.

## 5. Do not mix admin or contributor actions into traffic monitoring

This was already corrected in Shuma and should remain a hard rule.

# Recommended First Research-Aligned Telemetry Tranche For Shuma

If Shuma wants the minimum telemetry tranche that is both state-of-the-art-aligned and immediately useful, the recommended first set is:

1. traffic lane summary with exact/derived confidence and clear non-human subclasses,
2. identity verification summary for known bots, crawlers, and later signed agents,
3. human-friction summary by action family and route family,
4. suspicious-cost summary by lane, including bytes and forwarded requests,
5. defence-effectiveness funnel by defence family,
6. dimensioned drill-down for country, ASN, path, UA family, identity class, and detection family.

This tranche would align Shuma with the current direction of Cloudflare-style bot analytics, Google-style crawler categorization, and the emerging verified-agent ecosystem without abandoning its bounded hot-read design.

# Conclusions

The state of the art is now very clear:

1. automated traffic is multi-class,
2. good-faith identity and bad-behavior suspicion are separate dimensions,
3. operator analytics must be contextual and action-aware,
4. cost and etiquette are part of the telemetry model,
5. cryptographic identity is moving from optional curiosity to core infrastructure.

For Shuma, that means the Monitoring overhaul should not merely polish existing charts. It should establish a telemetry contract that is:

1. lane-aware,
2. identity-aware,
3. action-aware,
4. cost-aware,
5. friction-aware,
6. and explicit about exactness.

That is the telemetry model most likely to remain useful in the coming agentic era.
