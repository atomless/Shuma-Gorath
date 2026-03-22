use super::controller_action_catalog::{
    AllowedActionGroupDefinition, AllowedActionValueConstraintDefinition,
    ALLOWED_ACTION_GROUP_DEFINITIONS,
};
use super::controller_action_guardrails::{build_family_summaries, group_ids_with_status};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) const ALLOWED_ACTIONS_SCHEMA_VERSION: &str = "allowed_actions_v1";

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
        controller_status: definition.controller_status.to_string(),
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
