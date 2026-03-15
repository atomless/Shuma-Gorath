# Agentic-Era Oversight Research Synthesis

Date: 2026-03-15
Status: Active

## Scope

This research synthesis answers one practical question for Shuma's next operating era:

1. How should Shuma evolve from a request-path bot defense into a bot-defense system that can be observed, tuned, red-teamed, and maintained by agents without increasing human burden?
2. Which architectural patterns best fit Shuma's current codebase, current deployment targets, and the emerging split between beneficial authenticated agents and hostile or undeclared automation?

## Current Shuma Baseline

The repository already contains several of the primitives needed for an autonomous oversight model:

1. Backend-owned adversary-sim lifecycle and lease/idempotency controls exist today.
2. Monitoring freshness, retention health, security/privacy state, and query-budget signals already have explicit operator-facing contracts.
3. Hot-read telemetry documents already exist for low-cost operator reads across shared-host and Fermyon targets.
4. Config validation and config write surfaces already exist and are narrow enough to support bounded policy changes.
5. Robots and AI-policy surfaces already exist, but they remain advisory rather than authoritative.

The current gap is not "more telemetry" or "more knobs." The gap is a safe controller that can reconcile live traffic against explicit budgets, schedule verification, and apply bounded changes with rollback.

## External Findings

### 1. Site economics are increasingly dominated by AI crawler asymmetry

Cloudflare's 2025 reporting shows the core problem clearly:

1. AI crawling is no longer small enough to dismiss as background noise.
2. By mid-2025, Cloudflare reported that training-oriented crawling accounted for about 80% of AI crawler activity.
3. Cloudflare's July 2025 crawl-to-referral reporting still showed severe asymmetry between crawler volume and publisher benefit:
   - Anthropic was still around `38,065:1` crawls per referred visit.
   - OpenAI was still around `1,091:1` crawls per referred visit.

Shuma implication:

1. "Bot defense" must include explicit origin-cost and bandwidth-cost governance, not only abuse detection.
2. The operating goal should be to reduce bytes, CPU, and origin work served to suspicious automation, not merely to classify requests after serving them.

### 2. `robots.txt` remains necessary but is not a security control

RFC 9309 is explicit that the Robots Exclusion Protocol is not authorization or access control. Google's current crawler documentation also states that user-triggered fetchers generally ignore `robots.txt`.

Shuma implication:

1. Robots and AI preference signals should remain first-class communication surfaces.
2. They must not be treated as the enforcement plane.
3. Shuma needs a separate enforcement and identity model for agentic traffic.

### 3. "Bot" is no longer one traffic class

Current official platform guidance now distinguishes at least three meaningful automated cohorts:

1. training crawlers,
2. search/indexing or assistant retrieval bots,
3. user-triggered fetchers or agents acting on behalf of a user.

OpenAI, Anthropic, and Google all expose this split in different ways. Google explicitly distinguishes user-triggered fetchers from standard crawlers. OpenAI now documents separate crawler identities and a distinct ChatGPT agent allowlisting path. Anthropic documents crawler controls for site owners separately from other traffic classes.

Shuma implication:

1. Shuma should model automated traffic as separate policy lanes rather than one "good bot / bad bot" flag.
2. The correct long-term split is:
   - verified beneficial agents,
   - declared but unauthenticated crawlers,
   - undeclared or suspicious automation.

### 4. Cryptographic bot identity is becoming practical

RFC 9421 standardizes HTTP Message Signatures. The IETF `webbotauth` working group exists specifically to define bot authentication mechanisms. OpenAI's ChatGPT agent allowlisting documentation already describes signed requests. Cloudflare and Vercel now both expose signed-agent or verified-bot support for operator control.

Shuma implication:

1. User-agent strings and published IP ranges remain useful signals, but they are not a durable primary identity plane.
2. Shuma should plan for a verified-agent lane that prefers signed requests or equivalent cryptographic identity over legacy UA-only or CIDR-only allow rules.
3. This verified-agent lane should be low-friction and low-cost, because it is the place where future agentic conversion-positive traffic will live.

### 5. Low-friction agent handling is not only about allowing traffic

Cloudflare's AI Labyrinth work reinforces a deception and cost-shifting path for undeclared automation. Cloudflare's Markdown for Agents and Content Signals work reinforce the complementary idea that beneficial automated consumers should be able to fetch cheaper, more structured representations than full human-oriented HTML plus JS execution.

Shuma implication:

1. The future fit is two-sided:
   - increase cost for suspicious automation,
   - reduce defender and origin cost for authenticated or policy-compliant automation.
2. Shuma should eventually expose a low-cost agent content surface, not only a block/challenge surface.

### 6. Safe autonomous operations follow controller patterns, not free-form agent behavior

Kubernetes controllers, Google SRE toil-elimination guidance, canarying, and SLO alerting all point toward the same pattern:

1. define explicit desired state,
2. reconcile current state against budgets,
3. apply small changes,
4. observe,
5. roll back when budgets are breached.

Shuma implication:

1. The first autonomous tuning system should be a bounded reconciler.
2. LLMs should initially act as advisors or proposal generators, not unrestricted production mutators.
3. The applier should remain deterministic and schema-validated.
4. The strong pattern is controller + budgets + canary + rollback, not free-form agent autonomy.

### 7. Scheduling must remain platform-aware but architecture-neutral

Fermyon supports cron-triggered Spin jobs. Kubernetes supports `CronJob`. Traditional single-host deployments support systemd timers or equivalent schedulers. These are all reasonable execution adapters, but they should not define the product architecture.

Shuma implication:

1. Shuma should define one internal oversight reconcile contract.
2. Multiple scheduler adapters can call that contract:
   - host-side supervisor,
   - platform cron,
   - long-term hosted control plane.

## Research-Backed Design Requirements

R1. Keep request-path defense deterministic, Rust-owned, and free of LLM dependencies.

R2. Add a distinct oversight plane that reads one bounded budget snapshot instead of forcing agents to query many operator endpoints directly.

R3. Treat automated traffic as at least three classes:
- verified beneficial agents,
- declared crawlers/search bots,
- unverified or suspicious automation.

R4. Preserve robots and AI preference signaling, but keep it advisory and separate from enforcement.

R5. Add a verified-agent identity lane that can consume cryptographic identity signals when available.

R6. Define explicit budgets for:
- human friction,
- suspicious-traffic cost,
- beneficial-agent success,
- telemetry truthfulness,
- change safety.

R7. Restrict autonomous mutation to a small allowlist of reversible config families until replay, rollback, and ledgering are mature.

R8. Require lease-safe, idempotent, auditable control paths for oversight operations, matching the repo's existing adversary-sim control discipline.

R9. Make adversary simulation part of the control loop: every non-trivial policy change should be verified against deterministic red-team evidence.

R10. Keep scheduler adapters replaceable so hosted Shuma can grow from per-deployment supervisors into a fleet-level control plane without rewriting the contract.

R11. The controller should emit structured policy patches with expected impact and confidence metadata, not prose-only advice.

R12. The first autonomous version should follow "LLM advisor, deterministic applier" rather than direct free-form agent mutation.

## Shuma-Specific Conclusions

The evidence points toward a three-plane model:

1. Request plane:
   - current Shuma runtime defenses,
   - classification and enforcement in Rust,
   - no agent dependency.
2. Evidence plane:
   - hot-read monitoring documents,
   - retention and cost governance,
   - adversary-sim telemetry,
   - future oversight budget snapshot.
3. Oversight plane:
   - scheduler-owned periodic reconcile loop,
   - bounded config proposal and validation,
   - adversary verification,
   - rollback and audit ledger.

The most important architectural choice is this:

1. start with "agent advisor, deterministic applier,"
2. not "fully autonomous agent writes config directly."

That choice fits:

1. current Shuma architecture,
2. current operator trust expectations,
3. platform constraints,
4. and the repo's existing discipline around explicit contracts, low surprise, and auditable control planes.

## Recommended Sequence

1. Add an explicit oversight design and implementation plan.
2. Add a materialized budget snapshot document and low-risk reconcile contract.
3. Ship observe-only and recommend-only modes first.
4. Add canary-apply for a narrow set of low-risk knobs.
5. Add verified-agent identity and low-cost content surfaces.
6. Evolve from deployment-local schedulers to a Shuma-hosted control plane once the reconcile loop is trustworthy.

## Source Links

- Kubernetes, Controllers: https://kubernetes.io/docs/concepts/architecture/controller/
- Google SRE Workbook, Eliminating Toil: https://sre.google/workbook/eliminating-toil/
- Google SRE Workbook, Canarying Releases: https://sre.google/workbook/canarying-releases/
- Google SRE Workbook, Alerting on SLOs: https://sre.google/workbook/alerting-on-slos/
- OpenTelemetry specification principles: https://opentelemetry.io/docs/specs/otel/specification-principles/
- RFC 9309, Robots Exclusion Protocol: https://www.rfc-editor.org/rfc/rfc9309
- RFC 9421, HTTP Message Signatures: https://www.rfc-editor.org/rfc/rfc9421.html
- IETF Web Bot Auth working group: https://datatracker.ietf.org/wg/webbotauth/about/
- IETF AI Preferences working group: https://datatracker.ietf.org/wg/aipref/about/
- OpenAI crawler documentation: https://platform.openai.com/docs/bots
- OpenAI ChatGPT agent allowlisting: https://help.openai.com/en/articles/11845367
- Anthropic crawler controls for website owners: https://support.anthropic.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler
- Google user-triggered fetchers: https://developers.google.com/search/docs/crawling-indexing/google-user-triggered-fetchers
- Cloudflare, AI Week 2025 wrap-up: https://blog.cloudflare.com/ai-week-2025-wrapup/
- Cloudflare, The crawl-to-click gap: https://blog.cloudflare.com/crawlers-click-ai-bots-training/
- Cloudflare, AI Labyrinth: https://blog.cloudflare.com/ai-labyrinth/
- Cloudflare, Markdown for Agents: https://blog.cloudflare.com/markdown-for-agents/
- Cloudflare, Content Signals and AI crawler controls: https://blog.cloudflare.com/declaring-your-aindependence-block-ai-bots-scrapers-and-crawlers-with-a-single-click/
- Cloudflare, Web Bot Auth: https://blog.cloudflare.com/web-bot-auth/
- Vercel, Web Bot Auth support for bot verification: https://vercel.com/changelog/vercels-bot-verification-now-supports-web-bot-auth
- Fermyon, cron jobs for Spin apps: https://developer.fermyon.com/wasm-functions/using-cron-jobs
