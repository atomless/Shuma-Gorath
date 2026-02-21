use super::*;
use spin_sdk::http::{Method, Request};

fn request(method: Method, path: &str) -> Request {
    let mut builder = Request::builder();
    builder.method(method).uri(path);
    builder.build()
}

#[test]
fn early_router_short_circuits_health_path() {
    let req = request(Method::Get, "/health");
    let resp = maybe_handle_early_route(&req, "/health");
    assert!(resp.is_some());
    assert_eq!(*resp.unwrap().status(), 403u16);
}

#[test]
fn early_router_short_circuits_admin_options() {
    let req = request(Method::Options, "/admin/config");
    let resp = maybe_handle_early_route(&req, "/admin/config");
    assert!(resp.is_some());
    assert_eq!(*resp.unwrap().status(), 403u16);
}

#[test]
fn early_router_does_not_consume_cdp_report_path() {
    let req = request(Method::Post, "/cdp-report");
    let resp = maybe_handle_early_route(&req, "/cdp-report");
    assert!(resp.is_none());
}

#[test]
fn early_router_does_not_consume_unrelated_paths() {
    let req = request(Method::Get, "/totally-unrelated");
    let resp = maybe_handle_early_route(&req, "/totally-unrelated");
    assert!(resp.is_none());
}

#[test]
fn early_router_short_circuits_maze_asset_paths() {
    let path = crate::maze::assets::maze_script_path();
    let req = request(Method::Get, path);
    let resp = maybe_handle_early_route(&req, path);
    assert!(resp.is_some());
    assert_eq!(*resp.unwrap().status(), 200u16);
}

#[test]
fn early_router_redirects_dashboard_root_to_index_html() {
    let req = request(Method::Get, "/dashboard");
    let resp = maybe_handle_early_route(&req, "/dashboard");
    assert!(resp.is_some());
    let resp = resp.unwrap();
    assert_eq!(*resp.status(), 308u16);
    let location = resp
        .headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("location"))
        .and_then(|(_, value)| value.as_str())
        .unwrap_or("");
    assert_eq!(location, "/dashboard/index.html");
}

#[test]
fn not_a_bot_failure_policy_routes_user_failures_to_maze() {
    assert_eq!(
        classify_not_a_bot_failure_enforcement(crate::challenge::NotABotSubmitOutcome::FailedScore),
        ChallengeFailureEnforcement::MazeFallback
    );
    assert_eq!(
        classify_not_a_bot_failure_enforcement(crate::challenge::NotABotSubmitOutcome::Expired),
        ChallengeFailureEnforcement::MazeFallback
    );
}

#[test]
fn not_a_bot_failure_policy_routes_abuse_to_tarpit_or_short_ban() {
    let abuse_outcomes = [
        crate::challenge::NotABotSubmitOutcome::Replay,
        crate::challenge::NotABotSubmitOutcome::InvalidSeed,
        crate::challenge::NotABotSubmitOutcome::MissingSeed,
        crate::challenge::NotABotSubmitOutcome::SequenceViolation,
        crate::challenge::NotABotSubmitOutcome::BindingMismatch,
        crate::challenge::NotABotSubmitOutcome::InvalidTelemetry,
        crate::challenge::NotABotSubmitOutcome::AttemptLimitExceeded,
    ];
    for outcome in abuse_outcomes {
        assert_eq!(
            classify_not_a_bot_failure_enforcement(outcome),
            ChallengeFailureEnforcement::TarpitOrShortBan
        );
    }
}

#[test]
fn challenge_puzzle_failure_policy_routes_user_failures_to_maze() {
    let maze_outcomes = [
        crate::boundaries::ChallengeSubmitOutcome::Incorrect,
        crate::boundaries::ChallengeSubmitOutcome::SequenceOpExpired,
        crate::boundaries::ChallengeSubmitOutcome::SequenceWindowExceeded,
        crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooSlow,
    ];
    for outcome in maze_outcomes {
        assert_eq!(
            classify_challenge_failure_enforcement(outcome),
            Some(ChallengeFailureEnforcement::MazeFallback)
        );
    }
    assert_eq!(
        classify_challenge_failure_enforcement(crate::boundaries::ChallengeSubmitOutcome::Solved),
        None
    );
}

#[test]
fn challenge_puzzle_failure_policy_routes_abuse_to_tarpit_or_short_ban() {
    let abuse_outcomes = [
        crate::boundaries::ChallengeSubmitOutcome::AttemptLimitExceeded,
        crate::boundaries::ChallengeSubmitOutcome::SequenceOpMissing,
        crate::boundaries::ChallengeSubmitOutcome::SequenceOpInvalid,
        crate::boundaries::ChallengeSubmitOutcome::SequenceOpReplay,
        crate::boundaries::ChallengeSubmitOutcome::SequenceOrderViolation,
        crate::boundaries::ChallengeSubmitOutcome::SequenceBindingMismatch,
        crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooFast,
        crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooRegular,
        crate::boundaries::ChallengeSubmitOutcome::Forbidden,
        crate::boundaries::ChallengeSubmitOutcome::InvalidOutput,
    ];
    for outcome in abuse_outcomes {
        assert_eq!(
            classify_challenge_failure_enforcement(outcome),
            Some(ChallengeFailureEnforcement::TarpitOrShortBan)
        );
    }
}
