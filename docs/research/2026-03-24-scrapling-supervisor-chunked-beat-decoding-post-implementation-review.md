# Scrapling supervisor chunked beat decoding post-implementation review

Date: 2026-03-24

## Delivered

- Updated [`scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs) so the host-side supervisor now decodes chunked internal HTTP responses before writing the beat body that the Scrapling worker parses.
- Updated [`scripts/tests/test_adversary_sim_supervisor.py`](../../scripts/tests/test_adversary_sim_supervisor.py) so the focused supervisor contract gate now proves the presence of the chunked-response decoding path.

## Why this tranche was necessary

- The previous failure-surfacing slice exposed the real live shared-host crash: the worker was failing before plan execution because `json.loads(...)` was reading a chunked-encoded beat body rather than plain JSON.
- The supervisor's minimal raw HTTP client was preserving the chunk framing in `response.body`, which was enough for its string-based `dispatch_mode` checks but not valid JSON for the Python worker.

## Verification

- `make test-adversary-sim-scrapling-worker`
- `git diff --check`

## Remaining gap

- This slice should unblock live worker plan parsing, but the full shared-host proof still needs to be rerun end-to-end to confirm persisted Scrapling traffic, defense receipts, and operator snapshot coverage all materialize coherently.
