use rand::random;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spin_sdk::http::{Method, Request};

use crate::admin::adversary_sim::ControlPhase;
use crate::challenge::KeyValueStore;

const IDEMPOTENCY_KEY_PREFIX: &str = "adversary_sim:control:idempotency:";
const OPERATION_KEY_PREFIX: &str = "adversary_sim:control:operation:";
const LEASE_KEY_PREFIX: &str = "adversary_sim:control:lease:";
const SUBMISSION_DEBOUNCE_KEY_PREFIX: &str = "adversary_sim:control:debounce:";

pub const IDEMPOTENCY_TTL_SECONDS: u64 = 600;
pub const LEASE_TTL_SECONDS: u64 = 15;
pub const CONTROL_SESSION_LIMIT_PER_MINUTE: u32 = 6;
pub const CONTROL_IP_LIMIT_PER_MINUTE: u32 = 16;
pub const CONTROL_DEBOUNCE_SECONDS: u64 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdempotencyRecord {
    pub operation_id: String,
    pub payload_hash: String,
    pub actor_scope: String,
    pub session_scope: String,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControllerLease {
    pub owner: String,
    pub fencing_token: u64,
    pub acquired_at: u64,
    pub expires_at: u64,
    pub operation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ControlDecision {
    Accepted,
    Replayed,
    Rejected,
    Throttled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlOperationRecord {
    pub operation_id: String,
    pub requested_enabled: bool,
    pub requested_lane: Option<String>,
    pub requested_reason: String,
    pub desired_enabled: bool,
    pub desired_lane: Option<String>,
    pub actual_phase: String,
    pub actual_lane: Option<String>,
    pub actor_scope: String,
    pub session_scope: String,
    pub idempotency_key_hash: String,
    pub payload_hash: String,
    pub created_at: u64,
    pub completed_at: u64,
    pub decision: ControlDecision,
    pub decision_reason: String,
    pub origin_verdict: String,
    pub lease_fencing_token: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlAuditRecord {
    pub operation_id: Option<String>,
    pub actor_scope: String,
    pub session_scope: String,
    pub decision: ControlDecision,
    pub reason: String,
    pub origin_verdict: String,
    pub idempotency_key_hash: Option<String>,
    pub request_origin: Option<String>,
    pub requested_state: Option<String>,
    pub requested_lane: Option<String>,
    pub desired_state: Option<String>,
    pub desired_lane: Option<String>,
    pub actual_state: String,
    pub actual_lane: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdempotencyPlan {
    NewSubmission,
    Replay,
    PayloadMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustDecision {
    Allow,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrottleDecision {
    Allow,
    Throttle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmissionPlanDecision {
    AcceptNew,
    ReturnReplay,
    RejectPayloadMismatch,
    RejectTrustBoundary,
    RejectThrottled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionPlanInput {
    pub trust: TrustDecision,
    pub throttle: ThrottleDecision,
    pub idempotency: IdempotencyPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionPlan {
    pub decision: SubmissionPlanDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginValidation {
    pub request_origin: Option<String>,
    pub verdict: String,
}

#[derive(Debug, Clone, Copy)]
pub struct StateWriteCapability {
    _private: (),
}

#[derive(Debug, Clone, Copy)]
pub struct AuditWriteCapability {
    _private: (),
}

#[derive(Debug, Clone, Copy)]
pub struct ControlCapabilities {
    state_write: StateWriteCapability,
    audit_write: AuditWriteCapability,
}

impl ControlCapabilities {
    pub fn mint_for_trust_boundary() -> Self {
        Self {
            state_write: StateWriteCapability { _private: () },
            audit_write: AuditWriteCapability { _private: () },
        }
    }

    pub fn state_write(&self) -> &StateWriteCapability {
        &self.state_write
    }

    pub fn audit_write(&self) -> &AuditWriteCapability {
        &self.audit_write
    }
}

pub fn plan_submission(input: &SubmissionPlanInput) -> SubmissionPlan {
    let decision = if input.trust == TrustDecision::Reject {
        SubmissionPlanDecision::RejectTrustBoundary
    } else {
        match input.idempotency {
            IdempotencyPlan::Replay => SubmissionPlanDecision::ReturnReplay,
            IdempotencyPlan::PayloadMismatch => SubmissionPlanDecision::RejectPayloadMismatch,
            IdempotencyPlan::NewSubmission => {
                if input.throttle == ThrottleDecision::Throttle {
                    SubmissionPlanDecision::RejectThrottled
                } else {
                    SubmissionPlanDecision::AcceptNew
                }
            }
        }
    };
    SubmissionPlan { decision }
}

pub fn operation_id(now: u64) -> String {
    format!("simop-{}-{:016x}", now, random::<u64>())
}

pub fn canonical_reason(reason: Option<&str>) -> String {
    let normalized = reason
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("manual_toggle");
    normalized.chars().take(120).collect::<String>()
}

pub fn hash_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    format!("{:x}", digest)
}

pub fn canonical_payload_hash(enabled: bool, lane: Option<&str>, reason: &str) -> String {
    let payload = serde_json::json!({
        "enabled": enabled,
        "lane": lane,
        "reason": reason,
    });
    let encoded = serde_json::to_vec(&payload).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    format!("{:x}", hasher.finalize())
}

pub fn idempotency_scope(auth: &crate::admin::auth::AdminAuthResult) -> String {
    match auth.session_id.as_deref() {
        Some(session_id) if !session_id.is_empty() => format!("session:{}", session_id),
        _ => format!("actor:{}", auth.audit_actor_label()),
    }
}

pub fn actor_scope(auth: &crate::admin::auth::AdminAuthResult) -> String {
    auth.audit_actor_label().to_string()
}

pub fn control_idempotency_key(site_id: &str, scope: &str, idempotency_hash: &str) -> String {
    format!(
        "{}{}:{}:{}",
        IDEMPOTENCY_KEY_PREFIX, site_id, scope, idempotency_hash
    )
}

pub fn control_operation_key(site_id: &str, operation_id: &str) -> String {
    format!("{}{}:{}", OPERATION_KEY_PREFIX, site_id, operation_id)
}

pub fn control_lease_key(site_id: &str) -> String {
    format!("{}{}", LEASE_KEY_PREFIX, site_id)
}

pub fn control_debounce_key(site_id: &str, scope: &str) -> String {
    format!("{}{}:{}", SUBMISSION_DEBOUNCE_KEY_PREFIX, site_id, scope)
}

pub fn load_idempotency_record<S: KeyValueStore>(store: &S, key: &str) -> Option<IdempotencyRecord> {
    let raw = store.get(key).ok().flatten()?;
    serde_json::from_slice::<IdempotencyRecord>(&raw).ok()
}

pub fn save_idempotency_record<S: KeyValueStore>(
    store: &S,
    key: &str,
    record: &IdempotencyRecord,
    _capability: &StateWriteCapability,
) -> Result<(), ()> {
    let encoded = serde_json::to_vec(record).map_err(|_| ())?;
    store.set(key, &encoded)
}

pub fn load_operation_record<S: KeyValueStore>(store: &S, key: &str) -> Option<ControlOperationRecord> {
    let raw = store.get(key).ok().flatten()?;
    serde_json::from_slice::<ControlOperationRecord>(&raw).ok()
}

pub fn save_operation_record<S: KeyValueStore>(
    store: &S,
    key: &str,
    record: &ControlOperationRecord,
    _capability: &StateWriteCapability,
) -> Result<(), ()> {
    let encoded = serde_json::to_vec(record).map_err(|_| ())?;
    store.set(key, &encoded)
}

pub fn load_controller_lease<S: KeyValueStore>(store: &S, site_id: &str) -> Option<ControllerLease> {
    let key = control_lease_key(site_id);
    let raw = store.get(&key).ok().flatten()?;
    serde_json::from_slice::<ControllerLease>(&raw).ok()
}

pub fn save_controller_lease<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    lease: &ControllerLease,
    _capability: &StateWriteCapability,
) -> Result<(), ()> {
    let key = control_lease_key(site_id);
    let encoded = serde_json::to_vec(lease).map_err(|_| ())?;
    store.set(&key, &encoded)
}

pub fn acquire_controller_lease(
    now: u64,
    owner: &str,
    operation_id: Option<&str>,
    current: Option<&ControllerLease>,
) -> Result<ControllerLease, &'static str> {
    match current {
        Some(existing) if existing.expires_at > now && existing.owner != owner => {
            Err("controller_lease_held")
        }
        Some(existing) if existing.owner == owner => Ok(ControllerLease {
            owner: owner.to_string(),
            fencing_token: existing.fencing_token,
            acquired_at: existing.acquired_at,
            expires_at: now.saturating_add(LEASE_TTL_SECONDS),
            operation_id: operation_id.map(|value| value.to_string()),
        }),
        Some(existing) => Ok(ControllerLease {
            owner: owner.to_string(),
            fencing_token: existing.fencing_token.saturating_add(1),
            acquired_at: now,
            expires_at: now.saturating_add(LEASE_TTL_SECONDS),
            operation_id: operation_id.map(|value| value.to_string()),
        }),
        None => Ok(ControllerLease {
            owner: owner.to_string(),
            fencing_token: 1,
            acquired_at: now,
            expires_at: now.saturating_add(LEASE_TTL_SECONDS),
            operation_id: operation_id.map(|value| value.to_string()),
        }),
    }
}

pub fn validate_origin_and_fetch_metadata(req: &Request) -> Result<OriginValidation, &'static str> {
    if !matches!(
        req.method(),
        Method::Post | Method::Put | Method::Patch | Method::Delete
    ) {
        return Ok(OriginValidation {
            request_origin: None,
            verdict: "safe_method".to_string(),
        });
    }

    let fetch_site = req
        .header("sec-fetch-site")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    if let Some(fetch_site) = fetch_site.as_deref() {
        if !matches!(fetch_site, "same-origin" | "same-site" | "none") {
            return Err("fetch_metadata_cross_site");
        }
    }
    let fetch_metadata_missing = fetch_site.is_none();

    let expected_origin = expected_origin(req);

    if let Some(origin) = req
        .header("origin")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if expected_origin
            .as_ref()
            .map(|expected| origin == expected)
            .unwrap_or(false)
        {
            return Ok(OriginValidation {
                request_origin: Some(origin.to_string()),
                verdict: if fetch_metadata_missing {
                    "origin_match_fetch_metadata_missing".to_string()
                } else {
                    "origin_match".to_string()
                },
            });
        }
        if expected_origin.is_some() {
            return Err("origin_mismatch");
        }
        return Ok(OriginValidation {
            request_origin: Some(origin.to_string()),
            verdict: "origin_no_host_fallback".to_string(),
        });
    }

    if let Some(referer) = req
        .header("referer")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if expected_origin
            .as_ref()
            .map(|expected| referer.starts_with(expected.as_str()))
            .unwrap_or(false)
        {
            return Ok(OriginValidation {
                request_origin: expected_origin,
                verdict: if fetch_metadata_missing {
                    "referer_match_fetch_metadata_missing".to_string()
                } else {
                    "referer_match".to_string()
                },
            });
        }
        if expected_origin.is_some() {
            return Err("referer_mismatch");
        }
        return Ok(OriginValidation {
            request_origin: None,
            verdict: "referer_no_host_fallback".to_string(),
        });
    }

    let csrf_present = req
        .header("x-shuma-csrf")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some();
    if expected_origin.is_none() && csrf_present {
        return Ok(OriginValidation {
            request_origin: None,
            verdict: if fetch_metadata_missing {
                "csrf_origin_fallback".to_string()
            } else {
                "csrf_fetch_metadata_fallback".to_string()
            },
        });
    }

    if expected_origin.is_none() {
        return Err("origin_host_missing");
    }

    if csrf_present && fetch_metadata_missing {
        return Ok(OriginValidation {
            request_origin: expected_origin,
            verdict: "csrf_origin_fallback".to_string(),
        });
    }

    if fetch_metadata_missing {
        return Err("fetch_metadata_missing");
    }

    Err("origin_missing")
}

fn expected_origin(req: &Request) -> Option<String> {
    let host = req
        .header("host")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())?;

    if host.contains('/') || host.contains(' ') {
        return None;
    }

    let scheme = req
        .header("x-forwarded-proto")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            if crate::config::https_enforced() {
                "https"
            } else {
                "http"
            }
        });

    Some(format!("{}://{}", scheme, host))
}

pub fn parse_debounce_timestamp(bytes: &[u8]) -> Option<u64> {
    let raw = std::str::from_utf8(bytes).ok()?.trim();
    raw.parse::<u64>().ok()
}

pub fn save_debounce_timestamp<S: KeyValueStore>(
    store: &S,
    key: &str,
    ts: u64,
    _capability: &StateWriteCapability,
) -> Result<(), ()> {
    store.set(key, ts.to_string().as_bytes())
}

pub fn should_throttle_for_debounce(
    now: u64,
    last_submission_at: Option<u64>,
    throttle_window_seconds: u64,
) -> bool {
    match last_submission_at {
        Some(previous) => now.saturating_sub(previous) < throttle_window_seconds,
        None => false,
    }
}

pub fn status_reconciliation_needed(
    now: u64,
    desired_enabled: bool,
    actual_state: &crate::admin::adversary_sim::ControlState,
) -> bool {
    let (next, _) = crate::admin::adversary_sim::reconcile_state(now, desired_enabled, actual_state);
    next != *actual_state || crate::admin::adversary_sim::lane_reconciliation_needed(actual_state)
}

pub fn actual_phase_label(phase: ControlPhase) -> &'static str {
    phase.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_with_headers(path: &str, headers: &[(&str, &str)]) -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Post).uri(path).body(Vec::new());
        for (name, value) in headers {
            builder.header(*name, *value);
        }
        builder.build()
    }

    #[test]
    fn plan_submission_prioritizes_reject_paths_before_acceptance() {
        let rejected = plan_submission(&SubmissionPlanInput {
            trust: TrustDecision::Reject,
            throttle: ThrottleDecision::Allow,
            idempotency: IdempotencyPlan::NewSubmission,
        });
        assert_eq!(rejected.decision, SubmissionPlanDecision::RejectTrustBoundary);

        let throttled = plan_submission(&SubmissionPlanInput {
            trust: TrustDecision::Allow,
            throttle: ThrottleDecision::Throttle,
            idempotency: IdempotencyPlan::NewSubmission,
        });
        assert_eq!(throttled.decision, SubmissionPlanDecision::RejectThrottled);

        let replay = plan_submission(&SubmissionPlanInput {
            trust: TrustDecision::Allow,
            throttle: ThrottleDecision::Throttle,
            idempotency: IdempotencyPlan::Replay,
        });
        assert_eq!(replay.decision, SubmissionPlanDecision::ReturnReplay);
    }

    #[test]
    fn acquire_controller_lease_requires_expiry_or_same_owner() {
        let now = 1_000u64;
        let current = ControllerLease {
            owner: "owner-a".to_string(),
            fencing_token: 7,
            acquired_at: 900,
            expires_at: 1_100,
            operation_id: Some("simop-1".to_string()),
        };

        let denied = acquire_controller_lease(now, "owner-b", Some("simop-2"), Some(&current));
        assert_eq!(denied, Err("controller_lease_held"));

        let granted_same_owner =
            acquire_controller_lease(now, "owner-a", Some("simop-2"), Some(&current))
                .expect("lease for same owner");
        assert_eq!(granted_same_owner.fencing_token, 7);

        let expired = ControllerLease {
            expires_at: 999,
            ..current
        };
        let granted_new_owner =
            acquire_controller_lease(now, "owner-b", Some("simop-3"), Some(&expired))
                .expect("lease after expiry");
        assert_eq!(granted_new_owner.fencing_token, 8);
        assert_eq!(granted_new_owner.owner, "owner-b");
    }

    #[test]
    fn validate_origin_and_fetch_metadata_rejects_cross_site_or_missing_origin() {
        let cross_site = request_with_headers(
            "/admin/adversary-sim/control",
            &[
                ("host", "localhost:3000"),
                ("origin", "http://localhost:3000"),
                ("sec-fetch-site", "cross-site"),
            ],
        );
        assert_eq!(
            validate_origin_and_fetch_metadata(&cross_site),
            Err("fetch_metadata_cross_site")
        );

        let missing_origin = request_with_headers(
            "/admin/adversary-sim/control",
            &[("host", "localhost:3000"), ("sec-fetch-site", "same-origin")],
        );
        assert_eq!(
            validate_origin_and_fetch_metadata(&missing_origin),
            Err("origin_missing")
        );
    }

    #[test]
    fn validate_origin_and_fetch_metadata_accepts_matching_origin() {
        let req = request_with_headers(
            "/admin/adversary-sim/control",
            &[
                ("host", "localhost:3000"),
                ("origin", "http://localhost:3000"),
                ("sec-fetch-site", "same-origin"),
            ],
        );

        let verdict = validate_origin_and_fetch_metadata(&req).expect("origin validation");
        assert_eq!(verdict.verdict, "origin_match");
    }

    #[test]
    fn validate_origin_and_fetch_metadata_accepts_matching_origin_without_fetch_metadata() {
        let req = request_with_headers(
            "/admin/adversary-sim/control",
            &[("host", "localhost:3000"), ("origin", "http://localhost:3000")],
        );

        let verdict = validate_origin_and_fetch_metadata(&req).expect("origin validation");
        assert_eq!(verdict.verdict, "origin_match_fetch_metadata_missing");
    }

    #[test]
    fn validate_origin_and_fetch_metadata_allows_host_missing_with_csrf_fallback() {
        let req = request_with_headers(
            "/admin/adversary-sim/control",
            &[("sec-fetch-site", "same-origin"), ("x-shuma-csrf", "csrf-token")],
        );

        let verdict = validate_origin_and_fetch_metadata(&req).expect("origin validation");
        assert_eq!(verdict.verdict, "csrf_fetch_metadata_fallback");
    }

    #[test]
    fn validate_origin_and_fetch_metadata_rejects_host_missing_without_csrf() {
        let req = request_with_headers(
            "/admin/adversary-sim/control",
            &[("sec-fetch-site", "same-origin")],
        );
        assert_eq!(validate_origin_and_fetch_metadata(&req), Err("origin_host_missing"));
    }

    #[test]
    fn validate_origin_and_fetch_metadata_rejects_missing_fetch_metadata_without_origin_or_csrf() {
        let req = request_with_headers("/admin/adversary-sim/control", &[("host", "localhost:3000")]);
        assert_eq!(validate_origin_and_fetch_metadata(&req), Err("fetch_metadata_missing"));
    }

    #[test]
    fn should_throttle_for_debounce_uses_bounded_window() {
        assert!(should_throttle_for_debounce(100, Some(99), 2));
        assert!(!should_throttle_for_debounce(100, Some(95), 2));
    }

    #[test]
    fn canonical_payload_hash_changes_when_payload_changes() {
        let a = canonical_payload_hash(true, None, "manual_on");
        let b = canonical_payload_hash(false, None, "manual_on");
        let c = canonical_payload_hash(true, None, "manual_off");
        let d = canonical_payload_hash(true, Some("scrapling_traffic"), "manual_on");

        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }
}
