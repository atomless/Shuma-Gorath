# 🐙 Challenge & Human Verification Strategy

This document outlines an edge‑only, self‑hosted approach to human verification in Shuma‑Gorath.
It is designed for high usability, strong security, and accessibility without relying on third‑party services.

## 🐙 Goals

- Minimize friction for legitimate humans
- Make automation materially more expensive for bots
- Keep verification fully edge‑served and self‑hosted
- Provide an accessible path that is not a weaker bypass

## 🐙 Non‑Goals

- A single, permanent “bot‑proof” puzzle
- Dependence on third‑party CAPTCHA vendors or cloud APIs

## 🐙 Design Principles

- Risk‑gated: only challenge sessions with elevated risk signals
- Edge‑only: all challenge generation and verification happens at the edge
- Short‑lived: challenges expire quickly and are single‑use
- Session‑bound: answers are tied to a specific request/session
- Replay‑resistant: tokens cannot be reused
- Accessible parity: alternate modalities must be equivalent in strength
- No speed traps: avoid time‑based requirements that disadvantage users

## 🐙 Accessibility Requirements

CAPTCHAs can block users with disabilities, and WCAG requires text alternatives for non‑text content and recommends providing alternative modalities for CAPTCHAs.
We will provide an accessible modality that serves the same purpose and is validated with the same rigor.

## 🐙 Threat Model (Practical)

- Headless automation with real browsers and full JS execution
- Session replay, token reuse, and cookie theft
- CAPTCHA‑solver farms
- LLM‑assisted text reasoning

## 🐙 Edge‑Only Verification Flow

1. Risk engine decides whether to challenge.
2. Server issues a challenge with a nonce bound to session + IP bucket.
3. Client completes the challenge and submits response.
4. Server verifies answer, expiry, and nonce integrity.
5. Server issues a short‑lived signed token (human‑verified).
6. Token gates future requests until expiry.

## 🐙 Reference Design (Option 2 + Option 3)

This is the recommended first implementation:
1. Primary challenge: micro‑interaction (SVG) for visual users.
2. Accessible equivalent: text‑only logic prompt with equal strength.
3. Both share a single seed format, expiry rules, and token issuance.

## 🐙 Implementation Phases (Agreed Order)

Phase 1: Option 1 (PoW only)
- Add a small, risk‑gated PoW step with short TTL.
- No human‑verified token yet.

Phase 2: Option 2 + Option 3 (Micro‑interaction + accessible text)
- Implement the shared seed format and verification.
- Keep PoW as the first step on medium/high risk.
- Still no human‑verified token.

Phase 3: Option 6 (Human‑verified token)
- Add short‑lived signed token issuance on success.
- Gate protected paths on the token.

### 🐙 Seed Format (Deterministic)

The server generates a short seed and signs it:
1. `seed_id` (random)
2. `issued_at` (epoch seconds)
3. `expires_at` (issued_at + TTL)
4. `ip_bucket` (derived from request IP bucket)
5. `risk_level` (low, medium, high)
6. `puzzle_kind` (micro, text)
7. `payload` (puzzle parameters)
8. `sig` (HMAC of all fields)

The client never receives the signing key, only the signed seed.

### 🐙 Micro‑Interaction (SVG)

Example family: “Align the shape to the outline” or “Select the two identical glyphs.”
The puzzle parameters live entirely in `payload` so verification is deterministic.

Verification rules:
1. Reject expired seeds or invalid signature.
2. Ensure `ip_bucket` matches current request.
3. Ensure the answer is within tolerance for the seed.
4. Enforce single‑use by storing `seed_id` in a short‑lived KV set.

### 🐙 Text‑Only Equivalent (Accessible)

Uses the same seed, but renders a text prompt derived from `payload`.
Example family: “From this list, choose the two items that satisfy rule X.”

Verification rules are identical to the SVG puzzle:
1. Same expiry, signature, and IP bucket checks.
2. Same single‑use enforcement.
3. Same risk‑level gating.

### 🐙 Proof‑of‑Work (Option 1)

Phase 1 introduces PoW as the first step on medium/high risk.
In Phase 2, PoW remains before the puzzle.
The PoW token is short‑lived and bound to the same seed to prevent reuse.

### 🐙 Token Issuance (Option 6)

Phase 3 introduces token issuance:
1. Issue `human_verified` token with short TTL.
2. Bind token to `ip_bucket` and session cookie.
3. Require token on protected paths.

### 🐙 Accessibility Parity (Non‑Bypass)

The accessible path must not be a weaker bypass:
1. Same TTL and expiry rules.
2. Same PoW requirement.
3. Same attempt limits.
4. Same token issuance and replay protection.

### 🐙 Suggested Endpoints

1. `GET /challenge` returns a signed seed and puzzle metadata.
2. `POST /challenge/verify` validates the answer and issues token.
3. Optional `GET /challenge/a11y` returns the text‑only view of the same seed.

## 🐙 Challenge Families to Evaluate

- Perception + transformation tasks that require human pattern recognition
- Interactive micro‑tasks that remain keyboard‑navigable
- Context‑bound tasks derived from session‑specific content
- Multi‑modal pairing where visual and non‑visual versions are equivalent

Avoid relying solely on static text puzzles (e.g., letter‑counting) because they are increasingly solvable by automation.
If used, they should only be part of a layered risk score, not the primary gate.

## 🐙 Tokenization & Replay Protection

Issue a short‑lived, single‑use verification token after a successful challenge.
Bind it to the session and include a signed timestamp to enforce expiry.
Use server‑side verification for every protected action.

## 🐙 Accessibility Path (Same Strength)

- Provide an alternate modality that is not easier to automate than the primary one
- Use identical server‑side checks, expiry, and rate limits for all modalities
- Avoid requiring speed or fine motor precision
- Provide clear instructions and text alternatives for assistive tech

## 🐙 Metrics & Rollout

- Pass rate, failure rate, and abandonment rate by challenge type
- Median solve time and tail latency
- False‑positive rate (humans challenged repeatedly)
- Bot bypass rate and solver‑farm signals

Start with a small percentage of traffic and expand only when metrics are stable.

## 🐙 Research Backlog

- Track ARC‑AGI‑2 benchmark developments as inspiration (not as production puzzles)
- Identify puzzle families that are robust against LLM‑assisted answers
- Explore privacy‑preserving verification tokens for lower repeat friction
- Design an accessibility modality with equal strength
