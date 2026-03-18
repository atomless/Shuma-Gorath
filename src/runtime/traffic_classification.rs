#![cfg_attr(not(test), allow(dead_code))]

use crate::observability::hot_read_contract::{TelemetryBasis, TelemetryExactness};
use crate::runtime::policy_graph::PolicyDecision;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MeasurementScope {
    IngressPrimary,
    DefenceFollowup,
    BypassAndControl,
    Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RouteActionFamily {
    PublicContent,
    StaticAsset,
    DefenceFollowup,
    AllowlistBypass,
    ControlPlane,
    SimPublic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrafficLane {
    LikelyHuman,
    UnknownInteractive,
    SuspiciousAutomation,
    DeclaredCrawler,
    DeclaredUserTriggeredAgent,
    VerifiedBot,
    SignedAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TrafficLaneAssignment {
    pub lane: TrafficLane,
    pub exactness: TelemetryExactness,
    pub basis: TelemetryBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicySource {
    EarlyRoute,
    StaticAssetBypass,
    AllowlistBypass,
    PolicyGraphFirstTranche,
    PolicyGraphSecondTranche,
    CleanAllow,
    DefenceFollowup,
    SimPublic,
    BootstrapFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CurrentRuntimeBranch {
    ControlPlaneEarlyRoute,
    StaticAssetBypass,
    PathAllowlistBypass,
    IpAllowlistBypass,
    PolicyDecision(PolicyDecision),
    CleanAllow { not_a_bot_marker_valid: bool },
    DefenceFollowup,
    SimPublic,
    BootstrapFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MonitoringTrafficClassification {
    pub measurement_scope: MeasurementScope,
    pub route_action_family: RouteActionFamily,
    pub traffic_lane: Option<TrafficLaneAssignment>,
    pub policy_source: PolicySource,
}

const LIKELY_HUMAN_OBSERVED: TrafficLaneAssignment = TrafficLaneAssignment {
    lane: TrafficLane::LikelyHuman,
    exactness: TelemetryExactness::Exact,
    basis: TelemetryBasis::Observed,
};

const UNKNOWN_INTERACTIVE_RESIDUAL: TrafficLaneAssignment = TrafficLaneAssignment {
    lane: TrafficLane::UnknownInteractive,
    exactness: TelemetryExactness::Derived,
    basis: TelemetryBasis::Residual,
};

const SUSPICIOUS_POLICY: TrafficLaneAssignment = TrafficLaneAssignment {
    lane: TrafficLane::SuspiciousAutomation,
    exactness: TelemetryExactness::Exact,
    basis: TelemetryBasis::Policy,
};

const SUSPICIOUS_RESIDUAL: TrafficLaneAssignment = TrafficLaneAssignment {
    lane: TrafficLane::SuspiciousAutomation,
    exactness: TelemetryExactness::Derived,
    basis: TelemetryBasis::Residual,
};

pub(crate) fn classify_current_runtime_branch(
    branch: &CurrentRuntimeBranch,
) -> MonitoringTrafficClassification {
    match branch {
        CurrentRuntimeBranch::ControlPlaneEarlyRoute => MonitoringTrafficClassification {
            measurement_scope: MeasurementScope::Excluded,
            route_action_family: RouteActionFamily::ControlPlane,
            traffic_lane: None,
            policy_source: PolicySource::EarlyRoute,
        },
        CurrentRuntimeBranch::StaticAssetBypass => MonitoringTrafficClassification {
            // The current prerequisite tranche explicitly chooses not to burden the static fast
            // path with exact primary-ingress telemetry until a telemetry-safe low-cost path exists.
            measurement_scope: MeasurementScope::Excluded,
            route_action_family: RouteActionFamily::StaticAsset,
            traffic_lane: None,
            policy_source: PolicySource::StaticAssetBypass,
        },
        CurrentRuntimeBranch::PathAllowlistBypass | CurrentRuntimeBranch::IpAllowlistBypass => {
            MonitoringTrafficClassification {
                measurement_scope: MeasurementScope::BypassAndControl,
                route_action_family: RouteActionFamily::AllowlistBypass,
                traffic_lane: None,
                policy_source: PolicySource::AllowlistBypass,
            }
        }
        CurrentRuntimeBranch::PolicyDecision(decision) => classify_policy_decision(decision),
        CurrentRuntimeBranch::CleanAllow {
            not_a_bot_marker_valid,
        } => MonitoringTrafficClassification {
            measurement_scope: MeasurementScope::IngressPrimary,
            route_action_family: RouteActionFamily::PublicContent,
            traffic_lane: Some(if *not_a_bot_marker_valid {
                LIKELY_HUMAN_OBSERVED
            } else {
                UNKNOWN_INTERACTIVE_RESIDUAL
            }),
            policy_source: PolicySource::CleanAllow,
        },
        CurrentRuntimeBranch::DefenceFollowup => MonitoringTrafficClassification {
            measurement_scope: MeasurementScope::DefenceFollowup,
            route_action_family: RouteActionFamily::DefenceFollowup,
            traffic_lane: None,
            policy_source: PolicySource::DefenceFollowup,
        },
        CurrentRuntimeBranch::SimPublic => MonitoringTrafficClassification {
            measurement_scope: MeasurementScope::Excluded,
            route_action_family: RouteActionFamily::SimPublic,
            traffic_lane: None,
            policy_source: PolicySource::SimPublic,
        },
        CurrentRuntimeBranch::BootstrapFailure => MonitoringTrafficClassification {
            measurement_scope: MeasurementScope::Excluded,
            route_action_family: RouteActionFamily::ControlPlane,
            traffic_lane: None,
            policy_source: PolicySource::BootstrapFailure,
        },
    }
}

fn classify_policy_decision(decision: &PolicyDecision) -> MonitoringTrafficClassification {
    match decision {
        PolicyDecision::IpRangeEmergencyAllowlisted { .. } | PolicyDecision::IpRangeAdvisory { .. } => {
            MonitoringTrafficClassification {
                measurement_scope: MeasurementScope::BypassAndControl,
                route_action_family: RouteActionFamily::AllowlistBypass,
                traffic_lane: None,
                policy_source: PolicySource::AllowlistBypass,
            }
        }
        PolicyDecision::IpRangeForbidden { .. }
        | PolicyDecision::IpRangeCustomMessage { .. }
        | PolicyDecision::IpRangeDropConnection { .. }
        | PolicyDecision::IpRangeRedirect { .. }
        | PolicyDecision::IpRangeRateLimit { .. }
        | PolicyDecision::IpRangeHoneypot { .. }
        | PolicyDecision::IpRangeMaze { .. }
        | PolicyDecision::IpRangeTarpit { .. }
        | PolicyDecision::HoneypotHit
        | PolicyDecision::RateLimitHit
        | PolicyDecision::ExistingBan => MonitoringTrafficClassification {
            measurement_scope: MeasurementScope::IngressPrimary,
            route_action_family: RouteActionFamily::PublicContent,
            traffic_lane: Some(SUSPICIOUS_POLICY),
            policy_source: PolicySource::PolicyGraphFirstTranche,
        },
        PolicyDecision::GeoBlock
        | PolicyDecision::GeoMaze
        | PolicyDecision::GeoMazeFallbackChallenge
        | PolicyDecision::GeoChallenge
        | PolicyDecision::GeoChallengeFallbackMaze
        | PolicyDecision::GeoFallbackBlockFromMaze
        | PolicyDecision::GeoFallbackBlockFromChallenge
        | PolicyDecision::JsChallengeRequired => MonitoringTrafficClassification {
            measurement_scope: MeasurementScope::IngressPrimary,
            route_action_family: RouteActionFamily::PublicContent,
            traffic_lane: Some(UNKNOWN_INTERACTIVE_RESIDUAL),
            policy_source: PolicySource::PolicyGraphSecondTranche,
        },
        PolicyDecision::BotnessMaze { .. }
        | PolicyDecision::BotnessNotABot { .. }
        | PolicyDecision::BotnessChallenge { .. }
        | PolicyDecision::BotnessChallengeFallbackMaze { .. }
        | PolicyDecision::BotnessChallengeFallbackBlock { .. } => MonitoringTrafficClassification {
            measurement_scope: MeasurementScope::IngressPrimary,
            route_action_family: RouteActionFamily::PublicContent,
            traffic_lane: Some(SUSPICIOUS_RESIDUAL),
            policy_source: PolicySource::PolicyGraphSecondTranche,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_runtime_branch_mapping_keeps_control_and_site_traffic_separate() {
        let control = classify_current_runtime_branch(&CurrentRuntimeBranch::ControlPlaneEarlyRoute);
        assert_eq!(control.measurement_scope, MeasurementScope::Excluded);
        assert_eq!(control.route_action_family, RouteActionFamily::ControlPlane);
        assert_eq!(control.policy_source, PolicySource::EarlyRoute);
        assert!(control.traffic_lane.is_none());

        let clean_allow = classify_current_runtime_branch(&CurrentRuntimeBranch::CleanAllow {
            not_a_bot_marker_valid: false,
        });
        assert_eq!(clean_allow.measurement_scope, MeasurementScope::IngressPrimary);
        assert_eq!(clean_allow.route_action_family, RouteActionFamily::PublicContent);
        assert_eq!(clean_allow.policy_source, PolicySource::CleanAllow);
        assert_eq!(clean_allow.traffic_lane, Some(UNKNOWN_INTERACTIVE_RESIDUAL));
    }

    #[test]
    fn static_bypass_is_explicitly_excluded_from_exact_primary_ingress_for_now() {
        let classification =
            classify_current_runtime_branch(&CurrentRuntimeBranch::StaticAssetBypass);
        assert_eq!(classification.measurement_scope, MeasurementScope::Excluded);
        assert_eq!(
            classification.route_action_family,
            RouteActionFamily::StaticAsset
        );
        assert_eq!(
            classification.policy_source,
            PolicySource::StaticAssetBypass
        );
        assert!(classification.traffic_lane.is_none());
    }

    #[test]
    fn policy_decisions_map_to_shared_lane_contract() {
        let honeypot = classify_current_runtime_branch(&CurrentRuntimeBranch::PolicyDecision(
            PolicyDecision::HoneypotHit,
        ));
        assert_eq!(honeypot.measurement_scope, MeasurementScope::IngressPrimary);
        assert_eq!(honeypot.traffic_lane, Some(SUSPICIOUS_POLICY));
        assert_eq!(honeypot.policy_source, PolicySource::PolicyGraphFirstTranche);

        let geo = classify_current_runtime_branch(&CurrentRuntimeBranch::PolicyDecision(
            PolicyDecision::GeoChallenge,
        ));
        assert_eq!(geo.traffic_lane, Some(UNKNOWN_INTERACTIVE_RESIDUAL));
        assert_eq!(geo.policy_source, PolicySource::PolicyGraphSecondTranche);

        let botness = classify_current_runtime_branch(&CurrentRuntimeBranch::PolicyDecision(
            PolicyDecision::BotnessChallenge {
                score: 7,
                signal_ids: vec![],
            },
        ));
        assert_eq!(botness.traffic_lane, Some(SUSPICIOUS_RESIDUAL));
        assert_eq!(botness.policy_source, PolicySource::PolicyGraphSecondTranche);
    }

    #[test]
    fn direct_human_proof_promotes_clean_allow_to_likely_human() {
        let classification = classify_current_runtime_branch(&CurrentRuntimeBranch::CleanAllow {
            not_a_bot_marker_valid: true,
        });
        assert_eq!(classification.traffic_lane, Some(LIKELY_HUMAN_OBSERVED));
    }

    #[test]
    fn followup_and_bypass_branches_stay_out_of_primary_mix() {
        let allowlist =
            classify_current_runtime_branch(&CurrentRuntimeBranch::PathAllowlistBypass);
        assert_eq!(allowlist.measurement_scope, MeasurementScope::BypassAndControl);
        assert_eq!(
            allowlist.route_action_family,
            RouteActionFamily::AllowlistBypass
        );
        assert!(allowlist.traffic_lane.is_none());

        let followup = classify_current_runtime_branch(&CurrentRuntimeBranch::DefenceFollowup);
        assert_eq!(followup.measurement_scope, MeasurementScope::DefenceFollowup);
        assert_eq!(
            followup.route_action_family,
            RouteActionFamily::DefenceFollowup
        );
        assert_eq!(followup.policy_source, PolicySource::DefenceFollowup);
    }
}
