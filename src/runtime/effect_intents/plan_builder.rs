use super::intent_types::{BanIntent, DecisionPlan, EffectIntent, ResponseIntent};

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
