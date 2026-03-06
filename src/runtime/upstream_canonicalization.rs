const HOP_BY_HOP_HEADERS: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

const PRIVILEGED_REQUEST_HEADERS: [&str; 2] = ["authorization", "proxy-authorization"];

const FORWARDED_PROVENANCE_HEADERS: [&str; 6] = [
    "forwarded",
    "x-forwarded-for",
    "x-forwarded-host",
    "x-forwarded-proto",
    "x-forwarded-port",
    "x-real-ip",
];

pub(crate) fn normalize_header_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut normalized = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if !ch.is_ascii() {
            return None;
        }
        if ch.is_ascii_control() {
            return None;
        }
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() || lower == '-' {
            normalized.push(lower);
        } else {
            return None;
        }
    }
    Some(normalized)
}

pub(crate) fn parse_connection_tokens(raw: &str) -> Vec<String> {
    raw.split(',')
        .filter_map(normalize_header_name)
        .collect::<Vec<_>>()
}

pub(crate) fn should_strip_request_header(name: &str, connection_header: Option<&str>) -> bool {
    let Some(normalized_name) = normalize_header_name(name) else {
        return true;
    };
    if HOP_BY_HOP_HEADERS.contains(&normalized_name.as_str()) {
        return true;
    }
    if FORWARDED_PROVENANCE_HEADERS.contains(&normalized_name.as_str()) {
        return true;
    }
    if let Some(raw_connection) = connection_header {
        let connection_tokens = parse_connection_tokens(raw_connection);
        if connection_tokens
            .iter()
            .any(|token| token == normalized_name.as_str())
        {
            return true;
        }
    }
    false
}

pub(crate) fn is_privileged_request_header(name: &str) -> bool {
    let Some(normalized_name) = normalize_header_name(name) else {
        return true;
    };
    if PRIVILEGED_REQUEST_HEADERS.contains(&normalized_name.as_str()) {
        return true;
    }
    normalized_name.starts_with("x-shuma-")
}

pub(crate) fn should_strip_response_header(name: &str, connection_header: Option<&str>) -> bool {
    let Some(normalized_name) = normalize_header_name(name) else {
        return true;
    };
    if HOP_BY_HOP_HEADERS.contains(&normalized_name.as_str()) {
        return true;
    }
    if FORWARDED_PROVENANCE_HEADERS.contains(&normalized_name.as_str()) {
        return true;
    }
    if normalized_name.starts_with("x-shuma-") {
        return true;
    }
    if let Some(raw_connection) = connection_header {
        let connection_tokens = parse_connection_tokens(raw_connection);
        if connection_tokens
            .iter()
            .any(|token| token == normalized_name.as_str())
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
pub(crate) fn normalize_content_type(raw: &str) -> Option<String> {
    let value = raw
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or("")
        .to_ascii_lowercase();
    if value.is_empty() {
        return None;
    }
    if value.chars().any(|ch| ch.is_ascii_control() || ch == ' ') {
        return None;
    }
    if !value.contains('/') {
        return None;
    }
    Some(value)
}

pub(crate) fn canonicalize_forward_path(raw: &str) -> String {
    let without_fragment = raw.split('#').next().unwrap_or(raw).trim();
    let without_authority = if let Some(rest) = without_fragment.strip_prefix("//") {
        match rest.find(['/', '?']) {
            Some(index) => &rest[index..],
            None => "/",
        }
    } else if let Some((scheme, rest)) = without_fragment.split_once("://") {
        let valid_scheme = !scheme.is_empty()
            && scheme
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'));
        if valid_scheme {
            match rest.find(['/', '?']) {
                Some(index) => &rest[index..],
                None => "/",
            }
        } else {
            without_fragment
        }
    } else {
        without_fragment
    };
    let (path_part, query_part) = match without_authority.split_once('?') {
        Some((path, query)) => (path.trim(), Some(query)),
        None => (without_authority, None),
    };
    let mut normalized_path = if path_part.is_empty() {
        "/".to_string()
    } else if path_part.starts_with('/') {
        path_part.to_string()
    } else {
        format!("/{path_part}")
    };
    if let Some(query) = query_part {
        normalized_path.push('?');
        normalized_path.push_str(query);
    }
    normalized_path
}

#[cfg(test)]
mod tests {
    use super::{
        canonicalize_forward_path, is_privileged_request_header, normalize_content_type,
        normalize_header_name, parse_connection_tokens, should_strip_request_header,
        should_strip_response_header,
    };

    #[test]
    fn normalize_header_name_rejects_control_chars() {
        assert_eq!(
            normalize_header_name("Content-Type").as_deref(),
            Some("content-type")
        );
        assert!(normalize_header_name("x-bad\nheader").is_none());
        assert!(normalize_header_name("x bad").is_none());
    }

    #[test]
    fn strips_hop_by_hop_and_connection_token_headers() {
        assert!(should_strip_request_header("connection", None));
        assert!(should_strip_request_header("te", None));
        assert!(should_strip_request_header(
            "x-custom-token",
            Some("keep-alive, x-custom-token")
        ));
        assert!(!should_strip_request_header("content-type", Some("keep-alive")));
    }

    #[test]
    fn strips_privileged_request_headers() {
        assert!(is_privileged_request_header("authorization"));
        assert!(is_privileged_request_header("x-shuma-forwarded-secret"));
        assert!(!is_privileged_request_header("content-type"));
    }

    #[test]
    fn parse_connection_tokens_normalizes_mixed_case_input() {
        assert_eq!(
            parse_connection_tokens("Keep-Alive, X-Trace-Id"),
            vec!["keep-alive".to_string(), "x-trace-id".to_string()]
        );
    }

    #[test]
    fn normalize_content_type_handles_ambiguous_parameters() {
        assert_eq!(
            normalize_content_type("Application/JSON ; charset=utf-8").as_deref(),
            Some("application/json")
        );
        assert_eq!(
            normalize_content_type("text/plain; charset=utf-8; boundary=abc").as_deref(),
            Some("text/plain")
        );
        assert!(normalize_content_type("text plain").is_none());
    }

    #[test]
    fn canonicalize_forward_path_preserves_query_and_encoded_segments() {
        assert_eq!(
            canonicalize_forward_path("foo/%2Fbar?x=1&y=2#fragment"),
            "/foo/%2Fbar?x=1&y=2"
        );
        assert_eq!(canonicalize_forward_path(""), "/");
    }

    #[test]
    fn canonicalize_forward_path_strips_absolute_uri_authority() {
        assert_eq!(
            canonicalize_forward_path("http://127.0.0.1:3000/css/style.css"),
            "/css/style.css"
        );
        assert_eq!(
            canonicalize_forward_path("https://public.example.com?x=1"),
            "/?x=1"
        );
        assert_eq!(
            canonicalize_forward_path("//public.example.com/assets/app.js?x=1"),
            "/assets/app.js?x=1"
        );
    }

    #[test]
    fn strips_hop_by_hop_and_internal_response_headers() {
        assert!(should_strip_response_header("connection", None));
        assert!(should_strip_response_header("x-shuma-forward-reason", None));
        assert!(should_strip_response_header("x-forwarded-for", None));
        assert!(!should_strip_response_header("set-cookie", Some("keep-alive")));
        assert!(should_strip_response_header(
            "x-custom-token",
            Some("x-custom-token, keep-alive")
        ));
    }
}
