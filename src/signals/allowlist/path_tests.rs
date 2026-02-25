use super::*;

#[test]
fn test_exact_path_match() {
    let wl = vec!["/webhook/stripe".to_string()];
    assert!(is_path_allowlisted("/webhook/stripe", &wl));
    assert!(!is_path_allowlisted("/webhook/stripe2", &wl));
}

#[test]
fn test_prefix_wildcard_match() {
    let wl = vec!["/api/integration/*".to_string()];
    assert!(is_path_allowlisted("/api/integration/foo", &wl));
    assert!(is_path_allowlisted("/api/integration/bar/baz", &wl));
    assert!(!is_path_allowlisted("/api/other", &wl));
}

#[test]
fn test_inline_comment_and_whitespace() {
    let wl = vec![
        "/hook/* # trusted hooks".to_string(),
        "  /public/*   # public apis ".to_string(),
    ];
    assert!(is_path_allowlisted("/hook/abc", &wl));
    assert!(is_path_allowlisted("/public/xyz", &wl));
    assert!(!is_path_allowlisted("/private/xyz", &wl));
}

#[test]
fn test_empty_and_comment_only_lines() {
    let wl = vec![
        "# just a comment".to_string(),
        "   ".to_string(),
        "/foo/*".to_string(),
    ];
    assert!(is_path_allowlisted("/foo/bar", &wl));
    assert!(!is_path_allowlisted("/bar/foo", &wl));
}
