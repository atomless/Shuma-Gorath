# LLM-Driven Adversarial Testing Research Synthesis

Date: 2026-02-25  
Status: Completed

## Scope

This synthesis answers one practical question for Shuma's simulation roadmap:

1. How can LLMs be used to run realistic adversarial tests against bot defenses?
2. Which techniques should be promoted into canonical regression scenarios (`SIM-T0`..`SIM-T4`)?

## Primary Source Findings

### 1. LLMs are effective red-team generators, not just static prompt corpora

Research on model-vs-model red teaming demonstrates that LLMs can automatically generate diverse adversarial inputs and discover policy gaps at scale. The same pattern directly maps to bot-defense simulation: use LLMs as attacker policy engines that synthesize behavior variants, then replay only validated deterministic cases in CI.

- `Red Teaming Language Models with Language Models` (Perez et al.)
- `Pair: Prompt Automatic Iterative Refinement` (Chao et al.)

### 2. Iterative optimization and transfer attacks are material risks

Optimization-driven jailbreak methods (for example, gradient-guided or suffix-optimization families) show that adaptive attack generation can outperform static attack sets and transfer across targets. For Shuma, this implies scenario matrices must include adaptive evasion families (replay/stale/order/cadence abuse), not only fixed signature checks.

- `Universal and Transferable Adversarial Attacks on Aligned Language Models` (Zou et al.)
- `Gradient-Based Language Model Red Teaming` (Geisler et al.)

### 3. Benchmarking quality depends on determinism and explicit metrics

`JailbreakBench` emphasizes that evaluation disagreement and non-standard setups produce misleading security conclusions. The operational implication is to keep one versioned manifest contract, deterministic seeds, and explicit gates beyond route pass/fail (latency/cost/amplification), so release-to-release comparisons remain stable.

- `JailbreakBench: An Open Robustness Benchmark for Jailbreaking Large Language Models` (Chao et al.)

### 4. Agentic workflows can execute multi-step exploit paths

Recent work on teams of LLM agents shows practical exploit-chain capability growth. For Shuma, this supports modeling ordered multi-step abuse (for example: seed harvest -> replay -> cadence adaptation), not only single-request probes.

- `Teams of LLM Agents can Exploit Zero-Day Vulnerabilities` (Fang et al.)

### 5. CAPTCHA/challenge-only controls are increasingly bypassable

Recent CAPTCHA-oriented datasets and benchmarks indicate continuing advances in multimodal solver capability. This reinforces Shuma's layered posture: challenge outcomes should be one signal in a broader escalation ladder, not the sole decision primitive.

- `CAPTURE: Benchmarking LLMs Against Diverse Real-World Captchas` (Agarwal et al.)
- `CAPTCHA Attack Dataset: Advancing AI's Understanding and Solving of CAPTCHA Challenges` (Alqahtani)

### 6. Defender vendor guidance remains laddered (monitor/challenge/deny) and signal-driven

Operational bot-defense docs from major edge providers continue to recommend score-driven staged responses and adversarial-bot playbooks rather than single hard blocks. This aligns with Shuma's desired outcome vocabulary (`allow`, `monitor`, `not-a-bot`, `challenge`, `maze`, `deny_temp`).

- Akamai: adversarial-bot handling guidance
- Cloudflare: bot score + AI Labyrinth deception approach

## Shuma-Specific Implications

The following are inference-based conclusions from the sources above:

1. The canonical simulation matrix should treat LLM-generated attacks as upstream scenario-discovery inputs, but CI should execute only curated deterministic scenarios.
2. `SIM-3` abuse cases should be first-class and versioned (replay, stale token, ordering/cadence), because adaptive optimization literature shows static suites drift quickly.
3. `SIM-4` must include quantitative cost assertions (latency bands and telemetry amplification), because benchmark literature shows binary pass/fail hides regressions.
4. Akamai fixture-driven payload suites should be maintained as stable corpus files so external-signal behavior is reproducible without live edge dependencies.

## Source Links

- Perez et al., *Red Teaming Language Models with Language Models* (2022): https://arxiv.org/abs/2202.03286
- Chao et al., *PAIR: Prompt Automatic Iterative Refinement* (2023): https://arxiv.org/abs/2310.08419
- Zou et al., *Universal and Transferable Adversarial Attacks on Aligned Language Models* (2023): https://arxiv.org/abs/2307.15043
- Geisler et al., *Gradient-Based Language Model Red Teaming* (2024): https://arxiv.org/abs/2401.16656
- Chao et al., *JailbreakBench* (2024): https://arxiv.org/abs/2404.01318
- Fang et al., *Teams of LLM Agents can Exploit Zero-Day Vulnerabilities* (2024): https://arxiv.org/abs/2406.01637
- Calzavara et al., *Latent Jailbreak* (2026): https://arxiv.org/abs/2601.04603
- Agarwal et al., *CAPTURE* (2025): https://arxiv.org/abs/2512.02318
- Alqahtani, *CAPTCHA Attack Dataset* (2025): https://arxiv.org/abs/2512.11323
- Akamai TechDocs, *Handle adversarial bots*: https://techdocs.akamai.com/cloud-security/docs/handle-adversarial-bots
- Cloudflare Docs, *Bot score*: https://developers.cloudflare.com/bots/concepts/bot-score/
- Cloudflare Blog, *AI Labyrinth*: https://blog.cloudflare.com/ai-labyrinth/
