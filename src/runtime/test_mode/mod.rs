use spin_sdk::http::Response;

use crate::runtime::effect_intents::{ExecutionMode, ShadowAction, ShadowSource};

pub(crate) fn effective_execution_mode(cfg: &crate::config::Config) -> ExecutionMode {
    if cfg.test_mode {
        ExecutionMode::Shadow {
            source: ShadowSource::TestMode,
        }
    } else {
        ExecutionMode::Enforced
    }
}

pub(crate) fn shadow_mode_active(cfg: &crate::config::Config) -> bool {
    effective_execution_mode(cfg).shadow_source().is_some()
}

pub(crate) fn shadow_passthrough_available() -> bool {
    crate::config::gateway_upstream_origin()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        && crate::runtime::upstream_proxy::forwarding_available_in_current_runtime()
}

pub(crate) fn synthetic_shadow_response(action: ShadowAction) -> Response {
    Response::new(200, synthetic_shadow_body(action))
}

pub(crate) fn synthetic_shadow_allow_response() -> Response {
    Response::new(200, synthetic_shadow_allow_body())
}

pub(crate) fn synthetic_shadow_allow_body() -> &'static str {
    "TEST MODE: Would allow"
}

pub(crate) fn synthetic_shadow_body(action: ShadowAction) -> &'static str {
    match action {
        ShadowAction::NotABot => "TEST MODE: Would serve Not-a-Bot",
        ShadowAction::Challenge => "TEST MODE: Would serve challenge",
        ShadowAction::JsChallenge => "TEST MODE: Would inject JS challenge",
        ShadowAction::Maze => "TEST MODE: Would route to maze",
        ShadowAction::Block => "TEST MODE: Would block",
        ShadowAction::Tarpit => "TEST MODE: Would tarpit",
        ShadowAction::Redirect => "TEST MODE: Would redirect",
        ShadowAction::DropConnection => "TEST MODE: Would drop connection",
    }
}

#[cfg(test)]
mod tests;
