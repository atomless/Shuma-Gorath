# Scrapling 0.4.3 Upgrade And Realism Impact Review

Date: 2026-03-30

## Question

Should Shuma upgrade its repo-owned Scrapling worker runtime from `0.4.2` to the latest upstream release before starting `SIM-REALISM-1A`, and do any upstream changes warrant immediate adoption ahead of the realism chain?

## Current repo state

The repo-owned Scrapling worker runtime is provisioned outside a Python lockfile. The canonical pin lives in:

- [`scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)

At the start of this review, that helper pinned:

- `SCRAPLING_RUNTIME_PACKAGE_VERSION=0.4.2`
- `SCRAPLING_RUNTIME_PACKAGE_SPEC=scrapling[fetchers]==0.4.2`

The same helper also fail-closed the readiness check if the installed runtime was not exactly `0.4.2`.

## Upstream release truth

Primary upstream sources show that Scrapling `0.4.3` is now the latest release:

- PyPI release history for `scrapling` lists `0.4.3` as released on 2026-03-30: [PyPI](https://pypi.org/project/scrapling/)
- GitHub tags for `D4Vinci/Scrapling` include `v0.4.3` at `e173f813da2a7c2d6984042cd5cfbe6d5ec90dce`: [GitHub tags](https://github.com/D4Vinci/Scrapling)

There is no formal upstream `CHANGELOG.md` in the tagged tree, so the authoritative change record is the `v0.4.2..v0.4.3` tag diff plus the tagged commit history.

## Upstream changes that matter to Shuma

The upstream diff is broad overall, but the realism-relevant and worker-relevant changes are concentrated in five items:

1. `feat(browser sessions): Collect XHR requests done while loading the page`
   - commit `68f7c5c`
   - This is directly relevant to browser-persona realism because real browser automation often triggers background `fetch` / XHR activity during page load.
2. `feat(browsers): Add a new option to set browser path`
   - commit `8a4c5ff`
   - This is operationally useful when a specific browser binary is needed for realism or local-host compatibility.
3. `fix: preserve HTTP method across retries in spider session`
   - commit `5bf921b`
   - This is a correctness fix for non-GET spider retries.
4. `fix: add max retry limit to _get_page_content to prevent infinite loop`
   - commit `d3c251c`
   - This reduces the chance that browser fetchers hang indefinitely in broken-page states.
5. `fix(fetchers/content): increase the default max number of retries and raise error on max retries`
   - commit `1dc0b7a`
   - This is a general fetcher robustness improvement.

## Assessment against Shuma's current worker

Shuma currently uses:

- `FetcherSession` for crawler/request-native spider work,
- `DynamicSession` for browser automation,
- `StealthySession` for stealth browser automation,

through [`scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

That means the upgrade is worth taking now, because:

1. the browser-session retry and hang fixes reduce runtime risk for the exact classes Shuma already uses,
2. the upstream background-request capture feature is directly relevant to later realism receipt truth for browser personas,
3. the browser-path option gives useful deployment flexibility without changing Shuma's trust boundary,
4. and the repo already pins exact Scrapling runtime versions, so staying one release behind adds churn with no benefit.

## What should land now vs later

### Land now

- Upgrade the repo-owned Scrapling runtime pin from `0.4.2` to `0.4.3`.
- Keep the readiness check exact and synchronized to the new pin.
- Add a focused proof that the pin and the readiness check cannot silently drift apart.

### Do not land yet

- Do not wire the new upstream browser-path option into Shuma before `SIM-REALISM-1A`.
- Do not start projecting background XHR activity into Shuma's worker receipts in this dependency tranche.
- Do not change Scrapling pacing, burst, session, retry, or persona behavior here beyond whatever upstream `0.4.3` already changes under the same public API.

## Why those later items should wait

The two tempting follow-ons are both realism-adjacent, but they belong inside the realism chain rather than before it:

1. Background XHR capture should be evaluated as part of `SIM-REALISM-1B` receipt truth, because it changes what Shuma claims the browser personas actually emitted.
2. Browser-path support should be evaluated as a bounded operational override during browser-persona realism work, not as an unplanned new control surface in an otherwise simple dependency bump.

## Decision

Upgrade to Scrapling `0.4.3` now.

Treat the upstream browser background-request capture and browser-path support as realism inputs for the next active chain, but not as new blockers ahead of `SIM-REALISM-1A`.
