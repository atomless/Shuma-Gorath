# Compact Event Telemetry Research

Date: 2026-03-13  
Status: Research complete

## Question

Can Shuma reduce per-event telemetry size significantly without undermining:

1. operator truthfulness,
2. dashboard analysis/render performance,
3. hot-read telemetry performance on Fermyon and shared hosts,
4. future maintainability?

The motivating symptom was the size of Monitoring raw-feed rows such as challenge events that currently include:

- `null` optional fields,
- verbose blended human-readable and machine-readable `outcome` strings,
- display-derived fields that are not actually part of the persisted event record.

## Repository Findings

### 1. The Monitoring “raw feed” is not actually raw persisted telemetry

The dashboard feed is built in:

- [`dashboard/src/lib/components/dashboard/MonitoringTab.svelte`](/Users/jamestindall/Projects/Shuma-Gorath/dashboard/src/lib/components/dashboard/MonitoringTab.svelte)
- [`dashboard/src/lib/components/dashboard/monitoring-view-model.js`](/Users/jamestindall/Projects/Shuma-Gorath/dashboard/src/lib/components/dashboard/monitoring-view-model.js)

It normalizes events for display before serializing them into the “raw” feed, including fields such as:

- `executionModeLabel`
- `outcomeToken`

Those fields are derived for presentation. They are not the canonical persisted event record.

This means current operator perception of “raw telemetry bloat” is partially caused by a UI truthfulness problem, not only by storage shape.

### 2. Persisted event rows are not sparse enough

The persisted event model is in:

- [`src/admin/api.rs`](/Users/jamestindall/Projects/Shuma-Gorath/src/admin/api.rs)

`EventLogEntry` currently stores:

- `ip: Option<String>`
- `reason: Option<String>`
- `outcome: Option<String>`
- `admin: Option<String>`

Unlike the newer simulation and execution metadata fields, those optional fields do **not** use `skip_serializing_if = "Option::is_none"`, so explicit `null` values are stored.

That is a real but modest inefficiency.

### 3. The dominant size problem is the verbose `outcome` string

The large example row is big primarily because `outcome` mixes together:

1. human-readable prose,
2. machine-readable tokens,
3. zero/default signal-state entries,
4. mode/provider state,
5. taxonomy metadata.

The policy taxonomy itself already exists in structured form in:

- [`src/runtime/policy_taxonomy.rs`](/Users/jamestindall/Projects/Shuma-Gorath/src/runtime/policy_taxonomy.rs)

The current event outcome therefore duplicates information in a costly string-oriented representation.

### 4. Event-row size matters more now that hot-read documents embed recent event rows

The hot-read telemetry work now embeds recent event rows into bounded hot-read documents in:

- [`src/observability/hot_read_documents.rs`](/Users/jamestindall/Projects/Shuma-Gorath/src/observability/hot_read_documents.rs)
- [`src/observability/hot_read_projection.rs`](/Users/jamestindall/Projects/Shuma-Gorath/src/observability/hot_read_projection.rs)

That means oversized event rows now directly inflate:

- hot-read storage,
- edge bootstrap payload size,
- dashboard bootstrap parse/render work.

## External Best-Practice Research

### OpenTelemetry

Relevant guidance:

- [OpenTelemetry Logs Data Model](https://opentelemetry.io/docs/specs/otel/logs/data-model/)
- [OpenTelemetry Attribute Requirement Levels](https://opentelemetry.io/docs/specs/otel/common/attribute-requirement-level/)
- [How to write semantic conventions](https://opentelemetry.io/docs/specs/semconv/how-to-write-conventions/)
- [OpenTelemetry logs best practices](https://opentelemetry.io/docs/languages/dotnet/logs/best-practices/)

Applicable conclusions:

1. Structured logging is more efficient than unstructured string logging.
2. Expensive or high-cardinality attributes should not be defaulted in blindly; requirement levels must account for performance and availability.
3. Complex/unbounded values should be avoided as attributes because they increase overhead and query difficulty.
4. Human-readable message/body and machine-readable attributes should be kept conceptually separate rather than duplicated unnecessarily.

### Google Cloud Logging

Relevant guidance:

- [Structured logging](https://cloud.google.com/logging/docs/structured-logging)
- [Logging query language](https://cloud.google.com/logging/docs/view/logging-query-language)

Applicable conclusions:

1. Structured sparse JSON is a first-class query model.
2. Missing fields and explicit `NULL_VALUE` are not the same thing, but missing fields are fully normal and queryable.
3. There is no best-practice requirement to emit explicit `null` for every absent field.

## Architectural Options

### Option A: Minimal sparsity cleanup only

Changes:

1. omit `null` optional fields,
2. keep the current verbose `outcome` string,
3. relabel the dashboard feed so it stops pretending to be raw.

Pros:

- low-risk
- easy to ship

Cons:

- only modest storage win
- leaves the main cost driver untouched
- preserves a poor machine/human telemetry contract

### Option B: Compact canonical event schema plus UI-derived display text

Changes:

1. make event rows sparse,
2. replace the verbose `outcome` string with structured compact machine fields,
3. derive human-readable display strings in the dashboard/read path,
4. keep an optional richer forensic/debug expansion only where truly needed,
5. make the Monitoring raw feed either truly raw or honestly named.

Pros:

- largest durable storage and payload win
- clearer machine contract
- better hot-read bootstrap efficiency
- aligns with telemetry best practice

Cons:

- requires coordinated backend and dashboard change
- requires careful schema/version handling

### Option C: Reference/dictionary-based rehydration against a shared “comprehensive object”

Changes:

1. persist only deltas or compact IDs,
2. rehydrate against a global default/reference object later.

Pros:

- potentially strong size reduction

Cons:

- couples stored records to external dictionary/version context
- makes each event less self-describing
- complicates query/export/forensic use
- introduces new failure modes and migration friction

## Recommendation

Recommend **Option B**, not Option C.

Reason:

1. It captures most of the value of “reference-based” compaction without the extra indirection and drift risk.
2. A compact canonical schema with implicit defaults is simpler and more robust than a separate external hydration dictionary.
3. It keeps each event self-describing enough for export, debugging, and future tooling.

## Architectural / Performance / Cost Smell Test

From the perspective of the completed `TEL-HOT` work, the cleanest event-compaction path must also reject a few tempting-but-wrong optimizations:

1. **Do not introduce shared reference-object hydration.**
   A global default/dictionary object would save bytes, but it would also couple each stored event to external versioned context, complicate exports, and add read-time coupling to the hot-read path we just simplified.
2. **Do not minify schema keys for size.**
   The dominant byte cost is in verbose values, especially the blended `outcome` payload, not the field names. Cryptic short keys would hurt maintainability and queryability for marginal gain.
3. **Do not keep storing full provider/mode matrices on every event by default.**
   Those are largely runtime/context state, not event-specific facts. If retained at all, they belong in a bounded forensic/debug expansion rather than the canonical hot-path row.
4. **Measure document and payload wins, not just single-row wins.**
   Because `TEL-HOT` hot-read documents now embed recent events, event compaction must reduce:
   - persisted row bytes,
   - recent-events-tail hot-read document bytes,
   - bootstrap payload bytes.
   A smaller single row that does not materially shrink the hot-read payload is not enough.
5. **Keep rendered/operator views from rebuilding fat event objects.**
   Any raw-vs-rendered correction must avoid creating two heavyweight parallel event representations in memory or on the wire.

## Recommended Direction

### Persisted event contract

Persist only canonical machine-meaningful fields, for example:

1. event type
2. timestamp
3. optional actor/IP/admin only when present
4. execution mode
5. canonical reason/action/detection/taxonomy tokens
6. compact list of active/non-default signals
7. optional compact score/band values where meaningful

Avoid storing by default:

1. display labels
2. duplicate token + prose variants
3. zero/default/disabled signal entries unless absence would change meaning
4. full provider/mode matrices on every row when those are default and inferable
5. human-readable narrative expansions that can be deterministically derived from compact canonical fields

### Dashboard contract

The dashboard should:

1. derive human-readable text from the compact event record,
2. reserve “raw” for truly persisted raw rows,
3. either rename the current feed or provide a true raw-vs-rendered distinction.

### Explicit null/absence rule

Use explicit absence where semantically safe.

Keep explicit `null` or explicit values only when:

1. the difference between “unknown”, “not applicable”, and “empty” matters operationally,
2. consumers cannot safely infer the meaning.

## Research Decision

The best next tranche is:

1. compact the persisted event schema,
2. make rows sparse,
3. stop storing verbose outcome narratives as the canonical event payload,
4. make dashboard display text derived rather than stored,
5. correct the “raw feed” truthfulness problem.

This should reduce both:

1. event-log storage cost,
2. hot-read bootstrap payload weight,
3. recent-events-tail hot-read document weight,

without introducing a Fermyon-only path or a separate telemetry system.
