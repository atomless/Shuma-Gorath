# Live Linode Feedback-Loop Proof

Date: 2026-03-22
Status: passed

## Goal

Prove that the first shared-host recommend-only feedback loop is running on the active Linode deployment at the current committed `HEAD`, and that both trigger families work on the live host:

1. periodic shared-host supervisor execution, and
2. immediate post-sim execution after an adversary-sim run transitions back to `off`.

## Target proved

- remote: `dummy-static-site-fresh`
- provider: `linode`
- public base URL: `https://shuma.jamestindall.org`
- deployed commit: `12671c6ef8c153c5af79a308f3c7f663d9474911`
- deployed at UTC: `2026-03-22T10:08:42.187340Z`
- receipt: `.shuma/remotes/dummy-static-site-fresh.json`
- local live-proof receipt: `.spin/live_feedback_loop_remote.json`

## Commands executed

1. `make test-remote-target-contract`
2. `make test-live-feedback-loop-remote-unit`
3. `make remote-update`
4. `make test-live-feedback-loop-remote`
5. `git diff --check`

## Live proof evidence

The final local proof receipt records:

1. initial public oversight status existed and was empty-but-truthful before manual triggering:
   - `schema_version=oversight_agent_status_v1`
   - `execution_boundary=shared_host_only`
   - `recent_run_count=0`
2. the shared-host periodic supervisor trigger executed on the live host:
   - periodic run id: `ovragent-1774174127-370d401c3dbe41bd`
   - decision id: `oversight-1774174127-a7bf71bad6114107`
   - latest trigger kind after the run: `periodic_supervisor`
3. the live adversary sim generated real traffic and completed cleanly:
   - enable operation id: `simop-1774174127-a472ea6fbba0a693`
   - run id during execution: `simrun-1774174127-d17cbeb1205a8e57`
   - completed state: `phase=off`
   - completed counters: `tick_count=23`, `request_count=640`
   - completed `last_run_id=simrun-1774174127-d17cbeb1205a8e57`
4. post-sim agent execution was recorded through the same shared-host feedback loop:
   - post-sim run id: `ovragent-1774174309-1275bd7e90040b59`
   - post-sim decision id: `oversight-1774174309-039c1ce0343449ee`
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
3. a live adversary-sim run generates traffic and transitions back to `off`,
4. the post-sim hook records a linked agent run,
5. and the resulting evidence is durably inspectable through both the public status/history surfaces and the local proof receipt.
