use spin_sdk::http::{Method, Request, Response};

use crate::runtime::upstream_canonicalization::{
    canonicalize_forward_path, is_privileged_request_header, normalize_header_name,
    parse_connection_tokens, should_strip_request_header, should_strip_response_header,
};
use crate::runtime::upstream_telemetry::{sanitize_forward_reason, ForwardProvenance};

const LOOP_HOP_HEADER: &str = "x-shuma-gateway-hop";
const FALLBACK_FAILURE_BODY: &str = "Gateway forwarding unavailable";
const FORWARD_REASON_HEADER: &str = "x-shuma-forward-reason";
const MAX_FORWARD_REQUEST_BODY_BYTES: usize = 1_048_576;
const MAX_FORWARD_RESPONSE_BODY_BYTES: usize = 2_097_152;
#[cfg(not(target_arch = "wasm32"))]
const NATIVE_TEST_MODE_ENV: &str = "SHUMA_GATEWAY_NATIVE_TEST_MODE";

pub(crate) struct ForwardResult {
    pub response: Response,
    pub failure_class: Option<&'static str>,
}

#[derive(Clone, Copy)]
pub(crate) struct ForwardRequestContext<'a> {
    pub req: &'a Request,
    pub ip: &'a str,
}

#[derive(Debug, Clone)]
struct UpstreamOrigin {
    scheme: String,
    authority: String,
    host: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForwardFailureClass {
    Timeout,
    Transport,
    PolicyDenied,
    Misconfiguration,
    LoopDetected,
}

impl ForwardFailureClass {
    fn as_str(self) -> &'static str {
        match self {
            ForwardFailureClass::Timeout => "timeout",
            ForwardFailureClass::Transport => "transport",
            ForwardFailureClass::PolicyDenied => "policy_denied",
            ForwardFailureClass::Misconfiguration => "misconfiguration",
            ForwardFailureClass::LoopDetected => "loop_detected",
        }
    }

    fn status_code(self) -> u16 {
        match self {
            ForwardFailureClass::Timeout => 504,
            ForwardFailureClass::Transport => 502,
            ForwardFailureClass::PolicyDenied => 403,
            ForwardFailureClass::Misconfiguration => 500,
            ForwardFailureClass::LoopDetected => 508,
        }
    }
}

fn method_label(method: &Method) -> String {
    format!("{:?}", method).to_ascii_uppercase()
}

fn normalize_authority(raw: &str) -> String {
    raw.trim().trim_end_matches('.').to_ascii_lowercase()
}

fn split_authority_and_suffix(raw: &str) -> (String, String) {
    let cut = raw.find(['/', '?', '#']).unwrap_or(raw.len());
    let authority = raw[..cut].trim().to_string();
    let suffix = if cut < raw.len() {
        raw[cut..].to_string()
    } else {
        "/".to_string()
    };
    (authority, suffix)
}

fn host_without_port(authority: &str) -> String {
    let trimmed = authority.trim();
    if let Some(rest) = trimmed.strip_prefix('[') {
        if let Some(end) = rest.find(']') {
            return rest[..end].to_string();
        }
    }
    if trimmed.matches(':').count() == 1 {
        return trimmed.split(':').next().unwrap_or("").to_string();
    }
    trimmed.to_string()
}

fn normalize_upstream_origin(raw: &str) -> Result<UpstreamOrigin, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("missing SHUMA_GATEWAY_UPSTREAM_ORIGIN".to_string());
    }
    let (scheme_raw, authority_raw) = trimmed
        .split_once("://")
        .ok_or_else(|| "upstream origin must include scheme://authority".to_string())?;
    let scheme = scheme_raw.trim().to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https") {
        return Err("upstream origin scheme must be http or https".to_string());
    }
    if authority_raw.is_empty()
        || authority_raw.contains('/')
        || authority_raw.contains('?')
        || authority_raw.contains('#')
        || authority_raw.contains('@')
    {
        return Err("upstream origin must not include path, query, fragment, or userinfo".to_string());
    }
    let authority = normalize_authority(authority_raw);
    if authority.is_empty() {
        return Err("upstream origin authority must not be empty".to_string());
    }
    let host = host_without_port(authority.as_str());
    if host.is_empty() {
        return Err("upstream origin host must not be empty".to_string());
    }
    Ok(UpstreamOrigin {
        scheme,
        authority,
        host,
    })
}

fn parse_inbound_hop(raw: Option<&str>) -> Result<u8, String> {
    let Some(value) = raw else {
        return Ok(0);
    };
    let parsed = value
        .trim()
        .parse::<u8>()
        .map_err(|_| "invalid inbound hop marker".to_string())?;
    Ok(parsed)
}

fn next_hop_marker(inbound_hop: u8, max_hops: u8) -> Result<u8, ForwardFailureClass> {
    let next = inbound_hop.saturating_add(1);
    if next > max_hops {
        return Err(ForwardFailureClass::LoopDetected);
    }
    Ok(next)
}

fn classify_transport_error(error: &str) -> ForwardFailureClass {
    let normalized = error.to_ascii_lowercase();
    if normalized.contains("timeout") || normalized.contains("timed out") {
        ForwardFailureClass::Timeout
    } else {
        ForwardFailureClass::Transport
    }
}

fn failure_response(class: ForwardFailureClass, upstream_origin: &str, reason: &str) -> ForwardResult {
    let provenance = ForwardProvenance::new(class.as_str(), upstream_origin, reason);
    crate::log_line(&format!(
        "[gateway-forward] failed status={} class={} reason={} {}",
        class.status_code(),
        class.as_str(),
        reason,
        provenance.as_event_fields()
    ));
    ForwardResult {
        response: Response::new(class.status_code(), FALLBACK_FAILURE_BODY),
        failure_class: Some(class.as_str()),
    }
}

fn is_upgrade_request(context: ForwardRequestContext<'_>) -> bool {
    if method_label(context.req.method()) == "CONNECT" {
        return true;
    }
    if context.req.header("upgrade").is_some() {
        return true;
    }
    let connection_tokens = context
        .req
        .header("connection")
        .and_then(|value| value.as_str())
        .map(parse_connection_tokens)
        .unwrap_or_default();
    connection_tokens
        .iter()
        .any(|token| token.eq_ignore_ascii_case("upgrade"))
}

fn trusted_public_host(req: &Request) -> Option<String> {
    req.header("host")
        .and_then(|value| value.as_str())
        .map(|raw| raw.trim().to_string())
        .filter(|value| {
            !value.is_empty()
                && value
                    .chars()
                    .all(|ch| ch.is_ascii() && !ch.is_ascii_control() && ch != ' ')
        })
}

fn cookie_domain_from_public_host(public_host: Option<&str>) -> Option<String> {
    let host = public_host
        .map(host_without_port)
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_default();
    if host.is_empty() || host.contains(':') {
        return None;
    }
    Some(host)
}

fn host_matches_cookie_domain(host: &str, domain: &str) -> bool {
    let host = host.trim().to_ascii_lowercase();
    let domain = domain.trim().trim_start_matches('.').to_ascii_lowercase();
    if host.is_empty() || domain.is_empty() {
        return false;
    }
    host == domain || host.ends_with(format!(".{}", domain).as_str())
}

fn canonicalize_set_cookie(
    raw_cookie: &str,
    upstream_host: &str,
    public_cookie_domain: Option<&str>,
) -> Option<String> {
    let parts: Vec<&str> = raw_cookie.split(';').collect();
    let base = parts.first().map(|part| part.trim()).unwrap_or("");
    if base.is_empty() {
        return None;
    }

    let mut output = vec![base.to_string()];
    for attr in parts.iter().skip(1) {
        let trimmed = attr.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((key_raw, value_raw)) = trimmed.split_once('=') else {
            output.push(trimmed.to_string());
            continue;
        };
        if !key_raw.trim().eq_ignore_ascii_case("domain") {
            output.push(trimmed.to_string());
            continue;
        }

        let domain = value_raw.trim();
        if !host_matches_cookie_domain(upstream_host, domain) {
            return None;
        }
        let rewritten = public_cookie_domain?;
        output.push(format!("Domain={}", rewritten));
    }

    Some(output.join("; "))
}

fn canonicalize_location_header(
    raw_location: &str,
    upstream_authority: &str,
    public_scheme: &str,
    public_host: Option<&str>,
) -> Result<Option<String>, String> {
    let trimmed = raw_location.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if trimmed.starts_with('/') && !trimmed.starts_with("//") {
        return Ok(Some(trimmed.to_string()));
    }

    let absolute_target = if let Some(rest) = trimmed.strip_prefix("//") {
        Some(rest)
    } else if let Some((_, rest)) = trimmed.split_once("://") {
        Some(rest)
    } else {
        None
    };

    let Some(target) = absolute_target else {
        return Ok(Some(trimmed.to_string()));
    };

    let (authority, suffix) = split_authority_and_suffix(target);
    if normalize_authority(authority.as_str()) != normalize_authority(upstream_authority) {
        return Err("redirect_policy_denied_cross_authority".to_string());
    }

    let Some(public_host) = public_host.map(|host| host.trim()).filter(|host| !host.is_empty()) else {
        return Ok(Some(trimmed.to_string()));
    };

    Ok(Some(format!(
        "{}://{}{}",
        public_scheme.to_ascii_lowercase(),
        public_host,
        suffix
    )))
}

fn format_forwarded_for(ip: &str) -> String {
    if ip.contains(':') {
        format!("for=\"[{}]\"", ip)
    } else {
        format!("for={}", ip)
    }
}

fn format_forwarded_header(ip: &str, proto: &str, host: Option<&str>) -> String {
    let mut value = format!("{};proto={}", format_forwarded_for(ip), proto);
    if let Some(host) = host {
        value.push_str(";host=\"");
        value.push_str(host);
        value.push('"');
    }
    value
}

#[cfg(target_arch = "wasm32")]
fn dispatch_outbound(request: Request) -> Result<Response, String> {
    spin_sdk::http::run(spin_sdk::http::send(request)).map_err(|err| err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn native_test_echo_response(request: &Request) -> Response {
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;

    let mut headers = BTreeMap::new();
    for (name, value) in request.headers() {
        if let Some(value) = value.as_str() {
            headers.insert(name.to_ascii_lowercase(), value.to_string());
        }
    }

    let body = request.body();
    let payload = serde_json::json!({
        "mode": "native_echo",
        "method": method_label(request.method()),
        "uri": request.uri(),
        "headers": headers,
        "body_len": body.len(),
        "body_sha256": format!("{:x}", Sha256::digest(body)),
    });
    let body_json = payload.to_string().into_bytes();

    let mut builder = Response::builder();
    builder
        .status(200)
        .header("content-type", "application/json; charset=utf-8")
        .header("cache-control", "no-store")
        .body(body_json)
        .build()
}

#[cfg(not(target_arch = "wasm32"))]
fn native_test_redirect_cookie_response() -> Response {
    Response::builder()
        .status(302)
        .header("location", "https://origin.example.com/redirected/path?from=fixture")
        .header(
            "set-cookie",
            "session=abc123; Path=/; Domain=origin.example.com; HttpOnly; Secure",
        )
        .header("x-shuma-forward-reason", "must-not-leak")
        .body(Vec::new())
        .build()
}

#[cfg(not(target_arch = "wasm32"))]
fn dispatch_outbound(request: Request) -> Result<Response, String> {
    let mode = std::env::var(NATIVE_TEST_MODE_ENV)
        .ok()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    if cfg!(test) {
        if mode == "redirect_cookie" {
            return Ok(native_test_redirect_cookie_response());
        }
        return Ok(native_test_echo_response(&request));
    }
    if mode == "echo" {
        return Ok(native_test_echo_response(&request));
    }
    if mode == "redirect_cookie" {
        return Ok(native_test_redirect_cookie_response());
    }

    Err("outbound forwarding is only available on wasm32 runtime".to_string())
}

fn canonicalize_upstream_response(
    context: ForwardRequestContext<'_>,
    upstream: &UpstreamOrigin,
    response: Response,
    public_host: Option<&str>,
) -> Result<Response, (ForwardFailureClass, String)> {
    let body = response.body().to_vec();
    if body.len() > MAX_FORWARD_RESPONSE_BODY_BYTES {
        return Err((
            ForwardFailureClass::PolicyDenied,
            "response_body_too_large".to_string(),
        ));
    }

    let connection_header = response.header("connection").and_then(|value| value.as_str());
    let public_scheme = if crate::request_is_https(context.req) {
        "https"
    } else {
        "http"
    };
    let public_cookie_domain = cookie_domain_from_public_host(public_host);

    let mut response_builder = Response::builder();
    let mut builder = response_builder.status(*response.status());

    for (name, value) in response.headers() {
        let Some(value_str) = value.as_str() else {
            continue;
        };
        if should_strip_response_header(name, connection_header) {
            continue;
        }
        let Some(normalized_name) = normalize_header_name(name) else {
            continue;
        };

        if normalized_name == "location" {
            let location = canonicalize_location_header(
                value_str,
                upstream.authority.as_str(),
                public_scheme,
                public_host,
            )
            .map_err(|reason| (ForwardFailureClass::PolicyDenied, reason))?;
            if let Some(location) = location {
                builder = builder.header(name, location.as_str());
            }
            continue;
        }

        if normalized_name == "set-cookie" {
            if let Some(cookie) = canonicalize_set_cookie(
                value_str,
                upstream.host.as_str(),
                public_cookie_domain.as_deref(),
            ) {
                builder = builder.header(name, cookie.as_str());
            }
            continue;
        }

        builder = builder.header(name, value_str);
    }

    Ok(builder.body(body).build())
}

pub(crate) fn forward_allow_request(context: ForwardRequestContext<'_>, reason: &str) -> ForwardResult {
    let upstream_origin_raw = crate::config::gateway_upstream_origin().unwrap_or_default();
    let upstream = match normalize_upstream_origin(upstream_origin_raw.as_str()) {
        Ok(value) => value,
        Err(message) => {
            return failure_response(
                ForwardFailureClass::Misconfiguration,
                upstream_origin_raw.as_str(),
                message.as_str(),
            );
        }
    };

    if is_upgrade_request(context) {
        return failure_response(
            ForwardFailureClass::PolicyDenied,
            upstream.authority.as_str(),
            "unsupported_upgrade_or_connect_request",
        );
    }

    let inbound_hop_raw = context
        .req
        .header(LOOP_HOP_HEADER)
        .and_then(|value| value.as_str());
    let inbound_hop = match parse_inbound_hop(inbound_hop_raw) {
        Ok(value) => value,
        Err(message) => {
            return failure_response(
                ForwardFailureClass::PolicyDenied,
                upstream.authority.as_str(),
                message.as_str(),
            );
        }
    };
    let next_hop = match next_hop_marker(inbound_hop, crate::config::gateway_loop_max_hops()) {
        Ok(value) => value,
        Err(class) => {
            return failure_response(class, upstream.authority.as_str(), "loop hop cap exceeded");
        }
    };

    let canonical_path = canonicalize_forward_path(context.req.uri());
    let target_uri = format!("{}://{}{}", upstream.scheme, upstream.authority, canonical_path);

    let public_host = trusted_public_host(context.req);
    let inbound_scheme = if crate::request_is_https(context.req) {
        "https"
    } else {
        "http"
    };
    let forwarded_header = format_forwarded_header(context.ip, inbound_scheme, public_host.as_deref());
    let safe_reason = sanitize_forward_reason(reason);

    let origin_auth_header_lower = normalize_header_name(crate::config::gateway_origin_auth_header_name().as_str());

    let mut request_builder = Request::builder();
    let mut builder = request_builder
        .method(context.req.method().clone())
        .uri(target_uri.as_str());

    let connection_header = context
        .req
        .header("connection")
        .and_then(|value| value.as_str());

    for (name, value) in context.req.headers() {
        let Some(value_str) = value.as_str() else {
            continue;
        };
        if should_strip_request_header(name, connection_header) {
            continue;
        }
        if is_privileged_request_header(name) {
            continue;
        }
        let Some(normalized_name) = normalize_header_name(name) else {
            continue;
        };
        if matches!(normalized_name.as_str(), "host" | "content-length") {
            continue;
        }
        if origin_auth_header_lower
            .as_deref()
            .map(|header| header == normalized_name)
            .unwrap_or(false)
        {
            continue;
        }
        builder = builder.header(name, value_str);
    }

    builder = builder
        .header(LOOP_HOP_HEADER, next_hop.to_string())
        .header("forwarded", forwarded_header.as_str())
        .header("x-forwarded-proto", inbound_scheme)
        .header("x-forwarded-for", context.ip)
        .header(FORWARD_REASON_HEADER, safe_reason.as_str());

    if let Some(host) = public_host.as_deref() {
        builder = builder.header("x-forwarded-host", host);
    }

    if matches!(
        crate::config::gateway_origin_auth_mode(),
        crate::config::GatewayOriginAuthMode::SignedHeader
    ) {
        let header_name = crate::config::gateway_origin_auth_header_name();
        let header_value = crate::config::gateway_origin_auth_header_value();
        if !header_name.is_empty() && !header_value.is_empty() {
            builder = builder.header(header_name.as_str(), header_value.as_str());
        }
    }

    let request_body = context.req.body().to_vec();
    if request_body.len() > MAX_FORWARD_REQUEST_BODY_BYTES {
        return failure_response(
            ForwardFailureClass::PolicyDenied,
            upstream.authority.as_str(),
            "request_body_too_large",
        );
    }

    let outbound_request = builder.body(request_body).build();
    let upstream_response = match dispatch_outbound(outbound_request) {
        Ok(response) => response,
        Err(error) => {
            let class = classify_transport_error(error.as_str());
            return failure_response(class, upstream.authority.as_str(), error.as_str());
        }
    };

    match canonicalize_upstream_response(
        context,
        &upstream,
        upstream_response,
        public_host.as_deref(),
    ) {
        Ok(response) => ForwardResult {
            response,
            failure_class: None,
        },
        Err((class, reason)) => failure_response(class, upstream.authority.as_str(), reason.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canonicalize_location_header, canonicalize_set_cookie, canonicalize_upstream_response,
        classify_transport_error, forward_allow_request, host_matches_cookie_domain,
        next_hop_marker, normalize_upstream_origin, parse_inbound_hop, split_authority_and_suffix,
        ForwardFailureClass, ForwardRequestContext, UpstreamOrigin,
    };
    use spin_sdk::http::{Method, Request, Response};

    fn build_request(uri: &str, headers: &[(&str, &str)], body: &[u8]) -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Get).uri(uri);
        for (name, value) in headers {
            builder.header(*name, *value);
        }
        builder.body(body.to_vec());
        builder.build()
    }

    #[test]
    fn normalizes_upstream_origin_and_rejects_path_segments() {
        let upstream = normalize_upstream_origin("HTTPS://Origin.Example.com:443").unwrap();
        assert_eq!(upstream.scheme, "https");
        assert_eq!(upstream.authority, "origin.example.com:443");
        assert_eq!(upstream.host, "origin.example.com");
        assert!(normalize_upstream_origin("https://origin.example.com/path").is_err());
    }

    #[test]
    fn inbound_hop_marker_parsing_defaults_and_rejects_invalid_values() {
        assert_eq!(parse_inbound_hop(None), Ok(0));
        assert_eq!(parse_inbound_hop(Some("2")), Ok(2));
        assert!(parse_inbound_hop(Some("bogus")).is_err());
    }

    #[test]
    fn next_hop_marker_enforces_max_hop_guard() {
        assert_eq!(next_hop_marker(0, 3), Ok(1));
        assert_eq!(
            next_hop_marker(3, 3),
            Err(ForwardFailureClass::LoopDetected)
        );
    }

    #[test]
    fn classify_transport_error_prefers_timeout_class() {
        assert_eq!(
            classify_transport_error("request timed out"),
            ForwardFailureClass::Timeout
        );
        assert_eq!(
            classify_transport_error("connection reset by peer"),
            ForwardFailureClass::Transport
        );
        assert_eq!(
            classify_transport_error("certificate verify failed"),
            ForwardFailureClass::Transport
        );
    }

    #[test]
    fn canonicalize_location_blocks_cross_authority_redirects() {
        let result = canonicalize_location_header(
            "https://evil.example.com/path",
            "origin.example.com:443",
            "https",
            Some("public.example.com"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn canonicalize_location_rewrites_same_origin_absolute_to_public_host() {
        let result = canonicalize_location_header(
            "https://origin.example.com:443/foo?bar=1",
            "origin.example.com:443",
            "https",
            Some("public.example.com"),
        )
        .unwrap();
        assert_eq!(
            result.as_deref(),
            Some("https://public.example.com/foo?bar=1")
        );
    }

    #[test]
    fn canonicalize_set_cookie_rewrites_origin_domain_to_public_domain() {
        let rewritten = canonicalize_set_cookie(
            "session=abc; Path=/; Domain=origin.example.com; HttpOnly",
            "origin.example.com",
            Some("public.example.com"),
        )
        .unwrap();
        assert!(rewritten.contains("Domain=public.example.com"));
        assert!(rewritten.contains("HttpOnly"));
    }

    #[test]
    fn canonicalize_set_cookie_drops_foreign_domain() {
        assert!(
            canonicalize_set_cookie(
                "session=abc; Domain=evil.example.com; Path=/",
                "origin.example.com",
                Some("public.example.com"),
            )
            .is_none()
        );
    }

    #[test]
    fn cookie_domain_matching_accepts_suffix_relationships() {
        assert!(host_matches_cookie_domain(
            "app.origin.example.com",
            "origin.example.com"
        ));
        assert!(!host_matches_cookie_domain("origin.example.com", "evil.example.com"));
    }

    #[test]
    fn split_authority_handles_suffixless_absolute_target() {
        let (authority, suffix) = split_authority_and_suffix("origin.example.com:443");
        assert_eq!(authority, "origin.example.com:443");
        assert_eq!(suffix, "/");
    }

    #[test]
    fn response_canonicalization_rewrites_location_cookie_and_strips_internal_headers() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
        let req = build_request(
            "/allow",
            &[
                ("host", "public.example.com"),
                ("x-shuma-forwarded-secret", "test-forwarded-secret"),
                ("x-forwarded-for", "198.51.100.42"),
                ("x-forwarded-proto", "https"),
            ],
            &[],
        );
        let context = ForwardRequestContext {
            req: &req,
            ip: "198.51.100.42",
        };
        let upstream = UpstreamOrigin {
            scheme: "https".to_string(),
            authority: "origin.example.com:443".to_string(),
            host: "origin.example.com".to_string(),
        };
        let response = Response::builder()
            .status(302)
            .header("Location", "https://origin.example.com:443/redirected")
            .header(
                "Set-Cookie",
                "session=abc; Path=/; Domain=origin.example.com; HttpOnly",
            )
            .header("Connection", "x-secret-hop")
            .header("X-Secret-Hop", "drop-me")
            .header("X-Shuma-Forward-Reason", "drop-me")
            .header("X-Forwarded-For", "drop-me")
            .header("Content-Type", "text/html; charset=utf-8")
            .body(b"<html>ok</html>".to_vec())
            .build();

        let canonical = canonicalize_upstream_response(
            context,
            &upstream,
            response,
            Some("public.example.com"),
        )
        .expect("response should canonicalize");

        assert_eq!(*canonical.status(), 302);
        let location = canonical
            .headers()
            .find(|(name, _)| name.eq_ignore_ascii_case("location"))
            .and_then(|(_, value)| value.as_str())
            .unwrap_or("");
        assert_eq!(location, "http://public.example.com/redirected");
        let cookie = canonical
            .headers()
            .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
            .and_then(|(_, value)| value.as_str())
            .unwrap_or("");
        assert!(cookie.contains("Domain=public.example.com"));
        assert!(canonical
            .headers()
            .all(|(name, _)| !name.eq_ignore_ascii_case("x-shuma-forward-reason")));
        assert!(canonical
            .headers()
            .all(|(name, _)| !name.eq_ignore_ascii_case("x-forwarded-for")));
        assert!(canonical
            .headers()
            .all(|(name, _)| !name.eq_ignore_ascii_case("x-secret-hop")));
        std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    }

    #[test]
    fn response_canonicalization_denies_cross_authority_redirect_targets() {
        let req = build_request(
            "/allow",
            &[("host", "public.example.com"), ("x-forwarded-proto", "https")],
            &[],
        );
        let context = ForwardRequestContext {
            req: &req,
            ip: "198.51.100.42",
        };
        let upstream = UpstreamOrigin {
            scheme: "https".to_string(),
            authority: "origin.example.com:443".to_string(),
            host: "origin.example.com".to_string(),
        };
        let response = Response::builder()
            .status(302)
            .header("Location", "https://evil.example.com/redirected")
            .body(Vec::new())
            .build();
        let result = canonicalize_upstream_response(
            context,
            &upstream,
            response,
            Some("public.example.com"),
        );
        assert!(matches!(
            result,
            Err((ForwardFailureClass::PolicyDenied, reason))
            if reason.contains("redirect_policy_denied")
        ));
    }

    #[test]
    fn signed_header_origin_auth_is_proxy_owned_and_overrides_client_value() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "https://origin.example.com");
        std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_MODE", "signed_header");
        std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME", "x-origin-auth");
        std::env::set_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE", "proxy-secret");

        let req = build_request(
            "/allow",
            &[
                ("host", "public.example.com"),
                ("x-shuma-forwarded-secret", "test-forwarded-secret"),
                ("x-forwarded-for", "198.51.100.99"),
                ("x-origin-auth", "attacker-supplied"),
            ],
            br#"{"ok":true}"#,
        );
        let forward = forward_allow_request(
            ForwardRequestContext {
                req: &req,
                ip: "198.51.100.99",
            },
            "policy_clean_allow",
        );
        assert!(forward.failure_class.is_none());
        let payload: serde_json::Value =
            serde_json::from_slice(forward.response.body()).expect("native echo json");
        let injected = payload["headers"]["x-origin-auth"]
            .as_str()
            .unwrap_or_default();
        assert_eq!(injected, "proxy-secret");

        std::env::remove_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE");
        std::env::remove_var("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME");
        std::env::remove_var("SHUMA_GATEWAY_ORIGIN_AUTH_MODE");
        std::env::remove_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN");
    }
}
