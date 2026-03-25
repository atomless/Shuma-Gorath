Date: 2026-03-25
Status: Proposed

Related context:

- [`2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md`](2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md)
- [`../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`](../plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md)
- [`../plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md`](../plans/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-plan.md)
- [`../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`](../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md)
- [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [Scrapling dynamic fetcher docs](https://scrapling.readthedocs.io/en/latest/fetching/dynamic.html)
- [Scrapling stealthy fetcher docs](https://scrapling.readthedocs.io/en/latest/fetching/stealthy/)

# Scrapling Full Attacker-Capability Principle Review

## Question

Should Shuma treat fuller upstream Scrapling capability as a reluctant later contingency, or as the default expectation for Scrapling-owned surfaces?

## Current planning drift

The repo already has the right instinct about attacker-faithfulness, but some of the current planning language is still too conservative in practice.

The March 24 Scrapling planning chain correctly says:

1. adversary lanes must be attacker-faithful,
2. upstream capability claims are not the same as Shuma proof,
3. and browser or stealth Scrapling should not be adopted without receipts, safety boundaries, and a truthful owned-surface contract.

But parts of the backlog still phrase broader Scrapling capability as something Shuma should reopen only if a later local receipt gap forces it.

That is not the strongest long-term principle.

## Why the current conservative framing is not the best principle

Real attackers do not intentionally leave attacker-relevant capability unused once it becomes practical and valuable.

If a tool like Scrapling expands its real attacker utility, then the truthful defensive posture is not:

1. wait until Shuma's current narrower sim produces a local proof of insufficiency,
2. then reluctantly widen it.

The stronger posture is:

1. continuously track upstream attacker-relevant capability,
2. map it against Scrapling-owned surfaces,
3. adopt it by default where Scrapling should own that pressure,
4. and make every omission explicit.

Otherwise the adversary lane risks becoming a bounded but stale half-sim rather than a maintained attacker-faithful lane.

## Correct governing principle

Shuma should adopt all attacker-relevant upstream Scrapling capability for Scrapling-owned surfaces by default, subject only to explicit exclusions.

Those exclusions must be justified as one of:

1. not actually relevant to a Scrapling-owned surface,
2. explicitly assigned to another lane,
3. unsafe to run inside Shuma's bounded harness,
4. not yet receipt-verifiable in Shuma's runtime and observability path,
5. or operationally unjustified for now with a recorded reconsideration trigger.

This is stronger than "adopt every upstream Scrapling feature."

The correct filter is not feature breadth. The correct filter is attacker relevance to Scrapling-owned surfaces.

## Important distinction

This principle still preserves two important boundaries:

1. It does not mean Shuma should blur Scrapling into every non-human category. `automated_browser`, `browser_agent`, and `agent_on_behalf_of_human` remain separate ownership questions.
2. It does not mean marketing claims count as proof. Upstream capability must still be translated into Shuma runtime contracts, receipts, API surfaces, and verification targets before it becomes part of Shuma's truth basis.

## Consequence for planning and backlog

The repo should stop treating fuller attacker-relevant Scrapling capability as a mere gap-triggered contingency.

Instead it should:

1. add an explicit active lane that freezes the upstream attacker-capability matrix and an omission ledger for Scrapling-owned surfaces,
2. keep browser or stealth Scrapling implementation as the next bounded follow-on where that matrix says Scrapling should own it,
3. keep `automated_browser` browser-runtime adoption separate from broader owned-surface hardening,
4. and keep the later LLM attacker runtime blocked until Scrapling's owned-surface capability matrix is either adopted or explicitly excluded.

## Decision

The stronger principle should now become source-of-truth:

1. Shuma should continuously track upstream Scrapling capability.
2. Shuma should adopt attacker-relevant upstream Scrapling capability by default for Scrapling-owned surfaces.
3. Shuma should require explicit, auditable exclusions for any attacker-relevant upstream capability it does not adopt.
4. Shuma should still require receipt-backed proof before any adopted capability becomes part of the claimed adversary truth surface.

## Result

The next planning question is no longer only:

1. "is request-native Scrapling currently enough?"

It is now:

1. which upstream Scrapling capabilities are attacker-relevant for Scrapling-owned surfaces,
2. which of those Shuma already adopts,
3. which of those are explicitly excluded and why,
4. which require browser or stealth runtime,
5. and which remain a separate category-ownership question rather than a Scrapling-owned-surface question.
