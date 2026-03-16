use super::*;

#[test]
fn effective_execution_mode_is_enforced_when_shadow_mode_disabled() {
    let mut cfg = crate::config::defaults().clone();
    cfg.shadow_mode = false;

    assert_eq!(effective_execution_mode(&cfg), ExecutionMode::Enforced);
    assert!(!shadow_mode_active(&cfg));
}

#[test]
fn effective_execution_mode_is_shadow_when_shadow_mode_enabled() {
    let mut cfg = crate::config::defaults().clone();
    cfg.shadow_mode = true;

    assert_eq!(effective_execution_mode(&cfg), ExecutionMode::Shadow);
    assert!(shadow_mode_active(&cfg));
}

#[test]
fn synthetic_shadow_body_uses_stable_labels() {
    assert_eq!(synthetic_shadow_allow_body(), "SHADOW MODE: Would allow");
    assert_eq!(
        synthetic_shadow_body(ShadowAction::JsChallenge),
        "SHADOW MODE: Would inject JS challenge"
    );
    assert_eq!(
        synthetic_shadow_body(ShadowAction::Maze),
        "SHADOW MODE: Would route to maze"
    );
    assert_eq!(
        synthetic_shadow_body(ShadowAction::Block),
        "SHADOW MODE: Would block"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn shadow_passthrough_requires_native_forwarding_capability_on_host_runtime() {
    let original_mode = std::env::var("SHUMA_GATEWAY_NATIVE_TEST_MODE").ok();
    std::env::set_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "https://origin.example.com");
    std::env::remove_var("SHUMA_GATEWAY_NATIVE_TEST_MODE");
    assert!(!shadow_passthrough_available());

    std::env::set_var("SHUMA_GATEWAY_NATIVE_TEST_MODE", "echo");
    assert!(shadow_passthrough_available());

    std::env::remove_var("SHUMA_GATEWAY_UPSTREAM_ORIGIN");
    if let Some(value) = original_mode {
        std::env::set_var("SHUMA_GATEWAY_NATIVE_TEST_MODE", value);
    } else {
        std::env::remove_var("SHUMA_GATEWAY_NATIVE_TEST_MODE");
    }
}
