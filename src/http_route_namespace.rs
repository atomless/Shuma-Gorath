#[allow(dead_code)] // Declared now so later route-migration tranches reuse one canonical prefix.
pub(crate) const SHUMA_PREFIX: &str = "/shuma";
pub(crate) const SHUMA_ADMIN_PREFIX: &str = "/shuma/admin";
#[allow(dead_code)] // Declared now so later dashboard migration does not invent a second path contract.
pub(crate) const SHUMA_DASHBOARD_PREFIX: &str = "/shuma/dashboard";
#[allow(dead_code)] // Declared now so later dashboard migration does not invent a second path contract.
pub(crate) const SHUMA_DASHBOARD_ROOT_PATH: &str = "/shuma/dashboard";
#[allow(dead_code)] // Declared now so later dashboard migration does not invent a second path contract.
pub(crate) const SHUMA_DASHBOARD_ROOT_PATH_WITH_SLASH: &str = "/shuma/dashboard/";
#[allow(dead_code)] // Declared now so later dashboard migration does not invent a second path contract.
pub(crate) const SHUMA_DASHBOARD_INDEX_PATH: &str = "/shuma/dashboard/index.html";
#[allow(dead_code)] // Declared now so later dashboard migration does not invent a second path contract.
pub(crate) const SHUMA_DASHBOARD_LOGIN_PATH: &str = "/shuma/dashboard/login.html";
#[allow(dead_code)] // Declared now so later internal-route migration does not invent a second path contract.
pub(crate) const SHUMA_INTERNAL_PREFIX: &str = "/shuma/internal";
pub(crate) const SHUMA_HEALTH_PATH: &str = "/shuma/health";
pub(crate) const SHUMA_METRICS_PATH: &str = "/shuma/metrics";

#[allow(dead_code)] // Declared now so later public-root site migration reuses one canonical root contract.
pub(crate) const PUBLIC_ROOT_PATH: &str = "/";
pub(crate) const PUBLIC_ROBOTS_TXT_PATH: &str = "/robots.txt";
#[allow(dead_code)] // Declared now so later public-root site migration reuses one canonical sitemap path.
pub(crate) const PUBLIC_SITEMAP_XML_PATH: &str = "/sitemap.xml";
#[allow(dead_code)] // Declared now so later public-root site migration reuses one canonical feed path.
pub(crate) const PUBLIC_ATOM_FEED_PATH: &str = "/atom.xml";

fn path_matches_prefix_boundary(path: &str, prefix: &str) -> bool {
    path == prefix
        || path
            .strip_prefix(prefix)
            .map(|remainder| remainder.starts_with('/'))
            .unwrap_or(false)
}

pub(crate) fn is_shuma_admin_path(path: &str) -> bool {
    path_matches_prefix_boundary(path, SHUMA_ADMIN_PREFIX)
}

#[allow(dead_code)] // Becomes live once dashboard routes move under /shuma/*.
pub(crate) fn is_shuma_dashboard_root_path(path: &str) -> bool {
    matches!(
        path,
        SHUMA_DASHBOARD_ROOT_PATH | SHUMA_DASHBOARD_ROOT_PATH_WITH_SLASH
    )
}

#[allow(dead_code)] // Becomes live once internal routes move under /shuma/internal/*.
pub(crate) fn is_shuma_internal_path(path: &str) -> bool {
    path_matches_prefix_boundary(path, SHUMA_INTERNAL_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_shuma_owned_routes_live_under_shuma_prefix() {
        for path in [
            SHUMA_ADMIN_PREFIX,
            SHUMA_DASHBOARD_PREFIX,
            SHUMA_DASHBOARD_INDEX_PATH,
            SHUMA_DASHBOARD_LOGIN_PATH,
            SHUMA_INTERNAL_PREFIX,
            SHUMA_HEALTH_PATH,
            SHUMA_METRICS_PATH,
        ] {
            assert!(
                path.starts_with(SHUMA_PREFIX),
                "expected {path} to live under {SHUMA_PREFIX}"
            );
        }

        for path in [
            PUBLIC_ROOT_PATH,
            PUBLIC_ROBOTS_TXT_PATH,
            PUBLIC_SITEMAP_XML_PATH,
            PUBLIC_ATOM_FEED_PATH,
        ] {
            assert!(
                !path.starts_with(SHUMA_PREFIX),
                "expected public path {path} to remain outside {SHUMA_PREFIX}"
            );
        }
    }

    #[test]
    fn shuma_route_matchers_are_boundary_aware() {
        assert!(is_shuma_admin_path("/shuma/admin"));
        assert!(is_shuma_admin_path("/shuma/admin/config"));
        assert!(!is_shuma_admin_path("/shuma/administrator"));

        assert!(is_shuma_dashboard_root_path("/shuma/dashboard"));
        assert!(is_shuma_dashboard_root_path("/shuma/dashboard/"));
        assert!(!is_shuma_dashboard_root_path("/shuma/dashboard/index.html"));

        assert!(is_shuma_internal_path("/shuma/internal"));
        assert!(is_shuma_internal_path("/shuma/internal/oversight/agent/run"));
        assert!(!is_shuma_internal_path("/shuma/internals"));
    }
}
