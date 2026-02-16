use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalAvailability {
    Active,
    Disabled,
    Unavailable,
}

impl SignalAvailability {
    pub fn as_str(self) -> &'static str {
        match self {
            SignalAvailability::Active => "active",
            SignalAvailability::Disabled => "disabled",
            SignalAvailability::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalProvenance {
    Internal,
    ExternalTrusted,
    ExternalUntrusted,
    Derived,
}

impl SignalProvenance {
    pub fn as_str(self) -> &'static str {
        match self {
            SignalProvenance::Internal => "internal",
            SignalProvenance::ExternalTrusted => "external_trusted",
            SignalProvenance::ExternalUntrusted => "external_untrusted",
            SignalProvenance::Derived => "derived",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SignalFamily {
    RequestIntegrity,
    Geo,
    Rate,
    Deception,
    FingerprintHeaderRuntime,
    FingerprintTransport,
    FingerprintTemporal,
    FingerprintPersistence,
    FingerprintBehavior,
    Other,
}

impl SignalFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            SignalFamily::RequestIntegrity => "request_integrity",
            SignalFamily::Geo => "geo",
            SignalFamily::Rate => "rate",
            SignalFamily::Deception => "deception",
            SignalFamily::FingerprintHeaderRuntime => "fingerprint_header_runtime",
            SignalFamily::FingerprintTransport => "fingerprint_transport",
            SignalFamily::FingerprintTemporal => "fingerprint_temporal",
            SignalFamily::FingerprintPersistence => "fingerprint_persistence",
            SignalFamily::FingerprintBehavior => "fingerprint_behavior",
            SignalFamily::Other => "other",
        }
    }

    fn is_fingerprint(self) -> bool {
        matches!(
            self,
            SignalFamily::FingerprintHeaderRuntime
                | SignalFamily::FingerprintTransport
                | SignalFamily::FingerprintTemporal
                | SignalFamily::FingerprintPersistence
                | SignalFamily::FingerprintBehavior
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SignalBudgetPolicy {
    pub fingerprint_total_cap: u8,
    pub fingerprint_header_runtime_cap: u8,
    pub fingerprint_transport_cap: u8,
    pub fingerprint_temporal_cap: u8,
    pub fingerprint_persistence_cap: u8,
    pub fingerprint_behavior_cap: u8,
}

impl SignalBudgetPolicy {
    fn family_cap(self, family: SignalFamily) -> Option<u8> {
        match family {
            SignalFamily::FingerprintHeaderRuntime => Some(self.fingerprint_header_runtime_cap),
            SignalFamily::FingerprintTransport => Some(self.fingerprint_transport_cap),
            SignalFamily::FingerprintTemporal => Some(self.fingerprint_temporal_cap),
            SignalFamily::FingerprintPersistence => Some(self.fingerprint_persistence_cap),
            SignalFamily::FingerprintBehavior => Some(self.fingerprint_behavior_cap),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BotSignal {
    pub key: &'static str,
    pub label: &'static str,
    pub active: bool,
    pub contribution: u8,
    pub availability: SignalAvailability,
    pub provenance: SignalProvenance,
    pub confidence: u8,
    pub family: SignalFamily,
}

fn scale_weight_by_confidence(weight: u8, confidence: u8) -> u8 {
    let confidence = confidence.clamp(0, 10);
    (((weight as u16 * confidence as u16) + 9) / 10) as u8
}

impl BotSignal {
    pub fn scored(key: &'static str, label: &'static str, active: bool, weight: u8) -> Self {
        Self::scored_with_metadata(
            key,
            label,
            active,
            weight,
            SignalProvenance::Internal,
            10,
            SignalFamily::Other,
        )
    }

    pub fn scored_with_metadata(
        key: &'static str,
        label: &'static str,
        active: bool,
        weight: u8,
        provenance: SignalProvenance,
        confidence: u8,
        family: SignalFamily,
    ) -> Self {
        let confidence = confidence.clamp(0, 10);
        let contribution = if active {
            scale_weight_by_confidence(weight, confidence)
        } else {
            0
        };
        Self {
            key,
            label,
            active,
            contribution,
            availability: SignalAvailability::Active,
            provenance,
            confidence,
            family,
        }
    }

    pub fn disabled(key: &'static str, label: &'static str) -> Self {
        Self::disabled_with_metadata(
            key,
            label,
            SignalProvenance::Internal,
            10,
            SignalFamily::Other,
        )
    }

    pub fn disabled_with_metadata(
        key: &'static str,
        label: &'static str,
        provenance: SignalProvenance,
        confidence: u8,
        family: SignalFamily,
    ) -> Self {
        Self {
            key,
            label,
            active: false,
            contribution: 0,
            availability: SignalAvailability::Disabled,
            provenance,
            confidence: confidence.clamp(0, 10),
            family,
        }
    }

    pub fn unavailable(key: &'static str, label: &'static str) -> Self {
        Self::unavailable_with_metadata(
            key,
            label,
            SignalProvenance::Internal,
            10,
            SignalFamily::Other,
        )
    }

    pub fn unavailable_with_metadata(
        key: &'static str,
        label: &'static str,
        provenance: SignalProvenance,
        confidence: u8,
        family: SignalFamily,
    ) -> Self {
        Self {
            key,
            label,
            active: false,
            contribution: 0,
            availability: SignalAvailability::Unavailable,
            provenance,
            confidence: confidence.clamp(0, 10),
            family,
        }
    }
}

#[derive(Debug, Default)]
pub struct SignalAccumulator {
    score: u8,
    signals: Vec<BotSignal>,
    budget_policy: Option<SignalBudgetPolicy>,
    fingerprint_total: u8,
    family_totals: BTreeMap<SignalFamily, u8>,
}

impl SignalAccumulator {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            score: 0,
            signals: Vec::with_capacity(capacity),
            budget_policy: None,
            fingerprint_total: 0,
            family_totals: BTreeMap::new(),
        }
    }

    pub fn with_capacity_and_policy(capacity: usize, budget_policy: SignalBudgetPolicy) -> Self {
        Self {
            score: 0,
            signals: Vec::with_capacity(capacity),
            budget_policy: Some(budget_policy),
            fingerprint_total: 0,
            family_totals: BTreeMap::new(),
        }
    }

    fn apply_budget(&mut self, family: SignalFamily, contribution: u8) -> u8 {
        let Some(policy) = self.budget_policy else {
            return contribution;
        };

        let mut allowed = contribution;
        if family.is_fingerprint() {
            let remaining = policy.fingerprint_total_cap.saturating_sub(self.fingerprint_total);
            allowed = allowed.min(remaining);
        }

        if let Some(cap) = policy.family_cap(family) {
            let used = self.family_totals.get(&family).copied().unwrap_or(0);
            let remaining = cap.saturating_sub(used);
            allowed = allowed.min(remaining);
        }

        if allowed > 0 {
            *self.family_totals.entry(family).or_insert(0) = self
                .family_totals
                .get(&family)
                .copied()
                .unwrap_or(0)
                .saturating_add(allowed);
            if family.is_fingerprint() {
                self.fingerprint_total = self.fingerprint_total.saturating_add(allowed);
            }
        }
        allowed
    }

    pub fn push(&mut self, mut signal: BotSignal) {
        let capped = self.apply_budget(signal.family, signal.contribution);
        signal.contribution = capped;
        self.score = self.score.saturating_add(capped);
        self.signals.push(signal);
    }

    pub fn finish(self) -> (u8, Vec<BotSignal>) {
        (self.score.clamp(0, 10), self.signals)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BotSignal, SignalAccumulator, SignalAvailability, SignalBudgetPolicy, SignalFamily,
        SignalProvenance,
    };

    #[test]
    fn accumulator_keeps_signal_order_and_score() {
        let mut accumulator = SignalAccumulator::with_capacity(2);
        accumulator.push(BotSignal::scored("a", "A", true, 3));
        accumulator.push(BotSignal::scored("b", "B", false, 3));

        let (score, signals) = accumulator.finish();
        assert_eq!(score, 3);
        assert_eq!(signals[0].key, "a");
        assert_eq!(signals[1].key, "b");
        assert_eq!(signals[0].availability, SignalAvailability::Active);
        assert_eq!(signals[1].availability, SignalAvailability::Active);
    }

    #[test]
    fn accumulator_saturates_to_botness_cap() {
        let mut accumulator = SignalAccumulator::with_capacity(2);
        accumulator.push(BotSignal::scored("a", "A", true, 9));
        accumulator.push(BotSignal::scored("b", "B", true, 9));

        let (score, _signals) = accumulator.finish();
        assert_eq!(score, 10);
    }

    #[test]
    fn disabled_and_unavailable_signals_are_explicit_zero_contribution() {
        let disabled = BotSignal::disabled("a", "A");
        let unavailable = BotSignal::unavailable("b", "B");

        assert_eq!(disabled.contribution, 0);
        assert!(!disabled.active);
        assert_eq!(disabled.availability, SignalAvailability::Disabled);

        assert_eq!(unavailable.contribution, 0);
        assert!(!unavailable.active);
        assert_eq!(unavailable.availability, SignalAvailability::Unavailable);
    }

    #[test]
    fn scored_signal_scales_with_confidence() {
        let signal = BotSignal::scored_with_metadata(
            "fp",
            "Fingerprint",
            true,
            5,
            SignalProvenance::Internal,
            4,
            SignalFamily::FingerprintHeaderRuntime,
        );
        assert_eq!(signal.contribution, 2);
        assert_eq!(signal.confidence, 4);
    }

    #[test]
    fn fingerprint_budget_policy_caps_total_and_family_contribution() {
        let policy = SignalBudgetPolicy {
            fingerprint_total_cap: 3,
            fingerprint_header_runtime_cap: 2,
            fingerprint_transport_cap: 3,
            fingerprint_temporal_cap: 3,
            fingerprint_persistence_cap: 3,
            fingerprint_behavior_cap: 3,
        };
        let mut accumulator = SignalAccumulator::with_capacity_and_policy(2, policy);
        accumulator.push(BotSignal::scored_with_metadata(
            "fp_hdr",
            "FP header mismatch",
            true,
            4,
            SignalProvenance::Internal,
            10,
            SignalFamily::FingerprintHeaderRuntime,
        ));
        accumulator.push(BotSignal::scored_with_metadata(
            "fp_transport",
            "FP transport",
            true,
            4,
            SignalProvenance::ExternalTrusted,
            10,
            SignalFamily::FingerprintTransport,
        ));

        let (score, signals) = accumulator.finish();
        assert_eq!(score, 3);
        assert_eq!(signals[0].contribution, 2);
        assert_eq!(signals[1].contribution, 1);
    }

    #[test]
    fn signal_availability_has_stable_labels() {
        assert_eq!(SignalAvailability::Active.as_str(), "active");
        assert_eq!(SignalAvailability::Disabled.as_str(), "disabled");
        assert_eq!(SignalAvailability::Unavailable.as_str(), "unavailable");
    }

    #[test]
    fn provenance_and_family_have_stable_labels() {
        assert_eq!(SignalProvenance::Internal.as_str(), "internal");
        assert_eq!(
            SignalProvenance::ExternalTrusted.as_str(),
            "external_trusted"
        );
        assert_eq!(
            SignalFamily::FingerprintHeaderRuntime.as_str(),
            "fingerprint_header_runtime"
        );
    }
}
