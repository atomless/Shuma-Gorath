# Completed TODO History

Moved from active TODO files on 2026-02-14.

## Additional completions (2026-03-17)

### Stage 0 Roadmap: Complete Operator-Surface Truth Prerequisites

- [x] Complete Stage 0 by finishing the heartbeat-owned dashboard connection-state hardening tranche, surfacing operator-facing connection diagnostics in `Status`, and splitting `GET /admin/config` into a truthful `{ config, runtime }` envelope that no longer presents read-only operational overlays as writable admin config.
- [x] Why:
  - Stage 0 existed to make the operator surfaces truthful before Shuma builds a fuller monitoring control plane or hands more tuning authority to humans and future scheduled agents.
  - connection-state ownership had to be explicit and test-proven so tab-local request failures, cancelled requests, and polling churn could not masquerade as backend disconnects.
  - the config contract had to stop muddying the adversary-sim and runtime posture story by mixing writable KV settings with read-only operational facts in one flat payload.
- [x] Evidence:
  - `dashboard/src/lib/state/dashboard-store.js`
  - `dashboard/src/lib/runtime/dashboard-native-runtime.js`
  - `dashboard/src/lib/components/dashboard/StatusTab.svelte`
  - `dashboard/src/lib/domain/config-runtime.js`
  - `dashboard/src/lib/domain/dashboard-state.js`
  - `dashboard/src/lib/domain/api-client.js`
  - `dashboard/src/routes/+page.svelte`
  - `src/admin/api.rs`
  - `e2e/dashboard.modules.unit.test.js`
  - `e2e/dashboard.smoke.spec.js`
  - `scripts/tests/integration.sh`
  - `scripts/tests/edge_signal_smoke_common.py`
  - `docs/configuration.md`
  - `docs/api.md`
  - `docs/dashboard-tabs/status.md`
  - `docs/testing.md`
  - `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
  - `make test-dashboard-unit`
  - `make test-dashboard-e2e PLAYWRIGHT_ARGS='--grep "status tab resolves fail mode without requiring monitoring bootstrap|tab-local monitoring failures do not flip the global dashboard connection state"'`
  - `make test-unit`
  - `git diff --check`

## Additional completions (2026-03-16)

### Telemetry Cleanup: Restore Architectural Clarity, Keep Monitoring External-Only, And Revalidate The Full Suite

- [x] Complete the external-only monitoring cleanup by removing operator-originated activity from Monitoring surfaces, keeping active ban state separate, tightening the adversarial browser harness to satisfy the real not-a-bot contract, restoring the shared half-doughnut style contract, and aligning stale dashboard smoke expectations to the intentionally updated operator UI.
- [x] Why:
  - the architectural intent of the cleanup is that Monitoring represents observed external traffic only, while operator actions, current active ban state, and low-level diagnostics each stay in their own clearer surfaces.
  - the first verification pass proved some lower layers but missed a full rendered dashboard path, so this tranche explicitly corrected that methodology and carried the work through the complete repo test suite.
  - keeping telemetry excellence here meant fixing harnesses and tests to match truthful current contracts rather than weakening runtime behavior or reintroducing muddied monitoring semantics.
- [x] Evidence:
  - `src/admin/api.rs`
  - `scripts/tests/adversarial_browser_driver.mjs`
  - `dashboard/style.css`
  - `e2e/dashboard.smoke.spec.js`
  - `Makefile`
  - `.spin/last-full-test-pass.json`
  - `make test-monitoring-telemetry-contract`
  - `make test-adversarial-smoke`
  - `make test-dashboard-unit`
  - `make test`
  - `git diff --check`

### Monitoring Telemetry: Exclude Operator-Originated Events From Monitoring Surfaces While Preserving Active Ban State

- [x] Remove admin/dashboard/config/manual-ban activity from Monitoring-facing telemetry so Monitoring, monitoring deltas/stream, range event reads, and the raw telemetry feed all reflect external traffic only, while `IP Bans` continues to expose the real active ban state for the site.
- [x] Why:
  - mixing operator actions into Monitoring made the dashboard harder to interpret and risked misleading both humans and future tuning agents about what was actually happening on the public traffic surface.
  - manual ban/unban actions are operational interventions, not observed incoming traffic, so they belong in active ban state and future audit surfaces rather than in Monitoring charts, recent-event tables, or raw monitoring feeds.
  - the delta/stream fix had to preserve the bounded-read intent of the monitoring cursor contract, so the implementation now skips operator-originated rows without regressing into a full-window page scan.
- [x] Evidence:
  - `src/admin/api.rs`
  - `dashboard/src/lib/components/dashboard/monitoring/RecentEventsTable.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring/RawTelemetryFeed.svelte`
  - `docs/dashboard-tabs/monitoring.md`
  - `docs/dashboard-tabs/ip-bans.md`
  - `Makefile`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-monitoring-telemetry-contract`
  - `git diff --check`

### Dashboard Charts: Disable First-Paint Growth Animation Via Shared Zero-Duration Chart Runtime Defaults

- [x] Apply a shared Chart.js runtime default of `duration: 0` so dashboard charts render instantly instead of animating awkwardly from the top-left on first paint, and remove the local one-off animation override from the IP Bans doughnut path.
- [x] Why:
  - the current first-paint chart growth animation is visually poor and inconsistent with the rest of the dashboard's controlled design language.
  - this behavior belonged in the shared chart runtime contract, not as scattered per-chart overrides.
  - removing the IP Bans-only override restores the reuse-first rule and keeps future chart behavior aligned through one canonical path.
- [x] Evidence:
  - `dashboard/src/lib/domain/services/chart-runtime-adapter.js`
  - `dashboard/src/lib/components/dashboard/IpBansTab.svelte`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`
  - `git diff --check`

### Monitoring Diagnostics: Move Freshness, Transport, Overflow, And Raw Feed Into One Collapsed Bottom Section

- [x] Remove the contributor-style freshness and transport strips from the top of `Monitoring` and `IP Bans`, and consolidate those low-level diagnostics with the raw telemetry feed into one collapsed `Telemetry Diagnostics` section at the bottom of `Monitoring`, immediately before the Prometheus helper.
- [x] Why:
  - the top-of-tab freshness and transport strips were crowding the operator surfaces with low-level read-path details that are useful for contributors and diagnostics, but not the right default reading experience for operators or future controller-facing monitoring.
  - the IP-ban partial-view warning is only meaningful as bounded recent-feed diagnostics, not as a headline state for the operational ban-management surface.
  - consolidating those details into one collapsed diagnostics section preserves the evidence for debugging without letting it dominate the primary monitoring and ban-management views.
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/primitives/DisclosureSection.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring/DiagnosticsSection.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring/RawTelemetryFeed.svelte`
  - `dashboard/src/lib/components/dashboard/MonitoringTab.svelte`
  - `dashboard/src/lib/components/dashboard/IpBansTab.svelte`
  - `dashboard/src/routes/+page.svelte`
  - `dashboard/style.css`
  - `docs/dashboard-tabs/monitoring.md`
  - `docs/dashboard-tabs/ip-bans.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`
  - `git diff --check`

### Status Tab: Remove Internal Transport Rows And Align Runtime Telemetry Copy With Shared Auto-Refresh Tabs

- [x] Remove the `Monitoring update path` and `IP bans update path` rows from the Status tab's `Telemetry Delivery Health` section and update the `Runtime Performance Telemetry` copy so it correctly names all shared auto-refresh tabs, including `red-team`.
- [x] Why:
  - the update-path rows exposed internal transport details with no real operator choice or action attached, so they added noise rather than useful health signal.
  - the runtime-performance guidance had drifted behind the actual dashboard refresh model and still described only `monitoring` and `ip-bans`, even though `red-team` now shares that auto-refresh path too.
  - this keeps the Status surface focused on actionable health facts while preserving consistent copy with the current shared refresh contract.
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/StatusTab.svelte`
  - `docs/dashboard-tabs/status.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`
  - `git diff --check`

### Status Tab: Hydrate IP-Ban Freshness Without Visiting The IP Bans Tab First

- [x] Extend the Status-tab refresh path so it hydrates `ipBansFreshness` from the lightweight IP-ban cursor-delta endpoint, allowing the IP-ban health rows to render immediately without requiring an operator to visit the `IP Bans` tab first.
- [x] Why:
  - the new `Telemetry Delivery Health` section in `Status` correctly consumed `ipBansFreshness`, but the shared refresh runtime only populated that snapshot when the `IP Bans` tab itself had already refreshed.
  - that made the Status surface look broken and violated the operator-health intent of the connection-state hardening tranche, because one of the core health rows was blank until another tab happened to bootstrap it.
  - the fix keeps ownership boundaries clean by reading only the lightweight freshness delta from `Status`, without mutating IP-ban cursors or pretending the `IP Bans` tab dataset has been fully loaded.
- [x] Evidence:
  - `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`
  - `git diff --check`

### Status Tab: Remove Superfluous Dashboard Connectivity "Last Change" Row

- [x] Remove the `Last Change` row from the Status tab's `Dashboard Connectivity` section and delete the now-unused connection-reason formatting helper so the section keeps only the operator-relevant heartbeat state and timing facts.
- [x] Why:
  - the row added little value once the connectivity section already showed current status, last success, last failure, and consecutive failures against threshold.
  - removing it keeps the section tighter and less interpretive while avoiding another low-signal line in the operator health surface.
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/StatusTab.svelte`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`

### Status Tab: Remove Redundant Health Wrapper Heading And Keep Both Panels On The Same Heading Hierarchy

- [x] Remove the redundant `Dashboard and Telemetry Health` wrapper heading/copy from the Status tab and promote the lower-pane subsection headings so the health pane matches the heading hierarchy and visual rhythm of the status inventory above it.
- [x] Why:
  - the wrapper heading added no real information and made the lower pane feel like a nested sub-surface instead of a peer panel.
  - the subsection titles below it were visually one level lower than the status items above, which made the pane look inconsistent even after the row-based telemetry cleanup.
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/StatusTab.svelte`
  - `docs/dashboard-tabs/status.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`

### Status Tab: Promote Connection, Telemetry Delivery, And Retention Health Into A Dedicated Operator Surface

- [x] Restructure the dashboard `Status` tab so the generic feature-status inventory no longer carries `Retention and Freshness Health`, and instead the tab presents a dedicated operator health panel for heartbeat-owned connection state, monitoring-feed freshness, IP-ban-feed freshness, retention-worker posture, and runtime performance telemetry.
- [x] Why:
  - the duplicated freshness and transport strips in Monitoring and IP Bans were diagnostic in tone, while the Status tab lacked the explicit operator health view needed by the planned connection-state hardening tranche.
  - retention and freshness were previously rendered as if they were just another config-derived status card, which made an operational read-path concern look like a static feature posture.
  - this change keeps Monitoring and IP Bans focused on operational outcomes while making Status the clearer home for “can I trust what this dashboard is telling me right now?”
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/StatusTab.svelte`
  - `dashboard/src/lib/domain/status.js`
  - `dashboard/src/lib/domain/telemetry-freshness.js`
  - `dashboard/src/lib/components/dashboard/MonitoringTab.svelte`
  - `dashboard/src/lib/components/dashboard/IpBansTab.svelte`
  - `dashboard/src/routes/+page.svelte`
  - `docs/dashboard-tabs/status.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`

### Roadmap Sequencing Note: Insert Existing Backlog Prerequisites And Release Gates Into The Master Order

- [x] Update the roadmap sequence so it explicitly includes the already-listed backlog items that materially change ordering: dashboard connection-state hardening, admin-config contract truthfulness, production adversary-sim operating-envelope work, shared-host discovery baseline, privacy/state-minimization gates, and the final pre-launch performance gate.
- [x] Why:
  - those items were already present in `todos/todo.md` and `todos/security-review.md`, but the roadmap still read as if monitoring, tuning, verified identity, mature sim, and central intelligence were the only sequence-defining stages.
  - in practice, several of those backlog items are prerequisites for a truthful control plane or a safe later stage: connection-state and config truth before operator surfaces, sim operating-envelope and shared-host discovery before mature emergent lanes, privacy/state-minimization before shared intelligence, and a final performance gate before launch.
  - the roadmap now also marks optional Akamai list mirroring and breach-to-replay work as side branches rather than accidentally implying they belong on the mainline critical path.
- [x] Evidence:
  - `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
  - `todos/completed-todo-history.md`
  - `git diff --check`
  - verification intentionally scoped as docs-only; tests not run

### Roadmap Sequencing Note: Place Edge-Instance Ban Sync Before Mature Sim And Keep It Separate From Central Intelligence

- [x] Update the roadmap and central-intelligence design docs so Shuma's planned edge-instance ban synchronization is explicitly scheduled before mature adversary-sim and clearly separated from the later centralized worst-offender or intelligence layer.
- [x] Why:
  - the repo already had a planned multi-instance enterprise ban-sync track, but the higher-level roadmap did not yet spell out where that work belongs relative to verified identity, mature sim, central intelligence, and the scheduled agent loop.
  - edge-instance ban sync is deployment-local correctness for exact active bans, not cross-site memory, so it should land before serious multi-instance adversary-sim and well before any centralized worst-offender record or CTI-style architecture.
  - making this distinction explicit reduces the risk of future work blurring local authoritative enforcement state with advisory or high-confidence shared reputation data.
- [x] Evidence:
  - `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
  - `docs/plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`
  - `todos/completed-todo-history.md`
  - `git diff --check`
  - verification intentionally scoped as docs-only; tests not run

### Roadmap Sequencing Note: Place Verified Bot Identity Before Mature Sim, Central Intelligence, And Scheduled Agentic Reconfiguration

- [x] Update the roadmap documents so the planned sequence explicitly places verified bot identity and Web Bot Auth work after monitoring and tuning foundations, but before mature adversary-sim, central intelligence, and the scheduled agent operator loop.
- [x] Why:
  - the new verified-identity research and design made it clear that Shuma should formalize authentication and local authorization before layering on realistic verified-agent adversary simulation, external intelligence, or autonomous recommendation and reconfiguration.
  - making that ordering explicit in the roadmap reduces the chance of later implementation slices blurring identity, authorization, reputation, and autonomy concerns.
  - the feature-specific implementation plan now also states its roadmap placement directly, so the sequencing is visible both in the master roadmap and in the verified-identity plan itself.
- [x] Evidence:
  - `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
  - `docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`
  - `todos/completed-todo-history.md`
  - `git diff --check`
  - verification intentionally scoped as docs-only; tests not run

### Verified Bot Identity Research And Planning: Web Bot Auth, Signed Agents, And Named Agent Policy

- [x] Gather current primary-source research and write repo-native design docs for Shuma's verified bot identity lane, including Web Bot Auth, HTTP Message Signatures, signed agents, verified bots, local authorization policy, and low-cost authenticated-agent handling.
- [x] Why:
  - the existing March 15 and March 16 agentic-era docs established that cryptographic bot identity matters, but Shuma still lacked a dedicated deep synthesis and implementation plan for how verified identity should actually work in the product.
  - this area is moving quickly, and the current ecosystem now includes active IETF work, vendor support from Cloudflare and Vercel, signed user-triggered agent traffic from OpenAI, and clearer crawler/agent splits from Anthropic and Google.
  - the resulting docs make the critical Shuma design separation explicit: identity/authentication, local authorization policy, crawler preference signaling, and central intelligence must remain distinct even though they work together in the broader bot-defence system.
- [x] Evidence:
  - `docs/research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`
  - `docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`
  - `docs/plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`
  - `docs/research/README.md`
  - `docs/index.md`
  - `git diff --check`
  - verification intentionally scoped as docs-only; tests not run

### Red Team Run History Follow-Up: Fix The Dropped Dashboard Payload Boundary And Tighten Verification Methodology

- [x] Repair the broken `Recent Red Team Runs` rendering path after the new `recent_sim_runs` hot-read field was found to be disappearing before it reached dashboard state, then strengthen repo methodology so future dashboard telemetry changes are verified through the smallest rendered proof path that actually covers the changed boundary.
- [x] Why:
  - the initial hot-read implementation was not enough on its own because the new `recent_sim_runs` field was silently dropped in the dashboard API normalization layer, so the Red Team tab rendered no rows even though backend hot-read tests were green.
  - that exposed a verification shortfall: proving backend documents and refresh-runtime behavior was insufficient without also proving the API adapter boundary and the final rendered tab DOM.
  - the correct response was not to rely on a broad dashboard suite by default, but to add focused boundary coverage and a minimal rendered browser proof targeted at the exact regression surface.
- [x] Evidence:
  - `dashboard/src/lib/domain/api-client.js`
  - `e2e/dashboard.modules.unit.test.js`
  - `e2e/dashboard.smoke.spec.js`
  - `AGENTS.md`
  - `make test-dashboard-unit`
  - `make test-dashboard-e2e PLAYWRIGHT_ARGS='--grep "red team recent runs table renders compact run-history rows from monitoring payloads"'`
  - `git diff --check`

### Red Team Run History: Stop Deriving Recent Runs From The Evictable Event Tail

- [x] Add a compact exact `recent_sim_runs` hot-read summary, thread it through Monitoring bootstrap/delta payloads, and switch the Red Team tab to consume that run-history surface instead of grouping the bounded raw-event tail.
- [x] Why:
  - the old `Recent Red Team Runs` table was not actually a run-history view. It grouped `events.recent_events`, so a noisy newer run could quickly evict an older run’s events from the bounded tail and make the older row disappear.
  - the hot-read architecture already prefers bounded materialized read models over UI-side reconstruction from raw telemetry, so the correct fix was a compact run-history document rather than inflating the recent-event cap or adding a scan-heavy endpoint.
  - keeping the run-history summary on the shared Monitoring refresh path preserves the existing operator flow while making the table truthful and resilient to event-tail churn.
- [x] Evidence:
  - `src/observability/hot_read_contract.rs`
  - `src/observability/hot_read_documents.rs`
  - `src/observability/hot_read_projection.rs`
  - `src/admin/api.rs`
  - `src/admin/mod.rs`
  - `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
  - `dashboard/src/lib/components/dashboard/monitoring-view-model.js`
  - `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
  - `docs/dashboard-tabs/red-team.md`
  - `Makefile`
  - `e2e/dashboard.modules.unit.test.js`
  - `e2e/dashboard.smoke.spec.js`
  - `make test-telemetry-hot-read-contract`
  - `make test-telemetry-hot-read-projection`
  - `make test-telemetry-hot-read-bootstrap`
  - `make test-dashboard-unit`
  - `git diff --check`

### Pre-Launch Roadmap Capture: Surface The Missing Planning Tracks And Sequence Them

- [x] Capture the major pre-launch work that Shuma still needs but had not yet been turned into a coordinated roadmap, then record deferred stub backlog entries for those tracks without pretending they are execution-ready implementation tasks.
- [x] Why:
  - several major areas were known but not yet clearly held together in one sequencing view: mature adversary-sim beyond the deterministic lane, completion of the tuning surface, operator-grade monitoring and shadow/enforced separation, distinct adversary-sim telemetry retention, central-intelligence storage architecture, and the scheduled agent analyzer/recommender/reconfigurer.
  - because the repo treats `todos/todo.md` as the execution-ready queue, the cleanest way to capture these without creating fake-ready work was to add a roadmap note plus deferred `blocked-todo` stubs that point back to the roadmap and existing plans.
  - this keeps the pre-launch pursuit of excellence explicit while reducing the risk of opportunistic slices that would conflict with the intended architecture.
- [x] Evidence:
  - `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
  - `todos/blocked-todo.md`
  - `docs/index.md`
  - `git diff --check`
  - verification intentionally scoped as docs-only; tests not run

### Agentic-Era Defence Research: Banded Ban Jitter, Local Recidive, And Central Intelligence

- [x] Gather fresh primary-source research and turn it into repo-native documentation for three coordinated features:
  - percentage-banded ban jitter,
  - repeat-offender escalation with bounded local recidive memory,
  - and optional central intelligence in the style of advisory CTI plus higher-confidence deny feeds.
- [x] Why:
  - these features should not be designed in isolation, because in Shuma they sit across different horizons of the same system: immediate request-path cost shaping, short-lived local memory, medium-horizon shared intelligence, and long-horizon oversight-controller tuning.
  - the new documentation needed to preserve the agentic-era model already established in the repo: separate automation lanes, cryptographic agent identity, low-friction beneficial-agent handling, and controller-plus-budgets rather than free-form agent autonomy.
  - the research pass re-grounded the design in current evidence from AWS, Fail2ban, CrowdSec, Spamhaus, OpenAI, Anthropic, Google, Cloudflare, the IETF, and Kubernetes so Shuma's next policy primitives stay current and defensible.
- [x] Evidence:
  - `docs/research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`
  - `docs/plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`
  - `docs/plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-implementation-plan.md`
  - `docs/index.md`
  - `docs/research/README.md`
  - `git diff --check`
  - verification intentionally scoped as docs-only; tests not run

## Additional completions (2026-03-15)

### Test Audit Cleanup: Remove Low-Value Dashboard Archaeology Checks And Reframe Telemetry Contracts

- [x] Audit the current test surface for low-value or stale checks, trim the worst dashboard source-contract archaeology assertions, and replace old-shape telemetry evidence language with present-tense contract checks.
- [x] Why:
  - the biggest concentration of low-signal tests had drifted into `e2e/dashboard.modules.unit.test.js`, where several assertions were mostly proving that old helper names, old ids, or removed chrome were absent rather than proving the current dashboard contract worked.
  - keeping the valuable positive assertions while deleting or relaxing those archaeology checks makes the test lane more useful: it now validates current shared panels, runtime responsibilities, refresh paths, and monitoring/fingerprinting/tuning surfaces without overfitting to historical implementation details.
  - the telemetry evidence helper previously reported `legacy_js_verification_rows`, which framed the check as compatibility archaeology. Renaming that signal to `js_verification_contract_violations` makes the tests assert the actual desired contract: compact JS-verification rows remain structurally correct with zero violations.
  - the audit also surfaced real missing coverage, so new TODOs were added for rendered monitoring shadow-mode truthfulness, Red Team multi-run row coverage, and continued replacement of source-string archaeology with behavior-level dashboard tests.
- [x] Evidence:
  - `e2e/dashboard.modules.unit.test.js`
  - `scripts/tests/telemetry_evidence_common.py`
  - `scripts/tests/test_telemetry_shared_host_evidence.py`
  - `scripts/tests/test_telemetry_fermyon_edge_evidence.py`
  - `todos/todo.md`
  - `make test-unit`
  - `make test-config-lifecycle`
  - `make test-telemetry-hot-read-evidence`
  - `make test-adversarial-python-unit`
  - `make test-dashboard-unit`
  - `git diff --check`

### Dashboard Monitoring Cleanup: Remove Duplicated Tarpit State Chrome

- [x] Remove the `State` and active-bucket/top-offender cards from the Monitoring tab's `Tarpit Progression` section, leaving only progression and outcome telemetry that is actionable in monitoring.
- [x] Why:
  - tarpit enabled/disabled state already belongs in `Traps` and `Status`, so repeating it in Monitoring added duplication without helping operators diagnose behavior.
  - the active-bucket card mixed ephemeral live budget state into a section otherwise dominated by cumulative outcome counters, which made the tarpit surface read as contradictory.
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/monitoring/TarpitSection.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring-view-model.js`
  - `docs/testing.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`

### Dashboard Fingerprinting Copy Cleanup: Promote The Real Panel Title

- [x] Remove the defunct `Diagnostics` heading from the fingerprinting signal panel, promote `Botness Scoring Signals` to the canonical panel title, and replace the helper copy so the panel explains that these additive signals help route bot-like traffic.
- [x] Why:
  - once the nested `Botness Scoring Signals` block became the real content of the panel, leaving `Diagnostics` as the outer title created redundant chrome and a weaker information scent than the actual subject of the panel.
  - the helper text should describe operator intent, not internal implementation language, so routing-oriented copy is clearer than generic "active definitions" wording.
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`
  - `docs/dashboard-tabs/fingerprinting.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`

### Dashboard Chart Formatting Alignment: Reuse Compact Stat Readouts In Half-Doughnut Charts

- [x] Change the shared half-doughnut readout helper so chart center values reuse the canonical compact-number formatter already used by dashboard stat cards, instead of falling back to plain full-number formatting.
- [x] Why:
  - the doughnut readout is a stat-value presentation surface, so using a different magnitude format from the surrounding cards made the UI feel inconsistent and implied a second formatting policy where there should only be one.
  - the correct fix is at the shared helper boundary, because both `Monitoring` and `IP Bans` consume the same half-doughnut readout path and should inherit the same number formatting automatically.
- [x] Evidence:
  - `dashboard/src/lib/domain/half-doughnut-chart.js`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`

### Dashboard Tuning Cleanup: Remove Redundant Botness Status Readout

- [x] Remove the read-only `Status` info block from the `Tuning` tab's `Botness Scoring` section and delete the associated unused default-threshold wiring.
- [x] Follow up by removing the read-only `Terminal Signals` info block from the same `Botness Scoring` section and delete the associated unused terminal-signal wiring.
- [x] Finish the cleanup by removing the remaining read-only `Scored Signals` inventory from `Tuning` and re-home the useful botness-scoring signal context into the canonical diagnostics surface on `Fingerprinting`.
- [x] Move the dedicated `Fingerprint Akamai edge signal (additive)` readout out of the generic fingerprinting diagnostics list and surface it in the `Akamai Bot Signal` pane where operators expect Akamai-specific contribution details.
- [x] Remove duplicated runtime counters from `Fingerprinting`, move the useful fingerprint mismatch counters into `Monitoring`'s CDP section, and rename the ban summary wording there so it reads as detection-caused bans rather than unexplained "auto-bans".
- [x] Why:
  - the `Config`, `Default Not-a-Bot`, `Default Challenge`, and `Default Maze` readouts were duplicating information already available more clearly in the dedicated tabs and config surfaces, while adding visual noise to the Tuning workflow.
  - the `Terminal Signals` inventory had the same problem: it repeated model facts that are already surfaced elsewhere, but did not add an actionable tuning control in this tab.
  - the remaining `Scored Signals` inventory in `Tuning` was similarly non-actionable, but a subset of those signals does belong in the operator mental model for fingerprinting because they are passive scoring inputs that corroborate botness.
  - re-homing the scored-signal inventory into `Fingerprinting` gives operators one diagnostics surface for score contributions, while leaving `Tuning` focused on editable thresholds and weights.
  - the dedicated Akamai additive signal is conceptually owned by the Akamai integration settings, so leaving it in the generic diagnostics list made the dashboard imply it was just another internal passive fingerprint signal.
  - the fingerprint runtime counters were live telemetry rather than configuration context, and the two headline counts were already duplicated in Monitoring, so keeping them under `Fingerprinting` blurred the line between tuning surfaces and observability surfaces.
  - removing the now-unused reactive defaults and signal-definition wiring keeps the `Tuning` component honest and avoids carrying dead local state for UI that no longer exists.
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`
  - `dashboard/src/lib/components/dashboard/MonitoringTab.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring/CdpSection.svelte`
  - `dashboard/src/routes/+page.svelte`
  - `dashboard/src/lib/components/dashboard/TuningTab.svelte`
  - `docs/dashboard-tabs/fingerprinting.md`
  - `docs/dashboard-tabs/monitoring.md`
  - `docs/dashboard-tabs/tuning.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`

### Dashboard Refresh Alignment: Put Red Team On The Shared Auto-Refresh Path

- [x] Let the `Red Team` tab participate in the same dashboard auto-refresh affordance used by `Monitoring` and `IP Bans`, and make the tab refresh hydrate the monitoring-backed run table instead of only refreshing config chrome.
- [x] Why:
  - the `Recent Red Team Runs` panel is derived from monitoring snapshots, so leaving `red-team` outside the auto-refresh tab set made the table go stale while operators were actively using the tab.
  - the grouping logic was already correct for distinct `sim_run_id` values; the real issue was that the Red Team tab was not pulling fresh monitoring snapshots, so rapid successive runs could collapse to one visible row simply because only one run survived in the stale or bounded recent-event input.
  - this completion keeps the existing grouping model intact, reuses the canonical dashboard refresh path instead of inventing a parallel poller, and updates the smoke/unit/docs contracts so the behavior is explicit.
- [x] Evidence:
  - `dashboard/src/routes/+page.svelte`
  - `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
  - `e2e/dashboard.modules.unit.test.js`
  - `e2e/dashboard.smoke.spec.js`
  - `docs/dashboard-tabs/red-team.md`
  - `docs/dashboard-tabs/monitoring.md`
  - `make test-dashboard-unit`
  - `make test-dashboard-e2e`

### Dashboard UX: Move Recent Adversary Runs From Monitoring To Red Team

- [x] Move the `Recent Adversary Runs` surface out of the `Monitoring` tab and place it at the bottom of the `Red Team` tab with copy tuned to the adversary-simulation workflow instead of generic monitoring language.
- [x] Why:
  - the panel is more actionable in `Red Team`, where operators start, stop, and reason about adversary simulation runs, than in the broader `Monitoring` surface.
  - the move keeps the existing run-row derivation and linkage to `Monitoring` and `IP Bans`, but removes one simulation-specific panel from `Monitoring` so that tab stays focused on general live defense telemetry.
  - the copy refresh clarifies that the panel is about recent adversary-simulation runs and preserves freshness-aware empty-state messaging so delayed telemetry is not mistaken for no activity.
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
  - `dashboard/src/lib/components/dashboard/MonitoringTab.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte`
  - `dashboard/src/routes/+page.svelte`
  - `docs/dashboard-tabs/red-team.md`
  - `docs/dashboard-tabs/monitoring.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`
  - `make test-dashboard-e2e`

### Agentic-Era Oversight Research, Design, and Phased Plan Capture

- [x] Capture the long-horizon research synthesis for Shuma's agentic-era operating model and write repo-native design and implementation-plan documents covering the oversight control contract, budget schema, deployment adapters, and rollout stages.
- [x] Why:
  - the previous response produced the strategy in-chat, but this repository expects durable architecture and plan context to live in docs rather than disappear into conversation history.
  - Shuma already has the core ingredients for autonomous oversight (`adversary-sim` control discipline, hot-read telemetry, config validation/write surfaces, and AI-policy outputs), so the right next step was to formalize how those pieces become a bounded oversight plane instead of leaving the idea as an informal note.
  - documenting the work now keeps future implementation aligned with current project principles: request-path logic remains deterministic and Rust-owned, autonomous change is budgeted and reversible, and scheduler choice stays adapter-level rather than architecture-defining.
- [x] Evidence:
  - `docs/research/2026-03-15-agentic-era-oversight-research-synthesis.md`
  - `docs/plans/2026-03-15-agentic-era-oversight-design.md`
  - `docs/plans/2026-03-15-agentic-era-oversight-implementation-plan.md`
  - `docs/research/README.md`
  - `docs/index.md`
  - docs-only slice: verification intentionally skipped per repo policy
  - external sources captured in the new research synthesis

## Additional completions (2026-03-14)

### Dashboard CI Repair: Give The Native Remount Soak Test Its Own Timeout Budget

- [x] Repair the remaining `main` CI failure after the Red Team notice fix by giving the deliberate native-remount soak smoke test an explicit test-local timeout budget instead of forcing it through Playwright's default 30-second cap.
- [x] Why:
  - once the no-frontier Red Team regression was fixed, GitHub Actions revealed the remaining failure was not a product bug but a test-budget mismatch: the soak test intentionally spends multiple cadence windows waiting through remount cycles, and the global 30-second budget was too tight for CI hardware jitter.
  - the failure was deterministic in Actions because the test timed out during its own `page.waitForTimeout(soakWindowMs)` block on both attempts, which means the soak coverage itself was still valuable but the enclosing timeout was wrong.
  - this completion keeps the cadence and latency assertions intact, adds an explicit 90-second budget to the soak test, and pins that requirement with a dashboard unit contract so future edits cannot silently drop the dedicated timeout.
- [x] Evidence:
  - `e2e/dashboard.smoke.spec.js`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`
  - `make test`

### Dashboard CI Repair: Restore Reactive Red Team Pane Notice Projection

- [x] Repair the `main` CI regression where the Red Team no-frontier cancel path raised the confirmation dialog but failed to surface the follow-up operator warning in the tab notice area.
- [x] Why:
  - the route had been refactored from an explicit reactive `paneNoticeValues` projection into inline `readPaneNotice(...)` calls in markup.
  - that looked equivalent in source, but it removed the template's direct reactive dependency on `paneNotices`, so the managed Red Team tab could miss notice prop updates even though `setPaneNotice(...)` ran.
  - GitHub Actions caught the real consequence in the dashboard smoke lane: the no-frontier cancel flow showed the confirm dialog and still left `[data-tab-notice="red-team"]` empty.
  - this completion restores the explicit reactive projection and adds a dashboard unit contract that pins the route to that dependency-safe pattern so the regression cannot silently re-enter.
- [x] Evidence:
  - `dashboard/src/routes/+page.svelte`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`
  - `make test`

### Runtime Surface Gate Stabilization: Separate JS-Required Proof From Geo Preemption

- [x] Stabilize the runtime-toggle adversary-sim surface gate so it deterministically proves `js_required` under the compact telemetry shape instead of depending on a single mixed profile where geo and not-a-bot precedence can preempt `JsChallengeRequired`.
- [x] Why:
  - the gate had drifted behind two real system changes at once: compact telemetry now carries the JS-missing signal in taxonomy, and the runtime policy graph evaluates geo and not-a-bot before JS.
  - the sim’s primary public probes also inject `RU` geo headers on `/sim/public/*`, so the old one-phase profile could collect geo/rate/maze/ban surfaces while never emitting any JS-required event at all.
  - this completion updates the gate to read taxonomy signals, adds a short JS-focused preemption-free phase to surface `js_required`, then restores the broader runtime profile to gather the rest of the categories.
- [x] Evidence:
  - `scripts/tests/adversary_runtime_toggle_surface_gate.py`
  - `scripts/tests/test_adversary_runtime_toggle_surface_gate.py`
  - `make test-adversarial-python-unit`
  - `make test-adversary-sim-runtime-surface`
  - `make test`

### Integration Test Stabilization: Deterministic External Rate-Limiter Fallback Proof

- [x] Stabilize the external rate-limiter fallback integration contract so `make test` no longer depends on unrelated earlier requests sharing the same `/24` rate bucket or on a high production-style limit crossing the minute window during the proof loop.
- [x] Why:
  - the runtime rate limiter intentionally buckets IPv4 traffic to `/24`, but the integration suite had been using `10.0.0.232`, which shared a bucket with many earlier `10.0.0.*` test actors in the same run.
  - the old test also reused the default limit (`80`), which made the downgrade proof slower and timing-sensitive for a contract that only needs to show fallback enforcement, not production-volume behavior.
  - this completion moves the check onto a dedicated fresh `/24` and a small deterministic fallback limit so the test proves the intended behavior without inheriting suite-order or minute-window flakiness.
- [x] Evidence:
  - `scripts/tests/integration.sh`
  - `make test-integration`
  - `make test`

### Dashboard Rescue: Restore Live Red Team Follow-up After Worktree Cleanup

- [x] Recover the live Red Team/dashboard follow-up that was accidentally removed from the working tree during root-worktree cleanup, restore the intended route/animation/operator-warning behavior on `main`, and re-verify the dashboard surfaces before retiring the rescue artifacts.
- [x] Why:
  - the root-worktree cleanup correctly backed the dirty state up into a stash and patch archive, but it incorrectly treated “recoverable” as equivalent to “still present in the live tree,” which dropped an in-progress dashboard follow-up from the working copy.
  - the highest-impact lost behavior was the progress-bar stripe timing change, but the rescued local slice also included a route-side pane-notice cleanup and an explicit operator warning when the Red Team toggle is cancelled because frontier provider keys are missing.
  - this completion restores the live behavior that still fits current `main`, keeps the older stale smoke-spec reshuffle shelved instead of replaying it blindly, and proves the restored state with the canonical dashboard make targets rather than leaving the fix half-recovered.
- [x] Evidence:
  - `dashboard/src/routes/+page.svelte`
  - `dashboard/style.css`
  - `e2e/dashboard.modules.unit.test.js`
  - `e2e/dashboard.smoke.spec.js`
  - `.spin/worktree-backups/20260314T153724Z-codex-tel-evt-live-proof/substantive-red-team-followup.patch`
  - `git stash list -n 1`
  - `make test-dashboard-unit`
  - `make test-dashboard-e2e`

### Mainline Merge Closure, CI Stabilization, and Verification-Docs Truthfulness

- [x] Merge the validated `codex/*` work onto `main`, fix the flaky Akamai adversarial latency accounting on `main`, and refresh the testing/operator docs so they describe the real verification contract and the fixture-vs-live edge distinction truthfully.
- [x] Why:
  - the merge tranche was not actually complete until `main` itself was green end to end locally and on GitHub, and the Akamai adversarial lane was still carrying a CI-only failure mode where coarse runner wall-clock could exceed the latency budget even when request-level evidence was healthy.
  - the fix moved `edge_fixture` latency accounting onto explicit request latency plus modeled think/retry time, which matches what the scenario is intended to measure and prevents incidental runner descheduling from masquerading as a product regression.
  - the accompanying docs audit corrected the user/operator surface that had drifted from reality: `make test` scope had grown beyond the old summaries, `make test-adversarial-akamai` is a local fixture proof rather than a live edge proof, and the dashboard/operator wording needed to align with the `Red Team` control path.
  - closing the merge responsibly also meant cleaning up merged remote/local topic branches while leaving the user’s still-dirty root worktree untouched.
- [x] Evidence:
  - `scripts/tests/adversarial_simulation_runner.py`
  - `scripts/tests/test_adversarial_simulation_runner.py`
  - `docs/testing.md`
  - `docs/adversarial-operator-guide.md`
  - `docs/quick-reference.md`
  - `make test-adversarial-python-unit`
  - `make test-adversarial-akamai`
  - `make test`
  - `gh run list -R atomless/Shuma-Gorath --commit 49016ab3b5faf944fe1d3a7c58bd3d928a8ae0cd --limit 10 --json databaseId,workflowName,status,conclusion,url`

### Documentation Audit and Architecture-Review Backlog Refresh

- [x] Audit the dashboard/adversary-sim docs against current `main`, add the missing `Red Team` tab guide, and convert the code-review findings from this pass into execution-ready TODO items instead of leaving them as ephemeral notes.
- [x] Why:
  - current `main` had already moved adversary-sim control into the `Red Team` tab and made `/admin/adversary-sim/status` read-only, but several docs and one active TODO still described the superseded contract.
  - the same audit exposed a few architecture-maintenance gaps that are better tracked explicitly: duplicated dashboard tab registries, the mixed writable/read-only `/admin/config` payload contract, the still-split backend adversary-sim desired state, and one remaining env-mutating test file that bypasses `lock_env()`.
  - recording those as durable TODOs keeps the repo honest about what is done, what is merely reviewed, and what still needs deliberate cleanup for long-term maintainability.
- [x] Evidence:
  - `README.md`
  - `docs/dashboard.md`
  - `docs/dashboard-tabs/README.md`
  - `docs/dashboard-tabs/red-team.md`
  - `docs/testing.md`
  - `docs/deployment.md`
  - `docs/adversarial-operator-guide.md`
  - `docs/api.md`
  - `docs/configuration.md`
  - `todos/todo.md`

### SIM-DEPLOY-2-4: Adversary-Sim Status Read Path Contract Closure

- [x] SIM-DEPLOY-2-4 Resolve the `/admin/adversary-sim/status` contract mismatch so the status read path is non-mutating and the active docs/backlog no longer claim write-on-read behaviour.
- [x] Why:
  - the Red Team single-writer tranche had already removed write-on-read behavior from current `main`, but the active TODO and a few operator docs still described the older contract, which made the backlog and written guidance untruthful.
  - this cleanup closes the stale active item, updates the operator/testing/API docs to the actual read-only status contract, and keeps the remaining `SIM-DEPLOY-2` queue focused on real production-envelope work rather than already-delivered behavior.
- [x] Evidence:
  - `src/admin/api.rs`
  - `docs/api.md`
  - `docs/testing.md`
  - `docs/dashboard.md`
  - `docs/dashboard-tabs/red-team.md`
  - `todos/todo.md`

### Ad Hoc Fermyon Reliability: Blank-Slate Deploy, Edge Signals, and Telemetry Exactness

- [x] Re-prove the Fermyon / Akamai path from a blank-slate app, add a Fermyon-native trusted-edge signal smoke, and refresh live telemetry receipts against that fresh deploy so the edge baseline is exact instead of inferred from older happy-path checks.
- [x] Why:
  - the earlier Fermyon "working" claim was incomplete because it did not prove a fresh app from zero state and did not exercise the real edge signal contract with Fermyon-native request identity semantics.
  - a shared-host-shaped trusted-forwarding smoke is not a truthful Fermyon proof, so this slice split the proof paths: SSH loopback remains the shared-host proof, while Fermyon now has its own deploy-receipt-based live signal smoke.
  - the fresh Fermyon app now proves the full operator path end to end: setup, deploy, config bootstrap, adversary-sim traffic generation, and live signal handling.
  - authoritative fingerprint on enterprise Fermyon is now documented and proven as an explicit distributed-state guardrail until `DEP-ENT-1..5` land, instead of being misread as either a generic success or a hidden failure.
  - telemetry evidence was rerun against the same fresh app so the current live Fermyon receipt carries the exact deployed `git_head` rather than the older stale-receipt caveat.
- [x] Evidence:
  - `scripts/tests/edge_signal_smoke_common.py`
  - `scripts/tests/fermyon_edge_signal_smoke.py`
  - `scripts/tests/remote_edge_signal_smoke.py`
  - `scripts/tests/test_fermyon_edge_signal_smoke.py`
  - `scripts/tests/test_remote_edge_signal_smoke.py`
  - `Makefile`
  - `docs/research/2026-03-14-fermyon-edge-signal-and-blank-slate-live-proof.md`
  - `docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`
  - `.shuma/shuma-edge-fresh-20260314-112021/fermyon-akamai-edge-deploy.json`
  - `.spin/fermyon_edge_signal_smoke.json`
  - `.spin/telemetry_fermyon_edge_evidence.json`
  - `make test-deploy-fermyon`
  - `make test-deploy-linode`
  - `make test-fermyon-edge-signal-smoke ENV_LOCAL=/Users/jamestindall/Projects/Shuma-Gorath/.env.local SHUMA_LOCAL_STATE_DIR=/Users/jamestindall/Projects/Shuma-Gorath/.worktrees/tel-evt-sparse-rows/.shuma/shuma-edge-fresh-20260314-112021`
  - `make telemetry-fermyon-edge-evidence ENV_LOCAL=/Users/jamestindall/Projects/Shuma-Gorath/.env.local SHUMA_LOCAL_STATE_DIR=/Users/jamestindall/Projects/Shuma-Gorath/.worktrees/tel-evt-sparse-rows/.shuma/shuma-edge-fresh-20260314-112021`
  - `make test-telemetry-hot-read-live-evidence ENV_LOCAL=/Users/jamestindall/Projects/Shuma-Gorath/.env.local SHUMA_LOCAL_STATE_DIR=/Users/jamestindall/Projects/Shuma-Gorath/.worktrees/tel-evt-sparse-rows/.shuma/shuma-edge-fresh-20260314-112021 REMOTE_RECEIPTS_DIR=/Users/jamestindall/Projects/Shuma-Gorath/.shuma/remotes`

### TEL-RET-2: Post-Compaction Telemetry Retention Rebaseline

- [x] TEL-RET-2-1 Capture or refresh a challenge-heavy telemetry evidence sample with retained-byte pressure by tier so raw event rows, hot-read documents, and retention metadata can be compared from live compact-schema deployments rather than from anecdotal low-volume receipts.
- [x] TEL-RET-2-2 Decide whether the current effective retention windows (`72h` high-risk raw events, `168h` monitoring summaries, `720h` monitoring rollups) should remain or change, with explicit rationale tied to the compact-schema live evidence and ADR 0009 lifecycle governance.
- [x] TEL-RET-2-3 If the evidence justifies changing retention windows, implement the config/default/bootstrap/docs/test updates together while preserving automatic purge, truthful retention health, and the current single-architecture telemetry model.
- [x] TEL-RET-2-4 Re-prove shared-host and Fermyon live telemetry budgets plus retention health after the retention decision so the tranche closes on measured operator truth, not local-only assumptions.
- [x] Why:
  - the first post-compaction receipt was still too low-volume to justify a lifecycle decision, so this tranche strengthened the live evidence until both shared-host and Fermyon had challenge-heavy recent-event samples carrying the compact `js_verification` shape.
  - the final shared-host proof shows `22` `js_verification` rows in a `27`-row recent-event sample, with sampled compact challenge rows uniformly at `146 B`, but the same host still spends far more retained bytes on hot-read documents (`26551 B`) and retention metadata than on raw eventlog values (`5039 B`).
  - because of that measured tier balance, the correct retention decision was to keep the current windows unchanged: raw compaction alone does not justify longer high-risk raw retention, and the current summary/rollup windows are already budget-green on both shared-host and Fermyon.
  - no config/default changes were made in this tranche because the evidence did not justify them; making no retention change was the correct outcome, not unfinished work.
- [x] Evidence:
  - `docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`
  - `scripts/tests/telemetry_evidence_common.py`
  - `scripts/tests/telemetry_shared_host_evidence.py`
  - `scripts/tests/telemetry_fermyon_edge_evidence.py`
  - `scripts/tests/test_telemetry_shared_host_evidence.py`
  - `scripts/tests/test_telemetry_fermyon_edge_evidence.py`
  - `docs/configuration.md`
  - `docs/observability.md`
  - `make test-telemetry-hot-read-evidence`
  - `make test-telemetry-hot-read-live-evidence`

### TEL-EVT-1: Compact Event Telemetry and Raw-Feed Truthfulness

- [x] TEL-EVT-1-5 Extend live telemetry evidence to capture representative persisted-row bytes, recent-events-tail document bytes, and bootstrap payload bytes, and prove the compact event contract improves storage/payload weight while preserving analysis and dashboard usability; treat any regression in the current `TEL-HOT` live budget envelope as tranche-blocking and treat failure to achieve a material challenge-heavy sample size reduction as a review gate.
- [x] Why:
  - the compact schema work was not done until live receipts proved it actually changed stored and served bytes on deployed targets instead of only in unit tests.
  - the refreshed shared-host evidence now shows a fresh compact `js_verification` row at `146 B` versus retained legacy rows at `259-260 B`, while shared-host bootstrap and delta stay well inside budget at `86.77 ms` / `64.15 ms`.
  - the live retained-byte breakdown also exposed the next real optimization frontier: on the current low-volume shared host, hot-read documents (`17295 B`) and retention metadata outweigh raw eventlog values (`2411 B`), so the follow-on retention reassessment has to reason about tier balance, not only raw-row compaction.
- [x] Evidence:
  - `docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`
  - `.spin/telemetry_shared_host_evidence.json`
  - `.spin/telemetry_fermyon_edge_evidence.json`
  - `scripts/tests/telemetry_shared_host_evidence.py`
  - `scripts/tests/test_telemetry_shared_host_evidence.py`
  - `make test-telemetry-hot-read-evidence`
  - `make test-telemetry-hot-read-live-evidence`

- [x] TEL-EVT-1-6 Once `TEL-EVT-1` lands with live size evidence, write the follow-on retention/lifecycle plan and active TODO tranche that re-evaluates raw event, summarized hot-read, and rollup retention windows in light of the new compact schema, preserving automatic purge/default-on lifecycle governance.
- [x] Why:
  - now that the compact schema has measured deployed footprint, leaving the next retention/lifecycle move stranded in `blocked-todo.md` would hide the real follow-on work.
  - the new live evidence makes two things explicit: compact raw rows are a real win, and the currently retained footprint is still shaped heavily by hot-read documents and retention metadata on low-volume hosts.
  - this completion closes the dependency gate by converting the blocked reminder into an execution-ready plan plus active `TEL-RET-2` TODO tranche, which keeps retention decisions evidence-driven and lifecycle-governed instead of speculative.
- [x] Evidence:
  - `docs/plans/2026-03-14-telemetry-retention-rebaseline-implementation-plan.md`
  - `docs/research/2026-03-14-compact-event-telemetry-live-evidence.md`
  - `todos/todo.md`
  - `todos/blocked-todo.md`

- [x] TEL-EVT-1-2 Replace verbose blended challenge outcome strings with a compact structured event contract that preserves analysis value without duplicating human-readable and machine-readable variants in storage, avoids per-event default provider/mode/state matrices unless a genuine non-default event deviation exists, and remains compatible with the existing hot-read bootstrap/delta path.
- [x] TEL-EVT-1-3 Make the dashboard Monitoring feed truthful: either expose a true raw persisted-event feed plus a rendered feed, or rename/reframe the current feed so it stops claiming to be raw, and keep any display-side hydration cheap enough that it does not erode the `TEL-HOT` latency gains or create duplicate heavyweight raw/rendered event objects.
- [x] TEL-EVT-1-4 Update hot-read document and monitoring bootstrap/delta paths so they use the compact event shape without regressing current latency budgets on Fermyon or shared-host targets, without reintroducing whole-keyspace scans or alternate hot-read storage paths, and without relying on schema-minification or reference-dictionary hydration tricks.
- [x] Why:
  - botness-driven challenge and maze rows were still paying to store blended `score/signals/signal_states/providers` prose even though the canonical analysis value was the compact state, botness score, and taxonomy.
  - the dashboard’s so-called raw feed was also serializing display-normalized rows rather than the persisted event shape, and the default monitoring path was still hiding `taxonomy` while reconstructing the legacy `outcome ... taxonomy[...]` string.
  - this slice replaces those verbose producer strings with compact `outcome_code`/`botness_score` persistence, keeps default monitoring and hot-read bootstrap/delta on the same compact row shape as forensic mode, and makes the dashboard raw feed stringify the raw row instead of a derived display model.
- [x] Evidence:
  - `src/runtime/effect_intents/plan_builder.rs`
  - `src/lib.rs`
  - `src/admin/api.rs`
  - `src/observability/hot_read_documents.rs`
  - `dashboard/src/lib/components/dashboard/MonitoringTab.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring-view-model.js`
  - `dashboard/src/lib/domain/ip-range-policy.js`
  - `dashboard/src/lib/components/dashboard/monitoring/RawTelemetryFeed.svelte`
  - `docs/api.md`
  - `docs/configuration.md`
  - `Makefile`
  - `make test-unit`
  - `make test-dashboard-unit` (telemetry regressions green; one unrelated pre-existing progress-bar stylesheet failure remains)

- [x] TEL-EVT-1-1 Define the compact persisted event schema as a strict evolution of the completed `TEL-HOT` architecture: separate canonical machine fields from display-derived fields, make rows sparse where semantically safe, document which fields must remain explicit instead of omitted, and forbid any new parallel telemetry storage/query path.
- [x] Why:
  - persisted `eventlog:v2` rows were still storing absent optionals as explicit `null` and were blending operator-readable outcome text together with machine taxonomy in a single `outcome` string.
  - this slice keeps the existing event-log plus hot-read architecture intact while compacting the stored row: absent optionals are now omitted, canonical taxonomy is stored in a sparse structured `taxonomy` object, and default monitoring keeps the legacy rendered `outcome ... taxonomy[...]` shape only as a compatibility presentation layer.
  - forensic/raw monitoring now exposes the compact split representation directly, which gives the next telemetry tranches a truthful machine-oriented base without introducing a second storage or query path.
- [x] Evidence:
  - `src/admin/api.rs`
  - `src/runtime/policy_taxonomy.rs`
  - `docs/api.md`
  - `docs/configuration.md`
  - `Makefile`
  - `make test-telemetry-storage`
  - `make test-telemetry-hot-read-bootstrap`

## Additional completions (2026-03-13)

### Red Team adversary-sim duration progress bar

- [x] Add a pane-local adversary-sim duration progress bar at the bottom of the `Red Team` control panel that grows across the backend-reported run window and snaps back to zero width when the operator turns the sim off.
- [x] Why:
  - the Red Team pane already had lifecycle text, but it did not visually communicate how far through the allowed sim window the current run had progressed.
  - this slice keeps the change local to the Red Team pane and uses backend truth instead of inventing a browser-only timer: the dashboard now preserves `started_at`, `ends_at`, and `remaining_seconds` from the status payload, derives progress from that timing window, and only ticks the visual bar while the pane is visible.
  - the progress fill reuses the existing striped dashboard accent background and stays inset within the shared config-panel padding so it reads as part of the pane rather than a separate chrome layer.
- [x] Evidence:
  - `dashboard/src/lib/runtime/dashboard-adversary-sim.js`
  - `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
  - `dashboard/src/routes/+page.svelte`
  - `dashboard/style.css`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`

### Dashboard pane-scoped feedback cleanup

- [x] Remove the global bottom dashboard message bar and route only valuable feedback into the owning pane or control.
- [x] Why:
  - the old bottom bar was frequently off-screen and mixed unrelated save chatter, shadow-mode notices, adversary-sim warnings, and IP-ban actions into one global sink that was easy to miss and hard to trust.
  - this slice keeps feedback close to the control surface that owns it: pane-local notices now render through the shared tab-state primitive, Red Team owns sim warnings/errors, IP Bans owns ban/unban errors, and config-save flows stop emitting generic success/progress noise.
  - the redundant `Shadow mode disabled (blocking active)` message is gone because the page header already communicates shadow-mode state, while shadow-mode write failures still surface near that header control.
- [x] Evidence:
  - `dashboard/src/routes/+page.svelte`
  - `dashboard/src/lib/components/dashboard/primitives/TabStateMessage.svelte`
  - `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
  - `dashboard/src/lib/components/dashboard/IpBansTab.svelte`
  - `dashboard/src/lib/components/dashboard/VerificationTab.svelte`
  - `dashboard/src/lib/components/dashboard/TrapsTab.svelte`
  - `dashboard/src/lib/components/dashboard/RateLimitingTab.svelte`
  - `dashboard/src/lib/components/dashboard/GeoTab.svelte`
  - `dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`
  - `dashboard/src/lib/components/dashboard/RobotsTab.svelte`
  - `dashboard/src/lib/components/dashboard/TuningTab.svelte`
  - `dashboard/src/lib/components/dashboard/AdvancedTab.svelte`
  - `dashboard/style.css`
  - `e2e/dashboard.modules.unit.test.js`
  - `e2e/dashboard.smoke.spec.js`
  - `make test-dashboard-unit`
  - `make test-dashboard-e2e PLAYWRIGHT_ARGS='--grep "route remount preserves keyboard navigation, ban/unban, verification save, and polling"'`

### Adversary-Sim Red Team tab and single-writer contract cleanup

- [x] Move adversary-sim controls into a dedicated top-level `Red Team` dashboard tab, add a page-scoped debounced controller, separate immediate UI intent from backend truth, remove `adversary_sim_enabled` from writable Advanced JSON/admin config writes, and make `GET /admin/adversary-sim/status` read-only.
- [x] Why:
  - the existing adversary-sim flow had three conflicting authorities across config, status, and pending UI state, which made the toggle feel fast but unreliable and let the page root class disagree with backend lifecycle truth.
  - this tranche tightened the architecture around one operator-facing control path, one read-only status path, and a dedicated controller that keeps the switch responsive without letting optimistic intent masquerade as backend state.
  - the dashboard now renders the sim inside a dedicated `Red Team` tab, keeps the switch on the latest operator intent, keeps lifecycle copy/root classes tied to backend status, and removes stale helper/config surfaces that encoded the older multi-authority behavior.
  - on the backend, `/admin/config` now rejects `adversary_sim_enabled` with explicit guidance to use `/admin/adversary-sim/control`, while `/admin/adversary-sim/status` reports `controller_reconciliation_required` instead of mutating stored state on read.
- [x] Evidence:
  - `dashboard/src/lib/components/dashboard/RedTeamTab.svelte`
  - `dashboard/src/lib/runtime/dashboard-red-team-controller.js`
  - `dashboard/src/lib/runtime/dashboard-body-classes.js`
  - `dashboard/src/lib/runtime/dashboard-global-controls.js`
  - `dashboard/src/routes/+page.svelte`
  - `dashboard/src/lib/domain/config-schema.js`
  - `src/admin/api.rs`
  - `docs/plans/2026-03-13-red-team-tab-adversary-sim-toggle-plan.md`
  - `docs/api.md`
  - `docs/configuration.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`
  - `make test-unit`

### TEL-HOT-1: Unified Hot-Read Telemetry Architecture

- [x] TEL-HOT-1-7 Reassess only after the shared hot-read architecture lands whether any secondary in-memory memoization or cold-tier compression is still justified.
- [x] TEL-HOT-1-6 Add canonical verification and live proof for telemetry-read budgets on Fermyon edge and shared-host deploys, including concurrent-writer correctness checks where the chosen projection contract depends on it, and update deploy skills/docs so telemetry responsiveness is part of the operator acceptance contract.
- [x] TEL-HOT-1-5 Prove the design stays shared across Fermyon and Linode: no Fermyon-only telemetry store, no SQLite split, no new external database requirement, no new whole-keyspace scans or shadow storage paths, and no correctness dependence on non-atomic shared KV mutation.
- [x] Why:
  - the earlier TEL-HOT slices had defined and wired the hot-read documents, but the tranche was not complete until the live evidence proved the design stayed unified across both deployment classes instead of quietly turning into a Fermyon-special telemetry system.
  - the final architecture proof came from two places together: backend regression coverage proving the hot bootstrap and initial delta paths stay on bounded hot-read documents without raw eventlog or whole-keyspace scans, and live budget evidence proving the same shared KV-backed design is fast on both Linode and Fermyon.
  - that live evidence also answered the final architectural question from the plan: no secondary in-memory memoization or cold-tier compression is justified now. The shared hot-read documents alone are already fast enough on both targets, so another cache/compression layer would add complexity and freshness risk without a demonstrated host-cost win.
  - this slice also closed the operator acceptance loop by making telemetry responsiveness part of the deploy contract in the Linode and Fermyon skills/docs, rather than leaving it as an undocumented manual check after health/auth smoke.
- [x] Evidence:
  - `src/admin/api.rs`
  - `src/observability/hot_read_documents.rs`
  - `src/observability/hot_read_projection.rs`
  - `scripts/tests/telemetry_shared_host_evidence.py`
  - `scripts/tests/test_telemetry_shared_host_evidence.py`
  - `Makefile`
  - `docs/research/2026-03-13-unified-hot-read-telemetry-live-evidence.md`
  - `docs/testing.md`
  - `docs/quick-reference.md`
  - `docs/deployment.md`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`
  - `skills/deploy-shuma-on-akamai-fermyon/SKILL.md`
  - `skills/deploy-shuma-on-akamai-fermyon/references/OPERATIONS.md`
  - `make test-telemetry-hot-read-bootstrap`
  - `make test-telemetry-hot-read-evidence`
  - `make test-deploy-fermyon`
  - `make telemetry-shared-host-evidence`
  - `make telemetry-fermyon-edge-evidence`
  - `make test-telemetry-hot-read-live-evidence`

### Ad Hoc Completion Records

- [x] Fix the SSH remote day-2 update/install path so live Linode `make remote-update` stays truthful through pre-swap validation and post-swap startup.
- [x] Why:
  - after the TEL-HOT live-evidence work forced a real shared-host proof, the Linode `remote-update` path exposed three coupled correctness bugs that the focused unit coverage had not yet covered end to end: nested deploy validation was reading the stale current-app Spin manifest instead of the freshly rendered `.next` manifest, the recursive `make` path was not honoring the intended manifest override strongly enough, and the post-swap runtime env was not deriving `SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL=true` for the local `http://127.0.0.1:8080` origin.
  - those bugs did not just break the update path. They also threatened the architectural truth of the generic SSH day-2 model by letting env-file residue and shared-host local-origin posture drift break a supposedly zero-friction update.
  - the fix set now makes the precedence explicit and robust: `SHUMA_SPIN_MANIFEST` can prefer process env where needed, nested deploy validation is forwarded explicitly, the generated remote update script passes the fresh manifest/upstream as GNU make command-line vars, and the local-origin insecure-http allowance is derived the same way the original Linode deploy already does.
  - this restores the broader project goal here: one truthful SSH remote update path that is low-friction for operators, but still strict about deploy validation and production posture.
- [x] Evidence:
  - `Makefile`
  - `scripts/deploy/remote_target.py`
  - `scripts/tests/test_prod_start_spin_manifest.py`
  - `scripts/tests/test_remote_target.py`
  - `make test-deploy-linode`
  - live `make remote-update` against `dummy-static-site-fresh` on `https://shuma.jamestindall.org`

- [x] Fix SSH remote prebuilt updates to validate against the freshly rendered `.next` Spin manifest instead of the previous deployment’s persisted `SHUMA_SPIN_MANIFEST`.
- [x] Why:
  - after isolating the SSH remote upstream-origin contract, the next live `remote-update` failure showed that overlay merge was also restoring `SHUMA_SPIN_MANIFEST` from the current deployment.
  - that caused `deploy-env-validate` inside the pre-swap install to inspect the old manifest and old allowed outbound hosts, which made the update path fail even though the freshly rendered `.next/spin.gateway.toml` was correct.
- [x] Evidence:
  - `scripts/deploy/remote_target.py`
  - `scripts/tests/test_remote_target.py`
  - `make test-deploy-linode`

- [x] Fix SSH remote day-2 operations so they use the remote receipt’s authoritative upstream origin instead of leaking cross-target `SHUMA_GATEWAY_UPSTREAM_ORIGIN` from shared `.env.local` state.
- [x] Why:
  - while closing the TEL-HOT live-evidence path, the active Linode `remote-update` started comparing forwarded smoke traffic against the Fermyon edge upstream because the shared local env had been updated by the Fermyon setup lane.
  - that meant the generic SSH remote layer was no longer target-truthful: provider-specific deploy state was bleeding across targets, and the remote update path was also failing to restore the SSH remote’s own upstream origin after overlay merge.
- [x] Evidence:
  - `scripts/deploy/remote_target.py`
  - `scripts/deploy/merge_env_overlay.py`
  - `scripts/deploy/linode_shared_host_setup.py`
  - `scripts/deploy_linode_one_shot.sh`
  - `scripts/tests/test_merge_env_overlay.py`
  - `scripts/tests/test_remote_target.py`
  - `scripts/tests/test_remote_edge_signal_smoke.py`
  - `make test-deploy-linode`

- [x] Fix shared-host forwarding parity smoke so it compares the forwarded public page after JS verification has been satisfied, rather than comparing the JS verification interstitial against the direct origin page.
- [x] Why:
  - the first live `make remote-update` proof after the TEL-HOT evidence tooling work failed on `/about.html` even though both gateway and origin returned `200`, because the gateway quite correctly served the JS verification page while the direct origin served the real document.
  - that exposed a proof bug in the canonical shared-host smoke contract: forwarded parity is meant to validate origin forwarding fidelity, not first-touch verification gating.
- [x] Evidence:
  - `scripts/tests/smoke_single_host.sh`
  - `scripts/tests/test_smoke_single_host.py`
  - `make test-deploy-linode`

- [x] Fix the Linode release-bundle builder to ship the prebuilt Wasm runtime artifact alongside the prebuilt dashboard assets.
- [x] Why:
  - once `remote-update` was corrected to use a prebuilt deploy baseline, the next live blocker showed up immediately: the release bundle builder only created `dist/dashboard`, so the remote prebuilt baseline correctly rejected the bundle for missing `dist/wasm/shuma_gorath.wasm`.
  - that meant the exact-committed-bundle contract was still incomplete at the bundle-builder layer, even though the remote update flow was now checking it honestly.
- [x] Evidence:
  - `scripts/deploy/build_linode_release_bundle.py`
  - `scripts/tests/test_build_linode_release_bundle.py`
  - `scripts/tests/test_deploy_linode_one_shot.py`
  - `make test-deploy-linode`

- [x] Fix `make remote-update` to use a truthful prebuilt deploy baseline on SSH remotes instead of the normal build-on-host deployment baseline.
- [x] Why:
  - the live TEL-HOT proof exposed that `remote-update` was calling `deploy-self-hosted-minimal` on the remote host, which rebuilds dashboard/runtime artifacts and therefore depends on remote build tooling.
  - that violated the exact-committed-bundle contract for day-2 updates and blocked the Linode proof when remote dashboard build prerequisites were not present.
  - the correct architecture is a separate prebuilt baseline that verifies seeded config and shipped artifacts only, then runs the normal deploy posture checks.
- [x] Evidence:
  - `Makefile`
  - `scripts/deploy/remote_target.py`
  - `scripts/tests/test_remote_target.py`
  - `docs/deployment.md`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`
  - `make test-deploy-linode`

## Additional completions (2026-03-13)

### TEL-HOT-1: Unified Hot-Read Telemetry Architecture

- [x] TEL-HOT-1-4 Rewrite `/admin/monitoring?bootstrap=1...` and adjacent hot monitoring reads to prefer the materialized hot-read documents, while keeping bounded bucket/raw reads for lazy detail, cursor, delta, and forensic follow-up.
- [x] Why:
  - the write-path projection work was only half the architecture; operators still would not feel the benefit until the admin monitoring bootstrap stopped rebuilding the same expensive payloads in the request path.
  - the clean next step was not a broad monitoring rewrite. It was to move only the canonical hot path onto the new documents: the default 24-hour, top-10 bootstrap shape used by the dashboard and the edge bounded-details posture.
  - that keeps one shared storage model while preserving the existing bounded raw paths for custom windows, forensic reads, deltas, streams, and drill-down sections.
  - the slice also closed the bootstrap cursor contract by carrying `window_end_cursor` through the hot-read documents so dashboard follow-up semantics remain intact.
- [x] Evidence:
  - `src/admin/api.rs`
  - `src/observability/hot_read_documents.rs`
  - `src/observability/hot_read_projection.rs`
  - `docs/observability.md`
  - `todos/todo.md`
  - `Makefile`
  - `make test-telemetry-hot-read-projection`
  - `make test-telemetry-hot-read-bootstrap`

- [x] TEL-HOT-1-3 Update flush, event-append, retention, and relevant admin mutation paths so the hot-read documents are maintained centrally as projections of the existing KV source of truth rather than rebuilt in the request path, without introducing multi-writer projection races.
- [x] Why:
  - the hot-read architecture was still only descriptive until the existing write paths actually maintained the new documents.
  - without that central projection layer, the eventual bootstrap rewrite would have been forced either to rebuild the same expensive monitoring payloads in the request path again or to trust documents that were never refreshed by the real event, config, ban, and retention mutations that operators cause.
  - the clean slice was to keep one shared projection module, expose only the narrow admin helpers it needed, and hook it into the existing source-of-truth write paths: counter flush, immutable event append, retention worker passes, persisted config changes, and ban/unban mutations.
  - this also let us close a small contract gap from the earlier document-definition slice by adding a first-class hot-read contract for the top-level monitoring summary, because bootstrap reads need both `summary` and `details` to become cheap together.
- [x] Evidence:
  - `src/observability/hot_read_projection.rs`
  - `src/observability/hot_read_documents.rs`
  - `src/observability/monitoring.rs`
  - `src/observability/retention.rs`
  - `src/admin/api.rs`
  - `src/admin/mod.rs`
  - `src/enforcement/ban/mod.rs`
  - `src/enforcement/ban/tests.rs`
  - `docs/observability.md`
  - `todos/todo.md`
  - `Makefile`
  - `make test-telemetry-hot-read-contract`
  - `make test-telemetry-hot-read-projection`

- [x] TEL-HOT-1-2 Define the durable hot-read document contract for monitoring bootstrap and supporting summaries (schema, freshness, bounded size, rebuild rules, and which fields remain drill-down-only).
- [x] Why:
  - the telemetry plan was ready to move past abstract architecture only once the repository had one explicit, versioned schema for what the fast monitoring bootstrap is allowed to store and serve.
  - without that contract, the next implementation slice risked slipping into ad hoc JSON blobs, drifting freshness rules, and an unbounded bootstrap payload that would help neither Fermyon edge nor shared-host operators.
  - the clean slice was to define one shared hot-read document family for bootstrap and its supporting summaries: site-scoped storage keys, freshness and repair budgets, bounded payload caps, allowed update triggers, and a hard list of expensive fields that must remain lazy drill-down rather than creeping back into the hot path.
- [x] Evidence:
  - `src/observability/hot_read_documents.rs`
  - `src/observability/mod.rs`
  - `docs/observability.md`
  - `todos/todo.md`
  - `Makefile`
  - `make test-telemetry-hot-read-contract`

- [x] TEL-HOT-1-1 Resolve the authoritative-source and correctness contract for telemetry under non-atomic KV: identify which current counters/catalogs are exact versus best-effort, and choose a hot-read projection model that does not rely on unsafe shared read-modify-write across concurrent edge writers.
- [x] Why:
  - the unified hot-read plan was not ready for implementation until the repository made the current telemetry truth contract explicit in code, because Fermyon edge KV does not support atomic multi-key mutation and the existing monitoring counters plus retention catalogs still use shared read-modify-write patterns.
  - without codifying that distinction first, it would have been too easy to build a fast bootstrap path on top of telemetry sources that are still race-prone under concurrent edge writers.
  - the clean first slice was to make the exact versus best-effort sources explicit, record the projection guardrails in a dedicated observability module, and keep the plan, docs, and backlog aligned to that contract before any hot-read document implementation starts.
- [x] Evidence:
  - `src/observability/hot_read_contract.rs`
  - `src/observability/mod.rs`
  - `docs/observability.md`
  - `docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md`
  - `todos/todo.md`
  - `Makefile`
  - `make test-telemetry-hot-read-contract`

## Additional completions (2026-03-12)

### Ad Hoc Fermyon Reliability: Live Dashboard Control Convergence and Monitoring Truthfulness

- [x] Raise edge-specific dashboard request budgets, add retry-aware adversary-sim control handling, and extend the canonical Fermyon proof so success requires the real dashboard UI and Monitoring tab to behave correctly on the deployed edge app.
- [x] Why:
  - the earlier Fermyon proof was still incomplete because endpoint-level success and cron generation did not prove that the real dashboard UI worked under edge latency and controller-lease behavior.
  - on the deployed edge app, Shadow Mode and Adversary Sim writes could take longer than shared-host/local defaults and adversary-sim control could transiently return controller-lease `409` responses with `Retry-After`.
  - that caused the UI to roll toggles back even though the backend finished enabling shortly afterwards, which made the dashboard appear broken and hid real Monitoring activity behind a misleading client-side failure.
  - the clean fix was to treat `edge-fermyon` as a distinct dashboard request-budget posture, preserve `Retry-After`, retry bounded lease/throttle failures, and require a real external dashboard smoke in the canonical deploy helper instead of trusting endpoint-only probes.
- [x] Evidence:
  - `dashboard/src/lib/domain/api-client.js`
  - `dashboard/src/lib/runtime/dashboard-adversary-sim.js`
  - `dashboard/src/lib/runtime/dashboard-global-controls.js`
  - `dashboard/src/lib/runtime/dashboard-request-budgets.js`
  - `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
  - `dashboard/src/routes/+page.svelte`
  - `e2e/dashboard.modules.unit.test.js`
  - `scripts/tests/dashboard_external_live_smoke.mjs`
  - `scripts/tests/test_deploy_fermyon_akamai_edge.py`
  - `scripts/deploy/fermyon_akamai_edge_deploy.py`
  - `src/admin/api.rs`
  - `src/admin/adversary_sim.rs`
  - `docs/research/2026-03-12-fermyon-akamai-edge-live-proof.md`
  - `skills/deploy-shuma-on-akamai-fermyon/SKILL.md`
  - `skills/deploy-shuma-on-akamai-fermyon/references/OPERATIONS.md`
  - `docs/deployment.md`
  - `make test-dashboard-unit`
  - `make test-deploy-fermyon`
  - live `make deploy-fermyon-akamai-edge`
  - standalone external smoke against `https://79b823de-37b6-4a85-b3cc-16a40738c5a7.fwf.app`
  - live dashboard verification that Shadow Mode and Adversary Sim toggles converged from the real UI and Monitoring showed a fresh simulation event

### Ad Hoc Fermyon Reliability: Dashboard Bootstrap Readiness Must Not Wait on Slow Cursor Seeding

- [x] Shorten the live Fermyon dashboard bootstrap critical path so the global Shadow Mode and Adversary Sim controls become usable once monitoring/config data is loaded, instead of staying disabled behind slow edge cursor-seeding work.
- [x] Why:
  - the live Fermyon backend was responsive, but the dashboard still looked broken because the initial Monitoring-tab bootstrap awaited slow edge cursor seeding before it loaded config or marked runtime bootstrap complete.
  - that left the global controls disabled with `Waiting for the dashboard to finish loading.` and made the live edge deployment appear non-responsive even though `/admin/session`, `/admin/config`, and `/admin/monitoring` were all succeeding.
  - the clean fix was to keep cursor seeding off the readiness critical path and stop serializing monitoring/config fetches during bootstrap, while locking that behavior in with focused dashboard regression coverage.
- [x] Evidence:
  - `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
  - `e2e/dashboard.modules.unit.test.js`
  - `make test-dashboard-unit`
  - `make test-deploy-fermyon`
  - live `make deploy-fermyon-akamai-edge`
  - live browser verification on the deployed Fermyon app showing the dashboard becoming ready and enabling the global toggles in roughly 8s instead of remaining stuck in the bootstrap-disabled state
  - live browser verification that Shadow Mode and Adversary Sim controls responded again on the deployed Fermyon app after bootstrap completed

### Ad Hoc Fermyon Reliability: Edge Adversary-Sim Generation Proof

- [x] Repair the live Fermyon / Akamai-edge adversary-sim path so enabling it on the deployed app produces observable monitoring traffic, and harden the deploy helper/tests so that failure cannot slip through again.
- [x] Why:
  - the first "live-proven" Fermyon edge baseline was incomplete because enabling adversary sim on the deployed app did not actually generate telemetry, which exposed both a real deploy/runtime gap and a coverage shortfall in the helper proof.
  - the live platform contract forced a more precise solution than the original implementation assumed: Fermyon cron requires each individual job to run no more frequently than every five minutes, the edge cron beat arrives as `GET`, and truthful operator feedback needed an immediate bounded prime on enable plus proof of a later autonomous follow-up tick.
- [x] Evidence:
  - `config/defaults.env`
  - `scripts/bootstrap/setup.sh`
  - `scripts/deploy/spin_manifest.py`
  - `scripts/deploy/fermyon_akamai_edge_setup.py`
  - `scripts/deploy/fermyon_akamai_edge_deploy.py`
  - `scripts/tests/test_prepare_fermyon_akamai_edge.py`
  - `scripts/tests/test_deploy_fermyon_akamai_edge.py`
  - `src/config/mod.rs`
  - `src/config/tests.rs`
  - `src/admin/auth.rs`
  - `src/admin/api.rs`
  - `src/admin/adversary_sim.rs`
  - `docs/research/2026-03-12-fermyon-akamai-edge-live-proof.md`
  - `skills/prepare-shuma-on-akamai-fermyon/SKILL.md`
  - `skills/prepare-shuma-on-akamai-fermyon/references/OPERATIONS.md`
  - `skills/deploy-shuma-on-akamai-fermyon/SKILL.md`
  - `skills/deploy-shuma-on-akamai-fermyon/references/OPERATIONS.md`
  - `docs/deployment.md`
  - `docs/quick-reference.md`
  - `make test-deploy-fermyon`
  - live `make deploy-fermyon-akamai-edge`
  - live edge proof that enable returned `generation.tick_count >= 1` and later polling showed a follow-up cron-driven tick beyond that baseline

### SIM2-R4-4: Config Seeding Lifecycle and Shadow-Mode Semantics

- [x] SIM2-R4-4-4 Resolve `shadow_mode` semantics end-to-end, defaulting to ephemeral runtime/session state unless a narrower exception is deliberately approved.
- [x] SIM2-R4-4-5 Update operator docs and record the final lifecycle contract in an architecture note or ADR if the scope widens.
- [x] Why:
  - `shadow_mode` was still semantically incomplete after the config-seeding work because the runtime relied on a separate early short-circuit path, long-running hosted use still emitted noisy per-request stdout logs, and monitoring blurred simulated actions with enforced outcomes.
  - the final contract is now a true shadow-execution posture layered on the normal policy/effect path, with explicit backend-authored execution metadata and operator docs describing shadow mode as a long-running shadow-tuning tool rather than a short-lived terminal-only diagnostic.
- [x] Evidence:
  - `docs/plans/2026-03-12-shadow-mode-telemetry-monitoring-truthfulness-plan.md`
  - `src/runtime/policy_pipeline.rs`
  - `src/runtime/request_flow.rs`
  - `src/runtime/request_router.rs`
  - `src/runtime/shadow_mode/mod.rs`
  - `src/runtime/effect_intents/intent_executor.rs`
  - `src/admin/api.rs`
  - `docs/configuration.md`
  - `docs/observability.md`
  - `docs/dashboard-tabs/monitoring.md`
  - `make test-shadow-mode`
  - `make test-integration`
  - `make test`

### TMON-1: Shadow-Mode Telemetry and Monitoring Truthfulness

- [x] TMON-1-1 Rebase shadow mode on the normal policy graph and effect/plan boundary instead of the current early `src/runtime/shadow_mode/mod.rs` short-circuit, so shadow mode observes the same `PolicyDecision` path as real enforcement.
- [x] TMON-1-2 Define the canonical shadow telemetry contract for shadow mode: backend-authored execution semantics (`shadow` vs `enforced`, intended action, enforcement applied, and source) instead of relying on free-text `"[TEST MODE]"` / `would_*` parsing in the dashboard.
- [x] TMON-1-3 Remove default per-request stdout logging from the hosted shadow-mode path, or explicitly isolate any retained logging behind a deliberate local-only debug contract; do not leave noisy terminal output as the implicit operator surface.
- [x] TMON-1-4 Keep shadow observability storage-bounded by distinguishing between raw-event-worthy shadow outcomes and aggregate-only pass/no-op traffic; do not solve the current gap by logging one raw event for every clean pass on busy sites.
- [x] TMON-1-3a Preserve telemetry-efficiency guarantees while doing this work: no new whole-keyspace scan paths, no new shadow-specific unbounded cardinality dimensions, and no parallel storage/query path that escapes existing bucket-indexed retention, rollup, and query-budget governance.
- [x] TMON-1-5 Update `/admin/monitoring`, delta/stream payloads, and any related presentation helpers so monitoring surfaces can distinguish "would challenge/block/maze/tarpit" from actions actually enforced.
- [x] TMON-1-6 Update dashboard monitoring summaries, trend blocks, filters, and raw-feed helpers so operators can inspect long-running shadow mode as a truthful shadow posture without heuristic string parsing or misleading enforcement language.
- [x] TMON-1-7 Add unit, integration, and dashboard end-to-end coverage proving quiet stdout behavior, shadow telemetry presence, bounded storage impact, correct monitoring rendering, and reuse of the normal policy/effect path under sustained shadow-mode traffic.
- [x] TMON-1-8 Update operator docs and verification guidance so shadow mode is described as a long-running shadow-tuning posture for hosted deployments, and close the remaining `SIM2-R4-4` semantics/docs items against that delivered contract.
- [x] Why:
  - operators need to observe Shuma’s simulated behaviour on live traffic without confusing “would have happened” telemetry with real enforcement or paying avoidable storage/query costs for long-running shadow mode.
  - the clean implementation had to reuse the real policy graph and effect executor, suppress enforcement side effects at a single boundary, keep clean pass-through traffic aggregate-only, and make monitoring truthfully distinguish shadow from enforced behaviour.
- [x] Evidence:
  - `src/runtime/effect_intents/intent_types.rs`
  - `src/runtime/effect_intents/intent_executor.rs`
  - `src/runtime/effect_intents/response_renderer.rs`
  - `src/runtime/effect_intents/plan_builder.rs`
  - `src/runtime/effect_intents.rs`
  - `src/runtime/shadow_mode/mod.rs`
  - `src/runtime/shadow_mode/tests.rs`
  - `src/runtime/policy_pipeline.rs`
  - `src/runtime/request_flow.rs`
  - `src/runtime/request_router.rs`
  - `src/observability/monitoring.rs`
  - `src/admin/api.rs`
  - `dashboard/src/lib/components/dashboard/MonitoringTab.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring-view-model.js`
  - `dashboard/src/lib/components/dashboard/monitoring/ShadowSection.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring/RecentEventsTable.svelte`
  - `dashboard/src/lib/components/dashboard/monitoring/DefenseTrendBlocks.svelte`
  - `e2e/dashboard.modules.unit.test.js`
  - `e2e/dashboard.smoke.spec.js`
  - `scripts/tests/integration.sh`
  - `scripts/tests/adversarial_browser_driver.mjs`
  - `scripts/tests/test_adversarial_browser_driver.mjs`
  - `scripts/tests/test_config_lifecycle.py`
  - `docs/configuration.md`
  - `docs/observability.md`
  - `docs/dashboard-tabs/monitoring.md`
  - `make test-shadow-mode`
  - `make test-dashboard-unit`
  - `make test-integration`
  - `make test`

## Additional completions (2026-03-11)

### Ad Hoc Dashboard UX: Shadow-Mode Header Eye Overlay

- [x] Overlay the dashboard header image with the `eye.png` marker only while `shadow_mode` is enabled, so operators can see at a glance that the current session is in logging-only posture without adding more permanent chrome.
- [x] Why:
  - shadow mode already changes runtime semantics, but the dashboard header gave no persistent visual cue once the operator scrolled past the banner/toggle area
  - the requested cue needed to stay local to the existing header, avoid disturbing the broader dashboard visual language, and keep the styling in the canonical dashboard stylesheet rather than route-local style blocks
- [x] Evidence:
  - `dashboard/src/routes/+page.svelte`
  - `dashboard/static/assets/eye.png`
  - `e2e/dashboard.modules.unit.test.js`
  - `e2e/dashboard.smoke.spec.js`
  - `docs/dashboard.md`
  - `make test-dashboard-unit`
  - `make test-dashboard-e2e PLAYWRIGHT_ARGS="--grep 'dashboard header overlays the eye only while shadow mode is enabled|dashboard login route remains functional after direct navigation and refresh'"`

### Ad Hoc Runtime Reliability: Remote Deploy Env Default Seeding

- [x] Ensure Linode deploy/update flows seed the latest `.env.local` defaults before restoring overlay values so newly introduced `SHUMA_*` runtime vars do not leave existing remotes in a stale or blank env state.
- [x] Why:
  - the telemetry tranche added `SHUMA_MONITORING_RETENTION_HOURS` and `SHUMA_MONITORING_ROLLUP_RETENTION_HOURS`
  - the live Linode proof exposed that `make remote-update` was copying the old remote `.env.local` forward verbatim, so pre-existing remotes could panic on admin reads when new required runtime vars were introduced
  - the deploy/update path now shares a reusable env-overlay merger and reseeds defaults before applying prior remote overrides
- [x] Evidence:
  - `scripts/deploy/merge_env_overlay.py`
  - `scripts/deploy/remote_target.py`
  - `scripts/deploy_linode_one_shot.sh`
  - `scripts/tests/test_remote_target.py`
  - `scripts/tests/test_deploy_linode_one_shot.py`
  - `scripts/tests/test_prepare_linode_shared_host.py`
  - `Makefile`
  - `make test-deploy-linode`
  - `make remote-update`

### Ad Hoc Verification Reliability: Telemetry Cleanup and Harness Boundary Hardening

- [x] Fix verification-only issues uncovered while proving `TEL-STORE-1` end to end: retained telemetry cleanup now clears retention bucket catalogs/worker state, adversarial browser/setup scenarios reset first-touch challenge state truthfully, header-spoofing abuse aligns to the fingerprint contract rather than stale event-stream expectations, and integration tarpit cleanup explicitly unbans all dynamically generated abuse IPs.
- [x] Why:
  - the first full-suite telemetry proof exposed a false `query_budget_exceeded` failure caused by stale `telemetry:retention:v1:*` metadata surviving `make telemetry-clean`
  - `allow_browser_allowlist` and header-spoofing scenarios were still inheriting unrelated JS/PoW/CDP/rate-pressure friction, which made fast/adversarial verification less truthful than the actual intended contracts
  - the integration tarpit cleanup path was not clearing every dynamically generated IP, leaving avoidable state leakage between runs
- [x] Evidence:
  - `src/admin/api.rs`
  - `scripts/tests/adversarial_browser_driver.mjs`
  - `scripts/tests/test_adversarial_browser_driver.mjs`
  - `scripts/tests/adversarial_simulation_runner.py`
  - `scripts/tests/test_adversarial_simulation_runner.py`
  - `scripts/tests/adversarial/scenario_manifest.v2.json`
  - `scripts/tests/adversarial/scenario_intent_matrix.v1.json`
  - `scripts/tests/test_adversarial_scenario_intent_matrix.py`
  - `scripts/tests/integration.sh`
  - `scripts/tests/test_integration_cleanup.py`
  - `make test-adversarial-fast`
  - `make test-sim2-operational-regressions`
  - `make test`

### TEL-STORE-1: Telemetry Storage and Query Efficiency Excellence

- [x] TEL-STORE-1-1 Capture shared-host telemetry evidence and cost baselines: monitoring/event key counts, telemetry-adjacent monitoring-detail key counts (`maze_hits:*`, tarpit active-bucket state, and other remaining scans), keys per retained hour, retention lag, payload sizes, and read latency for `/admin/monitoring`, `/admin/monitoring/delta`, and `/admin/monitoring/stream`.
- [x] TEL-STORE-1-2 Replace whole-keyspace scans in monitoring summary reads with bucket-catalog/index-driven reads so normal monitoring refresh cost scales with requested window, not total keyspace size.
- [x] TEL-STORE-1-3 Replace whole-keyspace scans in event-history, monitoring-delta, and monitoring-stream reads with bucket-catalog/index-driven reads while preserving cursor semantics, forensic-mode behavior, and bounded response shaping.
- [x] TEL-STORE-1-4 Eliminate or explicitly bound the remaining telemetry-adjacent scans in normal monitoring details (for example `maze_hits:*` and tarpit active-bucket state) so the full operator monitoring surface no longer quietly depends on whole-keyspace enumeration.
- [x] TEL-STORE-1-5 Define and implement a smarter retention-tier contract separating raw event evidence, operational monitoring counters, and longer-lived derived rollups, with an explicit config contract for whether those tiers remain under one governing knob or split into separate `SHUMA_*` retention controls.
- [x] TEL-STORE-1-6 Add derived telemetry rollups for the dominant monitoring views so dashboard summary reads do not repeatedly reconstruct long-window aggregates from base counters and raw events.
- [x] TEL-STORE-1-7 Upgrade monitoring cost governance from simple `hours * limit` heuristics to storage/query-aware budgets that account for bucket density, payload size, response shaping, and residual scan dependence.
- [x] TEL-STORE-1-8 Evaluate cold-tier compression only after the new read path and retention tiers are measured; reject hot-path KV compression unless evidence shows clear net benefit without harming retrieval/searchability.
- [x] TEL-STORE-1-9 Add focused verification, operator docs, and evidence receipts proving the revised telemetry model reduces shared-host storage/query cost without degrading operator visibility or forensic utility.
- [x] Evidence:
  - `docs/plans/2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md`
  - `docs/research/2026-03-11-shared-host-telemetry-storage-query-evidence.md`
  - `.spin/telemetry_shared_host_evidence.json`
  - `src/observability/key_catalog.rs`
  - `src/observability/monitoring.rs`
  - `src/observability/retention.rs`
  - `src/admin/api.rs`
  - `src/maze/mod.rs`
  - `src/tarpit/runtime.rs`
  - `src/deception/primitives.rs`
  - `config/defaults.env`
  - `scripts/tests/telemetry_shared_host_evidence.py`
  - `scripts/tests/test_telemetry_shared_host_evidence.py`
  - `docs/observability.md`
  - `docs/testing.md`
  - `todos/security-review.md`
  - `make test-telemetry-storage`
  - `make test-deploy-linode`
  - `make remote-update`
  - `make telemetry-shared-host-evidence`

## Additional completions (2026-03-12)

### P1 Fermyon / Akamai Edge Deployment Baseline

- [x] FERM-SKILL-3 Run a real Fermyon / Akamai edge deployment proof, capture the happy path and crucial gotchas, and fold the verified steps back into the Fermyon setup and deploy skills.
- [x] Why:
  - the Akamai-edge-only backlog and UI gating were intentionally blocked on a real `gateway_deployment_profile=edge-fermyon` proof rather than repo-local assumptions
  - the live path exposed real deployment-contract gaps that helper tests alone did not cover: missing defaults propagation into the Python deploy helper, missing full-config bootstrap path for fresh edge apps, and missing edge trust extraction for client IP / HTTPS posture
- [x] Evidence:
  - `docs/research/2026-03-12-fermyon-akamai-edge-live-proof.md`
  - `docs/research/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md`
  - `docs/plans/2026-03-09-fermyon-akamai-edge-baseline-prerequisite-plan.md`
  - `docs/plans/2026-03-10-fermyon-akamai-edge-skill-implementation-plan.md`
  - `scripts/deploy/fermyon_akamai_edge_setup.py`
  - `scripts/deploy/fermyon_akamai_edge_deploy.py`
  - `scripts/deploy/spin_manifest.py`
  - `scripts/config_seed.sh`
  - `scripts/tests/test_prepare_fermyon_akamai_edge.py`
  - `scripts/tests/test_deploy_fermyon_akamai_edge.py`
  - `scripts/tests/test_render_gateway_spin_manifest.py`
  - `scripts/tests/test_config_lifecycle.py`
  - `src/config/runtime_env.rs`
  - `src/config/mod.rs`
  - `src/config/tests.rs`
  - `src/lib.rs`
  - `src/lib_tests/security.rs`
  - `src/admin/api.rs`
  - `src/admin/auth.rs`
  - `skills/prepare-shuma-on-akamai-fermyon/SKILL.md`
  - `skills/prepare-shuma-on-akamai-fermyon/references/OPERATIONS.md`
  - `skills/deploy-shuma-on-akamai-fermyon/SKILL.md`
  - `skills/deploy-shuma-on-akamai-fermyon/references/OPERATIONS.md`
  - `.shuma/fermyon-akamai-edge-setup.json`
  - `.shuma/fermyon-akamai-edge-deploy.json`
  - `make test-deploy-fermyon`
  - `make deploy-fermyon-akamai-edge`
  - live proof checks:
    - `GET /dashboard/login.html -> 200`
    - `GET /index.html -> 200`
    - authenticated `GET /admin/config -> 200`

## Additional completions (2026-03-10)

### Process Auditability and Completion Recording

- [x] Record the requirement that ad hoc code-change tasks without pre-written TODOs must still leave a dated completion entry in `todos/completed-todo-history.md`, with a clear description of what changed, why it was done, and the evidence/verification.
- [x] Why:
  - ad hoc bugfixes and small architecture/process slices were leaving less durable audit trail than backlog-driven work
  - completion history must remain the minimum durable record even when there was no active TODO entry to move
- [x] Evidence:
  - `AGENTS.md`
  - `todos/completed-todo-history.md`

### P1 Fermyon / Akamai Edge Deployment Baseline

- [x] FERM-SKILL-1 Create an agent-oriented Fermyon / Akamai edge setup skill that captures the required account, mode, edge property, origin, and secret inputs, and leaves a deploy-ready handoff comparable to the Linode setup path.
- [x] FERM-SKILL-2 Refactor and tighten the existing Fermyon deploy skill so it is an agent-executable deploy path rather than a human runbook, with truthful mode selection, artifacts, and failure handling.
- [x] Evidence:
  - `docs/plans/2026-03-10-fermyon-akamai-edge-skill-implementation-plan.md`
  - `docs/research/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md`
  - `scripts/deploy/setup_common.py`
  - `scripts/deploy/fermyon_akamai_edge_setup.py`
  - `scripts/deploy/fermyon_akamai_edge_deploy.py`
  - `scripts/prepare_fermyon_akamai_edge.py`
  - `scripts/deploy_fermyon_akamai_edge.py`
  - `scripts/tests/test_prepare_fermyon_akamai_edge.py`
  - `scripts/tests/test_deploy_fermyon_akamai_edge.py`
  - `Makefile`
  - `skills/prepare-shuma-on-akamai-fermyon/SKILL.md`
  - `skills/prepare-shuma-on-akamai-fermyon/references/OPERATIONS.md`
  - `skills/deploy-shuma-on-akamai-fermyon/SKILL.md`
  - `skills/deploy-shuma-on-akamai-fermyon/references/OPERATIONS.md`
  - `docs/deployment.md`
  - `docs/quick-reference.md`
  - `docs/index.md`
  - `README.md`
  - `make test-deploy-fermyon`
  - live proof reached the real Fermyon device-login boundary after the helper automatically handled the known `spin aka` PAT-login panic; the remaining external blocker is recorded separately under `FERM-SKILL-3`

## Additional completions (2026-03-09)

### P2 Edge Control Expansion

- [x] AK-RG-1 Write a concise architecture note (or ADR if scope broadens) that defines exact semantics for Akamai controls on Rate Limiting and GEO (`off`, `additive`, `authoritative` behavior, precedence, fallback, and trust boundaries).
- [x] Evidence:
  - `docs/plans/2026-03-09-akamai-rate-geo-integration-semantics-note.md`
  - `dashboard/src/lib/components/dashboard/GeoTab.svelte`
  - `dashboard/src/lib/components/dashboard/RateLimitingTab.svelte`
  - `docs/dashboard-tabs/geo.md`
  - `docs/dashboard-tabs/rate-limiting.md`
  - `e2e/dashboard.modules.unit.test.js`
  - `e2e/dashboard.smoke.spec.js`

## Additional completions (2026-03-08)

### P0 Durable Operator State Lifecycle

- [x] OPS-STATE-1 Move durable operator state out of `.spin` into `.shuma`, narrow `make clean`, add `make reset-local-state`, and prove the full Linode setup/deploy/cleanup/remote-update path end to end.
- [x] Evidence:
  - `docs/plans/2026-03-08-durable-operator-state-and-clean-reset-semantics-plan.md`
  - `Makefile`
  - `.gitignore`
  - `scripts/deploy/remote_target.py`
  - `scripts/deploy/linode_shared_host_setup.py`
  - `scripts/deploy_linode_one_shot.sh`
  - `scripts/build_site_surface_catalog.py`
  - `docs/deployment.md`
  - `docs/quick-reference.md`
  - `README.md`
  - `skills/prepare-shared-host-on-linode/SKILL.md`
  - `skills/prepare-shared-host-on-linode/references/OPERATIONS.md`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`
  - live proof artifacts under `.shuma/`:
    - `linode-shared-host-setup.json`
    - `catalogs/dummy_static_site.surface-catalog.json`
    - `remotes/dummy-static-site-prod.json`
  - live proof outcomes:
    - fresh Linode setup + Shuma deploy on `https://172.239.98.201.sslip.io/dashboard`
    - `make clean` preserved durable `.shuma` state and `make remote-status` remained usable
    - `make remote-update` deployed a temporary dashboard CSS marker to the live host
    - a second `make remote-update` removed that marker and refreshed the remote receipt metadata

## Additional completions (2026-03-07)

### SIM2-R4-4: Config Seeding Lifecycle and Shadow-Mode Semantics

- [x] SIM2-R4-4-1 Make runtime start paths (`make dev`, `make dev-closed`, `make run`, `make run-prebuilt`, `make prod`) read-only with respect to persisted KV config.
- [x] SIM2-R4-4-2 Keep setup/backfill explicit via `make setup`, `make setup-runtime`, and `make config-seed`, with clear diagnostics for missing, stale, and invalid persisted config.
- [x] SIM2-R4-4-3 Add deterministic migration coverage for missing config, new-key backfill, schema-complete no-op, and invalid persisted JSON.
- [x] Evidence:
  - `scripts/config_seed.sh`
  - `scripts/bootstrap/verify-runtime.sh`
  - `scripts/tests/test_config_lifecycle.py`
  - `scripts/tests/test_prod_start_spin_manifest.py`
  - `Makefile`
  - `README.md`
  - `docs/configuration.md`
  - `docs/deployment.md`
  - `docs/quick-reference.md`

### REMOTE-OPS-1: Generic SSH Remote Target Layer

- [x] REMOTE-OPS-1-1 Define the normalized gitignored remote receipt contract at `.spin/remotes/<name>.json` with target identity, SSH transport, runtime contract, deploy contract, and deploy metadata.
- [x] REMOTE-OPS-1-2 Keep `.env.local` limited to the active remote selector (`SHUMA_ACTIVE_REMOTE=<name>`) plus normal env-only secrets; do not store structured remote target state there.
- [x] REMOTE-OPS-1-3 Make provider-specific setup/deploy paths write the same normalized receipt schema, with provider-specific extension fields allowed but ignored by generic remote maintenance commands.
- [x] REMOTE-OPS-1-4 Implement the first generic backend as `ssh_systemd`; do not claim identical lifecycle semantics for non-SSH backends such as Fermyon in this tranche.
- [x] REMOTE-OPS-1-5 Add thin repo-local helper dispatch for:
  - `make remote-use REMOTE=<name>`
  - `make remote-update`
  - `make remote-start`
  - `make remote-stop`
  - `make remote-status`
  - `make remote-logs`
  - `make remote-open-dashboard`
- [x] REMOTE-OPS-1-6 Define `make remote-update` truthfully as: build the exact local committed `HEAD` bundle, upload/install it on the selected `ssh_systemd` remote, preserve remote `.env.local` and `.spin`, restart the service, run smoke, refresh receipt metadata, and attempt rollback if smoke fails; do not imply uncommitted worktree sync.
- [x] REMOTE-OPS-1-7 Keep target naming truthful: do not add ambiguous generic commands such as `make dev-remote` or `make dev-prod-remote` unless the implementation can guarantee those semantics across the supported backend contract.
- [x] REMOTE-OPS-1-8 Update deploy/setup skills and operator docs so the Linode path becomes one provider-specific writer of the generic remote receipt, while the day-2 remote maintenance path is provider-agnostic within the `ssh_systemd` contract.
- [x] Evidence:
  - `docs/plans/2026-03-07-generic-ssh-remote-maintenance-layer-design.md`
  - `scripts/deploy/remote_target.py`
  - `scripts/deploy/local_env.py`
  - `scripts/manage_remote_target.py`
  - `scripts/deploy/linode_shared_host_setup.py`
  - `scripts/deploy_linode_one_shot.sh`
  - `scripts/tests/test_remote_target.py`
  - `scripts/tests/test_prepare_linode_shared_host.py`
  - `scripts/tests/test_deploy_linode_one_shot.py`
  - `Makefile`
  - `docs/deployment.md`
  - `README.md`
  - `scripts/README.md`
  - `skills/prepare-shared-host-on-linode/SKILL.md`
  - `skills/prepare-shared-host-on-linode/references/OPERATIONS.md`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`

## Additional completions (2026-03-06)

### P0 Shared-Host Deployment Readiness

#### DEP-SH-3: Capture One Real Shared-Host Deployment Evidence Set

- [x] DEP-SH-3-1 Run one end-to-end shared-host deployment on the canonical path and archive preflight, smoke, rollback, and operator notes.
- [x] DEP-SH-3-2 Feed any newly discovered blockers back into the active or blocked backlog with precise evidence.
- [x] DEP-SH-3-3 Fold the verified successful path and any crucial Linode-specific gotchas back into the relevant Linode setup/deploy skills and operations references so future agents can deploy Shuma on the shortest known-good path without rediscovering failure modes.
- [x] Evidence:
  - `docs/research/2026-03-06-linode-shared-host-live-proof.md`
  - `skills/prepare-shared-host-on-linode/SKILL.md`
  - `skills/prepare-shared-host-on-linode/references/OPERATIONS.md`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`
  - `docs/deployment.md`
  - `scripts/tests/smoke_single_host.sh`
  - `scripts/deploy/gateway_surface_catalog.py`
  - `src/runtime/upstream_canonicalization.rs`
  - `Makefile`

#### DEP-SH-SETUP-2: Close Same-Host Linode Handoff Gap

- [x] DEP-SH-SETUP-2-1 Extend the Linode deployment path so a prepared same-host Linode origin can hand off directly to Shuma without reprovisioning drift or manual out-of-band restaging.
- [x] DEP-SH-SETUP-2-2 Prove the same-host handoff using `../dummy_static_site` as the first static HTML acid test and archive timestamped operator evidence.
- [x] DEP-SH-SETUP-2-3 Fold the verified same-host happy path and gotchas back into `skills/prepare-shared-host-on-linode/SKILL.md`, `skills/prepare-shared-host-on-linode/references/OPERATIONS.md`, `skills/deploy-shuma-on-linode/SKILL.md`, and `skills/deploy-shuma-on-linode/references/OPERATIONS.md`.
- [x] Evidence:
  - `docs/research/2026-03-06-linode-shared-host-live-proof.md`
  - `scripts/deploy_linode_one_shot.sh`
  - `scripts/tests/test_deploy_linode_one_shot.py`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`
  - `skills/prepare-shared-host-on-linode/SKILL.md`
  - `skills/prepare-shared-host-on-linode/references/OPERATIONS.md`
  - `docs/deployment.md`
  - `README.md`

#### DEP-SH-SETUP-1: Publish Generic Linode Setup Skill And Local Surface-Catalog Helper

- [x] DEP-SH-SETUP-1-1 Publish a generic shared-host Linode setup skill that gathers operator prerequisites and prepares the Shuma deploy handoff bundle.
- [x] DEP-SH-SETUP-1-2 Add a deterministic local docroot-to-surface-catalog helper so simple sites do not require a human-authored sitemap before gateway collision preflight and smoke.
- [x] DEP-SH-SETUP-1-3 Refactor the Linode setup flow from a human-runbook-shaped skill into an agent-executable helper plus receipt contract.
- [x] Evidence:
  - `docs/plans/2026-03-06-linode-shared-host-setup-skill-and-handoff-plan.md`
  - `scripts/build_site_surface_catalog.py`
  - `scripts/prepare_linode_shared_host.py`
  - `scripts/deploy/linode_shared_host_setup.py`
  - `scripts/site_surface_catalog.py`
  - `scripts/tests/test_build_site_surface_catalog.py`
  - `scripts/tests/test_prepare_linode_shared_host.py`
  - `skills/prepare-shared-host-on-linode/SKILL.md`
  - `skills/prepare-shared-host-on-linode/references/OPERATIONS.md`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`
  - `docs/deployment.md`
  - `README.md`
  - `docs/index.md`
  - `scripts/README.md`

#### DEP-SH-1: Align Shared-Host Deployment Artifacts with the Gateway-First Production Contract

- [x] DEP-SH-1-1 Update Linode/shared-host scripts, skills, and docs so they emit the correct production env/profile and gateway upstream contract.
- [x] DEP-SH-1-2 Require gateway, origin-lock, and admin-edge confirmations in the shared-host path; remove stale defaults that contradict production validation.
- [x] DEP-SH-1-3 Keep `spin.toml` outbound requirements, deployment helpers, and runbooks in sync for shared-host deployment personas.
- [x] Evidence:
  - `docs/plans/2026-03-06-linode-shared-host-readiness-implementation-plan.md`
  - `scripts/deploy_linode_one_shot.sh`
  - `scripts/deploy/render_gateway_spin_manifest.py`
  - `scripts/deploy/spin_manifest.py`
  - `scripts/deploy/build_linode_release_bundle.py`
  - `scripts/tests/test_deploy_linode_one_shot.py`
  - `scripts/tests/test_build_linode_release_bundle.py`
  - `scripts/tests/test_render_gateway_spin_manifest.py`
  - `scripts/tests/test_prod_start_spin_manifest.py`
  - `docs/deployment.md`
  - `README.md`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`
  - `scripts/README.md`

#### DEP-SH-2: Make Deployment Validation and Smoke Authoritative

- [x] DEP-SH-2-1 Make `make deploy-env-validate` the canonical shared-host preflight path.
- [x] DEP-SH-2-2 Add a canonical post-deploy smoke path that proves upstream forwarding, reserved-route ownership, and admin access posture.
- [x] DEP-SH-2-3 Ensure deployment help and docs use truthful names and describe real blast radius, assumptions, and rollback steps.
- [x] Evidence:
  - `Makefile`
  - `scripts/tests/smoke_single_host.sh`
  - `scripts/deploy/gateway_surface_catalog.py`
  - `scripts/deploy/select_gateway_smoke_path.py`
  - `scripts/tests/test_smoke_single_host.py`
  - `scripts/tests/test_select_gateway_smoke_path.py`
  - `scripts/deploy_linode_one_shot.sh`
  - `scripts/tests/test_deploy_linode_one_shot.py`
  - `docs/deployment.md`
  - `docs/quick-reference.md`
  - `README.md`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`

## Security review closures (2026-03-06)

- [x] Retired stale rate-limiter TOCTOU finding from `todos/security-review.md`: external `rate_limiter` now uses Redis-backed atomic `INCR` + `EXPIRE` enforcement with explicit outage posture handling and drift observability. Remaining enterprise multi-instance strictness work is tracked under `DEP-ENT-1..5` in `todos/todo.md`.
- [x] Retired stale admin-hardening finding from `todos/security-review.md`: canonical production deployment validation now fails on missing or overbroad `SHUMA_ADMIN_IP_ALLOWLIST` and on missing `SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED`, with deployment docs and runbooks updated to require upstream `/admin/login` and `/admin/*` rate limits.

## Additional completions (2026-03-05)

### P0 Deployment Path Excellence (Single-Host + Akamai/Fermyon)

#### DEP-GW-POST: Gateway Follow-On Hardening

- [x] DEP-GW-POST-1 Add a wasm32-capable TLS failure integration harness (expired/untrusted/hostname-mismatch cert matrix) to strengthen upstream trust-path testing beyond native shadow-mode transport stubs.
- [x] Added wasm TLS trust-path harness and unit coverage:
  - `scripts/tests/gateway_tls_wasm_harness.py`
  - `scripts/tests/test_gateway_tls_wasm_harness.py`
  - `make test-gateway-wasm-tls-harness`
- [x] DEP-GW-POST-2 Add an optional active origin-bypass probe contract/tooling path (environment-permitting) to complement origin-lock attestation with executable verification.
- [x] Added optional active origin-bypass probe and unit coverage:
  - `scripts/deploy/probe_gateway_origin_bypass.py`
  - `scripts/tests/test_probe_gateway_origin_bypass.py`
  - `make test-gateway-origin-bypass-probe`
- [x] Updated operator/testing docs for both follow-on hardening paths:
  - `docs/deployment.md`
  - `docs/testing.md`
  - `scripts/README.md`

#### DEP-GW-1: Gateway-Only Existing-Site Integration (Only Production Mode)

##### DEP-GW-1-0: Evidence and Harness First

- [x] DEP-GW-1-0-1 Add gateway transport telemetry vocabulary and counters before behavior changes:
  - `forward_attempt_total`, `forward_success_total`, `forward_failure_total{class}`, `forward_latency_ms`;
  - request-level provenance fields (`transport_class`, `upstream_origin`, `forward_reason`).
- [x] DEP-GW-1-0-2 Add deterministic upstream fixture for integration tests (echo method/path/query/headers/body signature) and wire via Makefile test targets.
- [x] DEP-GW-1-0-3 Add failure-injection harness for timeout, connection-reset, TLS/transport error, and upstream non-2xx classification.
- [x] DEP-GW-1-0-4 Add adversarial canonicalization tests (ambiguous headers/content-type/path encoding) to establish parser-differential regression baseline.
- [x] Added gateway fixture and harness tooling:
  - `scripts/tests/gateway_upstream_fixture.py`
  - `scripts/tests/gateway_failure_harness.py`
  - `scripts/tests/test_gateway_failure_harness.py`
- [x] Added targeted Makefile verification gate: `make test-gateway-harness`.

##### DEP-GW-1-1: Contract and Guardrails (Spin-Aligned)

- [x] DEP-GW-1-1-10 Add reserved-route collision preflight contract:
  - compare Shuma-owned routes against discovered origin public surface before cutover;
  - fail preflight on unresolved collisions and emit deterministic remediation report.
- [x] Added deploy preflight guardrail script:
  - `scripts/deploy/validate_gateway_route_collisions.py`
- [x] Added reserved-route preflight unit coverage:
  - `scripts/tests/test_validate_gateway_route_collisions.py`
- [x] Wired preflight guardrail into deploy validation:
  - `make deploy-env-validate`

##### DEP-GW-1-2: Runtime Transport Refactor (No Policy Drift)

- [x] DEP-GW-1-2-1 Introduce `src/runtime/upstream_proxy.rs` as the single forwarding adapter behind capability-aware runtime boundary.
- [x] Added forwarding adapter and loop-hop guard foundation:
  - `src/runtime/upstream_proxy.rs`
  - `src/runtime/upstream_canonicalization.rs`
  - `src/runtime/upstream_telemetry.rs`
- [x] Routed typed `ForwardAllow` response intent through the adapter in effect-intent rendering:
  - `src/runtime/effect_intents/intent_types.rs`
  - `src/runtime/effect_intents/plan_builder.rs`
  - `src/runtime/effect_intents/response_renderer.rs`

##### DEP-GW-1-1: Contract and Guardrails (Spin-Aligned) - remaining tranche completion

- [x] DEP-GW-1-1-1 Publish ADR/addendum for gateway-only posture.
- [x] DEP-GW-1-1-2 Define upstream config contract (single-origin v1).
- [x] DEP-GW-1-1-3 Add env/runtime validation for gateway contract and invalid posture rejection.
- [x] DEP-GW-1-1-4 Add deploy guardrail verifying Spin outbound capability alignment.
- [x] DEP-GW-1-1-5 Add guardrail docs for Spin outbound limitations.
- [x] DEP-GW-1-1-6 Add outbound pressure governance defaults and docs.
- [x] DEP-GW-1-1-7 Add unit tests for config/guardrail parser and error messages.
- [x] DEP-GW-1-1-8 Add upstream loop-prevention guardrails and telemetry classification.
- [x] DEP-GW-1-1-9 Add target-specific origin-lock/auth contract and validation.
- [x] DEP-GW-1-1-11 Add explicit TLS security contract for upstream HTTPS and transport-class taxonomy.
- [x] Evidence:
  - `docs/adr/0011-gateway-only-upstream-contract.md`
  - `src/config/mod.rs`
  - `src/config/tests.rs`
  - `scripts/deploy/validate_gateway_contract.py`
  - `scripts/tests/test_validate_gateway_contract.py`
  - `docs/deployment.md`

##### DEP-GW-1-2: Runtime Transport Refactor (No Policy Drift) - remaining tranche completion

- [x] DEP-GW-1-2-2 Add typed allow transport intent in effect-intent system.
- [x] DEP-GW-1-2-3 Remove local allow-response exits from `src/runtime/request_flow.rs`.
- [x] DEP-GW-1-2-4 Update `plan_builder`/`response_renderer` to forward allow outcomes upstream.
- [x] DEP-GW-1-2-5 Keep early-route local ownership explicit for control-plane/policy-owned paths.
- [x] DEP-GW-1-2-6 Implement strict request canonicalization before forwarding.
- [x] DEP-GW-1-2-7 Implement strict response canonicalization from upstream to client.
- [x] DEP-GW-1-2-8 Add explicit forward failure taxonomy and fail-closed handling.
- [x] DEP-GW-1-2-9 Remove native/front-door allow-path runtime branches and stale local allow behavior.
- [x] DEP-GW-1-2-10 Implement redirect/cookie compatibility policy with authority confinement.
- [x] DEP-GW-1-2-11 Publish and enforce gateway v1 protocol support matrix.
- [x] Evidence:
  - `src/runtime/request_flow.rs`
  - `src/runtime/upstream_proxy.rs`
  - `src/runtime/upstream_canonicalization.rs`
  - `tests/routing_order_integration.rs`
  - `docs/deployment.md`

##### DEP-GW-1-3: Integration, Security, and Operationalization

- [x] DEP-GW-1-3-1 Integration tests for allow-path upstream fidelity.
- [x] DEP-GW-1-3-2 Integration tests proving enforcement outcomes remain local.
- [x] DEP-GW-1-3-3 Security tests for forwarded-header spoof rejection/regen and privileged stripping.
- [x] DEP-GW-1-3-4 Deterministic malformed/ambiguous canonicalization coverage.
- [x] DEP-GW-1-3-5 Origin-bypass risk controls captured in deploy runbook + origin-lock guardrails.
- [x] DEP-GW-1-3-6 Gateway smoke Makefile target added.
- [x] DEP-GW-1-3-7 Deployment docs and Linode/Fermyon skills updated for gateway-only cutover/rollback and origin-auth lifecycle.
- [x] DEP-GW-1-3-8 Shared-host discovery outputs integrated into gateway onboarding checklist.
- [x] DEP-GW-1-3-9 Loop-prevention tests for startup and runtime loop signatures.
- [x] DEP-GW-1-3-10 Deployment-profile integration coverage for shared-server and edge/Fermyon.
- [x] DEP-GW-1-3-11 Redirect/cookie compatibility integration coverage for existing-site parity.
- [x] DEP-GW-1-3-12 Explicit profile CI verification gates added.
- [x] DEP-GW-1-3-13 Upstream trust-path checks for TLS transport classification and origin-auth contract failures.
- [x] Evidence:
  - `Makefile`
  - `.github/workflows/ci.yml`
  - `.github/workflows/release-gate.yml`
  - `tests/routing_order_integration.rs`
  - `scripts/tests/gateway_failure_harness.py`
  - `scripts/tests/test_gateway_failure_harness.py`
  - `skills/deploy-shuma-on-linode/SKILL.md`
  - `skills/deploy-shuma-on-akamai-fermyon/SKILL.md`

##### DEP-GW-1-4: Product Cleanup and Positioning Consistency

- [x] DEP-GW-1-4-1 Remove front-door/native production guidance from deployment docs/help text.
- [x] DEP-GW-1-4-2 Ensure operator journeys present gateway-only production posture.
- [x] DEP-GW-1-4-3 Remove stale terminology in Makefile/docs implying dual-mode production support.
- [x] DEP-GW-1-4-4 Perform post-implementation conformance review.
- [x] DEP-GW-1-4-5 Perform codebase cleanup and knock-on architecture review.
- [x] Evidence:
  - `docs/research/2026-03-05-gateway-first-tranche-conformance-review.md`
  - `docs/research/2026-03-05-gateway-first-post-tranche-cleanup-review.md`

## Additional completions (2026-03-03)

### P0 Adversarial Traffic Simulation Program

#### Runtime heartbeat decoupling, parity hardening, and cleanup closure

- [x] SIM-DEPLOY-1 Re-evaluate current dev-only adversary-sim availability posture and deployment-path split (`runtime-dev` vs production) against product ambition; define decision criteria, abuse safeguards, tenant/isolation controls, explicit operator consent model, cost controls, and rollback strategy for possible production enablement.
- [x] SIM-CLEAN-1 After `SIM-ARCH-2`/`SIM-ARCH-3`, run a rigorous runtime+CI dead-code sweep and remove superseded deterministic-generation code paths (obsolete hardcoded batch builders, duplicate action definitions, and unused helper utilities) introduced before shared-corpus convergence.
- [x] SIM-HB-OOP-1 Introduce a dedicated internal adversary beat endpoint and move generation execution out of `/admin/adversary-sim/control` response path.
- [x] SIM-HB-OOP-2 Remove request-lifecycle-driven heartbeat execution from runtime entrypoint and make status diagnostics report explicit out-of-process heartbeat ownership.
- [x] SIM-HB-OOP-3 Implement transient Rust supervisor worker (`spawn-on-enable`, 1s cadence default, bounded retries/backoff) that exits on toggle-off, run-window expiry, or server unreachability.
- [x] SIM-HB-OOP-4 Add host launch adapters and operator docs for supervisor execution across target environments (local `make dev`, systemd/single-host, container sidecar, and external edge supervisor service).
- [x] SIM-HB-OOP-5 Enforce strict shutdown/off reconciliation and ephemeral toggle semantics across stop/restart paths: after server stop, state reconciles to `off`, no generator activity remains, and next start defaults to off.
- [x] SIM-HB-OOP-6 Deprecate and remove dashboard/runtime reliance on `POST /admin/adversary-sim/tick` once out-of-process beat ownership is live.
- [x] SIM-CLEAN-2 After `SIM-HB-OOP-6`, run a rigorous dead-code sweep for heartbeat migration fallout: remove request-loop supervisor remnants, deprecated tick endpoint wiring, stale dashboard runtime adapters, and superseded diagnostics fields/contracts.
- [x] SIM-DET-2 Add deterministic config-profiled coverage pass for config-dependent surfaces (GEO and optional IP-range actions) in automated verification only (CI/test harness), so category-level event emission is guaranteed without mutating operator runtime simulation configuration.
- [x] SIM-DET-3 Add runtime-toggle integration assertions that fail when required deterministic surface categories (challenge, JS, PoW, maze/tarpit, rate, fingerprint/CDP, ban, GEO-configured) are missing from observed event telemetry.
- [x] SIM-DET-7 Ensure automated verification telemetry remains ephemeral: CI/test adversarial traffic must not pollute operator runtime telemetry history (ephemeral stores and/or mandatory teardown cleanup).
- [x] SIM-TRUST-1 Remove simulation-context forwarded-IP trust bypass and require simulation requests to satisfy the same trust-boundary conditions as external traffic.
- [x] SIM-TRUST-2 Add enforcement/telemetry parity tests proving simulated and external-equivalent requests follow identical policy decisions and event accounting (differing only by simulation metadata tags).
- [x] SIM-CLEAN-3 End-of-tranche final code hygiene pass for all SIM surfaces (runtime, dashboard, CI harness, docs/tests): remove dead modules/branches/contracts, collapse temporary compatibility shims, and fail verification if any open TODO references code paths already removed or renamed.
- [x] Added production decision criteria dossier: `docs/research/2026-03-03-adversary-sim-production-availability-decision-criteria.md`.
- [x] Added runtime surface-category integration gate: `adversary_sim_runtime_toggle_emits_required_defense_surface_categories` (`src/admin/api.rs`) and wired to `make test-adversary-sim-lifecycle`.
- [x] Added CI telemetry ephemerality cleanup contract and tests: `scripts/tests/adversarial_simulation_runner.py` and `scripts/tests/test_adversarial_simulation_runner.py`.

## Additional completions (2026-03-02)

### P0 Adversarial Traffic Simulation Program

#### Toggle lifecycle hardening (`SIM-LEARN-1` to `SIM-LEARN-4`)

- [x] SIM-LEARN-1 Capture a concise adversary-toggle incident report and lifecycle invariants doc (what previously broke, why, and non-negotiable state semantics for toggle-on/off, auto-window expiry, server stop, and restart) and link it from SIM operator docs.
- [x] SIM-LEARN-2 Add targeted regression tests for the exact failure modes previously seen: toggle no-op, on->off bounce, stale enabled state after server restart, control/status disagreement, and supervisor-not-running while UI claims enabled.
- [x] SIM-LEARN-3 Add a fast deterministic verification target (single command) that validates toggle lifecycle end-to-end in runtime-dev before any SIM tranche merge.
- [x] SIM-LEARN-4 Add explicit structured diagnostics for toggle lifecycle troubleshooting (control decision, state transitions, supervisor heartbeat, last successful beat) so failures can be triaged without deep code spelunking.
- [x] Added incident/invariants report: `docs/research/2026-03-02-adversary-toggle-incident-report-and-lifecycle-invariants.md`.
- [x] Added lifecycle regression gate: `make test-adversary-sim-lifecycle`.
- [x] Extended status diagnostics contract with structured `lifecycle_diagnostics` in `/admin/adversary-sim/status`.

#### Shared deterministic corpus convergence (`SIM-ARCH-1` to `SIM-ARCH-4`)

- [x] SIM-ARCH-1 Define a canonical deterministic attack corpus contract (versioned artifact + runtime-safe and CI-oracle profiles) consumed by both runtime and CI executors.
- [x] SIM-ARCH-2 Refactor runtime deterministic generator to consume the shared corpus instead of hardcoded request batches, while preserving guardrails and bounded generation envelopes.
- [x] SIM-ARCH-3 Refactor CI Python oracle to consume the same shared corpus for attacker-plane action definitions while retaining its existing setup/gates/repeatability/report responsibilities.
- [x] SIM-ARCH-4 Add drift guards that fail when runtime and CI oracle execute different corpus revisions or taxonomy mappings for the deterministic lane.
- [x] Added canonical corpus artifact: `scripts/tests/adversarial/deterministic_attack_corpus.v1.json` and wired runtime/CI metadata exposure (`src/admin/adversary_sim.rs`, `scripts/tests/adversarial_simulation_runner.py`).
- [x] Added deterministic corpus parity gate and Makefile wiring: `scripts/tests/check_adversarial_deterministic_corpus.py`, `make test-adversarial-deterministic-corpus`.

## Additional completions (2026-03-01)

### P0 Adversarial Traffic Simulation Program

#### SIM-DET-1: Deterministic Lane Coverage Audit After Autonomous Heartbeat Decoupling

- [x] SIM-DET-1 Run deterministic-lane coverage audit after heartbeat decoupling and document request-surface coverage across challenge variants, JS pressure, PoW, GEO stimulation, maze/tarpit, rate pressure, fingerprint/CDP signals, and ban paths.
- [x] Expanded deterministic runtime-toggle request mix in `src/admin/adversary_sim.rs` to include explicit PoW verify abuse, tarpit progress abuse, fingerprint mismatch probe, challenge submit abuse, CDP probe, and same-IP rate bursts.
- [x] Added deterministic request-mix contract tests in `src/admin/adversary_sim.rs` and archived audit findings in `docs/research/2026-03-01-sim-deterministic-lane-coverage-audit.md`.
- [x] Opened immediate remediation follow-ups for config-dependent category emission guarantees (`SIM-DET-2`, `SIM-DET-3`) in `todos/todo.md`.

### P0 SIM2 Round 4 Stabilization: Monitoring Truthfulness + UX Consistency

#### SIM2-R4-2: Decouple Monitoring Render Pipeline from Adversary-Sim Toggle State

- [x] SIM2-R4-2-1 Remove any runtime/dashboard gating that suppresses monitoring fetch/render unless adversary sim is enabled.
- [x] SIM2-R4-2-2 Preserve historical telemetry visibility while appending newly ingested telemetry points without wiping history.
- [x] SIM2-R4-2-3 Validate cursor/SSE/polling interplay so real-time updates continue without requiring toggle transitions.

#### SIM2-R4-1: Restore Monitoring Initial Load and Refresh Control Correctness

- [x] SIM2-R4-1-1 Fix monitoring page bootstrap so charts/recent events initialize populated from the latest available snapshot on first load (without requiring adversary sim toggle-on).
- [x] SIM2-R4-1-2 Fix auto-refresh toggle semantics so enabling/disabling refresh actually starts/stops polling and updates view state deterministically.
- [x] SIM2-R4-1-3 Fix manual refresh semantics so button clicks trigger immediate reload when auto-refresh is off and do not no-op.
- [x] SIM2-R4-1-4 Ensure loading/empty/error states are explicit and recoverable (no stuck disabled/unpopulated state after transient failures).

#### SIM2-R4-3: Prove Adversary-Simulation Traffic Is Real, Generated, and Observable End-to-End

- [x] SIM2-R4-3-1 Verify adversary-sim execution path emits real HTTP/browser requests through the same request pipeline used for organic traffic.
- [x] SIM2-R4-3-2 Ensure emitted telemetry from adversary-sim traffic reaches monitoring ingest, chart aggregation, and recent-events feeds.
- [x] SIM2-R4-3-3 Add diagnostics path for “sim enabled but no traffic generated” so operators receive explicit cause/reason instead of silent success.

Acceptance criteria (archived):
1. Enabling adversary sim produces measurable request/event deltas visible in monitoring within one refresh interval/SSE cycle.
2. Recent events and chart series show adversary-sim-attributed activity alongside non-sim traffic without synthetic-only artifacts.
3. End-to-end verification (`make test` path + focused SIM2 monitoring checks) fails if sim run does not produce observable telemetry.

#### SIM2-R4-5: Enforce Monitoring-Page UI Control Style Parity with Canonical Dashboard Design System

- [x] SIM2-R4-5-1 Replace monitoring recent-events field/select controls that diverge from shared styling with canonical reusable controls/classes.
- [x] SIM2-R4-5-2 Remove duplicated/ad-hoc local CSS rules for those controls; reuse existing design tokens/patterns from shared dashboard style surfaces.
- [x] SIM2-R4-5-3 Add dashboard regression coverage (unit/visual/e2e as appropriate) that detects style/structure drift for monitoring form controls.

### P0 SIM2 Verification Hardening Wave 3 (Plan-Closure Priority)

#### SIM2-W3-1: Make Container-Blackbox Evidence Mandatory in Blocking Gates

- [x] SIM2-W3-1 remove permissive matrix behavior (`--allow-missing-container-report`) from blocking SIM2 gate paths and ensure strict failure when container evidence is absent.
- [x] Enforced strict SIM2 matrix invocation in `Makefile` (`test-sim2-verification-matrix`) and wired advisory mode only for manifest-only local checks (`test-sim2-verification-matrix-advisory`).
- [x] Added blocking container-lane execution in umbrella/release paths (`make test` and `.github/workflows/release-gate.yml`) and artifact upload wiring (`.github/workflows/ci.yml`, `.github/workflows/release-gate.yml`).
- [x] Added SIM2 verification-matrix unit coverage for strict missing-container failure and positive container-evidence pass paths (`scripts/tests/test_sim2_verification_matrix.py`).

Acceptance criteria (archived):
1. `make test-sim2-verification-matrix` fails when `scripts/tests/adversarial/container_blackbox_report.json` is missing.
2. Blocking CI/release workflows execute `make test-adversarial-container-blackbox` before matrix validation.
3. Verification-matrix report no longer emits pass/skipped status for missing container lane in blocking paths.
4. Unit tests cover missing-container failure behavior and strict lane evidence expectations.

#### SIM2-W3-2: Align Realtime Bench Contract to ADR `0008` Envelope and Profile Semantics

- [x] SIM2-W3-2 enforce declared benchmark envelope for blocking baseline profile and make profile metadata truthful for runtime-dev/runtime-prod verification semantics.
- [x] Raised baseline workload in `scripts/tests/sim2_realtime_bench.py` to `events_per_sec=1000` and `operator_clients=5` for blocking benchmark profile.
- [x] Added explicit verification scope metadata (`harness_type`, runtime-profile claims, and `claims_runtime_prod_verification=false`) to report and summary artifacts.
- [x] Extended realtime-benchmark unit tests to lock the workload envelope and runtime-profile claim semantics (`scripts/tests/test_sim2_realtime_bench.py`).

Acceptance criteria (archived):
1. Blocking baseline benchmark workload is `>=1000 events/sec` and `>=5 active operator clients`.
2. Benchmark artifacts include explicit profile metadata and do not claim runtime-prod verification when only synthetic/single-lane simulation is executed.
3. Threshold failures include percentile/budget diagnostics naming violated metric and required bound.
4. `make test-sim2-realtime-bench` remains deterministic and test-covered.

#### SIM2-W3-3: Remove Synthetic-Pass Fallbacks from Operational Regression Gate

- [x] SIM2-W3-3 remove generated default sections that can produce pass results when required operational domains are missing from report artifacts.
- [x] Reworked `scripts/tests/check_sim2_operational_regressions.py` to require explicit domain sections and fail deterministically with `domain_missing:<domain>` taxonomy.
- [x] Removed synthetic fallback section generation from operational regression checks so missing evidence cannot silently pass.
- [x] Updated unit tests to assert missing-domain failure behavior and taxonomy (`scripts/tests/test_sim2_operational_regressions.py`).

Acceptance criteria (archived):
1. Missing `failure_injection`, `prod_mode_monitoring`, `retention_lifecycle`, `cost_governance`, or `security_privacy` sections causes deterministic gate failure.
2. Failure taxonomy distinguishes `domain_missing` from `threshold_regression`.
3. Unit tests assert missing-domain failures and reject fallback-pass behavior.
4. No hardcoded positive defaults remain that can mask absent evidence.

#### SIM2-W3-5: Eliminate Pass-Oriented Synthetic Defaults in Retention/Cost/Security Diagnostics

- [x] SIM2-W3-5 ensure retention/cost/security diagnostics derive from measured report values or fail/degraded states; no optimistic synthetic values are permitted.
- [x] Added required-metric validation for retention/cost/security domains in `scripts/tests/check_sim2_operational_regressions.py` with deterministic `domain_missing_metric:<domain>` taxonomy.
- [x] Added missing-metric unit coverage for retention/cost/security checks (`scripts/tests/test_sim2_operational_regressions.py`).

Acceptance criteria (archived):
1. Retention diagnostics require real metric fields (`purge_lag_hours`, `pending_expired_buckets`, scan counters) and fail when absent.
2. Cost diagnostics require real cardinality/payload/compression/query-budget fields and fail when absent.
3. Security diagnostics require real classification/canary/pseudonymization/retention/incident-hook fields and fail when absent.
4. CI artifacts remain actionable and include exact missing metric names for each failed domain.

#### SIM2-W3-4: Strengthen ADR/Governance Conformance from Marker Checks to Evidence Checks

- [x] SIM2-W3-4 replace marker-presence-only conformance checks with assertions tied to measurable contract evidence and implementation behavior.
- [x] Upgraded ADR conformance checker to validate evidence artifacts from `latest_report.json` and `sim2_realtime_bench_report.json` (envelope + runtime-claim scope + retention/cost/security required fields) in addition to marker checks (`scripts/tests/check_sim2_adr_conformance.py`).
- [x] Upgraded governance checker to validate promotion-threshold alignment from parsed promotion constants, plus deterministic gate artifact status (`sim2_operational_regressions_report.json`, `sim2_realtime_bench_report.json`, `sim2_verification_matrix_report.json`) (`scripts/tests/check_sim2_governance_contract.py`).
- [x] Updated ADR/governance unit coverage for evidence-driven positive/negative paths (`scripts/tests/test_sim2_adr_conformance.py`, `scripts/tests/test_sim2_governance_contract.py`).

Acceptance criteria (archived):
1. ADR conformance checks validate artifact/report evidence for `0007`/`0008`/`0009` domains, not only string markers in source files.
2. Governance contract checks validate quantitative promotion thresholds and deterministic-blocking semantics from contract/report fields.
3. Missing evidence or threshold drift fails deterministically with explicit domain-specific diagnostics.
4. Updated unit tests cover positive and negative evidence-driven conformance paths.

## Additional completions (2026-02-28)

### P0 SIM2 Excellence Remediation Wave 2 (Architecture + Adversary Evolution)

#### SIM2-EX1: Complete Functional-Core Migration and Decompose Imperative Hot Paths

- [x] SIM2-EX1-1 Produce an architecture inventory of all remaining direct side-effect callsites in request handling (`metrics`, `monitoring`, `event log`, `ban writes`) and classify each as `retain`, `migrate`, or `delete`. (Artifact: `docs/plans/2026-02-28-sim2-ex1-1-request-side-effect-inventory.md`)
- [x] SIM2-EX1-2 Move all remaining request-path side effects still executed directly from `src/lib.rs` into effect-intent execution paths behind typed intents. (Artifacts: `src/lib.rs`, `src/runtime/effect_intents.rs`, `src/observability/metrics.rs`)
- [x] SIM2-EX1-3 Split `src/runtime/effect_intents.rs` into responsibility-focused modules (`intent_types`, `plan_builder`, `intent_executor`, `response_renderer`) with explicit dependency direction. (Artifacts: `src/runtime/effect_intents.rs`, `src/runtime/effect_intents/intent_types.rs`, `src/runtime/effect_intents/plan_builder.rs`, `src/runtime/effect_intents/intent_executor.rs`, `src/runtime/effect_intents/response_renderer.rs`)
- [x] SIM2-EX1-4 Remove or fully migrate legacy `#[allow(dead_code)]` policy handlers in `src/runtime/policy_pipeline.rs`; keep no dead-code rollback seam in active request path. (Artifact: `src/runtime/policy_pipeline.rs`)
- [x] SIM2-EX1-5 Introduce architectural guard tests/lints that fail if pure decision modules depend on `Store`, provider side effects, event logging, or mutable global state. (Artifacts: `src/runtime/architecture_guards.rs`, `src/runtime/mod.rs`)
- [x] SIM2-EX1-6 Add characterization parity tests around migrated seams and require parity snapshots before and after each extraction slice. (Artifacts: `src/runtime/effect_intents/plan_builder.rs`, `src/runtime/effect_intents/plan_builder_characterization_snapshot.txt`)
- [x] SIM2-EX1-7 Reduce `src/lib.rs` orchestration surface to route setup, trust-boundary setup, and tranche wiring only; move policy behavior decisions out of entrypoint logic. (Artifacts: `src/lib.rs`, `src/runtime/mod.rs`, `src/runtime/request_flow.rs`)
- [x] SIM2-EX1-8 Document final orchestration ownership map in `docs/module-boundaries.md` and update ADR references where boundaries changed. (Artifacts: `docs/module-boundaries.md`, `docs/adr/0006-functional-core-policy-orchestration.md`)


Acceptance criteria (archived):
1. No request-path privileged side effects are emitted directly from `src/lib.rs`; all flow through intent executor boundaries.
2. `src/runtime/policy_pipeline.rs` contains only active graph orchestration paths; legacy dead-code handlers are removed or isolated outside runtime path.
3. Pure decision modules compile and test without KV/provider dependencies.
4. Characterization parity suite shows no unintended behavior drift across extracted slices.
5. `src/lib.rs` becomes a thin orchestration shell with materially reduced complexity and clearly documented responsibilities.
6. `make test-unit`, `make test-integration`, `make test-dashboard-e2e`, `make test`, and `make build` pass after migration slices.
7. Updated docs make dependency direction and side-effect boundaries unambiguous to next contributors.
#### SIM2-EX2: Enforce Least-Authority Capability-by-Construction Across Privileged Effects

- [x] SIM2-EX2-1 Define capability lattice by operation class (`metrics_write`, `monitoring_write`, `event_log_write`, `ban_write`, optional `response_privileged`) and by orchestration phase. (Artifact: `src/runtime/capabilities.rs`)
- [x] SIM2-EX2-2 Replace single coarse `RuntimeCapabilities::for_request_path()` minting with phase-specific capability construction and explicit capability passing per execution step. (Artifacts: `src/runtime/capabilities.rs`, `src/runtime/request_flow.rs`, `src/runtime/policy_pipeline.rs`, `src/runtime/effect_intents/intent_executor.rs`, `src/runtime/effect_intents/response_renderer.rs`, `src/lib.rs`)
- [x] SIM2-EX2-3 Eliminate direct privileged helper calls that bypass capability checks; route every write path through capability-gated executor APIs. (Artifacts: `src/runtime/request_router.rs`, `src/runtime/shadow_mode/mod.rs`, `src/runtime/request_flow.rs`, `src/runtime/policy_pipeline.rs`, `src/runtime/effect_intents/intent_types.rs`, `src/runtime/effect_intents/intent_executor.rs`, `src/runtime/effect_intents/plan_builder.rs`)
- [x] SIM2-EX2-4 Add compile-time sealing for capability constructors so capabilities can only be minted at trust-boundary entrypoints. (Artifacts: `src/runtime/capabilities.rs`, `src/runtime/request_flow.rs`, `src/lib.rs`, `src/runtime/request_router.rs`, `src/runtime/request_router/tests.rs`)
- [x] SIM2-EX2-5 Add negative-path tests proving privileged effects fail/are impossible when capability is absent. (Artifact: `src/runtime/architecture_guards.rs`)
- [x] SIM2-EX2-6 Add regression tests ensuring no fallback path silently executes privileged writes outside capability-guarded APIs. (Artifact: `src/runtime/architecture_guards.rs`)
- [x] SIM2-EX2-7 Add architecture assertions (search-based CI guard or compile checks) preventing direct calls to privileged write APIs from disallowed modules. (Artifact: `src/runtime/architecture_guards.rs`)
- [x] SIM2-EX2-8 Update architecture docs and ADR notes with final capability model and enforcement guarantees. (Artifacts: `docs/module-boundaries.md`, `docs/adr/0006-functional-core-policy-orchestration.md`)


Acceptance criteria (archived):
1. Privileged side effects are capability-gated everywhere in request path, without convention-only exceptions.
2. Capability minting occurs only at explicit trust boundaries.
3. Least-authority capability scope is demonstrated by tests for each effect class.
4. Missing-capability scenarios fail deterministically and observably.
5. No privileged write API is reachable from pure decision modules.
6. CI guardrails fail fast on capability-bypass regressions.
#### SIM2-EX3: Increase Black-Box Realism by Removing Per-Scenario Control-Plane Preconditioning

- [x] SIM2-EX3-1 Define runner execution contract separating `suite_setup`, `attacker_execution`, and `suite_teardown`; forbid control-plane config writes during `attacker_execution`. (Artifacts: `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/test_adversarial_simulation_runner.py`)
- [x] SIM2-EX3-2 Replace per-scenario `admin_patch` choreography with baseline profile presets loaded before attacker execution starts. (Artifact: `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX3-3 Add explicit runner guardrail that fails the run if control-plane mutation occurs after attacker phase begins (except approved teardown/reset hooks). (Artifacts: `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/test_adversarial_simulation_runner.py`)
- [x] SIM2-EX3-4 Rework scenarios so expected defenses are triggered by attacker behavior and traffic progression, not repeated runtime reconfiguration. (Artifact: `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX3-5 Extend report schema with control-plane mutation audit trail (`count`, `phase`, `reason`) and fail criteria when mutation policy is violated. (Artifact: `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX3-6 Add deterministic tests for mutation-contract compliance in smoke/coverage profiles. (Artifacts: `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/test_adversarial_simulation_runner.py`)
- [x] SIM2-EX3-7 Update operator docs to distinguish deterministic reproducibility controls from attacker realism constraints. (Artifact: `scripts/tests/adversarial/README.md`)


Acceptance criteria (archived):
1. During attacker phase, control-plane config mutation count is zero by policy and verified by tests.
2. Coverage profile still passes without per-scenario config patching.
3. Gate failures clearly identify realism-contract violations vs defense regressions.
4. Deterministic reproducibility remains stable across repeated runs with fixed seeds.
5. Black-box realism improves without granting attacker plane privileged controls.
#### SIM2-EX4: Deliver True Browser-Executed “Browser Realistic” Drivers

- [x] SIM2-EX4-1 Define browser-driver architecture (`playwright`/equivalent) with deterministic seed control, bounded runtime, and resource budgets. (Artifacts: `scripts/tests/adversarial_browser_driver.mjs`, `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/adversarial/README.md`)
- [x] SIM2-EX4-2 Implement real browser execution path for `browser_realistic` class (navigation, DOM, JS execution, storage/cookie behavior, challenge interaction hooks). (Artifacts: `scripts/tests/adversarial_browser_driver.mjs`, `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX4-3 Keep non-browser drivers for scraper/load cohorts; enforce driver-class-specific capability boundaries and telemetry labels. (Artifacts: `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/adversarial/scenario_manifest.v2.json`)
- [x] SIM2-EX4-4 Add browser-lane observability fields (`js_executed`, `dom_events`, `storage_mode`, `challenge_dom_path`) to report evidence. (Artifacts: `scripts/tests/adversarial_browser_driver.mjs`, `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX4-5 Add deterministic replay harness for browser scenarios including strict timeout, retry policy, and anti-flake constraints. (Artifacts: `scripts/tests/adversarial_browser_driver.mjs`, `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX4-6 Add CI-safe fallback semantics only for unsupported environments, with explicit lane status reporting and no silent pass-through. (Artifacts: `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/test_adversarial_simulation_runner.py`)
- [x] SIM2-EX4-7 Expand E2E/adversarial tests to validate that browser-only defenses are exercised by real browser lanes. (Artifacts: `scripts/tests/test_adversarial_simulation_runner.py`, `scripts/tests/test_sim2_verification_matrix.py`, `scripts/tests/adversarial/scenario_intent_matrix.v1.json`)


Acceptance criteria (archived):
1. `browser_realistic` scenarios are executed by a real browser runtime, not raw HTTP request emulation.
2. Browser-only defense surfaces (JS verification/CDP/client-runtime checks) are exercised with explicit evidence in reports.
3. Browser lane remains deterministic enough for CI gating within bounded flake tolerance and declared retry policy.
4. Fallback behavior is explicit and cannot silently mask missing browser execution.
5. Required Makefile gates remain bounded and pass on supported CI lanes.
#### SIM2-EX5: Upgrade Frontier Discovery from Advisory Probe to Adaptive Attack Generation Program

- [x] SIM2-EX5-1 Define attack-generation contract for frontier lane (`objective`, `constraints`, `allowed actions`, `forbidden data`, `resource budgets`, `novelty expectations`). (Artifact: `scripts/tests/adversarial/frontier_attack_generation_contract.v1.json`)
- [x] SIM2-EX5-2 Implement candidate generation pipeline that proposes new attack variants/mutations instead of only rewrapping existing deterministic scenarios. (Artifact: `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX5-3 Add diversity scoring (`cross-provider agreement`, `novelty`, `behavioral class coverage`) with deterministic normalization for triage. (Artifact: `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX5-4 Add automatic sanitization and governance checks for generated payloads before any replay/promotion path. (Artifacts: `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/check_frontier_payload_artifacts.py`)
- [x] SIM2-EX5-5 Upgrade promotion pipeline to ingest generated candidates, replay them deterministically, and produce lineage from `generated candidate -> deterministic confirmation -> promoted scenario`. (Artifacts: `scripts/tests/adversarial_promote_candidates.py`, `scripts/tests/test_adversarial_promote_candidates.py`)
- [x] SIM2-EX5-6 Add protected-lane metrics for discovery quality (`candidate count`, `novel confirmed regressions`, `false discovery rate`, `provider outage impact`). (Artifacts: `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/adversarial_promote_candidates.py`)
- [x] SIM2-EX5-7 Keep blocking policy deterministic: no stochastic frontier output can block release without deterministic confirmation. (Artifacts: `scripts/tests/adversarial_promote_candidates.py`, `scripts/tests/adversarial/frontier_attack_generation_contract.v1.json`)
- [x] SIM2-EX5-8 Publish operator workflow for evaluating and curating generated candidates into canonical manifests. (Artifacts: `docs/adversarial-operator-guide.md`, `scripts/tests/adversarial/README.md`)


Acceptance criteria (archived):
1. Frontier lane produces novel candidate attacks beyond existing deterministic scenario catalog.
2. All promoted regressions show deterministic confirmation lineage.
3. Governance/redaction checks remain enforced and audited before replay.
4. Release-blocking semantics remain deterministic and policy-stable.
5. Operators can track discovery efficacy with explicit quality metrics, not only provider-health status.
#### SIM2-EX6: Deepen Coverage Contract Governance to Enforce Full Plan Intent

- [x] SIM2-EX6-1 Define `coverage_contract.v2` with explicit minima for currently under-specified plan intents (including tarpit progression and event-stream health depth metrics). (Artifacts: `scripts/tests/adversarial/coverage_contract.v2.json`, `scripts/tests/adversarial/scenario_manifest.v1.json`, `scripts/tests/adversarial/scenario_manifest.v2.json`)
- [x] SIM2-EX6-2 Add schema migration and compatibility handling for contract v1/v2 while pre-launch migration completes. (Artifacts: `scripts/tests/adversarial_simulation_runner.py`, `scripts/tests/check_adversarial_coverage_contract.py`)
- [x] SIM2-EX6-3 Add strict drift checks among plan rows, manifest expectations, runner extracted metrics, and contract requirements. (Artifacts: `scripts/tests/check_adversarial_coverage_contract.py`, `scripts/tests/adversarial/verification_matrix.v1.json`, `scripts/tests/adversarial/scenario_manifest.v1.json`, `scripts/tests/adversarial/scenario_manifest.v2.json`)
- [x] SIM2-EX6-4 Extend gate diagnostics with row-level failure output showing `required`, `observed`, `missing evidence`, and scenario contribution mapping. (Artifact: `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX6-5 Add focused tests for each new v2 coverage key and threshold boundary behavior. (Artifacts: `scripts/tests/test_adversarial_simulation_runner.py`, `scripts/tests/test_adversarial_coverage_contract.py`)
- [x] SIM2-EX6-6 Wire v2 governance into mandatory Makefile and CI coverage gates with fail-fast messaging. (Artifacts: `scripts/tests/check_adversarial_coverage_contract.py`, `scripts/tests/adversarial_simulation_runner.py`)
- [x] SIM2-EX6-7 Update docs/runbooks with contract evolution protocol and backwards-compatibility removal date. (Artifacts: `docs/adversarial-operator-guide.md`, `scripts/tests/adversarial/README.md`)


Acceptance criteria (archived):
1. Canonical coverage contract enforces every required plan-row intent with explicit measurable thresholds.
2. Tarpit progression and event-stream health rows cannot pass with shallow/partial evidence.
3. Drift across plan/manifest/runner/contract fails deterministically with actionable output.
4. Coverage contract versioning and migration are documented and test-backed.
5. Mandatory coverage gates continue to run via canonical Makefile paths.
#### SIM2-EX8: Establish Continuous Defender-Adversary Evolution Loop as First-Class Program

- [x] SIM2-EX8-1 Define canonical cycle contract: `run adversary -> analyze failures -> tune defenses -> replay -> promote scenarios -> repeat`. (Artifacts: `docs/adversarial-operator-guide.md`, `scripts/tests/adversarial/hybrid_lane_contract.v1.json`)
- [x] SIM2-EX8-2 Add report diff tooling that highlights defense deltas between runs (new passes, new regressions, cost shifts, collateral changes). (Artifacts: `scripts/tests/adversarial_report_diff.py`, `scripts/tests/test_adversarial_report_diff.py`, `Makefile`)
- [x] SIM2-EX8-3 Add backlog automation guidance for converting confirmed novel regressions into prioritized implementation todos with ownership and SLA. (Artifacts: `docs/adversarial-operator-guide.md`, `scripts/tests/adversarial_report_diff.py`)
- [x] SIM2-EX8-4 Add promotion hygiene rules so stale scenarios are retired, merged, or reclassified with explicit rationale. (Artifact: `docs/adversarial-operator-guide.md`)
- [x] SIM2-EX8-5 Define excellence KPIs for the loop (`time to regression confirmation`, `time to mitigation`, `collateral ceiling`, `cost asymmetry trend`) and expose them in operator docs. (Artifacts: `docs/adversarial-operator-guide.md`, `scripts/tests/adversarial/hybrid_lane_contract.v1.json`, `scripts/tests/check_sim2_governance_contract.py`)
- [x] SIM2-EX8-6 Add governance checkpoint requiring periodic architecture review against this cycle contract and documented outcomes. (Artifacts: `scripts/tests/adversarial/hybrid_lane_contract.v1.json`, `scripts/tests/check_sim2_governance_contract.py`)


Acceptance criteria (archived):
1. Shuma has a documented and testable closed-loop process for adversary-driven defense evolution.
2. Novel regressions move from discovery to deterministic confirmation to TODO execution without manual ambiguity.
3. Scenario corpus quality is maintained through promotion and retirement rules.
4. Excellence KPIs are measurable, reported, and used for release readiness decisions.
5. The loop preserves core project principles: low human friction, rising attacker cost, bounded defender resource cost.
### P0 SIM2 Post-Implementation Shortfall Remediation (Execution Priority)

#### SIM2-EX7: Harden Simulation-Telemetry Secret Ergonomics Without Weakening Security

- [x] SIM2-EX7-1 Add `make setup` and `make verify` checks that guarantee `SHUMA_SIM_TELEMETRY_SECRET` is created, non-placeholder, and surfaced clearly to operators.
- [x] SIM2-EX7-2 Add explicit adversarial preflight command/target that validates all required secrets and prints actionable remediation before runner execution.
- [x] SIM2-EX7-3 Add CI workflow explicit env wiring for `SHUMA_SIM_TELEMETRY_SECRET` in lanes that run adversarial coverage/promote jobs.
- [x] SIM2-EX7-4 Improve runner failure diagnostics with structured, copy-paste-safe setup guidance and clear distinction between missing secret vs invalid signature vs replay failure.
- [x] SIM2-EX7-5 Add docs for local rotation and CI secret lifecycle, including cadence and compromise-response workflow.
- [x] SIM2-EX7-6 Add automated tests for setup/preflight behavior ensuring missing/placeholder secret states fail early with deterministic guidance.
- [x] SIM2-EX7-7 Confirm security posture remains fail-closed: no unsigned sim metadata acceptance path is introduced.


Acceptance criteria (archived):
1. Local `make setup` leaves adversarial runs ready by default with valid sim telemetry secret material.
2. CI adversarial lanes explicitly provision required secret env and do not rely on implicit setup state.
3. Missing/invalid secret states fail before scenario execution with clear remediation output.
4. No change weakens sim-tag authenticity enforcement or introduces permissive bypass.
5. Operator docs clearly define setup, rotation, and incident-response steps.
### P0 SIM2 Gap-Closure Program: Real Execution + Realtime Monitoring

#### SIM2-GC-17: Telemetry and Adversary-Artifact Security/Privacy-by-Construction

- [x] SIM2-GC-17-1 Define canonical field-classification schema for telemetry/artifact fields (`public`, `internal`, `sensitive`, `secret-prohibited`) and persistence policy matrix.
- [x] SIM2-GC-17-2 Enforce classification at ingest/persist boundaries; reject prohibited classes and emit structured violation events.
- [x] SIM2-GC-17-3 Add deterministic secret scrubber for high-risk fields (`reason`, `outcome`, artifact payload fragments) with explicit redaction markers.
- [x] SIM2-GC-17-4 Add secret-canary detection for frontier/adversary artifacts and fail-closed persistence behavior on canary match.
- [x] SIM2-GC-17-5 Expand pseudonymization coverage to non-forensic monitoring/event views for sensitive identifiers, with explicit audited forensic break-glass mode.
- [x] SIM2-GC-17-6 Define and enforce sensitivity-tiered artifact retention windows (high-risk raw artifacts `<=72h` default; redacted summaries longer-lived).
- [x] SIM2-GC-17-7 Add incident-response hooks for leak/policy violations (`detect`, `contain`, `quarantine`, `operator action required`) with operation/run correlation IDs.
- [x] SIM2-GC-17-8 Update docs/runbooks for privacy posture, incident triage, forensic access controls, and retention override governance.


Acceptance criteria (archived):
1. Secret-prohibited data classes are blocked from persistence by construction and verified by tests.
2. Secret canary leakage to persisted telemetry/artifacts is zero in mandatory regression lanes.
3. Pseudonymization is default-on for sensitive identifiers in non-forensic views, with audited break-glass workflow for raw access.
4. High-risk raw artifact retention defaults to `<=72h` and overrides require explicit audit entries.
5. Incident hooks emit deterministic, actionable events for containment workflow without delaying core defense execution.
6. Security/privacy posture is operator-visible and CI-enforced with explicit threshold diagnostics.
#### SIM2-GC-16: Monitoring Cost Governance and Resource Efficiency Envelope

- [x] SIM2-GC-16-1 Define formal monitoring cost envelope (`ingest events/sec`, `query calls/sec`, `payload bytes`, `cardinality budget`, `compression ratio`) for dev/prod verification profiles.
- [x] SIM2-GC-16-2 Enforce guarded-dimension cardinality caps (`<=1000` distinct values/hour per guarded dimension) with deterministic `other` overflow bucket behavior.
- [x] SIM2-GC-16-3 Implement rollup windows (`1m`, `5m`, `1h`) for dashboard-default queries and preserve raw-event drill-down lineage.
- [x] SIM2-GC-16-4 Define unsampleable security-event class list and enforce `0` sampling/drop for those classes.
- [x] SIM2-GC-16-5 Add deterministic low-risk telemetry sampling policy for eligible high-volume classes with explicit sampled/unsampled counters.
- [x] SIM2-GC-16-6 Add payload budget controls (`p95 <= 512KB` default monitoring response) via pagination/cursor windowing and response shaping.
- [x] SIM2-GC-16-7 Add compression negotiation/reporting for monitoring payloads and enforce `>=30%` transfer reduction target for payloads `>64KB`.
- [x] SIM2-GC-16-8 Extend admin query budgets to cost-class aware controls and degraded-state signaling when budgets are exceeded.
- [x] SIM2-GC-16-9 Add operator-facing cost health telemetry (`cardinality_pressure`, `payload_budget_status`, `sampling_status`, `query_budget_status`) and runbook guidance.


Acceptance criteria (archived):
1. Monitoring pipeline remains within declared cost envelope under realtime benchmark scenarios.
2. Guarded dimensions respect cardinality caps with explicit overflow accounting and no unbounded growth.
3. Unsampleable defense-event classes are never sampled or dropped.
4. Default monitoring payloads meet size budget and expose pagination/window continuation when capped.
5. Compression and query-budget controls provide measurable transport/query cost savings without freshness regressions.
6. Cost health status is operator-visible and CI-enforced with threshold diagnostics.
#### SIM2-GC-15: Telemetry Retention Lifecycle Determinism and Health Visibility

- [x] SIM2-GC-15-1 Define canonical telemetry bucket/index schema for monitoring/event retention operations (`bucket_id`, `window_start`, `window_end`, `record_count`, `state`).
- [x] SIM2-GC-15-2 Migrate telemetry writes to update bucket/index metadata so expired windows are purge-addressable without full keyspace scans.
- [x] SIM2-GC-15-3 Implement background purge worker cadence with bounded batch budget and persisted purge watermark (`last_purged_bucket`, `last_attempt_ts`, `last_success_ts`).
- [x] SIM2-GC-15-4 Remove opportunistic retention cleanup from monitoring/admin read paths and replace with worker-triggered retention lifecycle.
- [x] SIM2-GC-15-5 Add retention health surface in admin/monitoring payloads (`retention_hours`, `oldest_retained_ts`, `purge_lag_hours`, `pending_expired_buckets`, `last_error`).
- [x] SIM2-GC-15-6 Add degraded-state signaling and operator guidance when retention drift exceeds thresholds.
- [x] SIM2-GC-15-7 Add deterministic failure-recovery behavior for purge partial failures (retry safety, idempotent bucket cleanup, explicit failure taxonomy).
- [x] SIM2-GC-15-8 Add docs/runbook updates for retention tuning, purge troubleshooting, and operational rollback.


Acceptance criteria (archived):
1. Retention enforcement no longer relies on monitoring refresh read paths performing keyspace-wide cleanup work.
2. Purge lag remains `<=1 hour` beyond configured retention window under declared normal envelope.
3. Healthy state reports `pending_expired_buckets == 0`; non-zero state is operator-visible with degraded status.
4. Bucket cutoff semantics are deterministic and test-backed across repeated purge cycles.
5. Purge worker remains bounded (`<=500ms` budget per cadence tick) and failure-retry behavior is idempotent.
6. Retention health telemetry is visible in dashboard/admin surfaces and included in CI diagnostics artifacts.
#### SIM2-GC-10: Dashboard UX for Arms-Race Operations (Evidence-First)

- [x] SIM2-GC-10-1 Add “recent adversary run” panel linking run ids to observed defense deltas in monitoring and IP-ban surfaces.
- [x] SIM2-GC-10-2 Add per-defense trend blocks (trigger count, pass/fail ratio, escalations, ban outcomes) keyed by source labels.
- [x] SIM2-GC-10-3 Add fast filters for `origin`, `scenario`, `lane`, `defense`, and `outcome` without introducing new visual language.
- [x] SIM2-GC-10-4 Add explicit empty/error/degraded states so missing data is never mistaken for “no attacks.”
- [x] SIM2-GC-10-5 Add operator workflow docs for triage, replay, tuning, and validation loops from the dashboard.


Acceptance criteria (archived):
1. Operators can directly correlate adversary runs with defense responses from the UI.
2. Missing/late telemetry is explicit and actionable.
3. Filtering and trends support fast tuning decisions without data ambiguity.
4. UI remains consistent with existing dashboard design system.
5. Operators can distinguish “no attacks observed” from “data is stale/degraded” without ambiguity.
#### SIM2-GC-11: Verification Suite Expansion for End-to-End Truthfulness (partial)

- [x] SIM2-GC-11-1 Define and publish mandatory verification matrix mapping each defense category to required scenarios, lanes, and evidence assertions.
- [x] SIM2-GC-11-2 Add e2e test suite that executes matrix-required crawler/scraper/browser/frontier scenarios and asserts monitoring/IP-ban updates.
- [x] SIM2-GC-11-3 Add contract tests for telemetry lineage integrity and monotonic event ordering across refresh cycles.
- [x] SIM2-GC-11-4 Add failure-injection tests (telemetry store delay, partial write failure, refresh race) with expected operator-visible outcomes.
- [x] SIM2-GC-11-8 Require failure diagnostics to name missing matrix row(s), missing evidence type(s), and failing telemetry lineage segment.
- [x] SIM2-GC-11-7 Add explicit prod-mode monitoring checks using non-sim traffic profiles to verify near-realtime visibility without adversary-sim toggle dependence.
- [x] SIM2-GC-11-9 Add control-plane race/idempotency tests for repeated UI toggle submissions, duplicate command replay, and multi-controller lease contention.
- [x] SIM2-GC-11-10 Add trust-boundary negative-path tests for adversary control endpoint (`csrf missing/invalid`, `origin mismatch`, `fetch-metadata cross-site`, `stale session`) and assert fail-closed behavior.
- [x] SIM2-GC-11-11 Add idempotency misuse tests proving key reuse with payload mismatch is rejected and exact retries map to stable `operation_id`.
- [x] SIM2-GC-11-12 Add throttling + audit tests proving rapid toggle storms are bounded and every accept/reject/throttle decision emits structured audit evidence.
- [x] SIM2-GC-11-13 Add container isolation regression tests for frontier lane (reject privileged mode, daemon-socket mount, disallowed host mount, and missing runtime hardening flags).
- [x] SIM2-GC-11-14 Add signed-envelope negative tests (`invalid signature`, `nonce replay`, `expiry exceeded`, `scope mismatch`) proving worker execution is blocked.
- [x] SIM2-GC-11-15 Add teardown determinism tests (`deadline exceeded`, `heartbeat loss`, forced-kill path) and assert terminal failure taxonomy plus cleanup completion.
- [x] SIM2-GC-11-5 Add Makefile targets for focused SIM2 realtime verification and wire them into `make test` gating policy.
- [x] SIM2-GC-11-6 Add CI diagnostics artifacts (timeline snapshots, event counts, refresh traces) for fast triage.
- [x] SIM2-GC-11-18 Add reproducible realtime benchmark verification target (`make test-sim2-realtime-bench`) and CI artifact outputs for latency percentiles, overflow/drop counts, and request-budget metrics.
- [x] SIM2-GC-11-19 Add retention lifecycle regression tests for bucket cutoff correctness, purge-watermark progression, purge-lag threshold, and no read-path full-keyspace cleanup scans.
- [x] SIM2-GC-11-20 Add cost-governance regression tests for cardinality caps, overflow-bucket accounting, unsampleable-event protection, payload-size budget, and compression effectiveness thresholds.
- [x] SIM2-GC-11-21 Add security/privacy regression suite for telemetry/artifacts (field-classification enforcement, secret-canary leak checks, pseudonymization default coverage, retention-tier policy, incident-hook emission).
- [x] SIM2-GC-11-22 Add ADR conformance verification checks ensuring implementation slices align with ADR `0007`, `0008`, and `0009` (or provide explicit supersession plan).
- [x] SIM2-GC-11-23 Add hybrid-governance threshold tests ensuring emergent promotion requires `>=95%` deterministic confirmation, `<=20%` false-discovery rate, and owner disposition SLA `<=48h`.


Acceptance criteria (archived):
1. Mandatory verification fails if any matrix-required defense/lane evidence is missing.
2. Tests cover happy path, degraded path, race path, and prod-mode non-sim freshness path.
3. CI artifacts make telemetry-visibility failures actionable at defense-row granularity.
4. `make test` remains canonical and sufficient for contributor verification.
5. Release gates cannot pass with partial coverage hidden by aggregate pass/fail summaries.
6. Toggle control regressions (duplicate starts, lease split-brain, missing operation lineage) fail deterministically in CI.
7. Trust-boundary regressions (CSRF/origin/session/replay/throttle/audit) fail deterministically in CI with explicit failure taxonomy.
8. Frontier isolation/envelope/teardown regressions fail deterministically in CI with explicit failure taxonomy.
9. Realtime cursor/stream ordering and resume regressions fail deterministically in CI with explicit failure taxonomy.
10. Benchmark-threshold regressions (`p95`, `p99`, overflow/drop, request-budget) fail deterministically in CI with scenario-specific diagnostics.
11. Retention-lifecycle regressions (purge lag, bucket cutoff drift, read-path scan fallback) fail deterministically in CI with explicit failure taxonomy.
12. Cost-governance regressions (cardinality/payload/sampling/compression/query-budget) fail deterministically in CI with explicit failure taxonomy.
13. Security/privacy regressions (classification/leak/pseudonymization/retention/incident hooks) fail deterministically in CI with explicit failure taxonomy.
14. ADR conformance drift for SIM2 core domains fails deterministically in CI unless an explicit supersession path is declared.
#### SIM2-GC-12: Program Governance for Continuous Defense Evolution

- [x] SIM2-GC-12-1 Define weekly/iteration cadence for `run -> review -> tune -> replay -> promote` cycle with ownership and SLA.
- [x] SIM2-GC-12-2 Define promotion rubric for new adversary techniques (severity, reproducibility, collateral risk, mitigation readiness).
- [x] SIM2-GC-12-3 Add KPI dashboard/reporting for attacker cost shift, human-friction impact, detection latency, and mitigation lead time.
- [x] SIM2-GC-12-4 Add explicit rollback playbooks for defensive changes that over-trigger on legitimate traffic.
- [x] SIM2-GC-12-5 Add periodic architecture review checkpoint to ensure orchestration remains decentralized, capability-safe, and evidence-driven.


Acceptance criteria (archived):
1. SIM2 is operated as an ongoing engineering system, not a one-off test feature.
2. New adversary discoveries have clear promotion and mitigation pathways.
3. KPIs demonstrate whether defenses are improving without unacceptable collateral.
4. Governance enforces architectural excellence and prevents drift back to imperative/convention-only controls.
#### SIM2-GC-14: Formalize Hybrid Adversary Model (Deterministic Oracle + Emergent Exploration)

- [x] SIM2-GC-14-1 Write architecture contract distinguishing `deterministic conformance lane` (blocking) and `emergent exploration lane` (non-blocking discovery).
- [x] SIM2-GC-14-2 Define what remains intentionally choreographed (seed scenarios, invariant assertions, resource guardrails) vs what must be emergent (crawl strategy, attack sequencing, adaptation).
- [x] SIM2-GC-14-3 Define emergent-lane objective model (target assets, success functions, allowed adaptation space, stop conditions) with bounded runtime budgets (`<=180s` and `<=500 actions` default envelope).
- [x] SIM2-GC-14-4 Define novelty scoring and triage policy (`novelty`, `severity`, `confidence`, `replayability`) for emergent findings.
- [x] SIM2-GC-14-5 Add lane metadata and report lineage fields so operators can see whether evidence came from deterministic or emergent execution.
- [x] SIM2-GC-14-6 Define promotion pipeline from emergent finding -> deterministic replay case -> blocking regression with explicit acceptance contract.
- [x] SIM2-GC-14-7 Add governance tests that fail if release-blocking decisions depend on stochastic-only emergent outcomes without deterministic confirmation.
- [x] SIM2-GC-14-8 Set and enforce quantitative promotion thresholds (`minimum deterministic confirmation rate >=95%`, `maximum tolerated false-discovery rate <=20%`, `owner-review disposition SLA <=48h`).
- [x] SIM2-GC-14-9 Update operator docs/runbooks so monitoring expectations reflect “real attacker behavior while enabled,” with deterministic replay used for release confidence.


Acceptance criteria (archived):
1. Deterministic and emergent lanes are explicit, testable, and operationally visible.
2. Blocking gates depend only on deterministic confirmation, never stochastic one-off outcomes.
3. Emergent lane drives realistic crawl/scrape/attack exploration without privileged control-plane access and within bounded budgets (`<=180s`, `<=500 actions` default envelope).
4. Promotion decisions enforce quantitative thresholds (`>=95%` deterministic confirmation, `<=20%` false-discovery rate, `<=48h` owner disposition SLA) and remain auditable from lineage artifacts.
5. False-discovery behavior is measured and kept within declared limit (`<=20%` rolling window target).
6. Operator documentation and UI terminology no longer conflate guardrail duration with procedural adversary progress.
#### SIM2-GC-9: Scenario Design Realism and Defense Exercise Guarantees

- [x] SIM2-GC-9-1 Add scenario intent matrix mapping each scenario to required defense signals and minimum evidence thresholds.
- [x] SIM2-GC-9-2 Remove scenario success criteria that can pass without exercising intended defenses.
- [x] SIM2-GC-9-3 Add progression logic for crawler/scraper/browser cohorts that models realistic retries, pacing, and evasion attempts.
- [x] SIM2-GC-9-4 Add contract tests that fail if scenarios labeled for a defense category do not generate corresponding events.
- [x] SIM2-GC-9-5 Add periodic coverage review process for stale, redundant, or non-realistic scenarios.


Acceptance criteria (archived):
1. Each scenario has explicit, test-backed defense exercise expectations.
2. Scenario passes without required defense evidence are impossible.
3. Coverage includes realistic multi-step adversary behavior, not single-request probes only.
4. Catalog quality is actively governed and measurable.
#### SIM2-GC-8: Containerized Frontier Integration as Real Actor (Not Metadata Generator) (partial)

- [x] SIM2-GC-8-1 Define frontier action contract (`allowed tools`, `network constraints`, `time/resource budgets`, `forbidden data access`).
- [x] SIM2-GC-8-2 Define reject-by-default action grammar/DSL and validation engine so only explicitly permitted action types are executable.
- [x] SIM2-GC-8-3 Implement container execution path that converts model output to validated actionable steps, then executes against target endpoints.
- [x] SIM2-GC-8-4 Enforce egress allowlist and capability boundaries at runtime with explicit deny/audit paths for policy violations.
- [x] SIM2-GC-8-5 Add strict sanitization/validation so unsafe or out-of-policy model outputs are rejected before execution.
- [x] SIM2-GC-8-6 Add negative-path security tests (secret-exfiltration canaries, out-of-scope URL attempts, privileged header injection attempts, replay envelope misuse).
- [x] SIM2-GC-8-7 Add trace lineage from model suggestion -> executed action -> runtime telemetry -> monitoring view.
- [x] SIM2-GC-8-8 Add degraded-mode behavior for key outages that remains explicit, does not fake execution success, and surfaces degraded state within one monitoring refresh/stream cycle.
- [x] SIM2-GC-8-9 Add operator kill-switch and deterministic emergency stop flow for active frontier runs with `p95 <= 10s` stop-latency target.
- [x] SIM2-GC-8-10 Enforce hardened container runtime profile for frontier workers (`non-root/rootless`, `no_new_privileges`, capability allowlist only, read-only rootfs with explicit scratch mounts, no privileged mode/host namespace joins).
- [x] SIM2-GC-8-11 Block sensitive host-control surfaces by policy (forbid daemon-socket mounts and disallowed host bind mounts; fail launch when isolation profile is violated).
- [x] SIM2-GC-8-12 Implement signed host-issued capability envelopes for executable worker actions (`run_id`, `step_id`, action scope, nonce, `issued_at`, `expires_at`, `key_id`) with strict signature/expiry/replay validation.
- [x] SIM2-GC-8-13 Implement bounded one-way command channel semantics (host -> worker command queue with backpressure; worker output restricted to append-only evidence/events without control-plane mutation rights).
- [x] SIM2-GC-8-14 Implement deterministic fail-closed teardown contract (hard runtime deadline, heartbeat timeout, forced process-tree kill, and terminal run-failed semantics on teardown failure).
- [x] SIM2-GC-8-15 Add lifecycle cleanup policy for frontier run artifacts/resources (TTL-driven cleanup, bounded retention, and explicit cleanup failure diagnostics).


Acceptance criteria (archived):
1. Frontier lane emits real traffic actions, not report-only suggestions.
2. Every executed frontier action has full lineage and monitoring evidence.
3. Out-of-policy model outputs are blocked deterministically by reject-by-default validation.
4. Egress allowlist and capability restrictions are enforced and test-covered.
5. Security abuse-path tests fail closed for exfiltration, privilege escalation, and envelope misuse attempts.
6. Key outage mode is visible, bounded, and non-deceptive.
7. Operators can force-stop frontier execution with deterministic teardown behavior.
8. Frontier workers cannot launch unless hardened runtime isolation profile is satisfied.
9. Signed capability envelopes are mandatory for execution and replay/signature/expiry failures are rejected deterministically.
10. Command/evidence channel semantics preserve one-way authority boundaries and bounded backpressure behavior.
11. Timeout/crash/heartbeat-loss paths fail closed with deterministic teardown and terminal diagnostics.
12. Executed frontier-action lineage completeness is `100%` (`model suggestion -> validated action -> runtime request -> monitoring evidence`).
13. Policy-violation execution rate is `0` in mandatory regression suites.
14. Kill-switch stop latency meets `p95 <= 10s` in benchmarked verification profiles.
15. Outage/degraded state transition is visible within one monitoring refresh/stream cycle.
#### SIM2-GC-7: Upgrade Browser-Adversary Lane to True Browser Execution

- [x] SIM2-GC-7-1 Replace HTTP-emulated browser lane with deterministic real-browser driver path.
- [x] SIM2-GC-7-2 Add challenge interaction primitives (DOM read/write, click/submit flows, storage/session behavior) with strict capability limits.
- [x] SIM2-GC-7-3 Ensure browser-only defenses (client runtime checks/CDP detections/challenge scripts) emit evidence when exercised.
- [x] SIM2-GC-7-4 Add anti-flake constraints, retries, and diagnostics that preserve CI reliability while proving real execution occurred.
- [x] SIM2-GC-7-5 Include per-run browser evidence fields in reports and monitoring correlation IDs.


Acceptance criteria (archived):
1. Browser lane traffic is generated by actual browser runtime, not request emulation.
2. Browser-only defenses register events under targeted scenarios.
3. Evidence ties browser actions to monitoring events and outcomes.
4. CI remains deterministic enough for gated verification.
#### SIM2-GC-10 + SIM2-GC-11: Realtime UX/Test Follow-through

- [x] SIM2-GC-10-6 Add explicit monitoring freshness indicators (`last event at`, `current lag`, `state: fresh/degraded/stale`) on monitoring and IP-ban tabs.
- [x] SIM2-GC-11-16 Add cursor-contract tests for monotonic ordering, resume-after-cursor correctness, overflow signaling, and deduped replay windows.
- [x] SIM2-GC-11-17 Add SSE-path tests for event-id ordering, `Last-Event-ID` reconnect behavior, and fallback-to-polling continuity when stream drops.

#### SIM2-GC-6: Deliver Realtime Monitoring Refresh Semantics and Backpressure Safety

- [x] SIM2-GC-6-1 Define quantitative freshness SLOs for runtime-dev and runtime-prod (`p50/p95/p99 visibility delay`, `manual refresh staleness bound`, `max allowed lag before degraded state`).
- [x] SIM2-GC-6-2 Define and enforce a load envelope for freshness SLO compliance (event ingest rate, operator refresh concurrency, query cost ceiling) with benchmark methodology.
- [x] SIM2-GC-6-3 Implement selected realtime delivery model (from `SIM2-GCR-4`) with deterministic ordering, cursor semantics, and bounded payload windows.
- [x] SIM2-GC-6-4 Add cache invalidation rules so high-signal events (new bans, challenge failures, maze escalations) invalidate stale views immediately without cache stampede behavior.
- [x] SIM2-GC-6-5 Add backend and UI rate-limit/backpressure controls to avoid self-induced load from aggressive refresh loops.
- [x] SIM2-GC-6-6 Add tests for freshness, monotonic ordering, deduplication, and behavior under bursty adversary runs.
- [x] SIM2-GC-6-7 Add explicit freshness-health telemetry and UI state (`fresh`, `degraded`, `stale`) with operator-facing lag indicators.
- [x] SIM2-GC-6-8 Replace run-active-only cache-bypass assumptions with a global freshness policy that preserves near-realtime visibility for real production attacker traffic.
- [x] SIM2-GC-6-12 Implement optional SSE delivery path (`text/event-stream`) that reuses the same cursor namespace and supports `Last-Event-ID` resume.
- [x] SIM2-GC-6-13 Add bounded server-side fan-out buffers/queues and explicit slow-consumer lag signaling (no unbounded memory growth).
- [x] SIM2-GC-6-14 Update dashboard refresh runtime to prefer cursor path (and SSE when available) with deterministic fallback to polling on stream failure.


Acceptance criteria (archived):
1. Under declared load envelope (`>=1000 events/s`, `>=5 active operator clients`), active live path achieves `p95 <= 300ms` and `p99 <= 500ms` freshness in runtime-dev and runtime-prod verification profiles.
2. Non-degraded active path has zero overflow/drop for monitored events within declared bounded buffer window.
3. Refresh behavior is deterministic (monotonic cursor progression, no silent loss, no duplicate replay beyond documented replay window rules).
4. Backpressure and cache behavior are bounded and benchmarked under expected burst load.
5. Freshness regressions fail automated tests with actionable diagnostics naming violated percentile/budget threshold.
6. Production monitoring freshness is independent of adversary-sim active state.
7. Operators can see explicit freshness health and lag state at all times.
8. Cursor-resume semantics are deterministic across manual refresh, auto-refresh, and reconnect flows.
9. When streaming path is available, active live updates stay within query budget `<=1 request/sec/client` average (excluding initial bootstrap requests); degraded fallback polling above that budget must surface explicit degraded state.
#### SIM2-GC-6: Deliver Realtime Monitoring Refresh Semantics and Backpressure Safety (partial)

- [x] SIM2-GC-6-9 Define canonical monitoring event cursor contract (strict monotonic sequence id, resume semantics, and overflow taxonomy shared by polling and stream paths).
- [x] SIM2-GC-6-10 Implement cursor-delta endpoint(s) for monitoring/IP-bans (`after_cursor`, bounded `limit`, `next_cursor`, `has_more`, `overflow`) with deterministic ordering.
- [x] SIM2-GC-6-11 Add conditional polling optimization (`If-None-Match`/`304`) on cursor-delta reads where unchanged windows can be proven safely.

#### SIM2-GC-5: Remove Simulation Telemetry Namespace Architecture Completely

- [x] SIM2-GC-5-1 Remove simulation namespace config flags, query paths, schema branches, and UI toggles.
- [x] SIM2-GC-5-2 Consolidate dev telemetry queries so SIM and manual dev traffic coexist in same dev plane with source labels.
- [x] SIM2-GC-5-3 Preserve source attribution fields (`origin=sim|manual|other`) for filtering without namespace-level partitioning.
- [x] SIM2-GC-5-4 Add migration/compat tests to ensure old namespace settings are rejected or ignored safely in pre-launch mode.
- [x] SIM2-GC-5-5 Update docs and runbooks to remove all namespace-era instructions and diagrams.


Acceptance criteria (archived):
1. No runtime, dashboard, or docs references remain to simulation telemetry namespace.
2. Dev/prod split is the only data-plane separation model.
3. Source filtering remains possible without schema or storage partition complexity.
4. Cleanup leaves no dead config/code remnants.
#### SIM2-GC-4: Guarantee Monitoring Ingest Uses Real Request Pipeline by Default

- [x] SIM2-GC-4-1 Audit all monitoring emitters and remove SIM-specific alternative emit paths that bypass request processing.
- [x] SIM2-GC-4-2 Ensure adversary requests traverse the same defense middleware/pipeline used for ordinary traffic.
- [x] SIM2-GC-4-3 Add per-defense telemetry assertions for PoW, challenge, maze, honeypot, CDP, rate-limit, and GEO decisions.
- [x] SIM2-GC-4-4 Add integration tests that run SIM traffic and assert monitoring counters/events increase through standard endpoints.
- [x] SIM2-GC-4-5 Add “no-op defense” detector tests that fail if a configured defense never emits events under targeted scenario load.


Acceptance criteria (archived):
1. SIM traffic hits the same runtime defense stack as real traffic.
2. Monitoring data for SIM runs comes from normal runtime telemetry, not synthetic ingestion.
3. Missing per-defense signals under expected attack scenarios fails tests.
4. Operators can refresh monitoring and immediately observe defense activity deltas.
#### SIM2-GC-3: Fix Runtime Toggle/Session Lifecycle So Traffic Persists Beyond Auto-Off

- [x] SIM2-GC-3-1 Split lifecycle semantics into `generation active` vs `historical data visible`.
- [x] SIM2-GC-3-2 Ensure toggle auto-off only stops producers and does not delete or hide prior records from monitoring queries.
- [x] SIM2-GC-3-3 Add explicit retention controls for dev telemetry history with safe defaults and cleanup commands.
- [x] SIM2-GC-3-4 Add regression tests for “run -> auto-off -> refresh monitoring” showing historical adversary traffic remains visible.
- [x] SIM2-GC-3-5 Update UI copy to communicate active-state vs retained-history semantics.


Acceptance criteria (archived):
1. Operators can inspect SIM-generated defense events after auto-off without rerunning simulation.
2. No monitoring view silently filters out prior SIM traffic solely because session ended.
3. Retention behavior is deterministic, documented, and tested.
#### SIM2-GC-13: Remove Adversary Sim Progress Bar and Eliminate Dead UI Runtime Paths

- [x] SIM2-GC-13-1 Remove progress-line markup (`#adversary-sim-progress-line`) and related style hooks from dashboard route/templates.
- [x] SIM2-GC-13-2 Remove progress-timer state (`adversarySimProgressNowMs`, tick interval) and associated scheduling/cleanup logic.
- [x] SIM2-GC-13-3 Delete `deriveAdversarySimProgress` runtime helper and remove any no-longer-used status fields from UI-only normalization contracts.
- [x] SIM2-GC-13-4 Keep lifecycle semantics explicit in UI copy/status (`off`, `running`, `stopping`) without representing run as procedural progress.
- [x] SIM2-GC-13-5 Update unit/e2e tests to assert control behavior and lifecycle state visibility without any progress-width assertions.
- [x] SIM2-GC-13-6 Update docs to remove “top progress line” references and describe auto-off as a guardrail window, not scenario progression.
- [x] SIM2-GC-13-7 Run dead-code sweep for dashboard/runtime modules to remove imports, helpers, and selectors no longer referenced after progress-line removal.


Acceptance criteria (archived):
1. Dashboard no longer renders or references a top adversary-sim progress bar.
2. No progress-timer/tick code remains in dashboard route/runtime modules.
3. Tests verify ON/OFF + lifecycle behavior only and pass without progress assumptions.
4. Operator docs describe sim as enabled/disabled attacker activity bounded by guardrail duration, not choreographed progression.
5. Removal leaves no dead selectors, helper exports, or stale tests.
#### SIM2-GC-2: Re-architect Host Orchestration into Capability-Gated Functional Flow

- [x] SIM2-GC-2-1 Refactor host orchestration into explicit phases: `plan`, `execute`, `collect evidence`, `publish report`.
- [x] SIM2-GC-2-2 Require capability tokens for privileged operations (config mutation, telemetry writes, admin APIs) and forbid implicit fallbacks.
- [x] SIM2-GC-2-3 Move phase decision logic into pure functions with typed inputs/outputs and side effects only in executor boundary.
- [x] SIM2-GC-2-4 Add characterization tests proving behavior parity before/after extraction and proving no telemetry side path bypasses runtime flow.
- [x] SIM2-GC-2-5 Update module-boundary docs with explicit dependency direction and trust-boundary ownership.
- [x] SIM2-GC-2-6 Introduce explicit command contract for adversary toggle control (`operation_id`, idempotency key semantics, requested/accepted state model).
- [x] SIM2-GC-2-7 Separate desired lifecycle state from actual lifecycle state and move reconciliation authority out of read-path status handlers.
- [x] SIM2-GC-2-8 Add controller lease/fencing ownership model so only one reconciler can mutate adversary lifecycle state at a time.
- [x] SIM2-GC-2-9 Add endpoint-specific trust-boundary gate for adversary control submissions (`admin auth`, `csrf token`, strict `origin/referer`, and fetch-metadata policy for unsafe methods).
- [x] SIM2-GC-2-10 Implement payload-bound idempotency replay policy (`Idempotency-Key` required, actor/session scoping, canonical payload hash binding, deterministic TTL expiry behavior).
- [x] SIM2-GC-2-11 Add control-plane abuse throttling envelope (per-session and per-IP ceilings, bounded debounce/queue semantics, explicit throttled outcomes).
- [x] SIM2-GC-2-12 Add structured control-operation security audit schema (`operation_id`, actor/session, decision, reason, origin verdict, idempotency-hash) with sensitive-field redaction rules.


Acceptance criteria (archived):
1. Orchestration no longer has hidden write paths that can fabricate monitoring outcomes.
2. Privileged effects are impossible without capability possession.
3. Pure-policy modules have no direct dependency on storage/providers.
4. Control submissions fail closed on trust-boundary violations (auth/CSRF/origin/replay abuse) with explicit reason taxonomy.
5. Control operations are audit-complete and incident-reconstructable without exposing sensitive material.
6. Tests prove evidence publication is coupled to actual execution outputs.
#### SIM2-GCR: Mandatory Research Program for Gap-Closure Execution

- [x] SIM2-GCR-8 Produce research synthesis docs and implementation plans for `GC-6`, `GC-8`, `GC-11`, and `GC-14`, then update todos with quantitative thresholds derived from research outcomes.
- [x] SIM2-GCR-10 Convert selected research outcomes into ADR-backed architecture decisions for: (a) UI-toggle-driven black-box adversary orchestration, (b) monitoring realtime data architecture, and (c) retention/cost/security lifecycle policies.
- [x] SIM2-GCR-7 Research security/privacy best practices for telemetry and adversary artifacts (secret-exposure prevention, data minimization, pseudonymization options, artifact retention risk controls, incident-response hooks).
- [x] SIM2-GCR-6 Research cost-efficiency patterns for monitoring pipelines (aggregation windows, cardinality controls, event sampling restrictions, compression/serialization tradeoffs, query budget controls).
- [x] SIM2-GCR-5 Research Rust storage/retention best practices for high-volume monitoring/event telemetry (TTL strategy, partitioning/indexing, cleanup cadence, deterministic purge semantics, operator-visible retention health).
- [x] SIM2-GCR-9 Run Rust-focused prototype/benchmark comparisons for realtime monitoring delivery candidates (at minimum cursor polling vs streaming candidate) and record latency/cpu/memory/query-cost tradeoffs.
- [x] SIM2-GCR-4 Research Rust-first realtime monitoring architectures for dashboard freshness (polling with cursoring vs SSE/WebSocket-style streams, backpressure patterns, ordering guarantees, bounded memory/cpu cost).
- [x] SIM2-GCR-2 Research capability-safe black-box runner orchestration patterns for containerized frontier actors (least-authority token handoff, envelope signing, bounded execution, one-way command channels, fail-closed teardown).
- [x] SIM2-GCR-3 Research trust-boundary controls specific to toggle-driven orchestration in a dev server admin interface (auth/CSRF/session boundaries, replay protection, abuse throttling, auditability requirements).
- [x] SIM2-GCR-1 Research architecture patterns for triggering/stopping a black-box LLM adversary from a dev-only admin UI toggle (control-plane API contract, lifecycle state model, idempotency, race handling, kill-switch behavior).


Acceptance criteria (archived):
1. Each `SIM2-GCR-*` track produces a dated research doc in `docs/research/` with source-backed recommendations.
2. Every research doc includes a decision matrix (`option`, `benefits`, `risks`, `resource cost`, `security impact`, `rollback complexity`).
3. Each track produces an implementation plan in `docs/plans/` mapped to specific `SIM2-GC-*` todos.
4. `GC-6`, `GC-8`, `GC-11`, and `GC-14` acceptance criteria are upgraded with explicit quantitative thresholds before implementation begins.
5. Research outputs explicitly justify why chosen approaches are preferred over rejected alternatives.
6. Realtime monitoring architecture decision is backed by benchmark evidence from `SIM2-GCR-9`, not only qualitative preference.
7. Final selected approaches are codified in ADR artifacts before high-risk implementation slices begin.
8. Each completed research track has linked artifacts in all three layers: research doc, plan doc, and TODO updates.
#### SIM2-GC-1: Define End-to-End Contract for “Real Adversary Traffic”

- [x] SIM2-GC-1-1 Write architecture contract doc that defines required invariants for `traffic source`, `execution lane`, `defense path`, `telemetry emission`, and `monitoring visibility`.
- [x] SIM2-GC-1-2 Define explicit prohibited patterns (mock telemetry injection, out-of-band metrics writes, control-plane-only “success” signals).
- [x] SIM2-GC-1-3 Define evidence schema for each run (`request id lineage`, `scenario id`, `lane`, `defenses touched`, `decision outcomes`, `latency/cost`).
- [x] SIM2-GC-1-4 Add contract tests that fail if runner marks scenario success without corresponding runtime telemetry evidence.
- [x] SIM2-GC-1-5 Publish operator-facing definition of done for SIM runs (what must appear in Monitoring and IP Ban views).
- [x] SIM2-GC-1-6 Extend evidence schema with control-plane lineage fields (`control_operation_id`, `requested_state`, `desired_state`, `actual_state`, `actor/session`) for toggle-driven orchestration traceability.

## Additional completions (2026-02-27)


Acceptance criteria (archived):
1. Contract exists in docs and is referenced by runner, runtime, and dashboard modules.
2. Any run lacking required telemetry evidence is marked invalid/failed.
3. Architecture docs clearly separate “executed traffic” from “report-only metadata.”
4. Contributors can no longer pass SIM coverage with synthetic-only monitoring artifacts.
### P0 SIM2 Post-Implementation Shortfall Remediation (Execution Priority)

#### SIM2-ARCH: Functional Orchestration and Capability-by-Construction Uplift

- [x] SIM2-ARCH-1 Publish ADR for functional-core/imperative-shell orchestration and explicit capability model (trust boundaries, migration order, rollback).
- [x] SIM2-ARCH-2 Add characterization test harness capturing current request-path decision outcomes for representative policy matrix.
- [x] SIM2-ARCH-3 Extract side-effect-free `RequestFacts` builders from request/config/provider inputs.
- [x] SIM2-ARCH-4 Extract first policy tranche into pure `PolicyDecisionGraph` stages (IP-range, honeypot, rate, existing-ban) with typed outputs.
- [x] SIM2-ARCH-5 Extract second policy tranche into pure stages (GEO, botness, JS/challenge routing) with typed outputs.
- [x] SIM2-ARCH-6 Introduce explicit effect-intent executor for bans, metrics, monitoring, and event logging side effects.
- [x] SIM2-ARCH-7 Replace convention-based privileged operations with explicit capability objects/tokens at trust boundaries.
- [x] SIM2-ARCH-8 Reduce `src/lib.rs` to thin orchestration shell (`facts -> decisions -> effects -> response`) while preserving behavior.
- [x] SIM2-ARCH-9 Add policy-graph unit coverage and parity tests to prove no behavior regressions across migration slices.
- [x] SIM2-ARCH-10 Update module-boundary docs and operator/developer architecture docs to reflect new orchestration model.
- [x] SIM2-ARCH-11 Ensure all verification remains Makefile-driven (`make test`, `make build`) with no lane bypass.


Acceptance criteria (archived):
1. Core policy decisions become predominantly pure and testable without side effects.
2. Privileged operations are blocked unless explicit capability objects are present.
3. Characterization parity tests prove behavior stability across extraction slices.
4. `src/lib.rs` orchestration complexity is materially reduced and role-focused.
5. Full required verification (`make test`, `make build`) remains green throughout migration.
### P0 Adversarial Traffic Simulation Program

Reference plan: [`docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](../docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)  
Refinement plan: [`docs/plans/2026-02-26-adversarial-simulation-v2-plan.md`](../docs/plans/2026-02-26-adversarial-simulation-v2-plan.md)

- [x] SIM-V2-7 Strict attacker/control-plane separation.
- [x] SIM-V2-12 CI policy tiers and scheduling.
- [x] SIM-V2-17 Release-gate enforcement wiring for coverage + frontier-redteam + deterministic oracle policy.
- [x] SIM-V2-15 Deterministic harness and containerized adversary coexistence contract.
- [x] SIM-V2-20 Simulation event tagging and environment data-plane separation.
- [x] SIM-V2-11 Containerized black-box adversary worker (bounded scope, strict isolation).
- [x] SIM-V2-16 Deterministic repeatability gate for adversarial profiles.
- [x] SIM-V2-18 Frontier finding triage and deterministic promotion pipeline.
- [x] SIM-V2-9 Adversarial live loop observability quality and tarpit monitoring completeness.
- [x] SIM-V2-3 Abuse/evasion regression suite parity (`replay`, `stale`, `ordering`, `cadence`, `retry-storm`).
- [x] SIM-V2-5 Full category coverage profile (`full_coverage`) as pre-release mandatory gate.
- [x] SIM-V2-8 Realistic mixed persona traffic model.
- [x] SIM-V2-21 Minimal simulator self-test harness (non-circular runner correctness anchor).
- [x] SIM-V2-13 Operator interpretation and tuning playbook.
- [x] SIM-V2-6 Dev/test crawl surface and toggle (`/sim/public/...`) with strict production exclusion.
- [x] SIM-V2-4 Quantitative gates for defense effectiveness and collateral/cost control.
- [x] SIM-V2-14 Black-box-only adversary governance and policy lock.
- [x] SIM-V2-2 Unified runner architecture with explicit driver classes (`browser_realistic`, `http_scraper`, `edge_fixture`, `cost_imposition`).
- [x] SIM-V2-1 Manifest v2 contract (`sim-manifest.v2`) for tiered scenarios, traffic model metadata, and category/cost assertions.
- [x] SIM-V2-9A Dev UI toggle orchestration for full adversary run lifecycle.
- [x] SIM-V2-10 Frontier-model adversary configuration and protected-lane enablement (fast/low-cost defaults).
- [x] SIM-V2-19 Frontier data-governance and outbound content minimization policy.

### P0 SIM2 Post-Implementation Shortfall Remediation (Execution Priority)

#### SIM2-SF4: Simulation Telemetry Authenticity

- [x] SIM2-SF4-1 Define signed simulation tag contract (`sim-tag.v1`) including canonical fields, HMAC algorithm, timestamp, and nonce requirements.
- [x] SIM2-SF4-2 Add env-only signing secret lifecycle wiring (defaults/bootstrap/setup/docs) for dev/test simulation environments.
- [x] SIM2-SF4-3 Implement runtime signature/timestamp/nonce validation in `sim_telemetry` before activating simulation context.
- [x] SIM2-SF4-4 Add nonce replay-window enforcement and bounded state handling for simulation tag verification.
- [x] SIM2-SF4-5 Update deterministic runner and container worker to emit valid signed simulation metadata.
- [x] SIM2-SF4-6 Add observability and event taxonomy for invalid simulation tag attempts and verification failures.
- [x] SIM2-SF4-7 Add unit/integration/adversarial tests for valid/invalid/stale/replay simulation-tag paths.
- [x] SIM2-SF4-8 Update operator docs for simulation-signing setup, key rotation, and failure troubleshooting.

#### SIM2-SF3: Traffic-Model Execution Realism

- [x] SIM2-SF3-1 Implement deterministic traffic-execution policy layer that all drivers must pass through.
- [x] SIM2-SF3-2 Implement deterministic think-time behavior from `traffic_model` bounds and scenario seeds.
- [x] SIM2-SF3-3 Implement retry strategy semantics (`single_attempt`, `bounded_backoff`, `retry_storm`) as execution behavior, not metadata.
- [x] SIM2-SF3-4 Implement cookie behavior semantics (`stateful_cookie_jar`, `stateless`, `cookie_reset_each_request`) in request execution.
- [x] SIM2-SF3-5 Add profile-level persona/cohort execution scheduler where required by realism profile contract.
- [x] SIM2-SF3-6 Extend report schema with runtime realism evidence (effective waits, retries, cookie mode usage).
- [x] SIM2-SF3-7 Add quantitative realism gates for persona pacing and retry/cookie envelope conformance.
- [x] SIM2-SF3-8 Add unit/integration/adversarial tests proving `traffic_model` settings change runtime behavior deterministically.
- [x] SIM2-SF3-9 Update adversarial operator documentation with realism metrics interpretation and tuning guidance.

#### SIM2-SF1: Black-Box Lane Capability Enforcement

- [x] SIM2-SF1-1 Add machine-readable lane capability contract artifact (attacker/control allowed paths, headers, and authority surfaces).
- [x] SIM2-SF1-2 Refactor deterministic runner request surface into explicit plane-typed clients so attacker and control capabilities are non-overlapping.
- [x] SIM2-SF1-3 Remove forwarded-secret propagation from attacker-plane headers and hard-fail attacker contract when privileged headers are present.
- [x] SIM2-SF1-4 Replace stale-token white-box re-signing path with black-box stale simulation flow that does not require signing secrets.
- [x] SIM2-SF1-5 Align container black-box worker contract assertions with deterministic lane capability contract for parity.
- [x] SIM2-SF1-6 Add focused tests for lane privilege isolation and stale-token black-box behavior.
- [x] SIM2-SF1-7 Add/refresh Makefile verification target for lane contract checks and wire into mandatory adversarial fast path.
- [x] SIM2-SF1-8 Update adversarial operator docs with explicit attacker/control capability boundary semantics.

#### SIM2-SF2: Coverage Contract Governance

- [x] SIM2-SF2-1 Create canonical coverage contract artifact (`coverage_contract.v1`) containing mandatory full-coverage categories and minimum thresholds.
- [x] SIM2-SF2-2 Add schema/validation rules for canonical contract artifact in manifest validation lane.
- [x] SIM2-SF2-3 Update `full_coverage` gate evaluation to require exact canonical category coverage (no silent omissions).
- [x] SIM2-SF2-4 Add drift-check logic comparing canonical contract vs manifest profile requirements.
- [x] SIM2-SF2-5 Add drift-check logic comparing canonical contract vs SIM2 plan contract rows.
- [x] SIM2-SF2-6 Extend report output with contract version/hash and explicit missing/extra coverage category diagnostics.
- [x] SIM2-SF2-7 Wire coverage drift checks into mandatory Makefile and CI gating paths.
- [x] SIM2-SF2-8 Update adversarial docs/runbooks with contract update protocol and failure triage.

### P0 Immediate Next-Agent Start (Highest Priority): Adversarial Simulation v2

- [x] SIM-V2-11A Dashboard adversary-sim UI state class and styling contract.
- [x] SIM-V2-11B Dashboard runtime environment body-class contract (`dev` vs `prod`).

### P0 Adversarial Traffic Simulation Program

Reference plan: [`docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](../docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)

- [x] SIM-1 Define canonical scenario manifest for botness/threat tiers (`SIM-T0`..`SIM-T4`) and expected outcomes (`allow`, `monitor`, `not-a-bot`, `challenge`, `maze`, `deny_temp`).
- [x] SIM-2 Build a unified simulation harness in `scripts/tests/` that combines browser-realistic, scraper, crawler, and load-generator traffic profiles with deterministic seeds.
- [x] SIM-3 Add replay/sequence-evasion simulation paths (token replay, stale token, order violation, cadence anomalies) to close current threat-coverage gaps.
- [x] SIM-4 Add simulation assertions for effectiveness and cost (`challenge/ban` ratios, false-positive envelope, monitoring write/read amplification guardrails).
- [x] SIM-5 Add tiered Make targets and CI policy (fast mandatory adversarial smoke + scheduled/deep soak profiles).
- [x] SIM-6 Document operator interpretation workflow for simulation failures and tuning actions.

## Additional completions (2026-02-25)

### P1 Research Dossiers (Paper-by-Paper TODOs)

#### Rate Limiting, Tarpit, and Cost-Imposition

##### Tarpit Asymmetry Hardening (`work-gated`, `token-chained`, `egress-budgeted`)

- [x] TAH-ARCH-1 Publish the target module/file map for maze+tarpit ownership and boundaries (provider adapter vs domain runtime vs route handler) and pin it in [`docs/module-boundaries.md`](../docs/module-boundaries.md).
- [x] TAH-ARCH-2 Move tarpit runtime logic out of `src/providers/internal.rs` into dedicated tarpit domain modules (`src/tarpit/runtime.rs`, `src/tarpit/types.rs`) so provider internals remain thin adapters.
- [x] TAH-ARCH-3 Introduce dedicated tarpit HTTP handler ownership (`src/tarpit/http.rs`) and route wiring for progression endpoints; remove ad-hoc route handling from non-domain layers.
- [x] TAH-ARCH-4 Extract shared maze+tarpit primitives for replay/chain/budget key handling into one shared module and migrate both maze runtime and tarpit runtime to consume it.
- [x] TAH-ARCH-5 Define one typed tarpit progression reason taxonomy (enum -> metrics/admin labels) and make request-router/admin/runtime emit only that normalized set.
- [x] TAH-ARCH-6 Reconcile tarpit config model for v2 progression (difficulty bounds + egress budgets) and explicitly deprecate/remove static-drip-only assumptions where no longer valid pre-launch.
- [x] TAH-1 Publish a concise design note/ADR for the v2 tarpit contract: tiny initial response, work-gated progression, token-chain continuity, strict byte/time budgets, and deterministic fallback matrix (`maze`, `block`, short-ban escalation). (See [`docs/adr/0004-tarpit-v2-progression-contract.md`](../docs/adr/0004-tarpit-v2-progression-contract.md).)
- [x] TAH-2 Define and document the tarpit progression envelope schema (signed, short-lived, single-use operation token with `flow_id`, `step`, `issued_at`, `expires_at`, `ip_bucket`, and optional work parameters), including replay and binding rules. (See [`docs/plans/2026-02-23-tarpit-v2-progression-envelope.md`](../docs/plans/2026-02-23-tarpit-v2-progression-envelope.md).)
- [x] TAH-3 Add a dedicated tarpit progression endpoint that verifies work proofs before issuing the next step token/content chunk; reject stale/replayed/out-of-order operations with explicit reason codes and monitoring labels.
- [x] TAH-4 Implement a low-cost server-verified work gate (hashcash-style by default; configurable bounded difficulty window), with clear hooks for future memory-hard alternatives.
- [x] TAH-5 Add adaptive work policy that raises/lower difficulty only within bounded limits based on abuse evidence and budget pressure; keep accessibility-safe fallback path to avoid impossible human flows.
- [x] TAH-6 Replace static large filler payload behavior with iterative gated progression so server sends small chunks per verified step rather than prebuilding a large in-memory body.
- [x] TAH-7 Add strict per-flow token-chain continuity checks (step monotonicity, parent-child digest linkage, single-use operation IDs, and replay TTL semantics) reusing maze shared token primitives where possible.
- [x] TAH-8 Add tarpit egress budget controls in config/admin: global bytes-per-window, per-IP-bucket bytes-per-window, per-flow max-bytes, and per-flow max-duration; classify as KV-tunable vs env-only according to project policy.
- [x] TAH-9 Enforce egress budgets before every tarpit step emission (admission + post-send accounting), with deterministic fallback when any budget is exhausted and explicit event/metric outcomes.
- [x] TAH-10 Add budget state keys and lifecycle semantics (TTL/reset window, counter granularity, site scoping) and ensure enterprise/distributed-state behavior is explicit (no silent divergence in authoritative mode).
- [x] TAH-13 Add tests:
  unit tests for envelope/proof/chain/budget logic;
  integration tests for abuse route -> gated progression -> fallback/escalation;
  concurrency/soak tests for egress-budget correctness under burst traffic.
- [x] TAH-14 Wire new tarpit asymmetry tests into Makefile/CI verification lanes (`make test-unit`, `make test-integration`, `make test`) and add explicit failure-gate criteria for regressions.
- [x] TAH-15 Update operator docs ([`docs/configuration.md`](../docs/configuration.md), [`docs/dashboard.md`](../docs/dashboard.md), runbooks) to explain that tarpit cost-imposition is now progression-gated and budget-capped, including tradeoffs between attacker cost, host egress, and false-positive risk.
- [x] TAH-16 Add bounded timing/content variability (jitter windows and template rotation with strict envelopes) to reduce tarpit fingerprintability without violating budget caps.
- [x] TAH-17 Add pre-generated/cached tarpit content shards (text/media templates) so per-request host compute stays low while preserving high bot ingestion cost.
- [x] TAH-18 Add an explicit crawler-safety policy path for known legitimate bots and sensitive endpoints (for example `robots.txt`) to avoid accidental tarpit impact on benign indexing/ops workflows.

### P0 Fingerprinting + Akamai Architecture Clarity and Runtime Alignment

- [x] Scope acceptance: operators can clearly distinguish `JS Verification`, `JS Verification Interstitial`, `Browser CDP Automation Detection`, `Internal Passive Fingerprint Signals`, and `Akamai Bot Signal` from dashboard/docs.
- [x] Scope acceptance: Fingerprinting tab is Akamai-focused (no generic multi-provider UI exposure).
- [x] Scope acceptance: internal CDP controls are in Config near JS Required and are disabled when JS Required is off.
- [x] Scope acceptance: Akamai modes are exactly `additive` and `authoritative`, with explicit behavior and tests.
- [x] Scope acceptance: JS verification report-path selection is coherent with selected ingestion path (no hardcoded `/cdp-report` mismatch).
- [x] FPAK-1 Canonical terminology contract implemented (new canonical terminology doc and removal of deprecated wording in active operator docs/UI copy).
- [x] FPAK-2 Architecture semantics spec implemented (four-plane trust-boundary/persistence/policy spec with explicit Akamai add-vs-replace matrix).
- [x] FPAK-3 Fingerprinting IA rewrite completed (`Akamai Bot Signal` pane title, top-right Akamai toggle, no provider dropdown).
- [x] FPAK-4 Mode model simplification completed (toggle + `additive|authoritative`; disabled-mode behavior and parsing aligned).
- [x] FPAK-5 Internal CDP controls moved back to Config under JS Required with disabled-state coupling.
- [x] FPAK-6 Runtime report-path coherence fix completed (mode-aware report endpoint selection with regression tests).
- [x] FPAK-7 Additive Akamai blending implemented with bounded contribution and non-short-circuit behavior.
- [x] FPAK-8 Authoritative behavior hardened with deterministic criteria and taxonomy/monitoring alignment.
- [x] FPAK-9 Provenance visibility improved in diagnostics/monitoring + documentation guidance.
- [x] FPAK-10 Edge ingestion trust-boundary hardening implemented (reject untrusted Akamai-shaped payloads with explicit reasons).
- [x] FPAK-11 Documentation synchronization pass completed across dashboard/configuration/api/observability/deployment/bot-defence docs.
- [x] FPAK-12 Verification matrix completed with canonical Makefile targets (`make test`, `make build`) and dashboard/unit/e2e coverage.
- [x] Workstream done: dashboard model is coherent, runtime behavior matches controls, terminology is consistent across UI/API/docs/metrics/tests, and no backward-compatibility shim clutter was introduced.

## Additional completions (2026-02-24)

### P0 Documentation Clarity and Information Architecture

- [x] DOC-OPS-1 Replace operator-facing jargon in user docs with plain language, including one canonical explanation of configurable values:
  variables editable in Admin (stored in runtime config) vs variables that are environment-only (set before startup).
- [x] DOC-OPS-2 Audit `docs/plans/` and `docs/research/` document-by-document, then archive completed or superseded material into dedicated `archive/` subdirectories while keeping active work at top level.
- [x] DOC-OPS-3 Create a single dedicated tarpit implementation reference document and consolidate scattered tarpit explanations to point to this canonical source.

## Additional completions (2026-02-23)

### P0 Deployment Path Excellence (Single-Host + Akamai/Fermyon)

- [x] DEP-SH-1 Publish an explicit single-host (`self_hosted_minimal`) production runbook for average VM/shared-host operators with a 10-minute secure baseline (start/health/rollback).
- [x] DEP-SH-2 Add a single-host post-deploy smoke verification Make target (health, admin auth, metrics, and challenge-route sanity).
- [x] DEP-SH-3 Add `make setup-runtime` for minimal single-host/runtime installs (Rust + wasm target + Spin + config bootstrap prerequisites) without full dev/e2e toolchain.
- [x] DEP-SH-4 Add `make verify-runtime` so single-host operators can validate runtime prerequisites without requiring Node/pnpm/Playwright checks.
- [x] DEP-SH-5 Keep `make setup` as full contributor/dev path; document explicit selection guidance between runtime-only and full-dev setup flows.
- [x] DEP-SH-6 Split build/deploy targets so single-host production build paths do not require dashboard bundle-budget checks; keep budget gates in CI/full-dev verification targets.
- [x] DEP-SH-7 Add profile-first deployment wrappers/docs that keep one common baseline path and layer enterprise-only steps on top (`self_hosted_minimal` base + `enterprise_akamai` overlay).

### P1 Research Dossiers (Paper-by-Paper TODOs)

#### Rate Limiting, Tarpit, and Cost-Imposition

- [x] R-RL-01 Review Raghavan et al., "Cloud Control with Distributed Rate Limiting" (SIGCOMM 2007) and extract distributed limiter semantics for Shuma provider adapters. https://www.microsoft.com/en-us/research/publication/cloud-control-with-distributed-rate-limiting/
- [x] R-RL-03 Review Veroff et al., "Evaluation of a low-rate DoS attack against application servers" (Computers & Security 2008) and capture queue/resource-starvation mitigation patterns. https://doi.org/10.1016/j.cose.2008.07.004
- [x] R-RL-05 Review Srivatsa et al., "Mitigating application-level denial of service attacks on Web servers" (ACM TWEB 2008) and assess admission/congestion control patterns for Shuma policy pipeline. https://research.ibm.com/publications/mitigating-application-level-denial-of-service-attacks-on-web-servers-a-client-transparent-approach
- [x] R-RL-06 Review Lemon, "Resisting SYN flood DoS attacks with a SYN cache" (BSDCon 2002) and capture edge-vs-origin queue protection lessons relevant to Akamai authoritative mode. https://www.usenix.org/legacy/publications/library/proceedings/bsdcon02/full_papers/lemon/lemon_html/index.html
- [x] R-RL-07 Review Chen et al., "SMARTCOOKIE" (USENIX Security 2024) and evaluate split-proxy edge-cookie architecture fit for enterprise Akamai deployments. https://collaborate.princeton.edu/en/publications/smartcookie-blocking-large-scale-syn-floods-with-a-split-proxy-de/
- [x] TP-C1 Reuse shared deception token primitives from maze scope (`MZ-2`) for tarpit progression; do not introduce a tarpit-only token format.
- [x] TP-C2 Reuse shared budget/fallback primitives from maze scope (`MZ-7`) for tarpit limits and deterministic fallback; do not fork budget logic by mode.
- [x] TP-0 Implement internal tarpit availability path so confirmed challenge attacks can sink into tarpit instead of immediate short-ban when maze/tarpit capability is available.
- [x] TP-1 Add tarpit config surface (`tarpit_enabled`, pacing/timeout caps, budget caps, fallback action) with secure defaults and clamping.
- [x] TP-2 Ensure all tarpit KV-editable variables appear in Advanced JSON config and admin/config schema parity checks (env-only exceptions remain env-only).
- [x] TP-3 Implement bounded progressive tarpit behavior with configurable byte-rate and hard timeout, reusing shared primitives.
- [x] TP-4 Enforce strict tarpit budgets (global concurrent streams and per-IP-bucket caps) through the shared budget governor and emit explicit saturation outcomes.
- [x] TP-5 Add deterministic fallback action when tarpit budget is exhausted (`maze` or `block`) via shared fallback matrix.
- [x] TP-6 Add tarpit metrics/admin visibility for activation, saturation, duration, bytes sent, budget fallback, and escalation outcomes.
- [x] TP-7 Escalate persistent tarpit clients to short-ban/block with guardrails to minimize false positives.
- [x] TP-8 Add tarpit integration/e2e coverage (abuse route -> tarpit, budget saturation fallback, replay/tamper paths, and mode/config propagation).
- [x] TP-9 Integrate tarpit budgets/counters with distributed-state work for multi-instance consistency (site-scoped counter keys and site-filtered admin visibility; enterprise authoritative external maze/tarpit backend remains tracked via `OUT-5`/`DEP-ENT-*`).

#### SSH Tarpit and Honeypot Evasion Resistance

- [x] R-SSH-02 Review Bythwood et al., "Fingerprinting Bots in a Hybrid Honeypot" (IEEE SoutheastCon 2023) and assess hybrid interaction design implications for SSH deception tiers. https://doi.org/10.1109/SoutheastCon51012.2023.10115143
- [x] R-SSH-03 Review Vetterl et al., "A Comparison of an Adaptive Self-Guarded Honeypot with Conventional Honeypots" (Applied Sciences 2022) and evaluate adaptive risk-vs-observability controls for Shuma SSH tarpit mode. https://doi.org/10.3390/app12105224
- [x] R-SSH-04 Review Cordeiro/Vasilomanolakis, "Towards agnostic OT honeypot fingerprinting" (TMA 2025) and extract transport-stack realism requirements applicable to SSH tarpit surfaces. https://doi.org/10.23919/TMA66427.2025.11097018

#### IP Range Policy, Reputation Feeds, and GEO Fencing

- [x] R-IP-01 Review Ramanathan et al., "BLAG: Improving the Accuracy of Blacklists" (NDSS 2020) and derive feed-aggregation + false-positive controls for managed CIDR sets. https://doi.org/10.14722/ndss.2020.24232
- [x] R-IP-02 Review Sheng et al., "An Empirical Analysis of Phishing Blacklists" (2009) and extract freshness/latency requirements for update cadence and rollout safety. https://kilthub.cmu.edu/articles/journal_contribution/An_Empirical_Analysis_of_Phishing_Blacklists/6469805
- [x] R-IP-03 Review Oest et al., "PhishTime" (USENIX Security 2020) and map continuous quality-measurement methodology to Shuma feed validation. https://www.usenix.org/conference/usenixsecurity20/presentation/oest-phishtime
- [x] R-IP-04 Review Li et al., "HADES Attack" (NDSS 2025) and define anti-poisoning controls for any external blocklist ingestion pipeline. https://doi.org/10.14722/ndss.2025.242156
- [x] R-IP-05 Review Deri/Fusco, "Evaluating IP Blacklists effectiveness" (FiCloud 2023) and identify practical precision/recall limits for aggressive edge enforcement. https://research.ibm.com/publications/evaluating-ip-blacklists-effectiveness

### P2 Challenge Roadmap

- [x] NAB-0 Research and policy synthesis: keep [`docs/research/2026-02-19-not-a-bot-challenge-research-synthesis.md`](../docs/research/2026-02-19-not-a-bot-challenge-research-synthesis.md) and [`docs/plans/2026-02-13-not-a-bot-excellence-plan.md`](../docs/plans/2026-02-13-not-a-bot-excellence-plan.md) aligned as the implementation source.
- [x] NAB-1 Implement Not-a-Bot checkbox (`/challenge/not-a-bot-checkbox`) per [`docs/plans/2026-02-13-not-a-bot-excellence-plan.md`](../docs/plans/2026-02-13-not-a-bot-excellence-plan.md) with signed short-lived single-use nonce and IP-bucket binding.
- [x] NAB-2 Implement Not-a-Bot telemetry capture/validation and deterministic scoring model (`0..10`) with threshold routing (`pass`, `escalate_puzzle`, `maze_or_block`).
- [x] NAB-3 Add Not-a-Bot verification marker/token issuance after pass and enforce it in routing flow.
- [x] NAB-4 Add Not-a-Bot routing integration so medium-certainty traffic hits Not-a-Bot before puzzle escalation, with deterministic maze/block fallback.
- [x] NAB-5 Add Not-a-Bot admin visibility/config controls for thresholds, TTL, and attempt caps (read-only defaults plus optional mutability controls).
- [x] NAB-6 Add Not-a-Bot monitoring parity (`served`, `pass`, `escalate`, `fail`, `replay`, solve-latency buckets, abandonment estimate) and dashboard exposure.
- [x] NAB-7 Add dedicated e2e browser coverage for Not-a-Bot lifecycle and replay/abuse paths (unit + integration coverage is now in place).
- [x] NAB-8 Add operator docs and threshold tuning guidance aligned to low-friction managed-first routing.
- [x] NAB-9 Align Not-a-Bot control behavior to one-step state-of-the-art UX (checkbox-like control + auto-progress on activation).
- [x] NAB-10 Explicitly document the very-low-certainty managed/invisible path mapping (passive + JS/PoW) and keep Not-a-Bot medium-certainty only.
- [x] NAB-11 Preserve accessibility-neutral scoring policy: keyboard/touch flows remain pass-capable; assistive paths are never direct negative signals.

## Additional completions (2026-02-20)

### P1 IP Range Policy Controls

- [x] IPR-1 Add typed IP-range policy config model (`off|advisory|enforce`, emergency allowlist, custom rules, managed-set policies) with strict schema validation.
- [x] IPR-2 Implement runtime CIDR evaluation engine with deterministic precedence:
  emergency allowlist > operator custom rules > managed set policies > default pipeline.
- [x] IPR-3 Implement response-action matrix for IP-range matches:
  `403_forbidden`, `custom_message`, `drop_connection`, `redirect_308`, `rate_limit`, `honeypot`, `maze`, `tarpit` with deterministic fallback behavior.
- [x] IPR-4 Add advisory/dry-run mode behavior: fully log/telemetry match outcomes while allowing normal request flow.
- [x] IPR-5 Add managed built-in set catalog with provenance/version metadata and official source snapshots:
  OpenAI (`gptbot`, `searchbot`, `chatgpt-user`) + GitHub `copilot`.
- [x] IPR-6 Add managed-set update tooling with anti-poisoning guardrails:
  source allowlist, HTTPS-only, schema checks, CIDR validation, broad-prefix guards, entry caps.
- [x] IPR-7 Add explicit "official source unavailable" handling for DeepSeek managed set (research-tracked gap; no unverified defaults).
- [x] IPR-8 Add admin API read/write + config export coverage for all IP-range policy fields.
- [x] IPR-9 Add observability for IP-range policy (matched source/action/mode, advisory vs enforce, fallback counts).
- [x] IPR-10 Add unit/integration coverage for precedence, action routing, advisory mode, and managed/custom override interactions.
- [x] IPR-11 Document operator runbook: false-positive mitigation, rollout sequence, rollback, stale-feed handling, and cost controls.
- [x] IPR-12 Enforce managed-catalog staleness controls in runtime (`ip_range_managed_max_staleness_hours`, `ip_range_allow_stale_managed_enforce`) with admin/docs visibility.

## Additional completions (2026-02-19)

### P0 Dashboard Monitoring Freshness + Efficiency Remediation (Next Priority)

- [x] DSH-MON-EX1 Unify monitoring freshness model so auto-refresh updates all monitoring sections/cards/tables/charts from equally fresh snapshots (remove mixed stale-vs-fresh snapshot wiring).
- [x] DSH-MON-EX2 Add explicit Monitoring refresh mode controls: default `manual` (auto-refresh OFF), user-toggle `auto`, visible last-updated age, and explicit manual refresh action.
- [x] DSH-MON-EX3 Rework week/month monitoring range fetch lifecycle: in manual mode fetch only on explicit user action/range change; in auto mode refresh on bounded cadence with abort/dedupe guards to keep host/API cost low.
- [x] DSH-MON-EX4 Fix AI search policy toggle semantics and naming so UI labels, variable names, baseline comparisons, and saved payload fields are non-inverted and unambiguous.
- [x] DSH-SX-EX5 Remove duplicated route/controller responsibilities (hash read/write, visibility, timing helpers) by consolidating browser orchestration primitives in controller/runtime utilities and keeping route layer declarative.
- [x] DSH-SX-EX6 Rebalance dashboard automated tests toward behavior/outcome contracts (refresh-mode behavior, freshness parity, range refresh cadence, toggle semantics) and trim brittle source-string-only assertions to a minimal architecture guard set.
- [x] DSH-SX-EX7 Add Svelte static diagnostics to setup/verification paths (`svelte-check` dependency + Makefile target + CI/`make test-dashboard-*` wiring) so Svelte compile/type warnings fail fast.

### P0 Monitoring Cost/Security Hardening (2026-02-19)

- [x] MON-COST-1 Fix event-log retention cleanup so all buckets older than retention are deleted (not just one cutoff hour bucket).
- [x] MON-COST-2 Move event/monitoring cleanup scans off hot request write paths and keep retention enforcement deterministic.
- [x] MON-COST-3 Add success-path abuse controls for expensive admin read endpoints (`/admin/events`, `/admin/cdp/events`, `/admin/monitoring`) to reduce KV/CPU amplification risk.
- [x] MON-COST-4 Reduce monitoring read-side self-amplification by removing routine read-view AdminAction event writes.
- [x] MON-COST-5 Harden monitoring write-path cost profile by reducing per-request KV read/modify/write amplification.
- [x] MON-COST-6 Add path-dimension cardinality guardrails for monitoring telemetry keys to prevent unbounded key explosion.
- [x] MON-COST-7 Treat provider Redis URLs as secret export values; never include credential-bearing URLs in `/admin/config/export`.
- [x] MON-COST-8 Reduce dashboard monitoring cache serialization/storage overhead and clear monitoring/IP-ban cache on logout/session end.
- [x] MON-COST-9 Sanitize external documentation URLs in dashboard monitoring helper rendering.
- [x] MON-COST-10 Document retention, read throttling, and monitoring telemetry cost controls in operator docs.
- [x] SEC-GDPR-1 Run a GDPR/privacy compliance review for telemetry/logging data collected by Shuma and determine whether deployment contexts require a cookie consent notice and/or other disclosure controls. ([`docs/privacy-gdpr-review.md`](../docs/privacy-gdpr-review.md))

## Additional completions (2026-02-19, TODO cleanup sweep)

### todos/todo.md

#### P0 Priority Override (Highest Priority Queue)

- [x] Complete the remaining SvelteKit migration work (`DSH-SVLT-NEXT1.*`, `DSH-SVLT-NEXT2.*`, `DSH-SVLT-NEXT3.*`, `DSH-SVLT-TEST1.*`, `DSH-SVLT-TEST2.*`) before non-critical roadmap work.
- [x] Treat all non-blocking research/backlog items below as lower priority until the Svelte-native dashboard path replaces the bridge path.
- [x] Complete the dashboard excellence remediation slice (`DSH-MON-EX1`..`DSH-MON-EX7`) before picking up additional roadmap/research backlog items.

#### Fingerprinting, JS Verification, and CDP-Adjacent Detection

- [x] Strengthen fingerprinting by hardening internal baseline signals first, then ingesting trusted upstream edge signals (JA3/JA4 and similar) with provenance checks and explicit internal fallback when edge headers are absent or untrusted.
- [x] Normalize fingerprint signals with provenance/confidence metadata for rule evaluation.
- [x] FP-R11 Add feature-family entropy budgeting and per-family confidence caps (avoid over-weighting high-cardinality unstable attributes).
- [x] FP-R20 Add fingerprint data-minimization and retention controls (TTL/pseudonymization/export visibility) plus operator documentation.
- [x] FP-R15 Expand cross-layer inconsistency rules: UA, client hints, runtime/browser APIs, and transport-level fingerprints.
- [x] Add mismatch heuristics (for example UA/client-hint versus transport fingerprint anomalies).
- [x] FP-R12 Add temporal coherence modeling with per-attribute churn classes and impossible-transition detection IDs.
- [x] FP-R16 Add flow-centric fingerprint telemetry extraction and bounded per-flow aggregation windows.
- [x] FP-R13 Add JS/CDP detector-surface rotation support (versioned probe families + staged rollout + rollback controls).
- [x] Add trusted-header ingestion for transport fingerprints supplied by CDN/proxy.
- [x] FP-R14 Add multi-store persistence-abuse signals (cookie/localStorage/sessionStorage/IndexedDB recovery patterns) as suspicious automation features.
- [x] FP-R17 Add optional challenge-bound, short-lived device-class marker path (Picasso-inspired) for replay-resistant continuity checks.
- [x] FP-R18 Add optional low-friction behavioral micro-signals in challenge contexts (mouse/timing), with privacy guardrails and conservative weighting.
- [x] Add fingerprint-centric admin visibility for investigations and tuning.
- [x] FP-R19 Add evasive-regression coverage for detector fingerprinting, temporal drift, and inconsistency-bypass attempts.

#### P3 Platform and Configuration Clarity

- [x] Initialize Ban IP pane duration controls from the current Admin Manual Ban default duration so Ban IP and Ban Durations panes stay consistent.
- [x] Dashboard modernization now follows SvelteKit full cutover (`DSH-SVLT-*`) with static adapter output served via Spin (`dist/dashboard`), superseding the prior framework migration direction.

#### P3 Monitoring Signal Expansion (Dashboard + Telemetry)

- [x] DSH-MON-1 Add a `Honeypot Hits` monitoring section (mirroring maze summary style) with: total hits, unique crawler buckets, top crawler buckets, and top honeypot paths hit.
- [x] DSH-MON-2 Add a `Challenge Failures` monitoring section with time-windowed totals and reason breakdown (`incorrect`, `expired/replay`, `sequence_violation`, `invalid_output`, `forbidden`), plus trend chart.
- [x] DSH-MON-3 Add a `PoW Failures` monitoring section with time-windowed totals and reason breakdown (`invalid_proof`, `missing_seed/nonce`, `sequence_violation`, `expired/replay`, `binding/timing mismatch`), plus trend chart.
- [x] DSH-MON-4 Add a `Rate Limiting Violations` monitoring section with totals, unique offender buckets, top offender buckets, and enforcement outcomes (`limited`, `banned`, `fallback_allow`, `fallback_deny`).
- [x] DSH-MON-5 Add a `GEO Violations` monitoring section with totals by route/action (`block`, `challenge`, `maze`) and top country codes causing policy actions.
- [x] DSH-MON-6 Add a Monitoring-page helper panel that explains how to export/scrape the same signals in Prometheus format (`/metrics`) for external visualization platforms (for example Prometheus/Grafana), including copyable scrape examples.
- [x] DSH-MON-7 Deliberate Prometheus parity scope for Monitoring: audit each Monitoring widget/signal as `already_exported`, `derivable_from_existing_series`, or `missing_export`; then define a prioritized add-list with cardinality/cost guardrails before implementing new metric series. ([`docs/monitoring-prometheus-parity-audit.md`](../docs/monitoring-prometheus-parity-audit.md))
- [x] MON-TEL-1 Add structured honeypot hit telemetry (KV/metric counters by IP bucket and path key) so dashboard can report path-level honeypot activity without relying on free-form event text parsing.
- [x] MON-TEL-2 Add challenge-submit failure telemetry with explicit counters and optional event records for failure classes that currently only increment coarse counters (enable top-offender and reason panels).
- [x] MON-TEL-3 Add explicit PoW verify outcome telemetry (success + failure classes) since invalid-proof and malformed-request paths are not currently surfaced as dashboard-ready counters/events.
- [x] MON-TEL-3.a Add PoW verify success-class telemetry and decide whether Monitoring should expose success/fail ratio or keep failures-only.
- [x] MON-TEL-5 Add GEO enforcement telemetry keyed by action + country (bounded cardinality, ISO country normalization) so GEO monitoring panels are robust and not dependent on outcome-string parsing.
- [x] MON-TEL-6 Add admin API surface for these monitoring summaries (`/admin/honeypot`, `/admin/challenge`, `/admin/pow`, `/admin/rate`, `/admin/geo` or consolidated endpoint) with strict response schema + docs.
- [x] MON-TEL-7 Add tests for telemetry correctness and dashboard rendering states (empty/loading/error/data) for each new monitoring section, including cardinality guardrails and retention-window behavior.
- [x] MON-TEL-7.a Extend dashboard automated tests to assert new monitoring cards/tables/charts across empty/loading/error/data states, not just adapter contracts.

#### P2 Modularization and Future Repository Boundaries

- [x] Restructure source into clearer domain modules (policy engine, maze/tarpit, challenges, fingerprint signals, admin adapters).
- [x] Extract policy decision flow from HTTP plumbing to enable isolated testing and future reuse.
- [x] Define module interface contracts and dependency direction (core domain first, adapters second).

#### Policy/Guideline Backlog Hygiene

- [x] Moved internal-first delivery policy from active TODO guidance into [`docs/project-principles.md`](../docs/project-principles.md) (`P7 Platform-Agnostic Core`) so it is governed as principle, not tracked as open feature work.
- [x] Removed duplicated policy-only sections from active TODO (`H4`, `H5`, and `Recurring Quality Gates`) because those rules are already governed by `CONTRIBUTING.md` and [`docs/project-principles.md`](../docs/project-principles.md).
- [x] Captured recurring quality-gate operations guidance in canonical docs: verification-lane health in `CONTRIBUTING.md` and periodic retention reassessment in [`docs/security-hardening.md`](../docs/security-hardening.md).

## todos/todo.md

- [x] Define sprint guardrails: refactor-only, no behavior changes, no new dependencies, tests must pass before each checkoff.
- [x] M1 Extract inline shadow-mode block from `src/lib.rs` into dedicated shadow-mode module (`src/runtime/shadow_mode/mod.rs`).
- [x] M2 Add focused unit tests for extracted shadow-mode behavior (cover bypass, block, and allow outcomes).
- [x] M3 Keep `src/lib.rs` behavior identical by routing existing shadow-mode flow through the new module.
- [x] M4 Run verification (`cargo test` and integration smoke path) and record result.
- [x] M5 Plan and execute next extraction slice from `src/lib.rs` (routing/decision helpers) with similarly scoped checklist items.
- [x] M5.1 Extract early endpoint routing (`/health`, `/admin`, `/metrics`, `/robots.txt`, challenge endpoints) into a dedicated router helper/module without changing semantics.
- [x] M5.2 Extract KV outage/open-close gate into a dedicated helper to isolate fail-open/fail-closed behavior.
- [x] M5.3 Extract post-config enforcement pipeline ordering into named helpers (honeypot, rate, ban, geo policy, botness, JS).
- [x] M5.4 Add regression tests for routing order and short-circuit precedence after extraction.
- [x] M6 Dashboard decomposition track (`dashboard/dashboard.js` split into domain modules under `dashboard/modules/`).
- [x] M6.1 Extract charts/timeseries logic into `dashboard/modules/charts.js` and wire via a stable API surface.
- [x] M6.2 Extract status panel state/rendering into `dashboard/modules/status.js` and remove status-specific globals from root script.
- [x] M6.3 Extract config/tuning save handlers into `dashboard/modules/config-controls.js`.
- [x] M6.4 Extract admin actions/session endpoint wiring into `dashboard/modules/admin-session.js`.
- [x] M6.5 Add frontend smoke checks for key interactions (login/session restore, chart refresh, config save buttons enabled/disabled state).
- [x] M7 Maze domain decomposition track (split `src/maze.rs` into `src/maze/` submodules: generation, routing hooks, telemetry, templates).
- [x] M7.1 Convert `src/maze.rs` into `src/maze/mod.rs` with identical public API (`is_maze_path`, `handle_maze_request`, `generate_maze_page`, `MazeConfig`).
- [x] M7.2 Extract deterministic generation primitives (`SeededRng`, path seeding, content generators) into focused submodules.
- [x] M7.3 Extract HTML rendering/template assembly into a dedicated maze template module.
- [x] M7.4 Isolate maze request/path helpers so later telemetry/routing-hook extraction from `src/lib.rs` has a stable seam.
- [x] M7.5 Keep/extend maze tests after extraction and run verification (`cargo test`, integration smoke).
- [x] M8 Challenge domain decomposition track (split `src/challenge.rs` into `src/challenge/` submodules: token/crypto, puzzle generation, HTTP handlers, validation/anti-replay).
- [x] M8.1 Convert `src/challenge.rs` into `src/challenge/mod.rs` while preserving public API used by `src/lib.rs` and tests.
- [x] M8.2 Extract seed token/HMAC logic into a dedicated `src/challenge/puzzle/token.rs` module.
- [x] M8.3 Extract puzzle generation/transform/validation logic into `src/challenge/puzzle/mod.rs`.
- [x] M8.4 Extract rendering and submit/anti-replay flow into focused HTTP modules (`src/challenge/puzzle/renders.rs`, `src/challenge/puzzle/submit.rs`).
- [x] M8.5 Run verification (`cargo test`) to confirm no behavior change.
- [x] M9 Directory structure prep for future repo boundaries (core policy, maze+tarpit, challenge, dashboard adapter) with explicit interface contracts.
- [x] M9.1 Add explicit Rust boundary contracts for challenge/maze/admin in `src/boundaries/contracts.rs`.
- [x] M9.2 Add default adapter implementations in `src/boundaries/adapters.rs` and route `src/lib.rs` through `src/boundaries/`.
- [x] M9.3 Document boundary rules and target split direction in [`docs/module-boundaries.md`](../docs/module-boundaries.md).

- [x] Enforce `SHUMA_ADMIN_IP_ALLOWLIST` in every production environment.
- [x] Configure CDN/WAF rate limits for `POST /admin/login` and all `/admin/*` in every deployment (Cloudflare and Akamai guidance already documented).
- [x] Rotate `SHUMA_API_KEY` using `make gen-admin-api-key` and set a regular rotation cadence.
- [x] Add deployment runbook checks for admin exposure, allowlist status, and login rate-limit posture.
- [x] Add stronger admin controls for production tuning: split read/write privileges and keep audit visibility for write actions. (`src/auth.rs`, `src/admin.rs`, dashboard, docs)
- [x] P0.1 slice completed: hardened `make deploy-env-validate` to require non-empty/non-overbroad `SHUMA_ADMIN_IP_ALLOWLIST` and expanded deployment runbook checklist coverage for admin exposure, allowlist status, and login rate-limit posture.
- [x] P0.2 slice completed: added deploy-time edge-rate-limit attestation guard (`SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`) so production deploys fail until `/admin/login` and `/admin/*` CDN/WAF rate limits are explicitly confirmed.
- [x] P0.3 slice completed: added optional read-only admin bearer key (`SHUMA_ADMIN_READONLY_API_KEY`), enforced write-access checks on mutating admin routes, hardened `/admin/unban` to POST-only, and logged denied write attempts for audit visibility.
- [x] P0.4 slice completed: added `gen-admin-api-key` alias + deploy-time API key rotation attestation guard (`SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`) and documented a recommended 90-day rotation cadence in deployment/security runbooks.

- [x] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Formalize profile-gated state-plane architecture: shared policy engine across personas, swappable state backends by profile (`self_hosted_minimal` vs `enterprise_akamai`), and no persona-specific policy fork.
- [x] P1.0 slice completed: documented the profile-gated state-plane architecture as ADR [`docs/adr/0001-profile-gated-state-plane.md`](../docs/adr/0001-profile-gated-state-plane.md) and synchronized policy/deployment/config docs.
- [x] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Design and implement atomic distributed rate limiting (Redis `INCR`/Lua) for main traffic and admin auth, aligned with edge-state sync work. (`src/rate.rs`, `src/auth.rs`, `spin.toml`)
- [x] (Enterprise/hybrid track; non-blocking for `self_hosted_minimal`) Define outage posture for distributed limiter (`fail-open` vs `fail-closed`) and add monitoring/alerts for limiter backend health. (architecture, ops, [`docs/deployment.md`](../docs/deployment.md))
- [x] (Enterprise/hybrid track) Add deploy guardrails that block unsafe multi-instance enterprise rollouts when `rate_limiter`/`ban_store` remain local-only, with explicit override attestation for temporary advisory-only exceptions.
- [x] P1.1 slice completed: `make deploy-env-validate` now enforces multi-instance enterprise state posture (`SHUMA_ENTERPRISE_MULTI_INSTANCE`) and blocks authoritative local-only rate/ban state while requiring explicit advisory/off exception attestation (`SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true`) for temporary unsynced operation.
- [x] (Enterprise/hybrid track) Add startup/runtime warnings or hard-fail hooks for enterprise multi-instance local-only state posture, aligned with deploy guardrail semantics.
- [x] P1.2 slice completed: runtime config loading now enforces enterprise multi-instance state guardrails (hard-fail on unsafe posture) and `/admin/config` surfaces enterprise guardrail warnings/errors and attestation visibility fields.
- [x] P1.3 slice completed: replaced external `rate_limiter` stub with a Redis-backed distributed adapter (`INCR` + TTL window key), added explicit fallback-to-internal behavior, and enforced `SHUMA_RATE_LIMITER_REDIS_URL` guardrails for enterprise multi-instance posture.
- [x] P1.4 slice completed: replaced external `ban_store` stub with a Redis-backed distributed adapter (JSON ban entries + Redis TTL), routed admin ban/unban/list paths through provider selection, and enforced `SHUMA_BAN_STORE_REDIS_URL` guardrails for enterprise multi-instance posture.
- [x] P1.5 slice completed: routed admin auth failure throttling through the provider-selected rate limiter so external distributed rate-limiter mode covers admin auth (`/admin/login`, `/admin/logout`, unauthorized admin endpoints) with safe internal fallback when runtime config/provider selection is unavailable.
- [x] P1.6 slice completed: added route-class outage posture controls for external rate limiter degradation (`SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN`, `SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH`) and shipped limiter degradation observability (`backend_errors`, `outage_decisions`, `usage_fallback`, `state_drift` metrics) with deployment/config docs.

- [x] S1.0 taxonomy spec drafted: canonical escalation levels, signal IDs, transition precedence, and current signal-collection map documented in [`docs/plans/archive/2026-02-14-stage1-policy-signal-taxonomy-spec.md`](../docs/plans/archive/2026-02-14-stage1-policy-signal-taxonomy-spec.md).
- [x] Add stable detection ID taxonomy and policy matching using canonical escalation/action IDs from [`docs/plans/archive/2026-02-14-stage1-policy-signal-taxonomy-spec.md`](../docs/plans/archive/2026-02-14-stage1-policy-signal-taxonomy-spec.md) (`L0_ALLOW_CLEAN` through `L11_DENY_HARD`) and canonical signal IDs (`S_*`) in logs/metrics/admin events.
- [x] S1.1 slice completed: added `src/runtime/policy_taxonomy.rs` canonical IDs + deterministic precedence tests, threaded policy-match telemetry through runtime/CDP/external event paths, and exposed canonical policy/signal metrics (`bot_defence_policy_matches_total`, `bot_defence_policy_signals_total`) plus taxonomy-annotated admin event outcomes.
- [x] Add static-resource bypass defaults to avoid expensive bot checks on obvious static assets.
- [x] S1.2 slice completed: added early static-asset bypass defaults for obvious `GET`/`HEAD` resource paths/extensions (with admin/challenge/control-path exclusions) to skip expensive JS/botness/geo challenge checks, plus unit + routing-order regression coverage.
- [x] Keep all generated build artifacts out of `src/` (including WASM binaries) and move them to a dedicated artifacts path (for example `dist/wasm/`).
- [x] Update `spin.toml`, Makefile targets, and bootstrap scripts to consume the new artifacts path without changing runtime behavior.
- [x] Keep Playwright and test outputs ephemeral (`playwright-report/`, `test-results/`) and confirm ignore rules remain correct after any directory changes.
- [x] Add a short doc section describing expected generated directories and what should never be committed.

- [x] Define and document project test conventions:
  unit tests colocated with module code,
  integration/behavior tests in `tests/`.
- [x] Create a shared test support module (request builders, env guards, common fixtures) to reduce duplication across current `src/*_tests.rs`.
- [x] Incrementally migrate top-level `src/*_tests.rs` files into colocated module tests and/or `tests/` integration suites (no behavior changes).
- [x] Keep test discovery and CI commands stable (`cargo test`, Make targets) throughout migration.
- [x] Add/adjust regression tests to ensure routing and enforcement order remain stable while tests move (runtime-backed early routes should be covered in integration-level tests, not native unit tests).
- [x] H2 slice A completed: moved ban/CDP/GEO/request-router/shadow-mode/allowlist test files to module-local paths and removed corresponding top-level test module wiring in `src/lib.rs`.
- [x] H2 slice A completed: added shared unit-test helpers in `src/test_support.rs` and adopted them in env-sensitive suites.
- [x] H2 slice A verification: `cargo test` passes after migration with no behavior changes.
- [x] H2 slice B completed: migrated config tests from `src/config_tests.rs` to module-local `src/config/tests.rs`, including shared env-lock adoption.
- [x] H2 slice C completed: migrated challenge tests from `src/challenge_tests.rs` to module-local `src/challenge/tests.rs`.
- [x] H2 slice B/C verification: `cargo test` passes with module-local config/challenge suites.
- [x] H2 slice D completed: migrated remaining crate-level test files (`risk`, `security`, `logging`) into structured `src/lib_tests/` modules and removed legacy top-level `src/*_tests.rs`.
- [x] H2 slice D verification: `cargo test` passes with stable test discovery after `src/lib_tests/` adoption.

- [x] Move orchestration helpers (`request_router`, `kv_gate`, `policy_pipeline`) into a cohesive runtime/policy directory with clear ownership boundaries.
- [x] Group admin/auth/config concerns into a cohesive adapter/domain boundary layout with minimal cross-module leakage.
- [x] Group signal and enforcement modules by domain (for example risk signals, enforcement actions, challenge/maze) and reduce root-level file sprawl.
- [x] Add thin compatibility re-exports during moves so refactors remain reviewable and low-risk.
- [x] Remove temporary compatibility shims once imports are fully migrated.
- [x] H3.1 slice completed: moved request orchestration modules into `src/runtime/` (`runtime/request_router.rs`, `runtime/kv_gate.rs`, `runtime/policy_pipeline.rs`) and rewired `src/lib.rs` call sites without behavior changes.
- [x] H3.2 slice completed: moved admin/auth into `src/admin/` (`admin/mod.rs`, `admin/api.rs`, `admin/auth.rs`) and moved config into `src/config/mod.rs`, then rewired module imports with no behavior change.
- [x] H3.3/H3.4 slice completed: regrouped signal modules under `src/signals/` and enforcement modules under `src/enforcement/`, then added crate-level compatibility re-exports in `src/lib.rs` to keep call sites stable during the move.
- [x] H3.5 slice completed: migrated remaining call sites to `src/signals/*` and `src/enforcement/*` paths and removed temporary compatibility re-exports from `src/lib.rs`.

- [x] Define and document the defence taxonomy with an explicit inventory of `signal`, `barrier`, and `hybrid` modules (for example `rate` as hybrid); include ownership and dependency direction. ([`docs/module-boundaries.md`](../docs/module-boundaries.md), Defence Taxonomy section)
- [x] Introduce a canonical per-request signal contract (for example `BotSignal` + `SignalAccumulator`) that every signal/hybrid module writes to.
- [x] Add explicit signal availability semantics (`active`, `disabled`, `unavailable`) so botness logic never treats missing modules as silent zero.
- [x] Split hybrid modules into distinct paths:
  rate telemetry signal contribution for scoring,
  hard rate-limit enforcement barrier for immediate protection.
- [x] Add composability modes for eligible modules (`off`, `signal`, `enforce`, `both`) while keeping safety-critical controls non-disableable.
- [x] Define clear behavior for each mode in config/admin surfaces and runtime flow (including invalid combinations and defaults).
- [x] Refactor botness scoring to consume normalized accumulator output rather than direct module internals.
- [x] Lock explicit pre-launch default mode semantics and enforce via tests (`rate=both`, `geo=both`, `js=both`, with JS still gated by `js_required_enforced`).
- [x] Add unit and integration regression tests for mode matrix behavior and ordering invariants (especially hybrid modules and early-route interactions).
- [x] Add observability for mode and signal-state visibility (metrics/log fields indicating enabled/disabled/unavailable contributors).
- [x] Update docs (`configuration`, `features`, `observability`, `module-boundaries`) to explain composability semantics and tuning implications.
- [x] Keep implementations internal-only for now; defer external provider registry/factory work until signal contract and mode semantics stabilize.
- [x] H3.6.1 slice completed: added explicit defence taxonomy + inventory (`signal`, `barrier`, `hybrid`) with ownership and dependency direction in [`docs/module-boundaries.md`](../docs/module-boundaries.md).
- [x] H3.6.2 slice completed: introduced `BotSignal`/`SignalAccumulator` in `src/signals/botness.rs` and rewired JS, GEO, and rate-pressure botness scoring paths in `src/lib.rs` to emit normalized signal contributions with no behavior change.
- [x] H3.6.3 slice completed: added explicit signal availability states (`active`, `disabled`, `unavailable`) across JS/GEO/rate signal emitters and botness assessment flow, with regression tests for non-silent disabled/unavailable handling.
- [x] H3.6.4 slice completed: split rate hybrid paths into `src/signals/rate_pressure.rs` (telemetry + pressure scoring signals) and `src/enforcement/rate.rs` (hard limit enforcement path), then rewired runtime botness flow accordingly.
- [x] H3.6.5 slice completed: added per-module composability modes (`off`, `signal`, `enforce`, `both`) for JS/GEO/rate with runtime signal/action gating and admin-config validation, preserving default behavior as `both`.
- [x] H3.6.6 slice completed: defined explicit mode semantics in runtime/config/admin surfaces, added effective-mode + warning payloads (`defence_modes_effective`, `defence_mode_warnings`), and validated invalid mode key/value combinations.
- [x] H3.6.7 slice completed: introduced `BotnessSignalContext` and split botness into contribution collection + score finalization (`collect_botness_contributions`, `compute_botness_assessment_from_contributions`) so runtime policy consumes normalized contributions rather than direct scoring internals.
- [x] H3.6.8 slice completed: locked pre-launch default-mode semantics with explicit config tests and added mode-matrix regression coverage for JS/GEO/rate signal paths (including rate hybrid signal behavior), while retaining early-route ordering integration guards.
- [x] H3.6.9 slice completed: added botness signal-state and effective defence-mode observability (`botness_signal_state_total`, `defence_mode_effective_total`) plus richer botness log outcomes (`signal_states`, `modes`) for maze/challenge decisions.
- [x] H3.6.10 slice completed: updated composability/tuning/operator docs ([`docs/configuration.md`](../docs/configuration.md), [`docs/features.md`](../docs/features.md), [`docs/observability.md`](../docs/observability.md), [`docs/module-boundaries.md`](../docs/module-boundaries.md)) with effective-mode semantics and observability guidance.
- [x] H3.6.11 slice completed: kept implementation internal-only (no provider registry/factory introduced) and explicitly deferred external-provider wiring to H4.

- [x] Define provider traits for swappable capabilities:
  rate limiting,
  ban store/sync,
  challenge engine,
  maze/tarpit serving,
  fingerprint signal source.
- [x] Add a provider registry/factory that selects implementations from config (compile-time/runtime config, no behavior change by default).
- [x] Implement `Internal*` providers matching current behavior as the default path.
- [x] Define and document provider externalization matrix by deployment persona:
  `self_hosted_minimal` (default),
  `enterprise_akamai` (target managed-edge integration),
  with advisory-by-default and authoritative-optional edge signal precedence.
- [x] Add explicit `External*` adapter stubs/contracts for high-leverage capabilities first:
  `fingerprint_signal`,
  `rate_limiter`,
  `ban_store`,
  `challenge_engine`,
  with explicit unsupported handling for `maze_tarpit` until a stable external API target exists.
- [x] Add contract tests that every provider implementation must pass to guarantee semantic parity and explicit unavailability behavior (`active`/`disabled`/`unavailable`) for external signal sources.
- [x] Add observability tags/metrics identifying active provider implementation per capability and edge integration mode (`off`/`advisory`/`authoritative`).
- [x] Document provider selection, rollout, and rollback procedures in deployment docs (including Akamai advisory/authoritative guidance and fallback-to-internal behavior).
- [x] H4.1 slice completed: formalized provider capability contracts in `src/providers/contracts.rs` (`RateLimiterProvider`, `BanStoreProvider`, `ChallengeEngineProvider`, `MazeTarpitProvider`, `FingerprintSignalProvider`) with stable enum labels and default-behavior regression tests.
- [x] H4.2 slice completed: added config-backed provider backend selection (`provider_backends` + `SHUMA_PROVIDER_*` defaults), plus `src/providers/registry.rs` factory/registry mapping (`internal`/`external`) with default internal selection and no behavior change to request handling paths.
- [x] H4.3 slice completed: implemented `Internal*` provider adapters in `src/providers/internal.rs` and routed core request/policy flow through registry-selected provider interfaces in `src/lib.rs` and `src/runtime/policy_pipeline.rs` (default behavior preserved under `internal` backends).
- [x] H4.4.1 slice completed: added `edge_integration_mode` posture (`off`/`advisory`/`authoritative`) to config/defaults and threaded it through runtime decision metadata plus metrics export (`bot_defence_edge_integration_mode_total`) without changing enforcement precedence.
- [x] H4.4.2 slice completed: added explicit `external` provider adapters in `src/providers/external.rs`; `fingerprint_signal` now routes to an external stub contract, while `challenge_engine` and `maze_tarpit` remain explicit unsupported adapter paths with safe fallback semantics.
- [x] H4.4.3 slice completed: added provider implementation observability with capability/backend/implementation metrics (`bot_defence_provider_implementation_effective_total`) and runtime event-tag provider summaries (`providers=...`) wired from registry-selected implementations.
- [x] H4.4.4 slice completed: added fingerprint provider contract availability semantics (`active`/`disabled`/`unavailable`) across internal/external adapters plus registry tests enforcing explicit unavailability behavior when external fingerprint is selected but not configured.
- [x] H4.4.5 slice completed: documented deployment personas plus provider selection matrix and added Akamai-focused advisory/authoritative rollout + rollback runbook with explicit fallback-to-internal procedure in [`docs/configuration.md`](../docs/configuration.md), [`docs/deployment.md`](../docs/deployment.md), and [`docs/observability.md`](../docs/observability.md).
- [x] H4.5 plan follow-up ([`docs/plans/2026-02-13-provider-externalization-design.md`](../docs/plans/2026-02-13-provider-externalization-design.md) step 3): replace external fingerprint stub with an Akamai-first adapter that maps edge/Bot Manager outcomes into normalized fingerprint signals.
- [x] H4.5 slice completed: external `fingerprint_signal` now uses an Akamai-first adapter (`/fingerprint-report`) that normalizes edge/Bot Manager-style outcomes into CDP-tier-compatible signals, retains explicit fallback to the internal CDP handler for non-Akamai/legacy payloads, and exports implementation label `external_akamai_with_internal_fallback`.
- [x] H4.6 plan follow-up ([`docs/plans/2026-02-13-provider-externalization-design.md`](../docs/plans/2026-02-13-provider-externalization-design.md) step 4): implemented external `rate_limiter` and `ban_store` adapters with distributed state/sync semantics and retired unsupported-stub behavior for those capabilities.
- [x] H4.6.1 slice completed: external `rate_limiter` now uses Redis-backed distributed counting (`INCR` + TTL) with explicit internal fallback and provider implementation labeling (`external_redis_with_internal_fallback`).
- [x] H4.6.2 slice completed: external `ban_store` now uses Redis-backed distributed ban state (JSON + TTL) with explicit internal fallback, provider implementation labeling (`external_redis_with_internal_fallback`), and admin ban/unban/list provider routing.
- [x] H4.7 plan follow-up ([`docs/plans/2026-02-13-provider-externalization-design.md`](../docs/plans/2026-02-13-provider-externalization-design.md) step 5): add integration tests for advisory vs authoritative mode precedence and explicit downgrade-to-internal behavior when external providers are unavailable.
- [x] H4.7 slice completed: admin config now supports validated `provider_backends` + `edge_integration_mode` updates, external fingerprint precedence is mode-aware (`off` ignore, `advisory` non-authoritative, `authoritative` strong-edge auto-ban), and integration coverage was added for advisory-vs-authoritative behavior plus external rate-limiter downgrade-to-internal fallback.
- [x] H4.7.1 UX follow-up completed: added an Admin Dashboard Config control for `edge_integration_mode` (`off`/`advisory`/`authoritative`) with save/dirty-state wiring and dashboard e2e smoke coverage so operators can stage and verify H4.7 precedence behavior without manual env edits.

- [x] Define a cutover checklist for enabling any external provider in non-dev environments (staging soak, SLOs, rollback trigger).

- [x] Define platform scope boundaries to avoid overreach by leaning on upstream bot managers (for example Akamai) for features better handled there.
- [x] Add non-secret runtime config export for deploy handoff (exclude secrets) so dashboard-tuned settings can be applied in immutable redeploys.
- [x] P3.1 slice completed: documented Akamai-vs-Shuma platform scope ownership boundaries, non-goals, and decision rules in [`docs/bot-defence.md`](../docs/bot-defence.md) to keep edge-vs-app responsibilities explicit.
- [x] S1.3.a slice completed: defined canonical request-sequence signal IDs (`S_SEQ_*`) and matching detection IDs (`D_SEQ_*`) in `src/runtime/policy_taxonomy.rs` and documented them in [`docs/plans/archive/2026-02-14-stage1-policy-signal-taxonomy-spec.md`](../docs/plans/archive/2026-02-14-stage1-policy-signal-taxonomy-spec.md).
- [x] S1.3.b slice completed: added signed operation-envelope primitives (`operation_id`, `flow_id`, `step_id`, `issued_at`, `expires_at`, `token_version`) for puzzle/PoW challenge seeds with shared integrity validation in `src/challenge/operation_envelope.rs`, enforced token parse-time validation before scoring paths, and added regression coverage.
- [x] S1.3.c slice completed: added binding/integrity primitives for signed challenge/PoW tokens (`ip_bucket`, `ua_bucket`, `path_class`) with shared request-binding validation, enforced mismatch handling on puzzle/PoW verification paths, and emitted canonical sequence mismatch taxonomy telemetry (`D_SEQ_BINDING_MISMATCH`, `S_SEQ_BINDING_MISMATCH`) instead of silent fallback.
- [x] S1.3.d slice completed: added ordering-window primitives (`step_index`, expected flow/step/index validation, and bounded step windows) for challenge submit and PoW verify paths, mapped order/window failures to canonical taxonomy transitions (`D_SEQ_ORDER_VIOLATION`, `S_SEQ_ORDER_VIOLATION`, `D_SEQ_WINDOW_EXCEEDED`, `S_SEQ_WINDOW_EXCEEDED`), and added deterministic coverage in challenge, envelope, and policy-taxonomy tests.
- [x] S1.3.e slice completed: added timing-threshold primitives (`min_step_latency`, `max_step_latency`, `max_flow_age`, cadence regularity windows/spread TTL) and wired enforcement to challenge submit + PoW verify flows with canonical timing taxonomy transitions (`D/S_SEQ_TIMING_TOO_FAST`, `D/S_SEQ_TIMING_TOO_REGULAR`, `D/S_SEQ_TIMING_TOO_SLOW`).
- [x] S1.3.f slice completed: added replay primitives for operation-level first-seen/duplicate/expired tracking with bounded TTL stores and mapped duplicate/expired operation reuse to canonical replay/expired transitions (`D/S_SEQ_OP_REPLAY`, `D/S_SEQ_OP_EXPIRED`) across challenge and PoW verification.
- [x] S1.3.g slice completed: threaded sequence transitions into policy telemetry (`bot_defence_policy_matches_total`, `bot_defence_policy_signals_total`) and taxonomy-annotated admin event outcomes for challenge submit and PoW sequence violation paths.
- [x] S1.3.h slice completed: added deterministic sequence correctness coverage for challenge/PoW/envelope flows, including valid progression, operation replay, stale expiry, reorder, binding mismatch, too-fast submissions, and too-regular cadence.
- [x] Stage 1 umbrella completion: request-sequence primitives are now end-to-end across taxonomy IDs, signed operation envelopes, binding, ordering windows, timing/replay primitives, telemetry wiring, and regression coverage.
- [x] AI-policy controls slice completed: added first-class admin config keys (`ai_policy_block_training`, `ai_policy_block_search`, `ai_policy_allow_search_engines`) and dashboard controls separate from robots-serving controls while preserving legacy robots-field compatibility.

### todos/todo.md (Stage 2 completion)

- [x] MZ-S1: Keep Stage 2 completion criteria internal-first (no external-provider dependency).
- [x] MZ-S2: Execute Stage 2 delivery order as `MZ-R0 -> MZ-R1 -> MZ-R2 -> MZ-R3 -> MZ-1 -> MZ-2 -> MZ-7 -> MZ-5 -> MZ-3 -> MZ-4 -> MZ-8 -> MZ-9 -> MZ-10 -> MZ-6`.
- [x] MZ-R0: Research-first hold gate accepted from [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md).
- [x] MZ-1 through MZ-10 completed (entropy rotation, signed traversal + replay, budgets, client checkpoint flow, polymorphic rendering, pluggable seed providers/refresh/metadata-only extraction, covert non-maze decoys, crawler simulation harness, botness + observability wiring, rollout/rollback runbook guidance, optional adaptive micro-PoW).


## todos/security-review.md

- [x] Event-log append race fixed (`85bff68`).
- [x] Panic on invalid bool env parsing fixed (`69603c5`).
- [x] Health endpoint spoofing risk hardened with strict trust gate plus optional secret (`163e0dc`).
- [x] Admin login brute-force gap fixed in-app (`add999d`), with deployment-layer guidance added (`40e120c`).
- [x] Unsanitized ban reason storage fixed with sanitization/truncation and dashboard escaping (`4b65e49`).
- [x] Per-request runtime config KV reads fixed with in-memory TTL cache (`09e0017`, docs `88155ab`).
- [x] Browser version parsing robustness improved for edge cases (`b44eeca`).
- [x] "Missing SameSite cookie" report assessed as false positive in current implementation.
- [x] Silent KV error suppression significantly reduced by logging critical write/delete failures (`393e0b1`); low-impact cases remain opportunistic cleanup.

## Additional completions (2026-02-14)

### todos/todo.md

- [x] R-FP-10 Review Li et al., "PathMarker: protecting web contents against inside crawlers" (Cybersecurity 2019) and map path/timing marker concepts to Shuma detection IDs.
- [x] R-RL-02 Review Kuzmanovic/Knightly, "Low-Rate TCP-Targeted DoS Attacks" (SIGCOMM 2003) and map low-rate adversary behaviors to Shuma tarpit/limiter heuristics.
- [x] R-RL-04 Review Veroff et al., "Defense techniques for low-rate DoS attacks against application servers" (Computer Networks 2010) and identify bounded-randomization strategies usable in Shuma tarpit controls.
- [x] R-RL-08 Review Vedula et al., "On the Detection of Low-Rate Denial of Service Attacks at Transport and Application Layers" (Electronics 2021) and map detector candidates to Shuma observability/tuning.
- [x] R-SSH-01 Review Vasilomanolakis et al., "Gotta catch 'em all: A Multistage Framework for Honeypot Fingerprinting" (Digital Threats 2023) and derive anti-fingerprint requirements for SSH tarpit realism.
- [x] MZ-R1: Complete and summarize the highest-impact Maze/Tarpit research items (`R-FP-10`, `R-RL-02`, `R-RL-04`, `R-RL-08`, `R-SSH-01`) with concrete anti-fingerprinting and bounded-cost implications.
- [x] MZ-R2: Map research outcomes to `self_hosted_minimal` vs `enterprise_akamai` ownership and explicitly define what remains internal-first for Stage 2.
- [x] MZ-R3: Convert research findings into enforceable implementation guardrails (budget caps, replay windows, fallback policy, rollout abort thresholds) and update Stage 2 acceptance criteria before coding.

## Additional completions (2026-02-15)

### todos/todo.md (Stage 2.5 completion)

- [x] MZ-X0.R through MZ-X10.R completed via Stage 2.5 research synthesis memo in [`/docs/research/archive/2026-02-15-stage2.5-maze-efficiency-and-asymmetry.md`](../docs/research/archive/2026-02-15-stage2.5-maze-efficiency-and-asymmetry.md).
- [x] MZ-X0.I completed: Web Worker-first client expansion now uses compact signed seed bootstrap with deterministic fallback behavior when worker/proof cannot complete.
- [x] MZ-X1.I + MZ-X5.I completed: exact path commitment, chain marker checks, sibling edge-operation uniqueness, replay enforcement, and branch-budget-aware progressive issuance checks.
- [x] MZ-X2.I + MZ-X9.I completed: compact maze shell with external versioned shared assets and adaptive styling tiers (full/lite/machine, optional no-CSS deep tier).
- [x] MZ-X3.I completed: hidden links are no longer shipped in bootstrap payload; links are issued progressively via proof/checkpoint-gated `/maze/issue-links`.
- [x] MZ-X4.I + MZ-X6.I completed: proactive pre-render budget/degrade controls and bounded host-write behavior were implemented to reduce per-hop synthesis pressure.
- [x] MZ-X7.I completed: deterministic maze asymmetry benchmark harness + CI gate added (`make test-maze-benchmark`, included in `make test`) with regression-threshold enforcement.
- [x] MZ-X8.I completed: deep-tier micro-PoW and link expansion compute moved off main thread with constrained-device safeguards.
- [x] MZ-X10.I completed: high-confidence violation accumulation now triggers deterministic early fallback before expensive maze serving continues.

## Additional completions (2026-02-15, section-preserving archive)

### todos/todo.md

#### P3 Dashboard Architecture Modernization (Tabbed SPA, Frameworkless-First)
- [x] DSH-4 completed: shared dashboard API client layer added with typed request/response adapters and centralized API error handling (`dashboard/modules/api-client.js`).
- [x] DSH-5 completed: shared dashboard state primitives added with explicit invalidation scopes and tab-local derived state (`dashboard/modules/dashboard-state.js`).
- [x] DSH-6 completed: CDN chart dependency removed; local pinned chart runtime vendored under `dashboard/assets/vendor/chart-lite-1.0.0.min.js` with provenance note in docs.
- [x] DSH-7 completed: active-tab scoped polling added with deterministic suspend/resume and bounded timer count.
- [x] DSH-8 completed: tab accessibility/keyboard behavior strengthened (ARIA visibility semantics, focus management, selected-state behavior).
- [x] DSH-9 completed: progressive `// @ts-check` typing enabled across dashboard modules and orchestration.
- [x] DSH-10 completed: per-tab loading/empty/error states implemented for silent-failure resistance.
- [x] DSH-11 completed: Playwright e2e coverage expanded for tabbed routing, keyboard navigation, and tab error-state surfacing.
- [x] DSH-12 completed: dashboard module unit-style tests added for API adapters, state invalidation, and tab normalization (`e2e/dashboard.modules.unit.test.js`).
- [x] DSH-13 completed: public docs updated (`README.md`, [`docs/dashboard.md`](../docs/dashboard.md), [`docs/testing.md`](../docs/testing.md)) for tab model and dashboard test workflow.
- [x] DSH-14 completed: migration/rollback notes added to public dashboard docs.
- [x] DSH-G1 closure: framework-adoption gate did not trip after DSH-1..DSH-14; Lit pilot deferred.

## Additional completions (2026-02-15, section-preserving archive)

### todos/todo.md

#### Fingerprinting, JS Verification, and CDP-Adjacent Detection
- [x] R-FP-10 Review Li et al., "PathMarker: protecting web contents against inside crawlers" (Cybersecurity 2019) and map path/timing marker concepts to Shuma detection IDs. https://cybersecurity.springeropen.com/articles/10.1186/s42400-019-0023-1 (summarized in [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md))

#### Rate Limiting, Tarpit, and Cost-Imposition
- [x] R-RL-02 Review Kuzmanovic/Knightly, "Low-Rate TCP-Targeted DoS Attacks" (SIGCOMM 2003) and map low-rate adversary behaviors to Shuma tarpit/limiter heuristics. https://doi.org/10.1145/863955.863966 (summarized in [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md))
- [x] R-RL-04 Review Veroff et al., "Defense techniques for low-rate DoS attacks against application servers" (Computer Networks 2010) and identify bounded-randomization strategies usable in Shuma tarpit controls. https://doi.org/10.1016/j.comnet.2010.05.002 (summarized in [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md))
- [x] R-RL-08 Review Vedula et al., "On the Detection of Low-Rate Denial of Service Attacks at Transport and Application Layers" (Electronics 2021) and map detector candidates to Shuma observability/tuning. https://doi.org/10.3390/electronics10172105 (summarized in [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md))

#### SSH Tarpit and Honeypot Evasion Resistance
- [x] R-SSH-01 Review Vasilomanolakis et al., "Gotta catch 'em all: A Multistage Framework for Honeypot Fingerprinting" (Digital Threats 2023) and derive anti-fingerprint requirements for SSH tarpit realism. https://doi.org/10.1145/3584976 (summarized in [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md))

#### Stage 1: Policy and signal prerequisites
- [x] Add request-sequence signal primitives end-to-end (canonical `S_SEQ_*`/`D_SEQ_*` taxonomy IDs, signed operation envelope fields, binding checks, ordering windows, timing thresholds, replay detection, telemetry wiring, and deterministic/integration coverage).
- [x] S1.3.e Add timing-threshold primitives (min-step-latency, max-step-latency, cadence-regularity threshold, max-flow-age) with conservative defaults tuned for low human false positives.
- [x] S1.3.f Add replay primitives (first-seen/duplicate/expired operation tracking with bounded TTL stores) and map duplicate/reused operations into canonical replay signals.
- [x] S1.3.g Thread sequence signals into botness/policy telemetry (`bot_defence_policy_signals_total`, taxonomy-annotated admin outcomes) and define escalation semantics for advisory vs enforce paths.
- [x] S1.3.h Add deterministic tests for sequence correctness (valid progression, reorder, replay, stale window, too-fast/too-regular cadence, binding mismatch) plus integration coverage for JS/PoW/challenge flows.
- [x] Add AI-bot policy controls as first-class admin config (separate from robots-only controls).

#### Stage 2: Maze excellence execution (Cloudflare-inspired, Shuma-native)
- [x] MZ-S1: Keep Stage 2 completion criteria internal-first (no external-provider dependency).
- [x] MZ-S2: Execute Stage 2 delivery order as `MZ-R0 -> MZ-R1 -> MZ-R2 -> MZ-R3 -> MZ-1 -> MZ-2 -> MZ-7 -> MZ-5 -> MZ-3 -> MZ-4 -> MZ-8 -> MZ-9 -> MZ-10 -> MZ-6`.
- [x] MZ-R0: Research-first hold gate. Do not start Stage 2 implementation slices until the Maze/Tarpit research tranche is synthesized and accepted. (accepted research baseline in [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md))
- [x] MZ-R1: Complete and summarize the highest-impact Maze/Tarpit research items (`R-FP-10`, `R-RL-02`, `R-RL-04`, `R-RL-08`, `R-SSH-01`) with concrete anti-fingerprinting and bounded-cost implications. ([`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md))
- [x] MZ-R2: Map research outcomes to `self_hosted_minimal` vs `enterprise_akamai` ownership and explicitly define what remains internal-first for Stage 2. ([`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md))
- [x] MZ-R3: Convert research findings into enforceable implementation guardrails (budget caps, replay windows, fallback policy, rollout abort thresholds) and update Stage 2 acceptance criteria before coding. ([`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](../docs/research/2026-02-14-maze-tarpit-research-synthesis.md))
- [x] MZ-1: Replace path-only deterministic seeding with rotating signed entropy for suspicious traffic; keep short TTL deterministic windows for cacheability/debugging.
- [x] MZ-2: Add signed traversal-link tokens with TTL, depth scope, branch budget, and replay protection.
- [x] MZ-7: Enforce maze cost budgets (global concurrency, per-bucket spend, response byte/time caps) with deterministic fallback behavior.
- [x] MZ-5: Make client-side expansion foundational for suspicious maze tiers (Web Worker branch generation + signed server verification) with explicit checkpoint cadence (every 3 nodes or 1500 ms), bounded step-ahead allowance, and no-JS fallback rules.
- [x] MZ-3: Add polymorphic maze rendering (layout/content/link-graph variant families with versioned selection).
- [x] MZ-3.1: Implement pluggable maze content-seed providers (internal default corpus + operator-provided source adapters).
- [x] MZ-3.2: Add manual/scheduled seed refresh for provider-fed corpora with robots/compliance guardrails, caching, and rate limits.
- [x] MZ-3.3: Enforce metadata/keyword-first extraction (avoid article-body copying) to reduce legal risk, bandwidth, and fingerprintability.
- [x] MZ-4: Inject covert decoys into eligible non-maze HTML responses for medium-confidence suspicious traffic while preserving UX/SEO safety.
- [x] MZ-8: Add a crawler simulation harness covering replay, deterministic fingerprinting attempts, JS/no-JS cohorts, and bypass attempts.
- [x] MZ-9: Feed maze traversal behavior into botness scoring/detection IDs and add observability for entropy/token/proof/cost/budget signals.
- [x] MZ-10: Roll out by phase (`instrument -> advisory -> enforce`) with explicit rollback triggers and operator runbook checks.
- [x] MZ-6: Add optional adaptive micro-PoW for deeper traversal tiers.

#### Stage 2 follow-up: Operator-safe Maze Preview
- [x] MZ-PV1: Add an admin-auth-only maze preview endpoint (`GET /admin/maze/preview`) so operators can inspect maze rendering before serving it.
- [x] MZ-PV2: Ensure preview output is non-operational by design (no live traversal tokens, no hidden decoy tracking links, no replay/checkpoint/budget side effects, and no maze hit/risk counter mutation).
- [x] MZ-PV3: Isolate preview entropy/signing from live trap flow (`SHUMA_MAZE_PREVIEW_SECRET` with safe fallback) so leaked preview artifacts cannot forge production traversal.
- [x] MZ-PV4: Add dashboard UX affordance in the Maze config pane ("Preview Maze") that opens the admin preview safely and clearly indicates auth/session requirements.
- [x] MZ-PV5: Add deterministic tests for preview safety guarantees (route auth/read-only behavior, no-live-token markers, and no state mutation) and update docs/API references.

#### Stage 2.5 follow-up: Maze excellence shortfall closure (research-first)
- [x] MZ-X0.R Research optimal client-side branch generation architecture (Web Worker-first, compact signed seed bootstrap, verification cadence) using current state-of-the-art anti-bot/anti-fingerprinting references; publish decision memo with host-cost, attacker-cost, and UX tradeoffs.
- [x] MZ-X0.I Implement Web Worker-based branch generation from compact signed seed for suspicious maze tiers, with server verification protocol and deterministic fallback when worker/proof path fails.
- [x] MZ-X1.R Research optimal signed traversal-token semantics (path commitment granularity, operation-id uniqueness, chain integrity, replay windows, branch budget accounting) and select robust envelope design.
- [x] MZ-X1.I Enforce exact per-link path commitment and full chain constraints in runtime token validation (including `branch_budget` and `prev_digest`) with deterministic fallback and compatibility migration.
- [x] MZ-X2.R Research low-bandwidth maze response delivery patterns (static shell + versioned assets, compression, cache partitioning, anti-fingerprint constraints under no-store policy) and choose target payload budget.
- [x] MZ-X2.I Replace per-hop full inline HTML/CSS/JS with a compact shell + reusable static assets where safe, while preserving deception variability and no-index semantics; include explicit hashed asset/version strategy and cache policy acceptance criteria (for example immutable long-cache static assets with controlled cache-busting on deploy).
- [x] MZ-X3.R Research concealed link-delivery strategies that preserve attacker cost asymmetry (progressive on-demand expansion, encrypted/obfuscated manifests, proof-gated link issuance) without obvious giveaway markers.
- [x] MZ-X3.I Stop shipping the full hidden-link set in bootstrap JSON; move to proof/checkpoint-gated progressive link issuance so bandwidth and traversal state are requester-amortized.
- [x] MZ-X4.R Research host-cost minimization strategies for per-hop maze serving (pre-generation pools, fragment caches, bounded KV write coalescing, lazy state persistence) and pick target CPU/write budgets.
- [x] MZ-X4.I Reduce per-hop host synthesis and write cost by implementing selected caching/pre-generation/write-coalescing strategy with hard budget guardrails.
- [x] MZ-X5.R Research operation-id construction and sibling-token uniqueness patterns to prevent cross-link token reuse and branch-collapse artifacts.
- [x] MZ-X5.I Issue unique child tokens per link edge (operation/path-bound), enforce single-edge replay semantics, and add regression tests for sibling traversal correctness.
- [x] MZ-X6.R Research proactive overload controls for deception systems (pre-render admission control, queue/latency-aware throttles, deterministic degrade ladders) to avoid post-render-only cap enforcement.
- [x] MZ-X6.I Add pre-render admission and deterministic degrade controls so byte/time caps are enforced proactively, not only after render cost is incurred.
- [x] MZ-X7.R Research measurable attacker-vs-defender cost models for maze/tarpit systems (CPU, bandwidth, latency, energy) and define project SLO/SLA thresholds and acceptance tests.
- [x] MZ-X7.I Add repeatable benchmark harness + CI gates that report host and attacker-side cost deltas and fail regressions against defined asymmetry targets.
- [x] MZ-X8.R Research client-side compute fairness controls (battery/thermal sensitivity, mobile safeguards, main-thread impact) for deep-tier micro-PoW and JS expansion.
- [x] MZ-X8.I Move deep-tier proof and expansion compute fully off the main thread, add adaptive safeguards for constrained clients, and validate no significant human UX regression.
- [x] MZ-X9.R Research deception-page styling minimalism: quantify anti-fingerprint benefit vs byte/energy cost of CSS, determine when no-CSS is safe, and define tiered styling policy by botness confidence + traversal depth.
- [x] MZ-X9.I Implement adaptive maze styling tiers: minified external shared stylesheet at low/medium suspicion, ultra-minimal style at high suspicion, and optional no-CSS templates at high-confidence deep traversal before ban threshold; tier selection must key on botness score + traversal depth + violation history, and no-CSS variants must remain plausible machine-oriented surfaces (not obviously broken or synthetic giveaway pages).
- [x] MZ-X10.R Research confidence-accumulation escalation models for deception systems (stacked violation semantics, false-positive controls, rollback criteria) to stop expensive maze serving earlier without premature giveaway.
- [x] MZ-X10.I Add pre-ban high-confidence early-escalation matrix (for example replay + binding mismatch + checkpoint/proof failures) that deterministically degrades from maze serving to lower-cost challenge/block actions.

## Additional completions (2026-02-16, section-preserving archive)

### todos/todo.md

#### Stage 2.7 follow-up: Honeypot + Maze stealth excellence (research-first, pre-launch no-compat mode)
- [x] MZ-SR1 Publish a current research synthesis for stealth deception routing and honeypot fingerprinting resistance with explicit source mapping and implementation requirements. ([`docs/research/2026-02-16-honeypot-maze-stealth-excellence.md`](../docs/research/2026-02-16-honeypot-maze-stealth-excellence.md))
- [x] MZ-S1 Remove explicit `/trap/*` route handling and trap-specific robots bait comments; keep deception routes non-semantic and reduce immediate classifier signal.
- [x] MZ-S2 Introduce an opaque, deployment-specific maze route namespace (secret-derived prefix) and route helper API; remove remaining public `/maze/*` labels from live routing paths.
- [x] MZ-S3 Move maze support endpoints (`checkpoint`, `issue-links`) and versioned maze assets under the same opaque namespace and update worker/bootstrap generation to consume helper paths only.
- [x] MZ-S4 Remove deception-path disclosure from `robots.txt` defaults (no explicit maze/trap path disallow lines or bait comments); keep robots focused on crawler policy communication, not trap advertisement.
- [x] MZ-S5 Update admin preview + dashboard links to use runtime path helpers so preview reflects live namespace while staying non-operational.
- [x] MZ-S6 Add regression tests for route stealth and canonicalization (slash variants, malformed prefixes, old explicit-path rejection) across unit/integration paths.
- [x] MZ-S7 Refresh public docs ([`docs/maze.md`](../docs/maze.md), [`docs/api.md`](../docs/api.md), [`docs/configuration.md`](../docs/configuration.md), `README.md`/[`docs/quick-reference.md`](../docs/quick-reference.md)) to describe the new opaque routing model and operator expectations.
- [x] MZ-S8 Re-run benchmark and verification gates (`make test`, `make build`) and record resource/behavior deltas for stealth migration.
  Verification notes (2026-02-16): `make test` passed end-to-end (unit + benchmark + integration + dashboard e2e), `make build` passed, and maze benchmark gate reported `pages=6 avg_page_bytes=6638 host_set_ops=46 host_write_bytes=511 attacker_requests=16 issue_links_calls=5 attacker_pow_iterations=3553`.

#### Direction Snapshot (for next implementation stages)
- [x] Evolve maze behavior toward Cloudflare-style selective covert decoys for suspicious traffic with opaque, non-semantic route namespaces (no explicit `/maze` or `/trap` public labels).

#### P3 Dashboard Architecture Modernization (Tabbed SPA, Frameworkless-First)
##### Baseline and decision gate
- [x] DSH-R1 Baseline current dashboard architecture and runtime costs (JS/CSS bytes, startup time, memory, polling cadence, bundle provenance, current e2e coverage) and publish a short decision memo in `docs/plans/`.
- [x] DSH-R2 Evaluate two implementation tracks against Shuma constraints: (A) frameworkless modular SPA + JSDoc typing, (B) ultra-light framework (Lit) with equivalent tab shell; include explicit tradeoffs for maintenance, DX, runtime weight, and migration risk.
- [x] DSH-R3 Define framework-adoption gate criteria (for example: unresolved lifecycle complexity, repeated DOM/state bugs, unacceptable change lead time after frameworkless refactor); default to no framework unless gate is tripped.

##### Tabbed SPA shell and structure (frameworkless path)
- [x] DSH-1 Implement tabbed SPA shell in `dashboard/index.html` + `dashboard/dashboard.js` with canonical tabs: `Monitoring`, `IP Bans`, `Status`, `Config`, `Tuning`.
- [x] DSH-2 Add URL-backed tab routing (`#monitoring`, `#ip-bans`, `#status`, `#config`, `#tuning`) with refresh-safe deep links and history navigation.
- [x] DSH-3 Refactor monolithic dashboard orchestration into tab-scoped controllers/modules with clear lifecycle (`init`, `mount`, `unmount`, `refresh`) and no cross-tab hidden coupling.

#### Fingerprinting, JS Verification, and CDP-Adjacent Detection
- [x] R-FP-01 Review Peter Eckersley, "How Unique Is Your Web Browser?" (PETS 2010) and extract entropy-design implications for Shuma fingerprint signals and replay windows. https://link.springer.com/chapter/10.1007/978-3-642-14527-8_1 (summarized in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md))
- [x] R-FP-02 Review Acar et al., "The Web Never Forgets" (CCS 2014) and derive tracking/fingerprint abuse patterns relevant to bot-detection evasion hardening. https://doi.org/10.1145/2660267.2660347 (summarized in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md))
- [x] R-FP-03 Review Vastel et al., "FP-STALKER" (IEEE S&P 2018) and define time-evolution checks for Shuma fingerprint consistency logic. https://doi.org/10.1109/SP.2018.00008 (summarized in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md))
- [x] R-FP-04 Review Jonker/Krumnow/Vlot, "Fingerprint Surface-Based Detection of Web Bot Detectors" (ESORICS 2019) and identify detector-surface minimization requirements. https://doi.org/10.1007/978-3-030-29962-0_28 (summarized in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md))
- [x] R-FP-05 Review Azad et al., "Web Runner 2049: Evaluating Third-Party Anti-bot Services" and extract anti-evasion architecture lessons for internal-vs-edge integration boundaries. https://pmc.ncbi.nlm.nih.gov/articles/PMC7338186/ (summarized in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md))
- [x] R-FP-06 Review Iliou et al., "Detection of advanced web bots by combining web logs with mouse behavioural biometrics" (DTRAP 2021) and assess feasibility of low-friction behavior features in Shuma. https://doi.org/10.1145/3447815 (summarized in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md))
- [x] R-FP-07 Review Zhao et al., "Toward the flow-centric detection of browser fingerprinting" (Computers & Security 2024) and evaluate flow-level JS signal extraction options. https://doi.org/10.1016/j.cose.2023.103642 (summarized in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md))
- [x] R-FP-08 Review Venugopalan et al., "FP-Inconsistent: Detecting Evasive Bots using Browser Fingerprint Inconsistencies" (2024) and define cross-attribute consistency checks for Shuma scoring. https://arxiv.org/abs/2406.07647 (summarized in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md))
- [x] R-FP-09 Review Bursztein et al., "Picasso: Lightweight Device Class Fingerprinting for Web Clients" (SPSM 2016) and assess replay-resistant challenge-bound fingerprint options. https://doi.org/10.1145/2994459.2994467 (summarized in [`docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md`](../docs/research/archive/2026-02-16-fingerprinting-research-synthesis.md))
- [x] Strengthen fingerprinting by hardening internal baseline signals first, then ingesting trusted upstream edge signals (JA3/JA4 and similar) with provenance checks and explicit internal fallback when edge headers are absent or untrusted.
- [x] Phase 1 completed: normalized fingerprint signals now carry provenance/confidence metadata, family entropy budgeting/caps are enforced, and data-minimization controls (TTL/pseudonymization/export visibility) are wired and documented.
- [x] Phase 2 completed: cross-layer mismatch heuristics (UA/client-hint/transport), temporal coherence detection IDs, and bounded flow-window fingerprint telemetry are active.
- [x] Phase 3 completed: versioned CDP probe-family rotation (`v1`/`v2`/`split`) is active, trusted transport-header ingestion is implemented, persistence-abuse signals are emitted, challenge-bound short-lived marker checks are wired, and low-friction micro-signal checks are added with conservative weighting.
- [x] Phase 4 completed (except Finch spike): fingerprint-focused admin visibility/tuning surfaces are shipped (`/admin/cdp` config + `fingerprint_stats`, dashboard cards), and evasive-regression coverage was added for detector variation, temporal drift, and inconsistency bypass classes.

## Additional completions (2026-02-17, section-preserving archive)

### todos/todo.md

#### P3 Dashboard Architecture Modernization (Frameworkless-First)
- [x] DSH-ARCH-1 Add shared dashboard core utilities: `core/format.js` (escaping + numeric/date helpers + shallow equality) and `core/dom.js` (DOM cache + safe setters + write scheduler), then consume them from feature modules.
- [x] DSH-ARCH-2 Consolidate writable config path inventories into a single `config-schema.js` source and consume it from both Status inventory rendering and Advanced Config template generation.
- [x] DSH-ARCH-3 Replace fragmented per-pane saved-state bags with a single `config-draft-store` baseline (`get/set/isDirty`) used by config dirty-check paths.
- [x] DSH-ARCH-4 Reduce config bind coupling by switching `config-controls.bind(...)` callsites to a typed `context` object and adding normalization coverage in dashboard module unit tests.
- [x] DSH-ARCH-5 Add render-performance guards: skip chart redraws when data/labels are unchanged and batch refresh-driven DOM writes through one scheduler cycle.
- [x] DSH-ARCH-6 Remove uncached hot-path `getElementById` usage from `dashboard.js` and `config-controls.js` by routing lookups through shared DOM cache helpers.

#### P3 Dashboard Native ESM + Functional JS Modernization (No Build Step)
- [x] DSH-ESM-1 Hard cutover selected for pre-launch: migrate dashboard JS to native ESM without dual global-script wiring; decision captured during the native ESM cutover tranche.
- [x] DSH-ESM-2 Freeze behavior contracts to preserve during refactor: tab routing/hash behavior, API payload expectations, status/config control semantics, and monitoring render states. ([`docs/plans/2026-02-17-dashboard-esm-behavior-contracts.md`](../docs/plans/2026-02-17-dashboard-esm-behavior-contracts.md))
- [x] DSH-ESM-3 Add/expand regression coverage before migration for all dashboard tabs (`loading`/`empty`/`error`/`data`) and critical config dirty-state/save flows. (`e2e/dashboard.smoke.spec.js`, `e2e/dashboard.modules.unit.test.js`)
- [x] DSH-ESM-4 Introduce a single native module entrypoint (`<script type="module">`) and convert dashboard boot from global-init order to explicit imports.
- [x] DSH-ESM-5 Replace `window.ShumaDashboard*` global module registry wiring with ESM `export`/`import` contracts across dashboard modules.
- [x] DSH-ESM-6 Define and enforce a stable module graph (`core` -> `services` -> `features` -> `main`) with no circular imports. ([`docs/plans/2026-02-17-dashboard-esm-module-graph.md`](../docs/plans/2026-02-17-dashboard-esm-module-graph.md) + module-graph guard test)
- [x] DSH-ESM-7 Refactor feature modules to functional boundaries: pure `deriveViewModel(snapshot, options)` and side-effectful `render(viewModel, effects)`; no class-based state.
- [x] DSH-ESM-8 Centralize side effects in dedicated effect adapters (DOM writes, network calls, clipboard, timers) so feature logic remains pure/testable. (`dashboard/modules/services/runtime-effects.js`)
- [x] DSH-ESM-9 Consolidate dashboard state updates around immutable transition functions (`nextState = reduce(prevState, event)`) and remove ad-hoc mutable globals where possible.
- [x] DSH-ESM-10 Standardize function style for new/changed dashboard code: default parameter values, arrow functions for local/pure helpers and callbacks, and explicit named function declarations only where hoisting/readability is clearly beneficial.
- [x] DSH-ESM-11 Remove legacy IIFE wrappers and duplicate helper code paths that were only needed for global-script loading.
- [x] DSH-ESM-12 Add lightweight static guard checks for dashboard JS (for example: fail on new `window.ShumaDashboard*` exports, fail on `class` usage in dashboard modules, fail on duplicate helper definitions across modules).
- [x] DSH-ESM-13 Execute migration in small slices with mandatory full verification per slice via Makefile (`make test` with dev Spin running).
- [x] DSH-ESM-14 Update public and contributor docs ([`docs/dashboard.md`](../docs/dashboard.md), architecture plan, contributor notes) with native ESM conventions, functional patterns, and module-boundary rules.
- [x] DSH-ESM-15 Run a final no-net-behavior-change audit against baseline contracts and capture known intentional deltas (if any) before merge. ([`docs/plans/2026-02-17-dashboard-esm-no-net-behavior-audit.md`](../docs/plans/2026-02-17-dashboard-esm-no-net-behavior-audit.md))

## Additional completions (2026-02-17, section-preserving archive)

### todos/todo.md

#### P3 Dashboard Modernization Follow-up (Functional + ESM Refinement)
- [x] DSH-FUP-1 Replace repeated config save-handler boilerplate in `dashboard/modules/config-controls.js` with shared functional helpers (save-state transitions, status message helpers, and error-path normalization) while preserving exact button labels/dirty-check timing.
- [x] DSH-FUP-2 Consolidate repeated `check*ConfigChanged` patterns in `dashboard/dashboard.js` into a schema-driven dirty-check registry + generic evaluator to reduce copy-paste state logic and event binding drift.
- [x] DSH-FUP-3 Replace repeated `configUiState` wrapper functions in `dashboard/dashboard.js` with a dispatch/invoke helper so config snapshot refresh is declarative and less error-prone.
- [x] DSH-FUP-4 Refactor tab refresh orchestration into a tab-handler map (including shared config-backed tabs) instead of if/else branching for `status`/`config`/`tuning`.
- [x] DSH-FUP-5 Add a shared status-panel patch helper to coalesce `statusPanel.update(...)` + `statusPanel.render()` across dashboard modules and remove duplicate render-trigger code paths.
- [x] DSH-FUP-6 Move monitoring loading placeholder reset logic out of `dashboard/dashboard.js` and into `dashboard/modules/monitoring-view.js` so monitoring rendering state is feature-owned.
- [x] DSH-FUP-7 Reduce `configControls.bind(...)` coupling by replacing the broad callback bag with a focused domain API object (typed by shape and covered by module tests).
- [x] DSH-FUP-8 Replace inline style mutations for shadow-mode visual state with semantic classes/CSS tokens and add coverage to prevent style regressions.
- [x] DSH-FUP-9 Expand dashboard save-flow test coverage for robots serving, AI policy, GEO scoring/routing, CDP config, and botness config to catch regressions that unit adapter tests miss.

## Additional completions (2026-02-17, section-preserving archive)

### todos/todo.md

#### P3 Dashboard Functional Excellence Remediation (Post-Review)
- [x] DSH-FEX-1 Remove global `window.fetch` monkey patching from admin session flow and move CSRF/session write handling into explicit request paths (idempotent, no global side effects).
- [x] DSH-FEX-2 Harden dashboard boot with safe DOM-binding guards for optional/missing elements so markup drift cannot crash initialization.
- [x] DSH-FEX-3 Refactor status rendering to instance-based state (`create(...)`) rather than module-level mutable singleton state.
- [x] DSH-FEX-4 Decompose `config-controls.bind(...)` orchestration into declarative save-handler wiring primitives to reduce mixed concerns and imperative branching.
- [x] DSH-FEX-5 Improve DOM cache semantics to avoid stale/null permanence (re-resolve disconnected or previously missing nodes) with focused unit coverage.
- [x] DSH-FEX-6 Reduce config-control coupling by replacing the monolithic `domainApi` callback bag with smaller capability namespaces and compatibility tests.
- [x] DSH-FEX-7 Add regression coverage for: session-auth write CSRF behavior, missing-control boot resilience, and status instance isolation.

## Additional completions (2026-02-18, section-preserving archive)

### todos/todo.md

#### P0 Dashboard SvelteKit Full Cutover (All Tabs, Excellence Architecture)
- [x] DSH-SVLT-R0 Record architecture decision for SvelteKit full cutover and supersede the prior framework migration direction ([`docs/adr/0002-dashboard-sveltekit-cutover.md`](../docs/adr/0002-dashboard-sveltekit-cutover.md)).
- [x] DSH-SVLT-R1 Preserve route and behavior contracts (`/dashboard/index.html`, `/dashboard/login.html`, hash-tab UX) during migration.
- [x] DSH-SVLT-R2 Keep deployment static-only (adapter-static + Spin fileserver), with no Node server in production runtime.
- [x] DSH-SVLT-PLAT1 Add SvelteKit app scaffolding under `dashboard/` with static adapter output to `dist/dashboard`.
- [x] DSH-SVLT-PLAT2 Wire `spin.toml` dashboard static source to `dist/dashboard`.
- [x] DSH-SVLT-PLAT3 Add canonical dashboard build integration to `make dev`, `make run`, and `make build`.
- [x] DSH-SVLT-UI1 Move dashboard/login page shells into Svelte routes while preserving exact design and DOM IDs.
- [x] DSH-SVLT-LIFE1 Introduce explicit Svelte route lifecycle bridges that mount legacy dashboard/login runtimes.
- [x] DSH-SVLT-LIFE2 Keep local chart runtime vendored and loaded from static assets under the SvelteKit base path.
- [x] DSH-SVLT-NEXT1 Replace legacy runtime bridge with Svelte-native store/actions for tab lifecycle, polling, and session/auth state.
- [x] DSH-SVLT-NEXT1.1 Add centralized dashboard store module (`state`, `actions`, `selectors`) for active tab, auth/session, tab status (loading/error/empty), snapshots, and stale flags.
- [x] DSH-SVLT-NEXT1.2 Add explicit effect adapters for network, timers, history/hash writes, and page-visibility events; forbid direct effect calls from UI components.
- [x] DSH-SVLT-NEXT1.3 Replace hash/tab behavior from legacy coordinator with Svelte-owned tab action pipeline (`activateTab`, keyboard nav, hash sync, reload persistence).
- [x] DSH-SVLT-NEXT1.4 Add Svelte-owned polling scheduler with per-tab cadence (`30s/45s/60s`) and visibility pause/resume semantics matching current behavior.
- [x] DSH-SVLT-NEXT1.5 Add Svelte-owned auth/session bootstrap (`/admin/session` check, login redirect, logout action, csrf token propagation).
- [x] DSH-SVLT-NEXT1.6 Move config dirty-state tracking from legacy runtime into store-level draft baselines and section-local derived selectors.
- [x] DSH-SVLT-NEXT1.7 Gate legacy bridge boot behind a migration toggle and switch default path to Svelte-native store/actions once parity tests pass.
- [x] DSH-SVLT-NEXT2 Split monitoring/ip-bans/status/config/tuning into dedicated Svelte component trees with declarative rendering.
- [x] DSH-SVLT-NEXT2.1 Create shared Svelte UI primitives for tab state messages, stat cards, table wrappers, and empty/loading/error blocks.
- [x] DSH-SVLT-NEXT2.2 Implement Monitoring component tree (cards, charts, events table, monitoring summaries, Prometheus helper) using declarative rendering only.
- [x] DSH-SVLT-NEXT2.3 Implement IP Bans component tree (ban table, quick-unban interactions, row-detail expansion) with store-driven actions.
- [x] DSH-SVLT-NEXT2.4 Implement Status component tree (status cards + runtime variable inventory tables) with shared schema-driven metadata.
- [x] DSH-SVLT-NEXT2.5 Implement Config component tree split by concern (maze, robots/ai policy, geo, honeypot, browser policy, bypass lists, challenge/pow, cdp, edge mode, advanced JSON).
- [x] DSH-SVLT-NEXT2.6 Implement Tuning component tree (botness thresholds/weights/status blocks) with the same save/dirty architecture as Config.
- [x] DSH-SVLT-NEXT2.7 Migrate chart lifecycle management into Svelte-friendly adapters (`onMount`/`onDestroy`, no global chart instance leaks).
- [x] DSH-SVLT-NEXT2.8 Complete no-net-behavior parity pass against current smoke contracts for all five tabs before deleting legacy path.
- [x] DSH-SVLT-NEXT3 Remove legacy shell source files once Svelte-native component parity is complete.
- [x] DSH-SVLT-NEXT3.1 Remove shell fragment injection path (`src/lib/shell/*.html` + `{@html ...}`) after Svelte-native component parity is complete.
- [x] DSH-SVLT-NEXT3.2 Remove bridge modules (`src/lib/bridges/*.js`) and legacy runtime boot globals once no longer referenced.
- [x] DSH-SVLT-NEXT3.3 Remove or archive superseded legacy dashboard entry shell dependencies (`dashboard/index.html`, `dashboard/login.html`) from active runtime path.
- [x] DSH-SVLT-NEXT3.4 Remove unused legacy orchestration modules from active dependency graph and keep only reusable domain adapters.
- [x] DSH-SVLT-NEXT3.5 Add static guardrails preventing reintroduction of bridge-era anti-patterns (`{@html}` shell injection, route-level legacy runtime imports).
- [x] DSH-SVLT-TEST1 Add targeted tests for Svelte route bridge lifecycle (single-mount guarantees, duplicate listener prevention, teardown behavior).
- [x] DSH-SVLT-TEST1.1 Add unit tests for single-mount guarantees when route is revisited (no duplicate listeners/timers/intervals).
- [x] DSH-SVLT-TEST1.2 Add unit tests for teardown behavior on route unmount (listener cleanup, polling stop, chart cleanup).
- [x] DSH-SVLT-TEST1.3 Add unit tests for auth/session bootstrap transitions (`authenticated`, `unauthenticated`, `expired`) in Svelte-native path.
- [x] DSH-SVLT-TEST1.4 Add unit tests for hash-route/tab keyboard behavior in Svelte-native tab actions.
- [x] DSH-SVLT-TEST2 Expand Playwright assertions for generated SvelteKit asset/runtime loading under `/dashboard` base path.
- [x] DSH-SVLT-TEST2.1 Add Playwright assertions that dashboard static assets resolve under `/dashboard/_app/*` and `/dashboard/assets/*` without 4xx/5xx.
- [x] DSH-SVLT-TEST2.2 Add Playwright assertion that `/dashboard/login.html` stays functional after direct navigation and refresh.
- [x] DSH-SVLT-TEST2.3 Add Playwright assertion that `/dashboard` redirect contract remains `308 -> /dashboard/index.html`.
- [x] DSH-SVLT-TEST2.4 Add Playwright runtime-failure guardrails for missing module/stylesheet/script requests in generated SvelteKit output.
- [x] DSH-SVLT-DOC1 Update dashboard docs to reflect SvelteKit runtime, file layout, and rollback procedure.

## Additional completions (2026-02-18, section-preserving archive)

### todos/todo.md

#### P1 Dashboard SvelteKit Post-Cutover Excellence
- [x] DSH-SVLT-EX1 Remove remaining import-time DOM/event bindings in `dashboard/dashboard.js`; move all bindings to mount-scoped setup with deterministic teardown so route remounts remain safe.
- [x] DSH-SVLT-EX2 Continue extracting orchestration out of `dashboard/dashboard.js` into `dashboard/src/lib/runtime/*`, leaving `dashboard/modules/*` as pure domain adapters.
- [x] DSH-SVLT-EX3 Resolve current Svelte a11y warnings in dashboard tab semantics (`tablist`/`tabpanel`) while preserving keyboard/hash contracts and smoke coverage.
- [x] DSH-SVLT-EX4 Add `AbortController`-based request cancellation/dedupe for tab switches and polling to prevent stale render overwrites and wasted refresh work.
- [x] DSH-SVLT-EX5 Add explicit dashboard runtime performance telemetry (fetch latency, render timing, polling skip/resume counters) and document operator thresholds.
- [x] DSH-SVLT-EX6 Add route-remount e2e coverage (navigate away/back) and assert that ban/unban, save flows, polling, and keyboard tab navigation still function.
- [x] DSH-SVLT-EX7 Replace the temporary query-param legacy toggle with an explicit config-driven runtime switch and rollout/rollback docs.

#### P1 Dashboard SvelteKit Excellence Round 3 (Native Hardening + Perf Budgets)
- [x] DSH-SVLT-EX13 Remove native-mode dependency on `mountDashboardRuntime` by extracting remaining refresh/session/tab adapter calls out of `dashboard/dashboard.js` into Svelte runtime modules; native mode should not require legacy app mount flags.
- [x] DSH-SVLT-EX14 Replace runtime chart script injection in `src/routes/+page.svelte` (`ensureScript`) with a deterministic static load path (preload/import strategy) to reduce mount-time variability and simplify lifecycle cleanup.
- [x] DSH-SVLT-EX15 Collapse Monitoring auto-refresh fan-out further by consuming a consolidated Monitoring summary contract (aligned with `MON-TEL-4`) so the native polling path does not require multiple endpoint reads per cycle.
- [x] DSH-SVLT-EX16 Add dashboard performance gates to CI/Make flow: bundle-size ceilings for `/dashboard/_app` assets and polling request-budget assertions for native remount/steady-state flows.
- [x] DSH-SVLT-EX17 Reduce repeated full-table DOM churn on Monitoring refresh by adding bounded row diff/patch updates (or virtualization where needed) for high-volume event/CDP tables.

## Additional completions (2026-02-18, section-preserving archive)

### todos/todo.md

#### P1 Dashboard SvelteKit Excellence Round 2 (Architecture + Performance)
- [x] DSH-SVLT-EX8 Continue shrinking the `dashboard/dashboard.js` hotspot by extracting config-dirty orchestration and save-check wiring into `dashboard/src/lib/runtime/*` with typed capability contracts.
- [x] DSH-SVLT-EX9 Reduce native Monitoring-tab auto-refresh fan-out by removing redundant request paths and documenting the bounded request budget per refresh cycle.
- [x] DSH-SVLT-EX10 Upgrade runtime telemetry aggregation from unbounded lifetime averages to bounded rolling windows (for example last `N` samples + p95) with deterministic reset semantics.
- [x] DSH-SVLT-EX11 Add repeated remount stress coverage (multiple navigate-away/back loops) that asserts no timer/listener/request duplication over time.
- [x] DSH-SVLT-EX12 Remove remaining direct DOM/window reads from action pipelines (redirect path, focus target lookup) by routing them through effect adapters for stricter testability.

#### P1 Dashboard SvelteKit Excellence Round 4 (Native Decoupling + Perf Hardening)
- [x] DSH-SVLT-EX18 Remove `dashboard/dashboard.js` from the native runtime refresh path by moving remaining tab-refresh/session orchestration into `dashboard/src/lib/runtime/*` modules with explicit typed contracts.
- [x] DSH-SVLT-EX19 Implement and consume a consolidated Monitoring data contract for manual/native refresh cycles (close `MON-TEL-4` alignment) so Monitoring detail updates avoid multi-endpoint fan-out.
- [x] DSH-SVLT-EX20 Replace global chart runtime script dependency with a module-scoped chart adapter lifecycle (lazy import + singleton guard + teardown) to minimize global side effects.
- [x] DSH-SVLT-EX21 Add no-flicker Monitoring auto-refresh coverage (no placeholder reset on auto cycles, bounded table patch churn assertions) in dashboard smoke + module tests.
- [x] DSH-SVLT-EX22 Add native remount/refresh soak performance gate (bounded fetch/render p95 + stable polling cadence across repeated mount loops) and wire into Make/CI reporting.

#### P0 Branch Handoff (dashboard-sveltekit-port -> main)
- [x] HND-SVLT-1 Resume from branch `codex/dashboard-sveltekit-port` at commit `979fa2f` (with `c7291e5` included immediately before it in branch history).
  - Completed on branch `codex/dashboard-sveltekit-port`; current tip was `86b42bf` (contains remount fan-out test stabilization).
- [x] HND-SVLT-2 In an unrestricted shell, run canonical verification only through Makefile paths:
  - terminal A: `make dev`
  - terminal B: `make test`
  - required outcome: Rust unit + maze benchmark + integration + dashboard e2e all green.
  - Completed on 2026-02-18 after commit `86b42bf`; `make test` passed end-to-end (including dashboard e2e).
- [x] HND-SVLT-3 If verification is green, open/update PR from `codex/dashboard-sveltekit-port` into `main` and include:
  - SvelteKit migration summary (hard cutover with no archived legacy fallback assets),
  - Makefile-only workflow enforcement updates (`AGENTS.md`, `CONTRIBUTING.md`, `Makefile`),
  - dashboard runtime/perf guardrails (`e2e` remount fan-out + bundle budget gate).
  - Completed on 2026-02-18: PR opened as `https://github.com/atomless/Shuma-Gorath/pull/1` with required handoff summary.
  - DNS troubleshooting outcome in Codex runtime: resolved (`curl -I https://api.github.com` returned `HTTP/2 200`; `gh api rate_limit` succeeded).
- [x] HND-SVLT-4 Merge to `main` after CI is green; then continue Round 4 items (`DSH-SVLT-EX18..EX22`) on a fresh `codex/*` branch.
  - Completed on 2026-02-18: work merged into `main`; Round 4 implementation and canonical verification (`make verify`, `make test`, `make build`) completed cleanly from `main`.

## Additional completions (2026-02-18, section-preserving archive)

### todos/todo.md

#### P1 Dashboard SvelteKit Excellence Round 5 (State Convergence + Functionalization)
- [x] DSH-SVLT-EX22 Codify and enforce the pre-launch policy stance: no backward DOM-ID compatibility layer, no multi-instance runtime guarantees for now, and prioritize behavior/outcome contracts over legacy structural test contracts.
- [x] DSH-SVLT-EX23 Break up `dashboard/src/lib/runtime/dashboard-native-runtime.js` into focused runtime modules (session, refresh, config wiring, DOM binding lifecycle) and reduce coordinator hotspot size materially.
- [x] DSH-SVLT-EX24 Converge on one dashboard state source of truth by removing duplicate runtime snapshot/session/status state paths and routing tab/session/snapshot updates through a single store contract.
- [x] DSH-SVLT-EX25 Remove dead/unsafe native runtime event-controller leftovers (including undeclared `dashboardEventAbortController` helpers) and add regression guardrails preventing undeclared runtime globals.
- [x] DSH-SVLT-EX26 Move primary Monitoring rendering from imperative ID-driven DOM mutation/string HTML paths to Svelte reactive component state + declarative templates.
- [x] DSH-SVLT-EX27 Replace Ban table full rebuild + per-refresh rebinding with stable row patching and delegated action handling to reduce DOM/listener churn.
- [x] DSH-SVLT-EX28 Refactor chart orchestration to instance-scoped runtime services owned by mount lifecycle (no module-level chart singletons), while retaining the shared chart runtime loader adapter.
- [x] DSH-SVLT-EX29 Standardize dashboard static asset resolution on SvelteKit base-aware paths and remove hard-coded absolute asset references from route/component templates.
- [x] DSH-SVLT-EX30 Remove superseded/unused dashboard controller abstractions (for example unused feature-controller wrapper paths) and add dead-code guard checks to module tests.
- [x] DSH-SVLT-EX31 Add architecture/perf gates for the refactor: coordinator LOC budget, duplicate-state path regression checks, and remount/listener leak checks across decomposed runtime modules.
- [x] DSH-SVLT-EX32 Publish an ADR that locks the current dashboard runtime policy (single-instance pre-launch, no backward DOM-ID compatibility shims, no bridge flag matrix) and align implementation/tests to that scope.

## Additional completions (2026-02-19, section-preserving archive)

### todos/todo.md

#### P1 Dashboard SvelteKit Excellence Finalization
Policy stance for this section: no backward DOM-ID compatibility is required pre-launch; prefer idiomatic Svelte component-local state/props and declarative rendering over imperative runtime bridges.

- [x] DSH-SX-1 Convert Config/Ip Bans/Tuning from DOM-controlled islands to Svelte-owned state and event handling.
- [x] DSH-SX-1.a Remove `dashboard/src/lib/runtime/dashboard-native-runtime.js` dependencies on `document.getElementById`-driven field mutation for tab-level rendering.
- [x] DSH-SX-1.b Replace generic DOM save wiring with explicit Svelte submit handlers and domain API services.
- [x] DSH-SX-1.c Move config dirty-state to component-local declarative selectors (baseline + derived validity/dirty state) instead of per-input imperative listeners.
- [x] DSH-SX-2 Convert monitoring charts/time-range orchestration to Svelte-local lifecycle and derived state; remove imperative chart button binding in legacy modules.
- [x] DSH-SX-3 Convert IP bans table rendering and row actions to declarative Svelte lists; remove imperative row patching and delegated DOM event glue.
- [x] DSH-SX-4 Remove superseded runtime effect/action adapter layers and keep orchestration route-local in Svelte.
- [x] DSH-SX-5 Reduce route `onMount` orchestration where possible by using SvelteKit route data/state primitives for bootstrap and auth redirect decisions.
- [x] DSH-SX-5.a Keep route bootstrap inputs in `dashboard/src/routes/+page.js` (`base`, chart asset path, image asset path, initial hash tab).
- [x] DSH-SX-5.b Move hash-sync and visibility lifecycle handling to `<svelte:window>` / `<svelte:document>` in `dashboard/src/routes/+page.svelte`.
- [x] DSH-SX-6 Remove HTML injection surfaces in dashboard components (`{@html}`) unless sanitizer-backed and justified; prefer plain text rendering.
- [x] DSH-SX-7 Delete superseded dashboard runtime glue/modules after each cutover slice and enforce no-dead-module graph constraints.
- [x] DSH-SX-8 Rewrite dashboard tests toward behavior/outcome contracts for Svelte-owned UI flows (tab state, save state, auth, refresh, table/chart rendering), removing legacy DOM-glue assumptions.
- [x] DSH-SX-9 Update docs/ADR dashboard architecture notes to reflect final Svelte-owned runtime model and removed bridge layers.
- [x] DSH-SX-10 Audit dependency and setup paths (`package.json`, scripts, `make setup`, CI) so only required SvelteKit/runtime/test deps remain and all required deps bootstrap deterministically.
- [x] DSH-SX-10.a Keep `make setup` as deterministic bootstrap path for SvelteKit + Playwright Chromium and verify dependency checks gate on `node_modules/.pnpm`, `vite`, `svelte`, `@sveltejs/kit`, `@playwright/test`.
- [x] DSH-SX-10.b Validate restricted-sandbox e2e behavior path: Playwright preflight failure is surfaced and can cleanly short-circuit via `PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1` (without launching failing browser suite).

## Additional completions (2026-02-19, section-preserving archive)

### todos/todo.md

#### P1 Dashboard SvelteKit Excellence Finalization (Domain Convergence + Runtime Simplification)
- [x] DSH-SX-11 Move remaining dashboard shared domain adapters from `dashboard/modules/*` into `dashboard/src/lib/domain/*` and rewire all runtime/component/store imports.
- [x] DSH-SX-12 Remove superseded runtime wrapper layers (`dashboard/src/lib/runtime/dashboard-runtime.js`, `dashboard/src/lib/runtime/dashboard-runtime-tab-state.js`) and keep route/runtime wiring directly on `dashboard-native-runtime`.
- [x] DSH-SX-13 Make native runtime import path SvelteKit-safe by removing SSR-unsafe top-level `window` reads in runtime mount option normalization.
- [x] DSH-SX-14 Update dashboard architecture docs and module-boundary docs to reflect final `src/lib/domain` + direct native-runtime wiring model.
- [x] DSH-SX-15 Update dashboard module/unit graph tests to assert the new domain layering and direct route->native runtime wiring contract.
- [x] DSH-SX-16 Re-run canonical verification for this slice (`make test-dashboard-unit`, `make verify`, `make build`, `PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1 make test`) and confirm green results.

## Additional completions (2026-02-19, section-preserving archive)

### todos/todo.md

#### P0 Dashboard SvelteKit Security/Sanitization/Performance Hardening
- [x] DSH-SP-SEC-1 Add dashboard API-client request timeout guards with abort-safe behavior so stalled admin endpoints cannot hang route refresh/polling indefinitely.
- [x] DSH-SP-SAN-1 Add bounded sanitization for Monitoring summary/range datasets (rows, trend points, and numeric coercion) to prevent pathological payloads from driving excessive DOM/chart work.
- [x] DSH-SP-PERF-1 Remove redundant trend chart updates and abort inactive long-range fetches to reduce unnecessary chart redraw/network churn.
- [x] DSH-SP-TEST-1 Add/update dashboard unit contracts covering timeout behavior and monitoring data-bound guards.
- [x] DSH-SP-DOC-1 Update dashboard docs with the new timeout and bounded-monitoring render policy.

## Additional completions (2026-02-19, section-preserving archive)

### todos/todo.md

#### P3 Monitoring Signal Expansion (Dashboard + Telemetry)
- [x] DSH-MON-7 Deliberate Prometheus parity scope for Monitoring completed with widget-by-widget audit matrix, parity classifications, prioritized add-list, and cardinality/cost guardrails. ([`docs/monitoring-prometheus-parity-audit.md`](../docs/monitoring-prometheus-parity-audit.md))

#### P1 Dashboard SvelteKit Excellence Continuation
- [x] DSH-SX-17 Extract route orchestration from `dashboard/src/routes/+page.svelte` into a dedicated runtime controller module (`dashboard/src/lib/runtime/dashboard-route-controller.js`) while preserving hash/polling/session behavior contracts.
- [x] DSH-SX-18 Decompose `dashboard/src/lib/components/dashboard/MonitoringTab.svelte` into focused monitoring subsection components under `dashboard/src/lib/components/dashboard/monitoring/` and keep existing monitoring DOM ID contracts for smoke/e2e assertions.
- [x] DSH-SX-19 Update dashboard architecture guard tests to enforce controller/module decomposition and monitoring subsection component usage.

## Additional completions (2026-02-19, section-preserving archive)

### todos/todo.md

#### P3 Monitoring Signal Expansion (Dashboard + Telemetry)
- [x] DSH-MON-8 Implement Priority-1 low-cardinality missing-export metric families from [`docs/monitoring-prometheus-parity-audit.md`](../docs/monitoring-prometheus-parity-audit.md) (`cdp_detections`, challenge reason totals, PoW outcomes/reasons, rate outcomes, GEO action totals).
- [x] DSH-MON-9 Add `/metrics` regression coverage and dashboard parity assertions for newly exported monitoring families (including cardinality guardrail tests).
- [x] MON-TEL-4 Add rate-limit violation summary endpoint (or equivalent aggregation contract) that returns filtered offender/top-path/top-window data without requiring expensive client-side filtering over generic event feeds.

## Additional completions (2026-02-19, section-preserving archive)

### todos/todo.md

#### P3 Platform and Configuration Clarity
- [x] Initialize Ban IP pane duration controls from the current Admin Manual Ban default duration so Ban IP and Ban Durations panes stay consistent.
