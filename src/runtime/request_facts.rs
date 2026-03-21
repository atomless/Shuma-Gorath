use spin_sdk::http::{Method, Request};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestFacts {
    pub method: Method,
    pub path: String,
    pub site_id: String,
    pub ip: String,
    pub user_agent: String,
    pub ip_range_evaluation: crate::signals::ip_range_policy::Evaluation,
    pub honeypot_hit: bool,
    pub rate_limit_exceeded: bool,
    pub existing_ban: bool,
    pub geo_route: crate::signals::geo::GeoPolicyRoute,
    pub geo_country: Option<String>,
    pub needs_js: bool,
    pub botness_score: u8,
    pub botness_signal_ids: Vec<crate::runtime::policy_taxonomy::SignalId>,
    pub botness_summary: String,
    pub botness_state_summary: String,
    pub runtime_metadata_summary: String,
    pub provider_summary: String,
    pub verified_identity: Option<crate::bot_identity::contracts::VerifiedIdentityEvidence>,
    pub not_a_bot_marker_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestFactInputs {
    pub site_id: String,
    pub ip: String,
    pub user_agent: String,
    pub ip_range_evaluation: crate::signals::ip_range_policy::Evaluation,
    pub honeypot_hit: bool,
    pub rate_limit_exceeded: bool,
    pub existing_ban: bool,
    pub geo_route: crate::signals::geo::GeoPolicyRoute,
    pub geo_country: Option<String>,
    pub needs_js: bool,
    pub botness_score: u8,
    pub botness_signal_ids: Vec<crate::runtime::policy_taxonomy::SignalId>,
    pub botness_summary: String,
    pub botness_state_summary: String,
    pub runtime_metadata_summary: String,
    pub provider_summary: String,
    pub verified_identity: Option<crate::bot_identity::contracts::VerifiedIdentityEvidence>,
    pub not_a_bot_marker_valid: bool,
}

pub(crate) fn build_request_facts(req: &Request, inputs: RequestFactInputs) -> RequestFacts {
    RequestFacts {
        method: req.method().clone(),
        path: req.path().to_string(),
        site_id: inputs.site_id,
        ip: inputs.ip,
        user_agent: inputs.user_agent,
        ip_range_evaluation: inputs.ip_range_evaluation,
        honeypot_hit: inputs.honeypot_hit,
        rate_limit_exceeded: inputs.rate_limit_exceeded,
        existing_ban: inputs.existing_ban,
        geo_route: inputs.geo_route,
        geo_country: inputs.geo_country,
        needs_js: inputs.needs_js,
        botness_score: inputs.botness_score,
        botness_signal_ids: inputs.botness_signal_ids,
        botness_summary: inputs.botness_summary,
        botness_state_summary: inputs.botness_state_summary,
        runtime_metadata_summary: inputs.runtime_metadata_summary,
        provider_summary: inputs.provider_summary,
        verified_identity: inputs.verified_identity,
        not_a_bot_marker_valid: inputs.not_a_bot_marker_valid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_is_side_effect_free_projection_of_inputs() {
        let req = Request::builder().method(Method::Post).uri("/x").build();
        let facts = build_request_facts(
            &req,
            RequestFactInputs {
                site_id: "default".to_string(),
                ip: "203.0.113.10".to_string(),
                user_agent: "ua".to_string(),
                ip_range_evaluation: crate::signals::ip_range_policy::Evaluation::NoMatch,
                honeypot_hit: true,
                rate_limit_exceeded: false,
                existing_ban: false,
                geo_route: crate::signals::geo::GeoPolicyRoute::Challenge,
                geo_country: Some("US".to_string()),
                needs_js: true,
                botness_score: 7,
                botness_signal_ids: vec![crate::runtime::policy_taxonomy::SignalId::GeoRisk],
                botness_summary: "signals".to_string(),
                botness_state_summary: "states".to_string(),
                runtime_metadata_summary: "modes".to_string(),
                provider_summary: "providers".to_string(),
                verified_identity: Some(crate::bot_identity::contracts::VerifiedIdentityEvidence {
                    scheme: crate::bot_identity::contracts::IdentityScheme::ProviderSignedAgent,
                    stable_identity: "chatgpt-agent".to_string(),
                    operator: "openai".to_string(),
                    category: crate::bot_identity::contracts::IdentityCategory::UserTriggeredAgent,
                    verification_strength:
                        crate::bot_identity::contracts::VerificationStrength::ProviderAsserted,
                    end_user_controlled: true,
                    directory_source: None,
                    provenance: crate::bot_identity::contracts::IdentityProvenance::Provider,
                }),
                not_a_bot_marker_valid: false,
            },
        );

        assert_eq!(facts.method, Method::Post);
        assert_eq!(facts.path, "/x");
        assert!(facts.honeypot_hit);
        assert!(facts.needs_js);
        assert_eq!(facts.botness_score, 7);
        assert_eq!(
            facts.verified_identity.as_ref().map(|identity| identity.stable_identity.as_str()),
            Some("chatgpt-agent")
        );
    }
}
