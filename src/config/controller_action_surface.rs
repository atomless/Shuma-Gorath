use super::controller_action_catalog::{
    AllowedActionGroupDefinition, AllowedActionValueConstraintDefinition,
    ALLOWED_ACTION_GROUP_DEFINITIONS,
    CONTROLLER_TUNABLE_ADMIN_CONFIG_PATHS, MANUAL_ONLY_ADMIN_CONFIG_PATHS,
    NEVER_ADMIN_CONFIG_PATHS, NEVER_OPERATOR_OBJECTIVES_PATHS,
};
use super::controller_action_guardrails::{build_family_summaries, group_ids_with_status};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const ALLOWED_ACTIONS_SCHEMA_VERSION: &str = "allowed_actions_v1";
pub(crate) const CONTROLLER_MUTABILITY_SCHEMA_VERSION: &str = "controller_mutability_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControllerMutabilityPath {
    pub path: String,
    pub mutability: String,
}

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
    pub controller_mutability: String,
    pub auto_proposal_status: String,
    pub canary_requirement: String,
    pub patch_paths: Vec<String>,
    pub proposable_patch_paths: Vec<String>,
    pub targets: Vec<String>,
    pub value_constraints: Vec<AllowedActionValueConstraint>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionFamily {
    pub family: String,
    pub controller_status: String,
    pub controller_mutability: String,
    pub auto_proposal_status: String,
    pub group_ids: Vec<String>,
    pub proposable_patch_paths: Vec<String>,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct AllowedActionsSurface {
    pub schema_version: String,
    pub write_surface: String,
    pub proposal_mode: String,
    pub controller_mutability_schema_version: String,
    pub groups: Vec<AllowedActionGroup>,
    pub families: Vec<AllowedActionFamily>,
    pub allowed_group_ids: Vec<String>,
    pub manual_only_group_ids: Vec<String>,
    pub forbidden_group_ids: Vec<String>,
    pub admin_config_path_mutability: Vec<ControllerMutabilityPath>,
    pub operator_objectives_path_mutability: Vec<ControllerMutabilityPath>,
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

fn build_mutability_paths(
    paths: &[&str],
    mutability: &str,
) -> Vec<ControllerMutabilityPath> {
    paths.iter()
        .map(|path| ControllerMutabilityPath {
            path: (*path).to_string(),
            mutability: mutability.to_string(),
        })
        .collect()
}

fn build_admin_config_path_mutability() -> Vec<ControllerMutabilityPath> {
    let mut entries = Vec::new();
    entries.extend(build_mutability_paths(
        CONTROLLER_TUNABLE_ADMIN_CONFIG_PATHS,
        "controller_tunable",
    ));
    entries.extend(build_mutability_paths(
        MANUAL_ONLY_ADMIN_CONFIG_PATHS,
        "manual_only",
    ));
    entries.extend(build_mutability_paths(NEVER_ADMIN_CONFIG_PATHS, "never"));
    entries
}

fn build_operator_objectives_path_mutability() -> Vec<ControllerMutabilityPath> {
    build_mutability_paths(NEVER_OPERATOR_OBJECTIVES_PATHS, "never")
}

fn definition_path_mutability(
    definition: &AllowedActionGroupDefinition,
    admin_config_path_mutability: &[ControllerMutabilityPath],
) -> String {
    let matched = admin_config_path_mutability
        .iter()
        .filter(|entry| {
            definition.patch_paths.iter().any(|patch_path| {
                entry.path == *patch_path
                    || entry
                        .path
                        .starts_with(format!("{patch_path}.").as_str())
            })
        })
        .map(|entry| entry.mutability.clone())
        .collect::<BTreeSet<_>>();
    if matched.is_empty() {
        "unknown".to_string()
    } else if matched.len() == 1 {
        matched.into_iter().next().unwrap_or_default()
    } else {
        "mixed".to_string()
    }
}

fn build_group(
    definition: &AllowedActionGroupDefinition,
    admin_config_path_mutability: &[ControllerMutabilityPath],
) -> AllowedActionGroup {
    let controller_mutability = definition_path_mutability(
        definition,
        admin_config_path_mutability,
    );
    let auto_proposal_status = proposal_support_status(
        controller_mutability.as_str(),
        definition.patch_paths,
        definition.proposable_patch_paths,
    );
    AllowedActionGroup {
        group_id: definition.group_id.to_string(),
        family: definition.family.to_string(),
        controller_status: definition.controller_status.to_string(),
        controller_mutability,
        auto_proposal_status,
        canary_requirement: definition.canary_requirement.to_string(),
        patch_paths: definition
            .patch_paths
            .iter()
            .map(|path| path.to_string())
            .collect(),
        proposable_patch_paths: definition
            .proposable_patch_paths
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

fn proposal_support_status(
    controller_mutability: &str,
    patch_paths: &[&str],
    proposable_patch_paths: &[&str],
) -> String {
    if controller_mutability != "controller_tunable" {
        return "not_applicable".to_string();
    }

    if proposable_patch_paths.is_empty() {
        "not_supported".to_string()
    } else if proposable_patch_paths.len() == patch_paths.len()
        && proposable_patch_paths
            .iter()
            .all(|path| patch_paths.contains(path))
    {
        "supported".to_string()
    } else {
        "partial_support".to_string()
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
    let admin_config_path_mutability = build_admin_config_path_mutability();
    let operator_objectives_path_mutability = build_operator_objectives_path_mutability();
    let groups = ALLOWED_ACTION_GROUP_DEFINITIONS
        .iter()
        .map(|definition| build_group(definition, admin_config_path_mutability.as_slice()))
        .collect::<Vec<_>>();
    let allowed_group_ids = group_ids_with_status(groups.as_slice(), "allowed");
    let manual_only_group_ids = group_ids_with_status(groups.as_slice(), "manual_only");
    let forbidden_group_ids = group_ids_with_status(groups.as_slice(), "forbidden");
    let families = build_family_summaries(groups.as_slice(), controller_action_family_targets);

    AllowedActionsSurface {
        schema_version: ALLOWED_ACTIONS_SCHEMA_VERSION.to_string(),
        write_surface: "admin_config".to_string(),
        proposal_mode: "config_diff_only".to_string(),
        controller_mutability_schema_version: CONTROLLER_MUTABILITY_SCHEMA_VERSION.to_string(),
        groups,
        families,
        allowed_group_ids,
        manual_only_group_ids,
        forbidden_group_ids,
        admin_config_path_mutability,
        operator_objectives_path_mutability,
    }
}
