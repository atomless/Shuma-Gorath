# Testing Suite Structure And Mainline Friction Review

Date: 2026-03-25

## Question

From a professional testing perspective:

1. how well structured is Shuma's current test surface,
2. does it support efficient development on the current mainline,
3. and should testing work become the immediate next priority before more dashboard follow-ons?

## Short Answer

The suite is already strong on proof depth, but it is not yet optimally structured for efficient work on the current Scrapling -> game-loop mainline.

So the next testing priority should be narrow and practical:

- yes to one immediate tranche that reduces active-mainline friction,
- no to pausing current progress for a broad test-system redesign.

## Repo-Grounded Findings

### 1. The suite is already layered and more serious than average

The current test surface is not weak. It already includes:

- native Rust behavior proof,
- subprocess and helper harness proof,
- live Spin integration proof,
- adversary-sim and Scrapling proof,
- rendered dashboard proof,
- and separate hosted/shared-host operational proof.

Representative examples:

- [`Makefile`](../../Makefile#L582) defines `make test` as a real umbrella path rather than only unit coverage.
- [`Makefile`](../../Makefile#L1341) provides `make test-rsi-game-mainline` for the first working self-improving loop.
- [`Makefile`](../../Makefile#L1676) provides the deeper adversarial coverage lane.
- [`scripts/tests/test_scrapling_worker.py`](../../scripts/tests/test_scrapling_worker.py) and [`scripts/tests/test_adversary_runtime_toggle_surface_gate.py`](../../scripts/tests/test_adversary_runtime_toggle_surface_gate.py) are meaningful behavior gates, not only file-shape checks.

### 2. The biggest remaining problem is ergonomics and signal efficiency, not raw absence of tests

The active pain is not "we have no tests." It is:

- the path for the current active mainline is not surfaced as clearly as it should be,
- some target descriptions and help text still drift from reality,
- and some low-value source archaeology remains mixed into the wider suite.

Concrete examples:

- [`e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js) is still very large and still contains many source-text checks alongside stronger behavior proof.
- [`Makefile`](../../Makefile#L650) still contains stale scope/help wording.
- [`docs/testing.md`](../testing.md) is comprehensive, but it is now long enough that the "what do I run for the current mainline?" answer is not as frictionless as it should be.

### 3. The current active mainline proofs are good, but too scattered for fast repetitive development

For the current mainline, the meaningful focused proofs are spread across multiple narrow commands:

- `make test-adversary-sim-scrapling-owned-surface-contract`
- `make test-adversary-sim-scrapling-malicious-request-native`
- `make test-adversary-sim-scrapling-coverage-receipts`
- `make test-rsi-game-mainline`

That is good from a proof-composition perspective, but not yet ideal from a day-to-day development ergonomics perspective. The repo is missing one plainly named, truthful aggregate for the current mainline path.

### 4. The broader testing-rationalization chain is already correct, but it is not the immediate accelerant

The existing audit and plan remain sound:

- [`2026-03-23-testing-surface-audit.md`](./2026-03-23-testing-surface-audit.md)
- [`../plans/2026-03-23-testing-surface-rationalization-plan.md`](../plans/2026-03-23-testing-surface-rationalization-plan.md)

Those docs correctly identify:

- dashboard archaeology debt,
- selector-microtest naming drift,
- and generated artifact churn.

But that broader cleanup is not the first thing that will most reduce friction on the current mainline. The first thing that helps most is a truthful, cheap active-mainline verification path.

## External Testing Guidance That Matches What We Are Seeing

Three outside references line up closely with the current repo state:

1. Martin Fowler's "Self Testing Code" argues that the biggest benefit of good automated tests is confidence to make changes quickly, and that teams should be able to run one command frequently and trust the result. That is exactly the gap we still feel on the current active mainline: the proofs exist, but the fastest truthful path is not yet surfaced cleanly enough.  
   Source: [Martin Fowler, Self Testing Code](https://martinfowler.com/bliki/SelfTestingCode.html)

2. *Software Engineering at Google* argues for tests written against realistic behavior and public APIs, and warns that overuse of mocked or implementation-detail-oriented tests becomes brittle and impedes refactoring. That maps directly to the remaining source-archaeology debt in the dashboard/unit layer and the feature-lane selector microtests.  
   Source: [Software Engineering at Google, Chapter 13](https://abseil.io/resources/swe-book/html/ch13.html)

3. Google's testing guidance on hermetic servers emphasizes that high-value end-to-end tests are most reliable and efficient when dependencies are locally controlled and deterministic. That strongly supports keeping Shuma's best local Spin and focused runtime gates prominent rather than burying them under mixed target naming and stale help.  
   Source: [Google Testing Blog, Hermetic Servers](https://testing.googleblog.com/2012/10/hermetic-servers.html)

## Recommendation

### Immediate priority

Make one narrow testing tranche the next priority:

- define a truthful, obvious active-mainline verification bundle for the current Scrapling -> game-loop path,
- refresh help/docs around that path,
- and remove any remaining routine churn that still affects that exact fast path.

### Not the immediate priority

Do **not** stop to complete the whole remaining testing backlog before further product work. In particular, keep these as follow-on hygiene work rather than the immediate blocker:

- `TEST-HYGIENE-6C`
- `TEST-HYGIENE-3`
- `TEST-HYGIENE-4`
- `TEST-HYGIENE-5`

They still matter, but they are not the highest-leverage reduction in development friction right now.

## Conclusion

The right next move is:

1. one small active-mainline testing ergonomics tranche now,
2. then continue with the current mainline work,
3. and keep the broader testing-rationalization backlog alive behind it.

So the answer to "should testing be the next priority?" is:

- yes, but only as a narrow mainline-enabling slice,
- not as a broad pause for a wholesale test-system cleanup.
