use spin_sdk::http::Response;

use crate::runtime::effect_intents::{ExecutionMode, ShadowAction};

pub(crate) fn effective_execution_mode(cfg: &crate::config::Config) -> ExecutionMode {
    if cfg.shadow_mode {
        ExecutionMode::Shadow
    } else {
        ExecutionMode::Enforced
    }
}

pub(crate) fn shadow_mode_active(cfg: &crate::config::Config) -> bool {
    matches!(effective_execution_mode(cfg), ExecutionMode::Shadow)
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
    "SHADOW MODE: Would allow"
}

pub(crate) fn synthetic_shadow_body(action: ShadowAction) -> &'static str {
    match action {
        ShadowAction::NotABot => "SHADOW MODE: Would serve Not-a-Bot",
        ShadowAction::Challenge => "SHADOW MODE: Would serve challenge",
        ShadowAction::JsChallenge => "SHADOW MODE: Would inject JS challenge",
        ShadowAction::Maze => "SHADOW MODE: Would route to maze",
        ShadowAction::Block => "SHADOW MODE: Would block",
        ShadowAction::Tarpit => "SHADOW MODE: Would tarpit",
        ShadowAction::Redirect => "SHADOW MODE: Would redirect",
        ShadowAction::DropConnection => "SHADOW MODE: Would drop connection",
    }
}

#[cfg(test)]
mod tests;
