Date: 2026-03-16
Status: Active

# Scope

This research synthesis answers one coordinated Shuma design question:

1. How should Shuma reduce the predictability and exploitability of bans without flattening severity tiers?
2. How should Shuma remember repeated abuse locally without carrying high false-positive or storage risk?
3. How should Shuma consume or publish optional shared intelligence without turning community data into an unsafe auto-ban mechanism?
4. How should these features fit the already-established agentic-era model:
   - three automation lanes,
   - cryptographic bot identity,
   - low-friction beneficial-agent handling,
   - and bounded controller-style autonomous oversight?

Related context:

- [`2026-03-15-agentic-era-oversight-research-synthesis.md`](2026-03-15-agentic-era-oversight-research-synthesis.md)
- [`../plans/2026-03-15-agentic-era-oversight-design.md`](../plans/2026-03-15-agentic-era-oversight-design.md)
- [`../project-principles.md`](../project-principles.md)

# Current Shuma Baseline

Today, Shuma has:

1. fixed per-family ban durations in config,
2. fixed `600s` short-ban paths for challenge-abuse and tarpit persistence,
3. local ban expiry and pruning behavior,
4. bounded short-lived state patterns elsewhere in the system,
5. no first-class repeat-offender ladder,
6. no central intelligence ingestion or sharing contract,
7. and an emerging oversight-controller model that is intended to tune policy safely outside the request path.

Implication:

1. current ban behavior is easy to probe and synchronize against,
2. repeated bad actors are not yet treated differently from first offenders in a principled, bounded way,
3. and any future shared-intelligence work needs clear governance before it can safely influence enforcement.

# External Findings

## 1. Jitter is a standard way to break synchronized spikes and reduce wasted work

AWS continues to recommend backoff with jitter as a standard resilience pattern. The operational lesson is not limited to retries: fixed timers create clusters and idle gaps, while jitter spreads demand into a steadier rate. AWS also explicitly notes that bursts can come from client behavior, recovery, or even simple cron jobs.

Shuma implication:

1. fixed ban expiry times are a coordination surface for attackers,
2. rigid expiry creates avoidable re-entry waves,
3. bounded jitter is useful not only for unpredictability but also for smoothing defender load.

## 2. Mature ban systems already combine duration growth and randomness

Fail2ban's shipped configuration documents both:

1. `bantime.increment`, which uses prior ban history to increase future ban time for known bad IPs,
2. `bantime.rndtime`, described specifically as a way to prevent "clever" botnets from calculating the exact unban time.

Fail2ban maintainers also recommend incremental bans with a cap rather than immediate permanent bans, and explicitly note that a shorter initial ban can reduce false-positive severity while still allowing earlier intervention. The public issue history also shows a useful nuance: in Fail2ban, randomness historically applied only to repeated bans, and users asked for it to apply to the initial ban too.

Shuma implication:

1. percentage-banded jitter is a research-backed fit,
2. repeat-offender escalation is a mature pattern,
3. and Shuma can improve on existing tools by making first-ban jitter and safety guardrails explicit.

## 3. Local recidive memory is different from long-lived reputation

The mature pattern is not "retain every expired ban forever." It is:

1. keep the active ban separate,
2. keep a small decaying record of repeated offense,
3. use that memory only when deciding a future ban,
4. cap it,
5. and let it decay away.

This is operationally distinct from:

1. carrying request-path suspicion forever,
2. or keeping a durable global history of raw IP behavior.

Shuma implication:

1. a repeat-offender ladder can be cheap if it is short-lived, local, and consulted only on ban issue,
2. sticky suspicion is a different and more invasive feature and should not be bundled into the first tranche.

## 4. Shared intelligence is most useful when it is advisory, structured, and governed

CrowdSec CTI presents a strong model for shared intelligence:

1. it exposes IP reputation, attack history, and behavior data from a global sensor network,
2. it distinguishes `malicious`, `suspicious`, `known`, `safe`, `benign`, and `unknown`,
3. it also exposes nearby-range reputation,
4. and it supports separate remediation and blocklist distribution components.

CrowdSec's current product surface also shows the risks and governance requirements:

1. it has explicit false-positive handling and validation requirements,
2. it distinguishes CTI enrichment from remediation components,
3. and it documents organization-wide decision sync as a separate feature with real blast-radius implications.

Spamhaus DROP shows the opposite end of the confidence spectrum:

1. it is explicitly a "worst of the worst" dataset,
2. it distributes ranges and ASNs, not merely one-off single-IP claims,
3. it is positioned as "drop all traffic",
4. and it is re-evaluated daily with explicit removal linkage.

Shuma implication:

1. shared intelligence should not be one thing,
2. advisory reputation and high-confidence deny feeds need separate contracts,
3. and optional global bans should be reserved for the narrowest, highest-confidence sources.

## 5. The agentic era raises the value of low-friction differentiation

The previously gathered agentic-era research still matters directly here:

1. OpenAI, Anthropic, and Google distinguish training crawlers, search/indexing bots, and user-triggered agents.
2. RFC 9421 is live, the IETF `webbotauth` working group is active, and OpenAI's ChatGPT agent signs outbound requests.
3. Cloudflare and Vercel both now support cryptographically verified bot and agent identity paths.
4. Cloudflare is simultaneously investing in low-friction beneficial-agent paths such as Markdown for Agents and cost-shifting paths such as AI Labyrinth.

Shuma implication:

1. these three new features must not collapse all automation into one punishment bucket,
2. verified beneficial agents need a cheaper path than suspicious automation,
3. and shared intelligence must not downgrade signed beneficial traffic into community-blocklist collateral.

## 6. Autonomous tuning still points to controller + budgets + rollback

The oversight research remains the right control-plane pattern:

1. keep the request path deterministic and Rust-owned,
2. let the controller read bounded evidence,
3. let the agent propose narrow config changes,
4. validate,
5. canary,
6. watch,
7. and roll back when budgets are breached.

Shuma implication:

1. jitter bands, repeat-offender multipliers, and central-intelligence modes are good controller-tunable surfaces,
2. individual per-request decisions should not be delegated to an LLM,
3. and community intelligence promotion to stronger enforcement should be gated by budget evidence and governance.

# Shuma-Specific Research Conclusions

## A. Percentage-banded ban jitter is well justified

Shuma should:

1. preserve base durations by offense family,
2. apply bounded jitter around those base durations,
3. prevent overlap between severity families,
4. and make the jitter deterministic per ban issuance for auditability.

This feature addresses both:

1. exact-expiry probing,
2. and synchronized bot re-entry waves.

## B. Repeat-offender escalation fits Shuma if it stays narrow and local

Shuma should treat repeat-offender logic as:

1. local recidive memory,
2. short-lived,
3. high-confidence-family only,
4. capped,
5. and separate from active bans.

This feature should not be introduced as:

1. general hot-path suspicion,
2. indefinite history,
3. or a multiplier across all ban families.

## C. Central intelligence should start as advisory enrichment, not shared auto-banning

The central-intelligence pattern should begin with:

1. observe-only ingest,
2. enrichment into botness or routing bias,
3. clear source attribution,
4. explicit freshness and confidence semantics,
5. operator-visible blast radius,
6. and signed or authenticated distribution where possible.

High-confidence deny feeds should be treated as a separate class from community reputation feeds.

## D. These three features belong to different horizons

The clean coordination model is:

1. ban jitter = immediate request-plane cost shaping,
2. repeat-offender ladder = short-horizon local memory,
3. central intelligence = medium-horizon advisory intelligence,
4. oversight controller = long-horizon budget tuning.

Keeping those horizons separate is what makes the overall system understandable and safe.

# Research-Backed Design Requirements

R1. Preserve Shuma's three traffic lanes:
- verified beneficial agents,
- declared crawlers/search bots,
- unverified or suspicious automation.

R2. Keep the hot path deterministic and Rust-only.

R3. Implement ban jitter as bounded percentage bands on top of family base durations, not as unbounded random durations.

R4. Make jitter deterministic per ban issuance using a secret-keyed derivation so operators can explain outcomes and attackers cannot predict them.

R5. Enforce non-overlap between severity families so jitter does not erase policy meaning.

R6. Limit repeat-offender ladders to high-confidence ban families first.

R7. Store repeat-offender memory separately from active bans, with short TTLs and capped steps.

R8. Ensure manual unban and successful operator-clearing actions can reset or reduce repeat-offender state.

R9. Treat central intelligence as advisory by default:
- score bias,
- lane/risk bias,
- challenge bias,
- or deny-candidate generation.

R10. Reserve shared hard-block behavior for narrow, high-confidence feeds with explicit governance.

R11. Track source, confidence, freshness, and appeal/removal posture for every central-intelligence signal.

R12. Keep beneficial signed agents out of coarse reputation traps unless they violate declared or observed policy locally.

R13. Expose jitter, ladder, and intelligence outcomes in monitoring so the oversight controller can tune them against human-friction and bot-cost budgets.

R14. Let the oversight controller tune policy envelopes, not individual live decisions.

# Recommended Product Stance

Shuma should implement these features as one coordinated system:

1. jitter makes expiry harder to learn and harder to synchronize against,
2. the repeat-offender ladder makes repeated local abuse progressively more expensive,
3. central intelligence gives Shuma outside memory without replacing local truth,
4. and the oversight controller tunes the envelopes and promotion thresholds over time.

That combination fits the agentic era better than any one mechanism in isolation.

# Source Links

- [AWS Builders' Library: Timeouts, retries, and backoff with jitter](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/)
- [AWS Architecture Blog: Exponential Backoff and Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
- [Fail2ban `jail.conf` default configuration](https://raw.githubusercontent.com/fail2ban/fail2ban/master/config/jail.conf)
- [Fail2ban discussion: incremental bans over longer time spans](https://github.com/fail2ban/fail2ban/discussions/2952)
- [Fail2ban discussion: dynamic ban time and repeated attempts](https://github.com/fail2ban/fail2ban/discussions/3700)
- [Fail2ban issue: `bantime.rndtime` and initial ban time](https://github.com/fail2ban/fail2ban/issues/2834)
- [CrowdSec CTI overview](https://docs.crowdsec.net/cti/)
- [CrowdSec CTI object taxonomy and range reputation](https://docs.crowdsec.net/u/cti_api/taxonomy/cti_object/)
- [CrowdSec blocklist mirror](https://docs.crowdsec.net/u/bouncers/blocklist-mirror/)
- [CrowdSec remediation sync](https://docs.crowdsec.net/u/console/remediation_sync/)
- [CrowdSec CTI troubleshooting and false-positive workflow](https://docs.crowdsec.net/u/troubleshooting/cti/)
- [Spamhaus DROP](https://www.spamhaus.org/blocklists/do-not-route-or-peer/)
- [OpenAI crawler overview](https://developers.openai.com/api/docs/bots)
- [OpenAI ChatGPT agent allowlisting](https://help.openai.com/en/articles/11845367-chatgpt-agent-allowlisting/)
- [Anthropic crawler controls for website owners](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler)
- [Google user-triggered fetchers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-user-triggered-fetchers)
- [RFC 9421: HTTP Message Signatures](https://www.rfc-editor.org/rfc/rfc9421.html)
- [IETF Web Bot Auth working group](https://datatracker.ietf.org/wg/webbotauth/about/)
- [Cloudflare AI Labyrinth](https://blog.cloudflare.com/ai-labyrinth/)
- [Cloudflare Markdown for Agents](https://blog.cloudflare.com/markdown-for-agents/)
- [Cloudflare Web Bot Auth](https://blog.cloudflare.com/web-bot-auth/)
- [Cloudflare crawl-to-click gap research](https://blog.cloudflare.com/crawlers-click-ai-bots-training/)
- [Kubernetes controllers](https://kubernetes.io/docs/concepts/architecture/controller/)
