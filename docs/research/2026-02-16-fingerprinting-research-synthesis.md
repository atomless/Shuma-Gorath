# Fingerprinting Research Synthesis (`R-FP-01`..`R-FP-09`)

Date: 2026-02-16  
Status: Completed research tranche

## Scope

This dossier completes the paper-review TODOs under `Fingerprinting, JS Verification, and CDP-Adjacent Detection` and converts findings into concrete Shuma implementation work.

## Source Coverage and Confidence

| ID | Paper | Access level | Primary source(s) |
| --- | --- | --- | --- |
| `R-FP-01` | Eckersley 2010 (PETS) | Full text | EFF paper mirror (`browser-uniqueness.pdf`) |
| `R-FP-02` | Acar et al. 2014 (CCS) | Full text | ACM DOI + full-text mirror |
| `R-FP-03` | Vastel et al. 2018 (FP-STALKER) | Abstract + partial full-text excerpts | IEEE DOI program abstract + mirrored paper excerpts |
| `R-FP-04` | Jonker/Krumnow/Vlot 2019 (ESORICS) | Full text | Authors' PDF mirror |
| `R-FP-05` | Azad et al. 2020 (Web Runner 2049) | Full text | PMC open-access article |
| `R-FP-06` | Iliou et al. 2021 (DTRAP) | Full text | BURO repository PDF |
| `R-FP-07` | Zhao et al. 2024 (Computers & Security) | Abstract + metadata | ScienceDirect abstract/metadata page |
| `R-FP-08` | Venugopalan et al. 2024 (FP-Inconsistent) | Full text | arXiv + ar5iv text rendering |
| `R-FP-09` | Bursztein et al. 2016 (Picasso) | Full text | SPSM paper PDF mirror |

Note: `R-FP-03` and `R-FP-07` had partial-access constraints in this environment; conclusions for those are marked conservative.

## Completed Research Checklist

- [x] `R-FP-01` Review Peter Eckersley, "How Unique Is Your Web Browser?" (PETS 2010)
- [x] `R-FP-02` Review Acar et al., "The Web Never Forgets" (CCS 2014)
- [x] `R-FP-03` Review Vastel et al., "FP-STALKER" (IEEE S&P 2018)
- [x] `R-FP-04` Review Jonker/Krumnow/Vlot, "Fingerprint Surface-Based Detection of Web Bot Detectors" (ESORICS 2019)
- [x] `R-FP-05` Review Azad et al., "Web Runner 2049: Evaluating Third-Party Anti-bot Services"
- [x] `R-FP-06` Review Iliou et al., "Detection of advanced web bots by combining web logs with mouse behavioural biometrics" (DTRAP 2021)
- [x] `R-FP-07` Review Zhao et al., "Toward the flow-centric detection of browser fingerprinting" (Computers & Security 2024)
- [x] `R-FP-08` Review Venugopalan et al., "FP-Inconsistent: Detecting Evasive Bots using Browser Fingerprint Inconsistencies" (2024)
- [x] `R-FP-09` Review Bursztein et al., "Picasso: Lightweight Device Class Fingerprinting for Web Clients" (SPSM 2016)

## Paper-by-Paper Findings and Shuma Implications

### `R-FP-01` Eckersley 2010

Observed findings:
- Browser fingerprints are high-entropy at web scale (`83.6%` uniqueness in sampled browsers).
- Combining attribute families (UA headers + plugins + fonts/screen/timezone) sharply increases uniqueness (`94.2%` in sampled cohort).
- Plugin-family entropy alone was measured at `18.1` bits, showing why single-family over-weighting is risky for both privacy and stability.

Shuma implications:
- Use weighted feature families, not one-shot static fingerprints.
- Cap confidence contribution per family to avoid brittle overfitting to rare attributes.
- Keep replay windows short and cohort-aware because legitimate fingerprints drift.

### `R-FP-02` Acar et al. 2014

Observed findings:
- Fingerprinting scripts were widespread in top sites (`>5%` of top 10k had explicit canvas fingerprinting at study time).
- Third-party providers dominated deployment, creating concentrated detector surfaces and reuse patterns.
- Tracking stacks used multiple storage and recovery channels ("evercookie"-style persistence).

Shuma implications:
- Add first-class detection for multi-store persistence patterns in JS verification/challenge paths.
- Treat third-party script re-use and static probe signatures as attacker-observable surfaces.
- Keep fingerprinting logic modular and rotatable to avoid static detector signatures.

### `R-FP-03` Vastel et al. 2018 (FP-STALKER)

Observed findings:
- Browser fingerprints evolve over time; a purely static "same hash == same client" model is weak.
- Attribute-specific stability differs (some fields churn quickly, some are durable).
- Time-aware linking materially outperforms single-snapshot linking.

Shuma implications:
- Model temporal coherence per attribute class (fast-churn vs slow-churn) rather than global exact-match checks.
- Emit dedicated detection IDs for impossible short-window transitions.
- Use bounded TTL state stores for temporal checks to retain low host cost.

### `R-FP-04` Jonker/Krumnow/Vlot 2019

Observed findings:
- Bot detectors expose recognizable probing surfaces that can themselves be fingerprinted.
- The paper demonstrates attacker-side success uplift (reported `12.8%`) when adapting behavior after detector recognition.
- Static probe suites become liabilities over time.

Shuma implications:
- Treat detector-surface minimization and rotation as required, not optional.
- Separate feature collection from enforcement so probe rotation does not change policy semantics.
- Add regression tests that simulate detector fingerprinting attempts against Shuma JS/CDP paths.

### `R-FP-05` Azad et al. 2020 (Web Runner 2049)

Observed findings:
- Commercial anti-bot products are bypassable under adaptive adversaries, especially with static checks.
- False-positive tradeoffs and UX friction remain core constraints for robust deployment.
- Correlated signal fusion is stronger than single-point controls.

Shuma implications:
- Keep Shuma policy orchestration internal and fused (fingerprint + sequence + rate + challenge outcomes).
- Reserve hard actions for corroborated multi-signal evidence.
- Prefer progressive escalation ladders over single-trigger bans.

### `R-FP-06` Iliou et al. 2021

Observed findings:
- Combining web-log features with mouse-behavior biometrics improves bot discrimination.
- Low-friction behavior features can be useful when fused with traditional network/browser data.

Shuma implications:
- Add optional behavior micro-signals in challenge/JS contexts where human interaction already exists.
- Keep this path opt-in and bounded by privacy/retention policy.
- Never depend on behavior biometrics as the sole gate for enforcement.

### `R-FP-07` Zhao et al. 2024

Observed findings:
- Flow-centric detection can identify browser fingerprinting activity with strong measured F-scores (reported mid/high `90%` range).
- Request-flow context captures patterns missed by static endpoint signatures.

Shuma implications:
- Add flow-level JS telemetry normalization (API family + request sequence shape + timing windows).
- Use flow-centric flags as advisory signals into botness scoring.
- Keep per-flow state bounded with short TTL and coarse aggregation keys.

### `R-FP-08` Venugopalan et al. 2024 (FP-Inconsistent)

Observed findings:
- Cross-attribute inconsistency checks effectively expose evasive bots with limited feature sets.
- Large-scale empirical corpus (`507,080` fingerprints; `306` known bot browsers) supports practical deployment value.
- Early filtering can reduce analyst workload while preserving true-positive recall.

Shuma implications:
- Prioritize inconsistency-class detection IDs (UA/client hints/runtime/transport mismatches).
- Explicitly separate "strong impossible mismatch" from "soft suspicious mismatch."
- Tie strong mismatches to earlier escalation only when corroborated (sequence/replay/challenge failures).

### `R-FP-09` Bursztein et al. 2016 (Picasso)

Observed findings:
- Lightweight rendered-surface fingerprints can classify device classes at large scale (`~52M` fingerprints reported).
- Class-level fingerprints can be robust enough for abuse-defense use while being lighter than full persistent tracking schemes.

Shuma implications:
- Add an optional challenge-bound, short-lived device-class marker for replay resistance.
- Use device-class continuity checks for session integrity, not long-term user identity.
- Keep markers ephemeral, signed, and strictly TTL-bounded.

## Cross-Paper Guiding Principles (Record in Plans and Implementation)

1. Use fused weak-signal models; avoid single-signal hard enforcement.
2. Emphasize inconsistency and temporal-coherence checks over static fingerprint hashes.
3. Minimize and rotate detector surface; assume adversaries fingerprint detectors.
4. Bound all fingerprint state by TTL and scope to active defence windows.
5. Use challenge-bound ephemeral markers for replay defense; avoid persistent identity semantics.
6. Keep enforcement explainable through stable detection IDs and contribution traces.
7. Keep privacy risk bounded (pseudonymization, minimization, retention limits).
8. Preserve internal-first fallback when edge telemetry is unavailable or untrusted.

## Derived Implementation Backlog (from this Research Tranche)

- Add feature-family entropy budgeting and per-family confidence caps for fingerprint scoring.
- Add temporal coherence modeling with per-attribute churn classes and impossible-transition detection IDs.
- Add detector-surface rotation strategy for JS/CDP probes with versioned rollout controls.
- Add multi-store persistence-abuse detection signals (cookie/localStorage/sessionStorage/IndexedDB recovery patterns).
- Add cross-layer inconsistency rules (UA vs client hints vs runtime APIs vs transport fingerprints).
- Add flow-centric fingerprint telemetry extraction and bounded per-flow aggregation.
- Add optional challenge-bound device-class marker path (Picasso-inspired) with strict TTL and replay binding.
- Add optional low-friction behavior micro-signals in challenge contexts (mouse/timing), with privacy guardrails.
- Add evasive-regression test harness for detector fingerprinting, temporal drift, and inconsistency bypass attempts.
- Add privacy/retention policy controls and operator documentation for fingerprint data handling.

## Sources

- https://panopticlick.eff.org/static/browser-uniqueness.pdf
- https://doi.org/10.1145/2660267.2660347
- https://doi.org/10.1109/SP.2018.00008
- https://doi.org/10.1007/978-3-030-29962-0_28
- https://pmc.ncbi.nlm.nih.gov/articles/PMC7338186/
- https://doi.org/10.1145/3447815
- https://doi.org/10.1016/j.cose.2023.103642
- https://arxiv.org/abs/2406.07647
- https://doi.org/10.1145/2994459.2994467
