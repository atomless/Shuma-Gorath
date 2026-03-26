use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EscalationLevelId {
    L0AllowClean,
    L1AllowTagged,
    L2Monitor,
    L3Shape,
    L4VerifyJs,
    L5NotABot,
    L6ChallengeStrong,
    L7DeceptionExplicit,
    L8DeceptionCovert,
    L9CostImposition,
    L10DenyTemp,
    L11DenyHard,
}

impl EscalationLevelId {
    pub const fn as_str(self) -> &'static str {
        match self {
            EscalationLevelId::L0AllowClean => "L0_ALLOW_CLEAN",
            EscalationLevelId::L1AllowTagged => "L1_ALLOW_TAGGED",
            EscalationLevelId::L2Monitor => "L2_MONITOR",
            EscalationLevelId::L3Shape => "L3_SHAPE",
            EscalationLevelId::L4VerifyJs => "L4_VERIFY_JS",
            EscalationLevelId::L5NotABot => "L5_NOT_A_BOT",
            EscalationLevelId::L6ChallengeStrong => "L6_CHALLENGE_STRONG",
            EscalationLevelId::L7DeceptionExplicit => "L7_DECEPTION_EXPLICIT",
            EscalationLevelId::L8DeceptionCovert => "L8_DECEPTION_COVERT",
            EscalationLevelId::L9CostImposition => "L9_COST_IMPOSITION",
            EscalationLevelId::L10DenyTemp => "L10_DENY_TEMP",
            EscalationLevelId::L11DenyHard => "L11_DENY_HARD",
        }
    }

    pub const fn action_id(self) -> ActionId {
        match self {
            EscalationLevelId::L0AllowClean => ActionId::Allow,
            EscalationLevelId::L1AllowTagged => ActionId::AllowTagged,
            EscalationLevelId::L2Monitor => ActionId::Monitor,
            EscalationLevelId::L3Shape => ActionId::Shape,
            EscalationLevelId::L4VerifyJs => ActionId::VerifyJs,
            EscalationLevelId::L5NotABot => ActionId::NotABot,
            EscalationLevelId::L6ChallengeStrong => ActionId::ChallengeStrong,
            EscalationLevelId::L7DeceptionExplicit => ActionId::DeceptionExplicit,
            EscalationLevelId::L8DeceptionCovert => ActionId::DeceptionCovert,
            EscalationLevelId::L9CostImposition => ActionId::CostImposition,
            EscalationLevelId::L10DenyTemp => ActionId::DenyTemp,
            EscalationLevelId::L11DenyHard => ActionId::DenyHard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionId {
    Allow,
    AllowTagged,
    Monitor,
    Shape,
    VerifyJs,
    NotABot,
    ChallengeStrong,
    DeceptionExplicit,
    DeceptionCovert,
    CostImposition,
    DenyTemp,
    DenyHard,
}

impl ActionId {
    pub const fn as_str(self) -> &'static str {
        match self {
            ActionId::Allow => "A_ALLOW",
            ActionId::AllowTagged => "A_ALLOW_TAGGED",
            ActionId::Monitor => "A_MONITOR",
            ActionId::Shape => "A_SHAPE",
            ActionId::VerifyJs => "A_VERIFY_JS",
            ActionId::NotABot => "A_NOT_A_BOT",
            ActionId::ChallengeStrong => "A_CHALLENGE_STRONG",
            ActionId::DeceptionExplicit => "A_DECEPTION_EXPLICIT",
            ActionId::DeceptionCovert => "A_DECEPTION_COVERT",
            ActionId::CostImposition => "A_COST_IMPOSITION",
            ActionId::DenyTemp => "A_DENY_TEMP",
            ActionId::DenyHard => "A_DENY_HARD",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PolicyTelemetryTaxonomy {
    pub level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detection: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<String>,
}

impl PolicyTelemetryTaxonomy {
    fn from_policy_match(matched: &PolicyMatch) -> Self {
        Self {
            level: matched.level_id().to_string(),
            action: Some(matched.action_id().to_string()),
            detection: Some(matched.detection_id().to_string()),
            signals: matched
                .signal_ids()
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }

    fn annotation_suffix(&self) -> String {
        let mut parts = vec![format!("level={}", self.level)];
        if let Some(action) = self.action.as_deref() {
            parts.push(format!("action={action}"));
        }
        if let Some(detection) = self.detection.as_deref() {
            parts.push(format!("detection={detection}"));
        }
        parts.push(format!("signals={}", self.signals.join(",")));
        format!("taxonomy[{}]", parts.join(" "))
    }

    pub(crate) fn annotate_outcome(&self, outcome: Option<&str>) -> String {
        let suffix = self.annotation_suffix();
        match outcome.map(str::trim).filter(|value| !value.is_empty()) {
            Some(base) => format!("{base} {suffix}"),
            None => suffix,
        }
    }

    pub(crate) fn compact_for_persistence(&mut self, reason: Option<&str>) {
        let normalized_reason = reason.unwrap_or_default().trim();
        let compact_challenge_taxonomy =
            normalized_reason == "js_verification" || normalized_reason.starts_with("botness_gate_");
        if !compact_challenge_taxonomy {
            return;
        }

        if self.action.as_deref() == implied_action_id_for_level(self.level.as_str()) {
            self.action = None;
        }

        if self.detection.as_deref() == implied_detection_id_for_reason(normalized_reason) {
            self.detection = None;
        }

        if let Some(implied_signals) = implied_signal_ids_for_reason(normalized_reason) {
            let current = self.signals.iter().map(String::as_str).collect::<Vec<_>>();
            if current == implied_signals {
                self.signals.clear();
            }
        }
    }
}

fn implied_action_id_for_level(level: &str) -> Option<&'static str> {
    match level {
        "L0_ALLOW_CLEAN" => Some("A_ALLOW"),
        "L1_ALLOW_TAGGED" => Some("A_ALLOW_TAGGED"),
        "L2_MONITOR" => Some("A_MONITOR"),
        "L3_SHAPE" => Some("A_SHAPE"),
        "L4_VERIFY_JS" => Some("A_VERIFY_JS"),
        "L5_NOT_A_BOT" => Some("A_NOT_A_BOT"),
        "L6_CHALLENGE_STRONG" => Some("A_CHALLENGE_STRONG"),
        "L7_DECEPTION_EXPLICIT" => Some("A_DECEPTION_EXPLICIT"),
        "L8_DECEPTION_COVERT" => Some("A_DECEPTION_COVERT"),
        "L9_COST_IMPOSITION" => Some("A_COST_IMPOSITION"),
        "L10_DENY_TEMP" => Some("A_DENY_TEMP"),
        "L11_DENY_HARD" => Some("A_DENY_HARD"),
        _ => None,
    }
}

fn implied_detection_id_for_reason(reason: &str) -> Option<&'static str> {
    match reason {
        "js_verification" => Some("D_JS_VERIFICATION_REQUIRED"),
        "botness_gate_not_a_bot" => Some("D_BOTNESS_GATE_NOT_A_BOT"),
        "botness_gate_challenge" => Some("D_BOTNESS_GATE_CHALLENGE"),
        "botness_gate_maze" => Some("D_BOTNESS_GATE_MAZE"),
        "botness_gate_challenge_disabled_fallback_maze" => {
            Some("D_CHALLENGE_DISABLED_FALLBACK_MAZE")
        }
        "botness_gate_challenge_disabled_fallback_block" => {
            Some("D_CHALLENGE_DISABLED_FALLBACK_BLOCK")
        }
        _ => None,
    }
}

fn implied_signal_ids_for_reason(reason: &str) -> Option<Vec<&'static str>> {
    match reason {
        "js_verification" => Some(vec!["S_JS_REQUIRED_MISSING"]),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedAnnotatedOutcome {
    pub outcome_text: Option<String>,
    pub taxonomy: Option<PolicyTelemetryTaxonomy>,
}

fn is_canonical_taxonomy_token(raw: &str) -> bool {
    !raw.is_empty()
        && raw
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(crate) fn parse_annotated_outcome(raw: &str) -> ParsedAnnotatedOutcome {
    let trimmed = raw.trim();
    let Some(annotation_start) = trimmed.rfind("taxonomy[") else {
        return ParsedAnnotatedOutcome {
            outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
            taxonomy: None,
        };
    };
    let Some(annotation) = trimmed.get(annotation_start..) else {
        return ParsedAnnotatedOutcome {
            outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
            taxonomy: None,
        };
    };
    if !annotation.ends_with(']') {
        return ParsedAnnotatedOutcome {
            outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
            taxonomy: None,
        };
    }

    let mut level: Option<String> = None;
    let mut action: Option<String> = None;
    let mut detection: Option<String> = None;
    let mut signals: Option<Vec<String>> = None;
    let inner = &annotation["taxonomy[".len()..annotation.len() - 1];
    for pair in inner.split_whitespace() {
        let Some((key, value)) = pair.split_once('=') else {
            return ParsedAnnotatedOutcome {
                outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
                taxonomy: None,
            };
        };
        match key {
            "level" => {
                if !is_canonical_taxonomy_token(value) {
                    return ParsedAnnotatedOutcome {
                        outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
                        taxonomy: None,
                    };
                }
                level = Some(value.to_string());
            }
            "action" => {
                if !is_canonical_taxonomy_token(value) {
                    return ParsedAnnotatedOutcome {
                        outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
                        taxonomy: None,
                    };
                }
                action = Some(value.to_string());
            }
            "detection" => {
                if !is_canonical_taxonomy_token(value) {
                    return ParsedAnnotatedOutcome {
                        outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
                        taxonomy: None,
                    };
                }
                detection = Some(value.to_string());
            }
            "signals" => {
                let parsed = if value.is_empty() {
                    Vec::new()
                } else {
                    let mut next = Vec::new();
                    for signal in value.split(',') {
                        if !is_canonical_taxonomy_token(signal) {
                            return ParsedAnnotatedOutcome {
                                outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
                                taxonomy: None,
                            };
                        }
                        next.push(signal.to_string());
                    }
                    next
                };
                signals = Some(parsed);
            }
            _ => {
                return ParsedAnnotatedOutcome {
                    outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
                    taxonomy: None,
                };
            }
        }
    }

    let Some(level) = level else {
        return ParsedAnnotatedOutcome {
            outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
            taxonomy: None,
        };
    };
    let Some(action) = action else {
        return ParsedAnnotatedOutcome {
            outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
            taxonomy: None,
        };
    };
    let Some(detection) = detection else {
        return ParsedAnnotatedOutcome {
            outcome_text: (!trimmed.is_empty()).then(|| trimmed.to_string()),
            taxonomy: None,
        };
    };

    ParsedAnnotatedOutcome {
        outcome_text: (!trimmed[..annotation_start].trim().is_empty())
            .then(|| trimmed[..annotation_start].trim().to_string()),
        taxonomy: Some(PolicyTelemetryTaxonomy {
            level,
            action: Some(action),
            detection: Some(detection),
            signals: signals.unwrap_or_default(),
        }),
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignalId {
    CtxIpTrusted,
    CtxPathClass,
    CtxUa,
    SeqOpMissing,
    SeqOpInvalid,
    SeqOpExpired,
    SeqOpReplay,
    SeqBindingMismatch,
    SeqOrderViolation,
    SeqWindowExceeded,
    SeqTimingTooFast,
    SeqTimingTooRegular,
    SeqTimingTooSlow,
    RateUsageMedium,
    RateUsageHigh,
    RateLimitHit,
    HoneypotHit,
    VerifiedIdentityAuthenticated,
    VerifiedIdentityNamedPolicy,
    VerifiedIdentityCategoryDefault,
    VerifiedIdentityCanonicalPostureFallback,
    GeoRisk,
    GeoRouteChallenge,
    GeoRouteMaze,
    GeoRouteBlock,
    JsRequiredMissing,
    BrowserOutdated,
    CdpReportLow,
    CdpReportMedium,
    CdpReportStrong,
    FingerprintUaHintMismatch,
    FingerprintUaTransportMismatch,
    FingerprintTemporalTransition,
    FingerprintFlowViolation,
    FingerprintPersistenceMissing,
    FingerprintUntrustedHeader,
    EdgeFingerprintAdditive,
    EdgeFingerprintStrong,
    EdgeFingerprintAuthoritativeBan,
    MazeTraversal,
    MazeTokenInvalid,
    MazeTokenExpired,
    MazeTokenReplay,
    MazeTokenBindingMismatch,
    MazeDepthExceeded,
    MazeBudgetExceeded,
    MazeCheckpointMissing,
    MazeMicroPowFailed,
    MazeThreshold,
    DecoyInteraction,
    TarpitPersistence,
    IpRangeCustom,
    SimTagMissingSecret,
    SimTagMissingRequiredHeaders,
    SimTagInvalidHeaderValue,
    SimTagInvalidTimestamp,
    SimTagTimestampSkew,
    SimTagSignatureMismatch,
    SimTagNonceReplay,
}

impl SignalId {
    pub const fn as_str(self) -> &'static str {
        match self {
            SignalId::CtxIpTrusted => "S_CTX_IP_TRUSTED",
            SignalId::CtxPathClass => "S_CTX_PATH_CLASS",
            SignalId::CtxUa => "S_CTX_UA",
            SignalId::SeqOpMissing => "S_SEQ_OP_MISSING",
            SignalId::SeqOpInvalid => "S_SEQ_OP_INVALID",
            SignalId::SeqOpExpired => "S_SEQ_OP_EXPIRED",
            SignalId::SeqOpReplay => "S_SEQ_OP_REPLAY",
            SignalId::SeqBindingMismatch => "S_SEQ_BINDING_MISMATCH",
            SignalId::SeqOrderViolation => "S_SEQ_ORDER_VIOLATION",
            SignalId::SeqWindowExceeded => "S_SEQ_WINDOW_EXCEEDED",
            SignalId::SeqTimingTooFast => "S_SEQ_TIMING_TOO_FAST",
            SignalId::SeqTimingTooRegular => "S_SEQ_TIMING_TOO_REGULAR",
            SignalId::SeqTimingTooSlow => "S_SEQ_TIMING_TOO_SLOW",
            SignalId::RateUsageMedium => "S_RATE_USAGE_MEDIUM",
            SignalId::RateUsageHigh => "S_RATE_USAGE_HIGH",
            SignalId::RateLimitHit => "S_RATE_LIMIT_HIT",
            SignalId::HoneypotHit => "S_HONEYPOT_HIT",
            SignalId::VerifiedIdentityAuthenticated => "S_VERIFIED_IDENTITY_AUTHENTICATED",
            SignalId::VerifiedIdentityNamedPolicy => "S_VERIFIED_IDENTITY_NAMED_POLICY",
            SignalId::VerifiedIdentityCategoryDefault => {
                "S_VERIFIED_IDENTITY_CATEGORY_DEFAULT"
            }
            SignalId::VerifiedIdentityCanonicalPostureFallback => {
                "S_VERIFIED_IDENTITY_CANONICAL_POSTURE_FALLBACK"
            }
            SignalId::GeoRisk => "S_GEO_RISK",
            SignalId::GeoRouteChallenge => "S_GEO_ROUTE_CHALLENGE",
            SignalId::GeoRouteMaze => "S_GEO_ROUTE_MAZE",
            SignalId::GeoRouteBlock => "S_GEO_ROUTE_BLOCK",
            SignalId::JsRequiredMissing => "S_JS_REQUIRED_MISSING",
            SignalId::BrowserOutdated => "S_BROWSER_OUTDATED",
            SignalId::CdpReportLow => "S_CDP_REPORT_LOW",
            SignalId::CdpReportMedium => "S_CDP_REPORT_MEDIUM",
            SignalId::CdpReportStrong => "S_CDP_REPORT_STRONG",
            SignalId::FingerprintUaHintMismatch => "S_FP_UA_HINT_MISMATCH",
            SignalId::FingerprintUaTransportMismatch => "S_FP_UA_TRANSPORT_MISMATCH",
            SignalId::FingerprintTemporalTransition => "S_FP_TEMPORAL_TRANSITION",
            SignalId::FingerprintFlowViolation => "S_FP_FLOW_VIOLATION",
            SignalId::FingerprintPersistenceMissing => "S_FP_PERSISTENCE_MISSING",
            SignalId::FingerprintUntrustedHeader => "S_FP_UNTRUSTED_HEADER",
            SignalId::EdgeFingerprintAdditive => "S_FP_EDGE_ADDITIVE",
            SignalId::EdgeFingerprintStrong => "S_FP_EDGE_STRONG",
            SignalId::EdgeFingerprintAuthoritativeBan => "S_FP_EDGE_AUTHORITATIVE_BAN",
            SignalId::MazeTraversal => "S_MAZE_TRAVERSAL",
            SignalId::MazeTokenInvalid => "S_MAZE_TOKEN_INVALID",
            SignalId::MazeTokenExpired => "S_MAZE_TOKEN_EXPIRED",
            SignalId::MazeTokenReplay => "S_MAZE_TOKEN_REPLAY",
            SignalId::MazeTokenBindingMismatch => "S_MAZE_TOKEN_BINDING_MISMATCH",
            SignalId::MazeDepthExceeded => "S_MAZE_DEPTH_EXCEEDED",
            SignalId::MazeBudgetExceeded => "S_MAZE_BUDGET_EXCEEDED",
            SignalId::MazeCheckpointMissing => "S_MAZE_CHECKPOINT_MISSING",
            SignalId::MazeMicroPowFailed => "S_MAZE_MICRO_POW_FAILED",
            SignalId::MazeThreshold => "S_MAZE_THRESHOLD",
            SignalId::DecoyInteraction => "S_DECOY_INTERACTION",
            SignalId::TarpitPersistence => "S_TARPIT_PERSISTENCE",
            SignalId::IpRangeCustom => "S_IP_RANGE_CUSTOM",
            SignalId::SimTagMissingSecret => "S_SIM_TAG_MISSING_SECRET",
            SignalId::SimTagMissingRequiredHeaders => "S_SIM_TAG_MISSING_REQUIRED_HEADERS",
            SignalId::SimTagInvalidHeaderValue => "S_SIM_TAG_INVALID_HEADER_VALUE",
            SignalId::SimTagInvalidTimestamp => "S_SIM_TAG_INVALID_TIMESTAMP",
            SignalId::SimTagTimestampSkew => "S_SIM_TAG_TIMESTAMP_SKEW",
            SignalId::SimTagSignatureMismatch => "S_SIM_TAG_SIGNATURE_MISMATCH",
            SignalId::SimTagNonceReplay => "S_SIM_TAG_NONCE_REPLAY",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionId {
    AllowClean,
    HoneypotHit,
    RateLimitHit,
    ExistingBan,
    VerifiedIdentityPolicyAllow,
    VerifiedIdentityPolicyObserve,
    VerifiedIdentityPolicyRestrict,
    VerifiedIdentityPolicyDeny,
    SeqOpMissing,
    SeqOpInvalid,
    SeqOpExpired,
    SeqOpReplay,
    SeqBindingMismatch,
    SeqOrderViolation,
    SeqWindowExceeded,
    SeqTimingTooFast,
    SeqTimingTooRegular,
    SeqTimingTooSlow,
    GeoRouteBlock,
    GeoRouteChallenge,
    GeoRouteMaze,
    GeoRouteMazeFallbackChallenge,
    ChallengeDisabledFallbackMaze,
    ChallengeDisabledFallbackBlock,
    BotnessGateNotABot,
    BotnessGateChallenge,
    BotnessGateMaze,
    JsVerificationRequired,
    CdpReportLow,
    CdpReportMedium,
    CdpReportStrong,
    CdpAutoBan,
    FingerprintUaHintMismatch,
    FingerprintUaTransportMismatch,
    FingerprintTemporalTransition,
    FingerprintFlowViolation,
    FingerprintPersistenceMissing,
    FingerprintUntrustedHeader,
    EdgeFingerprintAdditive,
    EdgeFingerprintStrong,
    EdgeFingerprintAuthoritativeBan,
    MazeTraversal,
    MazeTokenInvalid,
    MazeTokenExpired,
    MazeTokenReplay,
    MazeTokenBindingMismatch,
    MazeDepthExceeded,
    MazeBudgetExceeded,
    MazeCheckpointMissing,
    MazeMicroPowFailed,
    MazeThresholdBan,
    IpRangeAdvisory,
    IpRangeForbidden,
    IpRangeCustomMessage,
    IpRangeDropConnection,
    IpRangeRedirect,
    IpRangeRateLimit,
    IpRangeHoneypot,
    IpRangeMaze,
    IpRangeTarpit,
}

impl DetectionId {
    pub const fn as_str(self) -> &'static str {
        match self {
            DetectionId::AllowClean => "D_ALLOW_CLEAN",
            DetectionId::HoneypotHit => "D_HONEYPOT_HIT",
            DetectionId::RateLimitHit => "D_RATE_LIMIT_HIT",
            DetectionId::ExistingBan => "D_EXISTING_BAN",
            DetectionId::VerifiedIdentityPolicyAllow => "D_VERIFIED_IDENTITY_POLICY_ALLOW",
            DetectionId::VerifiedIdentityPolicyObserve => "D_VERIFIED_IDENTITY_POLICY_OBSERVE",
            DetectionId::VerifiedIdentityPolicyRestrict => {
                "D_VERIFIED_IDENTITY_POLICY_RESTRICT"
            }
            DetectionId::VerifiedIdentityPolicyDeny => "D_VERIFIED_IDENTITY_POLICY_DENY",
            DetectionId::SeqOpMissing => "D_SEQ_OP_MISSING",
            DetectionId::SeqOpInvalid => "D_SEQ_OP_INVALID",
            DetectionId::SeqOpExpired => "D_SEQ_OP_EXPIRED",
            DetectionId::SeqOpReplay => "D_SEQ_OP_REPLAY",
            DetectionId::SeqBindingMismatch => "D_SEQ_BINDING_MISMATCH",
            DetectionId::SeqOrderViolation => "D_SEQ_ORDER_VIOLATION",
            DetectionId::SeqWindowExceeded => "D_SEQ_WINDOW_EXCEEDED",
            DetectionId::SeqTimingTooFast => "D_SEQ_TIMING_TOO_FAST",
            DetectionId::SeqTimingTooRegular => "D_SEQ_TIMING_TOO_REGULAR",
            DetectionId::SeqTimingTooSlow => "D_SEQ_TIMING_TOO_SLOW",
            DetectionId::GeoRouteBlock => "D_GEO_ROUTE_BLOCK",
            DetectionId::GeoRouteChallenge => "D_GEO_ROUTE_CHALLENGE",
            DetectionId::GeoRouteMaze => "D_GEO_ROUTE_MAZE",
            DetectionId::GeoRouteMazeFallbackChallenge => "D_GEO_ROUTE_MAZE_FALLBACK_CHALLENGE",
            DetectionId::ChallengeDisabledFallbackMaze => "D_CHALLENGE_DISABLED_FALLBACK_MAZE",
            DetectionId::ChallengeDisabledFallbackBlock => {
                "D_CHALLENGE_DISABLED_FALLBACK_BLOCK"
            }
            DetectionId::BotnessGateNotABot => "D_BOTNESS_GATE_NOT_A_BOT",
            DetectionId::BotnessGateChallenge => "D_BOTNESS_GATE_CHALLENGE",
            DetectionId::BotnessGateMaze => "D_BOTNESS_GATE_MAZE",
            DetectionId::JsVerificationRequired => "D_JS_VERIFICATION_REQUIRED",
            DetectionId::CdpReportLow => "D_CDP_REPORT_LOW",
            DetectionId::CdpReportMedium => "D_CDP_REPORT_MEDIUM",
            DetectionId::CdpReportStrong => "D_CDP_REPORT_STRONG",
            DetectionId::CdpAutoBan => "D_CDP_AUTO_BAN",
            DetectionId::FingerprintUaHintMismatch => "D_FP_UA_HINT_MISMATCH",
            DetectionId::FingerprintUaTransportMismatch => "D_FP_UA_TRANSPORT_MISMATCH",
            DetectionId::FingerprintTemporalTransition => "D_FP_TEMPORAL_TRANSITION",
            DetectionId::FingerprintFlowViolation => "D_FP_FLOW_VIOLATION",
            DetectionId::FingerprintPersistenceMissing => "D_FP_PERSISTENCE_MISSING",
            DetectionId::FingerprintUntrustedHeader => "D_FP_UNTRUSTED_HEADER",
            DetectionId::EdgeFingerprintAdditive => "D_EDGE_FP_ADDITIVE",
            DetectionId::EdgeFingerprintStrong => "D_EDGE_FP_STRONG",
            DetectionId::EdgeFingerprintAuthoritativeBan => "D_EDGE_FP_AUTHORITATIVE_BAN",
            DetectionId::MazeTraversal => "D_MAZE_TRAVERSAL",
            DetectionId::MazeTokenInvalid => "D_MAZE_TOKEN_INVALID",
            DetectionId::MazeTokenExpired => "D_MAZE_TOKEN_EXPIRED",
            DetectionId::MazeTokenReplay => "D_MAZE_TOKEN_REPLAY",
            DetectionId::MazeTokenBindingMismatch => "D_MAZE_TOKEN_BINDING_MISMATCH",
            DetectionId::MazeDepthExceeded => "D_MAZE_DEPTH_EXCEEDED",
            DetectionId::MazeBudgetExceeded => "D_MAZE_BUDGET_EXCEEDED",
            DetectionId::MazeCheckpointMissing => "D_MAZE_CHECKPOINT_MISSING",
            DetectionId::MazeMicroPowFailed => "D_MAZE_MICRO_POW_FAILED",
            DetectionId::MazeThresholdBan => "D_MAZE_THRESHOLD_BAN",
            DetectionId::IpRangeAdvisory => "D_IP_RANGE_ADVISORY",
            DetectionId::IpRangeForbidden => "D_IP_RANGE_FORBIDDEN",
            DetectionId::IpRangeCustomMessage => "D_IP_RANGE_CUSTOM_MESSAGE",
            DetectionId::IpRangeDropConnection => "D_IP_RANGE_DROP_CONNECTION",
            DetectionId::IpRangeRedirect => "D_IP_RANGE_REDIRECT",
            DetectionId::IpRangeRateLimit => "D_IP_RANGE_RATE_LIMIT",
            DetectionId::IpRangeHoneypot => "D_IP_RANGE_HONEYPOT",
            DetectionId::IpRangeMaze => "D_IP_RANGE_MAZE",
            DetectionId::IpRangeTarpit => "D_IP_RANGE_TARPIT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyMatch {
    level: EscalationLevelId,
    action: ActionId,
    detection: DetectionId,
    signal_ids: Vec<SignalId>,
}

impl PolicyMatch {
    fn new(
        level: EscalationLevelId,
        detection: DetectionId,
        mut signal_ids: Vec<SignalId>,
    ) -> Self {
        signal_ids.sort_by_key(|signal| signal.as_str());
        signal_ids.dedup();
        Self {
            level,
            action: level.action_id(),
            detection,
            signal_ids,
        }
    }

    pub fn level_id(&self) -> &'static str {
        self.level.as_str()
    }

    pub fn action_id(&self) -> &'static str {
        self.action.as_str()
    }

    pub fn detection_id(&self) -> &'static str {
        self.detection.as_str()
    }

    pub fn signal_ids(&self) -> Vec<&'static str> {
        self.signal_ids
            .iter()
            .map(|signal| signal.as_str())
            .collect()
    }

    pub fn annotate_outcome(&self, outcome: &str) -> String {
        PolicyTelemetryTaxonomy::from_policy_match(self).annotate_outcome(Some(outcome))
    }
}

pub enum PolicyTransition {
    AllowClean,
    HoneypotHit,
    RateLimitHit,
    ExistingBan,
    VerifiedIdentityPolicyAllow(Vec<SignalId>),
    VerifiedIdentityPolicyObserve(Vec<SignalId>),
    VerifiedIdentityPolicyRestrict(Vec<SignalId>),
    VerifiedIdentityPolicyDeny(Vec<SignalId>),
    SeqOpMissing,
    SeqOpInvalid,
    SeqOpExpired,
    SeqOpReplay,
    SeqTimingTooFast,
    SeqTimingTooRegular,
    SeqTimingTooSlow,
    GeoRouteBlock,
    GeoRouteChallenge,
    GeoRouteMaze,
    GeoRouteMazeFallbackChallenge,
    ChallengeDisabledFallbackMaze(Vec<SignalId>),
    ChallengeDisabledFallbackBlock(Vec<SignalId>),
    BotnessGateNotABot(Vec<SignalId>),
    BotnessGateChallenge(Vec<SignalId>),
    BotnessGateMaze(Vec<SignalId>),
    JsVerificationRequired,
    CdpReportLow,
    CdpReportMedium,
    CdpReportStrong,
    CdpAutoBan,
    EdgeFingerprintAdditive,
    EdgeFingerprintStrong,
    EdgeFingerprintAuthoritativeBan,
    SeqBindingMismatch,
    SeqOrderViolation,
    SeqWindowExceeded,
    MazeTraversal,
    MazeTokenInvalid,
    MazeTokenExpired,
    MazeTokenReplay,
    MazeTokenBindingMismatch,
    MazeDepthExceeded,
    MazeBudgetExceeded,
    MazeCheckpointMissing,
    MazeMicroPowFailed,
    MazeThresholdBan,
    IpRangeAdvisory(Vec<SignalId>),
    IpRangeForbidden(Vec<SignalId>),
    IpRangeCustomMessage(Vec<SignalId>),
    IpRangeDropConnection(Vec<SignalId>),
    IpRangeRedirect(Vec<SignalId>),
    IpRangeRateLimit(Vec<SignalId>),
    IpRangeHoneypot(Vec<SignalId>),
    IpRangeMaze(Vec<SignalId>),
    IpRangeTarpit(Vec<SignalId>),
}

pub fn resolve_policy_match(transition: PolicyTransition) -> PolicyMatch {
    match transition {
        PolicyTransition::AllowClean => PolicyMatch::new(
            EscalationLevelId::L0AllowClean,
            DetectionId::AllowClean,
            vec![],
        ),
        PolicyTransition::HoneypotHit => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::HoneypotHit,
            vec![SignalId::HoneypotHit],
        ),
        PolicyTransition::RateLimitHit => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::RateLimitHit,
            vec![SignalId::RateLimitHit],
        ),
        PolicyTransition::ExistingBan => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::ExistingBan,
            vec![],
        ),
        PolicyTransition::VerifiedIdentityPolicyAllow(signals) => PolicyMatch::new(
            EscalationLevelId::L1AllowTagged,
            DetectionId::VerifiedIdentityPolicyAllow,
            signals,
        ),
        PolicyTransition::VerifiedIdentityPolicyObserve(signals) => PolicyMatch::new(
            EscalationLevelId::L2Monitor,
            DetectionId::VerifiedIdentityPolicyObserve,
            signals,
        ),
        PolicyTransition::VerifiedIdentityPolicyRestrict(signals) => PolicyMatch::new(
            EscalationLevelId::L3Shape,
            DetectionId::VerifiedIdentityPolicyRestrict,
            signals,
        ),
        PolicyTransition::VerifiedIdentityPolicyDeny(signals) => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::VerifiedIdentityPolicyDeny,
            signals,
        ),
        PolicyTransition::SeqOpMissing => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqOpMissing,
            vec![SignalId::SeqOpMissing],
        ),
        PolicyTransition::SeqOpInvalid => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqOpInvalid,
            vec![SignalId::SeqOpInvalid],
        ),
        PolicyTransition::SeqOpExpired => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqOpExpired,
            vec![SignalId::SeqOpExpired],
        ),
        PolicyTransition::SeqOpReplay => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqOpReplay,
            vec![SignalId::SeqOpReplay],
        ),
        PolicyTransition::SeqTimingTooFast => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqTimingTooFast,
            vec![SignalId::SeqTimingTooFast],
        ),
        PolicyTransition::SeqTimingTooRegular => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqTimingTooRegular,
            vec![SignalId::SeqTimingTooRegular],
        ),
        PolicyTransition::SeqTimingTooSlow => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqTimingTooSlow,
            vec![SignalId::SeqTimingTooSlow],
        ),
        PolicyTransition::GeoRouteBlock => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::GeoRouteBlock,
            vec![SignalId::GeoRouteBlock],
        ),
        PolicyTransition::GeoRouteChallenge => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::GeoRouteChallenge,
            vec![SignalId::GeoRouteChallenge],
        ),
        PolicyTransition::GeoRouteMaze => PolicyMatch::new(
            EscalationLevelId::L7DeceptionExplicit,
            DetectionId::GeoRouteMaze,
            vec![SignalId::GeoRouteMaze],
        ),
        PolicyTransition::GeoRouteMazeFallbackChallenge => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::GeoRouteMazeFallbackChallenge,
            vec![SignalId::GeoRouteMaze],
        ),
        PolicyTransition::ChallengeDisabledFallbackMaze(signals) => PolicyMatch::new(
            EscalationLevelId::L7DeceptionExplicit,
            DetectionId::ChallengeDisabledFallbackMaze,
            signals,
        ),
        PolicyTransition::ChallengeDisabledFallbackBlock(signals) => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::ChallengeDisabledFallbackBlock,
            signals,
        ),
        PolicyTransition::BotnessGateNotABot(signals) => PolicyMatch::new(
            EscalationLevelId::L5NotABot,
            DetectionId::BotnessGateNotABot,
            signals,
        ),
        PolicyTransition::BotnessGateChallenge(signals) => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::BotnessGateChallenge,
            signals,
        ),
        PolicyTransition::BotnessGateMaze(signals) => PolicyMatch::new(
            EscalationLevelId::L7DeceptionExplicit,
            DetectionId::BotnessGateMaze,
            signals,
        ),
        PolicyTransition::JsVerificationRequired => PolicyMatch::new(
            EscalationLevelId::L4VerifyJs,
            DetectionId::JsVerificationRequired,
            vec![SignalId::JsRequiredMissing],
        ),
        PolicyTransition::CdpReportLow => PolicyMatch::new(
            EscalationLevelId::L2Monitor,
            DetectionId::CdpReportLow,
            vec![SignalId::CdpReportLow],
        ),
        PolicyTransition::CdpReportMedium => PolicyMatch::new(
            EscalationLevelId::L2Monitor,
            DetectionId::CdpReportMedium,
            vec![SignalId::CdpReportMedium],
        ),
        PolicyTransition::CdpReportStrong => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::CdpReportStrong,
            vec![SignalId::CdpReportStrong],
        ),
        PolicyTransition::CdpAutoBan => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::CdpAutoBan,
            vec![SignalId::CdpReportStrong],
        ),
        PolicyTransition::EdgeFingerprintAdditive => PolicyMatch::new(
            EscalationLevelId::L2Monitor,
            DetectionId::EdgeFingerprintAdditive,
            vec![SignalId::EdgeFingerprintAdditive],
        ),
        PolicyTransition::EdgeFingerprintStrong => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::EdgeFingerprintStrong,
            vec![SignalId::EdgeFingerprintStrong],
        ),
        PolicyTransition::EdgeFingerprintAuthoritativeBan => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::EdgeFingerprintAuthoritativeBan,
            vec![SignalId::EdgeFingerprintAuthoritativeBan],
        ),
        PolicyTransition::SeqBindingMismatch => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqBindingMismatch,
            vec![SignalId::SeqBindingMismatch],
        ),
        PolicyTransition::SeqOrderViolation => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqOrderViolation,
            vec![SignalId::SeqOrderViolation],
        ),
        PolicyTransition::SeqWindowExceeded => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::SeqWindowExceeded,
            vec![SignalId::SeqWindowExceeded],
        ),
        PolicyTransition::MazeTraversal => PolicyMatch::new(
            EscalationLevelId::L7DeceptionExplicit,
            DetectionId::MazeTraversal,
            vec![SignalId::MazeTraversal],
        ),
        PolicyTransition::MazeTokenInvalid => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::MazeTokenInvalid,
            vec![SignalId::MazeTokenInvalid],
        ),
        PolicyTransition::MazeTokenExpired => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::MazeTokenExpired,
            vec![SignalId::MazeTokenExpired],
        ),
        PolicyTransition::MazeTokenReplay => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::MazeTokenReplay,
            vec![SignalId::MazeTokenReplay],
        ),
        PolicyTransition::MazeTokenBindingMismatch => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::MazeTokenBindingMismatch,
            vec![SignalId::MazeTokenBindingMismatch],
        ),
        PolicyTransition::MazeDepthExceeded => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::MazeDepthExceeded,
            vec![SignalId::MazeDepthExceeded],
        ),
        PolicyTransition::MazeBudgetExceeded => PolicyMatch::new(
            EscalationLevelId::L9CostImposition,
            DetectionId::MazeBudgetExceeded,
            vec![SignalId::MazeBudgetExceeded],
        ),
        PolicyTransition::MazeCheckpointMissing => PolicyMatch::new(
            EscalationLevelId::L6ChallengeStrong,
            DetectionId::MazeCheckpointMissing,
            vec![SignalId::MazeCheckpointMissing],
        ),
        PolicyTransition::MazeMicroPowFailed => PolicyMatch::new(
            EscalationLevelId::L9CostImposition,
            DetectionId::MazeMicroPowFailed,
            vec![SignalId::MazeMicroPowFailed],
        ),
        PolicyTransition::MazeThresholdBan => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::MazeThresholdBan,
            vec![SignalId::MazeThreshold],
        ),
        PolicyTransition::IpRangeAdvisory(signals) => PolicyMatch::new(
            EscalationLevelId::L2Monitor,
            DetectionId::IpRangeAdvisory,
            signals,
        ),
        PolicyTransition::IpRangeForbidden(signals) => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::IpRangeForbidden,
            signals,
        ),
        PolicyTransition::IpRangeCustomMessage(signals) => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::IpRangeCustomMessage,
            signals,
        ),
        PolicyTransition::IpRangeDropConnection(signals) => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::IpRangeDropConnection,
            signals,
        ),
        PolicyTransition::IpRangeRedirect(signals) => PolicyMatch::new(
            EscalationLevelId::L3Shape,
            DetectionId::IpRangeRedirect,
            signals,
        ),
        PolicyTransition::IpRangeRateLimit(signals) => PolicyMatch::new(
            EscalationLevelId::L3Shape,
            DetectionId::IpRangeRateLimit,
            signals,
        ),
        PolicyTransition::IpRangeHoneypot(signals) => PolicyMatch::new(
            EscalationLevelId::L10DenyTemp,
            DetectionId::IpRangeHoneypot,
            signals,
        ),
        PolicyTransition::IpRangeMaze(signals) => PolicyMatch::new(
            EscalationLevelId::L7DeceptionExplicit,
            DetectionId::IpRangeMaze,
            signals,
        ),
        PolicyTransition::IpRangeTarpit(signals) => PolicyMatch::new(
            EscalationLevelId::L9CostImposition,
            DetectionId::IpRangeTarpit,
            signals,
        ),
    }
}

#[allow(dead_code)]
pub fn resolve_highest_level(candidates: &[EscalationLevelId]) -> EscalationLevelId {
    candidates
        .iter()
        .copied()
        .max()
        .unwrap_or(EscalationLevelId::L0AllowClean)
}

pub fn signal_id_for_botness_key(key: &str) -> Option<SignalId> {
    match key {
        "js_verification_required" => Some(SignalId::JsRequiredMissing),
        "browser_outdated" => Some(SignalId::BrowserOutdated),
        "geo_risk" => Some(SignalId::GeoRisk),
        "rate_pressure_medium" => Some(SignalId::RateUsageMedium),
        "rate_pressure_high" => Some(SignalId::RateUsageHigh),
        "maze_behavior" => Some(SignalId::MazeTraversal),
        "fp_ua_ch_mismatch" => Some(SignalId::FingerprintUaHintMismatch),
        "fp_ua_transport_mismatch" => Some(SignalId::FingerprintUaTransportMismatch),
        "fp_temporal_transition" => Some(SignalId::FingerprintTemporalTransition),
        "fp_flow_violation" => Some(SignalId::FingerprintFlowViolation),
        "fp_persistence_marker_missing" => Some(SignalId::FingerprintPersistenceMissing),
        "fp_untrusted_transport_header" => Some(SignalId::FingerprintUntrustedHeader),
        "fp_akamai_edge_additive" => Some(SignalId::EdgeFingerprintAdditive),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_highest_level, resolve_policy_match, signal_id_for_botness_key, DetectionId,
        EscalationLevelId, PolicyTransition, SignalId,
    };

    #[test]
    fn precedence_resolver_picks_most_restrictive_level() {
        let resolved = resolve_highest_level(&[
            EscalationLevelId::L2Monitor,
            EscalationLevelId::L7DeceptionExplicit,
            EscalationLevelId::L10DenyTemp,
            EscalationLevelId::L4VerifyJs,
        ]);
        assert_eq!(resolved.as_str(), "L10_DENY_TEMP");
    }

    #[test]
    fn transition_mapping_is_deterministic_for_hard_transitions() {
        let first = resolve_policy_match(PolicyTransition::HoneypotHit);
        let second = resolve_policy_match(PolicyTransition::HoneypotHit);
        assert_eq!(first.level_id(), "L10_DENY_TEMP");
        assert_eq!(first.action_id(), "A_DENY_TEMP");
        assert_eq!(first.detection_id(), "D_HONEYPOT_HIT");
        assert_eq!(first.signal_ids(), vec!["S_HONEYPOT_HIT"]);
        assert_eq!(first, second);
    }

    #[test]
    fn botness_signal_mapping_uses_canonical_ids() {
        assert_eq!(
            signal_id_for_botness_key("js_verification_required")
                .expect("known signal")
                .as_str(),
            "S_JS_REQUIRED_MISSING"
        );
        assert_eq!(
            signal_id_for_botness_key("browser_outdated")
                .expect("known signal")
                .as_str(),
            "S_BROWSER_OUTDATED"
        );
        assert_eq!(
            signal_id_for_botness_key("rate_pressure_high")
                .expect("known signal")
                .as_str(),
            "S_RATE_USAGE_HIGH"
        );
        assert_eq!(
            signal_id_for_botness_key("maze_behavior")
                .expect("known signal")
                .as_str(),
            "S_MAZE_TRAVERSAL"
        );
        assert_eq!(
            signal_id_for_botness_key("fp_temporal_transition")
                .expect("known signal")
                .as_str(),
            "S_FP_TEMPORAL_TRANSITION"
        );
        assert_eq!(
            signal_id_for_botness_key("fp_akamai_edge_additive")
                .expect("known signal")
                .as_str(),
            "S_FP_EDGE_ADDITIVE"
        );
        assert!(signal_id_for_botness_key("unknown").is_none());
    }

    #[test]
    fn botness_transition_deduplicates_and_sorts_signal_ids() {
        let matched = resolve_policy_match(PolicyTransition::BotnessGateChallenge(vec![
            SignalId::RateUsageHigh,
            SignalId::GeoRisk,
            SignalId::GeoRisk,
            SignalId::JsRequiredMissing,
        ]));
        assert_eq!(
            matched.signal_ids(),
            vec!["S_GEO_RISK", "S_JS_REQUIRED_MISSING", "S_RATE_USAGE_HIGH"]
        );
    }

    #[test]
    fn not_a_bot_transition_maps_to_l5_not_a_bot() {
        let matched = resolve_policy_match(PolicyTransition::BotnessGateNotABot(vec![
            SignalId::GeoRisk,
            SignalId::RateUsageMedium,
        ]));
        assert_eq!(matched.level_id(), "L5_NOT_A_BOT");
        assert_eq!(matched.action_id(), "A_NOT_A_BOT");
        assert_eq!(matched.detection_id(), "D_BOTNESS_GATE_NOT_A_BOT");
        assert_eq!(matched.signal_ids(), vec!["S_GEO_RISK", "S_RATE_USAGE_MEDIUM"]);
    }

    #[test]
    fn challenge_disabled_fallback_transition_uses_canonical_ids() {
        let matched = resolve_policy_match(PolicyTransition::ChallengeDisabledFallbackBlock(vec![
            SignalId::GeoRouteChallenge,
            SignalId::GeoRouteChallenge,
        ]));
        assert_eq!(matched.level_id(), "L10_DENY_TEMP");
        assert_eq!(matched.action_id(), "A_DENY_TEMP");
        assert_eq!(
            matched.detection_id(),
            "D_CHALLENGE_DISABLED_FALLBACK_BLOCK"
        );
        assert_eq!(matched.signal_ids(), vec!["S_GEO_ROUTE_CHALLENGE"]);
    }

    #[test]
    fn outcome_annotations_include_canonical_ids() {
        let matched = resolve_policy_match(PolicyTransition::JsVerificationRequired);
        let annotation = matched.annotate_outcome("js challenge");
        assert!(annotation.contains("L4_VERIFY_JS"));
        assert!(annotation.contains("A_VERIFY_JS"));
        assert!(annotation.contains("D_JS_VERIFICATION_REQUIRED"));
        assert!(annotation.contains("S_JS_REQUIRED_MISSING"));
    }

    #[test]
    fn sequence_binding_mismatch_maps_to_canonical_ids() {
        let matched = resolve_policy_match(PolicyTransition::SeqBindingMismatch);
        assert_eq!(matched.level_id(), "L6_CHALLENGE_STRONG");
        assert_eq!(matched.action_id(), "A_CHALLENGE_STRONG");
        assert_eq!(matched.detection_id(), "D_SEQ_BINDING_MISMATCH");
        assert_eq!(matched.signal_ids(), vec!["S_SEQ_BINDING_MISMATCH"]);
    }

    #[test]
    fn sequence_replay_transition_maps_to_canonical_ids() {
        let matched = resolve_policy_match(PolicyTransition::SeqOpReplay);
        assert_eq!(matched.level_id(), "L6_CHALLENGE_STRONG");
        assert_eq!(matched.action_id(), "A_CHALLENGE_STRONG");
        assert_eq!(matched.detection_id(), "D_SEQ_OP_REPLAY");
        assert_eq!(matched.signal_ids(), vec!["S_SEQ_OP_REPLAY"]);
    }

    #[test]
    fn sequence_order_violation_maps_to_canonical_ids() {
        let matched = resolve_policy_match(PolicyTransition::SeqOrderViolation);
        assert_eq!(matched.level_id(), "L6_CHALLENGE_STRONG");
        assert_eq!(matched.action_id(), "A_CHALLENGE_STRONG");
        assert_eq!(matched.detection_id(), "D_SEQ_ORDER_VIOLATION");
        assert_eq!(matched.signal_ids(), vec!["S_SEQ_ORDER_VIOLATION"]);
    }

    #[test]
    fn sequence_window_exceeded_maps_to_canonical_ids() {
        let matched = resolve_policy_match(PolicyTransition::SeqWindowExceeded);
        assert_eq!(matched.level_id(), "L6_CHALLENGE_STRONG");
        assert_eq!(matched.action_id(), "A_CHALLENGE_STRONG");
        assert_eq!(matched.detection_id(), "D_SEQ_WINDOW_EXCEEDED");
        assert_eq!(matched.signal_ids(), vec!["S_SEQ_WINDOW_EXCEEDED"]);
    }

    #[test]
    fn sequence_timing_regular_transition_maps_to_canonical_ids() {
        let matched = resolve_policy_match(PolicyTransition::SeqTimingTooRegular);
        assert_eq!(matched.level_id(), "L6_CHALLENGE_STRONG");
        assert_eq!(matched.action_id(), "A_CHALLENGE_STRONG");
        assert_eq!(matched.detection_id(), "D_SEQ_TIMING_TOO_REGULAR");
        assert_eq!(matched.signal_ids(), vec!["S_SEQ_TIMING_TOO_REGULAR"]);
    }

    #[test]
    fn sequence_signal_ids_are_canonical_and_stable() {
        let signal_ids = [
            SignalId::SeqOpMissing.as_str(),
            SignalId::SeqOpInvalid.as_str(),
            SignalId::SeqOpExpired.as_str(),
            SignalId::SeqOpReplay.as_str(),
            SignalId::SeqBindingMismatch.as_str(),
            SignalId::SeqOrderViolation.as_str(),
            SignalId::SeqWindowExceeded.as_str(),
            SignalId::SeqTimingTooFast.as_str(),
            SignalId::SeqTimingTooRegular.as_str(),
            SignalId::SeqTimingTooSlow.as_str(),
        ];
        assert_eq!(
            signal_ids,
            [
                "S_SEQ_OP_MISSING",
                "S_SEQ_OP_INVALID",
                "S_SEQ_OP_EXPIRED",
                "S_SEQ_OP_REPLAY",
                "S_SEQ_BINDING_MISMATCH",
                "S_SEQ_ORDER_VIOLATION",
                "S_SEQ_WINDOW_EXCEEDED",
                "S_SEQ_TIMING_TOO_FAST",
                "S_SEQ_TIMING_TOO_REGULAR",
                "S_SEQ_TIMING_TOO_SLOW",
            ]
        );
    }

    #[test]
    fn sequence_detection_ids_are_canonical_and_stable() {
        let detection_ids = [
            DetectionId::SeqOpMissing.as_str(),
            DetectionId::SeqOpInvalid.as_str(),
            DetectionId::SeqOpExpired.as_str(),
            DetectionId::SeqOpReplay.as_str(),
            DetectionId::SeqBindingMismatch.as_str(),
            DetectionId::SeqOrderViolation.as_str(),
            DetectionId::SeqWindowExceeded.as_str(),
            DetectionId::SeqTimingTooFast.as_str(),
            DetectionId::SeqTimingTooRegular.as_str(),
            DetectionId::SeqTimingTooSlow.as_str(),
        ];
        assert_eq!(
            detection_ids,
            [
                "D_SEQ_OP_MISSING",
                "D_SEQ_OP_INVALID",
                "D_SEQ_OP_EXPIRED",
                "D_SEQ_OP_REPLAY",
                "D_SEQ_BINDING_MISMATCH",
                "D_SEQ_ORDER_VIOLATION",
                "D_SEQ_WINDOW_EXCEEDED",
                "D_SEQ_TIMING_TOO_FAST",
                "D_SEQ_TIMING_TOO_REGULAR",
                "D_SEQ_TIMING_TOO_SLOW",
            ]
        );
    }
}
