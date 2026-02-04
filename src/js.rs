/// Returns true if the request needs JS verification (no valid js_verified cookie),
/// but bypasses challenge for whitelisted browsers.
pub fn needs_js_verification_with_whitelist(req: &Request, _store: &Store, _site_id: &str, ip: &str, browser_whitelist: &[(String, u32)]) -> bool {
    // Check for browser whitelist
    let ua = req.header("user-agent").map(|v| v.as_str().unwrap_or("")).unwrap_or("");
    for (name, min_version) in browser_whitelist {
        if let Some(ver) = super::browser::extract_version(ua, name) {
            if ver >= *min_version {
                return false;
            }
        }
    }
    // Fallback to normal JS verification logic
    needs_js_verification(req, _store, _site_id, ip)
}
// src/js.rs
// JavaScript verification and quiz logic for WASM Bot Trap
// Handles JS-based bot detection and challenge/response for suspicious clients.

use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};

/// Secret used for HMAC token generation for JS verification cookies.
/// Pull from env to avoid a repo-known static secret in production.
fn get_js_secret() -> String {
    // Prefer dedicated JS_SECRET, fall back to API_KEY for dev convenience.
    std::env::var("JS_SECRET")
        .or_else(|_| std::env::var("API_KEY"))
        .unwrap_or_else(|_| "changeme-js-secret".to_string())
}

/// Generates a HMAC-based token for a given IP, used in the js_verified cookie.
fn make_token(ip: &str) -> String {
    let secret = get_js_secret();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(ip.as_bytes());
    let result = mac.finalize().into_bytes();
    general_purpose::STANDARD.encode(result)
}

/// Returns true if the request needs JS verification (no valid js_verified cookie).
/// Checks for a valid js_verified cookie matching the HMAC token for the IP.
pub fn needs_js_verification(req: &Request, _store: &Store, _site_id: &str, ip: &str) -> bool {
    // Check for a valid js_verified cookie
    if let Some(header) = req.header("cookie") {
        let cookie = header.as_str().unwrap_or("");
        for part in cookie.split(';') {
            let trimmed = part.trim();
            if let Some(val) = trimmed.strip_prefix("js_verified=") {
                if val == make_token(ip) {
                    return false;
                }
            }
        }
    }
    true
}

/// Returns a Response with a JS challenge page that sets the js_verified cookie for the client IP.
/// Also injects CDP detection if enabled in the config.
pub fn inject_js_challenge(ip: &str) -> Response {
        let token = make_token(ip);
        let cdp_script = crate::cdp::get_cdp_detection_script();
        let html = format!(r#"
        <html><head><script>{cdp_script}</script></head><body>
        <script>
            // Run CDP detection before allowing access
            if (window._checkCDPAutomation) {{
                window._checkCDPAutomation().then(function(result) {{
                    if (result.detected) {{
                        fetch('/cdp-report', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{
                                cdp_detected: true,
                                score: result.score,
                                checks: result.checks
                            }})
                        }});
                    }}
                }});
            }}
            document.cookie = 'js_verified={token}; path=/; SameSite=Strict; Max-Age=86400';
            window.location.reload();
    </script>
    <noscript>Please enable JavaScript to continue.</noscript>
    </body></html>
    "#);
    Response::new(200, html)
}
