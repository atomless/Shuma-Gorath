use crate::config::{Config, IpRangePolicyAction, IpRangePolicyMode, IpRangePolicyRule};
use ipnet::IpNet;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::Mutex;

const CACHE_MAX_ENTRIES: usize = 16;
const MIN_IPV4_PREFIX_LEN: u8 = 8;
const MIN_IPV6_PREFIX_LEN: u8 = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchSource {
    CustomRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatchDetails {
    pub source: MatchSource,
    pub source_id: String,
    pub action: IpRangePolicyAction,
    pub matched_cidr: String,
    pub redirect_url: Option<String>,
    pub custom_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Evaluation {
    NoMatch,
    EmergencyAllowlisted { matched_cidr: String },
    Matched(MatchDetails),
}

#[derive(Debug, Clone)]
struct CompiledRule {
    source: MatchSource,
    source_id: String,
    action: IpRangePolicyAction,
    redirect_url: Option<String>,
    custom_message: Option<String>,
    nets: Vec<IpNet>,
}

#[derive(Debug, Clone, Default)]
struct CompiledPolicy {
    emergency_allowlist: Vec<IpNet>,
    custom_rules: Vec<CompiledRule>,
}

static COMPILED_POLICY_CACHE: Lazy<Mutex<HashMap<u64, CompiledPolicy>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn policy_cache_key(cfg: &Config) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    cfg.ip_range_policy_mode.as_str().hash(&mut hasher);
    if let Ok(bytes) = serde_json::to_vec(&cfg.ip_range_emergency_allowlist) {
        bytes.hash(&mut hasher);
    }
    if let Ok(bytes) = serde_json::to_vec(&cfg.ip_range_custom_rules) {
        bytes.hash(&mut hasher);
    }
    hasher.finish()
}

fn sanitize_redirect_url(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("https://") && !lower.starts_with("http://") {
        return None;
    }
    if trimmed.len() > 512 {
        return None;
    }
    Some(trimmed.to_string())
}

fn sanitize_custom_message(value: Option<&str>) -> Option<String> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
            continue;
        }
        out.push(ch);
        if out.chars().count() >= 280 {
            break;
        }
    }
    let trimmed = out.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn cidr_is_too_broad(net: &IpNet) -> bool {
    match net {
        IpNet::V4(v4) => v4.prefix_len() < MIN_IPV4_PREFIX_LEN,
        IpNet::V6(v6) => v6.prefix_len() < MIN_IPV6_PREFIX_LEN,
    }
}

pub(crate) fn parse_acceptable_cidr(raw: &str) -> Option<IpNet> {
    let candidate = raw.split('#').next().unwrap_or("").trim();
    if candidate.is_empty() {
        return None;
    }
    let net = candidate.parse::<IpNet>().ok()?;
    if cidr_is_too_broad(&net) {
        return None;
    }
    Some(net)
}

fn parse_cidr_list(cidrs: &[String]) -> Vec<IpNet> {
    let mut parsed = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in cidrs {
        let Some(net) = parse_acceptable_cidr(raw) else {
            continue;
        };
        let canonical = net.to_string();
        if seen.insert(canonical) {
            parsed.push(net);
        }
    }
    parsed
}

fn normalize_rule_id(raw: &str, fallback_index: usize) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return format!("custom_rule_{}", fallback_index);
    }
    let normalized = trimmed
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
        .collect::<String>();
    if normalized.is_empty() {
        format!("custom_rule_{}", fallback_index)
    } else {
        normalized
    }
}

fn compile_custom_rule(rule: &IpRangePolicyRule, index: usize) -> Option<CompiledRule> {
    if !rule.enabled {
        return None;
    }
    let nets = parse_cidr_list(&rule.cidrs);
    if nets.is_empty() {
        return None;
    }
    Some(CompiledRule {
        source: MatchSource::CustomRule,
        source_id: normalize_rule_id(rule.id.as_str(), index + 1),
        action: rule.action,
        redirect_url: sanitize_redirect_url(rule.redirect_url.as_deref()),
        custom_message: sanitize_custom_message(rule.custom_message.as_deref()),
        nets,
    })
}

fn compile_policy(cfg: &Config) -> CompiledPolicy {
    let emergency_allowlist = parse_cidr_list(&cfg.ip_range_emergency_allowlist);
    let custom_rules = cfg
        .ip_range_custom_rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| compile_custom_rule(rule, index))
        .collect::<Vec<_>>();
    CompiledPolicy {
        emergency_allowlist,
        custom_rules,
    }
}

fn compiled_policy_for(cfg: &Config) -> CompiledPolicy {
    let key = policy_cache_key(cfg);
    {
        let cache = COMPILED_POLICY_CACHE.lock().unwrap();
        if let Some(policy) = cache.get(&key) {
            return policy.clone();
        }
    }
    let policy = compile_policy(cfg);
    let mut cache = COMPILED_POLICY_CACHE.lock().unwrap();
    if cache.len() >= CACHE_MAX_ENTRIES {
        if let Some(oldest_key) = cache.keys().next().cloned() {
            cache.remove(&oldest_key);
        }
    }
    cache.insert(key, policy.clone());
    policy
}

fn match_rule(rule: &CompiledRule, ip: IpAddr) -> Option<MatchDetails> {
    for net in &rule.nets {
        if net.contains(&ip) {
            return Some(MatchDetails {
                source: rule.source.clone(),
                source_id: rule.source_id.clone(),
                action: rule.action,
                matched_cidr: net.to_string(),
                redirect_url: rule.redirect_url.clone(),
                custom_message: rule.custom_message.clone(),
            });
        }
    }
    None
}

fn evaluate_with_now(cfg: &Config, ip: &str, _now_unix: u64) -> Evaluation {
    if cfg.ip_range_policy_mode == IpRangePolicyMode::Off {
        return Evaluation::NoMatch;
    }
    let Ok(ip_addr) = ip.parse::<IpAddr>() else {
        return Evaluation::NoMatch;
    };
    let compiled = compiled_policy_for(cfg);

    for net in &compiled.emergency_allowlist {
        if net.contains(&ip_addr) {
            return Evaluation::EmergencyAllowlisted {
                matched_cidr: net.to_string(),
            };
        }
    }

    for rule in &compiled.custom_rules {
        if let Some(matched) = match_rule(rule, ip_addr) {
            return Evaluation::Matched(matched);
        }
    }

    Evaluation::NoMatch
}

pub(crate) fn evaluate(cfg: &Config, ip: &str) -> Evaluation {
    evaluate_with_now(cfg, ip, 0)
}

#[cfg(test)]
mod tests {
    use super::{evaluate, evaluate_with_now, Evaluation, MatchSource};
    use crate::config::{defaults, IpRangePolicyAction, IpRangePolicyMode, IpRangePolicyRule};

    #[test]
    fn emergency_allowlist_short_circuits_matches() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        cfg.ip_range_emergency_allowlist = vec!["203.0.113.0/24".to_string()];
        cfg.ip_range_custom_rules = vec![IpRangePolicyRule {
            id: "block_test".to_string(),
            enabled: true,
            cidrs: vec!["203.0.113.0/24".to_string()],
            action: IpRangePolicyAction::Forbidden403,
            redirect_url: None,
            custom_message: None,
        }];

        let result = evaluate(&cfg, "203.0.113.9");
        assert_eq!(
            result,
            Evaluation::EmergencyAllowlisted {
                matched_cidr: "203.0.113.0/24".to_string()
            }
        );
    }

    #[test]
    fn custom_rule_matches_and_returns_details() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        cfg.ip_range_custom_rules = vec![IpRangePolicyRule {
            id: "challenge_me".to_string(),
            enabled: true,
            cidrs: vec!["198.51.100.0/24".to_string()],
            action: IpRangePolicyAction::Maze,
            redirect_url: None,
            custom_message: None,
        }];

        let result = evaluate(&cfg, "198.51.100.10");
        let Evaluation::Matched(details) = result else {
            panic!("expected custom match");
        };
        assert_eq!(details.source, MatchSource::CustomRule);
        assert_eq!(details.source_id, "challenge_me");
        assert_eq!(details.action, IpRangePolicyAction::Maze);
        assert_eq!(details.matched_cidr, "198.51.100.0/24");
    }

    #[test]
    fn disabled_custom_rules_are_skipped() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        cfg.ip_range_custom_rules = vec![IpRangePolicyRule {
            id: "disabled".to_string(),
            enabled: false,
            cidrs: vec!["198.51.100.0/24".to_string()],
            action: IpRangePolicyAction::Forbidden403,
            redirect_url: None,
            custom_message: None,
        }];

        assert_eq!(evaluate_with_now(&cfg, "198.51.100.10", 123), Evaluation::NoMatch);
    }

    #[test]
    fn invalid_ip_or_mode_off_returns_no_match() {
        let mut cfg = defaults().clone();
        cfg.ip_range_policy_mode = IpRangePolicyMode::Off;
        assert_eq!(evaluate(&cfg, "203.0.113.4"), Evaluation::NoMatch);

        cfg.ip_range_policy_mode = IpRangePolicyMode::Enforce;
        assert_eq!(evaluate(&cfg, "not-an-ip"), Evaluation::NoMatch);
    }
}
