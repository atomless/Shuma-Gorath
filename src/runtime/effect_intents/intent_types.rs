use spin_sdk::http::Request;
use spin_sdk::key_value::Store;
#[derive(Clone)]
pub(crate) struct BanIntent {
    pub reason: String,
    pub duration_seconds: u64,
    pub score: Option<u8>,
    pub signals: Vec<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExecutionMode {
    Enforced,
    Shadow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShadowAction {
    NotABot,
    Challenge,
    JsChallenge,
    Maze,
    Block,
    Tarpit,
    Redirect,
    DropConnection,
}

impl ShadowAction {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ShadowAction::NotABot => "not_a_bot",
            ShadowAction::Challenge => "challenge",
            ShadowAction::JsChallenge => "js_challenge",
            ShadowAction::Maze => "maze",
            ShadowAction::Block => "block",
            ShadowAction::Tarpit => "tarpit",
            ShadowAction::Redirect => "redirect",
            ShadowAction::DropConnection => "drop_connection",
        }
    }
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
    RecordNotABotSubmit {
        outcome: String,
        solve_ms: Option<u64>,
    },
    RecordChallengeFailure {
        outcome: String,
    },
    RecordIpRangeChallengeSolved,
    RecordBotnessVisibility {
        assessment: crate::BotnessAssessment,
    },
    RecordLikelyHumanSample {
        sample_percent: u8,
        sample_hint: String,
    },
    RecordVerifiedIdentityTelemetry {
        record: crate::bot_identity::telemetry::IdentityVerificationTelemetryRecord,
    },
    RecordRequestOutcome {
        outcome: crate::runtime::request_outcome::RenderedRequestOutcome,
    },
    RecordShadowAction {
        action: ShadowAction,
    },
    RecordShadowPassThrough,
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
    ForwardAllow {
        reason: String,
    },
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
    pub execution_mode: ExecutionMode,
}
