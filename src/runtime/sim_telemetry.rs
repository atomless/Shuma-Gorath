use hmac::{Hmac, Mac};
use once_cell::sync::Lazy;
use sha2::Sha256;
use spin_sdk::http::Request;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const SIM_RUN_ID_HEADER: &str = "x-shuma-sim-run-id";
const SIM_PROFILE_HEADER: &str = "x-shuma-sim-profile";
const SIM_LANE_HEADER: &str = "x-shuma-sim-lane";
const SIM_TIMESTAMP_HEADER: &str = "x-shuma-sim-ts";
const SIM_NONCE_HEADER: &str = "x-shuma-sim-nonce";
const SIM_SIGNATURE_HEADER: &str = "x-shuma-sim-signature";
#[cfg(test)]
const SIM_TELEMETRY_SECRET_ENV: &str = "SHUMA_SIM_TELEMETRY_SECRET";
const SIM_VALUE_MAX_CHARS: usize = 96;
const SIM_SIGNATURE_CHARS: usize = 64;
const SIM_TAG_CANONICAL_PREFIX: &str = "sim-tag.v1";
const SIM_TAG_CANONICAL_SEPARATOR: &str = "\n";
const SIM_TAG_TIMESTAMP_MAX_SKEW_SECONDS: u64 = 300;
const SIM_TAG_NONCE_TTL_SECONDS: u64 = 600;
const SIM_TAG_NONCE_MAX_ENTRIES: usize = 4096;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SimTagValidationFailure {
    MissingSecret,
    MissingRequiredHeaders,
    InvalidHeaderValue,
    InvalidTimestamp,
    TimestampSkew,
    SignatureMismatch,
    NonceReplay,
}

impl SimTagValidationFailure {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            SimTagValidationFailure::MissingSecret => "sim_tag_missing_secret",
            SimTagValidationFailure::MissingRequiredHeaders => "sim_tag_missing_required_headers",
            SimTagValidationFailure::InvalidHeaderValue => "sim_tag_invalid_header_value",
            SimTagValidationFailure::InvalidTimestamp => "sim_tag_invalid_timestamp",
            SimTagValidationFailure::TimestampSkew => "sim_tag_timestamp_skew",
            SimTagValidationFailure::SignatureMismatch => "sim_tag_signature_mismatch",
            SimTagValidationFailure::NonceReplay => "sim_tag_nonce_replay",
        }
    }

    pub(crate) fn signal_id(self) -> crate::runtime::policy_taxonomy::SignalId {
        match self {
            SimTagValidationFailure::MissingSecret => {
                crate::runtime::policy_taxonomy::SignalId::SimTagMissingSecret
            }
            SimTagValidationFailure::MissingRequiredHeaders => {
                crate::runtime::policy_taxonomy::SignalId::SimTagMissingRequiredHeaders
            }
            SimTagValidationFailure::InvalidHeaderValue => {
                crate::runtime::policy_taxonomy::SignalId::SimTagInvalidHeaderValue
            }
            SimTagValidationFailure::InvalidTimestamp => {
                crate::runtime::policy_taxonomy::SignalId::SimTagInvalidTimestamp
            }
            SimTagValidationFailure::TimestampSkew => {
                crate::runtime::policy_taxonomy::SignalId::SimTagTimestampSkew
            }
            SimTagValidationFailure::SignatureMismatch => {
                crate::runtime::policy_taxonomy::SignalId::SimTagSignatureMismatch
            }
            SimTagValidationFailure::NonceReplay => {
                crate::runtime::policy_taxonomy::SignalId::SimTagNonceReplay
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SimulationRequestMetadata {
    pub sim_run_id: String,
    pub sim_profile: String,
    pub sim_lane: String,
}

#[derive(Debug, Clone)]
struct SimTagEvaluation {
    metadata: Option<SimulationRequestMetadata>,
    failure: Option<SimTagValidationFailure>,
}

#[derive(Default)]
struct NonceReplayWindow {
    entries: HashMap<String, u64>,
}

impl NonceReplayWindow {
    fn prune_expired(&mut self, now_unix: u64) {
        let floor = now_unix.saturating_sub(SIM_TAG_NONCE_TTL_SECONDS);
        self.entries.retain(|_, seen_at| *seen_at >= floor);
    }

    fn prune_overflow(&mut self) {
        if self.entries.len() <= SIM_TAG_NONCE_MAX_ENTRIES {
            return;
        }
        let mut by_age: Vec<(String, u64)> = self
            .entries
            .iter()
            .map(|(key, seen_at)| (key.clone(), *seen_at))
            .collect();
        by_age.sort_by_key(|(_, seen_at)| *seen_at);
        let to_remove = by_age.len().saturating_sub(SIM_TAG_NONCE_MAX_ENTRIES);
        for (key, _) in by_age.into_iter().take(to_remove) {
            self.entries.remove(&key);
        }
    }

    fn check_and_store(&mut self, nonce_key: String, now_unix: u64) -> Result<(), ()> {
        self.prune_expired(now_unix);
        if self.entries.contains_key(&nonce_key) {
            return Err(());
        }
        self.entries.insert(nonce_key, now_unix);
        self.prune_overflow();
        Ok(())
    }
}

thread_local! {
    static CURRENT_SIM_METADATA: RefCell<Option<SimulationRequestMetadata>> = const { RefCell::new(None) };
    static LAST_SIM_TAG_FAILURE: RefCell<Option<SimTagValidationFailure>> = const { RefCell::new(None) };
}

static NONCE_REPLAY_WINDOW: Lazy<Mutex<NonceReplayWindow>> =
    Lazy::new(|| Mutex::new(NonceReplayWindow::default()));
#[cfg(test)]
static TEST_SERIAL_GUARD: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn set_last_validation_failure(failure: Option<SimTagValidationFailure>) {
    LAST_SIM_TAG_FAILURE.with(|cell| {
        *cell.borrow_mut() = failure;
    });
}

pub(crate) fn take_last_validation_failure() -> Option<SimTagValidationFailure> {
    LAST_SIM_TAG_FAILURE.with(|cell| cell.borrow_mut().take())
}

fn raw_header_value(req: &Request, name: &str) -> Option<String> {
    req.header(name)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn sanitize_sim_value(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > SIM_VALUE_MAX_CHARS {
        return None;
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':' | '.'))
    {
        return None;
    }
    Some(trimmed.to_string())
}

fn sanitize_sim_signature(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() != SIM_SIGNATURE_CHARS {
        return None;
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_digit() || ('a'..='f').contains(&ch))
    {
        return None;
    }
    Some(trimmed.to_string())
}

fn parse_sim_timestamp(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 20 {
        return None;
    }
    trimmed.parse::<u64>().ok()
}

fn timestamp_within_skew(timestamp: u64, now_unix: u64) -> bool {
    if timestamp >= now_unix {
        timestamp - now_unix <= SIM_TAG_TIMESTAMP_MAX_SKEW_SECONDS
    } else {
        now_unix - timestamp <= SIM_TAG_TIMESTAMP_MAX_SKEW_SECONDS
    }
}

fn any_sim_tag_headers_present(req: &Request) -> bool {
    [
        SIM_RUN_ID_HEADER,
        SIM_PROFILE_HEADER,
        SIM_LANE_HEADER,
        SIM_TIMESTAMP_HEADER,
        SIM_NONCE_HEADER,
        SIM_SIGNATURE_HEADER,
    ]
    .iter()
    .any(|header| raw_header_value(req, header).is_some())
}

fn sim_telemetry_secret() -> Option<String> {
    crate::config::sim_telemetry_secret()
}

fn build_sim_tag_canonical_message(
    run_id: &str,
    profile: &str,
    lane: &str,
    timestamp: &str,
    nonce: &str,
) -> String {
    [
        SIM_TAG_CANONICAL_PREFIX,
        run_id,
        profile,
        lane,
        timestamp,
        nonce,
    ]
    .join(SIM_TAG_CANONICAL_SEPARATOR)
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push_str(&format!("{:02x}", byte));
    }
    output
}

fn sign_sim_tag(secret: &str, run_id: &str, profile: &str, lane: &str, timestamp: &str, nonce: &str) -> String {
    let message = build_sim_tag_canonical_message(run_id, profile, lane, timestamp, nonce);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC-SHA256 key initialization should not fail");
    mac.update(message.as_bytes());
    let digest = mac.finalize().into_bytes();
    hex_lower(&digest)
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (left, right) in a.bytes().zip(b.bytes()) {
        diff |= left ^ right;
    }
    diff == 0
}

fn nonce_cache_key(metadata: &SimulationRequestMetadata, nonce: &str) -> String {
    format!(
        "{}:{}:{}:{}",
        metadata.sim_run_id, metadata.sim_profile, metadata.sim_lane, nonce
    )
}

fn validate_and_store_nonce(
    metadata: &SimulationRequestMetadata,
    nonce: &str,
    now_unix: u64,
) -> Result<(), SimTagValidationFailure> {
    let cache_key = nonce_cache_key(metadata, nonce);
    let mut window = NONCE_REPLAY_WINDOW
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    window
        .check_and_store(cache_key, now_unix)
        .map_err(|_| SimTagValidationFailure::NonceReplay)
}

fn metadata_from_request_with_now(
    req: &Request,
    now_unix: u64,
    signing_secret: Option<String>,
) -> SimTagEvaluation {
    if !any_sim_tag_headers_present(req) {
        return SimTagEvaluation {
            metadata: None,
            failure: None,
        };
    }

    let run_id_raw = match raw_header_value(req, SIM_RUN_ID_HEADER) {
        Some(value) => value,
        None => {
            return SimTagEvaluation {
                metadata: None,
                failure: Some(SimTagValidationFailure::MissingRequiredHeaders),
            }
        }
    };
    let profile_raw = match raw_header_value(req, SIM_PROFILE_HEADER) {
        Some(value) => value,
        None => {
            return SimTagEvaluation {
                metadata: None,
                failure: Some(SimTagValidationFailure::MissingRequiredHeaders),
            }
        }
    };
    let lane_raw = match raw_header_value(req, SIM_LANE_HEADER) {
        Some(value) => value,
        None => {
            return SimTagEvaluation {
                metadata: None,
                failure: Some(SimTagValidationFailure::MissingRequiredHeaders),
            }
        }
    };
    let timestamp_raw = match raw_header_value(req, SIM_TIMESTAMP_HEADER) {
        Some(value) => value,
        None => {
            return SimTagEvaluation {
                metadata: None,
                failure: Some(SimTagValidationFailure::MissingRequiredHeaders),
            }
        }
    };
    let nonce_raw = match raw_header_value(req, SIM_NONCE_HEADER) {
        Some(value) => value,
        None => {
            return SimTagEvaluation {
                metadata: None,
                failure: Some(SimTagValidationFailure::MissingRequiredHeaders),
            }
        }
    };
    let signature_raw = match raw_header_value(req, SIM_SIGNATURE_HEADER) {
        Some(value) => value,
        None => {
            return SimTagEvaluation {
                metadata: None,
                failure: Some(SimTagValidationFailure::MissingRequiredHeaders),
            }
        }
    };

    let metadata = SimulationRequestMetadata {
        sim_run_id: match sanitize_sim_value(run_id_raw.as_str()) {
            Some(value) => value,
            None => {
                return SimTagEvaluation {
                    metadata: None,
                    failure: Some(SimTagValidationFailure::InvalidHeaderValue),
                }
            }
        },
        sim_profile: match sanitize_sim_value(profile_raw.as_str()) {
            Some(value) => value,
            None => {
                return SimTagEvaluation {
                    metadata: None,
                    failure: Some(SimTagValidationFailure::InvalidHeaderValue),
                }
            }
        },
        sim_lane: match sanitize_sim_value(lane_raw.as_str()) {
            Some(value) => value,
            None => {
                return SimTagEvaluation {
                    metadata: None,
                    failure: Some(SimTagValidationFailure::InvalidHeaderValue),
                }
            }
        },
    };

    let timestamp = match parse_sim_timestamp(timestamp_raw.as_str()) {
        Some(value) => value,
        None => {
            return SimTagEvaluation {
                metadata: None,
                failure: Some(SimTagValidationFailure::InvalidTimestamp),
            }
        }
    };
    if !timestamp_within_skew(timestamp, now_unix) {
        return SimTagEvaluation {
            metadata: None,
            failure: Some(SimTagValidationFailure::TimestampSkew),
        };
    }

    let nonce = match sanitize_sim_value(nonce_raw.as_str()) {
        Some(value) => value,
        None => {
            return SimTagEvaluation {
                metadata: None,
                failure: Some(SimTagValidationFailure::InvalidHeaderValue),
            }
        }
    };
    let signature = match sanitize_sim_signature(signature_raw.as_str()) {
        Some(value) => value,
        None => {
            return SimTagEvaluation {
                metadata: None,
                failure: Some(SimTagValidationFailure::InvalidHeaderValue),
            }
        }
    };

    let Some(secret) = signing_secret else {
        return SimTagEvaluation {
            metadata: None,
            failure: Some(SimTagValidationFailure::MissingSecret),
        };
    };

    let expected_signature = sign_sim_tag(
        secret.as_str(),
        metadata.sim_run_id.as_str(),
        metadata.sim_profile.as_str(),
        metadata.sim_lane.as_str(),
        timestamp_raw.as_str(),
        nonce.as_str(),
    );
    if !constant_time_eq(signature.as_str(), expected_signature.as_str()) {
        return SimTagEvaluation {
            metadata: None,
            failure: Some(SimTagValidationFailure::SignatureMismatch),
        };
    }

    if let Err(failure) = validate_and_store_nonce(&metadata, nonce.as_str(), now_unix) {
        return SimTagEvaluation {
            metadata: None,
            failure: Some(failure),
        };
    }

    SimTagEvaluation {
        metadata: Some(metadata),
        failure: None,
    }
}

pub(crate) fn metadata_from_request(
    req: &Request,
    runtime_environment: crate::config::RuntimeEnvironment,
    env_available: bool,
) -> Option<SimulationRequestMetadata> {
    set_last_validation_failure(None);

    if !runtime_environment.is_dev() || !env_available {
        return None;
    }

    let evaluation = metadata_from_request_with_now(req, now_unix_seconds(), sim_telemetry_secret());
    set_last_validation_failure(evaluation.failure);
    evaluation.metadata
}

pub(crate) struct SimulationContextGuard {
    previous: Option<SimulationRequestMetadata>,
}

pub(crate) fn enter(metadata: Option<SimulationRequestMetadata>) -> SimulationContextGuard {
    let previous = CURRENT_SIM_METADATA.with(|cell| {
        let mut slot = cell.borrow_mut();
        let previous = slot.clone();
        *slot = metadata;
        previous
    });
    SimulationContextGuard { previous }
}

impl Drop for SimulationContextGuard {
    fn drop(&mut self) {
        CURRENT_SIM_METADATA.with(|cell| {
            *cell.borrow_mut() = self.previous.clone();
        });
    }
}

pub(crate) fn current_metadata() -> Option<SimulationRequestMetadata> {
    CURRENT_SIM_METADATA.with(|cell| cell.borrow().clone())
}

pub(crate) fn is_simulation_context_active() -> bool {
    current_metadata().is_some()
}

#[cfg(test)]
fn reset_nonce_replay_window_for_tests() {
    let mut window = NONCE_REPLAY_WINDOW
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    window.entries.clear();
}

#[cfg(test)]
fn lock_test_serial() -> std::sync::MutexGuard<'static, ()> {
    TEST_SERIAL_GUARD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spin_sdk::http::Method;

    fn make_request_with_headers(headers: &[(&str, &str)]) -> Request {
        let mut builder = Request::builder();
        builder.method(Method::Get).uri("/");
        for (key, value) in headers {
            builder.header(*key, *value);
        }
        builder.build()
    }

    fn make_signed_request(
        secret: &str,
        run_id: &str,
        profile: &str,
        lane: &str,
        timestamp: u64,
        nonce: &str,
    ) -> Request {
        let timestamp_text = timestamp.to_string();
        let signature = sign_sim_tag(secret, run_id, profile, lane, timestamp_text.as_str(), nonce);
        let mut builder = Request::builder();
        builder.method(Method::Get).uri("/");
        builder.header(SIM_RUN_ID_HEADER, run_id);
        builder.header(SIM_PROFILE_HEADER, profile);
        builder.header(SIM_LANE_HEADER, lane);
        builder.header(SIM_TIMESTAMP_HEADER, timestamp_text.as_str());
        builder.header(SIM_NONCE_HEADER, nonce);
        builder.header(SIM_SIGNATURE_HEADER, signature.as_str());
        builder.build()
    }

    #[test]
    fn metadata_requires_dev_env_and_valid_signature() {
        let _serial = lock_test_serial();
        let _lock = crate::test_support::lock_env();
        reset_nonce_replay_window_for_tests();
        set_last_validation_failure(None);
        std::env::set_var(SIM_TELEMETRY_SECRET_ENV, "sim-secret");

        let now_unix = now_unix_seconds();
        let req = make_signed_request(
            "sim-secret",
            "run_123",
            "fast_smoke",
            "deterministic_black_box",
            now_unix,
            "nonce-1",
        );

        let metadata = metadata_from_request(
            &req,
            crate::config::RuntimeEnvironment::RuntimeDev,
            true,
        )
        .expect("expected valid metadata");
        assert_eq!(metadata.sim_run_id, "run_123");
        assert_eq!(metadata.sim_profile, "fast_smoke");
        assert_eq!(metadata.sim_lane, "deterministic_black_box");
        assert_eq!(take_last_validation_failure(), None);

        std::env::remove_var(SIM_TELEMETRY_SECRET_ENV);
    }

    #[test]
    fn metadata_rejects_missing_secret() {
        let _serial = lock_test_serial();
        reset_nonce_replay_window_for_tests();
        set_last_validation_failure(None);

        let now_unix = 1_800_000_000u64;
        let req = make_signed_request(
            "sim-secret",
            "run_123",
            "fast_smoke",
            "deterministic_black_box",
            now_unix,
            "nonce-2",
        );

        let evaluation = metadata_from_request_with_now(&req, now_unix, None);
        assert!(evaluation.metadata.is_none());
        assert_eq!(
            evaluation.failure,
            Some(SimTagValidationFailure::MissingSecret)
        );
    }

    #[test]
    fn metadata_rejects_signature_mismatch() {
        let _serial = lock_test_serial();
        reset_nonce_replay_window_for_tests();
        set_last_validation_failure(None);

        let req = make_request_with_headers(&[
            (SIM_RUN_ID_HEADER, "run_123"),
            (SIM_PROFILE_HEADER, "fast_smoke"),
            (SIM_LANE_HEADER, "deterministic_black_box"),
            (SIM_TIMESTAMP_HEADER, "1800000000"),
            (SIM_NONCE_HEADER, "nonce-3"),
            (
                SIM_SIGNATURE_HEADER,
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
        ]);

        let evaluation =
            metadata_from_request_with_now(&req, 1_800_000_000u64, Some("sim-secret".to_string()));
        assert!(evaluation.metadata.is_none());
        assert_eq!(
            evaluation.failure,
            Some(SimTagValidationFailure::SignatureMismatch)
        );
    }

    #[test]
    fn metadata_rejects_stale_timestamp() {
        let _serial = lock_test_serial();
        reset_nonce_replay_window_for_tests();
        set_last_validation_failure(None);

        let now_unix = 1_800_000_000u64;
        let stale_timestamp = now_unix.saturating_sub(SIM_TAG_TIMESTAMP_MAX_SKEW_SECONDS + 1);
        let req = make_signed_request(
            "sim-secret",
            "run_123",
            "fast_smoke",
            "deterministic_black_box",
            stale_timestamp,
            "nonce-4",
        );

        let evaluation =
            metadata_from_request_with_now(&req, now_unix, Some("sim-secret".to_string()));
        assert!(evaluation.metadata.is_none());
        assert_eq!(evaluation.failure, Some(SimTagValidationFailure::TimestampSkew));
    }

    #[test]
    fn metadata_rejects_replay_nonce() {
        let _serial = lock_test_serial();
        reset_nonce_replay_window_for_tests();
        set_last_validation_failure(None);

        let now_unix = 1_800_000_000u64;
        let req = make_signed_request(
            "sim-secret",
            "run_123",
            "fast_smoke",
            "deterministic_black_box",
            now_unix,
            "nonce-5",
        );

        let first =
            metadata_from_request_with_now(&req, now_unix, Some("sim-secret".to_string()));
        assert!(first.metadata.is_some());
        assert_eq!(first.failure, None);

        let second =
            metadata_from_request_with_now(&req, now_unix, Some("sim-secret".to_string()));
        assert!(second.metadata.is_none());
        assert_eq!(second.failure, Some(SimTagValidationFailure::NonceReplay));
    }

    #[test]
    fn metadata_ignores_requests_without_sim_headers() {
        let _serial = lock_test_serial();
        reset_nonce_replay_window_for_tests();
        set_last_validation_failure(None);

        let req = make_request_with_headers(&[]);
        let evaluation =
            metadata_from_request_with_now(&req, 1_800_000_000u64, Some("sim-secret".to_string()));
        assert!(evaluation.metadata.is_none());
        assert_eq!(evaluation.failure, None);
    }

    #[test]
    fn sim_tag_failure_reasons_map_to_canonical_policy_signals() {
        assert_eq!(
            SimTagValidationFailure::MissingSecret.signal_id().as_str(),
            "S_SIM_TAG_MISSING_SECRET"
        );
        assert_eq!(
            SimTagValidationFailure::MissingRequiredHeaders
                .signal_id()
                .as_str(),
            "S_SIM_TAG_MISSING_REQUIRED_HEADERS"
        );
        assert_eq!(
            SimTagValidationFailure::NonceReplay.signal_id().as_str(),
            "S_SIM_TAG_NONCE_REPLAY"
        );
    }

    #[test]
    fn context_guard_restores_previous_metadata() {
        let first = SimulationRequestMetadata {
            sim_run_id: "run_a".to_string(),
            sim_profile: "fast_smoke".to_string(),
            sim_lane: "deterministic_black_box".to_string(),
        };
        let second = SimulationRequestMetadata {
            sim_run_id: "run_b".to_string(),
            sim_profile: "abuse_regression".to_string(),
            sim_lane: "deterministic_black_box".to_string(),
        };

        let _guard_a = enter(Some(first.clone()));
        assert_eq!(current_metadata(), Some(first.clone()));
        {
            let _guard_b = enter(Some(second.clone()));
            assert_eq!(current_metadata(), Some(second));
        }
        assert_eq!(current_metadata(), Some(first));
    }
}
