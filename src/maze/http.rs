/// Check whether a request path targets a maze entry point.
pub fn is_maze_path(path: &str) -> bool {
    path.starts_with("/trap/") || path.starts_with("/maze/")
}
