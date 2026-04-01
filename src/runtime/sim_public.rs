use std::{
    fs,
    path::{Path, PathBuf},
};

use spin_sdk::http::{Method, Request, Response};

use crate::{
    config::{Config, RuntimeEnvironment},
    http_route_namespace,
};

const SIM_PUBLIC_SITE_DIRNAME: &str = "sim-public-site";
const SIM_PUBLIC_SITE_MANIFEST_FILENAME: &str = "manifest.json";
const SIM_PUBLIC_SITE_CONTENT_DIRNAME: &str = "site";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SimPublicAvailability {
    pub runtime_environment: RuntimeEnvironment,
    pub artifact_available: bool,
}

impl SimPublicAvailability {
    pub(crate) fn is_enabled(self) -> bool {
        sim_public_enabled(self.runtime_environment, self.artifact_available)
    }
}

pub(crate) fn sim_public_enabled(
    _runtime_environment: RuntimeEnvironment,
    artifact_available: bool,
) -> bool {
    artifact_available
}

pub(crate) fn sim_public_site_root() -> PathBuf {
    let local_state_dir = crate::config::runtime_var_trimmed_optional("SHUMA_LOCAL_STATE_DIR")
        .unwrap_or_else(|| ".shuma".to_string());
    PathBuf::from(local_state_dir).join(SIM_PUBLIC_SITE_DIRNAME)
}

pub(crate) fn sim_public_site_manifest_path() -> PathBuf {
    sim_public_site_root().join(SIM_PUBLIC_SITE_MANIFEST_FILENAME)
}

fn sim_public_site_content_root() -> PathBuf {
    sim_public_site_root().join(SIM_PUBLIC_SITE_CONTENT_DIRNAME)
}

fn sim_public_site_artifact_available() -> bool {
    sim_public_site_manifest_path().is_file()
}

pub(crate) fn availability_from_runtime(cfg: &Config) -> SimPublicAvailability {
    let _ = cfg;
    SimPublicAvailability {
        runtime_environment: crate::config::runtime_environment(),
        artifact_available: sim_public_site_artifact_available(),
    }
}

pub(crate) fn maybe_handle(req: &Request, path: &str, cfg: &Config) -> Option<Response> {
    maybe_handle_with_availability(req, path, availability_from_runtime(cfg))
}

pub(crate) fn maybe_handle_without_config(req: &Request, path: &str) -> Option<Response> {
    maybe_handle_with_availability(
        req,
        path,
        SimPublicAvailability {
            runtime_environment: crate::config::runtime_environment(),
            artifact_available: sim_public_site_artifact_available(),
        },
    )
}

pub(crate) fn maybe_handle_with_availability(
    req: &Request,
    path: &str,
    availability: SimPublicAvailability,
) -> Option<Response> {
    let Some(relative_asset_path) = sim_public_relative_asset_path_for_mode(
        path,
        crate::config::local_contributor_ingress_enabled(),
    ) else {
        return None;
    };
    if !availability.is_enabled() {
        return Some(Response::new(404, "Not Found"));
    }
    if !matches!(req.method(), Method::Get | Method::Head) {
        return Some(Response::new(405, "Method Not Allowed"));
    }

    let asset_path = sim_public_site_content_root().join(relative_asset_path);
    match fs::read(&asset_path) {
        Ok(body) => Some(render_asset_response(
            &asset_path,
            if *req.method() == Method::Head {
                Vec::new()
            } else {
                body
            },
        )),
        Err(_) => Some(Response::new(404, "Not Found")),
    }
}

fn sim_public_relative_asset_path_for_mode(
    path: &str,
    local_contributor_ingress_enabled: bool,
) -> Option<PathBuf> {
    let normalized_path = normalize_request_path(path);
    if !should_claim_public_root_path(normalized_path, local_contributor_ingress_enabled) {
        return None;
    }

    let trimmed = normalized_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return Some(PathBuf::from("index.html"));
    }

    let mut relative = PathBuf::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() {
            continue;
        }
        if segment == "." || segment == ".." || segment.contains('\\') {
            return Some(PathBuf::from("__invalid__"));
        }
        relative.push(segment);
    }

    if normalized_path.ends_with('/') || relative.extension().is_none() {
        relative.push("index.html");
    }

    Some(relative)
}

fn should_claim_public_root_path(path: &str, local_contributor_ingress_enabled: bool) -> bool {
    if http_route_namespace::is_generated_public_site_path(path) {
        return true;
    }
    if !local_contributor_ingress_enabled {
        return false;
    }
    !path.starts_with(http_route_namespace::SHUMA_PREFIX)
}

fn normalize_request_path(path: &str) -> &str {
    path.split('?')
        .next()
        .unwrap_or(path)
        .split('#')
        .next()
        .unwrap_or(path)
}

fn render_asset_response(asset_path: &Path, body: Vec<u8>) -> Response {
    Response::builder()
        .status(200)
        .header("Content-Type", content_type_for_path(asset_path))
        .header("Cache-Control", "no-store, max-age=0, must-revalidate")
        .body(body)
        .build()
}

fn content_type_for_path(path: &Path) -> &'static str {
    let file_name = path.file_name().and_then(|value| value.to_str());
    match path.extension().and_then(|value| value.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("xml") if file_name == Some("atom.xml") => "application/atom+xml; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: Method, path: &str) -> Request {
        let mut builder = Request::builder();
        builder.method(method).uri(path);
        builder.body(Vec::new());
        builder.build()
    }

    fn enabled_availability() -> SimPublicAvailability {
        SimPublicAvailability {
            runtime_environment: RuntimeEnvironment::RuntimeDev,
            artifact_available: true,
        }
    }

    fn header_value(resp: &Response, name: &str) -> Option<String> {
        resp.headers()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
            .and_then(|(_, value)| value.as_str().map(str::to_string))
    }

    fn seeded_site_root(base: &Path) -> PathBuf {
        base.join(".shuma")
            .join("sim-public-site")
            .join("site")
    }

    fn seed_generated_site(base: &Path) -> PathBuf {
        let site_root = seeded_site_root(base);
        fs::create_dir_all(site_root.join("about")).expect("about dir should be created");
        fs::create_dir_all(site_root.join("research").join("alpha")).expect("entry dir");
        fs::create_dir_all(site_root.join("sitemaps")).expect("sitemaps dir");
        fs::write(
            base.join(".shuma").join("sim-public-site").join("manifest.json"),
            "{}\n",
        )
        .expect("manifest");
        fs::write(site_root.join("index.html"), "<html><main>Latest</main></html>\n").expect("root html");
        fs::write(site_root.join("about").join("index.html"), "<html><main>About</main></html>\n")
            .expect("about html");
        fs::write(
            site_root.join("research").join("alpha").join("index.html"),
            "<html><main>Alpha Research</main></html>\n",
        )
        .expect("entry html");
        fs::write(site_root.join("atom.xml"), "<feed>Alpha Research</feed>\n").expect("atom");
        fs::write(
            site_root.join("robots.txt"),
            "User-agent: *\nAllow: /\nSitemap: http://127.0.0.1:3000/sitemap.xml\n",
        )
        .expect("robots");
        fs::write(
            site_root.join("sitemap.xml"),
            "<?xml version=\"1.0\" encoding=\"utf-8\"?><sitemapindex></sitemapindex>\n",
        )
        .expect("sitemap");
        site_root
    }

    #[test]
    fn sim_public_relative_asset_path_maps_root_hosted_routes() {
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/", false),
            Some(PathBuf::from("index.html"))
        );
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/about/", false),
            Some(PathBuf::from("about").join("index.html"))
        );
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/about", false),
            Some(PathBuf::from("about").join("index.html"))
        );
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/atom.xml", false),
            Some(PathBuf::from("atom.xml"))
        );
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/robots.txt", false),
            Some(PathBuf::from("robots.txt"))
        );
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/sitemap.xml", false),
            Some(PathBuf::from("sitemap.xml"))
        );
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/research/alpha/", false),
            Some(PathBuf::from("research").join("alpha").join("index.html"))
        );
    }

    #[test]
    fn sim_public_relative_asset_path_rejects_shuma_control_paths() {
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/shuma/health", false),
            None
        );
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/shuma/health", true),
            None
        );
    }

    #[test]
    fn sim_public_relative_asset_path_claims_unknown_public_paths_for_local_contributor_mode() {
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/favicon.ico", true),
            Some(PathBuf::from("favicon.ico"))
        );
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/totally-unlisted", true),
            Some(PathBuf::from("totally-unlisted").join("index.html"))
        );
        assert_eq!(
            sim_public_relative_asset_path_for_mode("/totally-unlisted", false),
            None
        );
    }

    #[test]
    fn sim_public_enabled_requires_generated_artifact_presence() {
        assert!(sim_public_enabled(RuntimeEnvironment::RuntimeDev, true));
        assert!(sim_public_enabled(RuntimeEnvironment::RuntimeProd, true));
        assert!(!sim_public_enabled(RuntimeEnvironment::RuntimeDev, false));
        assert!(!sim_public_enabled(RuntimeEnvironment::RuntimeProd, false));
    }

    #[test]
    fn maybe_handle_returns_none_for_non_sim_paths() {
        let req = request(Method::Get, "/shuma/health");
        assert!(
            maybe_handle_with_availability(&req, "/shuma/health", enabled_availability()).is_none()
        );
    }

    #[test]
    fn maybe_handle_returns_not_found_when_disabled() {
        let req = request(Method::Get, "/");
        let availability = SimPublicAvailability {
            runtime_environment: RuntimeEnvironment::RuntimeDev,
            artifact_available: false,
        };
        let resp = maybe_handle_with_availability(&req, "/", availability)
            .expect("sim route should be handled");
        assert_eq!(*resp.status(), 404u16);
    }

    #[test]
    fn maybe_handle_returns_local_not_found_for_unknown_public_paths_in_local_contributor_mode() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE", "true");

        let req = request(Method::Get, "/favicon.ico");
        let availability = SimPublicAvailability {
            runtime_environment: RuntimeEnvironment::RuntimeDev,
            artifact_available: false,
        };
        let resp = maybe_handle_with_availability(&req, "/favicon.ico", availability)
            .expect("local contributor mode should claim public paths");

        std::env::remove_var("SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE");
        assert_eq!(*resp.status(), 404u16);
    }

    #[test]
    fn sim_public_site_root_defaults_under_shuma_local_state_dir() {
        let _lock = crate::test_support::lock_env();
        std::env::remove_var("SHUMA_LOCAL_STATE_DIR");

        let root = sim_public_site_root();

        assert_eq!(root, PathBuf::from(".shuma").join("sim-public-site"));
    }

    #[test]
    fn sim_public_site_root_honors_shuma_local_state_dir_override() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_LOCAL_STATE_DIR", "/tmp/shuma-state");

        let root = sim_public_site_root();

        std::env::remove_var("SHUMA_LOCAL_STATE_DIR");
        assert_eq!(root, PathBuf::from("/tmp/shuma-state").join("sim-public-site"));
    }

    #[test]
    fn availability_from_runtime_uses_generated_artifact_presence_not_sim_controls() {
        let _lock = crate::test_support::lock_env();
        let base = std::env::temp_dir().join(format!("sim-public-contract-{}", std::process::id()));
        let local_state_dir = base.join(".shuma");
        let artifact_root = local_state_dir.join("sim-public-site");
        let manifest_path = artifact_root.join("manifest.json");
        fs::create_dir_all(&artifact_root).expect("artifact root should be creatable");
        fs::write(&manifest_path, "{}\n").expect("manifest should be writable");
        std::env::set_var("SHUMA_LOCAL_STATE_DIR", &local_state_dir);

        let mut cfg = crate::config::default_seeded_config();
        cfg.adversary_sim_enabled = false;
        let availability = availability_from_runtime(&cfg);

        std::env::remove_var("SHUMA_LOCAL_STATE_DIR");
        let _ = fs::remove_file(&manifest_path);
        let _ = fs::remove_dir_all(&base);

        assert!(availability.artifact_available);
        assert!(availability.is_enabled());
    }

    #[test]
    fn maybe_handle_rejects_non_get_head_methods() {
        let req = request(Method::Post, "/about/");
        let resp = maybe_handle_with_availability(&req, "/about/", enabled_availability())
            .expect("sim route should be handled");
        assert_eq!(*resp.status(), 405u16);
    }

    #[test]
    fn maybe_handle_serves_generated_site_files_when_enabled() {
        let _lock = crate::test_support::lock_env();
        let base = std::env::temp_dir().join(format!("sim-public-serving-{}", std::process::id()));
        seed_generated_site(&base);
        std::env::set_var("SHUMA_LOCAL_STATE_DIR", base.join(".shuma"));

        let req = request(Method::Get, "/research/alpha/");
        let resp = maybe_handle_with_availability(&req, "/research/alpha/", enabled_availability())
            .expect("sim route should be handled");

        std::env::remove_var("SHUMA_LOCAL_STATE_DIR");
        let _ = fs::remove_dir_all(&base);

        assert_eq!(*resp.status(), 200u16);
        assert_eq!(header_value(&resp, "content-type").as_deref(), Some("text/html; charset=utf-8"));
        let body = String::from_utf8(resp.into_body()).expect("sim page should be utf-8");
        assert!(body.contains("Alpha Research"));
    }

    #[test]
    fn maybe_handle_serves_atom_feed_with_xml_content_type() {
        let _lock = crate::test_support::lock_env();
        let base = std::env::temp_dir().join(format!("sim-public-feed-{}", std::process::id()));
        seed_generated_site(&base);
        std::env::set_var("SHUMA_LOCAL_STATE_DIR", base.join(".shuma"));

        let req = request(Method::Get, "/atom.xml");
        let resp = maybe_handle_with_availability(&req, "/atom.xml", enabled_availability())
            .expect("sim route should be handled");

        std::env::remove_var("SHUMA_LOCAL_STATE_DIR");
        let _ = fs::remove_dir_all(&base);

        assert_eq!(*resp.status(), 200u16);
        assert_eq!(
            header_value(&resp, "content-type").as_deref(),
            Some("application/atom+xml; charset=utf-8")
        );
    }

    #[test]
    fn maybe_handle_serves_robots_txt_with_plain_text_content_type() {
        let _lock = crate::test_support::lock_env();
        let base = std::env::temp_dir().join(format!("sim-public-robots-{}", std::process::id()));
        seed_generated_site(&base);
        std::env::set_var("SHUMA_LOCAL_STATE_DIR", base.join(".shuma"));

        let req = request(Method::Get, "/robots.txt");
        let resp = maybe_handle_with_availability(&req, "/robots.txt", enabled_availability())
            .expect("sim route should be handled");

        std::env::remove_var("SHUMA_LOCAL_STATE_DIR");
        let _ = fs::remove_dir_all(&base);

        assert_eq!(*resp.status(), 200u16);
        assert_eq!(
            header_value(&resp, "content-type").as_deref(),
            Some("text/plain; charset=utf-8")
        );
    }

    #[test]
    fn maybe_handle_serves_sitemap_with_xml_content_type() {
        let _lock = crate::test_support::lock_env();
        let base = std::env::temp_dir().join(format!("sim-public-sitemap-{}", std::process::id()));
        seed_generated_site(&base);
        std::env::set_var("SHUMA_LOCAL_STATE_DIR", base.join(".shuma"));

        let req = request(Method::Get, "/sitemap.xml");
        let resp = maybe_handle_with_availability(&req, "/sitemap.xml", enabled_availability())
            .expect("sim route should be handled");

        std::env::remove_var("SHUMA_LOCAL_STATE_DIR");
        let _ = fs::remove_dir_all(&base);

        assert_eq!(*resp.status(), 200u16);
        assert_eq!(
            header_value(&resp, "content-type").as_deref(),
            Some("application/xml; charset=utf-8")
        );
    }

    #[test]
    fn maybe_handle_returns_empty_body_for_head() {
        let _lock = crate::test_support::lock_env();
        let base = std::env::temp_dir().join(format!("sim-public-head-{}", std::process::id()));
        seed_generated_site(&base);
        std::env::set_var("SHUMA_LOCAL_STATE_DIR", base.join(".shuma"));

        let req = request(Method::Head, "/");
        let resp =
            maybe_handle_with_availability(&req, "/", enabled_availability())
                .expect("sim route should be handled");

        std::env::remove_var("SHUMA_LOCAL_STATE_DIR");
        let _ = fs::remove_dir_all(&base);

        assert_eq!(*resp.status(), 200u16);
        assert!(resp.body().is_empty());
    }
}
