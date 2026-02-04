/// Returns a simple admin identifier for event logging (e.g., 'admin' if authorized, '-' otherwise)
pub fn get_admin_id(req: &Request) -> String {
    if is_authorized(req) {
        "admin".to_string()
    } else {
        "-".to_string()
    }
}
// src/auth.rs
// Simple API key authentication for admin endpoints
// Checks for a static Bearer token in the Authorization header for admin access.

use spin_sdk::http::Request;
use crate::whitelist;


/// Returns the admin API key: uses the API_KEY environment variable if set, otherwise falls back to the hardcoded dev key.
fn get_admin_api_key() -> String {
    // Use Spin's std::env::var to read the environment variable if present
    std::env::var("API_KEY").unwrap_or_else(|_| "changeme-supersecret".to_string())
}

/// Returns true if the request contains a valid admin API key in the Authorization header.
/// Uses constant-time comparison to prevent timing attacks.
pub fn is_authorized(req: &Request) -> bool {
    if let Some(header) = req.header("authorization") {
        let val = header.as_str().unwrap_or("");
        let expected = format!("Bearer {}", get_admin_api_key());
        // Use constant-time comparison to prevent timing attacks
        if val.len() == expected.len() {
            let mut result = 0u8;
            for (a, b) in val.bytes().zip(expected.bytes()) {
                result |= a ^ b;
            }
            return result == 0;
        }
    }
    false
}

/// Returns true if admin access is allowed from this IP.
/// If ADMIN_IP_ALLOWLIST is unset, all IPs are allowed (auth still required).
pub fn is_admin_ip_allowed(req: &Request) -> bool {
    let list = match std::env::var("ADMIN_IP_ALLOWLIST") {
        Ok(v) => {
            let items: Vec<String> = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if items.is_empty() {
                return true;
            }
            items
        }
        Err(_) => return true,
    };

    let ip = crate::extract_client_ip(req);
    whitelist::is_whitelisted(&ip, &list)
}
