# Recursive Self-Improvement Audit Trail And GitHub Provenance Review

Date: 2026-03-24
Status: Proposed planning driver

Related context:

- [`2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md`](2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md)
- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`2026-03-24-llm-player-role-decomposition-review.md`](2026-03-24-llm-player-role-decomposition-review.md)
- [`2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md`](2026-03-24-scorecard-protocol-and-held-out-eval-separation-review.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`../../src/observability/operator_snapshot_recent_changes.rs`](../../src/observability/operator_snapshot_recent_changes.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../docs/api.md`](../../docs/api.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

Primary external sources inspected:

- [karpathy/autoresearch](https://github.com/karpathy/autoresearch)
- [GitHub Docs: Comparing commits](https://docs.github.com/en/pull-requests/committing-changes-to-your-project/viewing-and-comparing-commits/comparing-commits)
- [GitHub Docs: About pull request reviews](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/about-pull-request-reviews)
- [GitHub Docs: About status checks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/about-status-checks)
- [GitHub Docs: About protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [GitHub Docs: Using artifact attestations to establish provenance for builds](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)

# Purpose

Determine whether Shuma's later recursive-improvement path already has an adequate audit trail, and define how far it should lean on GitHub for provenance rather than inventing a parallel internal code-review and merge-history system.

The central question is not only "can we remember what changed?" but:

1. can we prove what the defender proposed,
2. what actually changed,
3. who or what approved it,
4. which evidence and scorecard revision judged it,
5. and, later for code evolution, which GitHub branch, pull request, checks, merge, and revert lineage enacted it.

# Executive Summary

Shuma already has a meaningful internal audit foundation for config-loop changes.

Today the repo already records:

1. bounded recent config changes in `operator_snapshot_v1.recent_changes`,
2. full decision payloads and persisted history through `/admin/oversight/history`,
3. and the beginnings of a later episode archive in `RSI-GAME-1C`.

That means the config side of the later game is not starting from zero.

The real remaining gap is **canonical provenance across later recursive-improvement episodes, especially once code changes enter the loop**.

The right answer is:

1. keep the machine-first judge as the authority for outcome truth,
2. keep config-episode lineage primarily inside Shuma,
3. and lean on GitHub as the canonical provenance spine for code evolution wherever possible.

More concretely:

1. GitHub should be the authoritative store for code diff lineage, pull-request discussion, required reviews, status checks, merge commits, revert commits, and later build attestations.
2. Shuma should store stable GitHub references and normalized receipts, not duplicate full PR review text or CI logs.
3. Shuma should remain authoritative for:
   1. episode ids,
   2. rule and scorecard revisions,
   3. evidence refs,
   4. benchmark deltas,
   5. retain or rollback outcomes,
   6. and held-out or protected-evidence judge verdicts.

So the design principle should be:

1. **GitHub is the code-lineage ledger.**
2. **Shuma is the judge and episode-outcome ledger.**
3. **A shared provenance contract links them.**

# Findings

## 1. Shuma already has a real config-change audit trail

This part is stronger than it may look at first glance.

The repo already records bounded, typed config-change summaries in [`../../src/observability/operator_snapshot_recent_changes.rs`](../../src/observability/operator_snapshot_recent_changes.rs), including:

1. `decision_id`,
2. `decision_kind`,
3. `decision_status`,
4. `objective_revision`,
5. `expected_impact_summary`,
6. evidence references,
7. and watch-window result state.

The underlying persisted oversight path in [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs) is also already more detailed than the dashboard projection:

1. decision payload,
2. reconcile output,
3. validation result,
4. apply result,
5. and persisted decision history.

The public contract in [`../../docs/api.md`](../../docs/api.md) already exposes both:

1. a bounded snapshot ledger via `/admin/operator-snapshot`,
2. and fuller persisted history via `/admin/oversight/history`.

Conclusion:

1. config provenance already exists,
2. but it still needs to be generalized into the later recursive-improvement episode vocabulary rather than remain an oversight-local concept.

## 2. `RSI-GAME-1C` gives Shuma episode memory, but not yet full provenance

The current later plan already says the episode archive should remember:

1. target stance,
2. baseline scorecard,
3. proposed move,
4. accepted or refused status,
5. watch-window result,
6. rollback or retain state,
7. benchmark deltas,
8. and guardrail triggers.

That is the right substrate for stepping-stone memory.

But it does not yet freeze the broader provenance vocabulary needed once code changes exist.

Still missing are explicit fields for:

1. proposal kind (`config`, later `code`),
2. originating role (`defender_agent`, `human_operator`, later `code_loop`),
3. scorecard and evaluation revision identifiers,
4. diff or patch digest,
5. and, for code changes, canonical GitHub refs.

Conclusion:

1. an episode archive is necessary,
2. but by itself it is not a full audit contract.

## 3. `autoresearch` is useful here mainly because it treats git history as memory

The most relevant lesson from [`karpathy/autoresearch`](https://github.com/karpathy/autoresearch) is not just the fixed evaluator.
It is also that the loop stays legible by keeping mutation small and leaving an ordinary git trail.

The repo README explicitly frames the loop as:

1. the agent modifies code,
2. runs a bounded experiment,
3. keeps or discards the result,
4. and leaves a log of experiments plus a better resulting state.

That works because the mutation surface is narrow and diffable.

For Shuma, the analog is:

1. bounded config patches now,
2. later bounded code proposals,
3. compareable scorecards and watch-window outcomes,
4. and ordinary repository-level lineage rather than a bespoke hidden mutation store.

Conclusion:

1. Shuma should copy the discipline of using git or GitHub as externalized memory for code evolution,
2. but keep its judge and no-harm semantics stricter than `autoresearch`.

## 4. GitHub already provides most of the code-side provenance Shuma will need

GitHub's native model already covers the main code-lineage questions Shuma would otherwise have to reinvent.

The official docs show:

1. compare views are first-class for branch and commit diffs ([Comparing commits](https://docs.github.com/en/pull-requests/committing-changes-to-your-project/viewing-and-comparing-commits/comparing-commits)),
2. pull-request reviews are the primary collaboration and approval surface, with `Comment`, `Approve`, and `Request changes` states ([About pull request reviews](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/about-pull-request-reviews)),
3. status checks expose commit-level CI outcomes and are already tied to pull requests and protected branches ([About status checks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/about-status-checks)),
4. protected branches can require pull-request reviews, status checks, conversation resolution, deployments, and linear history before merge ([About protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)),
5. and GitHub artifact attestations can later provide build provenance over binaries or images ([Using artifact attestations to establish provenance for builds](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)).

That means the later code loop does not need Shuma to reinvent:

1. diff visualization,
2. approval workflow,
3. merge gating,
4. merge or revert lineage,
5. or build provenance.

Conclusion:

1. Shuma should lean on GitHub heavily for code-side provenance,
2. and only mirror stable identifiers and summaries inside its own episode archive.

## 5. GitHub lineage must not replace the independent judge

This is the most important boundary.

GitHub can tell Shuma:

1. what code changed,
2. which branch or pull request carried it,
3. who approved it,
4. which checks passed,
5. and whether it merged or was reverted.

GitHub cannot tell Shuma:

1. whether human friction rose too far,
2. whether tolerated traffic was harmed,
3. whether adversary effectiveness improved,
4. whether held-out evidence shows overfitting,
5. or whether the strict reference stance regressed.

Those remain judge questions.

So GitHub should be treated as:

1. authoritative for **code-lineage provenance**,
2. but never authoritative for **game outcomes**.

Conclusion:

1. later recursive-improvement phases should link GitHub lineage to judge outcomes,
2. not substitute GitHub process success for benchmark success.

## 6. Shuma needs one shared provenance vocabulary across config and code episodes

The missing contract is one common lineage vocabulary that spans:

1. current config recommend/apply/watch episodes,
2. later defender recommend-only runs,
3. later autonomous run-to-homeostasis episodes,
4. and later code-evolution proposals.

At minimum, that vocabulary should include:

1. `episode_id`,
2. `proposal_id`,
3. `proposal_kind`,
4. `origin_role`,
5. `game_contract_revision`,
6. `scorecard_revision`,
7. `protocol_revision`,
8. `evaluation_revision`,
9. evidence refs,
10. baseline and result score refs,
11. enactment status,
12. rollback or revert status.

Then each proposal kind can extend it:

1. config proposal:
   1. config patch digest,
   2. config paths touched,
   3. canary or rollback refs.
2. code proposal:
   1. repository,
   2. branch,
   3. compare URL,
   4. pull request number or URL,
   5. review decision summary,
   6. head and merge commit SHAs,
   7. check-suite refs,
   8. revert refs,
   9. optional artifact-attestation refs.

Conclusion:

1. without this shared vocabulary, the later game will have memory,
2. but not clean auditability.

## 7. "Lean on GitHub where we can" should be a hard design rule, not a vague preference

This should become an explicit repo principle for later code evolution:

1. if provenance is already natively represented by GitHub, Shuma should store the GitHub ref rather than duplicate the full artifact,
2. if the lineage is internal to the running control plane and not naturally a GitHub artifact, Shuma should keep it internal,
3. and if a later phase produces deployable build artifacts, GitHub attestations should be preferred over bespoke provenance files where feasible.

That means:

1. store PR numbers, SHAs, compare URLs, review status, check refs, and attestation refs,
2. do not copy entire review threads or CI logs into Shuma,
3. but do copy the judge result and benchmark deltas because GitHub does not know them.

Conclusion:

1. this should be codified as a separate audit/provenance contract before `OVR-CODE-1` is reopened.

# Implications For Planning

The later recursive-improvement chain now needs one more explicit blocked contract:

1. `RSI-AUDIT-1`: canonical audit and provenance contract for recursive-improvement episodes.

That contract should split into at least:

1. `RSI-AUDIT-1A`: shared episode and proposal lineage schema across config and code moves,
2. `RSI-AUDIT-1B`: GitHub-backed code-evolution provenance contract,
3. `RSI-AUDIT-1C`: machine-first audit retrieval and operator projection surfaces.

This new contract should be required before:

1. later recommendation-only defender runtime is treated as execution-ready,
2. later autonomous defender episodes are treated as operationally accountable,
3. and especially before `OVR-CODE-1` is reopened.

# Recommended Principle

The repo should make this explicit:

1. **Shuma keeps the judge truth.**
2. **GitHub keeps the code-lineage truth.**
3. **Recursive-improvement episodes must link the two through stable provenance ids and refs.**
