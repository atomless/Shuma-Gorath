use spin_sdk::http::{Request, Response};

use super::super::{challenge_response, KeyValueStore};
use super::renders::render_challenge_with_seed_ttl;
use super::token::{parse_seed_token, SeedTokenError};
use super::{build_puzzle, parse_submission};

const CHALLENGE_FORBIDDEN_BODY: &str = "<html><body><h2 style='color:red;'>Forbidden. Please request a new challenge.</h2><a href='/challenge/puzzle'>Request new challenge.</a></body></html>";
const CHALLENGE_EXPIRED_BODY: &str = "<html><body><h2 style='color:red;'>Expired</h2><a href='/challenge/puzzle'>Request new challenge.</a></body></html>";
const CHALLENGE_INCORRECT_BODY: &str = "<html><body><h2 style='color:red;'>Incorrect.</h2><a href='/challenge/puzzle'>Request new challenge.</a></body></html>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChallengeSubmitOutcome {
    Solved,
    Incorrect,
    AttemptLimitExceeded,
    SequenceOpMissing,
    SequenceOpInvalid,
    SequenceOpExpired,
    SequenceOpReplay,
    SequenceWindowExceeded,
    SequenceOrderViolation,
    SequenceBindingMismatch,
    SequenceTimingTooFast,
    SequenceTimingTooRegular,
    SequenceTimingTooSlow,
    Forbidden,
    InvalidOutput,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AttemptState {
    window_start: u64,
    count: u32,
}

fn challenge_forbidden_response() -> Response {
    challenge_response(403, CHALLENGE_FORBIDDEN_BODY)
}

fn challenge_expired_response() -> Response {
    challenge_response(403, CHALLENGE_EXPIRED_BODY)
}

fn challenge_incorrect_response() -> Response {
    challenge_response(403, CHALLENGE_INCORRECT_BODY)
}

#[cfg(test)]
pub(crate) fn serve_challenge_page(
    req: &Request,
    test_mode: bool,
    transform_count: usize,
) -> Response {
    serve_challenge_page_with_seed_ttl(
        req,
        test_mode,
        transform_count,
        crate::config::defaults().challenge_puzzle_seed_ttl_seconds,
    )
}

pub(crate) fn serve_challenge_page_with_seed_ttl(
    req: &Request,
    test_mode: bool,
    transform_count: usize,
    seed_ttl_seconds: u64,
) -> Response {
    if !test_mode {
        return challenge_response(404, "Not Found");
    }
    render_challenge_with_seed_ttl(req, transform_count, seed_ttl_seconds)
}

#[cfg(test)]
pub(crate) fn handle_challenge_submit_with_outcome<S: KeyValueStore>(
    store: &S,
    req: &Request,
) -> (Response, ChallengeSubmitOutcome) {
    let defaults = crate::config::defaults();
    handle_challenge_submit_with_outcome_with_limits(
        store,
        req,
        defaults.challenge_puzzle_attempt_window_seconds,
        defaults.challenge_puzzle_attempt_limit_per_window,
    )
}

pub(crate) fn handle_challenge_submit_with_outcome_with_limits<S: KeyValueStore>(
    store: &S,
    req: &Request,
    challenge_puzzle_attempt_window_seconds: u64,
    challenge_puzzle_attempt_limit_per_window: u32,
) -> (Response, ChallengeSubmitOutcome) {
    if crate::request_validation::enforce_body_size(
        req.body(),
        crate::request_validation::MAX_CHALLENGE_FORM_BYTES,
    )
    .is_err()
    {
        return (
            challenge_forbidden_response(),
            ChallengeSubmitOutcome::Forbidden,
        );
    }
    let now = crate::admin::now_ts();
    let ip = crate::extract_client_ip(req);
    let request_ip_bucket = crate::signals::ip_identity::bucket_ip(ip.as_str());
    if increment_and_check_attempt_limit(
        store,
        request_ip_bucket.as_str(),
        now,
        challenge_puzzle_attempt_window_seconds,
        challenge_puzzle_attempt_limit_per_window,
    ) {
        return (
            challenge_forbidden_response(),
            ChallengeSubmitOutcome::AttemptLimitExceeded,
        );
    }
    let form = match std::str::from_utf8(req.body()) {
        Ok(v) => v.to_string(),
        Err(_) => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::Forbidden,
            )
        }
    };
    let seed_token = match get_form_field(&form, "seed") {
        Some(v) => v,
        None => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::Forbidden,
            )
        }
    };
    if !crate::request_validation::validate_seed_token(seed_token.as_str()) {
        return (
            challenge_forbidden_response(),
            ChallengeSubmitOutcome::Forbidden,
        );
    }
    let output_raw = match get_form_field(&form, "output") {
        Some(v) => v,
        None => {
            return (
                challenge_response(400, "Invalid output"),
                ChallengeSubmitOutcome::InvalidOutput,
            )
        }
    };
    if output_raw.len() > 128 {
        return (
            challenge_response(400, "Invalid output"),
            ChallengeSubmitOutcome::InvalidOutput,
        );
    }
    let seed = match parse_seed_token(&seed_token) {
        Ok(s) => s,
        Err(SeedTokenError::InvalidOperationEnvelope(
            crate::challenge::operation_envelope::EnvelopeValidationError::MissingOperationId,
        )) => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::SequenceOpMissing,
            )
        }
        Err(SeedTokenError::InvalidOperationEnvelope(_)) => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::SequenceOpInvalid,
            )
        }
        Err(_) => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::Forbidden,
            )
        }
    };
    if now > seed.expires_at {
        return (
            challenge_expired_response(),
            ChallengeSubmitOutcome::SequenceOpExpired,
        );
    }
    match crate::challenge::operation_envelope::validate_ordering_window(
        seed.flow_id.as_str(),
        seed.step_id.as_str(),
        seed.step_index,
        seed.issued_at,
        seed.expires_at,
        now,
        crate::challenge::operation_envelope::FLOW_CHALLENGE_PUZZLE,
        crate::challenge::operation_envelope::STEP_CHALLENGE_PUZZLE_SUBMIT,
        crate::challenge::operation_envelope::STEP_INDEX_CHALLENGE_PUZZLE_SUBMIT,
        crate::challenge::operation_envelope::MAX_STEP_WINDOW_SECONDS_CHALLENGE_PUZZLE,
    ) {
        Ok(_) => {}
        Err(crate::challenge::operation_envelope::OrderingValidationError::OrderViolation) => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::SequenceOrderViolation,
            )
        }
        Err(crate::challenge::operation_envelope::OrderingValidationError::WindowExceeded) => {
            return (
                challenge_expired_response(),
                ChallengeSubmitOutcome::SequenceWindowExceeded,
            )
        }
    }
    let ua = req
        .header("user-agent")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if crate::challenge::operation_envelope::validate_request_binding(
        seed.ip_bucket.as_str(),
        seed.ua_bucket.as_str(),
        seed.path_class.as_str(),
        ip.as_str(),
        ua,
        crate::challenge::operation_envelope::PATH_CLASS_CHALLENGE_PUZZLE_SUBMIT,
    )
    .is_err()
    {
        return (
            challenge_forbidden_response(),
            ChallengeSubmitOutcome::SequenceBindingMismatch,
        );
    }
    let timing_bucket = format!("{}:{}", seed.ip_bucket, seed.ua_bucket);
    match crate::challenge::operation_envelope::validate_timing_primitives(
        store,
        seed.flow_id.as_str(),
        timing_bucket.as_str(),
        seed.issued_at,
        now,
        crate::challenge::operation_envelope::MIN_STEP_LATENCY_SECONDS_CHALLENGE_PUZZLE,
        crate::challenge::operation_envelope::MAX_STEP_LATENCY_SECONDS_CHALLENGE_PUZZLE,
        crate::challenge::operation_envelope::MAX_FLOW_AGE_SECONDS_CHALLENGE_PUZZLE,
        crate::challenge::operation_envelope::TIMING_REGULARITY_WINDOW_CHALLENGE_PUZZLE,
        crate::challenge::operation_envelope::TIMING_REGULARITY_SPREAD_SECONDS_CHALLENGE_PUZZLE,
        crate::challenge::operation_envelope::TIMING_HISTORY_TTL_SECONDS_CHALLENGE_PUZZLE,
    ) {
        Ok(_) => {}
        Err(crate::challenge::operation_envelope::TimingValidationError::TooFast) => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::SequenceTimingTooFast,
            )
        }
        Err(crate::challenge::operation_envelope::TimingValidationError::TooRegular) => {
            return (
                challenge_forbidden_response(),
                ChallengeSubmitOutcome::SequenceTimingTooRegular,
            )
        }
        Err(crate::challenge::operation_envelope::TimingValidationError::TooSlow) => {
            return (
                challenge_expired_response(),
                ChallengeSubmitOutcome::SequenceTimingTooSlow,
            )
        }
    }
    match crate::challenge::operation_envelope::validate_operation_replay(
        store,
        seed.flow_id.as_str(),
        seed.operation_id.as_str(),
        now,
        seed.expires_at,
        crate::challenge::operation_envelope::MAX_OPERATION_REPLAY_TTL_SECONDS_CHALLENGE_PUZZLE,
    ) {
        Ok(_) => {}
        Err(crate::challenge::operation_envelope::ReplayValidationError::ReplayDetected) => {
            return (
                challenge_expired_response(),
                ChallengeSubmitOutcome::SequenceOpReplay,
            )
        }
        Err(crate::challenge::operation_envelope::ReplayValidationError::ExpiredOperation) => {
            return (
                challenge_expired_response(),
                ChallengeSubmitOutcome::SequenceOpExpired,
            )
        }
    }
    let output = match parse_submission(&output_raw, seed.grid_size as usize) {
        Ok(v) => v,
        Err(_e) => {
            return (
                challenge_response(400, "Invalid output"),
                ChallengeSubmitOutcome::InvalidOutput,
            )
        }
    };
    let puzzle = build_puzzle(&seed);
    if output == puzzle.test_output {
        return (
            challenge_response(
                200,
                "<html><body><h2>Thank you! Challenge complete.</h2></body></html>",
            ),
            ChallengeSubmitOutcome::Solved,
        );
    }
    (
        challenge_incorrect_response(),
        ChallengeSubmitOutcome::Incorrect,
    )
}

#[cfg(test)]
pub fn handle_challenge_submit<S: KeyValueStore>(store: &S, req: &Request) -> Response {
    handle_challenge_submit_with_outcome(store, req).0
}

fn increment_and_check_attempt_limit<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip_bucket: &str,
    now: u64,
    attempt_window_seconds: u64,
    attempt_limit_per_window: u32,
) -> bool {
    let key = format!("challenge_puzzle:attempt:{}", ip_bucket);
    let mut state = store
        .get(key.as_str())
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_slice::<AttemptState>(raw.as_slice()).ok())
        .unwrap_or(AttemptState {
            window_start: now,
            count: 0,
        });

    if now.saturating_sub(state.window_start) >= attempt_window_seconds {
        state.window_start = now;
        state.count = 0;
    }
    state.count = state.count.saturating_add(1);

    if let Ok(encoded) = serde_json::to_vec(&state) {
        if let Err(err) = store.set(key.as_str(), encoded.as_slice()) {
            eprintln!(
                "[challenge] failed to persist puzzle attempt counter for {}: {:?}",
                key, err
            );
        }
    }

    state.count > attempt_limit_per_window
}

fn get_form_field(form: &str, name: &str) -> Option<String> {
    for pair in form.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            if k == name {
                return Some(url_decode(v));
            }
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .to_string()
}
