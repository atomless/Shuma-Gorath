#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};
use spin_sdk::http::Response;

use crate::observability::hot_read_contract::{TelemetryBasis, TelemetryExactness};
use crate::runtime::effect_intents::{ExecutionMode, ShadowAction};
use crate::runtime::traffic_classification::{
    classify_current_runtime_branch, non_human_category_assignment_for_lane,
    CurrentRuntimeBranch, MeasurementScope, NonHumanCategoryAssignment, PolicySource,
    RouteActionFamily, TrafficLane, TrafficLaneAssignment,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrafficOrigin {
    Live,
    AdversarySim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequestOutcomeClass {
    Forwarded,
    ShortCircuited,
    ControlResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResponseKind {
    ForwardAllow,
    ForwardFailureFallback,
    SyntheticShadowAllow,
    SyntheticShadowAction,
    BlockPage,
    PlainTextBlock,
    Redirect,
    DropConnection,
    Challenge,
    NotABot,
    JsChallenge,
    Maze,
    Tarpit,
    CheckpointResponse,
    DefenceFollowupResponse,
    SimPublicResponse,
    ControlPlaneResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RequestOutcomeLane {
    pub lane: TrafficLane,
    pub exactness: TelemetryExactness,
    pub basis: TelemetryBasis,
}

impl From<TrafficLaneAssignment> for RequestOutcomeLane {
    fn from(value: TrafficLaneAssignment) -> Self {
        Self {
            lane: value.lane,
            exactness: value.exactness,
            basis: value.basis,
        }
    }
}

pub(crate) struct RenderedResponseEvidence {
    pub response: Response,
    pub response_kind: ResponseKind,
    pub forward_attempted: bool,
    pub forward_latency_ms: Option<u64>,
    pub forward_failure_class: Option<&'static str>,
    pub intended_action: Option<ShadowAction>,
}

impl RenderedResponseEvidence {
    pub(crate) fn local(response: Response, response_kind: ResponseKind) -> Self {
        Self {
            response,
            response_kind,
            forward_attempted: false,
            forward_latency_ms: None,
            forward_failure_class: None,
            intended_action: None,
        }
    }

    pub(crate) fn forwarded(
        response: Response,
        failure_class: Option<&'static str>,
        intended_action: Option<ShadowAction>,
    ) -> Self {
        Self {
            response,
            response_kind: if failure_class.is_some() {
                ResponseKind::ForwardFailureFallback
            } else {
                ResponseKind::ForwardAllow
            },
            forward_attempted: true,
            forward_latency_ms: None,
            forward_failure_class: failure_class,
            intended_action,
        }
    }

    pub(crate) fn synthetic_shadow_allow(response: Response) -> Self {
        Self {
            response,
            response_kind: ResponseKind::SyntheticShadowAllow,
            forward_attempted: false,
            forward_latency_ms: None,
            forward_failure_class: None,
            intended_action: None,
        }
    }

    pub(crate) fn synthetic_shadow_action(response: Response, intended_action: ShadowAction) -> Self {
        Self {
            response,
            response_kind: ResponseKind::SyntheticShadowAction,
            forward_attempted: false,
            forward_latency_ms: None,
            forward_failure_class: None,
            intended_action: Some(intended_action),
        }
    }
}

pub(crate) struct HandledRequestResponse {
    pub branch: CurrentRuntimeBranch,
    pub execution_mode: ExecutionMode,
    pub rendered: RenderedResponseEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RenderedRequestOutcome {
    pub traffic_origin: TrafficOrigin,
    pub measurement_scope: MeasurementScope,
    pub route_action_family: RouteActionFamily,
    pub execution_mode: ExecutionMode,
    pub traffic_lane: Option<RequestOutcomeLane>,
    pub non_human_category: Option<NonHumanCategoryAssignment>,
    pub outcome_class: RequestOutcomeClass,
    pub response_kind: ResponseKind,
    pub http_status: u16,
    pub response_bytes: u64,
    pub forwarded_upstream_latency_ms: Option<u64>,
    pub forward_attempted: bool,
    pub forward_failure_class: Option<&'static str>,
    pub intended_action: Option<ShadowAction>,
    pub policy_source: PolicySource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ObservedRequestOutcomeSummary {
    pub traffic_origin: String,
    pub measurement_scope: String,
    pub route_action_family: String,
    pub execution_mode: String,
    pub outcome_class: String,
    pub response_kind: String,
    pub policy_source: String,
    pub http_status: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traffic_lane: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lane_exactness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lane_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_human_category_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_human_assignment_status: Option<String>,
}

fn normalize_traffic_origin(origin: TrafficOrigin) -> &'static str {
    match origin {
        TrafficOrigin::Live => "live",
        TrafficOrigin::AdversarySim => "adversary_sim",
    }
}

fn normalize_measurement_scope(scope: MeasurementScope) -> &'static str {
    match scope {
        MeasurementScope::IngressPrimary => "ingress_primary",
        MeasurementScope::DefenceFollowup => "defence_followup",
        MeasurementScope::BypassAndControl => "bypass_and_control",
        MeasurementScope::Excluded => "excluded",
    }
}

fn normalize_route_action_family(family: RouteActionFamily) -> &'static str {
    match family {
        RouteActionFamily::PublicContent => "public_content",
        RouteActionFamily::StaticAsset => "static_asset",
        RouteActionFamily::DefenceFollowup => "defence_followup",
        RouteActionFamily::AllowlistBypass => "allowlist_bypass",
        RouteActionFamily::ControlPlane => "control_plane",
        RouteActionFamily::SimPublic => "sim_public",
    }
}

fn normalize_execution_mode(mode: ExecutionMode) -> &'static str {
    match mode {
        ExecutionMode::Enforced => "enforced",
        ExecutionMode::Shadow => "shadow",
    }
}

fn normalize_request_outcome_class(outcome_class: RequestOutcomeClass) -> &'static str {
    match outcome_class {
        RequestOutcomeClass::Forwarded => "forwarded",
        RequestOutcomeClass::ShortCircuited => "short_circuited",
        RequestOutcomeClass::ControlResponse => "control_response",
    }
}

fn normalize_response_kind(kind: ResponseKind) -> &'static str {
    match kind {
        ResponseKind::ForwardAllow => "forward_allow",
        ResponseKind::ForwardFailureFallback => "forward_failure_fallback",
        ResponseKind::SyntheticShadowAllow => "synthetic_shadow_allow",
        ResponseKind::SyntheticShadowAction => "synthetic_shadow_action",
        ResponseKind::BlockPage => "block_page",
        ResponseKind::PlainTextBlock => "plain_text_block",
        ResponseKind::Redirect => "redirect",
        ResponseKind::DropConnection => "drop_connection",
        ResponseKind::Challenge => "challenge",
        ResponseKind::NotABot => "not_a_bot",
        ResponseKind::JsChallenge => "js_challenge",
        ResponseKind::Maze => "maze",
        ResponseKind::Tarpit => "tarpit",
        ResponseKind::CheckpointResponse => "checkpoint_response",
        ResponseKind::DefenceFollowupResponse => "defence_followup_response",
        ResponseKind::SimPublicResponse => "sim_public_response",
        ResponseKind::ControlPlaneResponse => "control_plane_response",
    }
}

fn normalize_policy_source(source: PolicySource) -> &'static str {
    match source {
        PolicySource::EarlyRoute => "early_route",
        PolicySource::StaticAssetBypass => "static_asset_bypass",
        PolicySource::AllowlistBypass => "allowlist_bypass",
        PolicySource::PolicyGraphFirstTranche => "policy_graph_first_tranche",
        PolicySource::PolicyGraphVerifiedIdentityTranche => "policy_graph_verified_identity_tranche",
        PolicySource::PolicyGraphSecondTranche => "policy_graph_second_tranche",
        PolicySource::CleanAllow => "clean_allow",
        PolicySource::DefenceFollowup => "defence_followup",
        PolicySource::SimPublic => "sim_public",
        PolicySource::BootstrapFailure => "bootstrap_failure",
    }
}

fn normalize_traffic_lane(lane: TrafficLane) -> &'static str {
    match lane {
        TrafficLane::LikelyHuman => "likely_human",
        TrafficLane::UnknownInteractive => "unknown_interactive",
        TrafficLane::SuspiciousAutomation => "suspicious_automation",
        TrafficLane::DeclaredCrawler => "declared_crawler",
        TrafficLane::DeclaredUserTriggeredAgent => "declared_user_triggered_agent",
        TrafficLane::VerifiedBot => "verified_bot",
        TrafficLane::SignedAgent => "signed_agent",
    }
}

fn normalize_telemetry_exactness(exactness: TelemetryExactness) -> &'static str {
    match exactness {
        TelemetryExactness::Exact => "exact",
        TelemetryExactness::Derived => "derived",
        TelemetryExactness::BestEffort => "best_effort",
    }
}

fn normalize_telemetry_basis(basis: TelemetryBasis) -> &'static str {
    match basis {
        TelemetryBasis::Observed => "observed",
        TelemetryBasis::Policy => "policy",
        TelemetryBasis::Verified => "verified",
        TelemetryBasis::Residual => "residual",
        TelemetryBasis::Mixed => "mixed",
    }
}

impl RenderedRequestOutcome {
    pub(crate) fn from_handled_response(
        traffic_origin: TrafficOrigin,
        handled: &HandledRequestResponse,
        verified_identity_lane: Option<RequestOutcomeLane>,
        verified_identity_category: Option<NonHumanCategoryAssignment>,
    ) -> Self {
        let classification = classify_current_runtime_branch(&handled.branch);
        let outcome_class = if matches!(classification.route_action_family, RouteActionFamily::ControlPlane)
        {
            RequestOutcomeClass::ControlResponse
        } else if handled.rendered.forward_attempted {
            RequestOutcomeClass::Forwarded
        } else {
            RequestOutcomeClass::ShortCircuited
        };

        Self {
            traffic_origin,
            measurement_scope: classification.measurement_scope,
            route_action_family: classification.route_action_family,
            execution_mode: handled.execution_mode,
            traffic_lane: verified_identity_lane.or_else(|| classification.traffic_lane.map(Into::into)),
            non_human_category: verified_identity_category.or_else(|| {
                classification
                    .traffic_lane
                    .and_then(|lane| non_human_category_assignment_for_lane(lane.lane))
            }),
            outcome_class,
            response_kind: handled.rendered.response_kind,
            http_status: *handled.rendered.response.status(),
            response_bytes: handled.rendered.response.body().len() as u64,
            forwarded_upstream_latency_ms: handled.rendered.forward_latency_ms,
            forward_attempted: handled.rendered.forward_attempted,
            forward_failure_class: handled.rendered.forward_failure_class,
            intended_action: handled.rendered.intended_action,
            policy_source: classification.policy_source,
        }
    }

    pub(crate) fn observed_summary(&self) -> ObservedRequestOutcomeSummary {
        ObservedRequestOutcomeSummary {
            traffic_origin: normalize_traffic_origin(self.traffic_origin).to_string(),
            measurement_scope: normalize_measurement_scope(self.measurement_scope).to_string(),
            route_action_family: normalize_route_action_family(self.route_action_family)
                .to_string(),
            execution_mode: normalize_execution_mode(self.execution_mode).to_string(),
            outcome_class: normalize_request_outcome_class(self.outcome_class).to_string(),
            response_kind: normalize_response_kind(self.response_kind).to_string(),
            policy_source: normalize_policy_source(self.policy_source).to_string(),
            http_status: self.http_status,
            traffic_lane: self
                .traffic_lane
                .map(|lane| normalize_traffic_lane(lane.lane).to_string()),
            lane_exactness: self
                .traffic_lane
                .map(|lane| normalize_telemetry_exactness(lane.exactness).to_string()),
            lane_basis: self
                .traffic_lane
                .map(|lane| normalize_telemetry_basis(lane.basis).to_string()),
            non_human_category_id: self
                .non_human_category
                .map(|assignment| assignment.category_id.as_str().to_string()),
            non_human_assignment_status: self
                .non_human_category
                .map(|assignment| assignment.assignment_status.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(status: u16, body: &str) -> Response {
        Response::builder().status(status).body(body.as_bytes().to_vec()).build()
    }

    #[test]
    fn forwarded_clean_allow_derives_unknown_interactive_outcome() {
        let handled = HandledRequestResponse {
            branch: CurrentRuntimeBranch::CleanAllow {
                not_a_bot_marker_valid: false,
            },
            execution_mode: ExecutionMode::Enforced,
            rendered: RenderedResponseEvidence::forwarded(response(200, "ok"), None, None),
        };

        let outcome =
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled, None, None);

        assert_eq!(outcome.traffic_origin, TrafficOrigin::Live);
        assert_eq!(outcome.measurement_scope, MeasurementScope::IngressPrimary);
        assert_eq!(outcome.route_action_family, RouteActionFamily::PublicContent);
        assert_eq!(outcome.policy_source, PolicySource::CleanAllow);
        assert_eq!(outcome.outcome_class, RequestOutcomeClass::Forwarded);
        assert_eq!(outcome.response_kind, ResponseKind::ForwardAllow);
        assert_eq!(outcome.http_status, 200);
        assert_eq!(outcome.response_bytes, 2);
        assert_eq!(outcome.forwarded_upstream_latency_ms, None);
        assert!(outcome.forward_attempted);
        assert_eq!(
            outcome.traffic_lane,
            Some(RequestOutcomeLane {
                lane: TrafficLane::UnknownInteractive,
                exactness: TelemetryExactness::Derived,
                basis: TelemetryBasis::Residual,
            })
        );
        assert!(outcome.non_human_category.is_none());
    }

    #[test]
    fn direct_human_proof_preserves_exact_likely_human_lane() {
        let handled = HandledRequestResponse {
            branch: CurrentRuntimeBranch::CleanAllow {
                not_a_bot_marker_valid: true,
            },
            execution_mode: ExecutionMode::Enforced,
            rendered: RenderedResponseEvidence::forwarded(response(200, "ok"), None, None),
        };

        let outcome =
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled, None, None);

        assert_eq!(
            outcome.traffic_lane,
            Some(RequestOutcomeLane {
                lane: TrafficLane::LikelyHuman,
                exactness: TelemetryExactness::Exact,
                basis: TelemetryBasis::Observed,
            })
        );
        assert!(outcome.non_human_category.is_none());
    }

    #[test]
    fn observed_summary_normalizes_request_outcome_for_compact_persistence() {
        let outcome = RenderedRequestOutcome {
            traffic_origin: TrafficOrigin::AdversarySim,
            measurement_scope: MeasurementScope::DefenceFollowup,
            route_action_family: RouteActionFamily::DefenceFollowup,
            execution_mode: ExecutionMode::Enforced,
            traffic_lane: Some(RequestOutcomeLane {
                lane: TrafficLane::SuspiciousAutomation,
                exactness: TelemetryExactness::Exact,
                basis: TelemetryBasis::Policy,
            }),
            non_human_category: Some(NonHumanCategoryAssignment {
                category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::AutomatedBrowser,
                assignment_status: "crosswalk_exact",
            }),
            outcome_class: RequestOutcomeClass::ShortCircuited,
            response_kind: ResponseKind::Challenge,
            http_status: 403,
            response_bytes: 512,
            forwarded_upstream_latency_ms: None,
            forward_attempted: false,
            forward_failure_class: None,
            intended_action: None,
            policy_source: PolicySource::DefenceFollowup,
        };

        let summary = outcome.observed_summary();

        assert_eq!(summary.traffic_origin, "adversary_sim");
        assert_eq!(summary.measurement_scope, "defence_followup");
        assert_eq!(summary.route_action_family, "defence_followup");
        assert_eq!(summary.execution_mode, "enforced");
        assert_eq!(summary.outcome_class, "short_circuited");
        assert_eq!(summary.response_kind, "challenge");
        assert_eq!(summary.policy_source, "defence_followup");
        assert_eq!(summary.http_status, 403);
        assert_eq!(summary.traffic_lane.as_deref(), Some("suspicious_automation"));
        assert_eq!(summary.lane_exactness.as_deref(), Some("exact"));
        assert_eq!(summary.lane_basis.as_deref(), Some("policy"));
        assert_eq!(
            summary.non_human_category_id.as_deref(),
            Some("automated_browser")
        );
        assert_eq!(
            summary.non_human_assignment_status.as_deref(),
            Some("crosswalk_exact")
        );
    }

    #[test]
    fn forwarded_outcome_preserves_forwarded_upstream_latency() {
        let mut rendered = RenderedResponseEvidence::forwarded(response(200, "ok"), None, None);
        rendered.forward_latency_ms = Some(47);
        let handled = HandledRequestResponse {
            branch: CurrentRuntimeBranch::CleanAllow {
                not_a_bot_marker_valid: false,
            },
            execution_mode: ExecutionMode::Enforced,
            rendered,
        };

        let outcome =
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled, None, None);

        assert_eq!(outcome.forwarded_upstream_latency_ms, Some(47));
    }

    #[test]
    fn synthetic_shadow_action_preserves_actual_rendered_and_intended_truths() {
        let handled = HandledRequestResponse {
            branch: CurrentRuntimeBranch::PolicyDecision(
                crate::runtime::policy_graph::PolicyDecision::BotnessChallenge {
                    score: 87,
                    signal_ids: vec![],
                },
            ),
            execution_mode: ExecutionMode::Shadow,
            rendered: RenderedResponseEvidence::synthetic_shadow_action(
                response(200, "shadow"),
                ShadowAction::Challenge,
            ),
        };

        let outcome = RenderedRequestOutcome::from_handled_response(
            TrafficOrigin::AdversarySim,
            &handled,
            None,
            None,
        );

        assert_eq!(outcome.traffic_origin, TrafficOrigin::AdversarySim);
        assert_eq!(outcome.outcome_class, RequestOutcomeClass::ShortCircuited);
        assert_eq!(outcome.response_kind, ResponseKind::SyntheticShadowAction);
        assert_eq!(outcome.intended_action, Some(ShadowAction::Challenge));
        assert_eq!(outcome.execution_mode, ExecutionMode::Shadow);
        assert_eq!(
            outcome.traffic_lane,
            Some(RequestOutcomeLane {
                lane: TrafficLane::SuspiciousAutomation,
                exactness: TelemetryExactness::Derived,
                basis: TelemetryBasis::Residual,
            })
        );
        assert_eq!(
            outcome.non_human_category,
            Some(NonHumanCategoryAssignment {
                category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::UnknownNonHuman,
                assignment_status: "insufficient_evidence",
            })
        );
    }

    #[test]
    fn control_plane_branch_is_never_misclassified_as_site_traffic() {
        let handled = HandledRequestResponse {
            branch: CurrentRuntimeBranch::ControlPlaneEarlyRoute,
            execution_mode: ExecutionMode::Enforced,
            rendered: RenderedResponseEvidence::local(
                response(200, "OK"),
                ResponseKind::ControlPlaneResponse,
            ),
        };

        let outcome =
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled, None, None);

        assert_eq!(outcome.measurement_scope, MeasurementScope::Excluded);
        assert_eq!(outcome.route_action_family, RouteActionFamily::ControlPlane);
        assert_eq!(outcome.outcome_class, RequestOutcomeClass::ControlResponse);
        assert!(outcome.traffic_lane.is_none());
        assert!(outcome.non_human_category.is_none());
    }

    #[test]
    fn static_bypass_remains_explicitly_excluded_even_with_forwarded_response() {
        let handled = HandledRequestResponse {
            branch: CurrentRuntimeBranch::StaticAssetBypass,
            execution_mode: ExecutionMode::Enforced,
            rendered: RenderedResponseEvidence::forwarded(response(200, "asset"), None, None),
        };

        let outcome =
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled, None, None);

        assert_eq!(outcome.measurement_scope, MeasurementScope::Excluded);
        assert_eq!(outcome.route_action_family, RouteActionFamily::StaticAsset);
        assert_eq!(outcome.outcome_class, RequestOutcomeClass::Forwarded);
        assert!(outcome.traffic_lane.is_none());
        assert!(outcome.non_human_category.is_none());
    }

    #[test]
    fn bootstrap_failure_is_control_plane_but_not_silently_dropped_from_control_scope() {
        let handled = HandledRequestResponse {
            branch: CurrentRuntimeBranch::BootstrapFailure,
            execution_mode: ExecutionMode::Enforced,
            rendered: RenderedResponseEvidence::local(
                response(500, "Configuration unavailable"),
                ResponseKind::ControlPlaneResponse,
            ),
        };

        let outcome =
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled, None, None);

        assert_eq!(outcome.measurement_scope, MeasurementScope::BypassAndControl);
        assert_eq!(outcome.route_action_family, RouteActionFamily::ControlPlane);
        assert_eq!(outcome.policy_source, PolicySource::BootstrapFailure);
        assert_eq!(outcome.outcome_class, RequestOutcomeClass::ControlResponse);
        assert!(outcome.traffic_lane.is_none());
        assert!(outcome.non_human_category.is_none());
    }

    #[test]
    fn verified_identity_lane_override_marks_recognized_agents_without_changing_outcome_class() {
        let handled = HandledRequestResponse {
            branch: CurrentRuntimeBranch::PolicyDecision(
                crate::runtime::policy_graph::PolicyDecision::BotnessChallenge {
                    score: 87,
                    signal_ids: vec![],
                },
            ),
            execution_mode: ExecutionMode::Enforced,
            rendered: RenderedResponseEvidence::local(
                response(403, "challenge"),
                ResponseKind::Challenge,
            ),
        };

        let outcome = RenderedRequestOutcome::from_handled_response(
            TrafficOrigin::Live,
            &handled,
            Some(RequestOutcomeLane {
                lane: TrafficLane::SignedAgent,
                exactness: TelemetryExactness::Exact,
                basis: TelemetryBasis::Observed,
            }),
            Some(NonHumanCategoryAssignment {
                category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::IndexingBot,
                assignment_status: "classified",
            }),
        );

        assert_eq!(outcome.outcome_class, RequestOutcomeClass::ShortCircuited);
        assert_eq!(outcome.response_kind, ResponseKind::Challenge);
        assert_eq!(
            outcome.traffic_lane,
            Some(RequestOutcomeLane {
                lane: TrafficLane::SignedAgent,
                exactness: TelemetryExactness::Exact,
                basis: TelemetryBasis::Observed,
            })
        );
        assert_eq!(
            outcome.non_human_category,
            Some(NonHumanCategoryAssignment {
                category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::IndexingBot,
                assignment_status: "classified",
            })
        );
    }
}
