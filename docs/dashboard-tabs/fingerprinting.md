# Dashboard Tab: Fingerprinting

Status: Retired on 2026-03-30.

The dashboard no longer exposes a live `#fingerprinting` tab.

The old surface was split as follows:

- `Akamai Bot Signal` moved to [`verification.md`](verification.md) because it is trusted verification-source posture, not a loop-tunable control.
- `Botness Scoring Signals` moved temporarily to [`tuning.md`](tuning.md) as a read-only scoring-definition bridge pending the broader `TUNE-SURFACE-2*` realignment.

The underlying fingerprinting terminology and architecture docs remain current:

- [`../fingerprinting-terminology.md`](../fingerprinting-terminology.md)
- [`../fingerprinting-signal-planes.md`](../fingerprinting-signal-planes.md)
