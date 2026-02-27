# SIM2 Shortfall Research 4: Simulation Telemetry Authenticity

Date: 2026-02-27  
Status: Completed

## Shortfall Statement

SIM telemetry currently uses dev-only header presence checks to mark requests as simulation traffic. Because these tags are not cryptographically bound to a trusted simulation capability, classification can be spoofed by any caller who can send the three headers in runtime-dev.

## Current-State Evidence

1. Simulation metadata activation requires runtime-dev + availability + three headers, but no signature/capability verification.  
   Evidence: `src/runtime/sim_telemetry.rs:45-53`.
2. Monitoring prefix selection switches by `is_simulation_context_active()` only.  
   Evidence: `src/observability/monitoring.rs:156-161`.
3. Admin event queries include simulation records in dev based on a query parameter toggle.  
   Evidence: `src/admin/api.rs:3895-3903`.

## Research Findings

1. Forwarded/client identity headers are safe only when trust boundaries are explicit; otherwise spoofing risk is expected.  
   Source: RFC 7239 Forwarded header semantics  
   <https://www.rfc-editor.org/rfc/rfc7239>
2. Production proxy vendors explicitly warn that client-IP-like headers become spoof vectors if origin trust controls are not strict.  
   Source: Cloudflare True-Client-IP guidance  
   <https://developers.cloudflare.com/fundamentals/reference/http-request-headers/#true-client-ip-enterprise-plan-only>
3. Reverse-proxy trust should be bounded to trusted networks before accepting forwarded identity material.  
   Source: NGINX real_ip module (`set_real_ip_from`)  
   <https://nginx.org/en/docs/http/ngx_http_realip_module.html>
4. Log-management guidance emphasizes integrity and trustworthiness of telemetry sources for security operations value.  
   Source: NIST SP 800-92  
   <https://csrc.nist.gov/pubs/sp/800/92/final>
5. HMAC-based message authentication provides low-cost authenticity guarantees suitable for bounded metadata integrity checks.  
   Source: RFC 2104  
   <https://www.rfc-editor.org/rfc/rfc2104>

## Addressing Options

1. Keep unsigned headers; rely on dev-only runtime guard.
2. Add simulation metadata signature (HMAC) with timestamp/nonce and strict validation.
3. Move all simulation telemetry onto separate ingress endpoint/process and remove per-request tagging.

## Recommended Direction

Adopt option 2 now (strong authenticity, minimal architecture disruption), and preserve option 3 for future high-assurance isolation.

Recommended controls:

1. Add env-only dev/test secret for telemetry signing (for example `SHUMA_SIM_TELEMETRY_SECRET`).
2. Require additional metadata headers:
   - signature,
   - issued-at timestamp,
   - nonce.
3. Verify signature and freshness window before entering simulation context.
4. Reject/ignore invalid tags and emit explicit `sim_tag_invalid_signature` telemetry.
5. Keep production hard-off behavior unchanged.

## Success Signals

1. Unsigned or stale simulation headers never route to simulation data plane.
2. Signed headers from valid runner process continue to tag correctly.
3. Monitoring and event logs clearly expose rejected simulation-tag attempts for operator awareness.

## Source Links

1. RFC 7239 Forwarded: <https://www.rfc-editor.org/rfc/rfc7239>
2. Cloudflare header trust note: <https://developers.cloudflare.com/fundamentals/reference/http-request-headers/#true-client-ip-enterprise-plan-only>
3. NGINX real_ip module: <https://nginx.org/en/docs/http/ngx_http_realip_module.html>
4. NIST SP 800-92: <https://csrc.nist.gov/pubs/sp/800/92/final>
5. RFC 2104 HMAC: <https://www.rfc-editor.org/rfc/rfc2104>
