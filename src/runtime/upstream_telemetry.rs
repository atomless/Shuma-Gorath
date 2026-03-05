const MAX_FORWARD_REASON_CHARS: usize = 64;
const MAX_UPSTREAM_ORIGIN_CHARS: usize = 160;

pub(crate) fn normalize_forward_transport_class(raw: &str) -> &'static str {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "timeout" => "timeout",
        "transport" => "transport",
        "policy_denied" => "policy_denied",
        "misconfiguration" => "misconfiguration",
        "loop_detected" => "loop_detected",
        _ => "transport",
    }
}

pub(crate) fn sanitize_forward_reason(raw: &str) -> String {
    sanitize_tokenish(raw, MAX_FORWARD_REASON_CHARS, "unspecified")
}

pub(crate) fn sanitize_upstream_origin(raw: &str) -> String {
    sanitize_tokenish(raw, MAX_UPSTREAM_ORIGIN_CHARS, "unknown")
}

fn sanitize_tokenish(raw: &str, max_chars: usize, fallback: &str) -> String {
    let mut out = String::new();
    for ch in raw.trim().chars() {
        if out.len() >= max_chars {
            break;
        }
        let mapped = match ch {
            'A'..='Z' => ch.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '.' | ':' | '-' | '_' | '/' => ch,
            _ => '_',
        };
        out.push(mapped);
    }
    let collapsed = out
        .trim_matches('_')
        .split('_')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    if collapsed.is_empty() {
        fallback.to_string()
    } else {
        collapsed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ForwardProvenance {
    pub transport_class: &'static str,
    pub upstream_origin: String,
    pub forward_reason: String,
}

impl ForwardProvenance {
    pub(crate) fn new(transport_class: &str, upstream_origin: &str, forward_reason: &str) -> Self {
        Self {
            transport_class: normalize_forward_transport_class(transport_class),
            upstream_origin: sanitize_upstream_origin(upstream_origin),
            forward_reason: sanitize_forward_reason(forward_reason),
        }
    }

    pub(crate) fn as_event_fields(&self) -> String {
        format!(
            "transport_class={} upstream_origin={} forward_reason={}",
            self.transport_class, self.upstream_origin, self.forward_reason
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_forward_transport_class, sanitize_forward_reason, sanitize_upstream_origin,
        ForwardProvenance,
    };

    #[test]
    fn normalizes_forward_transport_class_to_allowed_vocab() {
        assert_eq!(normalize_forward_transport_class("timeout"), "timeout");
        assert_eq!(normalize_forward_transport_class(" LOOP_DETECTED "), "loop_detected");
        assert_eq!(
            normalize_forward_transport_class("unexpected_network_error"),
            "transport"
        );
    }

    #[test]
    fn sanitizes_forward_reason_and_origin() {
        assert_eq!(
            sanitize_forward_reason("  allowlist emergency\nbypass  "),
            "allowlist_emergency_bypass"
        );
        assert_eq!(
            sanitize_upstream_origin(" HTTPS://Example.COM:443/path?q=1 "),
            "https://example.com:443/path_q_1"
        );
    }

    #[test]
    fn forward_provenance_event_fields_are_deterministic() {
        let provenance = ForwardProvenance::new(
            "misconfiguration",
            "https://Origin.EXAMPLE.com:8443",
            "missing upstream config",
        );
        assert_eq!(
            provenance.as_event_fields(),
            "transport_class=misconfiguration upstream_origin=https://origin.example.com:8443 forward_reason=missing_upstream_config"
        );
    }
}
