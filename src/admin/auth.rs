use crate::challenge::KeyValueStore;
use crate::signals::allowlist;
use rand::Rng as _;
use serde::{Deserialize, Serialize};
use spin_sdk::http::{Method, Request};
use spin_sdk::key_value::Store;
use std::time::{SystemTime, UNIX_EPOCH};

const INSECURE_DEFAULT_API_KEY: &str = "changeme-supersecret";
const ADMIN_SESSION_COOKIE_NAME: &str = "shuma_admin_session";
const ADMIN_SESSION_KEY_PREFIX: &str = "admin_session:";
const ADMIN_SESSION_TTL_SECONDS: u64 = 3600;
const ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE_DEFAULT: u32 = 10;
const ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE_MIN: u32 = 1;
const ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE_MAX: u32 = 10_000;
const ADMIN_AUTH_FAILURE_SITE_LOGIN: &str = "admin-auth-login";
const ADMIN_AUTH_FAILURE_SITE_ENDPOINT: &str = "admin-auth-endpoint";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdminSessionRecord {
    csrf_token: String,
    expires_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminAuthMethod {
    BearerToken,
    SessionCookie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminAccessLevel {
    ReadOnly,
    ReadWrite,
}

impl AdminAccessLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            AdminAccessLevel::ReadOnly => "read_only",
            AdminAccessLevel::ReadWrite => "read_write",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAuthResult {
    pub method: Option<AdminAuthMethod>,
    pub access: Option<AdminAccessLevel>,
    pub csrf_token: Option<String>,
    pub session_id: Option<String>,
    pub session_expires_at: Option<u64>,
}

impl AdminAuthResult {
    pub fn unauthorized() -> Self {
        Self {
            method: None,
            access: None,
            csrf_token: None,
            session_id: None,
            session_expires_at: None,
        }
    }

    pub fn is_authorized(&self) -> bool {
        self.method.is_some()
    }

    pub fn requires_csrf(&self, req: &Request) -> bool {
        self.method == Some(AdminAuthMethod::SessionCookie) && method_is_write(req.method())
    }

    pub fn is_write_authorized(&self) -> bool {
        self.access == Some(AdminAccessLevel::ReadWrite)
    }

    pub fn access_label(&self) -> &'static str {
        match self.access {
            Some(level) => level.as_str(),
            None => "none",
        }
    }

    pub fn audit_actor_label(&self) -> &'static str {
        match (self.method, self.access) {
            (Some(AdminAuthMethod::BearerToken), Some(AdminAccessLevel::ReadOnly)) => {
                "admin_bearer_ro"
            }
            (Some(AdminAuthMethod::BearerToken), Some(AdminAccessLevel::ReadWrite)) => {
                "admin_bearer_rw"
            }
            (Some(AdminAuthMethod::SessionCookie), Some(AdminAccessLevel::ReadWrite)) => {
                "admin_session_rw"
            }
            _ => "-",
        }
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn method_is_write(method: &Method) -> bool {
    matches!(
        method,
        Method::Post | Method::Put | Method::Patch | Method::Delete
    )
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn random_hex(num_bytes: usize) -> String {
    let mut rng = rand::rng();
    let mut bytes = vec![0u8; num_bytes];
    rng.fill(bytes.as_mut_slice());
    to_hex(&bytes)
}

fn session_store_key(session_id: &str) -> String {
    format!("{}{}", ADMIN_SESSION_KEY_PREFIX, session_id)
}

fn parse_cookie(req: &Request, key: &str) -> Option<String> {
    let cookie_header = req.header("cookie")?.as_str()?;
    for part in cookie_header.split(';') {
        let trimmed = part.trim();
        let mut kv = trimmed.splitn(2, '=');
        let k = kv.next()?.trim();
        let v = kv.next()?.trim();
        if k == key && !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

fn get_admin_api_key() -> Option<String> {
    let key = crate::config::runtime_var_trimmed_optional("SHUMA_API_KEY")?;
    if key.is_empty() {
        return None;
    }
    if key == INSECURE_DEFAULT_API_KEY {
        return None;
    }
    Some(key)
}

fn get_admin_readonly_api_key() -> Option<String> {
    let key = crate::config::runtime_var_trimmed_optional("SHUMA_ADMIN_READONLY_API_KEY")?;
    if key.is_empty() {
        return None;
    }
    if key == INSECURE_DEFAULT_API_KEY {
        return None;
    }
    Some(key)
}

pub fn is_admin_api_key_configured() -> bool {
    get_admin_api_key().is_some()
}

fn classify_admin_api_key_candidate(candidate: &str) -> Option<AdminAccessLevel> {
    let candidate = candidate.trim();
    if candidate.is_empty() {
        return None;
    }
    if let Some(expected) = get_admin_api_key() {
        if constant_time_eq(candidate, &expected) {
            return Some(AdminAccessLevel::ReadWrite);
        }
    }
    if let Some(expected) = get_admin_readonly_api_key() {
        if constant_time_eq(candidate, &expected) {
            return Some(AdminAccessLevel::ReadOnly);
        }
    }
    None
}

pub fn verify_admin_api_key_candidate(candidate: &str) -> bool {
    classify_admin_api_key_candidate(candidate) == Some(AdminAccessLevel::ReadWrite)
}

fn bearer_token(req: &Request) -> Option<String> {
    let header = req.header("authorization")?.as_str()?;
    let prefix = "Bearer ";
    if !header.starts_with(prefix) {
        return None;
    }
    Some(header[prefix.len()..].trim().to_string())
}

pub fn bearer_access_level(req: &Request) -> Option<AdminAccessLevel> {
    let candidate = bearer_token(req)?;
    classify_admin_api_key_candidate(&candidate)
}

pub fn is_bearer_authorized(req: &Request) -> bool {
    bearer_access_level(req).is_some()
}

pub fn has_admin_session_cookie(req: &Request) -> bool {
    parse_cookie(req, ADMIN_SESSION_COOKIE_NAME).is_some()
}

pub fn get_admin_id(req: &Request) -> String {
    match bearer_access_level(req) {
        Some(AdminAccessLevel::ReadOnly) => "admin_ro".to_string(),
        Some(AdminAccessLevel::ReadWrite) => "admin_rw".to_string(),
        None if has_admin_session_cookie(req) => "admin_session".to_string(),
        None => "-".to_string(),
    }
}

fn load_session_record<S: KeyValueStore>(
    store: &S,
    session_id: &str,
) -> Option<AdminSessionRecord> {
    let key = session_store_key(session_id);
    let raw = store.get(&key).ok()??;
    let parsed = serde_json::from_slice::<AdminSessionRecord>(&raw).ok()?;
    if parsed.expires_at <= now_ts() {
        if let Err(e) = store.delete(&key) {
            eprintln!(
                "[auth] failed to delete expired admin session {}: {:?}",
                key, e
            );
        }
        return None;
    }
    Some(parsed)
}

pub fn authenticate_admin<S: KeyValueStore>(req: &Request, store: &S) -> AdminAuthResult {
    if let Some(access) = bearer_access_level(req) {
        return AdminAuthResult {
            method: Some(AdminAuthMethod::BearerToken),
            access: Some(access),
            csrf_token: None,
            session_id: None,
            session_expires_at: None,
        };
    }

    let Some(session_id) = parse_cookie(req, ADMIN_SESSION_COOKIE_NAME) else {
        return AdminAuthResult::unauthorized();
    };
    let Some(record) = load_session_record(store, &session_id) else {
        return AdminAuthResult::unauthorized();
    };
    AdminAuthResult {
        method: Some(AdminAuthMethod::SessionCookie),
        access: Some(AdminAccessLevel::ReadWrite),
        csrf_token: Some(record.csrf_token),
        session_id: Some(session_id),
        session_expires_at: Some(record.expires_at),
    }
}

pub fn validate_session_csrf(req: &Request, expected_csrf: &str) -> bool {
    let Some(header) = req.header("x-shuma-csrf").and_then(|v| v.as_str()) else {
        return false;
    };
    constant_time_eq(header.trim(), expected_csrf)
}

pub fn create_admin_session<S: KeyValueStore>(store: &S) -> Result<(String, String, u64), ()> {
    let session_id = random_hex(32);
    let csrf_token = random_hex(16);
    let expires_at = now_ts().saturating_add(ADMIN_SESSION_TTL_SECONDS);
    let record = AdminSessionRecord {
        csrf_token: csrf_token.clone(),
        expires_at,
    };
    let value = serde_json::to_vec(&record).map_err(|_| ())?;
    store.set(&session_store_key(&session_id), &value)?;
    Ok((session_id, csrf_token, expires_at))
}

pub fn clear_admin_session<S: KeyValueStore>(store: &S, req: &Request) -> Result<(), ()> {
    if let Some(session_id) = parse_cookie(req, ADMIN_SESSION_COOKIE_NAME) {
        store.delete(&session_store_key(&session_id))?;
    }
    Ok(())
}

pub fn admin_session_cookie_name() -> &'static str {
    ADMIN_SESSION_COOKIE_NAME
}

pub fn admin_session_ttl_seconds() -> u64 {
    ADMIN_SESSION_TTL_SECONDS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminAuthFailureScope {
    Login,
    Endpoint,
}

fn parse_admin_auth_failure_limit(value: Option<&str>) -> u32 {
    value
        .and_then(|raw| raw.trim().parse::<u32>().ok())
        .unwrap_or(ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE_DEFAULT)
        .clamp(
            ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE_MIN,
            ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE_MAX,
        )
}

pub fn admin_auth_failure_limit_per_minute() -> u32 {
    parse_admin_auth_failure_limit(
        crate::config::runtime_var_trimmed_optional("SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE")
            .as_deref(),
    )
}

fn admin_auth_failure_site(scope: AdminAuthFailureScope) -> &'static str {
    match scope {
        AdminAuthFailureScope::Login => ADMIN_AUTH_FAILURE_SITE_LOGIN,
        AdminAuthFailureScope::Endpoint => ADMIN_AUTH_FAILURE_SITE_ENDPOINT,
    }
}

/// Records a failed admin auth attempt and returns true when throttled.
pub fn register_admin_auth_failure<S: KeyValueStore>(
    store: &S,
    req: &Request,
    scope: AdminAuthFailureScope,
) -> bool {
    let ip = crate::extract_client_ip(req);
    let limit = admin_auth_failure_limit_per_minute();
    !crate::enforcement::rate::check_rate_limit(store, admin_auth_failure_site(scope), &ip, limit)
}

/// Records a failed admin auth attempt using the provider-selected rate limiter.
/// Returns true when throttled.
pub fn register_admin_auth_failure_with_provider(
    store: &Store,
    req: &Request,
    scope: AdminAuthFailureScope,
    provider_registry: &crate::providers::registry::ProviderRegistry,
) -> bool {
    let ip = crate::extract_client_ip(req);
    let limit = admin_auth_failure_limit_per_minute();
    provider_registry.rate_limiter_provider().check_rate_limit(
        store,
        admin_auth_failure_site(scope),
        &ip,
        limit,
    ) == crate::providers::contracts::RateLimitDecision::Limited
}

/// Returns true if admin access is allowed from this IP.
/// If SHUMA_ADMIN_IP_ALLOWLIST is unset, all IPs are allowed (auth still required).
pub fn is_admin_ip_allowed(req: &Request) -> bool {
    let Some(raw_allowlist) = crate::config::runtime_var_raw_optional("SHUMA_ADMIN_IP_ALLOWLIST") else {
        return true;
    };
    let list: Vec<String> = raw_allowlist
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if list.is_empty() {
        return true;
    }

    let ip = crate::extract_client_ip(req);
    allowlist::is_allowlisted(&ip, &list)
}

pub fn is_internal_adversary_sim_supervisor_request(req: &Request) -> bool {
    let marker = req
        .header("x-shuma-internal-supervisor")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .unwrap_or("");
    if marker != "adversary-sim" {
        return false;
    }

    if bearer_access_level(req) != Some(AdminAccessLevel::ReadWrite) {
        return false;
    }

    if !crate::forwarded_ip_trusted(req) || !crate::request_is_https(req) {
        return false;
    }

    req.header("x-forwarded-for")
        .and_then(|value| value.as_str())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .map(|value| value == "127.0.0.1" || value == "::1")
        .unwrap_or(false)
}

pub fn is_internal_oversight_supervisor_request(req: &Request) -> bool {
    let marker = req
        .header("x-shuma-internal-supervisor")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .unwrap_or("");
    if marker != "oversight-agent" {
        return false;
    }

    if bearer_access_level(req) != Some(AdminAccessLevel::ReadWrite) {
        return false;
    }

    if !crate::forwarded_ip_trusted(req) || !crate::request_is_https(req) {
        return false;
    }

    req.header("x-forwarded-for")
        .and_then(|value| value.as_str())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .map(|value| value == "127.0.0.1" || value == "::1")
        .unwrap_or(false)
}

pub fn is_internal_adversary_sim_edge_cron_request(req: &Request) -> bool {
    if !crate::config::gateway_deployment_profile().is_edge()
        || !matches!(req.method(), &Method::Get | &Method::Post)
    {
        return false;
    }

    if !crate::request_is_https(req) {
        return false;
    }

    let expected_secret =
        crate::config::runtime_var_trimmed_optional("SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET")
            .unwrap_or_default();
    if expected_secret.is_empty() {
        return false;
    }

    crate::request_validation::query_param(req.query(), "edge_cron_secret")
        .as_deref()
        .map(str::trim)
        .map(|value| value == expected_secret)
        .unwrap_or(false)
}

pub fn is_internal_adversary_sim_beat_request(req: &Request) -> bool {
    is_internal_adversary_sim_supervisor_request(req) || is_internal_adversary_sim_edge_cron_request(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spin_sdk::http::Request;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockStore {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl KeyValueStore for MockStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn request_with_auth(auth_header: Option<&str>) -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Get).uri("/shuma/admin/config");
        if let Some(auth) = auth_header {
            builder.header("authorization", auth);
        }
        builder.build()
    }

    fn request_for_admin_login() -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Post).uri("/shuma/admin/login");
        builder.build()
    }

    #[test]
    fn unauthorized_when_api_key_missing() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("SHUMA_API_KEY");
        let req = request_with_auth(Some("Bearer any-key"));
        assert!(!is_bearer_authorized(&req));
    }

    #[test]
    fn unauthorized_when_api_key_is_insecure_default() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", INSECURE_DEFAULT_API_KEY);
        let req = request_with_auth(Some("Bearer changeme-supersecret"));
        assert!(!is_bearer_authorized(&req));
    }

    #[test]
    fn unauthorized_when_api_key_is_insecure_default_always() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", INSECURE_DEFAULT_API_KEY);
        let req = request_with_auth(Some("Bearer changeme-supersecret"));
        assert!(!is_bearer_authorized(&req));
    }

    #[test]
    fn authorized_when_api_key_is_explicitly_set() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        let req = request_with_auth(Some("Bearer test-admin-key"));
        assert!(is_bearer_authorized(&req));
    }

    #[test]
    fn readonly_bearer_is_authorized_but_not_write_capable() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        std::env::set_var("SHUMA_ADMIN_READONLY_API_KEY", "test-readonly-key");
        let req = request_with_auth(Some("Bearer test-readonly-key"));
        let store = MockStore::default();

        assert!(is_bearer_authorized(&req));
        assert!(!verify_admin_api_key_candidate("test-readonly-key"));
        assert_eq!(bearer_access_level(&req), Some(AdminAccessLevel::ReadOnly));

        let auth = authenticate_admin(&req, &store);
        assert_eq!(auth.method, Some(AdminAuthMethod::BearerToken));
        assert_eq!(auth.access, Some(AdminAccessLevel::ReadOnly));
        assert!(!auth.is_write_authorized());
        assert_eq!(auth.access_label(), "read_only");
        assert_eq!(auth.audit_actor_label(), "admin_bearer_ro");
        assert_eq!(get_admin_id(&req), "admin_ro");
    }

    #[test]
    fn write_bearer_access_has_write_capability() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        std::env::set_var("SHUMA_ADMIN_READONLY_API_KEY", "test-readonly-key");
        let req = request_with_auth(Some("Bearer test-admin-key"));
        let store = MockStore::default();

        assert_eq!(bearer_access_level(&req), Some(AdminAccessLevel::ReadWrite));
        let auth = authenticate_admin(&req, &store);
        assert_eq!(auth.access, Some(AdminAccessLevel::ReadWrite));
        assert!(auth.is_write_authorized());
        assert_eq!(auth.access_label(), "read_write");
        assert_eq!(auth.audit_actor_label(), "admin_bearer_rw");
        assert_eq!(get_admin_id(&req), "admin_rw");
    }

    #[test]
    fn admin_ip_allowlist_uses_true_client_ip_on_edge_fermyon() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
        std::env::set_var("SHUMA_ADMIN_IP_ALLOWLIST", "203.0.113.8/32");

        let mut builder = Request::builder();
        builder
            .method(Method::Get)
            .uri("/shuma/admin/config")
            .header("authorization", "Bearer test-admin-key")
            .header("true-client-ip", "203.0.113.8");
        let req = builder.build();

        assert!(is_admin_ip_allowed(&req));

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
        std::env::remove_var("SHUMA_ADMIN_IP_ALLOWLIST");
    }

    #[test]
    fn create_and_authenticate_cookie_session() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        let store = MockStore::default();
        let (session_id, csrf_token, _expires) =
            create_admin_session(&store).expect("session should be created");
        assert!(!session_id.is_empty());
        assert!(!csrf_token.is_empty());

        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/shuma/admin/config")
            .header(
                "cookie",
                format!("{}={}", admin_session_cookie_name(), session_id),
            )
            .header("x-shuma-csrf", csrf_token.as_str());
        let req = builder.build();

        let auth = authenticate_admin(&req, &store);
        assert_eq!(auth.method, Some(AdminAuthMethod::SessionCookie));
        assert_eq!(auth.access, Some(AdminAccessLevel::ReadWrite));
        assert!(auth.requires_csrf(&req));
        assert!(auth.is_write_authorized());
        assert_eq!(auth.audit_actor_label(), "admin_session_rw");
        assert!(validate_session_csrf(
            &req,
            auth.csrf_token.as_deref().unwrap_or("")
        ));
    }

    #[test]
    fn admin_auth_failures_are_rate_limited() {
        let _lock = crate::test_support::lock_env();
        let req = request_for_admin_login();
        let store = MockStore::default();

        assert!(!register_admin_auth_failure(
            &store,
            &req,
            AdminAuthFailureScope::Login
        ));
        let ip = crate::extract_client_ip(&req);
        let bucket = crate::signals::ip_identity::bucket_ip(&ip);
        let now_window = super::now_ts() / 60;
        // Pre-seed to guaranteed saturation regardless runtime env limit (max clamp is 10_000).
        for window in [now_window, now_window + 1] {
            let key = format!(
                "rate:{}:{}:{}",
                ADMIN_AUTH_FAILURE_SITE_LOGIN, bucket, window
            );
            store
                .set(
                    &key,
                    ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE_MAX
                        .to_string()
                        .as_bytes(),
                )
                .expect("pre-seed login rate key");
        }
        assert!(register_admin_auth_failure(
            &store,
            &req,
            AdminAuthFailureScope::Login
        ));
    }

    #[test]
    fn admin_auth_failure_scopes_are_independent() {
        let _lock = crate::test_support::lock_env();
        let req = request_for_admin_login();
        let store = MockStore::default();
        let ip = crate::extract_client_ip(&req);
        let bucket = crate::signals::ip_identity::bucket_ip(&ip);
        let now_window = super::now_ts() / 60;
        // Pre-seed both current and next window to avoid minute-boundary flakiness.
        for window in [now_window, now_window + 1] {
            let key = format!(
                "rate:{}:{}:{}",
                ADMIN_AUTH_FAILURE_SITE_LOGIN, bucket, window
            );
            store
                .set(
                    &key,
                    ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE_MAX
                        .to_string()
                        .as_bytes(),
                )
                .expect("pre-seed login rate key");
        }

        assert!(register_admin_auth_failure(
            &store,
            &req,
            AdminAuthFailureScope::Login
        ));

        assert!(!register_admin_auth_failure(
            &store,
            &req,
            AdminAuthFailureScope::Endpoint
        ));
    }

    fn internal_supervisor_request() -> Request {
        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/internal/adversary-sim/beat")
            .header("authorization", "Bearer test-admin-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-proto", "https")
            .header("x-forwarded-for", "127.0.0.1")
            .header("x-shuma-internal-supervisor", "adversary-sim");
        builder.build()
    }

    fn internal_edge_cron_request() -> Request {
        let mut builder = Request::builder();
        builder
            .method(Method::Get)
            .uri("/internal/adversary-sim/beat?edge_cron_secret=test-edge-cron-secret")
            .header(
                "spin-full-url",
                "https://edge.example.com/internal/adversary-sim/beat?edge_cron_secret=test-edge-cron-secret",
            );
        builder.build()
    }

    fn internal_oversight_supervisor_request() -> Request {
        let mut builder = Request::builder();
        builder
            .method(Method::Post)
            .uri("/internal/oversight/agent/run")
            .header("authorization", "Bearer test-admin-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-proto", "https")
            .header("x-forwarded-for", "127.0.0.1")
            .header("x-shuma-internal-supervisor", "oversight-agent");
        builder.build()
    }

    #[test]
    fn internal_adversary_sim_supervisor_request_requires_marker_bearer_secret_https_and_loopback() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");

        let req = internal_supervisor_request();
        assert!(is_internal_adversary_sim_supervisor_request(&req));

        let mut missing_marker = Request::builder();
        missing_marker
            .method(Method::Post)
            .uri("/internal/adversary-sim/beat")
            .header("authorization", "Bearer test-admin-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-proto", "https")
            .header("x-forwarded-for", "127.0.0.1");
        assert!(!is_internal_adversary_sim_supervisor_request(
            &missing_marker.build()
        ));

        let mut wrong_ip = Request::builder();
        wrong_ip
            .method(Method::Post)
            .uri("/internal/adversary-sim/beat")
            .header("authorization", "Bearer test-admin-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-proto", "https")
            .header("x-forwarded-for", "203.0.113.9")
            .header("x-shuma-internal-supervisor", "adversary-sim");
        assert!(!is_internal_adversary_sim_supervisor_request(
            &wrong_ip.build()
        ));

        let mut insecure = Request::builder();
        insecure
            .method(Method::Post)
            .uri("/internal/adversary-sim/beat")
            .header("authorization", "Bearer test-admin-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-for", "127.0.0.1")
            .header("x-shuma-internal-supervisor", "adversary-sim");
        assert!(!is_internal_adversary_sim_supervisor_request(
            &insecure.build()
        ));

        std::env::remove_var("SHUMA_API_KEY");
        std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    }

    #[test]
    fn internal_adversary_sim_edge_cron_request_requires_edge_profile_https_and_secret() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
        std::env::set_var(
            "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET",
            "test-edge-cron-secret",
        );

        let req = internal_edge_cron_request();
        assert!(is_internal_adversary_sim_edge_cron_request(&req));
        assert!(is_internal_adversary_sim_beat_request(&req));

        let mut wrong_secret = Request::builder();
        wrong_secret
            .method(Method::Get)
            .uri("/internal/adversary-sim/beat?edge_cron_secret=wrong")
            .header(
                "spin-full-url",
                "https://edge.example.com/internal/adversary-sim/beat?edge_cron_secret=wrong",
            );
        assert!(!is_internal_adversary_sim_edge_cron_request(
            &wrong_secret.build()
        ));

        let mut insecure = Request::builder();
        insecure
            .method(Method::Get)
            .uri("/internal/adversary-sim/beat?edge_cron_secret=test-edge-cron-secret");
        assert!(!is_internal_adversary_sim_edge_cron_request(
            &insecure.build()
        ));

        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");
        assert!(!is_internal_adversary_sim_edge_cron_request(&req));

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
        std::env::remove_var("SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET");
    }

    #[test]
    fn internal_oversight_supervisor_request_requires_marker_bearer_secret_https_and_loopback() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "test-admin-key");
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");

        let req = internal_oversight_supervisor_request();
        assert!(is_internal_oversight_supervisor_request(&req));

        let mut wrong_marker = Request::builder();
        wrong_marker
            .method(Method::Post)
            .uri("/internal/oversight/agent/run")
            .header("authorization", "Bearer test-admin-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-proto", "https")
            .header("x-forwarded-for", "127.0.0.1")
            .header("x-shuma-internal-supervisor", "adversary-sim");
        assert!(!is_internal_oversight_supervisor_request(
            &wrong_marker.build()
        ));

        let mut wrong_ip = Request::builder();
        wrong_ip
            .method(Method::Post)
            .uri("/internal/oversight/agent/run")
            .header("authorization", "Bearer test-admin-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-proto", "https")
            .header("x-forwarded-for", "203.0.113.10")
            .header("x-shuma-internal-supervisor", "oversight-agent");
        assert!(!is_internal_oversight_supervisor_request(&wrong_ip.build()));

        std::env::remove_var("SHUMA_API_KEY");
        std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    }
}
