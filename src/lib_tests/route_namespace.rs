use spin_sdk::http::Method;

#[test]
fn static_bypass_preserves_route_namespace_public_and_shuma_control_paths() {
    let robots_path = crate::http_route_namespace::PUBLIC_ROBOTS_TXT_PATH;
    let robots_req =
        crate::test_support::request_with_method_and_headers(Method::Get, robots_path, &[]);
    assert!(!crate::should_bypass_expensive_bot_checks_for_static(
        &robots_req,
        robots_path
    ));

    let health_path = crate::http_route_namespace::SHUMA_HEALTH_PATH;
    let health_req =
        crate::test_support::request_with_method_and_headers(Method::Get, health_path, &[]);
    assert!(!crate::should_bypass_expensive_bot_checks_for_static(
        &health_req,
        health_path
    ));

    let admin_path = "/shuma/admin/config";
    let admin_req =
        crate::test_support::request_with_method_and_headers(Method::Get, admin_path, &[]);
    assert!(!crate::should_bypass_expensive_bot_checks_for_static(
        &admin_req,
        admin_path
    ));
}
