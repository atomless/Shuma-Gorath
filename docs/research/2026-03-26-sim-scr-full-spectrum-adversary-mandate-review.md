Date: 2026-03-26
Status: Completed

# SIM-SCR Full-Spectrum Adversary Mandate Review

## Question

Given the clarified product requirement that Shuma's adversary sim must cover the full spectrum of non-human attackers across the Scrapling and LLM lanes, how should the Scrapling capability matrix be judged, and when is it acceptable to leave an upstream Scrapling capability unused?

## Clarified source-of-truth requirement

The product requirement is now explicit:

1. the adversary-sim harness is supposed to represent the full spectrum of potential non-human attackers,
2. Scrapling plus the later LLM lane should cover that spectrum as completely as Shuma can make truthful,
3. and Scrapling must not remain a polite half-adversary if upstream Scrapling exposes capabilities that would materially increase attacker power against Shuma defenses.

That means the old comfort of "current request-native ownership" is no longer sufficient as a default reason to leave upstream Scrapling power unused.

## Governing test for every Scrapling capability

Each upstream Scrapling capability must now be evaluated against this question:

Does this capability materially increase effective adversary power against Shuma defenses, or does it close part of the non-human attacker spectrum that the current harness does not already cover truthfully?

If the answer is yes, the capability stays in scope for the active Scrapling maturity work unless one of the following is overtly documented:

1. the capability does not meaningfully increase adversary power against Shuma's current owned surfaces,
2. the capability is already covered truthfully by another adversary lane with explicit proof and no resulting spectrum gap,
3. the capability would be unsafe or untruthful to claim in the current harness until additional runtime or receipt work lands,
4. or the capability is operationally unjustified for now with a concrete reconsideration trigger.

Browser-class or stealth classification alone is not a sufficient exclusion reason.

## Consequence for the current matrix

The earlier `SIM-SCR-CAP-1` matrix was useful as an omission ledger, but it is no longer sufficient as the authoritative closure for `SIM-SCR-FULL-1A`.

Under the clarified full-spectrum mandate:

1. `DynamicFetcher` and `StealthyFetcher` can no longer be assigned away merely because the current Scrapling lane started as request-native,
2. Cloudflare or Turnstile-style solving cannot be dismissed solely because Shuma's current receipts do not yet prove it; instead the repo must decide whether the capability materially increases adversary power against Shuma's challenge surfaces and, if so, add the proof path,
3. proxy support cannot remain excluded merely because the current shared-host local mainline is simpler without it if distributed-origin behavior would materially change attacker power,
4. and Camoufox-style shaping must be judged on whether it materially increases power against Shuma defenses rather than on the previous "too far beyond remit" phrasing alone.

Some of these capabilities may still end up excluded, but only through overt reasoning under the power-increase test above.

## Taxonomy ownership versus capability use

The clarified requirement also tightens an important distinction:

1. taxonomy ownership still matters,
2. but taxonomy purity must not become a pretext for suppressing real attacker capability.

If Scrapling needs browser or stealth runtime to attack a Shuma surface truthfully, the repo should prefer extending the taxonomy, receipts, and operator evidence model over artificially weakening the Scrapling lane.

The separate `automated_browser`, `browser_agent`, and `agent_on_behalf_of_human` questions may remain open, but they must not be used as a parking lot for attacker-relevant Scrapling power that materially strengthens attacks on Shuma's currently owned surfaces.

## Decision

Reopen `SIM-SCR-FULL-1A` under the stronger full-spectrum mandate.

Practical consequences:

1. the active matrix work must be rerun under the "materially increases adversary power" test,
2. the earlier same-day interpretation that `SIM-SCR-FULL-1A` was already closed is superseded,
3. `SIM-SCR-FULL-1B` must implement every retained capability rather than only the polite request-native subset,
4. `SIM-SCR-CHALLENGE-2C` and `SIM-SCR-BROWSER-1` must no longer serve as a place to defer attacker-relevant Scrapling power that belongs in the active full-spectrum maturity tranche,
5. and any residual omission must be conspicuous, reasoned, and paired with a reconsideration trigger.

## Verification

This slice was docs-only.

Evidence:

- `git diff --check`
