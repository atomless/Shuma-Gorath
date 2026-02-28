use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

use crate::runtime::capabilities::{
    BanWriteCapability, EventLogCapability, MetricsCapability, MonitoringCapability,
    RuntimeCapabilities,
};

#[derive(Clone)]
pub(crate) struct BanIntent {
    pub reason: String,
    pub duration_seconds: u64,
    pub score: Option<u8>,
    pub signals: Vec<String>,
    pub summary: Option<String>,
}

pub(crate) enum EffectIntent {
    RecordPolicyMatch(crate::runtime::policy_taxonomy::PolicyTransition),
    IncrementMetric {
        metric: crate::observability::metrics::MetricName,
        label: Option<String>,
    },
    RecordRateViolation {
        path: Option<String>,
        outcome: String,
    },
    RecordGeoViolation {
        country: Option<String>,
        action: String,
    },
    RecordHoneypotHit {
        path: String,
    },
    RecordNotABotServed,
    RecordLikelyHumanSample {
        sample_percent: u8,
        sample_hint: String,
    },
    FlushPendingMonitoringCounters,
    LogEvent {
        event: crate::admin::EventType,
        reason: String,
        outcome: String,
    },
    Ban(BanIntent),
}

pub(crate) enum ResponseIntent {
    Continue,
    OkBody(String),
    BlockPage {
        status: u16,
        reason: crate::enforcement::block_page::BlockReason,
    },
    PlainTextBlock {
        body: String,
    },
    DropConnection,
    Redirect {
        location: String,
    },
    Maze {
        entry_path: String,
        event_reason: String,
        event_outcome: String,
        botness_score: Option<u8>,
    },
    Challenge,
    NotABot,
    JsChallenge,
    IpRangeTarpit {
        base_outcome: String,
        signal_ids: Vec<crate::runtime::policy_taxonomy::SignalId>,
    },
}

pub(crate) struct DecisionPlan {
    pub intents: Vec<EffectIntent>,
    pub response: ResponseIntent,
}

pub(crate) struct EffectExecutionContext<'a> {
    pub req: &'a Request,
    pub store: &'a Store,
    pub cfg: &'a crate::config::Config,
    pub provider_registry: &'a crate::providers::registry::ProviderRegistry,
    pub site_id: &'a str,
    pub ip: &'a str,
    pub ua: &'a str,
}

fn apply_metric_intent(
    _capability: &MetricsCapability,
    store: &Store,
    intent: EffectIntent,
) -> Option<EffectIntent> {
    match intent {
        EffectIntent::RecordPolicyMatch(transition) => {
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(transition);
            crate::observability::metrics::record_policy_match(store, &policy_match);
            None
        }
        EffectIntent::IncrementMetric { metric, label } => {
            crate::observability::metrics::increment(store, metric, label.as_deref());
            None
        }
        other => Some(other),
    }
}

fn apply_monitoring_intent(
    _capability: &MonitoringCapability,
    store: &Store,
    ip: &str,
    intent: EffectIntent,
) -> Option<EffectIntent> {
    match intent {
        EffectIntent::RecordRateViolation { path, outcome } => {
            crate::observability::monitoring::record_rate_violation_with_path(
                store,
                ip,
                path.as_deref(),
                outcome.as_str(),
            );
            None
        }
        EffectIntent::RecordGeoViolation { country, action } => {
            crate::observability::monitoring::record_geo_violation(store, country.as_deref(), action.as_str());
            None
        }
        EffectIntent::RecordHoneypotHit { path } => {
            crate::observability::monitoring::record_honeypot_hit(store, ip, path.as_str());
            None
        }
        EffectIntent::RecordNotABotServed => {
            crate::observability::monitoring::record_not_a_bot_served(store);
            None
        }
        EffectIntent::RecordLikelyHumanSample {
            sample_percent,
            sample_hint,
        } => {
            crate::observability::monitoring::maybe_record_ip_range_likely_human_sample(
                store,
                ip,
                sample_percent,
                sample_hint.as_str(),
            );
            None
        }
        EffectIntent::FlushPendingMonitoringCounters => {
            crate::observability::monitoring::flush_pending_counters(store);
            None
        }
        other => Some(other),
    }
}

fn apply_event_log_intent(
    _capability: &EventLogCapability,
    store: &Store,
    ip: &str,
    intent: EffectIntent,
) -> Option<EffectIntent> {
    match intent {
        EffectIntent::LogEvent {
            event,
            reason,
            outcome,
        } => {
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event,
                    ip: Some(ip.to_string()),
                    reason: Some(reason),
                    outcome: Some(outcome),
                    admin: None,
                },
            );
            None
        }
        other => Some(other),
    }
}

fn apply_ban_intent(
    _capability: &BanWriteCapability,
    store: &Store,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    site_id: &str,
    ip: &str,
    intent: EffectIntent,
) {
    if let EffectIntent::Ban(ban) = intent {
        provider_registry.ban_store_provider().ban_ip_with_fingerprint(
            store,
            site_id,
            ip,
            ban.reason.as_str(),
            ban.duration_seconds,
            Some(crate::enforcement::ban::BanFingerprint {
                score: ban.score,
                signals: ban.signals,
                summary: ban.summary,
            }),
        );
    }
}

fn ip_range_source_label(_source: &crate::signals::ip_range_policy::MatchSource) -> &'static str {
    "custom"
}

fn ip_range_signal_ids(
    _source: &crate::signals::ip_range_policy::MatchSource,
) -> Vec<crate::runtime::policy_taxonomy::SignalId> {
    vec![crate::runtime::policy_taxonomy::SignalId::IpRangeCustom]
}

fn ip_range_base_outcome(details: &crate::signals::ip_range_policy::MatchDetails) -> String {
    let source_label = ip_range_source_label(&details.source);
    format!(
        "source={} source_id={} action={} matched_cidr={}",
        source_label,
        details.source_id,
        details.action.as_str(),
        details.matched_cidr
    )
}

fn botness_base_outcome(facts: &crate::runtime::request_facts::RequestFacts) -> String {
    format!(
        "score={} signals={} signal_states={} {} providers={}",
        facts.botness_score,
        facts.botness_summary,
        facts.botness_state_summary,
        facts.runtime_metadata_summary,
        facts.provider_summary
    )
}

pub(crate) fn plan_for_decision(
    decision: &crate::runtime::policy_graph::PolicyDecision,
    facts: &crate::runtime::request_facts::RequestFacts,
    cfg: &crate::config::Config,
) -> DecisionPlan {
    use crate::runtime::policy_graph::PolicyDecision;
    use crate::runtime::policy_taxonomy::{resolve_policy_match, PolicyTransition, SignalId};

    match decision {
        PolicyDecision::IpRangeEmergencyAllowlisted { matched_cidr } => DecisionPlan {
            intents: vec![
                EffectIntent::IncrementMetric {
                    metric: crate::observability::metrics::MetricName::AllowlistedTotal,
                    label: None,
                },
                EffectIntent::LogEvent {
                    event: crate::admin::EventType::AdminAction,
                    reason: "ip_range_emergency_allowlist".to_string(),
                    outcome: format!("matched_cidr={}", matched_cidr),
                },
            ],
            response: ResponseIntent::OkBody("OK (ip range emergency allowlisted)".to_string()),
        },
        PolicyDecision::IpRangeAdvisory { details } => {
            let signal_ids = ip_range_signal_ids(&details.source);
            let policy_match = resolve_policy_match(PolicyTransition::IpRangeAdvisory(signal_ids));
            let base_outcome = ip_range_base_outcome(details);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeAdvisory(
                        ip_range_signal_ids(&details.source),
                    )),
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::AdminAction,
                        reason: "ip_range_policy_advisory".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::Continue,
            }
        }
        PolicyDecision::IpRangeForbidden { details } => {
            let signal_ids = ip_range_signal_ids(&details.source);
            let policy_match = resolve_policy_match(PolicyTransition::IpRangeForbidden(signal_ids));
            let base_outcome = ip_range_base_outcome(details);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeForbidden(
                        ip_range_signal_ids(&details.source),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "ip_range_policy_forbidden".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::IpRangePolicy,
                },
            }
        }
        PolicyDecision::IpRangeCustomMessage { details, message } => {
            let signal_ids = ip_range_signal_ids(&details.source);
            let policy_match =
                resolve_policy_match(PolicyTransition::IpRangeCustomMessage(signal_ids));
            let base_outcome = ip_range_base_outcome(details);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeCustomMessage(
                        ip_range_signal_ids(&details.source),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "ip_range_policy_custom_message".to_string(),
                        outcome: policy_match.annotate_outcome(
                            format!("{} message_len={}", base_outcome, message.len()).as_str(),
                        ),
                    },
                ],
                response: ResponseIntent::PlainTextBlock {
                    body: message.clone(),
                },
            }
        }
        PolicyDecision::IpRangeDropConnection { details } => {
            let signal_ids = ip_range_signal_ids(&details.source);
            let policy_match =
                resolve_policy_match(PolicyTransition::IpRangeDropConnection(signal_ids));
            let base_outcome = ip_range_base_outcome(details);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeDropConnection(
                        ip_range_signal_ids(&details.source),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "ip_range_policy_drop_connection".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::DropConnection,
            }
        }
        PolicyDecision::IpRangeRedirect { details, location } => {
            let base_outcome = ip_range_base_outcome(details);
            let signal_ids = ip_range_signal_ids(&details.source);
            if let Some(location) = location {
                let policy_match =
                    resolve_policy_match(PolicyTransition::IpRangeRedirect(signal_ids));
                return DecisionPlan {
                    intents: vec![
                        EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeRedirect(
                            ip_range_signal_ids(&details.source),
                        )),
                        EffectIntent::LogEvent {
                            event: crate::admin::EventType::AdminAction,
                            reason: "ip_range_policy_redirect".to_string(),
                            outcome: policy_match.annotate_outcome(
                                format!("{} location={}", base_outcome, location).as_str(),
                            ),
                        },
                    ],
                    response: ResponseIntent::Redirect {
                        location: location.clone(),
                    },
                };
            }

            let policy_match = resolve_policy_match(PolicyTransition::IpRangeForbidden(signal_ids));
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeForbidden(
                        ip_range_signal_ids(&details.source),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "ip_range_policy_redirect_missing_url".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::IpRangePolicy,
                },
            }
        }
        PolicyDecision::IpRangeRateLimit { details } => {
            let signal_ids = ip_range_signal_ids(&details.source);
            let policy_match =
                resolve_policy_match(PolicyTransition::IpRangeRateLimit(signal_ids));
            let base_outcome = ip_range_base_outcome(details);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeRateLimit(
                        ip_range_signal_ids(&details.source),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::RecordRateViolation {
                        path: Some(facts.path.clone()),
                        outcome: "limited".to_string(),
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "ip_range_policy_rate_limit".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 429,
                    reason: crate::enforcement::block_page::BlockReason::RateLimit,
                },
            }
        }
        PolicyDecision::IpRangeHoneypot { details } => {
            let signal_ids = ip_range_signal_ids(&details.source);
            let policy_match =
                resolve_policy_match(PolicyTransition::IpRangeHoneypot(signal_ids));
            let base_outcome = ip_range_base_outcome(details);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeHoneypot(
                        ip_range_signal_ids(&details.source),
                    )),
                    EffectIntent::Ban(BanIntent {
                        reason: "ip_range_honeypot".to_string(),
                        duration_seconds: cfg.get_ban_duration("honeypot"),
                        score: None,
                        signals: vec!["ip_range_policy".to_string()],
                        summary: Some(base_outcome.clone()),
                    }),
                    EffectIntent::RecordRateViolation {
                        path: Some(facts.path.clone()),
                        outcome: "banned".to_string(),
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BansTotal,
                        label: Some("ip_range_honeypot".to_string()),
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Ban,
                        reason: "ip_range_policy_honeypot".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::Honeypot,
                },
            }
        }
        PolicyDecision::IpRangeMaze { details } => {
            let signal_ids = ip_range_signal_ids(&details.source);
            let base_outcome = ip_range_base_outcome(details);
            if cfg.maze_enabled {
                let policy_match = resolve_policy_match(PolicyTransition::IpRangeMaze(signal_ids));
                return DecisionPlan {
                    intents: vec![EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeMaze(
                        ip_range_signal_ids(&details.source),
                    ))],
                    response: ResponseIntent::Maze {
                        entry_path: crate::maze::entry_path("ip-range-policy"),
                        event_reason: "ip_range_policy_maze".to_string(),
                        event_outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                        botness_score: None,
                    },
                };
            }

            if cfg.challenge_puzzle_enabled {
                let policy_match = resolve_policy_match(PolicyTransition::IpRangeMaze(signal_ids));
                return DecisionPlan {
                    intents: vec![
                        EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeMaze(
                            ip_range_signal_ids(&details.source),
                        )),
                        EffectIntent::IncrementMetric {
                            metric: crate::observability::metrics::MetricName::ChallengesTotal,
                            label: None,
                        },
                        EffectIntent::IncrementMetric {
                            metric: crate::observability::metrics::MetricName::ChallengeServedTotal,
                            label: None,
                        },
                        EffectIntent::LogEvent {
                            event: crate::admin::EventType::Challenge,
                            reason: "ip_range_policy_maze_fallback_challenge".to_string(),
                            outcome: policy_match.annotate_outcome(
                                format!("{} maze_disabled", base_outcome).as_str(),
                            ),
                        },
                    ],
                    response: ResponseIntent::Challenge,
                };
            }

            let policy_match = resolve_policy_match(PolicyTransition::IpRangeMaze(signal_ids));
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeMaze(
                        ip_range_signal_ids(&details.source),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "ip_range_policy_maze_fallback_block".to_string(),
                        outcome: policy_match.annotate_outcome(
                            format!("{} maze_disabled challenge_disabled", base_outcome).as_str(),
                        ),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::IpRangePolicy,
                },
            }
        }
        PolicyDecision::IpRangeTarpit { details } => {
            let signal_ids = ip_range_signal_ids(&details.source);
            DecisionPlan {
                intents: vec![EffectIntent::RecordPolicyMatch(PolicyTransition::IpRangeTarpit(
                    signal_ids.clone(),
                ))],
                response: ResponseIntent::IpRangeTarpit {
                    base_outcome: ip_range_base_outcome(details),
                    signal_ids,
                },
            }
        }
        PolicyDecision::HoneypotHit => {
            let policy_match = resolve_policy_match(PolicyTransition::HoneypotHit);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::HoneypotHit),
                    EffectIntent::Ban(BanIntent {
                        reason: "honeypot".to_string(),
                        duration_seconds: cfg.get_ban_duration("honeypot"),
                        score: None,
                        signals: vec!["honeypot".to_string()],
                        summary: Some(format!("path={}", facts.path)),
                    }),
                    EffectIntent::RecordHoneypotHit {
                        path: facts.path.clone(),
                    },
                    EffectIntent::RecordRateViolation {
                        path: Some(facts.path.clone()),
                        outcome: "banned".to_string(),
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BansTotal,
                        label: Some("honeypot".to_string()),
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Ban,
                        reason: "honeypot".to_string(),
                        outcome: policy_match.annotate_outcome("banned"),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::Honeypot,
                },
            }
        }
        PolicyDecision::RateLimitHit => {
            let policy_match = resolve_policy_match(PolicyTransition::RateLimitHit);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::RateLimitHit),
                    EffectIntent::Ban(BanIntent {
                        reason: "rate".to_string(),
                        duration_seconds: cfg.get_ban_duration("rate"),
                        score: None,
                        signals: vec!["rate_limit_exceeded".to_string()],
                        summary: Some(format!("rate_limit={}", cfg.rate_limit)),
                    }),
                    EffectIntent::RecordRateViolation {
                        path: Some(facts.path.clone()),
                        outcome: "banned".to_string(),
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BansTotal,
                        label: Some("rate_limit".to_string()),
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Ban,
                        reason: "rate".to_string(),
                        outcome: policy_match.annotate_outcome("banned"),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 429,
                    reason: crate::enforcement::block_page::BlockReason::RateLimit,
                },
            }
        }
        PolicyDecision::ExistingBan => {
            let policy_match = resolve_policy_match(PolicyTransition::ExistingBan);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::ExistingBan),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Ban,
                        reason: "banned".to_string(),
                        outcome: policy_match.annotate_outcome("block page"),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::Honeypot,
                },
            }
        }
        PolicyDecision::GeoBlock => {
            let country = facts.geo_country.clone();
            let country_summary = format!("country={}", country.as_deref().unwrap_or("unknown"));
            let policy_match = resolve_policy_match(PolicyTransition::GeoRouteBlock);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordGeoViolation {
                        country,
                        action: "block".to_string(),
                    },
                    EffectIntent::RecordPolicyMatch(PolicyTransition::GeoRouteBlock),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "geo_policy_block".to_string(),
                        outcome: policy_match.annotate_outcome(country_summary.as_str()),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::GeoPolicy,
                },
            }
        }
        PolicyDecision::GeoMaze => {
            let country = facts.geo_country.clone();
            let country_summary = format!("country={}", country.as_deref().unwrap_or("unknown"));
            let policy_match = resolve_policy_match(PolicyTransition::GeoRouteMaze);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordGeoViolation {
                        country,
                        action: "maze".to_string(),
                    },
                    EffectIntent::RecordPolicyMatch(PolicyTransition::GeoRouteMaze),
                ],
                response: ResponseIntent::Maze {
                    entry_path: crate::maze::entry_path("geo-policy"),
                    event_reason: "geo_policy_maze".to_string(),
                    event_outcome: policy_match.annotate_outcome(country_summary.as_str()),
                    botness_score: None,
                },
            }
        }
        PolicyDecision::GeoMazeFallbackChallenge => {
            let policy_match = resolve_policy_match(PolicyTransition::GeoRouteMazeFallbackChallenge);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordGeoViolation {
                        country: facts.geo_country.clone(),
                        action: "challenge".to_string(),
                    },
                    EffectIntent::RecordPolicyMatch(PolicyTransition::GeoRouteMazeFallbackChallenge),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengesTotal,
                        label: None,
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengeServedTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Challenge,
                        reason: "geo_policy_challenge_fallback".to_string(),
                        outcome: policy_match.annotate_outcome("maze_disabled"),
                    },
                ],
                response: ResponseIntent::Challenge,
            }
        }
        PolicyDecision::GeoChallenge => {
            let country_summary = format!(
                "country={}",
                facts.geo_country.as_deref().unwrap_or("unknown")
            );
            let policy_match = resolve_policy_match(PolicyTransition::GeoRouteChallenge);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordGeoViolation {
                        country: facts.geo_country.clone(),
                        action: "challenge".to_string(),
                    },
                    EffectIntent::RecordPolicyMatch(PolicyTransition::GeoRouteChallenge),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengesTotal,
                        label: None,
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengeServedTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Challenge,
                        reason: "geo_policy_challenge".to_string(),
                        outcome: policy_match.annotate_outcome(country_summary.as_str()),
                    },
                ],
                response: ResponseIntent::Challenge,
            }
        }
        PolicyDecision::GeoChallengeFallbackMaze => {
            let country_summary = format!(
                "country={}",
                facts.geo_country.as_deref().unwrap_or("unknown")
            );
            let policy_match = resolve_policy_match(PolicyTransition::ChallengeDisabledFallbackMaze(
                vec![SignalId::GeoRouteChallenge],
            ));
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordGeoViolation {
                        country: facts.geo_country.clone(),
                        action: "maze".to_string(),
                    },
                    EffectIntent::RecordPolicyMatch(PolicyTransition::ChallengeDisabledFallbackMaze(
                        vec![SignalId::GeoRouteChallenge],
                    )),
                ],
                response: ResponseIntent::Maze {
                    entry_path: crate::maze::entry_path("geo-policy-challenge-fallback"),
                    event_reason: "geo_policy_challenge_fallback_maze".to_string(),
                    event_outcome: policy_match.annotate_outcome(
                        format!("{} challenge_disabled", country_summary).as_str(),
                    ),
                    botness_score: None,
                },
            }
        }
        PolicyDecision::GeoFallbackBlockFromMaze => {
            let policy_match = resolve_policy_match(PolicyTransition::ChallengeDisabledFallbackBlock(
                vec![SignalId::GeoRouteMaze],
            ));
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordGeoViolation {
                        country: facts.geo_country.clone(),
                        action: "block".to_string(),
                    },
                    EffectIntent::RecordPolicyMatch(PolicyTransition::ChallengeDisabledFallbackBlock(
                        vec![SignalId::GeoRouteMaze],
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "geo_policy_challenge_disabled_fallback_block".to_string(),
                        outcome: policy_match
                            .annotate_outcome("maze_disabled challenge_disabled"),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::GeoPolicy,
                },
            }
        }
        PolicyDecision::GeoFallbackBlockFromChallenge => {
            let policy_match = resolve_policy_match(PolicyTransition::ChallengeDisabledFallbackBlock(
                vec![SignalId::GeoRouteChallenge],
            ));
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordGeoViolation {
                        country: facts.geo_country.clone(),
                        action: "block".to_string(),
                    },
                    EffectIntent::RecordPolicyMatch(PolicyTransition::ChallengeDisabledFallbackBlock(
                        vec![SignalId::GeoRouteChallenge],
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "geo_policy_challenge_disabled_fallback_block".to_string(),
                        outcome: policy_match
                            .annotate_outcome("challenge_disabled maze_disabled"),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::GeoPolicy,
                },
            }
        }
        PolicyDecision::BotnessMaze { score, signal_ids } => {
            let policy_match =
                resolve_policy_match(PolicyTransition::BotnessGateMaze(signal_ids.clone()));
            let base_outcome = botness_base_outcome(facts);
            DecisionPlan {
                intents: vec![EffectIntent::RecordPolicyMatch(
                    PolicyTransition::BotnessGateMaze(signal_ids.clone()),
                )],
                response: ResponseIntent::Maze {
                    entry_path: crate::maze::entry_path("botness-gate"),
                    event_reason: "botness_gate_maze".to_string(),
                    event_outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    botness_score: Some(*score),
                },
            }
        }
        PolicyDecision::BotnessNotABot { score: _, signal_ids } => {
            let policy_match =
                resolve_policy_match(PolicyTransition::BotnessGateNotABot(signal_ids.clone()));
            let base_outcome = botness_base_outcome(facts);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::BotnessGateNotABot(
                        signal_ids.clone(),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengesTotal,
                        label: None,
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::NotABotServedTotal,
                        label: None,
                    },
                    EffectIntent::RecordNotABotServed,
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Challenge,
                        reason: "botness_gate_not_a_bot".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::NotABot,
            }
        }
        PolicyDecision::BotnessChallenge { score: _, signal_ids } => {
            let policy_match =
                resolve_policy_match(PolicyTransition::BotnessGateChallenge(signal_ids.clone()));
            let base_outcome = botness_base_outcome(facts);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::BotnessGateChallenge(
                        signal_ids.clone(),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengesTotal,
                        label: None,
                    },
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengeServedTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Challenge,
                        reason: "botness_gate_challenge".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::Challenge,
            }
        }
        PolicyDecision::BotnessChallengeFallbackMaze { score, signal_ids } => {
            let policy_match = resolve_policy_match(PolicyTransition::ChallengeDisabledFallbackMaze(
                signal_ids.clone(),
            ));
            let base_outcome = botness_base_outcome(facts);
            DecisionPlan {
                intents: vec![EffectIntent::RecordPolicyMatch(
                    PolicyTransition::ChallengeDisabledFallbackMaze(signal_ids.clone()),
                )],
                response: ResponseIntent::Maze {
                    entry_path: crate::maze::entry_path("botness-challenge-fallback"),
                    event_reason: "botness_gate_challenge_disabled_fallback_maze".to_string(),
                    event_outcome: policy_match
                        .annotate_outcome(format!("{} challenge_disabled", base_outcome).as_str()),
                    botness_score: Some(*score),
                },
            }
        }
        PolicyDecision::BotnessChallengeFallbackBlock { score: _, signal_ids } => {
            let policy_match =
                resolve_policy_match(PolicyTransition::ChallengeDisabledFallbackBlock(
                    signal_ids.clone(),
                ));
            let base_outcome = botness_base_outcome(facts);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::ChallengeDisabledFallbackBlock(
                        signal_ids.clone(),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "botness_gate_challenge_disabled_fallback_block".to_string(),
                        outcome: policy_match.annotate_outcome(
                            format!("{} challenge_disabled maze_disabled", base_outcome).as_str(),
                        ),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::GeoPolicy,
                },
            }
        }
        PolicyDecision::JsChallengeRequired => {
            let policy_match = resolve_policy_match(PolicyTransition::JsVerificationRequired);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::JsVerificationRequired),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengesTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Challenge,
                        reason: "js_verification".to_string(),
                        outcome: policy_match.annotate_outcome("js challenge"),
                    },
                ],
                response: ResponseIntent::JsChallenge,
            }
        }
    }
}

pub(crate) fn execute_plan(
    plan: DecisionPlan,
    facts: &crate::runtime::request_facts::RequestFacts,
    context: &EffectExecutionContext<'_>,
    capabilities: &RuntimeCapabilities,
) -> Option<Response> {
    execute_effect_intents(plan.intents, context, capabilities);
    execute_response_intent(plan.response, facts, context, capabilities)
}

pub(crate) fn execute_effect_intents(
    intents: Vec<EffectIntent>,
    context: &EffectExecutionContext<'_>,
    capabilities: &RuntimeCapabilities,
) {
    for intent in intents {
        let Some(intent) = apply_metric_intent(capabilities.metrics(), context.store, intent) else {
            continue;
        };
        let Some(intent) =
            apply_monitoring_intent(capabilities.monitoring(), context.store, context.ip, intent)
        else {
            continue;
        };
        let Some(intent) =
            apply_event_log_intent(capabilities.event_log(), context.store, context.ip, intent)
        else {
            continue;
        };
        apply_ban_intent(
            capabilities.ban_write(),
            context.store,
            context.provider_registry,
            context.site_id,
            context.ip,
            intent,
        );
    }
}

pub(crate) fn execute_metric_intents(
    intents: Vec<EffectIntent>,
    store: &Store,
    capabilities: &RuntimeCapabilities,
) {
    for intent in intents {
        let Some(unhandled) = apply_metric_intent(capabilities.metrics(), store, intent) else {
            continue;
        };
        let _ = unhandled;
    }
}

pub(crate) fn execute_monitoring_store_intents(
    intents: Vec<EffectIntent>,
    store: &Store,
    capabilities: &RuntimeCapabilities,
) {
    for intent in intents {
        let Some(unhandled) = apply_monitoring_intent(capabilities.monitoring(), store, "", intent)
        else {
            continue;
        };
        let _ = unhandled;
    }
}

fn execute_response_intent(
    response_intent: ResponseIntent,
    facts: &crate::runtime::request_facts::RequestFacts,
    context: &EffectExecutionContext<'_>,
    capabilities: &RuntimeCapabilities,
) -> Option<Response> {
    match response_intent {
        ResponseIntent::Continue => None,
        ResponseIntent::OkBody(body) => Some(Response::new(200, body)),
        ResponseIntent::BlockPage { status, reason } => {
            Some(Response::new(status, crate::enforcement::block_page::render_block_page(reason)))
        }
        ResponseIntent::PlainTextBlock { body } => Some(
            Response::builder()
                .status(403)
                .header("Content-Type", "text/plain; charset=utf-8")
                .header("Cache-Control", "no-store")
                .body(body)
                .build(),
        ),
        ResponseIntent::DropConnection => Some(
            Response::builder()
                .status(444)
                .header("Connection", "close")
                .body("")
                .build(),
        ),
        ResponseIntent::Redirect { location } => Some(
            Response::builder()
                .status(308)
                .header("Location", location)
                .header("Cache-Control", "no-store")
                .body("")
                .build(),
        ),
        ResponseIntent::Maze {
            entry_path,
            event_reason,
            event_outcome,
            botness_score,
        } => Some(
            context
                .provider_registry
                .maze_tarpit_provider()
                .serve_maze_with_tracking(
                    context.req,
                    context.store,
                    context.cfg,
                    context.ip,
                    context.ua,
                    entry_path.as_str(),
                    event_reason.as_str(),
                    event_outcome.as_str(),
                    botness_score,
                ),
        ),
        ResponseIntent::Challenge => Some(
            context
                .provider_registry
                .challenge_engine_provider()
                .render_challenge(
                    context.req,
                    context.cfg.challenge_puzzle_transform_count as usize,
                    context.cfg.challenge_puzzle_seed_ttl_seconds,
                ),
        ),
        ResponseIntent::NotABot => {
            let not_a_bot_response = context
                .provider_registry
                .challenge_engine_provider()
                .render_not_a_bot(context.req, context.cfg);
            Some(crate::maze::covert_decoy::maybe_inject_non_maze_decoy(
                context.req,
                context.cfg,
                context.ip,
                context.ua,
                not_a_bot_response,
                facts.botness_score,
            ))
        }
        ResponseIntent::JsChallenge => {
            let report_endpoint = context
                .provider_registry
                .fingerprint_signal_provider()
                .report_path();
            Some(crate::signals::js_verification::inject_js_challenge(
                context.ip,
                context.ua,
                report_endpoint,
                context.cfg.pow_enabled,
                context.cfg.pow_difficulty,
                context.cfg.pow_ttl_seconds,
                context.cfg.cdp_probe_family,
                context.cfg.cdp_probe_rollout_percent,
            ))
        }
        ResponseIntent::IpRangeTarpit {
            base_outcome,
            signal_ids,
        } => {
            if let Some(response) = context
                .provider_registry
                .maze_tarpit_provider()
                .maybe_handle_tarpit(
                    context.req,
                    context.store,
                    context.cfg,
                    context.site_id,
                    context.ip,
                )
            {
                let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                    crate::runtime::policy_taxonomy::PolicyTransition::IpRangeTarpit(signal_ids),
                );
                let intent = EffectIntent::LogEvent {
                    event: crate::admin::EventType::Challenge,
                    reason: "ip_range_policy_tarpit".to_string(),
                    outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                };
                if let Some(intent) =
                    apply_event_log_intent(capabilities.event_log(), context.store, context.ip, intent)
                {
                    apply_ban_intent(
                        capabilities.ban_write(),
                        context.store,
                        context.provider_registry,
                        context.site_id,
                        context.ip,
                        intent,
                    );
                }
                return Some(response);
            }

            let transition = crate::runtime::policy_taxonomy::PolicyTransition::IpRangeTarpit(
                vec![crate::runtime::policy_taxonomy::SignalId::IpRangeCustom],
            );
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(transition);

            if context.cfg.maze_enabled {
                let event_outcome = policy_match.annotate_outcome(
                    format!("{} tarpit_unavailable fallback=maze", base_outcome).as_str(),
                );
                return Some(
                    context
                        .provider_registry
                        .maze_tarpit_provider()
                        .serve_maze_with_tracking(
                            context.req,
                            context.store,
                            context.cfg,
                            context.ip,
                            context.ua,
                            crate::maze::entry_path("ip-range-tarpit-fallback").as_str(),
                            "ip_range_policy_tarpit_fallback_maze",
                            event_outcome.as_str(),
                            None,
                        ),
                );
            }

            let block_intent = EffectIntent::IncrementMetric {
                metric: crate::observability::metrics::MetricName::BlocksTotal,
                label: None,
            };
            if let Some(intent) = apply_metric_intent(capabilities.metrics(), context.store, block_intent)
            {
                let _ = intent;
            }
            let log_intent = EffectIntent::LogEvent {
                event: crate::admin::EventType::Block,
                reason: "ip_range_policy_tarpit_fallback_block".to_string(),
                outcome: policy_match.annotate_outcome(
                    format!("{} tarpit_unavailable fallback=block", base_outcome).as_str(),
                ),
            };
            if let Some(intent) =
                apply_event_log_intent(capabilities.event_log(), context.store, context.ip, log_intent)
            {
                apply_ban_intent(
                    capabilities.ban_write(),
                    context.store,
                    context.provider_registry,
                    context.site_id,
                    context.ip,
                    intent,
                );
            }

            Some(Response::new(
                403,
                crate::enforcement::block_page::render_block_page(
                    crate::enforcement::block_page::BlockReason::IpRangePolicy,
                ),
            ))
        }
    }
}
