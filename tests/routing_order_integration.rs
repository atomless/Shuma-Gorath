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
    let mut builder = Request::builder();
    builder.method(method).uri(path);
    for (key, value) in headers {
        builder.header(*key, *value);
    }
    builder.body(Vec::new());
    builder.build()
}

fn with_runtime_env<T>(f: impl FnOnce() -> T) -> T {
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
    ];
    for (key, value) in vars {
        std::env::set_var(key, value);
    }
    std::env::remove_var("SHUMA_HEALTH_SECRET");
    std::env::remove_var("SHUMA_ADMIN_IP_ALLOWLIST");
    f()
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
        assert_eq!(
            String::from_utf8_lossy(resp.body()),
            "OK (passed bot defence)"
        );
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
