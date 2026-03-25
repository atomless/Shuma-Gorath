use serde_json::json;
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::key_value::Store;

use crate::tarpit::runtime::{advance_progress, ProgressAdvanceOutcome};
use crate::tarpit::runtime::BudgetExhaustionReason;
use crate::tarpit::types::ProgressRejectReason;

const PROGRESS_PATH: &str = "/tarpit/progress";

pub(crate) fn progress_path() -> &'static str {
    PROGRESS_PATH
}

#[derive(Debug)]
pub(crate) struct ProgressHttpResult {
    pub response: Response,
    pub reject_reason: Option<ProgressRejectReason>,
    pub chunk_bytes: Option<usize>,
    pub budget_exhaustion_reason: Option<BudgetExhaustionReason>,
}

fn reject_status(reason: ProgressRejectReason) -> u16 {
    match reason {
        ProgressRejectReason::Malformed
        | ProgressRejectReason::SignatureMismatch
        | ProgressRejectReason::InvalidVersion
        | ProgressRejectReason::InvalidWindow
        | ProgressRejectReason::PathMismatch => 400,
        ProgressRejectReason::BudgetExhausted => 429,
        ProgressRejectReason::Expired
        | ProgressRejectReason::BindingIpMismatch
        | ProgressRejectReason::BindingUaMismatch
        | ProgressRejectReason::StepOutOfOrder
        | ProgressRejectReason::ParentChainMissing
        | ProgressRejectReason::Replay
        | ProgressRejectReason::InvalidProof => 403,
    }
}

pub(crate) fn handle_progress(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    site_id: &str,
    ip: &str,
    user_agent: &str,
) -> ProgressHttpResult {
    if *req.method() != Method::Post {
        return ProgressHttpResult {
            response: Response::new(405, "Method Not Allowed"),
            reject_reason: Some(ProgressRejectReason::Malformed),
            chunk_bytes: None,
            budget_exhaustion_reason: None,
        };
    }
    let payload = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_POW_VERIFY_BYTES,
    ) {
        Ok(payload) => payload,
        Err(_) => {
            return ProgressHttpResult {
                response: Response::new(400, "Invalid tarpit progress payload"),
                reject_reason: Some(ProgressRejectReason::Malformed),
                chunk_bytes: None,
                budget_exhaustion_reason: None,
            };
        }
    };
    let raw_token = payload
        .get("token")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let nonce = payload
        .get("nonce")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if raw_token.is_empty() || nonce.is_empty() {
        return ProgressHttpResult {
            response: Response::new(400, "Missing token or nonce"),
            reject_reason: Some(ProgressRejectReason::Malformed),
            chunk_bytes: None,
            budget_exhaustion_reason: None,
        };
    }

    let ip_bucket = crate::signals::ip_identity::bucket_ip(ip);
    let ua_bucket = crate::maze::token::ua_bucket(user_agent);
    let result = advance_progress(
        store,
        cfg,
        site_id,
        ip_bucket.as_str(),
        ua_bucket.as_str(),
        raw_token.as_str(),
        nonce.as_str(),
    );
    match result.outcome {
        ProgressAdvanceOutcome::Advanced => {
            let success = result.success.expect("advanced should include success payload");
            let response = Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(
                    json!({
                        "ok": true,
                        "flow_id": success.flow_id,
                        "step": success.step,
                        "chunk": success.chunk,
                        "chunk_bytes": success.chunk_bytes,
                        "flow_bytes_emitted": success.flow_bytes_emitted,
                        "difficulty": success.next_difficulty,
                        "token": success.next_token
                    })
                    .to_string()
                    .into_bytes(),
                )
                .build();
            ProgressHttpResult {
                response,
                reject_reason: None,
                chunk_bytes: Some(success.chunk_bytes),
                budget_exhaustion_reason: None,
            }
        }
        ProgressAdvanceOutcome::Reject(reason) => {
            let response = Response::builder()
                .status(reject_status(reason))
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(
                    json!({
                        "ok": false,
                        "reason": reason.as_str()
                    })
                    .to_string()
                    .into_bytes(),
                )
                .build();
            ProgressHttpResult {
                response,
                reject_reason: Some(reason),
                chunk_bytes: None,
                budget_exhaustion_reason: result.budget_exhaustion_reason,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_path_is_stable() {
        assert_eq!(progress_path(), "/tarpit/progress");
    }
}
