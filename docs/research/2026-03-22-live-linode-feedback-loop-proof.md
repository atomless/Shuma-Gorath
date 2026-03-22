# Live Linode Feedback-Loop Proof

Date: 2026-03-22
Status: passed

## Goal

Prove that the active shared-host feedback loop is running on the current Linode deployment, and that both trigger families still work on the live host after the first bounded canary-apply controller tranche landed:

1. periodic shared-host supervisor execution, and
2. immediate post-sim execution after an adversary-sim run transitions back to `off`.

## Target proved

- remote: `dummy-static-site-fresh`
- provider: `linode`
- public base URL: `https://shuma.jamestindall.org`
- deployed commit: `58d07fd07fcb9539fbdeac4fda3f455825f60618`
- deployed at UTC: `2026-03-22T20:10:28.574369Z`
- receipt: `.shuma/remotes/dummy-static-site-fresh.json`
- local live-proof receipt: `.spin/live_feedback_loop_remote.json`

## Commands executed

1. `make test-oversight-post-sim-trigger`
2. `make test-oversight-agent`
3. `make test-live-feedback-loop-remote-unit`
4. `make remote-update`
5. `make test-live-feedback-loop-remote`
6. `git diff --check`

## Live proof evidence

The final local proof receipt records:

1. initial public oversight status existed and was empty-but-truthful before manual triggering:
   - `schema_version=oversight_agent_status_v1`
   - `execution_boundary=shared_host_only`
   - `latest_trigger_kind=periodic_supervisor`
2. the shared-host periodic supervisor trigger executed on the live host:
   - periodic run id: `ovragent-1774210234-dfacee03da991dfe`
   - decision id: `oversight-1774210234-f500f80c9558e983`
   - latest trigger kind after the run: `periodic_supervisor`
3. the live adversary sim generated real traffic and completed cleanly:
   - enable operation id: `simop-1774210234-bbb20ff5f5b13e22`
   - run id during execution: `simrun-1774210234-e94acd912c1ca8ee`
   - completed state: `phase=off`
   - completed counters remained `tick_count=0`, `request_count=0`
   - persisted recent-event evidence count for the completed run: `100`
   - completed `last_run_id=simrun-1774210234-e94acd912c1ca8ee`
4. post-sim agent execution was recorded through the same shared-host feedback loop:
   - post-sim run id: `ovragent-1774210415-c9a81c892ab70e13`
   - post-sim decision id: `oversight-1774210415-c49f017de552d270`
   - latest trigger kind after completion: `post_adversary_sim`
   - oversight history latest decision id matched the post-sim decision id
5. the live service process tree showed the expected wrapper chain:
   - `make prod-start`
   - `scripts/run_with_oversight_supervisor.sh`
   - `scripts/run_with_adversary_sim_supervisor.sh`

## Conclusion

The active Linode shared-host deployment now proves the first feedback loop end to end:

1. the public machine-first oversight status contract is live,
2. the shared-host-only internal trigger executes bounded periodic runs,
3. a live adversary-sim run transitions back to `off` and is proved by persisted simulation-event evidence even when terminal generation counters are zero,
4. the post-sim hook records a linked agent run,
5. and the resulting evidence is durably inspectable through both the public status/history surfaces and the local proof receipt.
