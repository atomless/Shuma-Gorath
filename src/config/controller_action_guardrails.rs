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
            group_ids: family_groups
                .iter()
                .map(|group| group.group_id.clone())
                .collect(),
            targets: family_targets(family.as_str()),
            family,
        })
        .collect()
}

fn family_status(statuses: &[String]) -> String {
    let all_allowed = statuses.iter().all(|status| status == "allowed");
    let all_manual_only = statuses.iter().all(|status| status == "manual_only");
    let all_forbidden = statuses.iter().all(|status| status == "forbidden");
    if all_allowed {
        "allowed".to_string()
    } else if all_manual_only {
        "manual_only".to_string()
    } else if all_forbidden {
        "forbidden".to_string()
    } else {
        "mixed".to_string()
    }
}
