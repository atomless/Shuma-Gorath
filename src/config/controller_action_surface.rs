use super::controller_action_catalog::{
    AllowedActionGroupDefinition, AllowedActionValueConstraintDefinition,
    ALLOWED_ACTION_GROUP_DEFINITIONS,
};
use super::controller_action_guardrails::{build_family_summaries, group_ids_with_status};
use super::controller_mutability_policy::{
    allowed_actions_status_for_admin_config_paths, CONTROLLER_MUTABILITY_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) const ALLOWED_ACTIONS_SCHEMA_VERSION: &str = "allowed_actions_v1";
pub(crate) const CONTROLLER_LEGAL_MOVE_RING_SCHEMA_VERSION: &str =
    "controller_legal_move_ring_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionValueConstraint {
    pub path: String,
    pub value_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_inclusive: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_inclusive: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionGroup {
    pub group_id: String,
    pub family: String,
    pub controller_status: String,
    pub canary_requirement: String,
    pub patch_paths: Vec<String>,
    pub targets: Vec<String>,
    pub value_constraints: Vec<AllowedActionValueConstraint>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionFamily {
    pub family: String,
    pub controller_status: String,
    pub group_ids: Vec<String>,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControllerActionFamilyRiskProfile {
    pub family: String,
    pub likely_human_risk: String,
    pub tolerated_non_human_risk: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionsSurface {
    pub schema_version: String,
    pub write_surface: String,
    pub proposal_mode: String,
    pub groups: Vec<AllowedActionGroup>,
    pub families: Vec<AllowedActionFamily>,
    pub allowed_group_ids: Vec<String>,
    pub manual_only_group_ids: Vec<String>,
    pub forbidden_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControllerLegalMoveRingSurface {
    pub schema_version: String,
    pub controller_mutability_schema_version: String,
    pub allowed_actions_schema_version: String,
    pub write_surface: String,
    pub proposal_mode: String,
    pub legal_ring: String,
    pub non_legal_rings: Vec<String>,
    pub controller_tunable_group_ids: Vec<String>,
    pub controller_tunable_family_ids: Vec<String>,
    pub review_posture: String,
    pub note: String,
}

fn build_value_constraint(
    definition: &AllowedActionValueConstraintDefinition,
) -> AllowedActionValueConstraint {
    AllowedActionValueConstraint {
        path: definition.path.to_string(),
        value_kind: definition.value_kind.to_string(),
        min_inclusive: definition.min_inclusive,
        max_inclusive: definition.max_inclusive,
        allowed_values: definition
            .allowed_values
            .iter()
            .map(|value| value.to_string())
            .collect(),
        rule: definition.rule.map(|rule| rule.to_string()),
    }
}

fn build_group(definition: &AllowedActionGroupDefinition) -> AllowedActionGroup {
    AllowedActionGroup {
        group_id: definition.group_id.to_string(),
        family: definition.family.to_string(),
        controller_status: allowed_actions_status_for_admin_config_paths(definition.patch_paths)
            .expect("allowed-actions groups must resolve through the canonical mutability policy")
            .to_string(),
        canary_requirement: definition.canary_requirement.to_string(),
        patch_paths: definition
            .patch_paths
            .iter()
            .map(|path| path.to_string())
            .collect(),
        targets: definition
            .targets
            .iter()
            .map(|target| target.to_string())
            .collect(),
        value_constraints: definition
            .value_constraints
            .iter()
            .map(build_value_constraint)
            .collect(),
        note: definition.note.to_string(),
    }
}

pub(crate) fn controller_config_family_for_patch_key(key: &str) -> Option<&'static str> {
    ALLOWED_ACTION_GROUP_DEFINITIONS
        .iter()
        .find(|definition| definition.patch_paths.contains(&key))
        .map(|definition| definition.family)
}

pub(crate) fn controller_action_family_targets(family: &str) -> Vec<String> {
    let mut targets = BTreeMap::<String, ()>::new();
    for definition in ALLOWED_ACTION_GROUP_DEFINITIONS
        .iter()
        .filter(|definition| definition.family == family)
    {
        for target in definition.targets {
            targets.insert((*target).to_string(), ());
        }
    }
    targets.into_keys().collect()
}

pub(crate) fn controller_action_family_risk_profile(
    family: &str,
) -> Option<ControllerActionFamilyRiskProfile> {
    let (likely_human_risk, tolerated_non_human_risk, note) = match family {
        "fingerprint_signal" => (
            "low",
            "low",
            "Tightens passive fingerprint thresholds before more human-visible gates move.",
        ),
        "cdp_detection" => (
            "low",
            "low",
            "Tightens browser-automation detection using bounded rollout and threshold controls.",
        ),
        "proof_of_work" => (
            "high",
            "medium",
            "Raises attacker work cost but directly increases challenge burden for legitimate traffic.",
        ),
        "challenge" => (
            "high",
            "medium",
            "Moves interactive challenge posture and is therefore more human-visible than passive signals.",
        ),
        "not_a_bot" => (
            "medium",
            "medium",
            "Moves low-friction verification posture but still affects borderline likely-human traffic.",
        ),
        "maze_core" => (
            "high",
            "high",
            "Escalates enforcement posture and should be used after lower-friction signal families.",
        ),
        "core_policy" => (
            "high",
            "high",
            "JS-required posture is broad and human-visible, so it belongs late in the move order.",
        ),
        "botness" => (
            "medium",
            "medium",
            "Botness threshold moves reshape multiple downstream defenses and should follow lower-cost signals.",
        ),
        _ => return None,
    };

    Some(ControllerActionFamilyRiskProfile {
        family: family.to_string(),
        likely_human_risk: likely_human_risk.to_string(),
        tolerated_non_human_risk: tolerated_non_human_risk.to_string(),
        note: note.to_string(),
    })
}

pub(crate) fn allowed_actions_v1() -> AllowedActionsSurface {
    let groups = ALLOWED_ACTION_GROUP_DEFINITIONS
        .iter()
        .map(build_group)
        .collect::<Vec<_>>();
    let allowed_group_ids = group_ids_with_status(groups.as_slice(), "allowed");
    let manual_only_group_ids = group_ids_with_status(groups.as_slice(), "manual_only");
    let forbidden_group_ids = group_ids_with_status(groups.as_slice(), "forbidden");
    let families = build_family_summaries(groups.as_slice(), controller_action_family_targets);

    AllowedActionsSurface {
        schema_version: ALLOWED_ACTIONS_SCHEMA_VERSION.to_string(),
        write_surface: "admin_config".to_string(),
        proposal_mode: "config_diff_only".to_string(),
        groups,
        families,
        allowed_group_ids,
        manual_only_group_ids,
        forbidden_group_ids,
    }
}

pub(crate) fn controller_legal_move_ring_v1() -> ControllerLegalMoveRingSurface {
    let allowed_actions = allowed_actions_v1();
    let controller_tunable_family_ids = allowed_actions
        .families
        .iter()
        .filter(|family| family.controller_status == "allowed")
        .map(|family| family.family.clone())
        .collect();

    ControllerLegalMoveRingSurface {
        schema_version: CONTROLLER_LEGAL_MOVE_RING_SCHEMA_VERSION.to_string(),
        controller_mutability_schema_version: CONTROLLER_MUTABILITY_SCHEMA_VERSION.to_string(),
        allowed_actions_schema_version: allowed_actions.schema_version.clone(),
        write_surface: allowed_actions.write_surface,
        proposal_mode: allowed_actions.proposal_mode,
        legal_ring: "controller_tunable".to_string(),
        non_legal_rings: vec!["manual_only".to_string(), "never".to_string()],
        controller_tunable_group_ids: allowed_actions.allowed_group_ids,
        controller_tunable_family_ids,
        review_posture: "manual_review_required".to_string(),
        note: "The controller's legal move set is the controller-tunable ring only. Admin writability alone does not make a surface legal for the recursive-improvement game."
            .to_string(),
    }
}
