# 🐙 Testing Guide

## 🐙 Quick Commands (Official)

```bash
make test             # Full umbrella suite: unit + canonical maze verification gate + Spin integration + adversary runtime-surface gate + mandatory fast adversarial matrix + SIM2 advisory gates + dashboard e2e
make clear-dev-loopback-bans # Clear local loopback-style bans (127.0.0.1, ::1, unknown) from a running dev server
make test-local-contributor-root-access-contract # Live proof that loopback-ban cleanup restores root browsing on a running local server
make test-unit        # Unit tests only (native Rust)
make unit-test        # alias for make test-unit
make test-native-build-warning-hygiene # Focused native Rust compile gate with warnings treated as errors
make test-env-isolation-contract # Focused Rust test env-mutation lock_env() contract gate
make test-ci-workflow-action-versions # Focused GitHub workflow official-action version contract gate
make test-tarpit-collateral-risk-contract # Focused tarpit exact-principal escalation and same-bucket no-cross-contamination gate
make test-maze-benchmark # Deterministic maze asymmetry benchmark gate
make test-maze-verification-wiring # Focused Makefile/CI wiring proof for the canonical maze verification gate
make test-maze-verification-gate # Canonical maze verification gate: benchmark + live traversal + live browser + concurrency proof
make test-maze-live-browser-unit # Focused helper/browser-driver checks for the live maze browser gate
make test-maze-live-browser-contract # Live Chromium gate for JS/no-JS maze traversal, micro-PoW, replay, and escalation
make test-maze-state-concurrency-contract # Focused native burst/concurrency proof for maze budget, replay, and checkpoint primitives
make test-integration # Integration tests only (waits for existing Spin readiness)
make integration-test # alias for make test-integration
make test-gateway-harness # Gateway fixture/failure harness + deploy guardrail parser tests
make test-gateway-wasm-tls-harness # wasm32 TLS cert-failure matrix (expired/self-signed/hostname-mismatch; external egress required)
make test-gateway-origin-bypass-probe # Optional active direct-origin bypass probe (requires URL args)
make test-gateway-profile-shared-server # Shared-server gateway contract + forwarding checks
make test-gateway-profile-edge # Edge/Fermyon gateway contract + signed-header origin-auth checks
make test-remote-edge-signal-smoke # Live shared-host trusted-edge proof (ssh-managed remote)
make test-live-feedback-loop-remote # Live shared-host feedback-loop proof (active ssh-managed remote)
make test-live-feedback-loop-remote-unit # Local behavior proof for the live feedback-loop verifier
make test-live-feedback-loop-remote-contracts # Local wrapper/process-tree and remote-wiring contract proof for the live feedback-loop verifier
make test-integration-cleanup-contract # Local integration shell cleanup/restore contract proof
make smoke-gateway-mode # Fast gateway smoke (allow forward, enforcement-local, fail-closed outage)
make test-adversarial-manifest # Validate adversarial scenario manifest + fixture references
make test-adversarial-coverage-contract # Validate canonical full_coverage contract parity (plan + manifests + runner)
make test-adversarial-fast # Mandatory fast adversarial matrix (smoke + abuse + Akamai)
make test-adversarial-preflight # Validate required adversarial secrets/setup and ensure browser-lane Chromium readiness
make test-adversarial-smoke # Mandatory adversarial fast smoke profile (waits for existing Spin readiness)
make test-adversarial-abuse # Replay/stale/order-cadence abuse regression profile
make test-adversarial-akamai # Akamai fixture-driven simulation profile
make test-adversarial-coverage # Expanded adversarial coverage profile (pre-release gate)
make test-adversarial-sim-selftest # Minimal deterministic simulator mechanics self-test (no Spin server required)
make test-adversarial-soak # Deep adversarial soak gate alias for full_coverage
make test-adversarial-live # Loop adversarial profile for live monitoring demos (Ctrl+C to stop)
make test-adversarial-repeatability # Deterministic drift gate across smoke/abuse/coverage (N=3)
make test-adversarial-promote-candidates # Frontier finding triage + deterministic replay/promotion checks
make test-adversarial-container-isolation # Validate black-box container isolation contract (Docker required)
make test-adversarial-container-blackbox # Run containerized black-box adversary worker (Docker required)
make test-adversarial-lane-realism-contract # Focused shared Scrapling/Agentic realism-profile contract gate
make test-adversarial-llm-realism # Focused Agentic request-mode burst/pause realism, worker receipt truth, and recent-run projection gate
make test-adversarial-llm-browser-runtime # Focused Agentic browser-mode live session, receipt truth, and recent-run projection gate (requires running Spin)
make test-adversarial-llm-fit # Focused bounded LLM fulfillment-plan contract gate
make test-adversary-sim-agentic-action-realism # Focused Agentic action-surface + degraded-fallback realism gate
make test-adversarial-llm-runtime-dispatch # Focused bounded LLM runtime dispatch + typed ingest gate
make test-adversarial-identity-envelope-contract # Focused proxy-pool and identity-envelope realism contract gate
make test-adversary-sim-exploration-envelope-realism # Focused Scrapling exploration-envelope contract and planner gate
make test-adversary-sim-exploration-receipts # Focused Scrapling traversal-frontier receipt realism gate
make test-adversary-sim-header-transport-realism # Focused header, locale, and transport-envelope realism gate
make test-adversary-sim-transport-fingerprint-realism # Focused achieved-transport-class, degraded-truth, and observer-surface realism gate
make test-adversary-sim-browser-secondary-traffic-realism # Focused browser secondary-traffic and compact projection realism gate
make test-adversary-sim-recurrence-realism # Focused bounded dormancy, re-entry, and recurrence receipt/dispatch gate
make test-adversary-sim-long-window-recurrence-realism # Focused campaign-return dormancy realism gate with representative-vs-local proof truth
make test-client-ip-topology-contract # Focused client-IP topology gate for shared-host, edge, and /shuma/health trust behavior
make test-adversary-sim-trusted-ingress-ip-realism # Focused sim trusted-ingress proxy, planner fallback, and no-worker-privilege IP realism gate
make test-adversary-sim-identity-observer-truth # Focused identity-provenance receipt and truthful Red Team/Game Loop wording gate
make test-scrapling-game-loop-mainline # Focused active-mainline bundle: attacker-faithful Scrapling plus the first working game loop
make test-adversary-sim-scrapling-owned-surface-contract # Focused Scrapling owned-surface matrix and success-contract gate
make test-adversary-sim-scrapling-category-fit # Focused Scrapling category-ownership and worker-plan contract gate
make test-adversary-sim-scrapling-browser-capability # Focused Scrapling browser and stealth persona capability gate
make test-adversary-sim-scrapling-proxy-capability # Focused Scrapling request and browser proxy-capability gate
make test-adversary-sim-scrapling-malicious-request-native # Focused attacker-faithful malicious request-native Scrapling behavior gate
make test-adversary-sim-scrapling-coverage-receipts # Focused Scrapling owned-surface receipt and recent-run closure gate
make test-rsi-game-mainline # Focused first-working-loop mainline proof: automatic post-sim hook, rerun-first judged episodes, and shared-host verifier terminal follow-on proof
make test-adversarial-coverage-receipts # Focused canonical category-coverage receipt and gating checks
make test-protected-tuning-evidence # Focused protected tuning-evidence eligibility and fail-closed benchmark checks
make test-verified-identity-calibration-readiness # Focused verified-identity taxonomy/benchmark/reconcile seam gate
make test-verified-identity-taxonomy-crosswalk # Focused verified-identity taxonomy crosswalk gate
make test-verified-identity-alignment-receipts # Focused verified-identity taxonomy alignment receipt gate
make test-verified-identity-botness-conflicts # Focused verified-identity vs botness conflict metric gate
make test-verified-identity-guardrails # Focused verified-identity no-harm guardrail gate
make test-host-impact-telemetry # Focused forwarded-latency telemetry and hot-read projection gate
make test-host-impact-benchmark # Focused host-impact snapshot and benchmark gate
make test-oversight-host-impact # Focused host-impact reconcile gate
make test-adversarial-frontier-attempt # Protected-lane frontier provider attempt probe (advisory/non-blocking)
make test-frontier-governance # Frontier artifact guard (forbidden keys + secret leak checks)
make test-frontier-unavailability-policy # Frontier degraded-threshold policy tracker + actionability artifact
make test-sim2-operational-regressions # SIM2 operational regressions for active deterministic profiles (retention/cost/security required; failure/prod checked when present)
make test-sim2-operational-regressions-strict # Strict SIM2 operational regressions (all failure/prod/retention/cost/security domains required)
make test-sim2-governance-contract # SIM2 hybrid lane + governance contract conformance diagnostics
make test-admin-machine-contracts # Focused recent-change ledger + operator snapshot + benchmark admin read contracts
make test-traffic-taxonomy-contract # Focused non-human taxonomy and snapshot taxonomy-contract checks
make test-traffic-classification-contract # Focused non-human classification receipts and fail-closed benchmark gating checks
make test-admin-api-routing-contract # Focused admin route-family contract gate for structural API refactors
make test-controller-mutability-policy # Focused controller mutability-ring policy checks
make test-controller-action-surface # Focused allowed-actions and controller-family mapping checks
make test-controller-action-surface-parity # Focused parity checks across mutability policy, allowed-actions, benchmark escalation, and patch shaping
make test-controller-hard-boundaries # Focused hard-boundary rejection checks for controller-forbidden surfaces
make test-benchmark-comparison-contract # Focused benchmark comparison helper contract checks
make test-operator-objectives-contract # Focused operator objectives, decision-ledger, and snapshot wiring checks
make test-operator-objectives-category-contract # Focused category-aware operator-objectives contract checks
make test-benchmark-category-eligibility # Focused category-aware benchmark eligibility and comparison checks
make test-replay-promotion-contract # Focused replay-promotion lineage and governance checks
make test-adversarial-runner-architecture # Focused adversarial runner CLI, unit, and validate-only checks
make test-adversary-sim-domain-contract # Focused adversary-sim lifecycle and lane-domain checks without live runtime-surface traffic
make test-adversary-sim-make-target-contract # Explicit Makefile selector/wiring contract for adversary-sim feature targets
make test-ip-range-suggestions # Focused IP-range suggestion regression gate (runtime + dashboard)
make test-verified-identity-make-target-contract # Explicit Makefile selector/wiring contract for verified-identity feature targets
make test-host-impact-make-target-contract # Explicit Makefile selector/wiring contract for host-impact feature targets
make test-coverage    # Unit coverage to lcov.info (requires cargo-llvm-cov)
make test-dashboard-unit # Dashboard module unit tests (Node `node:test`)
make test-dashboard-adversary-sim-lane-contract # Focused dashboard lane-contract checks for the red-team lane selector + diagnostics
make test-dashboard-red-team-lane-selector-contract # Focused Red Team lane-selector gate for Agentic and Scrapling + Agentic control truth
make test-dashboard-auth-gate # Focused dashboard auth-gate checks for logged-out /shuma/dashboard entry
make test-dashboard-tab-information-architecture # Focused dashboard source + rendered IA proof for tab registry alignment and Monitoring/Traffic/Diagnostics ownership
make test-dashboard-game-loop-accountability # Focused dashboard behavior + rendered proof for Game Loop observer-accountability projection
make test-rsi-game-mixed-restriction-score-spine # Focused mixed-attacker restriction score-spine proof for controller-grade benchmark/comparison/urgency/move-selection wiring
make test-dashboard-traffic-pane # Focused dashboard behavior + rendered proof for Traffic ownership, traffic-first ordering, and shared refresh-bar wiring
make test-dashboard-runtime-unit-contracts # Focused dashboard native/refresh runtime behavior checks
make test-dashboard-policy-pane-ownership # Focused dashboard unit checks that Policy owns the moved panes and Tuning stays botness-only
make test-dashboard-verified-identity-pane # Focused Verification-tab surfacing checks for verified identity controls + health summary
make test-dashboard-red-team-pane # Focused Red Team controls, lane selection, and recent-run surfacing checks
make test-ban-duration-family-truth # Focused config/runtime/Policy-tab ban-duration family parity checks
make test-dashboard-e2e-ban-duration-family-truth # Focused Playwright smoke for Ban Durations coverage in Policy
make test-dashboard-budgets # Dashboard /_app bundle-size ceilings report (warn-only by default)
make test-dashboard-budgets-strict # Dashboard /_app bundle-size ceilings (hard-fail)
make test-dashboard-e2e-adversary-sim # Focused Playwright adversary-sim dashboard smoke checks
make test-dashboard-e2e-tab-information-architecture # Focused Playwright tab label/order and keyboard-navigation smoke checks
make test-dashboard-e2e-policy-pane-ownership # Focused Playwright smoke for Tuning and Policy save flows after the pane move
make test-dashboard-e2e # Playwright dashboard smoke tests (waits for existing Spin readiness)
make seed-dashboard-data # Seed local dashboard sample records against running Spin
make test-dashboard   # Manual dashboard checklist
```

## 🐙 Current Active Mainline

If you are changing the current attacker-faithful Scrapling -> first-working-game-loop path, start with:

```bash
make test-scrapling-game-loop-mainline
```

That command is the fastest truthful local/pre-merge proof bundle for the current mainline. It intentionally combines:

- `make test-adversary-sim-scrapling-owned-surface-contract`
- `make test-adversary-sim-scrapling-malicious-request-native`
- `make test-adversary-sim-scrapling-coverage-receipts`
- `make test-rsi-game-mainline`

Additional scope notes:

- This local path is expected to run against the generated contributor root-hosted public surface: `/`, `/about/`, `/research/`, `/plans/`, `/work/`, `/atom.xml`, `/robots.txt`, and `/sitemap.xml`. Contributors can browse that surface on local dev whenever the generated artifact exists; it no longer depends on adversary-sim being actively enabled.
- It proves attacker-faithful Scrapling plus the first working game loop only.
- It does not yet prove repeated retained improvement under the strict `human_only_private` loop, the later human-traversal calibration that must be measured from real human sessions, or the live-host realism layer on its own.

## 🐙 Make-Target Contract Lanes

Selector-only Makefile proof now lives in explicit contract lanes instead of being hidden inside feature-behavior targets:

- `make test-adversary-sim-make-target-contract`
- `make test-verified-identity-make-target-contract`
- `make test-host-impact-make-target-contract`

Use those lanes when you need to protect target wiring or selector scope. Use the feature targets themselves when you want the actual behavior, telemetry, or benchmark proof.

It does **not** replace:

- `make test` for the broader local/CI umbrella suite,
- `make test-adversarial-coverage` for deeper adversarial oracle coverage,
- or `make test-live-feedback-loop-remote` for live shared-host operational proof.

Use the narrower focused targets when only one seam changed.

## 🐙 Canonical Test Tiers

Shuma now treats automated verification as five distinct proof tiers plus optional manual checks:

1. Static and source-contract checks
   - Examples: `make test-dashboard-svelte-check`, `make test-config-lifecycle`, focused contract or wiring guards
   - Purpose: fail fast on schema drift, source-shape contracts, and static diagnostics
2. Local behavior tests
   - Examples: `make test-unit`, `make test-gateway-harness`, focused domain/contract targets
   - Purpose: prove behavior in-process or in subprocess harnesses without a running Spin server
3. Spin runtime integration tests
   - Examples: `make test-integration`, `make test-adversarial-fast`, `make test-adversary-sim-runtime-surface`
   - Purpose: prove live local runtime behavior against a running `make dev` instance
4. Rendered dashboard tests
   - Examples: `make test-dashboard-unit`, `make test-dashboard-e2e`, focused dashboard Playwright targets
   - Purpose: prove operator-visible dashboard behavior and rendered contracts
5. Live operational proofs
   - Examples: `make test-live-feedback-loop-remote`, `make test-remote-edge-signal-smoke`, `make test-dashboard-e2e-external`
   - Purpose: prove current hosted/shared-host operational behavior against a real deployment

Manual checks such as `make test-dashboard` are useful for contributor inspection, but they are not a canonical automated proof tier.

Deferred edge-gateway proofs:

```bash
make test-fermyon-edge-signal-smoke # Later gateway-only edge proof (current deploy receipt)
make telemetry-fermyon-edge-evidence # Later gateway-only live telemetry proof
```

Notes:
- Use Makefile commands only (avoid running scripts directly)
- Integration tests require a running Spin server; targeted integration-only commands can run against `make dev` or `make dev-prod`, but the full umbrella `make test` contract requires `make dev` (`runtime-dev`).
- `make test` is the canonical local and CI pre-merge suite. It intentionally does not include live hosted or ssh-managed operational proofs.
- Live hosted/shared-host proof is a separate tier. Use `make test-live-feedback-loop-remote`, `make test-remote-edge-signal-smoke`, or `make test-dashboard-e2e-external` when you need deployment-level evidence.
- `make test-live-feedback-loop-remote-unit` now proves the verifier's local behavior only. Use `make test-live-feedback-loop-remote-contracts` when you intentionally want the retained wrapper and remote wiring contract lane.
- `make test-integration` and `make test` now call `make test-integration-cleanup-contract` before the real Spin HTTP integration run, so the retained shell-shape guard stays explicit about being contract proof.
- `make test`, `make test-integration`, and `make test-dashboard-e2e` wait for `/shuma/health` readiness before failing.
- `make test` now also checks `/shuma/admin/session` and fails fast if the running server is `runtime-prod`, because the full adversarial/dashboard contract is defined against `make dev`.
- `make test` includes the canonical maze verification gate (benchmark + live traversal + live browser + native concurrency proof), the adversary runtime-surface gate, the mandatory fast adversarial matrix (`smoke + abuse + Akamai`), SIM2 realtime/advisory gates, and Playwright dashboard e2e. If Docker is unavailable, the container black-box lane degrades to the advisory SIM2 verification matrix path instead of hard-failing the umbrella run.
- The container black-box runner chooses its own Docker reachability mode for loopback-hosted Spin instances: bridge + `host.docker.internal` on non-Linux hosts, and host-network mode on Linux when the target base URL is loopback-only. This keeps `make dev` bound to `127.0.0.1` while preserving container reachability in CI.
- Gateway profile gates are explicit and runnable independently:
  - `make test-gateway-profile-shared-server`
  - `make test-gateway-profile-edge`
  - `make smoke-gateway-mode`
- Gateway follow-on hardening gates:
  - `make test-gateway-wasm-tls-harness` runs a real wasm outbound TLS-failure matrix and requires external outbound HTTPS reachability.
  - `make test-gateway-origin-bypass-probe` is optional/operator-run and requires `GATEWAY_PROBE_GATEWAY_URL` + `GATEWAY_PROBE_ORIGIN_URL`.
    - optional strict mode: set `GATEWAY_PROBE_FAIL_ON_INCONCLUSIVE=1`.
- `make test-sim2-operational-regressions` enforces retention/cost/security domains and treats `failure_injection` + `prod_mode_monitoring` as optional when absent from the active deterministic profile artifact. Use `make test-sim2-operational-regressions-strict` when you need full-domain enforcement.
- `make test` keeps the fast adversarial matrix in the routine local/full-suite path and runs the SIM2 matrix in advisory mode against the resulting fast-profile artifact.
- `make test-adversarial-coverage` and `make test-adversarial-soak` remain the strict deterministic `full_coverage` oracle paths for deeper protected-lane verification.
- `make test-host-impact-telemetry`, `make test-host-impact-benchmark`, and `make test-oversight-host-impact` are the narrow pre-Monitoring proof path for the host-impact proxy track; use them instead of broader monitoring/controller suites when only the forwarded-latency cost proxy changed.
- `make test-dashboard-verified-identity-pane` is the narrow proof path for the first-class `Verified Identity` pane in `Verification`; it covers operator-snapshot hydration, shared config-surface ownership, and a rendered config-save round-trip without dragging in broader Monitoring work.
- `make test-dashboard-red-team-pane` is the narrow proof path for the leaner `Red Team` tab; it covers adversary-sim status adaptation, runtime normalization, shared control wiring, rendered lane-selection truth including the operator-selectable `Agentic Traffic` lane, and the recent-run contract without keeping retired status-truth or Scrapling-detail panels alive.
- `make test-dashboard-game-loop-accountability` is the narrow proof path for the observer-facing `Game Loop`; it covers operator/oversight adapter truth, bounded snapshot wiring through the refresh runtime, durable observer-round lineage, missing-receipt honesty, rendered `Agentic Traffic` lane labeling, and rendered Playwright proof that both archive-backed and freshest-recent LLM runs materialize defence-surface rows instead of going blank.
- `make test-dashboard-traffic-pane` is the narrow proof path for `TRAFFIC-TAB-1`; it covers traffic-first tab ordering, shared refresh-bar eligibility across `Traffic` and `Game Loop`, and a rendered Playwright proof that Traffic owns the traffic picture while Diagnostics narrows toward furniture proof.
- `make test-dashboard-runtime-unit-contracts` is the narrow non-rendered proof path for dashboard native-runtime and refresh-runtime behavior; use it when auth/session restore, cache invalidation, or config-mutation invalidation logic changes without needing broader Playwright churn.
- `make test-scrapling-game-loop-mainline` is the fastest truthful local/pre-merge proof path for the current active mainline. It bundles the attacker-faithful Scrapling owned-surface, malicious request-native, coverage-receipt, and first-working-game-loop gates without implying live/shared-host operational proof.
- `make test-adversarial-lane-realism-contract` is the focused proof path for `SIM-REALISM-1A`; it proves the new versioned `realism_profile` contract is emitted by both planners and validated by both host-side worker paths before later pacing/dwell behavior work starts consuming it.
- `make test-adversary-sim-scrapling-realism` is the focused proof path for `SIM-REALISM-1B`; it proves Scrapling now executes persona-specific pacing and dwell behavior, emits runtime `realism_receipt` payloads, and preserves the latest Scrapling realism receipt in recent-run monitoring history.
- `make test-adversarial-llm-realism` is the focused proof path for `SIM-REALISM-1C`; it proves Agentic request-mode now executes profile-driven focused micro-bursts and between-burst pauses, emits typed request-mode `realism_receipt` payloads from the container worker, and preserves the latest Agentic realism receipt in recent-run monitoring history.
- `make test-adversarial-llm-browser-runtime` is the focused proof path for `SIM-REALISM-1D`; it proves Agentic browser-mode now emits a real Playwright-driven black-box session against the running local public site, follows public hint breadcrumbs from root through `robots.txt` and sitemap discovery, emits browser-shaped `realism_receipt` payloads, and preserves that receipt in recent-run monitoring history.
- `make test-adversary-sim-pressure-envelope-realism` is the focused proof path for `SIM-REALISM-2A`; it proves request-native Scrapling personas now consume per-profile pressure envelopes instead of the old flat `8 requests / 2 seconds` ceiling, Agentic request-mode now records bounded concurrent burst groups instead of serializing every micro-burst, and both runtime and recent-run projection paths preserve peak concurrency plus effective cadence truth.
- `make test-adversarial-identity-envelope-contract` is the focused proof path for `SIM-REALISM-2B`; it proves the shared realism contract now carries a bounded identity envelope, Scrapling and Agentic planners can emit pool-backed identity assignments, host-side workers fail closed on drift, and observer-only receipts preserve truthful `pool_backed`, `fixed_proxy`, or `degraded_local` identity-realism posture without leaking that provenance into defence truth.
- `make test-adversary-sim-header-transport-realism` is the focused proof path for `SIM-REALISM-2C`; it proves the shared realism contract now carries bounded transport-envelope truth, Scrapling and Agentic request lanes emit coherent user-agent and Accept-Language posture instead of one static local default, Agentic browser-mode carries explicit locale and browser client posture into the Playwright session, and both worker receipt paths preserve those header or locale envelopes as observer-only truth.
- `make test-adversary-sim-transport-fingerprint-realism` is the focused proof path for `SIM-REALISM-3D`; it proves request-native and browser personas now preserve achieved transport-realism class, emission basis, and explicit degraded-truth reason through worker receipts, recent-run plus operator-snapshot hot reads, and the rendered Red Team or Game Loop runtime summaries instead of overclaiming field-grade transport realism from coarse posture names alone.
- `make test-adversary-sim-representativeness-readiness` is the focused proof path for `SIM-REALISM-3E`; it proves adversary-sim status now emits an explicit representativeness readiness contract, Red Team renders lane-specific representative/partial/degraded realism copy from that contract, and the dedicated Make target itself cannot silently drift away from the backend or dashboard selectors it is supposed to exercise.
- `make test-adversary-sim-browser-secondary-traffic-realism` is the focused proof path for `SIM-REALISM-2D`; it proves Scrapling browser personas preserve compact XHR-backed secondary-traffic counts, Agentic browser-mode preserves compact same-origin background plus subresource request counts, and recent-run plus operator-snapshot read models distinguish top-level browser actions from secondary browser activity without bloating hot reads into raw traces.
- `make test-adversary-sim-identity-observer-truth` is the focused proof path for `SIM-REALISM-2J`; it proves recent-run plus operator-snapshot payloads preserve compact identity-provenance truth, Red Team and Game Loop label trusted-ingress-backed versus degraded identity honestly, and the gate fails fast when the running Spin instance is serving stale `dist/dashboard` assets rather than the current dashboard build.
- `make test-adversarial-llm-runtime-dispatch` is the focused proof path for `SIM-LLM-1C2`; it covers the typed Rust worker-result ingest, supervisor dispatch knowledge, and the dedicated Python LLM runtime worker contract without pretending the later receipt-projection and operator-surface chain is already complete.
- `make test-adversary-sim-scrapling-owned-surface-contract` is the narrow proof path for the attacker-faithful Scrapling owned-surface matrix. Use it when changing which defenses the Scrapling lane owns, which fulfillment modes must touch them, or whether the contract says Scrapling should pass, fail, or expect mixed outcomes on those surfaces.
- `make test-adversary-sim-scrapling-malicious-request-native` is the narrow proof path for widened request-native Scrapling abuse behavior. Use it when changing worker-plan route hints, per-mode malicious submit behavior, or the rule that Scrapling personas must mix ordinary success traffic with hostile request-native challenge, PoW, or tarpit interactions on the surfaces they own.
- `make test-rsi-game-mainline` is now the focused proof path for the landed `RSI-GAME-MAINLINE-1A` plus `RSI-GAME-MAINLINE-1B` chain and the later `RSI-GAME-ARCH-1K` plus `RSI-GAME-ARCH-1L` follow-ons. It proves the automatic post-sim oversight trigger still fires once, the post-sim oversight route can apply a bounded canary, adversary-sim supervisor can auto-materialize exactly one protected post-change Scrapling candidate window, terminal improved or rolled-back judgments can persist one fresh bounded continuation rerun request, the internal adversary-sim beat can auto-start that rerun, and the later post-rerun oversight judgment can open the next bounded canary from fresh evidence instead of immediate patch chaining.
- Runtime-dev note: the supervisor-owned post-canary candidate run is intentionally shortened to `30s`, which is the smallest meaningful local window on the current shared-host cadence because it still yields roughly `30` one-second beats, about `6` full passes through Scrapling's current five personas, and up to `240` total request slots at the current per-tick cap.
- Proof-path note: the current fast adversarial and RSI continuity gates now read monitoring bootstrap and sim-event evidence from bounded hot-read paths with separate control-plane read, write, and observation timeout budgets. Keep that bounded read discipline in place when changing the runner or monitoring APIs, otherwise the local `30s` candidate or continuation window will regress into observation-path timeout noise rather than real loop failure.
- `make test-rsi-game-human-only-strict` is the focused proof path for `RSI-GAME-HO-1A`; it proves the live local loop now runs under `human_only_private`, preserves `strict_human_only` verified-identity suppression, derives strict suspicious-forwarded request, byte, and latency targets from adversary-sim scope as `0.0`, exercises the local root-hosted generated contributor surface, and records a matching `post_adversary_sim` oversight run for the latest Scrapling sim run.
- `make test-rsi-game-human-only-proof` is the focused proof path for the closed `RSI-GAME-HO-1` tranche; it bundles the strict runtime proof above with deterministic repeated retained-improvement proof, showing many bounded config moves across the strict baseline, judged retain outcomes archived with strict profile lineage, and measured movement to zero suspicious leakage without weakening `human_only_private`.
- `make test-native-build-warning-hygiene` is the focused proof path for `BUILD-HYGIENE-1`; it forces a fresh native host compile and treats warnings as errors so dead-code or cfg drift in canonical Rust test builds fails fast instead of quietly normalizing warning noise.
- `make test-env-isolation-contract` is the focused proof path for `TEST-ENV-1`; it scans Rust test functions and fails if any test mutates process env without acquiring `lock_env()` before the first mutation.
- `make test-ci-workflow-action-versions` is the focused proof path for `CI-WF-1`; it scans the workflow files and fails if the repo drifts back to the older Node20-backed `actions/checkout`, `actions/setup-node`, or `actions/upload-artifact` majors.
- `make test-tarpit-observability-contract` is the focused proof path for `TAH-11`; it covers the expanded tarpit metrics families, admin monitoring projection, capped offender-bucket catalog behavior, and entry-budget reason classification.
- `make test-tarpit-collateral-risk-contract` is the focused proof path for `TAH-19`; it proves tarpit escalation is driven by bounded exact-principal persistence instead of same-bucket neighbor pressure, while the coarse bucket view remains available for operator visibility.
- `make test-dashboard-e2e` now verifies the running Spin instance is serving the current `dist/dashboard/index.html` before Playwright runs; restart Spin after `make dashboard-build` if this check fails.
- `make test` now reseeds dashboard sample data at the end, so charts/tables stay populated for local inspection after the run.

## 🐙 Test Layers

This project uses the following practical environments inside the canonical tiers above:

1. Native Rust unit and crate-internal tests
2. Subprocess and helper harness tests
3. Spin integration and adversarial runtime tests against a running local server
4. Dashboard module tests in Node `node:test`
5. Dashboard rendered smoke tests in Playwright
6. Live hosted or ssh-managed remote operational proofs
7. Optional manual dashboard checks for contributor inspection

## 🐙 Test Layout Conventions

Rust test layout is now standardized as follows:

- Unit tests should live with the owning module, wired via `#[cfg(test)] mod tests;`.
- Module-specific test files should be placed under that module directory (for example `src/ban/tests.rs` or `src/allowlist/path_tests.rs`).
- Shared unit-test utilities belong in `src/test_support.rs` (request builders, env lock, in-memory <abbr title="Key-Value">KV</abbr> store fixtures).
- New black-box integration tests should be added in `tests/` when they can rely on public interfaces only.
- Cross-module crate-internal suites should live under `src/lib_tests/`.

## 🐙 Why Two Environments

Unit tests run natively in Rust and validate logic in isolation.
Integration tests must run in Spin because they require the <abbr title="Hypertext Transfer Protocol">HTTP</abbr> server, routing, Spin <abbr title="Key-Value">KV</abbr> storage, and real request headers.

## 🐙 Unit Tests (Native Rust)

Run with:

```bash
make test-unit
```

For the focused native warning gate, run:

```bash
make test-native-build-warning-hygiene
```

For the focused env-isolation contract, run:

```bash
make test-env-isolation-contract
```

For the focused GitHub workflow action-version contract, run:

```bash
make test-ci-workflow-action-versions
```

Unit tests validate core logic in isolation (ban logic, allowlist parsing, config defaults, <abbr title="Chrome DevTools Protocol">CDP</abbr> parsing, etc.).
Test counts may change as coverage evolves; rely on `make test-unit` output for the current total.
Coverage includes ban/unban flows, allowlists, maze generation, challenge logic, <abbr title="Chrome DevTools Protocol">CDP</abbr> parsing, and helper utilities.

## 🐙 Integration Tests (Spin Environment)

Run with:

```bash
# Terminal 1
make dev

# Terminal 2
make test-integration
```

These tests exercise the full <abbr title="Hypertext Transfer Protocol">HTTP</abbr> + <abbr title="Key-Value">KV</abbr> runtime and are required for end-to-end validation.
If your Spin environment sets `SHUMA_FORWARDED_IP_SECRET`, export it before running integration tests so the curl requests include the matching `X-Shuma-Forwarded-Secret` header:

```bash
export SHUMA_FORWARDED_IP_SECRET="your-forwarded-ip-secret"
```

If you configured `SHUMA_HEALTH_SECRET`, export it too so health checks include `X-Shuma-Health-Secret`:

```bash
export SHUMA_HEALTH_SECRET="your-health-secret"
```

The integration suite is implemented in `scripts/tests/integration.sh` and is invoked by `make test-integration`.

Integration coverage includes:
1. Health endpoint and <abbr title="Key-Value">KV</abbr> availability
2. Root endpoint behavior (block page / <abbr title="JavaScript">JS</abbr> challenge)
3. Honeypot ban flow
4. Admin config + shadow-mode toggling
5. Challenge single-use behavior (`Incorrect` then replay `Expired`)
6. Metrics endpoint
7. <abbr title="Chrome DevTools Protocol">CDP</abbr> report ingestion and auto-ban flow
8. <abbr title="Chrome DevTools Protocol">CDP</abbr> stats counters in `/shuma/admin/cdp`
9. Monitoring summary endpoint in `/shuma/admin/monitoring`
10. Unban behavior

## 🐙 Adversarial Simulation Profiles (Manifest-Driven)

Run with:

```bash
# Terminal 1
make dev

# Terminal 2
make test-adversarial-smoke
```

Available profiles:
- `make test-adversarial-fast` - mandatory fast matrix (`smoke + abuse + Akamai`)
- `make test-adversary-sim-runtime-surface` - runtime-toggle integration gate that proves a real recent `scrapling_traffic` run reaches `owned_surface_coverage.overall_status=covered` in `operator_snapshot_v1`, while live-only monitoring summary paths remain clean on the running target
- `make test-adversarial-smoke` - mandatory fast smoke gate (`SIM-T0`..`SIM-T4`)
- `make test-adversarial-abuse` - mandatory replay/stale/order-cadence abuse regressions
- `make test-adversarial-akamai` - mandatory Akamai signal fixture coverage
  - this lane is local and fixture-driven (`/fingerprint-report` with canned payloads), not a live edge proof
  - `edge_fixture` scenario latency uses explicit request latency plus modeled think/retry time, so incidental runner scheduling stalls do not masquerade as product regressions
- `make test-adversarial-coverage` - expanded coverage contract profile (`full_coverage`) including PoW success/failure, puzzle-failure fallback, replay-to-tarpit bootstrap abuse, CDP deny path, rate-limit enforcement, and GEO block coverage
  - includes defense no-op detector checks (`coverage_gates.defense_noop_checks`) that fail when targeted defenses emit zero telemetry deltas
- PR CI and release-gate workflows use this target for strict deterministic coverage proof; routine `make test` remains on the fast/advisory path.
- `make test-adversarial-sim-selftest` - minimal deterministic simulator mechanics harness (seed/order/budget/retry/gate math/teardown), intentionally non-circular
- `make test-adversarial-soak` - deep soak alias for `full_coverage` (scheduled/manual gate)
- `make test-adversarial-manifest` - schema/fixture validation without server
- `make test-adversarial-lane-contract` - black-box attacker/control capability contract parity check across deterministic/container tooling plus request-native header allowances used by Scrapling personas
- `make test-adversarial-lane-realism-contract` - shared Scrapling/Agentic realism-profile contract parity check across Rust planners, the Python LLM fulfillment helper, the host-side LLM runtime worker, and the Scrapling worker fail-closed validation path
- `make test-adversary-sim-header-transport-realism` - focused header, locale, and transport-envelope realism gate covering the canonical realism contract, the host-side Agentic execution-plan builders, the black-box container request receipt, and Scrapling request-native plus browser worker receipts
- `make test-adversary-sim-transport-fingerprint-realism` - focused transport-realism sufficiency gate covering achieved transport class, degraded-truth reason, recent-run and operator-snapshot projection, and the rendered dashboard summaries operators use to judge whether transport realism is real or degraded
- `make test-adversary-sim-browser-secondary-traffic-realism` - focused browser secondary-traffic realism gate covering the compact browser request-event or XHR receipt contract, the Agentic browser black-box integration path, the Scrapling browser worker receipt path, and recent-run plus operator-snapshot projection of compact secondary-traffic counts
- `make test-adversary-sim-scrapling-realism` - focused Scrapling pacing/dwell realism gate covering request-native and browser `realism_receipt` materialization plus recent-run projection of the latest Scrapling realism receipt
- `make test-client-ip-topology-contract` - focused trusted-header topology gate covering shared-host direct collapse to `unknown`, shared-host trusted proxy extraction, shared-host misconfigured proxy fail-closed behavior, edge `true-client-ip`, edge trusted-header fallback, and `/shuma/health` single-hop versus multi-hop rules
- `make test-adversary-sim-identity-observer-truth` - focused identity-provenance realism gate covering compact `realism_receipt` projection into recent-run and operator-snapshot payloads, truthful Red Team and Game Loop wording for trusted-ingress-backed versus degraded identity, and the served-dashboard freshness guard that prevents Playwright from passing against stale dashboard assets
- `make test-shared-host-scope-contract` - shared-host descriptor and fail-closed scope gate parity check across the versioned contract plus seed-tooling validator
- `make test-shared-host-seed-contract` - minimal shared-host seed inventory contract parity check, including required primary URL handling, bounded `robots.txt` hint ingestion, provenance merge, and rejection diagnostics
- `make prepare-scrapling-deploy` - shared-host deploy-prep helper that infers the fail-closed scope fence, root-only seed, runtime env mappings, and deploy-time receipt from the canonical public base URL
- `make test-scrapling-deploy-shared-host` - focused shared-host deploy proof for the Scrapling prep helper, Linode deploy wiring, normalized `ssh_systemd` receipt extension, and `make remote-update` preservation of the same scope/seed artifact contract
- `make test-adversary-sim-scrapling-category-fit` - focused Scrapling ownership-contract proof for canonical lane fulfillment rows, full-spectrum `fulfillment_mode` rotation, and the bounded worker-plan `category_targets` contract
- `make test-adversary-sim-scrapling-browser-capability` - focused Scrapling browser-capability gate covering dynamic-browser and stealth-browser worker execution plus browser-owned-surface receipts
- `make test-adversary-sim-scrapling-proxy-capability` - focused Scrapling proxy-capability gate covering optional request and browser proxy beat-plan plumbing plus worker proxy kwargs contracts
- `make test-adversary-sim-scrapling-coverage-receipts` - focused Scrapling owned-surface receipt proof across worker-emitted surface receipts, recent-sim-run coverage aggregation, and operator-snapshot projection of owned-surface closure
- `make test-adversary-sim-scrapling-worker` - focused real Scrapling lane gate covering the internal beat/result contract, bounded crawler and direct-request personas plus dynamic-browser and stealth-browser execution, mode-specific signed sim telemetry on real requests, host-side supervisor source-contract wiring, and the supervisor's HTTP transport parser contract
- `make test-adversarial-sim-tag-contract` - signed simulation-tag contract parity check across lane contract, runner, and container worker
- `make test-adversarial-coverage-contract` - canonical `full_coverage` contract parity check across SIM2 plan rows, manifests, runner enforcement, and the frozen unit-level coverage-contract matrix
- `make test-adversarial-llm-fit` - bounded LLM browser/request fulfillment-plan contract proof across runtime beat payloads, live frontier action-generation lineage, Shuma-blind host-hint sanitization, and frontier/container contract artifacts
- `make test-adversarial-coverage-receipts` - canonical category-coverage receipt proof across adversarial coverage/scenario contracts, recent-sim full-spectrum Scrapling category receipts, the non-human coverage summary, and fail-closed benchmark gating when mapped categories are not yet covered
- `make test-verified-identity-calibration-readiness` - focused bridge gate for the current verified-identity calibration seams: taxonomy-crosswalk entry point, verified-identity snapshot section, beneficial non-human benchmark materialization, and reconcile fail-closed readiness
- `make test-verified-identity-taxonomy-crosswalk` - focused `VID-TAX-1` proof for verified-identity category projection through runtime classification, request-outcome telemetry, non-human receipts, and snapshot projection
- `make test-verified-identity-alignment-receipts` - focused `VID-TAX-2` proof for machine-first verified-identity alignment receipts and bounded snapshot summary materialization
- `make test-verified-identity-botness-conflicts` - focused `VID-BOT-1` proof for beneficial non-human benchmark conflict metrics across verified taxonomy alignment, protected verified traffic, and user-triggered agent friction drift
- `make test-verified-identity-guardrails` - focused `VID-GUARD-1` proof for benchmark tuning blockers and reconcile fail-closed behavior when verified traffic is being harmed
- `make test-protected-tuning-evidence` - protected-evidence proof across replay-promotion summary state, operator-snapshot replay visibility, fail-closed benchmark gating when evidence is only advisory, and replay-review requirements in oversight patch shaping
- `make test-operator-objectives-category-contract` - category-aware objective proof across persisted objective rows, operator-objectives admin writes, decision lineage, and snapshot projection
- `make test-benchmark-category-eligibility` - category-aware benchmark proof across the suite registry, current-instance benchmark results, tuning-eligibility blockers, and prior-window or candidate comparison helpers
- `make test-adversarial-live` - repeated live traffic generator for operator monitoring drills
- `make test-adversarial-repeatability` - deterministic replay consistency gate across `fast_smoke`, `abuse_regression`, and `full_coverage`
- `make test-adversarial-promote-candidates` - frontier finding normalization + deterministic replay triage + promotion lineage report
- `make test-adversarial-container-isolation` - container self-check gate for mount/env/identity/tooling hardening contract
- `make test-adversarial-container-blackbox` - containerized black-box worker run (separate complementary lane)
- `make test-adversarial-frontier-attempt` - protected-lane frontier provider probe attempt (advisory, non-blocking)
- `make test-frontier-governance` - fail-fast guard for forbidden frontier artifact fields and secret leaks
- `make test-frontier-unavailability-policy` - degraded-threshold policy evaluation and refresh-action artifact

Shared-host Scrapling proof map:

- `make prepare-scrapling-deploy` proves deploy-time inference and receipt generation only.
- `make test-scrapling-deploy-shared-host` proves the shared-host deploy/update automation carries the same inferred scope/seed/env contract end to end.
- `make test-adversary-sim-scrapling-category-fit` proves the bounded category-ownership and worker-plan target contract for the current full-spectrum Scrapling track.
- `make test-adversary-sim-scrapling-browser-capability` proves the browser and stealth personas execute browser-owned surfaces and materialize receipt-backed coverage.
- `make test-adversary-sim-scrapling-proxy-capability` proves optional request and browser proxy inputs flow from the beat plan into the worker session kwargs without overstating live distributed-origin proof.
- `make test-adversary-sim-scrapling-coverage-receipts` proves the bounded owned-surface receipt and recent-run closure contract for the current full-spectrum Scrapling track.
- `make test-adversary-sim-scrapling-worker` proves the hosted worker lane itself, including host-side supervisor parsing and fail-closed worker result shaping.
- `make test-adversary-sim-runtime-surface` proves the running target records a covered recent Scrapling owned-surface run while adversary simulation remains no-impact to normal user traffic.
- none of those targets make Fermyon/Akamai edge a supported full hosted Scrapling worker target; that edge runtime remains outside the current supported contract.

Structural refactor proof map:

- `make test-admin-machine-contracts` is the focused admin read-contract gate for the recent-change ledger plus the machine-first operator snapshot and benchmark endpoints.
- `make test-traffic-taxonomy-contract` is the focused taxonomy gate for the seeded canonical non-human category catalog plus its first machine-first operator-snapshot projection.
- `make test-traffic-classification-contract` is the focused classification gate for lane-to-category receipts, the operator-snapshot non-human readiness surface, the March 27 restriction-vs-recognition rail split, and fail-closed benchmark escalation when category evidence is not yet protected enough for tuning.
- `make test-adversarial-coverage-receipts` is the focused `SIM-SCR-COVER-2` gate for canonical category-coverage receipts, explicit mapped-category blockers, and fail-closed benchmark escalation when mapped fulfillment coverage is not yet complete enough for tuning.
- `make test-protected-tuning-evidence` is the focused SIM-PROTECTED gate for advisory-versus-protected replay lineage, explicit synthetic-lane ineligibility, snapshot and benchmark replay-summary visibility, and replay-review requirements before tuning proposals can proceed.
- `make test-operator-objectives-category-contract` is the focused OPS-OBJECTIVES gate for canonical category posture rows, objective validation, admin persistence, decision-ledger targeting, and snapshot projection.
- `make test-benchmark-category-eligibility` is the focused OPS-BENCH gate for per-category posture alignment metrics, explicit benchmark tuning-eligibility blockers, bounded current-instance benchmark projection, and comparison-helper semantics reused later by canary apply.
- `make test-controller-mutability-policy` is the focused mutability-policy gate for the canonical `never` / `manual_only` / `controller_tunable` classification across operator objectives and the admin-config surface.
- `make test-controller-action-surface` is the focused config-side gate for `allowed_actions_v1` and controller patch-family mapping reuse.
- `make test-controller-action-surface-parity` is the focused `CTRL-SURFACE-2` gate for parity between the canonical mutability policy, `allowed_actions_v1`, benchmark escalation candidate families, and bounded patch-policy family ownership.
- `make test-controller-hard-boundaries` is the focused `CTRL-SURFACE-3` gate for explicit rejection of controller-forbidden families and apply refusal when a proposal is not controller-tunable.
- `make test-rsi-game-contract` is the focused `RSI-GAME-1A` gate for the canonical recursive-improvement game contract, its projection through `operator_snapshot_v1`, and its reuse in `oversight_history_v1`.
- `make test-rsi-scorecard-contract` is the focused `RSI-SCORE-1` gate for explicit judge-scorecard partitioning across optimization targets, hard guardrails, regression inputs, diagnostic contexts, and homeostasis comparison inputs.
- `make test-rsi-score-exploit-progress` is the focused `RSI-SCORE-2A` gate for exploit-progress scoring, breach-locus regression comparison, scorecard wiring, and the existing Scrapling coverage-receipt proof surface.
- `make test-rsi-score-evidence-quality` is the focused `RSI-SCORE-2B` gate for exploit-evidence quality, diagnosis-confidence surfacing, fail-closed tuning eligibility, and the reconcile-side `observe_longer` behavior when exploit proof is too weak for bounded config changes.
- `make test-rsi-score-urgency-and-homeostasis` is the focused `RSI-SCORE-2C` gate for urgency scoring, immediate homeostasis-break semantics, episode-archive break-reason lineage, and restart-baseline preservation alongside the existing repeated judged-cycle proofs.
- `make test-rsi-score-move-selection` is the focused `RSI-SCORE-2D` gate for ranked bounded move selection, judge-versus-diagnosis-versus-move separation, explicit code-evolution referral, `config_ring_exhausted` escalation when repeated safe config moves fail at the same repair surface, and the localized exploit-progress-to-config-tuning path when named breach loci expose bounded repair families.
- `make test-dashboard-game-loop-accountability` is also the focused `RSI-SCORE-2E` gate for the richer Game Loop judge projection; it now proves that exploit progress, evidence quality, urgency, homeostasis-break state, named breach loci, host-cost channels, repair-family guidance, and config-exhaustion or code-referral outcomes render as distinct planes instead of collapsing back into one blended attacker-success impression.
- `make test-rsi-game-mainline` is the focused `RSI-GAME-MAINLINE-1A` plus `RSI-GAME-MAINLINE-1B` and `RSI-GAME-ARCH-1L` gate for the first explicit working self-improving loop on the current mainline. It now proves the shared-host loop advances as `judge -> rerun -> judge -> next bounded move` until an explicit stop condition is reached.
- `make test-oversight-episode-archive` is the focused `RSI-GAME-1C` gate for bounded completed-episode persistence, baseline scorecards, truthful terminal proposal outcomes for both retained and rolled-back canaries, and conservative homeostasis classification over recent judged cycles.
- `make test-oversight-move-selection-policy` is the focused `RSI-GAME-1B` gate for shortfall-attribution, explicit problem classes, bounded family guidance, and the reconcile-to-patch-policy bridge.
- `make test-benchmark-comparison-contract` is the focused benchmark helper gate for explicit baseline-availability, improvement-status, and escalation comparison semantics.
- `make test-operator-objectives-contract` is the focused objective-profile and decision-evidence gate for persisted `operator_objectives_v1`, the operator-objectives admin endpoint, the bounded decision ledger, and snapshot wiring.
- `make test-verified-identity-calibration-readiness` is the focused bridge gate before `VID-TAX-1` through `VID-GUARD-1`; it proves the current verified-identity taxonomy, snapshot, benchmark, and reconcile seams without over-claiming future alignment or conflict behavior.
- `make test-verified-identity-taxonomy-crosswalk` is the truthful narrow gate for `VID-TAX-1`; it proves the verified-identity category crosswalk lands in request-outcome telemetry and the machine-first non-human snapshot path before later alignment or botness-conflict tranches.
- `make test-verified-identity-alignment-receipts` is the truthful narrow gate for `VID-TAX-2`; it proves the verified-identity alignment receipt schema and snapshot summary stay wired to the canonical taxonomy projection instead of silently drifting back to coarse beneficial buckets.
- `make test-verified-identity-botness-conflicts` is the truthful narrow gate for `VID-BOT-1`; it proves the beneficial non-human benchmark family surfaces explicit conflict metrics rather than hiding verified-traffic harm inside coarse totals.
- `make test-verified-identity-guardrails` is the truthful narrow gate for `VID-GUARD-1`; it proves benchmark tuning eligibility and reconcile both fail closed when tolerated or allowed verified traffic is in conflict with current botness or friction signals.
- `make test-oversight-reconcile` is the focused recommend-only oversight gate for bounded patch-policy shaping, stale/contradictory-evidence refusal behavior, the oversight decision ledger, and the admin reconcile/history adapter without touching any config write path.
- `make test-oversight-agent` is the focused shared-host oversight agent gate for the internal periodic trigger contract, bounded agent-run persistence, latest-run status shaping, and supervisor-auth enforcement without exercising the post-sim hook yet.
- `make test-oversight-apply` is the focused closed-loop oversight gate for manual apply-eligibility preview, shared-host canary refusal/apply semantics, protected watch-window progression, exact rollback, and retained-canary judgment.
- In `runtime_dev`, seeded default objectives now auto-upgrade to `canary_only` so the local strict Scrapling loop can exercise bounded canary apply. If the current site already has an operator-owned objectives profile, that profile remains authoritative and may still need an explicit operator update before live local apply occurs.
- In `runtime_dev`, `SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS` may shorten the effective canary watch window for local judged-cycle iteration. This is a development-only cadence seam, not a production-faithful replacement for the declared objective watch window, and the operator surfaces must make that override explicit when it is active.
- `make test-oversight-post-sim-trigger` is the focused post-sim oversight gate for completed-run trigger detection, bounded replay-once semantics for a finished sim run, and the shared-host wrapper contract that posts periodic agent runs through the internal supervisor surface.
- `make test-rsi-game-mixed-episode-orchestration` is the focused mixed-attacker episode-state gate for ordered candidate-window and continuation sequencing across `scrapling_traffic` and `bot_red_team`, including the rule that post-sim judgment must wait until the final required lane materializes.
- `make test-replay-promotion-contract` is the focused promotion-lineage and governance gate that stays off the full replay-runner path while still proving the Rust replay-promotion store/API contract, snapshot wiring, Python promotion tooling, and SIM2 governance markers together.
- `make test-adversarial-runner-architecture` is the focused CLI, unit, and validate-only gate for the Python adversarial runner and closely related governance helpers.
- `make test-adversary-sim-domain-contract` is the focused backend adversary-sim lifecycle and lane-domain gate that stays off the live runtime-surface path.

The generated contributor public surface now lives at the root host path, with stable feed and section routes including `/`, `/about/`, `/research/`, `/plans/`, `/work/`, `/atom.xml`, `/robots.txt`, and `/sitemap.xml`.
That surface is available whenever the generated artifact exists; it no longer depends on adversary-sim being actively enabled and must remain browseable even when adversary sim is disabled or idle.
These pages are the intended first local public surface for strict-loop development, human-friction assessment under the current config, and proof when contributors do not yet have a real hosted origin behind Shuma.
Contributor refresh paths:
- `make sim-public-refresh` rebuilds the generated artifact explicitly.
- `make sim-public-refresh-if-stale` rebuilds only when the artifact is missing, source-stale, or older than the bounded freshness window.
- `make dev` and `make run` reuse the stale-check path so contributors can browse the root-hosted generated public surface locally without running adversary sim first, including when adversary sim is disabled or idle.
- `make build`, `make setup-runtime`, and `make run-prebuilt` do not generate the contributor site artifact.
Dashboard DOM-class contract for runtime/simulation affordances:
- `<html>` must include exactly one runtime environment class: `runtime-dev` or `runtime-prod` (derived from trusted runtime config).
- `<html>` connection state classes are heartbeat-owned: runtime boots in `disconnected`, flips to `connected` after successful heartbeat, enters `degraded` on heartbeat failures, and transitions to `disconnected` after configured hysteresis threshold (`N`) of consecutive heartbeat failures.
- `<html>` must include `adversary-sim` only when backend truth reports `adversary_sim_enabled=true`.
- These classes are presentational hooks only and must not alter defence/auth behavior directly.

Dashboard adversary-sim orchestration control contract:
- `POST /shuma/admin/adversary-sim/control` is the explicit admin-authenticated + CSRF-protected control path for ON/OFF transitions.
- Control submissions must include `Idempotency-Key`, pass strict origin/referer + fetch-metadata trust checks, accept optional strict `lane` values (`synthetic_traffic`, `scrapling_traffic`, `bot_red_team`, `parallel_mixed_traffic`), and return `operation_id` + `decision`.
- `GET /shuma/admin/adversary-sim/status` is the operator/dashboard read path and returns lifecycle phase, fixed guardrails, desired/actual state, active lane-routing fields (`desired_lane`, `active_lane`, `lane_switch_seq`, `last_lane_switch_at`, `last_lane_switch_reason`), live `lane_diagnostics` counters, and controller reconciliation/lease metadata. Legacy `active_lane_count` plus `lanes.{deterministic,containerized}` remain during the migration. This endpoint is read-only: it reports reconciliation need via `controller_reconciliation_required` and does not mutate/persist state on read.
- `make test-adversary-sim-lifecycle` is the focused regression gate for this contract: it must prove seeded desired-state semantics, runtime/config projection after cache reset, stale expired-run recovery, stale-state reconciliation diagnostics, auto-window expiry without a second enabled flag, and internal beat diagnostics.
- `make test-adversary-sim-lane-contract` is the focused additive-migration gate for `SIM-SCR-0`: it must prove the new desired/active lane fields, the zeroed lane-diagnostics scaffold, and preservation of legacy lane-status compatibility without changing runtime routing.
- `make test-adversary-sim-lane-selection` is the focused control-path gate for `SIM-SCR-1`: it must prove strict lane validation, lane-aware idempotency, off-state lane persistence, and truthful desired-versus-active divergence while runtime routing is still synthetic-only.
- `make test-adversary-sim-scrapling-worker` is the focused worker-routing gate for `SIM-SCR-6`: it must prove beat-boundary lane activation, internal worker-plan/result exchange, fail-closed stale-result rejection, host-supervisor parser truth, and real Scrapling traffic bounded by the shared-host scope-and-seed contract.
- `make test-adversary-sim-parallel-lane-realism` is the focused `SIM-REALISM-3A` gate for bounded mixed-lane overlap: it must prove the runtime can dispatch Scrapling and Agentic worker plans in the same beat window and wait for both results before redispatching.
- `make test-dashboard-red-team-lane-selector-contract` is the focused Red Team control gate for `SIM-REALISM-3A`: it must prove the `Scrapling + Agentic` selector option is rendered, served against fresh dashboard and runtime artifacts, and accepted through the operator path.
- `make test-adversary-sim-representativeness-readiness` is the focused `SIM-REALISM-3E` gate for explicit infrastructure truth: it must prove representative versus partial versus degraded readiness through the admin status contract, rendered Red Team wording, and exact Make-target selector coverage rather than burying deployment caveats in prose.
- `make test-adversarial-llm-runtime-projection` is the focused `SIM-LLM-1C3` gate for recent-run projection truth: it must prove additive LLM runtime receipt persistence, recent-run mode/category projection, operator-snapshot preservation, and rendered `Recent Red Team Runs` visibility for `bot_red_team` runtime rows.
- `POST /shuma/internal/adversary-sim/beat` and `POST /shuma/internal/adversary-sim/worker-result` are internal-only endpoints used by host-side supervisor workers; dashboard clients never call them directly.
- Host-side supervisor requests must satisfy trusted-forwarding (`X-Shuma-Forwarded-Secret`, loopback `X-Forwarded-For`, `X-Forwarded-Proto: https`) and send the internal supervisor marker header. Only `/shuma/admin/adversary-sim/status`, `/shuma/internal/adversary-sim/beat`, and `/shuma/internal/adversary-sim/worker-result` bypass the public admin IP allowlist under that internal supervisor contract.
- Runtime generation cadence ownership is backend/supervisor-only: dashboard refresh cadence must not control traffic generation.
- The dashboard `Red Team` controller is page-scoped: the toggle reflects latest operator intent immediately, while status polling continues through submit/converge/running phases even if another top-level tab is selected.
- Toggle-driven runs use `adversary_sim_duration_seconds` (default `30`, hard-bounded `30..900`) under backend autonomous heartbeat generation, and dashboard surfaces lifecycle state only (`off`, `running`, `stopping`) without procedural progress rendering.
- If no frontier provider keys are configured, OFF -> ON toggle attempts must show a warning dialog with two outcomes:
  - continue without frontier calls, or
  - cancel, add `SHUMA_FRONTIER_*_API_KEY` values to `.env.local`, restart `make dev`, then toggle on again.
- Runtime guardrails are hard-coded: `max_concurrent_runs=1`, `cpu_cap_millicores=1000`, `memory_cap_mib=512`, `queue_policy=reject_new`.
- Lifecycle split is explicit: `generation_active` controls producer state, while retained telemetry visibility is independent (`historical_data_visible=true` until retention expiry or explicit cleanup).

Host-side supervisor launch adapters:
- Local development (`make dev`, `make dev-prod`, `make run`, `make run-prebuilt`, `make prod`) wraps `spin up` with `scripts/run_with_oversight_supervisor.sh`, which chains the existing adversary-sim supervisor wrapper and adds bounded periodic `POST /shuma/internal/oversight/agent/run` calls for the recommend-only agent loop.
- Build/run helper targets:
  - `make adversary-sim-supervisor-build`
  - `make adversary-sim-supervisor`
- Single-host/systemd style deployment should use the same wrapper/runtime contract as `make prod-start`: launch `scripts/run_with_oversight_supervisor.sh` around `spin up`, with `SHUMA_API_KEY` injected via service env/secret manager. That wrapper chains `scripts/run_with_adversary_sim_supervisor.sh`, manages the `target/tools/adversary_sim_supervisor` worker, polls `GET /shuma/admin/adversary-sim/status`, sends `POST /shuma/internal/adversary-sim/beat`, and when Scrapling is selected runs `scripts/supervisor/scrapling_worker.py` with the repo-owned `.venv-scrapling` runtime before posting `POST /shuma/internal/adversary-sim/worker-result`. It also sends bounded periodic `POST /shuma/internal/oversight/agent/run` calls with the `oversight-agent` internal supervisor marker so the recommend-only agent loop runs off the request path on shared-host deployments.
- Containerized deployment can run the same worker as a sidecar process sharing network reachability to the Shuma instance.
- Edge/no-local-process environments are not the current supported full hosted Scrapling worker target. External-supervisor productization remains deferred until there is a concrete edge runtime target worth supporting end to end.

Live loop examples:

```bash
# Infinite fast-smoke loop until Ctrl+C
make test-adversarial-live

# Five abuse cycles with a 1-second pause between cycles
ADVERSARIAL_PROFILE=abuse_regression ADVERSARIAL_RUNS=5 ADVERSARIAL_PAUSE_SECONDS=1 make test-adversarial-live

# Akamai fixture profile with custom report output
ADVERSARIAL_PROFILE=akamai_smoke ADVERSARIAL_REPORT_PATH=.spin/adversarial/live_akamai_report.json make test-adversarial-live

# Full coverage profile loop (bounded runtime is defined in manifest)
ADVERSARIAL_PROFILE=full_coverage ADVERSARIAL_RUNS=1 make test-adversarial-live

# Explicitly clear retained telemetry history (shared local keyspace; destructive)
make telemetry-clean
```

Routine adversarial and SIM2 `make` targets now follow a strict output split:

- tracked files under `scripts/tests/adversarial/` are input fixtures, schemas, manifests, contracts, or committed baselines,
- generated local reports and receipts produced by routine `make` workflows land under `.spin/adversarial/` by default,
- the committed baseline exception remains `scripts/tests/adversarial/latest_report.baseline.json`, which stays versioned on purpose for diffing.

Live loop controls:
- `ADVERSARIAL_PROFILE` (default `fast_smoke`) must be one of `fast_smoke`, `abuse_regression`, `akamai_smoke`, `full_coverage`.
- `ADVERSARIAL_RUNS` (default `0`) controls cycle count; `0` means run until interrupted.
- `ADVERSARIAL_PAUSE_SECONDS` (default `2`) controls delay between cycles.
- `ADVERSARIAL_REPORT_PATH` (default `.spin/adversarial/latest_report.json`) controls report output file.
- `ADVERSARIAL_CLEANUP_MODE` (default `0`) toggles preserve-vs-cleanup behavior per cycle:
  - `0`: preserve state by default for live observability loops.
  - `1`: force deterministic cleanup after each cycle.
- When cleanup mode is active (`SHUMA_ADVERSARIAL_PRESERVE_STATE=0`), the runner clears both ban state and retained telemetry history through `POST /shuma/admin/adversary-sim/history/cleanup` before and after the run.
- Resilience controls:
  - `ADVERSARIAL_FATAL_CYCLE_LIMIT` (default `3`) stops the loop only after N consecutive fatal cycles.
  - `ADVERSARIAL_TRANSIENT_RETRY_LIMIT` (default `4`) retries transient failures before converting to one fatal cycle.
  - `ADVERSARIAL_BACKOFF_BASE_SECONDS` / `ADVERSARIAL_BACKOFF_MAX_SECONDS` bound transient retry backoff.
- Live loop logs now include per-cycle failure classification (`transient` vs `fatal`), retry count, backoff, and terminal failure reason when exiting.
- Live loop enforces event-quality checks; admin-only noise is treated as a fatal cycle and logs a clear reason.
- Runner also emits `.spin/adversarial/attack_plan.json` with frontier mode/provider metadata and sanitized candidate payloads.
- Promotion lane emits `.spin/adversarial/promotion_candidates_report.json` with candidate -> replay -> promotion lineage and owner-review requirements.
- Frontier threshold lane emits `.spin/adversarial/frontier_unavailability_policy.json` and can auto-open/assign model-refresh action when protected-lane degradation thresholds are exceeded.
  - If repository Issues are disabled, it must remain artifact-only and report that status in the output summary instead of failing the lane.
- Browser-realistic lane executes through Playwright (`scripts/tests/adversarial_browser_driver.mjs`) instead of HTTP emulation.
  - Browser runner controls:
  - `SHUMA_ADVERSARIAL_BROWSER_DRIVER_ENABLED` (default `true`) toggles browser-driver execution.
  - `SHUMA_ADVERSARIAL_BROWSER_RETRIES` (default `2`, clamped `1..3`) controls retry attempts for transient browser launch/network failures.
  - `SHUMA_ADVERSARIAL_BROWSER_TIMEOUT_MS` (default `15000`) bounds per-scenario browser execution timeout.
  - `SHUMA_ADVERSARIAL_BROWSER_SETTLE_MS` (default `200`) controls post-navigation settle delay.
  - Browser evidence is attached per scenario (`browser_js_executed`, `browser_dom_events`, `browser_storage_mode`, `browser_challenge_dom_path`, request-lineage correlation IDs) and enforced via `browser_execution_gates`.
- Deterministic and container black-box runners now stamp attacker-plane traffic with signed simulation headers:
  - `X-Shuma-Sim-Run-Id`
  - `X-Shuma-Sim-Profile`
  - `X-Shuma-Sim-Lane`
  - `X-Shuma-Sim-Ts`
  - `X-Shuma-Sim-Nonce`
  - `X-Shuma-Sim-Signature`
  The canonical lane contract is versioned in `scripts/tests/adversarial/lane_contract.v1.json`.
  The signing contract is versioned in `scripts/tests/adversarial/sim_tag_contract.v1.json`.
  Attacker-plane requests must not include privileged headers (including `X-Shuma-Forwarded-Secret`).
  Runtime tagging is accepted only when `SHUMA_ADVERSARY_SIM_AVAILABLE=true` and signature/timestamp/nonce verification succeeds under `SHUMA_SIM_TELEMETRY_SECRET`.
  Container black-box workers receive bounded pre-signed sim-tag envelopes from the host runner (no runtime signing secret is injected into the container).
- `latest_report.json` includes quantitative `gates` and separate `coverage_gates` sections with per-check `threshold_source`.
- `latest_report.json` `coverage_gates` section includes `defense_noop_checks` for defense-level telemetry presence validation in `full_coverage`.
- `latest_report.json` also includes `cohort_metrics` (persona-level collateral/latency summaries) and `ip_range_suggestions` seed evidence for `full_coverage`.
- `latest_report.json` includes `realism_metrics` and `realism_gates` proving runtime execution behavior for traffic-model pacing, retry envelopes, and state-mode handling (`stateless`, `stateful_cookie_jar`, `cookie_reset_each_request`).
- `latest_report.json` includes `plane_contract` guardrail metadata confirming attacker/control-plane separation checks are enforced.
- `latest_report.json` includes `coverage_contract` metadata (schema/version/hash + canonical category keys) for coverage-audit traceability.

`make test` runs `test-adversarial-fast` (which executes `test-adversarial-smoke`, `test-adversarial-abuse`, and `test-adversarial-akamai`) in sequence.
`make test-adversarial-soak` runs `test-adversarial-coverage` (`full_coverage`) for deeper scheduled/manual validation.
`test-adversarial-fast` enforces `test-adversarial-lane-contract`, `test-adversarial-sim-tag-contract`, and `test-adversarial-coverage-contract` before running profile lanes.
`test-adversarial-coverage` enforces `test-adversarial-sim-tag-contract`, `test-adversarial-coverage-contract`, and `test-frontier-governance` after artifact generation.
`test-adversarial-coverage` forces deterministic cleanup plus per-run scenario-IP rotation (`SHUMA_ADVERSARIAL_PRESERVE_STATE=0`, `SHUMA_ADVERSARIAL_ROTATE_IPS=1`) to avoid stale local cadence/persistence collisions.
Diagnostics now includes explicit tarpit progression telemetry (admissions or denials, proof outcomes, chain violations, budget reasons and fallbacks, escalation outcomes, duration and bytes buckets, and capped offender buckets) sourced from `/shuma/admin/monitoring`.
Current `full_coverage` proves tarpit bootstrap entry and event-stream minimums, but it does not yet claim advanced tarpit progress-walker telemetry; reintroduce strict `tarpit_progress_advanced` depth gates only alongside a dedicated progress-following scenario.
Container black-box controls:
- worker image path: `scripts/tests/adversarial_container/Dockerfile` (non-root user, no workspace mount, read-only rootfs at runtime)
- runtime guardrails: dropped capabilities + `no-new-privileges` + bounded CPU/memory/pids + tmpfs `/tmp`
- isolation report: `.spin/adversarial/container_isolation_report.json`
- black-box run report: `.spin/adversarial/container_blackbox_report.json`
Repeatability controls:
- default repeats: `ADVERSARIAL_REPEATABILITY_REPEATS=3`
- default profile set: `ADVERSARIAL_REPEATABILITY_PROFILES=fast_smoke,abuse_regression,full_coverage`
- summary report: `.spin/adversarial/repeatability_report.json`
- drift policy: scenario pass/outcome vectors must match exactly; latency variance is bounded by `ADVERSARIAL_REPEATABILITY_LATENCY_TOLERANCE_MS` (default `250`).
CI policy is tiered:
- Push to `main`: `ci.yml` runs `make test` plus gateway profile gates (`make test-gateway-profile-shared-server`, `make test-gateway-profile-edge`, `make smoke-gateway-mode`).
- PR to `main`: `ci.yml` runs `make test`, then `make test-adversarial-coverage`, `make test-adversarial-frontier-attempt`, and `make test-adversarial-promote-candidates`.
- Release gate (`release-gate.yml`): blocks on gateway profile gates, `make test-adversarial-coverage`, and deterministic confirmed-regression triage (`make test-adversarial-promote-candidates`), and records `make test-adversarial-frontier-attempt` as advisory status.
- Scheduled/manual deep soak: `adversarial-soak.yml` runs `make test-adversarial-soak`, `make test-adversarial-container-isolation`, and `make test-adversarial-container-blackbox`.
Deterministic/container coexistence contract:
- Deterministic lanes remain canonical mandatory blockers until explicit parity sign-off is approved (`SIM-V2-15` policy).
- Containerized lanes remain complementary scheduled/manual coverage in this phase and must not silently replace deterministic protected-lane gates.
- Parity-signoff governance is tracked via ADR + checklist template:
  - `docs/adr/0005-adversarial-lane-coexistence-policy.md`
  - `docs/adr/adversarial-lane-parity-signoff-checklist.md`
Frontier lane policy:
- Local setup is optional (`make setup` can skip provider key entry).
- Protected-lane frontier attempt is mandatory to run (attempt status is always emitted), but degraded frontier status is advisory and does not override deterministic blocking gates.
- Deterministic replay/coverage remains the release-blocking oracle; stochastic one-off frontier anomalies do not block until deterministic replay confirms them.
- Degraded-threshold tracker (`make test-frontier-unavailability-policy`) opens/updates a refresh action when protected lanes remain degraded for 10 consecutive runs or 7 days.
Simulation telemetry read policy:
- `/shuma/admin/events`, `/shuma/admin/cdp/events`, and `/shuma/admin/monitoring` include simulation-tagged rows whenever tagged simulation traffic is present.
- Tagged rows remain identifiable via `sim_run_id`, `sim_profile`, `sim_lane`, and `is_simulation`.
- `POST /shuma/admin/adversary-sim/history/cleanup` is the explicit cleanup control path; auto-off is not a retention cleanup action.
  In `runtime-prod`, cleanup requires `X-Shuma-Telemetry-Cleanup-Ack: I_UNDERSTAND_TELEMETRY_CLEANUP` (the Make target sends this header).
`test-adversarial-akamai` is fixture-driven (local `/fingerprint-report` with canned payloads) and does not require a live Akamai edge instance.
`test-remote-edge-signal-smoke` is the live ssh-managed-host proof for the currently implemented trusted-edge surfaces. It runs against the active normalized remote, uses SSH loopback transport to `127.0.0.1:3000` on the host, and proves:
- additive `/fingerprint-report` ingestion,
- authoritative `/fingerprint-report` ban behavior,
- trusted GEO country-header routing for challenge, maze, and block.
`test-live-feedback-loop-remote` is the live ssh-managed-host proof for the first shared-host recommend-only feedback loop. It runs against the active normalized remote, uses public admin endpoints plus SSH loopback to the internal supervisor route, and proves:
- the running shared-host service is launched through `scripts/run_with_oversight_supervisor.sh`,
- `GET /shuma/admin/operator-snapshot` and `GET /shuma/admin/oversight/agent/status` are available on the deployed target,
- one bounded internal periodic agent trigger executes and becomes visible in the public status projection,
- one bounded adversary-sim run completes with generated traffic,
- and a linked `post_adversary_sim` agent run becomes visible in the public status and history surfaces.
`test-fermyon-edge-signal-smoke` remains available for the later deferred edge-gateway track. It runs against the current Fermyon deploy receipt using real edge client identity semantics rather than synthetic `X-Forwarded-For`, and proves:
- additive `/fingerprint-report` ingestion,
- trusted GEO country-header routing for challenge, maze, and block,
- authoritative `/fingerprint-report` either produces a visible ban when state allows it or returns the expected enterprise-state guardrail that is verified via `spin aka logs`.
Neither target proves future Akamai-native rate or rich-geo augmentations; those remain separate backlog work.
`test-telemetry-storage` is the focused telemetry-storage regression target for this tranche. It proves:
- monitoring summary and delta reads stay on bucket-indexed paths,
- daily monitoring rollups are built and reused,
- retention honors separate raw-event and monitoring retention controls,
- density-aware query budgeting trips when a narrow window becomes too key-dense,
- the shared-host evidence harness shape remains stable.
`make telemetry-shared-host-evidence` captures a live shared-host evidence report for the active normalized remote at `.spin/telemetry_shared_host_evidence.json`. Use it after deploying the current committed `HEAD` to confirm:
- total key counts by telemetry family,
- keys per retained hour for monitoring, eventlog, and rollups,
- telemetry-adjacent monitoring-detail key counts (`maze_hits:*`, tarpit active-bucket state),
- retention health and lag from `/shuma/admin/monitoring`,
- payload sizes and latency for `/shuma/admin/monitoring`, `/shuma/admin/monitoring/delta`, and `/shuma/admin/monitoring/stream`,
- transport gzip benefit for the monitoring snapshot.
The first live shared-host baseline and compression decision are archived in [`docs/research/2026-03-11-shared-host-telemetry-storage-query-evidence.md`](research/2026-03-11-shared-host-telemetry-storage-query-evidence.md).
`make telemetry-fermyon-edge-evidence` captures the equivalent live hot-read budget report for the deferred Fermyon/Akamai-edge deploy receipt at `.spin/telemetry_fermyon_edge_evidence.json`. Use it after deploying the current committed `HEAD` to confirm:
- bootstrap latency for `/shuma/admin/monitoring?bootstrap=1...`,
- delta latency for `/shuma/admin/monitoring/delta`,
- response shaping still comes from the bounded hot-read path,
- the live edge app stays within the current budget envelope.
`make test-telemetry-hot-read-live-evidence` remains available for cross-target telemetry acceptance proof work. It must pass against:
- the active shared-host SSH remote,
- the current Fermyon/Akamai-edge deploy receipt.
The unified shared-architecture proof and the decision not to add secondary memoization or cold-tier compression are archived in [`docs/research/2026-03-13-unified-hot-read-telemetry-live-evidence.md`](research/2026-03-13-unified-hot-read-telemetry-live-evidence.md).
Operator interpretation and tuning workflow is documented in `docs/adversarial-operator-guide.md`.

Manifest and fixtures live under:
- `scripts/tests/adversarial/scenario_manifest.v1.json`
- `scripts/tests/adversarial/scenario_manifest.v2.json`
- `scripts/tests/adversarial/scenario_manifest.schema.json`
- `scripts/tests/fixtures/akamai/`

Both manifests enforce `execution_lane=black_box`; unsupported lane values fail validation before runs start.
Makefile simulation targets execute `scenario_manifest.v2.json`; `make test-adversarial-manifest` validates both `v1` and `v2`.

## 🐙 Dashboard <abbr title="End-to-End">E2E</abbr> Smoke Tests (Playwright)

Run with:

```bash
# Terminal 1
make dev

# Terminal 2
make test-dashboard-e2e
```

Behavior:
1. Installs pinned Playwright dependencies via `pnpm` (through `corepack`).
2. Uses repo-local Playwright browser cache for deterministic execution:
   - browser cache: `.cache/ms-playwright`
   - by default the runner uses repo-local browser `HOME`/config at `.cache/playwright-home`
   - optional: set `PLAYWRIGHT_FORCE_LOCAL_HOME=0` to keep system `HOME`
   - if Chromium launch fails with a known sandbox signature while local HOME is forced, the runner retries preflight with system HOME
   - if preflight still fails with repo-local browser cache, the runner automatically retries with system Playwright browser cache (when `PLAYWRIGHT_BROWSERS_PATH` was not explicitly set)
3. Runs a Chromium launch preflight and fails fast with actionable diagnostics when sandbox permissions block browser startup.
4. Runs dashboard module unit tests via `make test-dashboard-unit`.
5. Runs dashboard bundle-size budget reporting (`scripts/tests/check_dashboard_bundle_budget.js`) against `dist/dashboard/_app` (in the e2e flow this checks the currently served build without rebuilding first).
6. Verifies the running Spin instance is serving the current dashboard artifact (`dist/dashboard/index.html`) and fails fast if the server is stale.
7. Seeds deterministic dashboard data via `make seed-dashboard-data`.
8. Runs browser smoke checks for core dashboard behavior:
   - only browser smoke specs (`e2e/*.spec.js`) are executed in this stage; Node unit tests (`e2e/*.unit.test.js`) run in `make test-dashboard-unit`
   - page loads and refresh succeeds
   - runtime page errors or failed <abbr title="JavaScript">JS</abbr>/CSS loads fail the run
   - only one dashboard tab panel is visible at a time (panel exclusivity)
   - Diagnostics is manual-refresh only, and auto-refresh defaults OFF on the eligible `IP Bans` and `Red Team` tabs
   - polling cadence assertions explicitly enable auto-refresh toggle (60s production cadence)
   - Diagnostics and <abbr title="Internet Protocol">IP</abbr>-bans tabs use cursor-delta refresh by default, prefer <abbr title="Server-Sent Events">SSE</abbr> when available, and surface explicit freshness state (`fresh`/`degraded`/`stale`)
   - native eligible-tab polling request fan-out stays within bounded per-cycle budget during remount/steady-state loops
   - seeded events/tables are visible
   - clean-state <abbr title="Application Programming Interface">API</abbr> payloads render explicit empty placeholders (no crash/blank <abbr title="User Interface">UI</abbr>)
   - form validation/submit-state behavior works
   - tab hash/keyboard routing works
   - `/shuma/dashboard` canonical path redirects to `/shuma/dashboard/index.html`
   - tab-level error states surface backend failures
   - sticky table headers remain applied
9. `make test` executes a final dashboard seed step (`make seed-dashboard-data`) after e2e so local dashboards retain recent sample data.

Notes:
- Seeding is test-only and does not run during `make setup`.
- Seeded rows are operational test data and may appear in local dashboard history.
- Restricted sandbox escape hatch (local-only): set `PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1` to skip dashboard e2e after a detected Chromium launch permission block.
- CI safeguard: when `CI` is set, `PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1` is rejected and the run fails so mandatory e2e checks cannot silently downgrade to skip.
- Bundle budgets are warn-only by default to preserve development flow; set `SHUMA_DASHBOARD_BUNDLE_BUDGET_ENFORCE=1` (or run `make test-dashboard-budgets-strict`) for hard-fail enforcement.

## 🐙 Build Mode Notes

The Makefile switches crate types between `rlib` (native tests) and `cdylib` (Spin <abbr title="WebAssembly">WASM</abbr>) via `scripts/set_crate_type.sh`.
Integration tests do not run `cargo clean`; this avoids interrupting an already-running `make dev` watcher session.
Integration <abbr title="Proof of Work">PoW</abbr>/challenge sequence checks use a fixed test user-agent plus timing guardrails/retries for deterministic behavior.
Use the Makefile targets rather than calling scripts directly.

## 🐙 Generated Directories

These directories are generated locally/<abbr title="Continuous Integration">CI</abbr> and should never be committed:

- `dist/wasm/` - built Spin component artifact (`shuma_gorath.wasm`)
- `target/` - Rust build cache/output
- `.spin/` - local Spin runtime data/logs
- `.spin/deploy/` - local deploy receipts and preflight reports
- `playwright-report/` - Playwright <abbr title="HyperText Markup Language">HTML</abbr> report output
- `test-results/` - Playwright test result artifacts
- `.cache/ms-playwright/` - repo-local Playwright browser cache
- `.cache/playwright-home/` - repo-local Playwright HOME/config sandbox

`make clean` removes core build/test outputs, including stale local `src/*.wasm` artifacts.
Use `make reset-local-state` when you intentionally want to wipe local `.spin` runtime/test state (SQLite KV, logs, local deploy receipts, and verification receipt) while preserving durable operator artifacts under `.shuma`.

## 🐙 Manual Test Sequence (Optional)

Use these steps to manually validate behavior. They mirror the integration suite but let you inspect responses in detail.
If `SHUMA_FORWARDED_IP_SECRET` is set, include the matching `X-Shuma-Forwarded-Secret` header on requests that use `X-Forwarded-For`.
If `SHUMA_HEALTH_SECRET` is set, include `X-Shuma-Health-Secret` on `/shuma/health`.
Start the server in another terminal with `make dev` before running these steps.

1. Health check (loopback only):
```bash
curl -H "X-Forwarded-For: 127.0.0.1" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  -H "X-Shuma-Health-Secret: $SHUMA_HEALTH_SECRET" \
  http://127.0.0.1:3000/shuma/health
```
Expected: `OK`. If `SHUMA_DEBUG_HEADERS=true`, headers `X-KV-Status` and `X-Shuma-Fail-Mode` are also present.

2. Root endpoint (<abbr title="JavaScript">JS</abbr> challenge / block page):
```bash
curl -i -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/
```
Expected: an "Access Blocked" page or a <abbr title="JavaScript">JS</abbr> verification interstitial.
If `SHUMA_JS_REQUIRED_ENFORCED=true`, the interstitial is used when no valid `js_verified` cookie is present.
If `SHUMA_POW_ENABLED=true`, the interstitial performs a short proof‑of‑work step before `js_verified` is issued by `/pow/verify`.
If `SHUMA_POW_ENABLED=false`, the interstitial sets `js_verified` directly in browser <abbr title="JavaScript">JS</abbr>.
After a valid `js_verified` cookie is set, the originally requested page reloads and access is re-evaluated.
For browser checks, use a private window and confirm the cookie is set after the first visit.

3. Honeypot ban:
```bash
curl -s -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/instaban > /dev/null
curl -s -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/ | head -5
```
Expected: "Access Blocked" for the banned <abbr title="Internet Protocol">IP</abbr>.

4. Admin ban:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","duration":3600}' \
  http://127.0.0.1:3000/shuma/admin/ban
```
Expected: a <abbr title="JavaScript Object Notation">JSON</abbr> response containing the new ban entry.
Optional: verify with `GET /shuma/admin/ban` to confirm the <abbr title="Internet Protocol">IP</abbr> is listed.

5. Admin unban:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/shuma/admin/unban?ip=1.2.3.4"
```
Expected: the <abbr title="Internet Protocol">IP</abbr> removed from the ban list.
Optional: verify with `GET /shuma/admin/ban` that the entry is gone.

6. Shadow mode toggle:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"shadow_mode": true}' \
  http://127.0.0.1:3000/shuma/admin/config
```
Expected: a <abbr title="JavaScript Object Notation">JSON</abbr> response with `"shadow_mode":true`.

7. Metrics endpoint:
```bash
curl http://127.0.0.1:3000/shuma/metrics
```
Expected: Prometheus metrics output.

8. <abbr title="Chrome DevTools Protocol">CDP</abbr> report intake:
```bash
curl -X POST -H "Content-Type: application/json" \
  -H "X-Forwarded-For: 10.0.0.200" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  -d '{"cdp_detected":true,"score":0.5,"checks":["webdriver"]}' \
  http://127.0.0.1:3000/cdp-report
```
Expected: a success response and a <abbr title="Chrome DevTools Protocol">CDP</abbr> event recorded in analytics.

9. Challenge replay behavior:
```bash
challenge_page=$(curl -s -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/challenge/puzzle)
seed=$(python3 -c 'import re,sys; m=re.search(r"name=\"seed\" value=\"([^\"]+)\"", sys.stdin.read()); print(m.group(1) if m else "")' <<< "$challenge_page")
output=$(python3 -c 'import re,sys; m=re.search(r"name=\"output\"[^>]*value=\"([^\"]+)\"", sys.stdin.read()); print(m.group(1) if m else "")' <<< "$challenge_page")
curl -s -X POST \
  -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  --data-urlencode "seed=$seed" \
  --data-urlencode "output=$output" \
  http://127.0.0.1:3000/challenge/puzzle
curl -s -X POST \
  -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  --data-urlencode "seed=$seed" \
  --data-urlencode "output=$output" \
  http://127.0.0.1:3000/challenge/puzzle
```
Expected: first submit returns `Incorrect.` with a new-challenge link; second submit returns `Expired` with the same link.

## 🐙 Complete Manual Test Sequence

Assumes the server is already running in another terminal via `make dev`.
If you are using `SHUMA_FORWARDED_IP_SECRET`, export it before running this sequence.

```bash
set -e
BASE_URL="http://127.0.0.1:3000"
if [[ -z "${SHUMA_API_KEY:-}" ]]; then
  SHUMA_API_KEY="$(grep -E '^SHUMA_API_KEY=' .env.local | tail -1 | cut -d= -f2- | sed -e 's/^"//' -e 's/"$//')"
fi
FORWARDED_SECRET_HEADER=()
if [[ -n "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  FORWARDED_SECRET_HEADER=(-H "X-Shuma-Forwarded-Secret: ${SHUMA_FORWARDED_IP_SECRET}")
fi
HONEYPOT_PATH="$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/shuma/admin/config" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); cfg=d.get("config") or {}; print((cfg.get("honeypots") or ["/instaban"])[0])')"

echo "1) Health"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/shuma/health"
echo ""

echo "2) Root (JS challenge / block page)"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/" | head -5
echo ""

echo "3) Honeypot ban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL$HONEYPOT_PATH" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/" | head -5
echo ""

echo "4) Admin ban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","reason":"manual_test","duration":3600}' \
  "$BASE_URL/shuma/admin/ban"
echo ""

echo "5) Admin unban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "$BASE_URL/shuma/admin/unban?ip=1.2.3.4"
echo ""

echo "6) Shadow mode on, then off"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" -d '{"shadow_mode": true}' \
  "$BASE_URL/shuma/admin/config"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" -d '{"shadow_mode": false}' \
  "$BASE_URL/shuma/admin/config"
echo ""

echo "7) Metrics"
curl -s "$BASE_URL/shuma/metrics" | head -20
echo ""

echo "8) CDP report"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Content-Type: application/json" \
  -H "X-Forwarded-For: 10.0.0.200" \
  -d '{"cdp_detected":true,"score":0.5,"checks":["webdriver"]}' \
  "$BASE_URL/cdp-report"
echo ""
```

## 🐙 Local Testing Notes

- `make dev`, `make run`, and `make run-prebuilt` now boot a local trusted ingress on `http://127.0.0.1:3000` and move the internal Spin origin to `http://127.0.0.1:3001`.
- Contributor browsing on the public local path now reaches Shuma through that ingress, so ordinary local browser requests are observed as `127.0.0.1` rather than collapsing to `unknown`.
- Direct origin traffic to `http://127.0.0.1:3001` without trusted forwarding still collapses to `unknown`; the local root-access proof uses that internal origin path intentionally to prove the public contributor path stays browseable even while `unknown` is banned.
- `make dev`, `make run`, and `make run-prebuilt` still schedule loopback-ban cleanup after the local server comes up, so stale `unknown` or loopback bans from earlier local browser automation should not strand contributor browsing at `/`.
- The dashboard Playwright wrapper now clears loopback-style bans before and after local browser runs for the same reason.
- If you visit `/instaban` in a browser through `http://127.0.0.1:3000`, your local contributor identity is observed as `127.0.0.1`.
- If you hit the internal origin directly at `http://127.0.0.1:3001` without trusted forwarding, your <abbr title="Internet Protocol">IP</abbr> still collapses to `unknown`.
- To unban yourself locally:
```bash
make clear-dev-loopback-bans
```

## 🐙 Additional Manual Checks

- Allowlist: add your <abbr title="Internet Protocol">IP</abbr> via `/shuma/admin/config` and confirm access is always allowed
- Rate limit: send a burst of requests and confirm auto-ban
- Browser policy signal: send a low-version User-Agent (example: `Chrome/50`) and confirm botness signal output reflects `browser_outdated`
- <abbr title="Geolocation">GEO</abbr> policy: set `geo_*` lists via `/shuma/admin/config`, then send `X-Geo-Country` with a trusted forwarded-secret request and verify `allow/challenge/maze/block` routing precedence
- Ban list: `GET /shuma/admin/ban` and confirm entries match recent actions

## 🐙 Troubleshooting

Problem: `/shuma/health` returns 403
- Ensure you passed `X-Forwarded-For: 127.0.0.1`
- If `SHUMA_FORWARDED_IP_SECRET` is set, include `X-Shuma-Forwarded-Secret`
- If `SHUMA_HEALTH_SECRET` is set, include `X-Shuma-Health-Secret`
- Confirm the server is running with `make status`

Problem: Admin calls fail with 401/403
- Confirm `SHUMA_API_KEY` is correct
- If `SHUMA_ADMIN_IP_ALLOWLIST` is set, ensure your <abbr title="Internet Protocol">IP</abbr> is included

Problem: `make test` failed preflight (server not ready)
- Start the server with `make dev`
- Re-run with `make test`
- If startup is slow, increase wait timeout: `make test SPIN_READY_TIMEOUT_SECONDS=180`

Problem: Unsure what <abbr title="Internet Protocol">IP</abbr> the bot defence detected
- Query the ban list:
```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  http://127.0.0.1:3000/shuma/admin/ban
```

## 🐙 Dashboard Manual Check

Open:
- `http://127.0.0.1:3000/shuma/dashboard/index.html`

Verify:
- Stats update on refresh
- Charts render correctly
- Ban/unban controls work
- Shadow mode toggle updates banner
- Fail-open/closed indicator matches deployment policy
- Login key should match `make api-key-show` (or your deployed `SHUMA_API_KEY`)
- Use the dashboard Ban <abbr title="Internet Protocol">IP</abbr> and Unban actions to validate the admin <abbr title="Application Programming Interface">API</abbr> wiring

## 🐙 Tips

Use browser developer tools to inspect:
- Network tab: headers, cookies, redirects
- Application tab: `js_verified` cookie
- Console: <abbr title="JavaScript">JS</abbr> errors
### Maze live traversal

- `make test-maze-verification-wiring`
  - focused proof that the canonical Makefile and CI wiring still route maze verification through the single aggregate maze gate
- `make test-maze-verification-gate`
  - canonical local and release maze gate combining:
    - asymmetry benchmark
    - live Spin traversal proof
    - live Chromium/session proof
    - native burst/concurrency proof
- `make test-maze-live-traversal-unit`
  - focused helper-contract proof for the live maze traversal gate
- `make test-maze-live-traversal-contract`
  - requires a running local Spin server
  - proves opaque public maze entry, tokenized follow, checkpoint acceptance, hidden-link issuance, and persisted fallback reason/action evidence
- `make test-maze-live-browser-unit`
  - focused helper and browser-driver proof for the live maze browser gate
- `make test-maze-live-browser-contract`
  - requires a running local Spin server
  - proves JS-enabled checkpointed hidden-link progression, JS-disabled bounded fallback, micro-PoW browser progression, replay rejection, and repeated checkpoint-missing escalation
- `make test-maze-state-concurrency-contract`
  - focused native burst/concurrency proof for maze shared budget, replay, and checkpoint state primitives
  - proves same-process burst contention stays bounded and does not admit duplicate replay progress
