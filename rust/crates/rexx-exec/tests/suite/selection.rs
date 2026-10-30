use super::{BTreeSet, GROUPS_ENV, group_path};

/// The groups [`GROUPS_ENV`] names, each of which must be discovered, else
/// every group.
pub(super) fn selected(groups: Vec<(String, String)>, selector: Option<&str>) -> Vec<(String, String)> {
    let Some(names) = selector else {
        return groups;
    };
    let names: BTreeSet<&str> = names
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    let chosen: Vec<(String, String)> = groups
        .into_iter()
        .filter(|(dir, group)| names.contains(group_path(dir, group).as_str()))
        .collect();
    let found: BTreeSet<String> = chosen
        .iter()
        .map(|(dir, group)| group_path(dir, group))
        .collect();
    let missing: Vec<&&str> = names
        .iter()
        .filter(|name| !found.contains(**name))
        .collect();
    assert!(
        missing.is_empty(),
        "{GROUPS_ENV} names no group {missing:?}"
    );
    chosen
}

