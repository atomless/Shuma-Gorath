use crate::admin::{EventLogEntry, EventType};
use crate::config::Config;
use base64::{engine::general_purpose, Engine as _};
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

const MONITORING_PREFIX: &str = "monitoring:v1";
const HUMAN_SAMPLE_WEIGHT: f64 = 0.35;
const IPV4_BASE_BUCKET_PREFIX_LEN: u8 = 24;
const IPV6_BASE_BUCKET_PREFIX_LEN: u8 = 64;
const MAX_SUGGESTION_HOURS: u64 = 720;
const MAX_SUGGESTION_LIMIT: usize = 50;
const MAX_SAFER_ALTERNATIVES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IpFamily {
    Ipv4,
    Ipv6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IpRangeSuggestionRiskBand {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum IpRangeSuggestionMode {
    #[serde(rename = "logging-only")]
    LoggingOnly,
    #[serde(rename = "enforce")]
    Enforce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum IpRangeSuggestionAction {
    #[serde(rename = "deny_temp")]
    DenyTemp,
    #[serde(rename = "tarpit")]
    Tarpit,
    #[serde(rename = "logging-only")]
    LoggingOnly,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IpRangeSuggestion {
    pub cidr: String,
    pub ip_family: IpFamily,
    pub bot_evidence_score: f64,
    pub human_evidence_score: f64,
    pub collateral_risk: f64,
    pub confidence: f64,
    pub risk_band: IpRangeSuggestionRiskBand,
    pub recommended_action: IpRangeSuggestionAction,
    pub recommended_mode: IpRangeSuggestionMode,
    pub evidence_counts: BTreeMap<String, u64>,
    pub safer_alternatives: Vec<String>,
    pub guardrail_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct IpRangeSuggestionSummary {
    pub suggestions_total: usize,
    pub low_risk: usize,
    pub medium_risk: usize,
    pub high_risk: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IpRangeSuggestionsPayload {
    pub generated_at: u64,
    pub hours: u64,
    pub summary: IpRangeSuggestionSummary,
    pub suggestions: Vec<IpRangeSuggestion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NetKey {
    family: IpFamily,
    network: u128,
    prefix: u8,
}

#[derive(Debug, Clone, Default)]
struct Evidence {
    bot_score: f64,
    human_score: f64,
    bot_events: u64,
    human_events: u64,
    evidence_counts: BTreeMap<String, u64>,
}

#[derive(Debug, Clone)]
struct Candidate {
    net: NetKey,
    evidence: Evidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BucketKey {
    V4(u32),
    V6(u128),
}

#[derive(Debug, Clone, Copy)]
struct Score {
    collateral_risk: f64,
    confidence: f64,
    risk_band: IpRangeSuggestionRiskBand,
}

pub(crate) fn normalize_suggestion_hours(hours: u64) -> u64 {
    hours.clamp(1, MAX_SUGGESTION_HOURS)
}

pub(crate) fn normalize_suggestion_limit(limit: usize) -> usize {
    limit.clamp(1, MAX_SUGGESTION_LIMIT)
}

fn decode_dim(value: &str) -> String {
    general_purpose::URL_SAFE_NO_PAD
        .decode(value.as_bytes())
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_else(|| value.to_string())
}

fn parse_counter(bytes: Vec<u8>) -> u64 {
    String::from_utf8(bytes)
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0)
}

fn read_counter<S: crate::challenge::KeyValueStore>(store: &S, key: &str) -> u64 {
    store
        .get(key)
        .ok()
        .flatten()
        .map(parse_counter)
        .unwrap_or(0)
}

fn event_reason(entry: &EventLogEntry) -> String {
    entry
        .reason
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .unwrap_or_default()
}

fn bucket_from_ip(ip: &str) -> Option<BucketKey> {
    let addr = ip.parse::<IpAddr>().ok()?;
    bucket_from_addr(addr)
}

fn bucket_from_addr(addr: IpAddr) -> Option<BucketKey> {
    match addr {
        IpAddr::V4(v4) => {
            let raw = u32::from(v4);
            let masked = raw & (!0u32 << (32 - IPV4_BASE_BUCKET_PREFIX_LEN));
            Some(BucketKey::V4(masked))
        }
        IpAddr::V6(v6) => {
            let raw = u128::from_be_bytes(v6.octets());
            let masked = raw & (!0u128 << (128 - IPV6_BASE_BUCKET_PREFIX_LEN));
            Some(BucketKey::V6(masked))
        }
    }
}

fn bucket_to_net(bucket: BucketKey) -> NetKey {
    match bucket {
        BucketKey::V4(network) => NetKey {
            family: IpFamily::Ipv4,
            network: u128::from(network),
            prefix: IPV4_BASE_BUCKET_PREFIX_LEN,
        },
        BucketKey::V6(network) => NetKey {
            family: IpFamily::Ipv6,
            network,
            prefix: IPV6_BASE_BUCKET_PREFIX_LEN,
        },
    }
}

fn prefix_mask(bits: u8, prefix: u8) -> u128 {
    if prefix == 0 {
        0
    } else {
        (!0u128) << (bits - prefix)
    }
}

fn family_bits(family: IpFamily) -> u8 {
    match family {
        IpFamily::Ipv4 => 32,
        IpFamily::Ipv6 => 128,
    }
}

fn family_base_prefix(family: IpFamily) -> u8 {
    match family {
        IpFamily::Ipv4 => IPV4_BASE_BUCKET_PREFIX_LEN,
        IpFamily::Ipv6 => IPV6_BASE_BUCKET_PREFIX_LEN,
    }
}

fn parse_bucket_label(label: &str) -> Option<BucketKey> {
    let trimmed = label.trim();
    if trimmed.is_empty() || trimmed.starts_with('h') {
        return None;
    }
    if trimmed.contains('/') {
        let net = trimmed.parse::<IpNet>().ok()?;
        return match net {
            IpNet::V4(v4) => {
                let addr = IpAddr::V4(v4.network());
                bucket_from_addr(addr)
            }
            IpNet::V6(v6) => {
                let addr = IpAddr::V6(v6.network());
                bucket_from_addr(addr)
            }
        };
    }
    bucket_from_ip(trimmed)
}

fn net_to_cidr(net: NetKey) -> Option<String> {
    match net.family {
        IpFamily::Ipv4 => {
            let addr = Ipv4Addr::from(net.network as u32);
            Ipv4Net::new(addr, net.prefix).ok().map(|value| value.to_string())
        }
        IpFamily::Ipv6 => {
            let addr = Ipv6Addr::from(net.network);
            Ipv6Net::new(addr, net.prefix).ok().map(|value| value.to_string())
        }
    }
}

fn bucket_to_cidr(bucket: BucketKey) -> Option<String> {
    net_to_cidr(bucket_to_net(bucket))
}

fn parse_human_ip_key(key: &str) -> Option<(String, u64)> {
    let mut parts = key.split(':');
    match (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) {
        (
            Some("monitoring"),
            Some("v1"),
            Some("ip_range_suggestions"),
            Some("human_ip"),
            Some(encoded_bucket),
            Some(hour),
            None,
        ) => Some((decode_dim(encoded_bucket), hour.parse::<u64>().ok()?)),
        _ => None,
    }
}

fn add_count(map: &mut BTreeMap<String, u64>, key: &str, count: u64) {
    if count == 0 {
        return;
    }
    let entry = map.entry(key.to_string()).or_insert(0);
    *entry = entry.saturating_add(count);
}

fn add_bot_evidence(target: &mut Evidence, key: &str, weight: f64) {
    target.bot_score += weight.max(0.0);
    target.bot_events = target.bot_events.saturating_add(1);
    add_count(&mut target.evidence_counts, key, 1);
}

fn add_human_evidence(target: &mut Evidence, key: &str, weight: f64, count: u64) {
    target.human_score += (count as f64) * weight.max(0.0);
    target.human_events = target.human_events.saturating_add(count);
    add_count(&mut target.evidence_counts, key, count);
}

fn classify_bot_event(entry: &EventLogEntry, reason: &str) -> Option<(&'static str, f64)> {
    if reason.starts_with("ip_range_policy_") {
        return None;
    }
    if reason == "not_a_bot_pass" || reason == "challenge_puzzle_pass" {
        return None;
    }
    if reason.contains("honeypot") {
        return Some(("honeypot", 4.0));
    }
    if reason.contains("maze_crawler") || reason.contains("tarpit") {
        return Some(("tarpit_or_maze", 3.0));
    }
    if reason.contains("sequence") || reason.contains("replay") {
        return Some(("sequence_or_replay_abuse", 2.6));
    }
    if reason.contains("not_a_bot_fail") {
        return Some(("not_a_bot_fail", 2.4));
    }
    if reason.contains("cdp") {
        return Some(("cdp_automation", 2.2));
    }
    if reason.contains("rate") {
        return Some(("rate_violation", 1.8));
    }
    if reason.contains("pow") {
        return Some(("pow_violation", 1.8));
    }
    match entry.event {
        EventType::Ban => Some(("ban", 3.0)),
        EventType::Block => Some(("block", 2.0)),
        EventType::Challenge => Some(("challenge_fail", 1.4)),
        EventType::Unban | EventType::AdminAction => None,
    }
}

fn apply_event_evidence(events: &[EventLogEntry], map: &mut HashMap<BucketKey, Evidence>) {
    for entry in events {
        let Some(ip) = entry.ip.as_deref() else {
            continue;
        };
        let Some(bucket) = bucket_from_ip(ip) else {
            continue;
        };
        let reason = event_reason(entry);
        let evidence = map.entry(bucket).or_default();
        if reason == "not_a_bot_pass" {
            add_human_evidence(evidence, "not_a_bot_pass", 2.2, 1);
            continue;
        }
        if reason == "challenge_puzzle_pass" {
            add_human_evidence(evidence, "challenge_puzzle_pass", 1.9, 1);
            continue;
        }
        if let Some((key, weight)) = classify_bot_event(entry, reason.as_str()) {
            add_bot_evidence(evidence, key, weight);
        }
    }
}

fn apply_likely_human_samples<S: crate::challenge::KeyValueStore>(
    store: &S,
    start_hour: u64,
    end_hour: u64,
    map: &mut HashMap<BucketKey, Evidence>,
) {
    let Ok(keys) = store.get_keys() else {
        return;
    };
    for key in keys {
        if !key.starts_with(MONITORING_PREFIX) {
            continue;
        }
        let Some((bucket_label, hour)) = parse_human_ip_key(key.as_str()) else {
            continue;
        };
        if hour < start_hour || hour > end_hour {
            continue;
        }
        let Some(bucket) = parse_bucket_label(bucket_label.as_str()) else {
            continue;
        };
        let count = read_counter(store, key.as_str());
        if count == 0 {
            continue;
        }
        let evidence = map.entry(bucket).or_default();
        add_human_evidence(
            evidence,
            "likely_human_sampled_allow",
            HUMAN_SAMPLE_WEIGHT,
            count,
        );
    }
}

fn combine_evidence(target: &mut Evidence, incoming: &Evidence) {
    target.bot_score += incoming.bot_score;
    target.human_score += incoming.human_score;
    target.bot_events = target.bot_events.saturating_add(incoming.bot_events);
    target.human_events = target.human_events.saturating_add(incoming.human_events);
    for (key, value) in &incoming.evidence_counts {
        add_count(&mut target.evidence_counts, key.as_str(), *value);
    }
}

fn parent_net(net: NetKey, parent_prefix: u8) -> NetKey {
    let bits = family_bits(net.family);
    let masked = net.network & prefix_mask(bits, parent_prefix);
    NetKey {
        family: net.family,
        network: masked,
        prefix: parent_prefix,
    }
}

fn sibling_keys(parent: NetKey, child_prefix: u8) -> [NetKey; 2] {
    let bits = family_bits(parent.family);
    let child_size = 1u128 << (bits - child_prefix);
    let first = NetKey {
        family: parent.family,
        network: parent.network,
        prefix: child_prefix,
    };
    let second = NetKey {
        family: parent.family,
        network: parent.network.saturating_add(child_size),
        prefix: child_prefix,
    };
    [first, second]
}

fn merge_candidates_for_family(
    initial: &[Candidate],
    family: IpFamily,
    min_prefix: u8,
) -> Vec<Candidate> {
    let base_prefix = family_base_prefix(family);
    let mut map: BTreeMap<NetKey, Evidence> = initial
        .iter()
        .filter(|candidate| candidate.net.family == family)
        .map(|candidate| (candidate.net, candidate.evidence.clone()))
        .collect();

    for prefix in (min_prefix..base_prefix).rev() {
        let child_prefix = prefix.saturating_add(1);
        let child_keys: Vec<NetKey> = map
            .keys()
            .copied()
            .filter(|key| key.family == family && key.prefix == child_prefix)
            .collect();
        let mut parents = BTreeMap::new();
        for child in child_keys {
            parents.insert(parent_net(child, prefix), ());
        }

        for parent in parents.keys() {
            let [left, right] = sibling_keys(*parent, child_prefix);
            let Some(left_evidence) = map.get(&left).cloned() else {
                continue;
            };
            let Some(right_evidence) = map.get(&right).cloned() else {
                continue;
            };
            map.remove(&left);
            map.remove(&right);
            let mut merged = Evidence::default();
            combine_evidence(&mut merged, &left_evidence);
            combine_evidence(&mut merged, &right_evidence);
            map.entry(*parent)
                .and_modify(|existing| combine_evidence(existing, &merged))
                .or_insert(merged);
        }
    }

    map.into_iter()
        .map(|(net, evidence)| Candidate { net, evidence })
        .collect()
}

fn score_candidate(evidence: &Evidence, cfg: &Config) -> Score {
    let low_threshold = (cfg.ip_range_suggestions_low_collateral_percent as f64 / 100.0)
        .clamp(0.0, 1.0);
    let high_threshold = (cfg.ip_range_suggestions_high_collateral_percent as f64 / 100.0)
        .clamp(0.0, 1.0);
    let total_score = evidence.bot_score + evidence.human_score;
    let total_events = evidence.bot_events.saturating_add(evidence.human_events);
    let collateral_risk = if total_score <= f64::EPSILON {
        0.0
    } else {
        (evidence.human_score / total_score).clamp(0.0, 1.0)
    };
    let observation_floor = f64::from(cfg.ip_range_suggestions_min_observations.max(1));
    let volume = ((total_events as f64) / observation_floor).clamp(0.0, 1.0);
    let dominance = if total_score <= f64::EPSILON {
        0.0
    } else {
        (evidence.bot_score / total_score).clamp(0.0, 1.0)
    };
    let consistency = if total_events == 0 {
        0.0
    } else {
        ((evidence.bot_events as f64) / (total_events as f64)).clamp(0.0, 1.0)
    };
    let confidence = (0.5 * volume + 0.3 * dominance + 0.2 * consistency).clamp(0.0, 1.0);
    let risk_band = if collateral_risk <= low_threshold {
        IpRangeSuggestionRiskBand::Low
    } else if collateral_risk <= high_threshold {
        IpRangeSuggestionRiskBand::Medium
    } else {
        IpRangeSuggestionRiskBand::High
    };
    Score {
        collateral_risk,
        confidence,
        risk_band,
    }
}

fn recommendation_for(score: Score) -> (IpRangeSuggestionAction, IpRangeSuggestionMode) {
    if score.risk_band == IpRangeSuggestionRiskBand::High {
        return (
            IpRangeSuggestionAction::LoggingOnly,
            IpRangeSuggestionMode::LoggingOnly,
        );
    }
    if score.risk_band == IpRangeSuggestionRiskBand::Low && score.confidence >= 0.85 {
        return (IpRangeSuggestionAction::DenyTemp, IpRangeSuggestionMode::Enforce);
    }
    (IpRangeSuggestionAction::Tarpit, IpRangeSuggestionMode::Enforce)
}

fn bucket_is_within_net(bucket: BucketKey, net: NetKey) -> bool {
    match (bucket, net.family) {
        (BucketKey::V4(value), IpFamily::Ipv4) => {
            let masked = u128::from(value) & prefix_mask(32, net.prefix);
            masked == net.network
        }
        (BucketKey::V6(value), IpFamily::Ipv6) => {
            let masked = value & prefix_mask(128, net.prefix);
            masked == net.network
        }
        _ => false,
    }
}

fn safer_alternatives_for(
    net: NetKey,
    bucket_evidence: &HashMap<BucketKey, Evidence>,
    cfg: &Config,
) -> Vec<String> {
    let min_confidence = (cfg.ip_range_suggestions_min_confidence_percent as f64 / 100.0)
        .clamp(0.0, 1.0);
    let min_bot_events = u64::from(cfg.ip_range_suggestions_min_bot_events.max(1));
    let mut rows = Vec::new();
    for (bucket, evidence) in bucket_evidence {
        if !bucket_is_within_net(*bucket, net) {
            continue;
        }
        let score = score_candidate(evidence, cfg);
        if score.risk_band == IpRangeSuggestionRiskBand::High {
            continue;
        }
        if score.confidence < min_confidence || evidence.bot_events < min_bot_events {
            continue;
        }
        let Some(cidr) = bucket_to_cidr(*bucket) else {
            continue;
        };
        rows.push((score.confidence, cidr));
    }
    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.1.cmp(&b.1)));
    rows.dedup_by(|a, b| a.1 == b.1);
    rows.into_iter()
        .take(MAX_SAFER_ALTERNATIVES)
        .map(|(_, cidr)| cidr)
        .collect()
}

pub(crate) fn build_ip_range_suggestions<S: crate::challenge::KeyValueStore>(
    store: &S,
    cfg: &Config,
    events: &[EventLogEntry],
    now: u64,
    hours: u64,
    limit: usize,
) -> IpRangeSuggestionsPayload {
    let hours = normalize_suggestion_hours(hours);
    let limit = normalize_suggestion_limit(limit);
    let end_hour = now / 3600;
    let start_hour = end_hour.saturating_sub(hours.saturating_sub(1));

    let mut bucket_evidence: HashMap<BucketKey, Evidence> = HashMap::new();
    apply_event_evidence(events, &mut bucket_evidence);
    apply_likely_human_samples(store, start_hour, end_hour, &mut bucket_evidence);

    let initial_candidates: Vec<Candidate> = bucket_evidence
        .iter()
        .map(|(bucket, evidence)| Candidate {
            net: bucket_to_net(*bucket),
            evidence: evidence.clone(),
        })
        .collect();

    let requested_ipv4_min = cfg.ip_range_suggestions_ipv4_min_prefix_len;
    let requested_ipv6_min = cfg.ip_range_suggestions_ipv6_min_prefix_len;
    let effective_ipv4_min = requested_ipv4_min.min(IPV4_BASE_BUCKET_PREFIX_LEN);
    let effective_ipv6_min = requested_ipv6_min.min(IPV6_BASE_BUCKET_PREFIX_LEN);
    let min_confidence = (cfg.ip_range_suggestions_min_confidence_percent as f64 / 100.0)
        .clamp(0.0, 1.0);
    let min_observations = u64::from(cfg.ip_range_suggestions_min_observations.max(1));
    let min_bot_events = u64::from(cfg.ip_range_suggestions_min_bot_events.max(1));

    let mut candidates = Vec::new();
    candidates.extend(merge_candidates_for_family(
        &initial_candidates,
        IpFamily::Ipv4,
        effective_ipv4_min,
    ));
    candidates.extend(merge_candidates_for_family(
        &initial_candidates,
        IpFamily::Ipv6,
        effective_ipv6_min,
    ));

    let mut suggestions = Vec::new();
    for candidate in candidates {
        let total_events = candidate
            .evidence
            .bot_events
            .saturating_add(candidate.evidence.human_events);
        if total_events < min_observations || candidate.evidence.bot_events < min_bot_events {
            continue;
        }
        let score = score_candidate(&candidate.evidence, cfg);
        if score.confidence < min_confidence {
            continue;
        }
        let mut guardrail_notes = Vec::new();
        if candidate.net.family == IpFamily::Ipv4 && requested_ipv4_min > IPV4_BASE_BUCKET_PREFIX_LEN
        {
            guardrail_notes.push(
                "ipv4_guardrail_clamped_to_/24_bucket_granularity".to_string(),
            );
        }
        if candidate.net.family == IpFamily::Ipv6 && requested_ipv6_min > IPV6_BASE_BUCKET_PREFIX_LEN
        {
            guardrail_notes.push(
                "ipv6_guardrail_clamped_to_/64_bucket_granularity".to_string(),
            );
        }

        let family_base_prefix = family_base_prefix(candidate.net.family);
        let mut safer_alternatives = Vec::new();
        if score.risk_band == IpRangeSuggestionRiskBand::High {
            if candidate.net.prefix < family_base_prefix {
                safer_alternatives = safer_alternatives_for(candidate.net, &bucket_evidence, cfg);
                if safer_alternatives.is_empty() {
                    continue;
                }
                guardrail_notes.push("high_collateral_split_recommended".to_string());
            } else {
                continue;
            }
        }

        let (recommended_action, recommended_mode) = recommendation_for(score);
        let Some(cidr) = net_to_cidr(candidate.net) else {
            continue;
        };
        suggestions.push(IpRangeSuggestion {
            cidr,
            ip_family: candidate.net.family,
            bot_evidence_score: (candidate.evidence.bot_score * 100.0).round() / 100.0,
            human_evidence_score: (candidate.evidence.human_score * 100.0).round() / 100.0,
            collateral_risk: (score.collateral_risk * 1000.0).round() / 1000.0,
            confidence: (score.confidence * 1000.0).round() / 1000.0,
            risk_band: score.risk_band,
            recommended_action,
            recommended_mode,
            evidence_counts: candidate.evidence.evidence_counts.clone(),
            safer_alternatives,
            guardrail_notes,
        });
    }

    suggestions.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.bot_evidence_score
                    .partial_cmp(&a.bot_evidence_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.cidr.cmp(&b.cidr))
    });
    suggestions.truncate(limit);

    let mut summary = IpRangeSuggestionSummary {
        suggestions_total: suggestions.len(),
        low_risk: 0,
        medium_risk: 0,
        high_risk: 0,
    };
    for suggestion in &suggestions {
        match suggestion.risk_band {
            IpRangeSuggestionRiskBand::Low => summary.low_risk += 1,
            IpRangeSuggestionRiskBand::Medium => summary.medium_risk += 1,
            IpRangeSuggestionRiskBand::High => summary.high_risk += 1,
        }
    }

    IpRangeSuggestionsPayload {
        generated_at: now,
        hours,
        summary,
        suggestions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenge::KeyValueStore;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl KeyValueStore for MockStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            let map = self.map.lock().unwrap();
            Ok(map.get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            let mut map = self.map.lock().unwrap();
            map.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            let mut map = self.map.lock().unwrap();
            map.remove(key);
            Ok(())
        }

        fn get_keys(&self) -> Result<Vec<String>, ()> {
            let map = self.map.lock().unwrap();
            Ok(map.keys().cloned().collect())
        }
    }

    fn set_counter(store: &MockStore, key: &str, value: u64) {
        store
            .set(key, value.to_string().as_bytes())
            .expect("counter write should succeed");
    }

    fn make_event(ip: &str, event: EventType, reason: &str) -> EventLogEntry {
        EventLogEntry {
            ts: 1_700_000_000,
            event,
            ip: Some(ip.to_string()),
            reason: Some(reason.to_string()),
            outcome: Some("ok".to_string()),
            admin: None,
        }
    }

    #[test]
    fn suggestion_generation_emits_low_collateral_enforce_recommendation() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.ip_range_suggestions_min_observations = 1;
        cfg.ip_range_suggestions_min_bot_events = 1;
        cfg.ip_range_suggestions_min_confidence_percent = 1;
        cfg.ip_range_suggestions_low_collateral_percent = 10;
        cfg.ip_range_suggestions_high_collateral_percent = 25;

        let events = vec![
            make_event("198.51.100.10", EventType::Ban, "honeypot"),
            make_event("198.51.100.20", EventType::Ban, "honeypot"),
        ];
        let payload = build_ip_range_suggestions(&store, &cfg, &events, 1_700_000_000, 24, 20);
        assert_eq!(payload.suggestions.len(), 1);
        let first = payload.suggestions.first().unwrap();
        assert_eq!(first.cidr, "198.51.100.0/24");
        assert_eq!(first.recommended_mode, IpRangeSuggestionMode::Enforce);
        assert!(
            first.recommended_action == IpRangeSuggestionAction::DenyTemp
                || first.recommended_action == IpRangeSuggestionAction::Tarpit
        );
    }

    #[test]
    fn high_collateral_base_bucket_is_suppressed() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.ip_range_suggestions_min_observations = 1;
        cfg.ip_range_suggestions_min_bot_events = 1;
        cfg.ip_range_suggestions_min_confidence_percent = 1;
        cfg.ip_range_suggestions_low_collateral_percent = 5;
        cfg.ip_range_suggestions_high_collateral_percent = 20;

        let now = 1_700_000_000u64;
        let hour = now / 3600;
        let encoded_bucket = general_purpose::URL_SAFE_NO_PAD.encode("198.51.100.0");
        let key = format!(
            "{}:ip_range_suggestions:human_ip:{}:{}",
            MONITORING_PREFIX, encoded_bucket, hour
        );
        set_counter(&store, key.as_str(), 100);

        let events = vec![make_event("198.51.100.42", EventType::Ban, "ban")];
        let payload = build_ip_range_suggestions(&store, &cfg, &events, now, 24, 20);
        assert!(payload.suggestions.is_empty());
    }

    #[test]
    fn ipv6_parent_with_high_collateral_returns_safer_split_alternatives() {
        let store = MockStore::default();
        let mut cfg = crate::config::defaults().clone();
        cfg.ip_range_suggestions_min_observations = 1;
        cfg.ip_range_suggestions_min_bot_events = 1;
        cfg.ip_range_suggestions_min_confidence_percent = 1;
        cfg.ip_range_suggestions_low_collateral_percent = 10;
        cfg.ip_range_suggestions_high_collateral_percent = 20;
        cfg.ip_range_suggestions_ipv6_min_prefix_len = 63;

        let now = 1_700_000_000u64;
        let hour = now / 3600;
        let encoded_human_bucket = general_purpose::URL_SAFE_NO_PAD.encode("2001:db8:1:1::/64");
        let human_key = format!(
            "{}:ip_range_suggestions:human_ip:{}:{}",
            MONITORING_PREFIX, encoded_human_bucket, hour
        );
        set_counter(&store, human_key.as_str(), 12);

        let events = vec![
            make_event("2001:db8:1:0::11", EventType::Ban, "honeypot"),
            make_event("2001:db8:1:1::22", EventType::Ban, "honeypot"),
            make_event("2001:db8:1:0::33", EventType::Ban, "honeypot"),
        ];
        let payload = build_ip_range_suggestions(&store, &cfg, &events, now, 24, 20);
        let parent = payload
            .suggestions
            .iter()
            .find(|suggestion| suggestion.cidr == "2001:db8:1::/63")
            .expect("expected merged /63 suggestion");
        assert_eq!(
            parent.recommended_action,
            IpRangeSuggestionAction::LoggingOnly
        );
        assert!(parent
            .guardrail_notes
            .iter()
            .any(|note| note == "high_collateral_split_recommended"));
        assert!(!parent.safer_alternatives.is_empty());
    }
}
