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
    let (path_part, query_part) = match without_fragment.split_once('?') {
        Some((path, query)) => (path.trim(), Some(query)),
        None => (without_fragment, None),
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
        canonicalize_forward_path, normalize_content_type, normalize_header_name, parse_connection_tokens,
        should_strip_request_header,
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
}
