use once_cell::sync::Lazy;

const ROUTE_NAMESPACE_PREFIX: &str = "/_/";
const ROUTE_SEGMENT_LEN: usize = 12;
const ROUTE_DERIVE_LABEL: &str = "maze-route-v1";

static ROUTE_SEGMENT: Lazy<String> = Lazy::new(|| {
    let seed = format!("{}::{}", ROUTE_DERIVE_LABEL, super::token::secret_from_env());
    super::token::digest(seed.as_str())
        .chars()
        .take(ROUTE_SEGMENT_LEN)
        .collect()
});

static MAZE_PATH_PREFIX: Lazy<String> =
    Lazy::new(|| format!("{}{}/", ROUTE_NAMESPACE_PREFIX, ROUTE_SEGMENT.as_str()));
static MAZE_ROOT_PATH: Lazy<String> = Lazy::new(|| MAZE_PATH_PREFIX.trim_end_matches('/').to_string());
static MAZE_CHECKPOINT_PATH: Lazy<String> =
    Lazy::new(|| format!("{}checkpoint", MAZE_PATH_PREFIX.as_str()));
static MAZE_ISSUE_LINKS_PATH: Lazy<String> =
    Lazy::new(|| format!("{}issue-links", MAZE_PATH_PREFIX.as_str()));
static MAZE_ASSETS_PREFIX: Lazy<String> =
    Lazy::new(|| format!("{}/assets", MAZE_ROOT_PATH.as_str()));

pub fn path_prefix() -> &'static str {
    MAZE_PATH_PREFIX.as_str()
}

pub fn root_path() -> &'static str {
    MAZE_ROOT_PATH.as_str()
}

pub fn checkpoint_path() -> &'static str {
    MAZE_CHECKPOINT_PATH.as_str()
}

pub fn issue_links_path() -> &'static str {
    MAZE_ISSUE_LINKS_PATH.as_str()
}

pub fn assets_prefix() -> &'static str {
    MAZE_ASSETS_PREFIX.as_str()
}

pub fn entry_path(suffix: &str) -> String {
    format!("{}{}", path_prefix(), suffix.trim_start_matches('/'))
}

pub fn default_preview_path() -> String {
    entry_path("preview")
}

/// Check whether a request path targets a maze entry point.
pub fn is_maze_path(path: &str) -> bool {
    if path == root_path() || path == path_prefix() {
        return true;
    }
    if !path.starts_with(path_prefix()) {
        return false;
    }
    let suffix = &path[path_prefix().len()..];
    if suffix.is_empty() || suffix.starts_with('/') {
        return false;
    }
    if suffix.contains("//") || suffix.contains("..") {
        return false;
    }
    suffix
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.' | '~'))
}

#[cfg(test)]
mod tests {
    use super::{
        assets_prefix, checkpoint_path, entry_path, is_maze_path, issue_links_path, path_prefix,
        root_path,
    };

    #[test]
    fn maze_path_helpers_are_accepted() {
        assert!(is_maze_path(root_path()));
        assert!(is_maze_path(path_prefix()));
        assert!(is_maze_path(entry_path("entry-segment").as_str()));
        assert!(is_maze_path(checkpoint_path()));
        assert!(is_maze_path(issue_links_path()));
        assert!(is_maze_path(
            format!("{}/maze.abcd123.min.css", assets_prefix()).as_str()
        ));
    }

    #[test]
    fn legacy_named_routes_are_rejected() {
        assert!(!is_maze_path("/maze/legacy-entry"));
        assert!(!is_maze_path("/maze/checkpoint"));
        assert!(!is_maze_path("/maze/issue-links"));
        assert!(!is_maze_path("/trap/legacy-entry"));
    }

    #[test]
    fn malformed_prefix_variants_are_rejected() {
        assert!(!is_maze_path(format!("{}{}", path_prefix(), "/double-slash").as_str()));
        assert!(!is_maze_path(format!("{}{}", path_prefix(), "invalid..segment").as_str()));
        assert!(!is_maze_path(format!("{}{}", path_prefix(), "bad%2Fencoding").as_str()));
        assert!(!is_maze_path("/_/deadbeef1234/segment"));
    }
}
