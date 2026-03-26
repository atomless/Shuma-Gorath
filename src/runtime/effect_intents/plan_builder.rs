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

fn compact_botness_outcome(outcome_code: &str, score: u8) -> String {
    format!("{outcome_code} score={score}")
}

fn verified_identity_signal_ids(
    resolution: &crate::bot_identity::policy::IdentityPolicyResolution,
) -> Vec<crate::runtime::policy_taxonomy::SignalId> {
    use crate::bot_identity::policy::IdentityPolicyResolutionSourceKind;
    use crate::runtime::policy_taxonomy::SignalId;

    let mut signal_ids = vec![SignalId::VerifiedIdentityAuthenticated];
    signal_ids.push(match resolution.source_kind() {
        IdentityPolicyResolutionSourceKind::NamedPolicy => SignalId::VerifiedIdentityNamedPolicy,
        IdentityPolicyResolutionSourceKind::CategoryDefault => {
            SignalId::VerifiedIdentityCategoryDefault
        }
        IdentityPolicyResolutionSourceKind::CanonicalCategoryPosture => {
            SignalId::VerifiedIdentityCanonicalPostureFallback
        }
    });
    signal_ids
}

fn verified_identity_base_outcome(
    facts: &crate::runtime::request_facts::RequestFacts,
    resolution: &crate::bot_identity::policy::IdentityPolicyResolution,
) -> String {
    let identity = facts
        .verified_identity
        .as_ref()
        .expect("verified-identity policy decisions require identity facts");
    let mut parts = vec![
        format!("source={}", resolution.source_label()),
        format!("source_id={}", resolution.source_id()),
        format!("profile_id={}", resolution.profile_id),
        format!(
            "verified_identity_override_mode={}",
            resolution.verified_identity_override_mode
        ),
        format!("canonical_category_id={}", resolution.canonical_category_id),
        format!("base_posture={}", resolution.base_posture),
        format!("policy_outcome={}", resolution.outcome.as_str()),
        format!("scheme={}", identity.scheme.as_str()),
        format!("operator={}", identity.operator),
        format!("stable_identity={}", identity.stable_identity),
        format!("category={}", identity.category.as_str()),
    ];
    if let Some(service_profile_id) = resolution.service_profile_id.as_deref() {
        parts.push(format!("service_profile_id={service_profile_id}"));
    }
    parts.join(" ")
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
            response: ResponseIntent::ForwardAllow {
                reason: "ip_range_emergency_allowlist".to_string(),
            },
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
                        duration_seconds: cfg.get_ban_duration("ip_range_honeypot"),
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
                        duration_seconds: cfg.get_ban_duration("rate_limit"),
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
        PolicyDecision::VerifiedIdentityPolicyDeny { resolution } => {
            let signal_ids = verified_identity_signal_ids(resolution);
            let policy_match =
                resolve_policy_match(PolicyTransition::VerifiedIdentityPolicyDeny(signal_ids));
            let base_outcome = verified_identity_base_outcome(facts, resolution);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::VerifiedIdentityPolicyDeny(
                        verified_identity_signal_ids(resolution),
                    )),
                    EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::BlocksTotal,
                        label: None,
                    },
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::Block,
                        reason: "verified_identity_policy_deny".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::BlockPage {
                    status: 403,
                    reason: crate::enforcement::block_page::BlockReason::VerifiedIdentityPolicy,
                },
            }
        }
        PolicyDecision::VerifiedIdentityPolicyAllow { resolution } => {
            let signal_ids = verified_identity_signal_ids(resolution);
            let policy_match =
                resolve_policy_match(PolicyTransition::VerifiedIdentityPolicyAllow(signal_ids));
            let base_outcome = verified_identity_base_outcome(facts, resolution);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(PolicyTransition::VerifiedIdentityPolicyAllow(
                        verified_identity_signal_ids(resolution),
                    )),
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::AdminAction,
                        reason: "verified_identity_policy_allow".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::ForwardAllow {
                    reason: "verified_identity_policy_allow".to_string(),
                },
            }
        }
        PolicyDecision::VerifiedIdentityPolicyObserve { resolution } => {
            let signal_ids = verified_identity_signal_ids(resolution);
            let policy_match =
                resolve_policy_match(PolicyTransition::VerifiedIdentityPolicyObserve(signal_ids));
            let base_outcome = verified_identity_base_outcome(facts, resolution);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(
                        PolicyTransition::VerifiedIdentityPolicyObserve(
                            verified_identity_signal_ids(resolution),
                        ),
                    ),
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::AdminAction,
                        reason: "verified_identity_policy_observe".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::Continue,
            }
        }
        PolicyDecision::VerifiedIdentityPolicyRestrict { resolution } => {
            let signal_ids = verified_identity_signal_ids(resolution);
            let policy_match =
                resolve_policy_match(PolicyTransition::VerifiedIdentityPolicyRestrict(signal_ids));
            let base_outcome = verified_identity_base_outcome(facts, resolution);
            DecisionPlan {
                intents: vec![
                    EffectIntent::RecordPolicyMatch(
                        PolicyTransition::VerifiedIdentityPolicyRestrict(
                            verified_identity_signal_ids(resolution),
                        ),
                    ),
                    EffectIntent::LogEvent {
                        event: crate::admin::EventType::AdminAction,
                        reason: "verified_identity_policy_restrict".to_string(),
                        outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                    },
                ],
                response: ResponseIntent::Continue,
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
            DecisionPlan {
                intents: vec![EffectIntent::RecordPolicyMatch(
                    PolicyTransition::BotnessGateMaze(signal_ids.clone()),
                )],
                response: ResponseIntent::Maze {
                    entry_path: crate::maze::entry_path("botness-gate"),
                    event_reason: "botness_gate_maze".to_string(),
                    event_outcome: policy_match
                        .annotate_outcome(compact_botness_outcome("served", *score).as_str()),
                    botness_score: Some(*score),
                },
            }
        }
        PolicyDecision::BotnessNotABot { score, signal_ids } => {
            let policy_match =
                resolve_policy_match(PolicyTransition::BotnessGateNotABot(signal_ids.clone()));
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
                        outcome: policy_match
                            .annotate_outcome(compact_botness_outcome("served", *score).as_str()),
                    },
                ],
                response: ResponseIntent::NotABot,
            }
        }
        PolicyDecision::BotnessChallenge { score, signal_ids } => {
            let policy_match =
                resolve_policy_match(PolicyTransition::BotnessGateChallenge(signal_ids.clone()));
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
                        outcome: policy_match
                            .annotate_outcome(compact_botness_outcome("served", *score).as_str()),
                    },
                ],
                response: ResponseIntent::Challenge,
            }
        }
        PolicyDecision::BotnessChallengeFallbackMaze { score, signal_ids } => {
            let policy_match = resolve_policy_match(PolicyTransition::ChallengeDisabledFallbackMaze(
                signal_ids.clone(),
            ));
            DecisionPlan {
                intents: vec![EffectIntent::RecordPolicyMatch(
                    PolicyTransition::ChallengeDisabledFallbackMaze(signal_ids.clone()),
                )],
                response: ResponseIntent::Maze {
                    entry_path: crate::maze::entry_path("botness-challenge-fallback"),
                    event_reason: "botness_gate_challenge_disabled_fallback_maze".to_string(),
                    event_outcome: policy_match
                        .annotate_outcome(
                            compact_botness_outcome("fallback_maze", *score).as_str(),
                        ),
                    botness_score: Some(*score),
                },
            }
        }
        PolicyDecision::BotnessChallengeFallbackBlock { score, signal_ids } => {
            let policy_match =
                resolve_policy_match(PolicyTransition::ChallengeDisabledFallbackBlock(
                    signal_ids.clone(),
                ));
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
                            compact_botness_outcome("fallback_block", *score).as_str(),
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
                        outcome: policy_match.annotate_outcome("required"),
                    },
                ],
                response: ResponseIntent::JsChallenge,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verified_identity() -> crate::bot_identity::contracts::VerifiedIdentityEvidence {
        crate::bot_identity::contracts::VerifiedIdentityEvidence {
            scheme: crate::bot_identity::contracts::IdentityScheme::ProviderSignedAgent,
            stable_identity: "chatgpt-agent".to_string(),
            operator: "openai".to_string(),
            category: crate::bot_identity::contracts::IdentityCategory::UserTriggeredAgent,
            verification_strength:
                crate::bot_identity::contracts::VerificationStrength::ProviderAsserted,
            end_user_controlled: true,
            directory_source: None,
            provenance: crate::bot_identity::contracts::IdentityProvenance::Provider,
        }
    }

    fn cfg() -> crate::config::Config {
        crate::config::defaults().clone()
    }

    fn facts() -> crate::runtime::request_facts::RequestFacts {
        crate::runtime::request_facts::build_request_facts(
            &spin_sdk::http::Request::builder()
                .method(spin_sdk::http::Method::Get)
                .uri("/characterization")
                .build(),
            crate::runtime::request_facts::RequestFactInputs {
                site_id: "default".to_string(),
                ip: "203.0.113.9".to_string(),
                user_agent: "ua".to_string(),
                ip_range_evaluation: crate::signals::ip_range_policy::Evaluation::NoMatch,
                honeypot_hit: false,
                rate_limit_exceeded: false,
                existing_ban: false,
                geo_route: crate::signals::geo::GeoPolicyRoute::None,
                geo_country: None,
                needs_js: false,
                botness_score: 0,
                botness_signal_ids: vec![],
                botness_summary: "none".to_string(),
                botness_state_summary: "none".to_string(),
                runtime_metadata_summary: "meta".to_string(),
                provider_summary: "providers".to_string(),
                verified_identity: None,
                not_a_bot_marker_valid: false,
            },
        )
    }

    fn intent_label(intent: &EffectIntent) -> &'static str {
        match intent {
            EffectIntent::RecordPolicyMatch(_) => "record_policy_match",
            EffectIntent::IncrementMetric { .. } => "increment_metric",
            EffectIntent::RecordRateViolation { .. } => "record_rate_violation",
            EffectIntent::RecordGeoViolation { .. } => "record_geo_violation",
            EffectIntent::RecordHoneypotHit { .. } => "record_honeypot_hit",
            EffectIntent::RecordNotABotServed => "record_not_a_bot_served",
            EffectIntent::RecordNotABotSubmit { .. } => "record_not_a_bot_submit",
            EffectIntent::RecordChallengeFailure { .. } => "record_challenge_failure",
            EffectIntent::RecordIpRangeChallengeSolved => "record_ip_range_challenge_solved",
            EffectIntent::RecordBotnessVisibility { .. } => "record_botness_visibility",
            EffectIntent::RecordLikelyHumanSample { .. } => "record_likely_human_sample",
            EffectIntent::RecordVerifiedIdentityTelemetry { .. } => "record_verified_identity_telemetry",
            EffectIntent::RecordRequestOutcome { .. } => "record_request_outcome",
            EffectIntent::RecordShadowAction { .. } => "record_shadow_action",
            EffectIntent::RecordShadowPassThrough => "record_shadow_pass_through",
            EffectIntent::FlushPendingMonitoringCounters => "flush_pending_monitoring_counters",
            EffectIntent::LogEvent { .. } => "log_event",
            EffectIntent::Ban(_) => "ban",
        }
    }

    fn response_label(response: &ResponseIntent) -> &'static str {
        match response {
            ResponseIntent::Continue => "continue",
            ResponseIntent::ForwardAllow { .. } => "forward_allow",
            ResponseIntent::BlockPage { .. } => "block_page",
            ResponseIntent::PlainTextBlock { .. } => "plain_text_block",
            ResponseIntent::DropConnection => "drop_connection",
            ResponseIntent::Redirect { .. } => "redirect",
            ResponseIntent::Maze { .. } => "maze",
            ResponseIntent::Challenge => "challenge",
            ResponseIntent::NotABot => "not_a_bot",
            ResponseIntent::JsChallenge => "js_challenge",
            ResponseIntent::IpRangeTarpit { .. } => "ip_range_tarpit",
        }
    }

    fn render_snapshot_line(
        case_name: &str,
        decision: &crate::runtime::policy_graph::PolicyDecision,
        plan: &DecisionPlan,
    ) -> String {
        let intents = plan
            .intents
            .iter()
            .map(intent_label)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{}|{}|{}|{}",
            case_name,
            decision.label(),
            intents,
            response_label(&plan.response)
        )
    }

    #[test]
    fn ip_range_emergency_allowlist_emits_forward_allow_response_intent() {
        let facts = {
            let mut f = facts();
            f.ip_range_evaluation = crate::signals::ip_range_policy::Evaluation::EmergencyAllowlisted {
                matched_cidr: "203.0.113.0/24".to_string(),
            };
            f
        };
        let mut cfg = cfg();
        cfg.ip_range_policy_mode = crate::config::IpRangePolicyMode::Enforce;
        let decisions = crate::runtime::policy_graph::evaluate_first_tranche(&facts, &cfg);
        assert_eq!(decisions.len(), 1);
        let plan = plan_for_decision(&decisions[0], &facts, &cfg);
        assert!(
            matches!(
                plan.response,
                ResponseIntent::ForwardAllow { ref reason }
                if reason == "ip_range_emergency_allowlist"
            ),
            "expected ForwardAllow response intent, got {:?}",
            response_label(&plan.response)
        );
    }

    #[test]
    fn verified_identity_policy_deny_blocks_with_verified_identity_reason() {
        let mut facts = facts();
        facts.verified_identity = Some(verified_identity());

        let mut cfg = cfg();
        cfg.verified_identity.enabled = true;
        let objectives =
            crate::observability::operator_snapshot_objectives::default_operator_objectives(1);
        let context = crate::runtime::non_human_policy::verified_identity_policy_context(
            &objectives,
            facts.verified_identity.as_ref().expect("verified identity"),
        );

        let resolution = crate::bot_identity::policy::resolve_identity_policy(
            &context,
            &cfg.verified_identity.named_policies,
            &cfg.verified_identity.category_defaults,
            &cfg.verified_identity.service_profiles,
            facts.verified_identity.as_ref().expect("verified identity"),
            facts.path.as_str(),
        );

        let plan = plan_for_decision(
            &crate::runtime::policy_graph::PolicyDecision::VerifiedIdentityPolicyDeny {
                resolution,
            },
            &facts,
            &cfg,
        );

        assert!(matches!(
            plan.response,
            ResponseIntent::BlockPage {
                status: 403,
                reason: crate::enforcement::block_page::BlockReason::VerifiedIdentityPolicy,
            }
        ));
        assert!(plan.intents.iter().any(|intent| matches!(
            intent,
            EffectIntent::RecordPolicyMatch(
                crate::runtime::policy_taxonomy::PolicyTransition::VerifiedIdentityPolicyDeny(_)
            )
        )));

        let outcome = plan
            .intents
            .iter()
            .find_map(|intent| match intent {
                EffectIntent::LogEvent { outcome, .. } => Some(outcome.as_str()),
                _ => None,
            })
            .expect("verified-identity deny log outcome");
        let parsed = crate::runtime::policy_taxonomy::parse_annotated_outcome(outcome);
        assert_eq!(
            parsed.taxonomy.as_ref().and_then(|taxonomy| taxonomy.detection.as_deref()),
            Some("D_VERIFIED_IDENTITY_POLICY_DENY")
        );
        assert!(parsed
            .outcome_text
            .as_deref()
            .is_some_and(|text| text.contains("source=canonical_category_posture")));
    }

    #[test]
    fn verified_identity_policy_allow_short_circuits_with_forward_allow() {
        let mut facts = facts();
        facts.verified_identity = Some(verified_identity());

        let mut cfg = cfg();
        cfg.verified_identity.enabled = true;
        cfg.verified_identity.named_policies = vec![crate::bot_identity::policy::IdentityPolicyEntry {
            policy_id: "structured-openai".to_string(),
            description: None,
            matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                operator: Some("openai".to_string()),
                ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
            },
            action: crate::bot_identity::policy::IdentityPolicyAction::UseServiceProfile(
                "structured_agent".to_string(),
            ),
        }];
        let objectives =
            crate::observability::operator_snapshot_objectives::humans_plus_verified_only_operator_objectives(1);
        let context = crate::runtime::non_human_policy::verified_identity_policy_context(
            &objectives,
            facts.verified_identity.as_ref().expect("verified identity"),
        );

        let resolution = crate::bot_identity::policy::resolve_identity_policy(
            &context,
            &cfg.verified_identity.named_policies,
            &cfg.verified_identity.category_defaults,
            &cfg.verified_identity.service_profiles,
            facts.verified_identity.as_ref().expect("verified identity"),
            facts.path.as_str(),
        );

        let plan = plan_for_decision(
            &crate::runtime::policy_graph::PolicyDecision::VerifiedIdentityPolicyAllow {
                resolution,
            },
            &facts,
            &cfg,
        );

        assert!(matches!(
            plan.response,
            ResponseIntent::ForwardAllow { ref reason }
                if reason == "verified_identity_policy_allow"
        ));
        assert!(plan.intents.iter().any(|intent| matches!(
            intent,
            EffectIntent::RecordPolicyMatch(
                crate::runtime::policy_taxonomy::PolicyTransition::VerifiedIdentityPolicyAllow(_)
            )
        )));
        let outcome = plan
            .intents
            .iter()
            .find_map(|intent| match intent {
                EffectIntent::LogEvent { outcome, .. } => Some(outcome.as_str()),
                _ => None,
            })
            .expect("verified-identity allow log outcome");
        assert!(outcome.contains("service_profile_id=structured_agent"));
    }

    #[test]
    fn verified_identity_policy_observe_and_restrict_continue_with_distinct_taxonomy() {
        let mut facts = facts();
        facts.verified_identity = Some(verified_identity());
        let objectives =
            crate::observability::operator_snapshot_objectives::humans_plus_verified_only_operator_objectives(1);
        let context = crate::runtime::non_human_policy::verified_identity_policy_context(
            &objectives,
            facts.verified_identity.as_ref().expect("verified identity"),
        );

        let observe_resolution = crate::bot_identity::policy::resolve_identity_policy(
            &context,
            &[crate::bot_identity::policy::IdentityPolicyEntry {
                policy_id: "observe-openai".to_string(),
                description: None,
                matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                    operator: Some("openai".to_string()),
                    ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
                },
                action: crate::bot_identity::policy::IdentityPolicyAction::Observe,
            }],
            &[],
            &cfg().verified_identity.service_profiles,
            facts.verified_identity.as_ref().expect("verified identity"),
            facts.path.as_str(),
        );
        let restrict_resolution = crate::bot_identity::policy::resolve_identity_policy(
            &context,
            &[crate::bot_identity::policy::IdentityPolicyEntry {
                policy_id: "restrict-openai".to_string(),
                description: None,
                matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                    operator: Some("openai".to_string()),
                    ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
                },
                action: crate::bot_identity::policy::IdentityPolicyAction::Restrict,
            }],
            &[],
            &cfg().verified_identity.service_profiles,
            facts.verified_identity.as_ref().expect("verified identity"),
            facts.path.as_str(),
        );

        let observe_plan = plan_for_decision(
            &crate::runtime::policy_graph::PolicyDecision::VerifiedIdentityPolicyObserve {
                resolution: observe_resolution,
            },
            &facts,
            &cfg(),
        );
        let restrict_plan = plan_for_decision(
            &crate::runtime::policy_graph::PolicyDecision::VerifiedIdentityPolicyRestrict {
                resolution: restrict_resolution,
            },
            &facts,
            &cfg(),
        );

        assert!(matches!(observe_plan.response, ResponseIntent::Continue));
        assert!(matches!(restrict_plan.response, ResponseIntent::Continue));

        let observe_outcome = observe_plan
            .intents
            .iter()
            .find_map(|intent| match intent {
                EffectIntent::LogEvent { outcome, .. } => Some(outcome.as_str()),
                _ => None,
            })
            .expect("verified-identity observe log outcome");
        let restrict_outcome = restrict_plan
            .intents
            .iter()
            .find_map(|intent| match intent {
                EffectIntent::LogEvent { outcome, .. } => Some(outcome.as_str()),
                _ => None,
            })
            .expect("verified-identity restrict log outcome");
        let observe_taxonomy =
            crate::runtime::policy_taxonomy::parse_annotated_outcome(observe_outcome);
        let restrict_taxonomy =
            crate::runtime::policy_taxonomy::parse_annotated_outcome(restrict_outcome);
        assert_eq!(
            observe_taxonomy
                .taxonomy
                .as_ref()
                .and_then(|taxonomy| taxonomy.detection.as_deref()),
            Some("D_VERIFIED_IDENTITY_POLICY_OBSERVE")
        );
        assert_eq!(
            restrict_taxonomy
                .taxonomy
                .as_ref()
                .and_then(|taxonomy| taxonomy.detection.as_deref()),
            Some("D_VERIFIED_IDENTITY_POLICY_RESTRICT")
        );
    }

    #[test]
    fn characterization_snapshot_captures_plan_parity_for_migrated_seams() {
        struct Case {
            name: &'static str,
            facts: crate::runtime::request_facts::RequestFacts,
            configure: fn(&mut crate::config::Config),
        }

        let cases = [
            Case {
                name: "ip-range-forbidden",
                facts: {
                    let mut f = facts();
                    f.ip_range_evaluation = crate::signals::ip_range_policy::Evaluation::Matched(
                        crate::signals::ip_range_policy::MatchDetails {
                            source: crate::signals::ip_range_policy::MatchSource::CustomRule,
                            source_id: "r1".to_string(),
                            action: crate::config::IpRangePolicyAction::Forbidden403,
                            matched_cidr: "198.51.100.0/24".to_string(),
                            redirect_url: None,
                            custom_message: None,
                        },
                    );
                    f
                },
                configure: |cfg| cfg.ip_range_policy_mode = crate::config::IpRangePolicyMode::Enforce,
            },
            Case {
                name: "honeypot-hit",
                facts: {
                    let mut f = facts();
                    f.honeypot_hit = true;
                    f
                },
                configure: |_| {},
            },
            Case {
                name: "existing-ban",
                facts: {
                    let mut f = facts();
                    f.existing_ban = true;
                    f
                },
                configure: |_| {},
            },
            Case {
                name: "geo-maze",
                facts: {
                    let mut f = facts();
                    f.geo_route = crate::signals::geo::GeoPolicyRoute::Maze;
                    f
                },
                configure: |cfg| {
                    cfg.defence_modes.geo = crate::config::ComposabilityMode::Enforce;
                    cfg.maze_enabled = true;
                    cfg.challenge_puzzle_enabled = true;
                },
            },
            Case {
                name: "botness-challenge",
                facts: {
                    let mut f = facts();
                    f.botness_score = 8;
                    f
                },
                configure: |cfg| {
                    cfg.challenge_puzzle_enabled = true;
                    cfg.challenge_puzzle_risk_threshold = 7;
                    cfg.botness_maze_threshold = 9;
                },
            },
            Case {
                name: "botness-not-a-bot",
                facts: {
                    let mut f = facts();
                    f.botness_score = 4;
                    f.needs_js = true;
                    f
                },
                configure: |cfg| {
                    cfg.not_a_bot_enabled = true;
                    cfg.challenge_puzzle_enabled = true;
                    cfg.not_a_bot_risk_threshold = 3;
                    cfg.challenge_puzzle_risk_threshold = 7;
                },
            },
            Case {
                name: "js-required",
                facts: {
                    let mut f = facts();
                    f.needs_js = true;
                    f
                },
                configure: |cfg| {
                    cfg.defence_modes.js = crate::config::ComposabilityMode::Enforce;
                    cfg.js_required_enforced = true;
                },
            },
            Case {
                name: "verified-identity-deny",
                facts: {
                    let mut f = facts();
                    f.verified_identity = Some(verified_identity());
                    f
                },
                configure: |cfg| {
                    cfg.verified_identity.enabled = true;
                },
            },
            Case {
                name: "verified-identity-allow",
                facts: {
                    let mut f = facts();
                    f.verified_identity = Some(verified_identity());
                    f
                },
                configure: |cfg| {
                    cfg.verified_identity.enabled = true;
                    cfg.verified_identity.named_policies =
                        vec![crate::bot_identity::policy::IdentityPolicyEntry {
                            policy_id: "allow-openai".to_string(),
                            description: None,
                            matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                                operator: Some("openai".to_string()),
                                ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
                            },
                            action: crate::bot_identity::policy::IdentityPolicyAction::Allow,
                        }];
                },
            },
            Case {
                name: "verified-identity-observe",
                facts: {
                    let mut f = facts();
                    f.path = "/observe/thing".to_string();
                    f.verified_identity = Some(verified_identity());
                    f
                },
                configure: |cfg| {
                    cfg.verified_identity.enabled = true;
                    cfg.verified_identity.named_policies =
                        vec![crate::bot_identity::policy::IdentityPolicyEntry {
                            policy_id: "observe-openai".to_string(),
                            description: None,
                            matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                                operator: Some("openai".to_string()),
                                path_prefixes: vec!["/observe".to_string()],
                                ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
                            },
                            action: crate::bot_identity::policy::IdentityPolicyAction::Observe,
                        }];
                },
            },
            Case {
                name: "verified-identity-restrict",
                facts: {
                    let mut f = facts();
                    f.path = "/restrict/thing".to_string();
                    f.verified_identity = Some(verified_identity());
                    f
                },
                configure: |cfg| {
                    cfg.verified_identity.enabled = true;
                    cfg.verified_identity.named_policies =
                        vec![crate::bot_identity::policy::IdentityPolicyEntry {
                            policy_id: "restrict-openai".to_string(),
                            description: None,
                            matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                                operator: Some("openai".to_string()),
                                path_prefixes: vec!["/restrict".to_string()],
                                ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
                            },
                            action: crate::bot_identity::policy::IdentityPolicyAction::Restrict,
                        }];
                },
            },
        ];

        let mut lines = Vec::new();
        for case in cases {
            let mut cfg = cfg();
            (case.configure)(&mut cfg);
            let objectives = if case.name.contains("observe") || case.name.contains("restrict") {
                crate::observability::operator_snapshot_objectives::humans_plus_verified_only_operator_objectives(1)
            } else {
                crate::observability::operator_snapshot_objectives::default_operator_objectives(1)
            };
            let mut decisions = crate::runtime::policy_graph::evaluate_first_tranche(&case.facts, &cfg);
            if decisions.is_empty() {
                decisions = crate::runtime::policy_graph::evaluate_verified_identity_tranche(
                    &case.facts,
                    &cfg,
                    &objectives,
                );
            }
            if decisions.is_empty() {
                decisions = crate::runtime::policy_graph::evaluate_second_tranche(
                    &case.facts,
                    &cfg,
                );
            }
            for decision in decisions {
                let plan = plan_for_decision(&decision, &case.facts, &cfg);
                lines.push(render_snapshot_line(case.name, &decision, &plan));
            }
        }

        let observed = lines.join("\n");
        let expected = include_str!("plan_builder_characterization_snapshot.txt").trim();
        assert_eq!(observed, expected);
    }

    #[test]
    fn botness_challenge_plan_avoids_verbose_blended_outcome_strings() {
        let mut facts = facts();
        facts.botness_score = 8;
        facts.botness_summary = "js_required,rate_high".to_string();
        facts.botness_state_summary = "js_required:active,rate_high:active".to_string();
        facts.provider_summary = "challenge_engine:internal".to_string();

        let decision = crate::runtime::policy_graph::PolicyDecision::BotnessChallenge {
            score: 8,
            signal_ids: vec![crate::runtime::policy_taxonomy::SignalId::GeoRisk],
        };

        let mut cfg = cfg();
        cfg.challenge_puzzle_enabled = true;
        cfg.challenge_puzzle_risk_threshold = 7;
        cfg.botness_maze_threshold = 9;

        let plan = plan_for_decision(&decision, &facts, &cfg);
        let outcome = plan
            .intents
            .iter()
            .find_map(|intent| match intent {
                EffectIntent::LogEvent { outcome, .. } => Some(outcome.as_str()),
                _ => None,
            })
            .expect("expected botness challenge log intent");

        assert!(!outcome.contains("score=8 signals="));
        assert!(!outcome.contains("signal_states="));
        assert!(!outcome.contains("providers="));
    }
}
