# 🐙 Fingerprinting Signal Planes

This document defines the four cooperating planes used in Shuma-Gorath fingerprinting-related defense paths and where trust boundaries apply.

## 🐙 Plane Summary

| Plane | Inputs | Trust Boundary | Persistence Keys | Policy/Routing Impact |
| --- | --- | --- | --- | --- |
| `JS Verification` | Browser execution of the JS verification flow and optional PoW. | Origin-controlled verification scripts and signed server envelopes. | `js_verified` cookie marker and signed verification/PoW envelopes. | Gates normal request flow before higher-friction escalation. |
| `Browser CDP Automation Detection` | Browser telemetry posted by Shuma’s internal probe path. | Browser payload is untrusted until validated by server checks/thresholds. | CDP event log + counters; config under `cdp_detection_*`. | Contributes automation evidence and can trigger auto-ban under configured thresholds. |
| `Internal Passive Fingerprint Signals` | Headers, timing/flow windows, transport-header trust state, persistence-marker coherence. | Forwarded/transport evidence is only trusted with valid forwarded-header secret. | `fp:state:*`, `fp:flow:*`, `fp:flow:last_bucket:*`. | Contributes to scored botness and routing outcomes. |
| `Akamai Bot Signal` | Akamai-shaped edge payloads on `/fingerprint-report`. | Requires trusted forwarding boundary and an Akamai edge deployment posture before operator controls are exposed. | `fp:edge:*` (short-window additive evidence state). | In `additive`, contributes bounded score. In `authoritative`, can trigger documented short-circuit enforcement. |

## 🐙 Add vs Replace Matrix

| Capability | Internal Plane | Akamai Plane | Effective Behavior |
| --- | --- | --- | --- |
| Browser-runtime CDP introspection | Yes (`Browser CDP Automation Detection`) | No | Akamai does not replace internal browser CDP automation probing. |
| Passive request fingerprinting | Yes (`Internal Passive Fingerprint Signals`) | No | Akamai augments evidence; internal passive signals remain active. |
| Edge/global bot intelligence | Limited at origin | Yes | Akamai adds edge/global confidence signals unavailable to origin runtime. |
| Final policy composition | Yes (Shuma policy pipeline) | No | Shuma remains the policy/routing source of truth. |

## 🐙 Runtime Endpoint Coherence

- When fingerprint provider backend is internal, JS verification posts to the internal report endpoint.
- When Akamai ingestion is enabled, JS verification posts to the Akamai report endpoint.
- The selected provider determines report-path wiring so telemetry emission and ingestion path stay aligned.

## 🐙 Explicit Non-Goal

Akamai integration does not provide direct browser-runtime CDP introspection. It provides trusted edge-origin bot outcomes that are normalized into Shuma’s internal scoring/enforcement model.
