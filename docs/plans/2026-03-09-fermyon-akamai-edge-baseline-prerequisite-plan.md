# Fermyon / Akamai Edge Baseline Prerequisite Plan

**Goal:** Separate generic shared-host functionality from Akamai-edge-only functionality so the repo does not imply that Linode or other non-edge deployments can exercise Akamai-specific operator integrations.

## Decisions

1. Akamai-specific operator controls are only available when the deployment posture reports `gateway_deployment_profile=edge-fermyon`.
2. Generic shared-host and generic trusted-edge behavior remain available outside Akamai edge posture.
3. Future Akamai Rate and GEO expansion work is execution-blocked until the Fermyon / Akamai edge path has the same setup, deploy, and proof maturity that the Linode shared-host path now has.

## Required Baseline Before Akamai Edge Expansion

The following tranche must be complete before `AK-RG-2..8` move back into the active queue:

- `FERM-SKILL-1`: agent-oriented Fermyon / Akamai edge setup skill
- `FERM-SKILL-2`: verified, agent-executable Fermyon deploy skill
- `FERM-SKILL-3`: real edge deployment proof with captured happy path and gotchas

## Scope Separation

### Available now outside Akamai edge posture

- baseline Shuma defense behavior
- generic trusted forwarded-header and GEO-header surfaces
- fingerprint edge adapter transport and trust boundary, where a trusted upstream maps data into Shuma's canonical contract

### Akamai-edge-only

- operator-facing Akamai integration controls
- any future Akamai-specific Rate and GEO mode controls
- documentation that describes Akamai edge as the active deployment posture rather than a possible future one

## Implementation Expectations

1. Code must expose a clear posture bit so UI and admin surfaces can hide Akamai-edge-only controls when not deployed on Akamai edge.
2. Docs must describe Akamai integrations as edge-posture-specific, not universal.
3. Backlog and design notes must keep Akamai Rate/GEO work blocked until the Fermyon/Akamai edge baseline is verified.
4. Existing Fermyon deploy guidance must not over-promise maturity before the setup skill and real proof exist.
