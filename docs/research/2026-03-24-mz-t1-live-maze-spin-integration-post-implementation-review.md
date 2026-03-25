# MZ-T1 Live Maze Spin Integration Post-Implementation Review

Date: 2026-03-24
Status: Complete

## Delivered

`MZ-T1` is now covered by a focused live Spin gate in:

- [`scripts/tests/maze_live_traversal.py`](../../scripts/tests/maze_live_traversal.py)
- [`scripts/tests/test_maze_live_traversal.py`](../../scripts/tests/test_maze_live_traversal.py)
- [`Makefile`](../../Makefile) as:
  - `make test-maze-live-traversal-unit`
  - `make test-maze-live-traversal-contract`

The live gate now proves:

- opaque public maze entry discovery via admin preview,
- tokenized public follow-on traversal,
- checkpoint-missing fallback with persisted `maze_checkpoint_missing action=challenge`,
- checkpoint acceptance (`204`),
- post-checkpoint hidden-link issuance via `issue-links`,
- continued traversal through an issued hidden link,
- replay escalation to persisted `maze_token_replay action=block`.

## Important implementation notes

- The admin preview surface is intentionally not itself the public path; the gate decodes the canonical opaque public path from preview links.
- Admin and health reads now use loopback identity, while public traversal uses a fresh external-looking forwarded IP.
- The gate restores only the exact config keys it mutates because `GET /admin/config` currently includes read-only fields that are not all accepted by `POST /admin/config`.

## Follow-on

`MZ-T2` is still required for real browser/session behavior, especially:

- JS-enabled versus JS-disabled cohorts,
- browser-managed checkpoint and worker behavior,
- micro-PoW,
- and live replay/escalation behavior under real browser state.
