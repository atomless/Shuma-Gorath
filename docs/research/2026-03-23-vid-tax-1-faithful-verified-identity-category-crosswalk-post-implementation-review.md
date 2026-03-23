# VID-TAX-1 Faithful Verified-Identity Category Crosswalk Post-Implementation Review

Date: 2026-03-23
Status: Completed

Related plan:

- [`../plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md`](../plans/2026-03-23-verified-identity-taxonomy-calibration-and-guardrails-implementation-plan.md)

# Delivered

`VID-TAX-1` now replaces the old verified-identity collapse with a faithful runtime crosswalk into the canonical non-human taxonomy:

1. `search` now projects to `indexing_bot`,
2. `training` now projects to `ai_scraper_bot`,
3. `user_triggered_agent` now projects to `agent_on_behalf_of_human`,
4. `preview` and `service_agent` now project to `http_agent`,
5. `other` remains explicit fallback to `verified_beneficial_bot` instead of pretending to be a more specific taxonomy class.

The implementation also carries that crosswalk through the machine-first path instead of leaving it as an isolated helper:

1. request outcomes now materialize non-human category rows directly,
2. monitoring summarizes those rows as bounded `by_non_human_category` data,
3. non-human classification now prefers those exact live category rows over the older lane-only fallback,
4. verified-identity summary rows now preserve category, provenance, and `end_user_controlled`,
5. and the operator snapshot verified-identity section now exposes top verified categories.

# Verification

- `make test-verified-identity-calibration-readiness`
- `make test-verified-identity-taxonomy-crosswalk`
- `git diff --check`

Focused proof now covers:

1. the runtime crosswalk helper,
2. request-flow request-outcome materialization for verified traffic,
3. monitoring category counters,
4. non-human classification receipt projection from live verified category rows,
5. non-human operator snapshot projection,
6. and verified-identity snapshot category summaries.

# Review Result

No tranche-local shortfall remains open.

One residual warning remains in the focused proof output:

1. `src/config/runtime_env.rs::spin_variable_name` is still a pre-existing dead-code warning unrelated to `VID-TAX-1`.

This tranche should not reopen that unrelated build-hygiene item; it should stay queued under the existing warning-cleanup follow-up rather than blocking verified-identity crosswalk delivery.
