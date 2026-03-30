use std::path::PathBuf;

use spin_sdk::http::{Method, Request, Response};

use crate::config::{Config, RuntimeEnvironment};

const SIM_PUBLIC_SITE_DIRNAME: &str = "sim-public-site";
const SIM_PUBLIC_SITE_MANIFEST_FILENAME: &str = "manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SimPublicPage {
    Landing,
    Docs,
    Pricing,
    Contact,
    Search,
}

impl SimPublicPage {
    const ALL: [Self; 5] = [
        Self::Landing,
        Self::Docs,
        Self::Pricing,
        Self::Contact,
        Self::Search,
    ];

    fn path(self) -> &'static str {
        match self {
            Self::Landing => "/sim/public/landing",
            Self::Docs => "/sim/public/docs",
            Self::Pricing => "/sim/public/pricing",
            Self::Contact => "/sim/public/contact",
            Self::Search => "/sim/public/search",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Landing => "landing",
            Self::Docs => "docs",
            Self::Pricing => "pricing",
            Self::Contact => "contact",
            Self::Search => "search",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Landing => "Sim Landing",
            Self::Docs => "Sim Docs",
            Self::Pricing => "Sim Pricing",
            Self::Contact => "Sim Contact",
            Self::Search => "Sim Search",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::Landing => "Baseline navigation landing for human-like browser sessions.",
            Self::Docs => "Reference docs endpoint for realistic crawler traversal depth.",
            Self::Pricing => "Static pricing-like endpoint used for deterministic crawl patterns.",
            Self::Contact => "Simple contact-like endpoint for mixed benign/adversarial traffic.",
            Self::Search => "Query endpoint for crawl/search traffic-shape realism.",
        }
    }
}

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

pub(crate) fn parse_page(path: &str) -> Option<SimPublicPage> {
    let normalized_path = path.split('?').next().unwrap_or(path);
    match normalized_path {
        "/sim/public/landing" => Some(SimPublicPage::Landing),
        "/sim/public/docs" => Some(SimPublicPage::Docs),
        "/sim/public/pricing" => Some(SimPublicPage::Pricing),
        "/sim/public/contact" => Some(SimPublicPage::Contact),
        "/sim/public/search" => Some(SimPublicPage::Search),
        _ => None,
    }
}

pub(crate) fn maybe_handle(req: &Request, path: &str, cfg: &Config) -> Option<Response> {
    maybe_handle_with_availability(req, path, availability_from_runtime(cfg))
}

pub(crate) fn maybe_handle_with_availability(
    req: &Request,
    path: &str,
    availability: SimPublicAvailability,
) -> Option<Response> {
    let page = parse_page(path)?;
    if !availability.is_enabled() {
        return Some(Response::new(404, "Not Found"));
    }
    if !matches!(req.method(), Method::Get | Method::Head) {
        return Some(Response::new(405, "Method Not Allowed"));
    }

    let search_query = if page == SimPublicPage::Search {
        extract_query_param(req.uri(), "q")
    } else {
        None
    };
    Some(render_page(page, search_query.as_deref(), req.method()))
}

fn render_page(page: SimPublicPage, search_query: Option<&str>, method: &Method) -> Response {
    let body = if *method == Method::Head {
        Vec::new()
    } else {
        render_html(page, search_query).into_bytes()
    };

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-store, max-age=0, must-revalidate")
        .body(body)
        .build()
}

fn render_html(page: SimPublicPage, search_query: Option<&str>) -> String {
    let query_value = search_query.map(escape_html).unwrap_or_default();
    let query_line = match search_query {
        Some(value) => format!(
            "<p>Current query: <code>{}</code></p>",
            escape_html(value)
        ),
        None => "<p>Current query: <code>(none)</code></p>".to_string(),
    };

    let search_block = if page == SimPublicPage::Search {
        format!(
            "<form action=\"/sim/public/search\" method=\"get\">\
             <label for=\"sim-q\">Search query</label>\
             <input id=\"sim-q\" name=\"q\" value=\"{}\" />\
             <button type=\"submit\">Search</button>\
             </form>{}",
            query_value, query_line
        )
    } else {
        String::new()
    };

    format!(
        "<!doctype html>\
         <html lang=\"en\">\
         <head>\
         <meta charset=\"utf-8\">\
         <title>{}</title>\
         <meta name=\"robots\" content=\"noindex,nofollow\">\
         </head>\
         <body>\
         <main>\
         <h1>{}</h1>\
         <p>{}</p>\
         <nav>{}</nav>\
         <section>{}</section>\
         {}\
         <p>Simulation crawl graph seed: \
         <a href=\"/sim/public/landing\">landing</a> \
         <a href=\"/sim/public/docs\">docs</a> \
         <a href=\"/sim/public/pricing\">pricing</a> \
         <a href=\"/sim/public/contact\">contact</a> \
         <a href=\"/sim/public/search?q=baseline\">search</a></p>\
         </main>\
         </body>\
         </html>",
        page.title(),
        page.title(),
        page.summary(),
        render_nav(page),
        render_page_content(page),
        search_block
    )
}

fn render_nav(active_page: SimPublicPage) -> String {
    SimPublicPage::ALL
        .iter()
        .map(|candidate| {
            let link = format!(
                "<a href=\"{}\">{}</a>",
                candidate.path(),
                candidate.label()
            );
            if *candidate == active_page {
                format!("<strong>{}</strong>", link)
            } else {
                link
            }
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn render_page_content(page: SimPublicPage) -> &'static str {
    match page {
        SimPublicPage::Landing => {
            "<p>Welcome. Continue through docs, pricing, contact, and search pages.</p>"
        }
        SimPublicPage::Docs => "<p>Docs index with stable links for crawler realism.</p>",
        SimPublicPage::Pricing => {
            "<p>Pricing snapshot: basic, growth, and enterprise simulation tiers.</p>"
        }
        SimPublicPage::Contact => "<p>Contact endpoint with deterministic static content.</p>",
        SimPublicPage::Search => {
            "<p>Search endpoint for query-bearing requests and realistic cadence.</p>"
        }
    }
}

fn extract_query_param(uri: &str, key: &str) -> Option<String> {
    let (_, query) = uri.split_once('?')?;
    for pair in query.split('&') {
        let (candidate, value) = pair.split_once('=').unwrap_or((pair, ""));
        if candidate == key {
            let normalized = value.replace('+', " ");
            if normalized.trim().is_empty() {
                return None;
            }
            return Some(normalized);
        }
    }
    None
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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

    #[test]
    fn parse_page_matches_supported_paths() {
        assert_eq!(parse_page("/sim/public/landing"), Some(SimPublicPage::Landing));
        assert_eq!(parse_page("/sim/public/docs"), Some(SimPublicPage::Docs));
        assert_eq!(parse_page("/sim/public/pricing"), Some(SimPublicPage::Pricing));
        assert_eq!(parse_page("/sim/public/contact"), Some(SimPublicPage::Contact));
        assert_eq!(parse_page("/sim/public/search"), Some(SimPublicPage::Search));
        assert_eq!(parse_page("/sim/public/unknown"), None);
    }

    #[test]
    fn parse_page_tolerates_query_suffix() {
        assert_eq!(
            parse_page("/sim/public/search?q=seeded"),
            Some(SimPublicPage::Search)
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
        let req = request(Method::Get, "/health");
        assert!(maybe_handle_with_availability(&req, "/health", enabled_availability()).is_none());
    }

    #[test]
    fn maybe_handle_returns_not_found_when_disabled() {
        let req = request(Method::Get, "/sim/public/landing");
        let availability = SimPublicAvailability {
            runtime_environment: RuntimeEnvironment::RuntimeDev,
            artifact_available: false,
        };
        let resp = maybe_handle_with_availability(&req, "/sim/public/landing", availability)
            .expect("sim route should be handled");
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
        std::fs::create_dir_all(&artifact_root).expect("artifact root should be creatable");
        std::fs::write(&manifest_path, "{}\n").expect("manifest should be writable");
        std::env::set_var("SHUMA_LOCAL_STATE_DIR", &local_state_dir);

        let mut cfg = crate::config::default_seeded_config();
        cfg.adversary_sim_enabled = false;
        let availability = availability_from_runtime(&cfg);

        std::env::remove_var("SHUMA_LOCAL_STATE_DIR");
        let _ = std::fs::remove_file(&manifest_path);
        let _ = std::fs::remove_dir_all(&base);

        assert!(availability.artifact_available);
        assert!(availability.is_enabled());
    }

    #[test]
    fn maybe_handle_rejects_non_get_head_methods() {
        let req = request(Method::Post, "/sim/public/docs");
        let resp = maybe_handle_with_availability(&req, "/sim/public/docs", enabled_availability())
            .expect("sim route should be handled");
        assert_eq!(*resp.status(), 405u16);
    }

    #[test]
    fn maybe_handle_serves_crawl_graph_when_enabled() {
        let req = request(Method::Get, "/sim/public/search?q=robot+audit");
        let resp = maybe_handle_with_availability(&req, "/sim/public/search", enabled_availability())
            .expect("sim route should be handled");

        assert_eq!(*resp.status(), 200u16);
        let body = String::from_utf8(resp.into_body()).expect("sim page should be utf-8");
        assert!(body.contains("/sim/public/landing"));
        assert!(body.contains("/sim/public/docs"));
        assert!(body.contains("/sim/public/pricing"));
        assert!(body.contains("/sim/public/contact"));
        assert!(body.contains("/sim/public/search"));
        assert!(body.contains("robot audit"));
    }

    #[test]
    fn maybe_handle_returns_empty_body_for_head() {
        let req = request(Method::Head, "/sim/public/landing");
        let resp =
            maybe_handle_with_availability(&req, "/sim/public/landing", enabled_availability())
                .expect("sim route should be handled");
        assert_eq!(*resp.status(), 200u16);
        assert!(resp.body().is_empty());
    }
}
