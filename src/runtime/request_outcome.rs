#![cfg_attr(not(test), allow(dead_code))]

use spin_sdk::http::Response;

use crate::observability::hot_read_contract::{TelemetryBasis, TelemetryExactness};
use crate::runtime::effect_intents::{ExecutionMode, ShadowAction};
use crate::runtime::traffic_classification::{
    classify_current_runtime_branch, CurrentRuntimeBranch, MeasurementScope, PolicySource,
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
    pub forward_failure_class: Option<&'static str>,
    pub intended_action: Option<ShadowAction>,
}

impl RenderedResponseEvidence {
    pub(crate) fn local(response: Response, response_kind: ResponseKind) -> Self {
        Self {
            response,
            response_kind,
            forward_attempted: false,
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
            forward_failure_class: failure_class,
            intended_action,
        }
    }

    pub(crate) fn synthetic_shadow_allow(response: Response) -> Self {
        Self {
            response,
            response_kind: ResponseKind::SyntheticShadowAllow,
            forward_attempted: false,
            forward_failure_class: None,
            intended_action: None,
        }
    }

    pub(crate) fn synthetic_shadow_action(response: Response, intended_action: ShadowAction) -> Self {
        Self {
            response,
            response_kind: ResponseKind::SyntheticShadowAction,
            forward_attempted: false,
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
    pub outcome_class: RequestOutcomeClass,
    pub response_kind: ResponseKind,
    pub http_status: u16,
    pub response_bytes: u64,
    pub forward_attempted: bool,
    pub forward_failure_class: Option<&'static str>,
    pub intended_action: Option<ShadowAction>,
    pub policy_source: PolicySource,
}

impl RenderedRequestOutcome {
    pub(crate) fn from_handled_response(
        traffic_origin: TrafficOrigin,
        handled: &HandledRequestResponse,
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
            traffic_lane: classification.traffic_lane.map(Into::into),
            outcome_class,
            response_kind: handled.rendered.response_kind,
            http_status: *handled.rendered.response.status(),
            response_bytes: handled.rendered.response.body().len() as u64,
            forward_attempted: handled.rendered.forward_attempted,
            forward_failure_class: handled.rendered.forward_failure_class,
            intended_action: handled.rendered.intended_action,
            policy_source: classification.policy_source,
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
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled);

        assert_eq!(outcome.traffic_origin, TrafficOrigin::Live);
        assert_eq!(outcome.measurement_scope, MeasurementScope::IngressPrimary);
        assert_eq!(outcome.route_action_family, RouteActionFamily::PublicContent);
        assert_eq!(outcome.policy_source, PolicySource::CleanAllow);
        assert_eq!(outcome.outcome_class, RequestOutcomeClass::Forwarded);
        assert_eq!(outcome.response_kind, ResponseKind::ForwardAllow);
        assert_eq!(outcome.http_status, 200);
        assert_eq!(outcome.response_bytes, 2);
        assert!(outcome.forward_attempted);
        assert_eq!(
            outcome.traffic_lane,
            Some(RequestOutcomeLane {
                lane: TrafficLane::UnknownInteractive,
                exactness: TelemetryExactness::Derived,
                basis: TelemetryBasis::Residual,
            })
        );
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
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled);

        assert_eq!(
            outcome.traffic_lane,
            Some(RequestOutcomeLane {
                lane: TrafficLane::LikelyHuman,
                exactness: TelemetryExactness::Exact,
                basis: TelemetryBasis::Observed,
            })
        );
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
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled);

        assert_eq!(outcome.measurement_scope, MeasurementScope::Excluded);
        assert_eq!(outcome.route_action_family, RouteActionFamily::ControlPlane);
        assert_eq!(outcome.outcome_class, RequestOutcomeClass::ControlResponse);
        assert!(outcome.traffic_lane.is_none());
    }

    #[test]
    fn static_bypass_remains_explicitly_excluded_even_with_forwarded_response() {
        let handled = HandledRequestResponse {
            branch: CurrentRuntimeBranch::StaticAssetBypass,
            execution_mode: ExecutionMode::Enforced,
            rendered: RenderedResponseEvidence::forwarded(response(200, "asset"), None, None),
        };

        let outcome =
            RenderedRequestOutcome::from_handled_response(TrafficOrigin::Live, &handled);

        assert_eq!(outcome.measurement_scope, MeasurementScope::Excluded);
        assert_eq!(outcome.route_action_family, RouteActionFamily::StaticAsset);
        assert_eq!(outcome.outcome_class, RequestOutcomeClass::Forwarded);
        assert!(outcome.traffic_lane.is_none());
    }
}
