# Tarpit Docs Re-review Addendum (2026-02-23)

## Scope

This addendum captures a focused re-review of:

- ASRG Deadlocked materials,
- Deadlocked-linked repositories (`Babble`, `fakejpeg`),
- active tarpit/deception implementations (`http-tarpit`, `caddy-defender`, `ai-troller`, `Sarracenia`, `Finch`),
- and existing Shuma tarpit research syntheses.

Goal: extract implementation lessons that improve attacker-cost asymmetry while keeping Shuma host cost bounded.

## Summary of Findings

### 1) Deadlocked is useful conceptually, but not yet a production reference

- Deadlocked remains explicitly work-in-progress.
- Its strongest transferable idea is the “infinite believable junk graph” pattern for crawler sink behavior.
- It should be treated as a concept source, not a hardened runtime blueprint.

Source: <https://algorithmic-sabotage.gitlab.io/asrg/deadlocked/>

### 2) Most useful implementation lessons from Deadlocked’s direct dependencies

- `Babble`: deterministic nonsense generation, simple operator telemetry (`stats.txt`, offender visibility), and robots-deny default posture.
- `fakejpeg`: template-driven fake media generation can produce structurally valid-looking assets with low CPU cost.

Implication for Shuma:

- prefer cached/pre-generated content shards for tarpit output over expensive per-request synthesis work.
- this directly supports the `TAH-17` backlog direction.

Sources:

- <https://git.jsbarretto.com/zesterer/babble/raw/branch/main/README.md>
- <https://github.com/gw1urf/fakejpeg>

### 3) Cost-asymmetry mechanics converging across active projects

- `http-tarpit`: explicit connection/worker/timing envelope and published resource profile.
- `caddy-defender`: clear operator controls (`timeout`, `bytes_per_second`, configurable content source).
- `ai-troller`: simplified packaging of the same core controls.
- `Sarracenia`: staged threat model, optional drip-feed variability, persistent corpus strategy.
- `Finch`: high-confidence fingerprint gating before tarpit action.

Implication for Shuma:

- your selected next directions (`work-gated progression`, `token-chain continuity`, `explicit egress budgets`) match the strongest operational patterns in current implementations.

Sources:

- <https://github.com/die-net/http-tarpit>
- <https://github.com/JasonLovesDoggo/caddy-defender/blob/main/docs/examples.md>
- <https://github.com/circa10a/ai-troller>
- <https://github.com/amenyxia/Sarracenia>
- <https://github.com/0x4D31/finch>
- <https://github.com/0x4D31/finch/blob/main/docs/rule-schema.md>

## Cross-check Against Existing Shuma Synthesis

This re-review reinforces existing Shuma research conclusions:

- bounded admission/budget controls are mandatory,
- strong gating should precede expensive tarpit tiers,
- static signatures are fingerprintable over time,
- distributed counter semantics matter for multi-instance correctness.

See:

- [`2026-02-22-http-tarpit-cost-shift-research-synthesis.md`](2026-02-22-http-tarpit-cost-shift-research-synthesis.md)
- [`2026-02-14-maze-tarpit-research-synthesis.md`](2026-02-14-maze-tarpit-research-synthesis.md)
- [`tarpit-research-2026-02-11.md`](tarpit-research-2026-02-11.md)

## Implementation Direction Mapping (Current Backlog)

The re-review maps directly to active `TAH-*` work in [`../../todos/todo.md`](../../todos/todo.md):

- `TAH-1`..`TAH-7`: work-gated progression + token-chain continuity,
- `TAH-8`..`TAH-10`: explicit egress budgets and deterministic enforcement,
- `TAH-11`..`TAH-15`: observability, operator controls, tests, and docs,
- `TAH-16`..`TAH-18`: bounded variability, cached shard strategy, and crawler-safety policy path.
