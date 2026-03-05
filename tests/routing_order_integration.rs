use once_cell::sync::Lazy;
use spin_sdk::http::{Method, Request};
use std::sync::{Mutex, MutexGuard};

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn lock_env() -> MutexGuard<'static, ()> {
    ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn request(method: Method, path: &str, headers: &[(&str, &str)]) -> Request {
    request_with_body(method, path, headers, &[])
}

fn request_with_body(method: Method, path: &str, headers: &[(&str, &str)], body: &[u8]) -> Request {
    let mut builder = Request::builder();
    builder.method(method).uri(path);
    for (key, value) in headers {
        builder.header(*key, *value);
    }
    builder.body(body.to_vec());
    builder.build()
}

fn with_runtime_env<T>(f: impl FnOnce() -> T) -> T {
    with_runtime_env_overrides(&[], f)
}

fn with_runtime_env_overrides<T>(overrides: &[(&str, Option<&str>)], f: impl FnOnce() -> T) -> T {
    let _lock = lock_env();
    let vars = [
        ("SHUMA_API_KEY", "test-admin-key"),
        ("SHUMA_JS_SECRET", "test-js-secret"),
        ("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret"),
        ("SHUMA_EVENT_LOG_RETENTION_HOURS", "168"),
        ("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "false"),
        ("SHUMA_KV_STORE_FAIL_OPEN", "true"),
        ("SHUMA_ENFORCE_HTTPS", "false"),
        ("SHUMA_DEBUG_HEADERS", "false"),
        ("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "https://origin.example.com"),
        ("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server"),
        ("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL", "false"),
        ("SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS", "false"),
        ("SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST", ""),
        ("SHUMA_GATEWAY_PUBLIC_AUTHORITIES", "shuma.example.com:443"),
        ("SHUMA_GATEWAY_LOOP_MAX_HOPS", "3"),
        ("SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED", "true"),
        ("SHUMA_GATEWAY_ORIGIN_AUTH_MODE", "network_only"),
        ("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME", ""),
        ("SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE", ""),
        ("SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS", "90"),
        ("SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS", "7"),
        ("SHUMA_GATEWAY_TLS_STRICT", "true"),
        (
            "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
            "true",
        ),
        ("SHUMA_GATEWAY_NATIVE_TEST_MODE", "echo"),
    ];
    for (key, value) in vars {
        std::env::set_var(key, value);
    }
    for (key, value) in overrides {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
    std::env::remove_var("SHUMA_HEALTH_SECRET");
    std::env::remove_var("SHUMA_ADMIN_IP_ALLOWLIST");
    f()
}

fn response_body_string(resp: &spin_sdk::http::Response) -> String {
    String::from_utf8_lossy(resp.body()).to_string()
}

fn response_json(resp: &spin_sdk::http::Response) -> serde_json::Value {
    serde_json::from_slice(resp.body()).expect("response body must be json")
}

fn json_header<'a>(payload: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    payload
        .get("headers")
        .and_then(|headers| headers.get(key))
        .and_then(|value| value.as_str())
}

#[test]
fn admin_options_is_rejected_before_main_pipeline() {
    with_runtime_env(|| {
        let req = request(
            Method::Options,
            "/admin/config",
            &[
                ("origin", "https://example.com"),
                ("access-control-request-method", "POST"),
            ],
        );
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 403u16);
        assert_eq!(String::from_utf8_lossy(resp.body()), "Forbidden");
    });
}

#[test]
fn admin_route_requires_auth_even_with_fail_open_enabled() {
    with_runtime_env(|| {
        let req = request(Method::Get, "/admin/config", &[]);
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        // Regression guard: /admin should be handled by the early router/admin adapter,
        // not fall through to KV fail-open bypass behavior.
        assert_eq!(*resp.status(), 401u16);
        assert_eq!(
            String::from_utf8_lossy(resp.body()),
            "Unauthorized: Invalid or missing API key"
        );
    });
}

#[test]
fn health_route_precedes_kv_fail_open_bypass() {
    with_runtime_env(|| {
        let req = request(Method::Get, "/health", &[]);
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        // Regression guard: /health should evaluate local/trusted-IP access first.
        assert_eq!(*resp.status(), 403u16);
        assert_eq!(String::from_utf8_lossy(resp.body()), "Forbidden");
    });
}

#[test]
fn static_asset_path_bypasses_expensive_bot_checks() {
    with_runtime_env(|| {
        let req = request(Method::Get, "/assets/app.bundle.js", &[]);
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 200u16);
        let body = String::from_utf8_lossy(resp.body());
        assert!(body.contains("\"mode\":\"native_echo\""), "body={}", body);
        assert!(
            body.contains("\"x-shuma-forward-reason\":\"static_asset_bypass\""),
            "body={}",
            body
        );
    });
}

#[test]
fn allow_path_forwards_fidelity_and_regenerates_trusted_forwarded_headers() {
    with_runtime_env(|| {
        let req = request_with_body(
            Method::Get,
            "/assets/catalog.json?alpha=1&beta=2",
            &[
                ("host", "public.example.com"),
                ("connection", "keep-alive, x-remove-me"),
                ("x-remove-me", "remove-this-hop-header"),
                ("authorization", "Bearer attacker-token"),
                ("forwarded", r#"for=1.1.1.1;proto=http;host="evil.example""#),
                ("x-forwarded-for", "198.51.100.77"),
                ("x-forwarded-host", "evil.example"),
                ("x-forwarded-proto", "http"),
                ("x-shuma-forwarded-secret", "test-forwarded-secret"),
                ("x-shuma-admin-api-key", "leak-attempt"),
            ],
            &[],
        );
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 200u16);
        let payload = response_json(&resp);
        assert_eq!(payload["mode"], "native_echo");
        assert!(
            payload["method"]
                .as_str()
                .unwrap_or_default()
                .contains("GET")
        );
        assert_eq!(
            payload["uri"],
            "https://origin.example.com/assets/catalog.json?alpha=1&beta=2"
        );
        assert_eq!(
            json_header(&payload, "x-shuma-forward-reason"),
            Some("static_asset_bypass")
        );
        assert_eq!(json_header(&payload, "x-shuma-gateway-hop"), Some("1"));
        assert_eq!(json_header(&payload, "x-forwarded-for"), Some("198.51.100.77"));
        assert_eq!(
            json_header(&payload, "x-forwarded-host"),
            Some("public.example.com")
        );
        assert_eq!(json_header(&payload, "x-forwarded-proto"), Some("http"));
        assert_eq!(
            json_header(&payload, "forwarded"),
            Some(r#"for=198.51.100.77;proto=http;host="public.example.com""#)
        );
        assert!(
            json_header(&payload, "host").is_none(),
            "proxy must not forward Host header"
        );
        assert!(
            json_header(&payload, "authorization").is_none(),
            "proxy must strip privileged request auth"
        );
        assert!(
            json_header(&payload, "x-remove-me").is_none(),
            "proxy must strip connection-token hop header"
        );
        assert!(
            json_header(&payload, "x-shuma-admin-api-key").is_none(),
            "proxy must strip x-shuma-* privileged headers"
        );
        assert_eq!(payload["body_len"], 0);
    });
}

#[test]
fn allow_path_rewrites_redirect_location_and_cookie_domain_from_upstream() {
    with_runtime_env_overrides(&[("SHUMA_GATEWAY_NATIVE_TEST_MODE", Some("redirect_cookie"))], || {
        let req = request(
            Method::Get,
            "/assets/app.js",
            &[("host", "public.example.com"), ("x-forwarded-for", "198.51.100.77")],
        );
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 302u16);
        let location = resp
            .headers()
            .find(|(name, _)| name.eq_ignore_ascii_case("location"))
            .and_then(|(_, value)| value.as_str())
            .unwrap_or("");
        assert_eq!(
            location,
            "http://public.example.com/redirected/path?from=fixture"
        );
        let set_cookie = resp
            .headers()
            .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
            .and_then(|(_, value)| value.as_str())
            .unwrap_or("");
        assert!(set_cookie.contains("Domain=public.example.com"));
        assert!(resp
            .headers()
            .all(|(name, _)| !name.eq_ignore_ascii_case("x-shuma-forward-reason")));
    });
}

#[test]
fn enforcement_paths_remain_local_and_do_not_require_upstream() {
    with_runtime_env_overrides(
        &[
            ("SHUMA_RUNTIME_ENV", Some("runtime-dev")),
            ("SHUMA_GATEWAY_UPSTREAM_ORIGIN", Some("")),
            ("SHUMA_GATEWAY_NATIVE_TEST_MODE", None),
        ],
        || {
            let req = request(Method::Get, "/health", &[("host", "public.example.com")]);
            let resp = shuma_gorath::handle_bot_defence_impl(&req);
            let body = response_body_string(&resp);

            assert_eq!(*resp.status(), 403u16);
            assert!(
                !body.contains("\"mode\":\"native_echo\""),
                "block/challenge paths must remain local and must not depend on upstream forwarding"
            );
        },
    );
}

#[test]
fn allow_paths_fail_closed_when_upstream_forwarding_is_unavailable() {
    with_runtime_env_overrides(&[("SHUMA_GATEWAY_NATIVE_TEST_MODE", None)], || {
        let req = request(Method::Get, "/assets/app.js", &[("host", "public.example.com")]);
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 502u16);
        assert_eq!(response_body_string(&resp), "Gateway forwarding unavailable");
    });
}

#[test]
fn loop_hop_guard_blocks_allow_path_when_hop_budget_exceeded() {
    with_runtime_env(|| {
        let req = request(
            Method::Get,
            "/assets/app.js",
            &[("host", "public.example.com"), ("x-shuma-gateway-hop", "3")],
        );
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 508u16);
        assert_eq!(response_body_string(&resp), "Gateway forwarding unavailable");
    });
}

#[test]
fn upgrade_requests_are_explicitly_unsupported_in_gateway_v1() {
    with_runtime_env(|| {
        let req = request(
            Method::Get,
            "/assets/app.js",
            &[
                ("host", "public.example.com"),
                ("connection", "Upgrade"),
                ("upgrade", "websocket"),
            ],
        );
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 403u16);
        assert_eq!(response_body_string(&resp), "Gateway forwarding unavailable");
    });
}

#[test]
fn dashboard_root_path_redirects_to_index_shell() {
    with_runtime_env(|| {
        let req = request(Method::Get, "/dashboard", &[]);
        let resp = shuma_gorath::handle_bot_defence_impl(&req);

        assert_eq!(*resp.status(), 308u16);
        let location = resp
            .headers()
            .find(|(name, _)| name.eq_ignore_ascii_case("location"))
            .and_then(|(_, value)| value.as_str())
            .unwrap_or("");
        assert_eq!(location, "/dashboard/index.html");
    });
}
