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
pub(crate) const PUBLIC_SITEMAPS_PREFIX: &str = "/sitemaps";
pub(crate) const PUBLIC_ABOUT_PATH: &str = "/about/";
pub(crate) const PUBLIC_RESEARCH_PATH: &str = "/research/";
pub(crate) const PUBLIC_PLANS_PATH: &str = "/plans/";
pub(crate) const PUBLIC_WORK_PATH: &str = "/work/";
pub(crate) const PUBLIC_PAGE_PREFIX: &str = "/page";

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

pub(crate) fn is_generated_public_site_path(path: &str) -> bool {
    matches!(
        path,
        PUBLIC_ROOT_PATH
            | PUBLIC_ROBOTS_TXT_PATH
            | PUBLIC_SITEMAP_XML_PATH
            | PUBLIC_ATOM_FEED_PATH
    ) || path_matches_prefix_boundary(path, PUBLIC_ABOUT_PATH.trim_end_matches('/'))
        || path_matches_prefix_boundary(path, PUBLIC_RESEARCH_PATH.trim_end_matches('/'))
        || path_matches_prefix_boundary(path, PUBLIC_PLANS_PATH.trim_end_matches('/'))
        || path_matches_prefix_boundary(path, PUBLIC_WORK_PATH.trim_end_matches('/'))
        || path_matches_prefix_boundary(path, PUBLIC_PAGE_PREFIX)
        || path_matches_prefix_boundary(path, PUBLIC_SITEMAPS_PREFIX)
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
            PUBLIC_ABOUT_PATH,
            PUBLIC_RESEARCH_PATH,
            PUBLIC_PLANS_PATH,
            PUBLIC_WORK_PATH,
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

    #[test]
    fn generated_public_site_matcher_covers_root_and_section_paths() {
        for path in [
            "/",
            "/about/",
            "/research/",
            "/research/2026-03-31-note/",
            "/plans/",
            "/work/",
            "/page/2/",
            "/sitemaps/pages.xml",
            "/atom.xml",
            "/sitemap.xml",
            "/robots.txt",
        ] {
            assert!(is_generated_public_site_path(path), "expected {path} to match");
        }
        for path in ["/admin/config", "/dashboard", "/health", "/metrics", "/challenge/puzzle"] {
            assert!(
                !is_generated_public_site_path(path),
                "did not expect {path} to match"
            );
        }
    }
}
