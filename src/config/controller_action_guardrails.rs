use std::collections::BTreeMap;

use super::controller_action_surface::{AllowedActionFamily, AllowedActionGroup};

pub(super) fn group_ids_with_status(
    groups: &[AllowedActionGroup],
    controller_status: &str,
) -> Vec<String> {
    groups
        .iter()
        .filter(|group| group.controller_status == controller_status)
        .map(|group| group.group_id.clone())
        .collect()
}

pub(super) fn build_family_summaries<F>(
    groups: &[AllowedActionGroup],
    mut family_targets: F,
) -> Vec<AllowedActionFamily>
where
    F: FnMut(&str) -> Vec<String>,
{
    let mut family_groups = BTreeMap::<String, Vec<&AllowedActionGroup>>::new();
    for group in groups {
        family_groups
            .entry(group.family.clone())
            .or_default()
            .push(group);
    }

    family_groups
        .into_iter()
        .map(|(family, family_groups)| AllowedActionFamily {
            controller_status: family_status(
                family_groups
                    .iter()
                    .map(|group| group.controller_status.clone())
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            controller_mutability: summary_value(
                family_groups
                    .iter()
                    .map(|group| group.controller_mutability.clone())
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            auto_proposal_status: family_auto_proposal_status(family_groups.as_slice()),
            group_ids: family_groups
                .iter()
                .map(|group| group.group_id.clone())
                .collect(),
            proposable_patch_paths: family_groups
                .iter()
                .flat_map(|group| group.proposable_patch_paths.iter().cloned())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
            targets: family_targets(family.as_str()),
            family,
        })
        .collect()
}

fn family_status(statuses: &[String]) -> String {
    summary_value(statuses)
}

fn summary_value(values: &[String]) -> String {
    let unique = values.iter().collect::<std::collections::BTreeSet<_>>();
    if unique.len() == 1 {
        unique
            .into_iter()
            .next()
            .cloned()
            .unwrap_or_else(|| "mixed".to_string())
    } else {
        "mixed".to_string()
    }
}

fn family_auto_proposal_status(groups: &[&AllowedActionGroup]) -> String {
    let tunable_paths = groups
        .iter()
        .filter(|group| group.controller_mutability == "controller_tunable")
        .flat_map(|group| group.patch_paths.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>();
    if tunable_paths.is_empty() {
        return "not_applicable".to_string();
    }

    let proposable_paths = groups
        .iter()
        .flat_map(|group| group.proposable_patch_paths.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>();

    if proposable_paths.is_empty() {
        "not_supported".to_string()
    } else if proposable_paths == tunable_paths {
        "supported".to_string()
    } else {
        "partial_support".to_string()
    }
}
