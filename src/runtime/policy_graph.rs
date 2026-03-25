use crate::runtime::policy_taxonomy::SignalId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PolicyDecision {
    IpRangeEmergencyAllowlisted { matched_cidr: String },
    IpRangeAdvisory {
        details: crate::signals::ip_range_policy::MatchDetails,
    },
    IpRangeForbidden {
        details: crate::signals::ip_range_policy::MatchDetails,
    },
    IpRangeCustomMessage {
        details: crate::signals::ip_range_policy::MatchDetails,
        message: String,
    },
    IpRangeDropConnection {
        details: crate::signals::ip_range_policy::MatchDetails,
    },
    IpRangeRedirect {
        details: crate::signals::ip_range_policy::MatchDetails,
        location: Option<String>,
    },
    IpRangeRateLimit {
        details: crate::signals::ip_range_policy::MatchDetails,
    },
    IpRangeHoneypot {
        details: crate::signals::ip_range_policy::MatchDetails,
    },
    IpRangeMaze {
        details: crate::signals::ip_range_policy::MatchDetails,
    },
    IpRangeTarpit {
        details: crate::signals::ip_range_policy::MatchDetails,
    },
    HoneypotHit,
    RateLimitHit,
    ExistingBan,
    VerifiedIdentityPolicyDeny {
        resolution: crate::bot_identity::policy::IdentityPolicyResolution,
    },
    VerifiedIdentityPolicyAllow {
        resolution: crate::bot_identity::policy::IdentityPolicyResolution,
    },
    VerifiedIdentityPolicyObserve {
        resolution: crate::bot_identity::policy::IdentityPolicyResolution,
    },
    VerifiedIdentityPolicyRestrict {
        resolution: crate::bot_identity::policy::IdentityPolicyResolution,
    },
    GeoBlock,
    GeoMaze,
    GeoMazeFallbackChallenge,
    GeoChallenge,
    GeoChallengeFallbackMaze,
    GeoFallbackBlockFromMaze,
    GeoFallbackBlockFromChallenge,
    BotnessMaze { score: u8, signal_ids: Vec<SignalId> },
    BotnessNotABot { score: u8, signal_ids: Vec<SignalId> },
    BotnessChallenge { score: u8, signal_ids: Vec<SignalId> },
    BotnessChallengeFallbackMaze { score: u8, signal_ids: Vec<SignalId> },
    BotnessChallengeFallbackBlock { score: u8, signal_ids: Vec<SignalId> },
    JsChallengeRequired,
}

impl PolicyDecision {
    #[cfg(test)]
    pub(crate) fn label(&self) -> &'static str {
        match self {
            PolicyDecision::IpRangeEmergencyAllowlisted { .. } => "ip_range_emergency_allowlisted",
            PolicyDecision::IpRangeAdvisory { .. } => "ip_range_advisory",
            PolicyDecision::IpRangeForbidden { .. } => "ip_range_forbidden",
            PolicyDecision::IpRangeCustomMessage { .. } => "ip_range_custom_message",
            PolicyDecision::IpRangeDropConnection { .. } => "ip_range_drop_connection",
            PolicyDecision::IpRangeRedirect { .. } => "ip_range_redirect",
            PolicyDecision::IpRangeRateLimit { .. } => "ip_range_rate_limit",
            PolicyDecision::IpRangeHoneypot { .. } => "ip_range_honeypot",
            PolicyDecision::IpRangeMaze { .. } => "ip_range_maze",
            PolicyDecision::IpRangeTarpit { .. } => "ip_range_tarpit",
            PolicyDecision::HoneypotHit => "honeypot_hit",
            PolicyDecision::RateLimitHit => "rate_limit_hit",
            PolicyDecision::ExistingBan => "existing_ban",
            PolicyDecision::VerifiedIdentityPolicyDeny { .. } => "verified_identity_policy_deny",
            PolicyDecision::VerifiedIdentityPolicyAllow { .. } => {
                "verified_identity_policy_allow"
            }
            PolicyDecision::VerifiedIdentityPolicyObserve { .. } => {
                "verified_identity_policy_observe"
            }
            PolicyDecision::VerifiedIdentityPolicyRestrict { .. } => {
                "verified_identity_policy_restrict"
            }
            PolicyDecision::GeoBlock => "geo_block",
            PolicyDecision::GeoMaze => "geo_maze",
            PolicyDecision::GeoMazeFallbackChallenge => "geo_maze_fallback_challenge",
            PolicyDecision::GeoChallenge => "geo_challenge",
            PolicyDecision::GeoChallengeFallbackMaze => "geo_challenge_fallback_maze",
            PolicyDecision::GeoFallbackBlockFromMaze => "geo_fallback_block_from_maze",
            PolicyDecision::GeoFallbackBlockFromChallenge => "geo_fallback_block_from_challenge",
            PolicyDecision::BotnessMaze { .. } => "botness_maze",
            PolicyDecision::BotnessNotABot { .. } => "botness_not_a_bot",
            PolicyDecision::BotnessChallenge { .. } => "botness_challenge",
            PolicyDecision::BotnessChallengeFallbackMaze { .. } => {
                "botness_challenge_fallback_maze"
            }
            PolicyDecision::BotnessChallengeFallbackBlock { .. } => {
                "botness_challenge_fallback_block"
            }
            PolicyDecision::JsChallengeRequired => "js_challenge_required",
        }
    }

    pub(crate) fn is_terminal(&self) -> bool {
        !matches!(
            self,
            PolicyDecision::IpRangeAdvisory { .. }
                | PolicyDecision::VerifiedIdentityPolicyObserve { .. }
                | PolicyDecision::VerifiedIdentityPolicyRestrict { .. }
        )
    }
}

fn decide_ip_range(
    facts: &crate::runtime::request_facts::RequestFacts,
    cfg: &crate::config::Config,
) -> Option<PolicyDecision> {
    match &facts.ip_range_evaluation {
        crate::signals::ip_range_policy::Evaluation::NoMatch => None,
        crate::signals::ip_range_policy::Evaluation::EmergencyAllowlisted { matched_cidr } => {
            Some(PolicyDecision::IpRangeEmergencyAllowlisted {
                matched_cidr: matched_cidr.clone(),
            })
        }
        crate::signals::ip_range_policy::Evaluation::Matched(details) => {
            if cfg.ip_range_policy_mode == crate::config::IpRangePolicyMode::Advisory {
                return Some(PolicyDecision::IpRangeAdvisory {
                    details: details.clone(),
                });
            }

            Some(match details.action {
                crate::config::IpRangePolicyAction::Forbidden403 => {
                    PolicyDecision::IpRangeForbidden {
                        details: details.clone(),
                    }
                }
                crate::config::IpRangePolicyAction::CustomMessage => {
                    PolicyDecision::IpRangeCustomMessage {
                        details: details.clone(),
                        message: details.custom_message.clone().unwrap_or_else(|| {
                            "Access blocked by IP range policy.".to_string()
                        }),
                    }
                }
                crate::config::IpRangePolicyAction::DropConnection => {
                    PolicyDecision::IpRangeDropConnection {
                        details: details.clone(),
                    }
                }
                crate::config::IpRangePolicyAction::Redirect308 => PolicyDecision::IpRangeRedirect {
                    details: details.clone(),
                    location: details.redirect_url.clone(),
                },
                crate::config::IpRangePolicyAction::RateLimit => PolicyDecision::IpRangeRateLimit {
                    details: details.clone(),
                },
                crate::config::IpRangePolicyAction::Honeypot => PolicyDecision::IpRangeHoneypot {
                    details: details.clone(),
                },
                crate::config::IpRangePolicyAction::Maze => PolicyDecision::IpRangeMaze {
                    details: details.clone(),
                },
                crate::config::IpRangePolicyAction::Tarpit => PolicyDecision::IpRangeTarpit {
                    details: details.clone(),
                },
            })
        }
    }
}

fn decide_geo(
    facts: &crate::runtime::request_facts::RequestFacts,
    cfg: &crate::config::Config,
) -> Option<PolicyDecision> {
    if !cfg.geo_action_enabled() {
        return None;
    }

    match facts.geo_route {
        crate::signals::geo::GeoPolicyRoute::Block => Some(PolicyDecision::GeoBlock),
        crate::signals::geo::GeoPolicyRoute::Maze => {
            if cfg.maze_enabled {
                Some(PolicyDecision::GeoMaze)
            } else if cfg.challenge_puzzle_enabled {
                Some(PolicyDecision::GeoMazeFallbackChallenge)
            } else {
                Some(PolicyDecision::GeoFallbackBlockFromMaze)
            }
        }
        crate::signals::geo::GeoPolicyRoute::Challenge => {
            if cfg.challenge_puzzle_enabled {
                Some(PolicyDecision::GeoChallenge)
            } else if cfg.maze_enabled {
                Some(PolicyDecision::GeoChallengeFallbackMaze)
            } else {
                Some(PolicyDecision::GeoFallbackBlockFromChallenge)
            }
        }
        crate::signals::geo::GeoPolicyRoute::Allow | crate::signals::geo::GeoPolicyRoute::None => {
            None
        }
    }
}

fn decide_verified_identity_policy(
    facts: &crate::runtime::request_facts::RequestFacts,
    cfg: &crate::config::Config,
) -> Option<PolicyDecision> {
    if !cfg.verified_identity.enabled {
        return None;
    }

    let identity = facts.verified_identity.as_ref()?;
    let resolution = crate::bot_identity::policy::resolve_identity_policy(
        &cfg.verified_identity.named_policies,
        &cfg.verified_identity.category_defaults,
        &cfg.verified_identity.service_profiles,
        identity,
        facts.path.as_str(),
    );

    Some(match resolution.outcome {
        crate::bot_identity::policy::IdentityPolicyOutcome::Deny
        | crate::bot_identity::policy::IdentityPolicyOutcome::UseServiceProfile(
            crate::bot_identity::policy::ServiceProfile::Denied,
        ) => PolicyDecision::VerifiedIdentityPolicyDeny { resolution },
        crate::bot_identity::policy::IdentityPolicyOutcome::Allow
        | crate::bot_identity::policy::IdentityPolicyOutcome::UseServiceProfile(_) => {
            PolicyDecision::VerifiedIdentityPolicyAllow { resolution }
        }
        crate::bot_identity::policy::IdentityPolicyOutcome::Observe => {
            PolicyDecision::VerifiedIdentityPolicyObserve { resolution }
        }
        crate::bot_identity::policy::IdentityPolicyOutcome::Restrict => {
            PolicyDecision::VerifiedIdentityPolicyRestrict { resolution }
        }
        crate::bot_identity::policy::IdentityPolicyOutcome::NoMatch => return None,
    })
}

fn decide_botness(
    facts: &crate::runtime::request_facts::RequestFacts,
    cfg: &crate::config::Config,
) -> Option<PolicyDecision> {
    let score = facts.botness_score;
    let signal_ids = facts.botness_signal_ids.clone();

    if cfg.maze_enabled && score >= cfg.botness_maze_threshold {
        return Some(PolicyDecision::BotnessMaze { score, signal_ids });
    }

    let not_a_bot_threshold = cfg.not_a_bot_risk_threshold;
    if cfg.not_a_bot_enabled
        && cfg.challenge_puzzle_enabled
        && not_a_bot_threshold > 0
        && score >= not_a_bot_threshold
        && score < cfg.challenge_puzzle_risk_threshold
        && !facts.not_a_bot_marker_valid
    {
        return Some(PolicyDecision::BotnessNotABot { score, signal_ids });
    }

    if score >= cfg.challenge_puzzle_risk_threshold {
        if cfg.challenge_puzzle_enabled {
            return Some(PolicyDecision::BotnessChallenge { score, signal_ids });
        }
        if cfg.maze_enabled {
            return Some(PolicyDecision::BotnessChallengeFallbackMaze { score, signal_ids });
        }
        return Some(PolicyDecision::BotnessChallengeFallbackBlock { score, signal_ids });
    }

    None
}

fn decide_js(
    facts: &crate::runtime::request_facts::RequestFacts,
    cfg: &crate::config::Config,
) -> Option<PolicyDecision> {
    if cfg.js_action_enabled() && facts.needs_js {
        return Some(PolicyDecision::JsChallengeRequired);
    }
    None
}

/// Evaluate the first policy tranche.
pub(crate) fn evaluate_first_tranche(
    facts: &crate::runtime::request_facts::RequestFacts,
    cfg: &crate::config::Config,
) -> Vec<PolicyDecision> {
    let mut decisions = Vec::new();

    if let Some(ip_range) = decide_ip_range(facts, cfg) {
        let terminal = ip_range.is_terminal();
        decisions.push(ip_range);
        if terminal {
            return decisions;
        }
    }

    if facts.honeypot_hit {
        decisions.push(PolicyDecision::HoneypotHit);
        return decisions;
    }

    if facts.rate_limit_exceeded {
        decisions.push(PolicyDecision::RateLimitHit);
        return decisions;
    }

    if facts.existing_ban {
        decisions.push(PolicyDecision::ExistingBan);
        return decisions;
    }

    decisions
}

/// Evaluate the second policy tranche.
pub(crate) fn evaluate_second_tranche(
    facts: &crate::runtime::request_facts::RequestFacts,
    cfg: &crate::config::Config,
) -> Vec<PolicyDecision> {
    let mut decisions = Vec::new();
    if let Some(geo) = decide_geo(facts, cfg) {
        decisions.push(geo);
        return decisions;
    }
    if let Some(botness) = decide_botness(facts, cfg) {
        decisions.push(botness);
        return decisions;
    }
    if let Some(js) = decide_js(facts, cfg) {
        decisions.push(js);
    }
    decisions
}

/// Evaluate the verified-identity policy tranche that sits between the
/// first coarse controls and the later geo/botness/JS tranche.
pub(crate) fn evaluate_verified_identity_tranche(
    facts: &crate::runtime::request_facts::RequestFacts,
    cfg: &crate::config::Config,
) -> Vec<PolicyDecision> {
    let mut decisions = Vec::new();
    if let Some(verified_identity_policy) = decide_verified_identity_policy(facts, cfg) {
        decisions.push(verified_identity_policy);
    }
    decisions
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
                .uri("/x")
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

    #[test]
    fn pre_tranche_respects_terminal_stage_ordering() {
        let mut request_facts = facts();
        request_facts.honeypot_hit = true;
        request_facts.rate_limit_exceeded = true;

        let decisions = evaluate_first_tranche(&request_facts, &cfg());
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].label(), "honeypot_hit");
    }

    #[test]
    fn advisory_is_non_terminal_and_preserves_follow_on_stage() {
        let mut request_facts = facts();
        request_facts.honeypot_hit = true;
        request_facts.ip_range_evaluation =
            crate::signals::ip_range_policy::Evaluation::Matched(
                crate::signals::ip_range_policy::MatchDetails {
                    source: crate::signals::ip_range_policy::MatchSource::CustomRule,
                    source_id: "a".to_string(),
                    action: crate::config::IpRangePolicyAction::RateLimit,
                    matched_cidr: "203.0.113.0/24".to_string(),
                    redirect_url: None,
                    custom_message: None,
                },
            );

        let mut cfg = cfg();
        cfg.ip_range_policy_mode = crate::config::IpRangePolicyMode::Advisory;

        let decisions = evaluate_first_tranche(&request_facts, &cfg);
        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].label(), "ip_range_advisory");
        assert_eq!(decisions[1].label(), "honeypot_hit");
    }

    #[test]
    fn post_tranche_prefers_not_a_bot_before_js_when_in_band() {
        let mut request_facts = facts();
        request_facts.botness_score = 4;
        request_facts.needs_js = true;

        let mut cfg = cfg();
        cfg.not_a_bot_enabled = true;
        cfg.challenge_puzzle_enabled = true;
        cfg.not_a_bot_risk_threshold = 3;
        cfg.challenge_puzzle_risk_threshold = 7;

        let decisions = evaluate_second_tranche(&request_facts, &cfg);
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].label(), "botness_not_a_bot");
    }

    #[test]
    fn verified_identity_policy_can_block_before_second_tranche_botness() {
        let mut request_facts = facts();
        request_facts.verified_identity = Some(verified_identity());
        request_facts.botness_score = 8;

        let mut cfg = cfg();
        cfg.verified_identity.enabled = true;
        cfg.challenge_puzzle_enabled = true;
        cfg.challenge_puzzle_risk_threshold = 7;
        cfg.botness_maze_threshold = 9;

        assert!(evaluate_first_tranche(&request_facts, &cfg).is_empty());
        assert_eq!(
            evaluate_verified_identity_tranche(&request_facts, &cfg)
                .into_iter()
                .map(|decision| decision.label())
                .collect::<Vec<_>>(),
            vec!["verified_identity_policy_deny"]
        );
        assert_eq!(
            evaluate_second_tranche(&request_facts, &cfg)
                .into_iter()
                .map(|decision| decision.label())
                .collect::<Vec<_>>(),
            vec!["botness_challenge"]
        );
    }

    #[test]
    fn verified_identity_policy_named_allow_short_circuits_before_later_tranches() {
        let mut request_facts = facts();
        request_facts.verified_identity = Some(verified_identity());
        request_facts.botness_score = 8;

        let mut cfg = cfg();
        cfg.verified_identity.enabled = true;
        cfg.verified_identity.named_policies = vec![crate::bot_identity::policy::IdentityPolicyEntry {
            policy_id: "allow-openai".to_string(),
            description: None,
            matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                operator: Some("openai".to_string()),
                ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
            },
            action: crate::bot_identity::policy::IdentityPolicyAction::Allow,
        }];
        cfg.challenge_puzzle_enabled = true;
        cfg.challenge_puzzle_risk_threshold = 7;
        cfg.botness_maze_threshold = 9;

        assert_eq!(
            evaluate_verified_identity_tranche(&request_facts, &cfg)
                .into_iter()
                .map(|decision| decision.label())
                .collect::<Vec<_>>(),
            vec!["verified_identity_policy_allow"]
        );
        assert_eq!(
            evaluate_second_tranche(&request_facts, &cfg)
                .into_iter()
                .map(|decision| decision.label())
                .collect::<Vec<_>>(),
            vec!["botness_challenge"]
        );
    }

    #[test]
    fn verified_identity_policy_preserves_observe_and_restrict_stage_outcomes() {
        let mut observe_facts = facts();
        observe_facts.verified_identity = Some(verified_identity());
        observe_facts.path = "/observe/thing".to_string();

        let mut restrict_facts = facts();
        restrict_facts.verified_identity = Some(verified_identity());
        restrict_facts.path = "/restrict/thing".to_string();

        let mut cfg = cfg();
        cfg.verified_identity.enabled = true;
        cfg.verified_identity.named_policies = vec![
            crate::bot_identity::policy::IdentityPolicyEntry {
                policy_id: "observe-openai".to_string(),
                description: None,
                matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                    operator: Some("openai".to_string()),
                    path_prefixes: vec!["/observe".to_string()],
                    ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
                },
                action: crate::bot_identity::policy::IdentityPolicyAction::Observe,
            },
            crate::bot_identity::policy::IdentityPolicyEntry {
                policy_id: "restrict-openai".to_string(),
                description: None,
                matcher: crate::bot_identity::policy::IdentityPolicyMatcher {
                    operator: Some("openai".to_string()),
                    path_prefixes: vec!["/restrict".to_string()],
                    ..crate::bot_identity::policy::IdentityPolicyMatcher::default()
                },
                action: crate::bot_identity::policy::IdentityPolicyAction::Restrict,
            },
        ];

        assert_eq!(
            evaluate_verified_identity_tranche(&observe_facts, &cfg)
                .into_iter()
                .map(|decision| decision.label())
                .collect::<Vec<_>>(),
            vec!["verified_identity_policy_observe"]
        );
        assert_eq!(
            evaluate_verified_identity_tranche(&restrict_facts, &cfg)
                .into_iter()
                .map(|decision| decision.label())
                .collect::<Vec<_>>(),
            vec!["verified_identity_policy_restrict"]
        );
    }

    #[test]
    fn characterization_matrix_captures_expected_policy_outcomes() {
        struct Case {
            name: &'static str,
            facts: crate::runtime::request_facts::RequestFacts,
            configure: fn(&mut crate::config::Config),
            expected: &'static [&'static str],
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
                expected: &["ip_range_forbidden"],
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
                expected: &["geo_maze"],
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
                expected: &["botness_challenge"],
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
                expected: &["js_challenge_required"],
            },
        ];

        for case in cases {
            let mut cfg = cfg();
            (case.configure)(&mut cfg);
            let mut labels = evaluate_first_tranche(&case.facts, &cfg)
                .into_iter()
                .map(|d| d.label())
                .collect::<Vec<_>>();
            if labels.is_empty() {
                labels.extend(
                    evaluate_second_tranche(&case.facts, &cfg)
                        .into_iter()
                        .map(|d| d.label()),
                );
            }
            assert_eq!(labels, case.expected, "case={}", case.name);
        }
    }
}
